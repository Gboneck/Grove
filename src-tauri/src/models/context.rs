use chrono::Local;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::commands::memory;
use crate::memory::working;
use crate::memory::longterm;
use crate::soul::parser::Soul;
use crate::soul::evolution::RelationshipPhase;

/// Shared cached context — refreshed in background, consumed instantly by reasoning.
pub struct ContextCache(pub Arc<Mutex<Option<GroveContext>>>);

impl ContextCache {
    pub fn new() -> Self {
        ContextCache(Arc::new(Mutex::new(None)))
    }

    /// Refresh the cache by re-gathering context (call from background task).
    pub async fn refresh(&self) {
        match GroveContext::gather(None) {
            Ok(ctx) => {
                let mut cache = self.0.lock().await;
                *cache = Some(ctx);
            }
            Err(e) => {
                eprintln!("[grove:cache] Failed to refresh context: {}", e);
            }
        }
    }

    /// Take the cached context, customizing it with user input and fresh timestamps.
    /// Falls back to a fresh gather if cache is empty.
    pub async fn get_or_gather(&self, user_input: Option<String>) -> Result<GroveContext, String> {
        let cached = {
            let cache = self.0.lock().await;
            cache.clone()
        };

        match cached {
            Some(mut ctx) => {
                // Update time-sensitive fields
                let now = Local::now();
                ctx.local_time = now.to_rfc3339();
                ctx.day_of_week = now.format("%A").to_string();
                ctx.date = now.format("%B %-d, %Y").to_string();
                ctx.user_input = user_input.clone();

                // Vector search is input-dependent, do it fresh
                if let Some(ref input) = user_input {
                    ctx.vector_context = match crate::memory::vector::search_sync(input, 3) {
                        Some(results) if !results.is_empty() => {
                            let items: Vec<String> = results
                                .iter()
                                .map(|r| format!("- [{}] {} (relevance: {:.0}%)", r.category, r.content, r.score * 100.0))
                                .collect();
                            format!("\n--- RELEVANT MEMORIES (semantic search) ---\n{}", items.join("\n"))
                        }
                        _ => String::new(),
                    };
                }

                // Reset fields that callers fill
                ctx.plugin_data = String::new();
                ctx.role_prompt = String::new();
                ctx.conversation_history = None;

                Ok(ctx)
            }
            None => GroveContext::gather(user_input),
        }
    }
}

/// Load detected behavioral patterns from heartbeat as context string.
fn load_detected_patterns(grove_dir: &Path) -> String {
    let path = grove_dir.join("memory").join("patterns").join("detected.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let patterns: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    if patterns.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n--- DETECTED PATTERNS ---\n");
    for p in patterns.iter().take(10) {
        let desc = p.get("description").and_then(|d| d.as_str()).unwrap_or("unknown");
        let conf = p.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
        let count = p.get("occurrences").and_then(|c| c.as_u64()).unwrap_or(0);
        if conf >= 0.3 {
            out.push_str(&format!("- {} (confidence: {:.0}%, seen {}x)\n", desc, conf * 100.0, count));
        }
    }
    out
}

/// Load pending thoughts from background reasoning and clear the file.
fn load_and_clear_pending_thoughts(grove_dir: &Path) -> String {
    let path = grove_dir.join("pending_thoughts.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let thoughts: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(t) => t,
        Err(_) => return String::new(),
    };
    if thoughts.is_empty() {
        return String::new();
    }

    // Clear the file now that we've consumed them
    std::fs::write(&path, "[]").ok();

    let mut out = String::from("\n--- WHAT I THOUGHT ABOUT WHILE YOU WERE AWAY ---\n");
    for t in &thoughts {
        let ts = t.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let summary = t.get("summary").and_then(|v| v.as_str()).unwrap_or("(no summary)");
        let insights = t.get("insights").and_then(|v| v.as_array());

        // Parse timestamp to relative time
        let when = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            let ago = chrono::Utc::now() - dt.with_timezone(&chrono::Utc);
            if ago.num_minutes() < 60 {
                format!("{}m ago", ago.num_minutes())
            } else {
                format!("{}h ago", ago.num_hours())
            }
        } else {
            "recently".to_string()
        };

        out.push_str(&format!("\n[{}] {}\n", when, summary));
        if let Some(ins) = insights {
            for i in ins.iter().filter_map(|v| v.as_str()) {
                out.push_str(&format!("  - {}\n", i));
            }
        }
    }
    eprintln!("[grove] Loaded {} pending thoughts", thoughts.len());
    out
}

/// Load recent prompt copy history — what the user sent to Claude Code.
fn load_prompt_history(grove_dir: &Path) -> String {
    let path = grove_dir.join("prompt_history.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let history: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(h) => h,
        Err(_) => return String::new(),
    };
    if history.is_empty() {
        return String::new();
    }

    // Show last 5 prompts the user copied
    let recent: Vec<_> = history.iter().rev().take(5).collect();
    let mut out = String::from("\n--- PROMPTS SENT TO CLAUDE CODE ---\n");
    out.push_str("The user copied these prompts (presumably pasted into Claude Code to execute):\n");
    for p in &recent {
        let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("untitled");
        let preview = p.get("preview").and_then(|v| v.as_str()).unwrap_or("");
        let when = p.get("copied_at").and_then(|v| v.as_str()).unwrap_or("");
        let relative = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(when) {
            let mins = (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_minutes();
            if mins < 60 { format!("{}m ago", mins) }
            else { format!("{}h ago", mins / 60) }
        } else { "recently".to_string() };

        out.push_str(&format!("- [{}] \"{}\" ({})\n", relative, title, &preview[..preview.len().min(80)]));
    }
    out.push_str("Follow up on these — ask how the build went, update ventures accordingly.\n");
    out
}

/// Load latest screen context from the screen observer cache.
fn load_screen_context(grove_dir: &Path) -> String {
    let path = grove_dir.join("screen_context.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let ctx: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    // Check freshness — only use if < 2 minutes old
    if let Some(ts) = ctx.get("timestamp").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            let age = chrono::Utc::now() - dt.with_timezone(&chrono::Utc);
            if age.num_seconds() > 120 {
                return String::new(); // Stale
            }
        }
    }

    let app = ctx.get("app").and_then(|v| v.as_str()).unwrap_or("");
    let title = ctx.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let text = ctx.get("text_preview").and_then(|v| v.as_str()).unwrap_or("");

    if app.is_empty() && text.is_empty() {
        return String::new();
    }

    let mut out = String::from("\n--- WHAT THE USER IS LOOKING AT RIGHT NOW ---\n");
    out.push_str(&format!("App: {} | Window: {}\n", app, title));
    if !text.is_empty() {
        out.push_str(&format!("Visible text: {}\n", text));
    }
    out.push_str("Use this to be contextually aware. Reference what they're working on without being creepy.\n");
    out
}

/// Enrich raw context.json with computed venture intelligence.
fn enrich_venture_context(raw_json: &str, mem: &memory::Memory) -> String {
    let parsed: serde_json::Value = match serde_json::from_str(raw_json) {
        Ok(v) => v,
        Err(_) => return raw_json.to_string(),
    };

    let ventures = match parsed.get("ventures").and_then(|v| v.as_array()) {
        Some(v) if !v.is_empty() => v,
        _ => return raw_json.to_string(),
    };

    // Count how many sessions ago each venture was last mentioned
    let session_texts: Vec<String> = mem.sessions.iter().rev().take(20).map(|s| {
        let mut text = s.session_summary.clone();
        for input in &s.user_inputs {
            text.push(' ');
            text.push_str(&input.text);
        }
        text.to_lowercase()
    }).collect();

    let mut out = String::from("VENTURES:\n");
    for v in ventures {
        let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
        let health = v.get("health").and_then(|h| h.as_str()).unwrap_or("green");
        let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("active");
        let next = v.get("nextAction").and_then(|n| n.as_str()).unwrap_or("none set");
        let deadline = v.get("deadline").and_then(|d| d.as_str());

        let name_lower = name.to_lowercase();
        let sessions_since = session_texts.iter()
            .position(|s| s.contains(&name_lower))
            .map(|i| i + 1);

        let staleness = match sessions_since {
            Some(1) => "active — mentioned last session",
            Some(2..=3) => "recent — mentioned 2-3 sessions ago",
            Some(4..=7) => "cooling — not mentioned in 4+ sessions",
            Some(_) => "stale — hasn't come up in a while",
            None => "never discussed",
        };

        let deadline_str = match deadline {
            Some(d) => {
                if let Ok(dl) = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                    let days = (dl - Local::now().date_naive()).num_days();
                    if days <= 0 { format!("OVERDUE by {} days", -days) }
                    else if days <= 3 { format!("URGENT: {} days left", days) }
                    else if days <= 7 { format!("{} days left", days) }
                    else { format!("due {}", d) }
                } else {
                    format!("due {}", d)
                }
            }
            None => "no deadline".to_string(),
        };

        out.push_str(&format!(
            "\n- {} [{}] (health: {}, {})\n  Next action: {}\n  Engagement: {}\n",
            name, status, health, deadline_str, next, staleness
        ));
    }
    out
}

/// All context assembled for the reasoning engine
#[derive(Debug, Clone, Serialize)]
pub struct GroveContext {
    pub soul_md: String,
    pub context_json: String,
    pub recent_memory: String,
    pub accumulated_insights: String,
    pub semantic_facts: String,
    pub tuning_hints: String,
    pub plugin_data: String,
    pub conversation_history: Option<String>,
    pub local_time: String,
    pub day_of_week: String,
    pub date: String,
    pub last_seen: String,
    pub user_input: Option<String>,
    /// Structured soul data (parsed from soul.md)
    pub soul_completeness: f64,
    /// Current relationship phase with the user
    pub relationship_phase: String,
    /// Phase-specific system prompt modifier
    pub phase_prompt: String,
    /// Weak soul sections that need enrichment
    pub soul_gaps: Vec<String>,
    /// Cross-session memory journal (recent MEMORY.md entries)
    pub working_memory: String,
    /// Long-term patterns context
    pub longterm_context: String,
    /// Active role prompt modifier (from YAML role config)
    pub role_prompt: String,
    /// Semantic vector search results relevant to current input
    pub vector_context: String,
    /// Grove's self-reference document (system.md)
    pub system_reference: String,
    /// Detected behavioral patterns from heartbeat
    pub detected_patterns: String,
    /// True when soul.md is still the default template (first-meeting flow)
    pub is_first_meeting: bool,
    /// Pending thoughts from background reasoning (accumulated while user was away)
    pub pending_thoughts: String,
    /// Latest screen context from screen observer (app, window, visible text)
    pub screen_context: String,
    /// Workspace context — what blocks are on the user's canvas
    pub workspace_context: String,
    /// Recent prompts the user copied (assumed sent to Claude Code)
    pub prompt_history: String,
}

impl GroveContext {
    pub fn gather(user_input: Option<String>) -> Result<Self, String> {
        let grove_dir = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".grove");

        // Read soul.md
        let soul_md = std::fs::read_to_string(grove_dir.join("soul.md"))
            .map_err(|e| format!("Failed to read soul.md: {}", e))?;
        let is_first_meeting = soul_md.contains("[Your name, what you do, where you're based]");

        // Read memory first (needed for venture enrichment)
        let memory_data = memory::read_memory_file().unwrap_or_default();

        // Read context.json and enrich with computed venture data
        let context_json = {
            let raw = std::fs::read_to_string(grove_dir.join("context.json"))
                .map_err(|e| format!("Failed to read context.json: {}", e))?;
            enrich_venture_context(&raw, &memory_data)
        };
        let recent_sessions: Vec<_> = memory_data.sessions.iter().rev().take(5).collect();

        let recent_memory = if recent_sessions.is_empty() {
            "No previous sessions.".to_string()
        } else {
            recent_sessions
                .iter()
                .map(|s| {
                    let inputs: Vec<String> =
                        s.user_inputs.iter().map(|i| i.text.clone()).collect();
                    format!(
                        "- {} ({}): {} | User said: {}",
                        s.timestamp,
                        s.time_of_day,
                        s.session_summary,
                        if inputs.is_empty() {
                            "(no input)".to_string()
                        } else {
                            inputs.join(", ")
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let accumulated_insights = if memory_data.accumulated_insights.is_empty() {
            "None yet.".to_string()
        } else {
            memory_data
                .accumulated_insights
                .iter()
                .map(|i| format!("- {}", i))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Semantic facts
        let semantic_facts = if memory_data.facts.is_empty() {
            String::new()
        } else {
            let facts: Vec<String> = memory_data
                .facts
                .iter()
                .filter(|f| f.superseded_by.is_none() && f.confidence >= 0.5)
                .take(30)
                .map(|f| format!("- [{}] {} (confidence: {:.1})", f.category, f.content, f.confidence))
                .collect();
            if facts.is_empty() {
                String::new()
            } else {
                format!("\n--- KNOWN FACTS ABOUT USER ---\n{}", facts.join("\n"))
            }
        };

        // Self-tuning hints
        let tuning_hints = {
            let t = &memory_data.tuning;
            if t.total_sessions == 0 {
                String::new()
            } else {
                let mut hints = Vec::new();

                // Find most/least engaged block types
                let mut engagements: Vec<_> = t.block_type_engagement.iter().collect();
                engagements.sort_by(|a, b| {
                    let rate_a = if a.1.shown > 0 { a.1.interacted as f64 / a.1.shown as f64 } else { 0.0 };
                    let rate_b = if b.1.shown > 0 { b.1.interacted as f64 / b.1.shown as f64 } else { 0.0 };
                    rate_b.partial_cmp(&rate_a).unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some((name, eng)) = engagements.first() {
                    if eng.interacted > 0 {
                        hints.push(format!("User engages most with: {}", name));
                    }
                }
                if let Some((name, eng)) = engagements.last() {
                    if eng.shown > 3 && eng.interacted == 0 {
                        hints.push(format!("User rarely engages with: {} (shown {} times, never clicked)", name, eng.shown));
                    }
                }

                if !t.preferred_session_times.is_empty() {
                    hints.push(format!("User typically opens Grove during: {}", t.preferred_session_times.join(", ")));
                }

                if hints.is_empty() {
                    String::new()
                } else {
                    format!("\n--- SELF-TUNING OBSERVATIONS ---\n{}", hints.join("\n"))
                }
            }
        };

        let last_seen = memory_data
            .last_seen
            .unwrap_or_else(|| "never — this is the first session".to_string());

        // Parse soul.md into structured sections
        let soul = Soul::parse(&soul_md);
        let soul_completeness = soul.completeness();
        let session_count = memory_data.sessions.len() as u32;
        let phase = RelationshipPhase::from_metrics(soul_completeness, session_count);
        let soul_gaps: Vec<String> = soul
            .weak_sections(0.5)
            .iter()
            .map(|s| s.heading.clone())
            .collect();

        // Read cross-session memory journal (recent entries, capped at 2000 chars)
        let working_memory_raw = working::recent_entries(2000);
        let working_memory = if working_memory_raw.trim().is_empty()
            || working_memory_raw.trim() == "# Memory Journal"
        {
            String::new()
        } else {
            format!("\n--- MEMORY JOURNAL (cross-session) ---\n{}", working_memory_raw)
        };

        // Read long-term patterns
        let longterm_context = longterm::context_summary();

        // Semantic vector search — if user has input, find relevant memories
        let vector_context = if let Some(ref input) = user_input {
            match crate::memory::vector::search_sync(input, 3) {
                Some(results) if !results.is_empty() => {
                    let items: Vec<String> = results
                        .iter()
                        .map(|r| {
                            format!(
                                "- [{}] {} (relevance: {:.0}%)",
                                r.category,
                                r.content,
                                r.score * 100.0
                            )
                        })
                        .collect();
                    format!(
                        "\n--- RELEVANT MEMORIES (semantic search) ---\n{}",
                        items.join("\n")
                    )
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };

        let now = Local::now();

        Ok(GroveContext {
            soul_md,
            context_json,
            recent_memory,
            accumulated_insights,
            semantic_facts,
            tuning_hints,
            plugin_data: String::new(), // Filled by caller if plugins are active
            conversation_history: None, // Filled by caller for multi-turn
            local_time: now.to_rfc3339(),
            day_of_week: now.format("%A").to_string(),
            date: now.format("%B %-d, %Y").to_string(),
            last_seen,
            user_input,
            soul_completeness,
            relationship_phase: phase.display_name().to_string(),
            phase_prompt: phase.system_prompt_modifier().to_string(),
            soul_gaps,
            working_memory,
            longterm_context,
            role_prompt: String::new(), // Filled by caller when a role is active
            vector_context,
            system_reference: std::fs::read_to_string(grove_dir.join("system.md")).unwrap_or_default(),
            detected_patterns: load_detected_patterns(&grove_dir),
            is_first_meeting,
            pending_thoughts: load_and_clear_pending_thoughts(&grove_dir),
            screen_context: load_screen_context(&grove_dir),
            workspace_context: crate::commands::workspace::workspace_context_for_model(),
            prompt_history: load_prompt_history(&grove_dir),
        })
    }

    /// Build the user message sent to the model
    pub fn to_user_message(&self) -> String {
        let soul_meta = format!(
            "Soul completeness: {:.0}% | Relationship phase: {} | Gaps: {}",
            self.soul_completeness * 100.0,
            self.relationship_phase,
            if self.soul_gaps.is_empty() {
                "none".to_string()
            } else {
                self.soul_gaps.join(", ")
            }
        );

        let system_ref = if self.system_reference.is_empty() {
            String::new()
        } else {
            format!("\n--- SYSTEM SELF-REFERENCE ---\n{}", self.system_reference)
        };

        format!(
            r#"Current time: {}
Day: {}, {}
Time since last session: {}

--- SOUL.MD ---
{}

--- SOUL METADATA ---
{}

--- ACTIVE CONTEXT ---
{}

--- RECENT MEMORY ---
{}

--- ACCUMULATED INSIGHTS ---
{}
{}{}{}{}{}{}{}{}{}{}{}{}
{}
{}

Decide what to show. Return JSON only."#,
            self.local_time,
            self.day_of_week,
            self.date,
            self.last_seen,
            self.soul_md,
            soul_meta,
            self.context_json,
            self.recent_memory,
            self.accumulated_insights,
            self.semantic_facts,
            self.tuning_hints,
            self.plugin_data,
            if self.vector_context.is_empty() { "" } else { &self.vector_context },
            if self.working_memory.is_empty() { "" } else { &self.working_memory },
            if self.longterm_context.is_empty() { "" } else { &self.longterm_context },
            if self.detected_patterns.is_empty() { "" } else { &self.detected_patterns },
            if self.pending_thoughts.is_empty() { "" } else { &self.pending_thoughts },
            if self.screen_context.is_empty() { "" } else { &self.screen_context },
            if self.workspace_context.is_empty() { "" } else { &self.workspace_context },
            if self.prompt_history.is_empty() { "" } else { &self.prompt_history },
            system_ref,
            self.conversation_history
                .as_ref()
                .map(|h| format!("\n--- CONVERSATION HISTORY ---\n{}", h))
                .unwrap_or_default(),
            self.user_input
                .as_ref()
                .map(|i| format!("--- USER INPUT ---\n{}", i))
                .unwrap_or_default()
        )
    }
}
