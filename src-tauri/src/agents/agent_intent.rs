use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIntent {
    pub agent_id: String,
    pub session_id: String,
    pub intent_id: String,
    pub task_type: String, // e.g. "completions", "search", "vision"
    pub payload: String,   // Raw task payload
    pub model_preference: String,
    pub latency_class: String, // e.g. "interactive", "batch"
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionLeaseRef {
    pub lease_id: String,
    pub agent_id: String,
    pub node_id: String,
    pub valid_until: u64,
}
