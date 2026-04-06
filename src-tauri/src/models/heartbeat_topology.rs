use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeartbeatClass {
    Presence,     // Small, fast (30s)
    Metadata,     // Large, slow (6h or on change)
    Anomaly,      // Critical, immediate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceHeartbeat {
    pub node_id: String,
    pub ts: u64,
    pub district: String,
    pub state: String, // "alive", "degraded", "maintenance"
    pub sequence_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub vram_gb: f32,
    pub ram_gb: f32,
    pub cpu_cores: u16,
    pub supported_runtimes: Vec<String>,
    pub model_inventory_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataHeartbeat {
    pub node_id: String,
    pub ts: u64,
    pub district: String,
    pub region: String,
    pub tier: String, // T1, T2, T3
    pub specialization: String,
    pub capabilities: NodeCapabilities,
    pub models: Vec<String>, // List of local model IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyMetrics {
    pub cpu_load: f32,
    pub memory_pressure: f32,
    pub queue_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyHeartbeat {
    pub node_id: String,
    pub ts: u64,
    pub district: String,
    pub anomaly_code: String,
    pub severity: String, // "warning", "critical", "fatal"
    pub message: String,
    pub metrics: AnomalyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorSnapshot {
    pub collector_id: String,
    pub district: String,
    pub ts: u64,
    pub healthy_count: u32,
    pub degraded_count: u32,
    pub offline_suspected_count: u32,
    pub avg_cpu: f32,
    pub avg_memory_pressure: f32,
    pub total_queue_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryPublishEnvelope {
    pub collector_id: String,
    pub payload: CollectorSnapshot,
    pub signature: Option<Vec<u8>>,
}
