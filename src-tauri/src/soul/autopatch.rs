use super::parser::Soul;
use super::patcher::SoulPatch;

/// Categories of auto-discoverable soul facts from model insights.
const SECTION_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "Goals",
        &["goal", "wants to", "aims to", "working toward", "aspires"],
    ),
    (
        "Values",
        &["values", "believes in", "cares about", "important to"],
    ),
    (
        "Skills",
        &["skilled at", "good at", "experienced with", "proficient"],
    ),
    (
        "Patterns",
        &["tends to", "usually", "pattern", "habit", "often"],
    ),
    (
        "Preferences",
        &["prefers", "likes", "enjoys", "favorite", "dislikes"],
    ),
    (
        "Relationships",
        &["partner", "friend", "family", "colleague", "team"],
    ),
    (
        "Identity",
        &["identifies as", "is a", "considers themselves"],
    ),
];

/// Attempt to extract soul patches from model-generated insights.
/// Returns patches that can be applied to the soul document.
pub fn extract_patches(insights: &[String], soul: &Soul) -> Vec<SoulPatch> {
    let mut patches = Vec::new();

    for insight in insights {
        let lower = insight.to_lowercase();

        // Match insight to a soul section based on keywords
        let target_section = SECTION_KEYWORDS
            .iter()
            .find(|(_, keywords)| keywords.iter().any(|kw| lower.contains(kw)))
            .map(|(section, _)| *section);

        if let Some(section) = target_section {
            // Check if this fact is already in the soul
            let already_exists = soul
                .section(section)
                .map(|s| {
                    let existing_lower = s.content.to_lowercase();
                    // Fuzzy match: check if the core of the insight is already present
                    let words: Vec<&str> = lower.split_whitespace().collect();
                    let significant_words: Vec<&&str> =
                        words.iter().filter(|w| w.len() > 4).collect();
                    // If >60% of significant words are already in the section, skip
                    if significant_words.is_empty() {
                        return false;
                    }
                    let matches = significant_words
                        .iter()
                        .filter(|w| existing_lower.contains(***w))
                        .count();
                    matches as f64 / significant_words.len() as f64 > 0.6
                })
                .unwrap_or(false);

            if !already_exists {
                patches.push(SoulPatch {
                    section: section.to_string(),
                    content: format!("- {}", insight),
                    confidence_delta: 0.05, // Small bump — model-inferred, not user-confirmed
                    replace: false,
                });
            }
        }
    }

    patches
}

/// Apply auto-patches to soul.md and write back to disk.
/// Returns the number of patches applied.
pub fn auto_patch_soul(insights: &[String]) -> Result<usize, String> {
    let grove_dir = dirs::home_dir().ok_or("No home directory")?.join(".grove");
    let soul_path = grove_dir.join("soul.md");

    let soul_raw = std::fs::read_to_string(&soul_path)
        .map_err(|e| format!("Failed to read soul.md: {}", e))?;
    let soul = Soul::parse(&soul_raw);

    let patches = extract_patches(insights, &soul);
    if patches.is_empty() {
        return Ok(0);
    }

    let mut patched = soul;
    let count = patches.len();
    for patch in &patches {
        patched = patched.apply_patch(patch);
        eprintln!(
            "[grove:autopatch] Patching '{}': {}",
            patch.section,
            patch.content.trim()
        );
    }

    // Write back
    let markdown = patched.to_markdown();
    std::fs::write(&soul_path, markdown).map_err(|e| format!("Failed to write soul.md: {}", e))?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_goal_patch() {
        let soul = Soul::parse("# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship Grove\n");
        let insights = vec!["User wants to learn Rust deeply".to_string()];
        let patches = extract_patches(&insights, &soul);
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].section, "Goals");
        assert!(patches[0].content.contains("learn Rust"));
    }

    #[test]
    fn test_skip_duplicate() {
        let soul = Soul::parse("# Soul.md\n\n## Goals [confidence: 0.6]\n- Ship Grove OS\n");
        let insights = vec!["User wants to ship Grove OS".to_string()];
        let patches = extract_patches(&insights, &soul);
        assert!(patches.is_empty(), "Should skip duplicate insight");
    }

    #[test]
    fn test_extract_pattern_patch() {
        let soul = Soul::parse("# Soul.md\n\n## Identity [confidence: 0.8]\nA builder.\n");
        let insights = vec!["User tends to work late at night".to_string()];
        let patches = extract_patches(&insights, &soul);
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].section, "Patterns");
    }

    #[test]
    fn test_no_patches_for_irrelevant() {
        let soul = Soul::parse("# Soul.md\n\n## Identity [confidence: 0.8]\nA builder.\n");
        let insights = vec!["The weather is nice today".to_string()];
        let patches = extract_patches(&insights, &soul);
        assert!(patches.is_empty());
    }
}
