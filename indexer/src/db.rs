//! Writes the per-PDF song index consumed by the TuneCall viewer.
//!
//! Schema (shared contract): `songs(title TEXT, page INTEGER)` where `page` is
//! the 0-based page to render.

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// (Re)create `out` and write all `(title, page0)` entries.
pub fn write_index(out: &Path, entries: &[(String, i32)]) -> Result<()> {
    let _ = std::fs::remove_file(out); // rebuild from scratch
    let mut conn = Connection::open(out)?;
    conn.execute_batch(
        "CREATE TABLE songs (
            title TEXT NOT NULL,
            page  INTEGER NOT NULL
        );
        CREATE INDEX idx_songs_title ON songs(title COLLATE NOCASE);",
    )?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO songs (title, page) VALUES (?1, ?2)")?;
        for (title, page) in entries {
            stmt.execute((title, page))?;
        }
    }
    tx.commit()?;
    Ok(())
}
