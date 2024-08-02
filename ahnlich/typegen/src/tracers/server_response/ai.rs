use ahnlich_types::ai::AIStoreInputTypes;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::similarity::Similarity;
use ahnlich_types::{
    ai::{AIModel, AIServerResponse, AIServerResult, AIStoreInfo},
    db::{ConnectedClient, ServerInfo, StoreUpsert},
    keyval::StoreName,
    metadata::{MetadataKey, MetadataValue},
    version::Version,
    ServerType,
};
use serde_reflection::Registry;
use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::time::SystemTime;

pub fn trace_ai_server_response_enum() -> Registry {
    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();

    // trace complex variants

    let connected_clients = HashSet::from_iter([ConnectedClient {
        address: "127.0.0.1".to_string(),
        time_connected: SystemTime::now(),
    }]);

    let client_list = AIServerResponse::ClientList(connected_clients.clone());

    let store_list = AIServerResponse::StoreList(HashSet::from_iter([AIStoreInfo {
        name: StoreName("testing".to_owned()),
        query_model: AIModel::Llama3,
        index_model: AIModel::Llama3,
        embedding_size: 20,
    }]));

    let info_server = AIServerResponse::InfoServer(ServerInfo {
        address: "127.0.0.1".to_owned(),
        version: Version {
            major: 1,
            minor: 0,
            patch: 1,
        },
        r#type: ServerType::AI,
        limit: 121,
        remaining: 20,
    });

    let set_variant = AIServerResponse::Set(StoreUpsert {
        inserted: 0,
        updated: 1,
    });

    let store_input = StoreInput::RawString("testing".into());

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

    let get_variant = AIServerResponse::Get(vec![(store_input.clone(), store_value.clone())]);

    // getsminN

    let getsimn_variant = AIServerResponse::GetSimN(vec![(
        store_input.clone(),
        store_value.clone(),
        Similarity(0.999_f32),
    )]);

    let _ = tracer
        .trace_value(&mut samples, &client_list)
        .expect("Error tracing ClientList variant");

    let _ = tracer
        .trace_value(&mut samples, &store_list)
        .expect("Error tracing StoreList variant");

    let _ = tracer
        .trace_value(&mut samples, &info_server)
        .expect("Error tracing InfoServer variant");

    let _ = tracer
        .trace_value(&mut samples, &set_variant)
        .expect("Error tracing Set variant");

    let _ = tracer
        .trace_value(&mut samples, &get_variant)
        .expect("Error tracing Get variant");

    let _ = tracer
        .trace_value(&mut samples, &getsimn_variant)
        .expect("Error tracing GetSimN variant");

    // trace server response

    let _ = tracer
        .trace_type::<AIServerResponse>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<Result<AIServerResponse, String>>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<AIServerResult>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<MetadataValue>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();
    let _ = tracer
        .trace_type::<ServerType>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();
    let _ = tracer
        .trace_type::<AIModel>(&samples)
        .expect("Error tracing AIModel");

    let _ = tracer
        .trace_type::<AIStoreInputTypes>(&samples)
        .expect("Error tracing AIStoreInputTypes");

    tracer
        .registry()
        .expect("Failed to create registry for server response")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tracers::load_type_into_registry;

    #[test]
    fn test_spec_documents_matches_current_ai_server_response_enum() {
        let server_response_json_path =
            std::path::PathBuf::from("../../type_specs/response").join("ai_response.json");
        let server_response_json = load_type_into_registry(server_response_json_path);
        let server_response_from_types = trace_ai_server_response_enum();
        assert_eq!(server_response_json, server_response_from_types)
    }
}
