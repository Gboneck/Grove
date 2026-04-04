use chrono::Utc;
use serde_json::Value;
use std::fs;

use crate::models::AutoAction;

/// Execute a list of autonomous actions from the reasoning engine.
/// Returns descriptions of what was done.
pub fn execute_auto_actions(actions: &[AutoAction]) -> Vec<String> {
    let mut results = Vec::new();

    for action in actions {
        match action.action_type.as_str() {
            "note" => {
                if let Some(result) = execute_note(action) {
                    results.push(result);
                }
            }
            "reminder" => {
                if let Some(result) = execute_reminder(action) {
                    results.push(result);
                }
            }
            "file_write" => {
                if let Some(result) = execute_file_write(action) {
                    results.push(result);
                }
            }
            "venture_status" => {
                // Handled separately via venture_updates
                results.push(format!("Venture update queued: {}", action.description));
            }
            "add_fact" => {
                if let Some(result) = execute_add_fact(action) {
                    results.push(result);
                }
            }
            other => {
                eprintln!("[grove] Unknown auto_action type: {}", other);
            }
        }
    }

    results
}

/// Write a note to ~/.grove/notes/
fn execute_note(action: &AutoAction) -> Option<String> {
    let grove_dir = dirs::home_dir()?.join(".grove");
    let notes_dir = grove_dir.join("notes");
    fs::create_dir_all(&notes_dir).ok()?;

    let content = action
        .params
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or(&action.description);

    let title = action
        .params
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("note");

    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("{}-{}.md", timestamp, sanitize_filename(title));
    let path = notes_dir.join(&filename);

    let full_content = format!(
        "# {}\n\n{}\n\n---\n*Auto-created by Grove at {}*\n",
        title,
        content,
        Utc::now().to_rfc3339()
    );

    fs::write(&path, full_content).ok()?;
    Some(format!("Created note: {}", filename))
}

/// Write a reminder to ~/.grove/reminders.json
fn execute_reminder(action: &AutoAction) -> Option<String> {
    let grove_dir = dirs::home_dir()?.join(".grove");
    let path = grove_dir.join("reminders.json");

    let mut reminders: Vec<Value> = if path.exists() {
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    let when = action
        .params
        .get("when")
        .and_then(|v| v.as_str())
        .unwrap_or("next_session");

    reminders.push(serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "message": action.description,
        "when": when,
        "created_at": Utc::now().to_rfc3339(),
        "dismissed": false,
    }));

    let content = serde_json::to_string_pretty(&reminders).ok()?;
    fs::write(&path, content).ok()?;
    Some(format!("Set reminder: {}", action.description))
}

/// Write content to a specified file
fn execute_file_write(action: &AutoAction) -> Option<String> {
    let file_path = action.params.get("path")?.as_str()?;
    let content = action
        .params
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Security: validate path and enforce grove directory restriction
    let expanded = match crate::security::validate_file_path(file_path, true) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[grove] Auto file_write blocked: {}", e);
            return None;
        }
    };

    // Ensure parent dir exists
    if let Some(parent) = std::path::Path::new(&expanded).parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(&expanded, content).ok()?;
    Some(format!("Wrote file: {}", file_path))
}

/// Add a semantic fact to memory
fn execute_add_fact(action: &AutoAction) -> Option<String> {
    let category = action
        .params
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("identity");
    let content = action
        .params
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or(&action.description);

    crate::commands::memory::upsert_fact(category, content, "auto_action").ok()?;
    Some(format!("Learned fact: {}", content))
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}
