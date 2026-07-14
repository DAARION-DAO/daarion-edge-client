use crate::inference::ndjson::NdjsonDecoder;
use crate::inference::policy::LocalEndpoint;
use crate::inference::provider::{
    InferenceProvider, InstalledProviderModel, ProviderChatRequest, ProviderHealth, RequestControl,
};
use crate::inference::types::InferenceError;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

pub struct OllamaProvider {
    endpoint: LocalEndpoint,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OllamaTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTag>,
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

    fn process_chat_record(
        record: Value,
        control: &RequestControl,
        output: &mut String,
        seen_done: &mut bool,
    ) -> Result<(), InferenceError> {
        if record.get("error").is_some() {
            return Err(InferenceError::ProviderProtocol(
                "Local provider reported an inference error".to_string(),
            ));
        }

        if let Some(content) = record
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
        {
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

        if record.get("done").and_then(Value::as_bool) == Some(true) {
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

    async fn list_installed_models(&self) -> Result<Vec<InstalledProviderModel>, InferenceError> {
        #[cfg(mobile)]
        {
            return Err(InferenceError::ProviderUnavailable);
        }
        #[cfg(not(mobile))]
        {
            let response = self
                .checked_response(
                    self.client
                        .get(self.endpoint.join("api/tags")?)
                        .send()
                        .await,
                )
                .await?;
            let payload = response.json::<OllamaTagsResponse>().await.map_err(|_| {
                InferenceError::ProviderProtocol("Local model inventory is malformed".to_string())
            })?;
            Ok(payload
                .models
                .into_iter()
                .map(|model| InstalledProviderModel {
                    provider_model_id: model.name,
                })
                .collect())
        }
    }

    async fn prepare_model(&self, provider_model_id: &str) -> Result<(), InferenceError> {
        #[cfg(mobile)]
        {
            let _ = provider_model_id;
            return Err(InferenceError::ProviderUnavailable);
        }
        #[cfg(not(mobile))]
        {
            let body = serde_json::json!({ "model": provider_model_id, "stream": false });
            let response = self
                .checked_response(
                    self.client
                        .post(self.endpoint.join("api/pull")?)
                        .json(&body)
                        .send()
                        .await,
                )
                .await?;
            let result = response.json::<Value>().await.map_err(|_| {
                InferenceError::ProviderProtocol(
                    "Local provider model preparation response is malformed".to_string(),
                )
            })?;
            if result.get("error").is_some()
                || result.get("status").and_then(Value::as_str) != Some("success")
            {
                return Err(InferenceError::ProviderProtocol(
                    "Local provider could not prepare the model".to_string(),
                ));
            }
            Ok(())
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
    use crate::inference::provider::{EventGate, EventSink};
    use crate::inference::types::{ChatMessage, InferenceEvent};
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::watch;

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

    async fn fixture_provider(
        status: &str,
        headers: &[(&str, &str)],
        body: &[u8],
        declared_length: Option<usize>,
    ) -> (OllamaProvider, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mut response = format!("HTTP/1.1 {status}\r\n");
        response.push_str(&format!(
            "Content-Length: {}\r\n",
            declared_length.unwrap_or(body.len())
        ));
        for (name, value) in headers {
            response.push_str(&format!("{name}: {value}\r\n"));
        }
        response.push_str("Connection: close\r\n\r\n");
        let response_head = response.into_bytes();
        let body = body.to_vec();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0u8; 8192];
            let _ = socket.read(&mut request).await;
            socket.write_all(&response_head).await.unwrap();
            socket.write_all(&body).await.unwrap();
            let _ = socket.shutdown().await;
        });
        let provider =
            OllamaProvider::new(LocalEndpoint::parse(&format!("http://{address}/")).unwrap())
                .unwrap();
        (provider, task)
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
        for (body, should_succeed) in [
            (b"{\"status\":\"success\"}".as_slice(), true),
            (b"{\"error\":\"private upstream detail\"}".as_slice(), false),
            (b"{}".as_slice(), false),
        ] {
            let (provider, task) = fixture_provider("200 OK", &[], body, None).await;
            let result = provider.prepare_model("fixture:1").await;
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
