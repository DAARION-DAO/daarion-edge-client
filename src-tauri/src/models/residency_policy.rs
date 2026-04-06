use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ResidencyDecision {
    KeepWarm,
    CoolDown,
    Evict,
    DoNotPreload,
}

pub struct ResidencyPolicy;

impl ResidencyPolicy {
    pub fn decide(score: f32, current_state: &str) -> ResidencyDecision {
        if score > 40.0 {
            ResidencyDecision::KeepWarm
        } else if score > 20.0 {
            ResidencyDecision::CoolDown
        } else if score > 10.0 {
            if current_state == "Warm" || current_state == "Loaded" {
                 ResidencyDecision::CoolDown
            } else {
                 ResidencyDecision::DoNotPreload
            }
        } else {
            ResidencyDecision::Evict
        }
    }

    pub fn should_evict_under_pressure(score: f32, is_specialized: bool) -> bool {
        // Under extreme pressure, we evict anything not specialized with score < 50
        if !is_specialized && score < 50.0 {
            return true;
        }
        score < 10.0
    }
}
