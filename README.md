# JamBook

A personal application, built in Rust with [Slint](https://slint.rs/) for the
user interface. Cross-platform (desktop + Android), primarily targeting Android.

It indexes a collection of (scanned) fake-book PDFs by song title, lets you
search for a song, and opens the matching book at the right page.

## How it works

- PDFs live in `<data-dir>/pdfs/`. On desktop `<data-dir>` is
  `dirs::data_dir()/jambook` (e.g. `~/.local/share/jambook`); on Android it is
  the app-specific data path.
- The song index is built by parsing the text of `MasterIndex.PDF` (a master
  index listing `Song Title  Book  Page`) into `<data-dir>/jambook.db`
  (table `songs`). The book PDFs themselves are scanned images with no text
  layer, so the master index is the only machine-readable source.
- `<data-dir>/books.toml` maps each book code used in the master index to its
  PDF file and a `first_page` offset (the viewer page that shows the book's
  printed page 1). It is auto-created from a template on first run; **measure
  and fill in each `first_page` once** (only `NewReal1 = 16` is known up front).
- Searching matches song titles; picking a result opens that book's PDF at
  `first_page + printed_page - 1`. Pages are rendered with
  [pdfium](https://pdfium.googlesource.com/pdfium/) via `pdfium-render`.

Books not listed in `MasterIndex.PDF` (e.g. *The Commercial Music Book*) are
not indexed. Appendix pages with non-numeric labels (RealBk1 `A1`…`A13`) open
at the book start rather than the exact page.

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
