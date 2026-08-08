# Migrating Hyades to Claude Code

This bundle is the Claude.ai Project reassembled as a normal cargo repo, so
Claude Code can build, test, and commit it directly.

## Layout

```
CLAUDE.md          standing context: design laws, R-codes, guardrails
Cargo.toml
src/               the engine (lib.rs wires the modules)
examples/          MC sweeps + arena drivers  (cargo run --release --example NAME)
tests/             smoke.rs, determinism.rs   (cargo test)
docs/              the design specs — formerly Project knowledge
```

## Landing it in your existing repo

The engine files here are your files plus the combat refactor. Suggested order:

1. Branch: `git checkout -b combat-module`
2. Copy `src/`, `examples/`, `tests/` over your tree. **`src/combat.rs` is new**;
   `src/arena.rs`, `src/lib.rs`, `src/sim.rs`, `src/math.rs`,
   `examples/laser_vs_missile.rs`, `examples/combat_arena.rs` are modified.
   Review `git diff` before committing — see the "reconstructed placeholders"
   warning in CLAUDE.md §7.
3. Copy `CLAUDE.md` to the repo root and `docs/` alongside.
4. `cargo test` — expect 79 unit + 4 smoke + 4 determinism passing.
5. `cargo run --release --example laser_vs_missile` — confirm the tuned balance
   (missiles favored closing at -0.002c; lasers at rest/receding; 200-v-100 lasers
   win with light casualties).

If your existing tree already has real definitions for the reconstructed
propulsion helpers (`hull_dry_mass`, `hull_base_thrust`,
`hull_thrust_multiplier_range`, `Vec3::cross`), **keep yours** and drop mine —
they set the absolute acceleration the laser-vs-missile balance depends on.

## Verified in this bundle

Compiled with rustc 1.75.0; library, all six examples, and all tests build clean.
Note: verification used bare `rustc` (no cargo in that sandbox), so run
`cargo test` once locally to confirm the manifest wiring.

## Not included

`guide_to_the_imperium_10_web.pdf` was left out deliberately — it's third-party
reference material, not project-authored, and committing it into a source repo is
a licensing question you should decide.

`Hyades.odt` has been converted to `docs/Hyades_design_notes.md` and the original
dropped. Despite the extension it was never an OpenDocument file — just plain
UTF-8 text — so the conversion was structural only (headings + list nesting);
wording is verbatim. Two ambiguities in the source are flagged in a "Conversion
notes" section at the end of that file rather than silently fixed.
