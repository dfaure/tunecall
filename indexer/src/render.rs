//! Render PDF pages to PNG images for OCR, using pdfium.

use std::path::Path;

use anyhow::{Result, anyhow};
use pdfium_render::prelude::*;

/// Bind to a pdfium shared library: try next-to-cwd and a couple of parents
/// (the jambook repo keeps `libpdfium.so` at its root), then the system one.
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

/// Render 0-based `page` of `pdf` to `out` (a `.png` path) at `dpi`.
pub fn render_page_png(pdfium: &Pdfium, pdf: &Path, page: u16, dpi: i32, out: &Path) -> Result<()> {
    let doc = pdfium.load_pdf_from_file(pdf, None)?;
    let page = doc.pages().get(page)?;
    // pdfium renders at 72 DPI for scale 1.0.
    let config = PdfRenderConfig::new().scale_page_by_factor(dpi.max(1) as f32 / 72.0);
    let bitmap = page.render_with_config(&config)?;
    bitmap.as_image().save(out)?;
    Ok(())
}

/// Number of pages in `pdf`.
pub fn page_count(pdfium: &Pdfium, pdf: &Path) -> Result<u16> {
    Ok(pdfium.load_pdf_from_file(pdf, None)?.pages().len())
}
