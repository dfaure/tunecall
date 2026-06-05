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
        .join("jambook")
}

#[cfg(target_os = "android")]
fn default_data_dir() -> PathBuf {
    // Fallback only; android_main normally calls set_data_dir() first.
    PathBuf::from("/storage/emulated/0/Download/jambook")
}

/// Directory holding the `.pdf` books and their sibling `.db` indexes.
pub fn pdf_dir() -> PathBuf {
    data_dir().join("pdfs")
}
