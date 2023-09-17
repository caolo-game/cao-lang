use std::convert::TryInto;

use cao_lang::prelude::*;

#[test]
fn test_init_table() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::set_global_var("g_foo", Card::CreateTable)),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
    let foo = vm
        .read_var_by_name("g_foo", &program.variables)
        .expect("Failed to read foo variable");

    let p: *mut CaoLangTable = foo.try_into().expect("Expected an Object");
    assert!(!p.is_null());
}

#[test]
fn test_get_set() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::set_var("foo", Card::CreateTable))
                .with_card(Card::set_property(
                    Card::ScalarInt(42),
                    Card::read_var("foo"),
                    Card::StringLiteral("bar".to_string()),
                )) // foo.bar
                .with_card(Card::set_global_var(
                    "scoobie",
                    Card::get_property(
                        Card::read_var("foo"),
                        Card::StringLiteral("bar".to_string()),
                    ),
                )),
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

    let myboi = move |vm: &mut Vm<State>, table: &CaoLangTable| {
        let key = vm.init_string("boi").unwrap();
        let res = table
            .get(&Value::Object(key.into_inner()))
            .copied()
            .unwrap_or_default();
        if let Value::Integer(i) = res {
            vm.get_aux_mut().param = i;
        } else {
            panic!("bad value");
        }
        Ok(Value::Nil)
    };

    let mut vm = Vm::new(State { param: 0 }).unwrap();
    vm.register_native_function("boii", into_f1(myboi)).unwrap();

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::set_var("foo", Card::CreateTable))
                .with_card(Card::set_property(
                    Card::ScalarInt(42),
                    Card::read_var("foo"),
                    Card::StringLiteral("boi".to_string()),
                )) // foo.bar
                .with_card(Card::call_native("boii", vec![Card::read_var("foo")])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&program).expect("run failed");
    assert_eq!(vm.unwrap_aux().param, 42);
}
