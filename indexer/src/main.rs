//! tunecall-indexer: build a per-PDF song index for TuneCall by OCR'ing a
//! fake-book's table of contents.
//!
//! Pipeline: render the TOC page(s) -> OCR (tesseract) -> parse `title + printed
//! page` -> map printed page to the real scan page -> write `<stem>.db`.
//!
//! LIMITATION (the reason TuneCall moved off a global master index): mapping the
//! printed page to the actual scan page currently uses a single `--offset`,
//! which is wrong as soon as the scan has missing/extra pages. The robust fix is
//! to OCR the printed page number off each scanned page and build a real
//! printed->scan map; that lives behind `resolve_page` and is the next step.

mod db;
mod ocr;
mod render;
mod repair;
mod toc;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use pdfium_render::prelude::Pdfium;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Args {
    /// PDF file to index.
    #[arg(long)]
    pdf: PathBuf,

    /// Table-of-contents pages, as 1-based scan page numbers. E.g. "6-9" or "6-9,12".
    /// Omit and pass --detect-toc to find them automatically.
    #[arg(long)]
    toc: Option<String>,

    /// Auto-detect the TOC page range (OCR the leading pages and pick the run
    /// of index pages). Used when --toc is not given.
    #[arg(long)]
    detect_toc: bool,

    /// Page offset = (1-based scan page) minus (printed page). 0 if they match;
    /// e.g. if printed page 1 is on scan page 16, pass 15. May be negative.
    /// (A single offset can't handle missing pages — see the limitation above.)
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    offset: i32,

    /// Output index DB. Default: "<pdf-stem>.db" next to the PDF.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Render resolution in DPI for OCR (~400 reads best with tesseract here).
    #[arg(long, default_value_t = 400)]
    dpi: i32,

    /// tesseract language code.
    #[arg(long, default_value = "eng")]
    lang: String,

    /// tesseract page-segmentation mode. 3 = auto layout (handles columns),
    /// 6 = single uniform block, 4 = single column. See `tesseract --help-psm`.
    #[arg(long, default_value = "3")]
    psm: String,

    /// Parse and print entries without writing the DB.
    #[arg(long)]
    dry_run: bool,

    /// Dump the raw OCR lines (before parsing) to stderr. Useful for tuning the
    /// TOC parser against a specific book's layout.
    #[arg(long)]
    dump_ocr: bool,

    /// Disable page-number repair (keep raw OCR pages).
    #[arg(long)]
    no_repair: bool,

    /// Max deviation (in pages) from the trend before a page is treated as a
    /// gross OCR outlier and corrected. Smaller inversions are kept as OCR'd.
    #[arg(long, default_value_t = 20)]
    repair_tolerance: i32,
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

/// 0-based scan page for a printed page. `offset` = (1-based scan page) − (printed page).
/// May return a negative number (caller clamps).
fn resolve_page(printed: i32, offset: i32) -> i32 {
    printed + offset - 1
}

/// Render one 0-based page to a temp PNG, OCR it, and return its text lines.
fn ocr_page(
    pdfium: &Pdfium,
    pdf: &Path,
    page: u16,
    dpi: i32,
    lang: &str,
    psm: &str,
) -> Result<Vec<String>> {
    let tmp = tempfile::Builder::new().suffix(".png").tempfile()?;
    render::render_page_png(pdfium, pdf, page, dpi, tmp.path())
        .with_context(|| format!("rendering page {}", page + 1))?;
    let text = ocr::ocr_image(tmp.path(), lang, psm)?;
    Ok(text.lines().map(|l| l.to_string()).collect())
}

// TOC auto-detection knobs. A TOC page has many index entries; content and
// reference pages (e.g. chord charts) have few. Detection only looks at the
// front of the book, where fake-book indexes live.
const DETECT_SCAN_PAGES: u16 = 16;
const DETECT_DPI: i32 = 300; // counts are robust at 300; faster than full --dpi
const DETECT_MIN_ENTRIES: usize = 20;

/// Auto-detect the TOC range: OCR the leading pages, count parsed entries per
/// page, and return the longest contiguous run scoring at least the threshold
/// (as 0-based page indices).
fn detect_toc(
    pdfium: &Pdfium,
    pdf: &Path,
    n_pages: u16,
    lang: &str,
    psm: &str,
) -> Result<Vec<u16>> {
    let scan = DETECT_SCAN_PAGES.min(n_pages);
    let mut counts = Vec::with_capacity(scan as usize);
    for page in 0..scan {
        let lines = ocr_page(pdfium, pdf, page, DETECT_DPI, lang, psm)?;
        counts.push(toc::parse_toc(&lines).len());
    }
    println!("TOC detection — entries per leading page: {counts:?}");

    let (mut best_start, mut best_len) = (0usize, 0usize);
    let (mut cur_start, mut cur_len) = (0usize, 0usize);
    for (i, &c) in counts.iter().enumerate() {
        if c >= DETECT_MIN_ENTRIES {
            if cur_len == 0 {
                cur_start = i;
            }
            cur_len += 1;
            if cur_len > best_len {
                best_len = cur_len;
                best_start = cur_start;
            }
        } else {
            cur_len = 0;
        }
    }
    if best_len == 0 {
        bail!(
            "could not auto-detect a TOC in the first {scan} pages (no run of >= \
             {DETECT_MIN_ENTRIES} entries); pass --toc explicitly"
        );
    }
    println!(
        "detected TOC: pages {}-{}",
        best_start + 1,
        best_start + best_len
    );
    Ok((best_start as u16..(best_start + best_len) as u16).collect())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.pdf.is_file() {
        bail!("no such PDF: {}", args.pdf.display());
    }
    let pdfium = render::bind_pdfium()?;
    let n_pages = render::page_count(&pdfium, &args.pdf)?;

    let toc_pages = match &args.toc {
        Some(spec) => parse_pages(spec)?,
        None if args.detect_toc => detect_toc(&pdfium, &args.pdf, n_pages, &args.lang, &args.psm)?,
        None => bail!("provide --toc <range> or --detect-toc"),
    };
    let toc_1based: Vec<u16> = toc_pages.iter().map(|p| p + 1).collect();
    println!(
        "{} has {n_pages} pages; OCR'ing TOC pages {toc_1based:?}",
        args.pdf.display()
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
        lines.extend(
            ocr_page(&pdfium, &args.pdf, page, args.dpi, &args.lang, &args.psm)
                .with_context(|| format!("OCR of page {}", page + 1))?,
        );
    }

    if args.dump_ocr {
        eprintln!("---- raw OCR lines ----");
        for (i, l) in lines.iter().enumerate() {
            eprintln!("{i:>4}: {l:?}");
        }
        eprintln!("---- end raw OCR lines ----");
    }

    // Parse, repair OCR'd page numbers, then resolve to scan pages.
    let parsed = toc::parse_toc(&lines);
    let raw_pages: Vec<i32> = parsed.iter().map(|(_, p)| *p).collect();
    let printed_pages = if args.no_repair {
        raw_pages.clone()
    } else {
        // Valid *printed* page range given the offset: printed P maps to scan
        // page P + offset - 1 (0-based), which must land in 0..n_pages.
        let lo = (1 - args.offset).max(1);
        let hi = n_pages as i32 - args.offset;
        let (fixed, n_fixed) = repair::repair_pages(&raw_pages, lo, hi, args.repair_tolerance);
        if n_fixed > 0 {
            println!("corrected {n_fixed} gross page-number outlier(s)");
        }
        fixed
    };

    let mut out_of_range = 0;
    let entries: Vec<(String, i32)> = parsed
        .iter()
        .zip(&printed_pages)
        .map(|((title, _), &printed)| {
            let scan = resolve_page(printed, args.offset);
            let clamped = scan.clamp(0, n_pages.saturating_sub(1) as i32);
            if scan != clamped {
                out_of_range += 1;
            }
            (title.clone(), clamped)
        })
        .collect();

    println!("\nparsed {} entries:", entries.len());
    for (i, (title, raw)) in parsed.iter().enumerate() {
        let printed = printed_pages[i];
        let mark = if *raw != printed {
            format!(" (ocr:{raw})")
        } else {
            String::new()
        };
        println!(
            "  p.{printed:<4}{mark} -> scan {:<4} {title}",
            entries[i].1 + 1
        );
    }
    if out_of_range > 0 {
        eprintln!(
            "\nwarning: {out_of_range} entries fell outside 1..={n_pages} and were clamped; \
             check --offset."
        );
    }
    if args.offset == 0 {
        println!(
            "\nNOTE: --offset is 0 (printed page == scan page). If that's wrong, measure the\n\
             scan page showing printed page 1 and pass offset = scan_page - 1."
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
