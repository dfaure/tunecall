#!/bin/sh
set -e
cargo build --lib --no-default-features --release --features slint/backend-android-activity-06
cp target/release/*.so app/src/main/jniLibs/arm64-v8a/
aarch64-linux-android-strip -s ./app/src/main/jniLibs/arm64-v8a/libtunecall.so
./gradlew bundleRelease
cp app/build/outputs/bundle/release/*.aab /sdcard/Download/
ls -l /sdcard/Download/*.aab
echo "Open Firefox and create a release there"
