use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveConfig {
    pub models: ModelsConfig,
    pub paths: PathsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    #[serde(default = "default_local_model")]
    pub local_model: String,
    #[serde(default = "default_local_url")]
    pub local_url: String,
    #[serde(default = "default_context_window")]
    pub local_context_window: u32,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
    #[serde(default = "default_cloud_model")]
    pub cloud_model: String,
    #[serde(default = "default_cloud_api_key_env")]
    pub cloud_api_key_env: String,
    #[serde(default = "default_true")]
    pub prefer_local: bool,
    #[serde(default)]
    pub offline_mode: bool,
    #[serde(default = "default_true")]
    pub escalation_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_soul_path")]
    pub soul_md: String,
    #[serde(default = "default_memory_dir")]
    pub memory_dir: String,
    #[serde(default)]
    pub watch_dirs: Vec<String>,
}

fn default_local_model() -> String { "gemma4:27b-it-qat".to_string() }
fn default_local_url() -> String { "http://localhost:11434".to_string() }
fn default_context_window() -> u32 { 32768 }
fn default_confidence_threshold() -> f64 { 0.7 }
fn default_cloud_model() -> String { "claude-sonnet-4-20250514".to_string() }
fn default_cloud_api_key_env() -> String { "ANTHROPIC_API_KEY".to_string() }
fn default_true() -> bool { true }
fn default_soul_path() -> String { "~/.grove/soul.md".to_string() }
fn default_memory_dir() -> String { "~/.grove/memory/".to_string() }

impl Default for GroveConfig {
    fn default() -> Self {
        GroveConfig {
            models: ModelsConfig {
                local_model: default_local_model(),
                local_url: default_local_url(),
                local_context_window: default_context_window(),
                confidence_threshold: default_confidence_threshold(),
                cloud_model: default_cloud_model(),
                cloud_api_key_env: default_cloud_api_key_env(),
                prefer_local: true,
                offline_mode: false,
                escalation_logging: true,
            },
            paths: PathsConfig {
                soul_md: default_soul_path(),
                memory_dir: default_memory_dir(),
                watch_dirs: Vec::new(),
            },
        }
    }
}

const DEFAULT_CONFIG_TOML: &str = r#"[models]
# Local model (Ollama)
local_model = "gemma4:27b-it-qat"
local_url = "http://localhost:11434"
local_context_window = 32768
confidence_threshold = 0.7

# Cloud model (Claude API)
cloud_model = "claude-sonnet-4-20250514"
cloud_api_key_env = "ANTHROPIC_API_KEY"  # reads from env var

# Routing
prefer_local = true          # default to Gemma when possible
offline_mode = false          # force local-only
escalation_logging = true     # log when/why Claude is called

[paths]
soul_md = "~/.grove/soul.md"
memory_dir = "~/.grove/memory/"
watch_dirs = []
"#;

pub fn ensure_config() {
    let path = grove_dir().join("config.toml");
    if !path.exists() {
        fs::write(&path, DEFAULT_CONFIG_TOML).ok();
    }
}

pub fn load_config() -> GroveConfig {
    let path = grove_dir().join("config.toml");
    if let Ok(content) = fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        GroveConfig::default()
    }
}
