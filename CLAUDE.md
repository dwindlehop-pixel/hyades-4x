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

**`docs/Hyades_industry.md` (Rev 1)** is the newest spec and the one that
**amends the planet model**: `K = min(hab, bio_max)` — infrastructure leaves the
carrying-capacity minimum and becomes an industrial stock that mines and
fabricates. Read §1 before touching anything economic, because it invalidates two
measured results on purpose (R-O76's founding-infrastructure finding and T-57's
ratified crew size) and says why in each case. It also carries the **layering
algebra** for how Design and Doctrine writes compose (§6) — products, simplices
and rotations, with commutativity as the acceptance test.

**Its §8.1 outranks everything else in it and is not about industry: refined mass
traverses real space.** Minerals, supers and apex cross the theater on hulls,
under light-lag, where they can be attacked, diverted, stolen and blockaded — so
a trade is a *voyage*, not a ledger entry, and the Exchange settles into a
freight leg. Every price gradient and every market in the design is downstream of
it; take traversal away and piracy, theft, blockade and conquest become flavour
text. Nothing else in the document is Monte-Carlo ratified; every magnitude is a
flagged placeholder.

**`docs/Hyades_trees_and_card_value.md` (Rev 1)** is what to read before
touching a card, a tree, or a balance measurement. Two things in it change how
you work. **The tone is satire** — every winning game is the winner's own story
about how love won, told sincerely, with the player registering the gap; §1 names
the target and forbids winking, and it *amends* `Hyades_galaxy_and_autopilot.md`
§7's deliberate ambiguity about the Beloved Republic. And **there are six
objectives, not one**: the single global colony-count objective every
ratification so far has used is correct for Expansion and actively misleading for
the other five trees, four of which had a live metric farm in their obvious
formulation (§2.5). Card value is the **fractional reduction in the doubling time
of its own tree's stock**, measured at earliest legal play, designed to the 92nd
percentile.

**`docs/Hyades_politics_trade_and_intelligence.md` (Rev 2)** specifies the two
systems the Politics tree needs: the Exchange with `$`, and granular shared
intelligence. Its §0 is the organizing thesis and worth reading before touching
anything in that tree — *eliminate the value of collusion by making the
simulation-state effects of collusion available without a confederate*. That is
why Politics cards are **not opt-in**.

**There is one spec per tree, and each carries only ratified and open
decisions.** `Hyades_autopilot_colonization_growth.md` (Expansion + Growth,
Rev 4), `Hyades_production_tree.md`, `Hyades_technology_tree.md`,
`Hyades_warfare_tree.md` and the politics spec above. The three new ones are
Rev 1 and are mostly `OPEN` **on purpose** — Technology has no objective at all
until `Q_i` is instrumented, and Warfare is blocked on T-30's missing
accept/decline site — so read their registers before assuming a question is
unasked.

**`docs/Hyades_experiments_appendix.md` is where the measurement record lives.**
Nothing in it is normative. It holds the runs, the refuted hypotheses and the
superseded design that used to be inlined in the specs, each linked from the
decision it supports, and its §C collects all seven measurement-artifact shapes
in one table. **Check it before re-opening a question** — several were closed by
a measurement whose bed no longer exists, and the entry says so.

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
| `cargo test --all-targets` (unit + determinism + smoke) | ~85 s (unit target ~35 s) |
| `tests/balance.rs` (release, `--ignored`) | ~52 s |
| `coverage_trace` | in the slow job; ~30 min for the whole `balance` job post-T-68 |
| ~~`coverage_time`~~ | **out of CI** — ~8 min, *and* its doctrine comparison is now vacuous (below) |
| `montecarlo` | 48 s — pinned to a 1,000-yr horizon at T-68; it was 6 full 4,000-yr runs and blew a 25-minute job budget |

**T-68 broke the `examples (fast)` job, and the fix was not uniform.**
`montecarlo` was six full-horizon runs to answer "does the engine run without
panicking", which needs seed breadth and not horizon — pinned to 1,000 yr and
the duplicated seed-42 run reused, 61 s → 48 s. `coverage_time` was **removed**
rather than trimmed, because trimming it would have preserved a comparison that
no longer compares anything: its second bed moves `medium_fleet_size`, which
T-56 stage 3c turned back into a pure price, and R-IND11's ablation B measured
colonizer hull price at −0.08%. Both beds now print identical coverage.
**Distinguish a check that got expensive from a check that stopped asking
anything** — the first wants fewer samples, the second wants deleting.

**The unit target went 97 s → 144 s at T-62, back to 59.5 s at T-64, then
87 s → 507 s → ≈55 s at T-68**, and no move was about the tests. T-62's cost was
hauling (`examples/haul_census`): the Banded mineral field multiplied ore-per-rock
by three orders of magnitude against a fixed freighter hold, so the round trips
multiplied from the first cycle. T-64 gave it back by taking the `ln`/`powf` out
of the growth step. **T-68 made `t_build` track hull mass** — a Medium hull went
from 10 yr to 3.0 — so centers decide three times as often and the entity count
follows. **T-69 did it again** — filling every berth amortizes `t_lead` across
slips, so a yard produces ~1.8x the hulls — taking unit 79 s → 168 s and
determinism 67 s → 102 s at once. The lesson every time: **when a test target
moves, look at what the simulation started doing, not at what the tests are
asking.** The remaining ratification is T-24's floor breach (R-O82).

**T-69's fix was one constant and one stride, and both were found by measuring
rather than reading.** `--report-time` is still nightly-only, so the tool is a
shell loop timing each test by name into a file — worth keeping, because the
answer was not where it looked:

- **`test_cfg`'s horizon was the whole unit target.** One constant, inherited by
  dozens of tests, at 600 yr. At **300** all 172 still pass and the target is
  **20.7 s** — and at 200 they *also* all pass in 17.6 s, which is how the trim
  is known to be safe rather than lucky. 300 ships: ~50% headroom above the
  point where anything binds, for 3 s. **Probe past the value you intend to ship
  and report where it actually breaks** — that number, not the one you chose, is
  what tells the next reader how much room is left.
- **`positions_never_exceed_lightspeed` was 97 s of a 102 s target**, and the
  sampling loop that *looks* expensive was not it: the stride is 7 yr, so it
  walks ~100 windows whatever the horizon, and `sim.run()` at 800 yr was the
  bill. 800 → 300 took it to **3.0 s** — a 32x saving for a 2.7x cut, which is
  T-24's superlinear degradation seen from the other side.
- **The trim guard is the reusable part.** Shortening a run until nothing is in
  flight leaves every assertion in that test vacuously true and the suite green.
  It now counts moving entities and fails below a floor — which **fired on the
  first attempt** (479 samples), and the answer was to sample the shorter
  timeline three times as densely rather than to lengthen it. Sampling is cheap;
  the run is not. **When you cut a horizon, assert that the mechanism still
  fires.**

**R-WAR9 did it a sixth time, and the guard caught it rather than a target
getting slow.** Flying the colonization leg at the rate its load implies (T-115)
made a laden Medium colonizer **4x slower over a short hop**, so `tests/smoke.rs`
at a 40-year horizon founded **no colonies at all** on the 2-seat arm — 5 and 3
mining outposts and nothing settled. That is the `colonies > 0` non-vacuity
guard doing exactly what it is for: **a change that slows the expansion loop is
the one that would otherwise leave a trimmed file green and testing nothing.**
Probed rather than guessed — **42 fails, 45 passes** — and 60 ships. Note the
direction: this is the first entry in this list where a horizon had to go **up**,
and the reason is that the mechanism got slower rather than the target getting
expensive. **Ask which of the two it is before reaching for the lever.**

**T-112 did it a fifth time and nobody checked**, which is the point of the
habit rather than a new lesson: pickets that never scrap plus a live engagement
layer put `tests/determinism.rs` at **69.08 s** against a 60-second budget, and
it landed that way because the horizons were not looked at in the same commit.
Measured at T-114 by building both revisions and timing the target on each —
67.36 s with T-113 applied, so the breach is inherited and not new. **When a
change raises entity count, time the test targets before you push, and time the
old binary too** — otherwise the next landing inherits the breach and gets
blamed for it.

**T-88 did it a fourth time, and it moved every target at once** — unit 34 →
110 s, determinism 37 → 267 s, smoke 22 → 149 s — because `cycle_years` 50 → 5
makes a simulated year cost ~10x the events. The fix was horizons again, and two
things about *how* are worth keeping:

- **Dividing every horizon by the event multiplier is wrong**, and the floor
  caught it. `full_run_reports_are_bit_identical` got its per-arm horizons cut
  10x to match the 10x denser event stream, and the 2-seat arm landed on **450
  events** against a floor of 1,000: event count is not linear in the horizon,
  because the early game has one center and the tick multiplier has nothing to
  multiply yet. Measured instead of scaled, the arms come to ~1,500 events each.
- **A horizon written out at four call sites is an edit waiting to go wrong.**
  `tests/smoke.rs` had `150.0` in four places, one of them inside a paired
  identity — changing one side gave 1,028 events against 10,574, which the
  identity caught immediately. It is now a single `SMOKE_HORIZON` constant. When
  a number is shared by a *paired* assertion, share the constant too.

**Then the unit target came back at 62 s anyway, and the horizon lever was
spent.** This is the case §"Reduce the galaxy before the horizon" was written
for, and the numbers are worth having because they are lopsided:

| lever | effect on the unit target |
|---|---|
| `test_cfg` horizon 60 → 45 → 35 → 25 | **62.4 s → 63.0 → 63.3 → 63.1** — nothing, and tests began failing at 25 |
| `test_galaxy`, planet count → 200 | **62.4 s → 4.1 s**, all 198 still passing |

**Fifteen times, from the lever that had already been named.** Measured in debug,
which is how tests run: a 2-seat 60-year run costs **2,382 ms on the full field
and 21 ms at 200 planets**, while galaxy *generation* is 5–8 ms either way. So
the cost was never the horizon and never the generation — it was per-planet work
inside the run, and no amount of shortening reaches it.

Two things to take from the shape rather than the numbers. **When a trim does
nothing, that is data**: three horizon cuts moving the target by under a second
said the model of the cost was wrong, and the right response was to measure where
the time went rather than to cut harder. And **a shared helper is what makes the
lever reusable** — 55 call sites said `Galaxy::generate(GalaxyConfig::new(n, s))`,
so one `test_galaxy(n, s)` converted them all and the next session can change the
planet count in one place.

**T-71/T-72 did it a third time in one session** — smoke 36 s → 68 s — and the
answer was the same shape: `all_fair_counts_run_and_expand` was **65.8 s of the
68** on its own, four seat counts at a 300-yr horizon inherited from when a
simulated year was cheap. Every assertion in the file is an invariant or a
paired identity, so **the whole target went to 150 yr and cost 17 s**. Three
targets, three sessions, one constant each time. The habit worth forming: **when
a change raises entity count, check the test horizons in the same commit** — the
cost lands there before it lands anywhere a user would notice, and the fix is
almost never in the assertions.

**T-68's 507 s was four tests paying in horizon for questions horizon does not
answer**, and finding that out took one measurement rather than a guess — timing
each suspect individually, after `--report-time` turned out to be nightly-only:

- **One cadence test was 437 s of the 507 on its own.** It bought extra round
  barriers with a 1,400-year run. Shortening the *cadence* instead of the horizon
  gives it **ten** barriers where it had four, for 6% of the cost — "cut samples,
  not the question", and here the sample was the wrong axis entirely.
- **Three paired-run tests assert an arithmetic identity** (logging is a side
  channel; the round layer is inert while everyone passes; same seed, same
  outcome) and pay *double* horizon for it. They now share `paired_cfg` at 250 yr,
  which is the same argument this file already makes for `tests/determinism.rs`.
- **Smoke went 500 → 300 yr** (87 s → 17 s). Every assertion in it is an
  invariant that holds at any horizon where expansion has started.

### Reduce the galaxy before the horizon

**Three sessions running, the answer to "this target got slow" was to cut a
horizon. That lever runs out, and it runs out badly** — cut far enough and the
mechanism stops firing, the assertions go vacuously true, and the suite stays
green while testing nothing. The `moving` guard in
`positions_never_exceed_lightspeed` exists because that nearly happened.

**The other lever is the scenario, and it is usually the right one.** Ask what
the assertion actually *reads*. That test asserts a property of
`math::position_along` — no entity moves faster than `c` — and reads ships in
flight and nothing else: no economy, no mineral field, no colonization. On the
standard bed it was paying for all three, and was **97 s of a 102 s target**.
`GalaxyConfig::planet_count` is a plain override, so:

| bed | horizon | in-flight samples | cost |
|---|---|---|---|
| standard | 800 yr | — | **97 s** |
| standard, trimmed | 300 yr | 479 — *below the guard's floor* | 3.0 s |
| **150 planets** | **800 yr** | **5,087** | **0.8 s** |

The horizon went back **up** and the test got 120x cheaper, with five times the
guard's margin. (60 planets gives 1,541 samples, 400 gives 13,682 at 2.1 s —
150 is measured, not guessed.) `tests/determinism.rs::tiny_galaxy` is the
helper.

**The rule: only tests that measure a gradient need to be comparable to each
other, and only those need the standard bed.** A test that pins a mechanism
wants the smallest galaxy that exercises it. Keep one shared bed for the
Monte-Carlo work — comparability across measurements is the whole value of it —
and stop treating it as the default scenery for everything else.

**What this trades, so it is not discovered later:** a smaller field exercises
every code path fewer times, so it is worse at catching a bug that only appears
at scale — an ordering fault in a large collection, say. That is a real cost and
it is why `full_run_reports_are_bit_identical` still runs full-size galaxies
across all five seat counts (R-NET14: a divergence at *any* seat count is a
desync). Shrink the scenery on tests that assert a mechanism; leave it on tests
whose question *is* scale.

### Measure the per-event budget, not only the aggregate

**`yr/s` is a rate over a population the change re-selects, and it cannot tell
two different problems apart.** A change that makes every event slower lowers
it. A change that raises events-per-simulated-year lowers it too, with every
event costing exactly what it did. Those want opposite fixes.

**T-69 is the worked example and it read backwards.** Throughput fell 27% while
vehicle count *fell* — the table's standing lesson ("entity count is the
first-order cost") pointed the wrong way, because the cost was **decision
count**: every commit schedules its own `BuildDecision`, so a yard with `k`
berths raises `k` events where it raised one.

This is §2's **seventh artifact shape** — an aggregate that moves against its
parts because the treatment changed the mix — applied to performance instead of
to colonies, and the prescription is the same: **report the mix beside the
mean.** `SimReport::events_processed` is already there, so `ns/event` costs a
division. `examples/work_years` prints it next to `yr/s`; do the same in any
harness that reports throughput.

**It paid for itself on the first run.** The standard bed reports **~174 µs per
event**, which reframes the whole throughput problem: the engine is not slow
because the design produces many events — that is design law #14 and is not
going away — it is slow because each event is expensive. T-52's `O(scanned)`
production candidate scan is the suspect, and that is a *profiling* job, not an
entity-count one. Years of this table have been read the other way.

**And the instrument has to be free, or it is measuring itself.** Every census
and ablation in this project reads a run with log categories enabled and
compares it against one that did not. `tests/telemetry.rs` bounds that at 5%;
measured across a 10x range of event counts the true cost is **below the
machine's own run-to-run variance** (ratios 1.001 / 0.974 / 1.020 at 15.6k /
59.2k / 148.6k events). Two of three land under 1.0, which is the honest signal
that this is a *bound* rather than an estimate.

**It flaked once, at five repeats, and the fix was samples and not the
threshold.** An unloaded machine ran the bed twice as fast, the per-pair ratios
spread **0.948 to 1.067**, and because the two arms are minimized
*independently* a small sample can pair a lucky bare run against an unlucky
logged one — reporting 1.069 for a cost that is actually zero. Nine repeats give
1.001 on the same bed. **A threshold widened to cover measurement noise stops
bounding anything**, which is the whole point of the test; if a timing guard is
flaky, buy more samples.

Read them together:

| `yr/s` | `ns/event` | what actually happened |
|---|---|---|
| down | flat | the simulation is doing **more**, each unit costs the same — a design change, not a regression |
| down | up | the engine got **slower per unit** — this is the one to profile |
| up | down | a real optimization |
| flat | up | events got dearer and fewer; something moved in both directions and needs decomposing further |

Only the second row is a performance bug. Treating the first as one is how an
optimization pass gets spent on code that was never the problem — which §4's
"profile before you optimize" already says, one level up.

**The table assumes a fixed workload, and T-111 is the case where that fails.**
Wiring combat into the sim moved throughput **up on all eight seeds** (109→126 …
124→137 yr/s) with `ns/event` **down** (20,556→18,060) — the third row, "a real
optimization". Nothing was optimized: the change destroys 5,488–12,819 mining
hulls per run, so there are fewer entities *and* a cheaper event mix, and the
simulation is simply doing less. **Before reading either column, ask whether the
change altered the amount of work rather than the cost of it** — a mechanic that
removes entities improves both and has optimized nothing.

The general form, worth having separately from the instances: **a test's horizon
is a cost, not a strength.** Ask what the assertion actually needs — an identity
needs none, a cadence needs periods rather than years, an invariant needs the
mechanism to have fired once. A horizon inherited from when runs were cheap is
the first thing to check when a target moves, and the last thing to defend.

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
- **But mtime only works if the harness has written something**, and a
  zero-byte file's mtime is its creation time. `colony_years` runs **two** full
  sims per seed and printed nothing until a seed finished — ~12 minutes at
  post-T-68 speeds — so it looked dead and was diagnosed as dead while running
  at 99.9% CPU. **`ps -C <name>` does not rescue you either:** these binaries
  are copied to a scratch name before being backgrounded, so the process is
  called `cy_t70` and `ps -C colony_years` matches nothing. Two habits follow,
  and the first is the one that generalises: **make every long harness print a
  flushed line before it starts work and after each expensive stage**, so mtime
  is a real signal; and when checking by process, match on `ps -eo comm` for the
  name you actually launched.
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

**Those costs are from three landings ago and are now wrong by two orders of
magnitude.** Re-measured after T-68 (`examples/horizon_cost`, seed 1, 3 seats):

| horizon | cost | colonies | % of the 4,000-yr total |
|---|---|---|---|
| 500 | 0.6 s | 261 | 7.8% |
| 1,000 | 11.4 s | 2,510 | 75.2% |
| 1,500 | 64.3 s | 3,269 | 97.9% |
| 2,000 | 123.2 s | 3,333 | 99.9% |
| 4,000 | ~400 s | 3,337 | 100% |

The ρ column above is **not** re-measured and should not be assumed to carry
across — it was calibrated on a bed whose expansion loop ran at a fraction of
this speed.

**Read the last column before choosing a horizon.** T-68 accelerated the
expansion loop enough that **colony count saturates by ~1,500 years**: the back
half of a 4,000-year run simulates a full galaxy at peak entity count to add
four colonies. Count stops discriminating there; colony-*years* keep accruing,
so the guard still wants the full run, but any search whose objective is count
is paying 3.3x for 0.1%. `horizon_cost` is the tool, it is not in CI, and it
should be re-run after anything that moves the expansion loop.

**`colonies@2000` is the default screen** (`examples/proxy_metric_calibration.rs`);
`colonies@1500` is the aggressive option when the search stays among working
configurations, which is the gradient-probe case. **Screen with it, ratify on
the real objective** — never ship a value the proxy alone chose.

**How wrong the screen can be, measured** (T-64's `growth_rate`,
`examples/growth_ratify`): the 2,000-year screen put `r = 1.35` at +11.20% and
`r = 1.90` at +22.94%; the 4,000-year objective put them at **+3.03% and
+6.19%** — an overstatement of ~3.7x, with the *ranking* preserved. So a truncated
horizon is a fine ranker and a bad estimator, and a "+23%" read off one is not a
result. Worse, the screen showed `r` as a **step function of itself** — 1.10 and
1.35 scored bit-identically, as did 1.60 and 1.90, because growth reaches the
objective only through how many 50-year cycles a center takes to cross a
`PopBands` edge — and **those plateaus were gone at the objective.** Step
structure is horizon-dependent, so a plateau map has to be run at the horizon
you intend to ratify on.

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

### Never let the metric's denominator be something the game can play

**The objective is an absolute colony count, not a fraction.** It used to be
`colonized / |{p : min(hab,bio) > 0.01}|`, and both halves of that were wrong:

- The **divisor** is a per-seed constant, so averaging fractions across seeds
  weights each seed by `1/denominator`. That is a weighting nobody chose, and
  it is not neutral — see below.
- The **membership filter** is derived from habitability, which is *exactly*
  what a terraforming card changes. Under a fraction, terraforming a world
  above the threshold **enlarges the denominator and lowers the score for
  colonizing more**; a bombardment that pushes one below **shrinks it and
  raises the score for destroying one.** The metric would report the opposite
  of the play. That is not a tuning inconvenience, it is a metric that can be
  farmed, and card design is the thing it would mislead.

Nothing in the shipped engine mutates habitability *yet*, so this looked
cosmetic. It was not — **correcting it reordered the top of the ranking**:

| knob | under the fraction | under colony count | confirmed? |
|---|---|---|---|
| `biosphere_regen_rate` | +0.84 ± 0.24 — third | **+141.2 ± 18.1 — first (7.8 SE)** | yes — *but see below* |
| `growth_rate` | +2.27 ± 0.50 — first | +73.8 ± 20.3 — second | yes |
| `survey_reserve` | −0.13 ± 0.23 — flat | −23.8 ± 10.1 — "significant" | **no — false positive** |

The top two genuinely swapped, and `growth_rate` was ratified while the
fraction under-weighted the actual top lever.

**`biosphere_regen_rate` is now bit-identically inert, and T-67 is why.** The
largest lever this project ever measured reports **exactly 0.0000** at ±10% on
the current bed (`examples/tree_gradient`). It is not a weakened effect, it is no
effect: since `K = min(hab, bio_max)` the ceiling is the *pristine* biosphere and
regrowth only refills the standing stock, which §7 already records as slack
(deleting the biomass draw outright reproduces the run bit-for-bit). The +141.2
is a true record of the engine that measured it and is not a fact about this one.
`center_mining_fraction`, `productivity_step` and `reinvest_bias` are flat the
same way. But **the third row is a sixth
artifact, and this one the corrected metric *created*** — a direct sweep of
`survey_reserve` (`gradient_step --sweep-reserve`) shows the probe had the sign
backwards: 1024 sits on a plateau (2048 is +3.5 ± 2.6, noise) with a cliff
*below* it (512 → −21.8, 256 → −96.8, 64 → −840). A ±10% perturbation on a
plateau read noise and cleared 2 SE by luck.

Two lessons, and the second is the more useful one. **A 2.4-SE reading on four
seeds is not a finding, it is a coin landing on its edge** — the 2-SE bar is a
floor for *considering* a knob, not a license to move it. And **"screen, then
confirm on the objective" caught this**, which is the entire reason the rule
exists: the screen proposed a direction, the confirmation refuted it, and no
default moved. A correction to the metric fixes some readings and can
manufacture others; both need the same confirmation step.

**It happened again at R-O87, and this time there is a cheap method that settles
it: replicate on seeds the candidate was not chosen against.** Sweeping
`reinvest_bias` against work-years produced `b = 0.972` at **+2.33% ± 0.96 on
the standard four-seed bed with 4/4 seeds positive** — which is 2.4 SE *and* a
1-in-16 sign test, and looks like a result. On seeds 2, 3, 5, 11 it scores
**−1.70% ± 2.42**, 1/4 positive. Pooled over all eight: **+0.32% ± 1.42.** Flat.

Four runs refuted it, and they are worth more than four more runs on the same
bed would have been: **a replication set cannot inherit whatever made the
original four agree.** More seeds on the bed the candidate was selected on
shrink the error bar around a number that was chosen partly *because* of those
seeds; a fresh set does not. Make it the last step before moving any globally
tuned default — `WY_SEEDS` in `examples/work_years` is the pattern.

Two corroborations were available and both are cheap enough to be routine.
**Sweep the neighbors**: `0.968` scored −0.20% ± 0.76 and `0.975` +2.24% ± 2.08
on the same bed, so adjacent values swing the full magnitude of the "effect" in
both directions — a chaotic reordering of a compounding run, not a gradient, and
no point on it is a place to stand. And **ask whether the knob has a mechanism
by which it could move the metric at all.**

**That last question has a right and a wrong way to answer it, and I got it
wrong first.** The answer I gave was an identity — deepening and founding buy
exactly the same works per mineral — and it is true, and it is about **stock**.
The knob's actual channel is **flow**: a colony is founded with a recycled hull
and cannot keep improving that way, so deepening buys build *rate* forever.
Priced off the engine's own functions that is **+29.0% hull/yr for nine
colonizers at rung I → II, paid back in 62 years** against a 1,500-year horizon.
The flat result had nothing to do with the identity.

**A flat objective means the knob's mechanism is throttled somewhere, and the
thing to measure is the utilization of whatever the knob buys.** Here the knob
buys yard throughput, and a homeworld's yard runs at **18.8% utilization**:
the gap from one production decision to the next is **1.5 yr after a committed
build and 29.6 yr after an `Idle`**, because a declined build schedules nothing
and waits out `cycle_years = 50`. **81% of the timeline is retry-wait**, so build
rate governs a fifth of it and +29% on a fifth is +5.5% at best. That is T-88,
and no objective would have found it — the number that did was a *ratio between
two gaps*, which no aggregate carries.

Three habits from it, and the first is the general one:

- **Before concluding a knob is neutral, measure how much of the time the
  resource it buys is even binding.** Utilization is one division and it
  distinguishes "this knob does nothing" from "this knob is fine and something
  else is in the way" — which want completely different next actions.
- **Separate stock from flow when you write the mechanism down.** An identity
  over what a purchase *is worth now* says nothing about what it *earns later*,
  and the second is usually where a compounding simulation's answer lives.
- **Split an aggregate by what preceded it.** "Mean gap between decisions" was
  6.7 yr and said nothing; the same data split by whether the previous decision
  built or idled gave 1.5 against 29.6 and named the mechanism outright.

**A zero in a multiplicative chain is not a small number, it is an absorbing
state (T-112).** A Warfare card was charged its honest price — a colonizer that
keeps its hull leaves the colony without the recycled stock roles §4.2 makes its
`Band I` infrastructure — by debiting the founding rung to zero. That took the
card's own player from **769 colonies to 10** and *raised* its neighbors 20%.
The line is `employment_rate`, which returns exactly `0.0` for a stock of zero,
and `fabrication_rate` is `slips × berth_rate`: the colony could never mine,
never build and never recover. **Before charging a cost against a stock, check
whether the stock multiplies anything** — a price that can reach zero on a term
that gates production is not a price, and the measurement it produces is of the
player deleting itself rather than of the mechanic. The ladder's floor rung
exists for this (design law #11/T-63).

**And the tell was in the split, not the total.** The neighbors' gain tracked
the card-player's loss one-for-one across three arms spanning 25 points — which
is what says "self-harm", where either number alone reads as "the card works on
somebody".

**The general rule, which is the transferable part:** an objective must be
invariant to everything the thing being optimized can change. Ask of any
metric — *what could a card do to move this without moving the world?* If the
answer is not "nothing", the metric is a target, not a measurement. This joins
the artifact list below as a fifth shape, and it is the only one that would
have gotten worse rather than better with time.

### Rank knobs against every tree, and normalize to the default

**Colony count is Expansion's objective and only Expansion's**
(`Hyades_trees_and_card_value.md` §2.1). Every gradient in this file was ranked
on it, and `work_years` has already caught it scoring a development regression as
an improvement four times running. `examples/tree_gradient` ranks against a
composite instead: **divide each tree's stock integral by its value at the
shipped default and take the geometric mean.**

Three properties, and the second is why it is worth the trouble:

- The default scores exactly **1.0** by construction, and units cancel before the
  average, so no tree's scale sets its weight.
- **`ln S` is the arithmetic mean of the per-tree log-ratios**, so the composite
  elasticity is *exactly* the mean of the per-tree elasticities. The
  decomposition is free and reconciles by construction — §2's mix rule satisfied
  rather than obeyed. Print the parts beside the whole.
- Everything is dimensionless (% per %), so knobs and *trees* are both comparable.

Three of the six trees are measurable (Expansion, Growth, Production); Warfare is
an algebraic zero on the 3-seat bed, Politics is Expansion exactly, Technology is
undefined — reasons in the harness docs and in trees §2.3. **Say which you left
out and why**; a composite over an unstated subset is worse than a single metric.

**First run, 32 knobs, 4 CRN seeds, 1,500 yr** — full raw per-seed dataset in
`data/tree_gradient.tsv` (T-50), ranking and reversals under T-45. Two findings
worth having here, then the traps, which are the transferable part:

- **Cross-tree conflicts are the thing the composite is for**, and they exist:
  `trade_decay_lambda` is +0.002 on Expansion and **−0.348 on Growth**, and
  `fab_cap` is **+0.069 on Expansion against −0.221 on Growth**. λ is the largest
  ratification in this project's history and it was measured on coverage alone.
  A single-metric probe cannot see either.
- **The defaults are on a local maximum in 18 of 32 knobs** — *both* arms score
  below 1.0. That is a result, and the elasticity ranking cannot express it.

### Six traps in reading a gradient

All six were live in this project's first composite run, and the first one is
the one that reorders the answer.

**1. `|∂f/∂ln x|` is not benefit. Rank by the gain you can actually take.**
A central difference averages the two arms, so a knob that is pure *downside*
ranks beside one that is pure *upside*. `cargo_unit_size` came **third of 32** by
|elasticity| and its upside is **−0.34%**: `S(+10%) = 0.997`, `S(−10%) = 0.484`.
There is nothing to win there, only something to lose. Report two columns —
`max(S(+δ), S(−δ)) − 1`, what the step can win, and the other arm, what it can
lose — and rank on the first. Under that ranking `cargo_unit_size` falls to
**22nd**, below six knobs that are bit-identically inert.

**2. Both arms below 1.0 is a local maximum, and it is a finding.** More than
half this bed's knobs are there. An |elasticity| ranking reports them as
"significant" in proportion to how *asymmetric* the peak is, which is not a
quantity anyone wants. Say "no gain either way" and move on.

**3. A central difference is meaningless on a stepped response — in *both*
directions.** The hold sweep, geomean against the default:

| `cargo_unit_size` | 0.90 | 0.95 | 0.98 | **1.00** | 1.05 | 1.10 | 1.25 |
|---|---|---|---|---|---|---|---|
| score | 0.490 | 0.488 | 0.494 | **1.000** | 0.987 | 0.983 | **1.047** |

A cliff below, then a *dip*, then +4.7% at +25%. A ±10% probe lands in the dip
and reports "no upside" for a knob that has 4.7% of upside one step further out.
So **neither** ranking is valid on a staircase: |elasticity| overstated it and
best-arm gain understated it. The tell is `S(+δ)` and `S(−δ)` being wildly
asymmetric; the fix is a sweep, not a better difference.

This also corrected a story that was half right. The three hull-ladder knobs
share a *cliff* — all three drop the Medium hold below the 0.900 kt rung
(R-O90) — but they do **not** share an upside: +19% of hold via the ladder's
geometry is worth **+7.8%**, while +10% of hold for free via `cargo_unit_size` is
worth **nothing**. "They are the same knob" was true of the collapse and false of
the gain, and only the two-column table separates them.

**4. Taking the better of two arms across many knobs is a selection-bias
machine.** `max` of two noisy arms is biased upward even when both are zero, and
reading the top of 32 of them compounds it. This is R-O87's trap in a new
costume, and the same fix caught it — **replicate on seeds the candidate was not
chosen against**:

| knob | move | orig (1,7,42,31337) | replication (2,3,5,11) | pooled n=8 |
|---|---|---|---|---|
| `medium_fleet_size` | −10% | +7.21% ± 1.31, 4/4 | +8.56% ± 0.79, 4/4 | **+7.88% ± 0.75, 8/8** |
| `general_vehicle_cost` | +10% | +7.79% ± 2.60, 4/4 | +7.81% ± 0.86, 4/4 | **+7.80% ± 1.27, 8/8** |
| `cycle_years` | −10% | +4.76% ± 1.95, 3/4 | +6.41% ± 1.41, 4/4 | **+5.58% ± 1.15, 7/8** |
| `growth_rate` | +10% | +2.18% ± 0.69, 4/4 | +0.97% ± 1.25, 3/4 | +1.57% ± 0.70, 7/8 |
| `trade_decay_lambda` | −10% | +1.23% ± 0.60, 3/4 | +0.34% ± 1.52, 1/4 | +0.78% ± 0.77, 4/8 |
| `rank.w_mineral` | +10% | **+1.44% ± 0.69, 4/4** | **−0.14% ± 1.22, 2/4** | +0.65% ± 0.71, 6/8 |

`rank.w_mineral` cleared 2 SE with a 1-in-16 sign test and is **refuted**. The
top three hold and their error bars *tighten* — which is what a real effect looks
like under replication, and is the cheapest way to tell one from a lucky max.

**5. A multiplicative step cannot move a knob whose default is zero.**
`risk_aversion = 0.0`, so `x(1 ± δ)` is `0.0` twice and the probe reports a
confident flat. A zero from a relative-step probe means *"no path to the
simulation"* **or** *"the knob is zero"*, and only one of those is about the
engine. Check before recording an inert verdict; an absolute step is the fix.

**6. "Exactly zero" and "inside noise" are different verdicts with different next
actions.** Six knobs here are **bit-identically** flat — the perturbed run
reproduces the base to the last bit, so the knob has no path to the simulation at
this operating point and more seeds cannot change that. Those are candidates for
*deletion*. `~noise` means a path too small to resolve at four seeds, and wants
more seeds. Printing both as "0.0" loses the distinction — and it is how
`center_mining_fraction` sat open for four sessions waiting for a bigger bed it
did not need (T-47).

### A rate is per *something* — check what, before you change the step

**T-88 swept `cycle_years` from 50 down to 1 and the objective reported +58.6%
work-years. It was mostly an artifact, and the tell was that it was too good.**

`growth_rate` is documented `1/cycle`, and the logistic stepped it once per tick
**regardless of how long the tick was** — as did `biosphere_regen_rate` and the
center's mining fraction. So shrinking the tick did not integrate the same
economy more finely, it ran a **fifty-times-faster one**. The sweep was measuring
its own step size.

Three things generalise:

- **Before sweeping a step size, audit every rate the step multiplies.** The `$`
  faucet in the same function was already written `rate × cycle_years` and was
  correct; three siblings beside it were not, and nothing in the types
  distinguished them because they are all `f64`. If a knob's doc comment says
  `1/cycle`, changing the cycle changes the knob.
- **Fix it by re-denominating, not retuning.** `tick_scale` multiplies each rate
  by `cycle_years / rate_reference_years`, which is exactly `1.0` at the cadence
  they were ratified at — so the change is bit-identical on the shipped bed and
  **no Monte-Carlo-tuned magnitude moves.** That is the R-O88 precedent and it is
  what makes a correction like this landable at all.
- **What survives the correction can still be large, and here it was.** With the
  denomination fixed, refining the tick is **+21.00% ± 4.19 work-years, 8/8
  seeds** — because `r·dt = 0.873` per step is a genuinely bad Euler step, stable
  under design law #11's `r < 2` bound and nowhere near accurate. A homeworld's
  population at 300 yr goes 1,143 → 2,275 → 3,516 → 4,297 as the tick goes
  50 → 25 → 10 → 5: the coarse step **under-integrates by ~4x**. Do not let the
  artifact discredit the question it was asked about.

**And the test for it asserts convergence, not invariance.** Those two failure
modes are indistinguishable in one ratio and obvious across three: an integrator
converging moves the answer *less* with each refinement (1.99, 1.55, 1.22), while
a rate applied per tick without scaling moves it by the step ratio every time,
forever. The first version of that test asserted invariance, failed at 2.92x, and
was wrong to — which is how the distinction got found.

**Then ask whether the step needs to be a step at all (T-94).** `cycle_years`
was refined because `r·Δ = 0.873` is a bad Euler step, and that was the right
call — but the logistic has a **closed form**, and using it makes the answer
*independent of the tick* rather than merely less wrong. Euler was 79% low at the
old tick and still **21% low** at the refined one. Two habits:

- **Check whether the thing you are refining has an analytic solution before you
  spend event count on it.** The tell here was that the engine already contained
  one: `settler_target` prices colonization off the exact logistic's inverse, so
  the policy and the economy were following different curves.
- **A better integrator and a finer step are not substitutes, and the sweep says
  which you are buying.** Exact at the *coarse* tick beats Euler at the coarse
  tick by +33% work-years at the same cost — real accuracy. But it is still 29%
  below exact at the fine tick, which says the step size was never only an
  integration step: it also quantises when a center mines, crosses a band edge
  and re-decides. **Refining a step that carries more than one job improves all
  of them, and fixing the integrator only pays for one.** Decompose before
  concluding the refinement is spent.

### A decision can be provably blind and fixing it change nothing

**T-90 is the worked example, and the diagnosis that produced it was mine, one
landing earlier.** `BaselineAutopilot::rank` scores a world's minerals as
`Σ_c scarcity_c · Band(m_c)`, and `scarcity_c` is written once at game start from
the homeworld archetype and never again — so outpost selection could say *mine
more* and never *mine **Cyan***. That is a real defect, it is visible in the
code, and it is **not** why 99.7% of banked ore cannot pay a rung.

Replacing it with the deciding center's live shortfall moved the mechanism check
from **0.043 to 0.043**, cost **−3.30% ± 0.49 colony-years on 0/4 seeds**, and
was reverted. Three habits, and the last one is the general shape:

- **Write the mechanism check down before you measure, and let it refuse the
  change.** The objective said +1.12% ± 4.31 — noise that could have been read as
  a small win on a tired evening. `bank_mix`'s payable fraction is a *direct*
  reading of the thing the fix claimed to move, and it said no. An objective
  answers "did anything change"; only a mechanism check answers "did the thing I
  described change".
- **Probe the gain across an order of magnitude before calling an axis inert.**
  "No effect at the shipped value" and "the axis does nothing" are different
  findings with different next actions, and four runs separate them: swept
  0 / 1 / 4 / 16, the payable fraction reads 0.043 / 0.043 / 0.057 / 0.045 and the
  dead share is 99.7% at every one. Without that sweep the honest write-up would
  have had to say "possibly undertuned" forever.
- **Check whether the thing upstream was ever short.** The premise was that
  color-blind selection mines the wrong mix. The empire's outpost holdings are
  **957k / 905k / 626k kt** across the three colors — already balanced. *The
  decision was blind and had nothing to see.* One census of the upstream stock
  would have refuted the diagnosis before a line was written, and it is the same
  question §2 already asks about knobs — measure whether the resource the change
  buys is even binding.

  **Ask it of a blocker in `docs/`, not only of a knob (T-111).** The Warfare
  spec said combat could not be wired in because there was no accept/decline
  site and nothing ever met. The round layer had shipped, and one census showed
  **68–75% of occupied sites already hosting more than one empire** at ~4,200
  contacts per run — mining is non-exclusive, so the engine had been producing
  co-locations since outposts existed. A blocker that has stood a long time is a
  *claim about the engine*, and it decays exactly the way a measurement does;
  the cost of re-checking one was a single run.

**And the reason no routing fix reaches it is structural, not statistical.** A
hold is filled from `outpost_stock[(player, rock)]` — one map entry — and a rock
is one color (0.789). So **every delivery is mono-colored by construction**, and
a bank is a sum of mono-colored lumps. Which rock, and which center the lump
goes to, are both choices *over indivisible single-source loads*:

> **If every unit of delivery is atomic in the dimension you need to mix, mixing
> is not a routing problem.**

Three interventions have now failed against that and one succeeded, and the split
is exactly along this line: T-81 changed where a hold goes (banks did not mix),
R-O89's pickup arm changed which rock it returns to (−52.3%), T-90 changed which
rocks are mined (0.043 → 0.043) — while R-O89's *load* leg changed the
composition **within** a hold and is the one that worked (+8.4%), bounded by what
the single rock holds. **Before optimizing a selection, check that the thing being
selected among can express the property you want.**

**The same question catches a *reduction*, not only a cargo hold (T-113).**
`commit_one_build` reduces the scanned pool to the per-class argmax before the
policy ever sees it (R-O70), and that reduction is **exact** for a consumer
reading the argmax of a class — the comment above it says so, and names the
consumers it was derived against. A new colonizer preference read the argmax of a
**subset** of a class (the worlds this empire's pickets hold), and `max(S)` does
not carry `max(S′)` for `S′ ⊂ S`: a held world reached the policy only when it
already won its class outright. The preference was inert by construction and
measured as inert — the arm reproduced the arm without it **to every printed
digit**. Widening the reduction to six slots (per-class winner, plus per-class
winner among held ground) moved it on the first run.

Two habits, and the second is the cheap one:

- **A reduction is exact only for the consumers it was derived against.
  Re-derive it when you add one.** Nothing in the types distinguishes "the best
  world" from "the best world with a property" — both are `Option<Candidate>`.
- **A behavioral change that reproduces the baseline *exactly* is inert or
  unreachable, not small.** An approximate match is a weak effect and wants more
  seeds; a bit-identical one is a structural claim, and the place to look is the
  **data the decision reads**, not the decision. This is §2's "exactly zero and
  inside noise are different verdicts" one level up, and it cost one run to read
  correctly instead of a bed.

**And then remove the atomicity, because that is the move the framing points at
and it produced the largest single result this project has measured.** T-91 let
one outbound leg visit two piles: **+55.13% ± 4.65 work-years, 8/8 seeds**, with
the mechanism check moving for the first time in four attempts (payable fraction
0.043 → 0.052). The sentence above is a diagnosis, not a dead end — *"mixing is
not a routing problem"* means change the **unit of delivery**, not give up on
mixing.

Three things from it that generalise past freight:

- **Cap a shared resource at its share, not at the need.** An intermediate stop
  takes each color capped at what is wanted **and** at its proportional share of
  the hold. The second cap is worth **+10.2%** on its own, because the bill is
  geometric in the rung: past the point where a bill outgrows a hold,
  `min(want, room)` *is* `room` and the first pile takes everything — so the cap
  is inert at today's magnitudes and load-bearing at the ones development
  actually reaches. Inert-now is not redundant.
- **Probe past the value you intend to ship, and report where it breaks.** One
  stop through six scored 184k / **284k** / 261k / 236k / 190k work-years. Two is
  a *peak*, which is a different claim from "two beats one" and is the one worth
  writing down.
- **"Check whether the resource is binding" needs the word *which*.** Mass was
  plainly slack — the empire banks 360,000 kt it cannot spend — and freight
  carries **1.73%** of everything that ever enters a bank, the other 98.3% being
  the center mining its own single-colored planet straight into its own bank. By
  the usual reading that channel is far too small to matter. It was worth +55%,
  because a **conjunction** makes the *minority* component the whole constraint:
  1.7% of the mass carried 100% of the scarcity. An aggregate that is slack can
  contain a component that is not, and a conjunction is the tell.

### A signal that outruns what it warns about is not a race

**T-115 is the worked example, and it invalidated a mechanic by implementing it
correctly.** T-112 turned a colony ship back when a warning beat it to the
target, and scheduled that warning by distance to the ship's *home center* —
one responder. Adding the second responder the observation model implies (the
crew notices the picket itself, and a closing hull meets the wavefront sooner
than a standing observer) made the warning arrive **every time**, because light
outruns a sub-light ship by construction. The question stopped discriminating,
and left alone it would have turned every ship back and deleted the arrival
fight.

- **When you model a signal properly, check whether the predicate it feeds still
  has two outcomes.** "Did the news arrive in time" is a real question only while
  the news travels at a comparable speed. At `c` against anything sub-light it is
  a foregone conclusion dressed as a race, and the gate has to move to what the
  receiver can still *do* — here, whether it is past the turnover point of its own
  brachistochrone, which costs no constant and is a property of the trajectory.
- **Price a kinematic mechanic before building content on it.** A ship's travel
  time exceeds light's by an amount that **saturates**, because it spends all but
  the first stretch at nearly `c`, and that overhead is the entire window an
  interceptor has. Measured: **8.14 years** for a laden Medium colonizer, in
  which an empty LOU picket covers **6.29 ly** — against a median
  nearest-neighbor spacing of **6.16 ly**, so about half the field is
  interceptable and the margin at the median is 0.13 years.
  `examples/intercept_probe` is four printed columns and it settles the
  feasibility of a whole mechanic before any of it is tuned.
- **A second consumer audits the first, again.** Implementing interception meant
  needing the colony ship's speed, which is how `spawn_courier` turned out to fly
  the colonization leg **unladen** — reading `civilian_accel_g · G` before the
  hold was loaded and never re-reading it — while §7 records R-O32 as having
  fixed exactly that. §3's *"a second caller is a cheap audit of the first"*
  holds for physics as well as for panics.
- **And then the probe was run against the defect anyway, which is the mistake
  worth naming.** The first pass priced the colony ship at the *empty* rate — the
  number the bug produces — and reported a 1.92-year window, an 1.86 ly radius,
  and *"fewer than a tenth of worlds"*. Every figure was internally consistent
  and the conclusion was backwards, because a laden hull's window is four times
  as wide. **A probe inherits every assumption of the code it measures.** When
  you have just found one of those assumptions to be wrong, the probe is part of
  what has to be re-derived — finding a defect and then measuring around it is
  worse than not having found it, because the measurement now carries the
  defect's authority.
- **And the sign of one term was the design.** A station 0.5 ly *beyond* the
  contested world loses the race; the same 0.5 ly on the *near* side wins it,
  because the far side pays the distance twice — once in light to hear the
  launch, once in flight. That makes interception a **forward-deployment**
  mechanic rather than a reaction one, which is a design statement no amount of
  tuning would have produced.
- **The window is back-loaded, and that refuted the obvious generalisation.**
  *"Meet the ship anywhere on its path instead of racing it to a world"* sounds
  strictly more capable and is **narrower**: the slack an interceptor lives on is
  `t_ship(along) − along`, which accumulates as the hull decelerates through the
  back half of a brachistochrone, so on a 25 ly voyage the tolerable
  perpendicular offset runs **1.24 ly at an eighth of the track, 1.95 at half,
  4.96 at the destination**. The destination is where the whole window is. One
  probe, eight printed rows, and a feature that would have cost a week is
  recorded as refuted instead of built.
- **Two hulls with different names can be the same object.** The armed scout
  write (`scout_hull_offensive`) reproduced its baseline **bit-identically**,
  and the cause is `hull_dry_mass` reading the cost *tier*: every Limited hull
  shares one, so an LOU and an LCV have the same 0.020 kt price and mass and the
  simulation cannot tell them apart. That is the *exactly zero* verdict and it
  names its own next action — the write goes live when R-O64/R-L0 give hull
  types differentiated cost, not when it is tuned.

### An inference is a mechanic; being told the answer is not

**T-120 is the worked example.** `offer_interception` was *handed* the colony
ship's destination, so a picket could not be wrong — and a mechanic that cannot
be wrong cannot be deceived. The engine had light-lag, bearings and a full
observation model, and one function reaching into the ship's `voyage.target`
deleted the whole yomi channel that sat on top of them.

Reading the *trajectory* instead — a departure point, a time and a bearing, with
the destination inferred from worlds the picket has itself scanned — is a few
lines, and it creates a move that did not exist: two worlds on one bearing are
indistinguishable at range, so a ship aimed at the far one puts a picket on the
near one **for free**.

- **Ask what a decision is allowed to read, not just what it decides.** Every
  design-law-#15 violation in this project has this shape: `Knowledge` storing
  membership rather than observations (T-33), colonization filtering on
  instantaneous global ownership (T-34), and this one. The decision was fine;
  its inputs were ground truth.
- **A guess needs a way to be revised, or it is a commitment.** The second half
  of the mechanic is re-reading the trajectory as fresh light arrives, at the
  *lagged* position — a ship that has already turned still looks, for `distance`
  years, like it is going where it was going. That delay is the thing a feint is
  actually buying.
- **Mechanism before policy, and say which you built.** Nothing in the engine
  chooses a deceptive bearing; `BaselineAutopilot` aims at the world it wants.
  The channel exists and no policy uses it. Write that down, or the absence gets
  read as a measurement that bluffing does not work.

### Count the consumers of a write, not the writes

**T-120's second half is the worked example.** The first Warfare card writes
`UnlockDesign(LimitedOffensive, _)` into the per-player `Roster` — a real write,
into real replicated state, through the real card path, charged the real price.
The engine counts it as an implemented card. It reaches **no decision**: the
Roster's only consumer outside tests is `roster_permits`, which returns `true`
before it looks anything up whenever `enforce_roster` is off, and that is the
shipped default.

So a twelve-seat head-to-head measured the card at **−0.0371 ± 0.1609**, inside
one standard error of zero, and the number was never the finding. Three habits:

- **Before measuring a feature, grep its state for readers and check what
  guards them.** One `grep` for `roster` returned six lines, one of which was a
  gate defaulting to off. That settles the question in a minute, where the bed
  cost 10 and answers it weakly — a wide error bar looks the same whether the
  effect is zero or merely unresolved.
- **A "did I implement it" counter measures the wrong thing.**
  `Sim::inert_card_plays` increments only on `CardEffect::NotYetImplemented`, so
  it tells you which match arm ran and not whether the write landed anywhere
  live. The honest predicate is about the *consumer*, and it is the one nobody
  writes because the write is the part you just finished.
- **Check whether the thing you are measuring is the thing the spec
  describes.** The Warfare spec's first card is one Design write plus three
  Doctrine writes; `DoctrineWrite` has four variants and **none of them is a
  Warfare write**, so every arm the spec had measured was reached by setting
  `Doctrine` fields directly — bypassing the card table and the price both. The
  published card and the specified card were different objects, and nothing in
  either document said so.

This is §4's *"a behavioral change that reproduces the baseline exactly is inert
or unreachable"* moved one step earlier: there the tell was a bit-identical run,
here there was no need to run anything at all.

**And a second trap from the same bed, which would have made any card
measurement vacuous without looking wrong.** `apply_orders` checks
affordability and `Order::coerce` turns a failure into a **pass** rather than an
error, so a bed that issues orders and then runs cannot distinguish *the card
did nothing* from *the card was never played*. Whether round 0 is legal is a
relation between `Card::cost` and a homeworld's seeded bank — two magnitudes
nobody reconciled. **Record when a card actually landed, and assert the
precondition in both directions**, rather than assuming the play took.

### A default that is not the ratified default is a contradiction nobody filed

**T-121 is the worked example, and the spec had been right the whole time.**
`Hyades_standing_layer_and_observation.md` §7.1 says *"Default doctrine: 100%
LSV in the Scout role"* and has said it since R-O42. The engine surveyed with a
Limited **Contact** hull. Neither document was wrong about itself and nothing
flagged the gap, because the spec described a default and the code implemented
a different one, and no test compared them.

That is how a card meant to *sell* the armed family ended up selling nothing
(§8.13): the Doctrine half of the standing layer defaulted unarmed and the
Design half defaulted armed, so a seat flew Contact hulls whether or not it
bought the right to. The asymmetry is invisible from either side alone.

- **When a spec states a default, assert the default — not the mechanism that
  produces it.** The tests here pinned the *build branch* and the *role map*,
  both of which agreed with the code and neither of which had read §7.1.
- **A lock needs both halves closed, and the tell is to enumerate rather than
  spot-check.** `the_warfare_card_is_the_only_key_to_the_contact_family` walks
  every assignable role, the colonizer ladder and the seeded roster and asserts
  no Contact hull appears in any of them — then plays the card and asserts every
  Contact hull the armed layer mounts is one the card unlocked. A test that only
  asserted the second half would pass while the default was armed too.
- **"Locked behind X" is a claim about the default, not about X.** It is
  tempting to check that the key works. The failure mode is a door that was
  never shut.

**And the cheapest fix for a duplicated read is to derive one from the other.**
`scout_hull` was public and read in three places — the build branch, the
affordability test, and `launch_survey` — which is the shape §6's standing-layer
rule names and T-116 had already paid for once. `Standing::scout_order()` is now
`design_for(Role::Scout)` wrapped in a `BuildOrder`, so the hull the yard is
charged for and the hull `role_of` reads back are the same call. Two of the
three call sites disappeared rather than being kept in agreement.

**Watch for a resolver whose tie-breaks become load-bearing when you merge two
cases onto one hull.** Scout and Miner now share `LimitedSystems`, so
`role_of`'s "first role that mounts this hull" pass went from unambiguous to
order-dependent — a silent semantic choice sitting in the order of a `const`
array. Making `ASSIGNABLE` agree with the competence table costs one comment;
noticing that it *had* to was the work.

**The same merge broke a build order, and no test could have caught it.**
Ordering a picket by *naming its hull* worked for as long as no other role used
that hull; once the picket and the armed scout shared one, the hull→class map
stamped the scout's class and the yard received a scout. That is T-116 again —
paid for one thing, given another. The branch sits behind two doctrine fields
that both default off and neither of which the new card writes, **so the suite
was green across the change and every bed reached zero of it.**

- **When you move a hull, grep the hull, not the behavior.** The defect was
  three call sites naming `HullType::LimitedOffensive`. Nothing was failing and
  nothing would have.
- **A function keyed on a shell cannot answer a question about a role.**
  `hull_order(hull)` and `Standing::order_for(role)` look interchangeable and
  are not, for exactly as long as two roles share a shell — which is a
  condition a card can create at runtime.
- **Pin the defect, not only the fix.** The round-trip test asserts both that
  `order_for` round-trips *and* that shell-ordering a picket still reads back as
  the armed scout. If that collision ever disappears, the rule loses its reason
  visibly instead of quietly becoming cargo.

### A bed that plays outside the protocol measures a game nobody plays

**T-122 is the worked example, and the defect was in my own harness for three
landings.** `card_table` and `card_probe` played cards at `t ≈ 0` and called it
"earliest legal play". The protocol says otherwise: the opening is card-free and
the first round barrier is at `years_to_first_round`, 200 yr. So every card paid
its price out of the 3 kt bootstrap bank — a state no game reaches — and that
price was **the largest effect either first card had**: ~550 colonies on one
seed, reproduced exactly by an inert card of the same price. At the barrier the
same price is invisible, and the Growth card, which had read as flat, clears
**t 4.18** with no engine change.

- **"Earliest legal play" is a protocol fact, not an affordability fact.** I
  had checked `empire_can_afford` and a test even pinned "round 0 is legal" —
  while reading *round 0* as *t = 0*. Affordability answers whether a play can
  land; the round layer answers when. Read the config that schedules the
  barrier before writing the loop that plays at it.
- **A pure-price control is the cheapest ablation a card bed has.** An inert
  card at the same cost (`TIER0[12]` while `enforce_roster` is off) separates
  what a card *does* from what it *costs* in one arm, and it is what exposed
  this.
- **A confounded arm can produce a tight zero.** One arm read **+0.003 ±
  0.040** and looked like proof that a mechanism was necessary. It carried the
  same price defect, and a narrow error bar around a confounded mean is still a
  confounded mean. Before reading a precise null as a mechanism, ask what else
  that arm changed.

**And an oracle ablation that does not help is a strong refutation.** "The
picket guesses the wrong world" explained zero fights perfectly. Handing the
picket the true destination produced 18 intercepts and still zero fights — so
information was never the constraint, and the time that would have gone into a
better guess went nowhere. When a hypothesis is "the agent lacks information",
give it the answer and see whether anything moves.

### Never leave an identified symptom without a proven mechanism

**A number is a symptom. Stop only when you can name the line of code that
produced it and show it produces it.** Not a hypothesis that fits the sign, not
a mechanism that is plausible given the change — a mechanism you have
*demonstrated*, by ablation, by instrumenting the decision, or by a test that
fails when the mechanism is removed.

This rule exists because the shortcut is so cheap and so convincing. Every
artifact in the table below was a real measurement with a plausible story
attached, and the story was what made it survive. Two from this project,
back to back:

- **"−178 colonies because the mass draw got bigger."** Mechanistic,
  right-signed, consistent with the change. Refuted in two one-line ablations —
  deleting the biomass draw *entirely* reproduced the number bit-for-bit. The
  real mechanism was a policy reallocation two modules away.
- **"the expansion loop's time constant is the limiter."** True, and useless as
  stated: a time constant is not a mechanism, it is the shape of one. Pushed to
  the code, it resolved to a specific dead branch —
  `reinvest_bias` comparing a Band against a rank score, so the deepen path
  cannot fire at the shipped value (R-O68). That is a mechanism: it names a
  line, it is measured (`examples/score_scale`), and a test fails if it changes.

Three things that count as proof, in rough order of preference:

1. **Ablation.** Remove the suspected cause and show the effect goes. Cheap —
   both of the above took one line and one run — and it is the only method that
   can *refute*.
2. **Instrument the decision.** Count the branch, print the two sides of the
   comparison, histogram the quantity. `coverage_trace` and `score_scale` exist
   for this.
3. **A characterization test.** Pin the mechanism where it lives, so it cannot
   silently change meaning. This is what stops the finding decaying back into
   prose the next time the operating point moves.

What does **not** count: a plausible story, a correlation across configurations,
or a mechanism inferred from the sign of a gradient. And when the symptom is
"X is slow" or "X is the limiter", the mechanism is never a *rate* — it is the
gate, branch, or serial dependency that sets the rate. Keep going until you
reach one.

**A proven mechanism is not automatically the cause, and R-O68 is the worked
example of the difference.** The dead deepen branch was proved three ways — the
two sides measured, the branch counted, a characterization test pinning the
crossover — and it was real. Fixing it changed **nothing**: the run is
bit-identical below `reinvest_bias = 0.96` on both seeds, and the branch is
still cold at the shipped `0.5`. The defect was genuine and it was not what was
producing the symptom; the price ladder was (R-O85), and that only became
visible once the units stopped hiding it.

Two habits follow, and the second is the transferable one:

- **Measure the fix against the old binary, not against the prediction.** Build
  both, run the same seeds, diff the numbers. Here it took one `git stash` and
  two binaries, and it is the only reason "this is a units fix, not a behavior
  change" is a statement rather than a hope.
- **A dead branch can be the right answer reached for a wrong reason.** Before
  reviving one, price what it would have chosen. Expansion returns 24–49x per
  kilotonne against deepening at the shipped ladder, so the policy was correct
  and its *reasoning* was not — and had the fix been landed without that check,
  the next step would have been to tune a dial toward a decision the economics
  say is bad.

**And the actual cause was found by an instrument built for a different
question** (R-O86). The survey harness printed `candidate_count`'s distribution
only to justify a sentence in a doc comment, and the answer — **median 0, max
164, against a `survey_reserve` of 1024** — said the reserve test is a constant
`true` and, one step on, that `candidates.is_empty()` was pre-empting the only
live deepen path. That was the thing keeping infrastructure at Band 1.03, not
the comparison and not (yet) the ladder.

Three habits, and the last one is the one that would have saved the most time:

- **Print the distribution of what a threshold is compared against, not just the
  threshold.** One column. It converted a plateau this file had already recorded
  as an unexplained measurement artifact into a one-line mechanism.
- **Check whether a predicate means what its name says.** `candidate_count` is
  "known and still available"; the survey question is "anything left to
  explore". Those coincide early and diverge permanently, and nothing in the
  types could tell them apart — both are `usize`.
- **Ask what fraction of a busy path produces nothing.** The engine spent 99% of
  its production issuing builds that created no object. Profiling cannot see
  that: every one of those builds really ran. The tell is a count that does not
  reconcile — 1,779,509 `BuildApplied` against 18,093 hulls — and nothing was
  comparing the two until a census happened to print both.

### The artifact pattern — four of them, one shape

Four measurements in this project were wrong in the same way, and the shape is
worth recognizing because none of them looked wrong:

| What was measured | What it actually was |
|---|---|
| `medium_fleet_size = 8` optimal, 12 a "cliff" | the capacity normalizer going to zero |
| `coverage_trace`: this knob "DID move it" | two of three sample points degenerate |
| `coverage_time`: cheaper colonizers at 6.0 | a General hull holding ~700× a Medium's |
| coverage "wants" a cheaper Medium hull | `cap_Medium` pinned by a live normalizer |

Every one produced a plausible number from a broken configuration, and every one
was invisible in the objective — because the quantity that broke was in a
*denominator* that the shipped autopilot never exercised. Two habits follow:

- **Assert ratios, not just numerator and denominator.** Design law #3's
  inversion survived because cost and capacity were each individually ratified.
- **A parameter that reaches the objective through a derived quantity cannot be
  swept alone.** Since R-O58 the cost ladder *is* the capacity ladder; sweeping
  one leg of it in isolation is measuring two things and reporting one.

### A seventh shape: an aggregate that moves against every one of its parts

R-IND11 (`Hyades_industry.md` §1.6) added one the table above does not cover,
and it is the only one where **nothing was broken.** A dwell metric — time from
a colony's founding to its own first build — *rose* 253.5 → 335.8 yr under the
policy that founds colonies **399 years earlier**, which read as a clean
refutation of the mechanism it was built to test. Split by what that first build
was, **both components had fallen** (72.4 → 66.3 and 401.4 → 342.9). A weighted
mean can only rise while both group means fall if the weights move, and they had:
the hull-first share went **55.0% → 97.4%**, on every seed.

So the measurement was correct, the population it averaged over was not the same
population, and the sign it reported was the opposite of the mechanism.

**The rule: any metric averaged over a set the intervention re-selects must
report its mix beside it.** This is the artifact list's invariance rule (§"never
let the metric's denominator be something the game can play") applied one level
down — there the denominator was farmable, here the *composition* is. Ask of an
aggregate: *does the treatment change who is in this average?* If yes, the
aggregate alone can say anything, and a decomposition is not optional polish.

**And it is why the ablation went first.** Both candidate mechanisms had been
refuted by their own signatures before either was believed, and the two one-line
ablations named the cause anyway — seed mass, not hull, not price, not transit.
`CLAUDE.md`'s ordering held: ablation refutes, instrumentation explains, and a
metric that disagrees with an ablation is the metric's problem to answer for.

### Decompose a rate into stock and per-unit before explaining it

**R-O89's rejected arm is the worked example, and it refuted three plausible
mechanisms in one table.** Need-routing the freight *pickup* leg cost −52.3% work
-years, which reads as "hauling got worse". It did not:

| | baseline | need-routed pickup |
|---|---|---|
| round trip | 123.77 yr | **109.24 yr** — *shorter* |
| trips per active hauler | 1.84 | **1.87** — *unchanged* |
| retirements to Reserve | 14,828 | 14,520 — *unchanged* |
| **active haulers** | **1,946** | **988** — *halved* |

Every per-unit measure is flat or better and the aggregate fell by half, because
the *population* moved. This is §2's mix rule (an aggregate over a set the
treatment re-selects) in its performance costume, and the decomposition is always
the same one: **`total = population × rate`, and you must print both.** A rate
that improves while the total collapses is not a paradox, it is a headcount
problem — and the two want opposite fixes.

Three habits from it:

- **Bucket by time before concluding anything about a compounding run.** The two
  arms are within noise until year 500 and diverge from there — which named the
  trigger (the router first has a *choice* once a player works many rocks)
  without any further instrumentation. A run-total would have said nothing.
- **Check the filter matches the state at logging time.** Counting freighter
  retirements on `VehicleParked { role: Role::Freighter }` returned **zero** in
  both arms and read as "nothing ever retires"; `release_to_reserve` re-roles the
  hull to `Role::Reserve` *before* it logs. A filter that silently matches
  nothing is indistinguishable from a real negative result, so sanity-check every
  new counter against something you already know is non-zero.
- **Stop at the boundary and say so.** Three mechanisms were refuted and the
  fourth — why the fleet stops growing — is *not* established. The candidate that
  fits the sign (a need-routed pickup breaks the 1:1 miner↔hauler pairing) is
  exactly the shape of all seven artifacts above. It is recorded as open under
  T-76 rather than written down as the cause; the arm is not shipped, so nothing
  depends on it.

**And the 2×2 is why the good half survived.** Selective loading is **+8.4%** and
pickup routing is **−52.3%**; run together they score −41.2%. Landed as one
change, the whole thing reverts and the +8.4% is never found. **When two
independent changes address the same diagnosis, ablate them apart before you
believe either** — `CLAUDE.md`'s "ablate before you explain", applied *before*
there is anything to explain.

**T-98 is the same lesson from the other side: the arm that looked like a
refinement was the load-bearing one.** Sizing a hauler's hull to its rock scores
**+51% work-years and −17% colony-years on 1/8 seeds** — a development gain
bought out of the expansion loop, and the honest response to that is to revert.
Adding a *liquidity* cap — only consider hulls the center can pay for now — takes
it to **+170% and +9.5%, 8/8 and 7/8**. Same rule, same objective; the second
term is the whole result.

The mechanism is worth having because it recurs: a score of the form
`value / cost` is a **rate**, true in steady state, and it says nothing about the
years spent saving for an indivisible purchase. A General hauler returns 2.86×
a Medium's per mineral and costs **12×**, so a thin-banked center buys one and
stops expanding while it saves. **When a decision picks among lumpy purchases,
price the wait as well as the return** — and note that the budget constraint was
already there, so it cost no new constant.

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
| `src/units.rs` | **`Band` / `Kilotons` newtypes and the `Measure` trait** — the one place the two units meet, and the fix for a `min` that compared a mass against two levels (R-O66) |
| `src/math.rs` | 3-vectors, relativistic 1 g flight, light-lag (`c = 1`, distance in ly, time in years) |
| `src/rng.rs` | seeded splitmix64; `fork()` per entity for order-independent determinism |
| `src/resources.rs` | CMY basics, RGB supers, apex, archetypes |
| `src/galaxy.rs` | galaxy generation → continuous 3D planet field |
| `src/autopilot.rs` | `Autopilot` trait (swappable per-seat policy) + `Doctrine` knobs |
| `src/belief.rs` | **believed kinematics** (R-O41) — one-sided `a_max` estimate from light-lagged observations, and the accept/decline predicate that runs on it |
| `src/cards.rs` | the **card layer** — 18 tier-0 placeholders (3 slants × 6 trees), `Order`, and the coerce-never-reject rule |
| `src/sim.rs` | the light-lagged discrete-event ECS engine |
| `src/combat.rs` | **engine-native combat**: kinematics, weapons, `resolve_engagement`, and the tuned station-keeping spread — **two callers since T-111**, the arena and `sim::sys_engagement` |
| `src/arena.rs` | Ship Testing Arena — *scenario seeder only*, owns no combat logic |
| `src/matching.rs` | the Exchange (order-book matching) — wired in at T-01; **it was never in the module list, so it did not compile as part of the crate and its tests never ran in CI** |
| `src/log.rs` | optional diagnostic event log (the interrogation seam) |
| `src/snapshot.rs` | read-only views for the presentation layer |

### The combat/arena split (load-bearing)

`combat.rs` is the engine's fighting model — the *same code* the production game and
the balancer resolve fights with. `arena.rs` exists only to **spawn ships outside the
constraints of Hyades production** (no economy, no mineral budget, no colonization),
place them, and call `combat::resolve_engagement`. **The arena resolves no damage.**
Dependency direction is `arena → combat`, never the reverse. Do not reintroduce
combat logic into the arena or into an example.

**Since T-111 the simulation is the second caller, and the rule extends rather
than bends: `sim → combat`, never `sim → arena`.** `sys_engagement` builds its
own `Combatant`s from hulls that were paid for; the arena's whole purpose is
spawning ones that were not. Tuned constants live on the `combat` side of that
line — the station-keeping spread moved there from `arena` when the sim needed
it, with `arena::ROU_STATION_*` kept as re-exports, because **a Monte-Carlo-tuned
number with two definitions is an edit waiting to go wrong.**

**Wiring a second caller is also how you find out what the first one assumed.**
Two arena assumptions had been invisible for as long as `resolve_engagement` had
one caller: `laser_ships[0]` panics on an empty side (a scenario always seeds
both fleets; the sim reaches "nobody left" legitimately), and `carrier_accel`
reads ship 0 under a comment saying *"same hull both sides"* — true of a one-hull
sweep, false of two empires bringing what they built. **A second caller is a
cheap audit of the first**, and neither defect was findable by reading.

---

## 4. Architectural invariants

- **No presentation in the engine.** Nothing renders, reads input, touches the clock,
  the filesystem, the network, threads, or the OS RNG.
- **No hexes in the engine.** The simulation is continuous 3D space; each star system
  is a point. Hexes are a *command-view* concept owned by the presentation layer.
- **A quantity carries its unit in the type, not in a comment.** `Band` is a
  magnitude *tier* on the ladder (`Hyades_mineral_cost_curve.md` §2.6);
  `Kilotons` is an amount of stuff. **A Band is a *reading*, not a second thing
  to store** — directed: *"everything is just counting. Growth and construction
  and cost are denominated by mass. Bands are a tool for game design, nothing
  more. All logistic functions are applied to real population, not Bands."*

  **The type is `Qty<S>`** (T-64): one `f64` of kilotons, `repr(transparent)`,
  with a zero-sized `Scale` marker saying which ladder its Band *reading* is
  taken on. `Kilotons` is `Qty<Mass>`. Two rules follow and both are measured,
  not asserted:

  - **Arithmetic is free; conversion is not.** `examples/qty_bench`: a `Qty`
    add/mul/min loop runs at **1.000x** bare `f64`, while `band()` is 2.2x an
    arithmetic op and `at_band()` 3.8x. So convert at the *edges* — never inside
    a loop over entities. `Factors::bio_max_band` exists only because a `ln` got
    onto the hot path (R-O70); denominating the ceilings in mass deletes it.
  - **There are two ladders and the type is what keeps them apart.** R-MC15
    ratified `F_mass = F_cost^(3/2)` because cost tracks surface area and the
    hold tracks volume, so a General hull costs 10x a Medium while holding
    31.6x. `general_vehicle_cost = 1.0` is kilotons (R-O57) and reads **`Band I`
    as a mass and `Band II` as a cost.** That is geometry, not a units bug;
    crossing is `Qty::on_scale`, free and explicit. Do not "simplify" the two
    ladders into one — `the_same_amount_reads_a_different_rung_on_each_ladder`
    is there to stop it.

  They are different types because the engine
  shipped `K = min(hab, bio, infra)` for a long time with `bio` a mass and the
  other two levels — a `min` across incompatible units that typechecked, read
  as plausible ecology, and put the largest measured lever on coverage
  (`biosphere_regen_rate`, +141.2 ± 18.1) on top of it. Nothing about `f64`
  could have caught it. Do not add a bare `f64` for a quantity that has a unit,
  and do not add a second conversion between the two — `src/units.rs` owns it.

  **A named rung is a name, not a number.** `BandTier` (`Empty, I, II, III, IV,
  V`) is the discrete ladder; `Band` is a *position* on it and can sit anywhere
  between rungs. Config constants that mean a rung are typed as the rung —
  `colony_seed_pop = 1.0` no longer compiles, and a `compile_fail` doctest keeps
  it that way. `V` is a **comparison ceiling that is unreachable in play**, so a
  bounds check has a rung one past the end instead of a magic number;
  `band_v_is_one_past_the_playable_end` pins that it stays unreachable, because
  a sentinel that quietly becomes attainable leaves every `< V` guard compiling
  and meaning nothing.

- **Determinism is a hard requirement.** All randomness flows from a seeded `Rng`;
  all time is the in-sim event clock in years. Iterate collections in deterministic
  order. Same seed ⇒ bit-identical results, native and wasm32. `tests/determinism.rs`
  guards this — never weaken it to make a feature fit.
- **Zero dependencies.** Do not add crates to `Cargo.toml`.
- **Entities evaluate on their own arrival events — never on a tick sweep, and never
  by rescanning the galaxy.** This is a discrete-event engine: a ship decides what to
  do next *when it arrives somewhere* (`ContactArrive`, `FreighterArrive`,
  `ColonyArrive`, `ScrapArrive`), which is the only moment its situation actually
  changed. ~~Production centers are the one cadence-driven exception, one tick per
  center per `cycle_years`~~ — **closed by R-O69.** A center's *economy* step
  (mine + grow) is still cadence-driven, and correctly so: both halves are rates
  over an interval, and an interval is what a rate needs. Its **decision** is
  not, and is now an event — `BuildDecision`, raised when the yard clears
  `build_years` after a build was committed. Everything is arrival- or
  completion-driven; nothing decides on a sweep.

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

    **T-101 did it again and it worked, and the difference is one word:
    *compaction*.** The candidate scan's two filters are monotone, so a rejected
    entry is rejected forever; pruning them cut the walk **251.6 M → 45.5 M steps,
    −82%**, bit-identical. What made it a win rather than a repeat of the above is
    that a retain-style compaction preserves ascending order — the walk stays
    sequential — where swap-removal does not. **And the win was 11%, not 82%:**
    the steps deleted were two bitmap lookups apiece, so *iteration count is not
    cost*. Both halves of that are worth carrying: prune monotone filters, and do
    not expect the speedup to track the count you removed.
  - **Memoize a scan whose answer only changes on an event** — but store the
    *recomputed* value, not a running total. `holdings_centroid` walked every
    planet once per production decision (1.14 G iterations on seed 1) for a value
    that moves ~3,400 times a run. A running sum would have accumulated in claim
    order where the walk accumulates in planet-id order, and float addition is not
    associative: the centroid would differ in its last bits, every rank score with
    it, and the run would diverge. Memoising the walk is bit-identical; only the
    *number* of walks changes.
  - **Replace a library transcendental with a polynomial fitted to the range
    the argument actually takes.** `rank`'s `centrality` calls `exp` 45.4 M times
    a run for a *classification weight*; a degree-7 minimax fit on `[−2, 0]` is
    **+2.7% throughput at 5.4e-7 relative error** (T-102, `math::exp_decay`).
    Four things about it generalise past this call site:

    - **Histogram the argument before you fit.** The measured span is
      **[−1.7348, 0]** across 2/3/12/18 seats. A degree-7 fit is excellent over
      [−2, 0] and worthless over [−40, 0], so the *range* is what buys the
      accuracy — and a range assumed rather than measured is a silent accuracy
      claim with nothing behind it.
    - **Estrin, not Horner.** Same polynomial, same operand count: 2.98 ns/call
      by Horner, **1.97** by Estrin, which is what Horner costs at *degree 5* and
      220× more accurate. On a modern core the serial dependency chain is the
      price, not the multiply count.
    - **A polynomial is *more* deterministic than the function it replaces.**
      `f64::exp` is platform libm natively and a Rust libm on wasm32; `+` and `*`
      are exactly specified by IEEE 754 and identical everywhere. **Never
      `mul_add`** — it rounds once where a multiply and an add round twice, which
      reintroduces exactly the cross-target divergence the change removes.
    - **Perturb the approximation to find out whether the call site is even
      live.** Scaling the polynomial by ×1.000001 moved seed 7's run, which said
      before any A/B that bit-identity would be luck rather than a property —
      and it was: five of six seeds reproduce exactly and one moves +0.07%
      colonies. **A cheap deliberate perturbation tells you what kind of change
      you are making, and it is the difference between reporting a disturbance
      and discovering one.**
  - **Pick the container for the access pattern, and check the siblings.**
    `Knowledge::visited` was converted from `BTreeSet` to a bitmap when one
    `contains` turned out to be 63% of engine instructions — and `targeted`, its
    sibling with the same contains-only access pattern, was left as a `BTreeSet`
    and reached **1.03 billion lookups**. `scanned`, which is *iterated* rather
    than probed, wants a sorted `Vec`: same order, sequential reads instead of a
    pointer chase over boxed nodes. Together with the memo above, 3.3x (R-O70).

  **Profile before you optimize, every time.** R-O70 began with two confident,
  plausible fixes to the production candidate scan — they were correct changes and
  bought **nothing measurable** (60 → 58 yr/s). Survey was then assumed to be the
  hot path on the strength of the worked example just above, and is an order of
  magnitude smaller than the two loops that actually mattered. One instrumented
  run counting loop iterations settled it. A slow program is a symptom; §2's rule
  about mechanisms applies to performance exactly as it does to behavior.

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

   **T-96 kept the broadcast and deleted the tax.** Those are two different
   things and thrust-∝-dry-mass conflated them: because thrust was the *shell*
   (`r²`) and the load was the *hold* (`r³`), laden acceleration fell as `1/r`
   and the law's own cost advantage was being repaid in turnaround — a laden GSV
   flew at **0.317×** a laden MSV. With thrust drawn from mounted drive instead
   (R-MC16), drive and cargo both scale `r³`, the shell term shrinks away, and
   the round trip goes **1.252 → 1.011** against an equal-cost Medium fleet. The
   *signature* is untouched, which is what §9.2 actually needs: a General hull
   still drops from 5.06 g empty to 0.23 g laden, a 22× swing, against a Limited
   hull's 1.00 → 0.70. **Read a law about a ratio as a claim about the ratio** —
   "bigger is more efficient" was being paid for in a dimension nobody had
   checked, and checking it took one round-trip column.

   **And the signature is not a free axis — it *is* this law's ratio (R-O95).**
   Because T-96 made thrust independent of the load and R-O57 makes cost equal
   dry mass, `a_empty / a_laden = 1 + C / M_dry` exactly, at every
   configuration — the empty-to-laden swing and the cargo efficiency are one
   number (`the_acceleration_swing_is_the_cargo_efficiency`). So a hull cannot
   be made worse at freight, faster empty and slower laden at once; that triple
   is over-determined, and it refuted a third of the first Warfare card's stated
   intent before a line was written (`Hyades_warfare_tree.md` §7.3).
   **Write a design intent down as algebra before measuring it** — three
   plausible properties collapsed to two in one line, where a sweep would have
   spent a bed discovering it.
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
   population growth consumes it. Biosphere is the one *renewable* stock,
   regrowing logistically toward `bio_max`, so ecology is a rate rather than an
   exemption — which is what makes biological damage durable and gives Warfare a
   target that is neither hulls nor infrastructure.

   **`Band Empty` is the mass ladder's *floor*, one metric tonne, and its width
   is set on its own** (T-63) — `KILOTONS_AT_BAND_EMPTY`, not `KT(I)/F₀`. R-MC15's
   growth rule and the `F_mass = F_cost^(3/2)` tie are claims about the playable
   rungs `I → II → III → IV`; the floor is a per-quantity anchor and the cost
   ladder's floor (5, the Limited hull's price) is untouched. Consequence to
   remember: a Limited hull's hold no longer sits on `Band Empty` — hull holds
   are geometry (`5^1.5`) and that coincided with the old floor width.

   **Every logistic runs on the mass, not on the reading (T-64).** Population
   growth steps `x + r·x·(1 − x/K)` on *people*, with `K`'s mass as the
   carrying capacity. It used to step on Band positions, which made `r` a rate
   of change of an *exponent* — the same `growth_rate` meaning a different
   number of people at every point on the ladder. Two consequences: `r` had
   a hard arithmetic ceiling at **2** (the step is conjugate to the logistic map
   with `μ = 1 + r`, so it period-doubles there), and **the engine's `clamp` at
   `K` hid that from below** — a too-large `r` did not visibly oscillate, it
   collapsed the logistic into a step function that filled a world in one cycle
   and *scored well* while doing it. `the_population_logistic_is_a_rate_and_not_a_step`
   is the guard.

   **Both of those are retired (T-94/R-O93): the step is now the closed form.**
   `x(t+Δ) = K·x / (x + (K − x)·e^(−rΔ))` — the ceiling is constant across a tick,
   so the step is autonomous and solvable, and `settler_target` had been pricing
   colonization off this solution's *inverse* since R-IND11. The `r < 2` ceiling
   was a property of the Euler map and not of the model: `e^(−rΔ) ∈ (0, 1)` at
   every positive `r`, so the map is monotone at any rate and the clamp can only
   fire on a last-bit rounding. What the closed form also removes is the
   **undershoot below `K`** — an over-capacity world now decays *toward* the
   ceiling (31.62 → 8.88 kt in one tick, a 72% die-off) rather than overshooting
   to zero. The collapse was design content; the undershoot was truncation error,
   and its severity was a function of `cycle_years`, so the outcome of an attack
   on a world's habitability was being set by a performance knob. **R-O84's
   `growth_rate = 0.873` is carried, not re-ratified** — its operating point and
   its plateau map are both consumed (T-95).

   **And population is conserved across space, not only across the biosphere
   (R-O74, `Hyades_industry.md` §1.7).** Growth draws people out of biomass;
   *founding* moves people that already exist. Until this landed, a colonizer's
   settlers were written into its hold with nothing debited anywhere — one
   exemption from this law, on the exact path the expansion loop runs on. It was
   not small: the colonizer policy that shipped the biggest seed measured
   **+13.97% colony-years on an identical colony count**, and two ablations put
   the entire effect on the seed mass rather than on the hull, its price or
   transit. **An exemption from conservation is not a modeling shortcut, it is
   a free resource, and a search will find it and call it a strategy.**

   **The exchange is a mass difference, not a level difference (R-O66).** A
   population at Band `b` masses `KT(b) = KT_I · BAND_STEP^(b−1)`, so a step from
   `b` to `b'` costs `KT(b') − KT(b)` — one Band up is `BAND_STEP` *times* the
   people, not `BAND_STEP` more of them. `src/units.rs` is the only place that
   conversion is written. Two consequences are load-bearing: **`K` is a minimum
   over Bands** — `min(hab, bio_max, infra)`, with the standing biomass
   deliberately *not* a term, because it is the mass growth is paid out of and
   not a ceiling; and **regrowth runs on living mass**, biosphere plus people,
   since people are biosphere. A world filled with citizens has no spare
   ecological niche, and one whose biosphere has been eaten to nothing is not
   sterile.
12. **No retroactive refits** (R-O47b). A Design write never reaches a hull
   already in the field by fiat; realization is `on_refit`, so a fleet-wide
   change lands staggered by transit time. Retroactive would change every
   acceleration signature at once, laglessly, with no build to watch for — the
   instant global state change the light-lagged observation model exists to rule
   out. The recall is itself a signal, and it cannot be offset by a thrust write
   because what leaks is the movement, not the mass.
13. **No categorical strategic classification may be co-extensive with a color
   domain** (L1/R-O34) — it would lock out exactly the archetype poor in that
   color. Continuous classifications expressed as magnitude are exempt.
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
  it describes the engine as it actually is. A PR that changes behavior, adds or
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
- **A spec carries ratified decisions and open decisions. Nothing else.**
  Every statement in a spec is one of exactly two things: **a decision that has
  been ratified**, or **a decision that is still open**. Everything else — the
  runs, the tables, the refuted hypotheses, the superseded design, the
  measurement that turned out to be an artifact — belongs in
  `docs/Hyades_experiments_appendix.md`, **linked from the decision it
  supports**, never deleted.

  **A proposal is an open decision, not a third category.** A recommendation
  with nobody's ratification behind it is `OPEN` with a recommendation attached,
  and it is labeled that way — the failure mode this rule exists to stop is a
  proposal written in the present indicative for long enough that a later reader
  takes it for settled. Mark the status explicitly on each item rather than
  leaving it to the prose's confidence.

  The reason is not tidiness, it is that the two kinds of statement **decay
  differently**. A decision is true until something contradicts it and is
  supposed to be read on every visit. A measurement is a record of a run on a
  bed that no longer exists — `biosphere_regen_rate = +141.2 ± 18.1` is a true
  record of the engine that measured it and is not a fact about this one — and
  is supposed to be read once, when someone re-opens the question. Interleaving
  them makes the spec grow without bound and, worse, makes a reader unable to
  tell which sentences they still have to believe. Both of the specs this rule
  was written for had passed 400 and 1,100 lines and were majority history;
  neither could be read for what the engine must do.

  Concretely, per claim, and the last one is the one that gets skipped:

  - **A decision states what holds, its magnitude and units, and whether the
    value is confirmed or a placeholder.** One line, plus a link.
  - **The evidence is a link, not an inlined table.** `See appendix §A.3` with
    the harness name, the bed, and the result in one clause — enough to decide
    whether to go and read it.
  - **An open decision states what would settle it**, which is the difference
    between an R-code and a complaint.
  - **Superseded design moves to the appendix under the decision that replaced
    it, and says what it was wrong about.** A retracted claim that is merely
    deleted takes its refutation with it, and the next reader re-derives the
    same wrong idea — this project has done that at least twice. Strike it
    through in place only when the correction is a *clarification* of the same
    decision; move it when the decision itself changed.

  **A spec section that is doing both jobs is the tell**, and the split is
  mechanical: everything in the past tense with a number in it is appendix,
  everything in the present tense saying what the engine does is spec.
- **Every PR accounts for its T-codes and for every ratified decision it touches.**
  Two lists in the body, and the second is the one that matters:

  1. **Tasks.** Which `T-nn` this **closed**, which it **advanced** without
     closing, and which it **opened**. A staged plan (`Hyades_industry.md` §6.7,
     politics §10.7) is only navigable if each landing says where in it you now
     are — and a T-code claimed as done that is only half done is worse than an
     open one, because nobody re-reads it.
  2. **Ratified decisions implemented or contradicted.** Every R-code, design
     law and ratified magnitude the change **satisfies**, and every one it
     **contradicts or invalidates** — named, with the resolution.

  **The contradiction half is the whole point.** `docs/` is authoritative only
  for as long as nothing lands that quietly disagrees with it; one unannounced
  contradiction and the next reader cannot tell which of the two to believe, so
  they must re-derive everything or trust nothing. Both are worse than a
  sentence in a PR body.

  Contradiction is **not** a reason to stop — this project has landed several on
  purpose, and they were the good changes:

  | Kind | Example | What the PR had to say |
  |---|---|---|
  | A ratified value stops meaning anything | `miners_per_outpost = 3` (+2.74% colony-years, every seed positive) retired at T-72, and its replacement retired at T-87 | *why* the measurement no longer applies — it was taken under a law with no deposit term at all |
  | A prediction in a spec is measured false | §4.5's "fewer, larger, closer" — larger and closer held, **fewer did not** | which half failed, and that site count is `rank`'s decision and not this section's to make |
  | A stated dependency has since landed | R-P16 shipped the `$` faucet against an infrastructure placeholder *because* T-74 had not landed | that it **resolves** rather than ships — the placeholder was never wanted, only unavoidable |
  | A spec formula is arithmetically wrong | §4.3's `ε·S·W` double-counts the deposit, so output goes as richness squared | the corrected form, and that every *ratio* the section asserts survives it (R-IND19) |
  | A ratified default is knowingly moved against the metric | `miner_vein_fraction`, then its deletion | the measured cost, and that it is a design call the objective cannot price |

  **Landing one silently is the defect.** Say which decision, why it no longer
  holds, and what replaces it — then mark it resolved where it is listed, which
  the bullet above already requires. A contradiction that is argued is a
  ratification; one that is not is a divergence nobody knows about yet.

  It also catches bookkeeping faults cheaply: T-83 was used for two different
  things in one session because no landing had been asked to enumerate its
  codes, and the collision surfaced only when the second one was read back.
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
- **Define every term and variable before you use it.** A symbol that appears in
  a formula, a table, a comment or a commit message without a stated meaning is a
  defect, not a shorthand — the reader cannot check the claim, and neither can
  the next measurement. This bit R-IND12: `§1.7` introduced `endowment_fraction`
  in prose, then wrote `× f` in one place and `31.6/f` in another with **`f`
  never defined anywhere**, so the one line carrying the design's actual
  consequence was unreadable.

  Concretely, and in this order:
  - **Name it in full at first use**, then bind the symbol explicitly —
    "…the endowment fraction `f`…" — not the symbol alone.
  - **A spec section with more than two symbols gets a table**: symbol, name,
    unit, and where it is set. `Hyades_industry.md` §3.2 does this and is
    readable years later; §1.7 did not and was not.
  - **Units are part of the definition.** `Band`, `Kilotons` and `Price` are
    different things (§4), and a formula that does not say which one a symbol is
    has already lost the argument the type system exists to win.
  - **Single letters are for quantities with a stated definition nearby**, never
    for a concept. If it takes a sentence to say what it is, it gets a name.
- **American spelling, everywhere.** Code, comments, doc comments, specs, commit
  messages, harness output. `color` not `color`, `center` not `center`,
  `colonize`/`colonizer`/`colonization` not `colonize`/`colonizer`/`colonization`,
  `behavior`, `optimize`, `normalize`, `utilization`, `analyze`, `modeling`,
  `favor`, `honor`, `neighbor`, `meter`, `program`, `judgment`, `maneuver`.
  Also `while` rather than `whilst`, `among` rather than `amongst`, `toward`
  rather than `towards`. This is not a style preference to be weighed against
  consistency with surrounding text — it applies to new text unconditionally,
  and a file being half-and-half is a reason to fix the file, not a reason to
  match its other half.

  The one exception is **flavor text, which is the author's own** (below) — do
  not respell it either way.
- **Weigh the whole simulation before you trust a transfer** (T-118, design
  law #11). `Simulation::mass_ledger` sums every mass-bearing store and
  `mass_is_conserved_with_regrowth_off` asserts the total does not move with the
  one legitimate source switched off. Run it after touching anything that moves
  mass between stores.

  It found **three** leaks on its first run, all of the same shape: a transfer
  with one end missing. Half of every scrapped hull vanished
  (`scrap_recovery_fraction` was a *discard*, not a split, where law #11 says
  wastage degrades to slag). A recycled colonizer was counted as a live hull
  **and** as the infrastructure it became. And founding did
  `f.infra = f.infra.max(credit)`, throwing away whatever was already standing.

  Three habits from it:

  - **Break the ledger out per store.** "Mass changed" names no path; `hulls`
    falling by exactly one hull's mass while `infrastructure` rose by less
    names one immediately.
  - **Localize in time by re-running to increasing horizons.** The run is
    deterministic, so drift is a function of time: the third leak appeared
    between 38 and 40 years, which pointed at the first colony founding without
    any guessing. Two hypotheses had already been wrong by then.
  - **A `max` is where a sum belongs, twice in two landings.** T-117 fixed one
    on the hold and T-118 found the other on the hull credit, in the same
    expression. When two things credit the same field, `max` silently drops
    one — and nothing in the types objects.

  **There are no exceptions left** (T-119). The absorbing-zero floor (§8.7) was
  the last one, and it is a *transfer* now — the founding center is billed for
  the top-up, and a parent too poor to pay leaves its child thin. A guard that
  degrades is worth more than a guard that conjures, because the conjuring is
  what a search finds and calls a strategy (law #11's own warning).

  **And a quantity that comes back has to come back as what it was.** Cost is
  one scalar (R-O57), so a hull's mass returning to a bank — salvage, wreckage,
  a ceiling's overflow — used to be split evenly across the colors, a guess in
  the one dimension the economy is constrained by (§6.19c). `hull_minerals`
  records what the bank handed over; `Minerals::try_take_total` is the capture
  point, because **the bank's mix moves the instant it returns** and a
  reconstruction afterwards reads the wrong proportions. Record the withdrawal,
  do not re-derive it.
- **The standing layer answers questions; it is not switched on** (T-117).
  Design and Doctrine are state written by cards, and a consumer **asks**
  `autopilot::Standing` what is currently active — `design_for(role)`,
  `colonizer_ladder()`, `role_of(hull, class)`, `recycles_on_founding()` —
  rather than branching on the write. Do not add `if doctrine.some_flag { A }
  else { B }` at a call site; add the case to the resolver and let the call
  site keep reading one function.

  The reason is not tidiness. A write that moves a role onto a different hull
  is read in at least three places — the build order, the price the production
  context carries, and the role a finished hull is tasked with — and three
  readings of one write is how they come to disagree. Three landings in a row
  added a write and added those three branches; twice they diverged silently,
  and once (`launch_survey`, T-116) the engine paid for a hull and then
  discarded it because the build site and the spawn site disagreed about which
  hull the role was on.

  **The property that makes it a layer is that the inverse is derived.**
  `role_of` is not a second table — it is `design_for` searched, narrowest
  first, falling through to a competence table (R-O44) for a hull no write has
  claimed. `role_of_inverts_design_for_every_role` checks it across every
  combination of the writes, because they compose and a resolver correct one
  write at a time is not one. A second property is worth asserting beside it:
  the resolver must be **total**, because a hull with no mission is a hull the
  yard was already charged for.
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
| 5 | Colony cargo mass ≡ mineral cargo mass | R-O32 | **done** — `laden_accel` now masses `pop_cargo`; it was massless, so a laden colony ship flew like an empty hull and the burn read out cargo *type*, the one thing §6.2 exists to hide. **Completed at R-WAR9 (T-115):** this row was true of `laden_accel` and not of the dispatcher that flies colony ships — `spawn_courier` read `civilian_accel_g · G` *before* loading the hold and never re-read it, so the colonization leg was still an empty hull's. A laden Medium colonizer makes **0.241 ly/yr² against 2.446 empty**, and fixing it moved every transit-dependent magnitude in the corpus. **A row marked done is a claim about a code path, and this one named the wrong one for several landings.** |
| 12 | Re-base hull mass on surface area (shell), contents on volume | R-O58/R-O58b | **done** — landed with 11; see below |
| 1 | `BuildOrder::Hull { hull_type, class }` + role assigned after production | R-O29 | **done** — the three mission-named variants are gone; `Autopilot::assign_role` returns a `Tasking { role, target }` for the finished hull, and the old `MiningPair`'s freighter is now a consequence of assigning `Role::Miner`. **Behavior-neutral**, verified by stashing the diff: seed 1 / 3 seats / 4 kyr gives 1,183 colonies, 1,594 miner taskings, 5,845 scanned, 240 scouts both with and without |
| 2 | Design/roster component | **R-O28** | **done** — `Roster` (a sorted, idempotent set of `(HullType, Class)`) is a per-player component written only by tree cards. Unblocks σ_vector for Design: the distance between pre- and post-card rosters is now computable. `Class` also introduces the Banks-convention design names (R-O42b: Meadow/Tor proposed, flavour subject to authorship) |
| 3 | Diplomatic fields on `Doctrine` | R-O27/R-A3 | open — no field list specified yet (**T-11**) |
| 4 | Throttle fraction; observe `a` from trajectory not the stat block | R-O40 | open (**T-09**) |
| 6 | `min_time_search` as a reachability-cone query | R-O31 | open — same function, reverse direction (**T-05**) |
| 7 | Route intercept and accept/decline through *believed* `a_max` | R-O41 | **half done** — `src/belief.rs` holds the one-sided estimator (belief is the *max* ever observed, because a ship never flies above peak) and the kinematic accept/decline. Belief is monotone, so masking is spend-once; a 4,851-case sweep pins that it errs only by optimism, which *is* the surprise attack. **Sim wiring blocked on T-30** — there is no accept/decline site in the engine yet (**T-10**) |
| 8 | Permissive role eligibility with varying competence | R-O44 | **done** — roles §4 now states the permissive rule once and every per-role list reads "Competent:", separating *competence* (a degree — an LSV scouts badly) from *capability* (a fact — a Limited hull has no cargo hold, so a Limited Colonizer founds nothing). Engine matches: `assign_role` declines on no viable target, never on hull type |
| 9 | `FAIR_COUNTS` rejects 18 while galaxy §2 lists it fair | R-O12 | **done** — now `[2, 3, 6, 12, 18]`. A radius-`r` hex ring holds `6r` cells, so the family is 6/12/18/24…; 9 and 15 are multiples of 3 but form no ring, so `% 3` would be the wrong predicate. The three ring radii were exactly `N/6 + 1.5`, so the existing `18 => 4.5` branch was the family's third term and the list was one term short — replaced by that closed form. **Balance targets the 2-neighbor configs (3/6/12/18); N=2 is supported but not a balance target**, which also settles R-O9's missing Green as accepted rather than open |
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
normalized to the Medium hull. Dry mass is the mineral cost. Two constants were
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
  | post-gradient-step defaults, 3 seats, 4 kyr *(other hardware)* | 11,602 | 148.8 yr/s | 60× |
  | + mining-pair recycling, same run | 10,965 | 142.4 yr/s | 57× |
  | pre-T-62 bed, 3 seats, 4 kyr *(container)* | 19,406 | 78.7 yr/s | 31× |
  | **T-62 (Banded mineral field), same run** | 23,227 | **35.0 yr/s** | **14×** |
  | + T-64 (logistic on mass, no conversion in the step) | 23,227 | 42 yr/s | 17× |
  | post-R-O70 bed, 3 seats, 4 kyr | ~14,700 | 183–217 yr/s | ~79× |
  | **T-68 (`t_build` tracks hull mass), same bed** | — | **10–11 yr/s** | **~4×** |
  | T-70/73/74 bed (`slips ≡ 1` ablation), 3 seats, 4 kyr | 24,470–25,984 | 9.0–9.7 yr/s | 3.6–3.9× |
  | **T-69 (yards fill every berth), same bed** | 25,200–25,523 | **6.6–7.3 yr/s** | **2.6–2.9×** |
  | T-71/T-72 (crowding, `f = 0.07` crews) | 37,269–37,706 | 9.8–9.9 yr/s | 3.9–4.0× |
  | **T-87 (crew from demand)** | **23,258–24,801** | **8.5–9.0 yr/s** | **3.4–3.6×** |
  | R-O86 bed *(same machine, same session)*, before | — | 14.9 yr/s | 6.0× |
  | **R-O86 (a scout needs somewhere to scout), same pair** | — | **83–98 yr/s** | **33–39×** |
  | R-O89 (freight loads by color), 3 seats, 1.5 kyr | ~32,500 | 93.3 yr/s | 37× |
  | **T-88 (`cycle_years` 50 → 5), 3 seats, 1.5 kyr** | ~35,400 | **71.1 yr/s** | **28×** |
  | **T-88, 3 seats, 4 kyr — `ns/event` 22,394** | 31,337 | **68.9 yr/s** | **28×** |
  | R-O92 off (`max_pickup_stops = 1`), 3 seats, 1.5 kyr — `ns/event` 26,494 | ~32,000 | 69.2 yr/s | 28× |
  | **R-O92 (the milk run), same pair — `ns/event` 29,786** | ~34,500 | **57.8 yr/s** | **23×** |
  | T-94 (logistic in closed form), 3 seats, 800 yr — `ns/event` 50,637 | ~34,700 | 50.4 yr/s | 20× |
  | **T-96 (the drive is a mass), same bed — `ns/event` 51,732** | ~35,100 | **48.0 yr/s** | **19×** |
  | **T-98 (the hauler's hull is a forecast), same bed — `ns/event` 27,812** | ~40,900 | **82.9 yr/s** | **33×** |
  | **T-100 (the Band readings are memoized), same bed — `ns/event` 19,549** | ~40,900 | **114.4 yr/s** | **46×** |
  | **T-101 (the candidate scan prunes what it has rejected)** — interleaved against T-100 in one session: **96.4 → 106.6** and **99.4 → 111.2 yr/s** | ~40,900 | **+11%** | — |
  | **R-WAR9 (the colonization leg is flown laden)**, 3 seats, 800 yr, 4 seeds | — | **83.8–90.0 yr/s**, `ns/event` 25,423–29,206 | **34–36×** |
  | **T-102 (`exp` is a polynomial)** — interleaved, 6 seeds, 800 yr: **104.9 → 107.8**, **109.4 → 112.4**, **107.3 → 110.6**, **114.5 → 118.3**, **120.1 → 123.0**, **108.6 → 110.1 yr/s** | ~40,900 | **+2.7%** | — |

  **R-WAR9's row is a case where the workload changed and the columns must be
  read that way** (§2's T-111 caveat). Flying colony ships at the rate their
  load implies makes every colonization leg longer, so an empire reaches fewer
  worlds inside the same horizon — 2,849–2,955 colonies where the bed used to
  reach more — and a simulation with fewer colonies is doing less work, not
  less work per unit. `ns/event` is flat against the pre-fix bed, which is the
  first row of the reading table: *the simulation is doing less, each unit costs
  the same.* **Nothing was optimized and nothing regressed.**

  **T-88's last row is the one to read, and it is `ns/event` that says why.**
  Per-event cost went from ~174,000 ns at T-87 to **22,394** — not because any
  event got cheaper, but because the mix changed: a ten-times-finer economy tick
  adds millions of *cheap* ticks and the severance kept the *expensive*
  decisions from multiplying with them. Throughput barely moved between the
  1.5-kyr and 4-kyr runs (71.1 → 68.9), which is the first time this table has
  shown that — the superlinear-in-duration degradation it records was entity
  count compounding, and the economy tick does not compound.

  **R-O86's row is the largest speedup in this table and it came from deleting
  work, not from optimizing it** — which is why it is worth more than its
  multiple. At the 4,000-year horizon the engine was issuing **1,779,509 hull
  builds to produce 18,093 hulls**: `apply_build_with` debits the bank and holds
  the yard *before* dispatching the role, and `launch_survey` spawns nothing when
  the frontier is empty, so 99.0% of all production spent minerals and berth-time
  on an object that never existed. Colony count is *identical* across the fix
  (3,340) and colony-years move **+0.007%**. **Before optimizing a hot path, check
  what fraction of the work it does is producing nothing** — no amount of profiling
  would have found this, because every one of those builds was genuinely running.

  **T-87's row is the first with `ns/event` beside it, and it is the number to
  look at**: 173,886 and 176,747 ns/event on seeds 1 and 7. **174 microseconds
  per event** is the real story that `yr/s` was hiding — the engine is not slow
  because it processes many events, it is slow because each one is expensive.
  That points at the production candidate scan (T-52, still `O(scanned)` and the
  largest loop left), not at entity count. Every throughput figure from here
  should carry this column.

  **T-62 halved it, and the mechanism is hauling — not vehicles and not mining**
  (`examples/haul_census`). Vehicles rose 1.20× and extraction ticks 1.02×, but
  **freighter transfers rose 6.81×** because the log-normal field made the same
  rocks hold 2,256× the ore and a freighter's hold is a fixed size. Cost is
  proportional to ore hauled, not to worlds mined.

  **T-69 is the first row where throughput moved and entity count did not.**
  Seed 7 carries *fewer* vehicles under T-69 (25,200 against 25,984) and runs
  27% slower. The cost is **decision count**: every commit schedules its own
  `BuildDecision`, so a yard with `k` berths raises `k` events where it raised
  one, and the fill loop re-runs the candidate scan per berth. Worth having as a
  second shape — the table's standing lesson is "entity count is the
  first-order cost", and that is still true as a default and was false here.
  **Check it rather than assuming it**, the same way §2 says to check a
  neutrality claim against the code.

  **T-68 took ~19x of it in one change, and that is the live problem.** Making
  `t_build` track hull mass dropped a Medium hull from 10 yr to 3.0, so yards
  decide three to four times as often and entity count follows. It bought
  **+10.5% colony-years on both measured seeds** with colony count unmoved, so
  the change is right and is not being reverted — but at **10–11 yr/s** the
  margin at 3 seats / 4 kyr is ~4x, and since degradation is superlinear in
  duration (below), the 12-seat / 8-kyr corner is now **under the floor** rather
  than near it. That corner has still never been measured; extrapolating it
  again would be the third time this table has been wrong about a number nobody
  ran. **Measure it (T-66), then optimize.**

  **`mineral_peak = Band IV` is ratified (R-O82), so this is now the first case
  where the scenario genuinely cannot be shrunk.** The 12-seat × 8-kyr corner
  extrapolates to ~2 yr/s, under the floor, and the only remaining move is to
  make hauling cheaper — the empire is paying full freight for ore it has no way
  to spend, and colony-years did not move for any of it. That is **T-66**, and
  its first step is to *measure* the corner rather than extrapolate it.

  **The 456 yr/s row is stale in a way worth naming.** It was taken before the
  gradient step raised coverage 38% → 49%, and vehicle count is the first-order
  cost: the same scenario now carries **11,602 vehicles against 5,649**. The last
  two rows are that scenario re-measured (`examples/mining_probe -- bench`), but
  on the ephemeral container rather than the machine the rest of the table came
  from, so they are **not** directly comparable to the rows above and do not
  isolate how much of the drop is entity count versus hardware. What they are
  good for is the comparison *within* the pair, which is same-machine and
  same-run: recycling costs ~4% of throughput and removes ~640 vehicles, so it
  is not a throughput risk. Re-measuring the whole table on one machine is
  outstanding.

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
  (T-20). **3,481.0 colonies, mean over the 4-seed CRN bed**, as of T-56 stage
  3b adopting the ratified hull cost ladder (3,459.8 before it, at R-O69's
  decoupling of the production decision from the economy tick; 3,294.0 before
  that; 3,472.5 before R-O66). 4,000 is the run length; the coverage reached
  within it is the objective. Doubling the horizon doubles every trial, and §2's
  60-second rule already had to absorb the snowball once.

  **Colony count is nearly saturated, so read colony-years instead.** The bed
  takes ~99% of what `k_high` admits, which leaves count almost no room to move:
  the T-56 ladder change is +0.6% on colonies and **+8.2% on colony-years**
  (8,011,139 → 8,670,020), with the doubling time 284.5 → 269.4 yr. On a
  saturated bed the informative metrics are *when* the worlds were taken, not
  how many.

  **`growth_rate` is a step function of itself, and the gradient is recorded**
  (T-64/R-O84). It reaches the objective only through how many 50-year cycles a
  center takes to cross a `PopBands` edge — an integer — so the objective is
  piecewise constant in it and a coarse grid picks a plateau *edge* by accident.
  It stays at **0.873**; the measured surface, the `r < 2` bifurcation ceiling,
  and the six things a future search should not have to rediscover are in the
  register under T-64. **Do not sweep it without reading that first** — in
  particular, the objective keeps *rising* past the ceiling because the `clamp`
  at `K` turns a broken logistic into a high-scoring step function.

  **Mineral input is a per-miner rate now, and the default crew is 3** (T-57).
  `outpost_mining_fraction` was the fraction of remaining density a *rock*
  yielded per tick — the hull standing on it contributed nothing but the
  schedule, so "how many miners per outpost" had no term in the model to tune.
  It is now the fraction **one miner** works. Ratified at 3 on the standard bed:
  **+2.74% colony-years, doubling 270.3 → 260.7 yr**, every seed positive.
  Five miners give +3.08% but *lose* 3.5 years of doubling — past three the
  extra hulls compete for the build slots expansion needs.

  **And the cost ladder is no longer the capacity ladder** (T-56 stage 3c).
  `hull_radius` solves `cost·η = r³ − (r − τ)³` instead of square-rooting the
  cost ratio, so `medium_fleet_size` is a price again rather than a price *and*
  a hold. That coupling is what four of the artifacts in §2's table have in
  common, and the `REFERENCE_MEDIUM_RADIUS` normalizer they all ran through is
  deleted. The correct geometry cost −0.31% against the cost ladder alone, so
  the artifact surface closed for free.

  **The bed saturates again, and this time honestly.** 99.4% of everything
  `k_high` admits (98.4 · 99.9 · 100.0 · 99.2 — seed 42 takes every single
  reachable world). The pre-R-O66 saturation was partly bought by an artificial
  cap on deepening; this one is not. `k_high` is once more the only limiter on
  the total, and the ~5% the unit fix cost has been recovered from the place it
  actually went.

  **The unit fix cost −178.5 ± 26.9 colonies (−5.1%), every seed down, 6.6 SE —
  and the obvious explanation for it was wrong.** "Growth is slower because the
  draw is now the real mass" is plausible, mechanistic, right-signed, and
  refuted by two one-line ablations: regrowth on living mass instead of biomass,
  and **the biomass draw deleted outright**, both reproduce 3,294.0
  *bit-identically*. The mass budget does not bind at the shipped defaults, so
  it cannot be paying for anything.

  The actual cause is **policy, not physics**. `k_potential` is the deepening
  guard, and under the old expression it eroded as a world's population ate its
  own biosphere — so centers ran out of deepening headroom and spent minerals on
  expansion. Correcting the units gives them real headroom and they take it:
  fewer colonies, deeper ones (seed 1: 3,426 → 3,227 colonies, mean infra
  1.420 → 1.443, mean `K` 1.418 → 1.430). Some of the old coverage was bought by
  an artificial cap on deepening, and the −178 is a deepen-versus-expand
  reallocation made on correct information — a policy question for `expand_bias`
  and T-20, not a reason to soften the units.

  **Two things to carry forward.** Every gradient measured before this landing is
  consumed: the operating point moved, so re-measure rather than stepping along
  the old direction (`growth_rate` and `biosphere_regen_rate` keep their ratified
  values until then). And **ablate before you explain** — a real number with a
  confident mechanism attached is the exact shape of all six prior measurement
  artifacts in this file, and here it took two disposable one-line variants to
  tell a plausible story from a true one.

  **And it put time back on the critical path.** `reach_limit` on the standard
  bed: the run used to take **99.8%** of everything `k_high` admits — a
  saturated bed, where the classifier was the only limiter and the ramp had
  slack. It now takes **94.6%** (93.9 · 95.9 · 92.6 · 96.0). So there are two
  limiters and they bind in sequence: **`k_high` sets the total** (47–48% of the
  galaxy permanently ineligible, and since R-O66 that set is *exactly* fixed),
  and **the compounding rate of the expansion loop sets the time** — founding
  rate ×~2 per 500 yr, peaking at 3,000–3,500 yr on all four seeds and turning
  over only in the final bucket. Survey is not it (11–41 above-gate worlds
  unscanned per seed) and neither is the economy (the biomass draw is slack;
  minerals were ruled out at R-AC17); the residual is 126–216 worlds per seed
  that were **scanned and not reached in time.**

  **Both halves of that time constant are now resolved — the second by fixing
  it (R-O69).** The decision was pinned to the same 50-year tick as the
  economy, which capped every center at one build per cycle however rich it
  was. Measured before the change (`examples/cadence_throttle`): **the median
  funded build fired at 5.5x the price of what it bought**, 82% of funded
  builds could have been made at least twice that cycle, 55% at least five
  times, worst case 112x. Decisions are now events — a build occupies the yard
  for `build_years` and the next decision comes when it clears — which takes
  the ceiling from one build per 50 years to one per 10 for a center that keeps
  finding things to buy. **+165.8 colonies (+5.0%), and the bed saturates.**

  **It cost 4.1x throughput, and R-O70 gave 3.3x of it back.** The decoupling
  took the standard bed from 235–269 to 61–66 yr/s (vehicles ~10,350 → ~14,700,
  events ~237k → ~421k); memoising `holdings_centroid` and fixing two container
  choices in `Knowledge` brought it to **183–217 yr/s** with **colony-years
  bit-identical** (7,819,401.0 and 8,480,172.0 on seeds 1 and 7). So R-O69's
  +165.8 colonies now cost about a quarter of the throughput rather than four
  times it, and the margin against T-24's floor is ~79x at 3 seats / 4 kyr with
  the 12-seat / 8-kyr corner extrapolating to ~3.8x rather than ~1.3x. The
  production candidate scan is still `O(scanned)` and is now the largest loop
  left (T-52).

  **Colony-years is the guard for work like this** (`examples/colony_years`).
  Colony *count* at the horizon is a weak invariant for an optimization — a
  change that founds the same worlds a century later scores identically — while
  `∫ colonies dt` falls the moment anything slows down. Hold it fixed to the
  decimal and a performance change is provably behavior-preserving.

  **And the time constant now has a proven mechanism, not just a name (R-O68,
  T-51 — both now closed).** `production_choice` preferred depth when
  `b · deepen_headroom ≥ (1 − b) · score`, and those sides were in different
  units — a Band difference bounded by 4 against `rank`'s unbounded weighted
  score. Measured (`examples/score_scale`): colony-class scores run p05 4.40 /
  median 6.17 / max 12.16, and the branch compares against the *max*, so depth
  won only at `b ≳ 0.8`: a step function wearing a dial's clothes, with no
  graded region for a search to climb. Same root cause as the `K` unit error one
  section up.

  **And the knob cannot move Growth's own objective either, by identity
  (R-O87).** Work-years is `∫ Σ_p infra_p dt`, and the two things
  `reinvest_bias` chooses between are worth the same to it: deepening bills
  `infra_step_price / eta_works` and raises works by `infra_step_price`, while
  founding bills the colonizer's price and the new colony's stock is
  `founding_infra = hull_cost` — the recycled hull's minerals *are* the stock
  (T-70) because a hull's mass is its cost (R-O57, design law #11). At the
  card-free `eta_works = 1` those are identical to the last bit, at every rung,
  and `a_mineral_buys_the_same_works_whether_it_deepens_or_founds` pins it.
  Measured to match: **+0.32% ± 1.42 over eight seeds** at 4,000 yr. It stays at
  **0.5**, and the lever it is not is `eta_works` — which divides the deepening
  bill and nothing else, so a Production card genuinely does make a mineral buy
  more works.

  **Both sides are now `rank` score per kilotonne committed** —
  `score / outward_cost` against `w_k · min(1, headroom) / infra_cost`, using
  `rank`'s own weight rather than a new constant — so the comparison is an odds
  ratio with a state-dependent crossover. And the result is the lesson: the run
  is **bit-identical below `b = 0.96`** on both seeds (`examples/deepen_census`),
  the old cliff at 0.9 moved to 1.0, and **the branch is still cold at the
  shipped `0.5`**. The infra rung above the founding one costs 0.9 kt against a
  Medium colonizer's 0.10 kt, `fabrication_rate` saturates by rung II, and
  `slips` is pinned at 2 from rung I onward because `fab_cap / slip_throughput =
  2` — so expansion returns 24–49x per kilotonne and *should* win. **The dead
  branch was the right answer reached for a wrong reason**, and the cause moved
  to the price ladder (R-O85/T-89) rather than going away.

  **And that `slips` figure was a spec contradiction, not a design magnitude
  (R-O88, now resolved).** §3.2 says the build-wide axis "scales without limit";
  §6.3 bounded `F` by `fab_cap`; `slips` read `F`. So the axis was **closed at
  two berths** — 10¹² kt of infrastructure still bought two — and homeworlds are
  *generated* past the only step it had. §6.3's reconciliation ("the bound is per
  yard, and an empire has many yards") answered a question about the empire
  total, not about a center's berth count, and is withdrawn.

  **The fix was to notice that one variable was doing two jobs.** `fab_cap` now
  bounds the rate **per berth** (quality) and `slips` reads the fabrication share
  of the **stock** (quantity, unbounded) — so both sections are true at once and
  §5.3's tree table reads off the engine directly: Production buys fast berths,
  Expansion buys many slow ones. Berths at rung II went 2 → 17; **fleet-years
  +26–34%** on both seeds, colony count flat (the bed is `k_high`-saturated, so
  more yard cannot buy worlds the classifier does not admit), and **throughput
  *improved* 88.7 → 110.7 yr/s** because a quarter of the events had been
  decisions that declined and stalled.

  Three things from it worth keeping:

  - **When two ratified claims collide, check whether one symbol is carrying two
    meanings before you pick a winner.** `F` was a per-planet rate *and* the
    berth currency. Splitting it cost one function and contradicted neither
    section — where choosing between them would have contradicted one.
  - **A re-denomination is not a retune, and it is worth engineering for.**
    `fab_cap` 0.2 → 0.1 reproduces the old per-berth rate *bit-for-bit* because
    the old `slips` was always exactly 2, so turnaround did not move at all and
    the only behavioral change in the landing is berth count. That is what makes
    the measurement readable.
  - **The cost landed in the test targets, not in throughput** — unit 19 → 54 s,
    determinism 30 → 58 s — exactly as §2 predicts, and both were fixed in the
    same commit. `full_run_reports_are_bit_identical` got a **per-seat-count**
    horizon rather than a uniform one: cost goes as seats × years, so one horizon
    for everybody made the 18-seat arm pay for the target while the 2-seat arm
    ran 812 events. Equalising was **cheaper and covered more**. When a shared
    horizon feeds arms of very different size, that is the trim to reach for.

  So the loop's time constant is the **unconditional pre-`medium_min_level`
  staircase** — found at `K = 1` with no headroom, then serially mine
  `round(infra)+1` minerals, deepen, grow past the level-3 `PopBands` edge,
  afford `colonizer_cost`, fly — all quantized to `cycle_years = 50`. **Half
  those gates are discrete** (`medium_min_level` and `limited_min_level` are
  `u8`; the infra cost ladder is hard-coded, not a parameter), so a
  central-difference probe has no gradient to read on them. That is why every
  probe so far has ranked ecology and hull-cost knobs: *the rate limiters are
  invisible to the instrument.* Sweep the gates discretely, widen the
  continuous surface to role and hull costs, and fix the comparison's units
  before touching the policy's shape.

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

  **And the whole mineral side of that argument has one cause, measured
  three ways** (`Hyades_industry.md` §6.17). Colonies sat at **Band 1.05
  against a ceiling of 3.60, with zero at cap and 756 Bands unbuilt** while the
  empire banked the ore to build it. It was read as R-O68's dead deepen branch;
  closing R-O68 left the number unmoved, and **R-O86 then moved it to Band
  1.462** by unblocking the `outward == None` fallback that a survey pre-emption
  had been swallowing. Still zero at cap against a 3.61 ceiling, so the residual
  cause is **the price of a rung** (R-O85).

  **And R-O85 — "infrastructure is priced out of reach" — is resolved as false
  (§6.19c).** Post-R-O88 the ladder is **scale-free**: cost and output are both
  geometric in the stock, so a rung pays for itself in **1.8 years at every
  rung**. Counted per decision on the bed, **0%** are gated at the ceiling,
  **0.7–0.9%** are outbid — so R-O68's crossover, which three sections of that
  document circled, is consulted in one decision per hundred and cannot have been
  causing anything — and **98.3% simply cannot pay the bill**, of which
  **43.8–46.6% of *all* decisions hold the total and lack a color.**

  **So the mineral economy's binding constraint is freight**, and the chain of
  four investigations that ended there is itself the lesson: units (R-O68),
  survey waste (R-O86), works identity (R-O87), the build-wide axis (R-O88) and
  the price ladder (R-O85) were each real and none was the cause. **What finally
  named it was logging the decision's own predicate instead of reconstructing
  it**: `can_afford_infra` is a per-color test, and inferring it from a *total*
  reported 44.7% "outbid" where the truth is 0.9%. A reconstruction that looks
  arithmetically equivalent is not, when the thing it reconstructs is a
  conjunction over three colors.

  **So every flat mineral-side result this project has recorded is downstream of
  the same thing** — `outpost_mining_fraction`, both crew policies, and the
  Exchange. Two things follow and the second is the transferable one. **They were
  blocked on freight moving color (T-76), not on price or policy**, and
  re-measuring any of them before that landed measured the same wall again.

  **It has now landed, on the load leg (R-O89, `Hyades_industry.md` §6.20).** A
  hauler fills against the destination's color deficit instead of in proportion
  to the pile it happens to be standing on — **+8.40% ± 1.86 work-years, 8/8
  seeds, replicated on four the candidate was not chosen against** — and it does
  it on *the same tonnage*: 20,259 → 20,292 kt over 26,800 → 26,746 trips. Same
  fleet, same trips, same round trip to two decimals; only the colors in the
  hold changed. **That is the cleanest confirmation of a diagnosis this project
  has produced** — the census said the constraint was color composition, and a
  change touching nothing but color composition moved the objective. The block
  above is lifted; each of those knobs is now its own re-measurement. And:
  **a metric that reads a decision's output cannot tell you what the decision
  declined to ask for** —
  `unmet_color_demand` summed each center's shortfall against its *next* rung,
  so a center with three Bands of headroom it never tried to buy reported zero
  demand, and the first conclusion drawn from it ("the economy has no demand
  side") was exactly backwards.

  **And the pickup leg followed it (R-O92/T-91), for four times as much.** A hold
  was filled from one map entry — `outpost_stock[(player, rock)]` — so every
  delivery was mono-colored whatever the routing; letting one outbound leg visit
  two piles is **+55.13% ± 4.65 work-years, 8/8 seeds**, and moves `bank_mix`'s
  payable fraction off the 0.043 it had held through three interventions. What it
  exposes is the next constraint and is worth knowing before the next freight
  idea: **freight is 1.73% of everything that ever enters a bank** — the rest is
  `sys_production_tick` mining the center's own single-colored planet directly
  into the bank (T-92).

  **And T-98 moved that ceiling by building the right hull for the rock**:
  freight's share of bank inflow **1.73% → 14.70%**, ore ever collected
  0.21% → 2.30%, for **+170.1% ± 16.2 work-years, 8/8 seeds**. `role_hull_type`
  was a constant whose rationale — *"picking the cheaper"* — was correct under
  the pre-R-O58 ladder and backwards for every landing since; it survived because
  the General hull's *turnaround* made it a bad idea for an unrelated reason,
  which T-96 had just removed. **Check whether a constant's stated reason still
  holds after you fix something else** — two of this project's largest results
  were one rule waiting on another.

  **Diminishing returns are visible, and the cause is now known.** +23.9, then
  +11.0, then +2.3 points. It is not that the economy ran out of headroom:
  `examples/reach_limit.rs` shows the binding constraint is **`k_high`, not the
  economy** — the bed already colonizes 90–100% of what that threshold
  *admits*, and it admits only 51–53% of the galaxy. The objective's
  denominator counts every world with `min(hab,bio) > 0.01`; the classifier
  does not.

  That predicts, correctly, that further economic gains stop adding.
  `growth_rate` (+2.31 alone) and mining-pair recycling (+1.19 alone on the
  standard four) combine to **+2.53, not +3.50** — recycling's marginal
  contribution on top of `growth_rate` is +0.22, inside the noise. Two
  independent, individually-real improvements, mostly canceling because they
  compete for headroom that is not economic. **Before tuning another economic
  knob, check whether the thing you are optimizing is what is actually
  scarce** — and note that both gains were measured on parallel branches
  *without each other*, which is the same operating-point trap in a new
  costume: concurrent work needs a joint re-measure at merge, not addition.

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
- ~~**No mineral seed for colonies** (homeworlds only)~~ — **superseded
  (R-O74/`Hyades_industry.md` §1.7).** A colonizer's hold is one kiloton budget
  carrying any mix of settlers and minerals: whatever volume the people do not
  fill leaves with minerals **out of the founding center's own bank** and lands
  in the new colony's stockpile. That is not a grant, it is a transfer a parent
  paid for — which is the distinction the old rule was protecting. Mining
  outposts with need-based hauling are still how a colony feeds itself
  thereafter.
- Idle military units do **not** auto-scrap; only exhausted LCVs scrap, others go to
  Reserve ("completable vs. standing mission").
- **K is player-relative by design**, and since `Hyades_industry.md` §1.1 it is
  `min(hab, bio_max)` — **infrastructure is not a term in it.** Infrastructure is
  the industrial stock; it mines and fabricates and can be razed, and razing it
  must not move population. That is the whole reason it left the minimum: T-64's
  logistic overshoots *below* a cut ceiling rather than settling at it, so while
  infrastructure sat in `K`, every industrial strike was a population strike.
- Galaxy distribution: XY Poisson (exponential disk profile); Z exponential with Z-max
  matching the XY parameter.
- **The mineral field is a Gaussian over *Bands*, stored as mass** (T-62). So it is
  log-normal in kilotons: a `Band IV` seam holds ~715,000× a `Band I` one, which is
  the design's "very high value planets located near each other" and not a bug. Two
  traps follow. **There are no barren worlds** — `Band(0).in_kilotons()` is the
  `Empty` rung, not zero, so the Gaussian's tail floors at a trace rather than
  decaying to nothing. And **"is this world rich?" has two readings that now differ by
  more than a Band**: `MineralField::abundance()` (the Band of the *total* mass,
  dominated by the richest color) and the per-color Band sum that
  `BaselineAutopilot::rank` compares against `mineral_high`. Routing §4.4's
  anticorrelation through the wrong one cost **−52% colony-years** before it was
  caught; the correct reading there is the *mean* Band.
