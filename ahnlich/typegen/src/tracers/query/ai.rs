use ahnlich_types::ai::{AIModel, AIStoreInputType, PreprocessAction};
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::predicate::Predicate;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::similarity::{Algorithm, NonLinearAlgorithm};
use ahnlich_types::{
    ai::{AIQuery, AIServerQuery},
    keyval::StoreName,
    metadata::{MetadataKey, MetadataValue},
};
use serde_reflection::Registry;
use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::{HashMap as StdHashMap, HashSet};
use std::num::NonZeroUsize;

pub fn trace_ai_query_enum() -> Registry {
    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();

    let sample_store_name = StoreName("ijdfsdf".into());
    let test_search_input = StoreInput::RawString(String::from("Hello"));
    let test_search_input_bin = StoreInput::Image(vec![2, 1, 1, 4, 5]);
    let test_predicate_condition = &PredicateCondition::Value(Predicate::Equals {
        key: MetadataKey::new("author".into()),
        value: MetadataValue::RawString("Lex Luthor".into()),
    });

    let test_create_predicates = HashSet::from_iter([MetadataKey::new(String::from("username"))]);

    let test_predicates = HashSet::from_iter(vec![
        MetadataKey::new("rank".into()),
        MetadataKey::new("job".into()),
    ]);
    //StoreValue = StdHashMap<MetadataKey, MetadataValue>
    let store_value = StdHashMap::from_iter([
        (
            MetadataKey::new(String::from("username")),
            MetadataValue::RawString(String::from("buster_matthews")),
        ),
        (
            MetadataKey::new(String::from("bin_data")),
            MetadataValue::Image(vec![6, 4, 2]),
        ),
    ]);
    let test_non_linear_indices = HashSet::from_iter([NonLinearAlgorithm::KDTree]);

    let create_store = AIQuery::CreateStore {
        store: sample_store_name.clone(),
        index_model: AIModel::AllMiniLML6V2,
        query_model: AIModel::AllMiniLML6V2,
        predicates: test_create_predicates.clone(),
        non_linear_indices: test_non_linear_indices,
        error_if_exists: false,
        store_original: true,
    };

    let get_pred = AIQuery::GetPred {
        store: sample_store_name.clone(),
        condition: test_predicate_condition.clone(),
    };

    let get_sim_n = AIQuery::GetSimN {
        store: sample_store_name.clone(),
        search_input: test_search_input.clone(),
        condition: Some(test_predicate_condition.clone()),
        closest_n: NonZeroUsize::new(4).unwrap(),
        algorithm: Algorithm::CosineSimilarity,
    };

    let create_index = AIQuery::CreatePredIndex {
        store: sample_store_name.clone(),
        predicates: test_predicates.clone(),
    };

    let drop_index_pred = AIQuery::DropPredIndex {
        store: sample_store_name.clone(),
        predicates: test_predicates.clone(),
        error_if_not_exists: true,
    };

    let set = AIQuery::Set {
        store: sample_store_name.clone(),
        preprocess_action: PreprocessAction::NoPreprocessing,
        inputs: vec![(test_search_input_bin.clone(), store_value)],
    };

    let del_key = AIQuery::DelKey {
        store: sample_store_name.clone(),
        key: test_search_input.clone(),
    };
    let drop_store = AIQuery::DropStore {
        store: sample_store_name.clone(),
        error_if_not_exists: true,
    };
    let trace_id = "00-djf9039023r3-1er".to_string();
    let server_query_with_trace_id = AIServerQuery::with_capacity_and_tracing_id(2, Some(trace_id));
    let server_query = AIServerQuery::from_queries(&[del_key.clone(), set.clone()]);

    // trace each query variant
    let _ = tracer
        .trace_value(&mut samples, &create_store)
        .expect("Error tracing the variant");

    let _ = tracer
        .trace_value(&mut samples, &get_sim_n)
        .expect("Error tracing the variant");

    let _ = tracer
        .trace_value(&mut samples, &get_pred)
        .expect("Error tracing the variant");

    let _ = tracer
        .trace_value(&mut samples, &create_index)
        .expect("Error tracing the variant");

    let _ = tracer
        .trace_value(&mut samples, &drop_index_pred)
        .expect("Error tracing the variant");
    let _ = tracer
        .trace_value(&mut samples, &set)
        .expect("Error tracing the variant");
    let _ = tracer
        .trace_value(&mut samples, &del_key)
        .expect("Error tracing the variant");
    let _ = tracer
        .trace_value(&mut samples, &drop_store)
        .expect("Error tracing the variant");
    // end of trace each query variant
    let _ = tracer
        .trace_value(&mut samples, &server_query)
        .expect("Error tracing the server_query");
    let _ = tracer
        .trace_value(&mut samples, &server_query_with_trace_id)
        .expect("Error tracing the server_query_with_trace_id");

    tracer
        .trace_simple_type::<Predicate>()
        .expect("Error tracing Predicate");
    tracer
        .trace_simple_type::<Algorithm>()
        .expect("Error tracing Algorithm");

    tracer
        .trace_simple_type::<NonLinearAlgorithm>()
        .expect("Error tracing NonLinearAlgorithm");
    // predicate conditions
    let _ = tracer
        .trace_type::<PredicateCondition>(&samples)
        .expect("Error tracing predicate condition");

    let _ = tracer
        .trace_type::<AIQuery>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<AIServerQuery>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<MetadataValue>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();
    let _ = tracer
        .trace_type::<AIStoreInputType>(&samples)
        .expect("Error tracing AIStoreInputType");

    let _ = tracer
        .trace_type::<AIModel>(&samples)
        .expect("Error tracing AIModel");
    let _ = tracer
        .trace_type::<PreprocessAction>(&samples)
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
            std::path::PathBuf::from("../../type_specs/query").join("ai_query.json");
        let query_json = load_type_into_registry(query_json_path);
        let query_from_types = trace_ai_query_enum();
        assert_eq!(query_json, query_from_types)
    }
}
