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

#[async_trait]
pub trait RelayClient: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn send_hello(&mut self, hello: WorkerHello) -> Result<WorkerHelloAck, String>;
    async fn send_enrollment(&mut self, req: EnrollmentRequest) -> Result<EnrollmentDecision, String>;
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
        Ok(EnrollmentDecision {
            event_type: "enrollment_dec".into(),
            payload: EnrollmentDecPayload {
                status: "approved".into(),
                token: Some("jwt-mock-123".into()),
            },
        })
    }
}

pub struct WsRelayClient {
    pub endpoint: String,
}

impl WsRelayClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }
}

#[async_trait]
impl RelayClient for WsRelayClient {
    async fn connect(&mut self) -> Result<(), String> {
        let url = Url::parse(&self.endpoint).map_err(|e| e.to_string())?;
        println!("[WsRelayClient] Dialing Dev WS Relay at {}...", url);
        // Note: For MVP we do sequential connect/disconnect for each message just to prove the payload boundary.
        // A Persistent connection loop will be added in Move 5.
        Ok(())
    }

    async fn send_hello(&mut self, hello: WorkerHello) -> Result<WorkerHelloAck, String> {
        let url = Url::parse(&self.endpoint).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("WS Connect Failed: {}", e))?;
        
        let (mut write, mut read) = ws_stream.split();
        let msg = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        write.send(Message::Text(msg.clone().into())).await.map_err(|e| e.to_string())?;
        println!("[WsRelayClient] TX: {}", msg);

        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| e.to_string())?;
            if let Message::Text(text) = msg {
                println!("[WsRelayClient] RX: {}", text);
                let ack: WorkerHelloAck = serde_json::from_str(text.as_str()).map_err(|e| format!("Decode err: {}", e))?;
                return Ok(ack);
            }
        }
        Err("No response/Session Dropped".to_string())
    }

    async fn send_enrollment(&mut self, req: EnrollmentRequest) -> Result<EnrollmentDecision, String> {
        let url = Url::parse(&self.endpoint).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("WS Connect Failed: {}", e))?;
        
        let (mut write, mut read) = ws_stream.split();
        let msg = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        write.send(Message::Text(msg.clone().into())).await.map_err(|e| e.to_string())?;
        println!("[WsRelayClient] TX: {}", msg);

        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| e.to_string())?;
            if let Message::Text(text) = msg {
                println!("[WsRelayClient] RX: {}", text);
                let dec: EnrollmentDecision = serde_json::from_str(text.as_str()).map_err(|e| format!("Decode err: {}", e))?;
                return Ok(dec);
            }
        }
        Err("No response/Session Dropped".to_string())
    }
}
