pub use query::trace_query_enum;
use serde_reflection::Registry;
pub use server_response::trace_server_response_enum;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

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

pub(crate) fn save_registry_into_file(registry: &Registry, file_path: std::path::PathBuf) {
    let query_file = std::fs::File::create(file_path).expect("Failed to create query file");
    let buffer = std::io::BufWriter::new(query_file);

    serde_json::to_writer_pretty(buffer, &registry)
        .expect("Query: Failed to write tracer registry into json file");
}

pub(crate) fn generate_language_definition(
    language: Language,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
) {
    let task = TypeGenTask::build(language, input_dir, output_dir);
    task.generate_type_def_for_language()
}

struct TypeGenTask {
    language: Language,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
}
impl TypeGenTask {
    fn build(
        language: Language,
        input_dir: std::path::PathBuf,
        output_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            language,
            input_dir,
            output_dir,
        }
    }

    fn generate_type_def_for_language(&self) {
        let dir_entries = self
            .input_dir
            .as_path()
            .read_dir()
            .expect("Failed to read input dir")
            .map(|entry| {
                let file_path = entry.unwrap().path();
                let file_stem = file_path.file_stem().unwrap();
                let str_filename = file_stem.to_str().unwrap().to_string();
                str_filename
            });
        for file_name in dir_entries.into_iter() {
            let registry = load_type_into_registry(
                self.input_dir
                    .as_path()
                    .join(format!("{}.json", &file_name)),
            );
            self.process_type_gen(&file_name, &registry);
        }
    }

    fn process_type_gen(&self, file_name: &str, registry: &Registry) {
        let config = serde_generate::CodeGeneratorConfig::new("".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        match self.language {
            Language::Python => {
                let query_file =
                    std::fs::File::create(self.output_dir.join(format!("{}.py",file_name)))
                    .expect("Failed to create file");
                let mut buffer: BufWriter<File> = std::io::BufWriter::new(query_file);

                serde_generate::python3::CodeGenerator::new(&config)
                    .output(&mut buffer, registry)
            }
            Language::Golang => {
                let query_file =
                    std::fs::File::create(self.output_dir.join(format!("{}.go",file_name)))
                    .expect("Failed to create file");
                let mut buffer: BufWriter<File> = std::io::BufWriter::new(query_file);
                serde_generate::golang::CodeGenerator::new(&config)
                    .output(&mut buffer, registry)
            }
            Language::Ocaml => {
                let query_file =
                    std::fs::File::create(self.output_dir.join(format!("{}.ml",file_name)))
                    .expect("Failed to create file");
                let mut buffer: BufWriter<File> = std::io::BufWriter::new(query_file);
                serde_generate::ocaml::CodeGenerator::new(&config)
                    .output(&mut buffer, registry)
            }
            Language::Typescript => {
                let query_file =
                    std::fs::File::create(self.output_dir.join(format!("{}.ts",file_name)))
                    .expect("Failed to create Typescript file");
                let mut buffer: BufWriter<File> = std::io::BufWriter::new(query_file);
                serde_generate::typescript::CodeGenerator::new(&config)
                    .output(&mut buffer, registry)
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
