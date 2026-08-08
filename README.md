# Hyades

A digital 4X space strategy game targeting 30–45 minute matches: deterministic
auto-battler combat, hard win conditions, hidden simultaneous orders.

This repository holds **`hyades-engine`**, the single Rust crate both consumers
link against — the production game and the Monte-Carlo balancer. It is
dependency-free, presentation-free, deterministic, and WASM-targetable.

- **Design specs** live in [`docs/`](docs/) and are authoritative.
- **[`CLAUDE.md`](CLAUDE.md)** is the standing working agreement: design laws,
  open R-codes, and guardrails. Read it before changing engine behaviour.
- **[`MIGRATION.md`](MIGRATION.md)** records how this tree was assembled and
  which propulsion helpers are reconstructed placeholders.

## Build and test

No third-party dependencies — everything is std-only.

```bash
cargo build
cargo test          # 79 unit + 4 determinism + 4 smoke
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

## Layout

```
CLAUDE.md    standing context: design laws, R-codes, guardrails
Cargo.toml
rustfmt.toml
src/         the engine (lib.rs wires the modules)
examples/    MC sweeps + arena drivers
tests/       smoke.rs, determinism.rs
docs/        the design specs
```

## Continuous integration

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs on every push and
pull request. It uses only first-party `actions/*` plus `rustup`, keeping the
zero-dependency posture out to the build system.

| job | gate |
|---|---|
| `test` | `cargo build --all-targets`, the full test suite, doctests |
| `lint` | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` |
| `wasm` | `cargo check --lib --target wasm32-unknown-unknown` |
| `examples` | builds every example, runs the four fast ones |
| `balance` | `tests/balance.rs` (tuned combat) and `coverage_trace` |

`RUSTFLAGS: -D warnings` is set workflow-wide, so a plain rustc warning fails
the build too, not just a clippy lint.

The `wasm` job exists because `src/lib.rs` claims the engine never touches the
clock, filesystem, network, threads, or OS RNG. Compiling for
`wasm32-unknown-unknown` is what keeps that claim honest.

## Licence

Apache-2.0 — see [`LICENSE`](LICENSE).
