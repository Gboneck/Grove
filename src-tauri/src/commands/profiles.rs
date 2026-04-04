use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

fn profiles_dir() -> PathBuf {
    grove_dir().join("profiles")
}

fn active_profile_path() -> PathBuf {
    grove_dir().join(".active_profile")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub context_file: String, // relative to ~/.grove/profiles/{name}/
    pub plugins: Vec<String>, // plugin names to enable for this profile
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub is_active: bool,
}

pub fn ensure_profiles_dir() {
    let dir = profiles_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }

    // Create default profile if none exist
    let default_dir = dir.join("default");
    if !default_dir.exists() {
        fs::create_dir_all(&default_dir).ok();

        let profile = Profile {
            name: "default".to_string(),
            description: "Default profile".to_string(),
            context_file: "context.json".to_string(),
            plugins: Vec::new(),
        };
        let manifest = toml::to_string_pretty(&profile).unwrap_or_default();
        fs::write(default_dir.join("profile.toml"), manifest).ok();

        // Copy current context.json to the default profile
        let main_context = grove_dir().join("context.json");
        if main_context.exists() {
            fs::copy(&main_context, default_dir.join("context.json")).ok();
        }
    }

    // Set active profile to default if not set
    if !active_profile_path().exists() {
        fs::write(active_profile_path(), "default").ok();
    }
}

pub fn get_active_profile_name() -> String {
    fs::read_to_string(active_profile_path())
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string()
}

fn load_profile(name: &str) -> Result<Profile, String> {
    let manifest_path = profiles_dir().join(name).join("profile.toml");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read profile '{}': {}", name, e))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse profile '{}': {}", name, e))
}

#[tauri::command]
pub async fn list_profiles() -> Result<Vec<ProfileInfo>, String> {
    let dir = profiles_dir();
    let active = get_active_profile_name();
    let mut profiles = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let description = load_profile(&name)
                    .map(|p| p.description)
                    .unwrap_or_default();
                profiles.push(ProfileInfo {
                    is_active: name == active,
                    name,
                    description,
                });
            }
        }
    }

    // Sort so active profile comes first
    profiles.sort_by(|a, b| b.is_active.cmp(&a.is_active).then(a.name.cmp(&b.name)));
    Ok(profiles)
}

#[tauri::command]
pub async fn switch_profile(name: String) -> Result<(), String> {
    let profile_dir = profiles_dir().join(&name);
    if !profile_dir.exists() {
        return Err(format!("Profile '{}' does not exist", name));
    }

    let profile = load_profile(&name)?;

    // Save current context.json back to the old profile
    let old_profile = get_active_profile_name();
    let old_profile_dir = profiles_dir().join(&old_profile);
    let main_context = grove_dir().join("context.json");
    if main_context.exists() && old_profile_dir.exists() {
        fs::copy(&main_context, old_profile_dir.join("context.json")).ok();
    }

    // Load new profile's context.json
    let new_context = profile_dir.join(&profile.context_file);
    if new_context.exists() {
        fs::copy(&new_context, &main_context)
            .map_err(|e| format!("Failed to load profile context: {}", e))?;
    }

    // Set active profile
    fs::write(active_profile_path(), &name)
        .map_err(|e| format!("Failed to write active profile: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn create_profile(
    name: String,
    description: String,
) -> Result<(), String> {
    let profile_dir = profiles_dir().join(&name);
    if profile_dir.exists() {
        return Err(format!("Profile '{}' already exists", name));
    }

    fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to create profile directory: {}", e))?;

    let profile = Profile {
        name: name.clone(),
        description,
        context_file: "context.json".to_string(),
        plugins: Vec::new(),
    };

    let manifest = toml::to_string_pretty(&profile)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    fs::write(profile_dir.join("profile.toml"), manifest)
        .map_err(|e| format!("Failed to write profile manifest: {}", e))?;

    // Create empty context for the profile
    fs::write(
        profile_dir.join("context.json"),
        "{\n  \"ventures\": []\n}\n",
    )
    .map_err(|e| format!("Failed to write profile context: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_profile(name: String) -> Result<(), String> {
    if name == "default" {
        return Err("Cannot delete the default profile".to_string());
    }

    let active = get_active_profile_name();
    if name == active {
        return Err("Cannot delete the active profile. Switch to another profile first.".to_string());
    }

    let profile_dir = profiles_dir().join(&name);
    if !profile_dir.exists() {
        return Err(format!("Profile '{}' does not exist", name));
    }

    fs::remove_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to delete profile: {}", e))?;

    Ok(())
}
