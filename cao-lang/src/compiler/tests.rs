use super::*;

#[test]
fn composite_card_test() {
    let functions = vec![(
        "main".into(),
        Function::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
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
        functions,
    };

    compile(cu, None).unwrap();
}

#[test]
#[cfg(feature = "serde")]
fn can_binary_de_serialize_output() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_cards(vec![
                Card::set_global_var("asdsdad", Card::ScalarNil),
                Card::ScalarNil,
                Card::ScalarNil,
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
        functions: [(
            "main".into(),
            Function::default().with_cards(vec![Card::set_global_var("", Card::ScalarNil)]),
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
            functions: vec![(
                "pooh".into(),
                Function::default().with_card(Card::ScalarNil),
            )],
        },
    )];
    let prog = CaoProgram {
        imports: Default::default(),
        submodules,
        functions: vec![(
            "main".into(),
            Function::default().with_cards(vec![Card::call_function("coggers.pooh", vec![])]),
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
        functions: [("main".into(), Function::default())].into(),
        ..Default::default()
    };

    let _ = compile(m, None).unwrap_err();
}

#[test]
fn duplicate_function_is_error_test() {
    let m = Module {
        submodules: [].into(),
        functions: [
            ("main".into(), Function::default()),
            ("main".into(), Function::default()),
        ]
        .into(),
        ..Default::default()
    };

    let _ = compile(m, None).unwrap_err();
}
