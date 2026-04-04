//! Integration tests for the heartbeat subsystem.
//!
//! Tests the observer → scheduler → pattern detector pipeline.

#[cfg(test)]
mod tests {
    use grove_os_lib::heartbeat::observer::{Observation, ObservationKind, Observer, TimeOfDay};
    use grove_os_lib::heartbeat::patterns::{PatternDetector, PatternType};
    use grove_os_lib::heartbeat::scheduler::HeartbeatScheduler;

    // --- Observer tests ---

    #[test]
    fn test_time_of_day_full_range() {
        assert_eq!(TimeOfDay::from_hour(0), TimeOfDay::LateNight);
        assert_eq!(TimeOfDay::from_hour(5), TimeOfDay::LateNight);
        assert_eq!(TimeOfDay::from_hour(6), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(11), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(12), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(16), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(17), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(21), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
    }

    #[test]
    fn test_observer_deadline_with_upcoming() {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().to_path_buf();

        // Create context with a deadline 3 days from now
        let deadline = (chrono::Local::now() + chrono::Duration::days(3))
            .format("%Y-%m-%d")
            .to_string();
        let ctx = format!(
            r#"{{"ventures": [{{"name": "Test Project", "status": "active", "deadline": "{}"}}]}}"#,
            deadline
        );
        std::fs::write(grove.join("context.json"), ctx).unwrap();

        let observer = Observer::new(grove);
        let deadlines = observer.check_deadlines();
        assert_eq!(deadlines.len(), 1);
        assert!(deadlines[0].detail.contains("Test Project"));
        assert!(deadlines[0].detail.contains("3 days"));
    }

    #[test]
    fn test_observer_deadline_past_ignored() {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().to_path_buf();

        let ctx = r#"{"ventures": [{"name": "Old", "status": "active", "deadline": "2020-01-01"}]}"#;
        std::fs::write(grove.join("context.json"), ctx).unwrap();

        let observer = Observer::new(grove);
        let deadlines = observer.check_deadlines();
        assert!(deadlines.is_empty());
    }

    #[test]
    fn test_observer_deadline_far_future_ignored() {
        let tmp = tempfile::TempDir::new().unwrap();
        let grove = tmp.path().to_path_buf();

        let deadline = (chrono::Local::now() + chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        let ctx = format!(
            r#"{{"ventures": [{{"name": "Future", "status": "active", "deadline": "{}"}}]}}"#,
            deadline
        );
        std::fs::write(grove.join("context.json"), ctx).unwrap();

        let observer = Observer::new(grove);
        let deadlines = observer.check_deadlines();
        assert!(deadlines.is_empty()); // >7 days away
    }

    // --- Scheduler tests ---

    #[test]
    fn test_scheduler_mixed_observations() {
        let mut sched = HeartbeatScheduler::new(300, 5);

        // Push file changes and a time shift
        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "soul.md"),
            make_obs(ObservationKind::TimeShift, "morning to afternoon"),
        ]);
        assert_eq!(sched.queue_size(), 2);
        assert!(!sched.should_trigger()); // Below threshold

        // Push more to hit threshold
        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "context.json"),
            make_obs(ObservationKind::FileChanged, "memory.json"),
            make_obs(ObservationKind::Idle, "30 minutes idle"),
        ]);
        assert_eq!(sched.queue_size(), 5);
        assert!(sched.should_trigger()); // At threshold
    }

    #[test]
    fn test_scheduler_summary_format() {
        let mut sched = HeartbeatScheduler::new(300, 10);
        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "soul.md changed"),
            make_obs(ObservationKind::DeadlineApproaching, "Grove: 2 days"),
        ]);
        let summary = sched.queue_summary();
        assert!(summary.contains("2 observation(s)"));
        assert!(summary.contains("[file]"));
        assert!(summary.contains("[deadline]"));
    }

    // --- Pattern detector tests ---

    #[test]
    fn test_pattern_detector_multiple_file_patterns() {
        let mut detector = PatternDetector::new();
        let observations: Vec<Observation> = (0..5)
            .flat_map(|_| {
                vec![
                    make_obs(ObservationKind::FileChanged, "Changed: soul.md"),
                    make_obs(ObservationKind::FileChanged, "Changed: context.json"),
                ]
            })
            .collect();

        detector.analyze(&observations);
        let patterns = detector.patterns();
        // Both files should be detected (5 occurrences each >= min 3)
        assert_eq!(patterns.len(), 2);
        assert!(patterns
            .iter()
            .any(|p| p.description.contains("soul.md")));
        assert!(patterns
            .iter()
            .any(|p| p.description.contains("context.json")));
    }

    #[test]
    fn test_pattern_detector_decay_removes_stale() {
        let mut detector = PatternDetector::new();
        detector.analyze(&vec![
            make_obs(ObservationKind::FileChanged, "Changed: soul.md"),
            make_obs(ObservationKind::FileChanged, "Changed: soul.md"),
            make_obs(ObservationKind::FileChanged, "Changed: soul.md"),
        ]);
        assert_eq!(detector.patterns().len(), 1);

        // Manually set the pattern's last_seen to be very old
        if let Some(p) = detector.patterns.iter_mut().next() {
            p.last_seen = "2020-01-01T00:00:00Z".to_string();
            p.confidence = 0.1; // Low confidence
        }

        // Decay should reduce confidence to 0.05, which gets pruned
        detector.decay(7, 0.06);
        assert!(detector.patterns().is_empty());
    }

    #[test]
    fn test_pattern_detector_with_preloaded() {
        use grove_os_lib::heartbeat::patterns::Pattern;

        let existing = vec![Pattern {
            id: "existing".to_string(),
            description: "Active during morning".to_string(),
            confidence: 0.7,
            occurrences: 10,
            first_seen: chrono::Utc::now().to_rfc3339(),
            last_seen: chrono::Utc::now().to_rfc3339(),
            pattern_type: PatternType::TimeOfDay,
        }];

        let detector = PatternDetector::new().with_patterns(existing);
        assert_eq!(detector.patterns().len(), 1);
        assert_eq!(detector.confident_patterns(0.5).len(), 1);
        assert_eq!(detector.confident_patterns(0.9).len(), 0);
    }

    fn make_obs(kind: ObservationKind, detail: &str) -> Observation {
        Observation {
            timestamp: chrono::Utc::now().to_rfc3339(),
            kind,
            detail: detail.to_string(),
        }
    }
}
