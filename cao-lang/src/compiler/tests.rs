use super::*;

#[test]
fn composite_card_test() {
    let functions = vec![(
        "main".into(),
        Function::default().with_card(CardBody::CompositeCard(Box::new(CompositeCard {
            ty: "triplepog".to_string(),
            cards: vec![
                CardBody::StringLiteral("poggers".to_owned()).into(),
                CardBody::StringLiteral("poggers".to_owned()).into(),
                CardBody::StringLiteral("poggers".to_owned()).into(),
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
                Card::set_global_var("asdsdad", CardBody::ScalarNil),
                CardBody::ScalarNil.into(),
                CardBody::ScalarNil.into(),
            ]),
        )]
        .into(),
    };

    let prog = compile(cu, CompileOptions::new()).unwrap();

    let pl = bincode::serde::encode_to_vec(&prog, bincode::config::standard()).unwrap();

    let (_prog, _): (CaoCompiledProgram, usize) =
        bincode::serde::decode_from_slice(&pl[..], bincode::config::standard()).unwrap();
}

#[test]
fn empty_varname_is_error() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_cards(vec![Card::set_global_var("", CardBody::ScalarNil)]),
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
                Function::default().with_card(CardBody::ScalarNil),
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

#[test]
fn test_swap_lhs_childof_rhs_fails() {
    let mut m = Module {
        submodules: [].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(CardBody::Not(UnaryExpression {
                card: Box::new(Card::scalar_int(42)),
            })),
        )]
        .into(),
        ..Default::default()
    };

    let err = m
        .swap_cards(
            &CardIndex::function(0).with_sub_index(0).with_sub_index(0),
            &CardIndex::function(0).with_sub_index(0),
        )
        .unwrap_err();

    assert!(matches!(err, SwapError::InvalidSwap));
}

#[test]
fn test_swap_rhs_childof_lhs_fails() {
    let mut m = Module {
        submodules: [].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(CardBody::Not(UnaryExpression {
                card: Box::new(Card::scalar_int(42)),
            })),
        )]
        .into(),
        ..Default::default()
    };

    let err = m
        .swap_cards(
            &CardIndex::function(0).with_sub_index(0),
            &CardIndex::function(0).with_sub_index(0).with_sub_index(0),
        )
        .unwrap_err();

    assert!(matches!(err, SwapError::InvalidSwap));
}

#[test]
fn test_swap() {
    let mut m = Module {
        submodules: [].into(),
        functions: [
            (
                "main".into(),
                Function::default()
                    .with_card(Card::scalar_int(42))
                    .with_card(Card::scalar_int(5)),
            ),
            (
                "other".into(),
                Function::default()
                    .with_card(Card::scalar_int(5))
                    .with_card(Card::scalar_int(42)),
            ),
        ]
        .into(),
        ..Default::default()
    };

    m.swap_cards(
        &CardIndex::function(0).with_sub_index(0),
        &CardIndex::function(1).with_sub_index(0),
    )
    .unwrap();

    let f = m
        .get_card(&CardIndex::function(0).with_sub_index(0))
        .unwrap();

    assert!(matches!(&f.body, &CardBody::ScalarInt(5)));

    let f = m
        .get_card(&CardIndex::function(0).with_sub_index(1))
        .unwrap();

    assert!(matches!(&f.body, &CardBody::ScalarInt(5)));

    let f = m
        .get_card(&CardIndex::function(1).with_sub_index(0))
        .unwrap();

    assert!(matches!(&f.body, &CardBody::ScalarInt(42)));

    let f = m
        .get_card(&CardIndex::function(1).with_sub_index(1))
        .unwrap();

    assert!(matches!(&f.body, &CardBody::ScalarInt(42)));

    assert_eq!(m.functions.len(), 2);
    assert_eq!(m.functions[0].1.cards.len(), 2);
    assert_eq!(m.functions[1].1.cards.len(), 2);
}
