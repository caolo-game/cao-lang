pub mod pre_hash_map {
    //! Hash table with pre-calculated hashes.
    //!
    use std::mem::{replace, swap, MaybeUninit};

    #[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Key(u32);

    pub const MAX_LOAD: f32 = 0.75;

    impl Key {
        pub fn from_str(key: &str) -> Self {
            // FNV-1a
            let mut hash = 2166136261;
            for byte in key.as_bytes() {
                hash ^= *byte as u32;
                hash *= 16777619;
            }
            debug_assert!(hash != 0);
            Self(hash)
        }

        pub fn from_un32(mut key: u32) -> Self {
            key = ((key >> 16) ^ key) * 0x45d0f3b;
            key = ((key >> 16) ^ key) * 0x45d0f3b;
            key = (key >> 16) ^ key;
            debug_assert!(key != 0);
            Self(key)
        }
    }

    impl From<u32> for Key {
        fn from(key: u32) -> Self {
            Self::from_un32(key)
        }
    }

    impl<'a> From<&'a str> for Key {
        fn from(key: &'a str) -> Self {
            Self::from_str(key)
        }
    }

    #[derive(Debug)]
    pub struct PreHashMap<T> {
        keys: Box<[Key]>,
        values: Box<[MaybeUninit<T>]>,

        count: usize,
        capacity: usize,
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

        pub fn capacity(&self) -> usize {
            self.capacity
        }

        pub fn len(&self) -> usize {
            self.count
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

        fn adjust_size(&mut self, capacity: usize) {
            let mut keys = Vec::with_capacity(capacity);
            let mut values = Vec::with_capacity(capacity);

            keys.resize_with(capacity, || Key(0));
            values.resize_with(capacity, || MaybeUninit::uninit());

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

        fn grow(&mut self) {
            let new_cap = self.capacity.max(2) * 3 / 2;
            debug_assert!(new_cap > self.capacity);
            self.adjust_size(new_cap);
        }

        pub fn insert(&mut self, key: Key, value: T) {
            debug_assert_ne!(key.0, 0, "0 keys mean unintialized entries");
            if (self.count + 1) as f32 > self.capacity as f32 * MAX_LOAD {
                self.grow();
            }
            self._insert(key, value)
        }

        fn _insert(&mut self, key: Key, value: T) {
            let ind = self.find_ind(key);
            let is_new_key = self.keys[ind].0 == 0;
            if is_new_key {
                self.count += 1;
            } else {
                let old = replace(&mut self.values[ind], MaybeUninit::uninit());
                unsafe {
                    let _old = old.assume_init();
                }
            }

            self.keys[ind] = key;
            self.values[ind] = MaybeUninit::new(value);
        }

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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn can_insert() {
            let mut map = PreHashMap::<i32>::with_capacity(16);

            assert_eq!(map.len(), 0);

            map.insert(Key(5), 42);
            map.insert(Key(2), 42);
            map.insert(Key(5), 31);

            assert_eq!(map.len(), 2);

            let val = map.get(Key(5)).expect("Expected to get back the value");
            assert_eq!(*val, 31);
        }

        #[test]
        fn can_grow() {
            let mut map = PreHashMap::<i32>::with_capacity(1);

            assert_eq!(map.len(), 0);

            map.insert(Key(5), 42);
            map.insert(Key(2), 42);
            map.insert(Key(5), 31);

            assert_eq!(map.len(), 2);

            let val = map.get(Key(5)).expect("Expected to get back the value");
            assert_eq!(*val, 31);

            let val = map.get(Key(2)).expect("Expected to get back the value");
            assert_eq!(*val, 42);
        }

        #[test]
        fn can_mutate_value() {
            let mut map = PreHashMap::<i32>::with_capacity(1);

            assert_eq!(map.len(), 0);

            map.insert(Key(5), 42);
            map.insert(Key(2), 42);
            map.insert(Key(5), 31);

            assert_eq!(map.len(), 2);

            let val = map.get_mut(Key(5)).expect("Expected to get back the value");
            assert_eq!(*val, 31);
            *val = 69;

            let val = map.get(Key(5)).expect("Expected to get back the value");
            assert_eq!(*val, 69);
        }

        #[test]
        fn drops_values() {
            let mut drops = Box::pin(0);

            struct Foo(*mut u32);
            impl Drop for Foo {
                fn drop(&mut self) {
                    assert_ne!(self.0 as *const _, std::ptr::null());
                    unsafe {
                        *self.0 += 1;
                    }
                }
            }

            {
                let mut map = PreHashMap::with_capacity(1);
                map.insert(Key(5), Foo(drops.as_mut().get_mut()));
                map.insert(Key(2), Foo(drops.as_mut().get_mut()));
                map.insert(Key(5), Foo(drops.as_mut().get_mut()));

                assert_eq!(map.len(), 2);
                assert_eq!(*drops, 1, "Drops the duplicated value");
            }
            assert_eq!(*drops, 3, "Drops the 2 items still in the map")
        }
    }
}
