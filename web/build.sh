#!/usr/bin/env bash
# Assemble the client site (docs/Hyades_interface.md §8) into <out>:
# the page, the viewer module and the engine's module built for wasm32. The
# demo games ship as recipes the engine records in the browser (T-180);
# --quick makes them the short, small versions CI's tests open. CI's viewer
# job and the Pages deployment both run this.
#
# Usage: web/build.sh <out dir> [--quick]
set -euo pipefail
out="${1:?usage: web/build.sh <out dir> [--quick]}"
shift
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release -p hyades-viewer --target wasm32-unknown-unknown
cargo build --release -p hyades-net --target wasm32-unknown-unknown
cargo build --release -p hyades-sim --target wasm32-unknown-unknown

mkdir -p "$out"
cp web/index.html web/palette.html web/style.css web/shell.js web/gpu.js web/newgame.js web/sim-worker.js "$out/"
cp target/wasm32-unknown-unknown/release/hyades_viewer.wasm target/wasm32-unknown-unknown/release/hyades_sim.wasm "$out/"
case " $* " in *" --quick "*) quick=true ;; *) quick=false ;; esac
echo "{\"quick\":$quick}" > "$out/demos.json"
# The networking module (net/, docs/Hyades_sessions_discovery_and_security.md)
# and its wasm-bindgen glue. The CLI's version must equal the wasm-bindgen
# crate's in Cargo.lock; CI installs that release's prebuilt binary.
wasm-bindgen --target web --no-typescript --out-dir "$out/net" target/wasm32-unknown-unknown/release/hyades_net.wasm
# Pages serves files as they are; no Jekyll processing.
touch "$out/.nojekyll"
ls -l "$out"
