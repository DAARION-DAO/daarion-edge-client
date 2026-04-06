use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::signal_class::SignalClass;
use super::signal_confidence::SignalConfidence;
use super::evidence_freshness::EvidenceFreshness;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelSignal {
    pub signal_id: Uuid,
    pub source_ref: String,
    pub signal_class: SignalClass,
    pub raw_value: String,
    pub normalized_value: f32, // 0.0 to 1.0
    pub confidence: SignalConfidence,
    pub freshness: EvidenceFreshness,
    pub evidence_basis_ref: Option<Uuid>,
    pub trust_scope: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct SentinelStandardizer;

impl SentinelStandardizer {
    pub fn normalize(raw_telemetry: String, class: SignalClass, source: String) -> SentinelSignal {
        // Simplified M1 normalization logic
        let normalized = match class {
            SignalClass::Load => raw_telemetry.parse::<f32>().unwrap_or(0.0) / 100.0,
            SignalClass::Health => if raw_telemetry == "healthy" { 1.0 } else { 0.0 },
            _ => 0.5,
        };

        SentinelSignal {
            signal_id: Uuid::new_v4(),
            source_ref: source,
            signal_class: class,
            raw_value: raw_telemetry,
            normalized_value: normalized,
            confidence: SignalConfidence::default(),
            freshness: EvidenceFreshness {
                last_updated: Utc::now(),
                ttl_seconds: 60,
            },
            evidence_basis_ref: Some(Uuid::new_v4()),
            trust_scope: None,
            created_at: Utc::now(),
        }
    }
}
