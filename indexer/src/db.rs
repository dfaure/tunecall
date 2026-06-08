//! Writes the per-PDF song index consumed by the TuneCall viewer.
//!
//! Schema (shared contract): `songs(title TEXT, page INTEGER)` where `page` is
//! the 0-based page to render, plus a `meta(key TEXT, value TEXT)` table
//! carrying book-level metadata — currently just the human-readable book title
//! (`key = 'title'`). The viewer still tolerates a missing `meta` table (older
//! DBs predate it), but this indexer always writes one.

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// (Re)create `out` and write all `(title, page0)` entries plus the book's
/// display title into the `meta` table for the viewer to show.
pub fn write_index(out: &Path, entries: &[(String, i32)], book_title: &str) -> Result<()> {
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
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('title', ?1)",
            [book_title],
        )?;
    }
    tx.commit()?;
    Ok(())
}
