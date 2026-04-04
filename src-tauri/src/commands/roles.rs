use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A reasoning role loaded from YAML config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub display: String,
    pub description: String,
    pub system_prompt_prefix: String,
    #[serde(default)]
    pub block_preferences: Vec<String>,
    #[serde(default)]
    pub avoid_blocks: Vec<String>,
    #[serde(default = "default_autonomy")]
    pub autonomy_level: String,
}

fn default_autonomy() -> String {
    "medium".to_string()
}

/// Load all role configs from the roles/ directory.
pub fn load_roles() -> Vec<Role> {
    let roles_dir = roles_dir();
    if !roles_dir.exists() {
        return Vec::new();
    }

    let mut roles = Vec::new();
    if let Ok(entries) = fs::read_dir(&roles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                || path.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                match load_role_file(&path) {
                    Ok(role) => roles.push(role),
                    Err(e) => eprintln!("[grove] Failed to load role {}: {}", path.display(), e),
                }
            }
        }
    }

    roles
}

/// Load a single role from a YAML file.
fn load_role_file(path: &std::path::Path) -> Result<Role, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read: {}", e))?;

    // Simple YAML parser (avoid adding a full YAML crate)
    // Parses the flat key-value YAML structure used by role files
    parse_role_yaml(&content)
}

/// Parse role YAML without a full YAML library.
/// Handles the simple structure: key: value, key: |, - items
fn parse_role_yaml(content: &str) -> Result<Role, String> {
    let mut name = String::new();
    let mut display = String::new();
    let mut description = String::new();
    let mut system_prompt_prefix = String::new();
    let mut block_preferences: Vec<String> = Vec::new();
    let mut avoid_blocks: Vec<String> = Vec::new();
    let mut autonomy_level = "medium".to_string();

    let mut current_key = String::new();
    let mut in_multiline = false;
    let mut in_list = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if in_multiline {
                system_prompt_prefix.push('\n');
            }
            continue;
        }

        // Check for list item
        if trimmed.starts_with("- ") && in_list {
            let item = trimmed[2..].trim().to_string();
            match current_key.as_str() {
                "block_preferences" => block_preferences.push(item),
                "avoid_blocks" => avoid_blocks.push(item),
                _ => {}
            }
            continue;
        }

        // Check for key: value
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();

            in_multiline = false;
            in_list = false;

            if value == "|" {
                // Start multiline
                current_key = key.to_string();
                in_multiline = true;
                continue;
            }

            if value.is_empty() {
                // Start list
                current_key = key.to_string();
                in_list = true;
                continue;
            }

            // Simple key: value
            let val = value.trim_matches('"').trim_matches('\'').to_string();
            match key {
                "name" => name = val,
                "display" => display = val,
                "description" => description = val,
                "autonomy_level" => autonomy_level = val,
                _ => {}
            }
        } else if in_multiline {
            // Multiline content
            if current_key == "system_prompt_prefix" {
                if !system_prompt_prefix.is_empty() {
                    system_prompt_prefix.push('\n');
                }
                system_prompt_prefix.push_str(trimmed);
            }
        }
    }

    if name.is_empty() {
        return Err("Missing 'name' field".to_string());
    }

    Ok(Role {
        name,
        display,
        description,
        system_prompt_prefix,
        block_preferences,
        avoid_blocks,
        autonomy_level,
    })
}

/// Get the roles directory path (project root/roles/)
fn roles_dir() -> PathBuf {
    // Try to find the roles/ dir relative to the executable or CWD
    let cwd = std::env::current_dir().unwrap_or_default();

    // Check common locations
    for candidate in &[
        cwd.join("roles"),
        cwd.join("../roles"),
        cwd.join("../../roles"),
    ] {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Also check ~/.grove/roles/
    if let Some(home) = dirs::home_dir() {
        let grove_roles = home.join(".grove").join("roles");
        if grove_roles.exists() {
            return grove_roles;
        }
    }

    cwd.join("roles")
}

/// Get a role by name.
pub fn get_role(name: &str) -> Option<Role> {
    load_roles().into_iter().find(|r| r.name == name)
}

/// Build the system prompt modifier for a role.
/// Includes the role's prompt prefix and block preferences.
pub fn role_prompt_modifier(role: &Role) -> String {
    let mut modifier = format!("[ROLE: {}]\n{}", role.display, role.system_prompt_prefix);

    if !role.block_preferences.is_empty() {
        modifier.push_str(&format!(
            "\nPrefer these block types: {}",
            role.block_preferences.join(", ")
        ));
    }
    if !role.avoid_blocks.is_empty() {
        modifier.push_str(&format!(
            "\nAvoid these block types: {}",
            role.avoid_blocks.join(", ")
        ));
    }

    modifier
}

/// Tauri commands for role management
#[tauri::command]
pub async fn list_roles() -> Result<Vec<Role>, String> {
    Ok(load_roles())
}

#[tauri::command]
pub async fn get_active_role(
    state: tauri::State<'_, crate::RoleState>,
) -> Result<Option<String>, String> {
    let role = state.0.lock().await;
    Ok(role.clone())
}

#[tauri::command]
pub async fn set_active_role(
    name: Option<String>,
    state: tauri::State<'_, crate::RoleState>,
) -> Result<(), String> {
    // Validate role exists if a name is given
    if let Some(ref n) = name {
        if get_role(n).is_none() {
            return Err(format!("Role '{}' not found", n));
        }
    }
    let mut role = state.0.lock().await;
    *role = name;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_role_yaml() {
        let yaml = r#"
name: builder
display: "Builder"
description: "Building, coding, and shipping mode."
system_prompt_prefix: |
  You are in BUILDER mode. Focus on concrete actions.
  Prioritize actionable outputs over reflection.
block_preferences:
  - actions
  - status
  - progress
avoid_blocks:
  - quote
autonomy_level: medium
"#;
        let role = parse_role_yaml(yaml).unwrap();
        assert_eq!(role.name, "builder");
        assert_eq!(role.display, "Builder");
        assert!(role.system_prompt_prefix.contains("BUILDER mode"));
        assert_eq!(
            role.block_preferences,
            vec!["actions", "status", "progress"]
        );
        assert_eq!(role.avoid_blocks, vec!["quote"]);
        assert_eq!(role.autonomy_level, "medium");
    }

    #[test]
    fn test_parse_minimal_role() {
        let yaml = "name: test\ndisplay: Test\ndescription: A test role\n";
        let role = parse_role_yaml(yaml).unwrap();
        assert_eq!(role.name, "test");
        assert!(role.block_preferences.is_empty());
    }

    #[test]
    fn test_role_prompt_modifier() {
        let role = Role {
            name: "reflector".to_string(),
            display: "Reflector".to_string(),
            description: "Reflection mode".to_string(),
            system_prompt_prefix: "Focus on depth.".to_string(),
            block_preferences: vec!["text".to_string(), "insight".to_string()],
            avoid_blocks: vec!["metric".to_string()],
            autonomy_level: "low".to_string(),
        };
        let modifier = role_prompt_modifier(&role);
        assert!(modifier.contains("[ROLE: Reflector]"));
        assert!(modifier.contains("Focus on depth."));
        assert!(modifier.contains("Prefer these block types: text, insight"));
        assert!(modifier.contains("Avoid these block types: metric"));
    }
}
