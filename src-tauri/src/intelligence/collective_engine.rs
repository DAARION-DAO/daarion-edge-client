use crate::intelligence::signal_aggregator::{SignalAggregator, IntelligenceSignal};
use crate::intelligence::reasoning_model::{ReasoningModel, CollectiveInsight, ReasoningTrace};
use crate::intelligence::strategy_vector::{StrategicDirective, SystemStateVector};
use tauri::{AppHandle, Emitter};

pub struct CollectiveEngine;

impl CollectiveEngine {
    pub async fn process_intelligence_cycle(app: AppHandle, signals: Vec<IntelligenceSignal>) -> Result<CollectiveInsight, String> {
        // 1. Aggregate
        let top_signals = SignalAggregator::aggregate(signals);
        
        // 2. Synthesize (M1 Simulation)
        let summary = if top_signals.is_empty() {
            "System state is quiescent. Nominal background synthesis active."
        } else {
            "High-priority cross-layer signals detected. Synthesizing strategic direction."
        };

        let insight = ReasoningModel::generate_insight(summary, 0.85);

        // 3. Meta-Cognitive Reflection (Slice RR)
        let _judgment = crate::metacognition::MetaCognitionEngine::reflect(app.clone(), &insight).await?;

        // 4. Emit Insight Pulse
        app.emit("intelligence-insight-pulse", &insight).unwrap();

        Ok(insight)
    }

    pub fn generate_strategic_directive(insight: &CollectiveInsight) -> StrategicDirective {
        use uuid::Uuid;
        StrategicDirective {
            directive_id: Uuid::new_v4(),
            vector_target: SystemStateVector {
                stability: 0.9,
                value_velocity: 1.1,
                alignment_index: 0.8,
                trust_density: 0.95,
            },
            suggestion: format!("Focus on: {}", insight.summary),
            priority: 5,
        }
    }
}
