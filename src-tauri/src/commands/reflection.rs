use chrono::{Datelike, DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use super::memory::{read_memory_file, Session};

/// A weekly digest synthesized from session history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyDigest {
    pub week_start: String,
    pub week_end: String,
    pub generated_at: String,
    pub session_count: usize,
    pub active_days: Vec<String>,
    pub top_topics: Vec<String>,
    pub mood_trend: String,
    pub key_insights: Vec<String>,
    pub stuck_ventures: Vec<String>,
    pub momentum_ventures: Vec<String>,
    pub behavioral_patterns: Vec<String>,
    pub recommendation: String,
}

/// Generate a weekly digest from the last 7 days of sessions.
pub fn generate_weekly_digest() -> Result<WeeklyDigest, String> {
    let memory = read_memory_file()?;
    let now = Utc::now();
    let week_ago = now - Duration::days(7);

    // Filter sessions from the past week
    let week_sessions: Vec<&Session> = memory
        .sessions
        .iter()
        .filter(|s| {
            DateTime::parse_from_rfc3339(&s.timestamp)
                .map(|dt| dt.with_timezone(&Utc) >= week_ago)
                .unwrap_or(false)
        })
        .collect();

    let session_count = week_sessions.len();

    // Active days
    let active_days: Vec<String> = week_sessions
        .iter()
        .map(|s| s.day_of_week.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Extract topics from user inputs and session summaries
    let mut topic_counts: HashMap<String, u32> = HashMap::new();
    for session in &week_sessions {
        // Count words from summaries as rough topics
        for word in session.session_summary.split_whitespace() {
            let w = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if w.len() > 4 && !is_stop_word(&w) {
                *topic_counts.entry(w).or_default() += 1;
            }
        }
        for input in &session.user_inputs {
            for word in input.text.split_whitespace() {
                let w = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                if w.len() > 4 && !is_stop_word(&w) {
                    *topic_counts.entry(w).or_default() += 1;
                }
            }
        }
    }
    let mut top_topics: Vec<(String, u32)> = topic_counts.into_iter().collect();
    top_topics.sort_by(|a, b| b.1.cmp(&a.1));
    let top_topics: Vec<String> = top_topics.into_iter().take(5).map(|(w, _)| w).collect();

    // Mood trend from time_of_day distribution
    let mut time_dist: HashMap<String, u32> = HashMap::new();
    for session in &week_sessions {
        *time_dist.entry(session.time_of_day.clone()).or_default() += 1;
    }
    let mood_trend = if time_dist.get("morning").copied().unwrap_or(0) > time_dist.get("evening").copied().unwrap_or(0) {
        "proactive — you're front-loading your days".to_string()
    } else if time_dist.get("evening").copied().unwrap_or(0) > time_dist.get("morning").copied().unwrap_or(0) {
        "reactive — most activity is late in the day".to_string()
    } else if time_dist.get("late night").copied().unwrap_or(0) > 2 {
        "burning late — multiple late night sessions this week".to_string()
    } else {
        "balanced — sessions spread throughout the day".to_string()
    };

    // Key insights from accumulated
    let key_insights: Vec<String> = memory
        .accumulated_insights
        .iter()
        .rev()
        .take(5)
        .cloned()
        .collect();

    // Detect stuck ventures — mentioned in sessions but no progress keywords
    let context_path = dirs::home_dir()
        .ok_or("No home dir")?
        .join(".grove")
        .join("context.json");
    let (stuck_ventures, momentum_ventures) = if let Ok(ctx_str) = fs::read_to_string(&context_path) {
        if let Ok(ctx) = serde_json::from_str::<serde_json::Value>(&ctx_str) {
            analyze_venture_momentum(&ctx, &week_sessions)
        } else {
            (Vec::new(), Vec::new())
        }
    } else {
        (Vec::new(), Vec::new())
    };

    // Behavioral patterns from memory
    let behavioral_patterns: Vec<String> = memory
        .patterns
        .iter()
        .take(3)
        .map(|p| p.description.clone())
        .collect();

    // Generate recommendation
    let recommendation = if session_count == 0 {
        "You didn't use Grove this week. Consider checking in at least once a day to build the habit.".to_string()
    } else if !stuck_ventures.is_empty() {
        format!(
            "Focus on unblocking: {}. These haven't shown progress this week.",
            stuck_ventures.join(", ")
        )
    } else if session_count > 20 {
        "You're checking in frequently — that's great. Make sure you're acting on insights, not just consuming them.".to_string()
    } else {
        "Solid week. Keep the momentum on your active ventures.".to_string()
    };

    Ok(WeeklyDigest {
        week_start: week_ago.format("%Y-%m-%d").to_string(),
        week_end: now.format("%Y-%m-%d").to_string(),
        generated_at: now.to_rfc3339(),
        session_count,
        active_days,
        top_topics,
        mood_trend,
        key_insights,
        stuck_ventures,
        momentum_ventures,
        behavioral_patterns,
        recommendation,
    })
}

/// Analyze which ventures have momentum vs which are stuck
fn analyze_venture_momentum(
    context: &serde_json::Value,
    sessions: &[&Session],
) -> (Vec<String>, Vec<String>) {
    let mut stuck = Vec::new();
    let mut momentum = Vec::new();

    let ventures = match context.get("ventures").and_then(|v| v.as_array()) {
        Some(v) => v,
        None => return (stuck, momentum),
    };

    // Build a map of venture mentions in recent sessions
    let all_text: String = sessions
        .iter()
        .map(|s| {
            let mut t = s.session_summary.clone();
            for input in &s.user_inputs {
                t.push(' ');
                t.push_str(&input.text);
            }
            t
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    for venture in ventures {
        let name = match venture.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let status = venture
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        // Skip completed/paused ventures
        if status == "completed" || status == "paused" || status == "archived" {
            continue;
        }

        let health = venture
            .get("health")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown");

        let mentioned = all_text.contains(&name.to_lowercase());

        if health == "red" || (status == "active" && !mentioned) {
            stuck.push(name.to_string());
        } else if mentioned && (health == "green" || health == "yellow") {
            momentum.push(name.to_string());
        }
    }

    (stuck, momentum)
}

/// Save weekly digest to ~/.grove/digests/
pub fn save_digest(digest: &WeeklyDigest) -> Result<String, String> {
    let grove_dir = dirs::home_dir()
        .ok_or("No home dir")?
        .join(".grove")
        .join("digests");
    fs::create_dir_all(&grove_dir)
        .map_err(|e| format!("Failed to create digests dir: {}", e))?;

    let filename = format!("week-{}.json", digest.week_start);
    let path = grove_dir.join(&filename);
    let content = serde_json::to_string_pretty(digest)
        .map_err(|e| format!("Failed to serialize digest: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write digest: {}", e))?;

    Ok(filename)
}

/// Check if a weekly digest should be generated (once per week)
pub fn should_generate_digest() -> bool {
    let grove_dir = match dirs::home_dir() {
        Some(h) => h.join(".grove").join("digests"),
        None => return false,
    };

    if !grove_dir.exists() {
        return true;
    }

    let now = Utc::now();
    let week_start = (now - Duration::days(now.weekday().num_days_from_monday() as i64))
        .format("%Y-%m-%d")
        .to_string();

    let expected = grove_dir.join(format!("week-{}.json", week_start));
    !expected.exists()
}

/// Build context string from the latest digest for injection into reasoning
pub fn digest_context() -> String {
    let grove_dir = match dirs::home_dir() {
        Some(h) => h.join(".grove").join("digests"),
        None => return String::new(),
    };

    if !grove_dir.exists() {
        return String::new();
    }

    // Find the most recent digest
    let mut entries: Vec<_> = match fs::read_dir(&grove_dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return String::new(),
    };

    entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

    if let Some(latest) = entries.first() {
        if let Ok(content) = fs::read_to_string(latest.path()) {
            if let Ok(digest) = serde_json::from_str::<WeeklyDigest>(&content) {
                return format!(
                    "\n--- WEEKLY DIGEST ({} to {}) ---\nSessions: {} | Active days: {}\nMood: {}\nMomentum: {}\nStuck: {}\nRecommendation: {}",
                    digest.week_start,
                    digest.week_end,
                    digest.session_count,
                    digest.active_days.join(", "),
                    digest.mood_trend,
                    if digest.momentum_ventures.is_empty() { "none".to_string() } else { digest.momentum_ventures.join(", ") },
                    if digest.stuck_ventures.is_empty() { "none".to_string() } else { digest.stuck_ventures.join(", ") },
                    digest.recommendation,
                );
            }
        }
    }

    String::new()
}

/// Load active reminders for context injection
pub fn reminders_context() -> String {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".grove").join("reminders.json"),
        None => return String::new(),
    };

    if !path.exists() {
        return String::new();
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let reminders: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    let active: Vec<String> = reminders
        .iter()
        .filter(|r| !r.get("dismissed").and_then(|d| d.as_bool()).unwrap_or(false))
        .filter_map(|r| r.get("message").and_then(|m| m.as_str()).map(String::from))
        .collect();

    if active.is_empty() {
        return String::new();
    }

    format!(
        "\n--- ACTIVE REMINDERS ---\n{}",
        active.iter().map(|r| format!("- {}", r)).collect::<Vec<_>>().join("\n")
    )
}

fn is_stop_word(w: &str) -> bool {
    matches!(
        w,
        "about" | "after" | "being" | "between" | "could"
            | "during" | "every" | "first" | "found" | "great"
            | "having" | "their" | "there" | "these" | "thing"
            | "think" | "those" | "through" | "under" | "using"
            | "which" | "while" | "would" | "should" | "where"
            | "reasoning" | "session" | "cycle" | "completed"
    )
}

// ── Tauri Commands ──

#[tauri::command]
pub async fn get_weekly_digest() -> Result<serde_json::Value, String> {
    let digest = generate_weekly_digest()?;
    serde_json::to_value(&digest).map_err(|e| format!("Failed to serialize digest: {}", e))
}

#[tauri::command]
pub async fn generate_and_save_digest() -> Result<serde_json::Value, String> {
    let digest = generate_weekly_digest()?;
    save_digest(&digest)?;
    serde_json::to_value(&digest).map_err(|e| format!("Failed to serialize digest: {}", e))
}
