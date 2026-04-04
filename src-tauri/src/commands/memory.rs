use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub timestamp: String,
    pub text: String,
    pub response_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub timestamp: String,
    pub time_of_day: String,
    pub day_of_week: String,
    pub blocks_shown: Vec<String>,
    pub user_inputs: Vec<UserInput>,
    pub session_summary: String,
    pub insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub sessions: Vec<Session>,
    pub accumulated_insights: Vec<String>,
    pub last_seen: Option<String>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            sessions: Vec::new(),
            accumulated_insights: Vec::new(),
            last_seen: None,
        }
    }
}

pub fn ensure_memory() {
    let path = grove_dir().join("memory.json");
    if !path.exists() {
        let default = Memory::default();
        let content = serde_json::to_string_pretty(&default).unwrap();
        fs::write(&path, content).expect("Failed to write default memory.json");
    }
}

pub fn read_memory_file() -> Result<Memory, String> {
    let path = grove_dir().join("memory.json");
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read memory.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse memory.json: {}", e))
}

pub fn write_memory_file(memory: &Memory) -> Result<(), String> {
    let path = grove_dir().join("memory.json");
    let content = serde_json::to_string_pretty(memory)
        .map_err(|e| format!("Failed to serialize memory: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write memory.json: {}", e))
}

pub fn record_session(
    blocks_shown: Vec<String>,
    user_input: Option<&str>,
    session_summary: &str,
    insights: Vec<String>,
) -> Result<(), String> {
    let mut memory = read_memory_file()?;
    let now = Utc::now();

    let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(12);
    let time_of_day = match hour {
        0..=5 => "late night",
        6..=11 => "morning",
        12..=16 => "afternoon",
        17..=20 => "evening",
        _ => "night",
    }
    .to_string();

    let mut user_inputs = Vec::new();
    if let Some(input) = user_input {
        user_inputs.push(UserInput {
            timestamp: now.to_rfc3339(),
            text: input.to_string(),
            response_summary: session_summary.to_string(),
        });
    }

    let session = Session {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: now.to_rfc3339(),
        time_of_day,
        day_of_week: now.format("%A").to_string(),
        blocks_shown,
        user_inputs,
        session_summary: session_summary.to_string(),
        insights: insights.clone(),
    };

    memory.sessions.push(session);
    memory.last_seen = Some(now.to_rfc3339());

    // Prune to last 30 sessions
    if memory.sessions.len() > 30 {
        let start = memory.sessions.len() - 30;
        memory.sessions = memory.sessions[start..].to_vec();
    }

    // Add new accumulated insights, cap at 50
    for insight in insights {
        if !memory.accumulated_insights.contains(&insight) {
            memory.accumulated_insights.push(insight);
        }
    }
    if memory.accumulated_insights.len() > 50 {
        let start = memory.accumulated_insights.len() - 50;
        memory.accumulated_insights = memory.accumulated_insights[start..].to_vec();
    }

    write_memory_file(&memory)
}

#[tauri::command]
pub async fn get_memory(count: Option<usize>) -> Result<Vec<Session>, String> {
    let memory = read_memory_file()?;
    let n = count.unwrap_or(5);
    let sessions: Vec<Session> = memory
        .sessions
        .iter()
        .rev()
        .take(n)
        .cloned()
        .collect();
    Ok(sessions)
}
