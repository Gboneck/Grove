use chrono::{Local, Utc};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// An observation from the heartbeat cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub timestamp: String,
    pub kind: ObservationKind,
    pub detail: String,
}

/// Types of observations the heartbeat can make.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObservationKind {
    FileChanged,
    TimeShift,
    DeadlineApproaching,
    SystemState,
    Idle,
}

/// Time-of-day classification.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeOfDay {
    LateNight,
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            0..=5 => Self::LateNight,
            6..=11 => Self::Morning,
            12..=16 => Self::Afternoon,
            17..=20 => Self::Evening,
            _ => Self::Night,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::LateNight => "late night",
            Self::Morning => "morning",
            Self::Afternoon => "afternoon",
            Self::Evening => "evening",
            Self::Night => "night",
        }
    }
}

/// The Observer watches for changes in the environment and produces Observations.
pub struct Observer {
    grove_dir: PathBuf,
    last_time_of_day: Mutex<TimeOfDay>,
    observation_queue: Arc<Mutex<Vec<Observation>>>,
    _watcher: Option<notify::RecommendedWatcher>,
}

impl Observer {
    /// Create a new Observer that watches the ~/.grove/ directory.
    pub fn new(grove_dir: PathBuf) -> Self {
        let hour = Local::now().hour();
        let initial_tod = TimeOfDay::from_hour(hour);

        Observer {
            grove_dir,
            last_time_of_day: Mutex::new(initial_tod),
            observation_queue: Arc::new(Mutex::new(Vec::new())),
            _watcher: None,
        }
    }

    /// Start watching the filesystem for changes.
    /// Returns the observer with an active watcher, or without if watching fails.
    pub fn start_watching(mut self) -> Self {
        let queue = self.observation_queue.clone();
        let grove_dir = self.grove_dir.clone();

        let watcher_result = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                // Only track meaningful file changes (writes/creates/removes)
                let dominated = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                );
                if !dominated {
                    return;
                }

                // Filter out log files and temp files to avoid noise
                let dominated_paths: Vec<String> = event
                    .paths
                    .iter()
                    .filter(|p| {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        !name.ends_with(".log")
                            && !name.starts_with('.')
                            && !name.ends_with(".tmp")
                    })
                    .filter_map(|p| {
                        // Make path relative to grove_dir for readable detail
                        p.strip_prefix(&grove_dir)
                            .ok()
                            .and_then(|rel| rel.to_str())
                            .map(|s| s.to_string())
                    })
                    .collect();

                if dominated_paths.is_empty() {
                    return;
                }

                let detail = format!("Changed: {}", dominated_paths.join(", "));
                let obs = Observation {
                    timestamp: Utc::now().to_rfc3339(),
                    kind: ObservationKind::FileChanged,
                    detail,
                };

                if let Ok(mut q) = queue.lock() {
                    q.push(obs);
                }
            }
        });

        match watcher_result {
            Ok(mut watcher) => {
                if watcher
                    .watch(&self.grove_dir, RecursiveMode::Recursive)
                    .is_ok()
                {
                    eprintln!(
                        "[grove] File watcher active on {}",
                        self.grove_dir.display()
                    );
                    self._watcher = Some(watcher);
                } else {
                    eprintln!("[grove] File watcher failed to watch directory");
                }
            }
            Err(e) => {
                eprintln!("[grove] File watcher unavailable: {}", e);
            }
        }

        self
    }

    /// Check for time-of-day shifts.
    pub fn check_time_shift(&self) -> Option<Observation> {
        let hour = Local::now().hour();
        let current = TimeOfDay::from_hour(hour);

        let mut last = self.last_time_of_day.lock().ok()?;
        if *last != current {
            let detail = format!(
                "Time shifted from {} to {}",
                last.label(),
                current.label()
            );
            *last = current;
            Some(Observation {
                timestamp: Utc::now().to_rfc3339(),
                kind: ObservationKind::TimeShift,
                detail,
            })
        } else {
            None
        }
    }

    /// Check for approaching venture deadlines.
    pub fn check_deadlines(&self) -> Vec<Observation> {
        let context_path = self.grove_dir.join("context.json");
        let content = match std::fs::read_to_string(&context_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let parsed: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let today = Local::now().date_naive();
        let mut observations = Vec::new();

        if let Some(ventures) = parsed.get("ventures").and_then(|v| v.as_array()) {
            for venture in ventures {
                let name = venture
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown");
                let deadline_str = venture.get("deadline").and_then(|d| d.as_str());

                if let Some(dl) = deadline_str {
                    if let Ok(deadline) = chrono::NaiveDate::parse_from_str(dl, "%Y-%m-%d") {
                        let days_until = (deadline - today).num_days();
                        if days_until >= 0 && days_until <= 7 {
                            observations.push(Observation {
                                timestamp: Utc::now().to_rfc3339(),
                                kind: ObservationKind::DeadlineApproaching,
                                detail: format!(
                                    "{}: deadline in {} day{}",
                                    name,
                                    days_until,
                                    if days_until == 1 { "" } else { "s" }
                                ),
                            });
                        }
                    }
                }
            }
        }

        observations
    }

    /// Drain all queued observations (file changes + explicit observations).
    pub fn drain_observations(&self) -> Vec<Observation> {
        if let Ok(mut q) = self.observation_queue.lock() {
            std::mem::take(&mut *q)
        } else {
            Vec::new()
        }
    }

    /// Run a single observation tick: check time, deadlines, and drain file events.
    pub fn tick(&self) -> Vec<Observation> {
        let mut observations = self.drain_observations();

        if let Some(time_obs) = self.check_time_shift() {
            observations.push(time_obs);
        }

        let deadline_obs = self.check_deadlines();
        observations.extend(deadline_obs);

        observations
    }
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_observer() -> (Observer, TempDir) {
        let tmp = TempDir::new().unwrap();
        let obs = Observer::new(tmp.path().to_path_buf());
        (obs, tmp)
    }

    #[test]
    fn test_time_of_day_classification() {
        assert_eq!(TimeOfDay::from_hour(3), TimeOfDay::LateNight);
        assert_eq!(TimeOfDay::from_hour(9), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(19), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
    }

    #[test]
    fn test_deadline_check_empty_context() {
        let (obs, tmp) = test_observer();
        // No context.json — should return empty
        assert!(obs.check_deadlines().is_empty());

        // Write context with no deadlines
        let ctx = r#"{"ventures": [{"name": "Test", "status": "active"}]}"#;
        fs::write(tmp.path().join("context.json"), ctx).unwrap();
        assert!(obs.check_deadlines().is_empty());
    }

    #[test]
    fn test_drain_observations() {
        let (obs, _tmp) = test_observer();
        // Push a test observation
        obs.observation_queue.lock().unwrap().push(Observation {
            timestamp: "2026-04-04T00:00:00Z".to_string(),
            kind: ObservationKind::FileChanged,
            detail: "test.md".to_string(),
        });

        let drained = obs.drain_observations();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].kind, ObservationKind::FileChanged);

        // Queue should be empty now
        assert!(obs.drain_observations().is_empty());
    }

    #[test]
    fn test_tick_runs_all_checks() {
        let (obs, _tmp) = test_observer();
        // Tick should at least not panic and return observations
        let results = obs.tick();
        // May or may not have time shift depending on test timing
        assert!(results.len() <= 10); // sanity bound
    }
}
