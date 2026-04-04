use super::PluginManifest;
use std::fs;
use std::path::PathBuf;

fn plugins_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
        .join("plugins")
}

pub fn ensure_plugins_dir() {
    let dir = plugins_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }

    // Write example plugin if directory is empty
    let example_path = dir.join("_example.toml.disabled");
    if !example_path.exists() {
        fs::write(&example_path, EXAMPLE_PLUGIN).ok();
    }
}

const EXAMPLE_PLUGIN: &str = r#"# Example Grove Plugin
# Rename this file to enable it (remove .disabled)
# e.g., clipboard-actions.toml

name = "example"
version = "0.1.0"
description = "An example plugin to show the format"
enabled = true

# Custom block types this plugin adds
[[blocks]]
block_type = "progress"
description = "A progress bar block"
[blocks.schema]
label = "string"
value = "number"
max = "number"

# Executable actions this plugin provides
[[actions]]
id = "copy_to_clipboard"
label = "Copy to clipboard"
description = "Copies text to system clipboard"
executor = "clipboard"
[actions.executor_config]
# template = "{{content}}"

# Data sources this plugin brings into reasoning context
[[data_sources]]
id = "local_todos"
label = "Local Todo List"
source_type = "file"
[data_sources.source_config]
path = "~/.grove/todos.md"

# Lifecycle hooks (shell commands)
[hooks]
# on_startup = "echo 'Plugin loaded'"
# on_reason = ""
# on_action = ""
# on_file_change = ""
"#;

/// Load all enabled plugins from ~/.grove/plugins/
pub fn load_plugins() -> Vec<PluginManifest> {
    let dir = plugins_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut plugins = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                match fs::read_to_string(&path) {
                    Ok(content) => match toml::from_str::<PluginManifest>(&content) {
                        Ok(manifest) => {
                            if manifest.enabled {
                                plugins.push(manifest);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[grove] Failed to parse plugin {:?}: {}",
                                path.file_name().unwrap_or_default(),
                                e
                            );
                        }
                    },
                    Err(e) => {
                        eprintln!("[grove] Failed to read plugin {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    plugins
}
