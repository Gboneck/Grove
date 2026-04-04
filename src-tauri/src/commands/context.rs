use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

const DEFAULT_CONTEXT: &str = r#"{ "ventures": [] }"#;

pub fn ensure_context() {
    let path = grove_dir().join("context.json");
    if !path.exists() {
        fs::write(&path, DEFAULT_CONTEXT).expect("Failed to write default context.json");
    }
}

#[tauri::command]
pub async fn read_context() -> Result<Value, String> {
    let path = grove_dir().join("context.json");
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read context.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse context.json: {}", e))
}

#[tauri::command]
pub async fn write_context(context: Value) -> Result<(), String> {
    let path = grove_dir().join("context.json");
    let content = serde_json::to_string_pretty(&context)
        .map_err(|e| format!("Failed to serialize context: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write context.json: {}", e))
}
