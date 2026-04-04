use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::observer::{Observation, ObservationKind};

/// A detected behavioral pattern from accumulated observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub description: String,
    pub confidence: f64,
    pub occurrences: u32,
    pub first_seen: String,
    pub last_seen: String,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    /// User tends to work at certain times
    TimeOfDay,
    /// User edits certain files frequently
    FileActivity,
    /// User focuses on certain ventures at certain times
    VentureFocus,
    /// User has idle periods at predictable times
    IdlePeriod,
}

/// Detects patterns from a history of observations.
pub struct PatternDetector {
    /// Minimum occurrences before a pattern is considered real.
    min_occurrences: u32,
    /// Known patterns.
    patterns: Vec<Pattern>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            min_occurrences: 3,
            patterns: Vec::new(),
        }
    }

    /// Load existing patterns from file.
    pub fn with_patterns(mut self, patterns: Vec<Pattern>) -> Self {
        self.patterns = patterns;
        self
    }

    /// Analyze a batch of observations and detect/update patterns.
    pub fn analyze(&mut self, observations: &[Observation]) {
        self.detect_file_activity_patterns(observations);
        self.detect_time_patterns(observations);
    }

    /// Detect which files are changed most frequently.
    fn detect_file_activity_patterns(&mut self, observations: &[Observation]) {
        let mut file_counts: HashMap<String, u32> = HashMap::new();

        for obs in observations {
            if obs.kind != ObservationKind::FileChanged {
                continue;
            }
            // Extract filenames from detail like "Changed: soul.md, context.json"
            let detail = obs.detail.strip_prefix("Changed: ").unwrap_or(&obs.detail);
            for file in detail.split(", ") {
                let file = file.trim();
                if !file.is_empty() {
                    *file_counts.entry(file.to_string()).or_default() += 1;
                }
            }
        }

        let now = Utc::now().to_rfc3339();
        for (file, count) in &file_counts {
            if *count < self.min_occurrences {
                continue;
            }

            let desc = format!("Frequently edits {}", file);
            if let Some(existing) = self
                .patterns
                .iter_mut()
                .find(|p| p.pattern_type == PatternType::FileActivity && p.description == desc)
            {
                existing.occurrences += count;
                existing.last_seen = now.clone();
                existing.confidence = (existing.confidence + 0.05).min(1.0);
            } else {
                self.patterns.push(Pattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: desc,
                    confidence: 0.5,
                    occurrences: *count,
                    first_seen: now.clone(),
                    last_seen: now.clone(),
                    pattern_type: PatternType::FileActivity,
                });
            }
        }
    }

    /// Detect time-of-day usage patterns from TimeShift observations.
    fn detect_time_patterns(&mut self, observations: &[Observation]) {
        let mut time_counts: HashMap<String, u32> = HashMap::new();

        for obs in observations {
            if obs.kind != ObservationKind::TimeShift {
                continue;
            }
            // Detail like "Time shifted from morning to afternoon"
            if let Some(to_part) = obs.detail.split(" to ").last() {
                *time_counts.entry(to_part.trim().to_string()).or_default() += 1;
            }
        }

        let now = Utc::now().to_rfc3339();
        for (time, count) in &time_counts {
            if *count < self.min_occurrences {
                continue;
            }

            let desc = format!("Active during {}", time);
            if let Some(existing) = self
                .patterns
                .iter_mut()
                .find(|p| p.pattern_type == PatternType::TimeOfDay && p.description == desc)
            {
                existing.occurrences += count;
                existing.last_seen = now.clone();
                existing.confidence = (existing.confidence + 0.05).min(1.0);
            } else {
                self.patterns.push(Pattern {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: desc,
                    confidence: 0.4,
                    occurrences: *count,
                    first_seen: now.clone(),
                    last_seen: now.clone(),
                    pattern_type: PatternType::TimeOfDay,
                });
            }
        }
    }

    /// Get all detected patterns.
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    /// Get patterns above a confidence threshold.
    pub fn confident_patterns(&self, threshold: f64) -> Vec<&Pattern> {
        self.patterns
            .iter()
            .filter(|p| p.confidence >= threshold)
            .collect()
    }

    /// Decay pattern confidence for stale patterns.
    pub fn decay(&mut self, days_threshold: i64, decay_amount: f64) {
        let now = Utc::now();
        for pattern in &mut self.patterns {
            if let Ok(last) = chrono::DateTime::parse_from_rfc3339(&pattern.last_seen) {
                let days = (now - last.with_timezone(&Utc)).num_days();
                if days > days_threshold {
                    pattern.confidence = (pattern.confidence - decay_amount).max(0.0);
                }
            }
        }
        // Remove dead patterns
        self.patterns.retain(|p| p.confidence > 0.05);
    }

    /// Consume the detector, returning owned patterns.
    pub fn into_patterns(self) -> Vec<Pattern> {
        self.patterns
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_obs(detail: &str) -> Observation {
        Observation {
            timestamp: Utc::now().to_rfc3339(),
            kind: ObservationKind::FileChanged,
            detail: detail.to_string(),
        }
    }

    fn time_obs(detail: &str) -> Observation {
        Observation {
            timestamp: Utc::now().to_rfc3339(),
            kind: ObservationKind::TimeShift,
            detail: detail.to_string(),
        }
    }

    #[test]
    fn test_file_activity_detection() {
        let mut detector = PatternDetector::new();
        let observations = vec![
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
            file_obs("Changed: context.json"),
        ];
        detector.analyze(&observations);

        // soul.md should be detected (3 occurrences >= min 3)
        let patterns = detector.patterns();
        assert_eq!(patterns.len(), 1);
        assert!(patterns[0].description.contains("soul.md"));
        assert_eq!(patterns[0].occurrences, 3);
    }

    #[test]
    fn test_time_pattern_detection() {
        let mut detector = PatternDetector::new();
        let observations = vec![
            time_obs("Time shifted from morning to afternoon"),
            time_obs("Time shifted from morning to afternoon"),
            time_obs("Time shifted from morning to afternoon"),
        ];
        detector.analyze(&observations);

        let patterns = detector.patterns();
        assert_eq!(patterns.len(), 1);
        assert!(patterns[0].description.contains("afternoon"));
    }

    #[test]
    fn test_below_threshold_ignored() {
        let mut detector = PatternDetector::new();
        let observations = vec![
            file_obs("Changed: rare_file.md"),
            file_obs("Changed: rare_file.md"),
        ];
        detector.analyze(&observations);
        assert!(detector.patterns().is_empty());
    }

    #[test]
    fn test_pattern_accumulation() {
        let mut detector = PatternDetector::new();
        let batch1 = vec![
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
        ];
        detector.analyze(&batch1);
        assert_eq!(detector.patterns()[0].occurrences, 3);

        // Second batch should accumulate
        let batch2 = vec![
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
            file_obs("Changed: soul.md"),
        ];
        detector.analyze(&batch2);
        assert_eq!(detector.patterns()[0].occurrences, 6);
    }

    #[test]
    fn test_decay() {
        let mut detector = PatternDetector::new();
        detector.patterns.push(Pattern {
            id: "test".to_string(),
            description: "Old pattern".to_string(),
            confidence: 0.3,
            occurrences: 5,
            first_seen: "2026-01-01T00:00:00Z".to_string(),
            last_seen: "2026-01-01T00:00:00Z".to_string(), // Very old
            pattern_type: PatternType::FileActivity,
        });
        detector.decay(7, 0.1);
        assert!(detector.patterns()[0].confidence < 0.3);
    }

    #[test]
    fn test_confident_patterns() {
        let mut detector = PatternDetector::new();
        detector.patterns.push(Pattern {
            id: "high".to_string(),
            description: "High confidence".to_string(),
            confidence: 0.8,
            occurrences: 10,
            first_seen: Utc::now().to_rfc3339(),
            last_seen: Utc::now().to_rfc3339(),
            pattern_type: PatternType::TimeOfDay,
        });
        detector.patterns.push(Pattern {
            id: "low".to_string(),
            description: "Low confidence".to_string(),
            confidence: 0.3,
            occurrences: 2,
            first_seen: Utc::now().to_rfc3339(),
            last_seen: Utc::now().to_rfc3339(),
            pattern_type: PatternType::FileActivity,
        });

        let confident = detector.confident_patterns(0.5);
        assert_eq!(confident.len(), 1);
        assert_eq!(confident[0].description, "High confidence");
    }
}
