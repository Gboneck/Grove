use super::config::GroveConfig;
use super::{ModelError, ModelSource, RawReasoningResponse, ReasoningOutput};
use std::fs;

/// Claude API client — escalation layer for complex reasoning
pub struct ClaudeModel {
    client: reqwest::Client,
    model_name: String,
    api_key: Option<String>,
}

impl ClaudeModel {
    pub fn new(config: &GroveConfig) -> Self {
        let api_key = Self::resolve_api_key(&config.models.cloud_api_key_env);
        ClaudeModel {
            client: reqwest::Client::new(),
            model_name: config.models.cloud_model.clone(),
            api_key,
        }
    }

    fn resolve_api_key(env_var_name: &str) -> Option<String> {
        // Try env var first
        if let Ok(key) = std::env::var(env_var_name) {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Try ~/.grove/.env
        let env_path = dirs::home_dir()?.join(".grove").join(".env");
        if let Ok(content) = fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix(&format!("{}=", env_var_name)) {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }

        None
    }

    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn reason(
        &self,
        system_prompt: &str,
        user_message: &str,
        escalated: bool,
    ) -> Result<ReasoningOutput, ModelError> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| ModelError::Unavailable("ANTHROPIC_API_KEY not set".to_string()))?;

        let full_message = if escalated {
            format!(
                "{}\n\n[ESCALATED FROM LOCAL MODEL — apply deeper reasoning]",
                user_message
            )
        } else {
            user_message.to_string()
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": self.model_name,
                "max_tokens": 4096,
                "system": system_prompt,
                "messages": [{"role": "user", "content": full_message}]
            }))
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(format!("Claude API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ModelError::RequestFailed(format!(
                "Claude API error ({}): {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        struct ApiResponse {
            content: Vec<ContentBlock>,
        }
        #[derive(serde::Deserialize)]
        struct ContentBlock {
            text: Option<String>,
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| ModelError::ParseError(format!("Failed to parse Claude response: {}", e)))?;

        let text = api_response
            .content
            .first()
            .and_then(|c| c.text.as_ref())
            .ok_or_else(|| ModelError::ParseError("Empty response from Claude".to_string()))?;

        let raw: RawReasoningResponse = serde_json::from_str(text).map_err(|e| {
            ModelError::ParseError(format!(
                "Failed to parse Claude JSON: {}. Raw: {}",
                e, text
            ))
        })?;

        Ok(raw.into_output(ModelSource::Cloud))
    }
}
