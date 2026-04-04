use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A parsed Soul.md document with structured sections and confidence scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soul {
    pub name: Option<String>,
    pub sections: Vec<SoulSection>,
}

/// A single section of the Soul document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulSection {
    pub heading: String,
    pub confidence: f64,
    pub content: String,
    pub items: Vec<String>,
}

impl Soul {
    /// Parse a Soul.md string into structured sections.
    pub fn parse(raw: &str) -> Self {
        let lines: Vec<&str> = raw.lines().collect();
        let mut name: Option<String> = None;
        let mut sections: Vec<SoulSection> = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_confidence: f64 = 0.7; // default
        let mut current_lines: Vec<String> = Vec::new();

        for line in &lines {
            let trimmed = line.trim();

            // H1: extract name (e.g., "# Soul.md — Grif")
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                if let Some(dash_pos) = trimmed.find('—').or_else(|| trimmed.find('-')) {
                    let after = trimmed[dash_pos + '—'.len_utf8()..].trim();
                    if !after.is_empty() {
                        name = Some(after.to_string());
                    }
                }
                continue;
            }

            // H2: new section
            if trimmed.starts_with("## ") {
                // Flush previous section
                if let Some(heading) = current_heading.take() {
                    sections.push(build_section(heading, current_confidence, &current_lines));
                    current_lines.clear();
                }

                // Parse heading and optional confidence tag
                let heading_text = &trimmed[3..];
                let (heading, confidence) = parse_heading_confidence(heading_text);
                current_heading = Some(heading);
                current_confidence = confidence;
                continue;
            }

            // Content line
            if current_heading.is_some() {
                current_lines.push(line.to_string());
            }
        }

        // Flush last section
        if let Some(heading) = current_heading {
            sections.push(build_section(heading, current_confidence, &current_lines));
        }

        Soul { name, sections }
    }

    /// Get a section by heading (case-insensitive partial match).
    pub fn section(&self, query: &str) -> Option<&SoulSection> {
        let q = query.to_lowercase();
        self.sections.iter().find(|s| s.heading.to_lowercase().contains(&q))
    }

    /// Get all sections as a map of heading → content for easy context injection.
    pub fn as_context_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for section in &self.sections {
            let key = section.heading.to_lowercase().replace(' ', "_");
            map.insert(key, section.content.clone());
        }
        map
    }

    /// Render the Soul back to markdown, preserving structure and confidence tags.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();

        // H1
        if let Some(ref name) = self.name {
            out.push_str(&format!("# Soul.md — {}\n\n", name));
        } else {
            out.push_str("# Soul.md\n\n");
        }

        for section in &self.sections {
            // H2 with confidence
            out.push_str(&format!("## {} [confidence: {:.1}]\n", section.heading, section.confidence));
            out.push_str(&section.content);
            if !section.content.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }

        out
    }

    /// Calculate overall soul completeness (0.0-1.0).
    pub fn completeness(&self) -> f64 {
        if self.sections.is_empty() {
            return 0.0;
        }
        let total: f64 = self.sections.iter().map(|s| s.confidence).sum();
        total / self.sections.len() as f64
    }

    /// Get sections with confidence below a threshold (candidates for enrichment).
    pub fn weak_sections(&self, threshold: f64) -> Vec<&SoulSection> {
        self.sections.iter().filter(|s| s.confidence < threshold).collect()
    }
}

/// Parse confidence tag from heading like "Identity [confidence: 0.9]"
fn parse_heading_confidence(text: &str) -> (String, f64) {
    if let Some(bracket_start) = text.find("[confidence:") {
        let heading = text[..bracket_start].trim().to_string();
        let rest = &text[bracket_start + 12..];
        if let Some(bracket_end) = rest.find(']') {
            let conf_str = rest[..bracket_end].trim();
            if let Ok(conf) = conf_str.parse::<f64>() {
                return (heading, conf.clamp(0.0, 1.0));
            }
        }
        (heading, 0.7)
    } else {
        (text.trim().to_string(), 0.7)
    }
}

/// Build a SoulSection from collected lines.
fn build_section(heading: String, confidence: f64, lines: &[String]) -> SoulSection {
    let content = lines.join("\n").trim().to_string();

    // Extract bullet items
    let items: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                Some(trimmed[2..].to_string())
            } else if trimmed.len() > 2 && trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                // Numbered list: "1. Item"
                if let Some(dot_pos) = trimmed.find(". ") {
                    Some(trimmed[dot_pos + 2..].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    SoulSection {
        heading,
        confidence,
        content,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_soul() {
        let raw = r#"# Soul.md — Grif

## Identity [confidence: 0.9]
Self-taught builder-operator. Multi-venture entrepreneur based in Berkeley, CA.

## Active Ventures [confidence: 0.85]
- **Grizzly Peak Landscape** — Fire safety / EMBER compliance.
- **Signal Blueprint** — AI consulting practice.
- **Daemon** — Privacy-first identity platform.

## Priority Stack [confidence: 0.9]
1. Revenue — EMBER door-to-door outreach
2. Anthropic application
3. Daemon — ongoing portfolio build
"#;
        let soul = Soul::parse(raw);
        assert_eq!(soul.name, Some("Grif".to_string()));
        assert_eq!(soul.sections.len(), 3);

        let identity = soul.section("identity").unwrap();
        assert_eq!(identity.confidence, 0.9);
        assert!(identity.content.contains("Self-taught"));

        let ventures = soul.section("ventures").unwrap();
        assert_eq!(ventures.confidence, 0.85);
        assert_eq!(ventures.items.len(), 3);

        let priorities = soul.section("priority").unwrap();
        assert_eq!(priorities.confidence, 0.9);
        assert_eq!(priorities.items.len(), 3);
    }

    #[test]
    fn test_parse_no_confidence_tags() {
        let raw = r#"# Soul.md — Test

## Identity
A test user.

## Goals
- Goal one
- Goal two
"#;
        let soul = Soul::parse(raw);
        assert_eq!(soul.name, Some("Test".to_string()));
        let identity = soul.section("identity").unwrap();
        assert_eq!(identity.confidence, 0.7); // default
    }

    #[test]
    fn test_roundtrip() {
        let raw = r#"# Soul.md — Grif

## Identity [confidence: 0.9]
Builder-operator.

## Goals [confidence: 0.6]
- Ship Grove OS
"#;
        let soul = Soul::parse(raw);
        let rendered = soul.to_markdown();
        let reparsed = Soul::parse(&rendered);
        assert_eq!(reparsed.name, soul.name);
        assert_eq!(reparsed.sections.len(), soul.sections.len());
        assert_eq!(reparsed.sections[0].confidence, 0.9);
    }

    #[test]
    fn test_completeness() {
        let raw = r#"# Soul.md — Test

## A [confidence: 1.0]
Content.

## B [confidence: 0.5]
Content.
"#;
        let soul = Soul::parse(raw);
        assert!((soul.completeness() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_weak_sections() {
        let raw = r#"# Soul.md — Test

## Strong [confidence: 0.9]
Content.

## Weak [confidence: 0.3]
Content.

## Medium [confidence: 0.6]
Content.
"#;
        let soul = Soul::parse(raw);
        let weak = soul.weak_sections(0.5);
        assert_eq!(weak.len(), 1);
        assert_eq!(weak[0].heading, "Weak");
    }
}
