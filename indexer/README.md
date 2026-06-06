# tunecall-indexer

Linux-only tool that builds a per-PDF song index for TuneCall by OCR'ing a
fake-book's table of contents.

It writes `<book>.db` next to the PDF, with the contract the viewer expects:
`songs(title TEXT, page INTEGER)` where `page` is the 0-based page to render.

## Requirements

- The tesseract OCR engine on PATH. On openSUSE: `zypper install tesseract-ocr
  tesseract-ocr-traineddata-eng` — note the `tesseract` package is an unrelated
  *game* (its binary is `tesseract-game`); the OCR engine is `tesseract-ocr`.
- A pdfium shared library: found next to the cwd / a parent (the repo root
  keeps `libpdfium.so`) or installed system-wide.

## Usage

```
cd indexer
cargo run -- --pdf "../<...>/NEWREAL1.PDF" --toc 6-9 --offset 16
```

- `--toc`   the TOC pages, as 1-based scan page numbers (`6-9`, `6-9,12`).
- `--detect-toc` auto-detect the TOC range instead of `--toc`: OCRs the first
  16 pages, counts index entries per page, and picks the longest run scoring
  20+ (e.g. realbk1h → 3-8, realbk2h → 2-6). Pass `--toc` to override.
- `--offset` (1-based scan page) − (printed page); 0 if they match, e.g. `15` if
  printed page 1 is on scan page 16. May be negative.
- `--dry-run` parse and print without writing the DB (use this to tune `--toc`).
- `--dpi N` render resolution for OCR (default 400). ~400 reads best here;
  higher can make tesseract merge the two TOC columns again.
- `--psm N` tesseract page-segmentation mode (default 3 = auto layout).
- `--no-repair` disable page-number repair (see below).
- `--repair-tolerance N` how far (pages) a value may deviate from the trend
  before it's treated as a gross outlier (default 20).

## Page-number repair

A fake-book index is alphabetical and the book is only *roughly* in that order,
so page numbers trend upward but with genuine small inversions (e.g. `ALWAYS`
on p.23 listed right after `ALRIGHT` on p.24). The indexer finds the dominant
upward trend (the longest non-decreasing run of OCR'd pages) and only corrects
values that deviate from it by more than `--repair-tolerance`, or are out of
range — so it fixes gross OCR errors (`358 → 38`, `0 → interpolated`) while
leaving real inversions untouched. Corrected entries show as `p.38 (ocr:358)`
in the dry-run output. Pass `--no-repair` to keep raw OCR pages.

(This fixes misread *page numbers*; it does not fix the printed→scan mapping for
scans with missing pages — that is still the `--offset` limitation above.)

## Title corrections

Some titles are mangled by OCR beyond what the parser can repair — initialisms
like `T.J.R.C.` have no language-model support and come out as garbage
(`Ot > A`). Drop a sidecar `<stem>.corrections` next to the PDF/.db to override
them:

```
# realbk1h.corrections — one "<printed-page> <title>" per line
39 AUTUMN LEAVES
296 T.J.R.C.
```

The indexer overrides the title of the entry on that **printed** page, or adds
an entry if OCR dropped the page entirely. Keying on the printed page (not the
garbled text) means a correction keeps working after you re-tune OCR. Blank
lines and `#` comments are ignored; corrected/added rows are flagged in the
dry-run output. The file lives next to the book (it is not in git).

## Full-index sidecar (all-vision)

When a scan is degraded enough that OCR is useless (dense dot-leaders that eat
the page numbers, etc.), skip tesseract entirely: drop a `<stem>.index` sidecar
(same `<printed-page> <title>` format as `.corrections`) holding the **whole**
TOC, transcribed by reading the rendered TOC pages directly. If `<stem>.index`
exists, the indexer ignores OCR/`--toc`/`--detect-toc`/`.corrections` and builds
the DB straight from it (still applying `--offset` for printed→scan):

```
pdftoppm -f 2 -l 6 -png -r 300 realbk2h.pdf toc   # render the TOC pages, read them
# ... write realbk2h.index ...
cargo run -- --pdf realbk2h.pdf --offset 7        # builds realbk2h.db from the sidecar
```

Like `.corrections`, it lives next to the book and is not in git.

## Known limitation (next step)

Printed→scan page mapping currently uses a single `--offset`, which is wrong as
soon as the scan has missing/extra pages — exactly why TuneCall left the global
master index. The robust fix (planned, behind `resolve_page` in `main.rs`) is to
OCR the printed page number off each scanned page and build a real printed→scan
map. OCR parsing of messy TOC layouts will also need iteration.

Titles are stripped of dot-leader noise, but character-level OCR misreads
remain (e.g. `PARIS`→`PARIG`, `ME`→`MB`). Fixing those would need fuzzy /
dictionary matching; substring search still finds most songs.
