use crate::worker::sandbox_executor::JobResult;
use tauri::{AppHandle, Emitter};

pub struct ResultPublisher;

impl ResultPublisher {
    pub async fn publish(app: AppHandle, result: JobResult) -> Result<(), String> {
        // REAL FLOW: Send to NATS results.{task_id}
        // For M1, we emit a tauri event to show progress in the UI
        println!("Publishing result for task {}: {}", result.task_id, result.status);
        
        // Log to frontend dashboard
        let event_payload = serde_json::json!({
            "task_id": result.task_id,
            "status": result.status,
            "output": result.output,
            "time_ms": result.execution_time_ms,
        });

        app.emit("worker-job-finished", event_payload).map_err(|e| e.to_string())?;

        Ok(())
    }
}
