use chrono::Local;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub current_time: String,
    pub day_of_week: String,
    pub date: String,
    pub hostname: String,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let now = Local::now();
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        current_time: now.to_rfc3339(),
        day_of_week: now.format("%A").to_string(),
        date: now.format("%B %-d, %Y").to_string(),
        hostname,
    })
}
