use super::claude::ClaudeModel;
use super::config::GroveConfig;
use super::context::GroveContext;
use super::gemma::{self, GemmaModel};
use super::{ModelError, ModelSource, ReasoningIntent, ReasoningOutput};

/// Heuristic fallback for intent classification when model-based classification is unavailable
fn heuristic_classify(input: &str) -> ReasoningIntent {
    let lower = input.to_lowercase();

    // Plan/strategy keywords
    if lower.contains("plan")
        || lower.contains("prioritize")
        || lower.contains("think hard")
        || lower.contains("strategy")
        || lower.contains("roadmap")
        || lower.contains("next steps")
    {
        return ReasoningIntent::PlanAction;
    }

    // Status check patterns
    if lower.contains("how am i")
        || lower.contains("my progress")
        || lower.contains("show status")
        || lower.contains("check in")
        || lower.contains("where do i stand")
    {
        return ReasoningIntent::StatusCheck;
    }

    // Quick factual questions
    if lower.starts_with("what is")
        || lower.starts_with("what's")
        || lower.starts_with("how many")
        || lower.starts_with("when is")
        || lower.starts_with("who is")
        || (lower.contains('?') && lower.len() < 60)
    {
        return ReasoningIntent::QuickAnswer(input.to_string());
    }

    // Emotional support signals
    if lower.contains("overwhelmed")
        || lower.contains("stressed")
        || lower.contains("burned out")
        || lower.contains("frustrated")
        || lower.contains("struggling")
        || lower.contains("anxious")
        || lower.contains("i feel")
        || lower.contains("i'm feeling")
    {
        return ReasoningIntent::EmotionalSupport(input.to_string());
    }

    // Creative work
    if lower.contains("brainstorm")
        || lower.contains("name ideas")
        || lower.contains("write me")
        || lower.contains("help me write")
        || lower.contains("creative")
        || lower.contains("design ideas")
    {
        return ReasoningIntent::CreativeHelp(input.to_string());
    }

    ReasoningIntent::RespondToInput(input.to_string())
}

/// Mode override the user can set from the frontend
#[derive(Debug, Clone, PartialEq)]
pub enum ModelMode {
    Auto,       // Router decides
    LocalOnly,  // Force Gemma, no escalation
    CloudOnly,  // Force Claude
}

pub struct ModelRouter {
    gemma: GemmaModel,
    claude: ClaudeModel,
    config: GroveConfig,
    mode: ModelMode,
}

impl ModelRouter {
    pub fn new(config: GroveConfig) -> Self {
        let gemma = GemmaModel::new(&config);
        let claude = ClaudeModel::new(&config);
        let mode = if config.models.offline_mode {
            ModelMode::LocalOnly
        } else {
            ModelMode::Auto
        };

        ModelRouter {
            gemma,
            claude,
            config,
            mode,
        }
    }

    pub fn set_mode(&mut self, mode: ModelMode) {
        self.mode = mode;
    }

    /// Classify user input into an intent. Tries model-based classification first,
    /// falls back to keyword heuristics if the model is unavailable or fails.
    pub async fn classify_intent(&self, user_input: &str) -> ReasoningIntent {
        // Try model-based classification if Gemma is available
        if self.gemma.is_available().await {
            if let Some(intent) = self.gemma.classify_intent(user_input).await {
                eprintln!("[grove] Intent classified by model: {:?}", intent.label());
                return intent;
            }
        }
        // Fall back to heuristics
        let intent = heuristic_classify(user_input);
        eprintln!("[grove] Intent classified by heuristic: {:?}", intent.label());
        intent
    }

    pub async fn route(
        &self,
        context: &GroveContext,
        intent: &ReasoningIntent,
    ) -> Result<ReasoningOutput, ModelError> {
        let system_prompt = gemma::system_prompt();
        let user_message = context.to_user_message();

        match &self.mode {
            ModelMode::LocalOnly => {
                return self.try_local(system_prompt, &user_message).await;
            }
            ModelMode::CloudOnly => {
                return self.claude.reason(system_prompt, &user_message, false).await;
            }
            ModelMode::Auto => {}
        }

        // Auto routing
        let gemma_available = self.gemma.is_available().await;
        let claude_available = self.claude.is_available();

        // 1. If Claude unavailable, use Gemma
        if !claude_available {
            if gemma_available {
                return self.try_local(system_prompt, &user_message).await;
            }
            return Err(ModelError::Unavailable(
                "No models available. Start Ollama or set ANTHROPIC_API_KEY.".to_string(),
            ));
        }

        // 2. If Gemma unavailable, use Claude
        if !gemma_available {
            return self.claude.reason(system_prompt, &user_message, false).await;
        }

        // 3. Both available — route by intent
        if intent.requires_deep_reasoning() {
            return self.claude.reason(system_prompt, &user_message, false).await;
        }

        // 4. Fast-path or default: try Gemma first
        if intent.is_fast_path() || self.config.models.prefer_local {
            match self.gemma.reason(system_prompt, &user_message).await {
                Ok(result) => {
                    if result.confidence >= self.config.models.confidence_threshold
                        && !result.needs_escalation
                    {
                        return Ok(result);
                    }
                    // Escalate to Claude
                    if self.config.models.escalation_logging {
                        eprintln!(
                            "[grove] Escalating to Claude: confidence={:.2}, needs_escalation={}, reason={:?}",
                            result.confidence, result.needs_escalation, result.escalation_reason
                        );
                    }
                    self.claude.reason(system_prompt, &user_message, true).await
                }
                Err(_) => {
                    // Gemma failed, fall through to Claude
                    self.claude.reason(system_prompt, &user_message, false).await
                }
            }
        } else {
            // Non-fast-path: try Gemma with fallback
            match self.gemma.reason(system_prompt, &user_message).await {
                Ok(result) => {
                    if result.confidence >= self.config.models.confidence_threshold
                        && !result.needs_escalation
                    {
                        Ok(result)
                    } else {
                        self.claude.reason(system_prompt, &user_message, true).await
                    }
                }
                Err(_) => self.claude.reason(system_prompt, &user_message, false).await,
            }
        }
    }

    async fn try_local(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<ReasoningOutput, ModelError> {
        self.gemma.reason(system_prompt, user_message).await
    }

    /// Check which models are currently available
    pub async fn status(&self) -> ModelStatus {
        ModelStatus {
            gemma_available: self.gemma.is_available().await,
            claude_available: self.claude.is_available(),
            mode: match &self.mode {
                ModelMode::Auto => "auto".to_string(),
                ModelMode::LocalOnly => "local_only".to_string(),
                ModelMode::CloudOnly => "cloud_only".to_string(),
            },
            local_model: self.config.models.local_model.clone(),
            cloud_model: self.config.models.cloud_model.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelStatus {
    pub gemma_available: bool,
    pub claude_available: bool,
    pub mode: String,
    pub local_model: String,
    pub cloud_model: String,
}
