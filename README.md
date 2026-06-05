# JamBook

A personal application, built in Rust with [Slint](https://slint.rs/) for the
user interface. Cross-platform (desktop + Android), primarily targeting Android.

It scans an app-specific folder for PDF files, indexes their names in a local
SQLite database, and lets you pick one and view its pages in-app.

## How it works

- PDFs are read from `<data-dir>/pdfs/` — created on first launch. Drop `.pdf`
  files there and hit **Rescan** (or restart). On desktop `<data-dir>` is
  `dirs::data_dir()/jambook` (e.g. `~/.local/share/jambook`); on Android it is
  the app-specific data path.
- File names are stored in `<data-dir>/jambook.db` (table `pdfs`).
- Pages are rendered with [pdfium](https://pdfium.googlesource.com/pdfium/) via
  the `pdfium-render` crate.

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
