use serde_reflection::Registry;
use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use types::predicate::PredicateCondition;
use types::predicate::PredicateOp;
use types::similarity::Algorithm;
use types::{
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
    query::Query,
};

pub fn trace_query_enum() -> Registry {
    let input_arr_1 = ndarray::array![0.1, 0.2, 0.3, 0.4, 0.5];
    let store_key = StoreKey(input_arr_1.clone());

    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();

    let sample_store_name = StoreName("ijdfsdf".into());
    let sample_store_keys = vec![store_key.clone()];
    let test_predicate_condition = &PredicateCondition::Value(types::predicate::Predicate {
        key: MetadataKey::new("author".into()),
        value: MetadataValue::new("Lex Luthor".into()),
        op: PredicateOp::Equals,
    });

    let test_create_predicates = HashSet::from_iter([MetadataKey::new(String::from("username"))]);

    let create_store = Query::CreateStore {
        store: sample_store_name.clone(),
        dimension: NonZeroUsize::new(1).unwrap(),
        create_predicates: test_create_predicates.clone(),
        error_if_exists: true,
    };

    let get_key = Query::GetKey {
        store: sample_store_name.clone(),
        keys: sample_store_keys.clone(),
    };
    let delete_key = Query::DelKey {
        store: sample_store_name.clone(),
        keys: sample_store_keys.clone(),
    };
    let get_sim_n = Query::GetSimN {
        store: sample_store_name.clone(),
        search_input: store_key.clone(),
        closest_n: NonZeroUsize::new(2).unwrap(),
        algorithm: types::similarity::Algorithm::CosineSimilarity,
        condition: Some(test_predicate_condition.clone()),
    };

    //StoreValue = StdHashMap<MetadataKey, MetadataValue>
    let mut store_value = StdHashMap::new();
    store_value.insert(
        MetadataKey::new(String::from("username")),
        MetadataValue::new(String::from("buster_matthews")),
    );

    let set_query = Query::Set {
        store: sample_store_name.clone(),
        inputs: vec![(store_key.clone(), store_value)],
    };

    let test_predicate_condition = &PredicateCondition::Value(types::predicate::Predicate {
        key: MetadataKey::new("author".into()),
        value: MetadataValue::new("Lex Luthor".into()),
        op: PredicateOp::Equals,
    });

    let getpred_variant = Query::GetPred {
        store: sample_store_name.clone(),
        condition: test_predicate_condition.clone(),
    };
    let deletepred_variant = Query::DelPred {
        store: sample_store_name.clone(),
        condition: test_predicate_condition.clone(),
    };

    let _ = tracer
        .trace_value(&mut samples, &create_store)
        .expect("Error tracing the variant");
    let _ = tracer
        .trace_value(&mut samples, &get_key)
        .expect("Error tracing the getkey variant");
    let _ = tracer
        .trace_value(&mut samples, &delete_key)
        .expect("Error tracing the deleteKey variant");
    let _ = tracer
        .trace_value(&mut samples, &get_sim_n)
        .expect("Error tracing the GetSimN variant");
    let _ = tracer
        .trace_value(&mut samples, &set_query)
        .expect("Error tracing the setquery varient");
    let _ = tracer
        .trace_value(&mut samples, &getpred_variant)
        .expect("Error tracing the getpred variant");
    let _ = tracer
        .trace_value(&mut samples, &deletepred_variant)
        .expect("Error tracing the deletepred variant");

    // trace enums to fix missing variants error
    //
    // Also trace each enum type separately to fix any `MissingVariants` error.
    tracer
        .trace_simple_type::<Algorithm>()
        .expect("Error tracing Algorithm");
    //
    // predicate conditions
    let _ = tracer
        .trace_type::<PredicateCondition>(&samples)
        .expect("Error tracing predicate condition");

    let _ = tracer
        .trace_type::<Query>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    tracer
        .registry()
        .expect("Failed to create registry for query")
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::tracers::load_type_into_registry;

    #[test]
    fn test_spec_documents_matches_current_query_enum() {
        let query_json_path = std::path::PathBuf::from("../../type_specs").join("query.json");
        let query_json = load_type_into_registry(query_json_path);
        let query_from_types = trace_query_enum();
        assert!(query_json == query_from_types)
    }
}
