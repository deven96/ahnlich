pub use query::trace_query_enum;
use serde_reflection::Registry;
pub use server_response::trace_server_response_enum;
use std::io::BufReader;

use crate::cli::Language;

mod query;
mod server_response;

pub(crate) fn load_type_into_registry(file_path: std::path::PathBuf) -> Registry {
    let query_file = std::fs::File::open(file_path)
        .unwrap_or_else(|err| panic!("Failed to open file, error: {}", err));
    let reader = BufReader::new(query_file);
    let registry: Registry =
        serde_json::from_reader(reader).expect("Failed to read registry from json file");
    registry
}

pub(crate) fn generate_language_definition(
    language: &Language,
    spec_name: &str,
    filename: &str,
    input_dir: &std::path::Path,
    output_dir: &std::path::Path,
) {
    let config = serde_generate::CodeGeneratorConfig::new("".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode]);

    let registry = load_type_into_registry(input_dir.join(spec_name));

    match language {
        Language::Python => {
            let query_file =
                std::fs::File::create(output_dir.join(filename)).expect("Failed to create file");
            let mut buffer = std::io::BufWriter::new(query_file);
            let generator = serde_generate::python3::CodeGenerator::new(&config);
            generator
                .output(&mut buffer, &registry)
                .expect("Failed to generate python language type");
        }
        Language::Golang => {
            let _generator = serde_generate::golang::CodeGenerator::new(&config);
        }
    };
}
