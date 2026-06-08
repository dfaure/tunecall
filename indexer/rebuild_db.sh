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
index realbk6h    -2
index crealbk1    13
index 557standrd   7
index reasybk      8
index safakebk    -1
index disneyfake   0
index nrealbk1     15
index nrealbk2     12
index nrealbk3     10
index befakebk      3
index realxmasbk    0
index rdixieland    0
index creolejbfb    6
index tpdxmasjfb    6
index tnbobbook     7
index gridjazz      1
index bjazz50        0
index juststanrb     0
index rjstandfbk     1
index dhpccs100t     0
index lmjazz         4
index realrockb2     0
index strealbk      13
index rbblues        0
