use crate::runtime_store::error::RuntimeStoreError;
use crate::runtime_store::RuntimeStoreManager;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct RuntimeStoreLifecycle {
    shutdown_invoked: AtomicBool,
}

impl RuntimeStoreLifecycle {
    pub(crate) const fn new() -> Self {
        Self {
            shutdown_invoked: AtomicBool::new(false),
        }
    }

    pub(crate) fn on_run_event(
        &self,
        manager: &RuntimeStoreManager,
        event: &tauri::RunEvent,
    ) -> Option<Result<(), RuntimeStoreError>> {
        if matches!(
            event,
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
        ) {
            self.shutdown_once(manager)
        } else {
            None
        }
    }

    pub(crate) fn shutdown_once(
        &self,
        manager: &RuntimeStoreManager,
    ) -> Option<Result<(), RuntimeStoreError>> {
        if self
            .shutdown_invoked
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return None;
        }
        Some(manager.production_shutdown())
    }
}

impl Default for RuntimeStoreLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
