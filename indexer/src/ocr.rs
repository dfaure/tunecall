//! OCR by shelling out to the `tesseract` CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow, bail};

/// OCR `image` and return the recognized text. Install the OCR engine via
/// `zypper install tesseract-ocr` (note: the `tesseract` package is an
/// unrelated game, whose binary is `tesseract-game`).
pub fn ocr_image(image: &Path, lang: &str, psm: &str) -> Result<String> {
    let output = Command::new("tesseract")
        .arg(image)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .arg("--psm")
        .arg(psm)
        .output()
        .map_err(|e| {
            anyhow!("failed to run `tesseract` (install the tesseract-ocr package): {e}")
        })?;

    if !output.status.success() {
        bail!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
