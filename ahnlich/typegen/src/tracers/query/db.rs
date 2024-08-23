use ahnlich_types::predicate::Predicate;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::similarity::Algorithm;
use ahnlich_types::similarity::NonLinearAlgorithm;
use ahnlich_types::{
    db::{DBQuery, ServerDBQuery},
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
};
use serde_reflection::Registry;
use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;

pub fn trace_db_query_enum() -> Registry {
    let input_arr_1 = ndarray::array![0.1, 0.2, 0.3, 0.4, 0.5];
    let store_key = StoreKey(input_arr_1.clone());

    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();

    let sample_store_name = StoreName("ijdfsdf".into());
    let sample_store_keys = vec![store_key.clone()];
    let test_predicate_condition = &PredicateCondition::Value(Predicate::Equals {
        key: MetadataKey::new("author".into()),
        value: MetadataValue::RawString("Lex Luthor".into()),
    });

    let test_create_predicates = HashSet::from_iter([MetadataKey::new(String::from("username"))]);
    let test_non_linear_indices = HashSet::from_iter([NonLinearAlgorithm::KDTree]);

    let create_store = DBQuery::CreateStore {
        store: sample_store_name.clone(),
        dimension: NonZeroUsize::new(1).unwrap(),
        create_predicates: test_create_predicates.clone(),
        non_linear_indices: test_non_linear_indices,
        error_if_exists: true,
    };

    let get_key = DBQuery::GetKey {
        store: sample_store_name.clone(),
        keys: sample_store_keys.clone(),
    };
    let delete_key = DBQuery::DelKey {
        store: sample_store_name.clone(),
        keys: sample_store_keys.clone(),
    };
    let get_sim_n = DBQuery::GetSimN {
        store: sample_store_name.clone(),
        search_input: store_key.clone(),
        closest_n: NonZeroUsize::new(2).unwrap(),
        algorithm: ahnlich_types::similarity::Algorithm::CosineSimilarity,
        condition: Some(test_predicate_condition.clone()),
    };

    //StoreValue = StdHashMap<MetadataKey, MetadataValue>
    let mut store_value = StdHashMap::new();
    store_value.insert(
        MetadataKey::new(String::from("username")),
        MetadataValue::RawString(String::from("buster_matthews")),
    );
    store_value.insert(
        MetadataKey::new(String::from("bin_data")),
        MetadataValue::Image(vec![6, 4, 2]),
    );

    let set_query = DBQuery::Set {
        store: sample_store_name.clone(),
        inputs: vec![(store_key.clone(), store_value)],
    };

    let test_predicate_condition = &PredicateCondition::Value(Predicate::NotEquals {
        key: MetadataKey::new("author".into()),
        value: MetadataValue::RawString("Lex Luthor".into()),
    });

    let getpred_variant = DBQuery::GetPred {
        store: sample_store_name.clone(),
        condition: test_predicate_condition.clone(),
    };
    let deletepred_variant = DBQuery::DelPred {
        store: sample_store_name.clone(),
        condition: test_predicate_condition.clone(),
    };

    let server_query =
        ServerDBQuery::from_queries(&[deletepred_variant.clone(), set_query.clone()]);
    let trace_id = "00-djf9039023r3-1er".to_string();
    let server_query_with_trace_id = ServerDBQuery::with_capacity_and_tracing_id(2, Some(trace_id));

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

    let _ = tracer
        .trace_value(&mut samples, &server_query)
        .expect("Error tracing the server_query");

    let _ = tracer
        .trace_value(&mut samples, &server_query_with_trace_id)
        .expect("Error tracing the server_query_with_trace_id");

    // trace enums to fix missing variants error
    //
    // Also trace each enum type separately to fix any `MissingVariants` error.
    tracer
        .trace_simple_type::<Algorithm>()
        .expect("Error tracing Algorithm");

    tracer
        .trace_simple_type::<NonLinearAlgorithm>()
        .expect("Error tracing NonLinearAlgorithm");
    tracer
        .trace_simple_type::<Predicate>()
        .expect("Error tracing Predicate");
    //
    // predicate conditions
    let _ = tracer
        .trace_type::<PredicateCondition>(&samples)
        .expect("Error tracing predicate condition");

    let _ = tracer
        .trace_type::<DBQuery>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<ServerDBQuery>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<MetadataValue>(&samples)
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
    fn test_spec_documents_matches_current_db_query_enum() {
        let query_json_path =
            std::path::PathBuf::from("../../type_specs/query").join("db_query.json");
        let query_json = load_type_into_registry(query_json_path);
        let query_from_types = trace_db_query_enum();
        assert_eq!(query_json, query_from_types)
    }
}
