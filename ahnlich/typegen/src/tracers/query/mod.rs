mod ai;
mod db;

pub(crate) use db::trace_db_query_enum;

use super::save_registry_into_file;

pub(crate) fn save_queries_registries_into_file(output_dir: &std::path::PathBuf) {
    let query_path = output_dir.to_owned().join("query");
    let _ = std::fs::create_dir_all(&query_path);
    let query_registry = trace_db_query_enum();
    save_registry_into_file(&query_registry, query_path.join("db_query.json"));
}
