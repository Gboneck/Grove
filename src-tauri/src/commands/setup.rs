use serde::Serialize;
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub grove_dir_exists: bool,
    pub soul_md_exists: bool,
    pub soul_md_is_default: bool,
    pub api_key_set: bool,
    pub ollama_available: bool,
    pub needs_setup: bool,
    pub local_model: String,
    pub system_ram_gb: u64,
    pub recommended_model: String,
}

fn get_system_ram_gb() -> u64 {
    // macOS: use sysctl
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(bytes) = s.trim().parse::<u64>() {
                    return bytes / (1024 * 1024 * 1024); // bytes to GB
                }
            }
        }
    }

    // Linux: read /proc/meminfo
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb: u64 = rest
                        .trim()
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb / 1_048_576; // KB to GB
                }
            }
        }
    }

    // Fallback
    16
}

fn recommend_model(ram_gb: u64) -> &'static str {
    match ram_gb {
        0..=7 => "gemma3:1b",
        8..=15 => "gemma3:4b",
        16..=31 => "gemma3:12b",
        _ => "gemma3:27b",
    }
}

#[tauri::command]
pub async fn check_setup() -> Result<SetupStatus, String> {
    let dir = grove_dir();
    let soul_path = dir.join("soul.md");
    let env_path = dir.join(".env");

    let grove_dir_exists = dir.exists();
    let soul_md_exists = soul_path.exists();

    let soul_md_is_default = if soul_md_exists {
        fs::read_to_string(&soul_path)
            .map(|c| c.contains("[Your name, what you do, where you're based]"))
            .unwrap_or(true)
    } else {
        true
    };

    // Check API key
    let api_key_set = std::env::var("ANTHROPIC_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
        || env_path
            .exists()
            .then(|| {
                fs::read_to_string(&env_path)
                    .map(|c| {
                        c.lines().any(|l| {
                            l.trim()
                                .strip_prefix("ANTHROPIC_API_KEY=")
                                .map(|v| {
                                    let v = v.trim().trim_matches('"').trim_matches('\'');
                                    !v.is_empty() && v != "your-api-key-here"
                                })
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false);

    // Check Ollama
    let ollama_available = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let ram_gb = get_system_ram_gb();
    let recommended = recommend_model(ram_gb);

    let needs_setup = !api_key_set && !ollama_available;

    Ok(SetupStatus {
        grove_dir_exists,
        soul_md_exists,
        soul_md_is_default,
        api_key_set,
        ollama_available,
        needs_setup,
        local_model: recommended.to_string(),
        system_ram_gb: ram_gb,
        recommended_model: recommended.to_string(),
    })
}

#[tauri::command]
pub async fn save_api_key(key: String) -> Result<(), String> {
    let env_path = grove_dir().join(".env");
    let content = format!("ANTHROPIC_API_KEY={}\n", key.trim());
    fs::write(&env_path, content).map_err(|e| format!("Failed to save API key: {}", e))?;

    // Also set it in the current process environment
    std::env::set_var("ANTHROPIC_API_KEY", key.trim());
    Ok(())
}
