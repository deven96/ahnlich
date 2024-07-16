mod ai;
mod db;

pub(crate) use ai::trace_ai_server_response_enum;
pub(crate) use db::trace_db_server_response_enum;

pub(crate) fn save_server_response_registries(output_dir: &std::path::Path) {
    let response_path = output_dir.join("response");
    let _ = std::fs::create_dir_all(&response_path);
    let db_server_res = trace_db_server_response_enum();
    super::save_registry_into_file(&db_server_res, response_path.join("db_response.json"));

    let ai_server_res = trace_ai_server_response_enum();
    super::save_registry_into_file(&ai_server_res, response_path.join("ai_response.json"));
}
