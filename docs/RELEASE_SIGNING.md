# Release signing (A1)

Google Play will **not** accept an app signed with the debug key. The release
build reads its signing config from a **gitignored** `keystore.properties` at the
repo root; when that file is absent, release builds fall back to the debug
keystore (fine for sideloading, rejected by Play).

This is a one-time setup on the build device (the Android tablet / Termux, or
wherever you run `./gradlew`).

## 1. Understand the two keys (Play App Signing)

With Play App Signing (default for new apps), there are two keys:

- **App signing key** — held by Google. It signs the APKs delivered to users.
  You never see it; Google can manage/rotate it. Losing it is not catastrophic.
- **Upload key** — held by *you*. You sign every upload (`.aab`) with it; Google
  verifies it, strips it, and re-signs with the app signing key.

You create the **upload key** below. On your first release you can let Google
generate the app signing key automatically.

> **Back up the upload key and its passwords** (e.g. a password manager). If you
> lose it you can ask Google to reset it, but it's a hassle — don't rely on that.

## 2. Create the upload keystore

`keytool` ships with the JDK (the same one Gradle uses). On the build device:

```bash
keytool -genkeypair -v \
  -keystore "$HOME/tunecall-upload.jks" \
  -alias upload \
  -keyalg RSA -keysize 2048 \
  -validity 10000
```

It prompts for a keystore password, a key password (you can use the same), and a
name/organization (any sensible values; not shown to users).

Keep this `.jks` file **out of the repo** — `*.jks` and `*.keystore` are already
gitignored.

## 3. Create `keystore.properties`

At the repo root, create `keystore.properties` (gitignored) with the values from
step 2:

```properties
storeFile=/data/data/com.termux/files/home/tunecall-upload.jks
storePassword=YOUR_KEYSTORE_PASSWORD
keyAlias=upload
keyPassword=YOUR_KEY_PASSWORD
```

Use the **absolute path** to the `.jks`. `app/build.gradle` reads these four
keys for the `release` signing config.

## 4. Build the release bundle

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

Upload that `.aab` to the Play Console. Verify it's **not** debug-signed:

```bash
# the cert should be your "upload" key, not "androiddebugkey"
keytool -printcert -jarfile app/build/outputs/bundle/release/app-release.aab
```

## 5. Bump the version for each release

In `app/build.gradle`, increase `versionCode` (integer, must go up every upload)
and set a human `versionName` (e.g. `"1.1"`). Play rejects a re-used
`versionCode`.

## Troubleshooting

- *Release build still signed with `androiddebugkey`* → `keystore.properties` is
  missing or not at the repo root; the build silently fell back to debug.
- *`Keystore was tampered with, or password was incorrect`* → wrong
  `storePassword`/`keyPassword` in `keystore.properties`.
- *`storeFile ... does not exist`* → fix the absolute path in
  `keystore.properties`.
