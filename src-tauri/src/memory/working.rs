use chrono::Utc;
use std::fs;
use std::path::PathBuf;

/// Working memory — recent days, stored in MEMORY.md.
/// This is the bridge between ephemeral session state and long-term patterns.
///
/// MEMORY.md is a markdown journal that accumulates entries over time.
/// The reasoning model reads it at the start of every cycle for cross-session context.
/// Entries older than the retention period are archived to long-term memory.

fn memory_md_path() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".grove")
        .join("memory.md")
}

/// Ensure memory.md exists with a header.
pub fn ensure_memory_md() {
    let path = memory_md_path();
    if !path.exists() {
        let header = "# Memory Journal\n\nCross-session observations and events.\n";
        fs::write(&path, header).ok();
    }
}

/// Read the entire memory.md content.
pub fn read_memory_md() -> String {
    fs::read_to_string(memory_md_path()).unwrap_or_default()
}

/// Append a significant event to memory.md.
pub fn append_event(event_type: &str, detail: &str) -> Result<(), String> {
    let path = memory_md_path();
    let mut content = fs::read_to_string(&path).unwrap_or_default();

    if content.is_empty() {
        content = "# Memory Journal\n\nCross-session observations and events.\n".to_string();
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC");
    let entry = format!("\n### {} — {}\n{}\n", event_type, now, detail);
    content.push_str(&entry);

    fs::write(&path, content).map_err(|e| format!("Failed to write memory.md: {}", e))
}

/// Append a session summary to memory.md.
pub fn record_session_summary(
    user_input: Option<&str>,
    summary: &str,
    model_source: &str,
    insights: &[String],
) -> Result<(), String> {
    let mut detail = String::new();

    if let Some(input) = user_input {
        detail.push_str(&format!("**User**: {}\n", input));
    }
    detail.push_str(&format!("**Summary**: {}\n", summary));
    detail.push_str(&format!("**Model**: {}\n", model_source));

    if !insights.is_empty() {
        detail.push_str("**Insights**:\n");
        for insight in insights {
            detail.push_str(&format!("- {}\n", insight));
        }
    }

    append_event("Session", &detail)
}

/// Append a fact discovery to memory.md.
pub fn record_fact_discovery(category: &str, fact: &str) -> Result<(), String> {
    let detail = format!("Learned [{}]: {}", category, fact);
    append_event("Fact", &detail)
}

/// Append a pattern detection to memory.md.
pub fn record_pattern_detection(pattern: &str, confidence: f64) -> Result<(), String> {
    let detail = format!("Detected pattern (confidence {:.0}%): {}", confidence * 100.0, pattern);
    append_event("Pattern", &detail)
}

/// Append a venture update to memory.md.
pub fn record_venture_update(venture: &str, update: &str) -> Result<(), String> {
    let detail = format!("**{}**: {}", venture, update);
    append_event("Venture Update", &detail)
}

/// Get recent entries from memory.md (last N lines worth of content).
/// Returns a substring focused on the most recent entries.
pub fn recent_entries(max_chars: usize) -> String {
    let content = read_memory_md();
    if content.len() <= max_chars {
        return content;
    }

    // Find the last `max_chars` characters, but start at a section boundary
    let start = content.len() - max_chars;
    let adjusted_start = content[start..]
        .find("\n### ")
        .map(|pos| start + pos)
        .unwrap_or(start);

    format!(
        "...(earlier entries truncated)...\n{}",
        &content[adjusted_start..]
    )
}

/// Prune entries older than `days` from memory.md.
/// Moves pruned content to an archive file.
pub fn prune_old_entries(days: i64) -> Result<usize, String> {
    let content = read_memory_md();
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let lines: Vec<&str> = content.lines().collect();
    let mut keep_lines: Vec<&str> = Vec::new();
    let mut archive_lines: Vec<&str> = Vec::new();
    let mut in_old_section = false;
    let mut pruned_count = 0;

    for line in &lines {
        if line.starts_with("### ") {
            // Extract date from section header like "### Session — 2026-03-15 14:30 UTC"
            let is_old = line
                .split('—')
                .nth(1)
                .and_then(|date_part| {
                    let date_str = date_part.trim().get(..10)?;
                    Some(date_str < cutoff_str.as_str())
                })
                .unwrap_or(false);

            in_old_section = is_old;
            if is_old {
                pruned_count += 1;
            }
        }

        if in_old_section {
            archive_lines.push(line);
        } else {
            keep_lines.push(line);
        }
    }

    if pruned_count > 0 {
        // Write archive
        let archive_path = dirs::home_dir()
            .ok_or("No home dir")?
            .join(".grove")
            .join("memory")
            .join(format!("archive-{}.md", cutoff.format("%Y-%m-%d")));
        fs::create_dir_all(archive_path.parent().unwrap())
            .map_err(|e| format!("Failed to create archive dir: {}", e))?;
        fs::write(&archive_path, archive_lines.join("\n"))
            .map_err(|e| format!("Failed to write archive: {}", e))?;

        // Write trimmed memory.md
        let path = memory_md_path();
        fs::write(&path, keep_lines.join("\n"))
            .map_err(|e| format!("Failed to write pruned memory.md: {}", e))?;
    }

    Ok(pruned_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Tests use a temporary HOME to avoid touching real ~/.grove/
    fn with_temp_home<F: FnOnce()>(f: F) {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().join(".grove");
        fs::create_dir_all(&grove).unwrap();
        env::set_var("HOME", tmp.path());
        f();
    }

    #[test]
    fn test_ensure_creates_file() {
        with_temp_home(|| {
            ensure_memory_md();
            let content = read_memory_md();
            assert!(content.contains("Memory Journal"));
        });
    }

    #[test]
    fn test_append_event() {
        with_temp_home(|| {
            ensure_memory_md();
            append_event("Test", "Something happened").unwrap();
            let content = read_memory_md();
            assert!(content.contains("### Test"));
            assert!(content.contains("Something happened"));
        });
    }

    #[test]
    fn test_record_session_summary() {
        with_temp_home(|| {
            ensure_memory_md();
            record_session_summary(
                Some("what's my priority?"),
                "Showed EMBER status and priority stack",
                "cloud",
                &["User focuses on revenue".to_string()],
            )
            .unwrap();

            let content = read_memory_md();
            assert!(content.contains("### Session"));
            assert!(content.contains("what's my priority?"));
            assert!(content.contains("User focuses on revenue"));
        });
    }

    #[test]
    fn test_recent_entries_truncation() {
        with_temp_home(|| {
            ensure_memory_md();
            for i in 0..20 {
                append_event("Entry", &format!("Event number {}", i)).unwrap();
            }
            let recent = recent_entries(500);
            // Should be truncated
            assert!(recent.len() <= 600); // some margin
        });
    }
}
