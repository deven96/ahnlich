pub use query::trace_query_enum;
use serde_generate::CodeGeneratorConfig;
use serde_generate::SourceInstaller;
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
    input_dir: &std::path::PathBuf,
    output_dir: &std::path::PathBuf,
) {
    let task = SpecToLanguage::build(language, input_dir.to_owned(), output_dir.to_owned());
    task.generate_type_def_for_language()
}

struct SpecToLanguage {
    language: Language,
    input_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
}
impl SpecToLanguage {
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
        let output_file: OutputFile = (&self.output_dir, file_name, &self.language).into();
        output_file.generate(&config, registry)
    }
}

struct OutputFile<'a> {
    language: Language,
    output_dir: std::path::PathBuf,
    output_file: &'a str,
}

impl<'a> OutputFile<'a> {
    fn generate(&self, config: &CodeGeneratorConfig, registry: &Registry) {
        let extension: &str = (&self.language).into();
        let output_dir = self.output_dir.join(extension);
        let _ = std::fs::create_dir_all(&output_dir);
        let output_file = output_dir.join(format!("{}.{extension}", self.output_file));

        let spec_language_file =
            std::fs::File::create(output_file).expect("Failed to create typegen output file");
        let mut buffer: BufWriter<File> = std::io::BufWriter::new(spec_language_file);

        let _ = match self.language {
            Language::Python => {
                let installer = serde_generate::python3::Installer::new(output_dir, None);
                installer.install_bincode_runtime().unwrap();
                installer.install_serde_runtime().unwrap();
                serde_generate::python3::CodeGenerator::new(config).output(&mut buffer, registry)
            }
            Language::Golang => {
                // All packages are already published
                serde_generate::golang::CodeGenerator::new(config).output(&mut buffer, registry)
            }
            Language::Typescript => {
                let installer = serde_generate::typescript::Installer::new(output_dir);
                installer.install_serde_runtime().unwrap();
                installer.install_bincode_runtime().unwrap();
                serde_generate::typescript::CodeGenerator::new(config).output(&mut buffer, registry)
            }
            _others => {
                // checkout out cpp failure.
                // Also  does dart, indent etc  don't implement output thesame way as other
                // languages
                panic!("Failed to use other types for now, they don't implement output or implement it differently")
            }
        };
    }
}

impl From<&Language> for &str {
    fn from(value: &Language) -> Self {
        match value {
            Language::Python => "py",
            Language::Golang => "go",
            Language::Ocaml => "ml",
            Language::Typescript => "ts",
            _others => panic!("Cannot generate extension for type"),
        }
    }
}

impl<'a> From<(&std::path::PathBuf, &'a str, &Language)> for OutputFile<'a> {
    fn from((file_dir, file_name, language): (&std::path::PathBuf, &'a str, &Language)) -> Self {
        OutputFile {
            language: *language,
            output_file: file_name,
            output_dir: file_dir.to_path_buf(),
        }
    }
}
