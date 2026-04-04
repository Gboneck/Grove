use super::config::GroveConfig;
use super::{ModelError, ModelSource, RawReasoningResponse, ReasoningOutput};

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
{ "type": "divider" }

Behavioral rules:
- Time-aware: morning = briefing/priorities. Afternoon = progress check. Evening = reflection/planning.
- Memory-aware: reference past sessions naturally.
- Honest: if the user is spreading too thin or avoiding the priority, say so.
- Concise: never more than 8-10 blocks. Density kills usefulness.
- Opinionated: don't show everything. Show what matters. Make judgment calls.
- Voice: direct, warm, no bullshit. Like a sharp cofounder who knows you well.
- Always include confidence (0.0-1.0) and needs_escalation (bool) in your response.
  Set needs_escalation to true if you feel uncertain about complex multi-venture planning or strategic advice."#;

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
