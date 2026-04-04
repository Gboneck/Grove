use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::autopatch;
use super::parser::Soul;
use super::patcher::SoulPatch;
use crate::memory::longterm;
use crate::soul::evolution::RelationshipPhase;

/// A proposed evolution to the Soul document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionProposal {
    pub id: String,
    pub section: String,
    pub content: String,
    pub confidence_delta: f64,
    pub replace: bool,
    pub reason: String,
    pub source: EvolutionSource,
}

/// Where the evolution proposal came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionSource {
    ModelInsight,
    PatternDetection,
    ConfidenceDecay,
    UserConfirmation,
}

/// Result of the judge phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentResult {
    pub proposal_id: String,
    pub approved: bool,
    pub reason: String,
}

/// The self-evolution engine: propose → judge → apply.
/// Runs after reasoning cycles to keep Soul.md aligned with reality.
pub struct EvolutionEngine;

impl EvolutionEngine {
    /// Phase 1: Propose changes based on all available signals.
    pub fn propose(
        soul: &Soul,
        insights: &[String],
        phase: RelationshipPhase,
    ) -> Vec<EvolutionProposal> {
        let mut proposals = Vec::new();

        // 1. Insight-based proposals (from model output)
        let patches = autopatch::extract_patches(insights, soul);
        for patch in patches {
            proposals.push(EvolutionProposal {
                id: uuid::Uuid::new_v4().to_string(),
                section: patch.section,
                content: patch.content,
                confidence_delta: patch.confidence_delta,
                replace: patch.replace,
                reason: "Derived from model insight".to_string(),
                source: EvolutionSource::ModelInsight,
            });
        }

        // 2. Pattern-based proposals (from long-term memory)
        let lt_entries = longterm::read_entries();
        for entry in &lt_entries {
            if entry.confirmation_count >= 3 && entry.confidence >= 0.7 {
                // Check if this pattern is already in the soul
                let already = soul
                    .section("patterns")
                    .map(|s| {
                        s.content
                            .to_lowercase()
                            .contains(&entry.content.to_lowercase())
                    })
                    .unwrap_or(false);

                if !already {
                    proposals.push(EvolutionProposal {
                        id: uuid::Uuid::new_v4().to_string(),
                        section: "Patterns".to_string(),
                        content: format!("- {}", entry.content),
                        confidence_delta: 0.08,
                        replace: false,
                        reason: format!(
                            "Confirmed {} times with {:.0}% confidence",
                            entry.confirmation_count,
                            entry.confidence * 100.0
                        ),
                        source: EvolutionSource::PatternDetection,
                    });
                }
            }
        }

        // 3. Confidence decay proposals for stale sections
        for section in &soul.sections {
            if section.confidence > 0.3 && section.items.is_empty() && section.content.len() < 20 {
                proposals.push(EvolutionProposal {
                    id: uuid::Uuid::new_v4().to_string(),
                    section: section.heading.clone(),
                    content: String::new(),
                    confidence_delta: -0.05,
                    replace: false,
                    reason: "Section has minimal content — decaying confidence".to_string(),
                    source: EvolutionSource::ConfidenceDecay,
                });
            }
        }

        // Filter by phase — early phases get fewer proposals
        let max_proposals = match phase {
            RelationshipPhase::Awakening => 1,
            RelationshipPhase::Discovery => 2,
            RelationshipPhase::Deepening => 3,
            _ => proposals.len(),
        };

        proposals.truncate(max_proposals);
        proposals
    }

    /// Phase 2: Judge proposals based on safety and relevance.
    pub fn judge(proposals: &[EvolutionProposal], phase: RelationshipPhase) -> Vec<JudgmentResult> {
        proposals
            .iter()
            .map(|p| {
                let approved = Self::judge_one(p, phase);
                JudgmentResult {
                    proposal_id: p.id.clone(),
                    approved: approved.0,
                    reason: approved.1,
                }
            })
            .collect()
    }

    fn judge_one(proposal: &EvolutionProposal, phase: RelationshipPhase) -> (bool, String) {
        // Never allow replace in early phases
        if proposal.replace && phase.autonomy_level() < 0.3 {
            return (
                false,
                "Replace operations blocked in early relationship phases".to_string(),
            );
        }

        // Block large negative confidence deltas
        if proposal.confidence_delta < -0.1 {
            return (false, "Confidence reduction too aggressive".to_string());
        }

        // Block empty content additions (except decay)
        if proposal.content.is_empty() && proposal.confidence_delta >= 0.0 {
            return (false, "Empty content with no confidence change".to_string());
        }

        // Decay proposals are always allowed (they're conservative)
        if matches!(proposal.source, EvolutionSource::ConfidenceDecay) {
            return (true, "Confidence decay approved".to_string());
        }

        // Pattern-based proposals need higher phase
        if matches!(proposal.source, EvolutionSource::PatternDetection)
            && phase.autonomy_level() < 0.2
        {
            return (
                false,
                "Pattern-based evolution requires Deepening phase or later".to_string(),
            );
        }

        (true, "Proposal approved".to_string())
    }

    /// Phase 3: Apply approved proposals to the Soul.
    pub fn apply(
        soul: &Soul,
        proposals: &[EvolutionProposal],
        judgments: &[JudgmentResult],
    ) -> (Soul, Vec<String>) {
        let mut evolved = soul.clone();
        let mut applied = Vec::new();

        for proposal in proposals {
            let judgment = judgments.iter().find(|j| j.proposal_id == proposal.id);

            if let Some(j) = judgment {
                if !j.approved {
                    eprintln!(
                        "[grove:evolve] Rejected: {} — {}",
                        proposal.section, j.reason
                    );
                    continue;
                }
            } else {
                continue; // No judgment = skip
            }

            let patch = SoulPatch {
                section: proposal.section.clone(),
                content: proposal.content.clone(),
                confidence_delta: proposal.confidence_delta,
                replace: proposal.replace,
            };

            evolved = evolved.apply_patch(&patch);
            applied.push(format!(
                "Evolved '{}': {} ({})",
                proposal.section,
                proposal.reason,
                match proposal.source {
                    EvolutionSource::ModelInsight => "insight",
                    EvolutionSource::PatternDetection => "pattern",
                    EvolutionSource::ConfidenceDecay => "decay",
                    EvolutionSource::UserConfirmation => "confirmed",
                }
            ));
        }

        (evolved, applied)
    }

    /// Full cycle: propose → judge → apply → write.
    pub fn run_cycle(insights: &[String], phase: RelationshipPhase) -> Result<Vec<String>, String> {
        let grove_dir = dirs::home_dir().ok_or("No home directory")?.join(".grove");
        let soul_path = grove_dir.join("soul.md");

        let soul_raw = std::fs::read_to_string(&soul_path)
            .map_err(|e| format!("Failed to read soul.md: {}", e))?;
        let soul = Soul::parse(&soul_raw);

        let proposals = Self::propose(&soul, insights, phase);
        if proposals.is_empty() {
            return Ok(Vec::new());
        }

        let judgments = Self::judge(&proposals, phase);
        let (evolved, applied) = Self::apply(&soul, &proposals, &judgments);

        if !applied.is_empty() {
            let markdown = evolved.to_markdown();
            std::fs::write(&soul_path, markdown)
                .map_err(|e| format!("Failed to write soul.md: {}", e))?;
        }

        Ok(applied)
    }
}

/// Tauri command: get current evolution proposals (for UI preview).
#[tauri::command]
pub async fn get_evolution_proposals() -> Result<Vec<EvolutionProposal>, String> {
    let grove_dir = dirs::home_dir().ok_or("No home directory")?.join(".grove");

    let soul_raw = std::fs::read_to_string(grove_dir.join("soul.md"))
        .map_err(|e| format!("Failed to read soul.md: {}", e))?;
    let soul = Soul::parse(&soul_raw);

    let mem = crate::commands::memory::read_memory_file().unwrap_or_default();
    let phase = RelationshipPhase::from_metrics(soul.completeness(), mem.sessions.len() as u32);

    let proposals = EvolutionEngine::propose(&soul, &[], phase);
    Ok(proposals)
}

/// Tauri command: apply a specific evolution (user-confirmed).
#[tauri::command]
pub async fn apply_evolution(proposal_json: String) -> Result<String, String> {
    let proposal: EvolutionProposal =
        serde_json::from_str(&proposal_json).map_err(|e| format!("Invalid proposal: {}", e))?;

    let grove_dir = dirs::home_dir().ok_or("No home directory")?.join(".grove");
    let soul_path = grove_dir.join("soul.md");

    let soul_raw = std::fs::read_to_string(&soul_path)
        .map_err(|e| format!("Failed to read soul.md: {}", e))?;
    let soul = Soul::parse(&soul_raw);

    let patch = SoulPatch {
        section: proposal.section.clone(),
        content: proposal.content.clone(),
        confidence_delta: proposal.confidence_delta + 0.1, // Extra boost for user confirmation
        replace: proposal.replace,
    };

    let evolved = soul.apply_patch(&patch);
    let markdown = evolved.to_markdown();
    std::fs::write(&soul_path, markdown).map_err(|e| format!("Failed to write soul.md: {}", e))?;

    Ok(format!("Applied evolution to '{}'", proposal.section))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propose_from_insights() {
        let soul = Soul::parse("# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship Grove\n");
        let insights = vec!["User wants to learn Rust deeply".to_string()];
        let proposals = EvolutionEngine::propose(&soul, &insights, RelationshipPhase::Deepening);
        assert!(!proposals.is_empty());
        assert!(proposals[0].content.contains("Rust"));
    }

    #[test]
    fn test_judge_blocks_replace_early() {
        let proposal = EvolutionProposal {
            id: "test".to_string(),
            section: "Goals".to_string(),
            content: "- New goal".to_string(),
            confidence_delta: 0.0,
            replace: true,
            reason: "test".to_string(),
            source: EvolutionSource::ModelInsight,
        };
        let judgments = EvolutionEngine::judge(&[proposal], RelationshipPhase::Awakening);
        assert!(!judgments[0].approved);
    }

    #[test]
    fn test_judge_allows_decay() {
        let proposal = EvolutionProposal {
            id: "test".to_string(),
            section: "Goals".to_string(),
            content: String::new(),
            confidence_delta: -0.05,
            replace: false,
            reason: "decay".to_string(),
            source: EvolutionSource::ConfidenceDecay,
        };
        let judgments = EvolutionEngine::judge(&[proposal], RelationshipPhase::Awakening);
        assert!(judgments[0].approved);
    }

    #[test]
    fn test_full_apply_cycle() {
        let soul = Soul::parse("# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship Grove\n");
        let proposals = vec![EvolutionProposal {
            id: "p1".to_string(),
            section: "Goals".to_string(),
            content: "- Learn Rust".to_string(),
            confidence_delta: 0.05,
            replace: false,
            reason: "test".to_string(),
            source: EvolutionSource::ModelInsight,
        }];
        let judgments = vec![JudgmentResult {
            proposal_id: "p1".to_string(),
            approved: true,
            reason: "ok".to_string(),
        }];
        let (evolved, applied) = EvolutionEngine::apply(&soul, &proposals, &judgments);
        assert_eq!(applied.len(), 1);
        let goals = evolved.section("goals").unwrap();
        assert!(goals.content.contains("Learn Rust"));
    }

    #[test]
    fn test_early_phase_limits_proposals() {
        let soul = Soul::parse("# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship\n");
        let insights = vec![
            "User wants to learn Rust".to_string(),
            "User enjoys hiking".to_string(),
            "User values open source".to_string(),
        ];
        let proposals = EvolutionEngine::propose(&soul, &insights, RelationshipPhase::Awakening);
        assert!(proposals.len() <= 1);
    }
}
