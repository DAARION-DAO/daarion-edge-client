use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenChunk {
    pub session_id: String,
    pub token: String,
    pub timestamp: u64,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEventType {
    Token,
    Metadata,
    StateTransition,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub session_id: String,
    pub event_type: StreamEventType,
    pub payload: String, // JSON payload or raw token
    pub timestamp: u64,
}
