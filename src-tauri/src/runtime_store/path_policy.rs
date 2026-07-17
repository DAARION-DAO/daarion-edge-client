use crate::runtime_store::error::RuntimeStoreError;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
#[cfg(not(unix))]
use std::time::SystemTime;

const RUNTIME_STATE_DIRECTORY: &str = "runtime-state";
const DATABASE_FILENAME: &str = "runtime-state-v1.sqlite3";
const RESERVED_DIRECTORIES: [&str; 3] = ["tmp", "backups", "exports"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(not(unix))]
    length: u64,
    #[cfg(not(unix))]
    created: Option<SystemTime>,
    #[cfg(not(unix))]
    modified: Option<SystemTime>,
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedStoragePaths {
    pub(crate) database_path: PathBuf,
    pub(crate) existed_before_open: bool,
    app_root: PathBuf,
    app_root_identity: FileIdentity,
    runtime_root: PathBuf,
    runtime_root_identity: FileIdentity,
    identity_before_open: Option<FileIdentity>,
}

pub(crate) fn prepare_storage_paths(
    app_local_data_root: &Path,
) -> Result<PreparedStoragePaths, RuntimeStoreError> {
    validate_absolute_normal_path(app_local_data_root)?;
    fs::create_dir_all(app_local_data_root).map_err(|error| RuntimeStoreError::from_io(&error))?;
    reject_symlink_components(app_local_data_root)?;
    let app_root_identity = directory_identity(app_local_data_root)?;
    let canonical_app_root = fs::canonicalize(app_local_data_root)
        .map_err(|error| RuntimeStoreError::from_io(&error))?;
    ensure_directory(&canonical_app_root)?;
    if directory_identity(&canonical_app_root)? != app_root_identity {
        return Err(RuntimeStoreError::path_invalid());
    }
    enforce_private_directory(&canonical_app_root)?;

    let runtime_root = canonical_app_root.join(RUNTIME_STATE_DIRECTORY);
    reject_existing_symlink(&runtime_root)?;
    fs::create_dir_all(&runtime_root).map_err(|error| RuntimeStoreError::from_io(&error))?;
    let runtime_root_identity = directory_identity(&runtime_root)?;
    let canonical_runtime_root =
        fs::canonicalize(&runtime_root).map_err(|error| RuntimeStoreError::from_io(&error))?;
    if canonical_runtime_root.parent() != Some(canonical_app_root.as_path()) {
        return Err(RuntimeStoreError::path_invalid());
    }
    ensure_directory(&canonical_runtime_root)?;
    if directory_identity(&canonical_runtime_root)? != runtime_root_identity {
        return Err(RuntimeStoreError::path_invalid());
    }
    enforce_private_directory(&canonical_runtime_root)?;

    for directory in RESERVED_DIRECTORIES {
        let reserved_path = canonical_runtime_root.join(directory);
        reject_existing_symlink(&reserved_path)?;
        fs::create_dir_all(&reserved_path).map_err(|error| RuntimeStoreError::from_io(&error))?;
        let canonical_reserved =
            fs::canonicalize(&reserved_path).map_err(|error| RuntimeStoreError::from_io(&error))?;
        if canonical_reserved.parent() != Some(canonical_runtime_root.as_path()) {
            return Err(RuntimeStoreError::path_invalid());
        }
        ensure_directory(&canonical_reserved)?;
        enforce_private_directory(&canonical_reserved)?;
    }

    let database_path = canonical_runtime_root.join(DATABASE_FILENAME);
    if database_path.parent() != Some(canonical_runtime_root.as_path()) {
        return Err(RuntimeStoreError::path_invalid());
    }
    let identity_before_open = existing_regular_file_identity(&database_path)?;

    Ok(PreparedStoragePaths {
        database_path,
        existed_before_open: identity_before_open.is_some(),
        app_root: canonical_app_root,
        app_root_identity,
        runtime_root: canonical_runtime_root,
        runtime_root_identity,
        identity_before_open,
    })
}

pub(crate) fn validate_database_after_open(
    paths: &mut PreparedStoragePaths,
) -> Result<(), RuntimeStoreError> {
    revalidate_trusted_directories(paths)?;
    let identity_after_open = existing_regular_file_identity(&paths.database_path)?
        .ok_or_else(RuntimeStoreError::path_invalid)?;
    if paths
        .identity_before_open
        .as_ref()
        .is_some_and(|before| before != &identity_after_open)
    {
        return Err(RuntimeStoreError::path_invalid());
    }
    paths.identity_before_open = Some(identity_after_open);
    enforce_private_file(&paths.database_path)
}

pub(crate) fn revalidate_database(paths: &PreparedStoragePaths) -> Result<(), RuntimeStoreError> {
    revalidate_trusted_directories(paths)?;
    let current_identity = existing_regular_file_identity(&paths.database_path)?
        .ok_or_else(RuntimeStoreError::path_invalid)?;
    if paths.identity_before_open.as_ref() != Some(&current_identity) {
        return Err(RuntimeStoreError::path_invalid());
    }
    enforce_private_file(&paths.database_path)
}

fn revalidate_trusted_directories(paths: &PreparedStoragePaths) -> Result<(), RuntimeStoreError> {
    if directory_identity(&paths.app_root)? != paths.app_root_identity
        || directory_identity(&paths.runtime_root)? != paths.runtime_root_identity
        || paths.runtime_root.parent() != Some(paths.app_root.as_path())
    {
        return Err(RuntimeStoreError::path_invalid());
    }
    Ok(())
}

pub(crate) fn enforce_sidecar_permissions(database_path: &Path) -> Result<(), RuntimeStoreError> {
    for suffix in ["-wal", "-shm"] {
        let sidecar = sidecar_path(database_path, suffix);
        if sidecar.exists() {
            existing_regular_file_identity(&sidecar)?
                .ok_or_else(RuntimeStoreError::path_invalid)?;
            enforce_private_file(&sidecar)?;
        }
    }
    Ok(())
}

pub(crate) fn database_total_size(database_path: &Path) -> Result<u64, RuntimeStoreError> {
    let mut total = 0_u64;
    for path in [
        database_path.to_path_buf(),
        sidecar_path(database_path, "-wal"),
        sidecar_path(database_path, "-shm"),
    ] {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(RuntimeStoreError::path_invalid());
                }
                total = total
                    .checked_add(metadata.len())
                    .ok_or_else(RuntimeStoreError::resource_limit)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(RuntimeStoreError::from_io(&error)),
        }
    }
    Ok(total)
}

fn sidecar_path(database_path: &Path, suffix: &str) -> PathBuf {
    let mut value = OsString::from(database_path.as_os_str());
    value.push(suffix);
    PathBuf::from(value)
}

fn validate_absolute_normal_path(path: &Path) -> Result<(), RuntimeStoreError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(RuntimeStoreError::path_invalid());
    }
    Ok(())
}

fn reject_symlink_components(path: &Path) -> Result<(), RuntimeStoreError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        let metadata =
            fs::symlink_metadata(&current).map_err(|error| RuntimeStoreError::from_io(&error))?;
        if metadata.file_type().is_symlink() {
            return Err(RuntimeStoreError::path_invalid());
        }
    }
    Ok(())
}

fn reject_existing_symlink(path: &Path) -> Result<(), RuntimeStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(RuntimeStoreError::path_invalid()),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(RuntimeStoreError::from_io(&error)),
    }
}

fn ensure_directory(path: &Path) -> Result<(), RuntimeStoreError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| RuntimeStoreError::from_io(&error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeStoreError::path_invalid());
    }
    Ok(())
}

fn directory_identity(path: &Path) -> Result<FileIdentity, RuntimeStoreError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| RuntimeStoreError::from_io(&error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeStoreError::path_invalid());
    }
    Ok(file_identity(&metadata))
}

fn existing_regular_file_identity(path: &Path) -> Result<Option<FileIdentity>, RuntimeStoreError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(RuntimeStoreError::from_io(&error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RuntimeStoreError::path_invalid());
    }
    Ok(Some(file_identity(&metadata)))
}

#[cfg(unix)]
fn file_identity(metadata: &fs::Metadata) -> FileIdentity {
    use std::os::unix::fs::MetadataExt;
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(not(unix))]
fn file_identity(metadata: &fs::Metadata) -> FileIdentity {
    FileIdentity {
        length: metadata.len(),
        created: metadata.created().ok(),
        modified: metadata.modified().ok(),
    }
}

#[cfg(unix)]
fn enforce_private_directory(path: &Path) -> Result<(), RuntimeStoreError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| RuntimeStoreError::from_io(&error))
}

#[cfg(not(unix))]
fn enforce_private_directory(_path: &Path) -> Result<(), RuntimeStoreError> {
    Ok(())
}

#[cfg(unix)]
fn enforce_private_file(path: &Path) -> Result<(), RuntimeStoreError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| RuntimeStoreError::from_io(&error))
}

#[cfg(not(unix))]
fn enforce_private_file(_path: &Path) -> Result<(), RuntimeStoreError> {
    Ok(())
}
