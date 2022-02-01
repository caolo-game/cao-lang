use std::str::FromStr;
use test_log::test;

use cao_lang::{
    compiler::{CallNode, IntegerNode, LaneNode, StringNode, VarNode},
    prelude::*,
};

#[test]
fn test_trace_entry() {
    let ir = CaoIr {
        lanes: vec![
            Lane::default().with_card(Card::Jump(LaneNode::LaneId(1))),
            Lane::default().with_card(Card::CallNative(Box::new(CallNode(
                InputString::from_str("non-existent-function").unwrap(),
            )))),
        ],
    };
    let program = compile(&ir, None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    let err = vm.run(&program).expect_err("run");

    let trace = err.trace;
    assert_eq!(trace.lane, 1);
    assert_eq!(trace.card, 0);
}

#[test]
fn test_string_w_utf8() {
    let test_str = "winnie the pooh is 🔥🔥🔥 ";
    let program = CaoIr {
        lanes: vec![Lane::default()
            .with_card(Card::StringLiteral(StringNode(test_str.to_string())))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("result")))],
    };

    let program = compile(&program, Some(CompileOptions::new())).expect("compile");

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

    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_card(Card::StringLiteral(StringNode(test_str.to_string())))
            .with_card(Card::CallNative(Box::new(CallNode(
                InputString::from(name).unwrap(),
            ))))],
    };

    let program = compile(&cu, None).expect("compile");

    vm.run(&program).expect("run");
    let aux = vm.unwrap_aux();

    assert_eq!(aux.res, test_str);
}

#[test]
fn simple_if_statement() {
    let program = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("Main".to_owned())
                .with_card(Card::ScalarInt(IntegerNode(42)))
                .with_card(Card::IfTrue(LaneNode::LaneId(1))),
            Lane::default().with_cards(vec![
                Card::ScalarInt(IntegerNode(69)),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ]),
        ],
    };
    let program = compile(&program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), Value::Integer(69));
}

#[test]
fn simple_if_statement_skips_if_false() {
    let program = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("Main".to_owned())
                .with_card(Card::ScalarInt(IntegerNode(0)))
                .with_card(Card::IfTrue(LaneNode::LaneId(1))),
            Lane::default().with_cards(vec![
                Card::ScalarInt(IntegerNode(69)),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ]),
        ],
    };
    let program = compile(&program, Some(CompileOptions::new())).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).unwrap();

    let varid = program.variable_id("result").unwrap();
    let value = vm.read_var(varid);
    assert!(value.is_none(), "{:?}", value);
}

fn if_else_test(condition: Card, true_res: Card, false_res: Card, expected_result: Value) {
    let program = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("Main".to_owned())
                .with_card(condition)
                .with_card(Card::IfElse {
                    then: LaneNode::LaneId(1),
                    r#else: LaneNode::LaneId(2),
                })
                .with_card(Card::ScalarInt(IntegerNode(0xbeef)))
                .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("result2"))),
            Lane::default().with_cards(vec![
                true_res,
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ]),
            Lane::default().with_cards(vec![
                false_res,
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ]),
        ],
    };
    let program = compile(&program, Some(CompileOptions::new())).expect("compile");

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
    let program = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            // init the global variable
            .with_card(Card::ScalarInt(IntegerNode(420)))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("bar")))
            // set another value in local var
            .with_card(Card::ScalarInt(IntegerNode(123)))
            .with_card(Card::SetVar(VarNode::from_str_unchecked("foo")))
            // read the var and set the global variable
            .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo")))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("bar")))],
    };

    let program = compile(&program, None).expect("compile");

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
    let program = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("main")
                .with_card(Card::ScalarInt(IntegerNode(123)))
                .with_card(Card::SetVar(VarNode::from_str_unchecked("foo")))
                .with_card(Card::Jump(LaneNode::LaneId(1))),
            Lane::default()
                .with_name("bar")
                .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo"))),
        ],
    };

    let program = compile(&program, None).expect("compile");

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
    let program = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("Main".to_owned())
                .with_card(Card::ScalarInt(IntegerNode(69)))
                .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("result")))
                .with_card(Card::While(LaneNode::LaneId(1))),
            Lane::default().with_cards(vec![
                // Add 1 to the global 'result' variable in each iteration
                Card::ReadVar(VarNode::from_str_unchecked("result")),
                Card::ScalarInt(IntegerNode(1)),
                Card::Sub,
                Card::CopyLast, // return `result`
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ]),
        ],
    };
    /*let program =*/
    match compile(&program, Some(CompileOptions::new())).map_err(|e| e.payload) {
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
    let program = CaoIr {
        lanes: vec![
            Lane::default().with_name("Main").with_cards(vec![
                // init the result variable
                Card::ScalarInt(IntegerNode(0)),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                // loop
                Card::ScalarInt(IntegerNode(5)),
                Card::Repeat(LaneNode::LaneName("Loop".to_string())),
            ]),
            Lane::default()
                .with_name("Loop")
                .with_arg("i")
                .with_cards(vec![
                    // Add i to the global 'result' variable in each iteration
                    Card::ReadVar(VarNode::from_str_unchecked("i")),
                    Card::ReadVar(VarNode::from_str_unchecked("result")),
                    Card::Add,
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                ]),
        ],
    };
    let program = compile(&program, Some(CompileOptions::new())).expect("compile");

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
    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("Main")
            .with_cards(vec![Card::CallNative(Box::new(CallNode(
                InputString::from(name).unwrap(),
            )))])],
    };

    let prog = compile(&cu, CompileOptions::new()).unwrap();

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
lanes:
    - cards:
        - ty: CallNative
          val: "func0"
        - ty: ScalarInt
          val: 42
        - ty: CallNative
          val: "func1"
        - ty: ScalarInt
          val: 12
        - ty: ScalarFloat
          val: 4.2
        - ty: CallNative
          val: "func2"
        - ty: ScalarInt
          val: 33
        - ty: ScalarFloat
          val: 2.88
        - ty: ScalarInt
          val: 0
        - ty: CallNative
          val: "func3"
"#;
    let cu = serde_yaml::from_str(PROG).unwrap();
    let prog = compile(&cu, CompileOptions::new()).unwrap();

    vm.run(&prog).expect("run failed");

    let state = vm.unwrap_aux();
    assert!(state.call_0);
    assert!(state.call_1);
    assert!(state.call_2);
    assert!(state.call_3);
}

#[test]
fn jump_lane_w_params_test() {
    let cu = CaoIr {
        lanes: vec![
            Lane::default()
                .with_name("main")
                .with_card(Card::ScalarInt(IntegerNode(42)))
                .with_card(Card::StringLiteral(StringNode(
                    "winnie the pooh".to_owned(),
                )))
                .with_card(Card::Jump(LaneNode::LaneId(1))),
            Lane::default()
                .with_arg("foo")
                .with_arg("bar")
                .with_card(Card::ReadVar(VarNode::from_str_unchecked("foo")))
                .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("g_foo")))
                .with_card(Card::ReadVar(VarNode::from_str_unchecked("bar")))
                .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("g_bar"))),
        ],
    };

    let program = compile(&cu, CompileOptions::new()).expect("compile");

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
    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CreateTable)
            .with_card(Card::Len)
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")))],
    };

    let program = compile(&cu, CompileOptions::new()).expect("compile");

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
    let cu = CaoIr {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::CreateTable)
            .with_card(Card::SetVar(t.clone()))
            // first property
            .with_card(Card::ReadVar(t.clone()))
            .with_card(Card::StringLiteral(StringNode("asd".to_string())))
            .with_card(Card::ScalarInt(IntegerNode(42)))
            .with_card(Card::SetProperty)
            // same property as above
            .with_card(Card::ReadVar(t.clone()))
            .with_card(Card::StringLiteral(StringNode("asd".to_string())))
            .with_card(Card::ScalarInt(IntegerNode(42)))
            .with_card(Card::SetProperty)
            // new property
            .with_card(Card::ReadVar(t.clone()))
            .with_card(Card::StringLiteral(StringNode("basdasd".to_string())))
            .with_card(Card::ScalarInt(IntegerNode(42)))
            .with_card(Card::SetProperty)
            .with_card(Card::Len)
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("g_result")))],
    };

    let program = compile(&cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let len = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    assert_eq!(len, Value::Integer(2));
}
