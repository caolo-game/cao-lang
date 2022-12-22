use std::convert::TryInto;
use test_log::test;

use cao_lang::prelude::*;

#[test]
fn test_init_table() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::CreateTable)
                .with_card(Card::set_global_var("g_foo")),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
    let foo = vm
        .read_var_by_name("g_foo", &program.variables)
        .expect("Failed to read foo variable");

    let p: *mut FieldTable = foo.try_into().expect("Expected an Object");
    assert!(!p.is_null());
}

#[test]
fn test_get_set() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::CreateTable)
                .with_card(Card::set_var("foo"))
                .with_card(Card::read_var("foo"))
                .with_card(Card::StringLiteral("bar".to_string()))
                .with_card(Card::ScalarInt(42))
                .with_card(Card::SetProperty) // foo.bar
                .with_card(Card::read_var("foo"))
                .with_card(Card::StringLiteral("bar".to_string()))
                .with_card(Card::GetProperty)
                .with_card(Card::set_global_var("scoobie")),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let scoobie = vm
        .read_var_by_name("scoobie", &program.variables)
        .expect("Failed to read scoobie variable");

    assert!(matches!(scoobie, Value::Integer(42)));
}

#[test]
fn test_native_w_table_input() {
    struct State {
        param: i64,
    }

    let myboi = move |vm: &mut Vm<State>, table: &FieldTable| {
        let key = vm.init_string("boi").unwrap();
        let res = table.get(&Value::String(key)).copied().unwrap_or_default();
        if let Value::Integer(i) = res {
            vm.get_aux_mut().param = i;
        } else {
            panic!("bad value");
        }
        Ok(())
    };

    let mut vm = Vm::new(State { param: 0 }).unwrap();
    vm.register_function("boii", into_f1(myboi));

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::CreateTable)
                .with_card(Card::set_var("foo"))
                .with_card(Card::read_var("foo"))
                .with_card(Card::StringLiteral("boi".to_string()))
                .with_card(Card::ScalarInt(42))
                .with_card(Card::SetProperty) // foo.bar
                .with_card(Card::call_native("boii")),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&program).expect("run failed");
    assert_eq!(vm.unwrap_aux().param, 42);
}
