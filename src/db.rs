//! SQLite storage of the song index parsed from the master index.

use anyhow::Result;
use rusqlite::Connection;

use crate::index::RawEntry;
use crate::storage;

/// One indexed song (one row = one song in one book).
pub struct Song {
    pub title: String,
    pub book_code: String,
    pub printed_page: String,
}

fn open() -> Result<Connection> {
    let conn = Connection::open(storage::db_path())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS songs (
            id           INTEGER PRIMARY KEY,
            title        TEXT NOT NULL,
            book_code    TEXT NOT NULL,
            printed_page TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title COLLATE NOCASE);",
    )?;
    Ok(conn)
}

/// Number of indexed songs.
pub fn song_count() -> Result<i64> {
    let conn = open()?;
    Ok(conn.query_row("SELECT COUNT(*) FROM songs", [], |r| r.get(0))?)
}

/// Replace the whole song index with the given entries.
pub fn replace_songs(entries: &[RawEntry]) -> Result<usize> {
    let mut conn = open()?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM songs", [])?;
    {
        let mut stmt =
            tx.prepare("INSERT INTO songs (title, book_code, printed_page) VALUES (?1, ?2, ?3)")?;
        for e in entries {
            stmt.execute((&e.title, &e.code, &e.page))?;
        }
    }
    tx.commit()?;
    Ok(entries.len())
}

/// Search songs whose title contains `query` (case-insensitive substring).
pub fn search_songs(query: &str, limit: i64) -> Result<Vec<Song>> {
    let conn = open()?;
    // Strip LIKE wildcards from user input so they are treated literally.
    let cleaned: String = query.chars().filter(|c| *c != '%' && *c != '_').collect();
    let pattern = format!("%{cleaned}%");
    let mut stmt = conn.prepare(
        "SELECT title, book_code, printed_page FROM songs
         WHERE title LIKE ?1
         ORDER BY title COLLATE NOCASE, book_code
         LIMIT ?2",
    )?;
    let rows = stmt.query_map((pattern, limit), |r| {
        Ok(Song {
            title: r.get(0)?,
            book_code: r.get(1)?,
            printed_page: r.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
