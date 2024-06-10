pub use query::trace_query_enum;
use serde_reflection::Registry;
pub use server_response::trace_server_response_enum;
use std::{fs::File, io::BufReader};

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
    language: Language,
    spec_names: Vec<String>,
    filenames: Vec<String>,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
) {
    let task = TypeGenTask::build(language, spec_names, filenames, input_dir, output_dir);
    task.generate_type_def_for_language()
}

struct TypeGenTask {
    language: Language,
    spec_names: Vec<String>,
    filenames: Vec<String>,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
}
impl TypeGenTask {
    fn build(
        language: Language,
        spec_names: Vec<String>,
        filenames: Vec<String>,
        input_dir: std::path::PathBuf,
        output_dir: std::path::PathBuf,
    ) -> Self {
        if filenames.len() != spec_names.len() {
            panic!("Unequal length for filenames and spec_names");
        }
        Self {
            language,
            spec_names,
            filenames,
            input_dir,
            output_dir,
        }
    }

    fn generate_type_def_for_language(&self) {
        for (filename, specname) in self.filenames.iter().zip(self.spec_names.iter()) {
            let registry = load_type_into_registry(self.input_dir.as_path().join(specname));
            let query_file = std::fs::File::create(self.output_dir.join(filename))
                .expect("Failed to create file");
            let mut buffer = std::io::BufWriter::new(query_file);
            self.process_type_gen(&mut buffer, &registry);
        }
    }

    fn process_type_gen(&self, buffer: &mut std::io::BufWriter<File>, registry: &Registry) {
        let config = serde_generate::CodeGeneratorConfig::new("".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        match self.language {
            Language::Python => {
                serde_generate::python3::CodeGenerator::new(&config).output(buffer, &registry)
            }
            Language::Golang => {
                serde_generate::golang::CodeGenerator::new(&config).output(buffer, &registry)
            }
            Language::Ocaml => {
                serde_generate::ocaml::CodeGenerator::new(&config).output(buffer, &registry)
            }
            Language::Typescript => {
                serde_generate::typescript::CodeGenerator::new(&config).output(buffer, &registry)
            }
            _others => {
                // checkout out cpp failure.
                // Also  does dart, indent etc  don't implement output thesame way as other
                // languages

                panic!("Failed to use other types for now, they don't implement output or implement it differently")
            }
        }
        .expect("Failed to generate language definition");
    }
}
