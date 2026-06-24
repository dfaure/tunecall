//! Persisted app settings, stored as JSON in the data dir
//! (`storage::settings_path`). Same load/save shape as `setlist`: a missing or
//! corrupt file falls back to defaults so a bad file never blocks the app.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::storage;

/// User-adjustable app settings. Defaults are the "fresh install" state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    /// Write a debug log file to the data dir. Off by default: most users never
    /// need it, and it's only read at startup (the Android logger can't be
    /// reconfigured after start), so a change takes effect on the next launch.
    pub file_logging: bool,
}

/// Load the settings. A missing file means "defaults"; a corrupt or unreadable
/// file is logged and also treated as defaults.
pub fn load() -> Settings {
    load_from(&storage::settings_path())
}

/// Persist the settings, replacing the file. Written atomically (`.part` then
/// rename) so a crash mid-write can't truncate the existing file.
pub fn save(s: &Settings) -> Result<()> {
    save_to(&storage::settings_path(), s)
}

fn load_from(path: &Path) -> Settings {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Settings::default(),
        Err(e) => {
            log::warn!("reading {} failed: {e}", path.display());
            return Settings::default();
        }
    };
    match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("parsing {} failed: {e}", path.display());
            Settings::default()
        }
    }
}

fn save_to(path: &Path, s: &Settings) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(s)?;
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips() {
        let dir = std::env::temp_dir().join(format!("tunecall-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");

        let s = Settings { file_logging: true };
        save_to(&path, &s).unwrap();
        assert_eq!(load_from(&path), s);
        assert!(!path.with_extension("part").exists()); // atomic write left no temp

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_is_default() {
        let path = std::env::temp_dir().join(format!("tunecall-nope-{}.json", std::process::id()));
        std::fs::remove_file(&path).ok();
        assert_eq!(load_from(&path), Settings::default());
        assert!(!load_from(&path).file_logging);
    }

    #[test]
    fn corrupt_file_is_default() {
        let path = std::env::temp_dir().join(format!("tunecall-bad-{}.json", std::process::id()));
        std::fs::write(&path, b"{ not json").unwrap();
        assert_eq!(load_from(&path), Settings::default());
        std::fs::remove_file(&path).ok();
    }
}
