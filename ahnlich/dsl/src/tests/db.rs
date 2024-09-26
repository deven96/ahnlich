use crate::error::DslError;
use ahnlich_types::{
    db::DBQuery,
    keyval::{StoreKey, StoreName},
    metadata::MetadataKey,
};
use ndarray::Array1;
use std::{collections::HashSet, num::NonZeroUsize};

use ahnlich_types::{
    metadata::MetadataValue,
    predicate::{Predicate, PredicateCondition},
    similarity::{Algorithm, NonLinearAlgorithm},
};

use crate::db::parse_db_query;

#[test]
fn test_single_query_parse() {
    let input = r#"LISTCLIENTS"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::ListClients]
    );
    let input = r#"listclients"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::ListClients]
    );
    let input = r#"  Ping  "#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::Ping]
    );
}

#[test]
fn test_multi_query_parse() {
    let input = r#" INFOSERVER ; listSTORES;"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::InfoServer, DBQuery::ListStores]
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
        vec![DBQuery::CreateStore {
            store: StoreName("storename".to_string()),
            dimension: NonZeroUsize::new(23).unwrap(),
            create_predicates: HashSet::new(),
            non_linear_indices: HashSet::new(),
            error_if_exists: true
        }]
    );
    let input = r#"CREATEstore IF NOT EXISTS testing DIMENSION 43"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore {
            store: StoreName("testing".to_string()),
            dimension: NonZeroUsize::new(43).unwrap(),
            create_predicates: HashSet::new(),
            non_linear_indices: HashSet::new(),
            error_if_exists: false
        }]
    );
    let input = r#"CREATEstore IF NOT EXISTS school DIMENSION 39 PREDICATES (department, faculty)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore {
            store: StoreName("school".to_string()),
            dimension: NonZeroUsize::new(39).unwrap(),
            create_predicates: HashSet::from_iter([
                MetadataKey::new("department".to_string()),
                MetadataKey::new("faculty".to_string()),
            ]),
            non_linear_indices: HashSet::new(),
            error_if_exists: false
        }]
    );
    let input = r#"CREATEstore school DIMENSION 39 NONLINEARALGORITHMINDEX (kdtree)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore {
            store: StoreName("school".to_string()),
            dimension: NonZeroUsize::new(39).unwrap(),
            create_predicates: HashSet::new(),
            non_linear_indices: HashSet::from_iter([NonLinearAlgorithm::KDTree]),
            error_if_exists: true
        }]
    );
    let input = r#"CREATEstore school DIMENSION 77 PREDICATES(name, surname) NONLINEARALGORITHMINDEX (kdtree)"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::CreateStore {
            store: StoreName("school".to_string()),
            dimension: NonZeroUsize::new(77).unwrap(),
            create_predicates: HashSet::from_iter([
                MetadataKey::new("name".to_string()),
                MetadataKey::new("surname".to_string()),
            ]),
            non_linear_indices: HashSet::from_iter([NonLinearAlgorithm::KDTree]),
            error_if_exists: true
        }]
    );
}

#[test]
fn test_drop_store_parse() {
    let input = r#"DROPSTORE random"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropStore {
            store: StoreName("random".to_string()),
            error_if_not_exists: true
        }]
    );
    let input = r#"dropstore yeezy_store IF exists"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropStore {
            store: StoreName("yeezy_store".to_string()),
            error_if_not_exists: false,
        }]
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
        vec![DBQuery::CreatePredIndex {
            store: StoreName("tapHstore1".to_string()),
            predicates: HashSet::from_iter([
                MetadataKey::new("one".to_string()),
                MetadataKey::new("two".to_string()),
                MetadataKey::new("3".to_string()),
            ])
        }]
    );
}

#[test]
fn test_drop_pred_index_parse() {
    let input = r#"DROPPREDINDEX (here, th2) in store2"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropPredIndex {
            store: StoreName("store2".to_string()),
            predicates: HashSet::from_iter([
                MetadataKey::new("here".to_string()),
                MetadataKey::new("th2".to_string()),
            ]),
            error_if_not_exists: true,
        }]
    );
    let input = r#"DROPPREDINDEX IF EXISTS (off) in storememe"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropPredIndex {
            store: StoreName("storememe".to_string()),
            predicates: HashSet::from_iter([MetadataKey::new("off".to_string()),]),
            error_if_not_exists: false,
        }]
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
        vec![DBQuery::CreateNonLinearAlgorithmIndex {
            store: StoreName("store2".to_string()),
            non_linear_indices: HashSet::from_iter([NonLinearAlgorithm::KDTree]),
        }]
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
        vec![DBQuery::GetSimN {
            store: StoreName("random".to_string()),
            search_input: StoreKey(Array1::from_iter([34.1, 72.2])),
            closest_n: NonZeroUsize::new(5).unwrap(),
            algorithm: Algorithm::CosineSimilarity,
            condition: None
        }]
    );
    let input = r#"GETSIMN 8 with [3.7, 9.6] using euclideandistance in other where ((year != 2012) AND (month not in (december, october)))"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetSimN {
            store: StoreName("other".to_string()),
            search_input: StoreKey(Array1::from_iter([3.7, 9.6])),
            closest_n: NonZeroUsize::new(8).unwrap(),
            algorithm: Algorithm::EuclideanDistance,
            condition: Some(
                PredicateCondition::Value(Predicate::NotEquals {
                    key: MetadataKey::new("year".into()),
                    value: MetadataValue::RawString("2012".to_string())
                })
                .and(PredicateCondition::Value(Predicate::NotIn {
                    key: MetadataKey::new("month".into()),
                    value: HashSet::from_iter([
                        MetadataValue::RawString("december".to_string()),
                        MetadataValue::RawString("october".to_string()),
                    ]),
                }))
            ),
        }]
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
        vec![DBQuery::DropNonLinearAlgorithmIndex {
            store: StoreName("1234".to_string()),
            non_linear_indices: HashSet::from_iter([NonLinearAlgorithm::KDTree]),
            error_if_not_exists: true,
        }]
    );
    let input = r#"DROPNONLINEARALGORITHMINDEX IF EXISTS (kdtree) in 1234"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::DropNonLinearAlgorithmIndex {
            store: StoreName("1234".to_string()),
            non_linear_indices: HashSet::from_iter([NonLinearAlgorithm::KDTree]),
            error_if_not_exists: false,
        }]
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
        vec![DBQuery::GetKey {
            store: StoreName("1234".to_string()),
            keys: vec![
                StoreKey(Array1::from_iter([1.0, 2.0, 3.0])),
                StoreKey(Array1::from_iter([3.0, 4.0])),
            ],
        }]
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
        vec![DBQuery::DelKey {
            store: StoreName("1234".to_string()),
            keys: vec![
                StoreKey(Array1::from_iter([1.0, 2.0, 3.0])),
                StoreKey(Array1::from_iter([3.0, 4.0])),
            ],
        }]
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
        vec![DBQuery::GetPred {
            store: StoreName("store2".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("firstname".into()),
                value: MetadataValue::RawString("king".to_string())
            })
            .or(PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("surname".into()),
                value: MetadataValue::RawString("charles".to_string())
            })),
        }]
    );
    let input = r#"GETPRED ((pages in (0, 1, 2)) AND (author != dickens) OR (author NOT in (jk-rowlins, rick-riodan)) ) in bookshelf"#;
    assert_eq!(
        parse_db_query(input).expect("Could not parse query input"),
        vec![DBQuery::GetPred {
            store: StoreName("bookshelf".to_string()),
            condition: PredicateCondition::Value(Predicate::In {
                key: MetadataKey::new("pages".into()),
                value: HashSet::from_iter([
                    MetadataValue::RawString("0".to_string()),
                    MetadataValue::RawString("1".to_string()),
                    MetadataValue::RawString("2".to_string()),
                ]),
            })
            .and(
                PredicateCondition::Value(Predicate::NotEquals {
                    key: MetadataKey::new("author".into()),
                    value: MetadataValue::RawString("dickens".to_string())
                })
                .or(PredicateCondition::Value(Predicate::NotIn {
                    key: MetadataKey::new("author".into()),
                    value: HashSet::from_iter([
                        MetadataValue::RawString("jk-rowlins".to_string()),
                        MetadataValue::RawString("rick-riodan".to_string()),
                    ]),
                }))
            )
        }]
    );
}
