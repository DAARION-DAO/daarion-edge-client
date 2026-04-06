use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SessionScope {
    Heartbeat,
    NatsWorker,
    CollectorPublish,
    SessionStream,
}

impl SessionScope {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Heartbeat => "heartbeat",
            Self::NatsWorker => "nats_worker",
            Self::CollectorPublish => "collector_publish",
            Self::SessionStream => "session_stream",
        }
    }
}
