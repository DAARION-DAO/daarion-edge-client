use tauri::{AppHandle, Manager, Emitter};
use tokio::time::Instant;
use crate::models::inference_session::{LocalInferenceRequest, LocalInferenceResponse, InferenceSessionState};
use crate::models::inference_limits::InferenceLimits;
use crate::models::registry::ModelRegistry;
use crate::models::runtime_loader::RuntimeLoader;
use crate::models::inference_arbitrator::{InferenceArbitrator, InferenceExecutionDecision, ArbitrationResult};
use crate::models::eviction_policy::EvictionPolicy;
use crate::authorities::aip_enforcer::{AipEnforcer, AipContext};
use crate::authorities::authority_protocol::{AuthorityActionClass};
use crate::authorities::authority_decision::AuthorityDecision;

pub struct LocalInference;

impl LocalInference {
    pub async fn run_chat(app: AppHandle, request: LocalInferenceRequest) -> Result<LocalInferenceResponse, String> {
        let start_time = Instant::now();
        let request_id = request.request_id.clone();
        let model_id = request.model_id.clone();

        // 1. AIP Enforcement Gate (v2)
        let aip_ctx = AipContext {
            action_id: uuid::Uuid::new_v4(),
            action_type: AuthorityActionClass::ModelActivation,
            identity_scope: "inference.session".to_string(),
            evidence_ref: Some(uuid::Uuid::new_v4()), // Placeholder evidence
            trust_chain: None, // Derived from user session context
            target_resource: model_id.clone(),
            requested_by: "user".to_string(), // In reality derived from session context
            metadata: std::collections::HashMap::new(),
        };

        let aip_result = AipEnforcer::enforce(aip_ctx);
        if !matches!(aip_result.decision, AuthorityDecision::Allow) {
            return Err(format!("AIP Enforcement Veto: {}", aip_result.reason));
        }

        // 2. Validation
        let limits = InferenceLimits::default();
        limits.validate_prompt(&request.prompt)?;
        
        let _model_entry = ModelRegistry::get_supported_models().into_iter()
            .find(|m| m.model_id == model_id)
            .ok_or_else(|| format!("Model {} not found in registry", model_id))?;

        // 3. Arbitration
        let pressure = EvictionPolicy::get_system_memory_pressure();
        // M1: Simulated states for arbitration
        let is_warm = false; // Simulated: check real loader state in v2
        let residency_score = 45.0; // Simulated
        let local_queue_depth = 0; // Simulated
        
        let arbitration = InferenceArbitrator::decide(
            &model_id,
            is_warm,
            residency_score,
            pressure,
            local_queue_depth
        );

        app.emit("inference-arbitration-result", &arbitration).unwrap();

        match arbitration.decision {
            InferenceExecutionDecision::Reject(reason) => {
                return Err(format!("Request rejected: {}", reason));
            },
            InferenceExecutionDecision::RemoteExecution => {
                return Self::handle_remote_fallback(app, request, arbitration).await;
            },
            InferenceExecutionDecision::LocalExecution => {
                // Continue with local execution
                Self::execute_local(app, request).await
            }
        }
    }

    async fn execute_local(app: AppHandle, request: LocalInferenceRequest) -> Result<LocalInferenceResponse, String> {
        let start_time = Instant::now();
        let request_id = request.request_id.clone();
        let model_id = request.model_id.clone();

        // State Transition: Loading
        app.emit("inference-session-update", serde_json::json!({
            "request_id": &request_id,
            "state": "LoadingModel"
        })).unwrap();

        // Ensure model is loaded (warm)
        RuntimeLoader::load_model(app.clone(), model_id.clone()).await?;

        // State Transition: Running
        app.emit("inference-session-update", serde_json::json!({
            "request_id": &request_id,
            "state": "Running"
        })).unwrap();

        // Simulated inference logic
        println!("LocalInference: Executing chat prompt for {} locally...", model_id);
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        
        let output_text = format!(
            "Local execution on {}. Prompt: \"{}\"", 
            model_id, 
            request.prompt
        );

        let latency_ms = start_time.elapsed().as_millis() as u64;

        let response = LocalInferenceResponse {
            request_id,
            status: "Done".to_string(),
            model_id: model_id.clone(),
            runtime: "llama.cpp (Local)".to_string(),
            latency_ms,
            output_text,
        };

        app.emit("inference-session-update", serde_json::json!({
            "request_id": &response.request_id,
            "state": "Done",
            "result": &response
        })).unwrap();

        Ok(response)
    }

    async fn handle_remote_fallback(
        app: AppHandle, 
        request: LocalInferenceRequest,
        arbitration: ArbitrationResult
    ) -> Result<LocalInferenceResponse, String> {
        let start_time = Instant::now();
        let request_id = request.request_id.clone();
        
        app.emit("inference-session-update", serde_json::json!({
            "request_id": &request_id,
            "state": "RoutingToNetwork",
            "reason": arbitration.reason
        })).unwrap();

        // Simulated NATS publishing
        println!("LocalInference: Routing {} to network lane mm.online.reason.global.t2.smallllm", request_id);
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        let response = LocalInferenceResponse {
            request_id,
            status: "Done (Offloaded)".to_string(),
            model_id: request.model_id,
            runtime: "DAARION Network (Remote)".to_string(),
            latency_ms: start_time.elapsed().as_millis() as u64,
            output_text: "Inference result received from remote node in 'global.t2' lane.".to_string(),
        };

        app.emit("inference-session-update", serde_json::json!({
            "request_id": &response.request_id,
            "state": "Done",
            "result": &response
        })).unwrap();

        Ok(response)
    }
}
