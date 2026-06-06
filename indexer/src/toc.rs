//! Parse OCR'd table-of-contents text into `(title, printed_page)` entries.
//!
//! TOC lines typically look like `Affirmation .......... 1` or
//! `All The Things You Are    4`. We take the trailing integer as the printed
//! page and everything before it (minus dot leaders) as the title. Long titles
//! wrap across lines (page on the last line); a line with no page is treated as
//! a title fragment and prepended to the next entry. This is deliberately
//! simple; OCR noise and odd layouts are expected to need iteration.

/// Parse TOC lines into `(title, printed_page)` pairs.
pub fn parse_toc(lines: &[String]) -> Vec<(String, i32)> {
    let mut out = Vec::new();
    // Long titles wrap across lines, with the page on the last line. A line with
    // no page is buffered here and prepended to the next entry's title.
    let mut prefix: Vec<String> = Vec::new();

    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            // OCR inserts blank lines between wrapped halves, so don't treat a
            // blank as a separator; the prefix is cleared by an entry or header.
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        let last = toks.last().copied().unwrap_or("");
        // Trailing token is the page number (tolerate surrounding non-digits).
        let page = last
            .trim_matches(|c: char| !c.is_ascii_digit())
            .parse::<i32>();

        match page {
            Err(_) => {
                // No page: a wrapped-title fragment to carry forward, unless it's
                // a header/garbage line (which breaks any pending wrap).
                let frag = clean_title(line);
                if frag.chars().filter(|c| c.is_alphabetic()).count() >= 2
                    && !looks_like_header(&frag)
                {
                    prefix.push(frag);
                } else {
                    prefix.clear();
                }
            }
            Ok(page) => {
                let mut title = clean_title(&toks[..toks.len() - 1].join(" "));
                if !prefix.is_empty() {
                    title = format!("{} {}", prefix.join(" "), title);
                    prefix.clear();
                }
                let title = title.trim().to_string();
                if title.chars().filter(|c| c.is_alphabetic()).count() >= 2 {
                    out.push((title, page));
                }
            }
        }
    }
    out
}

/// True for common TOC header lines that must not be merged into a title.
fn looks_like_header(s: &str) -> bool {
    let u = s.to_uppercase();
    u.contains("INDEX")
        || matches!(
            u.as_str(),
            "SONG TITLE" | "SONG TITLE PAGE" | "TITLE" | "PAGE" | "CONTENTS" | "TABLE OF CONTENTS"
        )
        || is_continuation_header(&u)
}

/// True for alphabetical section-continuation headers like `S-Cont.`, `(Cont'd)`,
/// or `A CONTINUED` — the header atop a column that continues a letter section.
/// These sit between songs and must break a wrap, not merge into a title.
///
/// `upper` is the already-uppercased line. We reduce it to ASCII alphanumerics
/// and accept `CONT`/`CONTD`/`CONTINUED`, optionally prefixed by a single
/// section letter (`S-CONT.` -> `SCONT`). The exact-match keeps real titles that
/// merely start with "cont" (`CONTINENTAL`, `CONTACT`) from being dropped.
fn is_continuation_header(upper: &str) -> bool {
    let norm: String = upper
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let is_cont = |s: &str| matches!(s, "CONT" | "CONTD" | "CONTINUED");
    is_cont(&norm) || (norm.len() > 4 && is_cont(&norm[1..]))
}

/// Strip dot-leader OCR garbage from a TOC title.
///
/// Dot leaders OCR as a run of dots (often glued to the title's last word) plus
/// stray lowercase letters / digits (`PARIG............::ccceee`, `020`, `oo`).
/// We cut at the first run of 2+ dots, then drop leading/trailing "junk" tokens
/// — ones with no letters, or all-lowercase (these TOCs are uppercase, so an
/// all-lowercase token is leader noise) — and trim separators at both ends,
/// including OCR quote/dash junk glued to the front of a title (`““Smoke`,
/// `~——Lennie`). A leading apostrophe is kept (`'Round Midnight`).
fn clean_title(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut cut = chars.len();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == '.' && chars[i + 1] == '.' {
            cut = i;
            break;
        }
    }
    let head: String = chars[..cut].iter().collect();

    let is_junk = |t: &str| {
        let mut letters = t.chars().filter(|c| c.is_alphabetic()).peekable();
        match letters.peek() {
            None => true,                                 // no letters at all
            Some(_) => letters.all(|c| c.is_lowercase()), // all-lowercase => leader noise
        }
    };

    let mut toks: Vec<&str> = head.split_whitespace().collect();
    while toks.first().is_some_and(|t| is_junk(t)) {
        toks.remove(0);
    }
    while toks.last().is_some_and(|t| is_junk(t)) {
        toks.pop();
    }
    toks.join(" ")
        .trim_start_matches(['"', '“', '”', '~', '–', '—', '·', '_', '-', '=', ' '])
        .trim_end_matches(['.', ':', ',', ';', '-', '·', '_', ' '])
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_dot_leaders_and_plain() {
        let e = parse_toc(&lines(&[
            "Affirmation .......... 1",
            "All The Things You Are    4",
            "Some Tune · 12",
            "",
            "Index",
            "42",
        ]));
        assert_eq!(
            e,
            vec![
                ("Affirmation".to_string(), 1),
                ("All The Things You Are".to_string(), 4),
                ("Some Tune".to_string(), 12),
            ]
        );
    }

    #[test]
    fn cleans_real_ocr_noise() {
        // Lines as tesseract emits them: title + dot-leader garbage + page.
        let e = parse_toc(&lines(&[
            "AFTERNOON IN PARIG............::ccceeceeeeeeeees 13",
            "ALWAYS 020... cece cece e eee eee eee ees 23",
            "ALL BLUES oo... cece eee ee nes 18",
            "MRS. AMERICA ............. cee 163",
            "THE BLUE ROOM. ...............:ccccceeeceee eee ee ees 53",
            "ISN'T IT ROMANTIGC?..................0 ccc cee e eee 221",
            "AGUA DE BEBER (WATER TO DRINK) 14",
            "9210 00 ee 16", // pure garbage row -> dropped
        ]));
        let titles: Vec<&str> = e.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "AFTERNOON IN PARIG",
                "ALWAYS",
                "ALL BLUES",
                "MRS. AMERICA",
                "THE BLUE ROOM",
                "ISN'T IT ROMANTIGC?",
                "AGUA DE BEBER (WATER TO DRINK)",
            ]
        );
    }

    #[test]
    fn drops_section_continuation_headers() {
        // Real OCR from realbk2h: a column-2 header "S-Cont." (S continued) sat
        // between a song whose page OCR destroyed and the next real entry. It
        // must break the wrap, not merge into "Smoke Gets In Your Eyes".
        let e = parse_toc(&lines(&[
            "Slow, Hot; Wind...... i i nc PF ; |", // page lost to OCR -> buffered
            "S-Cont.",                             // continuation header -> clears buffer
            "““Smoke Gets In Your Eyes..... wee cccce ecnecness 329",
        ]));
        assert_eq!(e, vec![("Smoke Gets In Your Eyes".to_string(), 329)]);
    }

    #[test]
    fn strips_leading_punctuation_junk() {
        let e = parse_toc(&lines(&[
            "~——Lennie'’s PENNIES ....... 208",  // leading ~ and em-dashes
            "““Smoke Gets In Your Eyes ... 329", // leading OCR quotes
            "'Round Midnight ........ 50",       // legit leading apostrophe kept
        ]));
        assert_eq!(
            e,
            vec![
                ("Lennie'’s PENNIES".to_string(), 208),
                ("Smoke Gets In Your Eyes".to_string(), 329),
                ("'Round Midnight".to_string(), 50),
            ]
        );
    }

    #[test]
    fn joins_wrapped_titles() {
        let e = parse_toc(&lines(&[
            "YOU ARE THE SUNSHINE",                         // no page -> buffered
            "",                                             // OCR blank between halves
            "OF MY LIFE ................0000.ccccceee 456", // page here
            "INDEX",                                        // header, must not merge
            "WAVE ......... 100",
        ]));
        assert_eq!(
            e,
            vec![
                ("YOU ARE THE SUNSHINE OF MY LIFE".to_string(), 456),
                ("WAVE".to_string(), 100),
            ]
        );
    }
}
