//! Reads the per-PDF song indexes produced by the indexer (`../indexer`).
//!
//! A book `<stem>.PDF` (in `pdf_dir`) has an index `<stem>.db` with a
//! `songs(title TEXT, page INTEGER)` table, where `page` is the 0-based render
//! page. The `.db` is read from `pdf_dir` first (locally authored) and falls
//! back to `download_dir` (fetched by Reload), so downloads never shadow
//! work-in-progress. The viewer searches across all books; it never writes here.

use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::Connection;

use crate::storage;

/// One indexed song, resolved to a concrete PDF file and render page.
#[derive(Clone)]
pub struct Song {
    pub title: String,
    /// Book display name (the PDF file stem).
    pub book: String,
    /// Absolute path of the PDF to open.
    pub file: PathBuf,
    /// 0-based page to render.
    pub page: i32,
}

/// A book known to the app: it has an index (`<stem>.db`), and may or may not
/// have its PDF installed yet.
pub struct BookStatus {
    /// Book display name (the `.db`/PDF file stem).
    pub name: String,
    /// Human-readable title from the DB's `meta` table (e.g. "The Real Book,
    /// Vol. 1"), if the indexer recorded one. `None` for older DBs.
    pub title: Option<String>,
    /// Whether the matching PDF is present (i.e. the book is searchable now).
    pub has_pdf: bool,
}

/// Load the library: PDFs come from `pdf_dir`; each book's `<stem>.db` is taken
/// from `pdf_dir` first (locally authored), then `download_dir` (fetched by
/// Reload). Books with no index yet are skipped.
pub fn load_library() -> Result<Vec<Song>> {
    load_library_from(&storage::pdf_dir(), &storage::download_dir())
}

/// List every book that has an index, marking whether its PDF is installed.
///
/// PDFs are not shipped (copyright), so a fresh install has indexes but no PDFs.
/// This is what tells the user which PDFs they *can* install: each `.db` (from
/// `pdf_dir` or `download_dir`) is one supported book, `has_pdf` says if it's
/// usable yet. Sorted by name, deduplicated case-insensitively by stem.
pub fn list_books() -> Vec<BookStatus> {
    list_books_from(&storage::pdf_dir(), &storage::download_dir())
}

fn list_books_from(pdf_dir: &Path, download_dir: &Path) -> Vec<BookStatus> {
    let pdfs = list_with_ext(pdf_dir, "pdf");
    let local_dbs = list_with_ext(pdf_dir, "db");
    let downloaded_dbs = list_with_ext(download_dir, "db");

    // Keyed by lowercase stem so the BTreeMap both sorts and dedups; the first
    // occurrence (local before downloaded) keeps its original-case display name.
    let mut books: std::collections::BTreeMap<String, BookStatus> =
        std::collections::BTreeMap::new();
    for db in local_dbs.iter().chain(downloaded_dbs.iter()) {
        let Some(stem) = db.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        books
            .entry(stem.to_lowercase())
            .or_insert_with(|| BookStatus {
                name: stem.to_string(),
                title: read_book_title(db),
                has_pdf: find_by_stem(&pdfs, stem).is_some(),
            });
    }
    books.into_values().collect()
}

/// The human-readable book title from a DB's `meta` table (`key = 'title'`),
/// or `None` if the table/row is absent (older DBs) or the DB can't be opened.
fn read_book_title(db: &Path) -> Option<String> {
    let conn = Connection::open(db).ok()?;
    conn.query_row("SELECT value FROM meta WHERE key = 'title'", [], |r| {
        r.get::<_, String>(0)
    })
    .ok()
}

fn load_library_from(pdf_dir: &Path, download_dir: &Path) -> Result<Vec<Song>> {
    let pdfs = list_with_ext(pdf_dir, "pdf");
    let local_dbs = list_with_ext(pdf_dir, "db");
    let downloaded_dbs = list_with_ext(download_dir, "db");

    let mut songs = Vec::new();
    for pdf in &pdfs {
        let Some(stem) = pdf.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(db) =
            find_by_stem(&local_dbs, stem).or_else(|| find_by_stem(&downloaded_dbs, stem))
        else {
            continue; // no index for this book yet
        };
        match read_songs(db, pdf, stem) {
            Ok(mut s) => songs.append(&mut s),
            Err(e) => log::warn!("failed to read {}: {e}", db.display()),
        }
    }
    Ok(songs)
}

/// Files in `dir` with the given (case-insensitive) extension. Missing dir = none.
fn list_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| has_ext(p, ext))
        .collect()
}

/// Absolute path of an installed PDF by its stem (case-insensitive), or `None`
/// if no `<stem>.pdf` is in `pdf_dir`. Setlists store only the stem, so this
/// turns a setlist entry back into an openable file (PDFs live in `pdf_dir`
/// only; `download_dir` holds just `.db` indexes).
pub fn resolve_pdf(stem: &str) -> Option<PathBuf> {
    resolve_pdf_in(&storage::pdf_dir(), stem)
}

fn resolve_pdf_in(pdf_dir: &Path, stem: &str) -> Option<PathBuf> {
    let pdfs = list_with_ext(pdf_dir, "pdf");
    find_by_stem(&pdfs, stem).map(Path::to_path_buf)
}

/// First file whose stem matches `stem` (case-insensitive).
fn find_by_stem<'a>(files: &'a [PathBuf], stem: &str) -> Option<&'a Path> {
    files
        .iter()
        .find(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(stem))
        })
        .map(|p| p.as_path())
}

fn read_songs(db_path: &Path, pdf: &Path, stem: &str) -> Result<Vec<Song>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT title, page FROM songs")?;
    let rows = stmt.query_map([], |r| {
        let title: String = r.get(0)?;
        let page: i64 = r.get(1)?;
        Ok((title, page as i32))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (title, page) = row?;
        out.push(Song {
            title,
            book: stem.to_string(),
            file: pdf.to_path_buf(),
            page,
        });
    }
    Ok(out)
}

/// Case-insensitive extension check.
fn has_ext(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

/// Search `songs` for titles containing `query` (case-insensitive substring),
/// returning at most `limit`, ordered by title then book.
pub fn search<'a>(songs: &'a [Song], query: &str, limit: usize) -> Vec<&'a Song> {
    let needle = query.trim().to_lowercase();
    let mut hits: Vec<&Song> = songs
        .iter()
        .filter(|s| s.title.to_lowercase().contains(&needle))
        .collect();
    hits.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.book.cmp(&b.book))
    });
    hits.truncate(limit);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_db(path: &Path, rows: &[(&str, i32)]) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch("CREATE TABLE songs(title TEXT, page INTEGER);")
            .unwrap();
        for (title, page) in rows {
            conn.execute("INSERT INTO songs VALUES (?1, ?2)", (title, page))
                .unwrap();
        }
    }

    fn write_db_with_title(path: &Path, rows: &[(&str, i32)], title: &str) {
        write_db(path, rows);
        let conn = Connection::open(path).unwrap();
        conn.execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT);")
            .unwrap();
        conn.execute("INSERT INTO meta VALUES ('title', ?1)", [title])
            .unwrap();
    }

    #[test]
    fn prefers_local_db_then_downloaded() {
        let base = std::env::temp_dir().join(format!("tunecall-test-{}", std::process::id()));
        let pdfs = base.join("pdfs");
        let dl = base.join("downloaded");
        std::fs::create_dir_all(&pdfs).unwrap();
        std::fs::create_dir_all(&dl).unwrap();

        // Book One: local .db (page 30) AND a downloaded .db (page 999) -> local wins.
        std::fs::write(pdfs.join("Book One.PDF"), b"%PDF-1.4").unwrap();
        write_db(&pdfs.join("Book One.db"), &[("All Blues", 30)]);
        write_db(&dl.join("Book One.db"), &[("All Blues", 999)]);

        // Book Two: PDF in pdfs, index only downloaded -> used.
        std::fs::write(pdfs.join("Book Two.PDF"), b"%PDF-1.4").unwrap();
        write_db(&dl.join("Book Two.db"), &[("So What", 5)]);

        // A downloaded .db with no PDF is ignored.
        write_db(&dl.join("Ghost.db"), &[("Nope", 1)]);

        let songs = load_library_from(&pdfs, &dl).unwrap();
        assert_eq!(songs.len(), 2);

        let blues = search(&songs, "all blues", 10);
        assert_eq!(blues.len(), 1);
        assert_eq!(blues[0].page, 30); // local wins, not the downloaded 999
        assert_eq!(blues[0].book, "Book One");

        let so_what = search(&songs, "so what", 10);
        assert_eq!(so_what.len(), 1);
        assert_eq!(so_what[0].page, 5);
        assert!(so_what[0].file.ends_with("Book Two.PDF"));

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn lists_books_with_pdf_availability() {
        let base = std::env::temp_dir().join(format!("tunecall-books-{}", std::process::id()));
        let pdfs = base.join("pdfs");
        let dl = base.join("downloaded");
        std::fs::create_dir_all(&pdfs).unwrap();
        std::fs::create_dir_all(&dl).unwrap();

        // Installed: PDF + local index.
        std::fs::write(pdfs.join("Installed.PDF"), b"%PDF-1.4").unwrap();
        write_db(&pdfs.join("Installed.db"), &[("Song", 1)]);

        // Downloaded index, no PDF yet -> listed but not installed.
        write_db(&dl.join("Missing.db"), &[("Song", 1)]);

        // Same stem in both db dirs -> one entry only.
        write_db(&pdfs.join("Dup.db"), &[("Song", 1)]);
        write_db(&dl.join("Dup.db"), &[("Song", 1)]);

        let books = list_books_from(&pdfs, &dl);
        assert_eq!(books.len(), 3); // Dup, Installed, Missing (sorted)
        assert_eq!(books[0].name, "Dup");
        assert!(!books[0].has_pdf);
        assert_eq!(books[1].name, "Installed");
        assert!(books[1].has_pdf);
        assert_eq!(books[2].name, "Missing");
        assert!(!books[2].has_pdf);

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn reads_book_title_from_meta_when_present() {
        let base = std::env::temp_dir().join(format!("tunecall-titles-{}", std::process::id()));
        let pdfs = base.join("pdfs");
        let dl = base.join("downloaded");
        std::fs::create_dir_all(&pdfs).unwrap();
        std::fs::create_dir_all(&dl).unwrap();

        // Titled: index has a meta title -> surfaced.
        std::fs::write(pdfs.join("Titled.PDF"), b"%PDF-1.4").unwrap();
        write_db_with_title(&pdfs.join("Titled.db"), &[("Song", 1)], "The Real Book, Vol. 1");

        // Untitled: index without a meta table (older DB) -> None, no error.
        write_db(&dl.join("Untitled.db"), &[("Song", 1)]);

        let books = list_books_from(&pdfs, &dl);
        assert_eq!(books.len(), 2); // Titled, Untitled (sorted)
        assert_eq!(books[0].name, "Titled");
        assert_eq!(books[0].title.as_deref(), Some("The Real Book, Vol. 1"));
        assert_eq!(books[1].name, "Untitled");
        assert_eq!(books[1].title, None);

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn resolve_pdf_matches_stem_case_insensitively() {
        let dir = std::env::temp_dir().join(format!("tunecall-resolve-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("RealBk1h.PDF"), b"%PDF-1.4").unwrap();

        assert_eq!(resolve_pdf_in(&dir, "realbk1h"), Some(dir.join("RealBk1h.PDF")));
        assert_eq!(resolve_pdf_in(&dir, "REALBK1H"), Some(dir.join("RealBk1h.PDF")));
        assert_eq!(resolve_pdf_in(&dir, "missing"), None);

        std::fs::remove_dir_all(&dir).ok();
    }
}
