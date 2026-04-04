mod commands;
mod models;
mod plugins;

use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use commands::{
    actions::{execute_action, list_actions, list_plugins, set_plugin_enabled, PluginState},
    context::{read_context, write_context},
    identity::{generate_soul, is_soul_personalized},
    logs::get_reasoning_logs,
    mcp::{mcp_call_tool, mcp_list_tools},
    memory::{get_full_memory, get_memory, get_memory_stats, record_action_engagement},
    profiles::{create_profile, delete_profile, list_profiles, switch_profile},
    reason::{
        clear_conversation, get_model_status, reason, set_model_mode, ConversationState,
        RouterState,
    },
    setup::{check_setup, save_api_key},
    soul::{read_soul, write_soul},
    system::get_system_info,
    watch::{get_file_stamps, notify_file_change},
};
use models::config;
use models::context::GroveContext;
use models::router::ModelRouter;
use models::{ReasoningIntent, ModelSource};
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
    commands::profiles::ensure_profiles_dir();

    // Load .env from ~/.grove/.env if it exists
    let grove_env = dirs::home_dir()
        .map(|h| h.join(".grove").join(".env"))
        .unwrap_or_default();
    if grove_env.exists() {
        dotenvy::from_path(&grove_env).ok();
    }

    // Load config and create model router
    let grove_config = config::load_config();
    let periodic_minutes = grove_config.models.periodic_reasoning_minutes;
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
        .plugin(tauri_plugin_notification::init())
        .manage(router_state)
        .manage(plugin_state)
        .manage(conversation_state)
        .setup(move |app| {
            // Start periodic reasoning timer if configured
            if periodic_minutes > 0 {
                let router_arc = app.state::<RouterState>().0.clone();
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let duration = std::time::Duration::from_secs(periodic_minutes * 60);
                    let mut interval = tokio::time::interval(duration);
                    interval.tick().await; // skip first immediate tick
                    loop {
                        interval.tick().await;
                        // Try to gather context and reason
                        let context = match GroveContext::gather(None) {
                            Ok(ctx) => ctx,
                            Err(e) => {
                                eprintln!("[grove] Periodic reasoning context error: {}", e);
                                continue;
                            }
                        };
                        // Use try_lock to avoid blocking if a user-initiated reason is running
                        let router: tokio::sync::MutexGuard<'_, ModelRouter> = match router_arc.try_lock() {
                            Ok(r) => r,
                            Err(_) => {
                                eprintln!("[grove] Periodic reasoning skipped — router busy");
                                continue;
                            }
                        };
                        match router.route(&context, &ReasoningIntent::Reflect).await {
                            Ok(output) => {
                                let source_str = match output.source {
                                    ModelSource::Local => "local",
                                    ModelSource::Cloud => "cloud",
                                };

                                // Extract notification text from insights or summary
                                let notif_body = output.insights.as_ref()
                                    .and_then(|ins| ins.first().cloned())
                                    .or_else(|| output.session_summary.clone())
                                    .unwrap_or_else(|| "New reasoning update available".to_string());

                                // Check for urgent/important blocks
                                let has_urgent = output.blocks.iter().any(|b| {
                                    b.get("icon").and_then(|v| v.as_str()) == Some("warning")
                                        || b.get("icon").and_then(|v| v.as_str()) == Some("alert")
                                        || output.ambient_mood.as_deref() == Some("urgent")
                                });

                                let payload = serde_json::json!({
                                    "blocks": output.blocks,
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                    "model_source": source_str,
                                    "ambient_mood": output.ambient_mood,
                                    "theme_hint": output.ambient_theme,
                                    "has_urgent": has_urgent,
                                });
                                handle.emit("periodic-reasoning", &payload).ok();

                                // Send desktop notification
                                use tauri_plugin_notification::NotificationExt;
                                let title = if has_urgent { "Grove — Needs Attention" } else { "Grove" };
                                handle.notification()
                                    .builder()
                                    .title(title)
                                    .body(&notif_body)
                                    .show()
                                    .ok();
                            }
                            Err(e) => {
                                eprintln!("[grove] Periodic reasoning failed: {}", e);
                            }
                        }
                    }
                });
                eprintln!("[grove] Periodic reasoning enabled: every {} min", periodic_minutes);
            }
            Ok(())
        })
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
            notify_file_change,
            generate_soul,
            is_soul_personalized,
            execute_action,
            list_actions,
            get_full_memory,
            clear_conversation,
            list_plugins,
            set_plugin_enabled,
            list_profiles,
            switch_profile,
            create_profile,
            delete_profile,
            mcp_list_tools,
            mcp_call_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
