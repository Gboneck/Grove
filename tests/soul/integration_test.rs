//! Integration tests for the Soul subsystem.
//!
//! Tests parsing, enrichment, evolution, and the full propose→judge→apply cycle.

#[cfg(test)]
mod tests {
    use grove_os_lib::soul::parser::Soul;
    use grove_os_lib::soul::evolution::RelationshipPhase;
    use grove_os_lib::soul::enrichment;
    use grove_os_lib::soul::evolve::{EvolutionEngine, EvolutionProposal, EvolutionSource};

    #[test]
    fn test_soul_parse_roundtrip() {
        let input = "# Soul.md — Test\n\n\
                     ## Identity [confidence: 0.9]\n\
                     I am a developer.\n\n\
                     ## Goals [confidence: 0.6]\n\
                     - Ship Grove\n\
                     - Learn Rust\n";

        let soul = Soul::parse(input);
        let output = soul.to_markdown();

        // Should preserve key content
        assert!(output.contains("Identity"));
        assert!(output.contains("developer"));
        assert!(output.contains("Goals"));
        assert!(output.contains("Ship Grove"));
    }

    #[test]
    fn test_enrichment_generates_questions_for_incomplete_soul() {
        let soul = Soul::parse("# Soul.md\n\n## Identity [confidence: 0.5]\nJohn.\n");

        let prompts = enrichment::generate_prompts(&soul, RelationshipPhase::Awakening);
        assert!(!prompts.is_empty(), "Should generate prompts for incomplete soul");
        assert!(prompts.len() <= 3, "Awakening phase limits to 3 prompts");

        // Should ask about missing sections
        let sections: Vec<&str> = prompts.iter().map(|p| p.section.as_str()).collect();
        // Identity exists but is thin, other sections are missing
        assert!(
            sections.iter().any(|s| *s != "Identity"),
            "Should ask about sections other than Identity"
        );
    }

    #[test]
    fn test_enrichment_respects_phase_limits() {
        let soul = Soul::parse("# Soul.md\n");

        // Awakening: max 3
        let awk = enrichment::generate_prompts(&soul, RelationshipPhase::Awakening);
        assert!(awk.len() <= 3);

        // Discovery: max 2
        let disc = enrichment::generate_prompts(&soul, RelationshipPhase::Discovery);
        assert!(disc.len() <= 2);

        // Challenge: no prompts
        let chal = enrichment::generate_prompts(&soul, RelationshipPhase::Challenge);
        assert!(chal.is_empty());
    }

    #[test]
    fn test_enrichment_context_format() {
        let soul = Soul::parse("# Soul.md\n");
        let ctx = enrichment::enrichment_context(&soul, RelationshipPhase::Discovery);
        assert!(ctx.contains("SOUL ENRICHMENT"));
        assert!(ctx.contains("ask the user naturally"));
    }

    #[test]
    fn test_evolution_propose_judge_apply() {
        let soul = Soul::parse(
            "# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship Grove\n",
        );

        let insights = vec!["User wants to learn Rust deeply".to_string()];
        let phase = RelationshipPhase::Deepening;

        // Propose
        let proposals = EvolutionEngine::propose(&soul, &insights, phase);
        assert!(!proposals.is_empty());

        // Judge
        let judgments = EvolutionEngine::judge(&proposals, phase);
        assert_eq!(judgments.len(), proposals.len());

        // At least some should be approved in Deepening phase
        let approved_count = judgments.iter().filter(|j| j.approved).count();
        assert!(approved_count > 0, "Deepening phase should approve some proposals");

        // Apply
        let (evolved, applied) = EvolutionEngine::apply(&soul, &proposals, &judgments);
        assert!(!applied.is_empty());

        // Verify the soul was modified
        let goals = evolved.section("goals");
        assert!(goals.is_some());
    }

    #[test]
    fn test_evolution_safety_in_awakening() {
        let proposal = EvolutionProposal {
            id: "test-replace".to_string(),
            section: "Identity".to_string(),
            content: "Completely new identity".to_string(),
            confidence_delta: 0.0,
            replace: true,
            reason: "test".to_string(),
            source: EvolutionSource::ModelInsight,
        };

        let judgments = EvolutionEngine::judge(&[proposal], RelationshipPhase::Awakening);
        assert!(!judgments[0].approved, "Replace should be blocked in Awakening");
    }

    #[test]
    fn test_relationship_phase_progression() {
        // Verify phases progress with metrics
        let p1 = RelationshipPhase::from_metrics(0.1, 1);
        let p2 = RelationshipPhase::from_metrics(0.5, 15);
        let p3 = RelationshipPhase::from_metrics(0.7, 50);

        assert_eq!(p1, RelationshipPhase::Awakening);
        assert_eq!(p2, RelationshipPhase::Deepening);

        // Autonomy should increase
        assert!(p2.autonomy_level() > p1.autonomy_level());
        assert!(p3.autonomy_level() > p2.autonomy_level());
    }

    #[test]
    fn test_soul_completeness_calculation() {
        let empty = Soul::parse("# Soul.md\n");
        let partial = Soul::parse(
            "# Soul.md\n\n\
             ## Identity [confidence: 0.9]\nDeveloper.\n\n\
             ## Goals [confidence: 0.7]\n- Ship it.\n",
        );
        let full = Soul::parse(
            "# Soul.md\n\n\
             ## Identity [confidence: 0.9]\nI am a developer.\n\n\
             ## Active Ventures [confidence: 0.8]\n- Grove.\n\n\
             ## Current State [confidence: 0.7]\n- Busy.\n\n\
             ## Work Style [confidence: 0.7]\n- Morning person.\n\n\
             ## Priority Stack [confidence: 0.8]\n1. Ship.\n\n\
             ## Relationships [confidence: 0.5]\n- Zach.\n\n\
             ## Patterns [confidence: 0.6]\n- Overcommits.\n\n\
             ## Aspirations [confidence: 0.4]\n- Full-time.\n",
        );

        assert!(partial.completeness() > empty.completeness());
        assert!(full.completeness() > partial.completeness());
    }
}
