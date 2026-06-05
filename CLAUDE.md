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

**Application core** (`src/lib.rs`): `jambook_main()` is the entry point. Creates the `AppWindow`, wires up Slint callbacks (rescan, open-pdf, prev/next/close), and feeds Slint models. The UI model carries only PDF names; their paths stay in Rust (`Rc<RefCell<Vec<String>>>`, indexed by row). `android_main()` is the Android `#[no_mangle]` entry point that resolves the app-specific data dir, initializes logging-to-file and the Slint Android backend, then calls `jambook_main()`.

**Storage paths** (`src/storage.rs`): Resolves `data_dir()` / `pdf_dir()` / `db_path()`. Desktop uses `dirs::data_dir()`; Android sets the base via `set_data_dir()` from `android_main`.

**Database** (`src/db.rs`): `rusqlite` (bundled SQLite). `scan_and_store()` walks `pdf_dir()` for `*.pdf` and rebuilds the `pdfs` table; `list_pdfs()` reads it back.

**PDF rendering** (`src/pdf.rs`): `pdfium-render` bound dynamically at runtime (thread-local, lazily). `page_count()` and `render_page()` rasterize a page to a `slint::Image` via `as_rgba_bytes()` (no `image` crate dependency).

**Binary entry** (`src/bin/main.rs`): Sets up stderr logging via `flexi_logger` and calls `jambook_main()`. Only built on desktop (the `with-binary` feature, on by default).

## Runtime requirement: pdfium

`pdfium-render` binds to the pdfium shared library **at runtime**, not at build time. The loader (`src/pdf.rs`) tries `./libpdfium.so` (relative to the working directory) first, then the system library.

- Desktop: drop a prebuilt `libpdfium.so` / `pdfium.dll` / `libpdfium.dylib` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) at the repo root (git-ignored by `*.so`), or install it system-wide.
- Android: place `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/` so Gradle packages it into the APK.

The PDF list and rescan work without pdfium; only rendering a page needs it.

## Key Design Notes

- Library compiles as both `cdylib` (Android native lib) and `rlib` (desktop binary). The `with-binary` feature (default) enables the desktop binary, and is disabled for Android builds.
- `slint` and `slint-build` are versioned dependencies (no workspace); the Android backend is selected via the `slint/backend-android-activity-06` feature from the command line / Gradle.
- No test suite exists.

## Platform status

- Desktop (Linux): verified working — scan, list, and in-app rendering.
- Android: not yet built/tested. `android_main` is `#[cfg(target_os = "android")]`, so the desktop build never type-checks it; the app-specific data dir and `jniLibs` packaging still need a real NDK build to validate.
