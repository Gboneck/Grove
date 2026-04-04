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
    pub semantic_facts: String,
    pub tuning_hints: String,
    pub plugin_data: String,
    pub conversation_history: Option<String>,
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

        // Semantic facts
        let semantic_facts = if memory_data.facts.is_empty() {
            String::new()
        } else {
            let facts: Vec<String> = memory_data
                .facts
                .iter()
                .filter(|f| f.superseded_by.is_none() && f.confidence >= 0.5)
                .take(30)
                .map(|f| format!("- [{}] {} (confidence: {:.1})", f.category, f.content, f.confidence))
                .collect();
            if facts.is_empty() {
                String::new()
            } else {
                format!("\n--- KNOWN FACTS ABOUT USER ---\n{}", facts.join("\n"))
            }
        };

        // Self-tuning hints
        let tuning_hints = {
            let t = &memory_data.tuning;
            if t.total_sessions == 0 {
                String::new()
            } else {
                let mut hints = Vec::new();

                // Find most/least engaged block types
                let mut engagements: Vec<_> = t.block_type_engagement.iter().collect();
                engagements.sort_by(|a, b| {
                    let rate_a = if a.1.shown > 0 { a.1.interacted as f64 / a.1.shown as f64 } else { 0.0 };
                    let rate_b = if b.1.shown > 0 { b.1.interacted as f64 / b.1.shown as f64 } else { 0.0 };
                    rate_b.partial_cmp(&rate_a).unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some((name, eng)) = engagements.first() {
                    if eng.interacted > 0 {
                        hints.push(format!("User engages most with: {}", name));
                    }
                }
                if let Some((name, eng)) = engagements.last() {
                    if eng.shown > 3 && eng.interacted == 0 {
                        hints.push(format!("User rarely engages with: {} (shown {} times, never clicked)", name, eng.shown));
                    }
                }

                if !t.preferred_session_times.is_empty() {
                    hints.push(format!("User typically opens Grove during: {}", t.preferred_session_times.join(", ")));
                }

                if hints.is_empty() {
                    String::new()
                } else {
                    format!("\n--- SELF-TUNING OBSERVATIONS ---\n{}", hints.join("\n"))
                }
            }
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
            semantic_facts,
            tuning_hints,
            plugin_data: String::new(), // Filled by caller if plugins are active
            conversation_history: None, // Filled by caller for multi-turn
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
{}{}{}
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
            self.semantic_facts,
            self.tuning_hints,
            self.plugin_data,
            self.conversation_history
                .as_ref()
                .map(|h| format!("\n--- CONVERSATION HISTORY ---\n{}", h))
                .unwrap_or_default(),
            self.user_input
                .as_ref()
                .map(|i| format!("--- USER INPUT ---\n{}", i))
                .unwrap_or_default()
        )
    }
}
