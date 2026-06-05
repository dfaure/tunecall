# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

JamBook is a cross-platform (desktop + Android, primarily Android) Rust application using Slint for the UI.

## Build Commands

```bash
cargo build                  # Desktop debug build
cargo run                    # Run desktop app
cargo build --release        # Release build

cargo fmt                    # Format code
cargo clippy -- -D warnings  # Lint (all warnings are errors)
```

Pre-commit hooks enforce `cargo fmt` and `cargo clippy` on every commit.

Android builds target `aarch64-linux-android` and use Gradle (`./gradlew build` from root).

## Architecture

**UI layer** (`ui/*.slint`): Declarative Slint UI compiled at build time via `build.rs`. Uses `fluent-light` style. `app-window.slint` is the main window.

**Application core** (`src/lib.rs`): `jambook_main()` is the entry point. Creates the `AppWindow`, loads the book config, runs a first-run import, and wires up Slint callbacks (search, open-result, reimport, prev/next/close). Search results are held in Rust (`Rc<RefCell<Vec<db::Song>>>`) so a clicked row maps back to a song; `pdfium_page_0based()` turns a printed page label + the book's `first_page` into a 0-based pdfium page. `android_main()` is the Android `#[no_mangle]` entry point that resolves the app-specific data dir, initializes logging-to-file and the Slint Android backend, then calls `jambook_main()`.

**Storage paths** (`src/storage.rs`): Resolves `data_dir()` / `pdf_dir()` / `db_path()`. Desktop uses `dirs::data_dir()`; Android sets the base via `set_data_dir()` from `android_main`.

**Book config** (`src/config.rs`): `books.toml` (auto-created in the data dir) mapping each master-index book code to its PDF `file` and `first_page` offset. `serde` + `toml`.

**Index parsing** (`src/index.rs`): `parse_master_index()` turns the extracted master-index text into `RawEntry`s. Splits `<title> <code> <page>` from the right, matching the code as a known case-insensitive suffix of the second-to-last token (recovers missing-space lines like `LifeRealbk1`). Unit-tested.

**Database** (`src/db.rs`): `rusqlite` (bundled SQLite). `replace_songs()` rebuilds the `songs(title, book_code, printed_page)` table; `search_songs()` does a case-insensitive title `LIKE`; `song_count()` for first-run detection.

**PDF rendering / text** (`src/pdf.rs`): `pdfium-render` bound dynamically at runtime (thread-local, lazily). `page_count()` and `render_page()` rasterize a page to a `slint::Image` via `as_rgba_bytes()` (no `image` crate dependency); `all_text_lines()` extracts text for the master-index import. **The fake-book PDFs are scanned images (no text layer); only `MasterIndex.PDF` has extractable text** — hence the index-based approach rather than parsing the books or OCR.

**Binary entry** (`src/bin/main.rs`): Sets up stderr logging via `flexi_logger` and calls `jambook_main()`. Only built on desktop (the `with-binary` feature, on by default).

## Runtime requirement: pdfium

`pdfium-render` binds to the pdfium shared library **at runtime**, not at build time. The loader (`src/pdf.rs`) tries `./libpdfium.so` (relative to the working directory) first, then the system library.

- Desktop: drop a prebuilt `libpdfium.so` / `pdfium.dll` / `libpdfium.dylib` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) at the repo root (git-ignored by `*.so`), or install it system-wide.
- Android: place `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/` so Gradle packages it into the APK.

The PDF list and rescan work without pdfium; only rendering a page needs it.

## Key Design Notes

- Library compiles as both `cdylib` (Android native lib) and `rlib` (desktop binary). The `with-binary` feature (default) enables the desktop binary, and is disabled for Android builds.
- `slint` and `slint-build` are versioned dependencies (no workspace); the Android backend is selected via the `slint/backend-android-activity-06` feature from the command line / Gradle.
- Tests: only the master-index parser is unit-tested (`cargo test`, in `src/index.rs`). The UI and PDF/DB layers are not.

## Platform status

- Desktop (Linux): verified working — scan, list, and in-app rendering.
- Android: not yet built/tested. `android_main` is `#[cfg(target_os = "android")]`, so the desktop build never type-checks it; the app-specific data dir and `jniLibs` packaging still need a real NDK build to validate.
