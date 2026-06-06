# TuneCall — TODO

Open work, roughly by priority. (The missing-pages / per-page-number problem is
intentionally dropped: the current PDF set doesn't have that issue, so a single
per-book `--offset` is sufficient.)

## Index — done

All five books are indexed via a transcribed `<stem>.index` sidecar (the scans
are too degraded for OCR — see `indexer/README.md`). Per-book offsets are
recorded in `indexer/index.sh` (1h −1, 2h 7, 3h 5, 4h −1, 5h −2).

- [ ] Spot-check the vision-transcribed entries — titles are reliable, but some
      page numbers were best-effort reads (worst in **2h**, the most degraded
      scan). Fix any wrong page directly in the `.index` and re-run.

## 1. Android (main target — currently unverified)

The app is "primarily Android" but has never been built or run there.

- [ ] Get the NDK/Gradle build working (`./gradlew build`,
      `slint/backend-android-activity-06`). Note the Termux debug-keystore path
      hardcoded in `app/build.gradle`.
- [ ] Ship `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/`
      (`fetch-pdfium.sh` fetches the arm64-v8a lib).
- [ ] Verify `android_main` (never type-checked on desktop): app-specific
      data-dir resolution, file permissions to read `pdfs/`, FemtoVG + winit.
- [ ] Confirm search + render + page nav actually run on a device.

## 2. Viewer UX (for real playing use)

- [ ] Pinch-zoom / pan (currently fit-to-window only).
- [ ] Swipe-to-turn-page (buttons only today); consider volume-key / pedal page
      turns for hands-free use mid-tune.
- [ ] Render off the UI thread + prefetch/cache the next page (avoid jank on
      large pages, especially on Android).
- [ ] Accent-insensitive / fuzzy search (currently plain substring).

## 3. Polish

- [ ] Real app icon (still the placeholder copied from videofinder).
- [ ] Friendlier book display names in results (currently the PDF file stem).
