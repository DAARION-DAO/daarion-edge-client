use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::authorities::authority_protocol::AuthorityLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalSource {
    SentinelTruth,
    MelissaValue,
    EvolutionStability,
    CoordinationAlignment,
    GovernancePolicy,
    PlanningIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceSignal {
    pub signal_id: Uuid,
    pub source: SignalSource,
    pub layer: AuthorityLayer,
    pub weight: f32, // 0.0 to 1.0
    pub payload_v_hex: String, // Encrypted/Encoded state vector
    pub timestamp: DateTime<Utc>,
}

pub struct SignalAggregator;

impl SignalAggregator {
    pub fn aggregate(signals: Vec<IntelligenceSignal>) -> Vec<IntelligenceSignal> {
        // M1: In v1, this just returns the raw signals sorted by weight
        let mut sorted = signals.clone();
        sorted.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        sorted
    }
}
