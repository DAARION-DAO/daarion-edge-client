use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryNormalizationRule {
    pub rule_id: Uuid,
    pub signal_type: String,
    pub input_range: (f32, f32),
    pub normalized_range: (f32, f32),
    pub normalization_method: String, // e.g., "Linear", "Logarithmic"
}
