//! Soul enrichment — generates gap-filling questions during early phases.
//!
//! During Awakening and Discovery, the system actively identifies gaps
//! in Soul.md and generates targeted questions to fill them.

use super::parser::Soul;
use crate::soul::evolution::RelationshipPhase;

/// A question designed to fill a specific gap in Soul.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichmentPrompt {
    pub section: String,
    pub question: String,
    pub priority: f64,
    pub reason: String,
}

/// Expected sections in a complete Soul.md, with example prompts.
const SECTION_PROMPTS: &[(&str, &[&str])] = &[
    (
        "Identity",
        &[
            "What's your name and how would you describe yourself in a sentence?",
            "What do you do professionally?",
            "What's one thing people always get wrong about you?",
        ],
    ),
    (
        "Active Ventures",
        &[
            "What projects are you currently working on?",
            "Which project feels most urgent right now?",
            "Are any of your projects connected to each other?",
        ],
    ),
    (
        "Current State",
        &[
            "What's taking most of your energy right now?",
            "Are there any deadlines or time-sensitive things coming up?",
            "How would you describe your bandwidth — stretched thin, comfortable, or bored?",
        ],
    ),
    (
        "Work Style",
        &[
            "When do you do your best work — morning, afternoon, or night?",
            "Do you prefer long focus blocks or short bursts?",
            "How do you feel about context-switching between projects?",
        ],
    ),
    (
        "Priority Stack",
        &[
            "If you could only move one thing forward this week, what would it be?",
            "What's the thing you keep putting off but know matters?",
            "How do you decide what's most important?",
        ],
    ),
    (
        "Relationships",
        &[
            "Who are the key people in your work life right now?",
            "Is there anyone you collaborate with regularly?",
            "Who do you go to when you're stuck?",
        ],
    ),
    (
        "Patterns",
        &[
            "What habits help you get things done?",
            "What patterns do you notice in how you work?",
            "When do you tend to procrastinate, and on what?",
        ],
    ),
    (
        "Aspirations",
        &[
            "Where do you want to be in a year?",
            "What's a skill you want to develop?",
            "What would 'success' look like for your current projects?",
        ],
    ),
];

/// Generate enrichment prompts based on Soul.md gaps.
/// Returns questions sorted by priority (highest first).
pub fn generate_prompts(soul: &Soul, phase: RelationshipPhase) -> Vec<EnrichmentPrompt> {
    let max_prompts = match phase {
        RelationshipPhase::Awakening => 3,
        RelationshipPhase::Discovery => 2,
        RelationshipPhase::Deepening => 1,
        _ => return Vec::new(), // Later phases don't need active prompting
    };

    let mut prompts = Vec::new();

    for (section_name, questions) in SECTION_PROMPTS {
        let section = soul.section(&section_name.to_lowercase());

        let (is_missing, is_thin) = match section {
            None => (true, false),
            Some(s) => (false, s.content.len() < 30 && s.items.is_empty()),
        };

        if !is_missing && !is_thin {
            continue;
        }

        // Pick the first question for missing sections, varied for thin ones
        let question_idx = if is_missing {
            0
        } else {
            // Use section name hash to vary which question is asked
            let hash: usize = section_name.bytes().map(|b| b as usize).sum();
            hash % questions.len()
        };

        let priority = if is_missing { 1.0 } else { 0.6 };
        let reason = if is_missing {
            format!("'{}' section is missing from Soul.md", section_name)
        } else {
            format!("'{}' section has very little content", section_name)
        };

        prompts.push(EnrichmentPrompt {
            section: section_name.to_string(),
            question: questions[question_idx].to_string(),
            priority,
            reason,
        });
    }

    // Sort by priority descending
    prompts.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
    prompts.truncate(max_prompts);
    prompts
}

/// Format enrichment prompts as a context string for the reasoning model.
/// The model can weave these questions into its UI blocks naturally.
pub fn enrichment_context(soul: &Soul, phase: RelationshipPhase) -> String {
    let prompts = generate_prompts(soul, phase);
    if prompts.is_empty() {
        return String::new();
    }

    let mut parts = vec!["\n--- SOUL ENRICHMENT (ask the user naturally) ---".to_string()];
    for p in &prompts {
        parts.push(format!("- [{}] {}", p.section, p.question));
    }
    parts.push("Weave 1-2 of these into your response as natural questions or input prompts.".to_string());
    parts.join("\n")
}

/// Tauri command: get current enrichment prompts as blocks the frontend can render.
#[tauri::command]
pub async fn get_enrichment_prompts() -> Result<Vec<serde_json::Value>, String> {
    let grove_dir = dirs::home_dir().ok_or("No home directory")?.join(".grove");
    let soul_raw = std::fs::read_to_string(grove_dir.join("soul.md"))
        .map_err(|e| format!("Failed to read soul.md: {}", e))?;
    let soul = Soul::parse(&soul_raw);

    let mem = crate::commands::memory::read_memory_file().unwrap_or_default();
    let phase = RelationshipPhase::from_metrics(soul.completeness(), mem.sessions.len() as u32);

    let prompts = generate_prompts(&soul, phase);
    let blocks: Vec<serde_json::Value> = prompts
        .iter()
        .map(|p| {
            serde_json::json!({
                "type": "input",
                "prompt": p.question,
                "placeholder": format!("Tell Grove about your {}...", p.section.to_lowercase()),
                "section": p.section,
                "priority": p.priority,
            })
        })
        .collect();

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_soul_gets_prompts() {
        let soul = Soul::parse("# Soul.md\n");
        let prompts = generate_prompts(&soul, RelationshipPhase::Awakening);
        assert!(!prompts.is_empty());
        assert!(prompts.len() <= 3);
        // Should prioritize missing sections
        assert!(prompts[0].priority >= 0.9);
    }

    #[test]
    fn test_complete_soul_no_prompts() {
        let soul = Soul::parse(
            "# Soul.md\n\n\
             ## Identity [confidence: 0.9]\nI am a developer and entrepreneur.\n\n\
             ## Active Ventures [confidence: 0.8]\n- **Grove** — Building an OS.\n\n\
             ## Current State [confidence: 0.7]\n- Busy with shipping Grove v1.\n\n\
             ## Work Style [confidence: 0.7]\n- Morning person, deep focus blocks.\n\n\
             ## Priority Stack [confidence: 0.8]\n1. Ship Grove.\n\n\
             ## Relationships [confidence: 0.5]\n- Zach — business partner.\n\n\
             ## Patterns [confidence: 0.6]\n- Tends to overcommit on Monday.\n\n\
             ## Aspirations [confidence: 0.4]\n- Full-time on Grove within a year.\n",
        );
        let prompts = generate_prompts(&soul, RelationshipPhase::Awakening);
        assert!(prompts.is_empty());
    }

    #[test]
    fn test_later_phases_no_prompts() {
        let soul = Soul::parse("# Soul.md\n");
        let prompts = generate_prompts(&soul, RelationshipPhase::Challenge);
        assert!(prompts.is_empty());
    }

    #[test]
    fn test_enrichment_context_format() {
        let soul = Soul::parse("# Soul.md\n");
        let ctx = enrichment_context(&soul, RelationshipPhase::Discovery);
        assert!(ctx.contains("SOUL ENRICHMENT"));
    }
}
