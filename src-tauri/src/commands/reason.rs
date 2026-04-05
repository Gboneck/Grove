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

/// Conversation state — tracks multi-turn history, persisted to disk.
pub struct ConversationState(pub Arc<Mutex<Vec<ConversationTurn>>>);

const CONVERSATION_FILE: &str = "conversation.json";
const CONVERSATION_MAX_AGE_HOURS: i64 = 12;

/// Load persisted conversation from disk, if recent enough.
pub fn load_conversation() -> Vec<ConversationTurn> {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".grove").join(CONVERSATION_FILE),
        None => return Vec::new(),
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    #[derive(serde::Deserialize)]
    struct Saved {
        timestamp: String,
        turns: Vec<ConversationTurn>,
    }

    let saved: Saved = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Check age — don't load stale conversations
    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&saved.timestamp) {
        let age = Utc::now() - ts.with_timezone(&Utc);
        if age.num_hours() > CONVERSATION_MAX_AGE_HOURS {
            eprintln!("[grove] Conversation too old ({}h), starting fresh", age.num_hours());
            return Vec::new();
        }
    }

    eprintln!("[grove] Loaded {} conversation turns from disk", saved.turns.len());
    saved.turns
}

/// Persist conversation turns to disk.
pub fn save_conversation(turns: &[ConversationTurn]) {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".grove").join(CONVERSATION_FILE),
        None => return,
    };

    let payload = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "turns": turns,
    });

    if let Ok(content) = serde_json::to_string_pretty(&payload) {
        std::fs::write(&path, content).ok();
    }
}

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

    // 2. Determine intent (fast heuristic — no model call)
    let router = state.0.lock().await;
    let intent = match &user_input {
        Some(input) => router.classify_intent(input),
        None => ReasoningIntent::ComposeUI,
    };

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
    use tauri::{Emitter, Manager};

    let start = Instant::now();

    // 0. Validate user input
    let user_input = match user_input {
        Some(raw) => Some(crate::security::validate_user_input(&raw)?),
        None => None,
    };

    // Emit stream-start immediately so frontend shows loading state
    app_handle.emit("reason-stream-start", &serde_json::json!({})).ok();
    app_handle.emit("reason-progress", "gathering context").ok();

    // Run on_reason hooks
    {
        let registry = plugin_state.0.lock().await;
        registry.run_hook("on_reason");
    }

    // 1. Gather context — use warm cache when available, fall back to fresh gather
    let context_cache = app_handle.state::<crate::ContextCache>();
    let mut context = context_cache
        .get_or_gather(user_input.clone())
        .await
        .map_err(|e| format!("Context error: {}", e))?;

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

    // Inject any pending source reads from previous cycle's read_source actions
    if let Some(eph_state) = app_handle.try_state::<crate::EphemeralState>() {
        if let Ok(mut eph) = eph_state.0.try_lock() {
            if !eph.source_reads.is_empty() {
                let reads = std::mem::take(&mut eph.source_reads);
                context.plugin_data.push_str("\n--- SOURCE CODE (from read_source) ---\n");
                for read in &reads {
                    context.plugin_data.push_str(read);
                    context.plugin_data.push('\n');
                }
            }
        }
    }

    // 2. Classify intent (fast heuristic — no model call)
    let router = state.0.lock().await;
    let intent = match &user_input {
        Some(input) => router.classify_intent(input),
        None => ReasoningIntent::ComposeUI,
    };

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

    // 4. Route with streaming — emit each block as it arrives
    app_handle.emit("reason-progress", "thinking").ok();

    let handle_clone = app_handle.clone();
    let first_block = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let first_block_clone = first_block.clone();
    let block_emitter = move |block: Value| {
        if !first_block_clone.swap(true, std::sync::atomic::Ordering::Relaxed) {
            handle_clone.emit("reason-progress", "streaming").ok();
        }
        handle_clone.emit("reason-block", &block).ok();
    };

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
    // Persist conversation to disk
    save_conversation(&conv);
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

        // Store read_source results in ephemeral memory for next reasoning cycle
        let source_results: Vec<String> = results.iter()
            .filter(|r| r.starts_with("[source:"))
            .cloned()
            .collect();
        if !source_results.is_empty() {
            if let Some(eph_state) = app_handle.try_state::<crate::EphemeralState>() {
                if let Ok(mut eph) = eph_state.0.try_lock() {
                    eph.source_reads.extend(source_results);
                }
            }
        }

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

    // Refresh context cache in background for next cycle
    {
        let cache = app_handle.state::<crate::ContextCache>().0.clone();
        tokio::spawn(async move {
            match GroveContext::gather(None) {
                Ok(ctx) => { *cache.lock().await = Some(ctx); }
                Err(_) => {}
            }
        });
    }

    Ok(response)
}

/// Clear conversation history (e.g., when user refreshes without input)
#[tauri::command]
pub async fn clear_conversation(
    conversation: tauri::State<'_, ConversationState>,
) -> Result<(), String> {
    let mut conv = conversation.0.lock().await;
    conv.clear();
    save_conversation(&conv);
    Ok(())
}

#[tauri::command]
pub async fn record_prompt_copied(
    title: String,
    prompt_preview: String,
) -> Result<(), String> {
    let path = dirs::home_dir()
        .ok_or("No home dir")?
        .join(".grove")
        .join("prompt_history.json");

    let mut history: Vec<serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    history.push(serde_json::json!({
        "title": title,
        "preview": prompt_preview,
        "copied_at": Utc::now().to_rfc3339(),
        "status": "copied",
    }));

    // Keep last 20
    if history.len() > 20 {
        history = history[history.len() - 20..].to_vec();
    }

    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    eprintln!("[grove] Prompt copied: {}", title);
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
