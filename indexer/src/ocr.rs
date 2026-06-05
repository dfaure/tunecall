//! OCR by shelling out to the `tesseract` CLI.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow, bail};

/// OCR `image` and return the recognized text, using the `cmd` binary (usually
/// "tesseract"). Install the OCR engine via `zypper install tesseract-ocr`
/// (note: the `tesseract` package is an unrelated game).
pub fn ocr_image(image: &Path, lang: &str, cmd: &str) -> Result<String> {
    let output = Command::new(cmd)
        .arg(image)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .arg("--psm")
        .arg("6") // assume a uniform block of text
        .output()
        .map_err(|e| {
            anyhow!("failed to run `{cmd}` (install tesseract-ocr, or pass --tesseract): {e}")
        })?;

    if !output.status.success() {
        bail!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
