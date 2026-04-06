use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalClass {
    Health,
    Load,
    Trust,
    Capacity,
    Placement,
    Anomaly,
    SessionRuntime,
}
