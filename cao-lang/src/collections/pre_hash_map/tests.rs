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
