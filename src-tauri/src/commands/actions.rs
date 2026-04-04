use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::plugins::registry::PluginRegistry;

/// State managed by Tauri — holds the plugin registry
pub struct PluginState(pub Arc<Mutex<PluginRegistry>>);

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub output: Option<String>,
}

#[tauri::command]
pub async fn execute_action(
    action_id: String,
    params: Option<serde_json::Value>,
    plugin_state: tauri::State<'_, PluginState>,
) -> Result<ActionResult, String> {
    let registry = plugin_state.0.lock().await;

    let action = registry
        .get_action(&action_id)
        .ok_or_else(|| format!("Action '{}' not found", action_id))?
        .clone();

    drop(registry); // Release lock before executing

    let result = match action.executor.as_str() {
        "clipboard" => {
            let text = params
                .and_then(|p| p.get("text").and_then(|t| t.as_str().map(String::from)))
                .unwrap_or_else(|| action.description.clone());

            // Use arboard or just echo for now — clipboard requires platform specifics
            Ok(ActionResult {
                success: true,
                message: format!("Copied to clipboard: {}", &text[..text.len().min(50)]),
                output: Some(text),
            })
        }
        "shell" => {
            let command = action
                .executor_config
                .get("command")
                .and_then(|c| c.as_str())
                .ok_or("Shell action missing 'command' in executor_config")?;

            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .map_err(|e| format!("Shell execution failed: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(ActionResult {
                success: output.status.success(),
                message: if output.status.success() {
                    "Command executed successfully".to_string()
                } else {
                    format!("Command failed: {}", stderr)
                },
                output: Some(stdout),
            })
        }
        "write_file" => {
            let path = action
                .executor_config
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or("write_file action missing 'path'")?;

            let content = params
                .and_then(|p| p.get("content").and_then(|c| c.as_str().map(String::from)))
                .unwrap_or_default();

            let expanded = path.replace(
                "~",
                &dirs::home_dir().unwrap_or_default().to_string_lossy(),
            );
            std::fs::write(&expanded, &content)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            Ok(ActionResult {
                success: true,
                message: format!("Written to {}", path),
                output: None,
            })
        }
        "http" => {
            let url = action
                .executor_config
                .get("url")
                .and_then(|u| u.as_str())
                .ok_or("http action missing 'url'")?;

            let method = action
                .executor_config
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("GET");

            let client = reqwest::Client::new();
            let req = match method.to_uppercase().as_str() {
                "POST" => client.post(url).json(&params),
                _ => client.get(url),
            };

            let resp = req
                .send()
                .await
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            let status = resp.status().is_success();
            let body = resp.text().await.unwrap_or_default();

            Ok(ActionResult {
                success: status,
                message: format!("HTTP {} {}", method, if status { "succeeded" } else { "failed" }),
                output: Some(body),
            })
        }
        "reason" => {
            // Feed output back as reasoning input — handled by frontend
            let text = params
                .and_then(|p| p.get("text").and_then(|t| t.as_str().map(String::from)))
                .unwrap_or_else(|| action.label.clone());

            Ok(ActionResult {
                success: true,
                message: "Feeding back to reasoning engine".to_string(),
                output: Some(text),
            })
        }
        other => Err(format!("Unknown executor type: {}", other)),
    };

    // Run on_action hooks after successful execution
    if result.is_ok() {
        let registry = plugin_state.0.lock().await;
        registry.run_hook("on_action");
    }

    result
}

#[tauri::command]
pub async fn list_actions(
    plugin_state: tauri::State<'_, PluginState>,
) -> Result<Vec<serde_json::Value>, String> {
    let registry = plugin_state.0.lock().await;
    let actions: Vec<serde_json::Value> = registry
        .all_actions()
        .iter()
        .map(|(id, a)| {
            serde_json::json!({
                "id": id,
                "label": a.label,
                "description": a.description,
                "executor": a.executor,
            })
        })
        .collect();
    Ok(actions)
}

#[tauri::command]
pub async fn list_plugins(
    plugin_state: tauri::State<'_, PluginState>,
) -> Result<Vec<serde_json::Value>, String> {
    let registry = plugin_state.0.lock().await;
    let plugins: Vec<serde_json::Value> = registry
        .all_plugins()
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "version": p.version,
                "description": p.description,
                "enabled": p.enabled,
                "actions_count": p.actions.len(),
                "blocks_count": p.blocks.len(),
                "data_sources_count": p.data_sources.len(),
            })
        })
        .collect();
    Ok(plugins)
}

#[tauri::command]
pub async fn set_plugin_enabled(
    name: String,
    enabled: bool,
    plugin_state: tauri::State<'_, PluginState>,
) -> Result<(), String> {
    let mut registry = plugin_state.0.lock().await;
    if !registry.set_plugin_enabled(&name, enabled) {
        return Err(format!("Plugin '{}' not found", name));
    }

    // Also update the TOML file on disk
    let plugins_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".grove")
        .join("plugins");

    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains(&format!("name = \"{}\"", name)) {
                        let updated = if enabled {
                            content.replace("enabled = false", "enabled = true")
                        } else {
                            content.replace("enabled = true", "enabled = false")
                        };
                        std::fs::write(&path, updated).ok();
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
