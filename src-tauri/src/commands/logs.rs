use chrono::{Local, Utc};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub model_source: String,
    pub intent: String,
    pub confidence: f64,
    pub escalated: bool,
    pub escalation_reason: Option<String>,
    pub blocks_count: usize,
    pub user_input: Option<String>,
    pub duration_ms: u64,
}

pub fn write_reasoning_log(entry: &LogEntry) {
    let logs_dir = grove_dir().join("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir).ok();
    }

    let date = Local::now().format("%Y-%m-%d").to_string();
    let log_path = logs_dir.join(format!("{}.json", date));

    // Read existing log or start fresh
    let mut entries: Vec<serde_json::Value> = if log_path.exists() {
        fs::read_to_string(&log_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    if let Ok(val) = serde_json::to_value(entry) {
        entries.push(val);
    }

    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        fs::write(&log_path, json).ok();
    }
}

#[tauri::command]
pub async fn get_reasoning_logs(date: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let logs_dir = grove_dir().join("logs");
    let date_str = date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let log_path = logs_dir.join(format!("{}.json", date_str));

    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse log: {}", e))?;
    Ok(entries)
}
