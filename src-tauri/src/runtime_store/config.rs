use std::path::PathBuf;
use std::time::Duration;

pub(crate) const STORAGE_QUEUE_CAPACITY: usize = 128;
pub(crate) const ORDINARY_OPERATION_DEADLINE: Duration = Duration::from_secs(10);
pub(crate) const MIGRATION_DEADLINE: Duration = Duration::from_secs(120);
pub(crate) const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const DATABASE_WARNING_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(crate) const DATABASE_HARD_LIMIT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub(crate) const OPERATIONAL_RESERVE_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const CREATE_GROWTH_ENVELOPE_BYTES: u64 = 8 * 1024 * 1024;
pub(crate) const APPEND_GROWTH_ENVELOPE_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const WAL_AUTOCHECKPOINT_PAGES: u32 = 128;
pub(crate) const WAL_HARD_CEILING_BYTES: u64 = 10 * 1024 * 1024;
pub(crate) const WAL_CREATE_GROWTH_BOUND_BYTES: u64 = 2 * 1024 * 1024;
pub(crate) const WAL_APPEND_GROWTH_BOUND_BYTES: u64 = 4 * 1024 * 1024;
pub(crate) const CHECKPOINT_RECOVERY_OVERHEAD_BYTES: u64 = 2 * 1024 * 1024;
pub(crate) const REQUIRED_PAGE_SIZE_BYTES: u32 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContentStorageLimits {
    pub(crate) operational_reserve_bytes: u64,
    pub(crate) create_growth_envelope_bytes: u64,
    pub(crate) append_growth_envelope_bytes: u64,
    pub(crate) wal_hard_ceiling_bytes: u64,
    pub(crate) wal_create_growth_bound_bytes: u64,
    pub(crate) wal_append_growth_bound_bytes: u64,
    pub(crate) checkpoint_recovery_overhead_bytes: u64,
    pub(crate) required_page_size_bytes: u32,
    pub(crate) wal_autocheckpoint_pages: u32,
}

impl ContentStorageLimits {
    const fn production() -> Self {
        Self {
            operational_reserve_bytes: OPERATIONAL_RESERVE_BYTES,
            create_growth_envelope_bytes: CREATE_GROWTH_ENVELOPE_BYTES,
            append_growth_envelope_bytes: APPEND_GROWTH_ENVELOPE_BYTES,
            wal_hard_ceiling_bytes: WAL_HARD_CEILING_BYTES,
            wal_create_growth_bound_bytes: WAL_CREATE_GROWTH_BOUND_BYTES,
            wal_append_growth_bound_bytes: WAL_APPEND_GROWTH_BOUND_BYTES,
            checkpoint_recovery_overhead_bytes: CHECKPOINT_RECOVERY_OVERHEAD_BYTES,
            required_page_size_bytes: REQUIRED_PAGE_SIZE_BYTES,
            wal_autocheckpoint_pages: WAL_AUTOCHECKPOINT_PAGES,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitializationTestStage {
    BeforePathPreparation,
    BeforeInterruptRegistration,
    AfterInterruptRegistration,
}

#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub(crate) enum InitializationTestHook {
    #[default]
    None,
    LongQueryBeforeMigration,
    LongQueryInsideMigration,
    LongQueryDuringIntegrity,
    Block {
        stage: InitializationTestStage,
        entered: std::sync::mpsc::SyncSender<()>,
        release: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<()>>>,
        panic_after_release: bool,
    },
}

#[cfg(test)]
impl InitializationTestHook {
    pub(crate) fn blocking(
        stage: InitializationTestStage,
        panic_after_release: bool,
    ) -> (
        Self,
        std::sync::mpsc::Receiver<()>,
        std::sync::mpsc::SyncSender<()>,
    ) {
        let (entered, entered_response) = std::sync::mpsc::sync_channel(1);
        let (release, release_response) = std::sync::mpsc::sync_channel(1);
        (
            Self::Block {
                stage,
                entered,
                release: std::sync::Arc::new(std::sync::Mutex::new(release_response)),
                panic_after_release,
            },
            entered_response,
            release,
        )
    }

    pub(crate) fn wait_at(&self, stage: InitializationTestStage) -> Result<(), ()> {
        let Self::Block {
            stage: configured,
            entered,
            release,
            panic_after_release,
        } = self
        else {
            return Ok(());
        };
        if *configured != stage {
            return Ok(());
        }
        entered.try_send(()).map_err(|_| ())?;
        release.lock().map_err(|_| ())?.recv().map_err(|_| ())?;
        assert!(
            !panic_after_release,
            "runtime_store_cancelled_initialization_test_panic"
        );
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeStoreConfig {
    pub(crate) app_local_data_root: PathBuf,
    pub(crate) ordinary_deadline: Duration,
    pub(crate) migration_deadline: Duration,
    pub(crate) busy_timeout: Duration,
    pub(crate) database_warning_threshold_bytes: u64,
    pub(crate) database_hard_limit_bytes: u64,
    pub(crate) content_limits: ContentStorageLimits,
    #[cfg(test)]
    pub(crate) initialization_test_hook: InitializationTestHook,
}

impl RuntimeStoreConfig {
    pub(crate) fn production(app_local_data_root: PathBuf) -> Self {
        Self {
            app_local_data_root,
            ordinary_deadline: ORDINARY_OPERATION_DEADLINE,
            migration_deadline: MIGRATION_DEADLINE,
            busy_timeout: BUSY_TIMEOUT,
            database_warning_threshold_bytes: DATABASE_WARNING_THRESHOLD_BYTES,
            database_hard_limit_bytes: DATABASE_HARD_LIMIT_BYTES,
            content_limits: ContentStorageLimits::production(),
            #[cfg(test)]
            initialization_test_hook: InitializationTestHook::None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(app_local_data_root: PathBuf) -> Self {
        Self {
            app_local_data_root,
            ordinary_deadline: Duration::from_secs(2),
            migration_deadline: Duration::from_secs(5),
            busy_timeout: Duration::from_millis(150),
            database_warning_threshold_bytes: DATABASE_WARNING_THRESHOLD_BYTES,
            database_hard_limit_bytes: DATABASE_HARD_LIMIT_BYTES,
            content_limits: ContentStorageLimits::production(),
            initialization_test_hook: InitializationTestHook::None,
        }
    }
}
