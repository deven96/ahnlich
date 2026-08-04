use std::{collections::BTreeSet, io::Write, path::PathBuf};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");

    let cluster_admin = proto_dir.join("cluster_admin.proto");
    let raft_internal = proto_dir.join("raft_internal.proto");

    println!("cargo:rerun-if-changed={}", cluster_admin.display());
    println!("cargo:rerun-if-changed={}", raft_internal.display());

    let out_dir = PathBuf::from("src/proto");

    // Clean up old generated files and subdirectories
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)?;
    }
    std::fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(&[cluster_admin, raft_internal], &[proto_dir])?;

    // Restructure generated code into proper module hierarchy
    let mut proto_mod_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(out_dir.join("mod.rs"))?;

    restructure_generated_code(&out_dir, &mut proto_mod_file)?;
    format_generated_code(&out_dir);

    Ok(())
}

/// Run rustfmt over the freshly generated files so the output matches `cargo fmt`,
/// avoiding the "regenerate produces an unformatted diff" churn. rustfmt not being
/// available is non-fatal (formatting does not affect compilation).
fn format_generated_code(out_dir: &PathBuf) {
    let files: Vec<PathBuf> = WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect();
    match std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .args(&files)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => println!("cargo:warning=rustfmt exited with {status} on generated code"),
        Err(err) => println!("cargo:warning=could not run rustfmt on generated code: {err}"),
    }
}

fn restructure_generated_code(
    out_dir: &PathBuf,
    file: &mut std::fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
    let generated_code: Vec<PathBuf> = WalkDir::new(out_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "rs")
                && entry
                    .path()
                    .file_name()
                    .is_some_and(|name| name != "mod.rs")
        })
        .map(|entry| entry.into_path())
        .collect();

    let mut module_names = BTreeSet::new();

    for file_path in &generated_code {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str())
            && file_name.contains(".")
        {
            let parts: Vec<&str> = file_name.split('.').collect();

            if parts.len() == 3 && parts[0] == "services" {
                let module_name = parts[1];
                let mod_rs_path = out_dir.join(module_name).join("mod.rs");

                std::fs::create_dir_all(out_dir.join(module_name))?;
                std::fs::rename(file_path, &mod_rs_path)?;

                module_names.insert(module_name.to_string());
            }
        }
    }

    let buffer = module_names
        .into_iter()
        .map(|module| format!("pub mod {};", module))
        .collect::<Vec<String>>()
        .join("\n");

    writeln!(file, "{}", buffer)?;

    Ok(())
}
