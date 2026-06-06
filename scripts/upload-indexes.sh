#!/bin/sh
# Upload the per-PDF song indexes (<book>.db) to the TuneCall server via FTP,
# plus an `index.txt` manifest that lists them. The mobile app's Reload button
# downloads these over HTTPS from https://www.davidfaure.fr/tunecall/.
#
# Only the .db indexes are uploaded (publishable); the PDFs are not.
#
# The FTP password is read from ~/.kvideomanager_ftp_passwd (first line).
#
# Usage: scripts/upload-indexes.sh [pdf-dir]
#   pdf-dir defaults to ~/.local/share/tunecall/pdfs
set -eu

PDF_DIR="${1:-$HOME/.local/share/tunecall/pdfs}"
PASS_FILE="$HOME/.kvideomanager_ftp_passwd"
FTP_HOST="ftp.davidfaure.fr"
FTP_USER="david329069"
FTP_PATH="tunecall"

[ -d "$PDF_DIR" ] || { echo "no such directory: $PDF_DIR" >&2; exit 1; }
[ -r "$PASS_FILE" ] || { echo "cannot read password file: $PASS_FILE" >&2; exit 1; }
PASS=$(head -n1 "$PASS_FILE")

# Collect the .db files (fail clearly if there are none).
set -- "$PDF_DIR"/*.db
[ -e "$1" ] || { echo "no .db files in $PDF_DIR (run the indexer first)" >&2; exit 1; }

# Build the manifest the app reads to know which indexes to fetch.
MANIFEST="$PDF_DIR/index.txt"
: > "$MANIFEST"
for db in "$@"; do
    basename "$db" >> "$MANIFEST"
done

upload() { # $1 = local file -> ftp://host/path/<basename>
    curl -fsS --ftp-create-dirs -T "$1" --user "$FTP_USER:$PASS" "ftp://$FTP_HOST/$FTP_PATH/"
}

n=0
for db in "$@"; do
    echo "uploading $(basename "$db")"
    upload "$db"
    n=$((n + 1))
done
echo "uploading index.txt"
upload "$MANIFEST"
echo "done: $n index file(s) + manifest -> ftp://$FTP_HOST/$FTP_PATH/"
