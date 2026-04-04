use serde::{Deserialize, Serialize};

/// An observation from the heartbeat cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub timestamp: String,
    pub kind: ObservationKind,
    pub detail: String,
}

/// Types of observations the heartbeat can make.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationKind {
    /// A file was modified in a watched directory.
    FileChanged,
    /// Time-of-day context changed (morning → afternoon).
    TimeShift,
    /// A venture deadline is approaching.
    DeadlineApproaching,
    /// System state changed (went online/offline).
    SystemState,
    /// User hasn't interacted in a while.
    Idle,
}

// TODO (Session 2): Implement Observer that watches filesystem, clock, and system state.
// Will use the `notify` crate for real filesystem events instead of polling.
