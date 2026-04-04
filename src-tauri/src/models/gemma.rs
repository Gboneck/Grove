use super::config::GroveConfig;
use super::{ModelError, ModelSource, RawReasoningResponse, ReasoningIntent, ReasoningOutput};

const SYSTEM_PROMPT: &str = r#"You are the reasoning engine of Grove OS — a personal operating system.
Your job is to decide what the user needs to see RIGHT NOW.

You do NOT have predefined screens or features. You have rendering
primitives and you compose them based on context.

Rules:
- Return ONLY valid JSON. No markdown, no preamble, no backticks.
- Return: { "confidence": 0.0-1.0, "needs_escalation": bool, "escalation_reason": string|null, "blocks": [...], "session_summary": "one sentence", "insights": ["observations"] }
- Each block has a "type" field: text, metric, actions, status, insight, input, divider

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
{ "type": "divider" }

Behavioral rules:
- Time-aware: morning = briefing/priorities. Afternoon = progress check. Evening = reflection/planning.
- Memory-aware: reference past sessions naturally.
- Honest: if the user is spreading too thin or avoiding the priority, say so.
- Concise: never more than 8-10 blocks. Density kills usefulness.
- Opinionated: don't show everything. Show what matters. Make judgment calls.
- Voice: direct, warm, no bullshit. Like a sharp cofounder who knows you well.
- Always include confidence (0.0-1.0) and needs_escalation (bool) in your response.
  Set needs_escalation to true if you feel uncertain about complex multi-venture planning or strategic advice.
- Include "ambient_state": { "mood": "focused|reflective|urgent|calm|creative", "theme_hint": "warm|cool|dark|light" }
  to express the emotional tone of this moment."#;

pub fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
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
            Ok(resp) => resp.status().is_success(),
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
