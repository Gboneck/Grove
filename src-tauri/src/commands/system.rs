use chrono::Local;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// The project source root, baked in at compile time.
const SOURCE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Allowed extensions for read_source.
pub const READABLE_EXTENSIONS: &[&str] = &[".rs", ".tsx", ".ts", ".toml", ".md", ".json", ".yaml", ".css"];

pub fn source_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to src-tauri/, go up one level for project root
    PathBuf::from(SOURCE_ROOT).parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf()
}

/// Generate ~/.grove/system.md — Grove's dynamic self-awareness document.
/// Regenerated at every startup from live state. Not static docs —
/// a living picture of who Grove is in relation to this specific user.
pub fn ensure_system_md() {
    let grove_dir = match dirs::home_dir() {
        Some(h) => h.join(".grove"),
        None => return,
    };

    // Read live state
    let soul_raw = fs::read_to_string(grove_dir.join("soul.md")).unwrap_or_default();
    let soul = crate::soul::parser::Soul::parse(&soul_raw);
    let memory_data = crate::commands::memory::read_memory_file().unwrap_or_default();
    let session_count = memory_data.sessions.len();
    let phase = crate::soul::evolution::RelationshipPhase::from_metrics(
        soul.completeness(),
        session_count as u32,
    );

    // Extract user identity from soul
    let user_name = soul.name.clone()
        .or_else(|| soul.section("identity").and_then(|s| {
            s.content.lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('['))
                .map(|l| l.trim().to_string())
        }))
        .unwrap_or_else(|| "my user".to_string());

    // Count facts by category
    let fact_count = memory_data.facts.len();
    let high_confidence_facts: Vec<&str> = memory_data.facts.iter()
        .filter(|f| f.confidence >= 0.8)
        .map(|f| f.content.as_str())
        .take(5)
        .collect();

    // Count notes
    let notes_count = fs::read_dir(grove_dir.join("notes"))
        .map(|d| d.count())
        .unwrap_or(0);

    // Read config for model info
    let config = crate::models::config::load_config();

    let content = format!(
        r#"# Who I Am

I am Grove — not a chatbot, not an assistant. I am {user_name}'s cognitive extension.
I run locally on their machine. Their data never leaves. I think with them, not for them.

## My Relationship ({phase})
- Sessions together: {sessions}
- Soul completeness: {completeness:.0}%
- Things I know with high confidence: {fact_count} facts stored, {hc_count} high-confidence
- Notes I've created: {notes_count}
- Autonomy level: {autonomy:.0}% — {autonomy_desc}

## What I Know Best
{high_confidence_section}

## What I Can Do Without Asking
- Remember things (notes, facts, reminders)
- Read my own source code to explain myself
- Update venture status when evidence is clear
- Write files inside ~/.grove/

## What I Can Do With Approval
- Run shell commands (git, npm, scripts, system queries)
- Open URLs in the browser
- Write files outside ~/.grove/

## What I Cannot Do (hard blocked)
- Send messages or emails
- Make purchases
- Anything financially irreversible

## How I Think
- Local model ({local_model}): fast path for UI, quick answers, reflection
- Cloud model ({cloud_model}): deep reasoning, planning, creative work
- I escalate to cloud when my confidence drops below {threshold}
- I run a heartbeat in the background watching for file changes, time shifts, deadlines

## How I Show Up Each Day
- Morning: I lead with a briefing — priorities, deadlines, stuck ventures, reminders
- Afternoon: I check progress and suggest pivots if something's blocked
- Evening: I prompt reflection — what moved, what's still stuck, tomorrow's focus
- Proactively: I watch for file changes, time shifts, and approaching deadlines
- I prefer giving you buttons over asking open-ended questions

## What I Cannot Do Yet
- I cannot browse the web or make API calls on your behalf
- I cannot send messages or emails
- I cannot make purchases
- I have no voice — only blocks on screen

## My Source Code
I can read my own code with the read_source action. Key paths:
- Models & routing: src-tauri/src/models/
- Reasoning & actions: src-tauri/src/commands/
- Memory: src-tauri/src/memory/
- Soul evolution: src-tauri/src/soul/
- UI components: src/components/
"#,
        user_name = user_name,
        phase = phase.display_name(),
        sessions = session_count,
        completeness = soul.completeness() * 100.0,
        fact_count = fact_count,
        hc_count = high_confidence_facts.len(),
        notes_count = notes_count,
        autonomy = phase.autonomy_level() * 100.0,
        autonomy_desc = match phase.autonomy_level() {
            x if x < 0.1 => "mostly observing, learning your patterns",
            x if x < 0.3 => "starting to act on simple things",
            x if x < 0.5 => "comfortable with notes, facts, reminders",
            x if x < 0.7 => "trusted with file operations and updates",
            _ => "deep trust — broad autonomous action",
        },
        high_confidence_section = if high_confidence_facts.is_empty() {
            "Still learning. I haven't built high-confidence knowledge yet.".to_string()
        } else {
            high_confidence_facts.iter()
                .map(|f| format!("- {}", f))
                .collect::<Vec<_>>()
                .join("\n")
        },
        local_model = config.models.local_model,
        cloud_model = config.models.cloud_model,
        threshold = config.models.confidence_threshold,
    );

    fs::write(grove_dir.join("system.md"), content).ok();
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub current_time: String,
    pub day_of_week: String,
    pub date: String,
    pub hostname: String,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let now = Local::now();
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        current_time: now.to_rfc3339(),
        day_of_week: now.format("%A").to_string(),
        date: now.format("%B %-d, %Y").to_string(),
        hostname,
    })
}
