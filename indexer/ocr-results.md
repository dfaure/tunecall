# Per-book indexing notes (transcription method, scan defects, TOC quirks)

Reference notes on how each book was indexed, for whenever a book needs
re-indexing or a lookup misbehaves. Each `<stem>.index` header in the data dir
carries the same facts in short form; this file is the long form. Books not yet
documented here only have their offsets in `rebuild_db.sh`; add a section as each
is (re)indexed.

General workflow reminders live in CLAUDE.md ("Making/redoing a `<stem>.index`").
Decision guide in brief: try `pdftotext` first (some PDFs are digital or have a
prepended digital index); then try tesseract on typeset/typewritten indexes
(crop each column block, `--psm 6`); fall back to vision tiles when OCR garbles
numbers (dotted leaders, decorative fonts, pen marks). Always verify the offset
at start/middle/end by rendering pages and reading printed corner numbers.

Tooling note (2026-06): a KDE-built `pdftoppm`/`pdfinfo`/`pdftotext` early in
`$PATH` (`/d/kde/inst/.../bin`) broke after a system poppler update — it links
`libpoppler.so.161` which no longer exists (only `.160`), so it exits 127 with a
"cannot open shared object file" error mid-run. Use the system binaries at
`/usr/bin/pdftoppm` etc. explicitly until the KDE build is rebuilt.

## bestfbevr2 — The Best Fake Book Ever (2nd Ed), offset 1, 1074 songs

- Pure scan (Hal Leonard), 862 pages; alphabetical listing on scans 4–11
  (printed 3–10), 3 columns/page, read as 300-DPI vision tiles.
- **Printed 400 missing from the scan AND scan 498 duplicates printed 496**,
  so the offset is +1, then 0 for printed 401–497, then +1. Fixed by shifting
  `.index` entries for printed 401–497 down by 1 (single offset kept).
  Spot-check around those boundaries after any rebuild.
- Jump Shout Boogie's first page IS the missing printed 400; its entry
  (still 400 in the `.index`) lands on scan 401 where the song continues.
- Multi-song printed pages are real, not misreads: 828 holds three charts
  (Woodchopper's Ball/Woody Woodpecker/Wooly Bully), 514 two different songs
  both titled "My Love", 277 and 374 two each; the two "Jesse" (393/396) and
  "Superstar" (690/692) entries are distinct songs.
- Offset verified at scans 24/278/375/408/491/497–499/601/704/829/861.

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

## realjazzbk — The Hal Leonard Real Jazz Book, offset -2, 528 songs

- Pure scan, 386 pages. Alphabetical Contents on scans 1–3, 3 columns/page,
  read as 600-DPI half-column vision tiles. Scan 4 is a separate
  Composer/Lyricist index (ignored).
- Pages 2–3 have **no title header**, so the column tops start ~400px higher
  than on page 1: a page-1-sized crop silently clips the first 1–2 entries of
  every column — read a separate thin top strip per column to recover them.
- **The scan duplicates two page-pairs**: printed 323–324 appear twice (scans
  321–324) and printed 333–334 twice (scans 333–336), shifting the mapping by
  +2 at each pair. To keep a single offset, `.index` entries are written
  ADJUSTED: +2 for true printed 325–334, +4 for true printed ≥335 (true
  offsets: -2 / 0 / +2). The duplicated pages map to their first copy.
- Out-of-sequence TOC numbers are real, not misreads: Days of Wine and Roses
  226 (in the D's, sharing the page with Mr. Big Falls His J.G. Hand), Laura
  234, You Gotta Pay the Band 257 — all render-verified.
- Drift boundaries pinned by labeled top-strip montages of scans 312–384;
  offset verified at scans 16/17 (-2 front), 298/311 (-2 still), 321–336
  (the two duplicate pairs), 346/378 (+2), final checks at scans 321/337/384.

## rwlpfbk — Richard Wolfe's Legit Professional Fake Book, offset -1, 1016 songs

- **Clean digitally-typeset TOC** (charts are scans, but the alphabetical
  contents on scans 2–8 are a crisp modern render, not degraded). 2 columns/page,
  **page-number-first** format `<printed>...<TITLE>......<genre>`; genre column
  dropped. Cover declares "More Than 1010 Songs"; 1016 entries.
- Read as **300-DPI native column-half tiles** (~1160×1700). Do NOT use 600-DPI
  here: once several images are in context the Read tool caps images at ~2000px,
  so 600-DPI half-columns get rejected and downscaling them blurs the small bold
  page numbers — that produced real misreads (120→126, 96→206, ~one per few rows)
  until the switch to 300 DPI.
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages, and 380–477 are themed Christmas/Classical/Broadway/Latin appendix runs
  interleaved into the master index) — can't infer a number from sequence, read
  each. A few appendix-genre songs sit in the main body (e.g. "A Holly Jolly
  Christmas" is genuinely printed 186, sharing the page with "For Me And My Gal").
- **Multi-song printed pages are the norm (~2.2 songs/page), not misreads**:
  printed 96 holds four (Farewell My Lily Dear / The Farmer In The Dell /
  Fascination / I Hear A Dream). Distinct songs share a title: "The Man On The
  Flying Trapeze" (124 & 229), "Maybe" (231 & 242), "Joy To The World" (189 &
  395) — all kept.
- Offset -1 (printed 10 == 1-based scan 9). Verified at scans
  9/44/57/77/95/185/382/409 (printed 10/45/58/78/96/186/383/410); no clamps
  (max printed 476 → scan 475, book has 479 scans).

## classicfb — The Real Little Classical Fake Book, offset 0, 627 songs

- Pure scan (Acrobat 4.0 Scan, 2001), 412 pages. **The book has TWO indexes**:
  *Alphabetical Contents By Composer* (printed 2, composer-grouped) and
  *Alphabetical Contents By Title* (printed/scan 8–17). **Index the by-title
  one** — the by-composer index would duplicate every song. The cover (scan 1)
  lists the layout and confirms the spans: by-composer p2, by-title p8, *Time
  Line Of Major Classical Composers* p18, *Categorical Listing* p406.
- Title index is **number-first** (`<printed> <TITLE in caps>` then the composer
  on a subline below); composer subline dropped. Read as 600-DPI half-column
  vision tiles.
- **Offset 0: printed page == 1-based scan page** (front cover is scan 1 and the
  printed numbering coincides). Verified at scans 88/94/210/303/316/351 (When
  Jesus Wept, Habañera + Minuet, Zampa Overture, William Tell Overture, The Wild
  Horseman, Zueignung) and the corner numbers at 88/94/211/352. No clamps
  (max printed 404 → scan 404).
- Page numbers are **not monotonic with alphabetical order** (classical pieces
  placed to fit pages) and **multi-song printed pages are normal** (e.g. printed
  94 holds Habañera + Minuet) — read each, don't infer from sequence.
- Misread hazard: at low DPI "Waltz In **B** Major Op. 39, No. 1" (Brahms) looks
  like "Bb Major" — the half-column tile disambiguates. The error-prone part is
  always the small numbers/accidentals, not the titles.

## cpomnibook — Charlie Parker Omnibook, offset 0, 58 songs

- **Clean printed music, not a degraded scan** (143 pages, treble-clef / C
  instruments edition). **No table-of-contents page exists**: the only back
  matter is a Jamey Aebersold Scale Syllabus (scan 143). Titles were read off
  the top of each solo's first page via labeled top-strip montages — the cpomni-
  book method for any book without a usable index.
- The title sits top-center of a song's first page (with "By Charlie Parker"
  under it); continuation pages are music-only or "<title> - cont.". Offset 0
  (printed/scan coincide; the chart on scan N is render page N-1).
- **"Leap Frog" starts mid-page on scan 130**, below the end of "Shawnuff", so a
  top-strip-only pass misses it — caught only by a **full-page verification pass
  over all 143 pages** (4-per-row montages). Watch for mid-page starts wherever a
  song's title is the only thing not at a page top.
- Numbered variants kept distinct: Au Privave / She Rote / Mohawk / Kim / Now's
  The Time each carry (No. 1) and (No. 2). "Shawnuff" left as printed (not the
  usual "Shaw 'Nuff").
- Offset verified by rendering scans 1/26/54/128/130/142 (Confirmation, Au
  Privave No.2, Kim No.2, Shawnuff, the mid-page Leap Frog, Ballade).

## cufakebk — Cuban Fake Book, Vol. 1, offset 8, 122 songs

- Pure scan, 195 pages. Two-page **"General Index"** (scans 6–7: title /
  composer / printed page), very legible, read at moderate DPI. A preface (scan
  3) and a *General Discography* (scan 5, no page numbers — not a usable index)
  precede it. First chart Alardoso is printed 2 = scan 10 → **offset 8** (scans
  8–9 are a blank + a decorative illustration).
- **The book's own index has real errors, corrected against the pages** (read the
  printed corner numbers): Dos Gardenias..La engañadora were each listed one too
  high (fixed to 55–59), **"Ogguere" (printed 120) was omitted entirely** and is
  added, and "¡Oh vida!" is printed 122 not the index's 120. The index drifts +1
  then re-syncs — verify any region where it jumps.
- Spanish leading articles de-inverted ("bodeguero, El" → "El bodeguero") to
  match the chart headings; the two "La Bayamesa" disambiguated by composer;
  entry 61's "(M & A)" sorting annotation dropped (title is "Mujer ardiente").
- **Top-strip title reads can shift ±1 near 2-page songs** (Te Odio is scan 168
  not 167; La Tarde runs 166–167) — cross-check suspicious entries against full
  pages + corner numbers, don't trust the band alone. Offset 8 verified at scans
  10/28/62–66/90/106/138/152/155/193/195.

## realrockb1 — Real Rock Book, offset 0, 148 songs

- Pure scan, 231 pages. Clean printed **"TITLE/Recordings ... Page"** index on
  scans 3–5 (2 columns/page), after an "INTRO" preface (scan 2, "~150 songs").
  Music starts scan 6 = Abracadabra (printed 6) → **offset 0** (printed == scan).
- The page-number column is the error-prone part. Each entry has multi-line
  italic *Recordings* sub-text under the bold title, so the **right-aligned page
  number can visually align with the wrong title** at low DPI. Read each index
  page **column-by-column at 220 DPI and zoom the number column** — this caught
  Long Train Running 108 vs Love Hurts 216, and White Room 230 vs A Whiter Shade
  Of Pale 220 (both initially misaligned).
- **Pages 209–230 are a later/appendix section**, so the index page numbers are
  deliberately **non-monotonic** (each title still maps to its own page — don't
  "fix" them). English leading articles de-inverted ("Letter, the" → "The
  Letter").
- Offset verified at scans 6/68/162/230, plus spot-checks of close-numbered
  entries (Statesboro Blues 161, Signed Sealed Delivered 157, Whole Lotta Shakin'
  198, Why Can't This Be Love 204, Tush 178, My Generation 209).

## realrockb2 — Real Rock Book 2 (K G Johansson / Hans Hjortek, Warner/Chappell 2001), offset 0, 127 songs

- Pure scan, 246 pages. Same series/layout as realrockb1 (bold title + italic
  *Recordings* sub-text, "TITLES / Examples of famous recordings" header) but the
  index is **3 columns/page**, on scans 2–3, after an "INTRO" preface (scan 4).
  Music starts scan 5 = Born To Be Wild (printed 5) → **offset 0** (printed ==
  1-based scan). Read as 600-DPI per-column tiles, each split top/bottom.
- **Main hazard: the scan is skewed.** The column separator lines and the
  right-aligned page numbers tilt rightward toward the top of each page, so a
  straight per-column crop **clips the top entries' page numbers** while the
  bottom ones read fine. Fix with `magick -deskew 40%` before cropping, or extend
  the crop's right margin ~150px; verify by re-reading the clipped rows.
- Columns 2–3 have no header band, so their first entry sits ~400px above the
  page-1 column top — read a thin top strip to recover it (Bye Bye Love 46 tops
  col 2; Handle With Care 82 tops col 3).
- Page numbers are **non-monotonic**: a back/appendix run carries high printed
  numbers (~204–242) interleaved alphabetically — I Get Around 242, Rebel Rebel
  240, Nutbush City Limits 238, Life on Mars 236, Let's Dance (Bowie) 234, Joker
  222, Imagine 218. Don't "fix" them; render-verified.
- **Two distinct "Let's Dance"** kept (Chris Montez 128 / David Bowie 234).
- "Bye Bye Love" (46) lists Simon & Garfunkel's *Bridge Over Troubled Water* as a
  *recording* under it — that wrapping album line is NOT a separate "Bridge Over
  Troubled Water" song; don't add one.
- Offset 0 verified at scans 5/6/100/242 (Born To Be Wild / 2-4-6-8 Motorway /
  Hooked On a Feeling / I Get Around). Built clean, 127 entries, no clamp
  warnings (max printed 242 → scan 242, book has 246 scans).

## strealbk — The Standards Real Book (C Version, Sher Music, 2000), offset 13, 266 songs

- Pure scan, 574 pages. Alphabetical index on scans 4–8 (dot-leader format), but
  **its page numbers are unreliable**: the TOC under-numbers because the book
  interleaves play-along pages it doesn't count, so reading the leaders drifted by
  −2 across a stretch (e.g. Lullaby Of Broadway read 291, actual 293). Abandoned
  the TOC numbers and **derived every page from the chart itself** — rendered each
  tune's title page and read its corner number, with the constant `printed = scan
  − 13` (offset 13: printed 1 = scan 14).
- **Bass-clef and "(Rhythm Section)" play-along versions are intentionally
  omitted** to match the book's lead-sheet alphabetical index; these alternates
  sit between the lead sheets and are the source of most page-number gaps
  (omitted printed 35/119/219/285/303/351/355/403/431/471/507/511).
- A few tunes are **4-page charts** whose title repeats on the second spread
  (Sabiá 371, Takin' It To The Streets 437, What A Fool Believes 509) — single
  entries, not duplicates; their continuation pages (printed 373/439/513) also
  create gaps.
- **Genuine arrangement variants the index lists twice are kept**, disambiguated:
  All Of You (Standard / Bill Evans Versions, 15/17), But Not For Me (Standard /
  Coltrane Versions, 73/75).
- Montage hazard: the title-page-on-even-scan parity wobbles around multi-page
  play-along charts (a title-less continuation lands on an even scan), which
  mis-paired titles with strip labels until I re-montaged ALL pages of both
  parities with per-strip scan labels. Verify any region near a multi-page chart.
- Offset 13 verified at ~14 anchors spanning the book (scans
  14/46/50/120/134/234/306–312/348/470/530/574 = printed
  1/33/37/107/121/221/293–299/335/457/517/561); built clean, 266 entries, no
  clamp warnings.
- Naming note: the lead sheet prints "Cotton Tail" (with a space, printed 91), so
  a search for "cottontail" won't match — add an alias if desired.

## lmjazz — Library Of Musicians' Jazz, offset 4, 325 songs

- Pure scan (Acrobat 4.0 Scan, 1999/2001), 217 pages. Pink cover (scan 1) reads
  "Library Of Musicians' Jazz" with "Indice Generale" at the bottom (Italian for
  general index) — typewritten bebop/cool-jazz lead sheets.
- Alphabetical **"INDEX TO SONGS"** on scans 2–3 (A–V, **3 typeset columns/page**,
  right-aligned page numbers, no dot leaders) plus **W and Y on scan 4 in a
  different typewriter font with dot leaders** (a re-typed tail page; its numbers
  continue the same sequence). There is NO U section. Read as 400-DPI 3-column
  vision tiles; scan 4 is legible even at 150 DPI.
- **Offset 4: printed 1 == 1-based scan 5** (cover scan 1, index scans 2–4). The
  scan is clean with a constant offset — no missing/duplicated pages. Verified at
  scans 5/150/213 (printed 1/146/209 = Algo Bueno+Au Privave / St. Thomas+Valse
  Hot / Sweet Clifford+The Fat Man). No clamps (max printed 212 → scan 216, book
  has 217 scans).
- Page numbers are **not monotonic with alphabetical order** (songs placed to fit
  pages) and **multi-song printed pages are normal, not misreads**: printed 1
  holds Algo Bueno + Au Privave; 146 St. Thomas + Valse Hot; 189 Midgets +
  Miss Jackies Delight + Misterioso; 209 Sweet Clifford + The Fat Man; 212
  Duke, The + Pent-Up House; 111 Local Blues + Local 802 Blues. Render the page
  before "fixing" a shared number.
- Obvious scan misspellings normalized to the standard titles for searchability
  ("A Night In Tunesia"→A Night In Tunisia, "Daa Houd"→Dahoud, "D'Jango"→Django);
  intentional-looking oddities left as printed (Bockhanal, Bisquit Mix, Quasimado).
  Built clean, 325 entries, no clamp warnings.

## dhpccs100t — Dick Hyman's Professional Chord Changes and Substitutions for 100 Tunes Every Musician Should Know (Ekay Music, 1986), offset 0, 100 songs

- Pure scan (Acrobat 4.0 Scan, 2001), 127 pages — single-line lead sheets with
  Dick Hyman's reharmonizations of jazz standards.
- **This book has NO page-numbered TOC and NO printed page numbers anywhere on
  the charts** (pages carry only hole-punch marks). Scan 4 is an alphabetical
  *"Songs Included In This Volume"* list — 100 titles, two columns, but **no page
  numbers**. So no constant offset can be read from a TOC; the `.index` holds
  **1-based scan pages** with `--offset 0` (resolve_page = printed + 0 − 1, so
  scan 8 → render page 7).
- Method (the cpomnibook/fjfakebk "no usable index" approach): songs run strictly
  in the page-4 alphabetical order, one or two scan-pages each, starting at
  **scan 8 = Ain't She Sweet**. Walked every content page (scans 8–127) via
  labeled 4×3 montages at 80 DPI, reading the large top-center title and treating
  any **title-less, music-at-top page as a continuation** of the previous song
  (its start page is what gets indexed). The page-4 list was the ground-truth
  checklist for completeness and ordering.
- Front matter to skip: scan 1 cover photo, 2 title, 3 *Chart Of Chord Symbols*,
  4 songs-included list, 5 *Preface*, 6 *About The Author*, and **scan 7 is a
  "Fake Books - CD II" master-index splash inserted by the digital compilation**
  (not part of the original book). First real chart is scan 8.
- **20 two-page songs shift the numbering** — their start pages: April In Paris
  13, Autumn In New York 17, Begin The Beguine 20, Blues In The Night 23, How
  Long Has This Been Going On? 43, I Get A Kick Out Of You 47, I Got Rhythm 49,
  Just One Of Those Things 65, Love For Sale 69, Lover Come Back To Me! 71,
  Lullaby Of Birdland 73, Poor Butterfly 82, 'Round Midnight 84, Satin Doll 87,
  The Summer Knows 99, A Time For Love 107, You Go To My Head 118, You Make Me
  Feel So Young 120, You're Getting To Be A Habit With Me 123, You're The Top 125.
- **Completeness cross-check (arithmetic):** 80 one-page + 20 two-page songs = 120
  pages = scans 8–127 exactly, and the last song (Yours Is My Heart Alone) lands
  on the final scan 127. The page count closes perfectly, which is the strongest
  guarantee here given there are no numbers to verify against.
- Titles taken from the page-4 master list (canonical, cleaner for search).
  Subtitles kept only where that list shows them: *If I Could Be With You (One
  Hour Tonight)*, *The Summer Knows (Theme From Summer Of '42)*. Several chart
  pages carry subtitles the list omits (Blues In The Night "(My Mama Done Tol'
  Me)", Fools Rush In "(Where Angels Fear To Tread)", Liza "(All The Clouds'll
  Roll Away)", I May Be Wrong "(But I Think You're Wonderful)") — followed the
  list and dropped them.
- Offset 0 verified by DB query (Ain't She Sweet → page 7, Yours Is My Heart
  Alone → 126); built clean, 100 entries, no clamp warnings (max printed 127 →
  render 126, book has 127 pages). pdftoppm emits harmless "Syntax Warning: Bad
  annotation destination" lines on this PDF — ignore them.

## rjstandfbk — The Hal Leonard Real Jazz Standards Fake Book (C Edition), offset 1, 246 songs

- Scanned charts, but the **TOC is a clean printed *Contents*** (not handwritten/
  degraded), on scans 4–6, 2 columns/page, **number-first** (`<printed> <Title>`),
  alphabetical A–Y. Legible enough to read straight from 100-DPI page renders — no
  half-column tiling or montage walk needed. Cover (scan 2) says "Over 240 Songs!";
  246 entries.
- **Offset 1: printed 11 == 1-based scan 12** (cover + foreword + 3 contents pages
  precede the first chart). Verified end-to-end: scan 13 shows printed "12" with
  footer "ADIOS – 2" (so Adios's p.1 = printed 11 = scan 12), and scan 559 = printed
  558 = "You're Driving Me Crazy!" (the last TOC entry). Clean scan, single constant
  offset, no missing/duplicated pages. No clamps (max printed 558 → scan 559, book
  has 562 scans).
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence. Examples where the number jumps
  back within an alpha run: All or Nothing at All 17 (after All of You 24),
  Everything Happens to Me 75, Haunted Heart 135, I Get Along Without You Very Well
  183, Lazybones 283, Lover 315, My Favorite Things 337, A Nightingale Sang in
  Berkeley Square 367, Poor Butterfly 417, Sir Duke 425, St. Louis Blues 471,
  Witchcraft 541, You'd Be So Nice to Come Home To 556, You're Blasé 553.
- **Scan page 11 is a separate Composer index** (composer-grouped, e.g. "Eddie
  Woods → 11 Adios") — ignored; only the by-title *Contents* is indexed.
- Built clean, 246 entries, no clamp warnings.

## juststanrb — Just Standards Real Book (Warner Bros, 2001), offset 0, 250 songs

- Pure scan, 404 pages. Cover (scan 1) "JUST *Standards* REAL BOOK". The book has
  **two indexes**: an alphabetical-by-title *Contents* (scans 4–5, 2 columns/page)
  and a *Composer Index* (scans 6–8, composer-grouped). **Index the by-title one**;
  the composer index would duplicate every song. A "How To Use This Fakebook"
  prose section sits on scans 2–3.
- Title index is **dot-leader, title-first** (`<TITLE> ...... <printed>`). Read as
  600-DPI column-half vision tiles (split each page into L/R columns, then top/mid/
  bottom thirds) — the small bold right-margin numbers are the error-prone part.
- **Offset 0: printed page == 1-based scan page.** Verified directly: scan 9 =
  printed 9 = AIN'T MISBEHAVIN', scan 10 = printed 10 = AFTER YOU. Songs run printed
  9–395; appendices (Discography…) start printed 396, so scans 396–404 are back
  matter. No clamps (max printed 395 → scan 395).
- **TOC typo on A FINE ROMANCE**: the *Contents* misprints its page as "10" (which
  is AFTER YOU's page — impossible, each chart starts its own page). Its real page
  is **21**, render-confirmed; the `.index` carries the corrected 21 with a comment.
  This was chased down because 21 collides with nothing and pages 108–120 (its
  alphabetical neighbors) were all accounted for.
- Page numbers are **not monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence (e.g. THE BEST THINGS IN LIFE ARE
  FREE 47 sits among the 50s; FIVE FOOT TWO 115 precedes FALLING IN LOVE 116).
  No multi-song pages observed; most charts are 2 pages, several are 1 (EMILY 110,
  EVERYTHING MUST CHANGE 114, FIVE FEET TWO 115).
- **Scan page 111 is a divider photo** (a guitarist), not a song — it falls between
  EMILY (110, 1 page) and EVERGREEN (112–113); don't read a title off it.
- TOC misspelling normalized: "SOFTLY, AS IN A MORNNG SUNRISE" → "...MORNING...".
- Built clean, 250 entries, no clamp warnings; verified by DB query (AFTER YOU →
  page 9, A FINE ROMANCE → 20, YOU STEPPED OUT OF A DREAM → 394).

## bjazz50 — Jazz Bible Fake Book: 200 of the Best Songs from Jazz of the '50s (Rob DuBoff / Hal Leonard), offset 0, 200 songs

- Pure scan, 229 pages. Clean printed **Song Index** on scans 2–5 (printed 2–5),
  **one wide row per entry: TITLE / artist / printed-page**, alphabetical A–Y.
  Title page is scan 1 (also lists "Song Index ... 2" and "Artist Index ... 226").
- Method: rendered the index at 300 DPI, then for each page built a **composite
  crop that pastes the title column next to the page-number column (dropping the
  middle artist column)** and split it top/bottom. This keeps each title
  row-aligned with its page number in one image — avoids the misalignment risk of
  reading titles and numbers from two separate crops, which is the error-prone
  part. The right-margin page numbers are illegible at 100 DPI.
- **Offset 0: printed page == 1-based scan page** (Adios printed 11 is on 1-based
  scan 11, render page 10). First guessed −1 and it built off-by-one (Adios landed
  at render 9, not the verified 10) — caught because `resolve_page = printed +
  offset − 1`, so matching scan==printed needs offset 0, not −1.
- Page numbers are **not monotonic with alphabetical order** (songs placed to fit
  pages) and the index has small **out-of-order pairs**, not misreads: ANY PLACE I
  HANG MY HAT IS HOME 22 before ANYTHING YOU CAN DO 21; BEYOND THE SEA 30 before
  BLUE ORCHIDS 29; YOU'RE JUST IN LOVE 222 / YOU'RE SENSATIONAL 221 / YOU'VE
  CHANGED 224 sit among the 216–222 tail. Number gaps (e.g. no 23/31 in the A–B
  run) are normal — read each, don't infer from sequence.
- The **Artist Index (printed 226)** is a separate composer-grouped listing —
  ignored; only the by-title Song Index is indexed.
- Titles kept in the index's ALL-CAPS form (search is case-insensitive); "A LITTLE
  STREET WHERE OLD FRIENDS MEET" had its tail clipped by a tile edge — completed
  from the standard title.
- Offset 0 verified by rendering scans 11/133 (printed corner numbers on Adios /
  Misty) plus DB query (Adios → 10, Misty → 132, You've Changed → 223). Built
  clean, 200 entries, no clamp warnings (max printed 224 → render 223, book has
  229 pages).

## tnbobbook — The New Bob Book (Bob Roetker, Rev 6), offset 7, 666 songs

- **Digital PDF with a full text layer** (whole index via `pdftotext`), 691 pages.
  Clean **dot-leader index** ("The New Bob Book Index", `<TITLE> ...... <printed>`)
  on scans 3–7. Located it by counting dot-leader lines per page (0 on scans 1–2
  and 8+, 150-ish on 3–6, 54 on 7). One song per printed page, so titles never
  wrap; parsed with `perl` `(.+?)\s*\.{2,}\s*(\d+)` per line.
- **Offset 7: printed 1 == 1-based scan 8** (cover scan 1, the author's intro/
  preface scan 2, index scans 3–7). Verified end-to-end via `pdftotext`: scan 8 =
  "500 Miles High" (printed 1), scan 9 = "After You've Gone" (printed 2), scan 691
  = the last entry "Zingara" — whose chart is actually titled **"Zingaro"** (the
  index uses the alternate name). Clean scan, single constant offset, no missing/
  duplicated pages. No clamps (max printed 684 → scan 691 == page count).
- Printed pages run 1–684 but only **666 entries** — the ~18 gaps are 2-page tunes
  (e.g. "Armando's Rhumba" 26 then next entry 28). Page numbers are monotonic with
  alphabetical order here (unusual; the book is purely alphabetical, one chart per
  page). Curly apostrophes (’) kept as printed for searchability fidelity.

## gridjazz — Anthologie des Grilles de Jazz (French jazz grid anthology), offset 1, 1662 songs

- **Digital PDF with a full text layer**, 488 pages. Very dense **dot-leader index**
  (ALL-CAPS titles, French) on scans 1–7, **multiple songs per printed page** (the
  charts are one-line grids, so a printed page holds several tunes). Whole index
  via `pdftotext`; 1662 entries.
- **Offset 1: printed 7 == 1-based scan 8** — the printed page numbering starts at
  7 on the first chart page (front matter/index unnumbered), so the index's first
  entries are "...... 7". Verified at scans 14/100/300/488 (printed corner numbers
  13/99/299/487) and that printed 487 → scan 488 == page count. Strictly constant
  offset, no missing/duplicated pages; page numbers come out **strictly
  non-decreasing** (exact text layer, not OCR, so the numbers are trustworthy).
- **Two parsing wrinkles** (handle on any re-extract):
  - Long titles **wrap across two text lines** (the page number sits on the second
    line, e.g. `AGGRAVATIN' PAPA (DON'T TRY TO TWO-TIME / ME)......8`). Fixed by
    collapsing all whitespace incl. newlines into one blob, then globally matching
    `(.+?)\s*\.{2,}\s*(\d+)` — the non-greedy title spans the wrap.
  - Each index page carries a **stray standalone single-digit line** (0,1,2,3,4,5,6
    across the 7 pages — a column/section marker) that, after collapsing, leaks
    onto the front of the next title (gave "0 BOHEMIA AFTER DARK"). Dropped pure-
    digit lines (`grep -vP '^\s*\d+\s*$'`) before parsing.
- The repeated **"ANTHOLOGIE DES GRILLES DE JAZZ" page header** is stripped.
  **"OW" (printed 318) is a real tune** (Dizzy Gillespie's "Ow!"), not a parse
  fragment — left as-is. Built clean, 1662 entries, no clamp warnings.

## tpdxmasjfb — The Public Domain Christmas Jazz Fakebook (Stephen Cox / FreeMusicEd.org), offset 6, 23 songs

- **Digital MuseScore PDF with a full text layer** (not a scan), 117 pages. Whole
  TOC via `pdftotext -layout`; no tiling/OCR needed.
- The book holds **four transposed copies of the same 23 carols** — C
  (printed 1–27), Bb (29–55), Eb (57–83), Bass Clef (85–111). Per the C-only
  convention of every other book here, **only the C Instruments section is
  indexed**; the Bb/Eb/Bass Clef runs are intentionally omitted (same titles,
  would just quadruple the results).
- **Offset 6: printed 1 == 1-based scan 7.** The catch: the **TOC spans 3 pages**
  (scans 4–6), not 2. `pdftotext -f5 -l8 -layout` concatenates the tail of the
  TOC with the first song's text, which made it look like Angels (printed 1) was
  on scan 5 → a wrong offset 4 that built clean but mis-mapped (Silent Night
  landed on O Little Town's page). A **per-page `pdftotext` walk of scans 5–32**
  showed Angels actually starts on scan 7 → offset 6. Lesson: with `-layout`
  over a range, confirm the *page* a title sits on by dumping that page alone.
- Three carols are **2-page spreads** (O Come All Ye Faithful printed 15–16,
  O Holy Night 17–18, Twelve Days of Christmas 23–24), but the printed
  numbering already accounts for them, so a **single constant offset 6 holds
  end-to-end** — no drift. Verified at scans 7/27/31/33 and by DB query
  (Silent Night → render 26 = page 27, O Holy Night → 22 = page 23,
  What Child is This? → 32 = page 33, Carol of the Bells → 9 = page 10).
- **Kept the TOC spellings** even where the chart pages differ: "O Holy Night"
  (chart: "Oh Holy Night"), "Holly and the Ivy, The" (chart: "The Holly and the
  Ivy"), "Away In a Manger" (chart: "Away in a Manger"), "Twelve Days of
  Christmas, The" (chart drops the trailing ", The"), "O Tannenbaum" (chart adds
  "(O Christmas Tree)"). The TOC is the canonical title list.
- Built clean, 23 entries, no clamp warnings (max printed 27 → render 32, C
  section ends well before the Bb run).

## creolejbfb — The Creole Jazz Band Fake Book 1 (Pre-1923, C Treble; Kevin Yeates), offset 6, 172 songs

- **Born-digital PDF, not a degraded scan** (267 pages, clean vector charts in a
  decorative hand-lettered all-caps font). Alphabetical TOC on scans 4–6, 2
  columns/page (scan 6 is the T–Y tail, a single short column).
- Read by **300-DPI vision tiles** (each TOC page cropped into per-column bands,
  then top/middle/bottom). **But `pdftotext -layout` extracts the whole TOC
  cleanly** — the decorative font still carries a real text layer. Vision was the
  slower path here; on a re-index, try `pdftotext -f4 -l6 -layout` first (the
  text layer agrees with the vision reads, e.g. it confirms "And They Called It
  Dixieland 19", which looked like 21 at low DPI — the small bold dot-leader
  numbers are the error-prone part, the titles are reliable).
- **Offset 6: printed 1 == 1-based scan 7** (scan 1 cover, 2 logo-credit, 3
  preface/versions, 4–6 TOC; first chart scan 7 = "12th Street Rag" with printed
  corner "1"). Clean digital pages, single constant offset, no missing/duplicated
  pages. Verified by rendering scan 7 and by DB query (12th Street Rag → render 6,
  You've Got To See Your Mama Ev'ry Night → 265). No clamps (max printed 260 →
  scan 266, book has 267 pages).
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence: After The Ball Is Over 6 (between
  Afghanistan 3 and After You've Gone 4), Alabama Jubilee 18, China Boy 48 before
  Chinatown 46, If You Were The Only Girl In The World 110 before Indiana 108,
  Japanese Sandman 118.
- **Cross-reference kept as two entries**: "Sister Kate" and "I Wish I Could
  Shimmy Like My Sister Kate" both point to printed 104 (render 109) — the TOC
  lists the song under both names; both kept.
- **TOC typo corrected**: the TOC prints "212" for Storyville Blues, colliding
  with Suez (also 212); its real page is **210** (corrected in the `.index`).
- Kept the book's own spellings even where they look off: "Chesapeke" (Sailing
  Down Chesapeke Bay), "Tain't Nothin Else But Jazz", "Eh La Bas", "Rufe Johnsons'
  Harmony Band". Normalized the decorative-font glitch "A'int We Got Fun" → "Ain't
  We Got Fun" and used straight apostrophes throughout.
- Built clean, 172 entries, no clamp warnings.

## rdixieland — The Real Dixieland Book (C Instruments, Revised Edition; arr. Robert Rawlins, Hal Leonard), offset 0, 249 songs

- Scanned charts, but the **alphabetical TOC is a clean typeset render** on scans
  4–8 (5 pages, 2 columns/page, dot-leader title-first `<TITLE> ...... <printed>`,
  A–Y). Front matter: scan 1 cover ("THE REAL DIXIELAND BOOK", C Instruments,
  Revised Edition), 2–3 a Robert Rawlins essay, 4–8 TOC.
- **Offset 0: printed page == 1-based scan page** (the printed numbering counts the
  front matter — first song is printed 9, last is printed 378 == page count).
  Verified by rendering scans 9/10 = printed corner "9"/"10" = After I Say I'm
  Sorry / Ace In The Hole, and by DB query (After I Say I'm Sorry → render 8,
  You're The Cream In My Coffee → 377). Clean scan, single constant offset, no
  missing/duplicated pages. No clamps (max printed 378 → render 377).
- Read as **600-DPI left/right column-half crops** (`-crop 2550x6600+0+0` and
  `+2550+0` on the 5100×6600 page). The TOC is legible enough that whole-page
  Reads mostly worked, but the small bold dot-leader **page numbers are the
  error-prone part** — a first full-page pass misread several, all fixed against
  the column crops: **A Cottage For Sale 85** (not 93), **The Curse Of An Aching
  Heart 95** (not 96), **Honeysuckle Rose 148 / Hotter Than That 146** (swapped),
  **Muskrat Ramble 230** (looked like "Meekhat"). Titles were reliable.
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence: As Long As I Live 27 (among the
  A's that run to 36), I Ain't Gonna Give Nobody None O' This Jelly Roll 147 (in
  the middle of the I's), Margie 217 before Mandy Make Up Your Mind 218, Saint
  James Infirmary 291 (in the S's that run 294+), You Always Hurt The One You Love
  371 (before Yellow Dog Blues 374), Undecided 344 before Washington And Lee Swing
  343, My Monday Date 237 / My Honey's Loving Arms 238 (swapped order).
- Naming notes: **"Tuck Me To Sleep In My Old 'Tucky Home"** — the TOC line-wraps
  as "...MY OLD / TUCKY HOME"; kept the 'Tucky ('Tucky = Kentucky) apostrophe
  form. "Bei Mir Bist Du Schön (Means That You're Grand)" kept the umlaut.
  Several parenthetical alternates preserved from the TOC (e.g. Chicago Breakdown
  (Stratford Hunch), I've Found A New Baby (I Found A New Baby), Twelfth Street Rag
  (Dixieland Version) as a distinct entry from the plain Twelfth Street Rag).
- Built clean, 249 entries, no clamp warnings.

## befakebk — The Bill Evans Fake Book, offset 3, 67 songs

- Pure scan, 106 pages. **No page-numbered TOC** — the charts are printed
  alphabetically by title, so titles were read off each chart's first page (the
  cpomnibook/dhpccs100t "no usable index" method), and the printed corner number
  on that page is the index entry.
- **Offset 3: printed 2 == 1-based scan 5** (scans 1–4 are cover/front matter;
  the first chart "B Minor Waltz (For Ellaine)" is printed 2 on scan 5). Verified
  at the start and bottom of the book (max printed 100 → render scan 103, book has
  106 pages, no clamp warnings).
- **Alphabetical order is only approximate** — printed page is authoritative,
  read each: e.g. printed 10 Catch The Wind, 11 Chromatic Tune, 12 Children's
  Play Song (Chromatic before Children's). Don't infer a number from the
  alphabetical sequence.
- **Distinct arrangements at adjacent pages are kept as separate rows, not
  dropped** (the indexer only drops exact title+page duplicates): Only Child
  48/49, Time Remembered 76/77, Turn Out the Stars 78/80, T.T.T.T. region, The
  Two Lonely People 84/86, Very Early 88/89, Waltz For Debby 91/94 — these are
  the lyric vs. lead-sheet (or two-piano) versions, so a search surfaces both.
- Vision-read hazard recap: the small **bold corner page numbers** are the
  error-prone part (titles are reliable); a few looked duplicated/out of order
  and were chased down by rendering the page and reading its corner number.
- Built clean, 67 entries, no clamp warnings.

## realxmasbk — The Real Christmas Book (C Instruments, Hal Leonard), offset 0, 150 songs

- Pure scan, 216 pages. Cover (scan 1) "C INSTRUMENTS / THE REAL CHRISTMAS BOOK".
  Clean printed **alphabetical TOC** on scans 3–5 (2 columns/page, dot-leader
  title-first `<TITLE> ...... <printed>`, A–Y). Scan 2 is blank/front matter.
- **Unusually legible** — the printed page numbers were readable even at 100 DPI;
  re-rendered the 3 TOC pages at 300 DPI to confirm the numbers before trusting
  them, but no half-column tiling or montage walk was needed.
- **Offset 0: printed page == 1-based scan page** (the printed numbering counts
  the front matter — first song "A Caroling We Go" is printed 6 on scan 6, last
  "You're All I Want For Christmas" is printed 216 == page count). Verified by
  rendering scan 6 (printed corner "6") and scan 150 (printed "150" = My Only
  Wish This Year). Clean scan, single constant offset, no missing/duplicated
  pages. No clamps (max printed 216 → render 215).
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence: Blue Christmas 17 (among the B's
  that run 18–26), Mistletoe And Holly 137 (in the middle of the M's), Noël! Noël!
  149 (after The Night Before Christmas Song 152), Up On The Housetop 195 (its own
  U section, numerically among the T's). No multi-song pages — each carol starts
  its own printed page (1- or 2-page charts).
- Multi-line TOC titles joined into one entry (e.g. "Christmas Is The Time To Say
  I Love You" 36, "(There's No Place Like) Home For The Holidays" 90, "The Last
  Month Of The Year (What Month Was Jesus Born In?)" 120). Two distinct "Away In A
  Manger" settings disambiguated by composer (Spillman 15 / Murray 16). Kept the
  accented forms from the TOC: "The First Noël" 51, "Noël! Noël!" 149, "Gesù
  Bambino (The Infant Jesus)" 58.
- Built clean, 150 entries, no clamp warnings.

## nrealbk1 — The New Real Book, Vol. 1 (Sher Music, 1988), offset 15, 236 songs

- Pure scan, 453 pages (ISBN 0-9614701-4-3). Clean printed **Alphabetical Index**
  on scans 6–9, dot-leader `<COMPOSITION> ... <as played by> ... <printed>`,
  **unusually legible — read straight from 150-DPI page renders** (then re-rendered
  the four index pages at 300 DPI and cross-checked every number against a cropped
  page-number-column strip to be safe). No half-column tiling or montage walk needed.
- Front matter uses **roman numerals**: scans 1–2 cover/endorsements, 3 title page,
  4–5 the *categorical* contents (Jazz Classics / Choice Standards / Pop-Fusion — a
  useful cross-check, not indexed), 6–9 the alphabetical index, 10–15 = printed i–vi
  (Publisher's & Musical Editor's Forewords, General Rules, Chord Symbols). First
  chart is scan 16 = "Affirmation" (printed 1).
- **Offset 15: printed 1 == 1-based scan 16.** Single constant offset holds
  end-to-end — verified late in the book by rendering scan 426 = "Your Mind Is On
  Vacation" (printed 411). No clamps (max printed 437 → scan 452, book has 453 pages).
- Index numbers are monotonic with alphabetical order within each section; gaps are
  multi-page tunes (e.g. Endangered Species 83 → E.S.P. 90). The page-number column
  is the error-prone part — the cross-check caught a **missed entry "Dindi" (printed
  71)** hiding between Dig (70) and Don't Go To Strangers (74), plus a few dot-leader
  misreads. Titles were reliable.
- The two appendix entries (**Appendix I – Sample Drum Parts** 413, **Appendix II –
  Sources** 421) are intentionally omitted; the **Standards Supplement (U.S.A. only)**
  songs (printed 429–437: All Or Nothing At All, Do Nothing 'Til You Hear From Me,
  Don't Get Around Much Anymore, Good Morning Heartache, Misty, Speak Low, Stormy
  Weather) are kept — this set the convention nrealbk2/nrealbk3 follow.
- Built clean, 236 entries, no clamp warnings.

## nrealbk2 — The New Real Book, Vol. 2 (Sher Music), offset 12, 218 songs

- Pure scan (Acrobat 4.0 Scan, 1999), 497 pages. Clean printed **Alphabetical
  Index** on scans 5–8, dot-leader `<TITLE> ... <performer> ... <printed>`,
  **unusually legible — read straight from 100-DPI page renders**, no half-column
  tiling or montage walk needed (the right-margin page numbers were crisp and
  monotonic within each section, so this was one of the easier books).
- Front matter: scans 1–4 cover/front; scan 9 *Publisher's Foreword*, 10 *Musical
  Editor's Foreword*, 11 *General Rules For Using This Book*, 12 *Chord Symbols*.
  First chart is scan 13 = "Afro-Centric" (printed 1).
- **Offset 12: printed 1 == 1-based scan 13.** Single constant offset holds
  end-to-end — verified mid-book by rendering scan 247 = "My Ship" (printed 235).
  No clamps (max printed 483 → scan 495, book has 497 pages).
- Index numbers are monotonic with alphabetical order within each section; gaps
  are multi-page tunes.
- **Two distinct "I'll Be Around" entries** kept, disambiguated: the Chaka Khan
  arrangement (145) and "(Standard version)" (147).
- The two appendix entries (**Appendix I – Sample Drum Parts** 448, **Appendix II –
  Sources For Transcriptions** 461) are intentionally omitted; the **Standards
  Supplement (U.S.A. only)** songs (printed 473–483: The Joint Is Jumpin', More
  Than You Know, No Moon At All, Without A Song, Wrap Your Troubles In Dreams, You
  Say You Care) are kept — matching the nrealbk1 convention.
- Built clean, 218 entries, no clamp warnings.

## nrealbk3 — The New Real Book, Vol. 3 (C Version, Sher Music, 1995), offset 10, 196 songs

- Pure scan (Acrobat 4.0 Scan, 1999/2001). Clean printed **Alphabetical Index** on
  scans 5–8 (printed iii–vi), dot-leader `<TITLE> ... <performer> ... <printed>`,
  **unusually legible — read straight from 100-DPI page renders**, no half-column
  tiling or montage walk needed (the small right-margin page numbers were crisp and
  monotonic, so this was one of the easier books).
- Front matter: scan 1 cover ("THE NEW REAL BOOK / VOLUME 3 / C Version"), 2 title/
  credits, 3–4 the *categorical* contents (Jazz Classics / Choice Standards / Motown
  And Pop Classics / Contemporary Jazz — a useful cross-check, not indexed), 5–8 the
  alphabetical index, 9 *General Rules For Using This Book*, 10 *Chord Symbols*.
  First chart is scan 11 = "Actual Proof" (printed 1).
- **Offset 10: printed 1 == 1-based scan 11.** Each tune has interleaved C and
  Bass-clef versions (e.g. scan 11 "Actual Proof", scan 12 "Actual Proof (Bass)"),
  but the printed numbering counts BOTH pages sequentially, so a **single constant
  offset holds end-to-end** — no drift. Verified mid-book by rendering scan 241 =
  "Mamacita" (printed 231) and by DB query (Actual Proof → render 10). No clamps
  (max printed 424 → scan 434).
- The index numbers ARE essentially monotonic with alphabetical order here (unlike
  most Sher books) — gaps are multi-page tunes. **Two distinct "That Old Feeling"
  entries** kept: "(Standard version)" 380 and the Art Blakey arrangement 381.
- The two appendix entries (**Appendix I – Sample Drum Parts** 425, **Appendix II –
  Sources** 432) are intentionally omitted, matching the nrealbk2 convention.
- Built clean, 196 entries, no clamp warnings.

## disneyfake — The Disney Fake Book (Hal Leonard, ISBN 0-634-02578-3), offset 0, 243 songs

- Pure scan, 236 pages. The usable index is **"ALPHABETICAL BY SONG TITLE"**, read as
  600-DPI square-ish column tiles (full-page Reads downscale and blur the small bold
  dot-leader numbers — the page numbers are the error-prone part, titles are reliable).
- **The index spans THREE pages (scans 3, 4 AND 5), not two — this is the main trap.**
  Pages 3–4 are 2 columns each and stop at "Under the Sea" (206); **page 5 is a single
  short column holding the entire U–Z tail** (Up, Down and Touch the Ground 209 →
  Theme from "Zorro" 233, ~35 songs). Stopping at page 4 silently drops all of V–Z.
- Scan 2 is a partial *by-production* listing (Dumbo / Pinocchio / Snow White / Three
  Little Pigs) — **not** indexed (it's a subset of the alphabetical index); handy as a
  completeness cross-check.
- **Offset 0: printed page == 1-based scan page** (front cover is scan 1 and the
  printed numbering coincides). Clean scan, single constant offset, no missing/
  duplicated pages. Verified at scan 173 (printed corner "173", shared by the tail of
  Son of Man and Whistle While You Work) and scan 236 = "You've Got a Friend in Me"
  (printed 236) — the **last entry lands on the last page**, so the page count closes
  exactly. No clamps (max printed 236 → render 235).
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence (e.g. Hi-Diddle-Dee-Dee 91 among the
  H's, Little Wooden Head 137, The Merry Mouseketeers 161, Whistle While You Work 173
  sitting deep in the W run). The error-prone clip caught here: **"The Monkey's Uncle"
  (printed 120)** sits at a column edge between Minnie's Yoo Hoo (121) and The Morning
  Report (122) — its lower number looked wrong but is correct.
- **Shared/multi-song printed pages are normal, not misreads**: charts start mid-page,
  so many printed numbers repeat across adjacent titles — e.g. 89 (I'm Wishing +
  I've Got No Strings), 104 (Johnny Tremain + Jolly Holiday), 114, 146, 148, 150, 152,
  154, 167, 168, 170, 176, 180, 184, 187, 190, 196, 198, 208, 212, 223, 224, 226, 232.
- Filing quirk: **"Theme from 'Zorro'" (233) is the very last TOC entry**, filed after
  Zip-A-Dee-Doo-Dah (sorted on "Zorro", ignoring the leading "Theme from").
- Built clean, 243 entries, no clamp warnings.

## safakebk — Straight Ahead Jazz Fakebook (Charley Gerard, ed., 1999), offset -1, 68 songs

- Scanned charts, but the **TOC is a clean digital-quality typeset render** (crisp,
  fully legible at 150 DPI — no degradation, tiling, or montage walk needed), on
  scans 2–3, **3 columns: Composition / Composer / Recording + printed page**. Only
  the Composition column and the rightmost page number are indexed (Composer and
  Recording columns dropped). Hard-bop / Blue Note repertoire (Jackie McLean, Lee
  Morgan, Woody Shaw, Hal Galper, Tom Harrell, Mulgrew Miller…); intro page (scan 4)
  is signed "Charley Gerard, Editor, March 1999".
- **Offset -1: printed 6 == 1-based scan 5** (render page 4). Front matter: scan 1
  cover, 2–3 TOC, 4 *Introduction*. Verified at two points — scan 5 = printed corner
  "6" = "Appointment in Ghana" (Jackie McLean), and scan 9 = printed "10" = "Beyond
  All Limits" (Woody Shaw). Clean scan, single constant offset, no missing/duplicated
  pages. No clamps (max printed 158 → scan 157, book has 159 pages).
- **The TOC is ordered by the page-number column, NOT strictly alphabetically** —
  the Composition column is *roughly* A–Z but the page numbers are strictly
  monotonic and take precedence, so several titles sit out of alphabetical order:
  Most Like Lee 37 (between Figurine 36 and Five Will Get You Ten 38), Neither Here
  Nor There 45 (between Gotham Serenade 42 and Grew's Tune 46), Teeter Totter 103
  (between Sail Away 100 and Sakeena's Vision 104), Tune of the Unknown Samba 138.
  The monotonic page column is the reliable cross-check; don't "fix" the ordering.
- PDF typos in the book itself, left as printed for fidelity (search is
  case-insensitive; titles are otherwise the reliable part): **"Minor Aprehension"**
  (should be *Apprehension*, printed 72) and **"Spidit"** (Hal Galper, printed 120 —
  reads "Spidit" on the TOC, not a misread). The Recording column also misprints
  "Keys to the Ciy" under Song for Darnell, but that column isn't indexed.
  Normalized the TOC's double space in "Portrait of  a Mountain" → single space.
- The trailing **"Index of Composers" (printed 160)** is a back-matter listing, not a
  song — omitted. Built clean, 68 entries, no clamp warnings.

## realbk6h — The Real Book, Vol. VI (C Instruments), offset -2, 400 songs

- Pure scan, 486 pages. Clean printed **alphabetical TOC** on scans 2–7 (printed
  numbers in a `<TITLE> ...... <printed>` dot-leader format), 2 columns/page, A–Z.
  Cover (scan 1) reads "C INSTRUMENTS / Volume VI / THE REAL BOOK". Read as 600-DPI
  per-column half tiles — the small bold right-margin numbers are the error-prone
  part, titles are reliable. **The columns and their page numbers are wide enough
  that a straight 50% page split clips the left column's right-margin numbers** —
  crop the two columns with horizontal overlap (`-crop 2900x7015+0+0` and `+2060+0`
  on the 4960×7015 page) so each column's numbers survive.
- **Offset -2: printed 10 == 1-based scan 8** (first song "About A Quarter To Nine").
  Front matter: scan 1 cover, scans 2–7 TOC; the printed numbering starts at 10 on
  the first chart. Clean scan, single constant offset, **no missing/duplicated pages,
  no actual defects in the PDF**. Verified end-to-end: spot-rendered Eleanor Rigby
  (printed 107 → scan 105) and Temptation (printed 416 → scan 414), plus the bounds
  (About A Quarter To Nine → render 7, Zing! Went The Strings Of My Heart printed
  488 → render 485 == last page). No clamps (max printed 488 → render 485, book has
  486 pages).
- **Page numbers are NOT monotonic with alphabetical order** (songs placed to fit
  pages) — read each, don't infer from sequence. Plenty of intentional back-jumps,
  not misreads: Artistry In Rhythm 29 (after And When I Die 30), Do It Again 87
  (among the high-80s D's), How Long Has This Been Going On? 163 (after How Do You
  Keep The Music Playing? 164), Mr. Kenyatta 293 (deep in the M's), Spartacus –
  Love Theme 393, Straighten Up And Fly Right 397, Summer Wind 407, Volare 449,
  Mine 287, Mangos 271. Number gaps are normal (2-page charts).
- Transcription hazard caught during this run: **Two For The Road / Two Of A Kind
  read as a duplicate "446/446" at the tile edge** but are really **445/446** —
  confirmed from the adjacent column's left-margin numbers, which overlap into the
  facing crop. A duplicate-looking number is almost always a clip/misread; chase it.
- Titles kept in the TOC's ALL-CAPS form (search is case-insensitive); long wrapped
  TOC titles joined into one entry (e.g. "Brown Skin Gal In The Calico Gown" 62,
  "Don't Sit Under The Apple Tree (With Anyone Else But Me)" 95, "Put 'Em In A Box,
  Tie 'Em With A Ribbon (And Throw 'Em In The Deep Blue Sea)" 332). Two distinct
  "The Night Is Young" entries kept (308 and the "(And You're So Beautiful)" one 309).
- Built clean, 400 entries, no clamp warnings.

## reasybk — The Real Easy Book, Level 1 (C Version, Sher Music / Stanford Jazz Workshop), offset 8, 42 songs

- Pure scan, 95 pages. Cover (scan 1) "THE REAL EASY BOOK / TUNES FOR BEGINNING
  IMPROVISERS / LEVEL 1 / C VERSION". The whole **"Index Of Tunes" fits on a single
  page** (scan 3, TUNE / COMPOSER / PAGE columns, dot-leader). Unusually clean and
  small — read straight from one 400-DPI page render, no tiling or montage walk.
- **Offset 8: printed 1 == 1-based scan 9.** Long front matter pushes the first chart
  well in: scan 1 cover, 2 title page, 3 index, 4 "What Is Unique About This Book?",
  5 "How To Use This Book", 6 "Some Important Definitions", 7 "About the Stanford Jazz
  Workshop"/Editor's Note, 8 a "The Tunes" section-divider photo; first chart
  "Bags' Groove" (printed corner "1") is scan 9. Verified by rendering scan 9 and by
  DB query (Bags' Groove → render 8, Z's Blues → 90). No clamps (max printed 83 →
  render 90, book has 95 pages).
- **Every tune is exactly a 2-page chart, so all printed page numbers are odd**
  (1, 3, 5, … 83) and run strictly monotonic with the (near-alphabetical) tune order.
  The index lands the viewer on each tune's first page; the verso is the "For your
  use" blank-staff practice page.
- **Tune order is only roughly alphabetical** — a few are placed out of strict order
  but their printed numbers are still sequential, so nothing to "fix": Revelation 49
  before Road Song 51, St. John 57 before Sister Sadie 59, St. James Infirmary 67
  after Sonnymoon For Two 65. Read the numbers off the index column, which is the
  reliable part here.
- The index's last two lines (**Appendix I – Additional Educational Material** 85,
  **Appendix II – Discography** 87) are back matter, not songs — omitted.
- **Nothing wrong in the PDF itself**: clean scan, single constant offset, no
  missing/duplicated pages, no TOC typos or page-number errors. Kept the index's
  spellings (e.g. "So Danço Samba" with the cedilla). Built clean, 42 entries, no
  clamp warnings.
