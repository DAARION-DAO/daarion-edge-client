use crate::runtime_store::config::{RuntimeStoreConfig, STORAGE_QUEUE_CAPACITY};
use crate::runtime_store::connection::RuntimeStoreConnection;
use crate::runtime_store::control::RuntimeStoreControl;
use crate::runtime_store::deadline::ensure_before;
use crate::runtime_store::error::{
    ContentOperationError, ContentOperationErrorCode, RuntimeStoreError, RuntimeStoreErrorKind,
};
use crate::runtime_store::migrations::{migrate_and_validate_until, CURRENT_SCHEMA_VERSION};
use crate::runtime_store::models::{
    AppendMessageRequest, AuditEventRecord, AuditPage, ConversationPage, ConversationRecord,
    CreateConversationRequest, GetAuditEventRequest, GetConversationRequest, GetTaskRequest,
    ListAuditEventsRequest, ListConversationsRequest, ListMessagesRequest, ListTasksRequest,
    MessagePage, MessageRecord, RecordInertTaskRequest, TaskPage, TaskRecord,
};
use crate::runtime_store::repositories;
use crate::runtime_store::types::{StorageRuntimeErrorCode, StorageRuntimeStatus};
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) const PRODUCTION_SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);
const WORKER_CONTROL_POLL: Duration = Duration::from_millis(10);

#[cfg(test)]
struct ActiveWorkerGuard(Arc<JoinAccounting>);

#[cfg(test)]
impl ActiveWorkerGuard {
    fn enter(accounting: Arc<JoinAccounting>) -> Self {
        accounting.active_workers.fetch_add(1, Ordering::AcqRel);
        Self(accounting)
    }
}

#[cfg(test)]
impl Drop for ActiveWorkerGuard {
    fn drop(&mut self) {
        self.0.active_workers.fetch_sub(1, Ordering::AcqRel);
    }
}

enum RuntimeStoreRequest {
    Initialize {
        config: RuntimeStoreConfig,
        deadline: Instant,
        reply: Option<SyncSender<StorageRuntimeStatus>>,
    },
    ReadStatus {
        reply: SyncSender<StorageRuntimeStatus>,
    },
    #[allow(dead_code)]
    CreateConversation {
        request: CreateConversationRequest,
        deadline: Instant,
        reply: SyncSender<Result<ConversationRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    GetConversation {
        request: GetConversationRequest,
        deadline: Instant,
        reply: SyncSender<Result<ConversationRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    ListConversations {
        request: ListConversationsRequest,
        deadline: Instant,
        reply: SyncSender<Result<ConversationPage, ContentOperationError>>,
    },
    #[allow(dead_code)]
    AppendMessage {
        request: AppendMessageRequest,
        deadline: Instant,
        reply: SyncSender<Result<MessageRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    ListMessages {
        request: ListMessagesRequest,
        deadline: Instant,
        reply: SyncSender<Result<MessagePage, ContentOperationError>>,
    },
    #[allow(dead_code)]
    RecordInertTask {
        request: RecordInertTaskRequest,
        deadline: Instant,
        reply: SyncSender<Result<TaskRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    GetTask {
        request: GetTaskRequest,
        deadline: Instant,
        reply: SyncSender<Result<TaskRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    ListTasks {
        request: ListTasksRequest,
        deadline: Instant,
        reply: SyncSender<Result<TaskPage, ContentOperationError>>,
    },
    #[allow(dead_code)]
    GetAuditEvent {
        request: GetAuditEventRequest,
        deadline: Instant,
        reply: SyncSender<Result<AuditEventRecord, ContentOperationError>>,
    },
    #[allow(dead_code)]
    ListAuditEvents {
        request: ListAuditEventsRequest,
        deadline: Instant,
        reply: SyncSender<Result<AuditPage, ContentOperationError>>,
    },
    #[cfg(test)]
    Hold {
        duration: Duration,
        reply: SyncSender<()>,
    },
    #[cfg(test)]
    Block {
        entered: SyncSender<()>,
        release: Receiver<()>,
    },
    #[cfg(test)]
    Panic,
    #[cfg(test)]
    ExitUnexpectedly,
}

struct ShutdownRequest {
    deadline: Instant,
    reply: SyncSender<Result<(), RuntimeStoreError>>,
    #[cfg(test)]
    suppress_reply: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerExit {
    CleanShutdown,
    ControlledFailure,
    ChannelDisconnected,
    #[cfg(test)]
    UnexpectedExit,
    Panic,
}

#[derive(Clone, Copy)]
enum ShutdownPhase {
    Running,
    InProgress,
    Complete(Result<(), RuntimeStoreError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerJoinOwnership {
    Manager = 0,
    Reaper = 1,
    Completed = 2,
}

impl WorkerJoinOwnership {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Reaper,
            2 => Self::Completed,
            _ => Self::Manager,
        }
    }
}

struct JoinAccounting {
    ownership: AtomicU8,
    joined_after_exit_proof: AtomicBool,
    reaper_completed: AtomicBool,
    #[cfg(test)]
    active_workers: AtomicUsize,
}

struct ReaperJob {
    worker: JoinHandle<()>,
    exit_receiver: Option<Receiver<WorkerExit>>,
    exit_proof_seen: bool,
}

struct RuntimeStoreManagerInner {
    sender: SyncSender<RuntimeStoreRequest>,
    shutdown_sender: SyncSender<ShutdownRequest>,
    status: Arc<RwLock<StorageRuntimeStatus>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    worker_exit: Mutex<Option<Receiver<WorkerExit>>>,
    reaper_sender: Mutex<Option<SyncSender<ReaperJob>>>,
    reaper_handle: Mutex<Option<JoinHandle<()>>>,
    join_accounting: Arc<JoinAccounting>,
    control: Arc<RuntimeStoreControl>,
    #[cfg(test)]
    last_worker_exit: Arc<Mutex<Option<WorkerExit>>>,
    accepting: Arc<AtomicBool>,
    ordinary_deadline: RwLock<Duration>,
    shutdown_phase: Mutex<ShutdownPhase>,
    shutdown_complete: Condvar,
    #[cfg(test)]
    suppress_exit_signal: Arc<AtomicBool>,
    #[cfg(test)]
    exit_signal_delay: Arc<Mutex<Duration>>,
    #[cfg(test)]
    suppress_shutdown_reply: AtomicBool,
}

#[derive(Clone)]
pub(crate) struct RuntimeStoreManager {
    inner: Arc<RuntimeStoreManagerInner>,
}

impl RuntimeStoreManager {
    pub(crate) fn new() -> Self {
        let last_start_time_ms = current_time_ms();
        let initial_status = StorageRuntimeStatus::initializing(last_start_time_ms);
        let status = Arc::new(RwLock::new(initial_status));
        let accepting = Arc::new(AtomicBool::new(true));
        let control = Arc::new(RuntimeStoreControl::new());
        let join_accounting = Arc::new(JoinAccounting {
            ownership: AtomicU8::new(WorkerJoinOwnership::Manager as u8),
            joined_after_exit_proof: AtomicBool::new(false),
            reaper_completed: AtomicBool::new(false),
            #[cfg(test)]
            active_workers: AtomicUsize::new(0),
        });
        #[cfg(test)]
        let last_worker_exit = Arc::new(Mutex::new(None));
        let (sender, receiver) = mpsc::sync_channel(STORAGE_QUEUE_CAPACITY);
        let (shutdown_sender, shutdown_receiver) = mpsc::sync_channel(1);
        let (worker_exit_sender, worker_exit) = mpsc::sync_channel(1);
        let (reaper_sender_value, reaper_receiver) = mpsc::sync_channel(1);
        #[cfg(test)]
        let suppress_exit_signal = Arc::new(AtomicBool::new(false));
        #[cfg(test)]
        let exit_signal_delay = Arc::new(Mutex::new(Duration::ZERO));

        let reaper_accounting = Arc::clone(&join_accounting);
        let mut reaper_handle = thread::Builder::new()
            .name("daarion-runtime-store-reaper".to_string())
            .spawn(move || run_reaper(reaper_receiver, &reaper_accounting))
            .ok();
        let mut reaper_sender = reaper_handle.as_ref().map(|_| reaper_sender_value);

        let worker_status = Arc::clone(&status);
        let worker_accepting = Arc::clone(&accepting);
        let worker_control = Arc::clone(&control);
        #[cfg(test)]
        let worker_join_accounting = Arc::clone(&join_accounting);
        #[cfg(test)]
        let worker_last_exit = Arc::clone(&last_worker_exit);
        #[cfg(test)]
        let worker_suppress_exit_signal = Arc::clone(&suppress_exit_signal);
        #[cfg(test)]
        let worker_exit_signal_delay = Arc::clone(&exit_signal_delay);
        let join_handle = reaper_handle.as_ref().and_then(|_| {
            thread::Builder::new()
                .name("daarion-runtime-store".to_string())
                .spawn(move || {
                    #[cfg(test)]
                    let _active_worker = ActiveWorkerGuard::enter(worker_join_accounting);
                    let exit = match catch_unwind(AssertUnwindSafe(|| {
                        run_worker(
                            receiver,
                            shutdown_receiver,
                            &worker_status,
                            &worker_control,
                            last_start_time_ms,
                        )
                    })) {
                        Ok(exit) => exit,
                        Err(_) => WorkerExit::Panic,
                    };
                    worker_accepting.store(false, Ordering::Release);
                    if worker_exit_is_abnormal(exit) {
                        store_status(
                            &worker_status,
                            failure_status(RuntimeStoreError::internal(), last_start_time_ms),
                        );
                    }
                    #[cfg(test)]
                    if let Ok(mut recorded) = worker_last_exit.lock() {
                        *recorded = Some(exit);
                    }
                    #[cfg(test)]
                    if let Ok(delay) = worker_exit_signal_delay.lock() {
                        thread::sleep(*delay);
                    }
                    #[cfg(test)]
                    let suppress = worker_suppress_exit_signal.load(Ordering::Acquire);
                    #[cfg(not(test))]
                    let suppress = false;
                    if !suppress {
                        let _ = worker_exit_sender.try_send(exit);
                    }
                })
                .ok()
        });

        let spawned = join_handle.is_some();
        if !spawned {
            reaper_sender.take();
            if let Some(handle) = reaper_handle.take() {
                let _ = handle.join();
            }
            join_accounting
                .ownership
                .store(WorkerJoinOwnership::Completed as u8, Ordering::Release);
            store_status(
                &status,
                failure_status(RuntimeStoreError::unavailable(), last_start_time_ms),
            );
        }
        accepting.store(spawned, Ordering::Release);

        Self {
            inner: Arc::new(RuntimeStoreManagerInner {
                sender,
                shutdown_sender,
                status,
                join_handle: Mutex::new(join_handle),
                worker_exit: Mutex::new(Some(worker_exit)),
                reaper_sender: Mutex::new(reaper_sender),
                reaper_handle: Mutex::new(reaper_handle),
                join_accounting,
                control,
                #[cfg(test)]
                last_worker_exit,
                accepting,
                ordinary_deadline: RwLock::new(
                    crate::runtime_store::config::ORDINARY_OPERATION_DEADLINE,
                ),
                shutdown_phase: Mutex::new(if spawned {
                    ShutdownPhase::Running
                } else {
                    ShutdownPhase::Complete(Err(RuntimeStoreError::unavailable()))
                }),
                shutdown_complete: Condvar::new(),
                #[cfg(test)]
                suppress_exit_signal,
                #[cfg(test)]
                exit_signal_delay,
                #[cfg(test)]
                suppress_shutdown_reply: AtomicBool::new(false),
            }),
        }
    }

    pub(crate) fn start_initialization(&self, config: RuntimeStoreConfig) {
        self.store_ordinary_deadline(config.ordinary_deadline);
        let deadline = Instant::now() + config.migration_deadline;
        if self
            .send_with_deadline(
                RuntimeStoreRequest::Initialize {
                    config,
                    deadline,
                    reply: None,
                },
                deadline,
            )
            .is_err()
        {
            self.disable_with_fresh_failure(RuntimeStoreError::unavailable());
        }
    }

    pub(crate) fn fail_path_resolution(&self) {
        self.store_failure(RuntimeStoreError::path_invalid());
    }

    pub(crate) fn read_status(&self) -> StorageRuntimeStatus {
        if !self.inner.accepting.load(Ordering::Acquire) {
            return self.disable_with_fresh_failure(RuntimeStoreError::internal());
        }
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        if self
            .send_with_deadline(RuntimeStoreRequest::ReadStatus { reply }, deadline)
            .is_err()
        {
            return self.disable_with_fresh_failure(RuntimeStoreError::internal());
        }
        match response.recv_timeout(remaining_or_zero(deadline)) {
            Ok(status) => status,
            Err(_) => self.disable_with_fresh_failure(RuntimeStoreError::internal()),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn create_conversation(
        &self,
        request: CreateConversationRequest,
    ) -> Result<ConversationRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::CreateConversation {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn get_conversation(
        &self,
        request: GetConversationRequest,
    ) -> Result<ConversationRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::GetConversation {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn list_conversations(
        &self,
        request: ListConversationsRequest,
    ) -> Result<ConversationPage, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::ListConversations {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn append_message(
        &self,
        request: AppendMessageRequest,
    ) -> Result<MessageRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::AppendMessage {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn list_messages(
        &self,
        request: ListMessagesRequest,
    ) -> Result<MessagePage, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::ListMessages {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn record_inert_task(
        &self,
        request: RecordInertTaskRequest,
    ) -> Result<TaskRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::RecordInertTask {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn get_task(
        &self,
        request: GetTaskRequest,
    ) -> Result<TaskRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::GetTask {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn list_tasks(
        &self,
        request: ListTasksRequest,
    ) -> Result<TaskPage, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::ListTasks {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn get_audit_event(
        &self,
        request: GetAuditEventRequest,
    ) -> Result<AuditEventRecord, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::GetAuditEvent {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    #[allow(dead_code)]
    pub(crate) fn list_audit_events(
        &self,
        request: ListAuditEventsRequest,
    ) -> Result<AuditPage, ContentOperationError> {
        request.validate()?;
        let deadline = Instant::now() + self.ordinary_deadline();
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::ListAuditEvents {
                request,
                deadline,
                reply,
            },
            deadline,
        )
        .map_err(ContentOperationError::from_runtime)?;
        receive_content_response(response, deadline)
    }

    pub(crate) fn production_shutdown(&self) -> Result<(), RuntimeStoreError> {
        self.shutdown(PRODUCTION_SHUTDOWN_DEADLINE)
    }

    pub(crate) fn internal_failure_status(&self) -> StorageRuntimeStatus {
        let last_start_time_ms = self.current_status().last_start_time_ms;
        failure_status(RuntimeStoreError::internal(), last_start_time_ms)
    }

    fn current_status(&self) -> StorageRuntimeStatus {
        self.inner
            .status
            .read()
            .map(|status| status.clone())
            .unwrap_or_else(|_| failure_status(RuntimeStoreError::internal(), current_time_ms()))
    }

    fn store_failure(&self, error: RuntimeStoreError) {
        let last_start_time_ms = self.current_status().last_start_time_ms;
        store_status(
            &self.inner.status,
            failure_status(error, last_start_time_ms),
        );
    }

    fn disable_with_fresh_failure(&self, error: RuntimeStoreError) -> StorageRuntimeStatus {
        self.inner.accepting.store(false, Ordering::Release);
        let status = failure_status(error, self.current_status().last_start_time_ms);
        store_status(&self.inner.status, status.clone());
        status
    }

    fn ordinary_deadline(&self) -> Duration {
        self.inner
            .ordinary_deadline
            .read()
            .map(|deadline| *deadline)
            .unwrap_or(crate::runtime_store::config::ORDINARY_OPERATION_DEADLINE)
    }

    fn store_ordinary_deadline(&self, deadline: Duration) {
        if let Ok(mut current) = self.inner.ordinary_deadline.write() {
            *current = deadline;
        }
    }

    fn send_with_deadline(
        &self,
        request: RuntimeStoreRequest,
        deadline: Instant,
    ) -> Result<(), RuntimeStoreError> {
        if !self.inner.accepting.load(Ordering::Acquire) {
            return Err(RuntimeStoreError::unavailable());
        }
        try_send_with_deadline(&self.inner.sender, request, deadline)
    }

    fn shutdown(&self, budget: Duration) -> Result<(), RuntimeStoreError> {
        shutdown_inner(&self.inner, Instant::now() + budget)
    }

    #[cfg(test)]
    pub(crate) fn initialize_for_test(&self, config: RuntimeStoreConfig) -> StorageRuntimeStatus {
        self.store_ordinary_deadline(config.ordinary_deadline);
        let deadline = Instant::now() + config.migration_deadline;
        let fallback = self.internal_failure_status();
        let (reply, response) = mpsc::sync_channel(1);
        if self
            .send_with_deadline(
                RuntimeStoreRequest::Initialize {
                    config,
                    deadline,
                    reply: Some(reply),
                },
                deadline,
            )
            .is_err()
        {
            return self.disable_with_fresh_failure(RuntimeStoreError::internal());
        }
        response
            .recv_timeout(remaining_or_zero(deadline))
            .unwrap_or(fallback)
    }

    #[cfg(test)]
    pub(crate) fn hold_worker_for_test(&self, duration: Duration) -> Result<(), RuntimeStoreError> {
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::Hold { duration, reply },
            Instant::now() + Duration::from_millis(100),
        )?;
        response
            .recv_timeout(duration + Duration::from_secs(1))
            .map_err(|_| RuntimeStoreError::unavailable())
    }

    #[cfg(test)]
    pub(crate) fn block_worker_for_test(&self) -> Result<SyncSender<()>, RuntimeStoreError> {
        let (entered, entered_response) = mpsc::sync_channel(1);
        let (release, release_response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::Block {
                entered,
                release: release_response,
            },
            Instant::now() + Duration::from_millis(100),
        )?;
        entered_response
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| RuntimeStoreError::unavailable())?;
        Ok(release)
    }

    #[cfg(test)]
    pub(crate) fn try_enqueue_status_for_test(
        &self,
    ) -> Result<Receiver<StorageRuntimeStatus>, RuntimeStoreError> {
        let (reply, response) = mpsc::sync_channel(1);
        match self
            .inner
            .sender
            .try_send(RuntimeStoreRequest::ReadStatus { reply })
        {
            Ok(()) => Ok(response),
            Err(TrySendError::Full(_)) => {
                Err(RuntimeStoreError::new(RuntimeStoreErrorKind::BusyTimeout))
            }
            Err(TrySendError::Disconnected(_)) => Err(RuntimeStoreError::unavailable()),
        }
    }

    #[cfg(test)]
    pub(crate) fn trigger_worker_panic_for_test(&self) -> Result<(), RuntimeStoreError> {
        self.send_with_deadline(
            RuntimeStoreRequest::Panic,
            Instant::now() + Duration::from_secs(1),
        )
    }

    #[cfg(test)]
    pub(crate) fn trigger_unexpected_exit_for_test(&self) -> Result<(), RuntimeStoreError> {
        self.send_with_deadline(
            RuntimeStoreRequest::ExitUnexpectedly,
            Instant::now() + Duration::from_secs(1),
        )
    }

    #[cfg(test)]
    pub(crate) fn shutdown_for_test(&self, deadline: Duration) -> Result<(), RuntimeStoreError> {
        self.shutdown(deadline)
    }

    #[cfg(test)]
    pub(crate) fn worker_exit_for_test(&self) -> Option<WorkerExit> {
        self.inner
            .last_worker_exit
            .lock()
            .ok()
            .and_then(|exit| *exit)
    }

    #[cfg(test)]
    pub(crate) fn suppress_exit_signal_for_test(&self) {
        self.inner
            .suppress_exit_signal
            .store(true, Ordering::Release);
    }

    #[cfg(test)]
    pub(crate) fn joined_after_exit_for_test(&self) -> bool {
        self.inner
            .join_accounting
            .joined_after_exit_proof
            .load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn worker_join_ownership_for_test(&self) -> WorkerJoinOwnership {
        WorkerJoinOwnership::from_u8(self.inner.join_accounting.ownership.load(Ordering::Acquire))
    }

    #[cfg(test)]
    pub(crate) fn reaper_completed_for_test(&self) -> bool {
        self.inner
            .join_accounting
            .reaper_completed
            .load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn active_worker_count_for_test(&self) -> usize {
        self.inner
            .join_accounting
            .active_workers
            .load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn active_initialization_for_test(&self) -> bool {
        self.inner.control.has_active_initialization()
    }

    #[cfg(test)]
    pub(crate) fn active_watchdog_count_for_test(&self) -> usize {
        self.inner.control.active_watchdogs()
    }

    #[cfg(test)]
    pub(crate) fn shutdown_requested_for_test(&self) -> bool {
        self.inner.control.shutdown_requested()
    }

    #[cfg(test)]
    pub(crate) fn delay_exit_signal_for_test(&self, delay: Duration) {
        if let Ok(mut configured) = self.inner.exit_signal_delay.lock() {
            *configured = delay;
        }
    }

    #[cfg(test)]
    pub(crate) fn suppress_shutdown_reply_for_test(&self) {
        self.inner
            .suppress_shutdown_reply
            .store(true, Ordering::Release);
    }
}

impl Default for RuntimeStoreManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RuntimeStoreManagerInner {
    fn drop(&mut self) {
        let _ = shutdown_inner(self, Instant::now() + Duration::from_secs(2));
    }
}

fn shutdown_inner(
    inner: &RuntimeStoreManagerInner,
    deadline: Instant,
) -> Result<(), RuntimeStoreError> {
    let mut phase = inner
        .shutdown_phase
        .lock()
        .map_err(|_| RuntimeStoreError::internal())?;
    loop {
        match *phase {
            ShutdownPhase::Complete(result) => return result,
            ShutdownPhase::InProgress => {
                let wait = remaining_or_zero(deadline);
                if wait.is_zero() {
                    return Err(RuntimeStoreError::unavailable());
                }
                let (next, timeout) = inner
                    .shutdown_complete
                    .wait_timeout(phase, wait)
                    .map_err(|_| RuntimeStoreError::internal())?;
                phase = next;
                if timeout.timed_out() {
                    return Err(RuntimeStoreError::unavailable());
                }
            }
            ShutdownPhase::Running => {
                *phase = ShutdownPhase::InProgress;
                break;
            }
        }
    }
    drop(phase);

    inner.accepting.store(false, Ordering::Release);
    let control_result = inner.control.request_shutdown(deadline);
    let (reply, response) = mpsc::sync_channel(1);
    #[cfg(test)]
    let suppress_reply = inner.suppress_shutdown_reply.load(Ordering::Acquire);
    let send_result = inner
        .shutdown_sender
        .try_send(ShutdownRequest {
            deadline,
            reply,
            #[cfg(test)]
            suppress_reply,
        })
        .map_err(|error| match error {
            TrySendError::Full(_) => RuntimeStoreError::new(RuntimeStoreErrorKind::BusyTimeout),
            TrySendError::Disconnected(_) => RuntimeStoreError::unavailable(),
        });
    let close_result = match send_result {
        Ok(()) => response
            .recv_timeout(remaining_or_zero(deadline))
            .map_err(|_| RuntimeStoreError::unavailable())
            .and_then(|result| result),
        Err(error) => Err(error),
    };

    let exit_receiver = inner
        .worker_exit
        .lock()
        .map_err(|_| RuntimeStoreError::internal())
        .and_then(|mut receiver| receiver.take().ok_or_else(RuntimeStoreError::unavailable));
    let mut exit_receiver = exit_receiver.ok();
    let exit = exit_receiver
        .as_ref()
        .ok_or_else(RuntimeStoreError::unavailable)
        .and_then(|receiver| match receiver.try_recv() {
            Ok(exit) => Ok(exit),
            Err(TryRecvError::Empty) => receiver
                .recv_timeout(remaining_or_zero(deadline))
                .map_err(|_| RuntimeStoreError::unavailable()),
            Err(TryRecvError::Disconnected) => Err(RuntimeStoreError::unavailable()),
        });

    let projected_failure = projected_internal_error(&inner.status);
    let mut result = match (control_result, close_result, exit) {
        (Err(error), _, _) => Err(error),
        (_, Err(error), Ok(WorkerExit::ControlledFailure))
            if error.kind == RuntimeStoreErrorKind::Unavailable =>
        {
            Err(projected_failure.unwrap_or(error))
        }
        (_, Err(error), _) => Err(error),
        (_, Ok(()), Ok(WorkerExit::CleanShutdown)) => Ok(()),
        (_, Ok(()), Ok(WorkerExit::ControlledFailure)) => Err(RuntimeStoreError::internal()),
        (_, Ok(()), Ok(_)) | (_, Ok(()), Err(_)) => Err(RuntimeStoreError::unavailable()),
    };

    if exit.is_ok() {
        exit_receiver.take();
        if let Err(error) = join_worker_after_exit_proof(inner, deadline) {
            result = Err(error);
        }
    } else if let Err(error) = transfer_worker_to_reaper(inner, exit_receiver.take(), false) {
        result = Err(error);
    }

    if let Err(error) = result {
        store_status(
            &inner.status,
            failure_status(error, current_status_time(&inner.status)),
        );
    }

    if let Ok(mut phase) = inner.shutdown_phase.lock() {
        *phase = ShutdownPhase::Complete(result);
        inner.shutdown_complete.notify_all();
    }
    result
}

fn join_worker_after_exit_proof(
    inner: &RuntimeStoreManagerInner,
    deadline: Instant,
) -> Result<(), RuntimeStoreError> {
    let worker = inner
        .join_handle
        .lock()
        .map_err(|_| RuntimeStoreError::internal())?
        .take();
    let Some(worker) = worker else {
        return if WorkerJoinOwnership::from_u8(
            inner.join_accounting.ownership.load(Ordering::Acquire),
        ) == WorkerJoinOwnership::Completed
        {
            Ok(())
        } else {
            Err(RuntimeStoreError::unavailable())
        };
    };
    while !worker.is_finished() {
        if Instant::now() >= deadline {
            return transfer_specific_worker_to_reaper(inner, worker, None, true);
        }
        thread::yield_now();
    }
    worker.join().map_err(|_| RuntimeStoreError::internal())?;
    inner
        .join_accounting
        .joined_after_exit_proof
        .store(true, Ordering::Release);
    inner
        .join_accounting
        .ownership
        .store(WorkerJoinOwnership::Completed as u8, Ordering::Release);
    stop_idle_reaper(inner);
    Ok(())
}

fn transfer_worker_to_reaper(
    inner: &RuntimeStoreManagerInner,
    exit_receiver: Option<Receiver<WorkerExit>>,
    exit_proof_seen: bool,
) -> Result<(), RuntimeStoreError> {
    let worker = inner
        .join_handle
        .lock()
        .map_err(|_| RuntimeStoreError::internal())?
        .take()
        .ok_or_else(RuntimeStoreError::unavailable)?;
    transfer_specific_worker_to_reaper(inner, worker, exit_receiver, exit_proof_seen)
}

fn transfer_specific_worker_to_reaper(
    inner: &RuntimeStoreManagerInner,
    worker: JoinHandle<()>,
    exit_receiver: Option<Receiver<WorkerExit>>,
    exit_proof_seen: bool,
) -> Result<(), RuntimeStoreError> {
    let sender = inner
        .reaper_sender
        .lock()
        .map_err(|_| RuntimeStoreError::internal())?
        .take()
        .ok_or_else(RuntimeStoreError::unavailable)?;
    inner
        .join_accounting
        .ownership
        .store(WorkerJoinOwnership::Reaper as u8, Ordering::Release);
    match sender.try_send(ReaperJob {
        worker,
        exit_receiver,
        exit_proof_seen,
    }) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(job)) | Err(TrySendError::Disconnected(job)) => {
            inner
                .join_accounting
                .ownership
                .store(WorkerJoinOwnership::Manager as u8, Ordering::Release);
            if let Ok(mut guard) = inner.join_handle.lock() {
                *guard = Some(job.worker);
            }
            if let Ok(mut receiver) = inner.worker_exit.lock() {
                *receiver = job.exit_receiver;
            }
            Err(RuntimeStoreError::unavailable())
        }
    }
}

fn stop_idle_reaper(inner: &RuntimeStoreManagerInner) {
    if let Ok(mut sender) = inner.reaper_sender.lock() {
        sender.take();
    }
    if let Ok(mut reaper) = inner.reaper_handle.lock() {
        if reaper.as_ref().is_some_and(JoinHandle::is_finished) {
            if let Some(handle) = reaper.take() {
                let _ = handle.join();
            }
        }
    }
}

fn run_reaper(receiver: Receiver<ReaperJob>, accounting: &JoinAccounting) {
    if let Ok(job) = receiver.recv() {
        let exit_proof_seen = job.exit_proof_seen
            || job
                .exit_receiver
                .is_some_and(|exit_receiver| exit_receiver.recv().is_ok());
        let _ = job.worker.join();
        accounting
            .joined_after_exit_proof
            .store(exit_proof_seen, Ordering::Release);
        accounting
            .ownership
            .store(WorkerJoinOwnership::Completed as u8, Ordering::Release);
    }
    accounting.reaper_completed.store(true, Ordering::Release);
}

fn try_send_with_deadline(
    sender: &SyncSender<RuntimeStoreRequest>,
    mut request: RuntimeStoreRequest,
    expires_at: Instant,
) -> Result<(), RuntimeStoreError> {
    loop {
        match sender.try_send(request) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(returned)) if Instant::now() < expires_at => {
                request = returned;
                thread::yield_now();
            }
            Err(TrySendError::Full(_)) => {
                return Err(RuntimeStoreError::new(RuntimeStoreErrorKind::BusyTimeout));
            }
            Err(TrySendError::Disconnected(_)) => {
                return Err(RuntimeStoreError::unavailable());
            }
        }
    }
}

fn worker_exit_is_abnormal(exit: WorkerExit) -> bool {
    match exit {
        WorkerExit::ChannelDisconnected | WorkerExit::Panic => true,
        #[cfg(test)]
        WorkerExit::UnexpectedExit => true,
        WorkerExit::CleanShutdown | WorkerExit::ControlledFailure => false,
    }
}

fn run_worker(
    receiver: Receiver<RuntimeStoreRequest>,
    shutdown_receiver: Receiver<ShutdownRequest>,
    shared_status: &RwLock<StorageRuntimeStatus>,
    control: &Arc<RuntimeStoreControl>,
    last_start_time_ms: u64,
) -> WorkerExit {
    let mut connection: Option<RuntimeStoreConnection> = None;
    loop {
        match shutdown_receiver.try_recv() {
            Ok(request) => {
                return complete_worker_shutdown(
                    connection.take(),
                    request,
                    shared_status,
                    last_start_time_ms,
                )
            }
            Err(TryRecvError::Disconnected) if control.shutdown_requested() => {
                drop(connection.take());
                return WorkerExit::ControlledFailure;
            }
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => {}
        }
        if control.shutdown_requested() {
            match shutdown_receiver.recv() {
                Ok(request) => {
                    return complete_worker_shutdown(
                        connection.take(),
                        request,
                        shared_status,
                        last_start_time_ms,
                    )
                }
                Err(_) => {
                    drop(connection.take());
                    return WorkerExit::ControlledFailure;
                }
            }
        }

        let request = match receiver.recv_timeout(WORKER_CONTROL_POLL) {
            Ok(request) => request,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(opened) = connection.take() {
                    let _ = opened.close_until(Instant::now() + Duration::from_secs(2));
                }
                return WorkerExit::ChannelDisconnected;
            }
        };
        if control.shutdown_requested() {
            drop(request);
            continue;
        }
        match request {
            RuntimeStoreRequest::Initialize {
                config,
                deadline,
                reply,
            } => {
                let status = if connection.is_some() {
                    Ok((
                        None,
                        current_shared_status(shared_status, last_start_time_ms),
                    ))
                } else {
                    control
                        .begin_initialization()
                        .and_then(|attempt| {
                            initialize_connection(&config, deadline, &attempt, last_start_time_ms)
                        })
                        .map(|(opened, status)| (Some(opened), status))
                };
                if control.shutdown_requested() {
                    drop(status);
                    drop(reply);
                    continue;
                }
                let (mut opened, status) = match status {
                    Ok(value) => value,
                    Err(error) => (None, failure_status(error, last_start_time_ms)),
                };
                let published = control.publish_if_running(|| {
                    if let Some(opened) = opened.take() {
                        connection = Some(opened);
                    }
                    store_status(shared_status, status.clone());
                });
                drop(opened);
                if !published {
                    drop(reply);
                    continue;
                }
                if reply.is_some_and(|reply| reply.try_send(status).is_err()) {
                    return WorkerExit::ChannelDisconnected;
                }
            }
            RuntimeStoreRequest::ReadStatus { reply } => {
                let status = match connection.as_ref() {
                    Some(opened) => build_healthy_status(opened, last_start_time_ms)
                        .unwrap_or_else(|error| failure_status(error, last_start_time_ms)),
                    None => current_shared_status(shared_status, last_start_time_ms),
                };
                if !control.publish_if_running(|| store_status(shared_status, status.clone())) {
                    drop(reply);
                    continue;
                }
                if reply.try_send(status).is_err() {
                    return WorkerExit::ChannelDisconnected;
                }
            }
            RuntimeStoreRequest::CreateConversation {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| {
                        repositories::create_conversation(opened, &request, deadline)
                            .map(|execution| execution.record)
                    },
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::GetConversation {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::get_conversation(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::ListConversations {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::list_conversations(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::AppendMessage {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| {
                        repositories::append_message(opened, &request, deadline)
                            .map(|execution| execution.record)
                    },
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::ListMessages {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::list_messages(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::RecordInertTask {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| {
                        repositories::record_inert_task(opened, &request, deadline)
                            .map(|execution| execution.record)
                    },
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::GetTask {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::get_task(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::ListTasks {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::list_tasks(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::GetAuditEvent {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::get_audit_event(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            RuntimeStoreRequest::ListAuditEvents {
                request,
                deadline,
                reply,
            } => {
                let result = run_content_operation(
                    connection.as_mut(),
                    deadline,
                    shared_status,
                    last_start_time_ms,
                    |opened| repositories::list_audit_events(opened, &request, deadline),
                );
                let _ = reply.try_send(result);
            }
            #[cfg(test)]
            RuntimeStoreRequest::Hold { duration, reply } => {
                thread::sleep(duration);
                if reply.try_send(()).is_err() {
                    return WorkerExit::ChannelDisconnected;
                }
            }
            #[cfg(test)]
            RuntimeStoreRequest::Block { entered, release } => {
                if entered.try_send(()).is_err() || release.recv().is_err() {
                    return WorkerExit::ChannelDisconnected;
                }
            }
            #[cfg(test)]
            RuntimeStoreRequest::Panic => panic!("runtime_store_test_panic"),
            #[cfg(test)]
            RuntimeStoreRequest::ExitUnexpectedly => return WorkerExit::UnexpectedExit,
        }
    }
}

fn complete_worker_shutdown(
    connection: Option<RuntimeStoreConnection>,
    request: ShutdownRequest,
    shared_status: &RwLock<StorageRuntimeStatus>,
    last_start_time_ms: u64,
) -> WorkerExit {
    let close_result = connection.map_or(Ok(()), |opened| {
        opened.close_until(reserve_worker_exit_budget(request.deadline))
    });
    let terminal_status = match close_result {
        Ok(()) => failure_status(RuntimeStoreError::unavailable(), last_start_time_ms),
        Err(error) => failure_status(error, last_start_time_ms),
    };
    // The connection is no longer service-owned after this point. Remove any
    // stale healthy/warning projection before acknowledging shutdown so no
    // observer can mistake a closed store for a usable one.
    store_status(shared_status, terminal_status);
    #[cfg(test)]
    let suppress_reply = request.suppress_reply;
    #[cfg(not(test))]
    let suppress_reply = false;
    if !suppress_reply {
        let _ = request.reply.try_send(close_result);
    }
    if close_result.is_ok() {
        WorkerExit::CleanShutdown
    } else {
        WorkerExit::ControlledFailure
    }
}

fn initialize_connection(
    config: &RuntimeStoreConfig,
    deadline: Instant,
    attempt: &crate::runtime_store::control::InitializationAttempt,
    last_start_time_ms: u64,
) -> Result<(RuntimeStoreConnection, StorageRuntimeStatus), RuntimeStoreError> {
    ensure_before(deadline)?;
    attempt.ensure_running()?;
    let (mut opened, watchdog) =
        RuntimeStoreConnection::open_for_initialization(config, deadline, attempt)?;

    #[cfg(test)]
    if matches!(
        config.initialization_test_hook,
        crate::runtime_store::config::InitializationTestHook::LongQueryBeforeMigration
    ) {
        crate::runtime_store::migrations::run_long_query(&opened.connection)?;
    }

    #[cfg(test)]
    let schema_version = if matches!(
        config.initialization_test_hook,
        crate::runtime_store::config::InitializationTestHook::LongQueryInsideMigration
            | crate::runtime_store::config::InitializationTestHook::LongQueryDuringIntegrity
    ) {
        crate::runtime_store::migrations::migrate_and_validate_with_test_interrupt(
            &mut opened.connection,
            deadline,
            matches!(
                config.initialization_test_hook,
                crate::runtime_store::config::InitializationTestHook::LongQueryInsideMigration
            ),
            matches!(
                config.initialization_test_hook,
                crate::runtime_store::config::InitializationTestHook::LongQueryDuringIntegrity
            ),
        )?
    } else {
        migrate_and_validate_until(&mut opened.connection, deadline)?
    };
    #[cfg(not(test))]
    let schema_version = migrate_and_validate_until(&mut opened.connection, deadline)?;

    if schema_version != CURRENT_SCHEMA_VERSION {
        return Err(RuntimeStoreError::migration_mismatch());
    }
    ensure_before(deadline)?;
    attempt.ensure_running()?;
    opened.revalidate_artifacts()?;
    ensure_before(deadline)?;
    attempt.ensure_running()?;
    let status = build_healthy_status(&opened, last_start_time_ms)?;
    ensure_before(deadline)?;
    attempt.ensure_running()?;
    if watchdog.finish()? {
        return Err(RuntimeStoreError::deadline_exceeded());
    }
    ensure_before(deadline)?;
    attempt.ensure_running()?;
    Ok((opened, status))
}

fn build_healthy_status(
    opened: &RuntimeStoreConnection,
    last_start_time_ms: u64,
) -> Result<StorageRuntimeStatus, RuntimeStoreError> {
    if opened.content_integrity_failed {
        return Err(RuntimeStoreError::integrity_failed());
    }
    let database_size_bytes = opened.database_size_bytes()?;
    if database_size_bytes > opened.database_hard_limit_bytes {
        return Err(RuntimeStoreError::resource_limit());
    }
    Ok(StorageRuntimeStatus::healthy(
        last_start_time_ms,
        CURRENT_SCHEMA_VERSION,
        rusqlite::version().to_string(),
        database_size_bytes,
        opened.persistence_state,
        opened.database_warning_threshold_bytes,
        opened.database_hard_limit_bytes,
    ))
}

fn run_content_operation<T>(
    opened: Option<&mut RuntimeStoreConnection>,
    deadline: Instant,
    shared_status: &RwLock<StorageRuntimeStatus>,
    last_start_time_ms: u64,
    operation: impl FnOnce(&mut RuntimeStoreConnection) -> Result<T, ContentOperationError>,
) -> Result<T, ContentOperationError> {
    let opened = opened.ok_or_else(ContentOperationError::unavailable)?;
    let watchdog =
        crate::runtime_store::deadline::SqliteInterruptGuard::start(&opened.connection, deadline)
            .map_err(ContentOperationError::from_runtime)?;
    let result = operation(opened);
    let expired = watchdog
        .finish()
        .map_err(ContentOperationError::from_runtime)?;
    let result = if expired {
        Err(ContentOperationError::deadline_exceeded())
    } else {
        result
    };

    let poison = opened.content_integrity_failed
        || result
            .as_ref()
            .err()
            .is_some_and(|error| error.poisons_content_intake());
    if poison {
        opened.content_integrity_failed = true;
        store_status(
            shared_status,
            failure_status(RuntimeStoreError::integrity_failed(), last_start_time_ms),
        );
    } else if result.as_ref().err().is_some_and(|error| {
        matches!(
            error.code,
            ContentOperationErrorCode::Unavailable | ContentOperationErrorCode::Internal
        )
    }) {
        store_status(
            shared_status,
            failure_status(RuntimeStoreError::unavailable(), last_start_time_ms),
        );
    }
    result
}

#[allow(dead_code)]
fn receive_content_response<T>(
    response: Receiver<Result<T, ContentOperationError>>,
    deadline: Instant,
) -> Result<T, ContentOperationError> {
    match response.recv_timeout(remaining_or_zero(deadline)) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(ContentOperationError::deadline_exceeded()),
        Err(RecvTimeoutError::Disconnected) => Err(ContentOperationError::unavailable()),
    }
}

fn current_shared_status(
    shared_status: &RwLock<StorageRuntimeStatus>,
    last_start_time_ms: u64,
) -> StorageRuntimeStatus {
    shared_status
        .read()
        .map(|status| status.clone())
        .unwrap_or_else(|_| failure_status(RuntimeStoreError::internal(), last_start_time_ms))
}

fn current_status_time(shared_status: &RwLock<StorageRuntimeStatus>) -> u64 {
    shared_status
        .read()
        .map(|status| status.last_start_time_ms)
        .unwrap_or_else(|_| current_time_ms())
}

fn failure_status(error: RuntimeStoreError, last_start_time_ms: u64) -> StorageRuntimeStatus {
    StorageRuntimeStatus::failed(
        last_start_time_ms,
        error.public_state(),
        error.public_code(),
    )
}

fn store_status(shared: &RwLock<StorageRuntimeStatus>, status: StorageRuntimeStatus) {
    if let Ok(mut current) = shared.write() {
        *current = status;
    }
}

fn remaining_or_zero(deadline: Instant) -> Duration {
    deadline
        .checked_duration_since(Instant::now())
        .unwrap_or(Duration::ZERO)
}

fn reserve_worker_exit_budget(deadline: Instant) -> Instant {
    let remaining = remaining_or_zero(deadline);
    let reserve = (remaining / 4).min(Duration::from_millis(500));
    deadline.checked_sub(reserve).unwrap_or(deadline)
}

fn projected_internal_error(status: &RwLock<StorageRuntimeStatus>) -> Option<RuntimeStoreError> {
    let code = status.read().ok()?.error_code?;
    let kind = match code {
        StorageRuntimeErrorCode::PathInvalid => RuntimeStoreErrorKind::PathInvalid,
        StorageRuntimeErrorCode::PermissionDenied => RuntimeStoreErrorKind::PermissionDenied,
        StorageRuntimeErrorCode::Locked => RuntimeStoreErrorKind::Locked,
        StorageRuntimeErrorCode::BusyTimeout => RuntimeStoreErrorKind::BusyTimeout,
        StorageRuntimeErrorCode::MigrationMismatch => RuntimeStoreErrorKind::MigrationMismatch,
        StorageRuntimeErrorCode::NewerSchema => RuntimeStoreErrorKind::NewerSchema,
        StorageRuntimeErrorCode::MigrationFailed => RuntimeStoreErrorKind::MigrationFailed,
        StorageRuntimeErrorCode::IntegrityFailed => RuntimeStoreErrorKind::IntegrityFailed,
        StorageRuntimeErrorCode::ResourceLimit => RuntimeStoreErrorKind::ResourceLimit,
        StorageRuntimeErrorCode::Internal => RuntimeStoreErrorKind::Internal,
    };
    Some(RuntimeStoreError::new(kind))
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}
