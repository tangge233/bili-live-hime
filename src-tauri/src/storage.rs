use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Manager};

const CONFIG_FILENAME: &str = "app-config.json";
const SECRETS_FILENAME: &str = "app-secrets.json";
const MAX_CONFIG_BYTES: usize = 2 * 1024 * 1024;

const KEYRING_SERVICE: &str = "com.rsplwe.bili-live-hime";
const KEYRING_USER: &str = "app-secrets";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("failed to resolve app config directory: {e}"))
}

/// Creates the application config directory and, on Unix, restricts it to the
/// current user. This is done in Rust so the WebView never needs directory
/// access through the fs plugin.
pub fn ensure_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = config_dir(app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create {}: {e}", dir.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("failed to secure {}: {e}", dir.display()))?;
    }

    Ok(dir)
}

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ensure_config_dir(app)?.join(CONFIG_FILENAME))
}

fn secrets_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ensure_config_dir(app)?.join(SECRETS_FILENAME))
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    #[cfg(unix)]
    if path.exists() {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }

    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("failed to read {}: {error}", path.display())),
    }
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to remove {}: {error}", path.display())),
    }
}

fn write_private_atomic(path: &Path, contents: &str) -> Result<(), String> {
    if contents.len() > MAX_CONFIG_BYTES {
        return Err(format!(
            "refusing to write {} bytes to {}",
            contents.len(),
            path.display()
        ));
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("failed to secure {}: {e}", parent.display()))?;
    }

    let suffix = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config"),
        std::process::id(),
        suffix
    ));

    let result = (|| -> Result<(), String> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let mut file: File = options
            .open(&tmp)
            .map_err(|e| format!("failed to create {}: {e}", tmp.display()))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
        file.sync_all()
            .map_err(|e| format!("failed to sync {}: {e}", tmp.display()))?;
        drop(file);

        #[cfg(windows)]
        {
            // std::fs::rename cannot replace an existing file on Windows.
            if path.exists() {
                fs::remove_file(path)
                    .map_err(|e| format!("failed to replace {}: {e}", path.display()))?;
            }
        }

        fs::rename(&tmp, path).map_err(|e| format!("failed to replace {}: {e}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("failed to secure {}: {e}", path.display()))?;
        }

        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }

    result
}

pub fn load_config(app: &AppHandle) -> Result<Option<String>, String> {
    read_optional(&config_path(app)?)
}

pub fn save_config(app: &AppHandle, contents: &str) -> Result<(), String> {
    write_private_atomic(&config_path(app)?, contents)
}

fn keyring_load() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("keyring unavailable: {e}"))?;

    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("keyring read failed: {error}")),
    }
}

fn keyring_save(contents: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("keyring unavailable: {e}"))?;
    entry
        .set_password(contents)
        .map_err(|e| format!("keyring write failed: {e}"))
}

fn keyring_delete() -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("keyring unavailable: {e}"))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("keyring delete failed: {error}")),
    }
}

/// Secrets are stored in the OS credential store when available. On Linux
/// systems without a Secret Service (headless, minimal desktops, some CI
/// images) we fall back to a 0600 file inside the already-0700 config dir.
pub fn load_secrets(app: &AppHandle) -> Result<Option<String>, String> {
    let path = secrets_path(app)?;

    match keyring_load() {
        Ok(Some(value)) => {
            // A previous fallback file may still contain credentials after
            // the keyring became available. Do not leave a plaintext copy
            // behind once the canonical store has been read successfully.
            if let Err(error) = remove_file_if_exists(&path) {
                eprintln!("failed to remove stale secrets fallback: {error}");
            }
            Ok(Some(value))
        }
        Ok(None) => match read_optional(&path)? {
            Some(value) => {
                if keyring_save(&value).is_ok() {
                    if let Err(error) = remove_file_if_exists(&path) {
                        eprintln!("failed to remove migrated secrets fallback: {error}");
                    }
                }
                Ok(Some(value))
            }
            None => Ok(None),
        },
        Err(error) => {
            eprintln!("OS keyring unavailable, using private fallback: {error}");
            read_optional(&path)
        }
    }
}

pub fn save_secrets(app: &AppHandle, contents: &str) -> Result<(), String> {
    let path = secrets_path(app)?;

    match keyring_save(contents) {
        Ok(()) => {
            if let Err(error) = fs::remove_file(&path) {
                if error.kind() != ErrorKind::NotFound {
                    eprintln!(
                        "failed to remove stale secrets fallback {}: {error}",
                        path.display()
                    );
                }
            }
            Ok(())
        }
        Err(error) => {
            eprintln!("OS keyring unavailable, using private fallback: {error}");
            write_private_atomic(&path, contents)
        }
    }
}

pub fn clear_secrets(app: &AppHandle) -> Result<(), String> {
    let path = secrets_path(app)?;
    let keyring_result = keyring_delete();
    let file_result = match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to remove {}: {error}", path.display())),
    };

    if let Err(error) = &keyring_result {
        eprintln!("failed to clear OS keyring entry: {error}");
    }
    file_result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let suffix = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "bili-live-hime-storage-test-{}-{suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn writes_private_file_and_secures_parent() {
        let dir = TempDir::new();
        let path = dir.0.join("nested").join(CONFIG_FILENAME);

        write_private_atomic(&path, "{\"ok\":true}").expect("write config");
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"ok\":true}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            let dir_mode = fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(file_mode, 0o600);
            assert_eq!(dir_mode, 0o700);
        }

        let entries: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![CONFIG_FILENAME]);
    }

    #[test]
    fn replaces_existing_file_atomically() {
        let dir = TempDir::new();
        let path = dir.0.join(CONFIG_FILENAME);

        write_private_atomic(&path, "old").expect("write old config");
        write_private_atomic(&path, "new").expect("write new config");

        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
    }

    #[test]
    fn refuses_oversized_contents() {
        let dir = TempDir::new();
        let path = dir.0.join(CONFIG_FILENAME);
        let oversized = "x".repeat(MAX_CONFIG_BYTES + 1);

        assert!(write_private_atomic(&path, &oversized).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let dir = TempDir::new();
        assert_eq!(read_optional(&dir.0.join(CONFIG_FILENAME)).unwrap(), None);
    }
}
