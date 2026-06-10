# Google Play publishing checklist

Non-code requirements to get TuneCall onto the Play Store. (Build/signing gates
live in [RELEASE_SIGNING.md](RELEASE_SIGNING.md); the technical state — AAB,
SDK 35, permissions — is already handled in `app/build.gradle` and the manifest.)

## Prerequisites

- A **Google Play developer account** (one-time US$25 registration). Identity
  verification can take a few days — start early.
- The signed `app-release.aab` from [RELEASE_SIGNING.md](RELEASE_SIGNING.md).

## B5 — Privacy policy + Data Safety

The app holds the `INTERNET` permission and **downloads song indexes** from
`http://www.davidfaure.fr/tunecall/` on Reload, so Play requires both:

- **Privacy policy** — a public URL (can live on davidfaure.fr). State plainly:
  - The app does **not** collect, transmit, or share any personal data.
  - The only network activity is downloading public, non-personal song-index
    files (`index.txt` + `<book>.db`) from the developer's server.
  - PDFs are supplied by the user and never leave the device.
  - No analytics, no ads, no accounts.
- **Data Safety form** (Play Console questionnaire): declare **no data
  collected / no data shared**. Mention the index download is app
  functionality, not data collection.

> Consider serving the index download over **HTTPS** rather than `http://`.
> Cleartext traffic needs an extra manifest opt-in on modern Android and looks
> worse on review; an HTTPS endpoint avoids both.

## B7 — Store listing & ratings

- **Content rating**: complete the IARC questionnaire — TuneCall is a utility
  with no objectionable content, so it rates "Everyone".
- **App category**: Music & Audio (or Tools).
- **Listing assets**:
  - High-res icon: **512×512** PNG. (The in-app icon is
    `app/src/main/res/drawable/tunecall.png`; produce a clean 512×512 version,
    ideally as an adaptive icon.)
  - **Feature graphic**: 1024×500 PNG.
  - **Screenshots**: at least 2 phone screenshots (Search, the page viewer,
    Setlists make good ones). Add 7"/10" tablet shots since tablet is the
    primary form factor.
  - Short description (≤80 chars) and full description.
- **Target audience & content**: not directed at children.

## B8 — Copyright framing (important for review and for users)

TuneCall is a **viewer for PDFs the user already owns**. It does not bundle,
host, or distribute any copyrighted sheet music — only small song-title→page
index files. Make this explicit in the store description, e.g.:

> TuneCall does not include any sheet music. You supply your own PDF fake books;
> the app only indexes their tables of contents so you can search by song title
> and jump to the right page.

This both sets user expectations (the app is empty until they add PDFs) and
makes the copyright position clear to reviewers.

## Release flow recap

1. Bump `versionCode`/`versionName` in `app/build.gradle`.
2. Build the signed `.aab` (see RELEASE_SIGNING.md).
3. Play Console → create app → upload `.aab` to **internal testing** first.
4. Fill in Data Safety, content rating, store listing, privacy policy URL.
5. Roll out internal testing → closed/open testing → production.

## Device coverage note

The build targets **arm64-v8a only** (`aarch64-linux-android`), which satisfies
Play's 64-bit requirement and covers essentially all modern devices. If you want
older 32-bit devices or x86_64 emulators, add those Rust targets and ship the
matching `libpdfium.so` per ABI under `app/src/main/jniLibs/<abi>/`.
