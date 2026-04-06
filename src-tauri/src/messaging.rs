use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager, State, Emitter};
use rand::Rng;
use crate::enrollment::get_node_token;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectivityState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32, next_retry_sec: u64 },
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingSession {
    pub session_id: String,
    pub messaging_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomInfo {
    pub room_id: String,
    pub display_name: String,
    pub participants: Vec<String>,
}

pub struct MessagingState {
    pub connectivity: Mutex<ConnectivityState>,
    pub session: Mutex<Option<MessagingSession>>,
    pub room_info: Mutex<Option<RoomInfo>>,
    pub messages: Mutex<Vec<Message>>,
}

impl MessagingState {
    pub fn new() -> Self {
        Self {
            connectivity: Mutex::new(ConnectivityState::Disconnected),
            session: Mutex::new(None),
            room_info: Mutex::new(None),
            messages: Mutex::new(Vec::new()),
        }
    }
}

#[tauri::command]
pub async fn get_messaging_status(
    state: State<'_, Arc<MessagingState>>,
) -> Result<(ConnectivityState, Option<RoomInfo>, Option<MessagingSession>), String> {
    let connectivity = state.connectivity.lock().await.clone();
    let room_info = state.room_info.lock().await.clone();
    let session = state.session.lock().await.clone();
    Ok((connectivity, room_info, session))
}

#[tauri::command]
pub async fn bootstrap_messaging(
    app: AppHandle,
    state: State<'_, Arc<MessagingState>>,
) -> Result<(), String> {
    let mut conn = state.connectivity.lock().await;
    *conn = ConnectivityState::Connecting;
    app.emit("messaging-status-changed", conn.clone()).unwrap();

    let node_token = match get_node_token() {
        Ok(t) => t,
        Err(_) => {
            let err = "Node token not found. Enroll first.".to_string();
            *conn = ConnectivityState::Error(err.clone());
            app.emit("messaging-status-changed", conn.clone()).unwrap();
            return Err(err);
        }
    };

    // REAL FLOW: Request messaging session from backend using node_token
    // For M1.5, we implement the structure of the real flow
    tokio::time::sleep(Duration::from_secs(1)).await;

    let session = MessagingSession {
        session_id: format!("sess_{}", uuid::Uuid::new_v4()),
        messaging_token: format!("msg_{}", uuid::Uuid::new_v4()),
    };

    let room_info = RoomInfo {
        room_id: "!node_room_hardened:matrix.daarion.city".to_string(),
        display_name: "Node Control Plane".to_string(),
        participants: vec!["Guardian Agent".to_string(), "Steward Agent".to_string(), "User".to_string()],
    };

    *state.session.lock().await = Some(session);
    *state.room_info.lock().await = Some(room_info.clone());
    *conn = ConnectivityState::Connected;
    
    app.emit("messaging-status-changed", conn.clone()).unwrap();
    app.emit("messaging-room-ready", room_info).unwrap();

    // Start background poller
    start_poller(app.clone(), state.inner().clone());

    Ok(())
}

fn start_poller(app: AppHandle, state: Arc<MessagingState>) {
    tokio::spawn(async move {
        let mut retry_count = 0;
        loop {
            let session = state.session.lock().await.clone();
            if session.is_none() { break; }

            // Simulated real poll using reqwest (stubbed endpoint)
            let result = poll_endpoint(&state).await;

            match result {
                Ok(new_messages) => {
                    retry_count = 0;
                    if !new_messages.is_empty() {
                        let mut msgs = state.messages.lock().await;
                        for m in new_messages {
                            msgs.push(m.clone());
                            app.emit("messaging-new-message", m).unwrap();
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    retry_count += 1;
                    let next_retry = (2_u64.pow(retry_count.min(6))) * 5; // Exp backoff
                    
                    let mut conn = state.connectivity.lock().await;
                    *conn = ConnectivityState::Reconnecting { 
                        attempt: retry_count, 
                        next_retry_sec: next_retry 
                    };
                    app.emit("messaging-status-changed", conn.clone()).unwrap();
                    
                    eprintln!("Messaging poll error (attempt {}): {}", retry_count, e);
                    tokio::time::sleep(Duration::from_secs(next_retry)).await;
                    
                    // After fixed attempts, we might want to mark as error or just keep retrying
                    if retry_count > 10 {
                         *conn = ConnectivityState::Error("Persistent connection failure".to_string());
                         app.emit("messaging-status-changed", conn.clone()).unwrap();
                         break;
                    }
                }
            }
        }
    });
}

async fn poll_endpoint(state: &MessagingState) -> Result<Vec<Message>, String> {
    // In a final implementation, this would be:
    // let client = reqwest::Client::new();
    // let res = client.get("https://api.daarion.city/v1/messaging/poll")
    //     .header("Authorization", format!("Bearer {}", token))
    //     .send().await...
    
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.1) {
         return Ok(vec![Message {
             id: uuid::Uuid::new_v4().to_string(),
             sender: "Guardian Agent".to_string(),
             content: "Control plane heartbeat OK. Verified capability status.".to_string(),
             timestamp: chrono::Utc::now().timestamp(),
         }]);
    }
    
    // Simulate rare connection error for testing backoff
    if rng.gen_bool(0.02) {
        return Err("Gateway timeout".to_string());
    }

    Ok(vec![])
}

#[tauri::command]
pub async fn send_node_message(
    app: AppHandle,
    state: State<'_, Arc<MessagingState>>,
    content: String,
) -> Result<(), String> {
    let conn = state.connectivity.lock().await;
    if !matches!(*conn, ConnectivityState::Connected) {
        return Err("Messaging not connected".to_string());
    }

    // Mock sending message to real backend
    let msg = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender: "User".to_string(),
        content,
        timestamp: chrono::Utc::now().timestamp(),
    };

    state.messages.lock().await.push(msg.clone());
    app.emit("messaging-new-message", msg).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn get_node_messages(
    state: State<'_, Arc<MessagingState>>,
) -> Result<Vec<Message>, String> {
    Ok(state.messages.lock().await.clone())
}

#[tauri::command]
pub async fn reconnect_messaging(
    app: AppHandle,
    state: State<'_, Arc<MessagingState>>,
) -> Result<(), String> {
    bootstrap_messaging(app, state).await
}
