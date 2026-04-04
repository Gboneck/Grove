use futures_util::StreamExt;
use serde_json::Value;
use super::config::GroveConfig;
use super::streaming::BlockExtractor;
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

    /// Stream reasoning — calls the callback with each new block as it's parsed.
    pub async fn reason_streaming<F>(
        &self,
        system_prompt: &str,
        user_message: &str,
        escalated: bool,
        mut on_block: F,
    ) -> Result<ReasoningOutput, ModelError>
    where
        F: FnMut(Value) + Send,
    {
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
                "stream": true,
                "system": system_prompt,
                "messages": [{"role": "user", "content": full_message}]
            }))
            .send()
            .await
            .map_err(|e| ModelError::RequestFailed(format!("Claude stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ModelError::RequestFailed(format!(
                "Claude API error ({}): {}",
                status, body
            )));
        }

        let mut buffer = String::new();
        let mut extractor = BlockExtractor::new();
        let mut stream = response.bytes_stream();
        let mut line_buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                ModelError::RequestFailed(format!("Stream read error: {}", e))
            })?;

            let chunk_str = String::from_utf8_lossy(&chunk);
            line_buffer.push_str(&chunk_str);

            // Process complete SSE lines
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].trim().to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }
                    if let Ok(event) = serde_json::from_str::<Value>(data) {
                        // Extract text delta from content_block_delta events
                        if event.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                            if let Some(text) = event
                                .get("delta")
                                .and_then(|d| d.get("text"))
                                .and_then(|t| t.as_str())
                            {
                                buffer.push_str(text);

                                let new_blocks = extractor.extract_new_blocks(&buffer);
                                for block in new_blocks {
                                    on_block(block);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse final complete response
        let raw: RawReasoningResponse = serde_json::from_str(&buffer).map_err(|e| {
            ModelError::ParseError(format!(
                "Failed to parse streamed Claude JSON: {}. Raw: {}",
                e, buffer
            ))
        })?;

        Ok(raw.into_output(ModelSource::Cloud))
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
