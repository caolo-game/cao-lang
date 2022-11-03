#[cfg(test)]
mod tests;

use std::{alloc::Layout, mem::swap, num::Wrapping, ptr::NonNull, str::FromStr};

use crate::alloc::{Allocator, SysAllocator};

pub(crate) const MAX_LOAD: f32 = 0.7;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct HashValue(u32);

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

pub struct Entry<'a, K, V> {
    hash: HashValue,
    key: K,
    pl: EntryPayload<'a, K, V>,
}

enum EntryPayload<'a, K, V> {
    Occupied(&'a mut V),
    Vacant {
        hash: &'a mut HashValue,
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
        let hash_layout = Layout::array::<HashValue>(cap).unwrap();
        let keys_layout = Layout::array::<K>(cap).unwrap();
        let values_layout = Layout::array::<V>(cap).unwrap();

        let (result, keys_offset) = hash_layout.extend(keys_layout).unwrap();
        let (result, vals_offset) = result.extend(values_layout).unwrap();

        (result, [keys_offset, vals_offset])
    }

    pub fn clear(&mut self) {
        let handles = self.data.cast::<HashValue>().as_ptr();
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();

        unsafe {
            clear_arrays(handles, keys, values, self.capacity);
        }

        self.count = 0;
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<HashValue, MapError>
    where
        for<'a> HashValue: From<&'a K>,
        for<'a> &'a K: PartialEq,
    {
        let h = HashValue::from(&key);
        unsafe { self.insert_with_hint(h, key, value).map(|_| h) }
    }

    /// # Safety
    /// Caller must ensure that the hash is correct for the key
    pub unsafe fn insert_with_hint(
        &mut self,
        h: HashValue,
        key: K,
        value: V,
    ) -> Result<(), MapError>
    where
        for<'a> &'a K: PartialEq,
    {
        debug_assert!(h.0 != 0, "Bad handle, 0 values are reserved");

        // find the bucket
        let hashes = self.hashes();
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();

        let i = self.find_ind(h, &key);
        if hashes[i].0 != 0 {
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

    fn grow(&mut self) -> Result<(), MapError>
    where
        for<'a> &'a K: PartialEq,
    {
        let new_cap = (self.capacity.max(2) * 3) / 2;
        debug_assert!(new_cap > self.capacity);
        unsafe { self.adjust_capacity(new_cap) }
    }

    unsafe fn adjust_capacity(&mut self, capacity: usize) -> Result<(), MapError>
    where
        for<'a> &'a K: PartialEq,
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
            let hash = *data.as_ptr().cast::<HashValue>().add(i);
            if hash != HashValue(0) {
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

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        for<'a> &'a K: Into<HashValue>,
        for<'a> &'a K: PartialEq,
    {
        let hash = key.into();
        unsafe { self.remove_with_hint(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn remove_with_hint(&mut self, hash: HashValue, key: &K) -> Option<V>
    where
        for<'a> &'a K: PartialEq,
    {
        let i = self.find_ind(hash, key);
        if self.hashes()[i].0 != 0 {
            if std::mem::needs_drop::<K>() {
                std::ptr::drop_in_place(self.keys.as_ptr().add(i));
            }

            let result = std::ptr::read(self.values.as_ptr().add(i));
            self.hashes_mut()[i] = HashValue(0);
            return Some(result);
        }
        None
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        for<'a> &'a K: Into<HashValue>,
        for<'a> &'a K: PartialEq,
    {
        let hash = key.into();
        unsafe { self.get_with_hint(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn get_with_hint(&self, h: HashValue, k: &K) -> Option<&V>
    where
        for<'a> &'a K: PartialEq,
    {
        let i = self.find_ind(h, k);
        if self.hashes()[i] != HashValue(0) {
            Some(&*self.values.as_ptr().add(i))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        for<'a> &'a K: Into<HashValue>,
        for<'a> &'a K: PartialEq,
    {
        let hash = key.into();
        unsafe { self.get_with_hint_mut(hash, key) }
    }

    /// # Safety
    ///
    /// Hash must be produced from the key
    pub unsafe fn get_with_hint_mut(&mut self, h: HashValue, k: &K) -> Option<&mut V>
    where
        for<'a> &'a K: PartialEq,
    {
        let i = self.find_ind(h, k);
        if self.hashes()[i] != HashValue(0) {
            Some(&mut *self.values.as_ptr().add(i))
        } else {
            None
        }
    }

    fn find_ind(&self, needle: HashValue, k: &K) -> usize
    where
        for<'a> &'a K: PartialEq,
    {
        let len = self.capacity;

        // improve uniformity via fibonacci hashing
        // in wasm sizeof usize is 4, so multiply our already 32 bit hash
        let mut ind = (needle.0.wrapping_mul(2654435769) as usize) % len;
        let hashes = self.hashes();
        let keys = self.keys.as_ptr();
        loop {
            unsafe {
                debug_assert!(ind < len);
                let h = hashes[ind];
                if h.0 == 0 || (h == needle && (&*keys.add(ind)) == k) {
                    return ind;
                }
            }
            ind = (ind + 1) % len;
        }
    }

    pub fn hashes(&self) -> &[HashValue] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().cast(), self.capacity) }
    }

    pub fn hashes_mut(&mut self) -> &mut [HashValue] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr().cast(), self.capacity) }
    }

    /// Zero-out the hash buffer
    ///
    /// Call this function after a fresh alloc of the data buffer
    fn zero_hashes(&mut self) {
        self.hashes_mut().fill(HashValue(0));
    }

    /// This method eagerly allocated new buffers, if inserting via the entry
    /// would grow the buffer beyong its max load
    pub fn entry(&mut self, key: K) -> Result<Entry<K, V>, MapError>
    where
        for<'a> HashValue: From<&'a K>,
        for<'a> &'a K: PartialEq,
    {
        let hash = HashValue::from(&key);
        let i = self.find_ind(hash, &key);
        let pl;
        if self.hashes()[i].0 != 0 {
            pl = EntryPayload::Occupied(unsafe { &mut *self.values.as_ptr().add(i) });
        } else {
            // if it would need to grow on insert, then allocate the new buffer now
            if Self::needs_grow(self.count + 1, self.capacity) {
                self.grow()?;
            }
            unsafe {
                pl = EntryPayload::Vacant {
                    hash: &mut *self.data.cast::<HashValue>().as_ptr().add(i),
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
}

impl FromStr for HashValue {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(s.as_bytes()))
    }
}

impl HashValue {
    pub fn from_bytes(key: &[u8]) -> Self {
        let hash = hash_bytes(2166136261, key);
        debug_assert!(hash != 0);
        Self(hash as u32)
    }

    pub fn from_slice<'a, T>(keys: &'a [T]) -> Self
    where
        &'a T: Into<&'a [u8]>,
    {
        let mut hash = 2166136261;
        for key in keys {
            hash = hash_bytes(hash, key.into());
        }
        debug_assert!(hash != 0);
        Self(hash as u32)
    }

    pub fn from_bytes_iter<'a>(keys: impl Iterator<Item = &'a [u8]>) -> Self {
        let mut hash = 2166136261;
        for key in keys {
            hash = hash_bytes(hash, key);
        }
        debug_assert!(hash != 0);
        Self(hash as u32)
    }

    pub fn from_u32(key: u32) -> Self {
        const MASK: u64 = u32::MAX as u64;
        let key = hash_u64(key as u64, MASK);
        Self(key)
    }

    pub fn from_u64(key: u64) -> Self {
        const MASK: u64 = u64::MAX;
        let key = hash_u64(key, MASK);
        Self(key)
    }

    pub fn from_i64(key: i64) -> Self {
        const MASK: u64 = u64::MAX;
        let key = hash_u64(key as u64, MASK);
        Self(key)
    }
}

fn hash_bytes(mut hash: u64, key: &[u8]) -> u64 {
    const MASK: u64 = u32::MAX as u64;
    for byte in key {
        hash ^= *byte as u64;
        hash &= MASK;
        hash *= 16777619;
    }
    hash & MASK
}

// FNV-1a
#[inline]
fn hash_u64(key: u64, mask: u64) -> u32 {
    let key = key + mask * (key == 0) as u64; // to ensure non-zero keys

    let mut key = Wrapping(key);
    let mask = Wrapping(mask);
    key = (((key >> 16) ^ key) * Wrapping(0x45d0f3b)) & mask;
    key = (((key >> 16) ^ key) * Wrapping(0x45d0f3b)) & mask;
    key = ((key >> 16) ^ key) & mask;
    debug_assert!(key.0 != 0);
    ((key >> 32) ^ key).0 as u32
}

impl From<i64> for HashValue {
    fn from(key: i64) -> Self {
        Self::from_i64(key)
    }
}

impl From<u32> for HashValue {
    fn from(key: u32) -> Self {
        Self::from_u32(key)
    }
}

impl From<i32> for HashValue {
    fn from(key: i32) -> Self {
        Self::from_i64(key as i64)
    }
}

impl From<&'_ i32> for HashValue {
    fn from(key: &'_ i32) -> Self {
        Self::from_i64(*key as i64)
    }
}

impl<'a> From<&'a str> for HashValue {
    fn from(key: &'a str) -> Self {
        <Self as FromStr>::from_str(key).unwrap()
    }
}

/// # Safety
///
/// Must be called with valid arrays in a CaoHashMap
unsafe fn clear_arrays<K, V>(handles: *mut HashValue, keys: *mut K, values: *mut V, count: usize) {
    for i in 0..count {
        if (*handles.add(i)).0 != 0 {
            *handles.add(i) = HashValue(0);
            if std::mem::needs_drop::<K>() {
                std::ptr::drop_in_place(keys.add(i));
            }
            if std::mem::needs_drop::<V>() {
                std::ptr::drop_in_place(values.add(i));
            }
        }
    }
}
