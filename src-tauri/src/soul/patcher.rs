use super::parser::{Soul, SoulSection};

/// A proposed change to a Soul section.
#[derive(Debug, Clone)]
pub struct SoulPatch {
    /// Which section to update (case-insensitive match on heading).
    pub section: String,
    /// New content to append (or replace if `replace` is true).
    pub content: String,
    /// Confidence adjustment: positive to increase, negative to decrease.
    pub confidence_delta: f64,
    /// If true, replace the section content entirely. If false, append.
    pub replace: bool,
}

impl Soul {
    /// Apply a patch to the Soul, returning a new Soul with the changes.
    pub fn apply_patch(&self, patch: &SoulPatch) -> Soul {
        let mut new = self.clone();
        let query = patch.section.to_lowercase();

        if let Some(section) = new.sections.iter_mut().find(|s| s.heading.to_lowercase().contains(&query)) {
            if patch.replace {
                section.content = patch.content.clone();
            } else {
                if !section.content.is_empty() && !section.content.ends_with('\n') {
                    section.content.push('\n');
                }
                section.content.push_str(&patch.content);
            }
            section.confidence = (section.confidence + patch.confidence_delta).clamp(0.0, 1.0);

            // Re-extract items from updated content
            section.items = extract_items(&section.content);
        } else {
            // Section doesn't exist — create it
            let items = extract_items(&patch.content);
            new.sections.push(SoulSection {
                heading: patch.section.clone(),
                confidence: (0.5 + patch.confidence_delta).clamp(0.0, 1.0),
                content: patch.content.clone(),
                items,
            });
        }

        new
    }

    /// Decay all confidence scores by a factor (e.g., 0.99 for 1% decay per cycle).
    pub fn decay_confidence(&self, factor: f64) -> Soul {
        let mut new = self.clone();
        for section in &mut new.sections {
            section.confidence = (section.confidence * factor).clamp(0.0, 1.0);
        }
        new
    }

    /// Boost confidence for a section based on user confirmation.
    pub fn confirm_section(&self, section_query: &str, boost: f64) -> Soul {
        let mut new = self.clone();
        let query = section_query.to_lowercase();
        if let Some(section) = new.sections.iter_mut().find(|s| s.heading.to_lowercase().contains(&query)) {
            section.confidence = (section.confidence + boost).clamp(0.0, 1.0);
        }
        new
    }
}

fn extract_items(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                Some(trimmed[2..].to_string())
            } else if trimmed.len() > 2 && trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                trimmed.find(". ").map(|dot_pos| trimmed[dot_pos + 2..].to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soul::parser::Soul;

    #[test]
    fn test_append_patch() {
        let raw = "# Soul.md — Test\n\n## Goals [confidence: 0.6]\n- Ship Grove\n";
        let soul = Soul::parse(raw);
        let patched = soul.apply_patch(&SoulPatch {
            section: "goals".to_string(),
            content: "- Learn Rust".to_string(),
            confidence_delta: 0.1,
            replace: false,
        });
        let goals = patched.section("goals").unwrap();
        assert!(goals.content.contains("Learn Rust"));
        assert!((goals.confidence - 0.7).abs() < 0.01);
        assert_eq!(goals.items.len(), 2);
    }

    #[test]
    fn test_replace_patch() {
        let raw = "# Soul.md — Test\n\n## Goals [confidence: 0.6]\n- Old goal\n";
        let soul = Soul::parse(raw);
        let patched = soul.apply_patch(&SoulPatch {
            section: "goals".to_string(),
            content: "- New goal".to_string(),
            confidence_delta: 0.0,
            replace: true,
        });
        let goals = patched.section("goals").unwrap();
        assert!(!goals.content.contains("Old goal"));
        assert!(goals.content.contains("New goal"));
    }

    #[test]
    fn test_new_section_patch() {
        let raw = "# Soul.md — Test\n\n## Identity [confidence: 0.8]\nA tester.\n";
        let soul = Soul::parse(raw);
        let patched = soul.apply_patch(&SoulPatch {
            section: "Relationships".to_string(),
            content: "- Alice (business partner)".to_string(),
            confidence_delta: 0.1,
            replace: false,
        });
        assert_eq!(patched.sections.len(), 2);
        let rel = patched.section("relationships").unwrap();
        assert_eq!(rel.confidence, 0.6);
    }

    #[test]
    fn test_decay() {
        let raw = "# Soul.md — Test\n\n## A [confidence: 1.0]\nX\n\n## B [confidence: 0.5]\nY\n";
        let soul = Soul::parse(raw);
        let decayed = soul.decay_confidence(0.9);
        assert!((decayed.sections[0].confidence - 0.9).abs() < 0.01);
        assert!((decayed.sections[1].confidence - 0.45).abs() < 0.01);
    }
}
