#!/usr/bin/env bash
#
# Fetch the prebuilt pdfium shared library that pdfium-render binds to at
# runtime. pdfium-render does NOT bundle pdfium — you supply libpdfium.so.
#
# Places (all git-ignored: *.so, *.dylib, **/jniLibs — re-run this per machine):
#   Android arm64-v8a   -> app/src/main/jniLibs/arm64-v8a/libpdfium.so (into the APK)
#   Desktop Linux x64   -> ./libpdfium.so at the repo root (loader checks ./ first)
#   iOS simulator arm64 -> ios/libpdfium-simulator.dylib (Xcode embeds it in the .app)
#   iOS device arm64    -> ios/libpdfium-device.dylib    (Xcode embeds it in the .app)
#
# iOS note: bblanchon ships only a dynamic `libpdfium.dylib` for iOS (no static
# `libpdfium.a`), so the app embeds the dylib in its bundle (Frameworks/) and
# pdfium-render binds to it at runtime (see src/pdf.rs). The simulator and device
# builds are different Mach-O binaries, so they're fetched separately and kept in
# arch-suffixed files; the Xcode "Embed pdfium dylib" phase picks the right one.
# The tgz layout also differs — the library is `lib/libpdfium.dylib`, not `.so`.
#
# Source: https://github.com/bblanchon/pdfium-binaries (PDFium, BSD-3-Clause).
#
# Usage: ./fetch-pdfium.sh [android|linux|ios-simulator|ios-device|ios|all]
#        (default: all; `ios` is an alias for `ios-simulator`)
set -euo pipefail

# Pinned release. To update, bump this and confirm pdfium-render's `pdfium_latest`
# feature still matches. Releases: https://github.com/bblanchon/pdfium-binaries/releases
VERSION="chromium/7906"
BASE="https://github.com/bblanchon/pdfium-binaries/releases/download/${VERSION}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP="$(mktemp -d "${ROOT}/.pdfium-download.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

command -v curl >/dev/null || { echo "error: curl not found" >&2; exit 1; }

fetch() {
  # $1 = bblanchon asset basename (no .tgz); $2 = destination path for the library;
  # $3 = library filename inside the tgz's lib/ dir (default libpdfium.so).
  local asset="$1" dest="$2" lib="${3:-libpdfium.so}" d="${TMP}/$1"
  echo "Fetching ${asset}.tgz ..."
  mkdir -p "$d"
  curl -fL --retry 3 "${BASE}/${asset}.tgz" -o "${d}.tgz"
  tar xzf "${d}.tgz" -C "$d"
  [[ -f "${d}/lib/${lib}" ]] || { echo "error: lib/${lib} missing in ${asset}.tgz" >&2; exit 1; }
  mkdir -p "$(dirname "$dest")"
  cp "${d}/lib/${lib}" "$dest"
  echo "  -> ${dest} ($(du -h "$dest" | cut -f1))"
}

target="${1:-all}"
case "$target" in
  android | linux | ios-simulator | ios-device | ios | all) ;;
  *) echo "usage: $0 [android|linux|ios-simulator|ios-device|ios|all]" >&2; exit 2 ;;
esac

if [[ "$target" == android || "$target" == all ]]; then
  fetch pdfium-android-arm64 "${ROOT}/app/src/main/jniLibs/arm64-v8a/libpdfium.so"
fi
if [[ "$target" == linux || "$target" == all ]]; then
  fetch pdfium-linux-x64 "${ROOT}/libpdfium.so"
fi
# `all` fetches the iOS *simulator* dylib (the dev-machine default). The device
# dylib is a separate, larger download, fetched only when explicitly asked for
# (or on demand by a device build's "Embed pdfium dylib" phase).
if [[ "$target" == ios || "$target" == ios-simulator || "$target" == all ]]; then
  fetch pdfium-ios-simulator-arm64 "${ROOT}/ios/libpdfium-simulator.dylib" libpdfium.dylib
fi
if [[ "$target" == ios-device ]]; then
  fetch pdfium-ios-device-arm64 "${ROOT}/ios/libpdfium-device.dylib" libpdfium.dylib
fi

echo "Done (pdfium ${VERSION})."
