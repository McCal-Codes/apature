#!/usr/bin/env bash
# REL-32: the changelog's top section names the version Cargo.toml carries; a tag must match both.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo_v=$(grep -m1 '^version = ' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
log_v=$(grep -m1 -oE '^## \[[0-9]+\.[0-9]+\.[0-9]+[^]]*\]' CHANGELOG.md | tr -d '#[] ')
if [[ "$cargo_v" != "$log_v" ]]; then
  echo "::error::Cargo.toml is $cargo_v but CHANGELOG.md's top section is $log_v"; exit 1
fi
if [[ -n "${1:-}" ]]; then
  tag=${1#v}
  if [[ "$tag" != "$cargo_v" ]]; then echo "::error::tag $1 does not match version $cargo_v"; exit 1; fi
fi
echo "version $cargo_v consistent"
