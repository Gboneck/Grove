use serde::{Deserialize, Serialize};

/// A detected behavioral pattern from accumulated observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub description: String,
    pub confidence: f64,
    pub occurrences: u32,
    pub first_seen: String,
    pub last_seen: String,
}

// TODO (Session 2): Implement pattern detection from observation history.
// Detect: time-of-day patterns, file activity patterns, venture focus patterns.
