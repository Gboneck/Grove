use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

const DEFAULT_SOUL: &str = r#"# Soul.md

Welcome to Grove. This file is your identity document.
Edit it to tell the system who you are, what you're working on,
and what matters to you. The more context you provide, the more
useful Grove becomes.

## Who I Am
[Your name, what you do, where you're based]

## What I'm Working On
[Active projects, ventures, goals]

## What Matters Right Now
[Current priorities, deadlines, blockers]

## How I Work
[Preferences, patterns, energy rhythms]
"#;

pub fn ensure_grove_dir() {
    let dir = grove_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create ~/.grove/");
    }
    let logs_dir = dir.join("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir).ok();
    }
}

pub fn ensure_soul() {
    let path = grove_dir().join("soul.md");
    if !path.exists() {
        fs::write(&path, DEFAULT_SOUL).expect("Failed to write default soul.md");
    }
}

#[tauri::command]
pub async fn read_soul() -> Result<String, String> {
    let path = grove_dir().join("soul.md");
    fs::read_to_string(&path).map_err(|e| format!("Failed to read soul.md: {}", e))
}

#[tauri::command]
pub async fn write_soul(content: String) -> Result<(), String> {
    let path = grove_dir().join("soul.md");
    fs::write(&path, content).map_err(|e| format!("Failed to write soul.md: {}", e))
}
