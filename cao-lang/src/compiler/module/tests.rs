use crate::VarName;

use super::*;

#[test]
#[cfg(feature = "serde")]
fn module_bincode_serde_test() {
    use bincode::DefaultOptions;
    use serde::{Deserialize, Serialize};

    let default_prog = prog();
    let mut pl = vec![];
    let mut s = bincode::Serializer::new(&mut pl, DefaultOptions::new());
    default_prog.serialize(&mut s).unwrap();

    let mut reader = bincode::de::Deserializer::from_slice(pl.as_slice(), DefaultOptions::new());

    let _prog = Module::deserialize(&mut reader).unwrap();
}

fn prog() -> Module {
    let lanes = vec![(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard(Box::new(
            crate::compiler::CompositeCard {
                ty: "".to_string(),
                cards: vec![
                    Card::StringLiteral("poggers".to_owned()),
                    Card::StringLiteral("poggers".to_owned()),
                    Card::StringLiteral("poggers".to_owned()),
                ],
            },
        ))),
    )];
    let default_prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
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
            "lanes": [["main", {
                "arguments": [],
                "cards": [ {"Jump": "42" } ]
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
        comp_card,
        super::super::Card::CompositeCard { .. }
    ));

    let nested_card = m
        .get_card(&CardIndex {
            lane: 0,
            card_index: LaneCardIndex {
                indices: smallvec::smallvec![0, 1],
            },
        })
        .expect("failed to fetch nested card");

    assert!(matches!(nested_card, super::super::Card::StringLiteral(_)));
}

#[test]
fn remove_card_from_compositve_test() {
    let lanes = vec![(
        "main".into(),
        Lane::default().with_card(Card::CompositeCard(Box::new(
            crate::compiler::CompositeCard {
                ty: "".to_string(),
                cards: vec![
                    Card::StringLiteral("winnie".to_owned()),
                    Card::StringLiteral("pooh".to_owned()),
                    Card::StringLiteral("tiggers".to_owned()),
                ],
            },
        ))),
    )];
    let mut prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
    };

    let removed = prog
        .remove_card(&CardIndex {
            lane: 0,
            card_index: LaneCardIndex {
                indices: smallvec::smallvec![0, 1],
            },
        })
        .unwrap();

    match removed {
        Card::StringLiteral(s) => assert_eq!(s, "pooh"),
        _ => panic!(),
    }
}

#[test]
fn remove_card_from_ifelse_test() {
    let lanes = vec![(
        "main".into(),
        Lane::default().with_card(Card::IfElse(Box::new([
            Card::string_card("winnie"),
            Card::string_card("pooh"),
        ]))),
    )];
    let mut prog = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        lanes,
    };

    let removed = prog
        .remove_card(&CardIndex {
            lane: 0,
            card_index: LaneCardIndex {
                indices: smallvec::smallvec![0, 1],
            },
        })
        .unwrap();

    match removed {
        Card::StringLiteral(s) => assert_eq!(s, "pooh"),
        _ => panic!(),
    }

    let ifelse = prog.get_card(&CardIndex::new(0, 0)).unwrap();
    match ifelse {
        Card::IfElse(children) => {
            assert!(matches!(children[1], Card::Pass));
        }
        _ => panic!(),
    }
}

#[test]
fn insert_card_test() {
    let mut program = CaoProgram::default();
    program
        .lanes
        .push(("poggers".to_string(), Default::default()));

    program
        .insert_card(&CardIndex::new(0, 0), Card::CreateTable)
        .unwrap();
    program
        .insert_card(
            &CardIndex::new(0, 1),
            Card::composite_card("pog".to_string(), vec![]),
        )
        .unwrap();
    program
        .insert_card(&CardIndex::new(0, 1).with_sub_index(0), Card::Abort)
        .unwrap();

    let json = serde_json::to_string_pretty(&program).unwrap();

    const EXP: &str = r#"{
  "submodules": [],
  "lanes": [
    [
      "poggers",
      {
        "arguments": [],
        "cards": [
          "CreateTable",
          {
            "CompositeCard": {
              "ty": "pog",
              "cards": [
                "Abort"
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
                    lanes: vec![(
                        "poggers".to_string(),
                        Lane {
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

    let lane = program.lookup_lane("foo.bar.poggers").unwrap();

    assert_eq!(
        lane.arguments,
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
                    lanes: vec![(
                        "poggers".to_string(),
                        Lane {
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

    let lane = program.lookup_lane("foo.baz.poggers");
    assert!(lane.is_none());
}

#[test]
fn lookup_jump_target_invalid_lane_is_none_test() {
    let mut program = CaoProgram::default();
    program.submodules.push((
        "foo".to_string(),
        CaoProgram {
            submodules: vec![(
                "bar".to_string(),
                CaoProgram {
                    lanes: vec![(
                        "poggers".to_string(),
                        Lane {
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

    let lane = program.lookup_lane("foo.bar.poogers");
    assert!(lane.is_none());
}
