#[cfg(feature = "serde")]
mod serde_impl;

#[cfg(test)]
mod tests;

use std::{
    alloc::Layout,
    borrow::Borrow,
    hash::{Hash, Hasher},
    mem::swap,
    ptr::NonNull,
};

use crate::alloc::{Allocator, SysAllocator};

pub(crate) const MAX_LOAD: f32 = 0.7;

type ArrayTriplet<K, V> = (NonNull<u8>, NonNull<K>, NonNull<V>);

/// Hash map implemented for Cao-Lang
pub struct CaoHashMap<K, V, A: Allocator = SysAllocator> {
    /// beginning of the data, and the hash buffer
    /// layout:
    /// [hash hash hash][key key key][value value value]
    data: NonNull<u8>,
    /// begin of the keys array
    keys: NonNull<K>,
    /// begin of the values array
    values: NonNull<V>,

    count: usize,
    capacity: usize,

    alloc: A,
}

unsafe impl<K, V, A: Allocator + Send> Send for CaoHashMap<K, V, A> {}
unsafe impl<K, V, A: Allocator + Send> Sync for CaoHashMap<K, V, A> {}

impl<K, V, A> std::fmt::Debug for CaoHashMap<K, V, A>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
    A: Allocator,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = f.debug_map();
        for (k, v) in self.iter() {
            state.entry(k, v);
        }
        state.finish()
    }
}

impl<K, V, A> Clone for CaoHashMap<K, V, A>
where
    K: Clone + Eq + Hash,
    V: Clone,
    A: Allocator + Clone,
{
    fn clone(&self) -> Self {
        let mut result = CaoHashMap::with_capacity_in(self.capacity, self.alloc.clone()).unwrap();

        // TODO: could use insert with hint
        // or better yet, memcpy hashes, then clone the occupied entries
        for (k, v) in self.iter() {
            result.insert(k.clone(), v.clone()).unwrap();
        }
        result
    }
}

impl<K, V, A: Allocator + Default> Default for CaoHashMap<K, V, A> {
    fn default() -> Self {
        CaoHashMap::with_capacity_in(0, A::default()).unwrap()
    }
}

pub struct Entry<'a, K, V> {
    hash: u64,
    key: K,
    pl: EntryPayload<'a, K, V>,
}

enum EntryPayload<'a, K, V> {
    Occupied(&'a mut V),
    Vacant {
        hash: &'a mut u64,
        key: *mut K,
        value: *mut V,
        count: &'a mut usize,
    },
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn or_insert_with<F: FnOnce() -> V>(self, fun: F) -> &'a mut V {
        match self.pl {
            EntryPayload::Occupied(res) => res,
            EntryPayload::Vacant {
                hash,
                key,
                value,
                count,
            } => {
                *hash = self.hash;
                unsafe {
                    std::ptr::write(key, self.key);
                    std::ptr::write(value, fun());
                    *count += 1;
                    &mut *value
                }
            }
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MapError {
    #[error("Failed to allocate memory {0}")]
    AllocError(crate::alloc::AllocError),
}

impl<K, V, A: Allocator> Drop for CaoHashMap<K, V, A> {
    fn drop(&mut self) {
        self.clear();
        let (layout, _) = Self::layout(self.capacity);
        unsafe {
            self.alloc.dealloc(self.data, layout);
        }
    }
}

impl<K, V, A: Allocator> CaoHashMap<K, V, A> {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, MapError> {
        let capacity = capacity.max(1);
        let (data, keys, values) = unsafe { Self::alloc_storage(&alloc, capacity)? };
        let mut result = Self {
            data,
            keys,
            values,
            count: 0,
            capacity,
            alloc,
        };
        result.zero_hashes();
        Ok(result)
    }

    /// # Safety
    /// Caller must ensure that the hashes are zeroed
    unsafe fn alloc_storage(alloc: &A, cap: usize) -> Result<ArrayTriplet<K, V>, MapError> {
        let (layout, [ko, vo]) = Self::layout(cap);
        let data = alloc.alloc(layout).map_err(MapError::AllocError)?;
        let keys = data.as_ptr().add(ko).cast();
        let values = data.as_ptr().add(vo).cast();
        Ok((
            data,
            NonNull::new_unchecked(keys),
            NonNull::new_unchecked(values),
        ))
    }

    fn layout(cap: usize) -> (Layout, [usize; 2]) {
        let hash_layout = Layout::array::<u64>(cap).unwrap();
        let keys_layout = Layout::array::<K>(cap).unwrap();
        let values_layout = Layout::array::<V>(cap).unwrap();

        let (result, keys_offset) = hash_layout.extend(keys_layout).unwrap();
        let (result, vals_offset) = result.extend(values_layout).unwrap();

        (result, [keys_offset, vals_offset])
    }

    pub fn clear(&mut self) {
        let handles = self.data.cast::<u64>().as_ptr();
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();

        unsafe {
            clear_arrays(handles, keys, values, self.capacity);
        }

        self.count = 0;
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<u64, MapError>
    where
        K: Eq + Hash,
    {
        let h = hash(&key);
        unsafe { self.insert_with_hint(h, key, value).map(|_| h) }
    }

    /// # Safety
    /// Caller must ensure that the hash is correct for the key
    pub unsafe fn insert_with_hint(&mut self, h: u64, key: K, value: V) -> Result<(), MapError>
    where
        K: Eq,
    {
        debug_assert!(h != 0, "Bad handle, 0 values are reserved");

        // find the bucket
        let hashes = self.hashes();
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();

        let i = self.find_ind(h, &key);
        if hashes[i] != 0 {
            debug_assert_eq!(hashes[i], h);
            // delete the old entry
            if std::mem::needs_drop::<K>() {
                std::ptr::drop_in_place(keys.add(i));
            }
            if std::mem::needs_drop::<V>() {
                std::ptr::drop_in_place(values.add(i));
            }
        } else {
            self.hashes_mut()[i] = h;
            self.count += 1;
        }
        std::ptr::write(keys.add(i), key);
        std::ptr::write(values.add(i), value);
        // delaying grow so that no grow is triggered if the key overrides an existing value
        if Self::needs_grow(self.count, self.capacity) {
            self.grow()?;
        }
        Ok(())
    }

    fn needs_grow(count: usize, capacity: usize) -> bool {
        count as f32 > capacity as f32 * MAX_LOAD
    }

    pub fn reserve(&mut self, additional_cap: usize) -> Result<(), MapError>
    where
        K: Eq,
    {
        unsafe { self.adjust_capacity(self.capacity + additional_cap) }
    }

    fn grow(&mut self) -> Result<(), MapError>
    where
        K: Eq,
    {
        let new_cap = (self.capacity.max(2) * 3) / 2;
        debug_assert!(new_cap > self.capacity);
        unsafe { self.adjust_capacity(new_cap) }
    }

    unsafe fn adjust_capacity(&mut self, capacity: usize) -> Result<(), MapError>
    where
        K: Eq,
    {
        let (mut data, mut keys, mut values) = Self::alloc_storage(&self.alloc, capacity)?;
        swap(&mut self.data, &mut data);
        swap(&mut self.keys, &mut keys);
        swap(&mut self.values, &mut values);
        let capacity = std::mem::replace(&mut self.capacity, capacity);
        self.zero_hashes();
        let count = std::mem::replace(&mut self.count, 0); // insert will increment count
                                                           // copy over the existing values
        for i in 0..capacity {
            let hash = *data.as_ptr().cast::<u64>().add(i);
            if hash != 0 {
                let key = std::ptr::read(keys.as_ptr().add(i));
                let val = std::ptr::read(values.as_ptr().add(i));
                self.insert_with_hint(hash, key, val)?;
            }
        }

        assert_eq!(
            count, self.count,
            "Internal error: moving the values after realloc resulted in inconsistent count"
        );

        // free up the old storage
        let (layout, _) = Self::layout(capacity);
        self.alloc.dealloc(data, layout);

        Ok(())
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = hash(key);
        unsafe { self.remove_with_hint(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn remove_with_hint<Q: ?Sized>(&mut self, hash: u64, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        let i = self.find_ind(hash, key);
        if self.hashes()[i] != 0 {
            if std::mem::needs_drop::<K>() {
                std::ptr::drop_in_place(self.keys.as_ptr().add(i));
            }

            let result = std::ptr::read(self.values.as_ptr().add(i));
            self.hashes_mut()[i] = 0;

            // if the consecutive buckets are not empty, move them back, so lookups dont fail
            // and they aren't in their optimal position
            //
            let mut i = i; // track the last empty slot
            let mut j = (i + 1) % self.capacity();
            while self.hashes()[j] != 0 {
                // if the jth item is not in its optimal bucket, then move it back to the empty
                // slot
                if (self.hashes()[j] % self.capacity() as u64) != j as u64 {
                    self.hashes_mut()[i] = self.hashes()[j];
                    std::ptr::swap(self.keys.as_ptr().add(i), self.keys.as_ptr().add(j));
                    std::ptr::swap(self.values.as_ptr().add(i), self.values.as_ptr().add(j));
                    i = j;
                }
                j = (j + 1) % self.capacity();
            }

            return Some(result);
        }
        None
    }

    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = hash(key);
        unsafe { self.contains_with_hint(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn contains_with_hint<Q: ?Sized>(&self, h: u64, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let i = self.find_ind(h, k);
        self.hashes()[i] != 0
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = hash(key);
        unsafe { self.get_with_hint(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn get_with_hint<Q: ?Sized>(&self, h: u64, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        let i = self.find_ind(h, k);
        if self.hashes()[i] != 0 {
            Some(&*self.values.as_ptr().add(i))
        } else {
            None
        }
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = hash(key);
        unsafe { self.get_with_hint_mut(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn get_with_hint_mut<Q: ?Sized>(&mut self, h: u64, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let i = self.find_ind(h, k);
        if self.hashes()[i] != 0 {
            Some(&mut *self.values.as_ptr().add(i))
        } else {
            None
        }
    }

    fn find_ind<Q: ?Sized>(&self, needle: u64, k: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        let len = self.capacity;

        // improve uniformity via fibonacci hashing
        // in wasm sizeof usize is 4, so multiply our already 32 bit hash
        let mut ind = (needle.wrapping_mul(2654435769) as usize) % len;
        let hashes = self.hashes();
        let keys = self.keys.as_ptr();
        loop {
            unsafe {
                debug_assert!(ind < len);
                let h = hashes[ind];
                if h == 0 || (h == needle && (*keys.add(ind)).borrow() == k) {
                    return ind;
                }
            }
            ind = (ind + 1) % len;
        }
    }

    fn hashes(&self) -> &[u64] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().cast(), self.capacity) }
    }

    fn hashes_mut(&mut self) -> &mut [u64] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr().cast(), self.capacity) }
    }

    /// Zero-out the hash buffer
    ///
    /// Call this function after a fresh alloc of the data buffer
    fn zero_hashes(&mut self) {
        self.hashes_mut().fill(0u64);
    }

    /// This method eagerly allocated new buffers, if inserting via the entry
    /// would grow the buffer beyong its max load
    pub fn entry(&mut self, key: K) -> Result<Entry<K, V>, MapError>
    where
        K: Eq + Hash,
    {
        let hash = hash(&key);
        let i = self.find_ind(hash, &key);
        let pl;
        if self.hashes()[i] != 0 {
            pl = EntryPayload::Occupied(unsafe { &mut *self.values.as_ptr().add(i) });
        } else {
            // if it would need to grow on insert, then allocate the new buffer now
            if Self::needs_grow(self.count + 1, self.capacity) {
                self.grow()?;
            }
            unsafe {
                pl = EntryPayload::Vacant {
                    hash: &mut *self.data.cast::<u64>().as_ptr().add(i),
                    key: self.keys.as_ptr().add(i),
                    value: self.values.as_ptr().add(i),
                    count: &mut self.count,
                }
            }
        }
        Ok(Entry { hash, key, pl })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        (0..self.capacity)
            .filter(|i| self.hashes()[*i] != 0)
            .map(|i| unsafe { (&*self.keys.as_ptr().add(i), &*self.values.as_ptr().add(i)) })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        (0..self.capacity)
            .filter(|i| self.hashes()[*i] != 0)
            .map(|i| unsafe {
                (
                    &*self.keys.as_ptr().add(i),
                    &mut *self.values.as_ptr().add(i),
                )
            })
    }
}

struct CaoHasher(u64);
impl Default for CaoHasher {
    fn default() -> Self {
        Self(2166136261)
    }
}

impl Hasher for CaoHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const MASK: u64 = u32::MAX as u64;
        let mut hash = self.0;
        for byte in bytes {
            hash ^= *byte as u64;
            hash &= MASK;
            hash *= 16777619;
        }
        self.0 = hash & MASK;
    }
}

fn hash<T: ?Sized + Hash>(t: &T) -> u64 {
    let mut hasher = CaoHasher::default();
    t.hash(&mut hasher);
    let result = hasher.finish();
    debug_assert_ne!(result, 0, "0 hash is reserved");
    result
}

/// # Safety
///
/// Must be called with valid arrays in a CaoHashMap
unsafe fn clear_arrays<K, V>(handles: *mut u64, keys: *mut K, values: *mut V, count: usize) {
    for i in 0..count {
        if (*handles.add(i)) != 0 {
            *handles.add(i) = 0;
            if std::mem::needs_drop::<K>() {
                std::ptr::drop_in_place(keys.add(i));
            }
            if std::mem::needs_drop::<V>() {
                std::ptr::drop_in_place(values.add(i));
            }
        }
    }
}
