use serde::{Deserialize, Serialize};

/// Current session state — cleared when the app closes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EphemeralMemory {
    /// Blocks shown in the current session.
    pub blocks_shown: u32,
    /// User inputs in the current session.
    pub inputs: Vec<String>,
    /// Current conversation context.
    pub conversation_turns: u32,
    /// Current ambient mood.
    pub mood: Option<String>,
    /// Active role.
    pub active_role: Option<String>,
}

// TODO (Session 2): Wire into App state and pass to context builder.
