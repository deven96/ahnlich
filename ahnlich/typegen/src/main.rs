mod cli;
mod tracers;

use crate::cli::{Cli, Commands};
use crate::tracers::{
    generate_language_definition, save_registry_into_file, trace_query_enum,
    trace_server_response_enum,
};
use clap::Parser;
use std::error::Error;
const SPEC_DOC_PATH: &str = "../type_specs/";
const SDK_PATH: &str = "../sdk";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(config) => {
            if config.output_dir.is_some()
                && !config
                    .output_dir
                    .as_ref()
                    .map(|path| path.is_dir())
                    .unwrap()
            {
                panic!("Cannot generate type specs into invalid directory")
            }
            let extension: &str = (&config.language).into();
            let output_dir = &config
                .output_dir
                .to_owned()
                .unwrap_or(std::path::PathBuf::from(SPEC_DOC_PATH))
                .join(config.language.into());
            std::fs::create_dir_all(output_dir).expect("Failed to create sdk directory");

            let output_dir = &config
                .output_dir
                .to_owned()
                .unwrap_or(std::path::PathBuf::from(SPEC_DOC_PATH));

            let query_registry = trace_query_enum();
            save_registry_into_file(&query_registry, output_dir.to_owned().join("query.json"));
            let server_response_reg = trace_server_response_enum();
            save_registry_into_file(
                &server_response_reg,
                output_dir.to_owned().join("server_response.json"),
            );

            println!("Types spec successfully generated");
        }
        Commands::CreateClient(config) => {
            if config.input_spec_dir.is_some()
                && !config
                    .input_spec_dir
                    .as_ref()
                    .map(|path| path.is_dir())
                    .unwrap()
            {
                panic!("Cannot read type specs from invalid directory")
            }

            let input_dir = &config
                .input_spec_dir
                .to_owned()
                .unwrap_or(std::path::PathBuf::from(SPEC_DOC_PATH));

            let output_dir = &config
                .output_dir
                .to_owned()
                .unwrap_or(std::path::PathBuf::from(SDK_PATH));

            std::fs::create_dir_all(output_dir).expect("Failed to create sdk directory");

            generate_language_definition(config.language, input_dir, output_dir);

            println!("Language type definition generated");
        }
    }
    Ok(())
}
