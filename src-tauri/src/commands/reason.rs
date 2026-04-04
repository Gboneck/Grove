use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::memory;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

fn get_api_key() -> Result<String, String> {
    // Try environment variable first
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // Try ~/.grove/.env
    let env_path = grove_dir().join(".env");
    if env_path.exists() {
        if let Ok(content) = fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix("ANTHROPIC_API_KEY=") {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }

    Err("ANTHROPIC_API_KEY not set. Add it to ~/.grove/.env".to_string())
}

const SYSTEM_PROMPT: &str = r#"You are the reasoning engine for Grove OS, a local-first personal operating system.
You receive the user's Soul.md (identity), context.json (ventures/projects),
memory of recent sessions, and current system state.

Your job: decide what to show this person RIGHT NOW.

You are not a chatbot. You are not an assistant. You are the brain of their
operating system. You see their full context and decide what deserves their
attention at this moment.

Rules:
- Return ONLY valid JSON. No markdown, no preamble, no backticks.
- Return: { "blocks": [...], "session_summary": "one sentence describing this interaction", "insights": ["any new observations about patterns or state changes"] }
- Each block has a "type" field: text, metric, actions, status, insight, input, divider

Block schemas:
{ "type": "text", "heading": "string", "body": "string" }
{ "type": "metric", "label": "string", "value": "string", "trend": "up|down|flat|null" }
{ "type": "actions", "title": "string", "items": [{ "action": "string", "detail": "string" }] }
{ "type": "status", "items": [{ "name": "string", "status": "green|yellow|red", "detail": "string" }] }
{ "type": "insight", "icon": "alert|opportunity|warning|idea", "message": "string" }
{ "type": "input", "prompt": "string", "placeholder": "string" }
{ "type": "divider" }

Behavioral rules:
- Time-aware: morning = briefing/priorities. Afternoon = progress check. Evening = reflection/planning.
- Memory-aware: reference past sessions naturally. "Yesterday you said X" or "You've been focused on Y all week."
- Honest: if the user is spreading too thin or avoiding the priority, say so.
- Concise: never more than 8-10 blocks. Density kills usefulness.
- Opinionated: don't show everything. Show what matters. Make judgment calls.
- Voice: direct, warm, no bullshit. Like a sharp cofounder who knows you well."#;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReasonResponse {
    pub blocks: Vec<Value>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeApiResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReasoningResult {
    blocks: Vec<Value>,
    session_summary: Option<String>,
    insights: Option<Vec<String>>,
}

async fn call_claude(system_prompt: &str, user_message: &str) -> Result<String, String> {
    let api_key = get_api_key()?;
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 2000,
            "system": system_prompt,
            "messages": [{"role": "user", "content": user_message}]
        }))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(format!("Claude API error ({}): {}", status, body));
    }

    let api_response: ClaudeApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    api_response
        .content
        .first()
        .and_then(|c| c.text.clone())
        .ok_or_else(|| "Empty response from Claude".to_string())
}

#[tauri::command]
pub async fn reason(user_input: Option<String>) -> Result<ReasonResponse, String> {
    // 1. Read soul.md
    let soul_path = grove_dir().join("soul.md");
    let soul_content =
        fs::read_to_string(&soul_path).map_err(|e| format!("Failed to read soul.md: {}", e))?;

    // 2. Read context.json
    let context_path = grove_dir().join("context.json");
    let context_content = fs::read_to_string(&context_path)
        .map_err(|e| format!("Failed to read context.json: {}", e))?;

    // 3. Read memory
    let memory_data = memory::read_memory_file().unwrap_or_default();
    let recent_sessions: Vec<_> = memory_data.sessions.iter().rev().take(5).collect();

    // 4. Build time context
    let now = Local::now();
    let last_seen_str = memory_data
        .last_seen
        .as_deref()
        .unwrap_or("never — this is the first session");

    // 5. Format recent memory for prompt
    let memory_text = if recent_sessions.is_empty() {
        "No previous sessions.".to_string()
    } else {
        recent_sessions
            .iter()
            .map(|s| {
                let inputs: Vec<String> = s.user_inputs.iter().map(|i| i.text.clone()).collect();
                format!(
                    "- {} ({}): {} | User said: {}",
                    s.timestamp,
                    s.time_of_day,
                    s.session_summary,
                    if inputs.is_empty() {
                        "(no input)".to_string()
                    } else {
                        inputs.join(", ")
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let insights_text = if memory_data.accumulated_insights.is_empty() {
        "None yet.".to_string()
    } else {
        memory_data
            .accumulated_insights
            .iter()
            .map(|i| format!("- {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // 6. Assemble user message
    let user_message = format!(
        r#"Current time: {}
Day: {}, {}
Time since last session: {}

--- SOUL.MD ---
{}

--- ACTIVE CONTEXT ---
{}

--- RECENT MEMORY ---
{}

--- ACCUMULATED INSIGHTS ---
{}

{}

Decide what to show. Return JSON only."#,
        now.to_rfc3339(),
        now.format("%A"),
        now.format("%B %-d, %Y"),
        last_seen_str,
        soul_content,
        context_content,
        memory_text,
        insights_text,
        user_input
            .as_ref()
            .map(|i| format!("--- USER INPUT ---\n{}", i))
            .unwrap_or_default()
    );

    // 7. Call Claude
    let raw_response = call_claude(SYSTEM_PROMPT, &user_message).await?;

    // 8. Parse JSON response
    let result: ReasoningResult = serde_json::from_str(&raw_response)
        .map_err(|e| format!("Failed to parse reasoning response: {}. Raw: {}", e, raw_response))?;

    // 9. Record session in memory
    let blocks_shown: Vec<String> = result
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

    let summary = result
        .session_summary
        .as_deref()
        .unwrap_or("Reasoning cycle completed");
    let insights = result.insights.clone().unwrap_or_default();

    memory::record_session(
        blocks_shown,
        user_input.as_deref(),
        summary,
        insights,
    )
    .ok(); // Don't fail the whole request if memory write fails

    // 10. Return blocks to frontend
    Ok(ReasonResponse {
        blocks: result.blocks,
        timestamp: Utc::now().to_rfc3339(),
    })
}
