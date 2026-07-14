use crate::inference::types::{ChatMessage, InferenceError, InferenceEvent};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHealth {
    pub available: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledProviderModel {
    pub provider_model_id: String,
}

#[derive(Debug, Clone)]
pub struct ProviderChatRequest {
    pub provider_model_id: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub trait EventSink: Send + Sync {
    fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError>;
}

struct EventGateState {
    terminal: bool,
}

pub(crate) struct EventGate {
    state: Mutex<EventGateState>,
    sink: Arc<dyn EventSink>,
}

impl EventGate {
    pub(crate) fn new(sink: Arc<dyn EventSink>) -> Self {
        Self {
            state: Mutex::new(EventGateState { terminal: false }),
            sink,
        }
    }

    pub(crate) fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError> {
        let mut state = self.state.lock().map_err(|_| {
            InferenceError::Internal("Inference event gate is unavailable".to_string())
        })?;
        if state.terminal {
            return Ok(());
        }
        if event.is_terminal() {
            state.terminal = true;
        }
        self.sink.emit(event)
    }
}

#[derive(Clone)]
pub struct OperationControl {
    cancellation: watch::Receiver<bool>,
}

impl OperationControl {
    pub(crate) fn new(cancellation: watch::Receiver<bool>) -> Self {
        Self { cancellation }
    }

    pub fn ensure_active(&self) -> Result<(), InferenceError> {
        if *self.cancellation.borrow() {
            Err(InferenceError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub async fn cancelled(&mut self) {
        loop {
            if *self.cancellation.borrow() {
                return;
            }
            if self.cancellation.changed().await.is_err() {
                std::future::pending::<()>().await;
            }
        }
    }
}

#[derive(Clone)]
pub struct RequestControl {
    request_id: String,
    operation: OperationControl,
    gate: Arc<EventGate>,
}

impl RequestControl {
    pub(crate) fn new(
        request_id: String,
        cancellation: watch::Receiver<bool>,
        gate: Arc<EventGate>,
    ) -> Self {
        Self {
            request_id,
            operation: OperationControl::new(cancellation),
            gate,
        }
    }

    pub fn ensure_active(&self) -> Result<(), InferenceError> {
        self.operation.ensure_active()
    }

    pub fn emit_token(&self, content: String) -> Result<(), InferenceError> {
        self.ensure_active()?;
        self.gate.emit(InferenceEvent::Token {
            request_id: self.request_id.clone(),
            content,
        })
    }
}

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn endpoint(&self) -> String;

    async fn health(&self) -> Result<ProviderHealth, InferenceError>;
    async fn list_installed_models(&self) -> Result<Vec<InstalledProviderModel>, InferenceError>;
    async fn prepare_model(
        &self,
        provider_model_id: &str,
        control: OperationControl,
    ) -> Result<(), InferenceError>;
    async fn chat(
        &self,
        request: ProviderChatRequest,
        control: RequestControl,
    ) -> Result<String, InferenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<InferenceEvent>>);

    impl EventSink for RecordingSink {
        fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
    }

    #[test]
    fn terminal_gate_suppresses_late_tokens_and_completion() {
        let sink = Arc::new(RecordingSink::default());
        let gate = EventGate::new(sink.clone());
        gate.emit(InferenceEvent::Cancelled {
            request_id: "request".to_string(),
        })
        .unwrap();
        gate.emit(InferenceEvent::Token {
            request_id: "request".to_string(),
            content: "late".to_string(),
        })
        .unwrap();
        gate.emit(InferenceEvent::Completed {
            request_id: "request".to_string(),
        })
        .unwrap();

        let events = sink.0.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], InferenceEvent::Cancelled { .. }));
    }
}
