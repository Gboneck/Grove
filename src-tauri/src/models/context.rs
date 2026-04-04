use chrono::Local;
use serde::Serialize;

use crate::commands::memory;

/// All context assembled for the reasoning engine
#[derive(Debug, Clone, Serialize)]
pub struct GroveContext {
    pub soul_md: String,
    pub context_json: String,
    pub recent_memory: String,
    pub accumulated_insights: String,
    pub local_time: String,
    pub day_of_week: String,
    pub date: String,
    pub last_seen: String,
    pub user_input: Option<String>,
}

impl GroveContext {
    pub fn gather(user_input: Option<String>) -> Result<Self, String> {
        let grove_dir = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".grove");

        // Read soul.md
        let soul_md = std::fs::read_to_string(grove_dir.join("soul.md"))
            .map_err(|e| format!("Failed to read soul.md: {}", e))?;

        // Read context.json
        let context_json = std::fs::read_to_string(grove_dir.join("context.json"))
            .map_err(|e| format!("Failed to read context.json: {}", e))?;

        // Read memory
        let memory_data = memory::read_memory_file().unwrap_or_default();
        let recent_sessions: Vec<_> = memory_data.sessions.iter().rev().take(5).collect();

        let recent_memory = if recent_sessions.is_empty() {
            "No previous sessions.".to_string()
        } else {
            recent_sessions
                .iter()
                .map(|s| {
                    let inputs: Vec<String> =
                        s.user_inputs.iter().map(|i| i.text.clone()).collect();
                    format!(
                        "- {} ({}): {} | User said: {}",
                        s.timestamp,
                        s.time_of_day,
                        s.session_summary,
                        if inputs.is_empty() {
                            "(no input)".to_string()
                        } else {
                            inputs.join(", ")
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let accumulated_insights = if memory_data.accumulated_insights.is_empty() {
            "None yet.".to_string()
        } else {
            memory_data
                .accumulated_insights
                .iter()
                .map(|i| format!("- {}", i))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let last_seen = memory_data
            .last_seen
            .unwrap_or_else(|| "never — this is the first session".to_string());

        let now = Local::now();

        Ok(GroveContext {
            soul_md,
            context_json,
            recent_memory,
            accumulated_insights,
            local_time: now.to_rfc3339(),
            day_of_week: now.format("%A").to_string(),
            date: now.format("%B %-d, %Y").to_string(),
            last_seen,
            user_input,
        })
    }

    /// Build the user message sent to the model
    pub fn to_user_message(&self) -> String {
        format!(
            r#"Current time: {}
Day: {}, {}
Time since last session: {}

--- SOUL.MD ---
{}

--- ACTIVE CONTEXT ---
{}

--- RECENT MEMORY ---
{}

--- ACCUMULATED INSIGHTS ---
{}

{}

Decide what to show. Return JSON only."#,
            self.local_time,
            self.day_of_week,
            self.date,
            self.last_seen,
            self.soul_md,
            self.context_json,
            self.recent_memory,
            self.accumulated_insights,
            self.user_input
                .as_ref()
                .map(|i| format!("--- USER INPUT ---\n{}", i))
                .unwrap_or_default()
        )
    }
}
