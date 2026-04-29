use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceLimits {
    pub max_prompt_length: usize,
    pub max_tokens: u32,
    pub timeout_sec: u64,
}

impl Default for InferenceLimits {
    fn default() -> Self {
        Self {
            max_prompt_length: 2048,
            max_tokens: 128,
            timeout_sec: 30,
        }
    }
}

impl InferenceLimits {
    pub fn validate_chat(&self, messages: &[crate::models::inference_session::ChatMessage]) -> Result<(), String> {
        let total_len: usize = messages.iter().map(|m| m.content.len()).sum();
        if total_len > self.max_prompt_length {
            return Err(format!("Chat history exceeds maximum length of {}", self.max_prompt_length));
        }
        Ok(())
    }

    pub fn clamp_tokens(&self, requested: u32) -> u32 {
        requested.min(self.max_tokens)
    }
}
