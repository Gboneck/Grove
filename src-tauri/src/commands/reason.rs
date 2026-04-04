use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::memory;
use crate::models::context::GroveContext;
use crate::models::router::{ModelRouter, ModelStatus};
use crate::models::{ModelSource, ReasoningIntent};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonResponse {
    pub blocks: Vec<Value>,
    pub timestamp: String,
    pub model_source: String,
}

/// State managed by Tauri — holds the model router
pub struct RouterState(pub Arc<Mutex<ModelRouter>>);

#[tauri::command]
pub async fn reason(
    user_input: Option<String>,
    state: tauri::State<'_, RouterState>,
) -> Result<ReasonResponse, String> {
    // 1. Gather context
    let context =
        GroveContext::gather(user_input.clone()).map_err(|e| format!("Context error: {}", e))?;

    // 2. Determine intent
    let intent = match &user_input {
        Some(input) => {
            let lower = input.to_lowercase();
            if lower.contains("plan")
                || lower.contains("prioritize")
                || lower.contains("think hard")
                || lower.contains("strategy")
            {
                ReasoningIntent::PlanAction
            } else {
                ReasoningIntent::RespondToInput(input.clone())
            }
        }
        None => ReasoningIntent::ComposeUI,
    };

    // 3. Route to model
    let router = state.0.lock().await;
    let output = router
        .route(&context, &intent)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Record session in memory
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
                "status" => "Venture Status".to_string(),
                "input" => "Input Prompt".to_string(),
                "divider" => "Divider".to_string(),
                _ => block_type.to_string(),
            };
            Some(label)
        })
        .collect();

    let summary = output
        .session_summary
        .as_deref()
        .unwrap_or("Reasoning cycle completed");
    let insights = output.insights.clone().unwrap_or_default();

    memory::record_session(blocks_shown, user_input.as_deref(), summary, insights).ok();

    let source_str = match output.source {
        ModelSource::Local => "local",
        ModelSource::Cloud => "cloud",
    };

    // 5. Return blocks to frontend
    Ok(ReasonResponse {
        blocks: output.blocks,
        timestamp: Utc::now().to_rfc3339(),
        model_source: source_str.to_string(),
    })
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
