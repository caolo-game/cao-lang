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
mod serde;

#[cfg(test)]
mod tests;

#[cfg(feature = "serde")]
pub use self::serde::*;

use std::{
    mem::{replace, swap, MaybeUninit},
    str::FromStr,
};

pub const MAX_LOAD: f32 = 0.69;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Key(u32);
impl crate::AutoByteEncodeProperties for Key {}

#[derive(Debug)]
pub struct PreHashMap<T> {
    keys: Box<[Key]>,
    values: Box<[MaybeUninit<T>]>,

    count: usize,
    capacity: usize,
}

pub struct Entry<'a, T> {
    key: Key,
    pl: EntryPayload<'a, T>,
}

enum EntryPayload<'a, T> {
    Occupied(&'a mut T),
    Vacant {
        key: &'a mut Key,
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

impl FromStr for Key {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(s.as_bytes()))
    }
}

impl Key {
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

        let mut key = key.max(1) as u64; // ensure non-zero key
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = (((key >> 16) ^ key) * 0x45d0f3b) & MASK;
        key = ((key >> 16) ^ key) & MASK;
        debug_assert!(key != 0);
        Self(key as u32)
    }
}

impl From<u32> for Key {
    fn from(key: u32) -> Self {
        Self::from_u32(key)
    }
}

impl<'a> From<&'a str> for Key {
    fn from(key: &'a str) -> Self {
        <Self as FromStr>::from_str(key).unwrap()
    }
}

impl<T> Default for PreHashMap<T> {
    fn default() -> Self {
        Self::with_capacity(16)
    }
}

impl<T> Clone for PreHashMap<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut res = Self::with_capacity(self.capacity);
        res.count = self.count;
        for (i, key) in self.keys.iter().enumerate().filter(|(_, Key(k))| *k != 0) {
            res.keys[i] = *key;
            res.values[i] = MaybeUninit::new(unsafe { &*self.values[i].as_ptr() }.clone());
        }

        res
    }
}

impl<T> Drop for PreHashMap<T> {
    fn drop(&mut self) {
        for (i, _) in self.keys.iter().enumerate().filter(|(_, Key(x))| *x != 0) {
            let value = replace(&mut self.values[i], MaybeUninit::uninit());
            unsafe {
                let _value = value.assume_init();
            }
        }
    }
}

impl<T> PreHashMap<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut res = Self {
            keys: Box::new([]),
            values: Box::new([]),
            count: 0,
            capacity,
        };
        res.adjust_size(capacity);
        res
    }

    /// Reserve enough space to hold `capacity` additional items
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        let new_cap = capacity + self.count;
        if new_cap > self.capacity {
            self.adjust_size((new_cap as f32 * (1.0 + MAX_LOAD)) as usize);
        }
    }

    pub fn entry(&mut self, key: Key) -> Entry<T> {
        let ind = self.find_ind(key);

        let pl = if self.keys[ind] != key {
            EntryPayload::Vacant {
                key: &mut self.keys[ind],
                value: &mut self.values[ind],
                count: &mut self.count,
            }
        } else {
            EntryPayload::Occupied(unsafe { &mut *self.values[ind].as_mut_ptr() })
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
    pub fn contains(&self, key: Key) -> bool {
        let ind = self.find_ind(key);
        self.keys[ind].0 != 0
    }

    pub fn get(&self, key: Key) -> Option<&T> {
        let ind = self.find_ind(key);
        if self.keys[ind].0 != 0 {
            unsafe {
                let r = self.values[ind].as_ptr();
                Some(&*r)
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let ind = self.find_ind(key);
        if self.keys[ind].0 != 0 {
            unsafe {
                let r = self.values[ind].as_mut_ptr();
                Some(&mut *r)
            }
        } else {
            None
        }
    }

    fn find_ind(&self, key: Key) -> usize {
        let len = self.keys.len();
        let mut ind = key.0 as usize % len;
        loop {
            if self.keys[ind] == key || self.keys[ind].0 == 0 {
                return ind;
            }
            ind = (ind + 1) % len;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Key, &'_ T)> + '_ {
        self.keys
            .iter()
            .enumerate()
            .filter(|(_, Key(k))| *k != 0)
            .map(move |(i, k)| (*k, unsafe { &*self.values[i].as_ptr() }))
    }

    fn adjust_size(&mut self, capacity: usize) {
        let keys = vec![Key(0); capacity];

        let mut values = Vec::with_capacity(capacity);
        values.resize_with(capacity, MaybeUninit::uninit);

        let mut keys = keys.into_boxed_slice();
        let mut values = values.into_boxed_slice();

        swap(&mut self.keys, &mut keys);
        swap(&mut self.values, &mut values);

        self.count = 0;
        self.capacity = capacity;
        for (i, key) in keys.iter().enumerate().filter(|(_, Key(x))| *x != 0) {
            let value = replace(&mut values[i], MaybeUninit::uninit());
            unsafe {
                self._insert(*key, value.assume_init());
            }
        }
    }

    #[inline]
    fn grow(&mut self) {
        let new_cap = self.capacity.max(2) * 3 / 2;
        debug_assert!(new_cap > self.capacity);
        self.adjust_size(new_cap);
    }

    /// Returns mutable reference to the just inserted value
    pub fn insert(&mut self, key: Key, value: T) -> &mut T {
        debug_assert_ne!(key.0, 0, "0 keys mean unintialized entries");
        if (self.count + 1) as f32 > self.capacity as f32 * MAX_LOAD {
            self.grow();
        }
        self._insert(key, value)
    }

    #[inline]
    fn _insert(&mut self, key: Key, value: T) -> &mut T {
        let ind = self.find_ind(key);
        let is_new_key = self.keys[ind].0 == 0;
        self.count += is_new_key as usize;

        if std::mem::needs_drop::<T>() && !is_new_key {
            unsafe {
                std::ptr::drop_in_place(self.values[ind].as_mut_ptr());
            }
        }

        self.keys[ind] = key;
        self.values[ind] = MaybeUninit::new(value);
        unsafe { &mut *self.values[ind].as_mut_ptr() }
    }

    /// Removes the element and returns `Some(value)` if it was present, else None
    pub fn remove(&mut self, key: Key) -> Option<T> {
        let ind = self.find_ind(key);
        if self.keys[ind].0 != 0 {
            self.count -= 1;
            self.keys[ind] = Key(0);
            let val = replace(&mut self.values[ind], MaybeUninit::uninit());
            unsafe { Some(val.assume_init()) }
        } else {
            None
        }
    }
}
