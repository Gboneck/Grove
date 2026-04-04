use super::PluginManifest;
use serde_json::Value;
use std::collections::HashMap;

/// Parsed from TOML — the on-disk format
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub blocks: Vec<RawBlockDef>,
    #[serde(default)]
    pub actions: Vec<RawActionDef>,
    #[serde(default)]
    pub data_sources: Vec<RawDataSourceDef>,
    #[serde(default)]
    pub hooks: RawHooks,
    #[serde(default)]
    pub config: HashMap<String, Value>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawBlockDef {
    pub block_type: String,
    pub description: String,
    #[serde(default)]
    pub schema: Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawActionDef {
    pub id: String,
    pub label: String,
    pub description: String,
    pub executor: String,
    #[serde(default)]
    pub executor_config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct RawDataSourceDef {
    pub id: String,
    pub label: String,
    pub source_type: String,
    #[serde(default)]
    pub source_config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct RawHooks {
    pub on_startup: Option<String>,
    pub on_reason: Option<String>,
    pub on_action: Option<String>,
    pub on_file_change: Option<String>,
}

impl From<RawManifest> for PluginManifest {
    fn from(raw: RawManifest) -> Self {
        use super::*;
        PluginManifest {
            name: raw.name,
            version: raw.version,
            description: raw.description,
            enabled: raw.enabled,
            blocks: raw.blocks.into_iter().map(|b| CustomBlockDef {
                block_type: b.block_type,
                description: b.description,
                schema: b.schema,
            }).collect(),
            actions: raw.actions.into_iter().map(|a| ActionDef {
                id: a.id,
                label: a.label,
                description: a.description,
                executor: a.executor,
                executor_config: a.executor_config,
            }).collect(),
            data_sources: raw.data_sources.into_iter().map(|d| DataSourceDef {
                id: d.id,
                label: d.label,
                source_type: d.source_type,
                source_config: d.source_config,
            }).collect(),
            hooks: PluginHooks {
                on_startup: raw.hooks.on_startup,
                on_reason: raw.hooks.on_reason,
                on_action: raw.hooks.on_action,
                on_file_change: raw.hooks.on_file_change,
            },
            config: raw.config,
        }
    }
}
