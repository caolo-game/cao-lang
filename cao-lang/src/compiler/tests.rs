use super::*;
use crate::traits::ByteEncodeProperties;
use crate::vm::Vm;
use crate::{procedures::FunctionWrapper, scalar::Scalar};
use arrayvec::ArrayString;

#[test]
fn input_string_decode_error_handling() {
    const NEGATIVELEN: i32 = -123i32;

    let mut negativelen = vec![];
    NEGATIVELEN.encode(&mut negativelen).unwrap();

    let err = InputString::decode(&negativelen).unwrap_err();
    match err {
        StringDecodeError::LengthError(e) => assert_eq!(e, NEGATIVELEN),
        _ => panic!("Bad error {:?}", err),
    }

    let err = InputString::decode(&negativelen[..3]).unwrap_err();
    match err {
        StringDecodeError::LengthDecodeError => {}
        _ => panic!("Bad error {:?}", err),
    }

    let len = 1_000_000i32;
    let mut bytes = vec![];
    len.encode(&mut bytes).unwrap();
    bytes.extend((0..len).map(|_| 69));

    let err = InputString::decode(&bytes).unwrap_err();
    match err {
        StringDecodeError::CapacityError(_len) => {}
        _ => panic!("Bad error {:?}", err),
    }
}
#[test]
fn test_string_literal() {
    let program = CompilationUnit {
        lanes: vec![Lane {
            name: "main".to_string(),
            cards: vec![
                Card::StringLiteral(StringNode("Boiiii".to_string())),
                Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
            ],
        }],
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
fn simple_while_loop() {
    let program = CompilationUnit {
        lanes: vec![
            Lane {
                name: "Main".to_owned(),
                cards: vec![
                    // init the result variable
                    Card::ScalarInt(IntegerNode(69)),
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                    Card::While(LaneNode("Loop".to_string())),
                ],
            },
            Lane {
                name: "Loop".to_owned(),
                cards: vec![
                    // Add 1 to the global 'result' variable in each iteration
                    Card::ReadGlobalVar(VarNode::from_str_unchecked("result")),
                    Card::ScalarInt(IntegerNode(1)),
                    Card::Sub,
                    Card::CopyLast, // return `result`
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
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
    assert_eq!(vm.read_var(varid).unwrap(), Scalar::Integer(0));
}

#[test]
fn simple_for_loop() {
    let program = CompilationUnit {
        lanes: vec![
            Lane {
                name: "Main".to_owned(),
                cards: vec![
                    // init the result variable
                    Card::ScalarInt(IntegerNode(0)),
                    Card::SetGlobalVar(VarNode::from_str_unchecked("result")),
                    // loop
                    Card::ScalarInt(IntegerNode(69)),
                    Card::Repeat(LaneNode("Loop".to_string())),
                ],
            },
            Lane {
                name: "Loop".to_owned(),
                cards: vec![
                    // Add 1 to the global 'result' variable in each iteration
                    Card::ScalarInt(IntegerNode(1)),
                    Card::ReadGlobalVar(VarNode::from_str_unchecked("result")),
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
            name: "Main".to_owned(),
            cards: vec![Card::Pass, Card::Pass, Card::Pass],
        }],
    };

    let prog = compile(cu.clone(), CompileOptions::new().with_breadcrumbs(true)).unwrap();
    let mut vm = Vm::new(());
    vm.run(&prog).expect("run failed");

    assert_eq!(
        vm.history,
        vec![
            crate::vm::HistoryEntry {
                id: NodeId { lane: 0, pos: 0 },
                instr: Some(Instruction::Pass)
            },
            crate::vm::HistoryEntry {
                id: NodeId { lane: 0, pos: 1 },
                instr: Some(Instruction::Pass)
            },
            crate::vm::HistoryEntry {
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
            name: "Main".to_owned(),
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
    let name = ArrayString::from("foo").unwrap();
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Main".to_owned(),
            cards: vec![Card::Call(CallNode(name))],
        }],
    };

    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    struct State {
        called: bool,
    }

    let fun = move |vm: &mut Vm<State>, ()| {
        vm.auxiliary_data.called = true;
        Ok(())
    };
    let fun = FunctionWrapper::new(fun);

    let mut vm = Vm::new(State { called: false });
    vm.register_function(name, fun);
    vm.run(&prog).expect("run failed");
    assert!(vm.unwrap_aux().called);
}

#[test]
fn lane_names_must_be_unique() {
    let cu = CompilationUnit {
        lanes: vec![
            Lane {
                name: "Foo".to_owned(),
                cards: vec![],
            },
            Lane {
                name: "Foo".to_owned(),
                cards: vec![],
            },
        ],
    };

    let err = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap_err();
    assert!(matches!(err, CompilationError::DuplicateName(_)));
}

#[test]
fn can_json_de_serialize_output() {
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Foo".to_owned(),
            cards: vec![
                Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
                Card::Pass,
                Card::Pass,
            ],
        }],
    };

    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    let ser = serde_json::to_string(&prog).unwrap();

    let _prog: CompiledProgram = serde_json::from_str(&ser).unwrap();
}
