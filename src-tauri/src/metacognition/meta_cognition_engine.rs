use crate::intelligence::reasoning_model::CollectiveInsight;
use crate::metacognition::reasoning_quality::ReasoningQualityScore;
use crate::metacognition::meta_risk::{MetaRiskProfile, MetaRiskClass};
use crate::metacognition::reflection_gate::MetaCognitiveDecision;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitiveJudgment {
    pub judgment_id: Uuid,
    pub source_insight_id: Uuid,
    pub quality: ReasoningQualityScore,
    pub risk: MetaRiskProfile,
    pub decision: MetaCognitiveDecision,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct MetaCognitionEngine;

impl MetaCognitionEngine {
    pub async fn reflect(app: AppHandle, insight: &CollectiveInsight) -> Result<MetaCognitiveJudgment, String> {
        // M1: Simulated meta-cognitive evaluation
        let quality = ReasoningQualityScore {
            coherence_index: 0.88,
            logical_consistency: 0.92,
            grounding_score: insight.confidence,
            trustworthiness_grade: 'A',
        };
        
        let risk = MetaRiskProfile {
            risk_class: MetaRiskClass::None,
            risk_score: 0.1,
            evidence_sufficiency: 0.85,
            authority_balance: 0.9,
        };

        let decision = if quality.grounding_score < 0.5 {
            MetaCognitiveDecision::RequireReview
        } else if risk.risk_score > 0.7 {
            MetaCognitiveDecision::Escalate
        } else {
            MetaCognitiveDecision::Continue
        };

        let judgment = MetaCognitiveJudgment {
            judgment_id: Uuid::new_v4(),
            source_insight_id: insight.insight_id,
            quality,
            risk,
            decision,
            created_at: Utc::now(),
        };

        // Emit Meta-Cognition Pulse
        app.emit("metacognition-judgment-pulse", &judgment).unwrap();

        Ok(judgment)
    }
}
