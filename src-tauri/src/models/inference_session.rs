use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InferenceSessionState {
    Queued,
    LoadingModel,
    Running,
    Streaming,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceLimits {
    pub max_stream_duration_sec: u32,
    pub max_tokens: u32,
    pub session_timeout_sec: u32,
}

impl Default for InferenceLimits {
    fn default() -> Self {
        Self {
            max_stream_duration_sec: 300,
            max_tokens: 2048,
            session_timeout_sec: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalInferenceRequest {
    pub request_id: String,
    pub model_id: String,
    pub task_type: String, // e.g. "chat"
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalInferenceResponse {
    pub request_id: String,
    pub status: String,
    pub model_id: String,
    pub runtime: String,
    pub latency_ms: u64,
    pub output_text: String,
}

pub struct InferenceSession {
    pub id: String,
    pub state: InferenceSessionState,
    pub request: LocalInferenceRequest,
}

impl InferenceSession {
    pub fn new(request: LocalInferenceRequest) -> Self {
        Self {
            id: request.request_id.clone(),
            state: InferenceSessionState::Queued,
            request,
        }
    }
}
