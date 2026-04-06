pub mod alignment_signal;
pub mod divergence_detector;
pub mod coordination_policy;
pub mod coordination_engine;

pub use alignment_signal::{AlignmentSignal, AlignmentClass, GlobalDirectionVector};
pub use divergence_detector::{DivergenceMetric, DivergenceType};
pub use coordination_policy::{CoordinationDecision, CoordinationScope, CoordinationPolicy};
pub use coordination_engine::{CoordinationEngine, CoordinationState};
