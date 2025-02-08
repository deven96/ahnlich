use std::{
    io::{Result, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

fn main() -> Result<()> {
    // Get the current package directory
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Move up to the workspace root
    let workspace_root = manifest_dir
        .parent()
        .expect("Failed parent 1")
        .parent()
        .expect("Failed parent 2"); // Adjust if needed

    let proto_dir = workspace_root.join("protos/");

    let protofiles: Vec<PathBuf> = WalkDir::new(proto_dir.clone())
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "proto"))
        .map(|a| a.path().to_path_buf())
        .collect();

    let mut config = prost_build::Config::new();

    let out_dir = "src/";

    if let Ok(entries) = std::fs::read_dir(out_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.file_name().map_or(false, |name| name != "lib.rs") {
                if path.is_dir() {
                    std::fs::remove_dir_all(&path).expect("Failed to remove directory");
                } else {
                    std::fs::remove_file(&path).expect("Failed to remove file");
                }
            }
        }
    }

    config
        .out_dir(out_dir)
        .compile_protos(&protofiles, &[proto_dir])
        .inspect_err(|err| println!("{}", err.to_string()))
        .expect("failed");

    restructure_generated_code(&PathBuf::from("src/"));

    Ok(())
}

fn restructure_generated_code(out_dir: &PathBuf) {
    let generated_code: Vec<PathBuf> = WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "rs"))
        .map(|entry| entry.into_path())
        .collect();

    for file in &generated_code {
        if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
            if file_name.contains(".") {
                let parts: Vec<&str> = file_name.split('.').collect();

                if parts.len() > 2 {
                    let module_name = parts[0];
                    let struct_file = format!("{}.rs", parts[1]);

                    let module_path = out_dir.join(module_name);
                    let final_file_path = module_path.join(struct_file);

                    // create module dir if missing
                    std::fs::create_dir_all(&module_path)
                        .expect("Failed to create module directory");

                    std::fs::rename(file, &final_file_path)
                        .expect("Failed to move generated file to new location");

                    let mod_rs_path = module_path.join("mod.rs");

                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&mod_rs_path)
                        .expect("Failed to create mod file");

                    let buffer = format!("pub mod {};\n", parts[1]);

                    file.write(buffer.as_bytes())
                        .expect("Failed to write to mod file");
                }
            }
        }
    }
}
