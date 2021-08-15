use super::*;

#[test]
fn can_insert() {
    let mut map = KeyMap::<i32>::default();

    assert_eq!(map.len(), 0);

    map.insert(Handle(5), 42).expect("insert 0");
    map.insert(Handle(2), 42).expect("insert 1");
    map.insert(Handle(5), 31).expect("insert 2");

    assert_eq!(map.len(), 2);

    let val = map.get(Handle(5)).expect("Expected to get back the value");
    assert_eq!(*val, 31);
}

#[test]
fn can_grow() {
    let mut map = KeyMap::<i32>::with_capacity(1, SysAllocator::default()).unwrap();

    assert_eq!(map.len(), 0);

    map.insert(Handle(5), 42).expect("insert 0");
    map.insert(Handle(2), 42).expect("insert 1");
    map.insert(Handle(5), 31).expect("insert 2");

    assert_eq!(map.len(), 2);

    let val = map.get(Handle(5)).expect("Expected to get back the value");
    assert_eq!(*val, 31);

    let val = map.get(Handle(2)).expect("Expected to get back the value");
    assert_eq!(*val, 42);
}

#[test]
fn can_mutate_value() {
    let mut map = KeyMap::<i32>::with_capacity(1, SysAllocator::default()).unwrap();

    assert_eq!(map.len(), 0);

    map.insert(Handle(5), 42).expect("insert 0");
    map.insert(Handle(2), 42).expect("insert 1");
    map.insert(Handle(5), 31).expect("insert 2");

    assert_eq!(map.len(), 2);

    let val = map
        .get_mut(Handle(5))
        .expect("Expected to get back the value");
    assert_eq!(*val, 31);
    *val = 69;

    let val = map.get(Handle(5)).expect("Expected to get back the value");
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
        let mut map = KeyMap::with_capacity(1, SysAllocator::default()).unwrap();
        map.insert(Handle(5), Foo(drops.as_mut().get_mut()))
            .expect("insert 0");
        map.insert(Handle(2), Foo(drops.as_mut().get_mut()))
            .expect("insert 1");
        map.insert(Handle(5), Foo(drops.as_mut().get_mut()))
            .expect("insert 2");

        assert_eq!(map.len(), 2);
        assert_eq!(*drops, 1, "Drops the duplicated value");
    }
    assert_eq!(*drops, 3, "Drops the 2 items still in the map")
}
