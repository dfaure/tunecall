# Do NOT strip libtunecall.so here: AGP strips the .so it packages into the
# release APK itself, and with ndk.debugSymbolLevel = 'FULL' it first extracts
# the debug symbols into the bundle metadata (kept server-side by Play). A
# manual `strip` would discard those symbols before AGP can save them, so Play
# could not symbolicate native crash/ANR traces. (Cargo.toml's [profile.release]
# keeps line tables so the symbols carry line numbers.)
cargo build --lib --no-default-features --release --features slint/backend-android-activity-06 && cp target/release/*.so app/src/main/jniLibs/arm64-v8a/ && ./gradlew bundleRelease && cp app/build/outputs/bundle/release/*.aab /sdcard/Download/
