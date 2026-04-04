use serde::{Deserialize, Serialize};

/// The 5 factors used to score whether an action should be taken autonomously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyScore {
    /// Can this action be undone? (0.0 = irreversible, 1.0 = fully reversible)
    pub reversibility: f64,
    /// How broad is the impact? (0.0 = external/financial, 1.0 = local file only)
    pub scope: f64,
    /// Model's self-assessed certainty (0.0-1.0)
    pub confidence: f64,
    /// Has the user approved similar actions before? (0.0 = never, 1.0 = always)
    pub precedent: f64,
    /// How time-sensitive is this? (0.0 = can wait, 1.0 = urgent)
    pub urgency: f64,
}

/// The decision gate for autonomous actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionGate {
    /// Execute automatically without asking.
    Auto,
    /// Ask the user before executing.
    Ask,
    /// Block entirely — never execute autonomously.
    Block,
}

impl AutonomyScore {
    /// Calculate the weighted composite score (0.0-1.0).
    pub fn composite(&self) -> f64 {
        let weights = [0.25, 0.25, 0.20, 0.15, 0.15];
        let values = [
            self.reversibility,
            self.scope,
            self.confidence,
            self.precedent,
            self.urgency,
        ];
        weights
            .iter()
            .zip(values.iter())
            .map(|(w, v)| w * v)
            .sum()
    }

    /// Determine the gate based on composite score and relationship phase autonomy level.
    pub fn gate(&self, phase_autonomy: f64) -> ActionGate {
        let score = self.composite();
        let threshold = 1.0 - phase_autonomy; // Higher autonomy = lower threshold

        if score >= threshold.max(0.3) {
            ActionGate::Auto
        } else if score >= 0.15 {
            ActionGate::Ask
        } else {
            ActionGate::Block
        }
    }
}

/// Pre-defined gates for common action categories.
pub fn category_gate(category: &str) -> ActionGate {
    match category {
        "ui_composition" | "memory_update" => ActionGate::Auto,
        "file_write_grove" => ActionGate::Auto, // Within ~/.grove/ sandbox
        "file_write_external" | "shell_command" | "external_api" | "system_change" => {
            ActionGate::Ask
        }
        "purchase" | "send_message" | "email" => ActionGate::Block,
        _ => ActionGate::Ask,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_confidence_auto() {
        let score = AutonomyScore {
            reversibility: 1.0,
            scope: 1.0,
            confidence: 0.9,
            precedent: 0.8,
            urgency: 0.5,
        };
        assert_eq!(score.gate(0.5), ActionGate::Auto);
    }

    #[test]
    fn test_low_confidence_ask() {
        let score = AutonomyScore {
            reversibility: 0.5,
            scope: 0.5,
            confidence: 0.3,
            precedent: 0.0,
            urgency: 0.2,
        };
        assert_eq!(score.gate(0.1), ActionGate::Ask);
    }

    #[test]
    fn test_category_gates() {
        assert_eq!(category_gate("ui_composition"), ActionGate::Auto);
        assert_eq!(category_gate("shell_command"), ActionGate::Ask);
        assert_eq!(category_gate("purchase"), ActionGate::Block);
    }
}
