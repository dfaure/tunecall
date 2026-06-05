//! Reads the per-PDF song indexes produced by the indexer (`../indexer`).
//!
//! Each PDF `<stem>.PDF` has a sibling SQLite file `<stem>.db` containing a
//! `songs(title TEXT, page INTEGER)` table, where `page` is the 0-based page to
//! render. The viewer loads every such index it finds in the PDF folder and
//! searches across all of them; it never writes to them.

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

/// Load every `<stem>.db` in the PDF folder that has a matching `<stem>.PDF`.
pub fn load_library() -> Result<Vec<Song>> {
    load_library_from(&storage::pdf_dir())
}

fn load_library_from(dir: &Path) -> Result<Vec<Song>> {
    let mut songs = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(songs); // folder not created yet
    };

    let files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    for db_path in files.iter().filter(|p| has_ext(p, "db")) {
        let Some(stem) = db_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // Find the sibling PDF (same stem, .pdf extension, case-insensitive).
        let Some(pdf) = files.iter().find(|p| {
            has_ext(p, "pdf")
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case(stem))
        }) else {
            log::warn!("index {} has no matching PDF; skipping", db_path.display());
            continue;
        };

        match read_songs(db_path, pdf, stem) {
            Ok(mut s) => songs.append(&mut s),
            Err(e) => log::warn!("failed to read {}: {e}", db_path.display()),
        }
    }
    Ok(songs)
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

    #[test]
    fn loads_songs_and_matches_sibling_pdf() {
        let dir = std::env::temp_dir().join(format!("jambook-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Book One.PDF"), b"%PDF-1.4").unwrap();
        let conn = Connection::open(dir.join("Book One.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE songs(title TEXT, page INTEGER);
             INSERT INTO songs VALUES ('Affirmation', 15), ('All Blues', 30);",
        )
        .unwrap();
        // A .db with no matching PDF must be ignored.
        Connection::open(dir.join("Orphan.db"))
            .unwrap()
            .execute_batch("CREATE TABLE songs(title TEXT, page INTEGER); INSERT INTO songs VALUES ('Nope', 1);")
            .unwrap();

        let songs = load_library_from(&dir).unwrap();
        assert_eq!(songs.len(), 2);

        let hits = search(&songs, "all", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "All Blues");
        assert_eq!(hits[0].book, "Book One");
        assert_eq!(hits[0].page, 30);
        assert!(hits[0].file.ends_with("Book One.PDF"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
