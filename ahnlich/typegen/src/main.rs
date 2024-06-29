mod cli;
mod tracers;

use crate::cli::{Cli, Commands};
use crate::tracers::{save_queries_registries_into_file, LanguageGeneratorTasks};
use clap::Parser;
use std::error::Error;
use tracers::save_server_response_registries;
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
            let output_dir = &config
                .output_dir
                .to_owned()
                .unwrap_or(std::path::PathBuf::from(SPEC_DOC_PATH));

            save_queries_registries_into_file(output_dir);
            save_server_response_registries(output_dir);

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

            let language_gen =
                LanguageGeneratorTasks::build(input_dir, output_dir, config.language);
            language_gen.generate_language_definition();

            println!("Language type definition generated");
        }
    }
    Ok(())
}
