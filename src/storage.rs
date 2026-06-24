//! App-specific filesystem locations.
//!
//! On desktop the base directory comes from `dirs::data_dir()`; on Android it
//! is set from `android_main` via [`set_data_dir`] using the app-specific path
//! handed to us by the Android runtime.

use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Override the base data directory. Called once from `android_main`. A no-op
/// if the directory was already resolved.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn set_data_dir(dir: PathBuf) {
    let _ = DATA_DIR.set(dir);
}

/// Base directory for all app data (DB + PDFs).
pub fn data_dir() -> PathBuf {
    DATA_DIR.get_or_init(default_data_dir).clone()
}

#[cfg(not(target_os = "android"))]
fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tunecall")
}

#[cfg(target_os = "android")]
fn default_data_dir() -> PathBuf {
    // Unreachable in practice: android_main resolves the app-specific path and
    // calls set_data_dir() before anything reads it. There is no usable
    // hardcoded fallback on modern Android — scoped storage blocks arbitrary
    // shared paths like /storage/emulated/0/Download.
    panic!("data dir not initialized; android_main must call set_data_dir() first");
}

/// Directory holding the `.pdf` books and any locally authored `.db` indexes.
pub fn pdf_dir() -> PathBuf {
    data_dir().join("pdfs")
}

/// Directory where Reload writes the indexes it downloads from the server.
/// Kept separate from [`pdf_dir`] so a download never clobbers locally authored
/// indexes — the viewer reads `pdf_dir` first, then this.
pub fn download_dir() -> PathBuf {
    data_dir().join("downloaded")
}

/// File holding the user's setlists (JSON). The app's only writable user data;
/// lives in the data-dir root, beside the `pdfs/` and `downloaded/` folders.
pub fn setlists_path() -> PathBuf {
    data_dir().join("setlists.json")
}

/// File holding the user's app settings (JSON). Lives in the data-dir root,
/// beside `setlists.json`.
pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}
