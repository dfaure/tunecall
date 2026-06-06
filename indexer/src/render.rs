//! Bind the pdfium shared library and read a PDF's page count (used to validate
//! `--offset` and clamp out-of-range entries).

use std::path::Path;

use anyhow::{Result, anyhow};
use pdfium_render::prelude::*;

/// Bind to a pdfium shared library: try next-to-cwd and a couple of parents
/// (the repo root keeps `libpdfium.so`), then the system one.
pub fn bind_pdfium() -> Result<Pdfium> {
    for dir in ["./", "../", "../../"] {
        if let Ok(bindings) =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir))
        {
            return Ok(Pdfium::new(bindings));
        }
    }
    let bindings = Pdfium::bind_to_system_library()
        .map_err(|e| anyhow!("could not load a pdfium library (cwd, .., ../.., or system): {e}"))?;
    Ok(Pdfium::new(bindings))
}

/// Number of pages in `pdf`.
pub fn page_count(pdfium: &Pdfium, pdf: &Path) -> Result<u16> {
    Ok(pdfium.load_pdf_from_file(pdf, None)?.pages().len())
}
