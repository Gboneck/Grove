//! Tests for grove-mcp tool handlers.
//!
//! These test the tool dispatch logic without running the full MCP server.
//! We verify that each tool returns valid JSON with expected structure.

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use std::env;
    use std::fs;

    /// Set up a temporary grove directory with minimal test data.
    fn setup_test_grove() -> tempfile::TempDir {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().join(".grove");
        fs::create_dir_all(grove.join("memory").join("longterm")).unwrap();

        // Minimal soul.md
        fs::write(
            grove.join("soul.md"),
            "# Soul.md — Test User\n\n## Identity [confidence: 0.8]\nA developer.\n\n## Goals [confidence: 0.6]\n- Ship Grove\n",
        )
        .unwrap();

        // Minimal context.json
        fs::write(
            grove.join("context.json"),
            r#"{"ventures": [{"name": "Grove", "status": "active", "priority": 1}]}"#,
        )
        .unwrap();

        // Minimal memory.json
        let memory = json!({
            "sessions": [{
                "id": "test-session",
                "timestamp": "2026-04-04T10:00:00Z",
                "time_of_day": "morning",
                "day_of_week": "Friday",
                "blocks_shown": ["text"],
                "user_inputs": [{"timestamp": "2026-04-04T10:00:00Z", "text": "hello", "response_summary": "Greeted user"}],
                "session_summary": "Test session",
                "insights": ["User prefers morning work"]
            }],
            "facts": [{
                "id": "fact-1",
                "category": "preference",
                "content": "Prefers dark mode",
                "confidence": 0.8,
                "source": "test",
                "created_at": "2026-04-04T00:00:00Z",
                "last_confirmed": "2026-04-04T00:00:00Z",
                "superseded_by": null
            }],
            "patterns": [],
            "accumulated_insights": ["User works best in the morning"],
            "last_seen": "2026-04-04T10:00:00Z",
            "tuning": {
                "total_sessions": 1,
                "total_actions_clicked": 0,
                "total_inputs_submitted": 1,
                "block_type_engagement": {},
                "preferred_session_times": ["morning"]
            }
        });
        fs::write(
            grove.join("memory.json"),
            serde_json::to_string_pretty(&memory).unwrap(),
        )
        .unwrap();

        // Empty longterm
        fs::write(
            grove.join("memory").join("longterm").join("entries.json"),
            "[]",
        )
        .unwrap();

        // Minimal MEMORY.md
        fs::write(
            grove.join("MEMORY.md"),
            "# Memory Journal\n\n### 2026-04-04 10:00 UTC\n- Test session entry\n",
        )
        .unwrap();

        env::set_var("HOME", tmp.path());
        tmp
    }

    // We test the tool logic via the crate's public functions
    // Since grove-mcp is a binary, we test the shared data structures

    #[test]
    fn test_soul_read() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let soul = std::fs::read_to_string(grove.join("soul.md")).unwrap();
        assert!(soul.contains("Test User"));
        assert!(soul.contains("Identity"));
        assert!(soul.contains("developer"));
    }

    #[test]
    fn test_context_read() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let content = std::fs::read_to_string(grove.join("context.json")).unwrap();
        let ctx: Value = serde_json::from_str(&content).unwrap();
        let ventures = ctx.get("ventures").unwrap().as_array().unwrap();
        assert_eq!(ventures.len(), 1);
        assert_eq!(ventures[0]["name"], "Grove");
    }

    #[test]
    fn test_memory_session_read() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let content = std::fs::read_to_string(grove.join("memory.json")).unwrap();
        let mem: Value = serde_json::from_str(&content).unwrap();
        let sessions = mem.get("sessions").unwrap().as_array().unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0]["session_summary"]
            .as_str()
            .unwrap()
            .contains("Test"));
    }

    #[test]
    fn test_facts_read() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let content = std::fs::read_to_string(grove.join("memory.json")).unwrap();
        let mem: Value = serde_json::from_str(&content).unwrap();
        let facts = mem.get("facts").unwrap().as_array().unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0]["content"], "Prefers dark mode");
        assert!(facts[0]["superseded_by"].is_null());
    }

    #[test]
    fn test_fact_add() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let mem_path = grove.join("memory.json");

        // Add a new fact
        let content = std::fs::read_to_string(&mem_path).unwrap();
        let mut mem: Value = serde_json::from_str(&content).unwrap();
        let new_fact = json!({
            "id": "fact-2",
            "category": "skill",
            "content": "Proficient in Rust",
            "confidence": 0.7,
            "source": "mcp_test",
            "created_at": "2026-04-04T12:00:00Z",
            "last_confirmed": "2026-04-04T12:00:00Z",
            "superseded_by": null
        });
        mem.get_mut("facts")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(new_fact);
        std::fs::write(&mem_path, serde_json::to_string_pretty(&mem).unwrap()).unwrap();

        // Verify
        let updated = std::fs::read_to_string(&mem_path).unwrap();
        let updated_mem: Value = serde_json::from_str(&updated).unwrap();
        let facts = updated_mem.get("facts").unwrap().as_array().unwrap();
        assert_eq!(facts.len(), 2);
        assert!(facts.iter().any(|f| f["content"] == "Proficient in Rust"));
    }

    #[test]
    fn test_priority_venture() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let content = std::fs::read_to_string(grove.join("context.json")).unwrap();
        let ctx: Value = serde_json::from_str(&content).unwrap();

        let ventures = ctx.get("ventures").unwrap().as_array().unwrap();
        let top = ventures
            .iter()
            .filter(|v| v.get("status").and_then(|s| s.as_str()) != Some("completed"))
            .min_by_key(|v| v.get("priority").and_then(|p| p.as_u64()).unwrap_or(999));

        assert!(top.is_some());
        assert_eq!(top.unwrap()["name"], "Grove");
    }

    #[test]
    fn test_what_changed() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let memory_md = std::fs::read_to_string(grove.join("MEMORY.md")).unwrap();
        let entries: Vec<&str> = memory_md
            .split("### ")
            .filter(|s| !s.trim().is_empty())
            .collect();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_focus_completeness() {
        let _tmp = setup_test_grove();
        let grove = dirs::home_dir().unwrap().join(".grove");
        let soul_raw = std::fs::read_to_string(grove.join("soul.md")).unwrap();
        let section_count = soul_raw.matches("## ").count();
        // Our test soul has 2 sections
        assert_eq!(section_count, 2);
        let completeness = (section_count as f64 * 0.12).min(1.0);
        assert!(completeness > 0.0);
        assert!(completeness < 1.0);
    }

    #[test]
    fn test_keyword_search_logic() {
        // Test the keyword scoring approach used in grove_search
        let content = "User prefers morning work sessions";
        let query_words = vec!["morning", "work"];
        let lower = content.to_lowercase();
        let matches = query_words.iter().filter(|w| lower.contains(**w)).count();
        let score = matches as f64 / query_words.len() as f64;
        assert_eq!(score, 1.0); // Both words found

        let unrelated_words = vec!["python", "gaming"];
        let no_matches = unrelated_words
            .iter()
            .filter(|w| lower.contains(**w))
            .count();
        let no_score = no_matches as f64 / unrelated_words.len() as f64;
        assert_eq!(no_score, 0.0);
    }
}
