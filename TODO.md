# TuneCall — TODO

Open work, roughly by priority. (The missing-pages / per-page-number problem is
intentionally dropped: the current PDF set doesn't have that issue, so a single
per-book `--offset` is sufficient.)

## Now: build the index

- [ ] Determine the per-book `--offset` for each PDF in `~/.local/share/tunecall/pdfs`.
- [ ] Index all PDFs: `--detect-toc` finds the TOC range; run with each book's
      offset to write the sibling `<book>.db`.
- [ ] Add a small batch runner (a `book -> offset` table + loop) to index all
      books in one go.

## 1. Android (main target — currently unverified)

The app is "primarily Android" but has never been built or run there.

- [ ] Get the NDK/Gradle build working (`./gradlew build`,
      `slint/backend-android-activity-06`). Note the Termux debug-keystore path
      hardcoded in `app/build.gradle`.
- [ ] Ship `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/`.
- [ ] Verify `android_main` (never type-checked on desktop): app-specific
      data-dir resolution, file permissions to read `pdfs/`, FemtoVG + winit.
- [ ] Confirm search + render + page nav actually run on a device.

## 2. Index quality

- [ ] Fuzzy / dictionary-assisted title correction for residual OCR misreads
      (`PARIS->PARIG`, `ME->MB`), to improve search recall.
- [ ] Fallback for books whose index isn't in the first 16 pages (scan the tail,
      or require manual `--toc`); detection currently `bail`s.
- [ ] Dedupe duplicate OCR rows (e.g. `Walk'in Thing` listed twice).

## 3. Viewer UX (for real playing use)

- [ ] Pinch-zoom / pan (currently fit-to-window only).
- [ ] Swipe-to-turn-page (buttons only today); consider volume-key / pedal page
      turns for hands-free use mid-tune.
- [ ] Render off the UI thread + prefetch/cache the next page (avoid jank on
      large pages, especially on Android).
- [ ] Accent-insensitive / fuzzy search (currently plain substring).

## 4. Polish

- [ ] Real app icon (still the placeholder copied from videofinder).
- [ ] Friendlier book display names in results (currently the PDF file stem).
