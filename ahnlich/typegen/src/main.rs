mod cli;
mod tracers;

use crate::cli::{Cli, Commands};
use crate::tracers::{generate_language_definition, trace_query_enum, trace_server_response_enum};
use clap::Parser;
use std::error::Error;
const SPEC_DOC_PATH: &str = "../type_specs/";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(config) => {
            let output_dir = if let Some(path) = &config.output_dir {
                path.to_owned()
            } else {
                std::path::PathBuf::from(SPEC_DOC_PATH)
            };

            trace_query_enum(&output_dir);
            trace_server_response_enum(&output_dir);
            println!("Types spec successfully generated");
        }
        Commands::CreateClient(config) => {
            let input_dir = if let Some(path) = &config.input_spec_dir {
                path.to_owned()
            } else {
                std::path::PathBuf::from(SPEC_DOC_PATH)
            };

            let output_dir = if let Some(path) = &config.output_dir {
                path.to_owned()
            } else {
                std::path::PathBuf::from("../sdk")
            };

            std::fs::create_dir_all(&output_dir).expect("Failed to create sdk directory");

            generate_language_definition(config.language, input_dir, output_dir);

            println!("Language type definition generated");
        }
    }
    Ok(())
}
