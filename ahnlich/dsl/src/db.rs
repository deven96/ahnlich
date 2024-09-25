use ahnlich_types::{db::DBQuery, keyval::StoreName};
use pest::Parser;
use pest_derive::Parser;

use crate::error::DslError;

#[derive(Parser)]
#[grammar = "syntax/db.pest"]
struct DBQueryParser;

pub fn parse_db_query(input: &str) -> Result<Vec<DBQuery>, DslError> {
    let pairs = DBQueryParser::parse(Rule::query, input).map_err(Box::new)?;
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
}
