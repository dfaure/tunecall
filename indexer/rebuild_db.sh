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
    cargo run -- --pdf "$PDF_DIR/$1.pdf" --offset "$2" --title "$3"
}

#     <stem>      <offset> <title (read off the cover)>
index realbk1h    -1 "The Real Book, Vol. 1 (Sixth Edition)"
index realbk2h     7 "The Real Book, Vol. 2"
index realbk3h     5 "The Real Book, Vol. 3"
index realbk4h    -1 "The Real Book, Vol. 4"
index realbk5h    -2 "The Real Book, Vol. 5"
index realbk6h    -2 "The Real Book, Vol. 6"
index crealbk1    13 "The Real Book, Vol. 1 (Fifth Edition)"
index 557standrd   7 "557 Standards"
index reasybk      8 "The Real Easy Book"
index safakebk    -1 "Straight Ahead Jazz Fakebook"
index disneyfake   0 "Disney Fake Book"
index nrealbk1     15 "The New Real Book, Vol. 1"
index nrealbk2     12 "The New Real Book, Vol. 2"
index nrealbk3     10 "The New Real Book, Vol. 3"
index befakebk      3 "Bill Evans Fake Book"
index realxmasbk    0 "The Real Christmas Book"
index rdixieland    0 "The Real Dixieland Book"
index creolejbfb    6 "The Creole Jazz Band Fake Book"
index tpdxmasjfb    6 "The Public Domain Christmas Jazz Fakebook"
index tnbobbook     7 "The New Bob Book"
index gridjazz      1 "Anthologie des Grilles de Jazz"
index bjazz50        0 "Jazz of the '50s"
index juststanrb     0 "Just Standards Real Book"
index rjstandfbk     1 "Real Jazz Standards Fake Book"
index dhpccs100t     0 "Dick Hyman's Professional Chord Changes and Substitutions for 100 Tunes"
index lmjazz         4 "Library of Musicians' Jazz"
index realrockb2     0 "Real Rock Book 2"
index strealbk      13 "The Standards Real Book"
index rbblues        0 "The Real Book of Blues"
index colcookbk      3 "The Colorado Cookbook"
index classicfb      0 "The Real Little Classical Fake Book"
index cpomnibook     0 "Charlie Parker Omnibook"
index cufakebk       8 "Cuban Fake Book, Vol. 1"
index realrockb1     0 "Real Rock Book"
index ajrealbk      13 "The All-Jazz Real Book"
index realjazzbk    -2 "The Hal Leonard Real Jazz Book"
index rwlpfbk       -1 "Richard Wolfe's Legit Professional Fake Book"
# bestfbevr2: printed 400 missing from scan and scan 498 duplicates printed 496,
# so the .index entries for printed 401-497 are shifted down by 1.
index bestfbevr2     1 "The Best Fake Book Ever (2nd Edition)"
