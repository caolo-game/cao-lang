use test_log::test;

use cao_lang::{
    compiler::{CompositeCard, Module},
    prelude::*,
};

#[test]
fn composite_card_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
                ty: "triplepog".to_string(),
                cards: vec![
                    Card::StringLiteral("poggers".to_owned()),
                    Card::set_global_var("result"),
                ],
            }))),
        )]
        .into(),
    };

    let program = compile(cu, None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let res = vm.read_var_by_name("result", &program.variables).unwrap();
    let ress = unsafe { res.as_str().expect("Failed to read string") };

    assert_eq!(ress, "poggers");
}

#[test]
fn test_trace_entry() {
    let ir = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_card(Card::Jump("pooh".to_owned())),
            ),
            (
                "pooh".into(),
                Lane::default().with_card(Card::call_native("non-existent-function")),
            ),
        ]
        .into(),
    };
    let program = compile(ir.clone(), None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    let err = vm.run(&program).expect_err("run");

    let trace = err.trace;

    let error_card = ir
        .get_card(&trace[0])
        .expect("Expected to find the errored card");

    assert!(matches!(error_card, Card::CallNative(_)));
}

#[test]
fn test_string_w_utf8() {
    let test_str = "winnie the pooh is 🔥🔥🔥 ";
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::StringLiteral(test_str.to_string()))
                .with_card(Card::set_global_var("result")),
        )]
        .into(),
    };

    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let varid = program.variable_id("result").expect("varid");
    let ptr = vm.read_var(varid).expect("read var");
    let ress = unsafe { ptr.as_str().expect("Failed to read string") };

    assert_eq!(test_str, ress);
}

#[test]
fn test_string_param() {
    let name = "fooboi";
    let test_str = "tiggers boi";

    struct State {
        res: String,
    }

    let fun = move |vm: &mut Vm<State>, arg: cao_lang::StrPointer| {
        let vm_str = unsafe { arg.get_str().unwrap().to_string() };
        vm.auxiliary_data.res = vm_str;
        Ok(())
    };

    let mut vm = Vm::new(State {
        res: "".to_string(),
    })
    .unwrap();
    vm.register_function(name, into_f1(fun));

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::StringLiteral(test_str.to_string()))
                .with_card(Card::call_native(name)),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    vm.run(&program).expect("run");
    let aux = vm.unwrap_aux();

    assert_eq!(aux.res, test_str);
}

#[test]
fn simple_if_statement() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(Card::ScalarInt(42))
                    .with_card(Card::IfTrue(Box::new(Card::Jump("pooh".to_owned())))),
            ),
            (
                "pooh".into(),
                Lane::default()
                    .with_cards(vec![Card::ScalarInt(69), Card::set_global_var("result")]),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), Value::Integer(69));
}

#[test]
fn simple_if_statement_skips_if_false() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::ScalarInt(0))
                .with_card(Card::IfTrue(Box::new(Card::CompositeCard(Box::new(
                    CompositeCard {
                        ty: "".to_string(),
                        cards: vec![Card::ScalarInt(69), Card::set_global_var("result")],
                    },
                )))))
                .with_card(Card::ScalarInt(1))
                .with_card(Card::IfFalse(Box::new(Card::CompositeCard(Box::new(
                    CompositeCard {
                        ty: "".to_string(),
                        cards: vec![Card::ScalarInt(42), Card::set_global_var("result")],
                    },
                ))))),
        )]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).unwrap();

    let varid = program.variable_id("result").unwrap();
    let value = vm.read_var(varid);
    assert!(
        value.is_none(),
        "expected value to be none, instead got: {:?}",
        value
    );
}

fn if_else_test(condition: Card, true_res: Card, false_res: Card, expected_result: Value) {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(condition)
                    .with_card(Card::IfElse(Box::new([
                        Card::Jump("pooh".to_string()),
                        Card::Jump("tiggers".to_string()),
                    ])))
                    .with_card(Card::ScalarInt(0xbeef))
                    .with_card(Card::set_global_var("result2")),
            ),
            (
                "pooh".into(),
                Lane::default().with_cards(vec![true_res, Card::set_global_var("result")]),
            ),
            (
                "tiggers".into(),
                Lane::default().with_cards(vec![false_res, Card::set_global_var("result")]),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("program run");

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), expected_result);

    // test if the cards after the if statement were executed
    let varid = program.variable_id("result2").expect("varid");
    assert_eq!(
        vm.read_var(varid).expect("read var"),
        Value::Integer(0xbeef)
    );
}

#[test]
fn simple_if_else_statement_test_then() {
    if_else_test(
        Card::ScalarInt(1),
        Card::ScalarInt(42),
        Card::ScalarInt(69),
        Value::Integer(42),
    );
}

#[test]
fn simple_if_else_statement_test_else() {
    if_else_test(
        Card::ScalarInt(0),
        Card::ScalarInt(42),
        Card::ScalarInt(69),
        Value::Integer(69),
    );
}

#[test]
fn test_local_variable() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                // init the global variable
                .with_card(Card::ScalarInt(420))
                .with_card(Card::set_global_var("bar"))
                // set another value in local var
                .with_card(Card::ScalarInt(123))
                .with_card(Card::set_var("foo"))
                // read the var and set the global variable
                .with_card(Card::read_var("foo"))
                .with_card(Card::set_global_var("bar")),
        )]
        .into(),
    };

    let program = compile(program, None).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    vm.run(&program).unwrap();

    let res = vm
        .read_var_by_name("bar", &program.variables)
        .expect("Failed to read result variable");
    assert_eq!(res, Value::Integer(123));
}

#[test]
fn local_variable_doesnt_leak_out_of_scope() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(Card::ScalarInt(123))
                    .with_card(Card::set_var("foo"))
                    .with_card(Card::Jump("bar".to_string())),
            ),
            (
                "bar".into(),
                Lane::default().with_card(Card::read_var("foo")),
            ),
        ]
        .into(),
    };

    let program = compile(program, None).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    let res = vm.run(&program);
    let _name = "foo".to_string();
    assert!(matches!(
        res.map_err(|err| err.payload),
        Err(ExecutionErrorPayload::VarNotFound(_name))
    ));
}

#[test]
fn simple_for_loop() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_cards(vec![
                    // init the result variable
                    Card::ScalarInt(0),
                    Card::set_global_var("result"),
                    // loop
                    Card::ScalarInt(5),
                    Card::Repeat {
                        i: Some("i".to_string()),
                        body: Box::new(Card::composite_card(
                            "",
                            vec![Card::read_var("i"), Card::Jump("Loop".to_string())],
                        )),
                    },
                ]),
            ),
            (
                "Loop".into(),
                Lane::default().with_arg("i").with_cards(vec![
                    // Add i to the global 'result' variable in each iteration
                    Card::read_var("i"),
                    Card::read_var("result"),
                    Card::Add,
                    Card::set_global_var("result"),
                ]),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    vm.run(&program).unwrap();

    let res = vm
        .read_var_by_name("result", &program.variables)
        .expect("Failed to read result variable");
    assert_eq!(res, Value::Integer((0..5).fold(0, std::ops::Add::add)));
}

#[test]
fn call_native_test() {
    let name = "foo";
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default().with_cards(vec![Card::call_native(name)]),
        )]
        .into(),
    };

    let prog = compile(cu, CompileOptions::new()).unwrap();

    struct State {
        called: bool,
    }

    let fun = move |vm: &mut Vm<State>| {
        vm.auxiliary_data.called = true;
        Ok(())
    };

    let mut vm = Vm::new(State { called: false }).unwrap();
    vm.register_function(name, fun);
    vm.run(&prog).expect("run failed");
    assert!(vm.unwrap_aux().called);
}

#[test]
fn test_function_registry() {
    struct State {
        call_0: bool,
        call_1: bool,
        call_2: bool,
        call_3: bool,
    }

    fn myfunc0(vm: &mut Vm<State>) -> Result<(), ExecutionErrorPayload> {
        vm.auxiliary_data.call_0 = true;
        Ok(())
    }

    fn myfunc1(vm: &mut Vm<State>, i: i64) -> Result<(), ExecutionErrorPayload> {
        vm.auxiliary_data.call_1 = true;
        assert_eq!(i, 42);
        Ok(())
    }

    fn myfunc2(vm: &mut Vm<State>, i: i64, j: f64) -> Result<(), ExecutionErrorPayload> {
        vm.auxiliary_data.call_2 = true;
        assert_eq!(i, 12);
        assert_eq!(j, 4.2);
        Ok(())
    }

    fn myfunc3(vm: &mut Vm<State>, i: i64, j: f64, b: bool) -> Result<(), ExecutionErrorPayload> {
        vm.auxiliary_data.call_3 = true;
        assert_eq!(i, 33);
        assert_eq!(j, 2.88);
        assert_eq!(b, false);
        Ok(())
    }

    let mut vm = Vm::new(State {
        call_0: false,
        call_1: false,
        call_2: false,
        call_3: false,
    })
    .unwrap();

    // if this compiles we're good to go
    vm.register_function("func0", myfunc0);
    vm.register_function("func1", into_f1(myfunc1));
    vm.register_function("func2", into_f2(myfunc2));
    vm.register_function("func3", into_f3(myfunc3));

    const PROG: &str = r#"
submodules: []
imports: []
lanes:
    - - main
      - arguments: []
        cards:
            - !CallNative "func0"
            - !ScalarInt 42
            - !CallNative "func1"
            - !ScalarInt 12
            - !ScalarFloat 4.2
            - !CallNative "func2"
            - !ScalarInt 33
            - !ScalarFloat 2.88
            - !ScalarInt 0
            - !CallNative "func3"
"#;
    let cu = serde_yaml::from_str(PROG).unwrap();
    let prog = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&prog).expect("run failed");

    let state = vm.unwrap_aux();
    assert!(state.call_0);
    assert!(state.call_1);
    assert!(state.call_2);
    assert!(state.call_3);
}

#[test]
fn jump_lane_w_params_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(Card::StringLiteral("winnie the pooh".to_owned()))
                    .with_card(Card::ScalarInt(42))
                    .with_card(Card::Jump("pooh".to_owned())),
            ),
            (
                "pooh".into(),
                Lane::default()
                    .with_arg("foo")
                    .with_arg("bar")
                    .with_card(Card::read_var("foo"))
                    .with_card(Card::set_global_var("g_foo"))
                    .with_card(Card::read_var("bar"))
                    .with_card(Card::set_global_var("g_bar")),
            ),
        ]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
    let foo = vm
        .read_var_by_name("g_foo", &program.variables)
        .expect("Failed to read foo variable");
    let bar = vm
        .read_var_by_name("g_bar", &program.variables)
        .expect("Failed to read bar variable");
    dbg!(foo, bar);
    assert!(matches!(foo, Value::Integer(42)));
    match bar {
        Value::String(s) => unsafe {
            let val = s.get_str().unwrap();
            assert_eq!(val, "winnie the pooh");
        },
        _ => panic!("Unexpected value set for bar {:?}", bar),
    }
}

#[test]
fn len_test_empty() {
    // happy path
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::CreateTable)
                .with_card(Card::Len)
                .with_card(Card::set_global_var("g_result")),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let len = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    assert_eq!(len, Value::Integer(0));
}

#[test]
fn len_test_happy() {
    // happy path
    let t = "t";
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(Card::CreateTable)
                .with_card(Card::set_var(t))
                // first property
                .with_card(Card::read_var(t))
                .with_card(Card::StringLiteral("asd".to_string()))
                .with_card(Card::ScalarInt(42))
                .with_card(Card::SetProperty)
                // same property as above
                .with_card(Card::read_var(t))
                .with_card(Card::StringLiteral("asd".to_string()))
                .with_card(Card::ScalarInt(69))
                .with_card(Card::SetProperty)
                // new property
                .with_card(Card::read_var(t))
                .with_card(Card::StringLiteral("basdasd".to_string()))
                .with_card(Card::ScalarInt(89))
                .with_card(Card::SetProperty)
                // len
                .with_card(Card::read_var(t))
                .with_card(Card::Len)
                .with_card(Card::set_global_var("g_result")),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let len = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    assert_eq!(len, Value::Integer(2));
}

#[test]
fn nested_module_can_call_self_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                lanes: [
                    (
                        "win".into(),
                        Lane::default().with_card(Card::Jump("nie".to_owned())),
                    ),
                    (
                        "nie".into(),
                        Lane::default()
                            .with_card(Card::StringLiteral("poggers".to_owned()))
                            .with_card(Card::set_global_var("g_result")),
                    ),
                ]
                .into(),
            },
        )]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("winnie.win".to_owned())),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn invalid_import_is_error_test() {
    let cu = CaoProgram {
        submodules: [].into(),
        imports: ["winnie..pooh".into()].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("pooh".to_owned())),
        )]
        .into(),
    };

    compile(cu, CompileOptions::new()).expect_err("compile");
}

#[test]
fn non_existent_import_is_error_test() {
    let cu = CaoProgram {
        submodules: [].into(),
        imports: ["winnie..pooh".into()].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("pooh".to_owned())),
        )]
        .into(),
    };

    let err = compile(cu, CompileOptions::new()).expect_err("compile");

    // TODO: the error itself should be more meaningful
    dbg!(&err);
    assert!(matches!(
        err.payload,
        CompilationErrorPayload::InvalidJump { .. }
    ));
}

#[test]
fn import_in_submodule_test() {
    let cu = Module {
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                lanes: [(
                    "pooh".into(),
                    Lane::default()
                        .with_card(Card::StringLiteral("poggers".to_owned()))
                        .with_card(Card::set_global_var("g_result")),
                )]
                .into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        lanes: [(
            "run".into(),
            Lane::default().with_card(Card::Jump("pooh".to_owned())),
        )]
        .into(),
    };
    let cu = Module {
        imports: ["foo.run".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("run".to_owned())),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn can_import_submodule_test() {
    let cu = Module {
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                lanes: [(
                    "pooh".into(),
                    Lane::default()
                        .with_card(Card::StringLiteral("poggers".to_owned()))
                        .with_card(Card::set_global_var("g_result")),
                )]
                .into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        lanes: [(
            "run".into(),
            Lane::default().with_card(Card::Jump("pooh".to_owned())),
        )]
        .into(),
    };
    let cu = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("winnie.pooh".to_owned())),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn can_import_function_from_super_module_test() {
    let winnie = Module {
        imports: ["super.super.pog".into()].into(),
        submodules: Default::default(),
        lanes: [(
            "pooh".into(),
            Lane::default().with_card(Card::Jump("pog".to_owned())),
        )]
        .into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        lanes: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        lanes: [
            (
                "run".into(),
                Lane::default().with_card(Card::Jump("winnie.pooh".to_owned())),
            ),
            (
                "pog".into(),
                Lane::default()
                    .with_card(Card::StringLiteral("poggers".to_owned()))
                    .with_card(Card::set_global_var("g_result")),
            ),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("bar.run".to_owned())),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn import_super_module_test() {
    let winnie = Module {
        imports: ["super.super.bar".into()].into(),
        submodules: Default::default(),
        lanes: [(
            "pooh".into(),
            Lane::default().with_card(Card::Jump("bar.pog".to_owned())),
        )]
        .into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        lanes: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        lanes: [
            (
                "run".into(),
                Lane::default().with_card(Card::Jump("winnie.pooh".to_owned())),
            ),
            (
                "pog".into(),
                Lane::default()
                    .with_card(Card::StringLiteral("poggers".to_owned()))
                    .with_card(Card::set_global_var("g_result")),
            ),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(Card::Jump("bar.run".to_owned())),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn local_variable_regression_test() {
    /*
    In main:
        In composite card:
            set local var (1)

    In other lane:
        in composite card:
            set another local var (2) (name can be different)
        in another composite card:
            read the local var

    Bug: Reads the variable (1) from main
    Should read variable (2)
    */
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(Card::composite_card(
                        "asd",
                        vec![Card::scalar_int(69), Card::set_var("pog")],
                    ))
                    .with_card(Card::jump("tiggers")),
            ),
            (
                "tiggers".into(),
                Lane::default()
                    .with_card(Card::composite_card(
                        "basd",
                        vec![Card::scalar_int(42), Card::set_var("foo")],
                    ))
                    .with_card(Card::composite_card(
                        "zasd",
                        vec![Card::read_var("foo"), Card::set_global_var("pooh")],
                    )),
            ),
        ]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("pooh", &program.variables)
        .expect("Failed to read pooh variable");

    assert_eq!(result.as_int().unwrap(), 42);
}

#[test]
fn simple_while_test() {
    const N: i64 = 42;
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        lanes: [(
            "main".to_string(),
            Lane::default()
                .with_card(Card::ScalarInt(N))
                .with_card(Card::set_var("i"))
                .with_card(Card::ScalarInt(0))
                .with_card(Card::set_global_var("pooh"))
                .with_card(Card::While(Box::new([
                    Card::read_var("i"),
                    Card::composite_card(
                        "body",
                        vec![
                            // Increment pooh
                            Card::ScalarInt(1),
                            Card::read_var("pooh"),
                            Card::Add,
                            Card::set_global_var("pooh"),
                            // decrement loop counter
                            Card::read_var("i"),
                            Card::ScalarInt(1),
                            Card::Sub,
                            Card::set_var("i"),
                        ],
                    ),
                ]))),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("pooh", &program.variables)
        .expect("Failed to read pooh variable");

    assert_eq!(result.as_int().unwrap(), N);
}

#[test]
fn set_var_to_empty_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        lanes: [(
            "main".to_string(),
            Lane::default().with_card(Card::set_var("i")),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
}
