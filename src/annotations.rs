use rusqlite::{Connection, Result, params};

use crate::storage;

/// Two shapes of annotation share the same table, so they live under one lifecycle
/// (page load / move / delete). Text is a floating blue label; Rect is a filled
/// rectangle used to blank out something on the scan.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnnotationKind {
    Text,
    Rect,
}

impl AnnotationKind {
    fn from_db(s: &str) -> Self {
        match s {
            "rect" => AnnotationKind::Rect,
            _ => AnnotationKind::Text,
        }
    }
}

pub struct Annotation {
    pub id: i64,
    pub kind: AnnotationKind,
    pub x: f32,
    pub y: f32,
    /// Normalized width / height (0.0–1.0 of the page). Zero for text, whose box
    /// is sized to fit its label.
    pub w: f32,
    pub h: f32,
    /// Label for text annotations; empty for rectangles.
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
    // Idempotent schema upgrade: rectangles are a later addition, so pre-existing
    // DBs need these columns added. Every ADD COLUMN after the first will fail
    // with "duplicate column name" — that's the steady-state and we ignore it.
    for stmt in [
        "ALTER TABLE annotations ADD COLUMN kind TEXT NOT NULL DEFAULT 'text'",
        "ALTER TABLE annotations ADD COLUMN w REAL NOT NULL DEFAULT 0",
        "ALTER TABLE annotations ADD COLUMN h REAL NOT NULL DEFAULT 0",
    ] {
        let _ = conn.execute(stmt, []);
    }
    Ok(conn)
}

pub fn load(book: &str, page: u16) -> Vec<Annotation> {
    let Ok(conn) = open_db() else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT rowid, kind, x, y, w, h, text FROM annotations \
         WHERE book = ?1 AND page = ?2 ORDER BY rowid",
    ) else {
        return Vec::new();
    };
    stmt.query_map(params![book, page as i64], |row| {
        Ok(Annotation {
            id: row.get(0)?,
            kind: AnnotationKind::from_db(&row.get::<_, String>(1)?),
            x: row.get::<_, f64>(2)? as f32,
            y: row.get::<_, f64>(3)? as f32,
            w: row.get::<_, f64>(4)? as f32,
            h: row.get::<_, f64>(5)? as f32,
            text: row.get(6)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn save_text(book: &str, page: u16, x: f32, y: f32, text: &str) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO annotations (book, page, kind, x, y, w, h, text) \
         VALUES (?1, ?2, 'text', ?3, ?4, 0, 0, ?5)",
        params![book, page as i64, x as f64, y as f64, text],
    )?;
    Ok(())
}

pub fn save_rect(book: &str, page: u16, x: f32, y: f32, w: f32, h: f32) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO annotations (book, page, kind, x, y, w, h, text) \
         VALUES (?1, ?2, 'rect', ?3, ?4, ?5, ?6, '')",
        params![book, page as i64, x as f64, y as f64, w as f64, h as f64,],
    )?;
    Ok(())
}

pub fn update_position(id: i64, x: f32, y: f32) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "UPDATE annotations SET x = ?1, y = ?2 WHERE rowid = ?3",
        params![x as f64, y as f64, id],
    )?;
    Ok(())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = open_db()?;
    conn.execute("DELETE FROM annotations WHERE rowid = ?1", params![id])?;
    Ok(())
}
