#!/usr/bin/env bash
# Installs the prebuilt wasm-bindgen CLI whose version equals the exact pin
# of the wasm-bindgen crate in net/Cargo.toml (they must match; Cargo.lock is
# not tracked, so the pin is what fixes the version), into
# ~/.local/bin, and puts that on the GitHub Actions PATH when run there.
# A prebuilt release takes seconds; `cargo install` takes minutes.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/^wasm-bindgen = "=\([0-9.]*\)"$/\1/p' "$root/net/Cargo.toml")"
[ -n "$version" ] || { echo "net/Cargo.toml must pin wasm-bindgen exactly (=x.y.z)" >&2; exit 1; }
dest="$HOME/.local/bin"
mkdir -p "$dest"
if [ -x "$dest/wasm-bindgen" ] && [ "$("$dest/wasm-bindgen" --version)" = "wasm-bindgen $version" ]; then
  echo "wasm-bindgen $version already installed"
else
  name="wasm-bindgen-$version-x86_64-unknown-linux-musl"
  tmp="$(mktemp -d)"
  curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$version/$name.tar.gz" | tar xz -C "$tmp"
  install "$tmp/$name/wasm-bindgen" "$dest/wasm-bindgen"
  rm -rf "$tmp"
fi
"$dest/wasm-bindgen" --version
if [ -n "${GITHUB_PATH:-}" ]; then echo "$dest" >> "$GITHUB_PATH"; fi
