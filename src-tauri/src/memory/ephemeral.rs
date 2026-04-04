use serde::{Deserialize, Serialize};

/// Current session state — lives in Tauri managed state, cleared on app close.
/// This is the fastest memory tier: no disk I/O, pure in-process state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EphemeralMemory {
    /// Number of blocks shown in the current session.
    pub blocks_shown: u32,
    /// User inputs in the current session.
    pub inputs: Vec<String>,
    /// Number of conversation turns.
    pub conversation_turns: u32,
    /// Current ambient mood set by the model.
    pub mood: Option<String>,
    /// Active role (builder/reflector/planner/coach).
    pub active_role: Option<String>,
    /// Timestamp when the session started.
    pub session_start: Option<String>,
    /// Model source used most recently.
    pub last_model_source: Option<String>,
    /// Observations from the heartbeat during this session.
    pub heartbeat_observations: Vec<String>,
}

impl EphemeralMemory {
    pub fn new() -> Self {
        Self {
            session_start: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    /// Record a user input.
    pub fn record_input(&mut self, input: &str) {
        self.inputs.push(input.to_string());
        self.conversation_turns += 1;
    }

    /// Record blocks being shown.
    pub fn record_blocks(&mut self, count: u32) {
        self.blocks_shown += count;
    }

    /// Record a heartbeat observation.
    pub fn record_observation(&mut self, detail: &str) {
        self.heartbeat_observations.push(detail.to_string());
    }

    /// Get session duration in seconds (if session_start is set).
    pub fn session_duration_secs(&self) -> Option<i64> {
        let start = self.session_start.as_ref()?;
        let start_time = chrono::DateTime::parse_from_rfc3339(start).ok()?;
        let elapsed = chrono::Utc::now() - start_time.with_timezone(&chrono::Utc);
        Some(elapsed.num_seconds())
    }

    /// Build a summary string for inclusion in reasoning context.
    pub fn context_summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(duration) = self.session_duration_secs() {
            let mins = duration / 60;
            parts.push(format!("Session active for {}m", mins));
        }

        parts.push(format!("{} blocks shown", self.blocks_shown));
        parts.push(format!("{} inputs", self.inputs.len()));

        if let Some(ref mood) = self.mood {
            parts.push(format!("Mood: {}", mood));
        }
        if let Some(ref role) = self.active_role {
            parts.push(format!("Role: {}", role));
        }

        if !self.heartbeat_observations.is_empty() {
            parts.push(format!(
                "{} heartbeat observations",
                self.heartbeat_observations.len()
            ));
        }

        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let mem = EphemeralMemory::new();
        assert!(mem.session_start.is_some());
        assert_eq!(mem.blocks_shown, 0);
        assert!(mem.inputs.is_empty());
    }

    #[test]
    fn test_record_input() {
        let mut mem = EphemeralMemory::new();
        mem.record_input("hello");
        mem.record_input("what's my priority?");
        assert_eq!(mem.inputs.len(), 2);
        assert_eq!(mem.conversation_turns, 2);
    }

    #[test]
    fn test_context_summary() {
        let mut mem = EphemeralMemory::new();
        mem.record_blocks(5);
        mem.record_input("test");
        mem.mood = Some("focused".to_string());

        let summary = mem.context_summary();
        assert!(summary.contains("5 blocks shown"));
        assert!(summary.contains("1 inputs"));
        assert!(summary.contains("Mood: focused"));
    }
}
