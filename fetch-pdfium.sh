#!/usr/bin/env bash
#
# Fetch the prebuilt pdfium shared library that pdfium-render binds to at
# runtime. pdfium-render does NOT bundle pdfium — you supply libpdfium.so.
#
# Places (both are git-ignored: *.so, **/jniLibs — re-run this per machine):
#   Android arm64-v8a -> app/src/main/jniLibs/arm64-v8a/libpdfium.so (into the APK)
#   Desktop Linux x64 -> ./libpdfium.so at the repo root (loader checks ./ first)
#
# Source: https://github.com/bblanchon/pdfium-binaries (PDFium, BSD-3-Clause).
#
# Usage: ./fetch-pdfium.sh [android|linux|all]   (default: all)
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
  # $1 = bblanchon asset basename (no .tgz); $2 = destination path for libpdfium.so
  local asset="$1" dest="$2" d="${TMP}/$1"
  echo "Fetching ${asset}.tgz ..."
  mkdir -p "$d"
  curl -fL --retry 3 "${BASE}/${asset}.tgz" -o "${d}.tgz"
  tar xzf "${d}.tgz" -C "$d"
  [[ -f "${d}/lib/libpdfium.so" ]] || { echo "error: lib/libpdfium.so missing in ${asset}.tgz" >&2; exit 1; }
  mkdir -p "$(dirname "$dest")"
  cp "${d}/lib/libpdfium.so" "$dest"
  echo "  -> ${dest} ($(du -h "$dest" | cut -f1))"
}

target="${1:-all}"
case "$target" in
  android | linux | all) ;;
  *) echo "usage: $0 [android|linux|all]" >&2; exit 2 ;;
esac

if [[ "$target" == android || "$target" == all ]]; then
  fetch pdfium-android-arm64 "${ROOT}/app/src/main/jniLibs/arm64-v8a/libpdfium.so"
fi
if [[ "$target" == linux || "$target" == all ]]; then
  fetch pdfium-linux-x64 "${ROOT}/libpdfium.so"
fi

echo "Done (pdfium ${VERSION})."
