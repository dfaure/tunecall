//! User-created setlists: named, ordered collections of songs, persisted as
//! JSON in the data dir (`storage::setlists_path`).
//!
//! A setlist song is stored by **book stem + page + title**, never an absolute
//! path: paths differ between desktop and Android, PDFs aren't bundled (a
//! setlist may name a book that isn't installed), and the rest of the app is
//! already keyed on the stem. The concrete PDF path is resolved at play time
//! (`db::resolve_pdf`), exactly like the library loader.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::storage;

/// One song in a setlist. `book` is the PDF file stem; `page` is the 0-based
/// render page (same convention as `db::Song`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetlistSong {
    pub title: String,
    pub book: String,
    pub page: i32,
}

/// A named, ordered list of songs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Setlist {
    pub name: String,
    pub songs: Vec<SetlistSong>,
}

/// Load every setlist. A missing file is simply "no setlists yet"; a corrupt or
/// unreadable file is logged and treated the same, so a bad file never blocks
/// the app (the user can recreate setlists).
pub fn load() -> Vec<Setlist> {
    load_from(&storage::setlists_path())
}

/// Persist all setlists, replacing the file. Written atomically (`.part` then
/// rename) so a crash mid-write can't truncate the existing file.
pub fn save(lists: &[Setlist]) -> Result<()> {
    save_to(&storage::setlists_path(), lists)
}

fn load_from(path: &Path) -> Vec<Setlist> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            log::warn!("reading {} failed: {e}", path.display());
            return Vec::new();
        }
    };
    match serde_json::from_slice(&bytes) {
        Ok(lists) => lists,
        Err(e) => {
            log::warn!("parsing {} failed: {e}", path.display());
            Vec::new()
        }
    }
}

fn save_to(path: &Path, lists: &[Setlist]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(lists)?;
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Move the song at `i` one position earlier (no-op at the start or out of range).
pub fn move_up(songs: &mut [SetlistSong], i: usize) {
    if i > 0 && i < songs.len() {
        songs.swap(i - 1, i);
    }
}

/// Move the song at `i` one position later (no-op at the end or out of range).
pub fn move_down(songs: &mut [SetlistSong], i: usize) {
    if i + 1 < songs.len() {
        songs.swap(i, i + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(title: &str) -> SetlistSong {
        SetlistSong {
            title: title.into(),
            book: "realbk1h".into(),
            page: 1,
        }
    }

    #[test]
    fn json_round_trips() {
        let dir = std::env::temp_dir().join(format!("tunecall-setlist-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("setlists.json");

        let lists = vec![
            Setlist {
                name: "Gig".into(),
                songs: vec![song("All Blues"), song("So What")],
            },
            Setlist {
                name: "Empty".into(),
                songs: vec![],
            },
        ];
        save_to(&path, &lists).unwrap();
        assert_eq!(load_from(&path), lists);
        assert!(!path.with_extension("part").exists()); // atomic write left no temp

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_is_empty() {
        let path = std::env::temp_dir().join(format!("tunecall-nope-{}.json", std::process::id()));
        std::fs::remove_file(&path).ok();
        assert!(load_from(&path).is_empty());
    }

    #[test]
    fn corrupt_file_is_empty() {
        let path = std::env::temp_dir().join(format!("tunecall-bad-{}.json", std::process::id()));
        std::fs::write(&path, b"{ not json").unwrap();
        assert!(load_from(&path).is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn reorder_edges_are_noops() {
        let mut s = vec![song("a"), song("b"), song("c")];
        move_up(&mut s, 0); // first up: no-op
        assert_eq!(
            s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        move_down(&mut s, 2); // last down: no-op
        assert_eq!(
            s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        move_down(&mut s, 0); // a <-> b
        assert_eq!(
            s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
            ["b", "a", "c"]
        );
        move_up(&mut s, 2); // a <-> c
        assert_eq!(
            s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
            ["b", "c", "a"]
        );
    }

    #[test]
    fn reorder_out_of_range_is_safe() {
        let mut s = vec![song("only")];
        move_up(&mut s, 5);
        move_down(&mut s, 5);
        assert_eq!(s.len(), 1);
    }
}
