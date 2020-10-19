use super::*;
use crate::scalar::Scalar;
use crate::traits::ByteEncodeProperties;
use crate::vm::VM;
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

    let len = InputString::BYTELEN * 2;
    assert_eq!(len, len as i32 as usize); // sanity check
    let len = len as i32;
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
    let program = compile(None, program).unwrap();

    // Compilation was successful

    let mut vm = VM::new(None, ());
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
        Card::SetVar(VarNode(ArrayString::from("i").unwrap())),
        Card::Jump(JumpToLane("Loop".to_owned())),
    ];
    let loop_cards = vec![
        // push this value in each iteration
        Card::ScalarInt(IntegerNode(42069)),
        Card::ReadVar(VarNode(ArrayString::from("i").unwrap())),
        Card::ScalarInt(IntegerNode(1)),
        Card::Sub,
        Card::CopyLast,
        Card::SetVar(VarNode(ArrayString::from("i").unwrap())),
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
    let program = compile(None, program).unwrap();

    // Compilation was successful

    let mut vm = VM::new(None, ()).with_max_iter(50);
    let exit_code = vm.run(&program).unwrap();

    assert_eq!(exit_code, 0);
    assert_eq!(vm.read_var("i").unwrap(), Scalar::Integer(0));

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
