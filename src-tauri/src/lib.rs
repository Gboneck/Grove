mod commands;
mod models;
mod plugins;

use std::sync::Arc;
use tokio::sync::Mutex;

use commands::{
    actions::{execute_action, list_actions, PluginState},
    context::{read_context, write_context},
    identity::{generate_soul, is_soul_personalized},
    logs::get_reasoning_logs,
    memory::{get_full_memory, get_memory, get_memory_stats, record_action_engagement},
    reason::{clear_conversation, get_model_status, reason, set_model_mode, ConversationState, RouterState},
    setup::{check_setup, save_api_key},
    soul::{read_soul, write_soul},
    system::get_system_info,
    watch::get_file_stamps,
};
use models::config;
use models::router::ModelRouter;
use plugins::loader;
use plugins::registry::PluginRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize ~/.grove/ directory and default files
    commands::soul::ensure_grove_dir();
    commands::soul::ensure_soul();
    commands::context::ensure_context();
    commands::memory::ensure_memory();
    config::ensure_config();
    loader::ensure_plugins_dir();

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

    // Load plugins
    let plugin_manifests = loader::load_plugins();
    let plugin_count = plugin_manifests.len();
    let registry = PluginRegistry::new(plugin_manifests);
    let plugin_state = PluginState(Arc::new(Mutex::new(registry.clone())));

    // Run on_startup hooks
    registry.run_hook("on_startup");

    if plugin_count > 0 {
        eprintln!("[grove] Loaded {} plugin(s)", plugin_count);
    }

    // Initialize conversation state
    let conversation_state = ConversationState(Arc::new(Mutex::new(Vec::new())));

    tauri::Builder::default()
        .manage(router_state)
        .manage(plugin_state)
        .manage(conversation_state)
        .invoke_handler(tauri::generate_handler![
            reason,
            set_model_mode,
            get_model_status,
            read_soul,
            write_soul,
            read_context,
            write_context,
            get_memory,
            get_memory_stats,
            record_action_engagement,
            get_system_info,
            check_setup,
            save_api_key,
            get_reasoning_logs,
            get_file_stamps,
            generate_soul,
            is_soul_personalized,
            execute_action,
            list_actions,
            get_full_memory,
            clear_conversation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
