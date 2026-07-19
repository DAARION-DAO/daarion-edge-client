use crate::runtime_store::error::RuntimeStoreError;
use rusqlite::{Connection, InterruptHandle};
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct ActiveInitialization {
    generation: u64,
    interrupt: Arc<InterruptHandle>,
}

/// Private lifecycle control shared by the manager and its single SQLite owner.
///
/// Lock ordering is deliberately one-way: the short publication gate may be
/// taken before the active-registration lock, while the registration path never
/// takes the publication gate. Neither lock is held while SQLite or filesystem
/// work runs, or while `InterruptHandle::interrupt` is invoked.
pub(crate) struct RuntimeStoreControl {
    shutdown_requested: AtomicBool,
    next_generation: AtomicU64,
    publication_gate: Mutex<()>,
    active_initialization: Mutex<Option<ActiveInitialization>>,
    #[cfg(test)]
    active_watchdogs: AtomicUsize,
}

impl RuntimeStoreControl {
    pub(crate) fn new() -> Self {
        Self {
            shutdown_requested: AtomicBool::new(false),
            next_generation: AtomicU64::new(1),
            publication_gate: Mutex::new(()),
            active_initialization: Mutex::new(None),
            #[cfg(test)]
            active_watchdogs: AtomicUsize::new(0),
        }
    }

    pub(crate) fn begin_initialization(
        self: &Arc<Self>,
    ) -> Result<InitializationAttempt, RuntimeStoreError> {
        self.ensure_running()?;
        let generation = self.next_generation.fetch_add(1, Ordering::AcqRel);
        if generation == u64::MAX {
            return Err(RuntimeStoreError::internal());
        }
        Ok(InitializationAttempt {
            control: Arc::clone(self),
            generation,
        })
    }

    pub(crate) fn request_shutdown(&self, deadline: Instant) -> Result<(), RuntimeStoreError> {
        let publication = self
            .publication_gate
            .lock()
            .map_err(|_| RuntimeStoreError::internal())?;
        self.shutdown_requested.store(true, Ordering::Release);
        drop(publication);

        // `sqlite3_interrupt` only affects statements that are running when it
        // is observed. Pulse the registered generation until its RAII guard is
        // cleared so shutdown cannot land in the narrow gap between two
        // initialization statements and then wait for the migration deadline.
        // No lifecycle lock is held while interrupting or waiting.
        loop {
            let interrupt = self
                .active_initialization
                .lock()
                .map_err(|_| RuntimeStoreError::internal())?
                .as_ref()
                .map(|active| Arc::clone(&active.interrupt));
            let Some(interrupt) = interrupt else {
                break;
            };
            interrupt.interrupt();
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            if remaining.is_zero() {
                break;
            }
            thread::sleep(remaining.min(Duration::from_millis(1)));
        }
        Ok(())
    }

    pub(crate) fn ensure_running(&self) -> Result<(), RuntimeStoreError> {
        if self.shutdown_requested.load(Ordering::Acquire) {
            Err(RuntimeStoreError::unavailable())
        } else {
            Ok(())
        }
    }

    pub(crate) fn shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Acquire)
    }

    pub(crate) fn publish_if_running(&self, publish: impl FnOnce()) -> bool {
        let Ok(_publication) = self.publication_gate.lock() else {
            return false;
        };
        if self.shutdown_requested() {
            return false;
        }
        publish();
        true
    }

    #[cfg(test)]
    pub(crate) fn has_active_initialization(&self) -> bool {
        self.active_initialization
            .lock()
            .map(|active| active.is_some())
            .unwrap_or(false)
    }

    #[cfg(test)]
    pub(crate) fn active_watchdogs(&self) -> usize {
        self.active_watchdogs.load(Ordering::Acquire)
    }
}

pub(crate) struct InitializationAttempt {
    control: Arc<RuntimeStoreControl>,
    generation: u64,
}

impl InitializationAttempt {
    pub(crate) fn ensure_running(&self) -> Result<(), RuntimeStoreError> {
        self.control.ensure_running()
    }

    pub(crate) fn register_interrupt(
        &self,
        connection: &Connection,
    ) -> Result<ActiveInitializationRegistration, RuntimeStoreError> {
        self.ensure_running()?;
        let interrupt = Arc::new(connection.get_interrupt_handle());
        {
            let mut active = self
                .control
                .active_initialization
                .lock()
                .map_err(|_| RuntimeStoreError::internal())?;
            if self.control.shutdown_requested() || active.is_some() {
                return Err(RuntimeStoreError::unavailable());
            }
            *active = Some(ActiveInitialization {
                generation: self.generation,
                interrupt: Arc::clone(&interrupt),
            });
        }
        if self.control.shutdown_requested() {
            interrupt.interrupt();
            self.clear_registration();
            return Err(RuntimeStoreError::unavailable());
        }
        #[cfg(test)]
        self.control.active_watchdogs.fetch_add(1, Ordering::AcqRel);
        Ok(ActiveInitializationRegistration {
            attempt: Some(Self {
                control: Arc::clone(&self.control),
                generation: self.generation,
            }),
        })
    }

    fn clear_registration(&self) {
        if let Ok(mut active) = self.control.active_initialization.lock() {
            if active
                .as_ref()
                .is_some_and(|registered| registered.generation == self.generation)
            {
                *active = None;
            }
        }
    }
}

pub(crate) struct ActiveInitializationRegistration {
    attempt: Option<InitializationAttempt>,
}

impl ActiveInitializationRegistration {
    pub(crate) fn clear(&mut self) {
        if let Some(attempt) = self.attempt.take() {
            attempt.clear_registration();
            #[cfg(test)]
            attempt
                .control
                .active_watchdogs
                .fetch_sub(1, Ordering::AcqRel);
        }
    }
}

impl Drop for ActiveInitializationRegistration {
    fn drop(&mut self) {
        self.clear();
    }
}
