#!/bin/sh
# Upload the per-PDF song indexes (<book>.db) to the TuneCall server, plus an
# `index.txt` manifest that lists them. The mobile app's Reload button then
# downloads these over http from http://www.davidfaure.fr/tunecall/.
#
# Only the .db indexes are uploaded (publishable); the PDFs are not.
#
# Uses ncftp's `davidfaure` bookmark, which stores the host/user/password
# (so there's nothing to read here, and it copes with passive-mode FTP that
# plain curl tripped over).
#
# Usage: indexer/upload-indexes.sh [pdf-dir]
#   pdf-dir defaults to ~/.local/share/tunecall/pdfs
set -eu

PDF_DIR="${1:-$HOME/.local/share/tunecall/pdfs}"
BOOKMARK="davidfaure"
REMOTE_DIR="tunecall"

[ -d "$PDF_DIR" ] || { echo "no such directory: $PDF_DIR" >&2; exit 1; }

# Collect the .db files (fail clearly if there are none).
set -- "$PDF_DIR"/*.db
[ -e "$1" ] || { echo "no .db files in $PDF_DIR (run the indexer first)" >&2; exit 1; }
n=$#

# Build the manifest the app reads to know which indexes to fetch.
MANIFEST="$PDF_DIR/index.txt"
: > "$MANIFEST"
for db in "$@"; do
    basename "$db" >> "$MANIFEST"
done

# Upload every .db plus the manifest in one go (-m creates the remote dir).
ncftpput -m "$BOOKMARK" "$REMOTE_DIR" "$@" "$MANIFEST"
echo "uploaded $n index file(s) + manifest to $BOOKMARK:$REMOTE_DIR/"
