use crate::parser::{QueryParser, Rule};
use ahnlich_types::{
    db::DBQuery,
    keyval::{StoreKey, StoreName},
    metadata::MetadataKey,
    similarity::NonLinearAlgorithm,
};
use ndarray::Array1;
use pest::iterators::Pair;
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

fn to_non_linear(input: &str) -> Option<NonLinearAlgorithm> {
    match input.to_lowercase().trim() {
        "kdtree" => Some(NonLinearAlgorithm::KDTree),
        _ => None,
    }
}

fn parse_multi_f32_array(f32_arrays_pair: Pair<Rule>) -> Vec<StoreKey> {
    f32_arrays_pair.into_inner().map(parse_f32_array).collect()
}

fn parse_f32_array(pair: Pair<Rule>) -> StoreKey {
    StoreKey(Array1::from_iter(pair.into_inner().map(|f32_pair| {
        f32_pair
            .as_str()
            .parse::<f32>()
            .expect("Cannot parse single f32 num")
    })))
}

// Parse raw strings separated by ; into a Vec<DBQuery>. Examples include but are not restricted
// to
//
// PING
// LISTCLIENTS
// LISTSTORES
// INFOSERVER
// DROPSTORE store_name IF EXISTS
// CREATEPREDINDEX (key_1, key_2) in store_name
// DROPPREDINDEX IF EXISTS (key1, key2) in store_name
// CREATENONLINEARALGORITHMINDEX (kdtree) in store_name
// DROPNONLINEARALGORITHMINDEX IF EXISTS (kdtree) in store_name
// GETKEY ((1.0, 2.0), (3.0, 4.0)) IN my_store
// DELKEY ((1.2, 3.0), (5.6, 7.8)) IN my_store
// GETPRED ((author = dickens) OR (country != Nigeria)) IN my_store
//
// #TODO
// SET
// CREATESTORE
// GETSIMN
pub fn parse_db_query(input: &str) -> Result<Vec<DBQuery>, DslError> {
    let pairs = QueryParser::parse(Rule::db_query, input).map_err(Box::new)?;
    let statements = pairs.into_iter().collect::<Vec<_>>();
    let mut queries = Vec::with_capacity(statements.len());
    for statement in statements {
        let start_pos = statement.as_span().start_pos().pos();
        let end_pos = statement.as_span().end_pos().pos();
        let query = match statement.as_rule() {
            Rule::ping => DBQuery::Ping,
            Rule::list_clients => DBQuery::ListClients,
            Rule::list_stores => DBQuery::ListStores,
            Rule::info_server => DBQuery::InfoServer,
            Rule::get_pred => {
                let mut inner_pairs = statement.into_inner();
                let predicate_conditions = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::GetPred {
                    store: StoreName(store.to_string()),
                    condition: parse_predicate_expression(predicate_conditions)?,
                }
            }
            Rule::get_key => {
                let mut inner_pairs = statement.into_inner();
                let f32_arrays_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_multi_f32_array(f32_arrays_pair);

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::GetKey {
                    store: StoreName(store.to_string()),
                    keys,
                }
            }
            Rule::del_key => {
                let mut inner_pairs = statement.into_inner();
                let f32_arrays_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_multi_f32_array(f32_arrays_pair);

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::DelKey {
                    store: StoreName(store.to_string()),
                    keys,
                }
            }
            Rule::create_non_linear_algorithm_index => {
                let mut inner_pairs = statement.into_inner();
                let index_name_pairs = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let non_linear_indices = index_name_pairs
                    .into_inner()
                    .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                    .collect();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::CreateNonLinearAlgorithmIndex {
                    store: StoreName(store.to_string()),
                    non_linear_indices,
                }
            }
            Rule::create_pred_index => {
                let mut inner_pairs = statement.into_inner();
                let index_name_pairs = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let predicates = index_name_pairs
                    .into_inner()
                    .map(|index_pair| MetadataKey::new(index_pair.as_str().to_string()))
                    .collect();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::CreatePredIndex {
                    store: StoreName(store.to_string()),
                    predicates,
                }
            }
            Rule::drop_non_linear_algorithm_index => {
                let mut inner_pairs = statement.into_inner().peekable();
                let mut if_exists = false;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::if_exists {
                        inner_pairs.next(); // Consume rule
                        if_exists = true;
                    }
                };
                let index_names_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let non_linear_indices = index_names_pair
                    .into_inner()
                    .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                    .collect();
                DBQuery::DropNonLinearAlgorithmIndex {
                    store: StoreName(store.to_string()),
                    non_linear_indices,
                    error_if_not_exists: !if_exists,
                }
            }
            Rule::drop_pred_index => {
                let mut inner_pairs = statement.into_inner().peekable();
                let mut if_exists = false;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::if_exists {
                        inner_pairs.next();
                        if_exists = true;
                    }
                };
                let index_names_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let predicates = index_names_pair
                    .into_inner()
                    .map(|index_pair| MetadataKey::new(index_pair.as_str().to_string()))
                    .collect();
                DBQuery::DropPredIndex {
                    store: StoreName(store.to_string()),
                    predicates,
                    error_if_not_exists: !if_exists,
                }
            }
            Rule::drop_store => {
                let mut inner_pairs = statement.into_inner();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let if_exists = match inner_pairs.next() {
                    None => false,
                    Some(p) => {
                        if p.as_str().trim().to_lowercase() != "if exists" {
                            let start_pos = p.as_span().start_pos().pos();
                            let end_pos = p.as_span().end_pos().pos();
                            return Err(DslError::UnexpectedSpan((start_pos, end_pos)));
                        }
                        true
                    }
                };
                DBQuery::DropStore {
                    store: StoreName(store.to_string()),
                    error_if_not_exists: !if_exists,
                }
            }
            _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
        };
        queries.push(query);
    }
    Ok(queries)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use ahnlich_types::{
        metadata::MetadataValue,
        predicate::{Predicate, PredicateCondition},
    };

    use super::*;

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
        let input = r#"getkey ((a, b, c), (3.0, 4.0)) in 1234"#;
        let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
            panic!("Unexpected error pattern found")
        };
        assert_eq!((start, end), (0, 38));
        let input = r#"getkey ((1, 2, 3), (3.0, 4.0)) in 1234"#;
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
        let input = r#"DELKEY ((a, b, c), (3.0, 4.0)) in 1234"#;
        let DslError::UnexpectedSpan((start, end)) = parse_db_query(input).unwrap_err() else {
            panic!("Unexpected error pattern found")
        };
        assert_eq!((start, end), (0, 38));
        let input = r#"DELKEY ((1, 2, 3), (3.0, 4.0)) in 1234"#;
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
        let input = r#"GETPRED ((a, b, c), (3.0, 4.0)) in 1234"#;
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
}
