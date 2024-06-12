use serde_reflection::Registry;
use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::time::SystemTime;
use types::similarity::Similarity;
use types::{
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
    server::{ConnectedClient, ServerInfo, ServerResponse, ServerResult, ServerType, StoreInfo, StoreUpsert},
    version::Version,
};

pub fn trace_server_response_enum() -> Registry {
    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();

    // trace complex variants

    let connected_clients = HashSet::from_iter([ConnectedClient {
        address: "127.0.0.1".to_string(),
        time_connected: SystemTime::now(),
    }]);

    let client_list = ServerResponse::ClientList(connected_clients.clone());

    let store_list = ServerResponse::StoreList(HashSet::from_iter([StoreInfo {
        name: StoreName("testing".to_owned()),
        len: 12,
        size_in_bytes: 91,
    }]));

    let info_server = ServerResponse::InfoServer(ServerInfo {
        address: "127.0.0.1".to_owned(),
        version: Version {
            major: 1,
            minor: 0,
            patch: 1,
        },
        r#type: ServerType::Database,
        limit: 121,
        remaining: 20,
    });

    let set_variant = ServerResponse::Set(StoreUpsert {
        inserted: 0,
        updated: 1,
    });

    // Get variant
    let input_arr_1 = ndarray::array![0.1, 0.2, 0.3, 0.4, 0.5];
    let store_key = StoreKey(input_arr_1.clone());

    //StoreValue = StdHashMap<MetadataKey, MetadataValue>
    let mut store_value = StdHashMap::new();
    store_value.insert(
        MetadataKey::new(String::from("username")),
        MetadataValue::new(String::from("buster_matthews")),
    );

    let get_variant = ServerResponse::Get(vec![(store_key.clone(), store_value.clone())]);

    // getsminN

    let getsimn_variant = ServerResponse::GetSimN(vec![(
        store_key.clone(),
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
        .trace_type::<ServerResponse>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    let _ = tracer
        .trace_type::<Result<ServerResponse, String>>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();
    
    let _ = tracer
        .trace_type::<ServerResult>(&samples)
        .inspect_err(|err| println!("Failed to parse type {}", err.explanation()))
        .unwrap();

    tracer
        .registry()
        .expect("Failed to create registry for server response")
}

#[cfg(test)]
mod tests {

    use crate::tracers::{load_type_into_registry, trace_server_response_enum};

    #[test]
    fn test_spec_documents_matches_current_server_response_enum() {
        let server_response_json_path =
            std::path::PathBuf::from("../../type_specs").join("server_response.json");
        let server_response_json = load_type_into_registry(server_response_json_path);
        let server_response_from_types = trace_server_response_enum();
        assert_eq!(server_response_json, server_response_from_types)
    }
}
