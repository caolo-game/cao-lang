use std::str::FromStr;
use test_log::test;

use cao_lang::{
    compiler::{CallNode, CardId, IntegerNode, LaneNode, Module, StringNode, VarNode},
    prelude::*,
};

#[test]
fn composite_card_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
        cards: [
            (
                1.into(),
                Card::CompositeCard {
                    name: "triplepog".to_string().into(),
                    cards: vec![2.into(), 3.into()],
                },
            ),
            (
                2.into(),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ),
            (
                3.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
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
            ("main".into(), Lane::default().with_card(1)),
            ("pooh".into(), Lane::default().with_card(2)),
        ]
        .into(),
        cards: [
            (1.into(), Card::Jump(LaneNode("pooh".to_owned()))),
            (
                2.into(),
                Card::CallNative(Box::new(CallNode(
                    InputString::from_str("non-existent-function").unwrap(),
                ))),
            ),
        ]
        .into(),
    };
    let program = compile(ir, None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    let err = vm.run(&program).expect_err("run");

    let trace = err.trace;
    assert_eq!(trace.lane.as_ref(), "pooh");
    assert_eq!(trace.card, 0);
}

#[test]
fn test_string_w_utf8() {
    let test_str = "winnie the pooh is 🔥🔥🔥 ";
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes: [("main".into(), Lane::default().with_card(1).with_card(2))].into(),
        cards: [
            (
                1.into(),
                Card::StringLiteral(StringNode(test_str.to_string())),
            ),
            (
                2.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
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
        lanes: [("main".into(), Lane::default().with_card(1).with_card(2))].into(),
        cards: [
            (
                1.into(),
                Card::StringLiteral(StringNode(test_str.to_string())),
            ),
            (
                2.into(),
                Card::CallNative(Box::new(CallNode(InputString::from(name).unwrap()))),
            ),
        ]
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
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(42))),
            (2.into(), Card::IfTrue(3.into())),
            (3.into(), Card::Jump(LaneNode("pooh".to_owned()))),
            (4.into(), Card::ScalarInt(IntegerNode(69))),
            (
                5.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
        .into(),
        lanes: [
            ("main".into(), Lane::default().with_card(1).with_card(2)),
            (
                "pooh".into(),
                Lane::default().with_cards(vec![4.into(), CardId(5)]),
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
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(0))),
            (
                2.into(),
                Card::CompositeCard {
                    name: None,
                    cards: vec![3.into(), 4.into()],
                },
            ),
            (3.into(), Card::ScalarInt(IntegerNode(69))),
            (
                4.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
            (5.into(), Card::IfTrue(2.into())),
            (6.into(), Card::ScalarInt(IntegerNode(1))),
            (7.into(), Card::IfFalse(8.into())),
            (
                8.into(),
                Card::CompositeCard {
                    name: None,
                    cards: vec![9.into(), 10.into()],
                },
            ),
            (9.into(), Card::ScalarInt(IntegerNode(42))),
            (
                10.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default()
                .with_card(1)
                .with_card(5)
                .with_card(6)
                .with_card(7),
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
        cards: [
            (1.into(), condition),
            (
                2.into(),
                Card::IfElse {
                    then: 5.into(),
                    r#else: 6.into(),
                },
            ),
            (3.into(), Card::ScalarInt(IntegerNode(0xbeef))),
            (
                4.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result2")),
            ),
            (5.into(), Card::Jump(LaneNode("pooh".to_string()))),
            (6.into(), Card::Jump(LaneNode("tiggers".to_string()))),
            (7.into(), true_res),
            (
                8.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
            (9.into(), false_res),
        ]
        .into(),
        lanes: [
            (
                "main".into(),
                Lane::default()
                    .with_card(1)
                    .with_card(2)
                    .with_card(3)
                    .with_card(4),
            ),
            ("pooh".into(), Lane::default().with_card(7).with_card(8)),
            ("tiggers".into(), Lane::default().with_card(9).with_card(8)),
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
        Card::ScalarInt(IntegerNode(1)),
        Card::ScalarInt(IntegerNode(42)),
        Card::ScalarInt(IntegerNode(69)),
        Value::Integer(42),
    );
}

#[test]
fn simple_if_else_statement_test_else() {
    if_else_test(
        Card::ScalarInt(IntegerNode(0)),
        Card::ScalarInt(IntegerNode(42)),
        Card::ScalarInt(IntegerNode(69)),
        Value::Integer(69),
    );
}

#[test]
fn test_local_variable() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(420))),
            (
                2.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("bar")),
            ),
            (3.into(), Card::ScalarInt(IntegerNode(123))),
            (4.into(), Card::SetVar(VarNode::from_str_unchecked("foo"))),
            (5.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
            (
                6.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("bar")),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default()
                // init the global variable
                .with_card(1)
                .with_card(2)
                // set another value in local var
                .with_card(3)
                .with_card(4)
                // read the var and set the global variable
                .with_card(5)
                .with_card(6),
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
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(123))),
            (2.into(), Card::SetVar(VarNode::from_str_unchecked("foo"))),
            (3.into(), Card::Jump(LaneNode("bar".to_string()))),
            (4.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
        ]
        .into(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_card(1).with_card(2).with_card(3),
            ),
            ("bar".into(), Lane::default().with_card(4)),
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
fn simple_while_loop() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(69))),
            (
                2.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
            (3.into(), Card::While(LaneNode("pooh".to_string()))),
            (
                4.into(),
                Card::ReadVar(VarNode::from_str_unchecked("result")),
            ),
            (5.into(), Card::ScalarInt(IntegerNode(1))),
            (6.into(), Card::Sub),
            (7.into(), Card::CopyLast), // return `result`
            (
                8.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
        .into(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_card(1).with_card(2).with_card(3),
            ),
            (
                "pooh".into(),
                Lane::default().with_cards(vec![
                    // Add 1 to the global 'result' variable in each iteration
                    4.into(),
                    5.into(),
                    6.into(),
                    7.into(),
                    8.into(),
                ]),
            ),
        ]
        .into(),
    };
    /*let program =*/
    match compile(program, Some(CompileOptions::new())).map_err(|e| e.payload) {
        Ok(_) => {
            panic!("Expected error, update this test pls")
        }
        Err(CompilationErrorPayload::Unimplemented(_)) => {}
        Err(err) => {
            panic!("Expected unimplemented error, instead got: {}", err)
        }
    }

    // Compilation was successful
    // TODO: once while is implemented

    // let mut vm = Vm::new(()).with_max_iter(10000);
    // let exit_code = vm.run(&program).unwrap();
    // assert_eq!(exit_code, 0);
    //
    // let varid = program.variable_id("result").unwrap();
    // assert_eq!(vm.read_var(varid).unwrap(), Scalar::Integer(0));
}

#[test]
fn simple_for_loop() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(0))),
            (
                2.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
            (3.into(), Card::ScalarInt(IntegerNode(5))),
            (4.into(), Card::Repeat(LaneNode("Loop".to_string()))),
            (5.into(), Card::ReadVar(VarNode::from_str_unchecked("i"))),
            (
                6.into(),
                Card::ReadVar(VarNode::from_str_unchecked("result")),
            ),
            (7.into(), Card::Add),
            (
                8.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ),
        ]
        .into(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_cards(vec![1.into(), 2.into(), 3.into(), 4.into()]),
            ),
            (
                "Loop".into(),
                Lane::default().with_arg("i").with_cards(vec![
                    // Add i to the global 'result' variable in each iteration
                    5.into(),
                    6.into(),
                    7.into(),
                    8.into(),
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
        cards: [(
            1.into(),
            Card::CallNative(Box::new(CallNode(InputString::from(name).unwrap()))),
        )]
        .into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
submodules: {}
imports: []
cards:
    1: !CallNative "func0"
    2: !ScalarInt 42
    3: !CallNative "func1"
    4: !ScalarInt 12
    5: !ScalarFloat 4.2
    6: !CallNative "func2"
    7: !ScalarInt 33
    8: !ScalarFloat 2.88
    9: !ScalarInt 0
    10: !CallNative "func3"
lanes:
    main:
        name: main
        arguments: []
        cards: [1,2,3,4,5,6,7,8,9,10]
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
        cards: [
            (1.into(), Card::ScalarInt(IntegerNode(42))),
            (
                2.into(),
                Card::StringLiteral(StringNode("winnie the pooh".to_owned())),
            ),
            (3.into(), Card::Jump(LaneNode("pooh".to_owned()))),
            (4.into(), Card::ReadVar(VarNode::from_str_unchecked("foo"))),
            (
                5.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_foo")),
            ),
            (6.into(), Card::ReadVar(VarNode::from_str_unchecked("bar"))),
            (
                7.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_bar")),
            ),
        ]
        .into(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_card(1).with_card(2).with_card(3),
            ),
            (
                "pooh".into(),
                Lane::default()
                    .with_arg("foo")
                    .with_arg("bar")
                    .with_card(4)
                    .with_card(5)
                    .with_card(6)
                    .with_card(7),
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
        cards: [
            (1.into(), Card::CreateTable),
            (2.into(), Card::Len),
            (
                3.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default().with_card(1).with_card(2).with_card(3),
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
    let t = VarNode::from_str_unchecked("t");
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (1.into(), Card::CreateTable),
            (2.into(), Card::SetVar(t.clone())),
            (3.into(), Card::ReadVar(t.clone())),
            (4.into(), Card::StringLiteral(StringNode("asd".to_string()))),
            (5.into(), Card::ScalarInt(IntegerNode(42))),
            (6.into(), Card::SetProperty),
            (7.into(), Card::ReadVar(t.clone())),
            (8.into(), Card::StringLiteral(StringNode("asd".to_string()))),
            (9.into(), Card::ScalarInt(IntegerNode(42))),
            (10.into(), Card::SetProperty),
            (11.into(), Card::ReadVar(t.clone())),
            (
                12.into(),
                Card::StringLiteral(StringNode("basdasd".to_string())),
            ),
            (13.into(), Card::ScalarInt(IntegerNode(42))),
            (14.into(), Card::SetProperty),
            (15.into(), Card::ReadVar(t.clone())),
            (16.into(), Card::Len),
            (
                17.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
            ),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default().with_cards((1..=17).map(Into::into).collect::<Vec<_>>()),
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
                cards: [
                    (1.into(), Card::Jump(LaneNode("nie".to_owned()))),
                    (
                        2.into(),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                    ),
                    (
                        3.into(),
                        Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
                    ),
                ]
                .into(),
                lanes: [
                    ("win".into(), Lane::default().with_card(1)),
                    ("nie".into(), Lane::default().with_card(2).with_card(3)),
                ]
                .into(),
            },
        )]
        .into(),
        cards: [(1.into(), Card::Jump(LaneNode("winnie.win".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
        cards: [(1.into(), Card::Jump(LaneNode("pooh".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
    };

    compile(cu, CompileOptions::new()).expect_err("compile");
}

#[test]
fn non_existent_import_is_error_test() {
    let cu = CaoProgram {
        submodules: [].into(),
        imports: ["winnie..pooh".into()].into(),
        cards: [(1.into(), Card::Jump(LaneNode("pooh".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
                cards: [
                    (
                        1.into(),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                    ),
                    (
                        2.into(),
                        Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
                    ),
                ]
                .into(),
                lanes: [("pooh".into(), Lane::default().with_card(1).with_card(2))].into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        cards: [(1.into(), Card::Jump(LaneNode("pooh".to_owned())))].into(),
        lanes: [("run".into(), Lane::default().with_card(1))].into(),
    };
    let cu = Module {
        imports: ["foo.run".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        cards: [(1.into(), Card::Jump(LaneNode("run".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
                cards: [
                    (
                        1.into(),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                    ),
                    (
                        2.into(),
                        Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
                    ),
                ]
                .into(),
                lanes: [("pooh".into(), Lane::default().with_card(1).with_card(2))].into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        cards: [(1.into(), Card::Jump(LaneNode("pooh".to_owned())))].into(),
        lanes: [("run".into(), Lane::default().with_card(1))].into(),
    };
    let cu = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        cards: [(1.into(), Card::Jump(LaneNode("winnie.pooh".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
        submodules: Default::default(),
        cards: [(1.into(), Card::Jump(LaneNode("pog".to_owned())))].into(),
        lanes: [("pooh".into(), Lane::default().with_card(1))].into(),
        imports: ["super.super.pog".into()].into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        cards: [].into(),
        lanes: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        cards: [
            (1.into(), Card::Jump(LaneNode("winnie.pooh".to_owned()))),
            (
                2.into(),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ),
            (
                3.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
            ),
        ]
        .into(),
        lanes: [
            ("run".into(), Lane::default().with_card(1)),
            ("pog".into(), Lane::default().with_card(2).with_card(3)),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        cards: [(1.into(), Card::Jump(LaneNode("bar.run".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
        cards: [(1.into(), Card::Jump(LaneNode("bar.pog".to_owned())))].into(),
        lanes: [("pooh".into(), Lane::default().with_card(1))].into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        cards: [].into(),
        lanes: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        cards: [
            (1.into(), Card::Jump(LaneNode("winnie.pooh".to_owned()))),
            (
                2.into(),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ),
            (
                3.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")),
            ),
        ]
        .into(),
        lanes: [
            ("run".into(), Lane::default().with_card(1)),
            ("pog".into(), Lane::default().with_card(2).with_card(3)),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        cards: [(1.into(), Card::Jump(LaneNode("bar.run".to_owned())))].into(),
        lanes: [("main".into(), Lane::default().with_card(1))].into(),
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
