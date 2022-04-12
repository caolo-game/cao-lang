use super::*;

#[test]
fn composite_card_test() {
    let cu = CaoProgram {
        module: Module {
            submodules: Default::default(),
            lanes: vec![Lane::default()
                .with_name("main")
                .with_card(Card::CompositeCard {
                    name: "triplepog".to_owned(),
                    cards: vec![
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                    ],
                })]
            .into_iter()
            .map(|lane| (lane.name.clone(), lane))
            .collect(),
        },
    };

    compile(cu, None).unwrap();
}

#[test]
fn can_binary_de_serialize_output() {
    let cu = CaoProgram {
        module: Module {
            submodules: Default::default(),
            lanes: vec![Lane::default().with_name("main").with_cards(vec![
                Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
                Card::Pass,
                Card::Pass,
            ])]
            .into_iter()
            .map(|lane| (lane.name.clone(), lane))
            .collect(),
        },
    };

    let prog = compile(cu, CompileOptions::new()).unwrap();

    let pl = bincode::serialize(&prog).unwrap();

    let _prog: CaoCompiledProgram = bincode::deserialize(&pl[..]).unwrap();
}

#[test]
fn empty_varname_is_error() {
    let cu = CaoProgram {
        module: Module {
            submodules: Default::default(),
            lanes: vec![Lane::default()
                .with_name("main")
                .with_cards(vec![Card::SetGlobalVar(VarNode::from_str_unchecked(""))])]
            .into_iter()
            .map(|lane| (lane.name.clone(), lane))
            .collect(),
        },
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
        module: Module {
            submodules: Default::default(),
            lanes: vec![
                Lane::default().with_name("main").with_card(Card::ForEach {
                    variable: VarNode::default(),
                    lane: LaneNode("pooh".to_owned()),
                }),
                Lane::default().with_name("pooh"),
            ]
            .into_iter()
            .map(|lane| (lane.name.clone(), lane))
            .collect(),
        },
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
        module: Module {
            submodules: Default::default(),
            lanes: vec![
                Lane::default().with_name("main").with_card(Card::ForEach {
                    variable: VarNode::default(),
                    lane: LaneNode("pooh".to_owned()),
                }),
                Lane::default().with_arg("asd").with_name("pooh"),
            ]
            .into_iter()
            .map(|lane| (lane.name.clone(), lane))
            .collect(),
        },
    };

    let err = compile(cu, CompileOptions::new()).unwrap_err();

    assert!(matches!(
        err.payload,
        CompilationErrorPayload::InvalidJump { .. }
    ));
}
