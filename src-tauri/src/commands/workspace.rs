use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Artifacts: persistent workspace objects the model builds and maintains ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type: String, // "dashboard", "journal", "brief", "tracker", "map", "custom"
    pub content: ArtifactContent,
    pub created_at: String,
    pub updated_at: String,
    pub update_count: u32,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default)]
    pub collapsed: bool,
}

fn default_width() -> f64 {
    360.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactContent {
    /// Rendered blocks that make up this artifact's visual content
    pub blocks: Vec<Value>,
    /// Optional summary line shown in collapsed view
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub last_saved: String,
    pub artifacts: Vec<Artifact>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            last_saved: Utc::now().to_rfc3339(),
            artifacts: Vec::new(),
        }
    }
}

pub struct WorkspaceState(pub Arc<Mutex<Workspace>>);

// ── Persistence ──

fn workspace_path() -> Result<std::path::PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".grove").join("workspace.json"))
        .ok_or_else(|| "No home dir".to_string())
}

pub fn load_workspace_from_disk() -> Workspace {
    let path = match workspace_path() {
        Ok(p) => p,
        Err(_) => return Workspace::default(),
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Workspace::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_workspace_to_disk(ws: &Workspace) -> Result<(), String> {
    let path = workspace_path()?;
    let json = serde_json::to_string_pretty(ws).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

// ── Tauri Commands ──

#[tauri::command]
pub async fn load_workspace(
    state: tauri::State<'_, WorkspaceState>,
) -> Result<Value, String> {
    let ws = state.0.lock().await;
    serde_json::to_value(&*ws).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_workspace(
    artifacts: Vec<Artifact>,
    state: tauri::State<'_, WorkspaceState>,
) -> Result<(), String> {
    let mut ws = state.0.lock().await;
    ws.artifacts = artifacts;
    ws.last_saved = Utc::now().to_rfc3339();
    save_workspace_to_disk(&ws)
}

#[tauri::command]
pub async fn remove_artifact(
    id: String,
    state: tauri::State<'_, WorkspaceState>,
) -> Result<(), String> {
    let mut ws = state.0.lock().await;
    ws.artifacts.retain(|a| a.id != id);
    save_workspace_to_disk(&ws)
}

/// Create or update an artifact — called via auto-actions from the model.
pub fn upsert_artifact(name: &str, artifact_type: &str, blocks: Vec<Value>, summary: Option<String>) -> Result<(), String> {
    let mut ws = load_workspace_from_disk();
    let now = Utc::now().to_rfc3339();

    if let Some(existing) = ws.artifacts.iter_mut().find(|a| a.name == name) {
        // Update existing artifact
        existing.content.blocks = blocks;
        if summary.is_some() {
            existing.content.summary = summary;
        }
        existing.updated_at = now;
        existing.update_count += 1;
        eprintln!("[grove:workspace] Updated artifact '{}' (v{})", name, existing.update_count);
    } else {
        // Create new artifact
        // Auto-place: offset from last artifact to avoid overlap
        let (ax, ay) = ws.artifacts.last()
            .map(|last| (last.x + 20.0, last.y + last.width.min(200.0) + 20.0))
            .unwrap_or((20.0, 20.0));

        let artifact = Artifact {
            id: format!("artifact-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            artifact_type: artifact_type.to_string(),
            content: ArtifactContent { blocks, summary },
            created_at: now.clone(),
            updated_at: now,
            update_count: 1,
            x: ax,
            y: ay,
            width: default_width(),
            collapsed: false,
        };
        eprintln!("[grove:workspace] Created artifact '{}'", name);
        ws.artifacts.push(artifact);
    }

    // Cap at 20 artifacts
    if ws.artifacts.len() > 20 {
        ws.artifacts = ws.artifacts[ws.artifacts.len() - 20..].to_vec();
    }

    ws.last_saved = Utc::now().to_rfc3339();
    save_workspace_to_disk(&ws)
}

/// Build workspace context for the reasoning engine.
pub fn workspace_context_for_model() -> String {
    let ws = load_workspace_from_disk();
    if ws.artifacts.is_empty() {
        return String::from("\n--- WORKSPACE ---\nNo artifacts yet. Build workspace artifacts for the user with create_artifact auto-actions.\n");
    }

    let mut out = String::from("\n--- WORKSPACE ARTIFACTS ---\n");
    out.push_str(&format!("{} artifact(s) on the user's workspace:\n", ws.artifacts.len()));

    for a in &ws.artifacts {
        let age = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.updated_at) {
            let mins = (Utc::now() - dt.with_timezone(&Utc)).num_minutes();
            if mins < 60 { format!("{}m ago", mins) }
            else if mins < 1440 { format!("{}h ago", mins / 60) }
            else { format!("{}d ago", mins / 1440) }
        } else { "unknown".to_string() };

        let summary = a.content.summary.as_deref().unwrap_or("no summary");
        out.push_str(&format!(
            "\n- \"{}\" [{}] — {} (updated {}, v{})\n",
            a.name, a.artifact_type, summary, age, a.update_count
        ));
    }

    out.push_str("\nUpdate existing artifacts when context changes. Don't recreate — use the same name to update.\n");
    out
}
