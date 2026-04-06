use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignalConfidence {
    pub score: f32, // 0.0 to 1.0
    pub sample_count: u32,
    pub source_reliability: f32,
}

impl Default for SignalConfidence {
    fn default() -> Self {
        Self {
            score: 1.0,
            sample_count: 1,
            source_reliability: 1.0,
        }
    }
}
