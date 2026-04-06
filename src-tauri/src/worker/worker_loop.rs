use crate::worker::lease_validator::{Lease, LeaseValidator};
use crate::worker::admission::{AdmissionController, AdmissionInput, JobClass, ExecutionDecision};
use crate::worker::sandbox_executor::SandboxExecutor;
use crate::worker::result_publisher::ResultPublisher;
use crate::worker::nats_client::NatsClient;
use crate::authorities::{aip_enforcer::{AipEnforcer, AipContext}, authority_protocol::{AuthorityActionClass}, authority_decision::AuthorityDecision};
use crate::worker::models::SpecializationPolicy;
use crate::identity::load_or_create_identity as get_identity;
use tauri::AppHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_worker_loop(app: AppHandle) {
    tokio::spawn(async move {
        // Init NATS (Simulation)
        let _ = NatsClient::connect().await;

        loop {
            // 1. Pull Task
            match NatsClient::pull_task().await {
                Ok(Some(_payload)) => {
                    // Logic to process a task:
                    // In a real environment, we would deserialize the lease here.
                    // For M1, we simulate a successful lease.
                    let lease = create_mock_lease();
                    
                    // 2. AIP Enforcement Gate (v2)
                    let aip_ctx = AipContext {
                        action_id: uuid::Uuid::new_v4(),
                        action_type: AuthorityActionClass::WorkerLease,
                        identity_scope: "worker.execution".to_string(),
                        evidence_ref: Some(uuid::Uuid::new_v4()), // Placeholder evidence
                        trust_chain: None, // In M1/M2, this would be derived from the Lease/NATS metadata
                        target_resource: lease.node_target.clone(),
                        requested_by: "system".to_string(),
                        metadata: std::collections::HashMap::new(),
                    };

                    let aip_result = AipEnforcer::enforce(aip_ctx);
                    if !matches!(aip_result.decision, AuthorityDecision::Allow) {
                        eprintln!("Worker: AIP Enforcement Veto: {}", aip_result.reason);
                        continue;
                    }

                    // 3. Validate Lease
                    // Using a dummy public key for M1 simulation
                    let dummy_pub_key = "0000000000000000000000000000000000000000000000000000000000000000";
                    if let Err(e) = LeaseValidator::validate(&app, &lease, dummy_pub_key) {
                        eprintln!("Worker: Lease validation failed: {}", e);
                        continue;
                    }

                    // 4. Local Admission Controller check
                    let profile = SpecializationPolicy::get_default_profile();
                    let input = AdmissionInput {
                        job_class: JobClass::Interactive, 
                        job_type: lease.payload.job_type.clone(),
                        estimated_latency_ms: 100,
                        cpu_pressure: 0.1,
                        memory_pressure: 0.1,
                        gpu_available: true,
                        on_battery: false,
                        queue_depth: 0,
                        specialization: profile,
                    };

                    match AdmissionController::decide(input) {
                        ExecutionDecision::ExecuteNow => {
                            // 4. Sandbox Execute
                            let node_id = get_identity(&app).ok().map(|id| id.node_id).unwrap_or_default();
                            let result = SandboxExecutor::execute_echo(
                                lease.task_id, 
                                node_id, 
                                lease.payload.input
                            ).await;

                            // 5. Result Publish
                            let _ = ResultPublisher::publish(app.clone(), result).await;
                        }
                        _ => {
                            // Defer or NAK simulation
                            println!("Worker: Admission Controller deferred/NAKed the task.");
                        }
                    }
                }
                _ => {}
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

fn create_mock_lease() -> Lease {
    use crate::worker::lease_validator::{ResourceLimits, JobPayload};
    Lease {
        task_id: format!("task_{}", uuid::Uuid::new_v4()),
        trace_id: uuid::Uuid::new_v4().to_string(),
        node_target: "*".to_string(),
        capabilities_required: vec!["echo".to_string()],
        resource_limits: ResourceLimits {
            cpu_limit: 1.0,
            memory_limit_mb: 512,
            gpu_allowed: false,
            timeout_sec: 5,
        },
        ttl: chrono::Utc::now().timestamp() + 3600,
        signature: vec![0u8; 64], // Dummy signature for M1
        payload: JobPayload {
            job_type: "echo".to_string(),
            input: "System Operational. M1 Worker Data Plane Active.".to_string(),
        },
    }
}
