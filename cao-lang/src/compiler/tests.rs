use std::collections::HashMap;

use super::*;

#[test]
fn composite_card_test() {
    let mut lanes = HashMap::new();
    lanes.insert(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard {
            name: "triplepog".to_owned(),
            cards: vec![
                Card::StringLiteral(StringNode("poggers".to_owned())),
                Card::StringLiteral(StringNode("poggers".to_owned())),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ],
        }),
    );
    let cu = CaoProgram {
        submodules: Default::default(),
        lanes,
    };

    compile(cu, None).unwrap();
}

#[test]
fn can_binary_de_serialize_output() {
    let cu = CaoProgram {
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default().with_cards(vec![
                Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
                Card::Pass,
                Card::Pass,
            ]),
        )]
        .into(),
    };

    let prog = compile(cu, CompileOptions::new()).unwrap();

    let pl = bincode::serialize(&prog).unwrap();

    let _prog: CaoCompiledProgram = bincode::deserialize(&pl[..]).unwrap();
}

#[test]
fn empty_varname_is_error() {
    let cu = CaoProgram {
        submodules: Default::default(),
        lanes: [(
            "main".into(),
            Lane::default().with_cards(vec![Card::SetGlobalVar(VarNode::from_str_unchecked(""))]),
        )]
        .into(),
    };

    let err = compile(cu, CompileOptions::new()).unwrap_err();

    assert!(matches!(
        err.payload,
        CompilationErrorPayload::EmptyVariable
    ));
}

#[test]
fn empty_arity_in_foreach_is_an_error() {
    let cu = CaoProgram {
        submodules: Default::default(),
        lanes: [
            (
                "main".into(),
                Lane::default().with_card(Card::ForEach {
                    variable: VarNode::default(),
                    lane: LaneNode("pooh".to_owned()),
                }),
            ),
            ("pooh".into(), Lane::default()),
        ]
        .into(),
    };

    let err = compile(cu, CompileOptions::new()).unwrap_err();

    assert!(matches!(
        err.payload,
        CompilationErrorPayload::InvalidJump { .. }
    ));
}

#[test]
fn arity_1_in_foreach_is_an_error() {
    let cu = CaoProgram {
        submodules: Default::default(),
        lanes: HashMap::from([
            (
                "main".into(),
                Lane::default().with_card(Card::ForEach {
                    variable: VarNode::default(),
                    lane: LaneNode("pooh".to_owned()),
                }),
            ),
            ("pooh".into(), Lane::default().with_arg("asd")),
        ]),
    };

    let err = compile(cu, CompileOptions::new()).unwrap_err();

    assert!(matches!(
        err.payload,
        CompilationErrorPayload::InvalidJump { .. }
    ));
}

#[test]
fn can_call_nested_function_test() {
    let mut submodules = HashMap::new();
    submodules.insert(
        "coggers".into(),
        Module {
            submodules: Default::default(),
            lanes: HashMap::from([("pooh".into(), Lane::default().with_card(Card::Noop))]),
        },
    );
    let prog = CaoProgram {
        submodules,
        lanes: HashMap::from([(
            "main".into(),
            Lane::default().with_cards(vec![Card::Jump(LaneNode("coggers.pooh".to_string()))]),
        )]),
    };

    compile(prog, None).unwrap();
}
