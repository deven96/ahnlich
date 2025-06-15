use crate::error::DslError;
use ahnlich_types::{
    ai::{
        execution_provider::ExecutionProvider,
        models::AiModel,
        pipeline::ai_query::Query as AiQuery,
        preprocess::PreprocessAction,
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetPred, GetSimN, InfoServer,
            ListStores, Ping, PurgeStores, Set,
        },
    },
    algorithm::{algorithms::Algorithm, nonlinear::NonLinearAlgorithm},
    keyval::{
        store_input::Value as StoreValue, AiStoreEntry, StoreInput, StoreValue as KvStoreValue,
    },
    metadata::{metadata_value::Value, MetadataValue},
    predicates::{
        predicate::Kind as PredicateKind, predicate_condition::Kind, Equals, In, NotEquals, NotIn,
        Predicate, PredicateCondition,
    },
};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use crate::ai::parse_ai_query;

#[test]
fn test_single_query_parse() {
    let input = r#"purgestores"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::PurgeStores(PurgeStores {})]
    );
    let input = r#"infoserver"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::InfoServer(InfoServer {})]
    );
    let input = r#"  Ping  "#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::Ping(Ping {})]
    );
}

#[test]
fn test_multi_query_parse() {
    let input = r#" INFOSERVER ; listSTORES;"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![
            AiQuery::InfoServer(InfoServer {}),
            AiQuery::ListStores(ListStores {})
        ]
    );
}

#[test]
fn test_no_valid_input_in_query() {
    let input = r#" random ; listSTORES;"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 8));
    let input = r#" INfoSERVER ; random; ping"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (13, 20));
}

#[test]
fn test_drop_store_parse() {
    let input = r#"DROPSTORE random"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropStore(DropStore {
            store: "random".to_string(),
            error_if_not_exists: true
        })]
    );
    let input = r#"dropstore yeezy_store IF exists"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropStore(DropStore {
            store: "yeezy_store".to_string(),
            error_if_not_exists: false,
        })]
    );
    let input = r#"dropstore yeezy IF NOT exists"#;
    // IF NOT EXISTS is not valid syntax
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (15, 29));
}

#[test]
fn test_create_predicate_index_parse() {
    let input = r#"CREATEPREDINDEX (one, two, 3) in tapHstore1"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::CreatePredIndex(CreatePredIndex {
            store: "tapHstore1".to_string(),
            predicates: vec!["one".to_string(), "two".to_string(), "3".to_string(),]
        })]
    );
}

#[test]
fn test_drop_pred_index_parse() {
    let input = r#"DROPPREDINDEX (here, th2) in store2"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropPredIndex(DropPredIndex {
            store: "store2".to_string(),
            predicates: vec!["here".to_string(), "th2".to_string()],
            error_if_not_exists: true,
        })]
    );
    let input = r#"DROPPREDINDEX IF EXISTS (off) in storememe"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropPredIndex(DropPredIndex {
            store: "storememe".to_string(),
            predicates: vec!["off".to_string()],
            error_if_not_exists: false,
        })]
    );
}

#[test]
fn test_create_store_parse() {
    // simple syntax
    let input = r#"CREATEstore storename QUERYMODEL resnet-50 INDEXMODEL all-minilm-l6-v2 "#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::CreateStore(CreateStore {
            store: "storename".to_string(),
            query_model: AiModel::Resnet50 as i32,
            index_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })]
    );
    let input = r#"CREATEstore IF NOT EXISTS storename QUERYMODEL resnet-50 INDEXMODEL all-minilm-l6-v2 PREDICATES (department, faculty) STOREORIGINAL"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::CreateStore(CreateStore {
            store: "storename".to_string(),
            query_model: AiModel::Resnet50 as i32,
            index_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec!["department".to_string(), "faculty".to_string()],
            non_linear_indices: vec![],
            error_if_exists: false,
            store_original: true,
        })]
    );
    let input = r#"createstore school QUERYMODEL all-minilm-l6-v2 INDEXMODEL resnet-50 NONLINEARALGORITHMINDEX (kdtree) STOREORIGINAL"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::CreateStore(CreateStore {
            store: "school".to_string(),
            query_model: AiModel::AllMiniLmL6V2 as i32,
            index_model: AiModel::Resnet50 as i32,
            predicates: vec![],
            non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
            error_if_exists: true,
            store_original: true,
        })]
    );
}

#[test]
fn test_create_non_linear_algorithm_parse() {
    let input = r#"createnonlinearalgorithmindex (fake) in store2"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 46));
    let input = r#"createnonlinearalgorithmindex (kdtree) in store2"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::CreateNonLinearAlgorithmIndex(
            CreateNonLinearAlgorithmIndex {
                store: "store2".to_string(),
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
            }
        )]
    );
}

#[test]
fn test_get_sim_n_parse() {
    // passing in a zero value to ensure it doesn't work
    let input = r#"GETSIMN 0 with [random] using kdtree in store1 where ( author = hi )"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 68));
    let input = r#"GETSIMN 5 with [hi my name is carter] using cosinesimilarity preprocessaction MODELPREPROCESSING in random"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::GetSimN(GetSimN {
            store: "random".to_string(),
            search_input: Some(StoreInput {
                value: Some(StoreValue::RawString("hi my name is carter".to_string()))
            }),
            closest_n: 5,
            algorithm: Algorithm::CosineSimilarity as i32,
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing as i32,
            execution_provider: None,
        })]
    );
    let input = r#"GETSIMN 8 with [testing the limits of life] using euclideandistance executionprovider tensorrt in other where ((year != 2012) AND (month not in (december, october)))"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::GetSimN(GetSimN {
            store: "other".to_string(),
            search_input: Some(StoreInput {
                value: Some(StoreValue::RawString(
                    "testing the limits of life".to_string()
                ))
            }),
            closest_n: 8,
            algorithm: Algorithm::EuclideanDistance as i32,
            condition: Some(
                PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(NotEquals {
                            key: "year".into(),
                            value: Some(MetadataValue {
                                value: Some(Value::RawString("2012".to_string()))
                            })
                        }))
                    })),
                }
                .and(PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::NotIn(NotIn {
                            key: "month".into(),
                            values: vec![
                                MetadataValue {
                                    value: Some(Value::RawString("december".to_string()))
                                },
                                MetadataValue {
                                    value: Some(Value::RawString("october".to_string()))
                                }
                            ]
                        }))
                    })),
                })
            ),
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
            execution_provider: Some(ExecutionProvider::TensorRt as i32)
        })]
    );
}

#[test]
fn test_drop_non_linear_algorithm_parse() {
    let input = r#"DROPNONLINEARALGORITHMINDEX (fake) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 42));
    let input = r#"DROPNONLINEARALGORITHMINDEX (kdtree) in 1234"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropNonLinearAlgorithmIndex(
            DropNonLinearAlgorithmIndex {
                store: "1234".to_string(),
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_not_exists: true,
            }
        )]
    );
    let input = r#"DROPNONLINEARALGORITHMINDEX IF EXISTS (kdtree) in 1234"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DropNonLinearAlgorithmIndex(
            DropNonLinearAlgorithmIndex {
                store: "1234".to_string(),
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_not_exists: false,
            }
        )]
    );
}

#[test]
fn test_get_pred_parse() {
    let input = r#"GETPRED ([a, b, c], [3.0, 4.0]) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 39));
    let input = r#"GETPRED ((firstname = king) OR (surname != charles)) in store2"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::GetPred(GetPred {
            store: "store2".to_string(),
            condition: Some(
                PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(Equals {
                            key: "firstname".into(),
                            value: Some(MetadataValue {
                                value: Some(Value::RawString("king".to_string()))
                            })
                        }))
                    })),
                }
                .or(PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(NotEquals {
                            key: "surname".into(),
                            value: Some(MetadataValue {
                                value: Some(Value::RawString("charles".to_string()))
                            })
                        }))
                    })),
                })
            ),
        })]
    );
    let input = r#"GETPRED ((pages in (0, 1, 2)) AND (author != dickens) OR (author NOT in (jk-rowlins, rick-riodan)) ) in bookshelf"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::GetPred(GetPred {
            store: "bookshelf".to_string(),
            condition: Some(
                PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::In(In {
                            key: "pages".into(),
                            values: vec![
                                MetadataValue {
                                    value: Some(Value::RawString("0".to_string()))
                                },
                                MetadataValue {
                                    value: Some(Value::RawString("1".to_string()))
                                },
                                MetadataValue {
                                    value: Some(Value::RawString("2".to_string()))
                                },
                            ]
                        }))
                    })),
                }
                .and(
                    PredicateCondition {
                        kind: Some(Kind::Value(Predicate {
                            kind: Some(PredicateKind::NotEquals(NotEquals {
                                key: "author".into(),
                                value: Some(MetadataValue {
                                    value: Some(Value::RawString("dickens".to_string()))
                                })
                            }))
                        })),
                    }
                    .or(PredicateCondition {
                        kind: Some(Kind::Value(Predicate {
                            kind: Some(PredicateKind::NotIn(NotIn {
                                key: "author".into(),
                                values: vec![
                                    MetadataValue {
                                        value: Some(Value::RawString("jk-rowlins".to_string()))
                                    },
                                    MetadataValue {
                                        value: Some(Value::RawString("rick-riodan".to_string()))
                                    }
                                ]
                            }))
                        })),
                    })
                )
            )
        })]
    );
}

#[test]
fn test_del_key_parse() {
    let input = r#"DELKEY ([a, b, c], [3.0, 4.0]) 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 35));
    let input = r#"DELKEY ([hi this is store input], [this should now get parsed too]) in 1234"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::DelKey(DelKey {
            store: "1234".to_string(),
            keys: vec![
                StoreInput {
                    value: Some(StoreValue::RawString("hi this is store input".to_string()))
                },
                StoreInput {
                    value: Some(StoreValue::RawString(
                        "this should now get parsed too".to_string()
                    ))
                },
            ],
        })]
    );
}

#[test]
fn test_set_in_store_parse() {
    let input = r#"set [help me here] in store"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 27));
    let input = r#"SET (([asdf], {upkeep: task})) in geo preprocessaction fakeaead"#;
    let DslError::UnexpectedSpan((start, end)) = parse_ai_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 63));
    let input = r#"SET (([This is the life of Haks paragraphed], {name: Haks, category: dev}), ([This is the life of Deven paragraphed], {name: Deven, category: dev})) in geo preprocessaction nopreprocessing"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::Set(Set {
            store: "geo".to_string(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(StoreValue::RawString(
                            "This is the life of Haks paragraphed".to_string()
                        ))
                    }),
                    value: Some(KvStoreValue {
                        value: HashMap::from_iter([
                            (
                                "name".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Haks".to_string()))
                                }
                            ),
                            (
                                "category".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("dev".to_string()))
                                }
                            ),
                        ])
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(StoreValue::RawString(
                            "This is the life of Deven paragraphed".to_string()
                        ))
                    }),
                    value: Some(KvStoreValue {
                        value: HashMap::from_iter([
                            (
                                "name".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Deven".to_string()))
                                }
                            ),
                            (
                                "category".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("dev".to_string()))
                                }
                            ),
                        ])
                    })
                }
            ],
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
            execution_provider: None,
        })]
    );
    let input = r#"SET (([This is the life of Haks paragraphed], {name: Haks, category: dev}), ([This is the life of Deven paragraphed], {name: Deven, category: dev})) in geo preprocessaction nopreprocessing executionprovider cuda"#;
    assert_eq!(
        parse_ai_query(input).expect("Could not parse query input"),
        vec![AiQuery::Set(Set {
            store: "geo".to_string(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(StoreValue::RawString(
                            "This is the life of Haks paragraphed".to_string()
                        ))
                    }),
                    value: Some(KvStoreValue {
                        value: HashMap::from_iter([
                            (
                                "name".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Haks".to_string()))
                                }
                            ),
                            (
                                "category".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("dev".to_string()))
                                }
                            ),
                        ])
                    })
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(StoreValue::RawString(
                            "This is the life of Deven paragraphed".to_string()
                        ))
                    }),
                    value: Some(KvStoreValue {
                        value: HashMap::from_iter([
                            (
                                "name".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Deven".to_string()))
                                }
                            ),
                            (
                                "category".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("dev".to_string()))
                                }
                            ),
                        ])
                    })
                }
            ],
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
            execution_provider: Some(ExecutionProvider::Cuda as i32),
        })]
    );
}
