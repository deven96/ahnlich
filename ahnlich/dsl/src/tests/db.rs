use crate::error::DslError;
use ahnlich_types::{
    algorithm::{
        algorithms::Algorithm,
        nonlinear::{KdTreeConfig, NonLinearAlgorithm, NonLinearIndex, non_linear_index},
    },
    db::{
        pipeline::db_query::Query as DBQuery,
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetKey, GetPred, GetSimN,
            GetStore, InfoServer, ListClients, ListStores, Ping, Set,
        },
    },
    keyval::{DbStoreEntry, StoreKey, StoreValue},
    metadata::{MetadataValue, metadata_value::Value},
    predicates::{
        Equals, In, NotEquals, NotIn, Predicate, PredicateCondition,
        predicate::Kind as PredicateKind, predicate_condition::Kind,
    },
};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use crate::db::parse_db_query;

#[test]
fn test_single_query_parse() {
    let input = r#"LISTCLIENTS"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::ListClients(ListClients {})]
    );
    let input = r#"listclients"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::ListClients(ListClients {})]
    );
    let input = r#"  Ping  "#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::Ping(Ping {})]
    );
}

#[test]
fn test_multi_query_parse() {
    let input = r#" INFOSERVER ; listSTORES;"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![
            DBQuery::InfoServer(InfoServer {}),
            DBQuery::ListStores(ListStores {})
        ]
    );
}

#[test]
fn test_no_valid_input_in_query() {
    let input = r#" random ; listSTORES;"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 8));
    let input = r#" INfoSERVER ; random; ping"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (13, 20));
}

#[test]
fn test_create_store_parse() {
    // simple syntax
    let input = r#"CREATEstore storename DIMENSION 23"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore(CreateStore {
            store: "storename".to_string(),
            dimension: 23,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true
        })]
    );
    let input = r#"CREATEstore IF NOT EXISTS testing DIMENSION 43"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore(CreateStore {
            store: "testing".to_string(),
            dimension: 43,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: false
        })]
    );
    let input = r#"CREATEstore IF NOT EXISTS school DIMENSION 39 PREDICATES (department, faculty)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore(CreateStore {
            store: "school".to_string(),
            dimension: 39,
            create_predicates: vec!["department".to_string(), "faculty".to_string()],
            non_linear_indices: vec![],
            error_if_exists: false
        })]
    );
    let input = r#"CREATEstore school DIMENSION 39 NONLINEARALGORITHMINDEX (kdtree)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore(CreateStore {
            store: "school".to_string(),
            dimension: 39,
            create_predicates: vec![],
            non_linear_indices: vec![NonLinearIndex {
                index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
            }],
            error_if_exists: true
        })]
    );
    let input = r#"CREATEstore school DIMENSION 77 PREDICATES(name, surname) NONLINEARALGORITHMINDEX (kdtree)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore(CreateStore {
            store: "school".to_string(),
            dimension: 77,
            create_predicates: vec!["name".to_string(), "surname".to_string(),],
            non_linear_indices: vec![NonLinearIndex {
                index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
            }],
            error_if_exists: true
        })]
    );
}

#[test]
fn test_drop_store_parse() {
    let input = r#"DROPSTORE random"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropStore(DropStore {
            store: "random".to_string(),
            error_if_not_exists: true
        })]
    );
    let input = r#"dropstore yeezy_store IF exists"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropStore(DropStore {
            store: "yeezy_store".to_string(),
            error_if_not_exists: false,
        })]
    );
    let input = r#"dropstore yeezy IF NOT exists"#;
    // IF NOT EXISTS is not valid syntax
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (15, 29));
}

#[test]
fn test_create_predicate_index_parse() {
    let input = r#"CREATEPREDINDEX (one, two, 3) in tapHstore1"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreatePredIndex(CreatePredIndex {
            store: "tapHstore1".to_string(),
            predicates: vec!["one".to_string(), "two".to_string(), "3".to_string(),]
        })]
    );
}

#[test]
fn test_drop_pred_index_parse() {
    let input = r#"DROPPREDINDEX (here, th2) in store2"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropPredIndex(DropPredIndex {
            store: "store2".to_string(),
            predicates: vec!["here".to_string(), "th2".to_string()],
            error_if_not_exists: true,
        })]
    );
    let input = r#"DROPPREDINDEX IF EXISTS (off) in storememe"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropPredIndex(DropPredIndex {
            store: "storememe".to_string(),
            predicates: vec!["off".to_string()],
            error_if_not_exists: false,
        })]
    );
}

#[test]
fn test_create_non_linear_algorithm_parse() {
    let input = r#"createnonlinearalgorithmindex (fake) in store2"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 46));
    let input = r#"createnonlinearalgorithmindex (kdtree) in store2"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateNonLinearAlgorithmIndex(
            CreateNonLinearAlgorithmIndex {
                store: "store2".to_string(),
                non_linear_indices: vec![NonLinearIndex {
                    index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                }],
            }
        )]
    );
}

#[test]
fn test_get_sim_n_parse() {
    // passing in a zero value to ensure it doesn't work
    let input = r#"GETSIMN 0 with [0.1, 0.2] using kdtree in store1 where ( author = hi )"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 70));
    let input = r#"GETSIMN 5 with [34.1, 72.2] using cosinesimilarity in random"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetSimN(GetSimN {
            store: "random".to_string(),
            search_input: Some(StoreKey {
                key: vec![34.1, 72.2]
            }),
            closest_n: 5,
            algorithm: Algorithm::CosineSimilarity as i32,
            condition: None
        })]
    );
    let input = r#"GETSIMN 8 with [3.7, 9.6] using euclideandistance in other where ((year != 2012) AND (month not in (december, october)))"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetSimN(GetSimN {
            store: "other".to_string(),
            search_input: Some(StoreKey {
                key: vec![3.7, 9.6]
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
        })]
    );
}

#[test]
fn test_drop_non_linear_algorithm_parse() {
    let input = r#"DROPNONLINEARALGORITHMINDEX (fake) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 42));
    let input = r#"DROPNONLINEARALGORITHMINDEX (kdtree) in 1234"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropNonLinearAlgorithmIndex(
            DropNonLinearAlgorithmIndex {
                store: "1234".to_string(),
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_not_exists: true,
            }
        )]
    );
    let input = r#"DROPNONLINEARALGORITHMINDEX IF EXISTS (kdtree) in 12345"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropNonLinearAlgorithmIndex(
            DropNonLinearAlgorithmIndex {
                store: "12345".to_string(),
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_not_exists: false,
            }
        )]
    );
}

#[test]
fn test_get_key_parse() {
    let input = r#"getkey ([a, b, c], [3.0, 4.0]) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 38));
    let input = r#"getkey ([1, 2, 3], [3.0, 4.0]) in 1234"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetKey(GetKey {
            store: "1234".to_string(),
            keys: vec![
                StoreKey {
                    key: vec![1.0, 2.0, 3.0]
                },
                StoreKey {
                    key: vec![3.0, 4.0]
                },
            ],
        })]
    );
}

#[test]
fn test_set_in_store_parse() {
    let input = r#"set 2134 in store"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 17));
    let input = r#"SET (([1,2,3], {state: Munich, country: Germany}), ([3.2, 4.5, 9.4], {country: USA})) in geo"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::Set(Set {
            store: "geo".to_string(),
            inputs: vec![
                DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.0, 2.0, 3.0]
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([
                            (
                                "state".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Munich".to_string()))
                                }
                            ),
                            (
                                "country".to_string(),
                                MetadataValue {
                                    value: Some(Value::RawString("Germany".to_string()))
                                }
                            ),
                        ])
                    })
                },
                DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![3.2, 4.5, 9.4]
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "country".to_string(),
                            MetadataValue {
                                value: Some(Value::RawString("USA".to_string()))
                            }
                        ),])
                    })
                },
            ],
        })]
    );
}

#[test]
fn test_del_key_parse() {
    let input = r#"DELKEY ([a, b, c], [3.0, 4.0]) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 38));
    let input = r#"DELKEY ([1, 2, 3], [3.0, 4.0]) in 1234"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DelKey(DelKey {
            store: "1234".to_string(),
            keys: vec![
                StoreKey {
                    key: vec![1.0, 2.0, 3.0]
                },
                StoreKey {
                    key: vec![3.0, 4.0]
                },
            ],
        })]
    );
}

#[test]
fn test_get_pred_parse() {
    let input = r#"GETPRED ([a, b, c], [3.0, 4.0]) in 1234"#;
    let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
        panic!("Unexpected error pattern found")
    };
    assert_eq!((start, end), (0, 39));
    let input = r#"GETPRED ((firstname = king) OR (surname != charles)) in store2"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetPred(GetPred {
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
    let input = r#"getstore my_store"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetStore(GetStore {
            store: "my_store".to_string(),
        })]
    );
    let input = r#"GETSTORE TestStore123"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetStore(GetStore {
            store: "TestStore123".to_string(),
        })]
    );
    let input = r#"GETPRED ((pages in (0, 1, 2)) AND (author != dickens) OR (author NOT in (jk-rowlins, rick-riodan)) ) in bookshelf"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetPred(GetPred {
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
