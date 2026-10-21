#!/usr/bin/env bash
# Capture the screenshot set used by the README, the site and the marketing
# composites, from a folder of photos. Runs the real app headless, with a
# clean temporary prefs file so the sidebar shows only that folder.
#
#   tools/marketing-shots.sh "/path/to/Jun 14 Storm damage" out/
#
# Put ratings and captions in XMP sidecars first (exiftool -o X.xmp ...) so
# the sheet and inspector look used. Keep outlet and client names out of the
# folder name, file names and captions.
set -euo pipefail
SRC=${1:?folder of photos}
OUT=${2:-docs/screenshots}
mkdir -p "$OUT"
PREFS="$HOME/Library/Application Support/dev.mccal.Apature/prefs.json"
BACKUP=$(mktemp)
[ -f "$PREFS" ] && cp "$PREFS" "$BACKUP"
restore() { [ -s "$BACKUP" ] && cp "$BACKUP" "$PREFS"; rm -f "$BACKUP"; }
trap restore EXIT
cargo build -q --release -p apature
shot() { # name appearance cell args...
  local name=$1 app=$2 cell=$3; shift 3
  printf '{"version":1,"appearance":"%s","cell":%s}' "$app" "$cell" > "$PREFS"
  cargo run -q --release -p apature -- "$@" --size 1280x800 --frames 500 --screenshot "$OUT/$name.png" "$SRC" 2>&1 | grep -i "error\|panic" || true
}
shot sheet Dark 156
shot sheet-light Light 156
shot preview Dark 200 --preview
shot loupe Dark 200 --preview --zoom11
shot compare Dark 200 --preview --compare
shot crop Dark 200 --preview --crop
shot caption Dark 200 --caption
shot ingest Dark 200 --ingest --ingest-source "$SRC"
shot export Dark 200 --export
shot settings Dark 156 --settings
echo "wrote $OUT"
