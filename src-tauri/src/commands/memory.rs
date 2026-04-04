use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn grove_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".grove")
}

// ── Episodic Memory: discrete session events ──

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
    #[serde(default)]
    pub model_source: Option<String>,
    #[serde(default)]
    pub engagement: Option<SessionEngagement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEngagement {
    pub actions_clicked: u32,
    pub inputs_submitted: u32,
    pub blocks_dismissed: Vec<String>,
    pub time_spent_seconds: Option<u64>,
}

// ── Semantic Memory: factual knowledge about the user ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: String,
    pub category: String, // "identity", "preference", "goal", "relationship", "skill"
    pub content: String,
    pub confidence: f64,
    pub source: String,           // which session/input created this
    pub created_at: String,
    pub last_confirmed: String,
    #[serde(default)]
    pub superseded_by: Option<String>, // if contradicted, points to newer fact
}

// ── Procedural Memory: learned patterns about what works ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralPattern {
    pub id: String,
    pub pattern_type: String, // "block_preference", "time_pattern", "topic_response", "action_habit"
    pub description: String,
    pub evidence_count: u32,      // how many times observed
    pub last_observed: String,
    pub effectiveness: f64,       // 0.0-1.0 how well this pattern works
}

// ── Full Memory Structure ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    // Episodic: what happened
    pub sessions: Vec<Session>,

    // Semantic: what we know
    #[serde(default)]
    pub facts: Vec<SemanticFact>,

    // Procedural: what works
    #[serde(default)]
    pub patterns: Vec<ProceduralPattern>,

    // Legacy (kept for backward compat, fed by model insights)
    pub accumulated_insights: Vec<String>,
    pub last_seen: Option<String>,

    // Self-tuning metrics
    #[serde(default)]
    pub tuning: TuningMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TuningMetrics {
    pub total_sessions: u64,
    pub total_actions_clicked: u64,
    pub total_inputs_submitted: u64,
    #[serde(default)]
    pub block_type_engagement: std::collections::HashMap<String, BlockEngagement>,
    #[serde(default)]
    pub preferred_session_times: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockEngagement {
    pub shown: u64,
    pub interacted: u64,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            sessions: Vec::new(),
            facts: Vec::new(),
            patterns: Vec::new(),
            accumulated_insights: Vec::new(),
            last_seen: None,
            tuning: TuningMetrics::default(),
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
        time_of_day: time_of_day.clone(),
        day_of_week: now.format("%A").to_string(),
        blocks_shown: blocks_shown.clone(),
        user_inputs,
        session_summary: session_summary.to_string(),
        insights: insights.clone(),
        model_source: None,
        engagement: None,
    };

    memory.sessions.push(session);
    memory.last_seen = Some(now.to_rfc3339());

    // Update tuning metrics
    memory.tuning.total_sessions += 1;
    if !memory.tuning.preferred_session_times.contains(&time_of_day) {
        memory.tuning.preferred_session_times.push(time_of_day);
    }

    // Track block type engagement (shown count)
    for block_label in &blocks_shown {
        let engagement = memory
            .tuning
            .block_type_engagement
            .entry(block_label.clone())
            .or_default();
        engagement.shown += 1;
    }

    // Prune to last 50 sessions
    if memory.sessions.len() > 50 {
        let start = memory.sessions.len() - 50;
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

/// Record that the user engaged with a specific action/block
pub fn record_engagement(action: &str) -> Result<(), String> {
    let mut memory = read_memory_file()?;
    memory.tuning.total_actions_clicked += 1;

    let engagement = memory
        .tuning
        .block_type_engagement
        .entry(action.to_string())
        .or_default();
    engagement.interacted += 1;

    write_memory_file(&memory)
}

/// Add or update a semantic fact
pub fn upsert_fact(category: &str, content: &str, source: &str) -> Result<(), String> {
    let mut memory = read_memory_file()?;
    let now = Utc::now().to_rfc3339();

    // Check if a similar fact exists
    let existing = memory.facts.iter_mut().find(|f| {
        f.category == category && f.content.to_lowercase() == content.to_lowercase()
    });

    if let Some(fact) = existing {
        fact.last_confirmed = now;
        fact.confidence = (fact.confidence + 0.1).min(1.0);
    } else {
        memory.facts.push(SemanticFact {
            id: uuid::Uuid::new_v4().to_string(),
            category: category.to_string(),
            content: content.to_string(),
            confidence: 0.7,
            source: source.to_string(),
            created_at: now.clone(),
            last_confirmed: now,
            superseded_by: None,
        });
    }

    // Cap facts at 200
    if memory.facts.len() > 200 {
        // Remove lowest confidence facts
        memory.facts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        memory.facts.truncate(200);
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

#[tauri::command]
pub async fn get_memory_stats() -> Result<serde_json::Value, String> {
    let memory = read_memory_file()?;
    Ok(serde_json::json!({
        "total_sessions": memory.tuning.total_sessions,
        "total_actions_clicked": memory.tuning.total_actions_clicked,
        "total_inputs_submitted": memory.tuning.total_inputs_submitted,
        "facts_count": memory.facts.len(),
        "patterns_count": memory.patterns.len(),
        "insights_count": memory.accumulated_insights.len(),
        "block_engagement": memory.tuning.block_type_engagement,
        "preferred_times": memory.tuning.preferred_session_times,
    }))
}

#[tauri::command]
pub async fn record_action_engagement(action: String) -> Result<(), String> {
    record_engagement(&action)
}
