//! jambook-indexer: build a per-PDF song index for JamBook by OCR'ing a
//! fake-book's table of contents.
//!
//! Pipeline: render the TOC page(s) -> OCR (tesseract) -> parse `title + printed
//! page` -> map printed page to the real scan page -> write `<stem>.db`.
//!
//! LIMITATION (the reason JamBook moved off a global master index): mapping the
//! printed page to the actual scan page currently uses a single `--offset`,
//! which is wrong as soon as the scan has missing/extra pages. The robust fix is
//! to OCR the printed page number off each scanned page and build a real
//! printed->scan map; that lives behind `resolve_page` and is the next step.

mod db;
mod ocr;
mod render;
mod toc;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Args {
    /// PDF file to index.
    #[arg(long)]
    pdf: PathBuf,

    /// Table-of-contents pages, as 1-based scan page numbers. E.g. "6-9" or "6-9,12".
    #[arg(long)]
    toc: String,

    /// Scan page (1-based) that shows the book's PRINTED page "1".
    /// Used to map printed pages to scan pages (see the missing-pages limitation).
    #[arg(long, default_value_t = 1)]
    offset: i32,

    /// Output index DB. Default: "<pdf-stem>.db" next to the PDF.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Render width in pixels for OCR (higher = sharper, slower).
    #[arg(long, default_value_t = 2200)]
    width: i32,

    /// tesseract language code.
    #[arg(long, default_value = "eng")]
    lang: String,

    /// Parse and print entries without writing the DB.
    #[arg(long)]
    dry_run: bool,
}

/// Parse "6-9,12" (1-based) into 0-based page indices.
fn parse_pages(spec: &str) -> Result<Vec<u16>> {
    let mut pages = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            let start: u16 = a
                .trim()
                .parse()
                .with_context(|| format!("bad page '{a}'"))?;
            let end: u16 = b
                .trim()
                .parse()
                .with_context(|| format!("bad page '{b}'"))?;
            if start == 0 || end < start {
                bail!("invalid page range '{part}'");
            }
            pages.extend(start..=end);
        } else {
            let p: u16 = part.parse().with_context(|| format!("bad page '{part}'"))?;
            if p == 0 {
                bail!("pages are 1-based; '0' is invalid");
            }
            pages.push(p);
        }
    }
    if pages.is_empty() {
        bail!("no TOC pages given");
    }
    Ok(pages.iter().map(|p| p - 1).collect()) // to 0-based
}

/// Map a printed page to a 0-based scan page. See the missing-pages limitation.
fn resolve_page(printed: i32, offset: i32) -> i32 {
    let first0 = (offset - 1).max(0);
    first0 + (printed - 1).max(0)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.pdf.is_file() {
        bail!("no such PDF: {}", args.pdf.display());
    }
    let toc_pages = parse_pages(&args.toc)?;

    let pdfium = render::bind_pdfium()?;
    let n_pages = render::page_count(&pdfium, &args.pdf)?;
    println!(
        "{} has {n_pages} pages; OCR'ing TOC pages {:?}",
        args.pdf.display(),
        &args.toc
    );

    // OCR each TOC page.
    let mut lines: Vec<String> = Vec::new();
    for &page in &toc_pages {
        if page >= n_pages {
            eprintln!(
                "warning: TOC page {} is past the end ({n_pages}); skipping",
                page + 1
            );
            continue;
        }
        let tmp = tempfile::Builder::new().suffix(".png").tempfile()?;
        render::render_page_png(&pdfium, &args.pdf, page, args.width, tmp.path())
            .with_context(|| format!("rendering page {}", page + 1))?;
        let text = ocr::ocr_image(tmp.path(), &args.lang)
            .with_context(|| format!("OCR of page {}", page + 1))?;
        lines.extend(text.lines().map(|l| l.to_string()));
    }

    // Parse and resolve to scan pages.
    let parsed = toc::parse_toc(&lines);
    let entries: Vec<(String, i32)> = parsed
        .iter()
        .map(|(title, printed)| (title.clone(), resolve_page(*printed, args.offset)))
        .collect();

    println!("\nparsed {} entries:", entries.len());
    for ((title, printed), (_, page0)) in parsed.iter().zip(&entries) {
        println!("  p.{printed:<4} -> scan page {:<4} {title}", page0 + 1);
    }
    if args.offset == 1 {
        println!(
            "\nNOTE: --offset defaulted to 1. Measure the scan page showing printed page 1\n\
             and pass it via --offset. (And see the missing-pages limitation in --help.)"
        );
    }

    if args.dry_run {
        println!("\n(dry run; no DB written)");
        return Ok(());
    }

    let out = args.out.unwrap_or_else(|| args.pdf.with_extension("db"));
    db::write_index(&out, &entries)?;
    println!("\nwrote {} entries to {}", entries.len(), out.display());
    Ok(())
}
