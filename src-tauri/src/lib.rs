mod commands;
mod models;
mod plugins;
pub mod soul;
pub mod heartbeat;
pub mod autonomy;
pub mod memory;
pub mod security;

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
    reflection::{generate_and_save_digest, get_weekly_digest, dismiss_reminder, snooze_reminder},
    reason::{
        clear_conversation, get_model_status, reason, reason_stream, set_model_mode,
        record_prompt_copied, ConversationState, RouterState,
    },
    roles::{list_roles, get_active_role, set_active_role},
    setup::{check_setup, save_api_key},
    soul::{read_soul, write_soul},
    system::get_system_info,
    vector::{vector_status, vector_sync, vector_search},
    watch::{get_file_stamps, notify_file_change},
    workspace::{
        load_workspace, save_workspace, remove_artifact, WorkspaceState,
    },
};
use models::config;
use models::context::GroveContext;
use models::router::ModelRouter;
use models::{ReasoningIntent, ModelSource};
use plugins::loader;
use plugins::registry::PluginRegistry;

use memory::ephemeral::EphemeralMemory;
use commands::enrichment::answer_enrichment;
use soul::evolve::{get_evolution_proposals, apply_evolution};
use soul::enrichment::get_enrichment_prompts;

/// Shared ephemeral memory state for the current session.
pub struct EphemeralState(pub Arc<Mutex<EphemeralMemory>>);

/// Reasoning cycle counter for debouncing vector sync.
pub struct CycleCounter(pub Arc<std::sync::atomic::AtomicU64>);

/// Active reasoning role state (e.g., "builder", "reflector").
pub struct RoleState(pub Arc<Mutex<Option<String>>>);

/// Shared heartbeat state.
pub struct HeartbeatStateWrapper(pub Arc<heartbeat::HeartbeatState>);

/// Re-export ContextCache for use as managed state.
pub use models::context::ContextCache;

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
    memory::working::ensure_memory_md();
    memory::longterm::ensure_longterm_dir();
    commands::system::ensure_system_md();

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
    let heartbeat_interval = grove_config.models.periodic_reasoning_minutes.max(1) * 60; // reuse config
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

    // Initialize conversation state — load from disk if recent
    let prior_turns = commands::reason::load_conversation();
    let conversation_state = ConversationState(Arc::new(Mutex::new(prior_turns)));

    // Initialize workspace state — persistent canvas
    let workspace = commands::workspace::load_workspace_from_disk();
    let workspace_state = WorkspaceState(Arc::new(Mutex::new(workspace)));
    eprintln!("[grove] Loaded workspace with {} artifacts", workspace_state.0.try_lock().map(|w| w.artifacts.len()).unwrap_or(0));

    // Initialize ephemeral memory for this session
    let ephemeral_state = EphemeralState(Arc::new(Mutex::new(EphemeralMemory::new())));

    // Initialize role state (no active role by default)
    let role_state = RoleState(Arc::new(Mutex::new(None)));

    // Initialize cycle counter for Qdrant sync debouncing
    let cycle_counter = CycleCounter(Arc::new(std::sync::atomic::AtomicU64::new(0)));

    // Initialize context cache — pre-warmed on startup
    let context_cache = ContextCache::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(router_state)
        .manage(plugin_state)
        .manage(conversation_state)
        .manage(ephemeral_state)
        .manage(role_state)
        .manage(cycle_counter)
        .manage(context_cache)
        .manage(workspace_state)
        .setup(move |app| {
            // Start the heartbeat background loop
            let grove_dir = dirs::home_dir()
                .expect("No home directory")
                .join(".grove");
            let hb_state = heartbeat::start_heartbeat(
                grove_dir,
                heartbeat_interval, // tick interval in seconds
                5,                  // observation threshold
                app.handle().clone(),
            );
            eprintln!("[grove] Heartbeat started: {}s interval", heartbeat_interval);

            // Pre-warm the context cache in the background
            {
                use tauri::Manager;
                let cache = app.state::<ContextCache>().0.clone();
                tauri::async_runtime::spawn(async move {
                    let start = std::time::Instant::now();
                    match GroveContext::gather(None) {
                        Ok(ctx) => {
                            let mut c = cache.lock().await;
                            *c = Some(ctx);
                            eprintln!("[grove] Context cache warmed in {}ms", start.elapsed().as_millis());
                        }
                        Err(e) => eprintln!("[grove] Context pre-warm failed: {}", e),
                    }
                });
            }

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

                                // Record to MEMORY.md
                                let insights = output.insights.clone().unwrap_or_default();
                                memory::working::record_session_summary(
                                    None,
                                    &output.session_summary.as_deref().unwrap_or("Periodic reasoning"),
                                    source_str,
                                    &insights,
                                ).ok();

                                // Save as pending thought for next interactive session
                                {
                                    let thought_path = dirs::home_dir()
                                        .unwrap_or_default()
                                        .join(".grove")
                                        .join("pending_thoughts.json");
                                    let mut thoughts: Vec<serde_json::Value> = std::fs::read_to_string(&thought_path)
                                        .ok()
                                        .and_then(|c| serde_json::from_str(&c).ok())
                                        .unwrap_or_default();
                                    thoughts.push(serde_json::json!({
                                        "timestamp": chrono::Utc::now().to_rfc3339(),
                                        "summary": output.session_summary,
                                        "insights": insights,
                                        "blocks": output.blocks,
                                        "model_source": source_str,
                                    }));
                                    // Keep last 5 thoughts
                                    if thoughts.len() > 5 {
                                        thoughts = thoughts[thoughts.len()-5..].to_vec();
                                    }
                                    if let Ok(json) = serde_json::to_string_pretty(&thoughts) {
                                        std::fs::write(&thought_path, json).ok();
                                    }
                                }

                                let payload = serde_json::json!({
                                    "blocks": output.blocks,
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                    "model_source": source_str,
                                    "ambient_mood": output.ambient_mood,
                                    "theme_hint": output.ambient_theme,
                                    "has_urgent": has_urgent,
                                });
                                handle.emit("periodic-reasoning", &payload).ok();

                                // Desktop notifications disabled — they interrupt reading
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
            reason_stream,
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
            get_weekly_digest,
            generate_and_save_digest,
            dismiss_reminder,
            snooze_reminder,
            list_roles,
            get_active_role,
            set_active_role,
            get_evolution_proposals,
            apply_evolution,
            vector_status,
            vector_sync,
            vector_search,
            get_enrichment_prompts,
            answer_enrichment,
            load_workspace,
            save_workspace,
            remove_artifact,
            record_prompt_copied,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
