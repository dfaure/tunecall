# Cutting a release (every time)

Steps to build and upload a new release to Google Play. Do these **each** time.
The one-time signing setup (upload keystore + `keystore.properties`) is a
prerequisite — see [RELEASE_SIGNING.md](RELEASE_SIGNING.md) if it isn't done yet.

## 1. Bump the version

There are two numbers, and they move independently:

- **`versionName`** (e.g. `1.0.0`) — the human-readable version shown on Play and
  in the app's Books-tab footer. It is **read from `Cargo.toml`'s
  `[package] version`** — the single source of truth. Bump it there;
  `app/build.gradle` and the in-app footer both pick it up. Don't hardcode a
  version in `build.gradle`.
- **`versionCode`** (the integer in `app/build.gradle`) — Play's internal build
  counter. It **must increase on every upload**.

> **`versionCode` is burned the moment you upload an AAB** — even to a draft or
> closed test you never roll out, and even if you then delete or replace that
> AAB. Play will say *"Version code N has already been used. Choose a different
> one."* You can't reuse it; just bump to the next integer. So `versionCode`
> often climbs faster than `versionName` (a re-upload, a hotfix, or a scrapped
> test all consume one) — that's normal.

Rule of thumb: bump `versionCode` for **every** `.aab` you upload; bump
`versionName` (in `Cargo.toml`) only when the user-facing version actually
changes.

## 2. Build the signed App Bundle

On the build device:

```bash
cargo build --lib --target aarch64-linux-android \
    --no-default-features \
    --features slint/backend-android-activity-06
./gradlew bundleRelease
```

The signed App Bundle lands at:

```
app/build/outputs/bundle/release/app-release.aab
```

Verify it's signed with your **upload** key, not the debug key:

```bash
# the cert should be your "upload" key, not "androiddebugkey"
keytool -printcert -jarfile app/build/outputs/bundle/release/app-release.aab
```

(If it shows `androiddebugkey`, the signing setup is missing — see
[RELEASE_SIGNING.md](RELEASE_SIGNING.md).)

## 3. Upload to the Play Console

Upload `app-release.aab` to your release track (e.g. *Internal testing* /
*Closed testing*). If Play rejects the version code as already used, go back to
step 1 and bump `versionCode`.
