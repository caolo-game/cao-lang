use super::*;
use crate::traits::ByteEncodeProperties;

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

    let len = 1_000_000i32;
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
fn lane_names_must_be_unique() {
    let cu = CompilationUnit {
        lanes: vec![
            Lane {
                name: Some("Foo".to_owned()),
                cards: vec![],
            },
            Lane {
                name: Some("Foo".to_owned()),
                cards: vec![],
            },
        ],
    };

    let err = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap_err();
    assert!(matches!(err, CompilationError::DuplicateName(_)));
}

#[test]
fn can_json_de_serialize_output() {
    let cu = CompilationUnit {
        lanes: vec![Lane {
            name: Some("Foo".to_owned()),
            cards: vec![
                Card::SetGlobalVar(VarNode::from_str_unchecked("asdsdad")),
                Card::Pass,
                Card::Pass,
            ],
        }],
    };

    let prog = compile(cu, CompileOptions::new().with_breadcrumbs(false)).unwrap();

    let ser = serde_json::to_string(&prog).unwrap();

    let _prog: CaoProgram = serde_json::from_str(&ser).unwrap();
}
