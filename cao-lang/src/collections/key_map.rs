//! Hash table with pre-calculated hashes.
//!
//! This hashmap does not care about the actual value of the key and assumes that the hash was
//! calculated ahead of time.
//!
//! ## Safety
//!
//! Since only the hashes are compared hash collisions will introduce bugs which are not addressed
//! at this moment. Use wisely
//!

#[cfg(feature = "serde")]
mod serde_impl;

#[cfg(test)]
mod tests;

use crate::alloc::{Allocator, SysAllocator};

#[cfg(feature = "serde")]
pub use self::serde_impl::*;

use std::{
    alloc::Layout,
    intrinsics::transmute,
    mem::{align_of, size_of, swap, MaybeUninit},
    ops::{Index, IndexMut},
    ptr::NonNull,
    str::FromStr,
};

pub(crate) const MAX_LOAD: f32 = 0.69;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Handle(u32);

pub struct KeyMap<T, A = SysAllocator>
where
    A: Allocator,
{
    keys: NonNull<Handle>,
    values: NonNull<T>,
    count: usize,
    capacity: usize,

    alloc: A,
}

impl<T, A: Allocator> std::fmt::Debug for KeyMap<T, A>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MapError {
    #[error("Failed to allocate memory {0}")]
    AllocError(crate::alloc::AllocError),
}

pub struct Entry<'a, T> {
    key: Handle,
    pl: EntryPayload<'a, T>,
}

enum EntryPayload<'a, T> {
    Occupied(&'a mut T),
    Vacant {
        key: &'a mut Handle,
        value: &'a mut MaybeUninit<T>,
        count: &'a mut usize,
    },
}

impl<'a, T: 'a> Entry<'a, T> {
    pub fn or_insert_with<F: FnOnce() -> T>(self, fun: F) -> &'a mut T {
        match self.pl {
            EntryPayload::Occupied(res) => res,
            EntryPayload::Vacant { count, key, value } => {
                *key = self.key;
                *value = MaybeUninit::new(fun());
                *count += 1;
                unsafe { &mut *value.as_mut_ptr() }
            }
        }
    }
}

impl FromStr for Handle {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(s.as_bytes()))
    }
}

impl Handle {
    pub fn from_bytes(key: &[u8]) -> Self {
        const MASK: u64 = u32::MAX as u64;
        // FNV-1a
        let mut hash = 2166136261u64;
        for byte in key {
            hash ^= *byte as u64;
            hash &= MASK;
            hash *= 16777619;
        }
        let hash = hash & MASK;
        debug_assert!(hash != 0);
        Self(hash as u32)
    }

    pub fn from_u32(key: u32) -> Self {
        const MASK: u64 = u32::MAX as u64;

        let mut key = key as u64 + MASK * (key == 0) as u64; // to ensure non-zero keys
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = ((key >> 16) ^ key) & MASK;
        debug_assert!(key != 0);
        Self(key as u32)
    }

    pub fn from_i64(key: i64) -> Self {
        const MASK: u64 = u32::MAX as u64;

        let mut key = key as u64 + MASK * (key == 0) as u64; // to ensure non-zero keys
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = ((key >> 16) ^ key) & MASK;
        debug_assert!(key != 0);
        Self(key as u32)
    }
}

impl From<i64> for Handle {
    fn from(key: i64) -> Self {
        Self::from_i64(key)
    }
}

impl From<u32> for Handle {
    fn from(key: u32) -> Self {
        Self::from_u32(key)
    }
}

impl<'a> From<&'a str> for Handle {
    fn from(key: &'a str) -> Self {
        <Self as FromStr>::from_str(key).unwrap()
    }
}

impl<T, A> Default for KeyMap<T, A>
where
    A: Allocator + Default,
{
    fn default() -> Self {
        Self::with_capacity(16, A::default()).expect("Failed to init map")
    }
}

impl<T, A> Drop for KeyMap<T, A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.alloc.dealloc(
                transmute(self.keys),
                Layout::from_size_align(self.capacity * size_of::<Handle>(), align_of::<Handle>())
                    .expect("old Key layout"),
            );
            self.alloc.dealloc(
                transmute(self.values),
                Layout::from_size_align(self.capacity * size_of::<T>(), align_of::<T>())
                    .expect("old T layout"),
            );
        }
    }
}

impl<T, A> KeyMap<T, A>
where
    A: Allocator,
{
    pub fn with_capacity(capacity: usize, allocator: A) -> Result<Self, MapError> {
        unsafe {
            let (keys, values) = Self::alloc_storage(&allocator, capacity)?;
            let res = Self {
                keys,
                values,
                alloc: allocator,
                count: 0,
                capacity,
            };
            Ok(res)
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            for (i, k) in (0..self.capacity)
                .map(|i| (i, &mut *self.keys.as_ptr().add(i)))
                .filter(|(_, Handle(x))| *x != 0)
            {
                if std::mem::needs_drop::<T>() {
                    std::ptr::drop_in_place(self.values.as_ptr().add(i));
                }
                k.0 = 0;
            }
            self.count = 0;
        }
    }

    /// Reserve enough space to hold `capacity` additional items
    #[inline]
    pub fn reserve(&mut self, capacity: usize) -> Result<(), MapError> {
        let new_cap = capacity + self.count;
        if new_cap > self.capacity {
            unsafe {
                self.adjust_size((new_cap as f32 * (1.0 + MAX_LOAD)) as usize)?;
            }
        }
        Ok(())
    }

    pub fn entry(&mut self, key: Handle) -> Entry<T> {
        let ind = self.find_ind(key);

        let pl = unsafe {
            if *self.keys.as_ptr().add(ind) != key {
                EntryPayload::Vacant {
                    key: &mut *self.keys.as_ptr().add(ind),
                    value: &mut *(self.values.as_ptr().add(ind) as *mut MaybeUninit<T>),
                    count: &mut self.count,
                }
            } else {
                EntryPayload::Occupied(&mut *self.values.as_ptr().add(ind))
            }
        };
        Entry { key, pl }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn contains(&self, key: Handle) -> bool {
        let ind = self.find_ind(key);
        unsafe { (*self.keys.as_ptr().add(ind)).0 != 0 }
    }

    pub fn get(&self, key: Handle) -> Option<&T> {
        let ind = self.find_ind(key);
        unsafe {
            if (*self.keys.as_ptr().add(ind)).0 != 0 {
                let r = self.values.as_ptr().add(ind);
                Some(&*r)
            } else {
                None
            }
        }
    }

    pub fn get_mut(&mut self, key: Handle) -> Option<&mut T> {
        let ind = self.find_ind(key);
        unsafe {
            if (*self.keys.as_ptr().add(ind)).0 != 0 {
                let r = self.values.as_ptr().add(ind);
                Some(&mut *r)
            } else {
                None
            }
        }
    }

    fn find_ind(&self, needle: Handle) -> usize {
        let len = self.capacity;

        debug_assert!(len >= 2);
        debug_assert!(
            (len & (len - 1)) == 0,
            "Expected self.capacity to be a power of two"
        );
        let len_mask = len - 1;
        let mut ind = needle.0 as usize & len_mask;
        let ptr = self.keys.as_ptr();
        loop {
            debug_assert!(ind < len);
            let k = unsafe { *ptr.add(ind) };
            if k == needle || k.0 == 0 {
                return ind;
            }
            ind = (ind + 1) & len_mask;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Handle, &'_ T)> + '_ {
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();
        (0..self.capacity).filter_map(move |i| unsafe {
            let k = *keys.add(i);
            (k.0 != 0).then(|| (k, &*values.add(i)))
        })
    }

    unsafe fn alloc_storage(
        alloc: &A,
        capacity: usize,
    ) -> Result<(NonNull<Handle>, NonNull<T>), MapError> {
        let keyslayout =
            Layout::from_size_align(capacity * size_of::<Handle>(), align_of::<Handle>())
                .expect("Failed to produce keys layout");
        let keys = alloc.alloc(keyslayout).map_err(MapError::AllocError)?;

        let values = match alloc
            .alloc(
                Layout::from_size_align(capacity * size_of::<T>(), align_of::<T>())
                    .expect("Failed to produce T layout"),
            )
            .map_err(MapError::AllocError)
        {
            Ok(ptr) => ptr,
            Err(err) => {
                alloc.dealloc(keys, keyslayout);
                return Err(err);
            }
        };
        // zero the keys
        let keys: NonNull<Handle> = transmute(keys);
        for i in 0..capacity {
            std::ptr::write(keys.as_ptr().add(i), Handle(0));
        }
        Ok((keys, transmute(values)))
    }

    unsafe fn adjust_size(&mut self, capacity: usize) -> Result<(), MapError> {
        let capacity = pad_pot(capacity).max(2); // allocate at least two items
        let (mut keys, mut values) = Self::alloc_storage(&self.alloc, capacity)?;

        swap(&mut self.keys, &mut keys);
        swap(&mut self.values, &mut values);

        let old_cap = self.capacity;
        // insert the old values
        self.count = 0;
        self.capacity = capacity;
        for (i, key) in (0..old_cap)
            .map(|i| (i, *keys.as_ptr().add(i)))
            .filter(|(_, Handle(x))| *x != 0)
        {
            let value: T = std::ptr::read(values.as_ptr().add(i));
            self._insert(key, value);
        }

        // dealloc old buffers
        self.alloc.dealloc(
            transmute(keys),
            Layout::from_size_align(old_cap * size_of::<Handle>(), align_of::<Handle>())
                .expect("old Key layout"),
        );
        self.alloc.dealloc(
            transmute(values),
            Layout::from_size_align(old_cap * size_of::<T>(), align_of::<T>())
                .expect("old T layout"),
        );

        Ok(())
    }

    #[inline]
    fn grow(&mut self) -> Result<(), MapError> {
        let new_cap = self.capacity.max(2) * 3 / 2;
        debug_assert!(new_cap > self.capacity);
        unsafe { self.adjust_size(new_cap) }
    }

    /// Returns mutable reference to the just inserted value
    pub fn insert(&mut self, key: Handle, value: T) -> Result<&mut T, MapError> {
        debug_assert_ne!(key.0, 0, "0 keys mean unintialized entries");
        if (self.count + 1) as f32 > self.capacity as f32 * MAX_LOAD {
            self.grow()?;
        }
        Ok(self._insert(key, value))
    }

    #[inline]
    fn _insert(&mut self, key: Handle, value: T) -> &mut T {
        let ind = self.find_ind(key);

        debug_assert!(ind < self.capacity);

        let is_new_key = unsafe { (*self.keys.as_ptr().add(ind)).0 == 0 };
        self.count += is_new_key as usize;

        if std::mem::needs_drop::<T>() && !is_new_key {
            unsafe {
                std::ptr::drop_in_place(self.values.as_ptr().add(ind));
            }
        }

        unsafe {
            std::ptr::write(self.keys.as_ptr().add(ind), key);
            std::ptr::write(self.values.as_ptr().add(ind), value);
            &mut *self.values.as_ptr().add(ind)
        }
    }

    /// Removes the element and returns `Some(value)` if it was present, else None
    pub fn remove(&mut self, key: Handle) -> Option<T> {
        let ind = self.find_ind(key);
        unsafe {
            let kptr = self.keys.as_ptr().add(ind);
            if (*kptr).0 != 0 {
                self.count -= 1;
                *kptr = Handle(0);
                Some(std::ptr::read(self.values.as_ptr().add(ind)))
            } else {
                None
            }
        }
    }
}

impl<T> Index<Handle> for KeyMap<T> {
    type Output = T;

    fn index(&self, key: Handle) -> &Self::Output {
        let ind = self.find_ind(key);
        unsafe {
            assert!((*self.keys.as_ptr().add(ind)).0 != 0);
        }
        unsafe {
            let r = self.values.as_ptr().add(ind);
            &*r
        }
    }
}
impl<T> IndexMut<Handle> for KeyMap<T> {
    fn index_mut(&mut self, key: Handle) -> &mut Self::Output {
        let ind = self.find_ind(key);
        unsafe {
            assert!((*self.keys.as_ptr().add(ind)).0 != 0);
        }
        unsafe {
            let r = self.values.as_ptr().add(ind);
            &mut *r
        }
    }
}

impl<T> Index<u32> for KeyMap<T> {
    type Output = T;

    fn index(&self, key: u32) -> &Self::Output {
        let key = Handle::from_u32(key);
        &self[key]
    }
}

impl<T> IndexMut<u32> for KeyMap<T> {
    fn index_mut(&mut self, key: u32) -> &mut Self::Output {
        let key = Handle::from_u32(key);
        &mut self[key]
    }
}

impl<T> Index<&[u8]> for KeyMap<T> {
    type Output = T;

    fn index(&self, key: &[u8]) -> &Self::Output {
        let key = Handle::from_bytes(key);
        &self[key]
    }
}
impl<T> IndexMut<&[u8]> for KeyMap<T> {
    fn index_mut(&mut self, key: &[u8]) -> &mut Self::Output {
        let key = Handle::from_bytes(key);
        &mut self[key]
    }
}

unsafe impl<T, A> Send for KeyMap<T, A> where A: Allocator + Send {}
unsafe impl<T, A> Sync for KeyMap<T, A> where A: Allocator + Sync {}

#[inline]
fn pad_pot(cap: usize) -> usize {
    let mut n = cap - 1; // to handle the case when cap is already POT
    while (n & (n - 1)) != 0 {
        n = n & (n - 1); // unset the rightmost bit
    }

    // return the next POT
    n << 1
}
