use crate::VarName;

use super::*;

fn prog() -> Module {
    let functions = vec![(
        "main".into(),
        Function::default().with_card(CardBody::CompositeCard(Box::new(
            crate::compiler::CompositeCard {
                ty: "".to_string(),
                cards: vec![
                    Card::string_card("poggers".to_owned()),
                    Card::string_card("poggers".to_owned()),
                    Card::string_card("poggers".to_owned()),
                ],
            },
        ))),
    )];
    let default_prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions,
    };
    default_prog
}

#[test]
#[cfg(feature = "serde")]
fn module_json_serde_test() {
    let default_prog = prog();
    let pl = serde_json::to_string_pretty(&default_prog).unwrap();

    let _prog: Module = serde_json::from_str(&pl).unwrap();
}

#[test]
#[cfg(feature = "serde")]
fn can_parse_json_test() {
    let json = r#"
        {
            "submodules": [],
            "imports": [],
            "functions": [["main", {
                "arguments": [],
                "cards": [ {"Call": {"function_name": "42", "args": []} } ]
            }]]
        }
"#;
    let _prog: Module = serde_json::from_str(&json).unwrap();
}

#[test]
fn module_card_fetch_test() {
    let m = prog();

    let comp_card = m
        .get_card(&CardIndex::new(0, 0))
        .expect("failed to fetch card");

    assert!(matches!(
        comp_card.body,
        super::super::CardBody::CompositeCard { .. }
    ));

    let nested_card = m
        .get_card(&CardIndex {
            function: 0,
            card_index: FunctionCardIndex {
                indices: smallvec::smallvec![0, 1],
            },
        })
        .expect("failed to fetch nested card");

    assert!(matches!(
        nested_card.body,
        super::super::CardBody::StringLiteral(_)
    ));
}

#[test]
fn remove_card_from_compositve_test() {
    let functions = vec![(
        "main".into(),
        Function::default().with_card(CardBody::CompositeCard(Box::new(
            crate::compiler::CompositeCard {
                ty: "".to_string(),
                cards: vec![
                    Card::string_card("winnie".to_owned()),
                    Card::string_card("pooh".to_owned()),
                    Card::string_card("tiggers".to_owned()),
                ],
            },
        ))),
    )];
    let mut prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions,
    };

    let removed = prog
        .remove_card(&CardIndex {
            function: 0,
            card_index: FunctionCardIndex {
                indices: smallvec::smallvec![0, 1],
            },
        })
        .unwrap();

    match removed.body {
        CardBody::StringLiteral(s) => assert_eq!(s, "pooh"),
        _ => panic!(),
    }
}

#[test]
fn remove_card_from_ifelse_test() {
    let functions = vec![(
        "main".into(),
        Function::default().with_card(CardBody::IfElse(Box::new([
            CardBody::ScalarNil.into(),
            Card::string_card("winnie"),
            Card::string_card("pooh"),
        ]))),
    )];
    let mut prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions,
    };

    let removed = prog
        .remove_card(&CardIndex {
            function: 0,
            card_index: FunctionCardIndex {
                indices: smallvec::smallvec![0, 2],
            },
        })
        .unwrap();

    match removed.body {
        CardBody::StringLiteral(s) => assert_eq!(s, "pooh"),
        _ => panic!(),
    }

    let ifelse = prog.get_card(&CardIndex::new(0, 0)).unwrap();
    match &ifelse.body {
        CardBody::IfElse(children) => {
            assert!(matches!(children[2].body, CardBody::ScalarNil));
        }
        _ => panic!(),
    }
}

#[test]
#[cfg(feature = "serde")]
fn insert_card_test() {
    let mut program = CaoProgram::default();
    program
        .functions
        .push(("poggers".to_string(), Default::default()));

    program
        .insert_card(&CardIndex::new(0, 0), CardBody::CreateTable.into())
        .unwrap();
    program
        .insert_card(
            &CardIndex::new(0, 1),
            Card::composite_card("pog".to_string(), vec![]),
        )
        .unwrap();
    program
        .insert_card(
            &CardIndex::new(0, 1).with_sub_index(0),
            CardBody::Abort.into(),
        )
        .unwrap();

    let json = serde_json::to_string_pretty(&program).unwrap();

    const EXP: &str = r#"{
  "submodules": [],
  "functions": [
    [
      "poggers",
      {
        "arguments": [],
        "cards": [
          {
            "CreateTable": null
          },
          {
            "CompositeCard": {
              "ty": "pog",
              "cards": [
                {
                  "Abort": null
                }
              ]
            }
          }
        ]
      }
    ]
  ],
  "imports": []
}"#;

    assert_eq!(json, EXP, "actual:\n{json}\nexpected:\n{EXP}");
}

#[test]
fn lookup_jump_target_test() {
    let mut program = CaoProgram::default();
    program.submodules.push((
        "foo".to_string(),
        CaoProgram {
            submodules: vec![(
                "bar".to_string(),
                CaoProgram {
                    functions: vec![(
                        "poggers".to_string(),
                        Function {
                            arguments: vec![
                                VarName::from("winnie"),
                                VarName::from("pooh"),
                                VarName::from("tiggers"),
                            ],
                            cards: vec![],
                        },
                    )],
                    ..Default::default()
                },
            )],
            ..Default::default()
        },
    ));

    let function = program.lookup_function("foo.bar.poggers").unwrap();

    assert_eq!(
        function.arguments,
        &[
            VarName::from("winnie"),
            VarName::from("pooh"),
            VarName::from("tiggers"),
        ]
    );
}

#[test]
fn lookup_jump_target_invalid_submodule_is_none_test() {
    let mut program = CaoProgram::default();
    program.submodules.push((
        "foo".to_string(),
        CaoProgram {
            submodules: vec![(
                "bar".to_string(),
                CaoProgram {
                    functions: vec![(
                        "poggers".to_string(),
                        Function {
                            arguments: vec![
                                VarName::from("winnie"),
                                VarName::from("pooh"),
                                VarName::from("tiggers"),
                            ],
                            cards: vec![],
                        },
                    )],
                    ..Default::default()
                },
            )],
            ..Default::default()
        },
    ));

    let function = program.lookup_function("foo.baz.poggers");
    assert!(function.is_none());
}

#[test]
fn lookup_jump_target_invalid_function_is_none_test() {
    let mut program = CaoProgram::default();
    program.submodules.push((
        "foo".to_string(),
        CaoProgram {
            submodules: vec![(
                "bar".to_string(),
                CaoProgram {
                    functions: vec![(
                        "poggers".to_string(),
                        Function {
                            arguments: vec![
                                VarName::from("winnie"),
                                VarName::from("pooh"),
                                VarName::from("tiggers"),
                            ],
                            cards: vec![],
                        },
                    )],
                    ..Default::default()
                },
            )],
            ..Default::default()
        },
    ));

    let function = program.lookup_function("foo.bar.poogers");
    assert!(function.is_none());
}
