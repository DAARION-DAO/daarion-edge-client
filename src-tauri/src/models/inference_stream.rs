use serde::{Deserialize, Serialize};
use crate::models::token_stream::{StreamEvent, StreamEventType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStreamState {
    pub session_id: String,
    pub tokens_emitted: u32,
    pub start_time: u64,
    pub last_token_time: u64,
    pub is_closed: bool,
}

pub struct InferenceStream;

impl InferenceStream {
    pub fn create_event(session_id: &str, event_type: StreamEventType, payload: &str) -> StreamEvent {
        StreamEvent {
            session_id: session_id.to_string(),
            event_type,
            payload: payload.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    pub fn calculate_tps(tokens: u32, duration_ms: u64) -> f32 {
        if duration_ms == 0 { return 0.0; }
        (tokens as f32 / duration_ms as f32) * 1000.0
    }
}
