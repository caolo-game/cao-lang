use std::collections::BTreeMap;

use super::*;

#[test]
fn composite_card_test() {
    let mut lanes = BTreeMap::new();
    lanes.insert(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
            name: "triplepog".to_string().into(),
            cards: vec![
                Card::StringLiteral(StringNode("poggers".to_owned())),
                Card::StringLiteral(StringNode("poggers".to_owned())),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ],
        }))),
    );
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
    };

    compile(cu, None).unwrap();
}

#[test]
fn empty_foreach_is_error_test() {
    let mut lanes = BTreeMap::new();
    lanes.insert(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
            name: "triplepog".to_string().into(),
            cards: vec![Card::ForEach {
                variable: VarNode::from_str_unchecked("pog"),
                lane: LaneNode("".to_string()),
            }],
        }))),
    );
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
    };

    compile(cu, None).unwrap_err();
}

#[test]
fn can_binary_de_serialize_output() {
    let cu = CaoProgram {
        imports: Default::default(),
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
        imports: Default::default(),
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
        imports: Default::default(),
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
        imports: Default::default(),
        submodules: Default::default(),
        lanes: BTreeMap::from([
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
    let mut submodules = BTreeMap::new();
    submodules.insert(
        "coggers".into(),
        Module {
            imports: Default::default(),
            submodules: Default::default(),
            lanes: BTreeMap::from([("pooh".into(), Lane::default().with_card(Card::Noop))]),
        },
    );
    let prog = CaoProgram {
        imports: Default::default(),
        submodules,
        lanes: BTreeMap::from([(
            "main".into(),
            Lane::default().with_cards(vec![Card::Jump(LaneNode("coggers.pooh".to_string()))]),
        )]),
    };

    compile(prog, None).unwrap();
}
