# Hyades

A digital 4X space strategy game targeting 30–45 minute matches: deterministic
auto-battler combat, hard win conditions, hidden simultaneous orders.

This repository holds **`hyades-engine`**, the single Rust crate both consumers
link against — the production game and the Monte-Carlo balancer. It is
dependency-free, presentation-free, deterministic, and WASM-targetable.

- **Design specs** live in [`docs/`](docs/) and are authoritative.
- **[`AGENTS.md`](AGENTS.md)** is the standing working agreement: design laws,
  open R-codes, and guardrails. Read it before changing engine behavior.
- **The game client** (`viewer/`, `web/`) runs in a browser. Its Replays menu
  plays recorded games back in a tactical and a juicy mode — see
  [`docs/Hyades_interface.md`](docs/Hyades_interface.md). It is deployed to
  GitHub Pages from `main`.
- **[`MIGRATION.md`](MIGRATION.md)** records how this tree was assembled and
  which propulsion helpers are reconstructed placeholders.

## Build and test

No third-party dependencies in the engine — everything is std-only. The game
interface and networking may link upstream packages (the author's ruling).

```bash
cargo build
cargo test --workspace   # engine and viewer
cargo test arena::  # combat/arena primitives only
```

Monte-Carlo sweeps are far too slow in debug; always use `--release`:

```bash
cargo run --release --example laser_vs_missile   # ROU laser-vs-missile sweep
cargo run --release --example combat_arena       # kinematic interception harness
cargo run --release --example montecarlo         # balance sweeps
cargo run --release --example coverage_time      # colonization coverage timing
cargo run --release --example min_time_search    # coverage parameter search (offline; ~40 min)
cargo run --release --example trace              # single-run diagnostic log
```

The replay viewer, built and opened locally:

```bash
web/build.sh /tmp/site              # the wasm module, the page, three recorded replays
python3 -m http.server -d /tmp/site # then open http://localhost:8000
cargo test -p hyades-viewer         # the viewer's tests, and the engine–viewer contract
node web/test/smoke.mjs /tmp/site/hyades_viewer.wasm /tmp/site/replays
node web/test/browser.mjs /tmp/site # needs Playwright
```

## Layout

```
AGENTS.md    standing context: design laws, R-codes, guardrails
Cargo.toml
rustfmt.toml
src/         the engine (lib.rs wires the modules)
examples/    MC sweeps + arena drivers; record_replay writes the viewer's replays
tests/       smoke.rs, determinism.rs, telemetry.rs, balance.rs
viewer/      hyades-viewer: the replay viewer, Rust, compiled to wasm32
web/         the game client (menu, replays, palette), WebGL2 renderer, site build and browser tests
docs/        the design specs
```

## Continuous integration

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs on every push and
pull request. It uses only first-party `actions/*` plus `rustup`, keeping the
zero-dependency posture out to the build system — except the `viewer` job,
which installs Playwright from npm to drive the runner's Chrome.

| job | gate |
|---|---|
| `test` | `cargo build --workspace --all-targets`, the full test suite, doctests |
| `lint` | `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` |
| `wasm` | `cargo check --lib --target wasm32-unknown-unknown` |
| `examples` | builds every example, runs the four fast ones |
| `balance` | `tests/balance.rs` (tuned combat) and `coverage_trace` |
| `viewer` | builds the site, drives the wasm module over every replay, opens the page in headless Chrome |

[`.github/workflows/pages.yml`](.github/workflows/pages.yml) builds and tests
the same site on every push to `main` and deploys it to GitHub Pages (the
repository's Pages source must be set to "GitHub Actions").

`RUSTFLAGS: -D warnings` is set workflow-wide, so a plain rustc warning fails
the build too, not just a clippy lint.

The `wasm` job exists because `src/lib.rs` claims the engine never touches the
clock, filesystem, network, threads, or OS RNG. Compiling for
`wasm32-unknown-unknown` is what keeps that claim honest.

## License

Apache-2.0 — see [`LICENSE`](LICENSE).
