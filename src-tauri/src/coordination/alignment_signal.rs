use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlignmentClass {
    Specialization,
    ResourceBalance,
    ValueFlow,
    EvolutionInertia,
    StructuralCoherence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentSignal {
    pub class: AlignmentClass,
    pub source_id: String, // District, Region, or Global
    pub vector_magnitude: f32, // 0.0 to 1.0
    pub direction_hint: String, // Description of alignment goal
    pub confidence: f32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDirectionVector {
    pub primary_goal: String,
    pub focus_areas: Vec<AlignmentClass>,
    pub intensity: f32,
    pub active_constraints: Vec<String>,
}
