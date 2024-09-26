use crate::error::DslError;
use crate::parser::Rule;
use ahnlich_types::metadata::MetadataValue;
use pest::iterators::Pair;
use std::collections::HashSet;

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
