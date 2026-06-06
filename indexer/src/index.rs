//! Load a per-book `<stem>.index` sidecar: the complete printed-page -> titles
//! index, transcribed by reading the rendered TOC pages (the scans are too
//! degraded for reliable OCR). One `<printed-page> <title>` per line; blank
//! lines and `#` comments ignored. Lives next to the PDF, not in git.
//!
//! A printed page may hold more than one tune (some books print two short
//! charts on a single page), so several lines can share the same page number;
//! all are kept.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Printed page number -> titles printed on it (a page may hold more than one
/// tune), kept in the order they appear in the sidecar.
pub type Index = BTreeMap<i32, Vec<String>>;

/// Load `<stem>.index` next to `pdf`. An absent file yields an empty map.
pub fn load(pdf: &Path) -> Result<Index> {
    let path = pdf.with_extension("index");
    match fs::read_to_string(&path) {
        Ok(text) => parse(&text).with_context(|| format!("in {}", path.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Index::new()),
        Err(e) => Err(e).context(format!("reading {}", path.display())),
    }
}

/// Parse sidecar text into printed-page -> title. Page, then whitespace (tab or
/// spaces), then the title (which may contain spaces).
fn parse(text: &str) -> Result<Index> {
    let mut map = Index::new();
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
        map.entry(page).or_default().push(title.to_string());
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pages_and_skips_comments_and_blanks() {
        let m = parse("# a comment\n\n295 Sy Clone\n296\tT.J.R.C.\n").unwrap();
        assert_eq!(
            m.get(&295).map(Vec::as_slice),
            Some(&["Sy Clone".to_string()][..])
        );
        assert_eq!(
            m.get(&296).map(Vec::as_slice),
            Some(&["T.J.R.C.".to_string()][..])
        );
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn keeps_spaces_in_titles_and_trims() {
        let m = parse("  298 \t  Tea For Two  \n").unwrap();
        assert_eq!(
            m.get(&298).map(Vec::as_slice),
            Some(&["Tea For Two".to_string()][..])
        );
    }

    #[test]
    fn keeps_multiple_titles_on_one_page() {
        // Some books print two short charts on a single page.
        let m = parse("38 Batterie\n38 Ictus\n").unwrap();
        assert_eq!(
            m.get(&38).map(Vec::as_slice),
            Some(&["Batterie".to_string(), "Ictus".to_string()][..])
        );
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn errors_on_bad_page_or_missing_title() {
        assert!(parse("notapage Title").is_err());
        assert!(parse("296").is_err()); // no title
    }
}
