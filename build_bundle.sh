#!/bin/sh
set -e
# Backend feature is selected by the target in Cargo.toml — no extra flags needed.
# Termux runs as aarch64-linux-android, so the host build is the Android .so.
cargo build --lib --release
cp target/release/*.so app/src/main/jniLibs/arm64-v8a/
aarch64-linux-android-strip -s ./app/src/main/jniLibs/arm64-v8a/libtunecall.so
./gradlew bundleRelease
cp app/build/outputs/bundle/release/*.aab /sdcard/Download/
ls -l /sdcard/Download/*.aab
