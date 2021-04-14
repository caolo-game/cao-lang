use super::*;

#[test]
fn test_binary_operatons() {
    let mut vm = Vm::new(()).unwrap();

    vm.runtime_data.stack.push(Value::Integer(512)).unwrap();
    vm.runtime_data.stack.push(Value::Integer(42)).unwrap();

    vm.binary_op(|a, b| (a + a / b) * b).unwrap();

    let result = vm.runtime_data.stack.pop();
    match result {
        Value::Integer(result) => assert_eq!(result, (512 + 512 / 42) * 42),
        _ => panic!("Invalid result type"),
    }
}
