use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::algorithm::algorithms::Algorithm;
use crate::metadata::MetadataValue;
use crate::{algorithm::nonlinear::NonLinearAlgorithm, db::query as db_query};
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::query_builders::db as db_query_builders;
use ahnlich_types::similarity::{
    Algorithm as InternalAlgorithm, NonLinearAlgorithm as InternalNonLinearAlgorithm,
};

impl From<NonLinearAlgorithm> for InternalNonLinearAlgorithm {
    fn from(value: NonLinearAlgorithm) -> Self {
        match value {
            NonLinearAlgorithm::KdTree => Self::KDTree,
        }
    }
}

impl From<Algorithm> for InternalAlgorithm {
    fn from(value: Algorithm) -> Self {
        match value {
            Algorithm::DotProductSimilarity => Self::DotProductSimilarity,
            Algorithm::CosineSimilarity => Self::CosineSimilarity,
            Algorithm::EuclideanDistance => Self::EuclideanDistance,
            Algorithm::KdTree => Self::KDTree,
        }
    }
}

//FIXME: Validation for Nonzerosize so it doesn't panic and also have this function return a GRPC
//error ``InvalidArgument``?

pub fn db_create_store(params: db_query::CreateStore) -> db_query_builders::CreateStoreParams {
    // convert

    db_query_builders::CreateStoreParams::builder()
        .store(params.store)
        .create_predicates(HashSet::from_iter(
            params.create_predicates.into_iter().map(MetadataKey::new),
        ))
        .dimension(params.dimension as usize)
        .non_linear_indices(HashSet::from_iter(
            params
                .non_linear_indices
                .into_iter()
                .filter_map(|index| NonLinearAlgorithm::try_from(index).ok())
                .map(|algo| algo.into()),
        ))
        .error_if_exists(params.error_if_exists)
        .build()
}

#[macro_export]
macro_rules! unwrap_or_invalid {
    ($opt:expr, $msg:expr) => {
        match $opt {
            Some(value) => value,
            None => return Err(tonic::Status::invalid_argument($msg)),
        }
    };
}

pub fn unwrap_predicate_condition(
    condition: Option<Box<crate::predicates::PredicateCondition>>,
) -> Result<ahnlich_types::predicate::PredicateCondition, tonic::Status> {
    let condition = unwrap_or_invalid!(condition, "predicate condition is required");
    let kind = unwrap_or_invalid!(condition.kind, "condition is required");

    match kind {
        crate::predicates::predicate_condition::Kind::Or(val) => {
            let left = unwrap_predicate_condition(val.left)?;
            let right = unwrap_predicate_condition(val.right)?;

            Ok(ahnlich_types::predicate::PredicateCondition::or(
                left, right,
            ))
        }

        crate::predicates::predicate_condition::Kind::And(val) => {
            let left = unwrap_predicate_condition(val.left)?;
            let right = unwrap_predicate_condition(val.right)?;

            Ok(ahnlich_types::predicate::PredicateCondition::and(
                left, right,
            ))
        }

        crate::predicates::predicate_condition::Kind::Value(val) => {
            let predicate_kind = unwrap_or_invalid!(val.kind, "predicate is required");
            let pred = match predicate_kind {
                crate::predicates::predicate::Kind::In(pred) => {
                    let key = ahnlich_types::metadata::MetadataKey::new(pred.key);
                    let value = pred
                        .values
                        .into_iter()
                        .map(convert_to_internal_metadatavalue)
                        .collect::<Result<_, _>>()?;
                    ahnlich_types::predicate::Predicate::In { key, value }
                }

                crate::predicates::predicate::Kind::NotIn(pred) => {
                    let key = ahnlich_types::metadata::MetadataKey::new(pred.key);
                    let value = pred
                        .values
                        .into_iter()
                        .map(convert_to_internal_metadatavalue)
                        .collect::<Result<_, _>>()?;
                    ahnlich_types::predicate::Predicate::NotIn { key, value }
                }

                crate::predicates::predicate::Kind::Equals(pred) => {
                    let key = ahnlich_types::metadata::MetadataKey::new(pred.key);

                    let value = convert_to_internal_metadatavalue(unwrap_or_invalid!(
                        pred.value,
                        "metadata value is required"
                    ))?;

                    ahnlich_types::predicate::Predicate::Equals { key, value }
                }

                crate::predicates::predicate::Kind::NotEquals(pred) => {
                    let key = ahnlich_types::metadata::MetadataKey::new(pred.key);

                    let value = convert_to_internal_metadatavalue(unwrap_or_invalid!(
                        pred.value,
                        "metadata value is required"
                    ))?;

                    ahnlich_types::predicate::Predicate::NotEquals { key, value }
                }
            };

            Ok(PredicateCondition::Value(pred))
        }
    }
}

fn convert_to_internal_metadatavalue(
    value: MetadataValue,
) -> Result<ahnlich_types::metadata::MetadataValue, tonic::Status> {
    Ok(
        match unwrap_or_invalid!(value.value, "metadata value is required") {
            crate::metadata::metadata_value::Value::Image(bin) => {
                ahnlich_types::metadata::MetadataValue::Image(bin)
            }

            crate::metadata::metadata_value::Value::RawString(text) => {
                ahnlich_types::metadata::MetadataValue::RawString(text)
            }
        },
    )
}

pub fn convert_to_nonzerousize(val: u64) -> Result<NonZeroUsize, tonic::Status> {
    match NonZeroUsize::try_from(val as usize) {
        Ok(value) => Ok(value),
        Err(_) => Err(tonic::Status::invalid_argument("Must be nonzero value")),
    }
}
