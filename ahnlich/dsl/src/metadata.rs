use crate::array::parse_f32_array;
use crate::error::DslError;
use crate::parser::Rule;
use ahnlich_types::keyval::{StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataKey, MetadataValue};
use pest::iterators::Pair;
use std::collections::{HashMap, HashSet};

pub(crate) fn parse_metadata_value(pair: Pair<Rule>) -> Result<MetadataValue, DslError> {
    match pair.as_rule() {
        Rule::raw_string => {
            let value = pair.as_str().to_string();
            Ok(MetadataValue::RawString(value))
        }
        Rule::image => {
            let hex_str = pair.as_str();
            match hex::decode(hex_str.strip_prefix("/x").ok_or(DslError::UnexpectedHex(
                "Image representation must have prefix /x".to_string(),
            ))?) {
                Ok(bytes) => Ok(MetadataValue::Image(bytes)),
                Err(_) => Err(DslError::UnexpectedHex(hex_str.to_string())),
            }
        }
        _ => Err(DslError::UnexpectedSpan((
            pair.as_span().start_pos().pos(),
            pair.as_span().end_pos().pos(),
        ))),
    }
}

pub(crate) fn parse_metadata_values(pair: Pair<Rule>) -> Result<HashSet<MetadataValue>, DslError> {
    let mut values = HashSet::new();
    for value_pair in pair.into_inner() {
        let metadata_value = parse_metadata_value(value_pair)?;
        values.insert(metadata_value);
    }
    Ok(values)
}

fn parse_into_store_key_and_value(pair: Pair<Rule>) -> Result<(StoreKey, StoreValue), DslError> {
    let start_pos = pair.as_span().start_pos().pos();
    let end_pos = pair.as_span().end_pos().pos();

    let mut inner_pairs = pair.into_inner();
    let f32_array = parse_f32_array(
        inner_pairs
            .next()
            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
    );
    let store_value = inner_pairs
        .next()
        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
    let mut store_value_map = HashMap::new();
    for store_value_single in store_value.into_inner() {
        let start_pos = store_value_single.as_span().start_pos().pos();
        let end_pos = store_value_single.as_span().end_pos().pos();
        let mut v = store_value_single.into_inner();
        let key = MetadataKey::new(
            v.next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string(),
        );
        let value = parse_metadata_value(
            v.next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
        )?;
        store_value_map.insert(key, value);
    }
    Ok((f32_array, store_value_map))
}

pub(crate) fn parse_store_keys_to_store_value(
    pair: Pair<Rule>,
) -> Result<Vec<(StoreKey, StoreValue)>, DslError> {
    let mut values = vec![];
    for value_pair in pair.into_inner() {
        values.push(parse_into_store_key_and_value(value_pair)?);
    }
    Ok(values)
}
