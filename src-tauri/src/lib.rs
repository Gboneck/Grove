mod commands;
mod models;

use std::sync::Arc;
use tokio::sync::Mutex;

use commands::{
    context::{read_context, write_context},
    memory::get_memory,
    reason::{get_model_status, reason, set_model_mode, RouterState},
    soul::{read_soul, write_soul},
    system::get_system_info,
};
use models::config;
use models::router::ModelRouter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize ~/.grove/ directory and default files
    commands::soul::ensure_grove_dir();
    commands::soul::ensure_soul();
    commands::context::ensure_context();
    commands::memory::ensure_memory();
    config::ensure_config();

    // Load .env from ~/.grove/.env if it exists
    let grove_env = dirs::home_dir()
        .map(|h| h.join(".grove").join(".env"))
        .unwrap_or_default();
    if grove_env.exists() {
        dotenvy::from_path(&grove_env).ok();
    }

    // Load config and create model router
    let grove_config = config::load_config();
    let router = ModelRouter::new(grove_config);
    let router_state = RouterState(Arc::new(Mutex::new(router)));

    tauri::Builder::default()
        .manage(router_state)
        .invoke_handler(tauri::generate_handler![
            reason,
            set_model_mode,
            get_model_status,
            read_soul,
            write_soul,
            read_context,
            write_context,
            get_memory,
            get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
