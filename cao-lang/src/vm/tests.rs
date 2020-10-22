use super::*;

#[test]
fn test_encode() {
    let value = Pointer(12342);
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    let (_, decoded) = Pointer::decode(&encoded).unwrap();

    assert_eq!(value, decoded);
}

#[test]
fn test_set_value_memory_limit_error_raised() {
    let mut vm = VM::new(None, ());
    vm.runtime_data.memory_limit = 10;
    vm.set_value("1234567890987654321".to_owned())
        .expect_err("Should return error");
}

#[test]
fn test_array_literal_memory_limit_error_raised() {
    let program = r#"{
  "lanes": [ {
    "name": "Foo",
    "cards": [
        { "ScalarInt": 42 },
        { "ScalarInt": 42 },
        { "ScalarInt": 42 },
        { "ScalarArray": 3 }
    ]
  } ]
}
            "#;

    let compilation_unit = serde_json::from_str(program).unwrap();
    let program = crate::compiler::compile(None, compilation_unit, None).unwrap();

    let mut vm = VM::new(None, ());
    vm.runtime_data.memory_limit = 8;

    let err = vm.run(&program).expect_err("Should have failed");

    match err {
        ExecutionError::OutOfMemory => {}
        _ => panic!("Expected out of memory {:?}", err),
    }
}

#[test]
fn test_binary_operatons() {
    let mut vm = VM::new(None, ());

    vm.runtime_data.stack.push(Scalar::Integer(512)).unwrap();
    vm.runtime_data.stack.push(Scalar::Integer(42)).unwrap();

    vm.binary_op(|a, b| (a + a / b) * b).unwrap();

    let result = vm.runtime_data.stack.pop();
    match result {
        Scalar::Integer(result) => assert_eq!(result, (512 + 512 / 42) * 42),
        _ => panic!("Invalid result type"),
    }
}

#[test]
fn test_str_get() {
    let mut vm = VM::new(None, ());

    let obj = vm.set_value("winnie".to_owned()).unwrap();
    let ind = obj.index.unwrap();

    let val1 = vm.get_value_in_place::<&str>(ind).unwrap();
    let val2 = vm.get_value_in_place::<&str>(ind).unwrap();

    assert_eq!(val1, val2);
    assert_eq!(val1, "winnie");
}

#[test]
fn test_str_get_drop() {
    let mut vm = VM::new(None, ());

    let obj = vm.set_value("winnie".to_owned()).unwrap();
    let ind = obj.index.unwrap();

    {
        let _val1 = vm.get_value_in_place::<&str>(ind).unwrap();
    }

    let val2 = vm.get_value_in_place::<&str>(ind).unwrap();

    assert_eq!(val2, "winnie");
}