//! SQLite storage of the discovered PDF files.

use anyhow::Result;
use rusqlite::Connection;

use crate::storage;

/// One PDF row.
pub struct PdfEntry {
    pub name: String,
    pub path: String,
}

fn open() -> Result<Connection> {
    let conn = Connection::open(storage::db_path())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pdfs (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            path TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

/// Scan the PDF directory and rebuild the table from what is currently on disk.
/// Returns the number of PDFs found.
pub fn scan_and_store() -> Result<usize> {
    let dir = storage::pdf_dir();
    std::fs::create_dir_all(&dir)?;

    let mut found: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        let is_pdf = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
        if is_pdf {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            found.push((name, path.to_string_lossy().into_owned()));
        }
    }

    let mut conn = open()?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM pdfs", [])?;
    for (name, path) in &found {
        tx.execute(
            "INSERT OR IGNORE INTO pdfs (name, path) VALUES (?1, ?2)",
            (name, path),
        )?;
    }
    tx.commit()?;

    Ok(found.len())
}

/// List stored PDFs, ordered by name (case-insensitive).
pub fn list_pdfs() -> Result<Vec<PdfEntry>> {
    let conn = open()?;
    let mut stmt = conn.prepare("SELECT name, path FROM pdfs ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| {
        Ok(PdfEntry {
            name: r.get(0)?,
            path: r.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
