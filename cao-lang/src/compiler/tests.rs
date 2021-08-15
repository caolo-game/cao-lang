use super::*;

#[test]
fn lane_names_must_be_unique() {
    let cu = CaoIr {
        lanes: vec![
            Lane::default().with_name("Foo").with_cards(vec![]),
            Lane::default().with_name("Foo").with_cards(vec![]),
        ],
    };

    let err = compile(&cu, CompileOptions::new()).unwrap_err();
    assert!(matches!(
        err.payload,
        CompilationErrorPayload::DuplicateName(_)
    ));
}

#[test]
fn can_json_de_serialize_output() {
    let cu = CaoIr {
        lanes: vec![Lane::default().with_name("Foo").with_cards(vec![
            Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
            Card::Pass,
            Card::Pass,
        ])],
    };

    let prog = compile(&cu, CompileOptions::new()).unwrap();

    let ser = serde_json::to_string(&prog).unwrap();

    let _prog: CaoProgram = serde_json::from_str(&ser).unwrap();
}

#[test]
fn empty_varname_is_error() {
    let cu = CaoIr {
        lanes: vec![
            Lane::default().with_cards(vec![Card::SetGlobalVar(VarNode::from_str_unchecked(""))])
        ],
    };

    let err = compile(&cu, CompileOptions::new()).unwrap_err();

    assert!(matches!(
        err.payload,
        CompilationErrorPayload::EmptyVariable
    ));
}
