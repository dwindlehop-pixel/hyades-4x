#!/usr/bin/env bash
# Assemble the client site (docs/Hyades_interface.md §8) into <out>:
# the page, the viewer module built for wasm32, and replays recorded by the
# engine. CI's viewer job and the Pages deployment both run this.
#
# Usage: web/build.sh <out dir> [--quick]
set -euo pipefail
out="${1:?usage: web/build.sh <out dir> [--quick]}"
shift
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release -p hyades-viewer --target wasm32-unknown-unknown
cargo build --release --example record_replay

mkdir -p "$out/replays"
cp web/index.html web/palette.html web/style.css web/shell.js web/gpu.js "$out/"
cp target/wasm32-unknown-unknown/release/hyades_viewer.wasm "$out/"
./target/release/examples/record_replay "$out/replays" "$@"
# Pages serves files as they are; no Jekyll processing.
touch "$out/.nojekyll"
ls -l "$out" "$out/replays"
