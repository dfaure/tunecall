//! PDF page rendering via pdfium (the library backing Chrome's PDF viewer).
//!
//! The pdfium shared library is bound dynamically at runtime; see README.md for
//! how to obtain it on each platform.

use std::cell::RefCell;

use anyhow::{Result, anyhow};
use pdfium_render::prelude::*;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

thread_local! {
    // Bind to pdfium lazily and keep it for the lifetime of the UI thread: binding
    // re-loads the shared library, which we don't want to repeat on every page turn.
    static PDFIUM: RefCell<Option<Pdfium>> = const { RefCell::new(None) };
}

fn with_pdfium<R>(f: impl FnOnce(&Pdfium) -> Result<R>) -> Result<R> {
    PDFIUM.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            // Prefer a pdfium library shipped next to the executable, then fall back
            // to a system-wide one. On Android it is packaged into the APK's jniLibs.
            let bindings =
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                    .or_else(|_| Pdfium::bind_to_system_library())
                    .map_err(|e| anyhow!("failed to load the pdfium library: {e}"))?;
            *slot = Some(Pdfium::new(bindings));
        }
        f(slot.as_ref().expect("pdfium just initialized"))
    })
}

/// Number of pages in the document at `path`.
pub fn page_count(path: &str) -> Result<u16> {
    with_pdfium(|pdfium| {
        let doc = pdfium.load_pdf_from_file(path, None)?;
        Ok(doc.pages().len())
    })
}

/// Render the 0-based `index` page of `path`, scaled to `target_width` pixels
/// wide. With `invert`, the RGB channels are flipped pixel-wise so a black-on-
/// white sheet renders white-on-black (a "dark mode" for the viewer).
pub fn render_page(path: &str, index: u16, target_width: i32, invert: bool) -> Result<Image> {
    with_pdfium(|pdfium| {
        let doc = pdfium.load_pdf_from_file(path, None)?;
        let page = doc.pages().get(index)?;
        let config = PdfRenderConfig::new().set_target_width(target_width.max(1));
        let bitmap = page.render_with_config(&config)?;

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let mut rgba = bitmap.as_rgba_bytes();
        if invert {
            for px in rgba.chunks_exact_mut(4) {
                px[0] = 255 - px[0];
                px[1] = 255 - px[1];
                px[2] = 255 - px[2];
            }
        }
        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&rgba, width, height);
        Ok(Image::from_rgba8(buffer))
    })
}
