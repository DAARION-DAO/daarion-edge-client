use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SpecializationClass {
    Routing,
    SmallLLM,
    Embedding,
    Vision,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModelRuntimeState {
    Unloaded,
    Cold,
    Loading,
    Warm,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInventoryEntry {
    pub model_id: String,
    pub runtime_class: String, // e.g., "llama.cpp", "onnx"
    pub size_bytes: u64,
    pub state: ModelRuntimeState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecializationProfile {
    pub current_class: SpecializationClass,
    pub active_model_id: Option<String>,
    pub switch_count: u32,
    pub warmth_since: Option<i64>,
}

pub struct SpecializationPolicy;

impl SpecializationPolicy {
    pub fn get_default_profile() -> SpecializationProfile {
        SpecializationProfile {
            current_class: SpecializationClass::Routing,
            active_model_id: None,
            switch_count: 0,
            warmth_since: None,
        }
    }

    /// Evaluates if a job matches the current specialization
    pub fn is_affinity_match(profile: &SpecializationProfile, job_type: &str) -> bool {
        match (profile.current_class, job_type) {
            (SpecializationClass::SmallLLM, "reasoning") => true,
            (SpecializationClass::Embedding, "embedding") => true,
            (SpecializationClass::Vision, "vision") => true,
            (SpecializationClass::Routing, "routing") => true,
            _ => false,
        }
    }
}
