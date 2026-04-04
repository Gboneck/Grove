pub mod claude;
pub mod config;
pub mod context;
pub mod gemma;
pub mod router;
pub mod streaming;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which model handled the reasoning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelSource {
    Local,  // Gemma via Ollama
    Cloud,  // Claude API
}

/// The intent behind a reasoning request — determines routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningIntent {
    ComposeUI,
    RespondToInput(String),
    PlanAction,
    Reflect,
    QuickAnswer(String),
    CreativeHelp(String),
    EmotionalSupport(String),
    StatusCheck,
    DualPass(String),
}

impl ReasoningIntent {
    pub fn is_fast_path(&self) -> bool {
        matches!(
            self,
            ReasoningIntent::ComposeUI
                | ReasoningIntent::Reflect
                | ReasoningIntent::QuickAnswer(_)
                | ReasoningIntent::StatusCheck
        )
    }

    pub fn requires_deep_reasoning(&self) -> bool {
        matches!(
            self,
            ReasoningIntent::PlanAction | ReasoningIntent::CreativeHelp(_)
        )
    }

    pub fn requires_dual_pass(&self) -> bool {
        matches!(self, ReasoningIntent::DualPass(_))
    }

    pub fn label(&self) -> &str {
        match self {
            ReasoningIntent::ComposeUI => "compose_ui",
            ReasoningIntent::RespondToInput(_) => "respond_to_input",
            ReasoningIntent::PlanAction => "plan_action",
            ReasoningIntent::Reflect => "reflect",
            ReasoningIntent::QuickAnswer(_) => "quick_answer",
            ReasoningIntent::CreativeHelp(_) => "creative_help",
            ReasoningIntent::EmotionalSupport(_) => "emotional_support",
            ReasoningIntent::StatusCheck => "status_check",
            ReasoningIntent::DualPass(_) => "dual_pass",
        }
    }
}

/// Parsed output from either model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutput {
    pub blocks: Vec<Value>,
    pub confidence: f64,
    pub needs_escalation: bool,
    pub escalation_reason: Option<String>,
    pub session_summary: Option<String>,
    pub insights: Option<Vec<String>>,
    pub source: ModelSource,
    pub ambient_mood: Option<String>,
    pub ambient_theme: Option<String>,
    pub auto_actions: Option<Vec<AutoAction>>,
    pub venture_updates: Option<Vec<VentureUpdate>>,
}

/// Raw JSON shape returned by the reasoning models
#[derive(Debug, Deserialize)]
pub struct RawReasoningResponse {
    pub blocks: Vec<Value>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub needs_escalation: bool,
    #[serde(default)]
    pub escalation_reason: Option<String>,
    #[serde(default)]
    pub session_summary: Option<String>,
    #[serde(default)]
    pub insights: Option<Vec<String>>,
    #[serde(default)]
    pub ambient_state: Option<AmbientState>,
    #[serde(default)]
    pub auto_actions: Option<Vec<AutoAction>>,
    #[serde(default)]
    pub venture_updates: Option<Vec<VentureUpdate>>,
}

/// An action the model wants executed autonomously
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAction {
    pub action_type: String, // "note", "timer", "venture_status", "reminder", "file_write"
    pub description: String,
    #[serde(default)]
    pub params: serde_json::Map<String, Value>,
}

/// A model-suggested change to a venture in context.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VentureUpdate {
    pub venture_name: String,
    pub field: String,       // "status", "health", "priority", "nextAction"
    pub new_value: Value,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmbientState {
    pub mood: Option<String>,
    pub theme_hint: Option<String>,
}

fn default_confidence() -> f64 {
    0.8
}

impl RawReasoningResponse {
    pub fn into_output(self, source: ModelSource) -> ReasoningOutput {
        let (mood, theme) = match self.ambient_state {
            Some(ref a) => (a.mood.clone(), a.theme_hint.clone()),
            None => (None, None),
        };
        ReasoningOutput {
            blocks: self.blocks,
            confidence: self.confidence,
            needs_escalation: self.needs_escalation,
            escalation_reason: self.escalation_reason,
            session_summary: self.session_summary,
            insights: self.insights,
            source,
            ambient_mood: mood,
            ambient_theme: theme,
            auto_actions: self.auto_actions,
            venture_updates: self.venture_updates,
        }
    }
}

#[derive(Debug)]
pub enum ModelError {
    Unavailable(String),
    RequestFailed(String),
    ParseError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::Unavailable(msg) => write!(f, "Model unavailable: {}", msg),
            ModelError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            ModelError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}
