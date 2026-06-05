# jambook-indexer

Linux-only tool that builds a per-PDF song index for JamBook by OCR'ing a
fake-book's table of contents.

It writes `<book>.db` next to the PDF, with the contract the viewer expects:
`songs(title TEXT, page INTEGER)` where `page` is the 0-based page to render.

## Requirements

- `tesseract` on PATH (e.g. `zypper install tesseract tesseract-data-eng`).
- A pdfium shared library: found next to the cwd / a parent (the jambook repo
  keeps `libpdfium.so` at its root) or installed system-wide.

## Usage

```
cd indexer
cargo run -- --pdf "../<...>/NEWREAL1.PDF" --toc 6-9 --offset 16
```

- `--toc`   the TOC pages, as 1-based scan page numbers (`6-9`, `6-9,12`).
- `--offset` (1-based scan page) − (printed page); 0 if they match, e.g. `15` if
  printed page 1 is on scan page 16. May be negative.
- `--dry-run` parse and print without writing the DB (use this to tune `--toc`).

## Known limitation (next step)

Printed→scan page mapping currently uses a single `--offset`, which is wrong as
soon as the scan has missing/extra pages — exactly why JamBook left the global
master index. The robust fix (planned, behind `resolve_page` in `main.rs`) is to
OCR the printed page number off each scanned page and build a real printed→scan
map. OCR parsing of messy TOC layouts will also need iteration.
