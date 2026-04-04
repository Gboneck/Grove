use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use super::autonomous;
use super::logs::{write_reasoning_log, LogEntry};
use super::memory;
use super::reflection;
use super::roles;
use super::ventures;
use crate::memory::working;
use crate::commands::actions::PluginState;
use crate::models::context::GroveContext;
use crate::models::router::{ModelRouter, ModelStatus};
use crate::models::{ModelSource, ReasoningIntent};
use crate::soul::parser::Soul;
use crate::soul::evolution::RelationshipPhase;
use crate::autonomy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonResponse {
    pub blocks: Vec<Value>,
    pub timestamp: String,
    pub model_source: String,
    pub ambient_mood: Option<String>,
    pub theme_hint: Option<String>,
    pub conversation_id: String,
    #[serde(default)]
    pub auto_action_results: Vec<String>,
    #[serde(default)]
    pub venture_update_results: Vec<String>,
}

/// State managed by Tauri — holds the model router and conversation history
pub struct RouterState(pub Arc<Mutex<ModelRouter>>);

/// Conversation state — tracks multi-turn history
pub struct ConversationState(pub Arc<Mutex<Vec<ConversationTurn>>>);

#[tauri::command]
pub async fn reason(
    user_input: Option<String>,
    state: tauri::State<'_, RouterState>,
    conversation: tauri::State<'_, ConversationState>,
    plugin_state: tauri::State<'_, PluginState>,
    role_state: tauri::State<'_, crate::RoleState>,
    cycle_counter: tauri::State<'_, crate::CycleCounter>,
) -> Result<ReasonResponse, String> {
    let start = Instant::now();

    // 0. Validate user input
    let user_input = match user_input {
        Some(raw) => Some(crate::security::validate_user_input(&raw)?),
        None => None,
    };

    // Run on_reason hooks
    {
        let registry = plugin_state.0.lock().await;
        registry.run_hook("on_reason");
    }

    // 1. Gather context (includes plugin data)
    let mut context =
        GroveContext::gather(user_input.clone()).map_err(|e| format!("Context error: {}", e))?;

    // Inject plugin data, digest, and reminders into context
    {
        let registry = plugin_state.0.lock().await;
        let plugin_data = registry.gather_data_context();
        let actions_ctx = registry.actions_context();
        let digest_ctx = reflection::digest_context();
        let reminders_ctx = reflection::reminders_context();
        context.plugin_data = format!("{}{}{}{}", plugin_data, actions_ctx, digest_ctx, reminders_ctx);
    }

    // Inject active role prompt
    {
        let active_role = role_state.0.lock().await;
        if let Some(ref role_name) = *active_role {
            if let Some(role) = roles::get_role(role_name) {
                context.role_prompt = roles::role_prompt_modifier(&role);
            }
        }
    }

    // Inject soul enrichment prompts for early phases
    {
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem = memory::read_memory_file().unwrap_or_default();
        let phase = RelationshipPhase::from_metrics(soul.completeness(), mem.sessions.len() as u32);
        let enrichment = crate::soul::enrichment::enrichment_context(&soul, phase);
        if !enrichment.is_empty() {
            context.plugin_data.push_str(&enrichment);
        }
    }

    // Generate weekly digest if needed (runs once per week)
    if reflection::should_generate_digest() {
        if let Ok(digest) = reflection::generate_weekly_digest() {
            reflection::save_digest(&digest).ok();
            eprintln!("[grove] Weekly digest generated for {}", digest.week_start);
        }
    }

    // 2. Determine intent — use model-based classification when input is present
    let router = state.0.lock().await;
    let intent = match &user_input {
        Some(input) => router.classify_intent(input).await,
        None => ReasoningIntent::ComposeUI,
    };
    drop(router);

    let intent_str = intent.label().to_string();

    // 3. Build conversation context
    let mut conv = conversation.0.lock().await;
    if let Some(ref input) = user_input {
        conv.push(ConversationTurn {
            role: "user".to_string(),
            content: input.clone(),
        });
    }

    // Inject recent conversation turns into context
    if conv.len() > 1 {
        let recent: Vec<String> = conv
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|t| format!("[{}]: {}", t.role, t.content))
            .collect();
        context.conversation_history = Some(recent.join("\n"));
    }

    // 4. Route to model
    let router = state.0.lock().await;
    let output = router
        .route(&context, &intent)
        .await
        .map_err(|e| e.to_string())?;
    drop(router);

    let duration_ms = start.elapsed().as_millis() as u64;

    // Track assistant response in conversation
    let summary_text = output
        .session_summary
        .as_deref()
        .unwrap_or("Reasoning cycle completed")
        .to_string();
    conv.push(ConversationTurn {
        role: "assistant".to_string(),
        content: summary_text.clone(),
    });
    // Keep conversation history bounded
    if conv.len() > 20 {
        let start_idx = conv.len() - 20;
        *conv = conv[start_idx..].to_vec();
    }
    drop(conv);

    // 5. Record session in memory
    let blocks_shown: Vec<String> = output
        .blocks
        .iter()
        .filter_map(|b| {
            let block_type = b.get("type")?.as_str()?;
            let label = match block_type {
                "text" => b.get("heading")?.as_str().unwrap_or("Text").to_string(),
                "metric" => b.get("label")?.as_str().unwrap_or("Metric").to_string(),
                "actions" => b.get("title")?.as_str().unwrap_or("Actions").to_string(),
                "insight" => b.get("message")?.as_str().unwrap_or("Insight").to_string(),
                "progress" => b.get("label")?.as_str().unwrap_or("Progress").to_string(),
                "list" => b.get("heading")?.as_str().unwrap_or("List").to_string(),
                "quote" => "Quote".to_string(),
                "status" => "Venture Status".to_string(),
                "input" => "Input Prompt".to_string(),
                "divider" => "Divider".to_string(),
                _ => block_type.to_string(),
            };
            Some(label)
        })
        .collect();

    let insights = output.insights.clone().unwrap_or_default();

    memory::record_session(blocks_shown, user_input.as_deref(), &summary_text, insights.clone()).ok();

    let source_str = match output.source {
        ModelSource::Local => "local",
        ModelSource::Cloud => "cloud",
    };

    // 5b. Record to MEMORY.md journal (cross-session context)
    working::record_session_summary(
        user_input.as_deref(),
        &summary_text,
        source_str,
        &insights,
    ).ok();

    // 5c. Run soul self-evolution engine (propose → judge → apply)
    {
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem_data = memory::read_memory_file().unwrap_or_default();
        let evo_phase = RelationshipPhase::from_metrics(
            soul.completeness(),
            mem_data.sessions.len() as u32,
        );
        match crate::soul::evolve::EvolutionEngine::run_cycle(&insights, evo_phase) {
            Ok(applied) if !applied.is_empty() => {
                for a in &applied {
                    eprintln!("[grove:evolve] {}", a);
                }
            }
            Ok(_) => {}
            Err(e) => eprintln!("[grove:evolve] Error: {}", e),
        }
    }

    // 6. Write reasoning log
    let log_entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
        intent: intent_str,
        confidence: output.confidence,
        escalated: output.source == ModelSource::Cloud && output.confidence < 0.7,
        escalation_reason: output.escalation_reason.clone(),
        blocks_count: output.blocks.len(),
        user_input: user_input.clone(),
        duration_ms,
    };
    write_reasoning_log(&log_entry);

    // 6b. Sync to Qdrant every 5th reasoning cycle (debounced)
    {
        let count = cycle_counter.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count % 5 == 0 {
            tokio::spawn(async {
                if crate::memory::vector::is_available().await {
                    crate::memory::vector::sync_from_json().await.ok();
                }
            });
        }
    }

    // 7. Execute autonomous actions through autonomy gate
    let auto_action_results = if let Some(ref actions) = output.auto_actions {
        // Determine relationship phase for autonomy gating
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem = memory::read_memory_file().unwrap_or_default();
        let phase = RelationshipPhase::from_metrics(
            soul.completeness(),
            mem.sessions.len() as u32,
        );

        let (approved, blocked) = autonomy::gate_actions(actions, phase);
        for b in &blocked {
            eprintln!("[grove:autonomy] {}", b);
        }

        let mut results = autonomous::execute_auto_actions(&approved);
        for b in blocked {
            results.push(b);
        }
        for result in &results {
            eprintln!("[grove] Auto-action: {}", result);
        }
        results
    } else {
        Vec::new()
    };

    // 8. Apply venture updates from the model
    let venture_update_results = if let Some(ref updates) = output.venture_updates {
        let results = ventures::apply_venture_updates(updates);
        for result in &results {
            eprintln!("[grove] Venture update: {}", result);
        }
        results
    } else {
        Vec::new()
    };

    // 9. Extract ambient state
    let ambient_mood = output.ambient_mood.clone();
    let theme_hint = output.ambient_theme.clone();

    let conversation_id = uuid::Uuid::new_v4().to_string();

    // 10. Return blocks to frontend
    Ok(ReasonResponse {
        blocks: output.blocks,
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
        ambient_mood,
        theme_hint,
        conversation_id,
        auto_action_results,
        venture_update_results,
    })
}

/// Streaming variant of reason — emits blocks one-by-one via Tauri events
#[tauri::command]
pub async fn reason_stream(
    app_handle: tauri::AppHandle,
    user_input: Option<String>,
    state: tauri::State<'_, RouterState>,
    conversation: tauri::State<'_, ConversationState>,
    plugin_state: tauri::State<'_, PluginState>,
    role_state: tauri::State<'_, crate::RoleState>,
    cycle_counter: tauri::State<'_, crate::CycleCounter>,
) -> Result<ReasonResponse, String> {
    use tauri::Emitter;

    let start = Instant::now();

    // 0. Validate user input
    let user_input = match user_input {
        Some(raw) => Some(crate::security::validate_user_input(&raw)?),
        None => None,
    };

    // Run on_reason hooks
    {
        let registry = plugin_state.0.lock().await;
        registry.run_hook("on_reason");
    }

    // 1. Gather context
    let mut context =
        GroveContext::gather(user_input.clone()).map_err(|e| format!("Context error: {}", e))?;

    // Inject plugin data, digest, and reminders
    {
        let registry = plugin_state.0.lock().await;
        let plugin_data = registry.gather_data_context();
        let actions_ctx = registry.actions_context();
        let digest_ctx = reflection::digest_context();
        let reminders_ctx = reflection::reminders_context();
        context.plugin_data = format!("{}{}{}{}", plugin_data, actions_ctx, digest_ctx, reminders_ctx);
    }

    // Inject active role prompt
    {
        let active_role = role_state.0.lock().await;
        if let Some(ref role_name) = *active_role {
            if let Some(role) = roles::get_role(role_name) {
                context.role_prompt = roles::role_prompt_modifier(&role);
            }
        }
    }

    // Inject soul enrichment prompts for early phases
    {
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem = memory::read_memory_file().unwrap_or_default();
        let phase = RelationshipPhase::from_metrics(soul.completeness(), mem.sessions.len() as u32);
        let enrichment = crate::soul::enrichment::enrichment_context(&soul, phase);
        if !enrichment.is_empty() {
            context.plugin_data.push_str(&enrichment);
        }
    }

    // 2. Classify intent
    let router = state.0.lock().await;
    let intent = match &user_input {
        Some(input) => router.classify_intent(input).await,
        None => ReasoningIntent::ComposeUI,
    };
    drop(router);

    let intent_str = intent.label().to_string();

    // 3. Build conversation context
    let mut conv = conversation.0.lock().await;
    if let Some(ref input) = user_input {
        conv.push(ConversationTurn {
            role: "user".to_string(),
            content: input.clone(),
        });
    }

    if conv.len() > 1 {
        let recent: Vec<String> = conv
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|t| format!("[{}]: {}", t.role, t.content))
            .collect();
        context.conversation_history = Some(recent.join("\n"));
    }

    // Emit "stream started" event
    app_handle.emit("reason-stream-start", &serde_json::json!({})).ok();

    // 4. Route with streaming — emit each block as it arrives
    let handle_clone = app_handle.clone();
    let block_emitter = move |block: Value| {
        handle_clone.emit("reason-block", &block).ok();
    };

    let router = state.0.lock().await;
    let output = router
        .route_streaming(&context, &intent, block_emitter)
        .await
        .map_err(|e| e.to_string())?;
    drop(router);

    let duration_ms = start.elapsed().as_millis() as u64;

    // Track in conversation
    let summary_text = output
        .session_summary
        .as_deref()
        .unwrap_or("Reasoning cycle completed")
        .to_string();
    conv.push(ConversationTurn {
        role: "assistant".to_string(),
        content: summary_text.clone(),
    });
    if conv.len() > 20 {
        let start_idx = conv.len() - 20;
        *conv = conv[start_idx..].to_vec();
    }
    drop(conv);

    // Record session in memory
    let blocks_shown: Vec<String> = output
        .blocks
        .iter()
        .filter_map(|b| {
            let block_type = b.get("type")?.as_str()?;
            let label = match block_type {
                "text" => b.get("heading")?.as_str().unwrap_or("Text").to_string(),
                "metric" => b.get("label")?.as_str().unwrap_or("Metric").to_string(),
                "actions" => b.get("title")?.as_str().unwrap_or("Actions").to_string(),
                "insight" => b.get("message")?.as_str().unwrap_or("Insight").to_string(),
                "progress" => b.get("label")?.as_str().unwrap_or("Progress").to_string(),
                "list" => b.get("heading")?.as_str().unwrap_or("List").to_string(),
                "quote" => "Quote".to_string(),
                "status" => "Venture Status".to_string(),
                "input" => "Input Prompt".to_string(),
                "divider" => "Divider".to_string(),
                _ => block_type.to_string(),
            };
            Some(label)
        })
        .collect();

    let insights = output.insights.clone().unwrap_or_default();
    memory::record_session(blocks_shown, user_input.as_deref(), &summary_text, insights.clone()).ok();

    let source_str = match output.source {
        ModelSource::Local => "local",
        ModelSource::Cloud => "cloud",
    };

    // Record to MEMORY.md journal
    working::record_session_summary(
        user_input.as_deref(),
        &summary_text,
        source_str,
        &insights,
    ).ok();

    // Run soul self-evolution engine (propose → judge → apply)
    {
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem_data = memory::read_memory_file().unwrap_or_default();
        let evo_phase = RelationshipPhase::from_metrics(
            soul.completeness(),
            mem_data.sessions.len() as u32,
        );
        match crate::soul::evolve::EvolutionEngine::run_cycle(&insights, evo_phase) {
            Ok(applied) if !applied.is_empty() => {
                for a in &applied {
                    eprintln!("[grove:evolve] {}", a);
                }
            }
            Ok(_) => {}
            Err(e) => eprintln!("[grove:evolve] Error: {}", e),
        }
    }

    let log_entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
        intent: intent_str,
        confidence: output.confidence,
        escalated: output.source == ModelSource::Cloud && output.confidence < 0.7,
        escalation_reason: output.escalation_reason.clone(),
        blocks_count: output.blocks.len(),
        user_input: user_input.clone(),
        duration_ms,
    };
    write_reasoning_log(&log_entry);

    // Sync to Qdrant every 5th reasoning cycle (debounced)
    {
        let count = cycle_counter.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count % 5 == 0 {
            tokio::spawn(async {
                if crate::memory::vector::is_available().await {
                    crate::memory::vector::sync_from_json().await.ok();
                }
            });
        }
    }

    // Execute autonomous actions through autonomy gate
    let auto_action_results = if let Some(ref actions) = output.auto_actions {
        let soul_raw = std::fs::read_to_string(
            dirs::home_dir().unwrap_or_default().join(".grove").join("soul.md")
        ).unwrap_or_default();
        let soul = Soul::parse(&soul_raw);
        let mem = memory::read_memory_file().unwrap_or_default();
        let phase = RelationshipPhase::from_metrics(
            soul.completeness(),
            mem.sessions.len() as u32,
        );

        let (approved, blocked) = autonomy::gate_actions(actions, phase);
        for b in &blocked {
            eprintln!("[grove:autonomy] {}", b);
        }

        let mut results = autonomous::execute_auto_actions(&approved);
        results.extend(blocked);
        results
    } else {
        Vec::new()
    };

    // Apply venture updates
    let venture_update_results = if let Some(ref updates) = output.venture_updates {
        let results = ventures::apply_venture_updates(updates);
        for result in &results {
            eprintln!("[grove] Venture update: {}", result);
        }
        results
    } else {
        Vec::new()
    };

    let ambient_mood = output.ambient_mood.clone();
    let theme_hint = output.ambient_theme.clone();
    let conversation_id = uuid::Uuid::new_v4().to_string();

    let response = ReasonResponse {
        blocks: output.blocks,
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
        ambient_mood,
        theme_hint,
        conversation_id,
        auto_action_results,
        venture_update_results,
    };

    // Emit completion event
    app_handle.emit("reason-stream-complete", &response).ok();

    Ok(response)
}

/// Clear conversation history (e.g., when user refreshes without input)
#[tauri::command]
pub async fn clear_conversation(
    conversation: tauri::State<'_, ConversationState>,
) -> Result<(), String> {
    let mut conv = conversation.0.lock().await;
    conv.clear();
    Ok(())
}

#[tauri::command]
pub async fn set_model_mode(
    mode: String,
    state: tauri::State<'_, RouterState>,
) -> Result<(), String> {
    use crate::models::router::ModelMode;
    let new_mode = match mode.as_str() {
        "local_only" => ModelMode::LocalOnly,
        "cloud_only" => ModelMode::CloudOnly,
        _ => ModelMode::Auto,
    };
    let mut router = state.0.lock().await;
    router.set_mode(new_mode);
    Ok(())
}

#[tauri::command]
pub async fn get_model_status(
    state: tauri::State<'_, RouterState>,
) -> Result<ModelStatus, String> {
    let router = state.0.lock().await;
    Ok(router.status().await)
}
