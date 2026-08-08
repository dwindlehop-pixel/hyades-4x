# CLAUDE.md — working agreement for the Hyades engine

This file is the engine's standing context. Read it before touching anything.
The authoritative design source is `docs/` — **the specs win over your priors,
and over this file, on any design question.**

---

## 1. What Hyades is

A digital 4X space strategy game targeting **30–45 minute matches**. Design pillars:

- **Digital board game**, not a CCG (lineage: Inis, 878 Vikings). No deck-building,
  no trap cards, no filler.
- **Deterministic auto-battler combat** — no execution/micro demands.
- **Hard win conditions** with progressive player elimination.
- Narrative thesis: *love/cooperation wins over deep time* (grounded in
  Traulsen & Nowak, PNAS 2006).
- Ship taxonomy and aesthetic draw on Iain M. Banks's Culture novels (GOU/ROU/LOU,
  GSV/MSV/LSV). The strategic arc recapitulates the classic PC game *Stars!*.

**Competitive frame:** a **meso + macro** game with a deliberately low micro floor.
Deterministic combat removes execution; the action/upgrade tree and economy are the
macro layer; hidden simultaneous orders create the meso layer (yomi/bluff, plus
bounded RNG via the wreck roll).

`hyades-engine` is the single Rust crate both consumers link against:
the **production game** and the **Monte-Carlo balancer**. It is WASM-targetable,
dependency-free, presentation-decoupled, and deterministic.

---

## 2. Build & test

No third-party dependencies — everything is std-only.

```bash
cargo build
cargo test                              # unit + integration
cargo test arena::                      # combat/arena primitives only
cargo run --release --example laser_vs_missile   # ROU laser-vs-missile sweep
cargo run --release --example combat_arena       # kinematic interception harness
cargo run --release --example montecarlo         # balance sweeps
```

Baseline as of the combat refactor: **79 unit + 4 smoke + 4 determinism tests pass.**
The MC sweeps are slow in debug; always use `--release` for them.

### CI gates

`.github/workflows/ci.yml` runs on every push and PR. Before you push, the four
things it will fail you on:

```bash
cargo fmt --all -- --check                          # rustfmt.toml: 120 cols, Max heuristics
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets && cargo test --doc
cargo check --lib --target wasm32-unknown-unknown   # holds the §1 portability claim honest
```

`RUSTFLAGS: -D warnings` is set workflow-wide, so a bare rustc warning is a
build failure too. The slow tuned sweeps (`laser_vs_missile`, `min_time_search`)
run in their own `balance` job so they do not block fast feedback.

---

## 3. Module map

| path | role |
|---|---|
| `src/math.rs` | 3-vectors, relativistic 1 g flight, light-lag (`c = 1`, distance in ly, time in years) |
| `src/rng.rs` | seeded splitmix64; `fork()` per entity for order-independent determinism |
| `src/resources.rs` | CMY basics, RGB supers, apex, archetypes |
| `src/galaxy.rs` | galaxy generation → continuous 3D planet field |
| `src/autopilot.rs` | `Autopilot` trait (swappable per-seat policy) + `Doctrine` knobs |
| `src/sim.rs` | the light-lagged discrete-event ECS engine |
| `src/combat.rs` | **engine-native combat**: kinematics, weapons, `resolve_engagement` |
| `src/arena.rs` | Ship Testing Arena — *scenario seeder only*, owns no combat logic |
| `src/matching.rs` | the Exchange (order-book matching) — **built, not yet wired into `lib.rs`** |
| `src/log.rs` | optional diagnostic event log (the interrogation seam) |
| `src/snapshot.rs` | read-only views for the presentation layer |

### The combat/arena split (load-bearing)

`combat.rs` is the engine's fighting model — the *same code* the production game and
the balancer resolve fights with. `arena.rs` exists only to **spawn ships outside the
constraints of Hyades production** (no economy, no mineral budget, no colonization),
place them, and call `combat::resolve_engagement`. **The arena resolves no damage.**
Dependency direction is `arena → combat`, never the reverse. Do not reintroduce
combat logic into the arena or into an example.

---

## 4. Architectural invariants

- **No presentation in the engine.** Nothing renders, reads input, touches the clock,
  the filesystem, the network, threads, or the OS RNG.
- **No hexes in the engine.** The simulation is continuous 3D space; each star system
  is a point. Hexes are a *command-view* concept owned by the presentation layer.
- **Determinism is a hard requirement.** All randomness flows from a seeded `Rng`;
  all time is the in-sim event clock in years. Iterate collections in deterministic
  order. Same seed ⇒ bit-identical results, native and wasm32. `tests/determinism.rs`
  guards this — never weaken it to make a feature fit.
- **Zero dependencies.** Do not add crates to `Cargo.toml`.

---

## 5. Design laws (non-negotiable)

These are settled. Do not relitigate them; if a change appears to require breaking
one, stop and flag it.

1. **Mineral substitution lives in the counter-graph, not the mineral ladder.**
   Red = general key (broad class access); Blue/Green = traversal keys (specific
   edges only).
2. **Hull supremacy must be slot-organic, not hyperparameter-tuned.** GOU superiority
   over an equal-cost ROU fleet (and ROU over LOU) must emerge from *hull slot counts
   and volume*, never from tuning battle constants like a `LASER_KILLS_PER_TICK`-style
   knob. Target scale: *Stars!* Dreadnought-vs-Cruiser; roughly 1 GOU handling 6–45 ROUs.
3. **Consolidation always wins under geometry alone.** Surface area is the cost basis,
   volume the value basis; the isoperimetric inequality guarantees bigger is more
   efficient. Any strategic value for smaller//fragmented fleets must therefore come
   from *combat-specific effects* (Lanchester's square law, indivisibility as a
   liability), not from the cost curve.
4. **The Ship Testing Arena is the required empirical harness** for setting per-class
   `r_eq`. These values cannot be derived analytically.
5. **`most_needed_center` is retained permanently as a test oracle** (single-supply
   degenerate matching provably reduces to it).
6. **No placeholder cost ratios as design targets.** 1:3:9 (GOU:ROU:LOU) and 1:12:9
   (GSV:MSV:LSV) are scaffolding to be replaced, not goals to hit.
7. **Cards operate at empire/macro scale only.** War Sun is the gold standard:
   flavorful, legible, places a behavior-rich board object. Combo cards are the
   connective tissue between trees — if single-tree play can win without cross-tree
   engagement, combo cards are undercosted.
8. **LOU role expectation:** chaff in late-game main battle fleets, *not* useful force
   projection — but genuinely useful for Mao-style insurgency (harass, avoid, strike
   the resting/retreating enemy).

---

## 6. Working agreement

- **Every PR updates `docs/` to match the design it lands.** `docs/` is the
  authoritative design source (see the header of this file), which only holds if
  it describes the engine as it actually is. A PR that changes behaviour, adds or
  retires a parameter, resolves an R-code, or invalidates something a spec asserts
  is **not complete until the affected spec is updated in the same PR** — the code
  change and the doc change are one unit of work, not a change plus a follow-up.
  Specifically:
  - Changed mechanics ⇒ update the spec section that describes them.
  - New tunable ⇒ document it where its siblings are documented, and say whether
    the value is confirmed or a placeholder.
  - Resolved R-code ⇒ mark it resolved where it is listed, with the resolution.
  - A measurement that contradicts a spec claim ⇒ correct the claim and cite the
    run, rather than leaving the doc to be believed and the code to be true.

  If a change genuinely touches no spec, say so explicitly in the PR body. Silence
  reads as an oversight, because usually it is one.
- **Make concrete decisions; flag open questions as R-codes.** A decision plus a
  flagged R-code beats an open-ended clarifying question. Existing families:
  `R-MC*` (mineral cost / combat), `R-L*` (loadout), `R-ARENA*`, `R-MX*` (matching),
  `R-CG*` (counter-graph), `R-XM*` (exotic matter).
- **Never silently change globally Monte-Carlo-tuned parameters.** They require
  explicit ratification. This includes everything in `combat::CombatConfig`.
- **Annotate superseded values; don't silently replace them.** Mark placeholders as
  placeholders. Regression tests asserting unverified values should be named
  "placeholder", not "confirmed".
- **Authoritative citations required for empirical claims** (papers, primary sources).
  No hand-waved numbers presented as derived.
- **Validate numerically before committing to a design.** Probe the scaling
  relationship (Python or a throwaway harness) *first*, then commit.
- **Flavor text is the author's own.** Never silently overwrite it.
- Direct, technical register. Concrete decisions over hedging.

---

## 7. Current state & next steps

**Just landed — the combat refactor.** Combat resolution moved out of the
`laser_vs_missile` example into `src/combat.rs`; `arena.rs` slimmed to a scenario
seeder; `ArenaShip` renamed `Combatant`; tuned weapon constants gathered into
`CombatConfig` (defaults reproduce the prior sweep bit-for-bit).

### ⚠️ Reconstructed placeholders — reconcile first

The uploaded bundle was internally inconsistent across branches; these were
**reconstructed** and their *magnitudes are placeholders*:

- `sim::hull_dry_mass`, `sim::hull_base_thrust`, `sim::hull_thrust_multiplier_range`
- `math::Vec3::cross`, public `sim::role_hull_type`
- `examples/combat_arena.rs` referenced `SimConfig::general_fleet_size` (absent here);
  substituted `1.0`, since General is the cost reference.

**These set the absolute ROU acceleration the laser-vs-missile balance rests on.**
If the prior definitions exist in git history, restore them and re-certify before
building on top.

### Next: R-MC9c (the active workstream)

Layer onto `combat::resolve_engagement`, all as `CombatConfig` fields + slot-derived stats:

- **HP pools** — a GOU must not die to a single hit.
- **Weapon count as firing units** — weapon *count* scales with hull slots; per-hit
  damage stays a single global constant (this is what keeps law #2 intact).
- **Missile AoE** — needed to handle dense LOU swarms; ships spread to avoid it.
- **Magazines** — LOUs model limited missiles; LOUs should be better laser platforms
  than missile platforms.

**Balance-preservation constraint:** keep the AoE radius *below* the baseline ROU
formation spacing, so the AoE term is identically zero in the ROU-vs-ROU case and the
existing laser-vs-missile balance is untouched. Verify by re-running
`--example laser_vs_missile` and comparing.

Prior numerical probing (to be re-derived in-engine): the GOU-vs-fleet crossover N\*
scales ~linearly with volume ratio ρ = V_GOU/V_ROU (N\* ≈ ρ/3.3), is nearly independent
of the cross-section constant, and lands in the 6–45 window for ρ ∈ [20, 150] —
i.e. supremacy is slot-organic by construction.

### Also open

- Wire `matching.rs` into `lib.rs` and swap the call sites in `sim.rs`.
- Refactor three long functions: `sys_production_tick` (~149 lines), `apply_build`
  (~148), `production_choice` (~137).
- Recalibrate `centrality_scale`/`k_high` — currently tuned to an old ~25 ly extent;
  the galaxy is now hundreds of ly.
- Pacing tension: at correct hex-derived scale, full galaxy exploration may need tens
  of thousands of simulated years against a 4,000-year default horizon.
- Counter-graph matrix partition (card contract §8.3): define Red-class positions vs.
  Blue/Green-edge positions so the mineral substitution law has mechanical grip.
- Open R-codes: R-ARENA1–7, R-MX1–6, R-XM5–7.

---

## 8. Simulation gotchas (learned the hard way)

- **`dt = 0.0005 yr` (~0.18 days) is required** for missile guidance convergence.
  At the old `dt = 0.006 yr`, missiles scored *zero* hits against a dodging target.
- **Fleet synchronization is an artifact generator.** Launch schedules must be
  desynchronized or burst size becomes meaningless.
- **`LASER_HIT_TOLERANCE` changes require dimensional analysis** against actual hull
  dimensions before acceptance. It is an abstracted fire-control stat, *not* literal
  hull cross-section.
- Combat runs just-in-time for 60 fps with a **< 2 ms per-tick budget**; presentation
  time is decoupled from simulation tick duration.
- The **Lanchester aggregate model** is reserved for imperial-scale resolution; the
  individual-missile arena exists only to calibrate its parameters.
- **No mineral seed for colonies** (homeworlds only) — mining outposts with need-based
  hauling are the intended fix.
- Idle military units do **not** auto-scrap; only exhausted LCVs scrap, others go to
  Reserve ("completable vs. standing mission").
- **K is player-relative by design.**
- Galaxy distribution: XY Poisson (exponential disk profile); Z exponential with Z-max
  matching the XY parameter.
