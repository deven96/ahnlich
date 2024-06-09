use serde_reflection::Registry;
use std::io::BufReader;
mod server_response;
mod trace_query;

pub use trace_query::trace_query_enum;

pub(crate) fn load_type_into_registry(file_path: std::path::PathBuf) -> Registry {
    let query_file = std::fs::File::open(file_path)
        .unwrap_or_else(|err| panic!("Failed to open file, error: {}", err));
    let reader = BufReader::new(query_file);
    let registry: Registry =
        serde_json::from_reader(reader).expect("Failed to read registry from json file");
    registry
}
