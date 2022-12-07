use super::*;

#[test]
fn composite_card_test() {
    let lanes = vec![(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
            ty: "triplepog".to_string(),
            cards: vec![
                Card::StringLiteral("poggers".to_owned()),
                Card::StringLiteral("poggers".to_owned()),
                Card::StringLiteral("poggers".to_owned()),
            ],
        }))),
    )];
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
    };

    compile(cu, None).unwrap();
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
fn can_call_nested_function_test() {
    let submodules = vec![(
        "coggers".into(),
        Module {
            imports: Default::default(),
            submodules: Default::default(),
            lanes: vec![("pooh".into(), Lane::default().with_card(Card::Pass))],
        },
    )];
    let prog = CaoProgram {
        imports: Default::default(),
        submodules,
        lanes: vec![(
            "main".into(),
            Lane::default().with_cards(vec![Card::Jump("coggers.pooh".to_string())]),
        )],
    };

    compile(prog, None).unwrap();
}

#[test]
fn duplicate_module_is_error_test() {
    let m = Module {
        submodules: [
            ("main".into(), Default::default()),
            ("main".into(), Default::default()),
        ]
        .into(),
        lanes: [("main".into(), Lane::default())].into(),
        ..Default::default()
    };

    let _ = compile(m, None).unwrap_err();
}

#[test]
fn duplicate_lane_is_error_test() {
    let m = Module {
        submodules: [].into(),
        lanes: [
            ("main".into(), Lane::default()),
            ("main".into(), Lane::default()),
        ]
        .into(),
        ..Default::default()
    };

    let _ = compile(m, None).unwrap_err();
}
