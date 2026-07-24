use std::path::PathBuf;
use std::time::Duration;

pub(crate) const STORAGE_QUEUE_CAPACITY: usize = 128;
pub(crate) const ORDINARY_OPERATION_DEADLINE: Duration = Duration::from_secs(10);
pub(crate) const MIGRATION_DEADLINE: Duration = Duration::from_secs(120);
pub(crate) const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const DATABASE_WARNING_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(crate) const DATABASE_HARD_LIMIT_BYTES: u64 = 4 * 1024 * 1024 * 1024;

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
            initialization_test_hook: InitializationTestHook::None,
        }
    }
}
