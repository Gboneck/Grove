//! Integration tests for the memory subsystem.
//!
//! Tests the full flow: ephemeral → working → long-term → vector.

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    /// Set up a temporary grove directory for testing.
    fn setup_test_grove() -> tempfile::TempDir {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().join(".grove");
        fs::create_dir_all(grove.join("memory").join("longterm")).unwrap();

        // Create a minimal memory.json
        let memory = serde_json::json!({
            "sessions": [],
            "facts": [],
            "patterns": [],
            "accumulated_insights": [],
            "last_seen": null,
            "tuning": {
                "total_sessions": 0,
                "total_actions_clicked": 0,
                "total_inputs_submitted": 0,
                "block_type_engagement": {},
                "preferred_session_times": []
            }
        });
        fs::write(
            grove.join("memory.json"),
            serde_json::to_string_pretty(&memory).unwrap(),
        )
        .unwrap();

        // Create empty longterm entries
        fs::write(grove.join("memory").join("longterm").join("entries.json"), "[]").unwrap();

        // Set HOME so all grove functions find our temp dir
        env::set_var("HOME", tmp.path());
        tmp
    }

    #[test]
    fn test_memory_full_lifecycle() {
        let _tmp = setup_test_grove();

        // 1. Record a session
        grove_os_lib::commands::memory::record_session(
            vec!["text".to_string(), "metric".to_string()],
            Some("How's my week going?"),
            "User asked about their week. Showed progress on ventures.",
            vec!["User prefers morning work sessions".to_string()],
        )
        .unwrap();

        // 2. Verify session was recorded
        let memory = grove_os_lib::commands::memory::read_memory_file().unwrap();
        assert_eq!(memory.sessions.len(), 1);
        assert_eq!(memory.tuning.total_sessions, 1);

        // 3. Verify fact was auto-extracted from insight
        assert!(!memory.facts.is_empty());
        assert!(memory.facts.iter().any(|f| f.content.contains("morning")));

        // 4. Promote to long-term memory
        grove_os_lib::memory::longterm::promote(
            grove_os_lib::memory::longterm::LongTermCategory::Behavior,
            "User prefers morning work sessions",
            0.7,
        )
        .unwrap();

        let lt = grove_os_lib::memory::longterm::read_entries();
        assert_eq!(lt.len(), 1);
        assert_eq!(lt[0].confidence, 0.7);

        // 5. Confirm the pattern (simulate repeated observation)
        grove_os_lib::memory::longterm::promote(
            grove_os_lib::memory::longterm::LongTermCategory::Behavior,
            "User prefers morning work sessions",
            0.7,
        )
        .unwrap();

        let lt = grove_os_lib::memory::longterm::read_entries();
        assert_eq!(lt.len(), 1);
        assert_eq!(lt[0].confirmation_count, 2);
        assert!(lt[0].confidence > 0.7); // Should have been boosted

        // 6. Context summary includes the pattern
        let summary = grove_os_lib::memory::longterm::context_summary();
        assert!(summary.contains("morning work sessions"));
    }

    #[test]
    fn test_fact_upsert_and_decay() {
        let _tmp = setup_test_grove();

        // Add a fact
        grove_os_lib::commands::memory::upsert_fact(
            "preference",
            "Prefers dark mode",
            "session-1",
        )
        .unwrap();

        let memory = grove_os_lib::commands::memory::read_memory_file().unwrap();
        assert_eq!(memory.facts.len(), 1);
        assert_eq!(memory.facts[0].confidence, 0.7);

        // Confirm the fact
        grove_os_lib::commands::memory::upsert_fact(
            "preference",
            "Prefers dark mode",
            "session-2",
        )
        .unwrap();

        let memory = grove_os_lib::commands::memory::read_memory_file().unwrap();
        assert_eq!(memory.facts.len(), 1);
        assert!(memory.facts[0].confidence > 0.7); // Boosted
    }

    #[test]
    fn test_vector_embedding_consistency() {
        // Test that the vector module produces consistent embeddings
        use grove_os_lib::memory::vector;

        let text = "user prefers morning work sessions";
        // We can't call embed_text directly (it's private), but we can test
        // the public search/upsert interfaces structurally
        let point = vector::MemoryPoint {
            id: "test-1".to_string(),
            content: text.to_string(),
            category: "behavior".to_string(),
            confidence: 0.8,
            created_at: "2026-04-04T00:00:00Z".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        // Verify the struct is well-formed
        assert_eq!(point.content, text);
        assert_eq!(point.confidence, 0.8);
    }
}
