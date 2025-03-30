use std::{
    collections::HashSet,
    io::{Result, Write},
    path::PathBuf,
};
use walkdir::WalkDir;
// TODO: this would serve as a stand in replacement for the types crate
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

    println!(
        "cargo:rerun-if-changed={}",
        proto_dir
            .as_path()
            .to_str()
            .expect("Cannot get proto dir str path")
    );
    println!("cargo:warning=Run `cargo fmt` after build to format generated files.");

    let protofiles: Vec<PathBuf> = WalkDir::new(proto_dir.clone())
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "proto"))
        .map(|a| a.path().to_path_buf())
        .collect();
    let out_dir = "src/";

    if let Ok(entries) = std::fs::read_dir(out_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.file_name().map_or(false, |name| name != "utils") {
                if path.is_dir() {
                    std::fs::remove_dir_all(&path).expect("Failed to remove directory");
                } else {
                    std::fs::remove_file(&path).expect("Failed to remove file");
                }
            }
        }
    }

    let out_dir = PathBuf::from("src/");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(out_dir.join("lib.rs"))
        .expect("Failed to create mod file");

    // nonlinear algorthim, storekeyid, storevalue, metadatakey and value,
    tonic_build::configure()
        .build_client(true)
        .build_client(true)
        .out_dir(out_dir.clone())
        .type_attribute(
            "algorithm.nonlinear.NonLinearAlgorithm",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("keyval.StoreKey", "#[derive(serde::Serialize)]")
        .type_attribute(
            "keyval.StoreValue",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "metadata.MetadataValue",
            "#[derive(serde::Serialize, serde::Deserialize, PartialOrd, Ord, Hash, Eq)]",
        )
        .type_attribute(
            "keyval.StoreName",
            "#[derive(serde::Serialize, serde::Deserialize, Eq, Hash, Ord, PartialOrd)]",
        )
        .type_attribute(
            "db.server.StoreInfo",
            "#[derive(Hash, Eq, Ord, PartialOrd)]",
        )
        .type_attribute(
            "metadata.MetadataValue.value",
            "#[derive(serde::Serialize, serde::Deserialize, PartialOrd, Ord, Hash, Eq)]",
        )
        .type_attribute(
            "ai.models.AIModel",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ai.server.AIStoreInfo",
            "#[derive(Eq, PartialOrd, Ord, Hash)]",
        )
        .type_attribute(
            "client.ConnectedClient",
            "#[derive(PartialOrd, Ord, Hash, Eq)]",
        )
        .compile_protos(&protofiles, &[proto_dir])
        .inspect_err(|err| println!("{}", err))
        .expect("failed");

    restructure_generated_code(&out_dir, &mut file);

    Ok(())
}

fn restructure_generated_code(out_dir: &PathBuf, file: &mut std::fs::File) {
    let generated_code: Vec<PathBuf> = WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "rs")
                && entry.path().parent().map_or(true, |parent| {
                    parent.file_name().expect("Failed to get filename") != "utils"
                })
        })
        .map(|entry| entry.into_path())
        .collect();

    let mut module_names = HashSet::new();

    for file in &generated_code {
        if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
            if file_name.contains(".") {
                let parts: Vec<&str> = file_name.split('.').collect();
                module_names.insert(parts[0]);

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

                    file.write_all(buffer.as_bytes())
                        .expect("Failed to write to mod file");
                }
            }
        }
    }

    module_names.insert("utils");

    let buffer = module_names
        .into_iter()
        .filter(|file| *file != "lib")
        .map(|sub_str| format!("pub mod {sub_str};"))
        .collect::<Vec<String>>()
        .join("\n");

    file.write_all(buffer.as_bytes())
        .expect("could not generate lib.rs");
}
