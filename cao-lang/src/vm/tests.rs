use super::*;

#[test]
fn test_binary_operatons() {
    let mut vm = Vm::new(()).unwrap();

    vm.runtime_data
        .value_stack
        .push(Value::Integer(512))
        .unwrap();
    vm.runtime_data
        .value_stack
        .push(Value::Integer(42))
        .unwrap();

    vm.binary_op(|a, b| (a + a / b) * b).unwrap();

    let result = vm.runtime_data.value_stack.pop();
    match result {
        Value::Integer(result) => assert_eq!(result, (512 + 512 / 42) * 42),
        _ => panic!("Invalid result type"),
    }
}

#[test]
fn test_can_init_str() {
    let mut vm = Vm::new(()).unwrap();

    let ptr = vm.init_string("poggers").unwrap();

    let val = Value::String(ptr);

    let result = unsafe { val.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[cfg(feature = "serde")]
#[test]
fn test_can_save_and_restore_values() {
    let mut vm = Vm::new(()).unwrap();

    // init an object `val` with 1 entry {'pog': 42}
    let mut obj = vm.init_table().unwrap();
    let pog = vm.init_string("pog").unwrap();
    unsafe { obj.as_mut() }
        .insert(Value::String(pog), 42.into())
        .unwrap();

    let val = Value::Object(obj.as_ptr());

    // serialize the object
    let owned = OwnedValue::from(val);
    let pl = serde_json::to_string_pretty(&owned).unwrap();

    // load the object in a new VM
    let loaded: OwnedValue = serde_json::from_str(pl.as_str()).unwrap();
    let mut vm = Vm::new(()).unwrap();
    let loaded = vm.insert_value(&loaded).unwrap();

    // check the contents
    let loaded_table = vm.get_table(loaded).unwrap();
    assert_eq!(loaded_table.len(), 1);
    for (k, v) in loaded_table.iter() {
        let k = unsafe { k.as_str().unwrap() };
        let v = v.as_int().unwrap();

        assert_eq!(k, "pog");
        assert_eq!(v, 42);
    }
}
