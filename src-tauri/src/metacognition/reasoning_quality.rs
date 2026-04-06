use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningQualityScore {
    pub coherence_index: f32, // 0.0 to 1.0
    pub logical_consistency: f32,
    pub grounding_score: f32, // Based on SENTINEL evidence
    pub trustworthiness_grade: char, // 'A', 'B', 'C', 'D', 'F'
}

impl ReasoningQualityScore {
    pub fn compute_grade(&self) -> char {
        let avg = (self.coherence_index + self.logical_consistency + self.grounding_score) / 3.0;
        if avg > 0.9 { 'A' }
        else if avg > 0.75 { 'B' }
        else if avg > 0.6 { 'C' }
        else if avg > 0.4 { 'D' }
        else { 'F' }
    }
}
