# TuneCall — TODO

## Index

All books are indexed via a transcribed `<stem>.index` sidecar (the scans are
too degraded for OCR — see `indexer/README.md`). Per-book offsets/titles live in
`indexer/rebuild_db.sh`.

- [ ] Spot-check the vision-transcribed entries — titles are reliable, but some
      page numbers were best-effort reads (worst in the most degraded scans).
      Fix any wrong page directly in the `.index` and re-run.

## 1. Viewer UX (for real playing use)

- [ ] Swipe-to-turn-page (buttons only today); consider volume-key / pedal page
      turns for hands-free use mid-tune.
- [ ] Accent-insensitive / fuzzy search (currently plain substring).
- [ ] Render off the UI thread + prefetch/cache the next page (avoid jank on
      large pages, especially on Android).

## 2. End-user onboarding

- [ ] In-app PDF import via the system file picker (SAF), so users don't have to
      copy files over USB. Until then, the USB flow is documented in
      `docs/adding-your-pdfs.md`.

## 3. Publishing to Google Play

Build/packaging gates are done in-repo (App Bundle, SDK 35, AGP 8.7, manifest
permissions, signing wired to `keystore.properties`). Remaining steps are
external — see `docs/PLAY_PUBLISHING.md`, `docs/RELEASE_SIGNING.md` (one-time
signing setup), and `docs/RELEASE_HOWTO.md` (per-release build/upload):

- [ ] Create the release/upload keystore + `keystore.properties` (RELEASE_SIGNING.md).
- [ ] Privacy policy URL + Data Safety form (ideally serve the index download
      over HTTPS instead of `http://`).
- [ ] Store listing: 512×512 icon, feature graphic, screenshots, description
      (with the "you supply your own PDFs" framing); content rating; dev account.

