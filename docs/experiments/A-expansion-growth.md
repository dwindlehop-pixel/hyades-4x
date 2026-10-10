# §A. Expansion and Growth

*Part of the experiments record. Nothing here is normative. Table of contents: [`README.md`](README.md); how the record is organized: [`AGENTS.md`](AGENTS.md).*

*Moved out of `docs/Hyades_autopilot_colonization_growth.md` at Rev 4. That spec
had reached 430 lines and was majority history.*

---

## A.1 R-AC3 — survey strategy has no measurable effect

**Harness:** `examples/survey_strategy_search.rs`. **Bed:** 4-seed CRN, 3 seats,
objective *years to 10% colonized*.

Three candidates on `Doctrine::survey_strategy`: `GlobalPool` (no heading bias
ever), `OpeningSectors` (the six bootstrap craft keep a soft cube-face
preference for their whole hop chain; every later paid Scout pools globally),
`PersistentSectors` (later Scouts also inherit a heading, outward from home
through the center that built them).

**All three land within 2 SE of each other** — 2,765–2,865 yr mean. No
significant difference.

**Why, diagnosed rather than assumed** (`examples/colonization_ramp_trace.rs`):
a homeworld crosses the medium-hull gate at t≈200 yr (seed 1), and known
candidates already vastly outnumber what the treasury can afford that cycle — 98
candidates by t=200, 151 by t=400, against a colonizer costing ~0.22 minerals
out of a ~1.5-mineral stockpile. **Survey was never the binding constraint at
any point measured.** A survey-targeting knob has no lever to pull in a regime
where the map is not the scarce resource.

Decision: ship `OpeningSectors`, unchanged, because every existing coverage
number in the tree was measured against it.

---

## A.2 R-AC20 — a sign conflict that was an artifact of two operating points

**The claim, now withdrawn:** *time-to-10%-colonized* and *coverage-at-4,000-yr*
disagree on the sign of `medium_fleet_size`.

**The refutation.** The two elasticities being compared were **measured at
different operating points**: coverage's `+32.7 pts/ln` was taken at
`medium_fleet_size = 3.0`, before the gradient step moved the shipped value to
4.45; the `+161.5 yr/ln` was taken at 4.45. A gradient is local, so the two were
never comparable and "the objectives disagree" did not follow from them.

Measured directly at the shipped value (`examples/proxy_metric_calibration.rs`,
±25%, 3 seeds), coverage is an **interior optimum**:

| seed | `lo` = 3.34 | **4.45** | `hi` = 5.56 |
|---|---|---|---|
| 1 | 48.6% | **49.8%** | 42.2% |
| 7 | 49.4% | **51.3%** | 49.5% |
| 42 | 41.8% | **46.2%** | 35.5% |

Both metrics agree at the operating point that ships. **The methodological
lesson is the yield: never compare two gradients taken at different operating
points** — the same "a gradient is local" caveat that has now produced a project
artifact rather than merely warning about one.

Two things the detour produced that outlived it: `rank.k_high` confirmed as a
**knife-edge rather than a slope** (±25% collapses coverage in *both*
directions — seed 1: 8.0% at 2.4, 0.6% at 4.0, against 49.8% at 3.2), and the
`colonies@2000` screening metric (ρ = 0.923 against true coverage at 31× less
cost).

> **Superseded since:** `medium_fleet_size` is now **10.0**, ratified with
> `limited_fleet_size = 50.0` on the cost ladder (+8.6% colony-years, doubling
> 284.5 → 265.0 yr, every seed positive). The 4.45 optimum above was an optimum
> of the *old* ladder, where cost and hold were the same knob. See A.8.

**Still open from this entry:** `center_mining_fraction` came back `~noise`
(1.33 SE) and wants a ten-seed bed.

---

## A.3 R-AC16 / `survey_reserve` — a threshold above the range of what it thresholds

**Harness:** `examples/survey_timing`, seed 1, 600 planets, 1,500 yr.

`survey_reserve` was ratified at **1024** and is compared against
`candidate_count` — known, unclaimed, non-Barren worlds — whose **median is 0
and whose maximum over the entire run is 164**. So the predicate
`candidate_count < survey_reserve` is a constant `true`, and **every value above
~200 is bit-identical.**

That is the plateau `AGENTS.md` §2 records as a measurement artifact: 2048 reads
as noise (+3.5 ± 2.6), while 512 → −21.8, 256 → −96.8, 64 → −840 fall off a
cliff. The ratification was not wrong about the *direction*; the magnitude
simply never reached the simulation.

**Superseded by R-O86** (A.4): the real precondition for a scout is
`survey_frontier`, not this.

---

## A.4 R-O86 — both survey tests read the wrong quantity, and 99% of production built nothing

`candidate_count` means "known and still available"; the survey question is "is
there anything left to explore". Those coincide early and diverge permanently,
and nothing in the types could tell them apart — both are `usize`.

`candidate_count` goes to zero the moment everything *scanned* is owned or
already targeted, which in a colonized galaxy is the common case. So both survey
paths fired almost always: the `candidates.is_empty()` **pre-emption** (justified
in the code by "no candidates means every other branch returns Idle" — false, and
load-bearing, because the branch it pre-empted was the `outward == None` deepen
fallback), and the `survey_fallback`.

**And the hull was never built.** `apply_build_with` debits the bank and holds
the yard *before* dispatching the role, and `launch_survey` spawns nothing when
`choose_survey_target` returns `None`. The order destroyed mass rather than
converting it.

**Measured** (`examples/deepen_census`, 3 seats, `reinvest_bias = 0.5`), at the
4,000-year objective horizon:

| | seed 1 before | seed 1 after | seed 7 after |
|---|---|---|---|
| hull builds | **1,779,509** | **18,093** | 16,207 |
| infrastructure builds | 88 | **1,539** | 1,445 |
| mean infrastructure | Band 1.027 | **1.462** | 1.432 |
| colonies | 3,340 | **3,340** | 3,349 |
| colony-years | 10,877,084.4 | **10,877,821.2** | 10,969,501.8 |
| wall clock | 268.8 s (14.9 yr/s) | **48.1 s (83.2 yr/s)** | 40.8 s (98.0 yr/s) |

**99.0% of everything the empire built was a hull that never existed.** The
objective does not move when that stops — colony count is *identical* and
colony-years differ by **+0.007%**. What moves is throughput (5.6×) and depth
(infrastructure builds ×17.5).

**Before optimizing a hot path, check what fraction of the work it does produces
nothing.** No amount of profiling would have found this: every one of those
builds genuinely ran. The tell was a count that did not reconcile.

---

## A.5 The mining knobs are measured and are nearly all noise

**Harness:** `examples/mining_probe`, `gradient_probe` method (CRN, paired
central differences, elasticity, an SE on every number). **Bed:** 4 seeds,
objective colonies.

| knob | value at measurement | d/dln x | SE | verdict |
|---|---|---|---|---|
| `rank.mineral_high` | 2.0 | **+3.92** | 1.86 | the only one clearing 2 SE, and barely |
| `rank.mineral_pressure_gain` | 1.0 | −3.61 | 1.88 | ~noise |
| `outpost_mining_fraction` | 0.238 | +3.49 | 2.58 | ~noise |
| `rank.w_mineral` | 0.8 | +2.34 | 1.33 | ~noise |
| `mining_tick_years` | 50 | −1.97 | 2.32 | ~noise |
| `density_floor` | 0.01 | +0.07 | 0.14 | flat — inert here |

`outpost_mining_fraction` measured +14.5 ± 5.8 at 0.20 *before* the gradient step
and +3.49 ± 2.58 at the ratified 0.238 — which is what a knob moved onto a local
optimum should look like, and an independent confirmation of that step.

**Five of six cannot be told from noise.** What is left on this surface is
*terms*, not values. A.6 is one.

> **Context added later:** every flat mineral-side result in this project —
> these knobs, both crew policies, and the Exchange — turned out to be downstream
> of one thing, freight not moving color (R-O89/R-O92, §B.4 and
> `Hyades_industry.md` §6.19c–6.20). Re-measuring any of them before that landed
> measured the same wall again.

---

## A.6 R-AC19 — mining-pair recycling, and three passes to measure it

A pair is built for **one** rock: `Shuttle { outpost, .. }` fixes the pickup leg
at spawn. Exhaustion therefore used to end both hulls' working lives. Measured
on seed 1 (`examples/mining_probe -- census`, 3 seats, 4,000 yr): 2,188 outposts
opened, **2,029 mined out**, mean productive life of a rock 808 yr, and
**2,769,957 idle hull-years** across 5,310 hulls — roughly five centuries each.

**And the freighter was never told.** `sys_mining_tick` stops when the *yield*
falls to the floor (metallicity 0.042 at the shipped values) while
`sys_freighter_arrive` waited for metallicity `< density_floor` = 0.01. In that
band the mine is dead and the hauler flies empty round trips for the rest of the
match: **not one freighter of 2,655 ever reached its stand-down branch.**

**Resolution:** `SimConfig::recycle_mining_pairs` — an exhausted pair goes to
Reserve (which is what roles §4.6 already says a *standing* mission that ends
does, as against the *completable* mission of an exhausted Scout, which scraps)
and the next center ordering a pair takes the reserved hulls nearest its target.

| bed | measurement | SE | verdict |
|---|---|---|---|
| 4 seeds, first cut | +0.76 | 0.42 | 1.8 SE — below the bar |
| 4 seeds, pricing corrected | +0.78 | 0.35 | 2.2 SE |
| 4 seeds, predicate corrected | +1.19 | 0.64 | 1.9 SE — effect grew, so did variance |
| **8 seeds** | **+1.69** | **0.53** | **3.2 SE — adopted** |

Final: 49.11% → 50.30% on the standard four, **47.67% → 49.36% over eight,
positive on all eight and negative on none.** Seed 42 alone swings +3.0 points,
which is why four seeds could not resolve it.

**Three corrections, all load-bearing:**

- **The first measurement was of the wrong quantity.** It reported "outpost-years
  spent on a dead rock, 39%" — but outpost-years are a property of the *rock*,
  and recycling cannot change how long a world holds ore. It duly reported
  39% → 40% and made the change look inert.
- **The decision was pricing a pair it was not going to buy.**
  `ProductionContext::mining_pair_cost` quoted the full price even when hulls sat
  in Reserve, so a center too poor for a new pair sat Idle beside hulls it
  already owned.
- **The freighter fix is inert on its own** — with recycling off, the corrected
  predicate reproduces 49.11% to the digit. What it *buys* is the freighter half
  of recycling.

Census, seed 1, same ~2,620 mining missions either way: miner hulls built
2,655 → 1,889, freighters 2,655 → 1,867, missions flown by a re-used hull
0 → 1,482, idle hull-years 2,769,957 → 1,236,512. **29% fewer hulls for the same
work.** Throughput cost ~4%.

---

## A.7 R-AC17 — `k_high`, and "the snowball is the design"

`RankWeights::k_high` was miscalibrated against the current galaxy and was the
binding constraint on the entire expansion loop. At the old **1.5**, against a
galaxy where 99% of planets have `min(hab, bio_max) ≥ 1.76`, the *Mining outpost*
class was unreachable: measured on seed 1, **zero** mining pairs were ever
ordered and **zero** freighter legs ever flew, leaving need-based hauling dead
code at runtime.

At **3.2**, just above the galaxy's median K (~3.22), the low-K half classifies
as mining and the high-K half as colonies. Probing `k_high` alone took seed 1
from 41 colonies to 537; with `survey_reserve = 1024`, `max_survey_hops = 120`
and an 8,000-year horizon it reaches **100% of colonisable worlds on 4 of 4
seeds**.

**"Colonisable" there means *at or above `k_high`*, and that is the whole of
T-20's gap.** The denominator is the set this threshold admits — 3,435 of seed
1's 6,725 planets, 51.1% — not the coverage objective's `min(hab, bio_max) >
0.01`, which is effectively every planet.

---

## A.8 Reach-limit — two limiters that bind in sequence

**Harness:** `examples/reach_limit.rs`, standard 4-seed bed, 4,000 yr.

Before R-O66 the bed was *saturated* — 99.7 · 99.9 · 99.7 · 99.7%, mean **99.8%**
of everything the gate admits — so `k_high` was the whole story and time was
free. After: **94.6%** (93.9 · 95.9 · 92.6 · 96.0). So:

- **On the total: `k_high`.** 47–48% of the galaxy is permanently ineligible,
  and since R-O66 that set is *exactly* fixed (`gate_erosion = 0` on every seed).
- **On the time to reach it: the compounding rate of the expansion loop.**
  Colonies founded per 500 yr, seed 1: `11·21·52·192·274·524·1103·1047` —
  near-geometric at ×2 per bucket, peaking at 3,000–3,500 yr on all four seeds
  and turning over only in the final bucket, where 83% of what remained is taken.
  Genuine saturation, not a stall; the horizon lands just past the knee.

Survey is not the limiter (only **11–41 worlds per seed** above the gate go
unscanned, 0.3–1.2%) and neither is the economy (the biomass draw is measurably
slack; minerals were ruled out at R-AC17). The residual is **126–216 worlds per
seed that were scanned and not reached in time**.

**Diminishing returns are visible and the cause is known:** +23.9, then +11.0,
then +2.3 points across three ratifications. `growth_rate` (+2.31 alone) and
mining-pair recycling (+1.19 alone) combine to **+2.53, not +3.50** — two
independent, individually-real improvements mostly canceling because they
compete for headroom that is not economic.

---

## A.9 R-O66 — the unit fix cost 178 colonies, and the obvious explanation was wrong

Correcting `K`'s units cost **−178.5 ± 26.9 colonies (−5.1%), every seed down,
6.6 SE**.

**"Growth is slower because the draw is now the real mass"** is plausible,
mechanistic, right-signed — and refuted by two one-line ablations: regrowth on
living mass instead of biomass, and **the biomass draw deleted outright**, both
reproduce 3,294.0 *bit-identically*. The mass budget does not bind at the shipped
defaults, so it cannot be paying for anything.

**The actual cause is policy, not physics.** `k_potential` is the deepening
guard, and under the old expression it eroded as a world's population ate its own
biosphere — so centers ran out of deepening headroom and spent minerals on
expansion instead. Correcting the units gives them real headroom and they take
it: seed 1, 3,426 → 3,227 colonies, mean infra 1.420 → 1.443, mean `K`
1.418 → 1.430. **Fewer colonies, deeper ones** — a deepen-versus-expand
reallocation made on correct information.

It also retired a claim: the class threshold *is* a fixed set. The 240/207/286/275
worlds per seed that "dropped out of the class that made them settleable" were
the unit error, not a design property. `gate_erosion` is now structurally zero and
is kept as a **guard**: the first card that lowers a world's pristine biosphere
makes the denominator playable again.

---

## A.10 R-O68 — a dead branch, proved three ways, and fixing it changed nothing

`production_choice` preferred depth when `b · deepen_headroom ≥ (1 − b) · score`,
and **the two sides were in different units** — a Band difference bounded by 4 on
the left, `rank`'s unbounded weighted score on the right. Measured
(`examples/score_scale`, seed 1) colony-class scores run p05 4.40 / median 6.17 /
max 12.16, and the branch compares against the *maximum*, so depth won only at
`b ≳ 0.8`. A step function wearing a dial's clothes.

**Resolving it moved the diagnosis rather than the behavior.** Both sides are now
`rank` score per kilotonne committed. Measured (`examples/deepen_census`, seeds 1
and 7) the run is **bit-identical below `b = 0.96`**, and the branch is **still
cold at the shipped 0.5** — because an infra rung above the founding one costs
0.9 kt against a Medium colonizer's 0.10 kt, so expansion returns 24–49× per
kilotonne and *ought* to win.

**The dead branch was the right answer reached for a wrong reason.** Had the fix
landed without pricing what the branch would have chosen, the next step would
have been to tune a dial toward a decision the economics say is bad.

---

## A.11 R-O87 — `reinvest_bias` cannot move Growth's objective, by identity

Work-years is `∫ Σ_p infra_p dt`, and the two things `reinvest_bias` chooses
between are worth the same to it: deepening bills `infra_step_price / eta_works`
and raises works by `infra_step_price`, while founding bills the colonizer's
price and the new colony's stock is `founding_infra = hull_cost` — the recycled
hull's minerals *are* the stock, because a hull's mass is its cost. At the
card-free `eta_works = 1` those are identical to the last bit, at every rung.

Measured to match: **+0.32% ± 1.42 over eight seeds** at 4,000 yr.

**And the replication is why that number is trusted.** The standard four-seed bed
gave `b = 0.972` at **+2.33% ± 0.96 with 4/4 seeds positive** — 2.4 SE *and* a
1-in-16 sign test. On seeds 2, 3, 5, 11 it scores **−1.70% ± 2.42**, 1/4 positive.
Neighbors swing the full magnitude in both directions (0.968 → −0.20% ± 0.76,
0.975 → +2.24% ± 2.08), which is a chaotic reordering of a compounding run rather
than a gradient.

**A replication set cannot inherit whatever made the original four agree.** More
seeds on the bed the candidate was *selected* on shrink the error bar around a
number chosen partly because of those seeds; a fresh set does not.

---

## A.12 T-90 — a decision that was provably blind, with nothing to see

`BaselineAutopilot::rank` scored a world's minerals as `Σ_c scarcity_c ·
Band(m_c)` with `scarcity_c` written once at game start from the homeworld
archetype and never again — so outpost selection could say *mine more* and never
*mine Cyan*. A real defect, visible in the code.

Replacing it with the deciding center's live shortfall moved the mechanism check
from **0.043 to 0.043**, cost **−3.30% ± 0.49 colony-years on 0/4 seeds**, and
was reverted.

**The premise was refutable before a line was written.** The empire's outpost
holdings are **957k / 905k / 626k kt** across the three colors — already
balanced. The decision was blind and had nothing to see. Swept across an order of
magnitude (0 / 1 / 4 / 16) the payable fraction reads 0.043 / 0.043 / 0.057 /
0.045 and the dead share is 99.7% at every one, which is what separates "no
effect at the shipped value" from "the axis does nothing".

---

## A.13 T-94 — what the Euler logistic was hiding, and what the sweep that found it was measuring

**Two retracted claims, kept because both were true of the engine that held
them and neither is true now.**

**Retracted: `growth_rate < 2` is an arithmetic ceiling.** It was — of the *Euler
map*. `x + r·x·(1 − x/K)` is conjugate to the logistic map with `μ = 1 + r`, so
the fixed point at `K` period-doubles at `r = 2` and goes chaotic near 2.57 (May,
*Nature* 261:459–467, 1976). Under the closed form `e^(−rΔ) ∈ (0,1)` at every
positive `r`, so the map is monotone at any rate and the clamp can only fire on a
last-bit rounding. **The ceiling was a property of the integrator, not of the
model.**

**Retracted: the clamp at `K` hid the bifurcation from below.** It did — a
population climbing toward `K` overshot, clamped exactly to it, and the growth
term went to zero, so a too-large `r` did not oscillate visibly. It **collapsed
the logistic into a step function that filled a world in one cycle and scored
well while doing it.** That is why the objective kept *rising* past the ceiling
and why a parameter sweep could not see the problem — only the shape of the
approach could.

**The sweep that led here was measuring its own step size.** `cycle_years` 50 → 1
reported **+58.6% work-years**, and the tell was that it was too good.
`growth_rate` is documented `1/cycle` and the logistic stepped it once per tick
**regardless of how long the tick was** — as did `biosphere_regen_rate` and the
center's mining fraction. Shrinking the tick did not integrate the same economy
more finely, it ran a fifty-times-faster one.

**The fix was re-denomination, not retuning.** `tick_scale` multiplies each rate
by `cycle_years / rate_reference_years`, exactly `1.0` at the cadence they were
ratified at — bit-identical on the shipped bed, **no Monte-Carlo-tuned magnitude
moved.**

**What survived the correction was still large.** With the denomination fixed,
refining the tick is **+21.00% ± 4.19 work-years, 8/8 seeds**, because
`r·dt = 0.873` is a genuinely bad Euler step: a homeworld's population at 300 yr
goes 1,143 → 2,275 → 3,516 → 4,297 as the tick goes 50 → 25 → 10 → 5 — the coarse
step **under-integrates by ~4×**. *Do not let the artifact discredit the question
it was asked about.*

**And the test asserts convergence, not invariance.** Those two failure modes are
indistinguishable in one ratio and obvious across three: an integrator converging
moves the answer *less* with each refinement (1.99, 1.55, 1.22), while a rate
applied per tick without scaling moves it by the step ratio every time, forever.
The first version of that test asserted invariance, failed at 2.92×, and was
wrong to — which is how the distinction got found.

**Closed form versus finer step, decomposed.** Exact at the *coarse* tick beats
Euler at the coarse tick by **+33% work-years at the same cost** — real accuracy.
But it is still **29% below** exact at the fine tick, which says the step size was
never only an integration step: it also quantises when a center mines, crosses a
band edge and re-decides. **Refining a step that carries more than one job
improves all of them; fixing the integrator pays for one.**

Final: **+6.57% ± 1.14 colony-years, 8/8 seeds**, throughput unchanged.
`growth_rate = 0.873` is **carried, not re-ratified** — its operating point and
its plateau map are both consumed.

---

## A.14 R-PROD5 — fleet-years in mass is provably indifferent to design law #3

**Harness:** `examples/volume_ladder`. **Bed:** none needed — this is the shipped
hull ladder's own geometry, not a simulation result, which is why it is a
two-second run and not a sweep.

**The question.** Design law #3 says consolidation wins under geometry alone:
cost is surface area, value is volume. Production's objective was fleet-years in
**mass**, and since R-O57 dry mass *is* mineral cost — so the objective was
"minerals committed to hulls, integrated". Does that express the law?

**No, and the failure is exact rather than approximate.** At equal mineral spend,
one General hull and the fleet of Mediums its minerals buy have **identical
mass** — `n · cs` *is* `cb`, by construction. The metric scores them 1.000, to
the bit, for every pair on the ladder.

| hull | dry kt | radius | vol `r³` | hold | shell | vol per kt |
|---|---|---|---|---|---|---|
| LSV | 0.02010 | 0.4742 | 0.10664 | 0.08944 | 0.01720 | **5.30** |
| MSV | 0.10921 | 1.0317 | 1.09800 | 1.00000 | 0.09800 | **10.05** |
| GSV | 1.31544 | 3.1953 | 32.62278 | 31.62278 | 1.00000 | **24.80** |
| LCV | 0.02000 | 0.3447 | 0.04097 | 0.02597 | 0.01500 | 2.05 |
| GCV | 1.09954 | 2.3511 | 12.99582 | 12.03582 | 0.96000 | 11.82 |
| LOU | 0.02000 | 0.2688 | 0.01942 | **0.00662** | 0.01280 | 0.97 |
| ROU | 0.10000 | 0.5529 | 0.16906 | 0.09606 | 0.07300 | 1.69 |
| GOU | 1.02187 | 1.8191 | 6.01987 | 5.08987 | 0.93000 | 5.89 |

**Volume per kilotonne rises monotonically with size inside every family**, which
is the law restated. At equal spend:

| | `r³` ratio | hold ratio | **mass ratio** |
|---|---|---|---|
| GSV vs MSV (12.05 small per big) | **2.467** | 2.625 | **1.000** |
| GOU vs ROU (10.22 per big) | **3.485** | 5.185 | **1.000** |
| MSV vs LSV (5.43 per big) | **1.895** | 2.058 | **1.000** |

**The choice of `r³` over the hold, and why the stronger number lost.** The
interior scores consolidation higher on all three pairs. It is still wrong for a
*fleet* metric: **a Limited Offensive hull's interior is 0.00662 against a
reserved core of 0.194**, so its hold is entirely spoken for and its cargo is
zero. Scoring a warship by its hold scores it by the one thing a warship does not
have — and design law #8 wants LOUs to be *useful*, not to score zero. The shell
is armor, not waste, and `r³` counts it.

**The decomposition is exact**, verified to 1e-12 for every hull:

```text
r³ = shell_volume + hold_volume,   shell_volume = η · shell_mass
```

So volume-years is the old mass objective's basis plus the interior, and the
interior is the term that grows as `cost^(3/2)`.

### What the change costs on the current bed: very little, and that is stated rather than buried

`examples/tree_gradient`, **seed 1, 600 yr, one seed, no error bar** — a sanity
check that the leg still measures something sensible, *not* a result:

| knob | Production elasticity, **mass** | **volume** |
|---|---|---|
| `medium_fleet_size` | −6.639 | −6.007 |
| `general_vehicle_cost` | +7.078 | +6.620 |
| `limited_fleet_size` | −0.549 | −0.441 |

**No sign flips and nothing reorders.** The second-order economic effect — cheaper
hulls mean more colonies mean more hulls — dominates the first-order geometric one
on this bed, which is why the direct ladder comparison above shows a 2.5–3.5×
effect and the simulated elasticity shows 0.1–0.6.

**So the change is principled, not numerically dramatic here**, and the honest
statement of its value is: the metric can now *express* design law #3. A metric
that is silent on a law will stay silent right up until a card makes the law
matter, and card design is the thing it would mislead — which is the same
argument `AGENTS.md` §2 makes about a denominator the game can play.

**`data/tree_gradient.tsv` (T-50) is stale** in its Production column and its
composite geomean. Not re-denominated, because that would keep the numbers'
authority while destroying their meaning. **R-TREE10** carries the re-run.
