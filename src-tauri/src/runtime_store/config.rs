use std::path::PathBuf;
use std::time::Duration;

pub(crate) const STORAGE_QUEUE_CAPACITY: usize = 128;
pub(crate) const ORDINARY_OPERATION_DEADLINE: Duration = Duration::from_secs(10);
pub(crate) const MIGRATION_DEADLINE: Duration = Duration::from_secs(120);
pub(crate) const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const DATABASE_WARNING_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(crate) const DATABASE_HARD_LIMIT_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum InitializationTestHook {
    #[default]
    None,
    LongQueryBeforeMigration,
    LongQueryInsideMigration,
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
