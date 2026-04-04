use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use super::logs::{write_reasoning_log, LogEntry};
use super::memory;
use crate::commands::actions::PluginState;
use crate::models::context::GroveContext;
use crate::models::router::{ModelRouter, ModelStatus};
use crate::models::{ModelSource, ReasoningIntent};

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
) -> Result<ReasonResponse, String> {
    let start = Instant::now();

    // Run on_reason hooks
    {
        let registry = plugin_state.0.lock().await;
        registry.run_hook("on_reason");
    }

    // 1. Gather context (includes plugin data)
    let mut context =
        GroveContext::gather(user_input.clone()).map_err(|e| format!("Context error: {}", e))?;

    // Inject plugin data into context
    {
        let registry = plugin_state.0.lock().await;
        let plugin_data = registry.gather_data_context();
        let actions_ctx = registry.actions_context();
        if !plugin_data.is_empty() || !actions_ctx.is_empty() {
            context.plugin_data = format!("{}{}", plugin_data, actions_ctx);
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

    memory::record_session(blocks_shown, user_input.as_deref(), &summary_text, insights).ok();

    let source_str = match output.source {
        ModelSource::Local => "local",
        ModelSource::Cloud => "cloud",
    };

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

    // 7. Extract ambient state
    let ambient_mood = output.ambient_mood.clone();
    let theme_hint = output.ambient_theme.clone();

    let conversation_id = uuid::Uuid::new_v4().to_string();

    // 8. Return blocks to frontend
    Ok(ReasonResponse {
        blocks: output.blocks,
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
        ambient_mood,
        theme_hint,
        conversation_id,
    })
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
