//! Session-only AG-UI adapter for the existing local City surface. No folder authority.
use super::{
    cancel, lock,
    store::{CityTaskContext, Task},
    Pilot, QuietSink,
};
use crate::inference::types::{ChatMessage, InferenceRequest};
use futures_util::StreamExt;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Full, Limited, StreamBody};
use hyper::{
    body::{Bytes, Frame, Incoming},
    Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tauri::State;
use tokio::{net::TcpListener, sync::mpsc, task::JoinSet};

const BIND: &str = "127.0.0.1:19436";
const CITY_API: &str = "http://127.0.0.1:3001";
const CITY_UI: &str = "http://127.0.0.1:3010/agents";
const MAX_BODY: usize = 64 * 1024;
type Body = UnsyncBoxBody<Bytes, Infallible>;
type Sender = mpsc::Sender<Result<Frame<Bytes>, Infallible>>;

#[derive(Clone, Serialize, Default)]
pub struct CityStatus {
    pub listener_ready: bool,
    pub profile_id: Option<String>,
    pub connected: bool,
    pub detail: String,
}
pub struct Bridge {
    task: tauri::async_runtime::JoinHandle<()>,
    token: String,
    endpoint: String,
    revoked: tokio::sync::watch::Sender<()>,
}
impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = self.revoked.send(());
        self.task.abort();
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunInput {
    thread_id: String,
    run_id: String,
    messages: Vec<Message>,
}
#[derive(Deserialize)]
struct Message {
    role: String,
    content: Option<Value>,
}

fn decode(bytes: &[u8]) -> Result<RunInput, String> {
    if bytes.len() > MAX_BODY {
        return Err("Request is too large".into());
    }
    let input: RunInput = serde_json::from_slice(bytes).map_err(|_| "Invalid AG-UI request")?;
    for id in [&input.thread_id, &input.run_id] {
        if id.is_empty() || id.len() > 160 || id.chars().any(char::is_control) {
            return Err("Invalid correlation ID".into());
        }
    }
    if input.messages.is_empty() || input.messages.len() > 128 {
        return Err("Invalid message count".into());
    }
    if !input.messages.iter().any(|m| {
        m.role == "user"
            && m.content
                .as_ref()
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
    }) {
        return Err("A text user message is required".into());
    }
    Ok(input)
}
fn task_id(agent_id: &str, input: &RunInput) -> String {
    let hash = Sha256::digest(
        serde_json::to_vec(&(agent_id, &input.thread_id, &input.run_id)).expect("string tuple"),
    );
    uuid::Uuid::from_bytes(hash[..16].try_into().expect("digest length")).to_string()
}
fn request_hash(input: &RunInput) -> String {
    let messages: Vec<_> = input
        .messages
        .iter()
        .map(|m| (&m.role, &m.content))
        .collect();
    hex::encode(Sha256::digest(
        serde_json::to_vec(&messages).expect("JSON values"),
    ))
}
fn authorized(value: Option<&str>, token: &str) -> bool {
    let expected = format!("Bearer {token}");
    let supplied = value.unwrap_or_default().as_bytes();
    supplied.len() == expected.len()
        && supplied
            .iter()
            .zip(expected.bytes())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
}
fn plain(status: StatusCode, message: &str) -> Response<Body> {
    let mut response = Response::new(Full::new(Bytes::from(message.to_owned())).boxed_unsync());
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}
async fn event(sender: &Sender, event: Value) -> bool {
    sender
        .send(Ok(Frame::data(Bytes::from(format!("data: {event}\n\n")))))
        .await
        .is_ok()
}
async fn terminal(sender: &Sender, input: &RunInput, task: &Task) {
    if task.status != "completed" {
        event(sender, json!({"type":"RUN_ERROR", "message": task.error.as_deref().unwrap_or("Local task did not complete")})).await;
        return;
    }
    let message_id = format!("edge-{}", task.id);
    event(
        sender,
        json!({"type":"TEXT_MESSAGE_START", "messageId":message_id, "role":"assistant"}),
    )
    .await;
    event(sender, json!({"type":"TEXT_MESSAGE_CONTENT", "messageId":message_id, "delta":task.answer.as_deref().unwrap_or_default()})).await;
    event(
        sender,
        json!({"type":"TEXT_MESSAGE_END", "messageId":message_id}),
    )
    .await;
    event(
        sender,
        json!({"type":"RUN_FINISHED", "threadId":input.thread_id, "runId":input.run_id}),
    )
    .await;
}

async fn execute(
    pilot: Pilot,
    input: RunInput,
    model_id: String,
    token: Arc<String>,
    sender: Sender,
) {
    let prepared = (|| {
        let mut state = lock(&pilot)?;
        if !state
            .bridge
            .as_ref()
            .is_some_and(|bridge| bridge.token == *token)
        {
            return Err("Local connection was revoked".to_string());
        }
        let profile = state.store.profile()?.ok_or("Local agent is missing")?;
        let id = task_id(&profile.agent_id, &input);
        let hash = request_hash(&input);
        if let Some(previous) = state.store.task(&id)? {
            if previous
                .city
                .as_ref()
                .is_none_or(|c| c.request_hash != hash)
            {
                return Err("Conflicting replay refused".into());
            }
            if previous.status == "running" {
                return Err("This task is already running".into());
            }
            return Ok((previous, None, profile));
        }
        if state.active.is_some() || state.starting || state.choosing {
            return Err("The local agent is busy".into());
        }
        if !state
            .process
            .as_mut()
            .is_some_and(super::model::ModelProcess::alive)
        {
            return Err("Start the local model in Edge first".into());
        }
        let service = state
            .inference
            .as_ref()
            .ok_or("Local model is unavailable")?
            .0
            .clone();
        if !state
            .models
            .iter()
            .any(|m| m.canonical_model_id == model_id && m.installed)
        {
            return Err("Local model is not verified".into());
        }
        let task = Task {
            id: id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "running".into(),
            model: Some(model_id.clone()),
            report: None,
            answer: None,
            error: None,
            city: Some(CityTaskContext {
                request_hash: hash,
                thread_id: input.thread_id.clone(),
                run_id: input.run_id.clone(),
            }),
        };
        state.store.save_task(&task)?;
        state.store.audit("city_task_started")?;
        state.active = Some(id);
        state.cancelled = false;
        Ok::<_, String>((task, Some(service), profile))
    })();
    let (mut task, service, profile) = match prepared {
        Ok(v) => v,
        Err(message) => {
            event(&sender, json!({"type":"RUN_ERROR","message":message})).await;
            return;
        }
    };
    if let Some(service) = service {
        let mut messages = vec![ChatMessage { role:"system".into(), content:format!(
            "You are the personal local Edge agent named {}. Your user's purpose: {}. Reply in Ukrainian. You run locally on the user's computer. This City chat supplies only conversation text. You have no file contents, no City tools, no delegated agents and no authority to claim network enrollment. Do not claim to have read or changed files. Do not invent task results. Keep answers concise.", profile.name, profile.purpose) }];
        let mut history: Vec<_> = input
            .messages
            .iter()
            .rev()
            .filter_map(|m| {
                if !matches!(m.role.as_str(), "user" | "assistant") {
                    return None;
                }
                m.content
                    .as_ref()
                    .and_then(Value::as_str)
                    .map(|text| ChatMessage {
                        role: m.role.clone(),
                        content: text.chars().take(3000).collect(),
                    })
            })
            .take(8)
            .collect();
        history.reverse();
        messages.extend(history);
        let request = InferenceRequest {
            request_id: task.id.clone(),
            canonical_model_id: model_id,
            messages,
            max_tokens: 200,
            temperature: 0.0,
            stream: true,
        };
        let result = tokio::select! {
            biased;
            _ = sender.closed() => { let _=service.cancel(&task.id); Err(crate::inference::types::InferenceError::Cancelled) },
            result = service.run(request, Arc::new(QuietSink)) => result,
        };
        match result {
            Ok(response) if !response.output_text.trim().is_empty() => {
                task.answer = Some(response.output_text)
            }
            Ok(_) => task.error = Some("Local model returned no text".into()),
            Err(error) => task.error = Some(error.public_message()),
        }
        let saved = (|| {
            let mut state = lock(&pilot)?;
            task.status = if state.cancelled || sender.is_closed() {
                "cancelled"
            } else if task.error.is_some() {
                "failed"
            } else {
                "completed"
            }
            .into();
            if task.status == "cancelled" {
                task.answer = None;
                task.error = Some("Local task was cancelled".into());
            }
            state.active = None;
            state.store.save_task(&task)?;
            state.store.audit(&format!("city_task_{}", task.status))
        })();
        if saved.is_err() {
            event(
                &sender,
                json!({"type":"RUN_ERROR","message":"Could not save the local result"}),
            )
            .await;
            return;
        }
    }
    terminal(&sender, &input, &task).await;
}

async fn handle(
    request: Request<Incoming>,
    pilot: Pilot,
    token: Arc<String>,
    route: Arc<String>,
    model_id: Arc<String>,
) -> Result<Response<Body>, Infallible> {
    if request.method() != hyper::Method::POST
        || request.uri().path() != route.as_str()
        || request.uri().query().is_some()
    {
        return Ok(plain(StatusCode::NOT_FOUND, "Not found"));
    }
    if request.headers().contains_key("origin")
        || !authorized(
            request
                .headers()
                .get("authorization")
                .and_then(|h| h.to_str().ok()),
            &token,
        )
    {
        return Ok(plain(
            StatusCode::UNAUTHORIZED,
            "Authenticated server connection required",
        ));
    }
    let body = match tokio::time::timeout(
        Duration::from_secs(5),
        Limited::new(request.into_body(), MAX_BODY).collect(),
    )
    .await
    {
        Ok(Ok(body)) => body.to_bytes(),
        _ => {
            return Ok(plain(
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request body rejected",
            ))
        }
    };
    let input = match decode(&body) {
        Ok(v) => v,
        Err(_) => return Ok(plain(StatusCode::BAD_REQUEST, "Invalid AG-UI request")),
    };
    let (sender, receiver) = mpsc::channel(8);
    event(
        &sender,
        json!({"type":"RUN_STARTED","threadId":input.thread_id,"runId":input.run_id}),
    )
    .await;
    tokio::spawn(execute(pilot, input, model_id.to_string(), token, sender));
    let stream = futures_util::stream::unfold(receiver, |mut rx| async {
        rx.recv().await.map(|frame| (frame, rx))
    });
    let mut response = Response::new(StreamBody::new(stream).boxed_unsync());
    response
        .headers_mut()
        .insert("content-type", "text/event-stream".parse().unwrap());
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    Ok(response)
}

impl Bridge {
    async fn start(pilot: Pilot, agent_id: &str, model_id: String) -> Result<Self, String> {
        let listener = TcpListener::bind(BIND).await.map_err(|_| {
            "Локальний порт з’єднання зайнятий. Чужий процес не використовується.".to_string()
        })?;
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let server_token = Arc::new(token.clone());
        let route = Arc::new(format!("/agents/{agent_id}/ag-ui"));
        let endpoint = format!("http://{BIND}{route}");
        let model = Arc::new(model_id);
        let task = tauri::async_runtime::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    Some(_) = connections.join_next(), if !connections.is_empty() => {},
                    result=listener.accept()=>{
                        let Ok((socket,peer))=result else {break};
                        if !peer.ip().is_loopback() || connections.len()>=4 {continue;}
                        let pilot=pilot.clone();let token=server_token.clone();let route=route.clone();let model=model.clone();
                        connections.spawn(async move {
                            let service=hyper::service::service_fn(move |req|handle(req,pilot.clone(),token.clone(),route.clone(),model.clone()));
                            let _=tokio::time::timeout(Duration::from_secs(195),hyper::server::conn::http1::Builder::new().keep_alive(false).serve_connection(TokioIo::new(socket),service)).await;
                        });
                    }
                }
            }
        });
        Ok(Self {
            task,
            token,
            endpoint,
            revoked: tokio::sync::watch::channel(()).0,
        })
    }
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| "City client unavailable".into())
}
async fn response_json(response: reqwest::Response) -> Result<Value, String> {
    if !response.status().is_success() {
        return Err(format!(
            "Панель відхилила запит ({}).",
            response.status().as_u16()
        ));
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Неповна відповідь панелі.")?;
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err("Відповідь панелі завелика.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| "Панель повернула непідтримувану відповідь.".into())
}
async fn register(
    endpoint: &str,
    token: &str,
    name: &str,
    agent_id: &str,
    node_id: &str,
) -> Result<String, String> {
    let client = client()?;
    let me = response_json(
        client
            .get(format!("{CITY_API}/api/me"))
            .send()
            .await
            .map_err(|_| "Міська панель недоступна на цьому комп’ютері.")?,
    )
    .await?;
    if me.pointer("/user/id").and_then(Value::as_str) != Some("dev-local-user")
        || me.pointer("/user/role").and_then(Value::as_str) != Some("admin")
    {
        return Err("Цей зріз підтримує лише локальну операторську панель. Вхід MicroDAO ще не налаштований.".into());
    }
    let probe = response_json(
        client
            .post(format!("{CITY_API}/api/agents/test-connection"))
            .json(
                &json!({"endpoint":endpoint,"headers":{"Authorization":format!("Bearer {token}")}}),
            )
            .send()
            .await
            .map_err(|_| "Не вдалося перевірити з’єднання з панеллю.")?,
    )
    .await?;
    if probe.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err("Панель не прийняла локального агента. Потрібно перевірити дозвіл на його адресу; профіль не створено.".into());
    }
    let events = probe
        .get("events")
        .and_then(Value::as_array)
        .ok_or("Немає підтвердження виконання агента.")?;
    if !events.iter().any(|v| v == "RUN_FINISHED") || events.iter().any(|v| v == "RUN_ERROR") {
        return Err("Агент не завершив перевірочне завдання. Профіль не створено.".into());
    }
    let agents = response_json(
        client
            .get(format!("{CITY_API}/api/agents"))
            .send()
            .await
            .map_err(|_| "Панель недоступна.")?,
    )
    .await?;
    let matching: Vec<_> = agents
        .get("agents")
        .and_then(Value::as_array)
        .ok_or("Немає переліку агентів.")?
        .iter()
        .filter(|a| a.get("endpoint").and_then(Value::as_str) == Some(endpoint))
        .collect();
    if matching.len() > 1
        || matching
            .first()
            .is_some_and(|a| a.get("mine").and_then(Value::as_bool) != Some(true))
    {
        return Err("Конфлікт власника або дубль профілю агента.".into());
    }
    let body = json!({"name":name,"title":"Особистий локальний агент Edge", "roleDescription":format!("Local Edge pilot. Agent reference: {agent_id}. Node reference: {node_id}. Conversation text only; no shared folder access. Operator-local connection, not MicroDAO enrollment."),"visibility":"private","endpoint":endpoint,"auth":{"header":"Authorization","value":format!("Bearer {token}")}});
    let response = if let Some(existing) = matching.first() {
        let id = existing
            .get("id")
            .and_then(Value::as_str)
            .ok_or("Неповний профіль агента.")?;
        client
            .patch(format!("{CITY_API}/api/agents/{id}"))
            .json(&body)
            .send()
            .await
    } else {
        client
            .post(format!("{CITY_API}/api/agents"))
            .json(&body)
            .send()
            .await
    };
    let saved = response_json(response.map_err(|_| {
        "Немає підтвердження збереження профілю; повторний запуск перевірить наявний запис."
    })?)
    .await?;
    let id = saved
        .pointer("/agent/id")
        .and_then(Value::as_str)
        .ok_or("Немає ідентифікатора міського профілю.")?;
    let readback = response_json(
        client
            .get(format!("{CITY_API}/api/agents/{id}"))
            .send()
            .await
            .map_err(|_| "Не вдалося прочитати збережений профіль.")?,
    )
    .await?;
    if readback.pointer("/agent/endpoint").and_then(Value::as_str) != Some(endpoint)
        || readback.pointer("/agent/hasAuth").and_then(Value::as_bool) != Some(true)
    {
        return Err("Збережений профіль не підтверджує це з’єднання.".into());
    }
    Ok(id.to_owned())
}

#[tauri::command]
pub async fn pilot_connect_city(model_id: String, pilot: State<'_, Pilot>) -> Result<(), String> {
    let profile = {
        let mut state = lock(&pilot)?;
        if state.active.is_some() || state.starting || state.choosing {
            return Err("Спочатку завершіть поточну дію.".into());
        }
        if !state
            .models
            .iter()
            .any(|m| m.canonical_model_id == model_id && m.installed)
        {
            return Err("Оберіть встановлену локальну модель.".into());
        }
        if state.bridge.is_some() {
            return Err("Спочатку від’єднайте поточне локальне з’єднання.".into());
        }
        let profile = state.store.profile()?.ok_or("Спочатку створіть агента.")?;
        state.cancelled = false;
        state.starting = true;
        profile
    };
    let bridge = Bridge::start(pilot.inner().clone(), &profile.agent_id, model_id).await;
    let (endpoint, token, mut revoked) = {
        let mut state = lock(&pilot)?;
        state.starting = false;
        let bridge = bridge?;
        if state.cancelled {
            return Err("Підключення скасовано.".into());
        }
        let values = (
            bridge.endpoint.clone(),
            bridge.token.clone(),
            bridge.revoked.subscribe(),
        );
        state.bridge = Some(bridge);
        state.city_status = CityStatus {
            listener_ready: true,
            profile_id: None,
            connected: false,
            detail: "Edge готовий. Перевіряється дозвіл міської панелі…".into(),
        };
        values
    };
    let result = tokio::select! {
        biased;
        _ = revoked.changed() => Err("Підключення відкликано.".to_string()),
        result = register(
        &endpoint,
        &token,
        &profile.name,
        &profile.agent_id,
        &profile.node_id,
        ) => result,
    };
    let mut state = lock(&pilot)?;
    if !state
        .bridge
        .as_ref()
        .is_some_and(|bridge| bridge.token == token)
    {
        return Err("Підключення відкликано.".into());
    }
    match result {
        Ok(id) => {
            state.city_status=CityStatus {listener_ready:true,profile_id:Some(id),connected:true,detail:"Підключено до локальної операторської панелі. Це ще не вхід працівника через MicroDAO.".into()};
            state.store.audit("city_profile_connected")?;
            Ok(())
        }
        Err(error) => {
            state.city_status.detail = error.clone();
            Err(error)
        }
    }
}
#[tauri::command]
pub fn pilot_disconnect_city(pilot: State<'_, Pilot>) -> Result<(), String> {
    let mut state = lock(&pilot)?;
    cancel(&mut state)?;
    state.bridge = None;
    state.city_status = CityStatus::default();
    state.store.audit("city_connection_revoked")
}
#[tauri::command]
pub fn pilot_open_city() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("/usr/bin/open")
            .arg(CITY_UI)
            .spawn()
            .map_err(|_| "Не вдалося відкрити панель.".to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", CITY_UI])
            .spawn()
            .map_err(|_| "Не вдалося відкрити панель.".to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_protocol_refuses_bad_auth_and_invalid_input() {
        assert!(!authorized(None, "secret"));
        assert!(!authorized(Some("Bearer wrong"), "secret"));
        assert!(authorized(Some("Bearer secret"), "secret"));
        assert!(decode(b"{}").is_err());
        assert!(decode(&vec![0; MAX_BODY + 1]).is_err());
        let body = json!({"threadId":"thread","runId":"run","messages":[{"role":"user","content":"hello"}],"tools":[{"name":"do_not_execute"}],"state":{"untrusted":true}});
        let input = decode(&serde_json::to_vec(&body).unwrap()).unwrap();
        assert_eq!(task_id("agent", &input), task_id("agent", &input));
        assert_ne!(task_id("agent", &input), task_id("other-agent", &input));
        let mut changed = body.clone();
        changed["messages"][0]["content"] = json!("different");
        assert_ne!(
            request_hash(&input),
            request_hash(&decode(&serde_json::to_vec(&changed).unwrap()).unwrap())
        );
    }
    #[tokio::test]
    async fn completed_and_failed_tasks_have_distinct_protocol_outcomes() {
        let input = RunInput {
            thread_id: "t".into(),
            run_id: "r".into(),
            messages: vec![],
        };
        let mut task = Task {
            id: "id".into(),
            created_at: "now".into(),
            status: "completed".into(),
            model: None,
            report: None,
            answer: Some("actual result".into()),
            error: None,
            city: None,
        };
        let (tx, mut rx) = mpsc::channel(8);
        terminal(&tx, &input, &task).await;
        drop(tx);
        let mut data = String::new();
        while let Some(Ok(frame)) = rx.recv().await {
            data.push_str(std::str::from_utf8(frame.data_ref().unwrap()).unwrap());
        }
        assert!(
            data.contains("TEXT_MESSAGE_CONTENT")
                && data.contains("RUN_FINISHED")
                && data.contains("actual result")
        );
        task.status = "failed".into();
        task.answer = None;
        task.error = Some("unavailable".into());
        let (tx, mut rx) = mpsc::channel(8);
        terminal(&tx, &input, &task).await;
        drop(tx);
        let bytes = rx.recv().await.unwrap().unwrap().into_data().unwrap();
        assert!(std::str::from_utf8(&bytes).unwrap().contains("RUN_ERROR"));
        assert!(rx.recv().await.is_none());
    }
}

#[cfg(test)]
mod live_acceptance {
    use super::super::{
        model::{discover, ModelProcess},
        store::Store,
        LocalState,
    };
    use super::*;
    use crate::inference::commands::InferenceRuntimeState;
    use std::sync::Mutex;

    /// Explicit opt-in: uses an already installed local model, its own process and synthetic store.
    #[tokio::test]
    #[ignore = "requires explicit local model execution and free pilot ports"]
    async fn real_local_ag_ui_round_trip_replay_and_shutdown() {
        let root = std::env::temp_dir().join(format!("edge-city-proof-{}", uuid::Uuid::new_v4()));
        let store = Store::open(&root.join("pilot.sqlite3")).unwrap();
        let profile = store
            .create_profile(
                "Local connection proof".into(),
                "Synthetic connection acceptance".into(),
            )
            .unwrap();
        let mut process = ModelProcess::start().unwrap();
        let tags = discover(&mut process).await.unwrap();
        assert!(
            tags.iter().any(|(id, _)| id == "qwen3.5:9b"),
            "Expected installed proof model is absent; do not download"
        );
        let inference = InferenceRuntimeState::for_local_agent_pilot(tags).unwrap();
        let models = inference.0.models().await.unwrap();
        let pilot = Pilot(Arc::new(Mutex::new(LocalState {
            store,
            grant: None,
            active: None,
            cancelled: false,
            choosing: false,
            starting: false,
            process: Some(process),
            inference: Some(inference),
            models,
            bridge: None,
            city_status: CityStatus::default(),
        })));
        let bridge = Bridge::start(
            pilot.clone(),
            &profile.agent_id,
            "pilot-installed:qwen3.5:9b".into(),
        )
        .await
        .unwrap();
        let token = bridge.token.clone();
        let endpoint = bridge.endpoint.clone();
        lock(&pilot).unwrap().bridge = Some(bridge);
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(190))
            .build()
            .unwrap();
        assert_eq!(
            client
                .post(&endpoint)
                .body("{}")
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
        let input = json!({"threadId":"synthetic-connection","runId":"same-run","messages":[{"id":"m1","role":"user","content":"Представся одним коротким реченням: де ти працюєш і чи маєш доступ до файлів у цьому чаті?"}],"tools":[],"state":{},"context":[],"forwardedProps":{}});
        let send = |body: Value| client.post(&endpoint).bearer_auth(&token).json(&body);
        let first = send(input.clone()).send().await.unwrap();
        assert_eq!(first.status(), 200);
        assert_eq!(
            first.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
        let first = first.text().await.unwrap();
        assert!(
            first.contains("RUN_STARTED")
                && first.contains("TEXT_MESSAGE_CONTENT")
                && first.contains("RUN_FINISHED"),
            "The real model did not finish successfully"
        );
        let second = send(input.clone())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_eq!(first, second);
        let mut conflicting = input.clone();
        conflicting["messages"][0]["content"] = json!("Different request with reused run ID");
        let conflict = send(conflicting)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(conflict.contains("Conflicting replay refused"));
        assert_eq!(
            client
                .post(&endpoint)
                .bearer_auth(&token)
                .header("Origin", "http://localhost:3010")
                .json(&input)
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
        let tasks = lock(&pilot).unwrap().store.tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, "completed");
        assert!(tasks[0].report.is_none());
        let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.pilot-evidence");
        std::fs::create_dir_all(&evidence).unwrap();
        std::fs::write(evidence.join("city-protocol-proof.json"),serde_json::to_vec_pretty(&json!({"mode":"isolated protocol proof; not City registration or MicroDAO login", "task":tasks[0],"unauthenticated_refused":true,"browser_origin_refused":true,"replay_idempotent":true,"conflicting_replay_refused":true,"model":"qwen3.5:9b"})).unwrap()).unwrap();
        lock(&pilot).unwrap().bridge = None;
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(client.post(&endpoint).body("{}").send().await.is_err());
        lock(&pilot).unwrap().process = None;
        drop(pilot);
        std::fs::remove_dir_all(root).unwrap();
        println!("PASS: real local AG-UI response, saved result, bearer/origin refusal, idempotent/conflicting replay, listener shutdown. No City profile or MicroDAO account was written.");
    }
}
