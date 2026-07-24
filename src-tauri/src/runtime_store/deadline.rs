use crate::runtime_store::control::{ActiveInitializationRegistration, InitializationAttempt};
use crate::runtime_store::error::RuntimeStoreError;
use rusqlite::Connection;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[cfg(test)]
static ACTIVE_WATCHDOGS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static STARTED_WATCHDOGS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static FINISHED_WATCHDOGS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) fn ensure_before(deadline: Instant) -> Result<(), RuntimeStoreError> {
    if Instant::now() >= deadline {
        Err(RuntimeStoreError::deadline_exceeded())
    } else {
        Ok(())
    }
}

pub(crate) fn remaining(deadline: Instant) -> Result<Duration, RuntimeStoreError> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(RuntimeStoreError::deadline_exceeded)
}

pub(crate) struct SqliteInterruptGuard {
    cancel: Option<SyncSender<()>>,
    thread: Option<JoinHandle<()>>,
    expired: Arc<AtomicBool>,
    registration: Option<ActiveInitializationRegistration>,
}

impl SqliteInterruptGuard {
    pub(crate) fn start(
        connection: &Connection,
        deadline: Instant,
    ) -> Result<Self, RuntimeStoreError> {
        Self::start_inner(connection, deadline, None)
    }

    pub(crate) fn start_initialization(
        connection: &Connection,
        deadline: Instant,
        attempt: &InitializationAttempt,
    ) -> Result<Self, RuntimeStoreError> {
        let registration = attempt.register_interrupt(connection)?;
        Self::start_inner(connection, deadline, Some(registration))
    }

    fn start_inner(
        connection: &Connection,
        deadline: Instant,
        registration: Option<ActiveInitializationRegistration>,
    ) -> Result<Self, RuntimeStoreError> {
        let wait = remaining(deadline)?;
        let interrupt = connection.get_interrupt_handle();
        let (cancel, receiver) = mpsc::sync_channel(1);
        let expired = Arc::new(AtomicBool::new(false));
        let worker_expired = Arc::clone(&expired);
        let thread = thread::Builder::new()
            .name("daarion-sqlite-deadline".to_string())
            .spawn(move || {
                #[cfg(test)]
                {
                    ACTIVE_WATCHDOGS.fetch_add(1, Ordering::AcqRel);
                    STARTED_WATCHDOGS.fetch_add(1, Ordering::AcqRel);
                }
                if receiver.recv_timeout(wait).is_err() {
                    worker_expired.store(true, Ordering::Release);
                    interrupt.interrupt();
                }
                #[cfg(test)]
                {
                    ACTIVE_WATCHDOGS.fetch_sub(1, Ordering::AcqRel);
                    FINISHED_WATCHDOGS.fetch_add(1, Ordering::AcqRel);
                }
            })
            .map_err(|_| RuntimeStoreError::internal())?;
        Ok(Self {
            cancel: Some(cancel),
            thread: Some(thread),
            expired,
            registration,
        })
    }

    pub(crate) fn finish(mut self) -> Result<bool, RuntimeStoreError> {
        self.disarm_and_join()?;
        Ok(self.expired.load(Ordering::Acquire))
    }

    fn disarm_and_join(&mut self) -> Result<(), RuntimeStoreError> {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.try_send(());
        }
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| RuntimeStoreError::internal())?;
        }
        if let Some(mut registration) = self.registration.take() {
            registration.clear();
        }
        Ok(())
    }
}

impl Drop for SqliteInterruptGuard {
    fn drop(&mut self) {
        let _ = self.disarm_and_join();
    }
}

#[cfg(test)]
pub(crate) fn active_watchdogs() -> usize {
    ACTIVE_WATCHDOGS.load(Ordering::Acquire)
}

#[cfg(test)]
pub(crate) fn watchdog_counts() -> (usize, usize) {
    (
        STARTED_WATCHDOGS.load(Ordering::Acquire),
        FINISHED_WATCHDOGS.load(Ordering::Acquire),
    )
}
