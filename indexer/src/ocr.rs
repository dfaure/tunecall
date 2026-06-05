//! OCR by shelling out to the `tesseract` CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow, bail};

/// OCR `image` and return the recognized text. Requires `tesseract` on PATH
/// (e.g. `zypper install tesseract tesseract-data-eng`).
pub fn ocr_image(image: &Path, lang: &str) -> Result<String> {
    let output = Command::new("tesseract")
        .arg(image)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .arg("--psm")
        .arg("6") // assume a uniform block of text
        .output()
        .map_err(|e| anyhow!("failed to run `tesseract` (is it installed and on PATH?): {e}"))?;

    if !output.status.success() {
        bail!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
