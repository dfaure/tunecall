//! Writes the per-PDF song index consumed by the TuneCall viewer.
//!
//! Schema (shared contract): `songs(title TEXT, page INTEGER)` where `page` is
//! the 0-based page to render. An optional `meta(key TEXT, value TEXT)` table
//! carries book-level metadata; currently just the human-readable book title
//! (`key = 'title'`). The viewer tolerates its absence (older DBs).

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// (Re)create `out` and write all `(title, page0)` entries. If `book_title` is
/// set, also record it in the `meta` table for the viewer to display.
pub fn write_index(out: &Path, entries: &[(String, i32)], book_title: Option<&str>) -> Result<()> {
    let _ = std::fs::remove_file(out); // rebuild from scratch
    let mut conn = Connection::open(out)?;
    conn.execute_batch(
        "CREATE TABLE songs (
            title TEXT NOT NULL,
            page  INTEGER NOT NULL
        );
        CREATE INDEX idx_songs_title ON songs(title COLLATE NOCASE);
        CREATE TABLE meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO songs (title, page) VALUES (?1, ?2)")?;
        for (title, page) in entries {
            stmt.execute((title, page))?;
        }
        if let Some(title) = book_title {
            tx.execute("INSERT INTO meta (key, value) VALUES ('title', ?1)", [title])?;
        }
    }
    tx.commit()?;
    Ok(())
}
