use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");

    let cluster_admin = proto_dir.join("cluster_admin.proto");
    let raft_internal = proto_dir.join("raft_internal.proto");

    println!("cargo:rerun-if-changed={}", cluster_admin.display());
    println!("cargo:rerun-if-changed={}", raft_internal.display());

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&[cluster_admin, raft_internal], &[proto_dir])?;

    Ok(())
}
