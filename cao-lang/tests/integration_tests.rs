use cao_lang::{
    collections::pre_hash_map::Key,
    compiler::{CallNode, IntegerNode, LaneNode, StringNode, VarNode},
    prelude::*,
};
use std::convert::TryInto;
use std::str::FromStr;

#[test]
fn test_array_literal_memory_limit_error_raised() {
    const PROGRAM: &str = r#"
lanes:
    - 
        name: Foo
        cards:
            - ty: ScalarInt
              val: 42
            - ty: ScalarInt
              val: 42
            - ty: ScalarInt
              val: 42
            - ty: ScalarArray
              val: 3
"#;

    let compilation_unit = serde_yaml::from_str(PROGRAM).unwrap();
    let program = cao_lang::compiler::compile(compilation_unit, None).unwrap();

    let mut vm = Vm::new(());
    vm.runtime_data.memory_limit = 8;

    let err = vm.run(&program).expect_err("Should have failed");

    match err {
        ExecutionError::OutOfMemory => {}
        _ => panic!("Expected out of memory {:?}", err),
    }
}

#[test]
fn test_string_literal() {
    let program = CompilationUnit {
        lanes: vec![Lane::default()
            .with_name("main")
            .with_card(Card::StringLiteral(StringNode("Boiiii".to_string())))
            .with_card(Card::SetGlobalVar(VarNode::from_str_unchecked("result")))],
    };

    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).unwrap();

    let varid = program.variable_id("result").unwrap();
    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(10000);

    // expect result variable to not exist at this point
    if let Some(s) = vm.read_var(varid) {
        panic!(
            "Expected variable to not be initialized at this point {:?}",
            s
        );
    }

    let res = vm.run(&program).unwrap();
    assert_eq!(res, 0);

    let myptr = vm.read_var(varid).expect("failed to get `result`");
    let resvalue: &str = vm
        .get_value_in_place::<&str>(myptr.try_into().unwrap())
        .expect("Failed to get value of `result`");

    assert_eq!(resvalue, "Boiiii");
}

#[test]
fn simple_if_statement() {
    let program = CompilationUnit {
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
    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(1000);
    let exit_code = vm.run(&program).expect("run");
    assert_eq!(exit_code, 0);

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), Scalar::Integer(69));
}

#[test]
fn simple_if_statement_skips_if_false() {
    let program = CompilationUnit {
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
    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(1000);
    let exit_code = vm.run(&program).unwrap();
    assert_eq!(exit_code, 0);

    let varid = program.variable_id("result").unwrap();
    let value = vm.read_var(varid);
    assert!(value.is_none(), "{:?}", value);
}

fn if_else_test(condition: Card, true_res: Card, false_res: Card, expected_result: Scalar) {
    let program = CompilationUnit {
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
    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(1000);
    let exit_code = vm.run(&program).expect("program run");
    assert_eq!(exit_code, 0);

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), expected_result);

    // test if the cards after the if statement were executed
    let varid = program.variable_id("result2").expect("varid");
    assert_eq!(
        vm.read_var(varid).expect("read var"),
        Scalar::Integer(0xbeef)
    );
}

#[test]
fn simple_if_else_statement_test_then() {
    if_else_test(
        Card::ScalarInt(IntegerNode(1)),
        Card::ScalarInt(IntegerNode(42)),
        Card::ScalarInt(IntegerNode(69)),
        Scalar::Integer(42),
    );
}

#[test]
fn simple_if_else_statement_test_else() {
    if_else_test(
        Card::ScalarInt(IntegerNode(0)),
        Card::ScalarInt(IntegerNode(42)),
        Card::ScalarInt(IntegerNode(69)),
        Scalar::Integer(69),
    );
}

#[test]
fn simple_while_loop() {
    let program = CompilationUnit {
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
    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(10000);
    let exit_code = vm.run(&program).unwrap();
    assert_eq!(exit_code, 0);

    let varid = program.variable_id("result").unwrap();
    assert_eq!(vm.read_var(varid).unwrap(), Scalar::Integer(0));
}

#[test]
fn simple_for_loop() {
    let program = CompilationUnit {
        lanes: vec![
            Lane {
                name: Some("Main".to_owned()),
                cards: vec![
                    // init the result variable
                    Card::ScalarInt(IntegerNode(0)),
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                    // loop
                    Card::ScalarInt(IntegerNode(69)),
                    Card::Repeat(LaneNode::LaneName("Loop".to_string())),
                ],
            },
            Lane {
                name: Some("Loop".to_owned()),
                cards: vec![
                    // Add 1 to the global 'result' variable in each iteration
                    Card::ScalarInt(IntegerNode(1)),
                    Card::ReadVar(VarNode::from_str_unchecked("result")),
                    Card::Add,
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                    Card::ScalarInt(IntegerNode(420)), // check if last value is discarded
                ],
            },
        ],
    };
    let program = compile(program, Some(CompileOptions { breadcrumbs: false })).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).with_max_iter(10000);
    let exit_code = vm.run(&program).unwrap();

    assert_eq!(exit_code, 0);
    let varid = *program
        .variables
        .0
        .get(Key::from_str("result").unwrap())
        .unwrap();
    assert_eq!(vm.read_var(varid).unwrap(), Scalar::Integer(69));
}

#[test]
fn breadcrumbs_work_as_expected() {
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: Some("Main".to_owned()),
            cards: vec![Card::Pass, Card::Pass, Card::Pass],
        }],
    };

    let prog = compile(cu.clone(), CompileOptions::new().with_breadcrumbs(true)).unwrap();
    let mut vm = Vm::new(());
    vm.run(&prog).expect("run failed");

    assert_eq!(
        vm.history,
        vec![
            cao_lang::vm::HistoryEntry {
                id: NodeId { lane: 0, pos: 0 },
                instr: Some(Instruction::Pass)
            },
            cao_lang::vm::HistoryEntry {
                id: NodeId { lane: 0, pos: 1 },
                instr: Some(Instruction::Pass)
            },
            cao_lang::vm::HistoryEntry {
                id: NodeId { lane: 0, pos: 2 },
                instr: Some(Instruction::Pass)
            }
        ]
    );
}

#[test]
fn no_breadcrumbs_emitted_when_compiled_with_off() {
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: Some("Main".to_owned()),
            cards: vec![Card::Pass, Card::Pass, Card::Pass],
        }],
    };

    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();
    let mut vm = Vm::new(());
    vm.run(&prog).expect("run failed");
    assert_eq!(vm.history, vec![]);
}

#[test]
fn call_test() {
    let name = "foo";
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: Some("Main".to_owned()),
            cards: vec![Card::Call(Box::new(CallNode(
                InputString::from(name).unwrap(),
            )))],
        }],
    };

    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    struct State {
        called: bool,
    }

    let fun = move |vm: &mut Vm<State>| {
        vm.auxiliary_data.called = true;
        Ok(())
    };

    let mut vm = Vm::new(State { called: false });
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

    fn myfunc0(vm: &mut Vm<State>) -> ExecutionResult {
        vm.auxiliary_data.call_0 = true;
        Ok(())
    }

    fn myfunc1(vm: &mut Vm<State>, i: i32) -> ExecutionResult {
        vm.auxiliary_data.call_1 = true;
        assert_eq!(i, 42);
        Ok(())
    }

    fn myfunc2(vm: &mut Vm<State>, i: i32, j: f32) -> ExecutionResult {
        vm.auxiliary_data.call_2 = true;
        assert_eq!(i, 12);
        assert_eq!(j, 4.2);
        Ok(())
    }

    fn myfunc3(vm: &mut Vm<State>, i: i32, j: f32, b: bool) -> ExecutionResult {
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
    });

    // if this compiles we're good to go
    vm.register_function("func0", myfunc0);
    vm.register_function("func1", into_f1(myfunc1));
    vm.register_function("func2", into_f2(myfunc2));
    vm.register_function("func3", into_f3(myfunc3));

    const PROG: &str = r#"
lanes:
    - cards:
        - ty: Call
          val: "func0"
        - ty: ScalarInt
          val: 42
        - ty: Call
          val: "func1"
        - ty: ScalarInt
          val: 12
        - ty: ScalarFloat
          val: 4.2
        - ty: Call
          val: "func2"
        - ty: ScalarInt
          val: 33
        - ty: ScalarFloat
          val: 2.88
        - ty: ScalarInt
          val: 0
        - ty: Call
          val: "func3"
"#;
    let cu = serde_yaml::from_str(PROG).unwrap();
    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    vm.run(&prog).expect("run failed");

    let state = vm.unwrap_aux();
    assert!(state.call_0);
    assert!(state.call_1);
    assert!(state.call_2);
    assert!(state.call_3);
}