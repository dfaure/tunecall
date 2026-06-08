//! tunecall-indexer: build a per-PDF song index (`<stem>.db`) for TuneCall from a
//! `<stem>.index` sidecar — a transcription of the book's table of contents
//! (`<printed-page> <title>` per line).
//!
//! The scanned fake-books are too degraded for reliable OCR, so the index is
//! transcribed by reading the rendered TOC pages directly (see README). This tool
//! maps each printed page to a 0-based scan page via `--offset` and writes the
//! `songs(title, page)` DB the viewer reads.
//!
//! LIMITATION: a single `--offset` can't model a scan with missing/extra pages;
//! entries that resolve outside the PDF are clamped to its last page (with a
//! warning). Per-entry pages can be fixed by editing the `.index` directly.

mod db;
mod index;
mod render;

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Args {
    /// PDF to index. Its sibling `<stem>.index` supplies the entries.
    #[arg(long)]
    pdf: PathBuf,

    /// Page offset = (1-based scan page) minus (printed page). 0 if they match;
    /// e.g. if printed page 1 is on scan page 16, pass 15. May be negative.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    offset: i32,

    /// Human-readable book title, recorded in the DB's `meta` table for the
    /// viewer to display (e.g. "The Real Book, Vol. 1"). Read off the cover.
    #[arg(long)]
    title: String,

    /// Output DB. Default: "<pdf-stem>.db" next to the PDF.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Resolve and print entries without writing the DB.
    #[arg(long)]
    dry_run: bool,
}

/// 0-based scan page for a printed page. `offset` = (1-based scan page) − (printed page).
fn resolve_page(printed: i32, offset: i32) -> i32 {
    printed + offset - 1
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.pdf.is_file() {
        bail!("no such PDF: {}", args.pdf.display());
    }
    let index_path = args.pdf.with_extension("index");
    if !index_path.is_file() {
        bail!(
            "no index sidecar {} — transcribe the book's TOC into it (see README)",
            index_path.display()
        );
    }

    let pdfium = render::bind_pdfium()?;
    let n_pages = render::page_count(&pdfium, &args.pdf)?;
    let index = index::load(&args.pdf)?;
    let total: usize = index.values().map(Vec::len).sum();
    println!(
        "{} has {n_pages} pages; using {} ({total} entries)",
        args.pdf.display(),
        index_path.display(),
    );

    // Resolve each printed page to a 0-based scan page (clamped to the PDF) and
    // drop exact-duplicate (title, page) rows. A printed page may carry more than
    // one title (two charts on a page); each resolves to the same scan page.
    let mut out_of_range = 0;
    let mut seen = HashSet::new();
    let mut rows: Vec<(i32, String, i32)> = Vec::with_capacity(total); // (printed, title, scan)
    for (&printed, titles) in &index {
        let scan = resolve_page(printed, args.offset);
        let clamped = scan.clamp(0, n_pages.saturating_sub(1) as i32);
        if scan != clamped {
            out_of_range += titles.len() as i32;
        }
        for title in titles {
            if seen.insert((title.clone(), clamped)) {
                rows.push((printed, title.clone(), clamped));
            }
        }
    }
    if rows.len() < total {
        println!("dropped {} exact-duplicate row(s)", total - rows.len());
    }

    println!("\n{} entries:", rows.len());
    for (printed, title, scan) in &rows {
        println!("  p.{printed:<4} -> scan {:<4} {title}", scan + 1);
    }
    if out_of_range > 0 {
        eprintln!(
            "\nwarning: {out_of_range} entries fell outside 1..={n_pages} and were clamped; \
             check --offset (or fix those pages in the .index)."
        );
    }

    if args.dry_run {
        println!("\n(dry run; no DB written)");
        return Ok(());
    }

    let entries: Vec<(String, i32)> = rows.iter().map(|(_, t, s)| (t.clone(), *s)).collect();
    let out = args.out.unwrap_or_else(|| args.pdf.with_extension("db"));
    db::write_index(&out, &entries, &args.title)?;
    println!("\nwrote {} entries to {}", entries.len(), out.display());
    Ok(())
}
