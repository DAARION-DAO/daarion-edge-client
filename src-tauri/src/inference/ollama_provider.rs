use crate::inference::ndjson::NdjsonDecoder;
use crate::inference::policy::LocalEndpoint;
use crate::inference::provider::{
    InferenceProvider, OperationControl, ProviderChatRequest, ProviderHealth, RequestControl,
    VerifiedLocalModel,
};
use crate::inference::types::InferenceError;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_EVIDENCE_BYTES: usize = 1024 * 1024;

pub struct OllamaProvider {
    endpoint: LocalEndpoint,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OllamaCloudStatus {
    disabled: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaStatusResponse {
    cloud: OllamaCloudStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OllamaModelDetails {
    format: String,
    family: String,
    families: Vec<String>,
    parameter_size: String,
    quantization_level: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaTag {
    name: String,
    model: String,
    #[serde(default)]
    remote_model: String,
    #[serde(default)]
    remote_host: String,
    size: u64,
    digest: String,
    details: OllamaModelDetails,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTag>,
}

#[derive(Debug, Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    remote_model: String,
    #[serde(default)]
    remote_host: String,
    details: OllamaModelDetails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalModelEvidence {
    digest: String,
    size: u64,
    details: OllamaModelDetails,
}

#[derive(Serialize)]
struct OllamaChatBody<'a> {
    model: &'a str,
    messages: &'a [crate::inference::types::ChatMessage],
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: u32,
    temperature: f32,
}

impl OllamaProvider {
    pub fn new(endpoint: LocalEndpoint) -> Result<Self, InferenceError> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| {
                InferenceError::Internal(
                    "Local provider HTTP client could not be created".to_string(),
                )
            })?;
        Ok(Self { endpoint, client })
    }

    pub fn new_default() -> Result<Self, InferenceError> {
        Self::new(LocalEndpoint::default_ollama()?)
    }

    async fn checked_response(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Result<Response, InferenceError> {
        let response = response.map_err(|_| InferenceError::ProviderUnavailable)?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(InferenceError::ProviderProtocol(
                "Local provider rejected the request".to_string(),
            ))
        }
    }

    async fn send_controlled(
        &self,
        request: RequestBuilder,
        control: &mut OperationControl,
    ) -> Result<Response, InferenceError> {
        control.ensure_active()?;
        tokio::select! {
            biased;
            _ = control.cancelled() => Err(InferenceError::Cancelled),
            response = request.send() => response.map_err(|_| InferenceError::ProviderUnavailable),
        }
    }

    async fn decode_json_bounded<T: DeserializeOwned>(
        response: Response,
        mut control: OperationControl,
        failure: InferenceError,
    ) -> Result<T, InferenceError> {
        let mut stream = response.bytes_stream();
        let mut body = Vec::new();
        loop {
            let next = tokio::select! {
                biased;
                _ = control.cancelled() => return Err(InferenceError::Cancelled),
                next = stream.next() => next,
            };
            let Some(chunk) = next else {
                break;
            };
            let chunk = chunk.map_err(|_| failure.clone())?;
            if body.len().saturating_add(chunk.len()) > MAX_EVIDENCE_BYTES {
                return Err(failure);
            }
            body.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&body).map_err(|_| failure)
    }

    fn validate_details(details: &OllamaModelDetails) -> Result<(), InferenceError> {
        let required = [
            details.format.as_str(),
            details.family.as_str(),
            details.parameter_size.as_str(),
            details.quantization_level.as_str(),
        ];
        if required.iter().any(|value| value.trim().is_empty())
            || details.families.is_empty()
            || details
                .families
                .iter()
                .any(|family| family.trim().is_empty())
            || !details
                .families
                .iter()
                .any(|family| family == &details.family)
        {
            return Err(InferenceError::LocalModelUnverified);
        }
        Ok(())
    }

    fn normalize_digest(digest: &str) -> Result<String, InferenceError> {
        let Some(hex) = digest.strip_prefix("sha256:") else {
            return Err(InferenceError::LocalModelUnverified);
        };
        if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(InferenceError::LocalModelUnverified);
        }
        Ok(format!("sha256:{}", hex.to_ascii_lowercase()))
    }

    fn exact_local_evidence(
        payload: OllamaTagsResponse,
        provider_model_id: &str,
    ) -> Result<Option<LocalModelEvidence>, InferenceError> {
        let mut matching = payload
            .models
            .into_iter()
            .filter(|model| model.name == provider_model_id)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Ok(None);
        }
        if matching.len() != 1 {
            return Err(InferenceError::LocalModelUnverified);
        }
        let Some(model) = matching.pop() else {
            return Err(InferenceError::LocalModelUnverified);
        };
        if !model.remote_model.trim().is_empty() || !model.remote_host.trim().is_empty() {
            return Err(InferenceError::ModelNotLocal);
        }
        if model.model != provider_model_id || model.size == 0 {
            return Err(InferenceError::LocalModelUnverified);
        }
        Self::validate_details(&model.details)?;
        Ok(Some(LocalModelEvidence {
            digest: Self::normalize_digest(&model.digest)?,
            size: model.size,
            details: model.details,
        }))
    }

    async fn read_tags(
        &self,
        mut control: OperationControl,
    ) -> Result<OllamaTagsResponse, InferenceError> {
        let request = self.client.get(self.endpoint.join("api/tags")?);
        let response = self.send_controlled(request, &mut control).await?;
        if !response.status().is_success() {
            return Err(InferenceError::LocalModelUnverified);
        }
        Self::decode_json_bounded(response, control, InferenceError::LocalModelUnverified).await
    }

    async fn read_show(
        &self,
        provider_model_id: &str,
        mut control: OperationControl,
    ) -> Result<OllamaShowResponse, InferenceError> {
        let request = self
            .client
            .post(self.endpoint.join("api/show")?)
            .json(&serde_json::json!({ "model": provider_model_id, "verbose": false }));
        let response = self.send_controlled(request, &mut control).await?;
        if !response.status().is_success() {
            return Err(InferenceError::LocalModelUnverified);
        }
        Self::decode_json_bounded(response, control, InferenceError::LocalModelUnverified).await
    }

    fn process_chat_record(
        record: Value,
        control: &RequestControl,
        output: &mut String,
        seen_done: &mut bool,
    ) -> Result<(), InferenceError> {
        if *seen_done {
            return Err(InferenceError::ProviderProtocol(
                "Local provider sent data after stream completion".to_string(),
            ));
        }
        if record.get("error").is_some() {
            return Err(InferenceError::ProviderProtocol(
                "Local provider reported an inference error".to_string(),
            ));
        }

        let content = match record.get("message") {
            Some(message) => Some(message.get("content").and_then(Value::as_str).ok_or_else(
                || {
                    InferenceError::ProviderProtocol(
                        "Local provider returned a malformed streaming record".to_string(),
                    )
                },
            )?),
            None => None,
        };
        let done = match record.get("done") {
            Some(value) => Some(value.as_bool().ok_or_else(|| {
                InferenceError::ProviderProtocol(
                    "Local provider returned a malformed streaming record".to_string(),
                )
            })?),
            None => None,
        };
        if content.is_none() && done.is_none() {
            return Err(InferenceError::ProviderProtocol(
                "Local provider returned a malformed streaming record".to_string(),
            ));
        }

        if let Some(content) = content {
            if output.len().saturating_add(content.len()) > MAX_OUTPUT_BYTES {
                return Err(InferenceError::ProviderProtocol(
                    "Local provider output exceeded the safety limit".to_string(),
                ));
            }
            output.push_str(content);
            if !content.is_empty() {
                control.emit_token(content.to_string())?;
            }
        }

        if done == Some(true) {
            *seen_done = true;
        }
        Ok(())
    }

    async fn drain_records(
        response: Response,
        control: &RequestControl,
    ) -> Result<String, InferenceError> {
        let mut stream = response.bytes_stream();
        let mut decoder = NdjsonDecoder::default();
        let mut output = String::new();
        let mut seen_done = false;

        while let Some(chunk) = stream.next().await {
            control.ensure_active()?;
            let chunk = chunk.map_err(|_| {
                InferenceError::ProviderProtocol(
                    "Local provider stream was interrupted".to_string(),
                )
            })?;
            for record in decoder.push(&chunk)? {
                Self::process_chat_record(record, control, &mut output, &mut seen_done)?;
            }
        }

        for record in decoder.finish()? {
            Self::process_chat_record(record, control, &mut output, &mut seen_done)?;
        }

        if !seen_done {
            return Err(InferenceError::ProviderProtocol(
                "Local provider stream ended without completion".to_string(),
            ));
        }

        Ok(output)
    }

    fn process_prepare_record(
        record: Value,
        seen_success: &mut bool,
    ) -> Result<(), InferenceError> {
        if *seen_success {
            return Err(InferenceError::ProviderProtocol(
                "Local provider sent data after model preparation completed".to_string(),
            ));
        }
        if record.get("error").is_some() {
            return Err(InferenceError::ProviderProtocol(
                "Local provider reported a model preparation error".to_string(),
            ));
        }
        let status = record
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                InferenceError::ProviderProtocol(
                    "Local provider returned a malformed model preparation record".to_string(),
                )
            })?;
        if status.is_empty() {
            return Err(InferenceError::ProviderProtocol(
                "Local provider returned a malformed model preparation record".to_string(),
            ));
        }
        if status == "success" {
            *seen_success = true;
        }
        Ok(())
    }

    async fn drain_prepare_records(
        response: Response,
        mut control: OperationControl,
    ) -> Result<(), InferenceError> {
        let mut stream = response.bytes_stream();
        let mut decoder = NdjsonDecoder::default();
        let mut seen_success = false;

        loop {
            let next = tokio::select! {
                biased;
                _ = control.cancelled() => return Err(InferenceError::Cancelled),
                next = stream.next() => next,
            };
            let Some(chunk) = next else {
                break;
            };
            control.ensure_active()?;
            let chunk = chunk.map_err(|_| {
                InferenceError::ProviderProtocol(
                    "Local provider model preparation stream was interrupted".to_string(),
                )
            })?;
            for record in decoder.push(&chunk)? {
                Self::process_prepare_record(record, &mut seen_success)?;
            }
        }

        control.ensure_active()?;
        for record in decoder.finish()? {
            Self::process_prepare_record(record, &mut seen_success)?;
        }
        if !seen_success {
            return Err(InferenceError::ProviderProtocol(
                "Local provider model preparation ended without completion".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl InferenceProvider for OllamaProvider {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn endpoint(&self) -> String {
        self.endpoint.display()
    }

    async fn health(&self) -> Result<ProviderHealth, InferenceError> {
        #[cfg(mobile)]
        {
            return Ok(ProviderHealth {
                available: false,
                detail: "Local Ollama inference is unavailable on this mobile build".to_string(),
            });
        }
        #[cfg(not(mobile))]
        {
            let url = self.endpoint.join("")?;
            let available = self
                .client
                .get(url)
                .send()
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false);
            Ok(ProviderHealth {
                available,
                detail: if available {
                    "Local Ollama endpoint is available".to_string()
                } else {
                    "Local Ollama endpoint is unavailable".to_string()
                },
            })
        }
    }

    async fn verify_local_execution(
        &self,
        mut control: OperationControl,
    ) -> Result<(), InferenceError> {
        #[cfg(mobile)]
        {
            let _ = control;
            return Err(InferenceError::ProviderCapabilityUnsupported);
        }
        #[cfg(not(mobile))]
        {
            let request = self.client.get(self.endpoint.join("api/status")?);
            let response = self.send_controlled(request, &mut control).await?;
            if !response.status().is_success() {
                return Err(InferenceError::ProviderCapabilityUnsupported);
            }
            let payload: OllamaStatusResponse = Self::decode_json_bounded(
                response,
                control,
                InferenceError::ProviderCapabilityUnsupported,
            )
            .await?;
            if !payload.cloud.disabled {
                return Err(InferenceError::LocalOnlyNotEnforced);
            }
            Ok(())
        }
    }

    async fn verify_local_model(
        &self,
        provider_model_id: &str,
        control: OperationControl,
    ) -> Result<Option<VerifiedLocalModel>, InferenceError> {
        #[cfg(mobile)]
        {
            let _ = (provider_model_id, control);
            return Err(InferenceError::ProviderCapabilityUnsupported);
        }
        #[cfg(not(mobile))]
        {
            control.ensure_active()?;
            let initial = Self::exact_local_evidence(
                self.read_tags(control.clone()).await?,
                provider_model_id,
            )?;
            let Some(initial) = initial else {
                return Ok(None);
            };

            let show = self.read_show(provider_model_id, control.clone()).await?;
            if !show.remote_model.trim().is_empty() || !show.remote_host.trim().is_empty() {
                return Err(InferenceError::ModelNotLocal);
            }
            Self::validate_details(&show.details)?;
            if show.details != initial.details {
                return Err(InferenceError::LocalModelUnverified);
            }

            let final_evidence =
                Self::exact_local_evidence(self.read_tags(control).await?, provider_model_id)?
                    .ok_or(InferenceError::LocalModelUnverified)?;
            if final_evidence != initial {
                return Err(InferenceError::LocalModelUnverified);
            }

            Ok(Some(VerifiedLocalModel {
                provider_model_id: provider_model_id.to_string(),
                digest: initial.digest,
            }))
        }
    }

    async fn prepare_model(
        &self,
        provider_model_id: &str,
        mut control: OperationControl,
    ) -> Result<(), InferenceError> {
        #[cfg(mobile)]
        {
            let _ = (provider_model_id, control);
            return Err(InferenceError::ProviderUnavailable);
        }
        #[cfg(not(mobile))]
        {
            control.ensure_active()?;
            let body = serde_json::json!({ "model": provider_model_id, "stream": true });
            let request = self
                .client
                .post(self.endpoint.join("api/pull")?)
                .json(&body);
            let response = tokio::select! {
                biased;
                _ = control.cancelled() => return Err(InferenceError::Cancelled),
                response = request.send() => response,
            };
            let response = self.checked_response(response).await?;
            Self::drain_prepare_records(response, control).await
        }
    }

    async fn chat(
        &self,
        request: ProviderChatRequest,
        control: RequestControl,
    ) -> Result<String, InferenceError> {
        #[cfg(mobile)]
        {
            let _ = (request, control);
            return Err(InferenceError::ProviderUnavailable);
        }
        #[cfg(not(mobile))]
        {
            control.ensure_active()?;
            let body = OllamaChatBody {
                model: &request.provider_model_id,
                messages: &request.messages,
                stream: true,
                options: OllamaOptions {
                    num_predict: request.max_tokens,
                    temperature: request.temperature,
                },
            };
            let response = self
                .checked_response(
                    self.client
                        .post(self.endpoint.join("api/chat")?)
                        .json(&body)
                        .send()
                        .await,
                )
                .await?;
            Self::drain_records(response, &control).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::model_resolver::ModelResolver;
    use crate::inference::provider::{EventGate, EventSink};
    use crate::inference::service::{InferenceService, ServiceLimits};
    use crate::inference::types::{
        ChatMessage, InferenceEvent, InferencePublicError, InferenceRequest,
        PrepareLocalModelRequest,
    };
    use std::future::pending;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::watch;
    use tokio::time::timeout;
    use uuid::Uuid;

    const SENTINEL_PROMPT: &str = "SENTINEL_PROMPT_MUST_NOT_EGRESS";
    const DIGEST_A: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<InferenceEvent>>);

    impl EventSink for RecordingSink {
        fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn chat_request() -> ProviderChatRequest {
        ProviderChatRequest {
            provider_model_id: "fixture:1".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "fixture".to_string(),
            }],
            max_tokens: 16,
            temperature: 0.0,
        }
    }

    fn control() -> (RequestControl, Arc<RecordingSink>) {
        let (_sender, receiver) = watch::channel(false);
        let sink = Arc::new(RecordingSink::default());
        let gate = Arc::new(EventGate::new(sink.clone()));
        (
            RequestControl::new("fixture-request".to_string(), receiver, gate),
            sink,
        )
    }

    fn operation_control() -> (watch::Sender<bool>, OperationControl) {
        let (sender, receiver) = watch::channel(false);
        (sender, OperationControl::new(receiver))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SanitizedRequest {
        path: String,
        model: Option<String>,
        json_keys: Vec<String>,
        contains_sentinel: bool,
    }

    enum ScriptedReply {
        Response {
            expected_path: String,
            status: String,
            headers: Vec<(String, String)>,
            body: Vec<u8>,
            declared_length: Option<usize>,
        },
        StallHeaders {
            expected_path: String,
        },
    }

    impl ScriptedReply {
        fn json(expected_path: &str, body: Value) -> Self {
            Self::Response {
                expected_path: expected_path.to_string(),
                status: "200 OK".to_string(),
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: serde_json::to_vec(&body).unwrap(),
                declared_length: None,
            }
        }

        fn raw(expected_path: &str, status: &str, body: &[u8]) -> Self {
            Self::Response {
                expected_path: expected_path.to_string(),
                status: status.to_string(),
                headers: Vec::new(),
                body: body.to_vec(),
                declared_length: None,
            }
        }

        fn ndjson(expected_path: &str, body: &[u8]) -> Self {
            Self::Response {
                expected_path: expected_path.to_string(),
                status: "200 OK".to_string(),
                headers: vec![(
                    "Content-Type".to_string(),
                    "application/x-ndjson".to_string(),
                )],
                body: body.to_vec(),
                declared_length: None,
            }
        }

        fn path(&self) -> &str {
            match self {
                Self::Response { expected_path, .. } | Self::StallHeaders { expected_path } => {
                    expected_path
                }
            }
        }
    }

    fn header_end(bytes: &[u8]) -> Option<usize> {
        bytes.windows(4).position(|window| window == b"\r\n\r\n")
    }

    async fn read_sanitized_request(socket: &mut tokio::net::TcpStream) -> SanitizedRequest {
        let mut bytes = Vec::new();
        let mut content_length = None;
        loop {
            let mut chunk = [0u8; 2048];
            let count = socket.read(&mut chunk).await.unwrap();
            assert!(count > 0, "client closed before sending a complete request");
            bytes.extend_from_slice(&chunk[..count]);
            if let Some(end) = header_end(&bytes) {
                if content_length.is_none() {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    content_length = Some(
                        headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().ok())
                                    .flatten()
                            })
                            .unwrap_or(0),
                    );
                }
                if bytes.len() >= end + 4 + content_length.unwrap_or(0) {
                    break;
                }
            }
        }

        let end = header_end(&bytes).unwrap();
        let headers = String::from_utf8_lossy(&bytes[..end]);
        let path = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("")
            .to_string();
        let length = content_length.unwrap_or(0);
        let body = &bytes[end + 4..end + 4 + length];
        let parsed = serde_json::from_slice::<Value>(body).ok();
        let model = parsed
            .as_ref()
            .and_then(|value| value.get("model"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let mut json_keys = parsed
            .as_ref()
            .and_then(Value::as_object)
            .map(|object| object.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        json_keys.sort();
        SanitizedRequest {
            path,
            model,
            json_keys,
            contains_sentinel: String::from_utf8_lossy(body).contains(SENTINEL_PROMPT),
        }
    }

    async fn scripted_provider(
        replies: Vec<ScriptedReply>,
    ) -> (
        OllamaProvider,
        Arc<Mutex<Vec<SanitizedRequest>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let records = Arc::new(Mutex::new(Vec::new()));
        let task_records = Arc::clone(&records);
        let task = tokio::spawn(async move {
            for reply in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let request = read_sanitized_request(&mut socket).await;
                assert_eq!(request.path, reply.path());
                task_records.lock().unwrap().push(request);
                match reply {
                    ScriptedReply::Response {
                        status,
                        headers,
                        body,
                        declared_length,
                        ..
                    } => {
                        let mut response = format!("HTTP/1.1 {status}\r\n");
                        response.push_str(&format!(
                            "Content-Length: {}\r\n",
                            declared_length.unwrap_or(body.len())
                        ));
                        for (name, value) in headers {
                            response.push_str(&format!("{name}: {value}\r\n"));
                        }
                        response.push_str("Connection: close\r\n\r\n");
                        socket.write_all(response.as_bytes()).await.unwrap();
                        socket.write_all(&body).await.unwrap();
                        let _ = socket.shutdown().await;
                    }
                    ScriptedReply::StallHeaders { .. } => pending::<()>().await,
                }
            }
        });
        let provider =
            OllamaProvider::new(LocalEndpoint::parse(&format!("http://{address}/")).unwrap())
                .unwrap();
        (provider, records, task)
    }

    async fn fixture_provider(
        status: &str,
        headers: &[(&str, &str)],
        body: &[u8],
        declared_length: Option<usize>,
    ) -> (OllamaProvider, tokio::task::JoinHandle<()>) {
        fixture_provider_for("/api/chat", status, headers, body, declared_length).await
    }

    async fn fixture_provider_for(
        expected_path: &str,
        status: &str,
        headers: &[(&str, &str)],
        body: &[u8],
        declared_length: Option<usize>,
    ) -> (OllamaProvider, tokio::task::JoinHandle<()>) {
        let reply = ScriptedReply::Response {
            expected_path: expected_path.to_string(),
            status: status.to_string(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
            body: body.to_vec(),
            declared_length,
        };
        let (provider, _records, task) = scripted_provider(vec![reply]).await;
        (provider, task)
    }

    fn details(family: &str) -> Value {
        serde_json::json!({
            "format": "gguf",
            "family": family,
            "families": [family],
            "parameter_size": "2B",
            "quantization_level": "Q4_K_M"
        })
    }

    fn tag(
        provider_model_id: &str,
        digest: &str,
        size: u64,
        remote_model: &str,
        remote_host: &str,
        model_details: Value,
    ) -> Value {
        serde_json::json!({
            "name": provider_model_id,
            "model": provider_model_id,
            "remote_model": remote_model,
            "remote_host": remote_host,
            "size": size,
            "digest": digest,
            "details": model_details
        })
    }

    fn tags(models: Vec<Value>) -> ScriptedReply {
        ScriptedReply::json("/api/tags", serde_json::json!({ "models": models }))
    }

    fn local_tags(provider_model_id: &str, digest: &str) -> ScriptedReply {
        tags(vec![tag(
            provider_model_id,
            digest,
            2048,
            "",
            "",
            details("qwen"),
        )])
    }

    fn show(remote_model: &str, remote_host: &str, model_details: Value) -> ScriptedReply {
        ScriptedReply::json(
            "/api/show",
            serde_json::json!({
                "remote_model": remote_model,
                "remote_host": remote_host,
                "details": model_details
            }),
        )
    }

    fn local_show() -> ScriptedReply {
        show("", "", details("qwen"))
    }

    fn cloud_status(disabled: bool) -> ScriptedReply {
        ScriptedReply::json(
            "/api/status",
            serde_json::json!({ "cloud": { "disabled": disabled } }),
        )
    }

    fn verified_model_replies(provider_model_id: &str) -> Vec<ScriptedReply> {
        vec![
            local_tags(provider_model_id, DIGEST_A),
            local_show(),
            local_tags(provider_model_id, DIGEST_A),
        ]
    }

    fn listing_replies() -> Vec<ScriptedReply> {
        let provider_models = [
            "qwen2.5:0.5b",
            "qwen3.5:0.8b",
            "qwen3.5:2b",
            "qwen3.5:4b",
            "qwen3.5:14b",
            "gemma4:e2b",
            "gemma4:e4b",
            "gemma4:26b",
        ];
        let mut replies = vec![cloud_status(true)];
        for provider_model_id in provider_models {
            if provider_model_id == "qwen3.5:2b" {
                replies.extend(verified_model_replies(provider_model_id));
            } else {
                replies.push(tags(Vec::new()));
            }
        }
        replies
    }

    fn sentinel_request() -> InferenceRequest {
        InferenceRequest {
            request_id: Uuid::new_v4().to_string(),
            canonical_model_id: "qwen35-2b-stable".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: SENTINEL_PROMPT.to_string(),
            }],
            max_tokens: 16,
            temperature: 0.0,
            stream: true,
        }
    }

    fn inference_service(provider: OllamaProvider, limits: ServiceLimits) -> InferenceService {
        InferenceService::new(
            Arc::new(provider),
            ModelResolver::from_bundled_registry().unwrap(),
            limits,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn daemon_cloud_policy_is_explicit_and_fail_closed() {
        let cases = vec![
            (cloud_status(true), Ok(())),
            (
                cloud_status(false),
                Err(InferenceError::LocalOnlyNotEnforced),
            ),
            (
                ScriptedReply::json("/api/status", serde_json::json!({ "cloud": {} })),
                Err(InferenceError::ProviderCapabilityUnsupported),
            ),
            (
                ScriptedReply::raw("/api/status", "200 OK", b"not-json"),
                Err(InferenceError::ProviderCapabilityUnsupported),
            ),
            (
                ScriptedReply::raw("/api/status", "404 Not Found", b"unsupported"),
                Err(InferenceError::ProviderCapabilityUnsupported),
            ),
        ];

        for (reply, expected) in cases {
            let (provider, records, task) = scripted_provider(vec![reply]).await;
            let (_sender, control) = operation_control();
            assert_eq!(provider.verify_local_execution(control).await, expected);
            task.await.unwrap();
            let records = records.lock().unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].path, "/api/status");
            assert!(!records[0].contains_sentinel);
        }
    }

    #[tokio::test]
    async fn stalled_daemon_cloud_policy_uses_the_service_probe_deadline() {
        let replies = vec![
            ScriptedReply::raw("/", "200 OK", b"Ollama is running"),
            ScriptedReply::StallHeaders {
                expected_path: "/api/status".to_string(),
            },
        ];
        let (provider, records, task) = scripted_provider(replies).await;
        let service = inference_service(
            provider,
            ServiceLimits {
                probe_timeout: Duration::from_millis(20),
                ..ServiceLimits::default()
            },
        );
        let result = timeout(Duration::from_millis(500), service.status())
            .await
            .expect("outer test guard elapsed");
        assert_eq!(result, Err(InferenceError::TimedOut));
        let records = records.lock().unwrap();
        assert_eq!(
            records
                .iter()
                .map(|record| record.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/", "/api/status"]
        );
        assert!(records.iter().all(|record| !record.contains_sentinel));
        drop(records);
        task.abort();
        let _ = task.await;
    }

    #[tokio::test]
    async fn stable_local_artifact_evidence_produces_a_verified_model() {
        let (provider, records, task) =
            scripted_provider(verified_model_replies("qwen3.5:2b")).await;
        let (_sender, control) = operation_control();
        let verified = provider
            .verify_local_model("qwen3.5:2b", control)
            .await
            .unwrap()
            .unwrap();
        task.await.unwrap();

        assert_eq!(verified.provider_model_id, "qwen3.5:2b");
        assert_eq!(verified.digest, DIGEST_A);
        let records = records.lock().unwrap();
        assert_eq!(
            records
                .iter()
                .map(|record| record.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/api/tags", "/api/show", "/api/tags"]
        );
        assert!(records.iter().all(|record| !record.contains_sentinel));
        assert_eq!(records[1].model.as_deref(), Some("qwen3.5:2b"));
    }

    #[tokio::test]
    async fn remote_markers_and_copied_aliases_are_rejected() {
        let cases = vec![
            vec![tags(vec![tag(
                "qwen3.5:2b",
                DIGEST_A,
                2048,
                "cloud/source",
                "",
                details("qwen"),
            )])],
            vec![tags(vec![tag(
                "qwen3.5:2b",
                DIGEST_A,
                2048,
                "",
                "cloud.invalid",
                details("qwen"),
            )])],
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                show("cloud/source", "", details("qwen")),
            ],
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                show("", "cloud.invalid", details("qwen")),
            ],
        ];

        for replies in cases {
            let (provider, records, task) = scripted_provider(replies).await;
            let (_sender, control) = operation_control();
            assert_eq!(
                provider.verify_local_model("qwen3.5:2b", control).await,
                Err(InferenceError::ModelNotLocal)
            );
            task.await.unwrap();
            assert!(records
                .lock()
                .unwrap()
                .iter()
                .all(|record| !record.contains_sentinel));
        }
    }

    #[tokio::test]
    async fn malformed_or_ambiguous_artifact_evidence_is_rejected() {
        let malformed_details = serde_json::json!({
            "format": "gguf",
            "family": "qwen",
            "families": ["qwen"],
            "parameter_size": "2B"
        });
        let local = tag("qwen3.5:2b", DIGEST_A, 2048, "", "", details("qwen"));
        let cases = vec![
            vec![ScriptedReply::json(
                "/api/tags",
                serde_json::json!({
                    "models": [{
                        "name": "qwen3.5:2b",
                        "model": "qwen3.5:2b",
                        "remote_model": "",
                        "remote_host": "",
                        "size": 2048,
                        "details": details("qwen")
                    }]
                }),
            )],
            vec![tags(vec![tag(
                "qwen3.5:2b",
                DIGEST_A,
                0,
                "",
                "",
                details("qwen"),
            )])],
            vec![tags(vec![tag(
                "qwen3.5:2b",
                "not-a-digest",
                2048,
                "",
                "",
                details("qwen"),
            )])],
            vec![tags(vec![tag(
                "qwen3.5:2b",
                DIGEST_A,
                2048,
                "",
                "",
                malformed_details,
            )])],
            vec![tags(vec![local.clone(), local])],
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                show("", "", details("other-family")),
            ],
        ];

        for replies in cases {
            let (provider, records, task) = scripted_provider(replies).await;
            let (_sender, control) = operation_control();
            assert_eq!(
                provider.verify_local_model("qwen3.5:2b", control).await,
                Err(InferenceError::LocalModelUnverified)
            );
            task.await.unwrap();
            assert!(records
                .lock()
                .unwrap()
                .iter()
                .all(|record| !record.contains_sentinel));
        }
    }

    #[tokio::test]
    async fn model_evidence_must_remain_stable_across_both_tag_reads() {
        let cases = vec![
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                local_show(),
                tags(Vec::new()),
            ],
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                local_show(),
                local_tags("qwen3.5:2b", DIGEST_B),
            ],
            vec![
                local_tags("qwen3.5:2b", DIGEST_A),
                local_show(),
                tags(vec![tag(
                    "qwen3.5:2b",
                    DIGEST_A,
                    2048,
                    "cloud/source",
                    "",
                    details("qwen"),
                )]),
            ],
        ];

        for replies in cases {
            let (provider, records, task) = scripted_provider(replies).await;
            let (_sender, control) = operation_control();
            assert!(matches!(
                provider.verify_local_model("qwen3.5:2b", control).await,
                Err(InferenceError::LocalModelUnverified | InferenceError::ModelNotLocal)
            ));
            task.await.unwrap();
            assert!(records
                .lock()
                .unwrap()
                .iter()
                .all(|record| !record.contains_sentinel));
        }
    }

    #[tokio::test]
    async fn rejected_chat_never_calls_chat_or_transmits_the_sentinel_prompt() {
        let (provider, records, task) = scripted_provider(vec![cloud_status(false)]).await;
        let service = inference_service(provider, ServiceLimits::default());
        let sink = Arc::new(RecordingSink::default());
        let error = service
            .run(sentinel_request(), sink.clone())
            .await
            .unwrap_err();
        task.await.unwrap();

        assert_eq!(error, InferenceError::LocalOnlyNotEnforced);
        let public = InferencePublicError::from(error);
        assert!(!public.message.contains(SENTINEL_PROMPT));
        let records = records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path, "/api/status");
        assert!(!records[0].contains_sentinel);
        assert!(!records.iter().any(|record| record.path == "/api/chat"));
        assert!(matches!(
            sink.0.lock().unwrap().last(),
            Some(InferenceEvent::Failed { code, error, .. })
                if code == "local_only_not_enforced"
                    && !error.contains(SENTINEL_PROMPT)
        ));
    }

    #[tokio::test]
    async fn cancellation_during_policy_verification_prevents_prompt_transmission() {
        let (provider, records, task) = scripted_provider(vec![ScriptedReply::StallHeaders {
            expected_path: "/api/status".to_string(),
        }])
        .await;
        let service = Arc::new(inference_service(
            provider,
            ServiceLimits {
                request_timeout: Duration::from_secs(2),
                ..ServiceLimits::default()
            },
        ));
        let request = sentinel_request();
        let request_id = request.request_id.clone();
        let task_service = Arc::clone(&service);
        let run_task = tokio::spawn(async move {
            task_service
                .run(request, Arc::new(RecordingSink::default()))
                .await
        });

        for _ in 0..100 {
            if !records.lock().unwrap().is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(records.lock().unwrap().len(), 1);
        assert!(service.cancel(&request_id).unwrap());
        assert_eq!(
            timeout(Duration::from_millis(500), run_task)
                .await
                .expect("outer test guard elapsed")
                .unwrap(),
            Err(InferenceError::Cancelled)
        );
        let records = records.lock().unwrap();
        assert_eq!(records[0].path, "/api/status");
        assert!(!records[0].contains_sentinel);
        drop(records);
        task.abort();
        let _ = task.await;
    }

    #[tokio::test]
    async fn verified_local_chat_calls_chat_exactly_once_after_evidence() {
        let mut replies = vec![cloud_status(true)];
        replies.extend(verified_model_replies("qwen3.5:2b"));
        replies.push(ScriptedReply::ndjson(
            "/api/chat",
            b"{\"message\":{\"content\":\"local\"}}\n{\"done\":true}\n",
        ));
        let (provider, records, task) = scripted_provider(replies).await;
        let service = inference_service(provider, ServiceLimits::default());
        let response = service
            .run(sentinel_request(), Arc::new(RecordingSink::default()))
            .await
            .unwrap();
        task.await.unwrap();

        assert_eq!(response.output_text, "local");
        let records = records.lock().unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record.path == "/api/chat")
                .count(),
            1
        );
        let chat_index = records
            .iter()
            .position(|record| record.path == "/api/chat")
            .unwrap();
        assert_eq!(chat_index, records.len() - 1);
        assert!(records[chat_index].contains_sentinel);
        assert!(records[..chat_index]
            .iter()
            .all(|record| !record.contains_sentinel));
        assert_eq!(records[1].json_keys, Vec::<String>::new());
        assert_eq!(
            records[2].json_keys,
            vec!["model".to_string(), "verbose".to_string()]
        );
        assert!(!records[2].json_keys.contains(&"messages".to_string()));
    }

    #[tokio::test]
    async fn stale_listing_cannot_authorize_changed_cloud_or_model_state() {
        for model_changes in [false, true] {
            let mut replies = listing_replies();
            if model_changes {
                replies.push(cloud_status(true));
                replies.push(tags(vec![tag(
                    "qwen3.5:2b",
                    DIGEST_A,
                    2048,
                    "cloud/source",
                    "",
                    details("qwen"),
                )]));
            } else {
                replies.push(cloud_status(false));
            }
            let (provider, records, task) = scripted_provider(replies).await;
            let service = inference_service(provider, ServiceLimits::default());
            let models = service.models().await.unwrap();
            assert!(models.iter().any(|model| {
                model.canonical_model_id == "qwen35-2b-stable" && model.installed
            }));

            let error = service
                .run(sentinel_request(), Arc::new(RecordingSink::default()))
                .await
                .unwrap_err();
            task.await.unwrap();
            assert_eq!(
                error,
                if model_changes {
                    InferenceError::ModelNotLocal
                } else {
                    InferenceError::LocalOnlyNotEnforced
                }
            );
            let records = records.lock().unwrap();
            assert!(!records.iter().any(|record| record.path == "/api/chat"));
            assert!(records.iter().all(|record| !record.contains_sentinel));
        }
    }

    #[tokio::test]
    async fn preparation_requires_verified_postflight_before_success() {
        let cases = vec![
            (
                {
                    let mut replies = vec![
                        cloud_status(true),
                        ScriptedReply::ndjson("/api/pull", b"{\"status\":\"success\"}\n"),
                        cloud_status(true),
                    ];
                    replies.extend(verified_model_replies("qwen3.5:2b"));
                    replies
                },
                true,
            ),
            (
                vec![
                    cloud_status(true),
                    ScriptedReply::ndjson("/api/pull", b"{\"status\":\"success\"}\n"),
                    cloud_status(true),
                    tags(vec![tag(
                        "qwen3.5:2b",
                        DIGEST_A,
                        2048,
                        "cloud/source",
                        "",
                        details("qwen"),
                    )]),
                ],
                false,
            ),
        ];

        for (replies, should_succeed) in cases {
            let (provider, records, task) = scripted_provider(replies).await;
            let service = inference_service(provider, ServiceLimits::default());
            let result = service
                .prepare_model(PrepareLocalModelRequest {
                    request_id: Uuid::new_v4().to_string(),
                    canonical_model_id: "qwen35-2b-stable".to_string(),
                })
                .await;
            task.await.unwrap();
            assert_eq!(result.is_ok(), should_succeed);
            if let Err(error) = result {
                assert_eq!(error, InferenceError::ModelNotLocal);
                assert_eq!(InferencePublicError::from(error).code, "model_not_local");
            }
            let records = records.lock().unwrap();
            assert_eq!(
                records
                    .iter()
                    .filter(|record| record.path == "/api/pull")
                    .count(),
                1
            );
            assert!(records.iter().all(|record| !record.contains_sentinel));
        }
    }

    #[tokio::test]
    async fn unavailable_loopback_provider_is_reported_without_fallback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let provider =
            OllamaProvider::new(LocalEndpoint::parse(&format!("http://{address}/")).unwrap())
                .unwrap();
        let health = provider.health().await.unwrap();
        assert!(!health.available);
    }

    #[tokio::test]
    async fn valid_done_record_and_final_unterminated_record_are_processed() {
        let body = b"{\"message\":{\"content\":\"local\"}}\n{\"done\":true}";
        let (provider, task) = fixture_provider("200 OK", &[], body, None).await;
        let (control, sink) = control();
        let output = provider.chat(chat_request(), control).await.unwrap();
        task.await.unwrap();
        assert_eq!(output, "local");
        assert!(matches!(
            sink.0.lock().unwrap().as_slice(),
            [InferenceEvent::Token { content, .. }] if content == "local"
        ));
    }

    #[tokio::test]
    async fn malformed_error_and_premature_eof_never_become_success() {
        for body in [
            b"not-json\n".as_slice(),
            b"{\"error\":\"sensitive provider detail\"}\n".as_slice(),
            b"{\"message\":{\"content\":\"partial\"}}\n".as_slice(),
            b"{}\n{\"done\":true}\n".as_slice(),
            b"{\"message\":{},\"done\":false}\n".as_slice(),
            b"{\"done\":\"true\"}\n".as_slice(),
            b"{\"done\":true}\n{\"message\":{\"content\":\"late\"}}\n".as_slice(),
        ] {
            let (provider, task) = fixture_provider("200 OK", &[], body, None).await;
            let (control, _) = control();
            let result = provider.chat(chat_request(), control).await;
            task.await.unwrap();
            let error = result.unwrap_err();
            assert!(matches!(error, InferenceError::ProviderProtocol(_)));
            assert!(!error.to_string().contains("sensitive provider detail"));
        }
    }

    #[tokio::test]
    async fn http_error_redirect_and_disconnect_are_controlled_failures() {
        let cases = [
            ("500 Internal Server Error", Vec::new(), Vec::new(), None),
            (
                "302 Found",
                vec![("Location", "https://example.com/")],
                Vec::new(),
                None,
            ),
            (
                "200 OK",
                Vec::new(),
                b"{\"done\":true}".to_vec(),
                Some(1024),
            ),
        ];

        for (status, headers, body, declared_length) in cases {
            let (provider, task) = fixture_provider(status, &headers, &body, declared_length).await;
            let (control, _) = control();
            let result = provider.chat(chat_request(), control).await;
            task.await.unwrap();
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn model_preparation_requires_explicit_success_without_leaking_errors() {
        let mut oversized = b"{\"status\":\"".to_vec();
        oversized.extend(vec![b'x'; 1024 * 1024]);
        oversized.extend_from_slice(b"\"}\n");
        let cases = vec![
            (
                b"{\"status\":\"pulling manifest\"}\n{\"status\":\"success\"}\n".to_vec(),
                true,
            ),
            (b"{\"status\":\"success\"}".to_vec(), true),
            (b"{\"error\":\"private upstream detail\"}\n".to_vec(), false),
            (b"{\"status\":\"pulling manifest\"}\n".to_vec(), false),
            (
                b"{\"status\":\"success\"}\n{\"status\":\"late\"}\n".to_vec(),
                false,
            ),
            (b"{}\n".to_vec(), false),
            (b"not-json\n".to_vec(), false),
            (oversized, false),
        ];

        for (body, should_succeed) in cases {
            let (provider, task) =
                fixture_provider_for("/api/pull", "200 OK", &[], &body, None).await;
            let (_sender, control) = operation_control();
            let result = provider.prepare_model("fixture:1", control).await;
            task.await.unwrap();
            assert_eq!(result.is_ok(), should_succeed);
            if let Err(error) = result {
                assert!(!error.to_string().contains("private upstream detail"));
            }
        }
    }

    #[test]
    fn response_size_limit_is_enforced_before_emission() {
        let (control, sink) = control();
        let mut output = String::new();
        let mut seen_done = false;
        let record = serde_json::json!({
            "message": { "content": "x".repeat(MAX_OUTPUT_BYTES + 1) },
            "done": true
        });
        assert!(matches!(
            OllamaProvider::process_chat_record(record, &control, &mut output, &mut seen_done),
            Err(InferenceError::ProviderProtocol(_))
        ));
        assert!(sink.0.lock().unwrap().is_empty());
    }
}
