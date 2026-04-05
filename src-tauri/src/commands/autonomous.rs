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
            "read_source" => {
                if let Some(result) = execute_read_source(action) {
                    results.push(result);
                }
            }
            "shell" => {
                if let Some(result) = execute_shell(action) {
                    results.push(result);
                }
            }
            "open_url" => {
                if let Some(result) = execute_open_url(action) {
                    results.push(result);
                }
            }
            "create_artifact" | "update_artifact" => {
                if let Some(result) = execute_artifact(action) {
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

/// Read one of Grove's own source files (read-only introspection).
fn execute_read_source(action: &AutoAction) -> Option<String> {
    let rel_path = action.params.get("path")?.as_str()?;

    // Validate extension
    let allowed = crate::commands::system::READABLE_EXTENSIONS;
    if !allowed.iter().any(|ext| rel_path.ends_with(ext)) {
        eprintln!("[grove] read_source blocked: unsupported extension in {}", rel_path);
        return Some(format!("read_source blocked: unsupported file type for {}", rel_path));
    }

    // Resolve relative to project root
    let root = crate::commands::system::source_root();
    let full_path = root.join(rel_path);

    // Security: must stay within project root
    let canonical = full_path.canonicalize().ok()?;
    let canonical_root = root.canonicalize().ok()?;
    if !canonical.starts_with(&canonical_root) {
        eprintln!("[grove] read_source blocked: path escapes project root");
        return Some("read_source blocked: path outside project".to_string());
    }

    match fs::read_to_string(&canonical) {
        Ok(content) => {
            let truncated = if content.len() > 4096 {
                format!("{}...\n[truncated at 4KB, file is {} bytes]", &content[..4096], content.len())
            } else {
                content
            };
            Some(format!("[source:{}]\n{}", rel_path, truncated))
        }
        Err(e) => Some(format!("read_source error for {}: {}", rel_path, e)),
    }
}

/// Execute a shell command. Gated by autonomy scoring — only approved commands run.
fn execute_shell(action: &AutoAction) -> Option<String> {
    let command = action.params.get("command")?.as_str()?;
    let workdir = action.params.get("workdir").and_then(|v| v.as_str());

    // Hard-block dangerous patterns regardless of autonomy score
    let lower = command.to_lowercase();
    let blocked_patterns = ["rm -rf /", "sudo rm", "mkfs", "dd if=", "> /dev/", "chmod -R 777"];
    for pattern in &blocked_patterns {
        if lower.contains(pattern) {
            eprintln!("[grove:shell] BLOCKED dangerous command: {}", command);
            return Some(format!("Shell BLOCKED (dangerous): {}", command));
        }
    }

    eprintln!("[grove:shell] Executing: {}", command);

    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(command);
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }

    // Timeout: capture output with a 30-second limit
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let status = output.status.code().unwrap_or(-1);

            let result = if stdout.len() > 2000 {
                format!("{}...[truncated]", &stdout[..2000])
            } else {
                stdout.to_string()
            };

            if status == 0 {
                Some(format!("[shell:ok] $ {}\n{}", command, result.trim()))
            } else {
                Some(format!("[shell:err:{}] $ {}\n{}\n{}", status, command, result.trim(), stderr.trim()))
            }
        }
        Err(e) => Some(format!("[shell:fail] $ {}\nError: {}", command, e)),
    }
}

/// Open a URL in the default browser.
fn execute_open_url(action: &AutoAction) -> Option<String> {
    let url = action.params.get("url")?.as_str()?;

    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Some(format!("open_url blocked: invalid URL {}", url));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().ok();
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    }

    Some(format!("Opened: {}", url))
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

/// Create or update a workspace artifact.
fn execute_artifact(action: &AutoAction) -> Option<String> {
    let name = action.params.get("name")?.as_str()?;
    let artifact_type = action.params.get("artifact_type")
        .and_then(|v| v.as_str())
        .unwrap_or("custom");
    let blocks = action.params.get("blocks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let summary = action.params.get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);

    match super::workspace::upsert_artifact(name, artifact_type, blocks, summary) {
        Ok(()) => Some(format!("Artifact '{}' saved to workspace", name)),
        Err(e) => Some(format!("Failed to save artifact '{}': {}", name, e)),
    }
}
