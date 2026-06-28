use rusqlite::{params, Connection, Result};

use crate::storage;

pub struct Annotation {
    pub x: f32,
    pub y: f32,
    pub text: String,
}

fn open_db() -> Result<Connection> {
    let path = storage::data_dir().join("annotations.db");
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS annotations (
            book TEXT NOT NULL,
            page INTEGER NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL,
            text TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

pub fn load(book: &str, page: u16) -> Vec<Annotation> {
    let Ok(conn) = open_db() else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT x, y, text FROM annotations WHERE book = ?1 AND page = ?2 ORDER BY rowid",
    ) else {
        return Vec::new();
    };
    stmt.query_map(params![book, page as i64], |row| {
        Ok(Annotation {
            x: row.get(0)?,
            y: row.get(1)?,
            text: row.get(2)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn save(book: &str, page: u16, x: f32, y: f32, text: &str) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO annotations (book, page, x, y, text) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![book, page as i64, x as f64, y as f64, text],
    )?;
    Ok(())
}
