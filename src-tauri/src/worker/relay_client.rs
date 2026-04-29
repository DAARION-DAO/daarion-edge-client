use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerHello {
    pub event_type: String, // "worker_hello"
    pub payload: WorkerHelloPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerHelloPayload {
    pub protocol_version: String,
    pub worker_uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerHelloAck {
    pub event_type: String, // "worker_hello_ack"
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnrollmentRequest {
    pub event_type: String, // "enrollment_req"
    pub payload: EnrollmentReqPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnrollmentReqPayload {
    pub worker_uuid: String,
    pub pubkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnrollmentDecision {
    pub event_type: String, // "enrollment_dec"
    pub payload: EnrollmentDecPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnrollmentDecPayload {
    pub status: String,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyDecision {
    pub event_type: String, // "verify_decision"
    pub payload: VerifyDecisionPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyDecisionPayload {
    pub status: String,
    pub reason: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskAssignment {
    pub event_type: String, // "task_assignment"
    pub payload: TaskPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskPayload {
    pub task_id: String,
    pub lease_id: String,
    pub lease_expires_at: u64,
    pub work_type: String,
    pub args: TaskArgs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskArgs {
    pub value: Option<u64>,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionReceipt {
    pub event_type: String, // "execution_receipt"
    pub payload: ExecutionReceiptPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionReceiptPayload {
    pub worker_id: String,
    pub lease_id: String,
    pub raw_advisory_json: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdvisoryResult {
    pub task_id: String,
    pub result: AdvisoryOutput,
    pub execution_ms: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdvisoryOutput {
    pub output: serde_json::Value,
}

#[async_trait]
pub trait RelayClient: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn send_hello(&mut self, hello: WorkerHello) -> Result<WorkerHelloAck, String>;
    async fn send_enrollment(&mut self, req: EnrollmentRequest) -> Result<EnrollmentDecision, String>;
    async fn wait_for_task(&mut self) -> Result<TaskAssignment, String>;
    async fn send_receipt(&mut self, receipt: ExecutionReceipt) -> Result<(), String>;
    async fn wait_for_verify(&mut self) -> Result<VerifyDecision, String>;
}


pub struct MockRelayClient {
    pub connected: bool,
}

impl MockRelayClient {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

#[async_trait]
impl RelayClient for MockRelayClient {
    async fn connect(&mut self) -> Result<(), String> {
        println!("[MockRelay] Connected locally (no socket)");
        self.connected = true;
        Ok(())
    }

    async fn send_hello(&mut self, hello: WorkerHello) -> Result<WorkerHelloAck, String> {
        println!("[MockRelay] -> {:?}", hello);
        Ok(WorkerHelloAck {
            event_type: "worker_hello_ack".into(),
            payload: serde_json::json!({"status": "active"}),
        })
    }

    async fn send_enrollment(&mut self, req: EnrollmentRequest) -> Result<EnrollmentDecision, String> {
        println!("[MockRelay] -> {:?}", req);
        // [GUARDRAIL 3]: Mock path must not produce backend-recognized or Active semantics.
        // Returns provisional/non-authoritative state with no credential.
        Ok(EnrollmentDecision {
            event_type: "enrollment_dec".into(),
            payload: EnrollmentDecPayload {
                status: "provisional".into(),
                token: None,
            },
        })
    }

    async fn wait_for_task(&mut self) -> Result<TaskAssignment, String> {
        println!("[MockRelay] Simulating scheduler wait...");
        Ok(TaskAssignment {
            event_type: "task_assignment".into(),
            payload: TaskPayload {
                task_id: "tsk-mock-001".into(),
                lease_id: "ls-mock-001".into(),
                lease_expires_at: 9999999999,
                work_type: "ping_math".into(),
                args: TaskArgs { value: Some(2), text: None },
            }
        })
    }

    async fn send_receipt(&mut self, receipt: ExecutionReceipt) -> Result<(), String> {
        println!("[MockRelay] Validated Receipt Locally: {:?}", receipt);
        Ok(())
    }

    async fn wait_for_verify(&mut self) -> Result<VerifyDecision, String> {
        Ok(VerifyDecision {
            event_type: "verify_decision".into(),
            payload: VerifyDecisionPayload {
                status: "accepted".into(),
                reason: None,
                task_id: Some("tsk-mock-001".into()),
            }
        })
    }
}

pub struct WsRelayClient {
    pub endpoint: String,
    pub tx: Option<tokio::sync::mpsc::Sender<String>>,
    pub rx: Option<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<String>>>,
}

impl WsRelayClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            tx: None,
            rx: None,
        }
    }
}

#[async_trait]
impl RelayClient for WsRelayClient {
    async fn connect(&mut self) -> Result<(), String> {
        let url = Url::parse(&self.endpoint).map_err(|e: url::ParseError| e.to_string())?;
        println!("[WsRelayClient] Dialing Dev WS Relay at {}...", url);
        
        let connect_future = connect_async(url);
        let (ws_stream, _) = tokio::time::timeout(std::time::Duration::from_secs(5), connect_future)
            .await
            .map_err(|_| "WS Connect Timeout".to_string())?
            .map_err(|e: tokio_tungstenite::tungstenite::Error| format!("WS Connect Failed: {}", e))?;
        
        // Setup channels to preserve stream state via Actor-like bridging
        let (tx_in, mut rx_in) = tokio::sync::mpsc::channel::<String>(10);
        let (tx_out, rx_out) = tokio::sync::mpsc::channel::<String>(10);
        
        let (mut write, mut read) = ws_stream.split();
        
        tokio::spawn(async move {
            while let Some(msg) = rx_in.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() { break; }
            }
        });
        
        let tx_out_clone = tx_out.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if tx_out_clone.send(text.to_string()).await.is_err() { break; }
                }
            }
        });

        self.tx = Some(tx_in);
        self.rx = Some(tokio::sync::Mutex::new(rx_out));
        Ok(())
    }

    async fn send_hello(&mut self, hello: WorkerHello) -> Result<WorkerHelloAck, String> {
        let msg = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        
        if let Some(tx) = &self.tx {
            tx.send(msg.clone()).await.map_err(|e| e.to_string())?;
            println!("[WsRelayClient] TX: {}", msg);
        } else {
            return Err("Not connected".into());
        }

        if let Some(rx_mutex) = &self.rx {
            let mut rx = rx_mutex.lock().await;
            match tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv()).await {
                Ok(Some(text)) => {
                    println!("[WsRelayClient] RX: {}", text);
                    let ack: WorkerHelloAck = serde_json::from_str(&text).map_err(|e| format!("Decode err: {}", e))?;
                    return Ok(ack);
                }
                Ok(None) => return Err("Session dropped".into()),
                Err(_) => return Err("Timeout waiting for hello ack".into()),
            }
        }
        Err("No response".into())
    }

    async fn send_enrollment(&mut self, req: EnrollmentRequest) -> Result<EnrollmentDecision, String> {
        let msg = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        
        if let Some(tx) = &self.tx {
            tx.send(msg.clone()).await.map_err(|e| e.to_string())?;
            println!("[WsRelayClient] TX: {}", msg);
        } else {
            return Err("Not connected".into());
        }

        if let Some(rx_mutex) = &self.rx {
            let mut rx = rx_mutex.lock().await;
            match tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv()).await {
                Ok(Some(text)) => {
                    println!("[WsRelayClient] RX: {}", text);
                    let dec: EnrollmentDecision = serde_json::from_str(&text).map_err(|e| format!("Decode err: {}", e))?;
                    return Ok(dec);
                }
                Ok(None) => return Err("Session dropped".into()),
                Err(_) => return Err("Timeout waiting for enrollment decision".into()),
            }
        }
        Err("No response".into())
    }

    async fn wait_for_task(&mut self) -> Result<TaskAssignment, String> {
        if let Some(rx_mutex) = &self.rx {
            let mut rx = rx_mutex.lock().await;
            // Block indefinitely until server issues task
            if let Some(text) = rx.recv().await {
                println!("[WsRelayClient] RX (Task): {}", text);
                let task: TaskAssignment = serde_json::from_str(&text).map_err(|e| format!("Decode err: {}", e))?;
                return Ok(task);
            }
        }
        Err("Session dropped while waiting for task".into())
    }

    async fn send_receipt(&mut self, receipt: ExecutionReceipt) -> Result<(), String> {
        let msg = serde_json::to_string(&receipt).map_err(|e| e.to_string())?;
        if let Some(tx) = &self.tx {
            tx.send(msg.clone()).await.map_err(|e| e.to_string())?;
            println!("[WsRelayClient] TX (Receipt): {}", msg);
            Ok(())
        } else {
            Err("Not connected".into())
        }
    }

    async fn wait_for_verify(&mut self) -> Result<VerifyDecision, String> {
        if let Some(rx_mutex) = &self.rx {
            let mut rx = rx_mutex.lock().await;
            match tokio::time::timeout(std::time::Duration::from_secs(10), rx.recv()).await {
                Ok(Some(text)) => {
                    println!("[WsRelayClient] RX (Verify): {}", text);
                    let dec: VerifyDecision = serde_json::from_str(&text).map_err(|e| format!("Decode err: {}", e))?;
                    return Ok(dec);
                }
                Ok(None) => return Err("Session dropped".into()),
                Err(_) => return Err("Timeout waiting for verify decision".into()),
            }
        }
        Err("Session dropped while waiting for verification".into())
    }
}
