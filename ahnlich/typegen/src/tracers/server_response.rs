use serde_reflection::{Samples, Tracer, TracerConfig};
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet;
use std::time::SystemTime;
use types::similarity::Similarity;
use types::{
    keyval::{StoreKey, StoreName},
    metadata::{MetadataKey, MetadataValue},
    server::{ConnectedClient, ServerInfo, ServerResponse, ServerType, StoreInfo, StoreUpsert},
    version::Version,
};

pub fn trace_server_response_enum(input_dir: &std::path::Path) {
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
        Similarity(0.999_f64),
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

    let registry = tracer
        .registry()
        .expect("Failed to create registry for server response");

    let file_path = input_dir.join("server_response.json");
    let query_file = std::fs::File::create(file_path).unwrap();
    let buffer = std::io::BufWriter::new(query_file);

    serde_json::to_writer_pretty(buffer, &registry)
        .expect("Server Response: Failed to write tracer registry into json file");
}
