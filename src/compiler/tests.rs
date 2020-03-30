use super::*;
use crate::scalar::Scalar;
use crate::vm::VM;
use arrayvec::ArrayString;

#[test]
fn post_process_raises_error_if_node_jumpts_to_itself() {
    let node = JumpNode { nodeid: 42 };
    check_jump_post_conditions(42, &node, &Default::default()).unwrap_err();
}

#[test]
fn post_process_raises_error_if_node_jumpts_to_non_existent() {
    let node = JumpNode { nodeid: 42 };
    check_jump_post_conditions(13, &node, &Default::default()).unwrap_err();
}

#[test]
fn compiling_simple_program() {
    simple_logger::init().unwrap_or(());
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
                node: InstructionNode::ScalarFloat(FloatNode { value: 42.0 }),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode { value: 512.0 }),
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

    let program = CompilationUnit { nodes };
    let program = compile(program).unwrap();

    log::warn!("Program: {:?}", program);

    // Compilation was successful

    let mut vm = VM::new(());
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
    simple_logger::init().unwrap_or(());
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
                node: InstructionNode::ScalarInt(IntegerNode { value: 4 }),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::SetVar(VarNode {
                    name: ArrayString::from("i").unwrap(),
                }),
                child: Some(2),
            },
        ),
        (
            7,
            AstNode {
                // push this value in each iteration
                node: InstructionNode::ScalarInt(IntegerNode { value: 42069 }),
                child: Some(2),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::ReadVar(VarNode {
                    name: ArrayString::from("i").unwrap(),
                }),
                child: Some(3),
            },
        ),
        (
            3,
            AstNode {
                node: InstructionNode::ScalarInt(IntegerNode { value: 1 }),
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
                node: InstructionNode::SetVar(VarNode {
                    name: ArrayString::from("i").unwrap(),
                }),
                child: Some(8),
            },
        ),
        (
            8,
            AstNode {
                node: InstructionNode::JumpIfTrue(JumpNode { nodeid: 7 }),
                child: Some(9),
            },
        ),
        (
            9,
            AstNode {
                // return value
                node: InstructionNode::ScalarInt(IntegerNode { value: 0 }),
                child: None,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let program = CompilationUnit { nodes };
    let program = compile(program).unwrap();

    // Compilation was successful

    let mut vm = VM::new(()).with_max_iter(50);
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
