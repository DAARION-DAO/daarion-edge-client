use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub intent_id: String,
    pub session_id: String,
    pub result_payload: String,
    pub metrics: AgentExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionMetrics {
    pub duration_ms: u64,
    pub tokens_generated: u32,
    pub cost_estimate: f32,
}

pub struct AgentResultBinder;

impl AgentResultBinder {
    pub fn bind_stream_to_session(session_id: &str, result_stream_ref: &str) -> String {
        format!("Binding stream {} to agent session {}", result_stream_ref, session_id)
    }
}
