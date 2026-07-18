use crate::runtime_store::config::{RuntimeStoreConfig, STORAGE_QUEUE_CAPACITY};
use crate::runtime_store::connection::RuntimeStoreConnection;
use crate::runtime_store::deadline::ensure_before;
use crate::runtime_store::error::{RuntimeStoreError, RuntimeStoreErrorKind};
use crate::runtime_store::migrations::{migrate_and_validate_until, CURRENT_SCHEMA_VERSION};
use crate::runtime_store::types::{StorageRuntimeErrorCode, StorageRuntimeStatus};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) const PRODUCTION_SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

enum RuntimeStoreRequest {
    Initialize {
        config: RuntimeStoreConfig,
        deadline: Instant,
        reply: Option<SyncSender<StorageRuntimeStatus>>,
    },
    ReadStatus {
        reply: SyncSender<StorageRuntimeStatus>,
    },
    Shutdown {
        deadline: Instant,
        reply: SyncSender<Result<(), RuntimeStoreError>>,
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

struct RuntimeStoreManagerInner {
    sender: SyncSender<RuntimeStoreRequest>,
    status: Arc<RwLock<StorageRuntimeStatus>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    worker_exit: Mutex<Receiver<WorkerExit>>,
    #[cfg(test)]
    last_worker_exit: Arc<Mutex<Option<WorkerExit>>>,
    accepting: Arc<AtomicBool>,
    ordinary_deadline: RwLock<Duration>,
    shutdown_phase: Mutex<ShutdownPhase>,
    shutdown_complete: Condvar,
    #[cfg(test)]
    suppress_exit_signal: Arc<AtomicBool>,
    #[cfg(test)]
    joined_after_exit: AtomicBool,
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
        #[cfg(test)]
        let last_worker_exit = Arc::new(Mutex::new(None));
        let (sender, receiver) = mpsc::sync_channel(STORAGE_QUEUE_CAPACITY);
        let (worker_exit_sender, worker_exit) = mpsc::sync_channel(1);
        #[cfg(test)]
        let suppress_exit_signal = Arc::new(AtomicBool::new(false));

        let worker_status = Arc::clone(&status);
        let worker_accepting = Arc::clone(&accepting);
        #[cfg(test)]
        let worker_last_exit = Arc::clone(&last_worker_exit);
        #[cfg(test)]
        let worker_suppress_exit_signal = Arc::clone(&suppress_exit_signal);
        let join_handle = thread::Builder::new()
            .name("daarion-runtime-store".to_string())
            .spawn(move || {
                let exit = match catch_unwind(AssertUnwindSafe(|| {
                    run_worker(receiver, &worker_status, last_start_time_ms)
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
                let suppress = worker_suppress_exit_signal.load(Ordering::Acquire);
                #[cfg(not(test))]
                let suppress = false;
                if !suppress {
                    let _ = worker_exit_sender.try_send(exit);
                }
            });

        let (join_handle, spawned) = match join_handle {
            Ok(handle) => (Some(handle), true),
            Err(_) => {
                store_status(
                    &status,
                    failure_status(RuntimeStoreError::unavailable(), last_start_time_ms),
                );
                (None, false)
            }
        };
        accepting.store(spawned, Ordering::Release);

        Self {
            inner: Arc::new(RuntimeStoreManagerInner {
                sender,
                status,
                join_handle: Mutex::new(join_handle),
                worker_exit: Mutex::new(worker_exit),
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
                joined_after_exit: AtomicBool::new(false),
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
        self.inner.joined_after_exit.load(Ordering::Acquire)
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
    let (reply, response) = mpsc::sync_channel(1);
    let send_result = try_send_with_deadline(
        &inner.sender,
        RuntimeStoreRequest::Shutdown { deadline, reply },
        deadline,
    );
    let close_result = match send_result {
        Ok(()) => response
            .recv_timeout(remaining_or_zero(deadline))
            .map_err(|_| RuntimeStoreError::unavailable())
            .and_then(|result| result),
        Err(error) => Err(error),
    };

    let exit = inner
        .worker_exit
        .lock()
        .map_err(|_| RuntimeStoreError::internal())
        .and_then(|receiver| match receiver.try_recv() {
            Ok(exit) => Ok(exit),
            Err(TryRecvError::Empty) => receiver
                .recv_timeout(remaining_or_zero(deadline))
                .map_err(|_| RuntimeStoreError::unavailable()),
            Err(TryRecvError::Disconnected) => Err(RuntimeStoreError::unavailable()),
        });

    let projected_failure = projected_internal_error(&inner.status);
    let mut result = match (close_result, exit) {
        (Err(error), Ok(WorkerExit::ControlledFailure))
            if error.kind == RuntimeStoreErrorKind::Unavailable =>
        {
            Err(projected_failure.unwrap_or(error))
        }
        (Err(error), _) => Err(error),
        (Ok(()), Ok(WorkerExit::CleanShutdown)) => Ok(()),
        (Ok(()), Ok(WorkerExit::ControlledFailure)) => Err(RuntimeStoreError::internal()),
        (Ok(()), Ok(_)) | (Ok(()), Err(_)) => Err(RuntimeStoreError::unavailable()),
    };

    if exit.is_ok() {
        match inner.join_handle.lock() {
            Ok(mut guard) => {
                if let Some(handle) = guard.take() {
                    if handle.join().is_err() {
                        result = Err(RuntimeStoreError::internal());
                    }
                    #[cfg(test)]
                    inner.joined_after_exit.store(true, Ordering::Release);
                }
            }
            Err(_) => result = Err(RuntimeStoreError::internal()),
        }
    } else if let Ok(mut guard) = inner.join_handle.lock() {
        let _ = guard.take();
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
    shared_status: &RwLock<StorageRuntimeStatus>,
    last_start_time_ms: u64,
) -> WorkerExit {
    let mut connection: Option<RuntimeStoreConnection> = None;
    loop {
        let request = match receiver.recv() {
            Ok(request) => request,
            Err(_) => {
                if let Some(opened) = connection.take() {
                    let _ = opened.close_until(Instant::now() + Duration::from_secs(2));
                }
                return WorkerExit::ChannelDisconnected;
            }
        };
        match request {
            RuntimeStoreRequest::Initialize {
                config,
                deadline,
                reply,
            } => {
                let status = if connection.is_some() {
                    current_shared_status(shared_status, last_start_time_ms)
                } else {
                    match initialize_connection(&config, deadline, last_start_time_ms) {
                        Ok((opened, status)) => {
                            connection = Some(opened);
                            status
                        }
                        Err(error) => failure_status(error, last_start_time_ms),
                    }
                };
                store_status(shared_status, status.clone());
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
                store_status(shared_status, status.clone());
                if reply.try_send(status).is_err() {
                    return WorkerExit::ChannelDisconnected;
                }
            }
            RuntimeStoreRequest::Shutdown { deadline, reply } => {
                let close_result = connection.take().map_or(Ok(()), |opened| {
                    opened.close_until(reserve_worker_exit_budget(deadline))
                });
                if let Err(error) = close_result {
                    store_status(shared_status, failure_status(error, last_start_time_ms));
                }
                let _ = reply.try_send(close_result);
                return if close_result.is_ok() {
                    WorkerExit::CleanShutdown
                } else {
                    WorkerExit::ControlledFailure
                };
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

fn initialize_connection(
    config: &RuntimeStoreConfig,
    deadline: Instant,
    last_start_time_ms: u64,
) -> Result<(RuntimeStoreConnection, StorageRuntimeStatus), RuntimeStoreError> {
    ensure_before(deadline)?;
    let (mut opened, watchdog) = RuntimeStoreConnection::open_for_initialization(config, deadline)?;

    #[cfg(test)]
    if config.initialization_test_hook
        == crate::runtime_store::config::InitializationTestHook::LongQueryBeforeMigration
    {
        crate::runtime_store::migrations::run_long_query(&opened.connection)?;
    }

    #[cfg(test)]
    let schema_version = if config.initialization_test_hook
        == crate::runtime_store::config::InitializationTestHook::LongQueryInsideMigration
    {
        crate::runtime_store::migrations::migrate_and_validate_with_test_interrupt(
            &mut opened.connection,
            deadline,
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
    opened.revalidate_artifacts()?;
    ensure_before(deadline)?;
    let status = build_healthy_status(&opened, last_start_time_ms)?;
    ensure_before(deadline)?;
    if watchdog.finish()? {
        return Err(RuntimeStoreError::deadline_exceeded());
    }
    ensure_before(deadline)?;
    Ok((opened, status))
}

fn build_healthy_status(
    opened: &RuntimeStoreConnection,
    last_start_time_ms: u64,
) -> Result<StorageRuntimeStatus, RuntimeStoreError> {
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
