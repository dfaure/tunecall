# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

TuneCall is a cross-platform (desktop + Android, primarily Android) Rust app using Slint for the UI. It is the **viewer**: it searches a song index and opens the matching (scanned) fake-book PDF at the right page.

The index is built by a **separate Linux-only tool** in `indexer/` (`tunecall-indexer`), which turns a transcribed table of contents (a `<stem>.index` sidecar) into a per-PDF SQLite file. See `indexer/README.md`.

### Why this split

The fake-book PDFs are scanned images with no text layer. An earlier approach parsed a single global `MasterIndex.PDF` and applied a per-book page offset, but a constant offset breaks whenever a scan has missing pages (every later page is off). So the index now stores the **actual** render page per song, computed offline by the indexer, and the viewer just renders it.

### Shared contract

Each PDF `<stem>.PDF` has a sibling `<stem>.db` (in the same folder) with:
`songs(title TEXT, page INTEGER)`, where `page` is the **0-based page to render**.

## Build Commands

```bash
cargo build                  # Desktop debug build (the viewer)
cargo run                    # Run desktop app
cargo test                   # Unit tests
cargo fmt                    # Format code
cargo clippy --all-targets -- -D warnings  # Lint (all warnings are errors)

cd indexer && cargo run -- --help           # The indexer (separate package)
```

Pre-commit hooks enforce `cargo fmt` and `cargo clippy` on every commit.

Android builds target `aarch64-linux-android` and use Gradle (`./gradlew build` from root). The `indexer/` crate is Linux-only and never part of the Android build.

## Architecture (viewer)

**UI layer** (`ui/*.slint`): Declarative Slint UI compiled at build time via `build.rs`. Uses `fluent-light` style. `app-window.slint` is the main window: a search box, a results list, and a full-window page viewer overlay.

**Application core** (`src/lib.rs`): `tunecall_main()` creates the `AppWindow`, loads the library, and wires up callbacks (search, open-result, reload, prev/next/close). The whole library and the current search results are held in Rust (`Rc<RefCell<Vec<db::Song>>>`) so a clicked row maps straight to its song (file + page). `android_main()` is the Android `#[no_mangle]` entry point that resolves the app-specific data dir, initializes logging-to-file and the Slint Android backend, then calls `tunecall_main()`.

**Storage paths** (`src/storage.rs`): Resolves `data_dir()` / `pdf_dir()`. Desktop uses `dirs::data_dir()`; Android sets the base via `set_data_dir()` from `android_main`.

**Library** (`src/db.rs`): `rusqlite` (bundled SQLite), read-only. `load_library()` scans `pdf_dir()` for `<stem>.db` files with a matching `<stem>.PDF` and reads their `songs` rows; `search()` is an in-memory case-insensitive substring match over titles. Unit-tested.

**PDF rendering** (`src/pdf.rs`): `pdfium-render` bound dynamically at runtime (thread-local, lazily). `page_count()` and `render_page()` rasterize a page to a `slint::Image` via `as_rgba_bytes()` (no `image` crate dependency).

**Binary entry** (`src/bin/main.rs`): Sets up stderr logging via `flexi_logger` and calls `tunecall_main()`. Only built on desktop (the `with-binary` feature, on by default).

## Architecture (indexer, `indexer/`)

Standalone package `tunecall-indexer` (not in the viewer's build). The scans are too degraded for reliable OCR, so the index is transcribed by reading the rendered TOC pages into a `<stem>.index` sidecar (`<printed-page> <title>` per line, next to the PDF, not in git). `src/main.rs` loads it (`index.rs`, parser unit-tested), maps each printed page to a 0-based scan page via `--offset` (`resolve_page`), drops exact-duplicate rows, and writes `<stem>.db` (`db.rs`). `render.rs` only reads the PDF page count (to validate `--offset` and clamp out-of-range entries), so a pdfium library is needed at runtime but tesseract is not. **Limitation:** a single `--offset` can't model a scan with missing/extra pages; out-of-range entries are clamped (fix the page in the `.index`). Earlier versions OCR'd via tesseract with per-book `.corrections`; that code is in the git history.

### Making/redoing a `<stem>.index` (the all-vision workflow)

tesseract can't read these scans, so **Claude transcribes the TOC by reading rendered images** — there is no automated OCR step. To (re)index a book:

1. First skim the TOC at low DPI (`-r 100`) to find which pages it spans and how the columns are laid out, then render those pages at 600 DPI:
   `pdftoppm -f <first> -l <last> -png -r 600 <book>.pdf <prefix>` → `<prefix>-NNN.png`.
2. **Don't Read a full 600-DPI page directly** — the Read tool downscales a ~5000×7000 image to fit, which blurs the small **bold page numbers** and silently produces misreads (e.g. 183→184, 186→189, 264→266). Instead, `magick`-crop each column into **half-column-height tiles** (~25–30 entries each, e.g. `magick page.png -crop 2480x3300+X+Y +repage tile.png`) so each tile renders near native resolution, then Read the tiles. Transcribe every entry as `<printed-page><TAB-or-space><title>`. Titles are reliable; the **page numbers are the error-prone part** — the printed page is the small right-margin number, not a scan page, and it is *not* monotonic in alphabetical order (songs are placed to fit pages), so you cannot infer it from the sequence — read each one.
3. Determine `--offset` **empirically, don't guess**: the indexer computes `render_page = printed + offset - 1` (0-based). Pick one song, render its scan page, and read the printed number at the page corner; solve for `offset`. For the common layout where the front cover is scan page 1 and printed numbers coincide with scan pages, `offset` is **0** (not -1).
4. Build: `cargo run -- --pdf <book>.pdf --offset N` (per-book offsets live in `indexer/rebuild_db.sh`; a clamp warning usually means the offset is wrong). Add the book's `index <stem> <offset>` line there.
5. **Verify before trusting it**: query a few rows (`sqlite3 <stem>.db "select page,title from songs where title in (...)"`), render each `page+1` (1-based scan page), and confirm the rendered title matches — especially any entries whose numbers looked duplicated or out of order, since those are exactly the misreads. A duplicate page number across two titles is almost always a transcription error; chase it down.

Use `--dry-run` to preview. The `.index` and `.db` live in the data dir, never in git.

## Runtime requirement: pdfium

`pdfium-render` binds to the pdfium shared library **at runtime**, not at build time. The loader (`src/pdf.rs`) tries `./libpdfium.so` (relative to the working directory) first, then the system library.

- Desktop: drop a prebuilt `libpdfium.so` / `pdfium.dll` / `libpdfium.dylib` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) at the repo root (git-ignored by `*.so`), or install it system-wide.
- Android: place `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/` so Gradle packages it into the APK.

Search works without pdfium; only rendering a page needs it.

## Key Design Notes

- Library compiles as both `cdylib` (Android native lib) and `rlib` (desktop binary). The `with-binary` feature (default) enables the desktop binary, and is disabled for Android builds.
- `slint` and `slint-build` are versioned dependencies (no workspace); the Android backend is selected via the `slint/backend-android-activity-06` feature from the command line / Gradle.
- Slint uses the **FemtoVG** renderer (`renderer-femtovg`), not Skia. FemtoVG is pure Rust + OpenGL and avoids Skia's heavy native build/binary download (a real pain for the Android cross-build). PDF page rendering still uses pdfium; that is unrelated to the UI renderer.
- Tests: `cargo test` covers the viewer's library loader (`src/db.rs`) and the indexer's index-file parser (`indexer/src/index.rs`). UI and rendering are not tested.

## Platform status

- Desktop (Linux): viewer verified building/clippy/tests; in-app rendering verified earlier. Indexer builds all five book DBs from their `.index` sidecars.
- Android: not yet built/tested. `android_main` is `#[cfg(target_os = "android")]`, so the desktop build never type-checks it; the app-specific data dir and `jniLibs` packaging still need a real NDK build to validate.
