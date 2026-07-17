use crate::runtime_store::config::{RuntimeStoreConfig, STORAGE_QUEUE_CAPACITY};
use crate::runtime_store::connection::RuntimeStoreConnection;
use crate::runtime_store::error::{RuntimeStoreError, RuntimeStoreErrorKind};
use crate::runtime_store::migrations::migrate_and_validate;
use crate::runtime_store::types::StorageRuntimeStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

enum RuntimeStoreRequest {
    Initialize {
        config: RuntimeStoreConfig,
        reply: SyncSender<StorageRuntimeStatus>,
    },
    ReadStatus {
        reply: SyncSender<StorageRuntimeStatus>,
    },
    Shutdown {
        reply: SyncSender<Result<(), RuntimeStoreError>>,
    },
    #[cfg(test)]
    Hold {
        duration: Duration,
        reply: SyncSender<()>,
    },
}

struct RuntimeStoreManagerInner {
    sender: SyncSender<RuntimeStoreRequest>,
    status: Arc<RwLock<StorageRuntimeStatus>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    accepting: AtomicBool,
    ordinary_deadline: RwLock<Duration>,
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
        let (sender, receiver) = mpsc::sync_channel(STORAGE_QUEUE_CAPACITY);
        let worker_status = Arc::clone(&status);
        let join_handle = thread::Builder::new()
            .name("daarion-runtime-store".to_string())
            .spawn(move || run_worker(receiver, worker_status, last_start_time_ms));
        let (join_handle, accepting) = match join_handle {
            Ok(handle) => (Some(handle), true),
            Err(_) => {
                store_status(
                    &status,
                    failure_status(RuntimeStoreError::unavailable(), last_start_time_ms),
                );
                (None, false)
            }
        };
        Self {
            inner: Arc::new(RuntimeStoreManagerInner {
                sender,
                status,
                join_handle: Mutex::new(join_handle),
                accepting: AtomicBool::new(accepting),
                ordinary_deadline: RwLock::new(
                    crate::runtime_store::config::ORDINARY_OPERATION_DEADLINE,
                ),
            }),
        }
    }

    pub(crate) fn start_initialization(&self, config: RuntimeStoreConfig) {
        self.store_ordinary_deadline(config.ordinary_deadline);
        let (reply, _response) = mpsc::sync_channel(1);
        if self
            .send_with_deadline(
                RuntimeStoreRequest::Initialize { config, reply },
                Duration::from_millis(250),
            )
            .is_err()
        {
            self.store_failure(RuntimeStoreError::unavailable());
        }
    }

    pub(crate) fn fail_path_resolution(&self) {
        self.store_failure(RuntimeStoreError::path_invalid());
    }

    pub(crate) fn read_status(&self) -> StorageRuntimeStatus {
        if !self.inner.accepting.load(Ordering::Acquire) {
            return self.internal_failure_status();
        }
        let deadline = self.ordinary_deadline();
        let fallback = self.current_status();
        let (reply, response) = mpsc::sync_channel(1);
        if self
            .send_with_deadline(RuntimeStoreRequest::ReadStatus { reply }, deadline)
            .is_err()
        {
            return fallback;
        }
        response.recv_timeout(deadline).unwrap_or(fallback)
    }

    pub(crate) fn internal_failure_status(&self) -> StorageRuntimeStatus {
        let last_start_time_ms = self.current_status().last_start_time_ms;
        StorageRuntimeStatus::failed(
            last_start_time_ms,
            RuntimeStoreError::internal().public_state(),
            RuntimeStoreError::internal().public_code(),
        )
    }

    fn current_status(&self) -> StorageRuntimeStatus {
        self.inner
            .status
            .read()
            .map(|status| status.clone())
            .unwrap_or_else(|_| {
                StorageRuntimeStatus::failed(
                    current_time_ms(),
                    RuntimeStoreError::internal().public_state(),
                    RuntimeStoreError::internal().public_code(),
                )
            })
    }

    fn store_failure(&self, error: RuntimeStoreError) {
        let last_start_time_ms = self.current_status().last_start_time_ms;
        store_status(
            &self.inner.status,
            StorageRuntimeStatus::failed(
                last_start_time_ms,
                error.public_state(),
                error.public_code(),
            ),
        );
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
        deadline: Duration,
    ) -> Result<(), RuntimeStoreError> {
        let expires_at = Instant::now() + deadline;
        try_send_with_deadline(&self.inner.sender, request, expires_at)
    }

    #[cfg(test)]
    pub(crate) fn initialize_for_test(&self, config: RuntimeStoreConfig) -> StorageRuntimeStatus {
        self.store_ordinary_deadline(config.ordinary_deadline);
        let deadline = config.migration_deadline;
        let fallback = self.internal_failure_status();
        let (reply, response) = mpsc::sync_channel(1);
        if self
            .send_with_deadline(RuntimeStoreRequest::Initialize { config, reply }, deadline)
            .is_err()
        {
            return fallback;
        }
        response.recv_timeout(deadline).unwrap_or(fallback)
    }

    #[cfg(test)]
    pub(crate) fn hold_worker_for_test(&self, duration: Duration) -> Result<(), RuntimeStoreError> {
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(
            RuntimeStoreRequest::Hold { duration, reply },
            Duration::from_millis(100),
        )?;
        response
            .recv_timeout(duration + Duration::from_secs(1))
            .map_err(|_| RuntimeStoreError::unavailable())
    }

    #[cfg(test)]
    pub(crate) fn shutdown_for_test(&self, deadline: Duration) -> Result<(), RuntimeStoreError> {
        self.shutdown(deadline)
    }

    #[cfg(test)]
    fn shutdown(&self, deadline: Duration) -> Result<(), RuntimeStoreError> {
        if !self.inner.accepting.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let (reply, response) = mpsc::sync_channel(1);
        self.send_with_deadline(RuntimeStoreRequest::Shutdown { reply }, deadline)?;
        let close_result = response
            .recv_timeout(deadline)
            .map_err(|_| RuntimeStoreError::unavailable())?;
        if let Ok(mut guard) = self.inner.join_handle.lock() {
            if let Some(handle) = guard.take() {
                handle.join().map_err(|_| RuntimeStoreError::internal())?;
            }
        }
        close_result
    }
}

impl Default for RuntimeStoreManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RuntimeStoreManagerInner {
    fn drop(&mut self) {
        self.accepting.store(false, Ordering::Release);
        let (reply, response) = mpsc::sync_channel(1);
        let deadline = Duration::from_secs(2);
        if try_send_with_deadline(
            &self.sender,
            RuntimeStoreRequest::Shutdown { reply },
            Instant::now() + deadline,
        )
        .is_ok()
            && response.recv_timeout(deadline).is_ok()
        {
            if let Ok(mut guard) = self.join_handle.lock() {
                if let Some(handle) = guard.take() {
                    let _ = handle.join();
                }
            }
            return;
        }
        if let Ok(mut guard) = self.join_handle.lock() {
            let _ = guard.take();
        }
    }
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

fn run_worker(
    receiver: Receiver<RuntimeStoreRequest>,
    shared_status: Arc<RwLock<StorageRuntimeStatus>>,
    last_start_time_ms: u64,
) {
    let mut connection: Option<RuntimeStoreConnection> = None;
    while let Ok(request) = receiver.recv() {
        match request {
            RuntimeStoreRequest::Initialize { config, reply } => {
                if connection.is_some() {
                    let status = shared_status
                        .read()
                        .map(|status| status.clone())
                        .unwrap_or_else(|_| {
                            failure_status(RuntimeStoreError::internal(), last_start_time_ms)
                        });
                    let _ = reply.try_send(status);
                    continue;
                }
                let started_at = Instant::now();
                let result = initialize_connection(&config, last_start_time_ms);
                let status = match result {
                    Ok(opened) if started_at.elapsed() <= config.migration_deadline => {
                        let status = build_healthy_status(&opened, last_start_time_ms)
                            .unwrap_or_else(|error| failure_status(error, last_start_time_ms));
                        if status.initialized {
                            connection = Some(opened);
                        } else {
                            let _ = opened.close();
                        }
                        status
                    }
                    Ok(opened) => {
                        let _ = opened.close();
                        failure_status(RuntimeStoreError::unavailable(), last_start_time_ms)
                    }
                    Err(error) => failure_status(error, last_start_time_ms),
                };
                store_status(&shared_status, status.clone());
                let _ = reply.try_send(status);
            }
            RuntimeStoreRequest::ReadStatus { reply } => {
                let status = match connection.as_ref() {
                    Some(opened) => {
                        let status = build_healthy_status(opened, last_start_time_ms)
                            .unwrap_or_else(|error| failure_status(error, last_start_time_ms));
                        store_status(&shared_status, status.clone());
                        status
                    }
                    None => shared_status
                        .read()
                        .map(|status| status.clone())
                        .unwrap_or_else(|_| {
                            failure_status(RuntimeStoreError::internal(), last_start_time_ms)
                        }),
                };
                let _ = reply.try_send(status);
            }
            RuntimeStoreRequest::Shutdown { reply } => {
                let close_result = connection
                    .take()
                    .map_or(Ok(()), RuntimeStoreConnection::close);
                let _ = reply.try_send(close_result);
                break;
            }
            #[cfg(test)]
            RuntimeStoreRequest::Hold { duration, reply } => {
                thread::sleep(duration);
                let _ = reply.try_send(());
            }
        }
    }
    if let Some(opened) = connection.take() {
        let _ = opened.close();
    }
}

fn initialize_connection(
    config: &RuntimeStoreConfig,
    _last_start_time_ms: u64,
) -> Result<RuntimeStoreConnection, RuntimeStoreError> {
    let mut opened = RuntimeStoreConnection::open(config)?;
    migrate_and_validate(&mut opened.connection)?;
    opened.revalidate_artifacts()?;
    Ok(opened)
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
        crate::runtime_store::migrations::CURRENT_SCHEMA_VERSION,
        rusqlite::version().to_string(),
        database_size_bytes,
        opened.persistence_state,
        opened.database_warning_threshold_bytes,
        opened.database_hard_limit_bytes,
    ))
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

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}
