//! Per-book configuration: maps the book codes used in `MasterIndex.PDF` to the
//! actual PDF file and the page offset needed to open it at the right place.
//!
//! Stored as `books.toml` in the data directory; auto-created from a template on
//! first run, then edited by hand once (the offsets must be measured per book).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::storage;

#[derive(Debug, Deserialize)]
pub struct BookConfig {
    /// PDF file name inside the `pdfs/` folder.
    pub file: String,
    /// The viewer page number (the "n / total" shown in the viewer) that
    /// displays the book's *printed* page 1. Defaults to 1 (printed numbering
    /// matches viewer numbering).
    #[serde(default = "default_first_page")]
    pub first_page: i32,
}

fn default_first_page() -> i32 {
    1
}

#[derive(Debug, Default)]
pub struct BooksConfig {
    books: BTreeMap<String, BookConfig>,
}

impl BooksConfig {
    /// Known book codes (used by the master-index parser to recognize entries).
    pub fn codes(&self) -> Vec<String> {
        self.books.keys().cloned().collect()
    }

    /// Look up a book by code, case-insensitively.
    pub fn get(&self, code: &str) -> Option<&BookConfig> {
        self.books
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(code))
            .map(|(_, v)| v)
    }
}

pub fn config_path() -> PathBuf {
    storage::data_dir().join("books.toml")
}

/// Read `books.toml`, creating it from the template if missing.
pub fn load_or_create() -> Result<BooksConfig> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, TEMPLATE).with_context(|| format!("writing {}", path.display()))?;
        log::info!("wrote default book config to {}", path.display());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let books: BTreeMap<String, BookConfig> =
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(BooksConfig { books })
}

/// Default config covering the books listed in MasterIndex.PDF. The `first_page`
/// values must be measured per book (open it, see which viewer page shows its
/// printed page "1"); only NewReal1 is known up front.
const TEMPLATE: &str = r#"# JamBook book configuration.
#
# One section per book code (as written in MasterIndex.PDF). For each:
#   file       = the PDF file name inside the pdfs/ folder
#   first_page = the VIEWER page number (the "n / total" shown while viewing)
#                that displays this book's PRINTED page 1.
#
# To measure first_page: open the book, scroll to where its printed page "1"
# is, and read the page number shown in the viewer. Put that number here.
# (Example: New Real Book 1's printed page 1 is viewer page 16.)
# Leave at 1 until measured; songs will then open a bit off until you fix it.

[NewReal1]
file = "NEWREAL1.PDF"
first_page = 16

[NewReal2]
file = "NEWREAL2.PDF"
first_page = 1

[NewReal3]
file = "NEWREAL3.PDF"
first_page = 1

[Realbk1]
file = "REALBK1.PDF"
first_page = 1

[RealBk2]
file = "REALBK2.PDF"
first_page = 1

[RealBk3]
file = "REALBK3.PDF"
first_page = 1

[JazzFake]
file = "Jazz Fake Book.PDF"
first_page = 1

[JazzLTD]
file = "Jazz LTD.PDF"
first_page = 1

[Colorado]
file = "Colorado.PDF"
first_page = 1

[Library]
file = "Library of Musicians Jazz.PDF"
first_page = 1

[EvansBk]
file = "Bill EvansFake Book.PDF"
first_page = 1
"#;
