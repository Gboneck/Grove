use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::commands::actions::PluginState;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

#[derive(Debug, Clone, Serialize)]
pub struct FileStamps {
    pub files: HashMap<String, u64>,
}

fn get_file_mtime(path: &std::path::Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0)
}

/// Returns modification timestamps for key ~/.grove/ files.
/// Frontend polls this to detect external edits.
#[tauri::command]
pub async fn get_file_stamps() -> Result<FileStamps, String> {
    let dir = grove_dir();
    let watched = ["soul.md", "context.json", "config.toml"];

    let mut files = HashMap::new();
    for name in watched {
        let path = dir.join(name);
        if path.exists() {
            files.insert(name.to_string(), get_file_mtime(&path));
        }
    }

    Ok(FileStamps { files })
}

/// Called by the frontend when it detects a file change, to run on_file_change hooks
#[tauri::command]
pub async fn notify_file_change(
    plugin_state: tauri::State<'_, PluginState>,
) -> Result<(), String> {
    let registry = plugin_state.0.lock().await;
    registry.run_hook("on_file_change");
    Ok(())
}
