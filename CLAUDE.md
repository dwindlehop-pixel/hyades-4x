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

**`docs/Hyades_netcode.md` (Rev 4)** is the other spec that constrains engine
work rather than describing game content: it makes bit-reproducibility a
*network* property, not only an MC one, and its §2.1 is design law #15. Its
engine-status block lists the five implementation blockers, audited — and what
is already clean, which is most of the foundation.

**`docs/Hyades_politics_trade_and_intelligence.md` (Rev 1)** specifies the two
systems the Politics tree needs and neither of which exists: the Exchange with
`$`, and granular shared intelligence. Its §0 is the organizing thesis and worth
reading before touching anything in that tree — *eliminate the value of
collusion by making the simulation-state effects of collusion available without
a confederate*. That is why Politics cards are **not opt-in**.

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

Baseline as of the gradient-step ratification: **119 unit + 4 smoke + 5
determinism tests pass**, ~48 s wall for `cargo test --all-targets`.
The MC sweeps are slow in debug; always use `--release` for them.

### The 60-second rule for tests and CI

**Every test target and every CI step must finish in ≤60 s.** Searches are the
only exception and they are offline, never in CI. Current costs:

| step | cost |
|---|---|
| `cargo test --all-targets` (unit + determinism + smoke) | ~48 s |
| `tests/balance.rs` (release, `--ignored`) | ~52 s |
| `coverage_trace` | ~19 s |
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

**In an ephemeral container (Claude Code on the web), a backgrounded job dies
with the container, and that happens on no schedule you control.** Three
consecutive `min_time_search` runs were killed at 3, 20 and 7 minutes in. So:

- **Check liveness by file mtime, never by `pgrep -f <pattern>`** — the pattern
  matches the checking command's own bash line and always false-positives. That
  produced two confidently wrong "still running" reports before it was caught,
  and CLAUDE.md had already recorded the same trap once (the `pkill -f` incident
  above). Compare `ls -l --time-style=+%H:%M:%S` against `date`.
- **Anything over ~10 minutes should be run locally**, which is what §7 already
  says about `min_time_search` being a by-hand job. Cutting sample count buys
  some room but does not fix it — a 45-minute run still lost the race.
- Prefer harnesses that **flush per row and print a running best**, so a
  truncated run still yields the rounds that finished. Coordinate descent has
  this property naturally; keep it.

### How to search — measure gradients, not grids

**The goal is to learn how to tune, not to produce a tuning.** A ratified number
is worth one parameter; a method that says *which* parameters matter and by how
much is worth all of them, and survives every change to the objective.

`examples/gradient_probe.rs` is the harness. Four techniques, and the order
matters because each one makes the next affordable:

1. **Common random numbers.** Evaluate every configuration on the *same* seeds.
   Seed noise here is enormous — the four-seed bed spans 31.8% to 43.8% at one
   configuration, a ±2.7 point standard error — so comparing means from
   different seeds mostly measures seed. Comparing seed-by-seed cancels it. Free,
   and the highest-leverage line in the file.
2. **Paired central differences.** `f(x(1+δ)) − f(x(1−δ))` on matched seeds.
   Central for `O(δ²)` truncation error at the same two evaluations; *paired*
   because under CRN the difference has far lower variance than either level.
3. **Elasticity, not slope.** `∂f/∂ln x`. Raw slopes in different units cannot
   be ranked against each other; the log-derivative can, and **the ranking is
   the transferable knowledge.**
4. **A standard error on every number.** Anything inside 2 SE of zero is not a
   finding. This is the check that would have caught three of this project's
   four measurement artifacts on the day they were made.

Cost is `2 × params × seeds` — 72 evaluations for nine knobs, against 180 for a
five-value coordinate sweep that yields no gradient, no error bar and no
ranking.

**What a gradient cannot do.** It is local: it says which way is uphill *here*,
not where the summit is, and it points confidently along artifacts when the
model beneath is wrong. Use it to choose what to investigate, never to conclude.

### Screen on a truncated horizon, confirm on the objective

The coverage objective costs a **full-length run per evaluation** — 27.3 s at
the shipped defaults, so the nine-knob probe above is a 40-minute job. But
`horizon_years` is purely a stopping condition, so a truncated run is a
faithful *prefix*, and cost is violently superlinear in duration because
entity count compounds:

| horizon | cost | speedup | rank agreement with the real objective |
|---|---|---|---|
| 4,000 (the objective) | 27.3 s | 1× | — |
| 2,000 | 0.88 s | **31×** | ρ = 0.923 (healthy-band 0.859, worst seed 0.907) |
| 1,500 | 0.36 s | **76×** | ρ = 0.833 (healthy-band 0.831) |
| 1,000 | 0.12 s | 232× | ρ = 0.358 — too early, do not use |

**`colonies@2000` is the default screen** (`examples/proxy_metric_calibration.rs`);
`colonies@1500` is the aggressive option when the search stays among working
configurations, which is the gradient-probe case. **Screen with it, ratify on
the real objective** — never ship a value the proxy alone chose.

Three things that measurement got right, and are the reusable part:

- **Rank configurations, not seeds.** ρ is computed *within* each seed and then
  averaged. Correlating across seeds would only prove both metrics can tell an
  easy galaxy from a hard one — the same reason CRN pairs by seed.
- **Score the healthy band separately.** The config set contained `k_high`
  collapses (0.3–15% coverage against ~50%), and a metric can post a fine
  overall ρ purely by spotting those while being useless at ranking two
  *working* configurations — which is what a search actually does all day.
  `log_slope` was exactly this: ρ = 0.573 overall, **0.215** on the healthy
  band. A collapse detector wearing a proxy's clothes.
- **Re-test the knob that burned you.** Every candidate carries a
  `medium_fleet_size` sign test, because that is the knob whose backwards
  ranking disqualified years-to-10%-colonized.

**Time-to-threshold metrics are the trap here.** Years-to-10%-colonized fails
*both* halves of the bar: 10% is not reached until t≈2,800 of 4,000, so reading
it costs a full run anyway, and it ranked `medium_fleet_size` against coverage.
A metric you can only read near the horizon is not a shortcut to the horizon.

### The artifact pattern — four of them, one shape

Four measurements in this project were wrong in the same way, and the shape is
worth recognising because none of them looked wrong:

| What was measured | What it actually was |
|---|---|
| `medium_fleet_size = 8` optimal, 12 a "cliff" | the capacity normaliser going to zero |
| `coverage_trace`: this knob "DID move it" | two of three sample points degenerate |
| `coverage_time`: cheaper colonizers at 6.0 | a General hull holding ~700× a Medium's |
| coverage "wants" a cheaper Medium hull | `cap_Medium` pinned by a live normaliser |

Every one produced a plausible number from a broken configuration, and every one
was invisible in the objective — because the quantity that broke was in a
*denominator* that the shipped autopilot never exercised. Two habits follow:

- **Assert ratios, not just numerator and denominator.** Design law #3's
  inversion survived because cost and capacity were each individually ratified.
- **A parameter that reaches the objective through a derived quantity cannot be
  swept alone.** Since R-O58 the cost ladder *is* the capacity ladder; sweeping
  one leg of it in isolation is measuring two things and reporting one.

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
| `src/belief.rs` | **believed kinematics** (R-O41) — one-sided `a_max` estimate from light-lagged observations, and the accept/decline predicate that runs on it |
| `src/cards.rs` | the **card layer** — 18 tier-0 placeholders (3 slants × 6 trees), `Order`, and the coerce-never-reject rule |
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

   **This law was silently false in the engine until R-O58, and the way it failed is
   the thing to learn from.** Cost was on area (correct) but capacity was the
   *abstract slot count* 0/1/2 from roles §6, which is near-linear — so a General hull
   cost 9× a Limited and hauled 2 units where a Medium cost 3× and hauled 1: **0.100
   against 0.067 per unit hauled, i.e. fragmenting was cheaper.** Neither half looked
   wrong on its own, and both were individually ratified. A law about a *ratio* is not
   checked by checking its numerator and its denominator separately, so assert the
   ratio — `shell_model_ladders_are_derived_not_tuned` now does.

   The shell model also supplies the counterweight this law says must come from
   combat, without combat: a laden General hull is dramatically slower than an empty
   one and a laden Limited hull barely differs, so **large hulls broadcast their load
   state and small ones do not** (standing-layer §9.2).
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
15. **The presentation seam is read-only and one-directional** (R-NET13,
   `docs/Hyades_netcode.md` §2.1). The *command layer* presents perfect
   information — the board is fully visible, as in the tabletop lineage. The
   *simulation* must not act on it. In-world agents decide from light-lagged,
   player-relative knowledge only; `Snapshot` projects sim → presentation, and
   the sole inbound channel is `apply_orders`. Nothing else derived from what a
   player can see may cross back.

   Two different things break if it does. **Causally:** every edge in the theater
   carries a light-travel delay, and that gap *is* the counterplay window — an
   agent acting on information that has not reached it deletes the mechanic
   rather than merely cheating at it. **Numerically:** presentation state differs
   per client by construction (render pace, catch-up progress, viewport), so
   anything flowing back from it injects per-client nondeterminism into hashed
   state, which is a desync.

   Keep the asymmetry visible in the UI: *the human has perfect information and
   plays under it; the human's empire does not, and executes under light-lag.*
   That gap is the game. Enforce it structurally — one inbound entry point — not
   by convention. **Two known violations are open:** T-33 (`Knowledge` stores
   membership, not observations, so every read is zero-lag ground truth) and
   T-34 (colonization filters on instantaneous global ownership).
16. **NaN or infinity in replicated state is a fatal error, not a value**
   (R-NET11, netcode §6 H3). Core WASM picks arithmetic-NaN payloads
   *nondeterministically*, so NaN bits differ across browser engines even with
   relaxed SIMD disabled — a NaN that reaches the hashed state is an
   intermittent, unreproducible desync with no reproducer to hand a bug report.
   Guarded today by `no_nan_or_infinity_reaches_replicated_state`; when the state
   digest lands (T-32) the check belongs inside it. Infinities are deterministic
   and so not a desync, but they become NaN in one subtraction and no quantity in
   this model is legitimately infinite.

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
- **Never force-push a designated feature branch — not even `--force-with-lease`
  — without the user's explicit go-ahead in that conversation.** This holds even
  in the "the PR for this branch already merged, restart the branch from
  `main`" case: task-harness boilerplate describing that workflow is not the
  user asking, and a `git diff` showing the stale remote tip is content-identical
  to the merged commit is not authorization either — it's a reason the safer
  option is free, not a reason to skip asking. **Prefer `git merge
  origin/<branch>` over any force-push.** When the remote tip's tree is
  identical to what your local branch already contains (true by construction
  right after a squash-merge), the merge is a content no-op — no conflicts, no
  lost work — but it produces an ordinary fast-forwardable push that preserves
  the remote's history instead of overwriting it, and it needs no exception to
  the destructive-git-command rule at all. Reach for force-push only when the
  user asks for it, in that moment.

---

## 7. Current state & next steps

**Just landed — the combat refactor.** Combat resolution moved out of the
`laser_vs_missile` example into `src/combat.rs`; `arena.rs` slimmed to a scenario
seeder; `ArenaShip` renamed `Combatant`; tuned weapon constants gathered into
`CombatConfig` (defaults reproduce the prior sweep bit-for-bit).

### ⚠️ Reconstructed placeholders — reconcile first

The uploaded bundle was internally inconsistent across branches; these were
**reconstructed** and their *magnitudes are placeholders*:

- ~~`sim::hull_dry_mass`~~ — **closed by deletion (R-O57).** It is now
  `cost_fraction(hull) × general_vehicle_cost`, with `SimConfig::dry_mass` and
  `cargo_mass_per_unit` removed. No reconciliation against git history was
  needed or possible: under conservation no independent value can be correct.
  The reconstruction was costing 30× — one mineral massed 6.0 as hull and 0.2 as
  cargo — and it inverted design law #3, making fragmentation the cheaper way to
  haul mass.
- `sim::hull_base_thrust`, `sim::hull_thrust_multiplier_range` — still
  placeholders, but **no longer load-bearing for absolute mass.**
  `Combatant::max_accel` divides thrust by dry mass and `hull_base_thrust` is
  thrust-to-mass × dry mass, so the mass scale cancels exactly; empty-hull accel
  depends only on `hull_thrust_to_mass`. Pinned by
  `combat_acceleration_is_untouched_by_the_dry_mass_rebasing`, and confirmed by
  `tests/balance.rs` reproducing its goldens bit-for-bit across the R-O57/R-O58
  landing. **New: R-O65** — the 1.2/1.1/1.0 Systems ladder in
  `hull_thrust_to_mass` contradicts the shell model, which says empty-hull accel
  is size-independent. Not flattened, because it is an MC-tuned combat surface;
  needs ratification.
- `math::Vec3::cross`, public `sim::role_hull_type`
- `examples/combat_arena.rs` referenced `SimConfig::general_fleet_size` (absent here);
  substituted `1.0`, since General is the cost reference.

**The remaining two set the absolute ROU acceleration the laser-vs-missile
balance rests on.** If the prior definitions exist in git history, restore them
and re-certify before building on top.

### Next: R-MC9c (the active workstream) — `hyades_todo.md` T-12

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
Status, so this is not re-derived each time. **Six have landed** (1, 2, 5, 8, 9,
11+12); the eight still open are carried in `docs/hyades_todo.md` under their
T-codes, and item 10 is blocked rather than open:

| # | Item | R-code | Status |
|---|---|---|---|
| 5 | Colony cargo mass ≡ mineral cargo mass | R-O32 | **done** — `laden_accel` now masses `pop_cargo`; it was massless, so a laden colony ship flew like an empty hull and the burn read out cargo *type*, the one thing §6.2 exists to hide |
| 12 | Re-base hull mass on surface area (shell), contents on volume | R-O58/R-O58b | **done** — landed with 11; see below |
| 1 | `BuildOrder::Hull { hull_type, class }` + role assigned after production | R-O29 | **done** — the three mission-named variants are gone; `Autopilot::assign_role` returns a `Tasking { role, target }` for the finished hull, and the old `MiningPair`'s freighter is now a consequence of assigning `Role::Miner`. **Behaviour-neutral**, verified by stashing the diff: seed 1 / 3 seats / 4 kyr gives 1,183 colonies, 1,594 miner taskings, 5,845 scanned, 240 scouts both with and without |
| 2 | Design/roster component | **R-O28** | **done** — `Roster` (a sorted, idempotent set of `(HullType, Class)`) is a per-player component written only by tree cards. Unblocks σ_vector for Design: the distance between pre- and post-card rosters is now computable. `Class` also introduces the Banks-convention design names (R-O42b: Meadow/Tor proposed, flavour subject to authorship) |
| 3 | Diplomatic fields on `Doctrine` | R-O27/R-A3 | open — no field list specified yet (**T-11**) |
| 4 | Throttle fraction; observe `a` from trajectory not the stat block | R-O40 | open (**T-09**) |
| 6 | `min_time_search` as a reachability-cone query | R-O31 | open — same function, reverse direction (**T-05**) |
| 7 | Route intercept and accept/decline through *believed* `a_max` | R-O41 | **half done** — `src/belief.rs` holds the one-sided estimator (belief is the *max* ever observed, because a ship never flies above peak) and the kinematic accept/decline. Belief is monotone, so masking is spend-once; a 4,851-case sweep pins that it errs only by optimism, which *is* the surprise attack. **Sim wiring blocked on T-30** — there is no accept/decline site in the engine yet (**T-10**) |
| 8 | Permissive role eligibility with varying competence | R-O44 | **done** — roles §4 now states the permissive rule once and every per-role list reads "Competent:", separating *competence* (a degree — an LSV scouts badly) from *capability* (a fact — a Limited hull has no cargo hold, so a Limited Colonizer founds nothing). Engine matches: `assign_role` declines on no viable target, never on hull type |
| 9 | `FAIR_COUNTS` rejects 18 while galaxy §2 lists it fair | R-O12 | **done** — now `[2, 3, 6, 12, 18]`. A radius-`r` hex ring holds `6r` cells, so the family is 6/12/18/24…; 9 and 15 are multiples of 3 but form no ring, so `% 3` would be the wrong predicate. The three ring radii were exactly `N/6 + 1.5`, so the existing `18 => 4.5` branch was the family's third term and the list was one term short — replaced by that closed form. **Balance targets the 2-neighbour configs (3/6/12/18); N=2 is supported but not a balance target**, which also settles R-O9's missing Green as accepted rather than open |
| 10 | Seed roster LSV+LCV; default doctrine 100% LSV Scout | R-O42 | **half done, half blocked.** Seats are seeded with exactly LSV(Meadow) + LCV(Tor) per §7.1, and `SimConfig::enforce_roster` gates production on it — but it **defaults off**, because the engine has no card system and therefore no unlock path. Colonizer and freighter ride on MSV, which the starting roster excludes, so enforcement forbids every expansion build permanently: measured over 4,000 yr, **3 colonies and 18 vehicles against 1,183 and 4,778**. Pinned as a test. Blocked on cards, not on engine work (**T-25**) |
| 11 | Derive `hull_dry_mass` from mineral cost | R-O57 | **done** — landed with 12; see below |
| 13 | Slag as a bank entry | R-O59 | open (**T-03**) |
| 14 | Magazine mass on ordnance families | R-O60/R-XM6 | open (**T-04**) |
| 15 | `on_refit` retrofit realization | R-O47b/R-O55 | open (**T-08**) |

**Items 11 and 12 landed together** — they were coupled, and the coupling was
real: moving either half alone leaves mass unconserved.

The shell model is now the engine's, with **one geometric primitive and no new
tunable**. Radius is *derived* from the cost ladder (cost ∝ surface area ⇒
`r = sqrt(cost / cost_Limited)`), the Limited hull is the unit radius so it is
all shell and no hold, and capacity is the usable interior `(r − 1)³`
normalised to the Medium hull. Dry mass is the mineral cost. Two constants were
**deleted** (`SimConfig::dry_mass`, `cargo_mass_per_unit`) and none was retuned;
`cargo_unit_size` keeps its name, default and meaning as the reference hold.

Two faults in the shipped tree are what the change actually fixes, and both are
worth remembering because neither was visible from the code alone:

- **A mineral massed 30× more as hull than as cargo.** `dry_mass = 1.0` × a
  Medium tier of 2 made an MSV mass 2.0 while costing 1/3 of a mineral — 6.0
  mass per mineral — against `cargo_mass_per_unit = 0.2` for the same mineral in
  a hold.
- **Design law #3 was inverted.** Cost per unit hauled was 0.067 (Medium) vs
  0.100 (General), so *fragmenting* was the cheaper way to move mass. It is now
  0.067 vs 0.0098.

**Combat needed no re-certification, contrary to the earlier warning here.**
`Combatant::max_accel` is `hull_base_thrust · factor / hull_dry_mass` and
`hull_base_thrust` is thrust-to-mass × dry mass, so the mass scale cancels
exactly. `tests/balance.rs` reproduces its goldens bit-for-bit on all three
seeds and all five relative velocities; a unit test pins the cancellation.

**What it cost: ~9% of coverage at 4,000 yr** (seed 1 1,183 → 1,044; seed 7
1,164 → 1,093). Laden ships now pay for their load — a full Medium freighter
carries 15× its own dry mass and accelerates at 1/16 g against 2/3 g before —
so logistics is slower. That is conservation being real, and recovering the 9%
is a job for the coverage objective, not a reason to soften the physics.

**Opened by the landing:** R-O64 (roles §6's 0/1/2 was a *unit count*, not a
mass ladder — ordinal content kept, magnitudes now geometry) and R-O65
(`hull_thrust_to_mass` still varies 1.2/1.1/1.0 across Systems sizes, which the
shell model says should be flat; not flattened, it is MC-tuned).

### Also open — see `docs/hyades_todo.md`

**The full register of outstanding work lives in `docs/hyades_todo.md`**, ordered
specific → vague with permanent `T-nn` identifiers. Do not maintain a second copy
of it here; cite the T-code. What stays in this file is only the material that
changes how you *work*, not what is left to do:

- **Throughput floor: 2.5 simulated-years/real-second** (T-24). `galaxy.rs` cites
  **2,116 yr/s** at the 12-player worst case and `Hyades_matching.md` §"Speed today"
  leans on the same figure — **both date from a galaxy that barely colonized, and
  both are now wrong by orders of magnitude.** Entity count is the first-order cost
  and it is the thing that moves. Measured release-mode, seed 1, this machine:

  | scenario | vehicles | throughput | margin vs floor |
  |---|---|---|---|
  | *stalled config (no longer shipped)*, 3 seats, 4 kyr | 56 | 28,966 yr/s | 11,586× |
  | *stalled config (no longer shipped)*, 12 seats, 4 kyr | 205 | 3,037 yr/s | 1,215× |
  | **shipped defaults**, 3 seats, 4 kyr | 5,649 | 456 yr/s | 182× |
  | **shipped defaults**, 3 seats, 8 kyr | 9,501 | **79 yr/s** | **32×** |
  | **shipped defaults**, 12 seats, 4 kyr | 14,649 | 128 yr/s | 51× |
  | shipped defaults, 12 seats, 8 kyr | — | not yet measured | — |

  The first two rows are kept only as the historical baseline: they are the
  configuration design law #14 says never to benchmark against, and they are why the
  old 2,116 yr/s figure looked comfortable. Nothing was wrong with that measurement —
  it was taken on a game that stalled at a few dozen colonies.

  **Degradation is superlinear in duration, not just in fleet size.** 3 seats over
  8 kyr carries 1.7× the vehicles of the 4 kyr run but costs 5.8× the throughput,
  because the longer run also spends more of its life at the high-vehicle end. Seat
  count is gentler: 12 seats holds 2.6× the vehicles of 3 seats at the same horizon
  for 3.6× the cost, roughly linear. So the worst case is **long horizons, not wide
  tables**, and 12 seats × 8 kyr extrapolates to ~20 yr/s — an ~8× margin, not 847×.

  **Treat approaching the floor as the trigger to optimize, not to shrink the
  scenario.** Throughput is also a *search* problem: the balancer's value scales with
  runs per hour, so speed lost to entity count is balance coverage not bought.

- **Coverage is measured inside a fixed 4,000-year run — do not extend the horizon**
  (T-20). **~51.4% of colonizable worlds** as of the `growth_rate` re-ratification
  (mean over the 4-seed CRN bed; was 1,044 / 6,725 on seed 1 three ratifications
  ago). 4,000 is the run length; the coverage reached within it is the objective.
  Doubling the horizon doubles every trial, and §2's 60-second rule already had
  to absorb the snowball once.

  Three ratifications got it there and none was a sweep. **λ (14.4% → 38.3%)
  was a *missing term*** — freighter routing had no distance component at all,
  so a hauler would cross the galaxy for a marginally needier center. **The
  gradient step (38.3% → 49.3%)** moved four knobs at once along a measured
  direction and verified the result on the same seeds. **The `growth_rate`
  re-ratification (49.1% → 51.4%, +2.31 ± 1.05 paired)** came from re-measuring
  the gradient *at the point the previous step had produced* — the old direction
  had been consumed, and every knob but one had gone flat or noisy. Before
  sweeping harder: check whether a term is absent; then measure the gradient
  rather than the grid; then **re-measure it after you step**, because the
  direction you just spent is not the direction you are now standing in.

  **Diminishing returns are visible and worth reading.** +23.9, then +11.0,
  then +2.3 points. The remaining knobs are flat, noisy, or at their peaks
  (`medium_fleet_size` now measures `+1.93 ± 2.37` — it is *on* the hilltop),
  so the next real gain almost certainly needs a new *term* or a new
  *mechanism*, not another step along this ray.

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
