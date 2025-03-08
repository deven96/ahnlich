use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::algorithm::algorithms::Algorithm;
use crate::metadata::metadata_value::Value;
use crate::metadata::{MetadataType, MetadataValue};
use crate::predicates::{Equals, In, NotEquals, NotIn, OrCondition, Predicate};
use crate::shared::info::StoreUpsert;
use crate::{algorithm::nonlinear::NonLinearAlgorithm, db::query as db_query};
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::query_builders::db as db_query_builders;
use ahnlich_types::similarity::{
    Algorithm as InternalAlgorithm, NonLinearAlgorithm as InternalNonLinearAlgorithm,
};

impl From<InternalNonLinearAlgorithm> for NonLinearAlgorithm {
    fn from(value: InternalNonLinearAlgorithm) -> Self {
        match value {
            InternalNonLinearAlgorithm::KDTree => Self::KdTree,
        }
    }
}

impl From<NonLinearAlgorithm> for InternalNonLinearAlgorithm {
    fn from(value: NonLinearAlgorithm) -> Self {
        match value {
            NonLinearAlgorithm::KdTree => Self::KDTree,
        }
    }
}

impl From<InternalAlgorithm> for Algorithm {
    fn from(value: InternalAlgorithm) -> Self {
        match value {
            InternalAlgorithm::DotProductSimilarity => Self::DotProductSimilarity,
            InternalAlgorithm::CosineSimilarity => Self::CosineSimilarity,
            InternalAlgorithm::EuclideanDistance => Self::EuclideanDistance,
            InternalAlgorithm::KDTree => Self::KdTree,
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

pub fn to_internal_create_store(
    params: db_query::CreateStore,
) -> db_query_builders::CreateStoreParams {
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

pub fn from_internal_create_store(
    params: db_query_builders::CreateStoreParams,
) -> db_query::CreateStore {
    db_query::CreateStore {
        store: params.store.0,
        create_predicates: params
            .create_predicates
            .into_iter()
            .map(|a| a.to_string())
            .collect(),
        dimension: params.dimension.get() as u32,
        non_linear_indices: params
            .non_linear_indices
            .into_iter()
            .filter_map(|a| NonLinearAlgorithm::from(a).try_into().ok())
            .collect(),
        error_if_exists: params.error_if_exists,
    }
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

pub fn to_grpc_predicate_condition(
    condition: ahnlich_types::predicate::PredicateCondition,
) -> Option<Box<crate::predicates::PredicateCondition>> {
    match condition {
        ahnlich_types::predicate::PredicateCondition::Or(left, right) => {
            Some(Box::new(crate::predicates::PredicateCondition {
                kind: Some(crate::predicates::predicate_condition::Kind::Or(Box::new(
                    OrCondition {
                        left: to_grpc_predicate_condition(*left),
                        right: to_grpc_predicate_condition(*right),
                    },
                ))),
            }))
        }
        ahnlich_types::predicate::PredicateCondition::And(left, right) => {
            Some(Box::new(crate::predicates::PredicateCondition {
                kind: Some(crate::predicates::predicate_condition::Kind::And(Box::new(
                    crate::predicates::AndCondition {
                        left: to_grpc_predicate_condition(*left),
                        right: to_grpc_predicate_condition(*right),
                    },
                ))),
            }))
        }
        ahnlich_types::predicate::PredicateCondition::Value(predicate) => match predicate {
            ahnlich_types::predicate::Predicate::Equals { key, value } => {
                Some(Box::new(crate::predicates::PredicateCondition {
                    kind: Some(crate::predicates::predicate_condition::Kind::Value(
                        Predicate {
                            kind: Some(crate::predicates::predicate::Kind::Equals(Equals {
                                key: key.to_string(),
                                value: Some(convert_from_internal_metadata(value)),
                            })),
                        },
                    )),
                }))
            }
            ahnlich_types::predicate::Predicate::NotEquals { key, value } => {
                Some(Box::new(crate::predicates::PredicateCondition {
                    kind: Some(crate::predicates::predicate_condition::Kind::Value(
                        Predicate {
                            kind: Some(crate::predicates::predicate::Kind::NotEquals(NotEquals {
                                key: key.to_string(),
                                value: Some(convert_from_internal_metadata(value)),
                            })),
                        },
                    )),
                }))
            }
            ahnlich_types::predicate::Predicate::In { key, value } => {
                Some(Box::new(crate::predicates::PredicateCondition {
                    kind: Some(crate::predicates::predicate_condition::Kind::Value(
                        Predicate {
                            kind: Some(crate::predicates::predicate::Kind::In(In {
                                key: key.to_string(),
                                values: value
                                    .into_iter()
                                    .map(convert_from_internal_metadata)
                                    .collect(),
                            })),
                        },
                    )),
                }))
            }
            ahnlich_types::predicate::Predicate::NotIn { key, value } => {
                Some(Box::new(crate::predicates::PredicateCondition {
                    kind: Some(crate::predicates::predicate_condition::Kind::Value(
                        Predicate {
                            kind: Some(crate::predicates::predicate::Kind::NotIn(NotIn {
                                key: key.to_string(),
                                values: value
                                    .into_iter()
                                    .map(convert_from_internal_metadata)
                                    .collect(),
                            })),
                        },
                    )),
                }))
            }
        },
    }
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

fn convert_from_internal_metadata(value: ahnlich_types::metadata::MetadataValue) -> MetadataValue {
    match value {
        ahnlich_types::metadata::MetadataValue::RawString(text) => crate::metadata::MetadataValue {
            r#type: MetadataType::RawString.into(),
            value: Some(Value::RawString(text)),
        },
        ahnlich_types::metadata::MetadataValue::Image(bin) => crate::metadata::MetadataValue {
            r#type: MetadataType::Image.into(),
            value: Some(Value::Image(bin)),
        },
    }
}

pub fn convert_to_nonzerousize(val: u64) -> Result<NonZeroUsize, tonic::Status> {
    match NonZeroUsize::try_from(val as usize) {
        Ok(value) => Ok(value),
        Err(_) => Err(tonic::Status::invalid_argument("Must be nonzero value")),
    }
}

pub static TRACE_HEADER: &str = "ahnlich-trace-id";

pub fn add_trace_parent<T>(req: &mut tonic::Request<T>, tracing_id: Option<String>) {
    if let Some(trace_parent) = tracing_id {
        req.metadata_mut().insert(
            TRACE_HEADER,
            trace_parent
                .parse()
                .expect("String keys should be valid ascii"),
        );
    };
}

impl StoreUpsert {
    pub fn modified(&self) -> bool {
        self.inserted + self.updated > 0
    }
}
