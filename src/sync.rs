//! Downloads the published per-PDF indexes from the TuneCall server.
//!
//! Same approach as videofinder: plain `http://` (no TLS — it crashes on
//! Android and the server serves http), streamed to disk with `reqwest`. The
//! caller drives this on Slint's event loop via `async-compat`.
//!
//! The server holds an `index.txt` manifest (one `<book>.db` per line, produced
//! by `scripts/upload-indexes.sh`) plus the `.db` files themselves. We fetch the
//! manifest, then each index into the PDF folder next to its book.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Client;

use crate::storage;

const BASE_URL: &str = "http://www.davidfaure.fr/tunecall";

/// Stream `url` to `path` via a temporary `.part` file (so a failed download
/// never leaves a truncated index in place).
async fn download_to(client: &Client, url: &str, path: &Path) -> Result<()> {
    let response = client.get(url).send().await?.error_for_status()?;
    let mut stream = response.bytes_stream();
    let tmp = path.with_extension("part");
    {
        let mut file = File::create(&tmp)?;
        while let Some(chunk) = stream.next().await {
            file.write_all(&chunk?)?;
        }
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Download `index.txt` and every `<book>.db` it lists into the download folder
/// (kept separate from locally authored indexes). Returns how many were fetched.
pub async fn download_indexes() -> Result<usize> {
    let dir = storage::download_dir();
    std::fs::create_dir_all(&dir)?;
    let client = Client::new();

    let manifest = client
        .get(format!("{BASE_URL}/index.txt"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let mut count = 0;
    for line in manifest.lines() {
        let name = line.trim();
        if name.is_empty() || !name.ends_with(".db") {
            continue;
        }
        let url = format!("{BASE_URL}/{name}");
        match download_to(&client, &url, &dir.join(name)).await {
            Ok(()) => count += 1,
            Err(e) => log::warn!("failed to download {name}: {e}"),
        }
    }
    Ok(count)
}
