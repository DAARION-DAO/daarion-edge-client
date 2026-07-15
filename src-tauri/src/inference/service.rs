use crate::inference::model_resolver::ModelResolver;
use crate::inference::policy::LocalEndpoint;
use crate::inference::provider::{
    EventGate, EventSink, InferenceProvider, OperationControl, ProviderChatRequest, RequestControl,
    VerifiedLocalModel,
};
use crate::inference::types::{
    InferenceError, InferenceEvent, InferenceModelSummary, InferencePolicy, InferenceRequest,
    InferenceResponse, InferenceStatus, ModelPreparationResponse, PrepareLocalModelRequest,
};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{watch, Semaphore};
use tokio::time::{sleep_until, timeout_at, Instant as TokioInstant};
use uuid::Uuid;

const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_TOKENS: u32 = 4096;
const MAX_CONCURRENT_REQUESTS: usize = 2;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PREPARE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone)]
pub struct ServiceLimits {
    pub max_prompt_bytes: usize,
    pub max_tokens: u32,
    pub request_timeout: Duration,
    pub probe_timeout: Duration,
    pub prepare_timeout: Duration,
    pub max_concurrent_requests: usize,
}

impl Default for ServiceLimits {
    fn default() -> Self {
        Self {
            max_prompt_bytes: MAX_PROMPT_BYTES,
            max_tokens: MAX_TOKENS,
            request_timeout: REQUEST_TIMEOUT,
            probe_timeout: PROBE_TIMEOUT,
            prepare_timeout: PREPARE_TIMEOUT,
            max_concurrent_requests: MAX_CONCURRENT_REQUESTS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationKind {
    Chat,
    ModelPreparation,
}

struct ActiveOperation {
    kind: OperationKind,
    cancellation: watch::Sender<bool>,
}

pub struct InferenceService {
    provider: Arc<dyn InferenceProvider>,
    resolver: ModelResolver,
    limits: ServiceLimits,
    concurrency: Arc<Semaphore>,
    active_operations: Arc<Mutex<HashMap<String, ActiveOperation>>>,
}

struct ActiveOperationGuard {
    request_id: String,
    active_operations: Arc<Mutex<HashMap<String, ActiveOperation>>>,
}

impl Drop for ActiveOperationGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active_operations.lock() {
            active.remove(&self.request_id);
        }
    }
}

impl InferenceService {
    pub fn new(
        provider: Arc<dyn InferenceProvider>,
        resolver: ModelResolver,
        limits: ServiceLimits,
    ) -> Result<Self, InferenceError> {
        LocalEndpoint::parse(&provider.endpoint())?;
        Ok(Self {
            provider,
            resolver,
            concurrency: Arc::new(Semaphore::new(limits.max_concurrent_requests)),
            active_operations: Arc::new(Mutex::new(HashMap::new())),
            limits,
        })
    }

    pub async fn status(&self) -> Result<InferenceStatus, InferenceError> {
        self.ensure_local_provider()?;
        let (_cancel_sender, cancel_receiver) = watch::channel(false);
        let control = OperationControl::new(cancel_receiver);
        let health = self
            .run_probe(async {
                let health = self.provider.health().await?;
                if health.available {
                    self.ensure_local_provider()?;
                    self.provider.verify_local_execution(control).await?;
                }
                Ok(health)
            })
            .await?;
        Ok(InferenceStatus {
            execution_policy: InferencePolicy::LocalOnly,
            provider_id: self.provider.provider_id().to_string(),
            endpoint: self.provider.endpoint(),
            available: health.available,
            local_only_verified: health.available,
            detail: if health.available {
                "Local provider is reachable and LocalOnly policy is verified".to_string()
            } else {
                health.detail
            },
        })
    }

    pub async fn models(&self) -> Result<Vec<InferenceModelSummary>, InferenceError> {
        self.ensure_local_provider()?;
        let models = self.resolver.resolved_models()?;
        let (_cancel_sender, cancel_receiver) = watch::channel(false);
        let control = OperationControl::new(cancel_receiver);
        let installed = self
            .run_probe(async {
                self.provider
                    .verify_local_execution(control.clone())
                    .await?;
                let mut installed = HashSet::new();
                for model in &models {
                    self.ensure_local_provider()?;
                    if let Some(verified) = self
                        .provider
                        .verify_local_model(&model.provider_model_id, control.clone())
                        .await?
                    {
                        Self::validate_verified_model(model.provider_model_id.as_str(), &verified)?;
                        installed.insert(verified.provider_model_id);
                    }
                }
                Ok(installed)
            })
            .await?;
        Ok(self.resolver.summaries(&installed))
    }

    pub async fn prepare_model(
        &self,
        request: PrepareLocalModelRequest,
    ) -> Result<ModelPreparationResponse, InferenceError> {
        Self::validate_request_id(&request.request_id)?;
        self.ensure_local_provider()?;
        let model = self.resolver.resolve(&request.canonical_model_id)?;
        let (mut cancel_receiver, _active_guard) =
            self.register_operation(&request.request_id, OperationKind::ModelPreparation)?;
        let deadline = TokioInstant::now() + self.limits.prepare_timeout;

        let permit = tokio::select! {
            biased;
            _ = cancel_receiver.changed() => return Err(InferenceError::Cancelled),
            _ = sleep_until(deadline) => return Err(InferenceError::TimedOut),
            permit = self.concurrency.clone().acquire_owned() => {
                permit.map_err(|_| InferenceError::Internal("Inference concurrency gate is closed".to_string()))?
            }
        };

        let control = OperationControl::new(cancel_receiver.clone());
        let canonical_model_id = request.canonical_model_id.clone();
        let provider_model_id = model.provider_model_id.clone();
        let provider_result = tokio::select! {
            biased;
            _ = cancel_receiver.changed() => Err(InferenceError::Cancelled),
            _ = sleep_until(deadline) => Err(InferenceError::TimedOut),
            result = async {
                self.ensure_local_provider()?;
                self.provider.verify_local_execution(control.clone()).await?;
                self.provider.prepare_model(&provider_model_id, control.clone()).await?;
                self.ensure_local_provider()?;
                self.provider.verify_local_execution(control.clone()).await?;
                let remapped = self.resolver.resolve(&canonical_model_id)?;
                if remapped != model {
                    return Err(InferenceError::LocalModelUnverified);
                }
                let verified = self
                    .provider
                    .verify_local_model(&provider_model_id, control)
                    .await?
                    .ok_or(InferenceError::LocalModelUnverified)?;
                Self::validate_verified_model(&provider_model_id, &verified)
            } => result,
        };
        drop(permit);
        provider_result?;

        Ok(ModelPreparationResponse {
            request_id: request.request_id,
            canonical_model_id: request.canonical_model_id,
            provider_id: self.provider.provider_id().to_string(),
        })
    }

    pub fn cancel(&self, request_id: &str) -> Result<bool, InferenceError> {
        self.cancel_operation(request_id, OperationKind::Chat)
    }

    pub fn cancel_preparation(&self, request_id: &str) -> Result<bool, InferenceError> {
        self.cancel_operation(request_id, OperationKind::ModelPreparation)
    }

    pub async fn run(
        &self,
        request: InferenceRequest,
        sink: Arc<dyn EventSink>,
    ) -> Result<InferenceResponse, InferenceError> {
        self.validate_request(&request)?;
        self.ensure_local_provider()?;
        let resolved = self.resolver.resolve(&request.canonical_model_id)?;
        let (mut cancel_receiver, _active_guard) =
            self.register_operation(&request.request_id, OperationKind::Chat)?;

        let gate = Arc::new(EventGate::new(sink));
        gate.emit(InferenceEvent::Started {
            request_id: request.request_id.clone(),
        })?;

        let deadline = TokioInstant::now() + self.limits.request_timeout;
        let permit = tokio::select! {
            biased;
            _ = cancel_receiver.changed() => {
                gate.emit(InferenceEvent::Cancelled { request_id: request.request_id.clone() })?;
                return Err(InferenceError::Cancelled);
            }
            _ = sleep_until(deadline) => {
                gate.emit(InferenceEvent::TimedOut { request_id: request.request_id.clone() })?;
                return Err(InferenceError::TimedOut);
            }
            permit = self.concurrency.clone().acquire_owned() => {
                permit.map_err(|_| InferenceError::Internal("Inference concurrency gate is closed".to_string()))?
            }
        };

        gate.emit(InferenceEvent::Running {
            request_id: request.request_id.clone(),
        })?;
        let control = RequestControl::new(
            request.request_id.clone(),
            cancel_receiver.clone(),
            Arc::clone(&gate),
        );
        let operation_control = control.operation_control();
        let started = Instant::now();

        let provider_result = tokio::select! {
            biased;
            _ = cancel_receiver.changed() => Err(InferenceError::Cancelled),
            _ = sleep_until(deadline) => Err(InferenceError::TimedOut),
            result = async {
                self.ensure_local_provider()?;
                self.provider
                    .verify_local_execution(operation_control.clone())
                    .await?;
                let remapped = self.resolver.resolve(&request.canonical_model_id)?;
                if remapped != resolved {
                    return Err(InferenceError::LocalModelUnverified);
                }
                let verified = self
                    .provider
                    .verify_local_model(&remapped.provider_model_id, operation_control)
                    .await?
                    .ok_or(InferenceError::LocalModelUnverified)?;
                Self::validate_verified_model(&remapped.provider_model_id, &verified)?;

                let provider_request = ProviderChatRequest {
                    provider_model_id: remapped.provider_model_id,
                    messages: request.messages.clone(),
                    max_tokens: request.max_tokens,
                    temperature: request.temperature,
                };
                self.provider.chat(provider_request, control).await
            } => result,
        };
        drop(permit);

        match provider_result {
            Ok(output_text) => {
                gate.emit(InferenceEvent::Completed {
                    request_id: request.request_id.clone(),
                })?;
                Ok(InferenceResponse {
                    request_id: request.request_id,
                    canonical_model_id: request.canonical_model_id,
                    provider_id: self.provider.provider_id().to_string(),
                    latency_ms: started.elapsed().as_millis() as u64,
                    output_text,
                })
            }
            Err(InferenceError::Cancelled) => {
                gate.emit(InferenceEvent::Cancelled {
                    request_id: request.request_id,
                })?;
                Err(InferenceError::Cancelled)
            }
            Err(InferenceError::TimedOut) => {
                gate.emit(InferenceEvent::TimedOut {
                    request_id: request.request_id,
                })?;
                Err(InferenceError::TimedOut)
            }
            Err(error) => {
                gate.emit(InferenceEvent::Failed {
                    request_id: request.request_id,
                    code: error.code().to_string(),
                    error: error.public_message(),
                })?;
                Err(error)
            }
        }
    }

    fn validate_request(&self, request: &InferenceRequest) -> Result<(), InferenceError> {
        Self::validate_request_id(&request.request_id)?;
        if request.messages.is_empty() {
            return Err(InferenceError::InvalidRequest(
                "At least one chat message is required".to_string(),
            ));
        }
        if request.max_tokens == 0 || request.max_tokens > self.limits.max_tokens {
            return Err(InferenceError::InvalidRequest(format!(
                "max_tokens must be between 1 and {}",
                self.limits.max_tokens
            )));
        }
        if !request.temperature.is_finite() || !(0.0..=2.0).contains(&request.temperature) {
            return Err(InferenceError::InvalidRequest(
                "temperature must be between 0 and 2".to_string(),
            ));
        }
        if !request.stream {
            return Err(InferenceError::InvalidRequest(
                "Only bounded streaming inference is supported".to_string(),
            ));
        }
        let prompt_bytes = request.messages.iter().try_fold(0usize, |total, message| {
            if !matches!(message.role.as_str(), "system" | "user" | "assistant") {
                return Err(InferenceError::InvalidRequest(
                    "Chat message role is not supported".to_string(),
                ));
            }
            if message.content.trim().is_empty() {
                return Err(InferenceError::InvalidRequest(
                    "Chat messages cannot be empty".to_string(),
                ));
            }
            Ok(total.saturating_add(message.content.len()))
        })?;
        if prompt_bytes > self.limits.max_prompt_bytes {
            return Err(InferenceError::InvalidRequest(
                "Chat context exceeds the local inference safety limit".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_request_id(request_id: &str) -> Result<(), InferenceError> {
        Uuid::parse_str(request_id)
            .map(|_| ())
            .map_err(|_| InferenceError::InvalidRequest("Request ID must be a UUID".to_string()))
    }

    fn register_operation(
        &self,
        request_id: &str,
        kind: OperationKind,
    ) -> Result<(watch::Receiver<bool>, ActiveOperationGuard), InferenceError> {
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        let mut active = self.active_operations.lock().map_err(|_| {
            InferenceError::Internal("Active operation registry is unavailable".to_string())
        })?;
        if active.contains_key(request_id) {
            return Err(InferenceError::DuplicateRequest);
        }
        active.insert(
            request_id.to_string(),
            ActiveOperation {
                kind,
                cancellation: cancel_sender,
            },
        );
        drop(active);
        Ok((
            cancel_receiver,
            ActiveOperationGuard {
                request_id: request_id.to_string(),
                active_operations: Arc::clone(&self.active_operations),
            },
        ))
    }

    fn cancel_operation(
        &self,
        request_id: &str,
        expected_kind: OperationKind,
    ) -> Result<bool, InferenceError> {
        let sender = self
            .active_operations
            .lock()
            .map_err(|_| {
                InferenceError::Internal("Active operation registry is unavailable".to_string())
            })?
            .get(request_id)
            .and_then(|operation| {
                (operation.kind == expected_kind).then(|| operation.cancellation.clone())
            });
        match sender {
            Some(sender) => {
                let _ = sender.send(true);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn ensure_local_provider(&self) -> Result<(), InferenceError> {
        LocalEndpoint::parse(&self.provider.endpoint()).map(|_| ())
    }

    fn validate_verified_model(
        expected_provider_model_id: &str,
        verified: &VerifiedLocalModel,
    ) -> Result<(), InferenceError> {
        if verified.provider_model_id != expected_provider_model_id || verified.digest.is_empty() {
            return Err(InferenceError::LocalModelUnverified);
        }
        Ok(())
    }

    async fn run_probe<T, F>(&self, probe: F) -> Result<T, InferenceError>
    where
        F: Future<Output = Result<T, InferenceError>>,
    {
        let deadline = TokioInstant::now() + self.limits.probe_timeout;
        timeout_at(deadline, probe)
            .await
            .map_err(|_| InferenceError::TimedOut)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::ollama_provider::OllamaProvider;
    use crate::inference::provider::{ProviderHealth, RequestControl, VerifiedLocalModel};
    use crate::inference::types::{ChatMessage, InferencePublicError};
    use async_trait::async_trait;
    use std::future::pending;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::Notify;
    use tokio::time::timeout;

    #[derive(Clone, Copy)]
    enum FakeMode {
        Success,
        SlowSuccess,
        Never,
        TokenThenNever,
        Error,
    }

    struct FakeProvider {
        mode: FakeMode,
    }

    #[derive(Clone, Copy)]
    enum PreparationMode {
        Success,
        SlowSuccess,
        Never,
        Error,
    }

    struct PreparationTestProvider {
        mode: PreparationMode,
        preparation_called: Arc<AtomicBool>,
    }

    fn verified_model(provider_model_id: &str) -> VerifiedLocalModel {
        VerifiedLocalModel {
            provider_model_id: provider_model_id.to_string(),
            digest: format!("sha256:{}", "a".repeat(64)),
        }
    }

    #[async_trait]
    impl InferenceProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "fake-local"
        }
        fn endpoint(&self) -> String {
            "http://127.0.0.1:1/".to_string()
        }
        async fn health(&self) -> Result<ProviderHealth, InferenceError> {
            Ok(ProviderHealth {
                available: true,
                detail: "available".to_string(),
            })
        }
        async fn verify_local_execution(
            &self,
            _control: OperationControl,
        ) -> Result<(), InferenceError> {
            Ok(())
        }
        async fn verify_local_model(
            &self,
            provider_model_id: &str,
            _control: OperationControl,
        ) -> Result<Option<VerifiedLocalModel>, InferenceError> {
            Ok((provider_model_id == "qwen3.5:2b").then(|| verified_model(provider_model_id)))
        }
        async fn prepare_model(
            &self,
            _provider_model_id: &str,
            _control: OperationControl,
        ) -> Result<(), InferenceError> {
            Ok(())
        }
        async fn chat(
            &self,
            _request: ProviderChatRequest,
            control: RequestControl,
        ) -> Result<String, InferenceError> {
            match self.mode {
                FakeMode::Success => {
                    control.emit_token("local".to_string())?;
                    Ok("local".to_string())
                }
                FakeMode::SlowSuccess => {
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    control.emit_token("local".to_string())?;
                    Ok("local".to_string())
                }
                FakeMode::Never => pending().await,
                FakeMode::TokenThenNever => {
                    control.emit_token("partial".to_string())?;
                    pending().await
                }
                FakeMode::Error => Err(InferenceError::ProviderProtocol(
                    "controlled failure".to_string(),
                )),
            }
        }
    }

    #[async_trait]
    impl InferenceProvider for PreparationTestProvider {
        fn provider_id(&self) -> &'static str {
            "preparation-test-local"
        }

        fn endpoint(&self) -> String {
            "http://127.0.0.1:1/".to_string()
        }

        async fn health(&self) -> Result<ProviderHealth, InferenceError> {
            unreachable!()
        }

        async fn verify_local_execution(
            &self,
            _control: OperationControl,
        ) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn verify_local_model(
            &self,
            provider_model_id: &str,
            _control: OperationControl,
        ) -> Result<Option<VerifiedLocalModel>, InferenceError> {
            Ok(Some(verified_model(provider_model_id)))
        }

        async fn prepare_model(
            &self,
            _provider_model_id: &str,
            control: OperationControl,
        ) -> Result<(), InferenceError> {
            self.preparation_called.store(true, Ordering::SeqCst);
            control.ensure_active()?;
            match self.mode {
                PreparationMode::Success => Ok(()),
                PreparationMode::SlowSuccess => {
                    tokio::time::sleep(Duration::from_millis(60)).await;
                    control.ensure_active()?;
                    Ok(())
                }
                PreparationMode::Never => pending().await,
                PreparationMode::Error => Err(InferenceError::ProviderProtocol(
                    "Controlled model preparation failure".to_string(),
                )),
            }
        }

        async fn chat(
            &self,
            _request: ProviderChatRequest,
            _control: RequestControl,
        ) -> Result<String, InferenceError> {
            pending().await
        }
    }

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<InferenceEvent>>);

    impl EventSink for RecordingSink {
        fn emit(&self, event: InferenceEvent) -> Result<(), InferenceError> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn request() -> InferenceRequest {
        InferenceRequest {
            request_id: Uuid::new_v4().to_string(),
            canonical_model_id: "qwen35-2b-stable".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            max_tokens: 32,
            temperature: 0.2,
            stream: true,
        }
    }

    fn preparation_request() -> PrepareLocalModelRequest {
        PrepareLocalModelRequest {
            request_id: Uuid::new_v4().to_string(),
            canonical_model_id: "qwen35-2b-stable".to_string(),
        }
    }

    fn service(mode: FakeMode, request_timeout: Duration) -> Arc<InferenceService> {
        service_with_concurrency(mode, request_timeout, MAX_CONCURRENT_REQUESTS)
    }

    fn service_with_concurrency(
        mode: FakeMode,
        request_timeout: Duration,
        max_concurrent_requests: usize,
    ) -> Arc<InferenceService> {
        Arc::new(
            InferenceService::new(
                Arc::new(FakeProvider { mode }),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits {
                    request_timeout,
                    max_concurrent_requests,
                    ..ServiceLimits::default()
                },
            )
            .unwrap(),
        )
    }

    fn preparation_service(
        mode: PreparationMode,
        prepare_timeout: Duration,
        max_concurrent_requests: usize,
    ) -> (Arc<InferenceService>, Arc<AtomicBool>) {
        let preparation_called = Arc::new(AtomicBool::new(false));
        let service = Arc::new(
            InferenceService::new(
                Arc::new(PreparationTestProvider {
                    mode,
                    preparation_called: preparation_called.clone(),
                }),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits {
                    prepare_timeout,
                    max_concurrent_requests,
                    ..ServiceLimits::default()
                },
            )
            .unwrap(),
        );
        (service, preparation_called)
    }

    async fn wait_until_called(called: &AtomicBool) {
        for _ in 0..100 {
            if called.load(Ordering::SeqCst) {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("provider was not called");
    }

    enum ProbeFixture {
        StallHeaders,
        StallBody {
            body_prefix: &'static [u8],
            declared_length: usize,
        },
    }

    async fn probe_fixture_service(
        fixture: ProbeFixture,
        probe_timeout: Duration,
    ) -> (InferenceService, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0u8; 8192];
            let _ = socket.read(&mut request).await;

            match fixture {
                ProbeFixture::StallHeaders => pending::<()>().await,
                ProbeFixture::StallBody {
                    body_prefix,
                    declared_length,
                } => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {declared_length}\r\nConnection: keep-alive\r\n\r\n"
                    );
                    socket.write_all(response.as_bytes()).await.unwrap();
                    socket.write_all(body_prefix).await.unwrap();
                    pending::<()>().await;
                }
            }
        });
        let provider =
            OllamaProvider::new(LocalEndpoint::parse(&format!("http://{address}/")).unwrap())
                .unwrap();
        let service = InferenceService::new(
            Arc::new(provider),
            ModelResolver::from_bundled_registry().unwrap(),
            ServiceLimits {
                probe_timeout,
                ..ServiceLimits::default()
            },
        )
        .unwrap();
        (service, task)
    }

    async fn abort_fixture(task: tokio::task::JoinHandle<()>) {
        task.abort();
        let _ = task.await;
    }

    enum PreparationFixture {
        StallHeaders,
        StallBody,
    }

    async fn preparation_fixture_service(
        fixture: PreparationFixture,
    ) -> (
        Arc<InferenceService>,
        Arc<Notify>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let ready = Arc::new(Notify::new());
        let task_ready = ready.clone();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0u8; 8192];
            let _ = socket.read(&mut request).await;

            if matches!(fixture, PreparationFixture::StallBody) {
                let body = b"{\"status\":\"pulling private-provider-detail\"}\n";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
                    body.len() + 256
                );
                socket.write_all(response.as_bytes()).await.unwrap();
                socket.write_all(body).await.unwrap();
            }
            task_ready.notify_one();

            let mut trailing = [0u8; 256];
            loop {
                match socket.read(&mut trailing).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => continue,
                }
            }
        });
        let provider =
            OllamaProvider::new(LocalEndpoint::parse(&format!("http://{address}/")).unwrap())
                .unwrap();
        let service = Arc::new(
            InferenceService::new(
                Arc::new(provider),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits {
                    prepare_timeout: Duration::from_secs(2),
                    ..ServiceLimits::default()
                },
            )
            .unwrap(),
        );
        (service, ready, task)
    }

    #[tokio::test]
    async fn probe_deadline_bounds_health_when_loopback_stalls_before_headers() {
        let (service, task) =
            probe_fixture_service(ProbeFixture::StallHeaders, Duration::from_millis(20)).await;
        let result = timeout(Duration::from_millis(500), service.status())
            .await
            .expect("outer test guard elapsed");
        abort_fixture(task).await;

        let error = result.unwrap_err();
        assert_eq!(error, InferenceError::TimedOut);
        assert_eq!(InferencePublicError::from(error).code, "timed_out");
    }

    #[tokio::test]
    async fn probe_deadline_bounds_models_when_loopback_stalls_before_headers() {
        let (service, task) =
            probe_fixture_service(ProbeFixture::StallHeaders, Duration::from_millis(20)).await;
        let result = timeout(Duration::from_millis(500), service.models())
            .await
            .expect("outer test guard elapsed");
        abort_fixture(task).await;

        let error = result.unwrap_err();
        assert_eq!(error, InferenceError::TimedOut);
        let public = InferencePublicError::from(error);
        assert_eq!(public.code, "timed_out");
        assert!(!public.message.contains("127.0.0.1"));
    }

    #[tokio::test]
    async fn probe_deadline_bounds_model_inventory_when_response_body_stalls() {
        let sensitive_prefix = b"{\"models\":[{\"name\":\"private-provider-detail";
        let (service, task) = probe_fixture_service(
            ProbeFixture::StallBody {
                body_prefix: sensitive_prefix,
                declared_length: sensitive_prefix.len() + 256,
            },
            Duration::from_millis(20),
        )
        .await;
        let result = timeout(Duration::from_millis(500), service.models())
            .await
            .expect("outer test guard elapsed");
        abort_fixture(task).await;

        let error = result.unwrap_err();
        assert_eq!(error, InferenceError::TimedOut);
        let public = InferencePublicError::from(error);
        assert_eq!(public.code, "timed_out");
        assert!(!public.message.contains("private-provider-detail"));
    }

    #[tokio::test]
    async fn probe_fast_health_response_still_succeeds() {
        let service = service(FakeMode::Success, Duration::from_millis(500));
        let status = timeout(Duration::from_millis(500), service.status())
            .await
            .expect("outer test guard elapsed")
            .unwrap();

        assert!(status.available);
        assert!(status.local_only_verified);
        assert_eq!(status.execution_policy, InferencePolicy::LocalOnly);
    }

    #[tokio::test]
    async fn probe_fast_model_inventory_still_succeeds() {
        let service = service(FakeMode::Success, Duration::from_millis(500));
        let models = timeout(Duration::from_millis(500), service.models())
            .await
            .expect("outer test guard elapsed")
            .unwrap();

        assert!(models
            .iter()
            .any(|model| { model.canonical_model_id == "qwen35-2b-stable" && model.installed }));
    }

    #[tokio::test]
    async fn probe_deadline_does_not_limit_streaming_chat() {
        let service = Arc::new(
            InferenceService::new(
                Arc::new(FakeProvider {
                    mode: FakeMode::SlowSuccess,
                }),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits {
                    probe_timeout: Duration::from_millis(1),
                    request_timeout: Duration::from_millis(500),
                    ..ServiceLimits::default()
                },
            )
            .unwrap(),
        );
        let sink = Arc::new(RecordingSink::default());
        let result = timeout(
            Duration::from_millis(500),
            service.run(request(), sink.clone()),
        )
        .await
        .expect("outer test guard elapsed")
        .unwrap();

        assert_eq!(result.output_text, "local");
        assert!(matches!(
            sink.0.lock().unwrap().last(),
            Some(InferenceEvent::Completed { .. })
        ));
    }

    #[tokio::test]
    async fn preparation_fast_success_is_request_scoped_and_cleans_up() {
        let (service, called) = preparation_service(
            PreparationMode::Success,
            Duration::from_millis(200),
            MAX_CONCURRENT_REQUESTS,
        );
        let request = preparation_request();
        let response = timeout(
            Duration::from_millis(500),
            service.prepare_model(request.clone()),
        )
        .await
        .expect("outer test guard elapsed")
        .unwrap();

        assert!(called.load(Ordering::SeqCst));
        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.canonical_model_id, request.canonical_model_id);
        assert_eq!(response.provider_id, "preparation-test-local");
        assert!(!service.cancel_preparation(&response.request_id).unwrap());
    }

    #[tokio::test]
    async fn preparation_rejects_invalid_uuid_and_provider_tag_before_execution() {
        let (service, called) = preparation_service(
            PreparationMode::Success,
            Duration::from_millis(200),
            MAX_CONCURRENT_REQUESTS,
        );
        let invalid_id = PrepareLocalModelRequest {
            request_id: "not-a-uuid".to_string(),
            canonical_model_id: "qwen35-2b-stable".to_string(),
        };
        assert!(matches!(
            service.prepare_model(invalid_id).await,
            Err(InferenceError::InvalidRequest(_))
        ));

        let provider_tag = PrepareLocalModelRequest {
            request_id: Uuid::new_v4().to_string(),
            canonical_model_id: "qwen3.5:2b".to_string(),
        };
        assert!(matches!(
            service.prepare_model(provider_tag).await,
            Err(InferenceError::UnknownModel(_))
        ));
        assert!(!called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn preparation_duplicate_active_request_id_is_rejected_and_cancelled_cleanup_is_final() {
        let (service, called) = preparation_service(
            PreparationMode::Never,
            Duration::from_secs(2),
            MAX_CONCURRENT_REQUESTS,
        );
        let request = preparation_request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let first_request = request.clone();
        let task = tokio::spawn(async move { task_service.prepare_model(first_request).await });
        wait_until_called(&called).await;

        assert_eq!(
            service.prepare_model(request).await,
            Err(InferenceError::DuplicateRequest)
        );
        assert!(service.cancel_preparation(&request_id).unwrap());
        assert_eq!(
            timeout(Duration::from_millis(500), task)
                .await
                .expect("outer test guard elapsed")
                .unwrap(),
            Err(InferenceError::Cancelled)
        );
        assert!(!service.cancel_preparation(&request_id).unwrap());
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(!service.cancel_preparation(&request_id).unwrap());
    }

    #[tokio::test]
    async fn preparation_cancellation_during_queue_wait_never_reaches_provider() {
        let (service, called) =
            preparation_service(PreparationMode::Success, Duration::from_secs(2), 1);
        let held_permit = service.concurrency.clone().acquire_owned().await.unwrap();
        let request = preparation_request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task = tokio::spawn(async move { task_service.prepare_model(request).await });

        tokio::task::yield_now().await;
        assert!(service.cancel_preparation(&request_id).unwrap());
        assert_eq!(
            timeout(Duration::from_millis(500), task)
                .await
                .expect("outer test guard elapsed")
                .unwrap(),
            Err(InferenceError::Cancelled)
        );
        assert!(!called.load(Ordering::SeqCst));
        assert!(!service.cancel_preparation(&request_id).unwrap());
        drop(held_permit);
    }

    #[tokio::test]
    async fn preparation_cancel_before_headers_drops_the_local_http_operation() {
        let (service, ready, fixture_task) =
            preparation_fixture_service(PreparationFixture::StallHeaders).await;
        let request = preparation_request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task = tokio::spawn(async move { task_service.prepare_model(request).await });

        timeout(Duration::from_millis(500), ready.notified())
            .await
            .expect("fixture did not accept the request");
        assert!(service.cancel_preparation(&request_id).unwrap());
        assert_eq!(
            timeout(Duration::from_millis(500), task)
                .await
                .expect("outer test guard elapsed")
                .unwrap(),
            Err(InferenceError::Cancelled)
        );
        timeout(Duration::from_millis(500), fixture_task)
            .await
            .expect("client connection remained open after cancellation")
            .unwrap();
        assert!(!service.cancel_preparation(&request_id).unwrap());
    }

    #[tokio::test]
    async fn preparation_cancel_during_streamed_body_drops_without_exposing_body() {
        let (service, ready, fixture_task) =
            preparation_fixture_service(PreparationFixture::StallBody).await;
        let request = preparation_request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task = tokio::spawn(async move { task_service.prepare_model(request).await });

        timeout(Duration::from_millis(500), ready.notified())
            .await
            .expect("fixture did not send its partial body");
        assert!(service.cancel_preparation(&request_id).unwrap());
        let error = timeout(Duration::from_millis(500), task)
            .await
            .expect("outer test guard elapsed")
            .unwrap()
            .unwrap_err();
        assert_eq!(error, InferenceError::Cancelled);
        let public = InferencePublicError::from(error);
        assert_eq!(public.code, "cancelled");
        assert!(!public.message.contains("private-provider-detail"));
        timeout(Duration::from_millis(500), fixture_task)
            .await
            .expect("client stream remained open after cancellation")
            .unwrap();
    }

    #[tokio::test]
    async fn preparation_timeout_remains_distinct_and_cannot_later_succeed() {
        let (service, called) = preparation_service(
            PreparationMode::SlowSuccess,
            Duration::from_millis(20),
            MAX_CONCURRENT_REQUESTS,
        );
        let request = preparation_request();
        let request_id = request.request_id.clone();
        assert_eq!(
            timeout(Duration::from_millis(500), service.prepare_model(request))
                .await
                .expect("outer test guard elapsed"),
            Err(InferenceError::TimedOut)
        );
        assert!(called.load(Ordering::SeqCst));
        tokio::time::sleep(Duration::from_millis(70)).await;
        assert!(!service.cancel_preparation(&request_id).unwrap());
    }

    #[tokio::test]
    async fn preparation_targeted_cancellation_does_not_cancel_another_preparation() {
        let (service, _) = preparation_service(
            PreparationMode::Never,
            Duration::from_secs(2),
            MAX_CONCURRENT_REQUESTS,
        );
        let first = preparation_request();
        let second = preparation_request();
        let first_id = first.request_id.clone();
        let second_id = second.request_id.clone();
        let first_service = service.clone();
        let second_service = service.clone();
        let first_task = tokio::spawn(async move { first_service.prepare_model(first).await });
        let second_task = tokio::spawn(async move { second_service.prepare_model(second).await });

        for _ in 0..100 {
            let both_active = {
                let active = service.active_operations.lock().unwrap();
                active.contains_key(&first_id) && active.contains_key(&second_id)
            };
            if both_active {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(service.cancel_preparation(&first_id).unwrap());
        assert_eq!(first_task.await.unwrap(), Err(InferenceError::Cancelled));
        assert!(service
            .active_operations
            .lock()
            .unwrap()
            .contains_key(&second_id));
        assert!(service.cancel_preparation(&second_id).unwrap());
        assert_eq!(second_task.await.unwrap(), Err(InferenceError::Cancelled));
    }

    #[tokio::test]
    async fn preparation_cancellation_does_not_cancel_active_chat() {
        let (service, _) = preparation_service(
            PreparationMode::Never,
            Duration::from_secs(2),
            MAX_CONCURRENT_REQUESTS,
        );
        let preparation = preparation_request();
        let preparation_id = preparation.request_id.clone();
        let chat = request();
        let chat_id = chat.request_id.clone();
        let preparation_service = service.clone();
        let chat_service = service.clone();
        let preparation_task =
            tokio::spawn(async move { preparation_service.prepare_model(preparation).await });
        let chat_task = tokio::spawn(async move {
            chat_service
                .run(chat, Arc::new(RecordingSink::default()))
                .await
        });

        for _ in 0..100 {
            let both_active = {
                let active = service.active_operations.lock().unwrap();
                active.contains_key(&preparation_id) && active.contains_key(&chat_id)
            };
            if both_active {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(service.cancel_preparation(&preparation_id).unwrap());
        assert_eq!(
            preparation_task.await.unwrap(),
            Err(InferenceError::Cancelled)
        );
        assert!(service
            .active_operations
            .lock()
            .unwrap()
            .contains_key(&chat_id));
        assert!(service.cancel(&chat_id).unwrap());
        assert_eq!(chat_task.await.unwrap(), Err(InferenceError::Cancelled));
    }

    #[tokio::test]
    async fn preparation_request_id_cannot_alias_active_chat() {
        let (service, _) = preparation_service(
            PreparationMode::Success,
            Duration::from_secs(2),
            MAX_CONCURRENT_REQUESTS,
        );
        let chat = request();
        let request_id = chat.request_id.clone();
        let task_service = service.clone();
        let chat_task = tokio::spawn(async move {
            task_service
                .run(chat, Arc::new(RecordingSink::default()))
                .await
        });
        for _ in 0..100 {
            if service
                .active_operations
                .lock()
                .unwrap()
                .contains_key(&request_id)
            {
                break;
            }
            tokio::task::yield_now().await;
        }

        assert_eq!(
            service
                .prepare_model(PrepareLocalModelRequest {
                    request_id: request_id.clone(),
                    canonical_model_id: "qwen35-2b-stable".to_string(),
                })
                .await,
            Err(InferenceError::DuplicateRequest)
        );
        assert!(!service.cancel_preparation(&request_id).unwrap());
        assert!(service.cancel(&request_id).unwrap());
        assert_eq!(chat_task.await.unwrap(), Err(InferenceError::Cancelled));
    }

    #[tokio::test]
    async fn preparation_provider_error_is_controlled_and_registry_is_cleaned() {
        let (service, called) = preparation_service(
            PreparationMode::Error,
            Duration::from_millis(200),
            MAX_CONCURRENT_REQUESTS,
        );
        let request = preparation_request();
        let request_id = request.request_id.clone();
        let result = service.prepare_model(request).await;
        assert!(matches!(result, Err(InferenceError::ProviderProtocol(_))));
        assert!(called.load(Ordering::SeqCst));
        assert!(!service.cancel_preparation(&request_id).unwrap());
    }

    #[test]
    fn preparation_unknown_request_id_cancellation_fails_safely() {
        let (service, _) = preparation_service(
            PreparationMode::Success,
            Duration::from_secs(1),
            MAX_CONCURRENT_REQUESTS,
        );
        assert!(!service
            .cancel_preparation(&Uuid::new_v4().to_string())
            .unwrap());
    }

    #[tokio::test]
    async fn completes_with_one_terminal_event() {
        let sink = Arc::new(RecordingSink::default());
        let response = service(FakeMode::Success, Duration::from_secs(1))
            .run(request(), sink.clone())
            .await
            .unwrap();
        assert_eq!(response.output_text, "local");
        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::Completed { .. })
        ));
    }

    #[tokio::test]
    async fn timeout_is_terminal_and_cleans_up() {
        let service = service(FakeMode::Never, Duration::from_millis(20));
        let sink = Arc::new(RecordingSink::default());
        let request = request();
        assert_eq!(
            service.run(request.clone(), sink.clone()).await,
            Err(InferenceError::TimedOut)
        );
        assert!(!service.cancel(&request.request_id).unwrap());
        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::TimedOut { .. })
        ));
    }

    #[tokio::test]
    async fn cancellation_is_terminal_and_cleans_up() {
        let service = service(FakeMode::Never, Duration::from_secs(2));
        let sink = Arc::new(RecordingSink::default());
        let request = request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task_sink = sink.clone();
        let task = tokio::spawn(async move { task_service.run(request, task_sink).await });
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(service.cancel(&request_id).unwrap());
        assert_eq!(task.await.unwrap(), Err(InferenceError::Cancelled));
        assert!(!service.cancel(&request_id).unwrap());
        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::Cancelled { .. })
        ));
    }

    #[tokio::test]
    async fn cancellation_during_queue_wait_never_reaches_the_provider() {
        let service = service_with_concurrency(FakeMode::Never, Duration::from_secs(2), 1);
        let held_permit = service.concurrency.clone().acquire_owned().await.unwrap();
        let sink = Arc::new(RecordingSink::default());
        let request = request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task_sink = sink.clone();
        let task = tokio::spawn(async move { task_service.run(request, task_sink).await });

        tokio::task::yield_now().await;
        assert!(service.cancel(&request_id).unwrap());
        assert_eq!(task.await.unwrap(), Err(InferenceError::Cancelled));
        drop(held_permit);

        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(!events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Running { .. })));
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::Cancelled { .. })
        ));
    }

    #[tokio::test]
    async fn timeout_during_queue_wait_never_reaches_the_provider() {
        let service = service_with_concurrency(FakeMode::Never, Duration::from_millis(20), 1);
        let held_permit = service.concurrency.clone().acquire_owned().await.unwrap();
        let sink = Arc::new(RecordingSink::default());

        assert_eq!(
            service.run(request(), sink.clone()).await,
            Err(InferenceError::TimedOut)
        );
        drop(held_permit);

        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(!events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Running { .. })));
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::TimedOut { .. })
        ));
    }

    #[tokio::test]
    async fn cancellation_during_streaming_keeps_the_token_then_cancels_once() {
        let service = service(FakeMode::TokenThenNever, Duration::from_secs(2));
        let sink = Arc::new(RecordingSink::default());
        let request = request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task_sink = sink.clone();
        let task = tokio::spawn(async move { task_service.run(request, task_sink).await });

        for _ in 0..100 {
            if sink
                .0
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, InferenceEvent::Token { .. }))
            {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(service.cancel(&request_id).unwrap());
        assert_eq!(task.await.unwrap(), Err(InferenceError::Cancelled));

        let events = sink.0.lock().unwrap();
        assert!(events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Token { .. })));
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::Cancelled { .. })
        ));
    }

    #[tokio::test]
    async fn timeout_during_streaming_is_the_only_terminal_event() {
        let sink = Arc::new(RecordingSink::default());
        assert_eq!(
            service(FakeMode::TokenThenNever, Duration::from_millis(20))
                .run(request(), sink.clone())
                .await,
            Err(InferenceError::TimedOut)
        );
        let events = sink.0.lock().unwrap();
        assert!(events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Token { .. })));
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::TimedOut { .. })
        ));
    }

    struct CompletionRaceProvider {
        token_emitted: Arc<Notify>,
        release: Arc<Notify>,
    }

    #[async_trait]
    impl InferenceProvider for CompletionRaceProvider {
        fn provider_id(&self) -> &'static str {
            "completion-race-local"
        }

        fn endpoint(&self) -> String {
            "http://127.0.0.1:1/".to_string()
        }

        async fn health(&self) -> Result<ProviderHealth, InferenceError> {
            unreachable!()
        }

        async fn verify_local_execution(
            &self,
            _control: OperationControl,
        ) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn verify_local_model(
            &self,
            provider_model_id: &str,
            _control: OperationControl,
        ) -> Result<Option<VerifiedLocalModel>, InferenceError> {
            Ok(Some(verified_model(provider_model_id)))
        }

        async fn prepare_model(&self, _: &str, _: OperationControl) -> Result<(), InferenceError> {
            unreachable!()
        }

        async fn chat(
            &self,
            _: ProviderChatRequest,
            control: RequestControl,
        ) -> Result<String, InferenceError> {
            control.emit_token("final".to_string())?;
            self.token_emitted.notify_one();
            self.release.notified().await;
            Ok("final".to_string())
        }
    }

    #[tokio::test]
    async fn cancellation_after_final_token_suppresses_provider_completion() {
        let token_emitted = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let service = Arc::new(
            InferenceService::new(
                Arc::new(CompletionRaceProvider {
                    token_emitted: token_emitted.clone(),
                    release: release.clone(),
                }),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits::default(),
            )
            .unwrap(),
        );
        let sink = Arc::new(RecordingSink::default());
        let request = request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let task_sink = sink.clone();
        let task = tokio::spawn(async move { task_service.run(request, task_sink).await });

        token_emitted.notified().await;
        assert!(service.cancel(&request_id).unwrap());
        release.notify_one();
        assert_eq!(task.await.unwrap(), Err(InferenceError::Cancelled));

        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::Cancelled { .. })
        ));
        assert!(!events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Completed { .. })));
    }

    struct DelayedErrorProvider {
        entered: Arc<Notify>,
        release: Arc<Notify>,
    }

    #[async_trait]
    impl InferenceProvider for DelayedErrorProvider {
        fn provider_id(&self) -> &'static str {
            "delayed-error-local"
        }

        fn endpoint(&self) -> String {
            "http://127.0.0.1:1/".to_string()
        }

        async fn health(&self) -> Result<ProviderHealth, InferenceError> {
            unreachable!()
        }

        async fn verify_local_execution(
            &self,
            _control: OperationControl,
        ) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn verify_local_model(
            &self,
            provider_model_id: &str,
            _control: OperationControl,
        ) -> Result<Option<VerifiedLocalModel>, InferenceError> {
            Ok(Some(verified_model(provider_model_id)))
        }

        async fn prepare_model(&self, _: &str, _: OperationControl) -> Result<(), InferenceError> {
            unreachable!()
        }

        async fn chat(
            &self,
            _: ProviderChatRequest,
            _: RequestControl,
        ) -> Result<String, InferenceError> {
            self.entered.notify_one();
            self.release.notified().await;
            Err(InferenceError::ProviderProtocol(
                "controlled late provider failure".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn timeout_suppresses_a_provider_error_released_after_the_deadline() {
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let service = Arc::new(
            InferenceService::new(
                Arc::new(DelayedErrorProvider {
                    entered: entered.clone(),
                    release: release.clone(),
                }),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits {
                    request_timeout: Duration::from_millis(20),
                    ..ServiceLimits::default()
                },
            )
            .unwrap(),
        );
        let sink = Arc::new(RecordingSink::default());
        let task_service = service.clone();
        let task_sink = sink.clone();
        let task = tokio::spawn(async move { task_service.run(request(), task_sink).await });

        entered.notified().await;
        tokio::time::sleep(Duration::from_millis(30)).await;
        release.notify_one();
        assert_eq!(task.await.unwrap(), Err(InferenceError::TimedOut));

        let events = sink.0.lock().unwrap();
        assert_eq!(events.iter().filter(|event| event.is_terminal()).count(), 1);
        assert!(matches!(
            events.last(),
            Some(InferenceEvent::TimedOut { .. })
        ));
        assert!(!events
            .iter()
            .any(|event| matches!(event, InferenceEvent::Failed { .. })));
    }

    #[tokio::test]
    async fn provider_error_is_controlled_and_terminal() {
        let sink = Arc::new(RecordingSink::default());
        let result = service(FakeMode::Error, Duration::from_secs(1))
            .run(request(), sink.clone())
            .await;
        assert!(matches!(result, Err(InferenceError::ProviderProtocol(_))));
        assert!(matches!(
            sink.0.lock().unwrap().last(),
            Some(InferenceEvent::Failed { .. })
        ));
    }

    #[tokio::test]
    async fn duplicate_active_request_is_rejected() {
        let service = service(FakeMode::Never, Duration::from_secs(2));
        let first_sink = Arc::new(RecordingSink::default());
        let request = request();
        let request_id = request.request_id.clone();
        let task_service = service.clone();
        let first = request.clone();
        let task = tokio::spawn(async move { task_service.run(first, first_sink).await });
        tokio::time::sleep(Duration::from_millis(10)).await;
        let second = service
            .run(request, Arc::new(RecordingSink::default()))
            .await;
        assert_eq!(second, Err(InferenceError::DuplicateRequest));
        service.cancel(&request_id).unwrap();
        let _ = task.await;
    }

    #[test]
    fn local_only_service_rejects_remote_provider_before_use() {
        struct RemoteProvider;

        #[async_trait]
        impl InferenceProvider for RemoteProvider {
            fn provider_id(&self) -> &'static str {
                "remote"
            }
            fn endpoint(&self) -> String {
                "https://example.com/".to_string()
            }
            async fn health(&self) -> Result<ProviderHealth, InferenceError> {
                unreachable!()
            }
            async fn prepare_model(
                &self,
                _: &str,
                _: OperationControl,
            ) -> Result<(), InferenceError> {
                unreachable!()
            }
            async fn chat(
                &self,
                _: ProviderChatRequest,
                _: RequestControl,
            ) -> Result<String, InferenceError> {
                unreachable!()
            }
        }

        assert!(matches!(
            InferenceService::new(
                Arc::new(RemoteProvider),
                ModelResolver::from_bundled_registry().unwrap(),
                ServiceLimits::default(),
            ),
            Err(InferenceError::PolicyViolation(_))
        ));
    }

    #[tokio::test]
    async fn local_endpoint_is_revalidated_before_each_provider_boundary() {
        struct MutableEndpointProvider {
            remote: AtomicBool,
        }

        #[async_trait]
        impl InferenceProvider for MutableEndpointProvider {
            fn provider_id(&self) -> &'static str {
                "mutable-local"
            }

            fn endpoint(&self) -> String {
                if self.remote.load(Ordering::SeqCst) {
                    "https://example.com/".to_string()
                } else {
                    "http://127.0.0.1:1/".to_string()
                }
            }

            async fn health(&self) -> Result<ProviderHealth, InferenceError> {
                unreachable!()
            }

            async fn prepare_model(
                &self,
                _: &str,
                _: OperationControl,
            ) -> Result<(), InferenceError> {
                unreachable!()
            }

            async fn chat(
                &self,
                _: ProviderChatRequest,
                _: RequestControl,
            ) -> Result<String, InferenceError> {
                unreachable!()
            }
        }

        let provider = Arc::new(MutableEndpointProvider {
            remote: AtomicBool::new(false),
        });
        let service = InferenceService::new(
            provider.clone(),
            ModelResolver::from_bundled_registry().unwrap(),
            ServiceLimits::default(),
        )
        .unwrap();
        provider.remote.store(true, Ordering::SeqCst);

        assert!(matches!(
            service.status().await,
            Err(InferenceError::PolicyViolation(_))
        ));
        assert!(matches!(
            service.models().await,
            Err(InferenceError::PolicyViolation(_))
        ));
        assert!(matches!(
            service
                .prepare_model(PrepareLocalModelRequest {
                    request_id: Uuid::new_v4().to_string(),
                    canonical_model_id: "qwen35-2b-stable".to_string(),
                })
                .await,
            Err(InferenceError::PolicyViolation(_))
        ));
        assert!(matches!(
            service
                .run(request(), Arc::new(RecordingSink::default()))
                .await,
            Err(InferenceError::PolicyViolation(_))
        ));
    }

    #[tokio::test]
    async fn invalid_request_bounds_fail_before_provider_execution() {
        let service = service(FakeMode::Success, Duration::from_secs(1));
        let cases = {
            let mut invalid_uuid = request();
            invalid_uuid.request_id = "not-a-uuid".to_string();
            let mut invalid_role = request();
            invalid_role.messages[0].role = "tool".to_string();
            let mut invalid_tokens = request();
            invalid_tokens.max_tokens = 0;
            let mut invalid_temperature = request();
            invalid_temperature.temperature = 3.0;
            let mut unsupported_stream = request();
            unsupported_stream.stream = false;
            let mut oversized_context = request();
            oversized_context.messages[0].content = "x".repeat(MAX_PROMPT_BYTES + 1);
            vec![
                invalid_uuid,
                invalid_role,
                invalid_tokens,
                invalid_temperature,
                unsupported_stream,
                oversized_context,
            ]
        };

        for candidate in cases {
            let result = service
                .run(candidate, Arc::new(RecordingSink::default()))
                .await;
            assert!(matches!(result, Err(InferenceError::InvalidRequest(_))));
        }
    }

    #[tokio::test]
    async fn unknown_canonical_model_fails_before_any_event() {
        let service = service(FakeMode::Success, Duration::from_secs(1));
        let sink = Arc::new(RecordingSink::default());
        let mut candidate = request();
        candidate.canonical_model_id = "not-in-registry".to_string();
        assert!(matches!(
            service.run(candidate, sink.clone()).await,
            Err(InferenceError::UnknownModel(_))
        ));
        assert!(sink.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn cancelling_one_request_does_not_cancel_another() {
        let service = service(FakeMode::Never, Duration::from_secs(2));
        let first = request();
        let second = request();
        let first_id = first.request_id.clone();
        let second_id = second.request_id.clone();
        let first_service = service.clone();
        let second_service = service.clone();
        let first_task = tokio::spawn(async move {
            first_service
                .run(first, Arc::new(RecordingSink::default()))
                .await
        });
        let second_task = tokio::spawn(async move {
            second_service
                .run(second, Arc::new(RecordingSink::default()))
                .await
        });
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(service.cancel(&first_id).unwrap());
        assert_eq!(first_task.await.unwrap(), Err(InferenceError::Cancelled));
        assert!(service.cancel(&second_id).unwrap());
        assert_eq!(second_task.await.unwrap(), Err(InferenceError::Cancelled));
    }

    #[tokio::test]
    async fn sensitive_prompt_is_absent_from_public_events_and_metadata() {
        const SENTINEL: &str = "SENSITIVE_PROMPT_SENTINEL_DO_NOT_LOG";
        let service = service(FakeMode::Success, Duration::from_secs(1));
        let sink = Arc::new(RecordingSink::default());
        let mut candidate = request();
        candidate.messages[0].content = SENTINEL.to_string();
        let response = service.run(candidate, sink.clone()).await.unwrap();
        let public_surface = format!("{response:?} {:?}", sink.0.lock().unwrap());
        assert!(!public_surface.contains(SENTINEL));
    }
}
