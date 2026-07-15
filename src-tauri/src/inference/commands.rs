use crate::inference::model_resolver::ModelResolver;
use crate::inference::ollama_provider::OllamaProvider;
use crate::inference::provider::EventSink;
use crate::inference::service::{InferenceService, ServiceLimits};
use crate::inference::types::{
    ChatMessage, InferenceError, InferenceEvent, InferenceModelSummary, InferencePublicError,
    InferenceRequest, InferenceResponse, InferenceStatus, ModelPreparationResponse,
    PrepareLocalModelRequest,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub const INFERENCE_EVENT_NAME: &str = "local-inference-event";

pub struct InferenceRuntimeState(pub Arc<InferenceService>);

impl InferenceRuntimeState {
    pub fn new_default() -> Result<Self, InferenceError> {
        let provider = Arc::new(OllamaProvider::new_default()?);
        let resolver = ModelResolver::from_bundled_registry()?;
        Ok(Self(Arc::new(InferenceService::new(
            provider,
            resolver,
            ServiceLimits::default(),
        )?)))
    }
}

struct TauriEventSink {
    app: AppHandle,
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError> {
        self.app
            .emit(INFERENCE_EVENT_NAME, event)
            .map_err(|_| InferenceError::EventDelivery)
    }
}

#[tauri::command]
pub async fn get_local_inference_status(
    state: State<'_, InferenceRuntimeState>,
) -> Result<InferenceStatus, InferencePublicError> {
    state.0.status().await.map_err(InferencePublicError::from)
}

#[tauri::command]
pub async fn list_inference_models(
    state: State<'_, InferenceRuntimeState>,
) -> Result<Vec<InferenceModelSummary>, InferencePublicError> {
    state.0.models().await.map_err(InferencePublicError::from)
}

#[tauri::command]
pub async fn prepare_local_model(
    request: PrepareLocalModelRequest,
    state: State<'_, InferenceRuntimeState>,
) -> Result<ModelPreparationResponse, InferencePublicError> {
    state
        .0
        .prepare_model(request)
        .await
        .map_err(InferencePublicError::from)
}

#[tauri::command]
pub async fn run_local_inference(
    app: AppHandle,
    request: InferenceRequest,
    state: State<'_, InferenceRuntimeState>,
) -> Result<InferenceResponse, InferencePublicError> {
    state
        .0
        .run(request, Arc::new(TauriEventSink { app }))
        .await
        .map_err(InferencePublicError::from)
}

#[tauri::command]
pub fn cancel_local_inference(
    request_id: String,
    state: State<'_, InferenceRuntimeState>,
) -> Result<bool, InferencePublicError> {
    state
        .0
        .cancel(&request_id)
        .map_err(InferencePublicError::from)
}

#[tauri::command]
pub fn cancel_local_model_preparation(
    request_id: String,
    state: State<'_, InferenceRuntimeState>,
) -> Result<bool, InferencePublicError> {
    state
        .0
        .cancel_preparation(&request_id)
        .map_err(InferencePublicError::from)
}

#[tauri::command]
pub async fn run_local_inference_smoke(
    app: AppHandle,
    canonical_model_id: String,
    state: State<'_, InferenceRuntimeState>,
) -> Result<InferenceResponse, InferencePublicError> {
    let request = InferenceRequest {
        request_id: Uuid::new_v4().to_string(),
        canonical_model_id,
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Reply with: OK, local edge inference is operational.".to_string(),
        }],
        max_tokens: 48,
        temperature: 0.0,
        stream: true,
    };
    state
        .0
        .run(request, Arc::new(TauriEventSink { app }))
        .await
        .map_err(InferencePublicError::from)
}
