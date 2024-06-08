use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::num::NonZeroUsize;
use types::predicate::PredicateCondition;
use types::predicate::PredicateOp;
use types::{
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
    query::Query,
};

fn main() {
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

    let _ = tracer.trace_value(&mut samples, &get_key).unwrap();
    let _ = tracer.trace_value(&mut samples, &delete_key).unwrap();
    let _ = tracer.trace_value(&mut samples, &get_sim_n).unwrap();
    let _ = tracer.trace_value(&mut samples, &set_query).unwrap();
    let _ = tracer.trace_value(&mut samples, &getpred_variant).unwrap();
    let _ = tracer
        .trace_value(&mut samples, &deletepred_variant)
        .unwrap();
    println!("traced_sample store keys");

    let _ = tracer
        .trace_type::<Query>(&samples)
        .inspect_err(|err| println!("Failed to parse error{}", err.explanation()))
        .unwrap();

    let registry = tracer.registry().expect("Failed to create registry");

    println!("{:?}", registry);
}
