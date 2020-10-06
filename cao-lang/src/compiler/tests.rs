use super::*;
use crate::scalar::Scalar;
use crate::traits::ByteEncodeProperties;
use crate::vm::VM;
use arrayvec::ArrayString;
use std::str::FromStr;

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
fn post_process_raises_error_if_node_jumpts_to_itself() {
    let node = JumpNode(42);
    let msg = check_jump_post_conditions(42, &node, &Default::default()).unwrap_err();
    match msg {
        CompilationError::InvalidJump { src, dst, .. } => assert_eq!(src, dst),
        _ => panic!("Bad error msg {:?}", msg),
    };
}

#[test]
fn post_process_raises_error_if_node_jumpts_to_non_existent() {
    let node = JumpNode(42);
    let msg = check_jump_post_conditions(13, &node, &Default::default()).unwrap_err();
    match msg {
        CompilationError::InvalidJump { src, dst, .. } => {
            assert_eq!(src, 13);
            assert_eq!(dst, 42);
        }
        _ => panic!("Bad error msg {:?}", msg),
    };
}

#[test]
fn compiling_simple_program() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();
    let nodes: Nodes = [
        (
            999,
            AstNode {
                node: InstructionNode::Start,
                child: Some(0),
            },
        ),
        (
            0,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode(42.0)),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode(512.0)),
                child: Some(2),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::Add,
                child: None,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let program = CompilationUnit {
        nodes,
        sub_programs: None,
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
    let nodes: Nodes = [
        (
            999,
            AstNode {
                node: InstructionNode::Start,
                child: Some(0),
            },
        ),
        (
            0,
            AstNode {
                node: InstructionNode::ScalarInt(IntegerNode(4)),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::SetVar(VarNode(ArrayString::from("i").unwrap())),
                child: Some(2),
            },
        ),
        (
            7,
            AstNode {
                // push this value in each iteration
                node: InstructionNode::ScalarInt(IntegerNode(42069)),
                child: Some(2),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::ReadVar(VarNode(ArrayString::from("i").unwrap())),
                child: Some(3),
            },
        ),
        (
            3,
            AstNode {
                node: InstructionNode::ScalarInt(IntegerNode(1)),
                child: Some(4),
            },
        ),
        (
            4,
            AstNode {
                node: InstructionNode::Sub,
                child: Some(5),
            },
        ),
        (
            5,
            AstNode {
                node: InstructionNode::CopyLast,
                child: Some(6),
            },
        ),
        (
            6,
            AstNode {
                node: InstructionNode::SetVar(VarNode(ArrayString::from("i").unwrap())),
                child: Some(8),
            },
        ),
        (
            8,
            AstNode {
                node: InstructionNode::JumpIfTrue(JumpNode(7)),
                child: Some(9),
            },
        ),
        (
            9,
            AstNode {
                // return value
                node: InstructionNode::ScalarInt(IntegerNode(0)),
                child: None,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let program = CompilationUnit {
        nodes,
        sub_programs: None,
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
        ]
    );
}

#[test]
fn can_define_sub_programs() {
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap_or_default();
    let nodes: Nodes = [
        (
            999,
            AstNode {
                node: InstructionNode::Start,
                child: Some(0),
            },
        ),
        (
            0,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode(42.0)),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode(512.0)),
                child: Some(2),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::SubProgram(SubProgramNode(
                    InputString::from_str("add").unwrap(),
                )),
                child: None,
            },
        ),
        (
            20,
            AstNode {
                node: InstructionNode::Add,
                child: None,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let mut sub_programs = HashMap::new();
    sub_programs.insert("add".to_owned(), SubProgram { start: 20 });
    let sub_programs = Some(sub_programs);

    let cu = CompilationUnit {
        nodes,
        sub_programs,
    };
    let program = compile(None, cu).unwrap();

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