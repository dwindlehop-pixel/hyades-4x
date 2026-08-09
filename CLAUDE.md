# CLAUDE.md — working agreement for the Hyades engine

This file is the engine's standing context. Read it before touching anything.
The authoritative design source is `docs/` — **the specs win over your priors,
and over this file, on any design question.**

**Start with `docs/Hyades_standing_layer_and_observation.md` (Rev 1).** It is the
most recent ratification and it *supersedes or amends six other specs*, so a
claim you find elsewhere in `docs/` may already be retracted — its §12 lists
exactly which. It sets the standing-layer model (Doctrine and Design as state
written only by tree cards), the observation model (acceleration is the
long-range observable, not mass or hull count), the counter-graph as a
per-player ladder disrupted by cards, and mass conservation. §11 is the engine
roadmap; see §7 below for what has landed.

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

### The 60-second rule for tests and CI

**Every test target and every CI step must finish in ≤60 s.** Searches are the
only exception and they are offline, never in CI. Current costs:

| step | cost |
|---|---|
| `cargo test` (unit + determinism + smoke) | ~24 s |
| `tests/balance.rs` (release, `--ignored`) | ~52 s |
| `coverage_trace` | ~17 s |
| `coverage_time` | ~49 s |
| `montecarlo` | ~56 s |

Ratifying the snowball defaults blew every one of these past the budget at once —
the unit suite alone went 6 s → 315 s — because a default-config run is now a
full-colonization sim. The fix is never to weaken what a check proves; it is to
spend fewer *runs* on it:

- **Pin an explicit horizon in tests.** Determinism is a property of the
  arithmetic, not of how long you accumulate it; 800 yr proves it as well as
  4,000 and costs a twelfth as much (85 s → 7 s).
- **Cut samples, not the question.** `coverage_trace` asks "does this knob move
  the run at all", which three values answer as well as five (230 s → 17 s, every
  verdict preserved). `balance.rs` went to three seeds (86 s → 52 s), and
  `coverage_time` to two, keeping its doctrine *comparison* — the thing it is for.
- **Say what the trim cost.** Fewer seeds is less variance coverage. That is the
  offline search's job, and it is not time-boxed — so record the tradeoff where
  the constant is defined rather than letting it look like the full bed.

### Always run sweeps unbuffered

**Never pipe a long run through `tail`, `head`, `sort`, or a bare `grep`.** Those
stages hold their input until EOF (or fill a 4 KiB block first), so a sweep that is
printing a row a minute shows *nothing at all* until it finishes. The run looks hung,
and the natural reaction — kill it and retry — throws away the work. This has already
cost two sweeps in this repo: an 80-run coordinate-descent sweep and a throughput
benchmark, both abandoned as "stalled" while they were in fact running fine behind a
`| tail`.

- Let the harness print straight to the terminal, or redirect to a file (`> out.txt`)
  and read the file as it grows. Both stream.
- Filtering is fine if the filter streams: `grep --line-buffered`, `awk` with
  `fflush()`, `stdbuf -oL <cmd>`.
- In a harness that prints one row per expensive trial, call
  `std::io::stdout().flush()` after each row. Rust's stdout is line-buffered to a
  terminal but **block**-buffered to a pipe or file, so the flush is what makes a
  partial run readable — and a partial result you can read beats a complete one you
  killed.
- Long sweeps belong in the background from the start, with output to a file, so
  progress is inspectable without blocking on them.

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
build failure too. The slow work runs in its own `balance` job so it does not
block fast feedback: `tests/balance.rs` (the tuned combat goldens) and
`coverage_trace`.

`min_time_search` is **not in CI** — it is an offline job. At the ratified
defaults each of its ~270 trials is a full-colonization sim, putting the search
around 40 minutes locally and longer on a runner. Run it by hand when tuning.

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
- **Entities evaluate on their own arrival events — never on a tick sweep, and never
  by rescanning the galaxy.** This is a discrete-event engine: a ship decides what to
  do next *when it arrives somewhere* (`ContactArrive`, `FreighterArrive`,
  `ColonyArrive`, `ScrapArrive`), which is the only moment its situation actually
  changed. Production centers are the one cadence-driven exception, one tick per
  center per `cycle_years`; everything else is arrival-driven.

  The trigger is only half the rule. **Evaluation count scales with entity count, so
  per-evaluation cost must be local — O(what the decision reads), not O(galaxy).**
  The two multiply, and that product is what sets simulation speed:

  > `cost = entities × arrivals-per-entity × work-per-evaluation`

  Fleets grow without bound as the game snowballs, so the first two factors are the
  *design*, not something to trim. Only the third is ours to control, and it is
  therefore the one that must stay small.

  Measured violation, kept here as the worked example (seed 1, 3 seats, full
  colonization, horizon 4,000): survey re-targeting fires on `ContactArrive`, which is
  the correct trigger — but each evaluation walked every planet in the galaxy and
  built a full `PlanetView` for each survivor. That is **15,653 evaluations × ~4,100–6,725
  planets ≈ 64–105 M view constructions**, for a decision that reads only `id` and
  `position` and keeps exactly one result. Callgrind put `survey_candidates` +
  `view_of` at **81% of all engine instructions**. The trigger was right and the
  engine was still spending four fifths of its life there.

  Practical form of the rule:
  - Hand a decision only the fields it reads. An 88-byte view for a query that uses
    28 bytes of it is a 3× memory-traffic tax on the hottest path.
  - Do not materialize a collection you only `min_by`/`max_by` over.
  - Prefer incremental or spatial structures to full scans — but **measure, because
    fewer items is not automatically faster.** An incrementally-maintained unvisited
    frontier cut the scanned count 39% (mean 4,114 of 6,725) and came out *slower*
    at horizon 4,000 (9.67 s vs 8.90 s): swap-removal scrambled the order, trading a
    sequential walk for random access across three component stores. Locality beat
    count. That attempt is reverted; the finding is not.

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
9. **Legibility is σ read from the other side** — not a separate stat. A card's
   slant *is* how much it would only be worth playing if you meant it, so the
   σ→value curve must be **convex** or everyone opens inscrutable and the yomi
   channel carries nothing (L3/R-O19, Spence's single-crossing condition). The
   physical cause is that commitment shifts and narrows a fleet's acceleration
   distribution.
10. **Acceleration is the observable, not mass and not hull count.**
   `a = thrust / (dry_mass + cargo_mass)` is one scalar over three latents, so
   the inverse problem is under-determined at range and concealment is a **combo
   property, not a card property** — arming a fleet is loud unless you also buy
   thrust. A ship may fly below peak and never above it, so observed `a` is a
   *lower bound*, which is where surprise attack comes from (L4, §6.2/§6.4).
11. **Mass is conserved with no exclusions, and cost and dry mass are one
   number** (L6/R-O57). Minerals spent become hull; wastage degrades to slag
   rather than vanishing; expended ordnance leaves the fleet lighter. Negative
   and imaginary mass are *not* exceptions — which is why exotic synthesis is
   pair production. **Nor is population:** biosphere is a mass in kilotons and
   population growth consumes it 1:1. Biosphere is the one *renewable* stock,
   regrowing logistically toward `bio_max`, so ecology is a rate rather than an
   exemption — which is what makes biological damage durable and gives Warfare a
   target that is neither hulls nor infrastructure.
12. **No retroactive refits** (R-O47b). A Design write never reaches a hull
   already in the field by fiat; realization is `on_refit`, so a fleet-wide
   change lands staggered by transit time. Retroactive would change every
   acceleration signature at once, laglessly, with no build to watch for — the
   instant global state change the light-lagged observation model exists to rule
   out. The recall is itself a signal, and it cannot be offset by a thrust write
   because what leaks is the movement, not the mass.
13. **No categorical strategic classification may be co-extensive with a colour
   domain** (L1/R-O34) — it would lock out exactly the archetype poor in that
   colour. Continuous classifications expressed as magnitude are exempt.
14. **The snowball is the design — simulate it, not the stalled baseline.** An empire
   that compounds until it has colonized every colonizable world is the intended
   arc, and the shipped defaults produce it (R-AC16/R-AC17). The configuration that
   plateaued at a few dozen colonies was a *bug surface*, never a reference point:
   do not benchmark against it, do not tune against it, and do not treat a
   parameter as inert because it did nothing while expansion was broken —
   `cargo_unit_size` and `outpost_mining_fraction` looked dead for exactly that
   reason. Corollaries that keep biting:
   - **Thousands of vehicles is normal.** Engine cost must be measured at full
     colonization; a benchmark on a stalled galaxy measures nothing real (§7).
   - **Tests must pin a short horizon explicitly.** Unit tests exercise mechanics,
     and a full-length default run now costs seconds each — the suite went 6 s → 315 s
     the moment the defaults were ratified, and back to ~5 s once tests set their own
     horizon. Anything genuinely about long-run coverage belongs in an example or
     the offline search.

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
  — **R-O57 supersedes the reconciliation**: cost and dry mass are one number, so
  `hull_dry_mass` should be *derived from mineral cost and deleted* as an
  independent field rather than reconciled against git history. Coupled to
  R-O58 (shell model); see the roadmap below for why the two must land together.
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

### The standing-layer ratification — engine roadmap

`docs/Hyades_standing_layer_and_observation.md` §11 lists 15 engine work items.
Status, so this is not re-derived each time:

| # | Item | R-code | Status |
|---|---|---|---|
| 5 | Colony cargo mass ≡ mineral cargo mass | R-O32 | **done** — `laden_accel` now masses `pop_cargo`; it was massless, so a laden colony ship flew like an empty hull and the burn read out cargo *type*, the one thing §6.2 exists to hide |
| 12 | Verify the shell model's 1 : 2.2 : 4 radius prediction | R-O58b | **checked — it fails.** See below |
| 1 | `BuildOrder::Hull { hull_type, class }` + role assigned after production | R-O29 | open — the build order naming the mission is a free doctrine leak |
| 2 | Design/roster component | **R-O28** | **done** — `Roster` (a sorted, idempotent set of `(HullType, Class)`) is a per-player component written only by tree cards. Unblocks σ_vector for Design: the distance between pre- and post-card rosters is now computable. `Class` also introduces the Banks-convention design names (R-O42b: Meadow/Tor proposed, flavour subject to authorship) |
| 3 | Diplomatic fields on `Doctrine` | R-O27/R-A3 | open — no field list specified yet |
| 4 | Throttle fraction; observe `a` from trajectory not the stat block | R-O40 | open |
| 6 | `min_time_search` as a reachability-cone query | R-O31 | open — same function, reverse direction |
| 7 | Route intercept and accept/decline through *believed* `a_max` | R-O41 | open — this is where surprise attack comes from |
| 8 | Permissive role eligibility with varying competence | R-O44 | open — mostly a roles-§4 doc change plus item 1 |
| 9 | `FAIR_COUNTS` rejects 18 while galaxy §2 lists it fair | R-O12 | **done** — now `[2, 3, 6, 12, 18]`. A radius-`r` hex ring holds `6r` cells, so the family is 6/12/18/24…; 9 and 15 are multiples of 3 but form no ring, so `% 3` would be the wrong predicate. The three ring radii were exactly `N/6 + 1.5`, so the existing `18 => 4.5` branch was the family's third term and the list was one term short — replaced by that closed form. **Balance targets the 2-neighbour configs (3/6/12/18); N=2 is supported but not a balance target**, which also settles R-O9's missing Green as accepted rather than open |
| 10 | Seed roster LSV+LCV; default doctrine 100% LSV Scout | R-O42 | **half done, half blocked.** Seats are seeded with exactly LSV(Meadow) + LCV(Tor) per §7.1, and `SimConfig::enforce_roster` gates production on it — but it **defaults off**, because the engine has no card system and therefore no unlock path. Colonizer and freighter ride on MSV, which the starting roster excludes, so enforcement forbids every expansion build permanently: measured over 4,000 yr, **3 colonies and 18 vehicles against 1,183 and 4,778**. Pinned as a test. Blocked on cards, not on engine work |
| 11 | Derive `hull_dry_mass` from mineral cost | R-O57 | open — **coupled to 12**, see below |
| 13 | Slag as a bank entry | R-O59 | open |
| 14 | Magazine mass on ordnance families | R-O60/R-XM6 | open |
| 15 | `on_refit` retrofit realization | R-O47b/R-O55 | open |

**Items 11 and 12 are coupled and cannot land independently.** The verification
§9.2 asks for was run and the prediction **does not hold in this tree**: computed
from the shipped constants, empty-to-laden spreads are **2.00 : 1.50 : 1.33**
(Limited : Medium : General) — *narrowing* with hull size, the opposite of the
1.82 : 2.79 : 4.30 the doc cites. The cause is that neither half of the shell
model exists yet: `hull_dry_mass` scales with a size *tier* (1/2/3), a
volume-like proxy rather than surface area, and there is **no per-hull cargo
capacity at all** — `cargo_unit_size` is one flat constant, so a General hull
hauls what a Limited one does. A fixed load against rising dry mass is
necessarily a shrinking penalty.

So R-O57/R-O58 are a behavioural change, not a re-derivation: both halves must
move together (dry mass → area, capacity → volume), after which the per-class
propulsion the laser-vs-missile balance rests on needs re-certifying and
`tests/balance.rs` goldens re-deriving. That is also the *good* news for the
§7 flagged placeholder — R-O57 deletes `hull_dry_mass` as an independent number
rather than reconciling it.

### Also open

- Wire `matching.rs` into `lib.rs` and swap the call sites in `sim.rs`.
- Refactor three long functions: `sys_production_tick` (~149 lines), `apply_build`
  (~148), `production_choice` (~137).
- Recalibrate **`centrality_scale`** — still tuned to an old ~25 ly extent; the galaxy
  is now hundreds of ly. (`k_high` was the other half of this and is now **resolved**
  at 3.2 — R-AC17.)
- **Watch simulation throughput as fleets grow — optimize before it nears 2.5 yr/s.**
  The confirmed floor is **2.5 simulated-years/real-second**. `galaxy.rs` cites
  **2,116 yr/s** at the 12-player worst case (an 847× margin) and `Hyades_matching.md`
  §"Speed today" leans on the same figure — but both date from a galaxy that barely
  colonized. **Entity count is the first-order cost**, and it is now the thing that
  moves. Measured release-mode, seed 1, this machine:

  | scenario | vehicles | throughput | margin vs floor |
  |---|---|---|---|
  | *stalled config (no longer shipped)*, 3 seats, 4 kyr | 56 | 28,966 yr/s | 11,586× |
  | *stalled config (no longer shipped)*, 12 seats, 4 kyr | 205 | 3,037 yr/s | 1,215× |
  | **shipped defaults**, 3 seats, 4 kyr | 5,649 | 456 yr/s | 182× |
  | **shipped defaults**, 3 seats, 8 kyr | 9,501 | **79 yr/s** | **32×** |
  | **shipped defaults**, 12 seats, 4 kyr | 14,649 | 128 yr/s | 51× |
  | shipped defaults, 12 seats, 8 kyr | — | not yet measured | — |

  The first two rows are kept only as the historical baseline: they are the
  configuration design law #9 says never to benchmark against, and they are why the
  old 2,116 yr/s figure looked comfortable. Nothing was wrong with that measurement —
  it was taken on a game that stalled at a few dozen colonies.

  **Degradation is superlinear in duration, not just in fleet size.** 3 seats over
  8 kyr carries 1.7× the vehicles of the 4 kyr run but costs 5.8× the throughput,
  because the longer run also spends more of its life at the high-vehicle end. Seat
  count is gentler than that: 12 seats holds 2.6× the vehicles of 3 seats at the same
  horizon for 3.6× the cost, roughly linear.

  So the worst case is **long horizons, not wide tables** — which matters because
  full coverage needs ~8 kyr. 12 seats at 8 kyr is the untested corner; extrapolating
  the 8-kyr penalty onto the 12-seat row puts it near **20 yr/s, an ~8× margin**.
  Still above the floor, but the 847× headroom `galaxy.rs` advertises is gone.

  Two things to do:
  1. **`examples/bench_hex_size.rs` does not exist in this tree** despite being cited
     by `galaxy.rs` (×3), `tests/smoke.rs`, `tests/determinism.rs`, and
     `Hyades_matching.md`. Restore or rewrite it so the claim is checkable, and make
     it sweep *entity count*, not just hex size.
  2. Treat approaching 2.5 yr/s in testing as the trigger to **optimize, not to shrink
     the scenario**. First suspects are the O(P) scans that now run against 6,725
     planets rather than the 600 `Hyades_matching.md` assumed — `most_needed_center`
     per freighter load and the candidate scan per production cycle — which is exactly
     what wiring `matching.rs` in is meant to fix. Throughput is also a *search*
     problem: the balancer's value scales with runs per hour, so speed lost to entity
     count is balance coverage not bought.
- **Raise coverage inside a fixed 4,000-year run — do not extend the horizon.**
  The ratified defaults reach ~24% of colonizable worlds by 4,000 yr and need
  roughly 8,000 for 100%. **4,000 is the run length; the coverage reached within it
  is the objective to improve.** That reframes the offline search: it is looking for
  a configuration that compounds *faster*, not for a longer clock. It also keeps the
  search affordable — doubling the horizon doubles every trial, and §2's 60-second
  rule already had to absorb the snowball once. The `centrality_scale` recalibration
  above and R-SIM2 are the two nearest levers.
- Counter-graph matrix partition (card contract §8.3): define Red-class positions vs.
  Blue/Green-edge positions so the mineral substitution law has mechanical grip.
- **R-SIM1 — light survey view.** `Autopilot::choose_survey_target` takes
  `&[PlanetView]` (88 B/entry) and reads 28 B of it — `id` and `position`. Sizing the
  view to the query is the largest remaining per-ship win (`view_of` alone was 18% of
  engine instructions), but it adds a second view type to the fog-of-war contract
  (`Hyades_simulation_model.md` §1/§2b), so it is a contract decision, not a free
  optimization. Deliberately not taken yet.
- **R-SIM4 — departure-traffic confidence.** R-SIM3 settled that occupancy is
  *inferable at range*: the pop-4 industrial signature is implemented (exact, no
  new state), but the graded signal — repeated sightings of ships leaving a world
  raising confidence it is held — needs accumulated light-lagged observations per
  player per planet, which is exactly the storage §4 warns about at fleet scale.
- Open R-codes: R-ARENA1–7, R-MX1–6, R-XM5–7, R-SIM2 (survey scan cost), R-SIM4
  (departure-traffic confidence). Resolved this round: R-AC16, R-AC17, R-SIM1,
  R-SIM3.

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
