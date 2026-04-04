pub mod loader;
pub mod manifest;
pub mod registry;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A plugin manifest — loaded from ~/.grove/plugins/{name}.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub blocks: Vec<CustomBlockDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub data_sources: Vec<DataSourceDef>,
    #[serde(default)]
    pub hooks: PluginHooks,
    #[serde(default)]
    pub config: HashMap<String, Value>,
}

fn default_true() -> bool {
    true
}

/// Custom block type a plugin can register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBlockDef {
    pub block_type: String,
    pub description: String,
    pub schema: Value, // JSON schema for this block's fields
}

/// An executable action a plugin provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub label: String,
    pub description: String,
    /// "clipboard" | "shell" | "http" | "write_file" | "reason"
    pub executor: String,
    #[serde(default)]
    pub executor_config: HashMap<String, Value>,
}

/// A data source the plugin brings into the reasoning context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceDef {
    pub id: String,
    pub label: String,
    /// "file" | "http" | "shell"
    pub source_type: String,
    #[serde(default)]
    pub source_config: HashMap<String, Value>,
}

/// Lifecycle hooks the plugin can declare
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginHooks {
    #[serde(default)]
    pub on_startup: Option<String>,
    #[serde(default)]
    pub on_reason: Option<String>,
    #[serde(default)]
    pub on_action: Option<String>,
    #[serde(default)]
    pub on_file_change: Option<String>,
}
