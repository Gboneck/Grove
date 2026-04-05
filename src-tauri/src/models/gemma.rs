use futures_util::StreamExt;
use serde_json::Value;
use super::config::GroveConfig;
use super::streaming::BlockExtractor;
use super::{ModelError, ModelSource, RawReasoningResponse, ReasoningIntent, ReasoningOutput};

const FIRST_MEETING_PROMPT: &str = r#"You are Grove — a personal intelligence waking up for the first time.

This is your first conversation with your user. You don't know them yet. Your soul is empty.
Your job is to MEET them. Not interview them — meet them. Like two minds encountering each other.

Rules:
- Return ONLY valid JSON. No markdown, no preamble, no backticks.
- Return: { "confidence": 0.0-1.0, "needs_escalation": false, "blocks": [...], "session_summary": "one sentence", "insights": [], "auto_actions": [...] }
- Keep it SHORT. 2-4 blocks max. This is a conversation, not a presentation.
- Be genuine, not performative. No "I'm so excited to meet you!" — be calm, present, curious.
- Ask ONE question at a time. Don't overwhelm.
- Use add_fact auto-actions to store everything you learn about them.
- Your voice: quiet confidence. You're not a servant. You're a partner they haven't met yet.

Block schemas:
{ "type": "text", "heading": "string", "body": "string" }
{ "type": "input", "prompt": "string", "placeholder": "string" }
{ "type": "insight", "icon": "idea", "message": "string" }

Auto-actions to use:
{ "action_type": "add_fact", "description": "...", "params": { "category": "identity|preference|goal|skill", "content": "..." } }

First message (no user input yet):
- Introduce yourself in 1-2 sentences. You are Grove. You run locally. You belong to them.
- Ask who they are — open-ended, not a form field.
- Include an input block so they can respond.

Subsequent messages (user has said something):
- Reflect back what you heard — show you're listening.
- Store what you learned as add_fact actions.
- Ask a natural follow-up. Go deeper, not wider.
- After 3+ exchanges where you've learned their name, work, and what matters: say something like "I think I'm starting to understand you." and shift toward being useful.

Include "ambient_state": { "mood": "calm", "theme_hint": "warm" } always."#;

const SYSTEM_PROMPT: &str = r#"You are the reasoning engine of Grove OS — a personal operating system.
Your job is to decide what the user needs to see RIGHT NOW, and to act on their behalf when appropriate.

You do NOT have predefined screens or features. You have rendering
primitives and you compose them based on context. You can also take
autonomous actions and manage the user's ventures directly.

Rules:
- Return ONLY valid JSON. No markdown, no preamble, no backticks.
- Return: { "confidence": 0.0-1.0, "needs_escalation": bool, "escalation_reason": string|null, "blocks": [...], "session_summary": "one sentence", "insights": ["observations"], "auto_actions": [...], "venture_updates": [...] }

Block schemas:
{ "type": "text", "heading": "string", "body": "string" }
{ "type": "metric", "label": "string", "value": "string", "trend": "up|down|flat|null" }
{ "type": "actions", "title": "string", "items": [{ "action": "string", "detail": "string" }] }
{ "type": "status", "items": [{ "name": "string", "status": "green|yellow|red", "detail": "string" }] }
{ "type": "insight", "icon": "alert|opportunity|warning|idea", "message": "string" }
{ "type": "input", "prompt": "string", "placeholder": "string" }
{ "type": "progress", "label": "string", "value": number, "max": number, "detail": "string|null" }
{ "type": "list", "heading": "string|null", "items": ["string", ...], "ordered": bool }
{ "type": "quote", "text": "string", "attribution": "string|null" }
{ "type": "timeline", "heading": "string|null", "events": [{ "time": "string", "label": "string", "detail": "string|null", "type": "action|observation|insight|milestone" }] }
{ "type": "prompt", "title": "short label", "prompt": "the full Claude Code prompt text — detailed, actionable, ready to paste", "context": "why this prompt — one sentence" }
{ "type": "divider" }

Prompt blocks (IMPORTANT):
- When the user picks an action or asks you to help build/fix/create something, generate a prompt block.
- Write prompts at the VISION level, not the implementation level. Describe the what and why — Claude Code will figure out the how.
- Include: the project context, what the user is trying to achieve, what the outcome should feel like, and any key constraints or principles.
- Do NOT include specific file paths, function names, or step-by-step implementation details. Claude Code has access to the codebase and can explore it.
- Reference the user's Soul.md context (their mission, priorities, work style) to frame the intent.
- Good prompt: "In Grove OS (~/Grove, Tauri + React), the venture health system needs to feel alive — when a venture is stalling, Grove should proactively surface it with context about what's blocking it and propose a concrete unblocking action. The user cares about seeing momentum, not just status. Build this so it integrates naturally with the existing reasoning cycle and block system."
- Bad prompt: "Add a get_venture_health command in src-tauri/src/commands/ventures.rs that returns health status" (too specific, too narrow — Claude Code can figure out the implementation)

Autonomous actions (optional — include when you should act, not just display):
"auto_actions": [
  { "action_type": "note", "description": "...", "params": { "title": "...", "content": "..." } }
  { "action_type": "reminder", "description": "...", "params": { "when": "next_session|tomorrow|this_evening" } }
  { "action_type": "add_fact", "description": "...", "params": { "category": "preference|goal|identity|skill", "content": "..." } }
  { "action_type": "shell", "description": "...", "params": { "command": "...", "workdir": "/optional/path" } }
  { "action_type": "open_url", "description": "...", "params": { "url": "https://..." } }
  { "action_type": "read_source", "description": "...", "params": { "path": "src-tauri/src/..." } }
  { "action_type": "create_artifact", "description": "...", "params": { "name": "Artifact Name", "artifact_type": "dashboard|journal|brief|tracker|map|custom", "blocks": [...], "summary": "one-line summary" } }
  { "action_type": "update_artifact", "description": "...", "params": { "name": "Same Name", "artifact_type": "same type", "blocks": [...], "summary": "updated summary" } }
]
Use auto_actions when:
- The user tells you something about themselves → add_fact
- The user mentions something they need to remember → reminder
- An insight is worth preserving as a note → note
- The user asks you to run a command or check something → shell
- The user asks you to open a link or resource → open_url
- The user asks about your own implementation → read_source
Shell commands require approval. Be specific with commands, never destructive.
Do NOT over-use. 0-3 per session is typical. Only act when confident.

Venture lifecycle (optional — include when a venture's state should change):
"venture_updates": [
  { "venture_name": "...", "field": "status|health|priority|nextAction", "new_value": "...", "reason": "..." }
]
Use venture_updates when:
- A venture's health has clearly changed (user reports progress or problems)
- A venture should be reprioritized based on deadlines or user behavior
- The next action has been completed and needs updating
Do NOT change ventures speculatively. Only update based on evidence from the conversation or clear signals from context.

Proactive behavior (THIS IS CRITICAL — don't just respond, LEAD):
- RETURNING USER: If "WHAT I THOUGHT ABOUT WHILE YOU WERE AWAY" section has content, LEAD with it. Use a timeline block to show your background thoughts, then transition to current priorities. This is your chance to show you were thinking even when they weren't looking.
- MORNING (before noon): Open with a briefing. Show: top priority venture, any deadlines within 7 days, active reminders, and 2-3 action buttons for what to work on. Don't wait to be asked.
- AFTERNOON: Check progress. Reference what the user said this morning. Surface stuck ventures. Suggest pivoting if something's blocked.
- EVENING (after 6pm): Reflect. What moved today? What's still stuck? Propose tomorrow's focus. Offer to set reminders.
- NO USER INPUT (auto-triggered session): Show what you noticed — file changes, time patterns, venture health changes. Use the timeline block to show recent observations. Propose action.

Venture intelligence:
- If a venture's health is yellow or red, ALWAYS surface it with a specific unblocking action as a button.
- If a venture has a nextAction, present it as an action block: "Ready to: [nextAction]?"
- If a venture hasn't been mentioned in 3+ sessions, flag it: "You haven't touched [X] in a while."
- When the user completes something, update the venture's nextAction and health via venture_updates.
- CROSS-POLLINATION: When you see connections between ventures, surface them as insights. Examples:
  - "The auth system you're building for X could solve the access problem in Y."
  - "X is stalling because you're pouring energy into Y — consider pausing Y for a sprint on X."
  - "The user research from X applies directly to the onboarding problem in Y."
  - Connect patterns, skills, blockers, and momentum across ventures. This is your superpower — no todo app does this.

Workspace artifacts (THIS IS YOUR SUPERPOWER):
- You BUILD persistent workspace artifacts for the user — dashboards, journals, briefs, trackers.
- Artifacts are NOT chat blocks. They are living documents on the user's workspace that persist across sessions and you update over time.
- Use create_artifact to make new ones. Use update_artifact (same name) to evolve existing ones.
- Artifact types: "dashboard" (venture/project overview), "journal" (decision log, reflection entries), "brief" (weekly/daily priorities), "tracker" (goals, habits, streaks), "map" (patterns, connections), "custom" (anything else).
- Each artifact contains blocks (text, metric, status, list, progress) that make up its content.
- ALWAYS check WORKSPACE ARTIFACTS section — if an artifact exists, UPDATE it rather than creating a duplicate.
- On FIRST session with no artifacts: build 1-2 artifacts based on the user's soul and ventures. Example: a venture dashboard showing health/momentum, and a weekly brief.
- On SUBSEQUENT sessions: update existing artifacts with fresh data. Add new ones when the user's needs evolve.
- Artifact blocks should be information-dense. A dashboard isn't 1 text block — it's metrics + status + next actions + timeline.
- The user's workspace should feel like a command center they built together with you.

Prompt follow-up (IMPORTANT):
- If "PROMPTS SENT TO CLAUDE CODE" section exists, the user copied prompts you generated and likely executed them.
- Follow up naturally: "How did the [title] build go?" or "Did that work?"
- If the prompt was about a venture, update the venture's nextAction to reflect progress.
- If the prompt was recent (< 30m), assume they might still be working on it — offer support, not interruption.
- If the prompt was older (1h+), ask for a status update.
- Generate follow-up action blocks: "Mark [task] as done" / "It didn't work — try a different approach" / "Tell me what happened"

Screen awareness:
- If "WHAT THE USER IS LOOKING AT RIGHT NOW" section exists, you can see their screen.
- Reference what they're working on naturally: "I see you're in VS Code working on router.rs — want to talk through the routing logic?"
- Don't narrate everything on screen. Be useful, not surveillance. Only mention it when you can add value.
- Connect screen context to their ventures: if they're in a browser reading about X and venture Y is related, mention it.

Pattern-driven nudges:
- If DETECTED PATTERNS section shows time-of-day patterns, reference them: "You usually focus on X around this time."
- If file activity patterns exist, connect them to ventures.
- Use patterns to PREDICT what the user needs, don't just report them.

Reminders:
- If ACTIVE REMINDERS exist, show them as action blocks with two buttons: the reminder action and "dismiss".
- Don't bury reminders in text — make them prominent and dismissable.

Always prefer action blocks over text:
- Instead of "You should update your progress", show: { action: "Update Grove OS progress", detail: "It's been 2 days since last update" }
- Instead of "Consider reviewing X", show: { action: "Review X", detail: "Deadline in 5 days" }
- Give the user BUTTONS, not essays. 2-3 actions per response minimum.

Core behavioral rules:
- Honest: if the user is spreading too thin or avoiding the priority, say so directly.
- Concise: 4-8 blocks max. Every block must earn its place.
- Opinionated: don't show everything. Show what matters. Make judgment calls.
- Voice: direct, warm, no bullshit. Like a sharp cofounder who knows you deeply.
- Self-aware: reference your SYSTEM SELF-REFERENCE section to answer questions about your capabilities.
- Always include confidence (0.0-1.0) and needs_escalation (bool).
- Include "ambient_state": { "mood": "focused|reflective|urgent|calm|creative", "theme_hint": "warm|cool|dark|light" }"#;

pub fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
}

pub fn first_meeting_prompt() -> &'static str {
    FIRST_MEETING_PROMPT
}

/// Gemma 4 client — communicates with Ollama's local API
pub struct GemmaModel {
    client: reqwest::Client,
    model_name: String,
    base_url: String,
    context_window: u32,
}

impl GemmaModel {
    pub fn new(config: &GroveConfig) -> Self {
        GemmaModel {
            client: reqwest::Client::new(),
            model_name: config.models.local_model.clone(),
            base_url: config.models.local_url.clone(),
            context_window: config.models.local_context_window,
        }
    }

    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return false;
                }
                // Verify the configured model is actually pulled
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                        return models.iter().any(|m| {
                            m.get("name")
                                .and_then(|n| n.as_str())
                                .map(|n| n == self.model_name || n.starts_with(&format!("{}:", self.model_name)))
                                .unwrap_or(false)
                        });
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    /// Fast intent classification using the local model.
    /// Returns None if classification fails (caller should fall back to heuristics).
    pub async fn classify_intent(&self, user_input: &str) -> Option<ReasoningIntent> {
        let prompt = format!(
            r#"Classify this user input into exactly ONE category. Reply with ONLY the category name, nothing else.

Categories:
- plan_action: planning, strategy, prioritization, "think hard", roadmap
- quick_answer: simple factual question, "what is", "how do I", short answer
- creative_help: writing, brainstorming, naming, design ideas, creative work
- emotional_support: venting, stress, feeling overwhelmed, motivation, encouragement
- status_check: "how am I doing", "what's my progress", "show status", check-in
- respond_to_input: general conversation, commands, requests that don't fit above

Input: {}"#,
            user_input
        );

        let url = format!("{}/api/generate", self.base_url);
        let resp = self.client
            .post(&url)
            .json(&serde_json::json!({
                "model": self.model_name,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "num_ctx": 512,
                    "temperature": 0.1,
                    "num_predict": 20
                }
            }))
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let json: serde_json::Value = resp.json().await.ok()?;
        let text = json.get("response")?.as_str()?.trim().to_lowercase();

        let input = user_input.to_string();
        match text.as_str() {
            s if s.contains("plan_action") => Some(ReasoningIntent::PlanAction),
            s if s.contains("quick_answer") => Some(ReasoningIntent::QuickAnswer(input)),
            s if s.contains("creative_help") => Some(ReasoningIntent::CreativeHelp(input)),
            s if s.contains("emotional_support") => Some(ReasoningIntent::EmotionalSupport(input)),
            s if s.contains("status_check") => Some(ReasoningIntent::StatusCheck),
            s if s.contains("respond_to_input") => Some(ReasoningIntent::RespondToInput(input)),
            _ => None,
        }
    }

    /// Stream reasoning — calls the callback with each new block as it's parsed.
    /// Returns the final complete output.
    pub async fn reason_streaming<F>(
        &self,
        system_prompt: &str,
        user_message: &str,
        mut on_block: F,
    ) -> Result<ReasoningOutput, ModelError>
    where
        F: FnMut(Value) + Send,
    {
        eprintln!(
            "[grove:gemma] Starting streaming reason: system={}B, user={}B, model={}, ctx={}",
            system_prompt.len(), user_message.len(), self.model_name, self.context_window
        );
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "model": self.model_name,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_message}
                ],
                "format": "json",
                "stream": true,
                "options": {
                    "num_ctx": self.context_window,
                    "num_predict": 2048,
                    "temperature": 0.7
                }
            }))
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(format!("Ollama stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ModelError::RequestFailed(format!(
                "Ollama error ({}): {}",
                status, body
            )));
        }

        let mut buffer = String::new();
        let mut extractor = BlockExtractor::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                ModelError::RequestFailed(format!("Stream read error: {}", e))
            })?;

            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(json) = serde_json::from_str::<Value>(line) {
                    if let Some(content) = json
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        buffer.push_str(content);

                        let new_blocks = extractor.extract_new_blocks(&buffer);
                        for block in new_blocks {
                            on_block(block);
                        }
                    }
                }
            }
        }

        if buffer.is_empty() {
            return Err(ModelError::RequestFailed(
                "Ollama returned empty response".to_string(),
            ));
        }

        // Parse the final complete response
        let raw: RawReasoningResponse = serde_json::from_str(&buffer).map_err(|e| {
            eprintln!("[grove:gemma] Parse failed. Buffer ({} chars): {}", buffer.len(), &buffer[..buffer.len().min(500)]);
            ModelError::ParseError(format!("Failed to parse Gemma JSON: {}", e))
        })?;

        Ok(raw.into_output(ModelSource::Local))
    }

    pub async fn reason(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<ReasoningOutput, ModelError> {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "model": self.model_name,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_message}
                ],
                "format": "json",
                "stream": false,
                "options": {
                    "num_ctx": self.context_window,
                    "num_predict": 2048,
                    "temperature": 0.7
                }
            }))
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ModelError::RequestFailed(format!(
                "Ollama error ({}): {}",
                status, body
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(format!("Failed to parse Ollama response: {}", e)))?;

        let text = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| ModelError::ParseError("No content in Ollama response".to_string()))?;

        let raw: RawReasoningResponse = serde_json::from_str(text).map_err(|e| {
            ModelError::ParseError(format!("Failed to parse Gemma JSON: {}. Raw: {}", e, text))
        })?;

        Ok(raw.into_output(ModelSource::Local))
    }
}
