pub mod observer;
pub mod patterns;
pub mod scheduler;

use observer::Observer;
use patterns::PatternDetector;
use scheduler::HeartbeatScheduler;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared heartbeat state accessible from Tauri commands.
pub struct HeartbeatState {
    pub scheduler: Arc<Mutex<HeartbeatScheduler>>,
    pub detector: Arc<Mutex<PatternDetector>>,
}

/// Create the heartbeat components and start the background loop.
/// Returns the shared state and spawns the tokio task.
pub fn start_heartbeat(
    grove_dir: PathBuf,
    tick_interval_secs: u64,
    queue_threshold: usize,
) -> HeartbeatState {
    let observer = Observer::new(grove_dir).start_watching();
    let scheduler = Arc::new(Mutex::new(HeartbeatScheduler::new(
        tick_interval_secs,
        queue_threshold,
    )));
    let detector = Arc::new(Mutex::new(PatternDetector::new()));

    // Load existing patterns from disk
    let patterns_dir = dirs::home_dir()
        .map(|h| h.join(".grove").join("memory").join("patterns"))
        .unwrap_or_default();
    if let Ok(loaded) = load_patterns_from_disk(&patterns_dir) {
        if let Ok(mut det) = detector.try_lock() {
            *det = PatternDetector::new().with_patterns(loaded);
        }
    }

    let sched_clone = scheduler.clone();
    let detector_clone = detector.clone();
    let tick_secs = tick_interval_secs;

    // Spawn the background heartbeat loop using tauri's async runtime
    tauri::async_runtime::spawn(async move {
        let interval_duration = std::time::Duration::from_secs(tick_secs);
        let mut interval = tokio::time::interval(interval_duration);
        interval.tick().await; // Skip first immediate tick

        let mut total_observations: Vec<observer::Observation> = Vec::new();

        loop {
            interval.tick().await;

            // 1. Run observer tick
            let observations = observer.tick();

            if !observations.is_empty() {
                eprintln!(
                    "[grove:heartbeat] {} observation(s) this tick",
                    observations.len()
                );
                total_observations.extend(observations.clone());

                // 2. Push to scheduler
                let mut sched = sched_clone.lock().await;
                sched.push_observations(observations);

                // 3. Check if we should trigger reasoning
                if sched.should_trigger() {
                    let drained = sched.drain();
                    eprintln!(
                        "[grove:heartbeat] Triggering reasoning with {} observations",
                        drained.len()
                    );

                    // 4. Run pattern detection on accumulated observations
                    let mut det = detector_clone.lock().await;
                    det.analyze(&total_observations);
                    det.decay(14, 0.05); // Decay patterns older than 14 days

                    // Save patterns to disk
                    let patterns = det.patterns().to_vec();
                    save_patterns_to_disk(&patterns_dir, &patterns).ok();

                    // Clear accumulated after analysis
                    total_observations.clear();

                    // Write observation summary to MEMORY.md
                    let summary = build_observation_summary(&drained);
                    append_to_memory_md(&summary).ok();
                }
            } else {
                // Still count the tick
                let mut sched = sched_clone.lock().await;
                sched.push_observations(vec![]);
            }
        }
    });

    HeartbeatState {
        scheduler,
        detector,
    }
}

/// Build a human-readable summary of observations for MEMORY.md.
fn build_observation_summary(observations: &[observer::Observation]) -> String {
    use chrono::Utc;

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let mut lines = vec![format!("### Heartbeat — {}", now)];

    for obs in observations {
        lines.push(format!("- [{}] {}", obs.kind.label(), obs.detail));
    }

    lines.join("\n")
}

/// Append an entry to ~/.grove/memory.md
fn append_to_memory_md(entry: &str) -> Result<(), String> {
    let path = dirs::home_dir()
        .ok_or("No home dir")?
        .join(".grove")
        .join("memory.md");

    let mut content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.is_empty() {
        content = "# Memory Journal\n\nCross-session observations and events.\n\n".to_string();
    }

    content.push('\n');
    content.push_str(entry);
    content.push('\n');

    std::fs::write(&path, content).map_err(|e| format!("Failed to write memory.md: {}", e))
}

/// Save patterns to individual JSON files in ~/.grove/memory/patterns/
fn save_patterns_to_disk(
    dir: &std::path::Path,
    patterns: &[patterns::Pattern],
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create patterns dir: {}", e))?;

    // Write all patterns to a single file for simplicity
    let path = dir.join("detected.json");
    let content = serde_json::to_string_pretty(patterns)
        .map_err(|e| format!("Failed to serialize patterns: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write patterns: {}", e))
}

/// Load patterns from disk.
fn load_patterns_from_disk(
    dir: &std::path::Path,
) -> Result<Vec<patterns::Pattern>, String> {
    let path = dir.join("detected.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read patterns: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse patterns: {}", e))
}
