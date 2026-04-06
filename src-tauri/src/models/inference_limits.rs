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
    pub fn validate_prompt(&self, prompt: &str) -> Result<(), String> {
        if prompt.len() > self.max_prompt_length {
            return Err(format!("Prompt exceeds maximum length of {}", self.max_prompt_length));
        }
        Ok(())
    }

    pub fn clamp_tokens(&self, requested: u32) -> u32 {
        requested.min(self.max_tokens)
    }
}
