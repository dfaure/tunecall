# TuneCall


This application lets the user search a collection of (scanned) fake-book PDFs
by song title and opens the matching book at the right page.

The song index is produced offline by a separate tool, [`indexer/`](indexer/README.md).

## How it works

- PDFs live in `<data-dir>/pdfs/`. On desktop `<data-dir>` is
  `dirs::data_dir()/tunecall` (e.g. `~/.local/share/tunecall`); on Android it is
  the app-specific data path.
- Each book `<name>.PDF` has a sibling index `<name>.db` in the same folder:
  a SQLite file with `songs(title TEXT, page INTEGER)`, where `page` is the
  0-based page to render. The viewer loads every such index and searches across
  all of them.
- Picking a result opens that book's PDF at the stored page. Pages are rendered
  with [pdfium](https://pdfium.googlesource.com/pdfium/) via `pdfium-render`.

The books are scanned images with no text layer, so the per-PDF indexes are
built by transcribing each book's table of contents (reading the rendered TOC
pages — the scans are too degraded for reliable OCR) — see `indexer/`. The DB
stores the actual 0-based render page, so the viewer just renders it.

### pdfium library

`pdfium-render` binds to the pdfium shared library **at runtime**, so building
does not require it, but running does. Get a prebuilt binary from
[bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) and
either install it system-wide or drop it next to the executable:

- Linux: `libpdfium.so`   Windows: `pdfium.dll`   macOS: `libpdfium.dylib`
- Android: place `libpdfium.so` for each ABI under `app/src/main/jniLibs/<abi>/`
  so Gradle packages it into the APK.

### Technology

This application is built in Rust with [Slint](https://slint.rs/) for the
user interface. Cross-platform (desktop + Android), primarily targeting Android.

## Compiling

### Desktop

```
cargo run
```

### Building on an Android tablet directly

First, follow the steps at https://github.com/dfaure/rust-android-hello-world

To build the native library for Android, point Cargo at the Android backend
feature instead of the desktop one, then let Gradle package the APK:

```
cargo build --lib --target aarch64-linux-android \
    --no-default-features \
    --features slint/backend-android-activity-06
./gradlew build
```

(See `Cargo.toml` for the desktop vs. Android feature comments.)

## Usage

Install the PDFs like realbk1h.pdf (the name matters!) in the directory
/storage/emulated/0/Android/data/fr.davidfaure.tunecall/files/pdfs/
from a PC (connected via USB to your Android device)

### Publishing / syncing the song indexes

The PDFs can't be published (copyright), but the small `.db` indexes can. The
app downloads them on demand so you don't have to re-copy them after every fix:

1. (Re)generate the indexes with the indexer (see `indexer/`).
2. Upload them: `scripts/upload-indexes.sh` — pushes every `<book>.db` plus an
   `index.txt` manifest to `ftp.davidfaure.fr/tunecall` via ncftp's `davidfaure`
   bookmark (which stores the credentials).
3. In the app, tap **Reload** — it downloads `index.txt` and each listed `.db`
   from `http://www.davidfaure.fr/tunecall/` into the device's `pdfs/` folder,
   then re-reads the library.

Download uses plain `http://` (no TLS — same as videofinder; HTTPS misbehaves on
Android and the server serves both).

