# TuneCall

A personal application, built in Rust with [Slint](https://slint.rs/) for the
user interface. Cross-platform (desktop + Android), primarily targeting Android.

It searches a collection of (scanned) fake-book PDFs by song title and opens
the matching book at the right page.

This repo is the **viewer**. The song index is produced offline by a separate
Linux tool, [`indexer/`](indexer/README.md).

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
built by OCR'ing each book's table of contents — see `indexer/`. Storing the
actual render page (rather than a printed page + offset) keeps the viewer
correct even when a scan is missing pages.

### pdfium library

`pdfium-render` binds to the pdfium shared library **at runtime**, so building
does not require it, but running does. Get a prebuilt binary from
[bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) and
either install it system-wide or drop it next to the executable:

- Linux: `libpdfium.so`   Windows: `pdfium.dll`   macOS: `libpdfium.dylib`
- Android: place `libpdfium.so` for each ABI under `app/src/main/jniLibs/<abi>/`
  so Gradle packages it into the APK.

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

Only meaningful to the author, move along ;-)
