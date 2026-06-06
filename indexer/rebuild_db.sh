#!/usr/bin/env bash
#
# Rebuild every per-book <stem>.db from its <stem>.index sidecar.
#
# Run from the indexer/ crate dir (it uses `cargo run`). The per-book --offset
# values live here; tweak one when a "clamped" warning shows up for that book.
#
# The .pdf/.index/.db files live in the viewer's data dir (not in git). Override
# PDF_DIR if your books are elsewhere:  PDF_DIR=/path/to/pdfs ./rebuild_db.sh
set -euo pipefail

PDF_DIR="${PDF_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/tunecall/pdfs}"

index() {
    cargo run -- --pdf "$PDF_DIR/$1.pdf" --offset "$2"
}

#     <stem>      <offset>
index realbk1h    -1
index realbk2h     7
index realbk3h     5
index realbk4h    -1
index realbk5h    -2
index crealbk1    13
