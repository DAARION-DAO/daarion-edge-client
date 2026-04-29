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
        limits.validate_chat(&request.messages)?;
        
        let payload = ModelRegistry::fetch_registry(app.clone()).await?;
        let _model_entry = payload.models.into_iter()
            .find(|m| m.id == model_id)
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

        let mut final_messages = vec![
            crate::models::inference_session::ChatMessage {
                role: "system".to_string(),
                content: "You are a local DAARION Edge agent. You operate directly on the user's device providing private, local assistance. You do not pretend to be a remote network sovereign agent spanning multiple nodes. You provide extremely helpful, intelligent, local, and private support.".to_string(),
            }
        ];
        
        for msg in request.messages {
            if msg.role != "system" {
                final_messages.push(msg);
            }
        }

        #[derive(serde::Serialize)]
        struct OllamaChatRequest {
            model: String,
            messages: Vec<crate::models::inference_session::ChatMessage>,
            stream: bool,
            keep_alive: String,
        }

        let chat_req = OllamaChatRequest {
            model: model_id.clone(),
            messages: final_messages,
            stream: true,
            keep_alive: "15m".to_string(),
        };

        let mut response = reqwest::Client::new()
            .post("http://localhost:11434/api/chat")
            .json(&chat_req)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        let mut full_output = String::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| format!("Chunk error: {}", e))? {
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if line.trim().is_empty() { continue; }
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(msg) = parsed.get("message").and_then(|m| m.as_object()) {
                        if let Some(c) = msg.get("content").and_then(|c| c.as_str()) {
                            full_output.push_str(c);
                            let _ = app.emit("inference-token-stream", serde_json::json!({
                                "request_id": &request_id,
                                "token": c
                            }));
                        }
                    }
                }
            }
        }

        let latency_ms = start_time.elapsed().as_millis() as u64;

        let res = LocalInferenceResponse {
            request_id,
            status: "Done".to_string(),
            model_id,
            runtime: "llama.cpp (Local) via /api/chat".to_string(),
            latency_ms,
            output_text: full_output,
        };

        app.emit("inference-session-update", serde_json::json!({
            "request_id": &res.request_id,
            "state": "Done",
            "result": &res
        })).unwrap();

        Ok(res)
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
