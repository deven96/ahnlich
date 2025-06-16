use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;

use crate::ai::models::AiStoreInputType;
use crate::client::ConnectedClient;
use crate::keyval::store_input::Value;
use crate::keyval::StoreInput;
use crate::metadata::MetadataValue;
use crate::predicates::{AndCondition, Equals, In, NotEquals, NotIn, OrCondition};
use crate::shared::info::StoreUpsert;

impl TryFrom<StoreInput> for MetadataValue {
    type Error = String;
    fn try_from(input: StoreInput) -> Result<Self, Self::Error> {
        let inner = input.value.ok_or_else(|| "Conversion failed".to_string())?;
        match inner {
            Value::Image(binary) => Ok(MetadataValue {
                value: Some(crate::metadata::metadata_value::Value::Image(binary)),
            }),

            Value::RawString(text) => Ok(MetadataValue {
                value: Some(crate::metadata::metadata_value::Value::RawString(text)),
            }),
        }
    }
}

impl TryFrom<MetadataValue> for StoreInput {
    type Error = String;
    fn try_from(input: MetadataValue) -> Result<Self, Self::Error> {
        let inner = input.value.ok_or_else(|| "Conversion failed".to_string())?;
        match inner {
            crate::metadata::metadata_value::Value::Image(binary) => Ok(StoreInput {
                value: Some(Value::Image(binary)),
            }),
            crate::metadata::metadata_value::Value::RawString(text) => Ok(StoreInput {
                value: Some(Value::RawString(text)),
            }),
        }
    }
}

impl TryFrom<&StoreInput> for AiStoreInputType {
    type Error = ();

    fn try_from(value: &StoreInput) -> Result<Self, Self::Error> {
        let value = value.value.as_ref().ok_or(())?;
        Ok(match value {
            Value::RawString(_) => Self::RawString,
            Value::Image(_) => Self::Image,
        })
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

pub fn convert_to_nonzerousize(val: u64) -> Result<NonZeroUsize, String> {
    match NonZeroUsize::try_from(val as usize) {
        Ok(value) => Ok(value),
        Err(_) => Err("Must be nonzero value".into()),
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

impl crate::predicates::PredicateCondition {
    pub fn and(self, other: crate::predicates::PredicateCondition) -> Self {
        Self {
            kind: Some(crate::predicates::predicate_condition::Kind::And(Box::new(
                AndCondition {
                    left: Some(Box::new(self)),
                    right: Some(Box::new(other)),
                },
            ))),
        }
    }

    pub fn or(self, other: crate::predicates::PredicateCondition) -> Self {
        Self {
            kind: Some(crate::predicates::predicate_condition::Kind::Or(Box::new(
                OrCondition {
                    left: Some(Box::new(self)),
                    right: Some(Box::new(other)),
                },
            ))),
        }
    }
}

impl crate::predicates::Predicate {
    pub fn get_key(&self) -> &String {
        match &self.kind {
            Some(crate::predicates::predicate::Kind::Equals(Equals { key, .. })) => key,

            Some(crate::predicates::predicate::Kind::NotEquals(NotEquals { key, .. })) => key,
            Some(crate::predicates::predicate::Kind::In(In { key, .. })) => key,
            Some(crate::predicates::predicate::Kind::NotIn(NotIn { key, .. })) => key,
            None => unreachable!(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for crate::keyval::StoreKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<f32>::deserialize(deserializer)?;
        Ok(crate::keyval::StoreKey { key: vec })
    }
}

impl serde::Serialize for crate::keyval::StoreKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.key.serialize(serializer)
    }
}

// NOTE: ConnectedClient should be unique purely by address assuming we are not doing any TCP magic
// to allow port reuse
impl Hash for ConnectedClient {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.address.hash(state)
    }
}
