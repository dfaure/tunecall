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
        render::render_page_png(&pdfium, &args.pdf, page, args.dpi, tmp.path())
            .with_context(|| format!("rendering page {}", page + 1))?;
        let text = ocr::ocr_image(tmp.path(), &args.lang, &args.psm)
            .with_context(|| format!("OCR of page {}", page + 1))?;
        lines.extend(text.lines().map(|l| l.to_string()));
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
