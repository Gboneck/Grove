pub mod scoring;

use scoring::{ActionGate, AutonomyScore, category_gate};
use crate::models::AutoAction;
use crate::soul::evolution::RelationshipPhase;

/// Filter and gate autonomous actions based on autonomy scoring.
/// Returns (approved, blocked) action lists.
pub fn gate_actions(
    actions: &[AutoAction],
    phase: RelationshipPhase,
) -> (Vec<AutoAction>, Vec<String>) {
    let phase_autonomy = phase.autonomy_level();
    let mut approved = Vec::new();
    let mut blocked = Vec::new();

    for action in actions {
        // First check category-level gate
        let cat_gate = match action.action_type.as_str() {
            "note" | "add_fact" | "reminder" => category_gate("memory_update"),
            "file_write" => {
                // Check if path is within ~/.grove/
                let path = action.params.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if path.contains(".grove") {
                    category_gate("file_write_grove")
                } else {
                    category_gate("file_write_external")
                }
            }
            "venture_status" => category_gate("ui_composition"),
            "shell" => category_gate("shell_command"),
            "http" | "api_call" => category_gate("external_api"),
            "send_message" | "email" => category_gate("send_message"),
            "purchase" => category_gate("purchase"),
            _ => category_gate("unknown"),
        };

        // If category is Block, reject immediately
        if cat_gate == ActionGate::Block {
            blocked.push(format!(
                "Blocked (category): {} — {}",
                action.action_type, action.description
            ));
            continue;
        }

        // Build autonomy score for this specific action
        let score = score_action(action);
        let gate = score.gate(phase_autonomy);

        match gate {
            ActionGate::Auto => {
                approved.push(action.clone());
            }
            ActionGate::Ask => {
                // In the current implementation, "Ask" actions are logged but auto-approved
                // for low-risk items. In a future UI update, these would prompt the user.
                if phase_autonomy >= 0.3 && score.composite() >= 0.4 {
                    approved.push(action.clone());
                    eprintln!(
                        "[grove:autonomy] Auto-approved (phase {}): {} — {}",
                        phase.display_name(),
                        action.action_type,
                        action.description
                    );
                } else {
                    blocked.push(format!(
                        "Needs approval (score {:.2}): {} — {}",
                        score.composite(),
                        action.action_type,
                        action.description
                    ));
                }
            }
            ActionGate::Block => {
                blocked.push(format!(
                    "Blocked (score {:.2}): {} — {}",
                    score.composite(),
                    action.action_type,
                    action.description
                ));
            }
        }
    }

    (approved, blocked)
}

/// Score an individual action based on its type and parameters.
fn score_action(action: &AutoAction) -> AutonomyScore {
    let (reversibility, scope) = match action.action_type.as_str() {
        "note" => (1.0, 1.0),       // Can delete notes, local only
        "add_fact" => (0.9, 1.0),    // Can remove facts, local only
        "reminder" => (1.0, 1.0),    // Can dismiss reminders
        "file_write" => (0.7, 0.8),  // Files can be restored but could overwrite
        "venture_status" => (0.9, 0.9), // Can revert status
        "shell" => (0.3, 0.3),       // Could be destructive
        "http" | "api_call" => (0.2, 0.2), // External side effects
        "send_message" | "email" => (0.0, 0.0), // Irreversible
        "purchase" => (0.0, 0.0),    // Financial
        _ => (0.5, 0.5),
    };

    // Extract confidence from action params if available
    let confidence = action.params
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7);

    AutonomyScore {
        reversibility,
        scope,
        confidence,
        precedent: 0.5, // Default; future: track per-action-type approval history
        urgency: action.params
            .get("urgency")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_action(action_type: &str, desc: &str) -> AutoAction {
        AutoAction {
            action_type: action_type.to_string(),
            description: desc.to_string(),
            params: serde_json::Map::new(),
        }
    }

    fn make_action_with_path(path: &str) -> AutoAction {
        let mut params = serde_json::Map::new();
        params.insert("path".to_string(), json!(path));
        AutoAction {
            action_type: "file_write".to_string(),
            description: "Write file".to_string(),
            params,
        }
    }

    #[test]
    fn test_notes_always_approved() {
        let actions = vec![make_action("note", "Test note")];
        let (approved, blocked) = gate_actions(&actions, RelationshipPhase::Awakening);
        assert_eq!(approved.len(), 1);
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_purchases_always_blocked() {
        let actions = vec![make_action("purchase", "Buy something")];
        let (approved, blocked) = gate_actions(&actions, RelationshipPhase::Transcendence);
        assert!(approved.is_empty());
        assert_eq!(blocked.len(), 1);
    }

    #[test]
    fn test_external_file_write_blocked_early_phase() {
        let actions = vec![make_action_with_path("/home/user/important.txt")];
        let (approved, blocked) = gate_actions(&actions, RelationshipPhase::Awakening);
        assert!(approved.is_empty());
        assert_eq!(blocked.len(), 1);
    }

    #[test]
    fn test_grove_file_write_approved() {
        let actions = vec![make_action_with_path("~/.grove/notes/test.md")];
        let (approved, _blocked) = gate_actions(&actions, RelationshipPhase::Discovery);
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn test_shell_blocked_early() {
        let actions = vec![make_action("shell", "rm -rf /")];
        let (approved, blocked) = gate_actions(&actions, RelationshipPhase::Awakening);
        assert!(approved.is_empty());
        assert!(!blocked.is_empty());
    }
}
