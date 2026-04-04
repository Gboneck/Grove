use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Long-term memory — persistent patterns stored in ~/.grove/memory/patterns/.
/// These are behavioral patterns that have been observed repeatedly and
/// promoted from working memory.
///
/// Future: migrate to vector DB (Qdrant) for semantic search.

/// A long-term memory entry — a pattern that has been confirmed enough times
/// to be considered persistent knowledge about the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermEntry {
    pub id: String,
    pub category: LongTermCategory,
    pub content: String,
    pub confidence: f64,
    pub first_observed: String,
    pub last_confirmed: String,
    pub confirmation_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LongTermCategory {
    /// Behavioral pattern (e.g., "works on EMBER Monday mornings")
    Behavior,
    /// Preference (e.g., "prefers copy-paste outputs")
    Preference,
    /// Relationship info (e.g., "Zach is business partner for Grizzly Peak")
    Relationship,
    /// Skill/capability (e.g., "proficient in Rust and TypeScript")
    Skill,
    /// Strategic pattern (e.g., "tends to context-switch when blocked")
    Strategic,
}

fn longterm_dir() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".grove")
        .join("memory")
        .join("longterm")
}

fn entries_path() -> PathBuf {
    longterm_dir().join("entries.json")
}

/// Ensure the long-term memory directory exists.
pub fn ensure_longterm_dir() {
    let dir = longterm_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }
}

/// Read all long-term memory entries.
pub fn read_entries() -> Vec<LongTermEntry> {
    let path = entries_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Save long-term memory entries.
pub fn write_entries(entries: &[LongTermEntry]) -> Result<(), String> {
    ensure_longterm_dir();
    let content = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Failed to serialize long-term entries: {}", e))?;
    fs::write(entries_path(), content)
        .map_err(|e| format!("Failed to write long-term entries: {}", e))
}

/// Promote a pattern from working memory to long-term memory.
/// If an entry with similar content exists, confirm it instead.
pub fn promote(
    category: LongTermCategory,
    content: &str,
    confidence: f64,
) -> Result<(), String> {
    let mut entries = read_entries();
    let now = Utc::now().to_rfc3339();
    let lower = content.to_lowercase();

    // Check for existing similar entry
    if let Some(existing) = entries
        .iter_mut()
        .find(|e| e.content.to_lowercase() == lower && e.category == category)
    {
        existing.confirmation_count += 1;
        existing.last_confirmed = now;
        existing.confidence = (existing.confidence + 0.05).min(1.0);
    } else {
        entries.push(LongTermEntry {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            content: content.to_string(),
            confidence,
            first_observed: now.clone(),
            last_confirmed: now,
            confirmation_count: 1,
        });
    }

    // Cap at 100 long-term entries
    if entries.len() > 100 {
        entries.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(100);
    }

    write_entries(&entries)
}

/// Get entries formatted for reasoning context.
pub fn context_summary() -> String {
    let entries = read_entries();
    if entries.is_empty() {
        return String::new();
    }

    let mut parts = vec!["--- LONG-TERM PATTERNS ---".to_string()];
    for entry in entries.iter().filter(|e| e.confidence >= 0.5) {
        parts.push(format!(
            "- [{}] {} (confidence: {:.0}%, confirmed {}x)",
            category_label(&entry.category),
            entry.content,
            entry.confidence * 100.0,
            entry.confirmation_count,
        ));
    }

    if parts.len() == 1 {
        return String::new(); // Only the header, no qualifying entries
    }

    parts.join("\n")
}

fn category_label(cat: &LongTermCategory) -> &'static str {
    match cat {
        LongTermCategory::Behavior => "behavior",
        LongTermCategory::Preference => "preference",
        LongTermCategory::Relationship => "relationship",
        LongTermCategory::Skill => "skill",
        LongTermCategory::Strategic => "strategic",
    }
}

/// Decay entries that haven't been confirmed recently.
pub fn decay_entries(days_threshold: i64, decay_amount: f64) -> Result<(), String> {
    let mut entries = read_entries();
    let now = Utc::now();

    for entry in &mut entries {
        if let Ok(last) = chrono::DateTime::parse_from_rfc3339(&entry.last_confirmed) {
            let days = (now - last.with_timezone(&Utc)).num_days();
            if days > days_threshold {
                entry.confidence = (entry.confidence - decay_amount).max(0.0);
            }
        }
    }

    // Remove entries that have fully decayed
    entries.retain(|e| e.confidence > 0.05);

    write_entries(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn with_temp_home<F: FnOnce()>(f: F) {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().join(".grove").join("memory").join("longterm");
        fs::create_dir_all(&grove).unwrap();
        env::set_var("HOME", tmp.path());
        f();
    }

    #[test]
    fn test_promote_new_entry() {
        with_temp_home(|| {
            promote(
                LongTermCategory::Behavior,
                "Works on EMBER Monday mornings",
                0.7,
            )
            .unwrap();

            let entries = read_entries();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].confirmation_count, 1);
            assert_eq!(entries[0].confidence, 0.7);
        });
    }

    #[test]
    fn test_promote_existing_confirms() {
        with_temp_home(|| {
            promote(
                LongTermCategory::Preference,
                "Prefers copy-paste outputs",
                0.6,
            )
            .unwrap();
            promote(
                LongTermCategory::Preference,
                "Prefers copy-paste outputs",
                0.6,
            )
            .unwrap();

            let entries = read_entries();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].confirmation_count, 2);
            assert!(entries[0].confidence > 0.6); // boosted
        });
    }

    #[test]
    fn test_context_summary_empty() {
        with_temp_home(|| {
            assert!(context_summary().is_empty());
        });
    }

    #[test]
    fn test_context_summary_with_entries() {
        with_temp_home(|| {
            promote(
                LongTermCategory::Behavior,
                "Works in the morning",
                0.8,
            )
            .unwrap();

            let summary = context_summary();
            assert!(summary.contains("LONG-TERM PATTERNS"));
            assert!(summary.contains("Works in the morning"));
        });
    }
}
