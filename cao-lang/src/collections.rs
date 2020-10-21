pub mod pre_hash_map {
    //! Hash table with pre-calculated hashes.
    use std::mem::{replace, take, MaybeUninit};

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
    }

    #[derive(Debug)]
    pub struct PreHashMap<T> {
        keys: Box<[Key]>,
        values: Box<[MaybeUninit<T>]>,

        count: usize,
        capacity: usize,
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
            let mut keys = take(&mut self.keys).into_vec();
            let mut values = take(&mut self.values).into_vec();

            keys.resize_with(capacity, || Key(0));
            values.resize_with(capacity, || MaybeUninit::uninit());

            self.keys = keys.into_boxed_slice();
            self.values = values.into_boxed_slice();
            self.capacity = capacity;
        }

        fn grow(&mut self) {
            let new_cap = self.capacity.max(1) * 3 / 2;
            self.adjust_size(new_cap);
        }

        pub fn insert(&mut self, key: Key, value: T) {
            debug_assert_ne!(key.0, 0, "0 keys mean unintialized entries");
            if (self.count + 1) as f32 > self.capacity as f32 * MAX_LOAD {
                self.grow();
            }
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
            unsafe {
                *self.values[ind].as_mut_ptr() = value;
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
    }
}
