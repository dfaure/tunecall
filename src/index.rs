//! Parsing of `MasterIndex.PDF`'s extracted text into song entries.
//!
//! Each data line looks like `<Song Title...> <BookCode> <PageLabel>`, e.g.
//! `Affirmation NewReal1 1`. Book codes are single tokens; page labels are
//! usually numbers but sometimes appendix labels like `A1`. A few lines have a
//! missing space between the title and the code (`...LifeRealbk1`), so we match
//! the code as a known suffix of the second-to-last token rather than trusting
//! whitespace alone.

/// One parsed index entry. `code` is the canonical code (the config key casing).
pub struct RawEntry {
    pub title: String,
    pub code: String,
    pub page: String,
}

/// Parse extracted index text lines into entries, recognizing only the given
/// `known_codes` (canonical casing). Header/footer lines and anything not
/// ending in a known book code are skipped.
pub fn parse_master_index(lines: &[String], known_codes: &[String]) -> Vec<RawEntry> {
    // Longest codes first so e.g. a code that is a suffix of another wins correctly.
    let mut codes: Vec<&String> = known_codes.iter().collect();
    codes.sort_by_key(|c| std::cmp::Reverse(c.len()));

    let mut out = Vec::new();
    for raw in lines {
        let line = raw.trim();
        if line.is_empty()
            || line.eq_ignore_ascii_case("Master Index")
            || line.eq_ignore_ascii_case("Song Title Book Page")
        {
            continue;
        }

        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() < 3 {
            continue;
        }
        let page = toks[toks.len() - 1];
        let book_tok = toks[toks.len() - 2];

        // Find a known code that is an ASCII-case-insensitive suffix of book_tok.
        let bt = book_tok.as_bytes();
        let Some(code) = codes.iter().find(|c| {
            let cb = c.as_bytes();
            bt.len() >= cb.len() && bt[bt.len() - cb.len()..].eq_ignore_ascii_case(cb)
        }) else {
            continue;
        };

        // Anything glued in front of the code (missing-space lines) is title text.
        let glued_prefix = &book_tok[..bt.len() - code.len()];
        let mut title = toks[..toks.len() - 2].join(" ");
        if !glued_prefix.is_empty() {
            if !title.is_empty() {
                title.push(' ');
            }
            title.push_str(glued_prefix);
        }
        if title.is_empty() {
            continue;
        }

        out.push(RawEntry {
            title,
            code: (*code).clone(),
            page: page.to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes() -> Vec<String> {
        ["NewReal1", "Realbk1", "RealBk2", "JazzFake"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_basic_entries() {
        let e = parse_master_index(
            &lines(&[
                "Song Title Book Page",
                "Affirmation NewReal1 1",
                "Master Index",
            ]),
            &codes(),
        );
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "Affirmation");
        assert_eq!(e[0].code, "NewReal1");
        assert_eq!(e[0].page, "1");
    }

    #[test]
    fn multiword_titles_and_appendix_pages() {
        let e = parse_master_index(
            &lines(&["All The Things You Are NewReal1 4", "Alfie Realbk1 A1"]),
            &codes(),
        );
        assert_eq!(e[0].title, "All The Things You Are");
        assert_eq!(e[0].page, "4");
        assert_eq!(e[1].title, "Alfie");
        assert_eq!(e[1].page, "A1");
    }

    #[test]
    fn recovers_missing_space_before_code() {
        // "...Life" glued to "Realbk1", and "(... Road)" glued to "NewReal1".
        let e = parse_master_index(
            &lines(&["Sweet Of LifeRealbk1 14", "Long Road)NewReal1 9"]),
            &codes(),
        );
        assert_eq!(e[0].code, "Realbk1");
        assert_eq!(e[0].title, "Sweet Of Life");
        assert_eq!(e[1].code, "NewReal1");
        assert_eq!(e[1].title, "Long Road)");
    }

    #[test]
    fn skips_unknown_codes() {
        let e = parse_master_index(&lines(&["Mystery Tune Commercial 5"]), &codes());
        assert!(e.is_empty());
    }
}
