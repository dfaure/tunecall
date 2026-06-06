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
        // Trailing token is the page number (parse_page tolerates surrounding
        // noise and recovers OCR letter-for-digit misreads like "3G3").
        match parse_page(last) {
            None => {
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
            Some(page) => {
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

/// Read a TOC line's trailing token as a printed page number.
///
/// Strips surrounding non-alphanumeric noise, then, if every remaining char is a
/// digit or a common OCR letter-for-digit lookalike (`O`->0, `I`/`l`->1, `Z`->2,
/// `S`->5, `G`->6, `B`->8) — and at least one is a real digit, so words like
/// `ZOO` don't become 200 — reads the whole token as a (possibly misread)
/// number: `3G3` -> 363, `S0` -> 50. Recovery runs *before* the plain reading so
/// a leading misread isn't mis-grabbed (`S0` -> 50, not 0). Otherwise it falls
/// back to the embedded digits (`p.14` -> 14). `None` if it isn't a page.
fn parse_page(tok: &str) -> Option<i32> {
    let core = tok.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    if core.is_empty() {
        return None;
    }
    // Whole-token recovery, anchored on at least one real digit.
    if core.chars().any(|c| c.is_ascii_digit())
        && let Some(n) = recover_page_digits(core)
    {
        return Some(n);
    }
    // Fall back to the digits embedded in the token, ignoring non-digit noise
    // (e.g. a "p.14" label prefix), once recovery has declined it.
    core.trim_matches(|c: char| !c.is_ascii_digit())
        .parse::<i32>()
        .ok()
}

/// Map a token of digits + OCR digit-lookalike letters to a number, or `None` if
/// any character is neither a digit nor a known lookalike.
fn recover_page_digits(core: &str) -> Option<i32> {
    let mut digits = String::with_capacity(core.len());
    for c in core.chars() {
        let d = match c {
            '0'..='9' => c,
            'O' | 'o' => '0',
            'I' | 'l' => '1',
            'Z' | 'z' => '2',
            'S' | 's' => '5',
            'G' => '6',
            'B' => '8',
            _ => return None,
        };
        digits.push(d);
    }
    digits.parse::<i32>().ok()
}

/// Strip dot-leader OCR garbage from a TOC title.
///
/// Dot leaders OCR as a run of dots (often glued to the title's last word) plus
/// stray lowercase letters / digits (`PARIG............::ccceee`, `020`, `oo`).
/// Sometimes the whole leader is misread as a run of `c`s (`Wall Street. cccccc
/// ...`). We cut at the first run of 2+ dots or 3+ `c`s (3, since `cc` occurs in
/// real words like `soccer`), then drop leading/trailing "junk" tokens — ones
/// with no letters, or all-lowercase (these TOCs are uppercase, so an
/// all-lowercase token is leader noise) — and trim separators at both ends,
/// including OCR quote/dash junk glued to the front of a title (`““Smoke`,
/// `~——Lennie`). A leading apostrophe is kept (`'Round Midnight`).
fn clean_title(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut cut = chars.len();
    for i in 0..chars.len() {
        let two_dots = chars[i] == '.' && chars.get(i + 1) == Some(&'.');
        let three_cs =
            chars[i] == 'c' && chars.get(i + 1) == Some(&'c') && chars.get(i + 2) == Some(&'c');
        if two_dots || three_cs {
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
    fn recovers_misread_page_and_c_run_leader() {
        // Real OCR from realbk2h: the dot leader read as a run of c's, and the
        // page 363 read as "3G3". Both lines must parse as separate entries
        // rather than merging "Wall Street" onto the next title.
        let e = parse_toc(&lines(&[
            "Wall Street. cccccccccssccccececesesee 3G3",
            "Watch What HappenS...ecccccsescccesese 304",
        ]));
        assert_eq!(
            e,
            vec![
                ("Wall Street".to_string(), 363),
                ("Watch What HappenS".to_string(), 304),
            ]
        );
    }

    #[test]
    fn parse_page_recovers_or_rejects() {
        assert_eq!(parse_page("304"), Some(304));
        assert_eq!(parse_page("3G3"), Some(363)); // G->6, letters between digits
        assert_eq!(parse_page("1O3"), Some(103)); // O->0
        assert_eq!(parse_page("S0"), Some(50)); // leading S->5 (not grabbed as 0)
        assert_eq!(parse_page("5O"), Some(50)); // trailing O->0
        assert_eq!(parse_page("p.14"), Some(14)); // unmappable prefix -> embedded digits
        assert_eq!(parse_page("Eyes"), None); // no digit to anchor on
        assert_eq!(parse_page("OZ"), None); // letters only, no digit
        assert_eq!(parse_page("3N3"), None); // 'N' is not a digit-lookalike
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
