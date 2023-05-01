use super::*;

#[test]
fn occupied_entry_test() {
    let mut map = CaoHashMap::<i32, i32>::with_capacity_in(1, SysAllocator::default()).unwrap();

    map.insert(42, 69).unwrap();

    let entry = map.entry(42).unwrap();

    let x = entry.or_insert_with(|| panic!("Shouldn't insert new"));

    assert_eq!(*x, 69);
}

#[test]
fn vacant_entry_inserts_test() {
    let mut map = CaoHashMap::<i32, i32>::with_capacity_in(1, SysAllocator::default()).unwrap();

    let cap = map.capacity();
    assert_eq!(
        cap, 1,
        "Test code assumes that the capacity is 1 at this point"
    );

    let entry = map.entry(42).unwrap();

    let mut called = false;
    let x = entry.or_insert_with(|| {
        called = true;
        69
    });

    assert_eq!(*x, 69);
    assert!(called);
    assert!(map.capacity() > cap);
}

#[test]
fn can_grow() {
    let mut map = CaoHashMap::<i32, i32>::with_capacity_in(1, SysAllocator::default()).unwrap();

    assert_eq!(map.len(), 0);

    map.insert(5, 42).expect("insert 0");
    map.insert(2, 42).expect("insert 1");
    map.insert(5, 31).expect("insert 2");

    assert_eq!(map.len(), 2);

    let val = map.get(&5).expect("Expected to get back the value");
    assert_eq!(*val, 31);

    let val = map.get(&2).expect("Expected to get back the value");
    assert_eq!(*val, 42);
}

#[test]
fn can_mutate_value() {
    let mut map = CaoHashMap::<i32, i32>::with_capacity_in(1, SysAllocator::default()).unwrap();

    assert_eq!(map.len(), 0);

    map.insert(5, 42).expect("insert 0");
    map.insert(2, 42).expect("insert 1");
    map.insert(5, 31).expect("insert 2");

    assert_eq!(map.len(), 2);

    let val = map.get_mut(&5).expect("Expected to get back the value");
    assert_eq!(*val, 31);
    *val = 69;

    let val = map.get(&5).expect("Expected to get back the value");
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
        let mut map = CaoHashMap::with_capacity_in(1, SysAllocator::default()).unwrap();
        map.insert(5, Foo(drops.as_mut().get_mut()))
            .expect("insert 0");
        map.insert(2, Foo(drops.as_mut().get_mut()))
            .expect("insert 1");
        map.insert(5, Foo(drops.as_mut().get_mut()))
            .expect("insert 2");

        assert_eq!(map.len(), 2);
        assert_eq!(*drops, 1, "Drops the duplicated value");
    }
    assert_eq!(*drops, 3, "Drops the 2 items still in the map")
}

#[test]
fn insert_duplicate_test() {
    let mut map = CaoHashMap::<&str, i32>::default();

    map.insert("asd", 42).unwrap();
    map.insert("asd", 31).unwrap();
    map.insert("asd", 92).unwrap();
    map.insert("asd", 22).unwrap();
    map.insert("asd", 82).unwrap();
    map.insert("asd", 12).unwrap();
    map.insert("asd", 82).unwrap();

    assert_eq!(map.len(), 1);
}

#[test]
fn removing_duplicate_hash_test() {
    // if the key of two distinct keys map to the same bucket,
    // then we should still be able to look up the second, after deleting the first
    unsafe {
        let mut map = CaoHashMap::<&str, i32>::default();
        map.insert_with_hint(42, "winnie", 42).unwrap();
        map.insert_with_hint(42, "pooh", 69).unwrap();

        let i = map.remove_with_hint(42, "winnie").unwrap();
        assert_eq!(i, 42);

        let i = *map.get_with_hint(42, "pooh").unwrap();
        assert_eq!(i, 69);
    }
}
