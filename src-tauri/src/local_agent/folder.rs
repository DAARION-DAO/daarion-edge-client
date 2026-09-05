use cap_std::fs::Dir;
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};

pub struct FolderGrant {
    dir: Dir,
    pub label: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FolderEntry {
    pub name: String,
    pub kind: String,
    pub bytes: u64,
    pub excerpt: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FolderReport {
    pub folder: String,
    pub entries: Vec<FolderEntry>,
    pub skipped: usize,
    pub truncated: bool,
}

impl FolderGrant {
    pub fn open(path: &Path) -> Result<Self, String> {
        let dir = Dir::open_ambient_dir(path, cap_std::ambient_authority())
            .map_err(|_| "Не вдалося отримати доступ до вибраної папки.".to_string())?;
        let label = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .chars()
            .take(160)
            .collect();
        Ok(Self { dir, label })
    }

    pub fn inspect(&self) -> Result<FolderReport, String> {
        let mut report = FolderReport {
            folder: self.label.clone(),
            entries: vec![],
            skipped: 0,
            truncated: false,
        };
        let mut excerpts = 0;
        let items = self
            .dir
            .entries()
            .map_err(|_| "Папка більше недоступна.".to_string())?;
        for (index, item) in items.enumerate() {
            if index >= 256 {
                report.truncated = true;
                break;
            }
            let Ok(item) = item else {
                report.skipped += 1;
                continue;
            };
            let name = item.file_name();
            let display = name.to_string_lossy();
            if display.starts_with('.') {
                report.skipped += 1;
                continue;
            }
            let Ok(kind) = item.file_type() else {
                report.skipped += 1;
                continue;
            };
            if kind.is_symlink() || !(kind.is_file() || kind.is_dir()) {
                report.skipped += 1;
                continue;
            }
            let Ok(metadata) = item.metadata() else {
                report.skipped += 1;
                continue;
            };
            let text = Path::new(&name)
                .extension()
                .is_some_and(|ext| ext == "txt" || ext == "md");
            let excerpt = if kind.is_file() && text && excerpts < 8 && metadata.len() <= 64 * 1024 {
                // All opens are relative to the held capability. No ambient path is reconstructed.
                match self.dir.open(&name) {
                    Ok(file) if file.metadata().is_ok_and(|m| m.is_file()) => {
                        let mut bytes = Vec::new();
                        if file.take(1024).read_to_end(&mut bytes).is_ok() {
                            excerpts += 1;
                            Some(String::from_utf8_lossy(&bytes).into_owned())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            report.entries.push(FolderEntry {
                name: display.chars().take(256).collect(),
                kind: if kind.is_dir() { "folder" } else { "file" }.into(),
                bytes: if kind.is_file() { metadata.len() } else { 0 },
                excerpt,
            });
        }
        report.entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(report)
    }
}

pub fn choose_native() -> Result<Option<FolderGrant>, String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = std::process::Command::new("/usr/bin/osascript");
        c.args(["-e", "try\nset chosen to choose folder with prompt \"Оберіть папку для локального агента (лише читання)\"\nreturn POSIX path of chosen\non error number -128\nreturn \"\"\nend try"]);
        c
    };
    #[cfg(target_os = "windows")]
    let mut command = {
        let root = std::env::var_os("SystemRoot").ok_or("Windows environment unavailable")?;
        let mut c = std::process::Command::new(
            std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe"),
        );
        c.args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; $dialog.Description = 'Choose a folder for read-only local access'; $dialog.ShowNewFolderButton = $false; if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { [Console]::Write($dialog.SelectedPath) }; $dialog.Dispose()"]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return Err("Вибір папки в цьому пілоті доступний на macOS і Windows.".into());
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let output = command
            .output()
            .map_err(|_| "Не вдалося відкрити вікно вибору папки.".to_string())?;
        if !output.status.success() || output.stdout.len() > 32768 {
            return Err("Вибір папки не завершено.".into());
        }
        let path = String::from_utf8(output.stdout)
            .map_err(|_| "Назва папки має непідтримуване кодування.".to_string())?;
        let path = path.trim_end_matches(['\r', '\n']);
        if path.is_empty() {
            return Ok(None);
        }
        FolderGrant::open(Path::new(path)).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_read_excludes_hidden_symlinks_and_subfolders() {
        let root = std::env::temp_dir().join(format!("edge-folder-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("visible.txt"), "allowed").unwrap();
        std::fs::write(root.join(".secret"), "hidden").unwrap();
        std::fs::write(root.join("nested/child.txt"), "nested secret").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join(".secret"), root.join("escape.txt")).unwrap();
        let report = FolderGrant::open(&root).unwrap().inspect().unwrap();
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.entries[1].excerpt.as_deref(), Some("allowed"));
        assert!(!serde_json::to_string(&report).unwrap().contains("secret"));
        let grant = FolderGrant::open(&root).unwrap();
        let outside = root.with_extension("outside.txt");
        std::fs::write(&outside, "outside secret").unwrap();
        let relative = Path::new("..").join(outside.file_name().unwrap());
        assert!(grant.dir.open(&relative).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, root.join("outside-link.txt")).unwrap();
            assert!(grant.dir.open("outside-link.txt").is_err());
            assert!(!serde_json::to_string(&grant.inspect().unwrap())
                .unwrap()
                .contains("outside secret"));
        }
        std::fs::remove_file(outside).unwrap();
        for n in 0..300 {
            std::fs::write(root.join(format!("{n}.txt")), "x".repeat(2048)).unwrap();
        }
        let report = grant.inspect().unwrap();
        assert!(report.truncated);
        assert!(report.entries.len() <= 256);
        assert!(
            report
                .entries
                .iter()
                .filter(|f| f.excerpt.is_some())
                .count()
                <= 8
        );
        assert!(report
            .entries
            .iter()
            .filter_map(|f| f.excerpt.as_ref())
            .all(|s| s.len() <= 1024));
        drop(grant);
        std::fs::remove_dir_all(root).unwrap();
    }
}
