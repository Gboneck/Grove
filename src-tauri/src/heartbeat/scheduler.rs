use super::observer::{Observation, ObservationKind};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// The heartbeat scheduler manages tick intervals and the observation queue.
/// It decides when accumulated observations warrant triggering a reasoning cycle.
pub struct HeartbeatScheduler {
    /// Observations queued since last reasoning cycle.
    queue: Vec<Observation>,
    /// How many observations trigger an automatic reasoning cycle.
    queue_threshold: usize,
    /// Tick interval in seconds.
    pub tick_interval_secs: u64,
    /// Timestamp of last triggered reasoning cycle.
    last_trigger: Option<String>,
    /// Minimum seconds between auto-triggered reasoning cycles.
    cooldown_secs: u64,
    /// Total ticks since startup.
    total_ticks: u64,
}

impl HeartbeatScheduler {
    pub fn new(tick_interval_secs: u64, queue_threshold: usize) -> Self {
        Self {
            queue: Vec::new(),
            queue_threshold,
            tick_interval_secs,
            last_trigger: None,
            cooldown_secs: 120, // At least 2 minutes between auto-triggered cycles
            total_ticks: 0,
        }
    }

    /// Push observations from a tick into the queue.
    pub fn push_observations(&mut self, observations: Vec<Observation>) {
        self.queue.extend(observations);
        self.total_ticks += 1;
    }

    /// Check whether we should trigger a reasoning cycle.
    pub fn should_trigger(&self) -> bool {
        if self.queue.is_empty() {
            return false;
        }

        // Check cooldown
        if let Some(ref last) = self.last_trigger {
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last) {
                let elapsed = (Utc::now() - last_time.with_timezone(&Utc)).num_seconds();
                if elapsed < self.cooldown_secs as i64 {
                    return false;
                }
            }
        }

        // Trigger if we've hit the observation threshold
        if self.queue.len() >= self.queue_threshold {
            return true;
        }

        // Trigger immediately for high-priority observations
        self.queue.iter().any(|obs| {
            matches!(
                obs.kind,
                ObservationKind::DeadlineApproaching | ObservationKind::SystemState
            )
        })
    }

    /// Drain the queue and mark the trigger time.
    pub fn drain(&mut self) -> Vec<Observation> {
        self.last_trigger = Some(Utc::now().to_rfc3339());
        std::mem::take(&mut self.queue)
    }

    /// Get the current queue size.
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Get total ticks since startup.
    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    /// Build a summary of queued observations for the reasoning context.
    pub fn queue_summary(&self) -> String {
        if self.queue.is_empty() {
            return "No pending observations.".to_string();
        }

        let mut summary = format!("{} observation(s) since last cycle:\n", self.queue.len());
        for obs in &self.queue {
            summary.push_str(&format!("- [{}] {}\n", obs.kind.label(), obs.detail));
        }
        summary
    }
}

impl Default for HeartbeatScheduler {
    fn default() -> Self {
        Self::new(300, 5) // 5-minute ticks, trigger after 5 observations
    }
}

impl ObservationKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FileChanged => "file",
            Self::TimeShift => "time",
            Self::DeadlineApproaching => "deadline",
            Self::SystemState => "system",
            Self::ScreenContext => "screen",
            Self::Idle => "idle",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obs(kind: ObservationKind, detail: &str) -> Observation {
        Observation {
            timestamp: Utc::now().to_rfc3339(),
            kind,
            detail: detail.to_string(),
        }
    }

    #[test]
    fn test_empty_queue_no_trigger() {
        let sched = HeartbeatScheduler::default();
        assert!(!sched.should_trigger());
        assert_eq!(sched.queue_size(), 0);
    }

    #[test]
    fn test_threshold_trigger() {
        let mut sched = HeartbeatScheduler::new(300, 3);
        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "soul.md"),
            make_obs(ObservationKind::FileChanged, "context.json"),
            make_obs(ObservationKind::TimeShift, "morning to afternoon"),
        ]);
        assert!(sched.should_trigger());
        assert_eq!(sched.queue_size(), 3);
    }

    #[test]
    fn test_priority_trigger() {
        let mut sched = HeartbeatScheduler::new(300, 10); // high threshold
        sched.push_observations(vec![make_obs(
            ObservationKind::DeadlineApproaching,
            "EMBER: 3 days",
        )]);
        // Should trigger even with 1 observation because it's high-priority
        assert!(sched.should_trigger());
    }

    #[test]
    fn test_drain_resets() {
        let mut sched = HeartbeatScheduler::new(300, 2);
        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "a.md"),
            make_obs(ObservationKind::FileChanged, "b.md"),
        ]);
        assert!(sched.should_trigger());

        let drained = sched.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(sched.queue_size(), 0);
        assert!(!sched.should_trigger());
    }

    #[test]
    fn test_cooldown() {
        let mut sched = HeartbeatScheduler::new(300, 1);
        sched.push_observations(vec![make_obs(ObservationKind::FileChanged, "a.md")]);
        sched.drain(); // Sets last_trigger to now

        // Push more observations immediately
        sched.push_observations(vec![make_obs(ObservationKind::FileChanged, "b.md")]);
        // Should NOT trigger because cooldown hasn't elapsed
        assert!(!sched.should_trigger());
    }

    #[test]
    fn test_queue_summary() {
        let mut sched = HeartbeatScheduler::default();
        assert_eq!(sched.queue_summary(), "No pending observations.");

        sched.push_observations(vec![
            make_obs(ObservationKind::FileChanged, "soul.md changed"),
            make_obs(ObservationKind::TimeShift, "morning to afternoon"),
        ]);

        let summary = sched.queue_summary();
        assert!(summary.contains("2 observation(s)"));
        assert!(summary.contains("[file]"));
        assert!(summary.contains("[time]"));
    }

    #[test]
    fn test_total_ticks() {
        let mut sched = HeartbeatScheduler::default();
        assert_eq!(sched.total_ticks(), 0);
        sched.push_observations(vec![]);
        assert_eq!(sched.total_ticks(), 1);
        sched.push_observations(vec![make_obs(ObservationKind::Idle, "test")]);
        assert_eq!(sched.total_ticks(), 2);
    }
}
