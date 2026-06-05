//! Parse OCR'd table-of-contents text into `(title, printed_page)` entries.
//!
//! TOC lines typically look like `Affirmation .......... 1` or
//! `All The Things You Are    4`. We take the trailing integer as the printed
//! page and everything before it (minus dot leaders) as the title. This is
//! deliberately simple; OCR noise and odd layouts are expected to need
//! iteration.

/// Parse TOC lines into `(title, printed_page)` pairs.
pub fn parse_toc(lines: &[String]) -> Vec<(String, i32)> {
    let mut out = Vec::new();
    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        let Some(last) = toks.last() else {
            continue;
        };
        // Trailing token must be a page number (tolerate a trailing '.' etc.).
        let Ok(page) = last
            .trim_matches(|c: char| !c.is_ascii_digit())
            .parse::<i32>()
        else {
            continue;
        };
        // Title is everything before the page, with dot leaders / separators trimmed.
        let title = toks[..toks.len() - 1]
            .join(" ")
            .trim_end_matches(['.', ' ', '·', '-'])
            .trim()
            .to_string();
        if title.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
            continue; // drop headers / noise like "Index" or stray numbers
        }
        out.push((title, page));
    }
    out
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
}
