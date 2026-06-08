# tunecall-indexer

Linux-only tool that builds a per-PDF song index for TuneCall from a transcribed
table of contents.

It writes `<book>.db` next to the PDF, with the contract the viewer expects:
`songs(title TEXT, page INTEGER)` where `page` is the 0-based page to render.

## Why there's no OCR

The scanned fake-books are degraded enough that tesseract mangled both the titles
*and* the small right-margin page numbers (dense dot-leaders swallow the digits),
so its output needed near-total correction. Instead, the index is transcribed by
**reading the rendered TOC pages directly** into a `<stem>.index` sidecar, and
this tool just maps printed pages to scan pages. (Earlier versions OCR'd with
tesseract and applied per-book corrections; that code is in the git history.)

## Requirements

- A pdfium shared library — used only to read each PDF's page count (to validate
  `--offset` and clamp out-of-range entries). Found next to the cwd / a parent
  (the repo root keeps `libpdfium.so`) or installed system-wide.

## The `<stem>.index` sidecar

One `<printed-page> <title>` per line, next to the PDF (not in git): the page
number, then whitespace (a tab or spaces — either works), then the title (which
may contain spaces). Blank lines and `#` comments are ignored.

```
# realbk3h.index
295 Sy Clone
296 T.J.R.C.
298 Tea For Two
```

Transcribe it by rendering the TOC pages and reading them (600 DPI reads page
numbers most reliably):

```
pdftoppm -f 2 -l 6 -png -r 600 realbk3h.pdf toc   # render TOC pages -> toc-*.png
# ...read the PNGs and write realbk3h.index...
```

## Usage

```
cd indexer
cargo run -- --pdf ~/.local/share/tunecall/pdfs/realbk3h.pdf --offset 5 --title "The Real Book, Vol. 3"
```

- `--pdf` the book; its sibling `<stem>.index` supplies the entries.
- `--offset` (1-based scan page) − (printed page); 0 if they match, e.g. `15` if
  printed page 1 is on scan page 16. May be negative.
- `--title` (required) the human-readable book name (read off the cover); stored
  in a `meta(key,value)` table and shown in the viewer's Books list. The viewer
  falls back to the file stem only for older DBs that predate this field.
- `--out` override the output DB path (default `<stem>.db`).
- `--dry-run` resolve and print the entries without writing the DB.

Exact-duplicate `(title, page)` rows are dropped. An entry whose resolved page
falls outside the PDF is clamped to the last page, with a warning — usually a
sign `--offset` is wrong, or a stray page number to fix in the `.index`.

## Publishing

`./upload-indexes.sh` uploads every `<book>.db` plus an `index.txt` manifest to
`ftp.davidfaure.fr/tunecall` (via ncftp's `davidfaure` bookmark). The app's
**Reload** button then downloads them. See the top-level README.

## Limitation

A single `--offset` can't model a scan with genuinely missing/extra pages mid-book.
The current PDF set doesn't have that problem, so one offset per book suffices;
if a specific entry is off, fix its page in the `.index` directly and re-run.
