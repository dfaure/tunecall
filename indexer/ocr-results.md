# Per-book indexing notes (transcription method, scan defects, TOC quirks)

Reference notes from the indexing sessions of 2026-06-10/11, for whenever a
book needs re-indexing or a lookup misbehaves. Each `<stem>.index` header in
the data dir carries the same facts in short form; this file is the long form.
Books indexed before these sessions (realbk*, nrealbk*, etc.) predate this log
and only have their offsets in `rebuild_db.sh`.

General workflow reminders live in CLAUDE.md ("Making/redoing a `<stem>.index`").
Decision guide in brief: try `pdftotext` first (some PDFs are digital or have a
prepended digital index); then try tesseract on typeset/typewritten indexes
(crop each column block, `--psm 6`); fall back to vision tiles when OCR garbles
numbers (dotted leaders, decorative fonts, pen marks). Always verify the offset
at start/middle/end by rendering pages and reading printed corner numbers.

## bestfbevr2 — The Best Fake Book Ever (2nd Ed), offset 1

- Pure scan. **Printed 400 missing from the scan AND scan 498 duplicates
  printed 496**, so the offset is +1, then 0 for printed 401–497, then +1.
  Fixed by shifting `.index` entries for printed 401–497 down by 1 (single
  offset kept). Spot-check around those boundaries after any rebuild.

## braziljrbk — Brazilian Jazz Real Book, offset 14, 167 songs

- **Digital PDF** (Overture/PrintToPDF) with a full text layer: whole TOC via
  `pdftotext -layout`; composer parentheticals stripped.
- Extraction artifact: "Você" lost its opening paren ("Você Roberto Menescal &
  Ronaldo Bôscoli)") and the strip regex missed it — fixed manually.
- Each chart restarts its own footer page numbering at 1, so the TOC numbers
  are positional: verify offsets by matching titles, never corner numbers.

## eurealbk — The European Real Book, offset 7, 180 songs

- Pure scan. **The A–F page of the alphabetical TOC is missing**: scans 4 and 5
  are two scans of the same F–M TOC page (double feed).
- A–F entries (printed 1–107) recovered by walking chart pages 8–117 reading
  title + printed corner number from top strips (labeled montage technique).
- Trailing `*` (mp3-available marker) stripped from titles. Some printed pages
  are photos (e.g. 93). Offset verified at 4 points.

## fjfakebk — Firehouse Jazz Band Commercial Dixieland Fake Book, offset 0, 532 songs

- Pure scan, 779 pages. **TOC numbers are SONG numbers, not page numbers**
  (songs span 1+ pages), so no constant offset can work: the `.index` holds
  **1-based scan pages** found by walking every page's top strip (song number
  in an oval + title) via labeled montages; `--offset 0`.
- tesseract could not read the oval-framed numbers even with preprocessing.
- **Songs 359, 404, 424, 445 absent from the scan** (numbers skip).
- **Songs 117 (Blues My Naughty Sweetie Gives to Me), 378 (Who'll Chop Your
  Suey?), 462 (I'm Gonna Stomp Mr. Henry Lee) lost their title page**; indexed
  at the surviving P.2 with "(p.2 only in scan)" in the title.
- Continuation pages are often bound BEFORE their title page (P.2 on the
  verso), so flipping back one page from a title page may show its P.2.
- Bonus entries: a second "The Memphis Blues" (1940 radio version, #71),
  "Ory's Creole Trombone (Trombone Part)" (#76A), "Sweethearts On Parade
  (Low Brass Part)" (#105A).

## jazzltd — Jazz LTD, offset 7, 520 songs

- Pure scan (Acrobat 4.0 Scan, 1999). Printed TOC on scans 2–7 read as 300-DPI
  half-column **vision tiles with ~100px vertical overlap** (no montage walk
  needed). Offset verified at 5 points (scans 8/108/208/308/400).
- TOC's "A Bid For" is actually **"A Bid For Sid"** on the chart (scan 8).
- **Multi-chart printed pages are real, not misreads**: printed 62 holds three
  charts (Cantelope Island, Footprints, Crepuscule With Nellie); printed 31
  holds Bird's Mother + "X" Stream (TOC "X-Stream...31" is correct); printed
  293 holds R.J. + Deltitnu. Render the page before "fixing" such numbers.

## wgfakebk — The World's Greatest Fake Book, offset 14, 207 songs

- Charts are scans, but the **PDF maker prepended a digital text-layer index**
  (scans 5–9, "not included in the original book"): extracted via `pdftotext`,
  composer parentheticals stripped, curly quotes normalized.
- Offset verified at scans 15/237/471 (printed 1/223/457).

## jfakebk — Jazz Fakebook, offset -1, 564 songs

- Typeset scan. **tesseract `--psm 1` on the full SONG INDEX pages (scans 3–8)
  worked nearly perfectly**; performer names after the em-dash stripped;
  two `~—` separator leftovers fixed in parsing.
- Charts start at scan 30 = printed 31. Multiple charts per printed page
  (e.g. 331 Serenade To A Bus Beat starts mid-page).

## ltrealbk — The Latin Real Book, offset 19, 177 songs

- Pure scan (Sher). OCR failed: **dotted leaders merge into the trailing page
  numbers** ("1"→"I", "31"→"3]"). Read by vision instead — single wide column,
  only 4 index pages (scans 4–7), cropped into overlapping thirds.
- Entries are **numbered 1–177, so completeness is guaranteed by the
  numbering**. Performer column dropped.
- Offset verified at scans 20/342/568 (printed 1/323/549).

## thebook — The Book, offset 11, 455 songs

- Typewritten fake book. OCR garbled the dot leaders (numbers unreliable);
  read by **vision tiles** (scans 3–7, column halves with overlap). Page scans
  3–7 are not horizontally aligned — measure/re-crop per page or numbers get
  cut off at the tile edge.
- **Printed 205–206 are missing from the scan** (scans jump 204→207): songs
  "Surrey With The Fringe On Top" and "The Love Boat Theme" are LOST (dropped
  from the `.index`; the TOC listed Love Boat twice — "Love Boat, The (Theme)"
  and "Theme From The Love Boat", both 206). To keep a single offset, `.index`
  entries for printed ≤204 are written as **printed+2** and `--offset 11`
  (true offset: 13 before the gap, 11 after). Verified at printed
  1/50/190/203/204/213/250/498.
- Transcription hazard caught during this run: a one-line eye-skew in the R
  section (Rhinestone Cowboy/Ring My Bell/Rise/Romulus/Rosetta all shifted by
  one) — caught by re-reading a zoomed crop. Duplicate-looking numbers are
  often real (printed 3 holds both "Fame" and "Dance: lo; Looks" per the
  categorical contents on scans 8–12, which is a useful cross-check).

## ulbdwyfb5e — The Ultimate Broadway Fake Book (5th Ed), offset 66, 603 songs

- Hal Leonard typeset scan, Index Of Songs on scans 3–5, **3 columns/page,
  number-first format**. OCR dropped too many leading numbers (240 orphan
  lines) — read by **vision half-column tiles** instead.
- Column x-positions shift between pages: measure per page (ink projection)
  or the numbers get cropped off.
- Charts start at scan 70 = printed 4 after a long "About The Shows" prose
  section (scans 16–68). Offset verified at scans 70/323/517.
- When several tiles are Read in one message the API may downscale them —
  re-read single tiles (or split in half) when digits look mushy.

## ulpoprock — The Ultimate Pop/Rock Fake Book (Joel Whitburn), offset 0, 399 songs

- Typeset scan; alphabetical listing on scans 4–13, one column per page
  (title block cropped, then tesseract `--psm 6`).
- **Reader pen checkmarks OCR as digits and corrupt page numbers.** All flagged
  entries vision-verified; real values: Mamma Told Me (Not To Come) **185**
  (OCR: 1485→"148"), New Moon On Monday **199** (1499), See You Later
  Alligator **258** (956), How Can You Mend A Broken Heart **121** (421),
  Hundred Pounds Of Clay **126** (426), Hungry Like The Wolf **126** (426).
- The first OCR pass **missed the last listing page (scan 13**, Waterloo→Your
  Song); added by vision. `|` misread for `I` in ten titles, fixed.
- Printed numbering includes the front matter → **offset 0** (verified at
  scans 24/185/334). The decade listing (printed 22–23) doubles as a
  cross-check for page numbers.

## ulfakebk — The Ultimate Fake Book (2nd Ed), offset 0, 1205 songs

- Hal Leonard typeset scan; alphabetical listing on scans 2–9, 3 columns/page,
  number-first. **Per-column tesseract `--psm 6`** with column boundaries
  measured by ink projection per page; whole-page `--psm 1` interleaves the
  column tops, don't use it.
- Cleanups applied: `|`/`!`/`1` misread as `I` ("Do! Hear A Waltz?",
  "If 1 Ruled The World", ...); 3 orphan lines with noise before the number;
  one exact duplicate line from tile overlap.
- Printed numbering includes the front matter → **offset 0** (verified at
  scans 18/400/771). Songs share printed pages and start mid-page (Satin Doll
  under San Francisco Bay Blues on 566; Younger Than Springtime under Young
  Blood's continuation on 771) — "duplicate" TOC numbers are normal here.
