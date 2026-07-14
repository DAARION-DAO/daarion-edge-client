use crate::inference::model_resolver::ModelResolver;
use crate::inference::policy::LocalEndpoint;
use crate::inference::provider::{
    EventGate, EventSink, InferenceProvider, ProviderChatRequest, RequestControl,
};
use crate::inference::types::{
    InferenceError, InferenceEvent, InferenceModelSummary, InferencePolicy, InferenceRequest,
    InferenceResponse, InferenceStatus,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{watch, Semaphore};
use tokio::time::{sleep_until, timeout, Instant as TokioInstant};
use uuid::Uuid;

const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_TOKENS: u32 = 4096;
const MAX_CONCURRENT_REQUESTS: usize = 2;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const PREPARE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone)]
pub struct ServiceLimits {
    pub max_prompt_bytes: usize,
    pub max_tokens: u32,
    pub request_timeout: Duration,
    pub max_concurrent_requests: usize,
}

impl Default for ServiceLimits {
    fn default() -> Self {
        Self {
            max_prompt_bytes: MAX_PROMPT_BYTES,
            max_tokens: MAX_TOKENS,
            request_timeout: REQUEST_TIMEOUT,
            max_concurrent_requests: MAX_CONCURRENT_REQUESTS,
        }
    }
}

pub struct InferenceService {
    provider: Arc<dyn InferenceProvider>,
    resolver: ModelResolver,
    limits: ServiceLimits,
    concurrency: Arc<Semaphore>,
    active_requests: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
}

struct ActiveRequestGuard {
    request_id: String,
    active_requests: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
}

impl Drop for ActiveRequestGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active_requests.lock() {
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
            active_requests: Arc::new(Mutex::new(HashMap::new())),
            limits,
        })
    }

    pub async fn status(&self) -> Result<InferenceStatus, InferenceError> {
        self.ensure_local_provider()?;
        let health = self.provider.health().await?;
        Ok(InferenceStatus {
            execution_policy: InferencePolicy::LocalOnly,
            provider_id: self.provider.provider_id().to_string(),
            endpoint: self.provider.endpoint(),
            available: health.available,
            detail: health.detail,
        })
    }

    pub async fn models(&self) -> Result<Vec<InferenceModelSummary>, InferenceError> {
        self.ensure_local_provider()?;
        let installed = match self.provider.list_installed_models().await {
            Ok(models) => models
                .into_iter()
                .map(|model| model.provider_model_id)
                .collect::<HashSet<_>>(),
            Err(InferenceError::ProviderUnavailable) => HashSet::new(),
            Err(error) => return Err(error),
        };
        Ok(self.resolver.summaries(&installed))
    }

    pub async fn prepare_model(&self, canonical_model_id: &str) -> Result<(), InferenceError> {
        self.ensure_local_provider()?;
        let model = self.resolver.resolve(canonical_model_id)?;
        timeout(
            PREPARE_TIMEOUT,
            self.provider.prepare_model(&model.provider_model_id),
        )
        .await
        .map_err(|_| InferenceError::TimedOut)??;
        Ok(())
    }

    pub fn cancel(&self, request_id: &str) -> Result<bool, InferenceError> {
        let sender = self
            .active_requests
            .lock()
            .map_err(|_| {
                InferenceError::Internal("Active request registry is unavailable".to_string())
            })?
            .get(request_id)
            .cloned();
        match sender {
            Some(sender) => {
                let _ = sender.send(true);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn run(
        &self,
        request: InferenceRequest,
        sink: Arc<dyn EventSink>,
    ) -> Result<InferenceResponse, InferenceError> {
        self.validate_request(&request)?;
        self.ensure_local_provider()?;
        let resolved = self.resolver.resolve(&request.canonical_model_id)?;
        let (cancel_sender, mut cancel_receiver) = watch::channel(false);

        {
            let mut active = self.active_requests.lock().map_err(|_| {
                InferenceError::Internal("Active request registry is unavailable".to_string())
            })?;
            if active.contains_key(&request.request_id) {
                return Err(InferenceError::DuplicateRequest);
            }
            active.insert(request.request_id.clone(), cancel_sender);
        }
        let _active_guard = ActiveRequestGuard {
            request_id: request.request_id.clone(),
            active_requests: Arc::clone(&self.active_requests),
        };

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
        let provider_request = ProviderChatRequest {
            provider_model_id: resolved.provider_model_id,
            messages: request.messages.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };
        let started = Instant::now();

        let provider_result = tokio::select! {
            biased;
            _ = cancel_receiver.changed() => Err(InferenceError::Cancelled),
            _ = sleep_until(deadline) => Err(InferenceError::TimedOut),
            result = self.provider.chat(provider_request, control) => result,
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
        Uuid::parse_str(&request.request_id)
            .map_err(|_| InferenceError::InvalidRequest("Request ID must be a UUID".to_string()))?;
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

    fn ensure_local_provider(&self) -> Result<(), InferenceError> {
        LocalEndpoint::parse(&self.provider.endpoint()).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::provider::{InstalledProviderModel, ProviderHealth, RequestControl};
    use crate::inference::types::ChatMessage;
    use async_trait::async_trait;
    use std::future::pending;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::Notify;

    #[derive(Clone, Copy)]
    enum FakeMode {
        Success,
        Never,
        TokenThenNever,
        Error,
    }

    struct FakeProvider {
        mode: FakeMode,
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
        async fn list_installed_models(
            &self,
        ) -> Result<Vec<InstalledProviderModel>, InferenceError> {
            Ok(vec![InstalledProviderModel {
                provider_model_id: "qwen3.5:2b".to_string(),
            }])
        }
        async fn prepare_model(&self, _provider_model_id: &str) -> Result<(), InferenceError> {
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

        async fn list_installed_models(
            &self,
        ) -> Result<Vec<InstalledProviderModel>, InferenceError> {
            unreachable!()
        }

        async fn prepare_model(&self, _: &str) -> Result<(), InferenceError> {
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

        async fn list_installed_models(
            &self,
        ) -> Result<Vec<InstalledProviderModel>, InferenceError> {
            unreachable!()
        }

        async fn prepare_model(&self, _: &str) -> Result<(), InferenceError> {
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
            async fn list_installed_models(
                &self,
            ) -> Result<Vec<InstalledProviderModel>, InferenceError> {
                unreachable!()
            }
            async fn prepare_model(&self, _: &str) -> Result<(), InferenceError> {
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

            async fn list_installed_models(
                &self,
            ) -> Result<Vec<InstalledProviderModel>, InferenceError> {
                unreachable!()
            }

            async fn prepare_model(&self, _: &str) -> Result<(), InferenceError> {
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
            service.prepare_model("qwen35-2b-stable").await,
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
