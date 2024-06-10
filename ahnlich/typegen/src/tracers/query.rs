use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use types::predicate::PredicateCondition;
use types::predicate::PredicateOp;
use types::{
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
    query::Query,
};

pub fn trace_query_enum(input_dir: &std::path::Path) {
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

    let _ = tracer
        .trace_type::<Query>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let registry = tracer
        .registry()
        .expect("Failed to create registry for query");

    let file_path = input_dir.join("query.json");

    let query_file = std::fs::File::create(file_path).expect("Failed to create query file");
    let buffer = std::io::BufWriter::new(query_file);

    serde_json::to_writer_pretty(buffer, &registry)
        .expect("Query: Failed to write tracer registry into json file");
}