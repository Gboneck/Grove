use super::{PluginManifest, ActionDef, CustomBlockDef, DataSourceDef};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

/// The plugin registry — holds all loaded plugins and provides lookups
#[derive(Debug, Clone, Serialize)]
pub struct PluginRegistry {
    plugins: Vec<PluginManifest>,
    actions: HashMap<String, ActionDef>,
    block_types: HashMap<String, CustomBlockDef>,
    data_sources: HashMap<String, DataSourceDef>,
}

impl PluginRegistry {
    pub fn new(plugins: Vec<PluginManifest>) -> Self {
        let mut actions = HashMap::new();
        let mut block_types = HashMap::new();
        let mut data_sources = HashMap::new();

        for plugin in &plugins {
            for action in &plugin.actions {
                actions.insert(
                    format!("{}:{}", plugin.name, action.id),
                    action.clone(),
                );
            }
            for block in &plugin.blocks {
                block_types.insert(block.block_type.clone(), block.clone());
            }
            for ds in &plugin.data_sources {
                data_sources.insert(
                    format!("{}:{}", plugin.name, ds.id),
                    ds.clone(),
                );
            }
        }

        PluginRegistry {
            plugins,
            actions,
            block_types,
            data_sources,
        }
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn get_action(&self, id: &str) -> Option<&ActionDef> {
        self.actions.get(id)
    }

    pub fn all_actions(&self) -> &HashMap<String, ActionDef> {
        &self.actions
    }

    pub fn all_block_types(&self) -> &HashMap<String, CustomBlockDef> {
        &self.block_types
    }

    /// Gather data from all plugin data sources for the reasoning context
    pub fn gather_data_context(&self) -> String {
        let mut context_parts = Vec::new();

        for (id, ds) in &self.data_sources {
            match ds.source_type.as_str() {
                "file" => {
                    if let Some(path) = ds.source_config.get("path").and_then(|p| p.as_str()) {
                        let expanded = path.replace("~", &dirs::home_dir().unwrap_or_default().to_string_lossy());
                        if let Ok(content) = fs::read_to_string(&expanded) {
                            context_parts.push(format!("--- {} ({}) ---\n{}", ds.label, id, content));
                        }
                    }
                }
                _ => {} // http and shell sources can be added later
            }
        }

        if context_parts.is_empty() {
            String::new()
        } else {
            format!("\n--- PLUGIN DATA ---\n{}", context_parts.join("\n\n"))
        }
    }

    /// Get a summary of available actions for the reasoning engine
    pub fn actions_context(&self) -> String {
        if self.actions.is_empty() {
            return String::new();
        }

        let action_list: Vec<String> = self
            .actions
            .iter()
            .map(|(id, a)| format!("- {} ({}): {}", a.label, id, a.description))
            .collect();

        format!(
            "\n--- AVAILABLE ACTIONS ---\nThe user has these executable actions available:\n{}",
            action_list.join("\n")
        )
    }

    /// Get plugin summaries for the reasoning engine
    pub fn plugins_context(&self) -> String {
        if self.plugins.is_empty() {
            return String::new();
        }

        let summaries: Vec<String> = self
            .plugins
            .iter()
            .map(|p| format!("- {} v{}: {}", p.name, p.version, p.description))
            .collect();

        format!(
            "\n--- ACTIVE PLUGINS ---\n{}",
            summaries.join("\n")
        )
    }
}
