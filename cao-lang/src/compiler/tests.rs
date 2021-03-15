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
fn compiling_simple_program() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();
    let cards = vec![
        Card::ScalarFloat(FloatNode(42.0)),
        Card::ScalarFloat(FloatNode(512.0)),
        Card::Add,
    ];

    let program = CompilationUnit {
        lanes: vec![Lane {
            name: "Foo".to_owned(),
            cards,
        }],
    };
    let program = compile(None, program, None).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(None, ());
    vm.run(&program).unwrap();

    assert_eq!(vm.stack().len(), 1, "{:?}", vm.stack());

    let value = vm.stack().last().unwrap();
    match value {
        Scalar::Floating(i) => assert_eq!(*i, 42.0 + 512.0),
        _ => panic!("Invalid value in the stack"),
    }
}

#[test]
fn simple_looping_program() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();
    let init_cards = vec![
        Card::ScalarInt(IntegerNode(4)),
        Card::SetGlobalVar(VarNode(ArrayString::from("i").unwrap())),
        Card::Jump(JumpToLane("Loop".to_owned())),
    ];
    let loop_cards = vec![
        // push this value in each iteration
        Card::ScalarInt(IntegerNode(42069)),
        Card::ReadGlobalVar(VarNode(ArrayString::from("i").unwrap())),
        Card::ScalarInt(IntegerNode(1)),
        Card::Sub,
        Card::CopyLast,
        Card::SetGlobalVar(VarNode(ArrayString::from("i").unwrap())),
        Card::JumpIfTrue(JumpToLane("Loop".to_owned())),
    ];

    let program = CompilationUnit {
        lanes: vec![
            Lane {
                name: "Main".to_owned(),
                cards: init_cards,
            },
            Lane {
                name: "Loop".to_owned(),
                cards: loop_cards,
            },
        ],
    };
    let program = compile(None, program, None).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(None, ()).with_max_iter(150);
    let exit_code = vm.run(&program).unwrap();

    assert_eq!(exit_code, 0);
    let varid = *program
        .variables
        .0
        .get(Key::from_str("i").unwrap())
        .unwrap();
    assert_eq!(*vm.read_var(varid).unwrap(), Scalar::Integer(0));

    assert_eq!(
        vm.stack(),
        &[
            Scalar::Integer(42069),
            Scalar::Integer(42069),
            Scalar::Integer(42069),
            Scalar::Integer(42069),
        ]
    );
}

#[test]
fn breadcrumbs_work_as_expected() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();

    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Main".to_owned(),
            cards: vec![Card::Pass, Card::Pass, Card::Pass],
        }],
    };

    let prog = compile(
        None,
        cu.clone(),
        CompileOptions::new().with_breadcrumbs(true),
    )
    .unwrap();
    let mut vm = Vm::new(None, ());
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
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();

    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Main".to_owned(),
            cards: vec![Card::Pass, Card::Pass, Card::Pass],
        }],
    };

    let prog = compile(None, cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();
    let mut vm = Vm::new(None, ());
    vm.run(&prog).expect("run failed");
    assert_eq!(vm.history, vec![]);
}

#[test]
fn call_test() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();

    let name = ArrayString::from("foo").unwrap();
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Main".to_owned(),
            cards: vec![Card::Call(CallNode(name))],
        }],
    };

    let prog = compile(None, cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    struct State {
        called: bool,
    }

    let fun = move |vm: &mut Vm<State>, ()| {
        vm.auxiliary_data.called = true;
        Ok(())
    };
    let fun = FunctionWrapper::new(fun);

    let mut vm = Vm::new(None, State { called: false });
    vm.register_function(name, fun);
    vm.run(&prog).expect("run failed");
    assert!(vm.unwrap_aux().called);
}

#[test]
fn lane_names_must_be_unique() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();

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

    let err = compile(None, cu, CompileOptions::new().with_breadcrumbs(false)).unwrap_err();
    assert!(matches!(err, CompilationError::DuplicateName(_)));
}

#[test]
fn can_json_de_serialize_output() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();

    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: "Foo".to_owned(),
            cards: vec![
                Card::SetGlobalVar(VarNode(ArrayString::from("asdsdad").unwrap())),
                Card::Pass,
                Card::Pass,
            ],
        }],
    };

    let prog = compile(None, cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    let ser = serde_json::to_string(&prog).unwrap();

    let _prog: CompiledProgram = serde_json::from_str(&ser).unwrap();
}
