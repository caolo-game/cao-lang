use std::collections::BTreeMap;

use super::*;

#[test]
fn composite_card_test() {
    let mut lanes = BTreeMap::new();
    lanes.insert("main".into(), Lane::default().with_card(1));
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
        cards: [
            (
                1.into(),
                Card::CompositeCard {
                    name: "triplepog".to_string().into(),
                    cards: vec![2.into(), 2.into(), 2.into()],
                },
            ),
            (
                2.into(),
                Card::StringLiteral(StringNode("poggers".to_owned())),
            ),
        ]
        .into(),
    };

    compile(cu, None).unwrap();
}

#[test]
fn missing_card_is_an_error() {
    let mut lanes = BTreeMap::new();
    lanes.insert("main".into(), Lane::default().with_card(1));
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
        cards: [].into(),
    };
    let res = compile(cu, None).unwrap_err();
    assert!(matches!(
        res.payload,
        CompilationErrorPayload::MissingCard { card_id: CardId(1) }
    ));
}

#[test]
fn empty_foreach_is_error_test() {
    let mut lanes = BTreeMap::new();
    lanes.insert("main".into(), Lane::default().with_card(1));
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (
                1.into(),
                Card::CompositeCard {
                    name: "triplepog".to_string().into(),
                    cards: vec![2.into()],
                },
            ),
            (
                2.into(),
                Card::ForEach {
                    variable: VarNode::from_str_unchecked("pog"),
                    lane: LaneNode("".to_string()),
                },
            ),
        ]
        .into(),
        lanes,
    };

    compile(cu, None).unwrap_err();
}

#[test]
fn can_binary_de_serialize_output() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        cards: [
            (
                1.into(),
                Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
            ),
            (2.into(), Card::Pass),
        ]
        .into(),
        lanes: [(
            "main".into(),
            Lane::default().with_cards(vec![1.into(), 2.into(), 2.into()]),
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
        cards: [(
            CardId(1),
            Card::SetGlobalVar(VarNode::from_str_unchecked("")),
        )]
        .into(),
        lanes: [("main".into(), Lane::default().with_cards(vec![1.into()]))].into(),
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
        cards: [(
            CardId(1),
            Card::ForEach {
                variable: VarNode::default(),
                lane: LaneNode("pooh".to_owned()),
            },
        )]
        .into(),
        lanes: [
            ("main".into(), Lane::default().with_card(1)),
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
        cards: [(
            CardId(1),
            Card::ForEach {
                variable: VarNode::default(),
                lane: LaneNode("pooh".to_owned()),
            },
        )]
        .into(),
        lanes: BTreeMap::from([
            ("main".into(), Lane::default().with_card(1)),
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
            cards: [(CardId(1), Card::Noop)].into(),
            lanes: BTreeMap::from([("pooh".into(), Lane::default().with_card(1))]),
        },
    );
    let prog = CaoProgram {
        imports: Default::default(),
        submodules,
        cards: [(CardId(1), Card::Jump(LaneNode("coggers.pooh".to_string())))].into(),
        lanes: BTreeMap::from([("main".into(), Lane::default().with_cards(vec![CardId(1)]))]),
    };

    compile(prog, None).unwrap();
}
