use serde::{Deserialize, Serialize};

/// The 9 phases of the Grove-user relationship, modeled after Daemon's conversation engine.
/// Each phase changes how the system behaves — early phases observe, later phases act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipPhase {
    /// Phase 1: First interactions. System observes, asks gentle questions, builds Soul.md.
    Awakening,
    /// Phase 2: Learning preferences, work patterns, venture details.
    Discovery,
    /// Phase 3: Offering insights, connecting dots between ventures.
    Deepening,
    /// Phase 4: Pushing back gently, questioning assumptions, suggesting alternatives.
    Challenge,
    /// Phase 5: Synthesizing patterns across time, generating weekly digests.
    Synthesis,
    /// Phase 6: Proactively suggesting actions, drafting outputs.
    Integration,
    /// Phase 7: Autonomous action on low-risk tasks, anticipating needs.
    Evolution,
    /// Phase 8: Deep strategic partnership, long-term planning.
    Mastery,
    /// Phase 9: System and user operate as seamless unit.
    Transcendence,
}

impl RelationshipPhase {
    /// Determine the phase based on Soul.md completeness and session count.
    pub fn from_metrics(soul_completeness: f64, session_count: u32) -> Self {
        match (soul_completeness, session_count) {
            (c, s) if c < 0.3 || s < 3 => Self::Awakening,
            (c, s) if c < 0.5 || s < 10 => Self::Discovery,
            (c, s) if c < 0.6 || s < 25 => Self::Deepening,
            (c, s) if c < 0.7 || s < 50 => Self::Challenge,
            (c, s) if c < 0.75 || s < 80 => Self::Synthesis,
            (c, s) if c < 0.8 || s < 120 => Self::Integration,
            (c, s) if c < 0.85 || s < 200 => Self::Evolution,
            (c, s) if c < 0.9 || s < 350 => Self::Mastery,
            _ => Self::Transcendence,
        }
    }

    /// System prompt prefix that modifies reasoning behavior for this phase.
    pub fn system_prompt_modifier(&self) -> &'static str {
        match self {
            Self::Awakening => concat!(
                "You are in the AWAKENING phase with this user. ",
                "Focus on observation and gentle questions. ",
                "Do not assume — ask. Build understanding before acting. ",
                "Suggest the user edit their Soul.md to add more context."
            ),
            Self::Discovery => concat!(
                "You are in the DISCOVERY phase. ",
                "You know some basics about the user. ",
                "Start connecting dots between their projects. ",
                "Offer observations but frame them as questions: 'I notice X — is that right?'"
            ),
            Self::Deepening => concat!(
                "You are in the DEEPENING phase. ",
                "You have moderate confidence in who this user is. ",
                "Offer insights that connect patterns across their ventures. ",
                "Start suggesting specific actions based on their priorities."
            ),
            Self::Challenge => concat!(
                "You are in the CHALLENGE phase. ",
                "You know this user well enough to push back gently. ",
                "Question assumptions. Suggest alternatives. ",
                "If they're spreading too thin, say so."
            ),
            Self::Synthesis => concat!(
                "You are in the SYNTHESIS phase. ",
                "Generate weekly patterns and cross-venture insights. ",
                "Proactively surface information the user hasn't asked for but needs."
            ),
            Self::Integration => concat!(
                "You are in the INTEGRATION phase. ",
                "Proactively suggest actions. Draft outputs. ",
                "You have strong confidence in the user's preferences and can anticipate needs."
            ),
            Self::Evolution => concat!(
                "You are in the EVOLUTION phase. ",
                "Take autonomous action on low-risk tasks. ",
                "Anticipate needs before the user expresses them. ",
                "You are becoming a trusted partner."
            ),
            Self::Mastery => concat!(
                "You are in the MASTERY phase. ",
                "Deep strategic partnership. Long-term planning. ",
                "You understand the user's multi-year trajectory and can advise accordingly."
            ),
            Self::Transcendence => concat!(
                "You are in the TRANSCENDENCE phase. ",
                "You and the user operate as a seamless unit. ",
                "Minimal friction. Maximum anticipation. ",
                "You know what they need before they do."
            ),
        }
    }

    /// Autonomy level for this phase (0.0 = ask everything, 1.0 = act freely).
    pub fn autonomy_level(&self) -> f64 {
        match self {
            Self::Awakening => 0.0,
            Self::Discovery => 0.1,
            Self::Deepening => 0.2,
            Self::Challenge => 0.3,
            Self::Synthesis => 0.4,
            Self::Integration => 0.6,
            Self::Evolution => 0.75,
            Self::Mastery => 0.85,
            Self::Transcendence => 0.95,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Awakening => "Awakening",
            Self::Discovery => "Discovery",
            Self::Deepening => "Deepening",
            Self::Challenge => "Challenge",
            Self::Synthesis => "Synthesis",
            Self::Integration => "Integration",
            Self::Evolution => "Evolution",
            Self::Mastery => "Mastery",
            Self::Transcendence => "Transcendence",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_progression() {
        assert_eq!(RelationshipPhase::from_metrics(0.1, 1), RelationshipPhase::Awakening);
        assert_eq!(RelationshipPhase::from_metrics(0.4, 8), RelationshipPhase::Discovery);
        assert_eq!(RelationshipPhase::from_metrics(0.55, 20), RelationshipPhase::Deepening);
        assert_eq!(RelationshipPhase::from_metrics(0.65, 40), RelationshipPhase::Challenge);
        assert_eq!(RelationshipPhase::from_metrics(0.95, 500), RelationshipPhase::Transcendence);
    }

    #[test]
    fn test_autonomy_increases() {
        let phases = [
            RelationshipPhase::Awakening,
            RelationshipPhase::Discovery,
            RelationshipPhase::Deepening,
            RelationshipPhase::Challenge,
            RelationshipPhase::Synthesis,
            RelationshipPhase::Integration,
            RelationshipPhase::Evolution,
            RelationshipPhase::Mastery,
            RelationshipPhase::Transcendence,
        ];
        for i in 1..phases.len() {
            assert!(phases[i].autonomy_level() > phases[i - 1].autonomy_level());
        }
    }
}
