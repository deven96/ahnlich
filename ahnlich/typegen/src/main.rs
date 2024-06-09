mod cli;
mod tracers;

use crate::cli::{Cli, Commands};
use crate::tracers::{load_type_into_registry, trace_query_enum, trace_server_response_enum};
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
            let spec_dir = if let Some(path) = &config.input_spec_dir {
                path.to_owned()
            } else {
                std::path::PathBuf::from(SPEC_DOC_PATH)
            };

            for entry in std::fs::read_dir(spec_dir)? {
                let file_path = entry?.path();
                let _registry = load_type_into_registry(file_path);
            }
        }
    }
    Ok(())
}
