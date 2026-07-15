use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferencePolicy {
    LocalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceRequest {
    pub request_id: String,
    pub canonical_model_id: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceResponse {
    pub request_id: String,
    pub canonical_model_id: String,
    pub provider_id: String,
    pub latency_ms: u64,
    pub output_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceStatus {
    pub execution_policy: InferencePolicy,
    pub provider_id: String,
    pub endpoint: String,
    pub available: bool,
    pub local_only_verified: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceModelSummary {
    pub canonical_model_id: String,
    pub family: String,
    pub tier: String,
    pub provider_id: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrepareLocalModelRequest {
    pub request_id: String,
    pub canonical_model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelPreparationResponse {
    pub request_id: String,
    pub canonical_model_id: String,
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InferenceEvent {
    Started {
        request_id: String,
    },
    Running {
        request_id: String,
    },
    Token {
        request_id: String,
        content: String,
    },
    Completed {
        request_id: String,
    },
    Failed {
        request_id: String,
        code: String,
        error: String,
    },
    Cancelled {
        request_id: String,
    },
    TimedOut {
        request_id: String,
    },
}

impl InferenceEvent {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. }
                | Self::Failed { .. }
                | Self::Cancelled { .. }
                | Self::TimedOut { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    InvalidRequest(String),
    PolicyViolation(String),
    UnknownModel(String),
    ProviderUnavailable,
    LocalOnlyNotEnforced,
    ProviderCapabilityUnsupported,
    ModelNotLocal,
    LocalModelUnverified,
    ProviderProtocol(String),
    Cancelled,
    TimedOut,
    DuplicateRequest,
    EventDelivery,
    Internal(String),
}

impl InferenceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::PolicyViolation(_) => "policy_violation",
            Self::UnknownModel(_) => "unknown_model",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::LocalOnlyNotEnforced => "local_only_not_enforced",
            Self::ProviderCapabilityUnsupported => "provider_capability_unsupported",
            Self::ModelNotLocal => "model_not_local",
            Self::LocalModelUnverified => "local_model_unverified",
            Self::ProviderProtocol(_) => "provider_protocol",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::DuplicateRequest => "duplicate_request",
            Self::EventDelivery => "event_delivery",
            Self::Internal(_) => "internal",
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            Self::InvalidRequest(message)
            | Self::PolicyViolation(message)
            | Self::UnknownModel(message)
            | Self::ProviderProtocol(message)
            | Self::Internal(message) => message.clone(),
            Self::ProviderUnavailable => "Local inference provider is unavailable".to_string(),
            Self::LocalOnlyNotEnforced => {
                "Ollama cloud must be disabled before LocalOnly inference is eligible".to_string()
            }
            Self::ProviderCapabilityUnsupported => {
                "Ollama cannot prove the required LocalOnly policy; upgrade or configure Ollama"
                    .to_string()
            }
            Self::ModelNotLocal => {
                "The selected model is not verified as a local Ollama artifact".to_string()
            }
            Self::LocalModelUnverified => {
                "The selected local model could not be verified safely".to_string()
            }
            Self::Cancelled => "Inference request was cancelled".to_string(),
            Self::TimedOut => "Inference request timed out".to_string(),
            Self::DuplicateRequest => {
                "An inference request with this ID is already active".to_string()
            }
            Self::EventDelivery => "Unable to deliver the local inference event".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InferencePublicError {
    pub code: String,
    pub message: String,
}

impl From<InferenceError> for InferencePublicError {
    fn from(error: InferenceError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.public_message(),
        }
    }
}

impl fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.public_message())
    }
}

impl std::error::Error for InferenceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_errors_have_stable_codes_and_controlled_messages() {
        let error = InferencePublicError::from(InferenceError::ProviderProtocol(
            "Controlled provider failure".to_string(),
        ));
        assert_eq!(error.code, "provider_protocol");
        assert_eq!(error.message, "Controlled provider failure");

        let unavailable = InferencePublicError::from(InferenceError::ProviderUnavailable);
        assert_eq!(unavailable.code, "provider_unavailable");
        assert_eq!(
            unavailable.message,
            "Local inference provider is unavailable"
        );

        for (error, code) in [
            (
                InferenceError::LocalOnlyNotEnforced,
                "local_only_not_enforced",
            ),
            (
                InferenceError::ProviderCapabilityUnsupported,
                "provider_capability_unsupported",
            ),
            (InferenceError::ModelNotLocal, "model_not_local"),
            (
                InferenceError::LocalModelUnverified,
                "local_model_unverified",
            ),
        ] {
            let public = InferencePublicError::from(error);
            assert_eq!(public.code, code);
            assert!(!public.message.contains("http"));
            assert!(!public.message.contains("remote_host"));
        }
    }
}
