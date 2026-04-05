use std::process::Command;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use serde::Deserialize;

use super::observer::{Observation, ObservationKind};

#[derive(Debug, Deserialize)]
struct ScreenCapture {
    app: String,
    title: String,
    text: String,
    timestamp: String,
}

/// Observes the user's screen periodically via macOS native APIs.
/// Screenshots are never stored — only extracted text and app context.
pub struct ScreenObserver {
    last_capture: Mutex<Instant>,
    capture_interval: Duration,
    last_context: Mutex<String>, // Dedup: skip if nothing changed
    script_path: String,
    enabled: bool,
}

impl ScreenObserver {
    pub fn new(interval_secs: u64) -> Self {
        // Locate the OCR script relative to the binary
        let script_path = Self::find_script();
        let enabled = !script_path.is_empty();

        if enabled {
            eprintln!("[grove:screen] Screen observer enabled ({}s interval)", interval_secs);
        } else {
            eprintln!("[grove:screen] Screen observer disabled — script not found");
        }

        Self {
            last_capture: Mutex::new(Instant::now()),
            capture_interval: Duration::from_secs(interval_secs),
            last_context: Mutex::new(String::new()),
            script_path,
            enabled,
        }
    }

    fn find_script() -> String {
        // Check several possible locations
        let candidates = [
            // Development: relative to CARGO_MANIFEST_DIR
            concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/screen-ocr.sh"),
            // Installed: next to the binary
            "scripts/screen-ocr.sh",
        ];

        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        // Try ~/.grove/scripts/
        if let Some(home) = dirs::home_dir() {
            let grove_script = home.join(".grove").join("scripts").join("screen-ocr.sh");
            if grove_script.exists() {
                return grove_script.to_string_lossy().to_string();
            }
        }

        String::new()
    }

    /// Attempt a screen capture + OCR. Returns an observation if new context was found.
    pub fn tick(&self) -> Option<Observation> {
        if !self.enabled {
            return None;
        }

        // Check interval
        {
            let mut last = self.last_capture.lock().ok()?;
            if last.elapsed() < self.capture_interval {
                return None;
            }
            *last = Instant::now();
        }

        // Run the OCR script with a timeout
        let output = Command::new("bash")
            .arg(&self.script_path)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let capture: ScreenCapture = serde_json::from_str(stdout.trim()).ok()?;

        // Build context string
        let context = format!("{}|{}|{}", capture.app, capture.title, &capture.text[..capture.text.len().min(200)]);

        // Dedup: skip if context hasn't meaningfully changed
        {
            let mut last_ctx = self.last_context.lock().ok()?;
            if *last_ctx == context {
                return None; // Same app, same content — skip
            }
            *last_ctx = context;
        }

        // Build a concise detail string for the observation
        let detail = if capture.text.is_empty() {
            format!("Screen: {} — {}", capture.app, capture.title)
        } else {
            // Take first 300 chars of OCR text
            let text_preview = if capture.text.len() > 300 {
                format!("{}...", &capture.text[..300])
            } else {
                capture.text.clone()
            };
            format!(
                "Screen: {} — {} | Visible text: {}",
                capture.app, capture.title, text_preview
            )
        };

        // Write latest context to cache file for immediate reasoning access
        if let Some(home) = dirs::home_dir() {
            let cache_path = home.join(".grove").join("screen_context.json");
            let cache = serde_json::json!({
                "app": capture.app,
                "title": capture.title,
                "text_preview": &capture.text[..capture.text.len().min(500)],
                "timestamp": capture.timestamp,
            });
            if let Ok(json) = serde_json::to_string(&cache) {
                std::fs::write(&cache_path, json).ok();
            }
        }

        Some(Observation {
            timestamp: capture.timestamp,
            kind: ObservationKind::ScreenContext,
            detail,
        })
    }
}
