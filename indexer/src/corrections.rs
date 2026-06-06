//! Optional per-book title corrections, applied after OCR/parse.
//!
//! Some titles are mangled by OCR beyond what the parser can repair —
//! initialisms like `T.J.R.C.` have no language-model support and come out as
//! garbage (`Ot > A`). A sidecar `<stem>.corrections` next to the PDF lets you
//! fix them by *printed* page number, which is stable across OCR changes (unlike
//! the garbled text itself, so re-tuning OCR never invalidates a correction).
//! The indexer overrides the matching entry's title, or adds an entry if OCR
//! dropped that page entirely.
//!
//! Format: one `<printed-page> <title>` per line (page, then whitespace, then
//! the title — which may contain spaces). Blank lines and lines starting with
//! `#` are ignored. The file lives next to the PDF/.db, not in git.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Printed page number -> corrected title.
pub type Corrections = BTreeMap<i32, String>;

/// User-facing warnings for corrections that don't cleanly override an OCR'd
/// entry. A correction whose printed page matches an OCR'd entry overrides it
/// quietly (the intended case); the surprising ones — worth flagging — are those
/// that matched no entry (added as a new row, often a typo'd page) or whose page
/// resolves outside the PDF.
///
/// `ocr_pages` is the set of OCR'd printed pages; `offset`/`n_pages` resolve a
/// printed page to a 0-based scan page (`printed + offset - 1`, must land in
/// `0..n_pages`), mirroring `resolve_page` in main.
pub fn warnings(
    corrections: &Corrections,
    ocr_pages: &BTreeSet<i32>,
    offset: i32,
    n_pages: i32,
) -> Vec<String> {
    let mut out = Vec::new();
    for (&printed, title) in corrections {
        if ocr_pages.contains(&printed) {
            continue; // overrides an OCR'd entry — the intended, quiet case
        }
        let scan = printed + offset - 1;
        if (0..n_pages).contains(&scan) {
            out.push(format!(
                "correction for p.{printed} ({title:?}) matched no OCR'd entry — \
                 added as a new row (typo in the page number?)"
            ));
        } else {
            out.push(format!(
                "correction for p.{printed} ({title:?}) maps to scan page {} — outside the \
                 PDF's 1..={n_pages}; clamped (check the page number or --offset)",
                scan + 1
            ));
        }
    }
    out
}

/// Load `<stem>.corrections` next to `pdf`. An absent file yields an empty map.
pub fn load(pdf: &Path) -> Result<Corrections> {
    let path = pdf.with_extension("corrections");
    match fs::read_to_string(&path) {
        Ok(text) => parse(&text).with_context(|| format!("in {}", path.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Corrections::new()),
        Err(e) => Err(e).context(format!("reading {}", path.display())),
    }
}

/// Parse the sidecar text into printed-page -> title.
fn parse(text: &str) -> Result<Corrections> {
    let mut map = Corrections::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.splitn(2, char::is_whitespace);
        let page_tok = it.next().unwrap_or("");
        let title = it.next().map(str::trim).unwrap_or("");
        let page: i32 = page_tok.parse().with_context(|| {
            format!(
                "line {}: expected '<page> <title>', bad page {page_tok:?}",
                i + 1
            )
        })?;
        if title.is_empty() {
            bail!("line {}: missing title for page {page}", i + 1);
        }
        map.insert(page, title.to_string());
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pages_and_skips_comments_and_blanks() {
        let m = parse("# a comment\n\n295 Cyclone\n296\tT.J.R.C.\n").unwrap();
        assert_eq!(m.get(&295).map(String::as_str), Some("Cyclone"));
        assert_eq!(m.get(&296).map(String::as_str), Some("T.J.R.C."));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn keeps_spaces_in_titles_and_trims() {
        let m = parse("  298 \t  Tea For Two  \n").unwrap();
        assert_eq!(m.get(&298).map(String::as_str), Some("Tea For Two"));
    }

    #[test]
    fn errors_on_bad_page_or_missing_title() {
        assert!(parse("notapage Title").is_err());
        assert!(parse("296").is_err()); // no title
    }

    #[test]
    fn warns_for_added_and_out_of_range_but_not_overrides() {
        let mut c = Corrections::new();
        c.insert(100, "Override Me".into()); // matches an OCR page -> quiet
        c.insert(200, "Added Tune".into()); // no OCR entry, in range -> "added"
        c.insert(9000, "Typo Page".into()); // no OCR entry, out of range
        let ocr: BTreeSet<i32> = [100, 150].into_iter().collect();
        let w = warnings(&c, &ocr, 0, 500); // offset 0 -> scan = printed - 1
        assert_eq!(w.len(), 2); // the override is not warned about
        assert!(
            w[0].contains("p.200") && w[0].contains("added as a new row"),
            "{:?}",
            w[0]
        );
        assert!(
            w[1].contains("p.9000") && w[1].contains("outside"),
            "{:?}",
            w[1]
        );
    }
}
