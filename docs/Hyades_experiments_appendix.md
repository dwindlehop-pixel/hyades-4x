# Hyades — Experiments Appendix

*The measurement record behind the specs. **Nothing here is normative.** A spec
says what the engine must do; this file says what was run, on which bed, and
what it showed — including the runs that were wrong and the design that was
retracted.*

---

## 0. Why this file exists, and how to use it

`CLAUDE.md` §6 splits a spec into **decisions the engine must honor** and
**decisions still open**, and sends everything else here. The reason is that the
two kinds of statement decay differently:

- **A decision is true until something contradicts it**, and is meant to be read
  on every visit.
- **A measurement is a record of a run on a bed that no longer exists**, and is
  meant to be read once — when somebody re-opens the question.

`biosphere_regen_rate = +141.2 ± 18.1 colonies per ln` is the largest lever this
project ever measured and it is **bit-identically inert today**. The number was
never wrong; the engine it described stopped existing. A spec that inlines it
invites a reader to act on it; an appendix that records it lets a reader
*check* whether it still applies.

**Three rules for entries here:**

1. **Every entry names its bed.** Seeds, seat count, horizon, harness. A result
   without a bed cannot be compared against anything and is not worth keeping.
2. **Refuted results stay, marked refuted, with the refutation.** Deleting a
   retracted claim takes its correction with it, and the next reader re-derives
   the same wrong idea. This project has done that at least twice.
3. **Entries are append-only in spirit.** Correct one by adding the correction
   beneath it, not by editing the original into agreement with the present.

**How to cite:** a spec links a decision to an appendix section
(`see appendix §A.4`); this file links back to the decision it supports. If an
entry supports nothing, it is a curiosity and should say so.

---

# §A. Expansion and Growth

*Moved out of `docs/Hyades_autopilot_colonization_growth.md` at Rev 4. That spec
had reached 430 lines and was majority history.*

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

## A.3 R-AC16 / `survey_reserve` — a threshold above the range of what it thresholds

**Harness:** `examples/survey_timing`, seed 1, 600 planets, 1,500 yr.

`survey_reserve` was ratified at **1024** and is compared against
`candidate_count` — known, unclaimed, non-Barren worlds — whose **median is 0
and whose maximum over the entire run is 164**. So the predicate
`candidate_count < survey_reserve` is a constant `true`, and **every value above
~200 is bit-identical.**

That is the plateau `CLAUDE.md` §2 records as a measurement artifact: 2048 reads
as noise (+3.5 ± 2.6), while 512 → −21.8, 256 → −96.8, 64 → −840 fall off a
cliff. The ratification was not wrong about the *direction*; the magnitude
simply never reached the simulation.

**Superseded by R-O86** (A.4): the real precondition for a scout is
`survey_frontier`, not this.

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
argument `CLAUDE.md` §2 makes about a denominator the game can play.

**`data/tree_gradient.tsv` (T-50) is stale** in its Production column and its
composite geomean. Not re-denominated, because that would keep the numbers'
authority while destroying their meaning. **R-TREE10** carries the re-run.

---

# §B. Politics, Trade and Intelligence

*Moved out of `docs/Hyades_politics_trade_and_intelligence.md` at Rev 2. That
spec had reached 1,125 lines, roughly half of it implementation history.*

## B.1 R-P2 — λ, and a trade mechanism that paid for itself before trade existed

The transit burn was proposed as a `$` sink that happened to give the
travel-time behavior the brief asked for. The ratification condition was
stronger: it had to also be *the* solution to freighter routing.

Before it, a laden freighter picked the highest-pressure owned center with **no
distance term at all** (`most_needed_center`), so it would cross the galaxy for a
marginally needier destination. `λ = 0` reduces exactly to `most_needed_center`,
which is also the oracle design law #5 keeps for single-supply matching — one
function checking two independent degeneracies.

**Harness:** `examples/lambda_routing.rs`, 3 seats / 3 seeds / 4,000 yr.

| λ | half-life | mean coverage |
|---|---|---|
| 0 (`most_needed_center`) | ∞ | 14.35% |
| 0.002 | 347 yr | 27.71% |
| 0.005 | 139 yr | 35.20% |
| **0.010** | **69 yr** | **39.04%** |
| 0.020 | 35 yr | 36.88% |
| 0.050 | 14 yr | 36.74% |

A genuine interior optimum and **2.7× the shipped baseline** — a larger effect
than the entire five-parameter doctrine search produced.

**The scale is physically sensible rather than merely fitted:** a laden hop of
10–30 ly at 1 g takes 20–45 years, so a 69-year half-life discriminates exactly at
the range real hauls happen. Below that the discount is too sharp and freighters
stop serving genuinely needy distant centers; above it, need swamps distance again
and the rule degenerates toward `most_needed_center`.

**Three seeds is thin for a ratified constant.** Direction and order of magnitude
are confirmed; the precise optimum wants the ten-seed bed.

> **Cross-tree conflict found later** (`examples/tree_gradient`):
> `trade_decay_lambda` is **+0.002 on Expansion and −0.348 on Growth.** λ is the
> largest ratification in this project's history and it was measured on coverage
> alone. A single-metric probe cannot see this.

## B.2 The faucet and sink — four models, and why three were rejected

| | Faucet | Sink | Verdict |
|---|---|---|---|
| **M1 Closed** | fixed endowment at genesis | none | Elegant and inflation-proof, but a player who never trades is illiquid forever and early trade becomes compulsory rather than chosen. **Rejected — it removes the decision.** |
| **M2 Volume-minted** | minted on trade completion | none | Rewards churn, inflates without bound, and wash-trading with a confederate becomes dominant. **Rejected outright — it *pays* for collusion, inverting §0.** |
| **M3 Pop-backed** | accrues per pop per cycle | card costs | Ties liquidity to empire size, double-counting the snowball: the biggest empire also gets the deepest purse. **Rejected as sole model, kept as a component.** |
| **M4 Transit-burn** ✅ | production × Politics depth | **the travel-time discount itself** | Shipped. |

**M4's merit is that the sink and the travel-time discount are the same
mechanism** — one tunable doing four jobs: the discount the brief asked for, a
real sink scaling with volume *and* distance, geography entering the economy (a
near partner is strictly better than a far one at the same price), and denial
being expensive (a cornering bid pays full escrow and burns the transit share).

**The faucet is production, not population, and the author's reasoning overrode
the original recommendation:** production is the sum of *both* halves of an
economy — population growth and infrastructure deepening both feed it, where
population alone counts only one. An empire that invested in infra rather than
bodies is not poorer, and a pop-only faucet would say it was.

## B.3 T-77 — settlement landed, and the screen that preceded it was wrong twice

**Bed:** 3 seats, 4,000 yr.

Contracts clear, price, escrow, deliver into the buyer's pile at the shared rock,
default when the seller's bank is short the color it owes, and conserve mass.
**64,642 contracts settled and 3,484 defaulted** (seed 1); **64,553 and 3,905**
(seed 7). Only *geography* rejected anything — 47 and 31 fills out of ~68,000
found no shared rock.

**Yellow dominates the flow, which is the design goal arriving:**

| color | seed 1 delivered | seed 7 delivered | share |
|---|---|---|---|
| **Yellow** | **15,416.6 kt** | **14,380.5 kt** | **52%** |
| Cyan | 9,188.0 kt | 9,204.6 kt | 31% |
| Magenta | 4,855.0 kt | 5,012.6 kt | 17% |

That is the `3:2:1` Y:C:M works mix reproduced as *trade flow*, on both seeds,
without anything in the market being told about it. **Demand is Doctrine, and
Doctrine is the works bill** — measured, not asserted.

### The 400-planet screen was wrong twice

A 400-planet / 1,200-year probe reported **776 contracts and 327 kt** — 0.002% of
extracted mass — and the conclusion written from it was *"the market clears and
moves nothing that matters"*. The standard bed reports **64,642 contracts and
29,460 kt**, ~0.2%: **83× the contracts and 90× the volume.** The screen was not
merely imprecise, it was *qualitatively* wrong, because trade volume is
superlinear in galaxy size — more empires' worth of outposts overlap, so more
pairs can reach each other at all.

**A conclusion of the form "X does not matter" cannot be drawn from a screen at
all.** It is an absolute statement, and a screen only ever supports a relative
one.

### What it costs — and the mechanism is unproven

**Trade is measurably negative on Growth's objective**, on both seeds
(`examples/work_years`, against T-87's bed):

| | work-years | colony-years |
|---|---|---|
| seed 1, no Exchange | 1,495,212.5 | 10,888,100 |
| seed 1, with trade | 1,425,905.0 (**−4.64%**) | 10,889,400 (+0.01%) |
| seed 7, no Exchange | 1,540,712.5 | 10,983,925 |
| seed 7, with trade | 1,477,482.5 (**−4.11%**) | 10,982,875 (−0.01%) |

Colony-years is flat to a hundredth on both seeds — which is why it is not the
guard (B.5) — while work-years falls ~4.4% consistently.

**The leading hypothesis, recorded as unproven:** the two legs are not
symmetric. The seller's ore leaves its bank *immediately* at settlement, where it
was spendable; the buyer's ore lands in an **outpost pile** and stays there until
the buyer's own freighter happens to call. If collection lags delivery, trade is
a machine for moving minerals out of banks and into piles — strictly worse than
not trading, regardless of which color moves where.

That is checkable and must be checked before anything is tuned: compare banked
against piled holdings over time, and measure the dwell between a contract
settling and its ore reaching a bank. **A plausible mechanism attached to a real
number is the shape of every measurement artifact in this project.** Carried as
**R-P18**.

## B.4 The stage plan predicted the wrong risky stage

| # | Stage | Predicted | Actual |
|---|---|---|---|
| 1 | `$` ledger + faucet | neutral | neutral, bit-identical |
| 2 | `Commodity` gains color; `Offer` gains an owner | neutral | neutral, bit-identical |
| 3 | Cross-empire book; centers post `wtp` | neutral | neutral, bit-identical |
| 4 | Clearing at the barrier → contracts + escrow | **the risky one** | **inert** |
| 5 | The freight leg; escrow settles on arrival | — | **the risky one: −4.4% work-years** |

Stages 1–3 were deliberately inert, for the reason `Hyades_industry.md` §6.7's
stages 3–5 were: **a system that lands neutral can be verified against a
bit-identical bed before anything switches on.**

**Stage 4 turned out inert too, and the §10.6 amendment is why.** The table was
written when a cleared match was a cross-empire *delivery*, so clearing and
moving goods were one step. Once settlement moved to a shared rock, stage 4
strikes contracts and locks `$` while every kilotonne stays where it was.

**So the stage expected to be risky was not, and the risk moved with the goods.**
A stage plan is a claim about code, and an amendment to the design invalidates
the plan's predictions along with everything else it touches.

## B.5 Colony-years is inverted as a guard for anything that changes how minerals are spent

Seed 1 and 7, across four industry landings:

| | infra builds | colony-years, seed 1 | colony-years, seed 7 |
|---|---|---|---|
| pre-works | 1,032 | 10,558,680 | 10,474,865 |
| T-73 | 57 | 10,606,309 | 10,583,150 |
| T-81 | 31 | **10,633,441** | **10,599,130** |
| R-IND17 | 66 | 10,582,211 | 10,546,759 |

**Monotone inverse on both seeds:** colony-years *rises* as development collapses
and *falls* as it recovers, because minerals denied to infrastructure buy hulls
and a `k_high`-bound bed takes worlds earlier. **Four industry changes were
guarded on it and it rewarded the breakage every time.**

The interim guard for Exchange work is the build-mix census
(`examples/bank_mix`), which is what actually caught every defect on the works
branch. Colony-years stays as a *side-effect* read, never as the verdict.

## B.6 R-P17 — the venue question was wrong, not merely unanswered

**The question as framed:** which outpost do buyer and seller settle at, when
they share more than one? The answer written first minimized the two parties'
*summed* transit.

**The author's ruling made the question disappear.** A contract has **two drops,
not one venue**: the seller leaves Yellow at a shared outpost near *itself*, the
buyer leaves Magenta or Cyan at a shared outpost near *itself*. Summed transit is
the right objective only if there is a single venue for both legs, and there is
not — each shipper pays for its own leg, and one compromise venue would make each
side pay for the other's geography, which contradicts the default transaction
being *balanced in value*.

## B.7 R-IND10 — a register entry that was already answered

"Who bears the loss when a matched contract's carrier is destroyed" was carried
open while §3.3 already said it: on non-delivery **escrow returns to the buyer
minus the burn**, so the buyer loses the burn, the seller loses the cargo, and the
loss is shared. That is what makes escorting worth paying for.

**Kept as an entry because the failure is instructive**: an open register is only
as good as the sweep that reconciles it against the spec body, and nothing was
performing that sweep.

## B.8 R-P8 — a question dissolved by a better representation

"Is `Diplomacy::excluded` an exception to design law #13 (no categorical
classification co-extensive with a color domain)?" The earlier draft carried a
`Vec<PlayerId>` of counterparties to refuse, which was a per-player categorical.

**There is no list now.** Refusal is `conduct[Foe].clears_directly == false` — a
policy about a *kind* of relationship rather than about named players. The law
was never in danger; the representation was.

---

# §C. Cross-cutting — measurement artifacts, collected

*Seven shapes, all of them live in this project at some point. `CLAUDE.md` §2
carries the working rules; this is the case list.*

| # | What was measured | What it actually was | Entry |
|---|---|---|---|
| 1 | `medium_fleet_size = 8` optimal, 12 a "cliff" | the capacity normalizer going to zero | — |
| 2 | `coverage_trace`: this knob "DID move it" | two of three sample points degenerate | — |
| 3 | `coverage_time`: cheaper colonizers at 6.0 | a General hull holding ~700× a Medium's | — |
| 4 | coverage "wants" a cheaper Medium hull | `cap_Medium` pinned by a live normalizer | — |
| 5 | `survey_reserve` is "significant" at −23.8 ± 10.1 | a ±10% probe on a plateau, clearing 2 SE by luck | A.3 |
| 6 | time-to-10% and coverage disagree on a knob's sign | two gradients taken at different operating points | A.2 |
| 7 | a dwell metric *rose* under the policy that founds colonies 399 yr earlier | the mix moved: hull-first share 55.0% → 97.4%, both components fell | — |

Shapes 1–4 share a cause: **the quantity that broke was in a denominator the
shipped autopilot never exercised**, so none of them was visible in the
objective. Shape 7 is the only one where **nothing was broken** — the
measurement was correct, the population it averaged over was not the same
population, and the sign it reported was the opposite of the mechanism.

---

# §D. Cards — where a card's effect goes

## D.1 T-122 — isolating the constraints on the first Growth and Warfare cards

**Supports:** `Hyades_warfare_tree.md` §8.15, `Hyades_industry.md` §1.8 (T-107),
`Hyades_trees_and_card_value.md` §2.4 (earliest legal play), and the T-122
entry in `hyades_todo.md`.

**Bed.** `examples/card_probe`: 3 seats, **asymmetric** (R-TREE8/R-TREE12) —
seat 0 plays, every other seat passes, and the counterfactual is the same seed
with seat 0 passing too. Eight **independent** seeds (1, 7, 42, 31337, 2, 3, 5,
11), so each seed is one paired difference and the standard error counts
galaxies, not seats. Engagements on. Every figure is an **estimate** (mean ±
standard error over the eight), not a bound.

| symbol | meaning | unit |
|---|---|---|
| `ΔlnG` | `ln(G_card / G_pass)`, `G = ∫ infra dt` for seat 0 — Growth's work-years interim (R-TREE3) | dimensionless |
| `ΔW` | `W_card − W_pass`, `W = ∫ [C_0 − mean_{j≠0} C_j] dt` with `w` uniform (R-WAR3 open) | colony-years |
| `u` | stock-weighted staffing factor `I_worked / I` (T-107) | dimensionless |

### The first bed played cards inside the card-free opening, and that was the largest single effect measured

`card_probe`'s first version, and `examples/card_table` behind T-120 and T-121,
played cards at `t ≈ 0`. **The opening is card-free by protocol**
(`Hyades_netcode.md` §1): the first round barrier is at `years_to_first_round`,
200 yr. So those beds charged each card's 0.5 kt out of the 3 kt bootstrap bank
— a state no game reaches — and the price dominated both cards:

| arm, played at `t ≈ 0` (**superseded**) | pays 0.5 kt? | Δcolonies, seed 31337 | Δcolonies, mean of 8 |
|---|---|---|---|
| Warfare card as shipped | yes | −534 | −119 ± 62 (8/8 negative) |
| pure-price control (`TIER0[12]`, inert, same price) | yes | −554 | −94 ± 67 |
| Warfare scout write alone, no card | **no** | **+23** | −23 ± 36 |
| Growth card as shipped | yes | −274 | +33 ± 70 |

Re-run at the round-0 barrier (**the bed from here on**), the pure-price control
reads **Δcolonies −0.1 ± 0.1** and ΔW **+3 ± 4**: a tier-0 price is invisible
at legal play.

### ~~The Growth card, played legally, already clears 3 SE~~ — retracted at T-124 (§D.3)

**Superseded.** Every Growth figure in this entry was measured on a rung bill
that destroyed 0.0292 kt per purchase on stocks founded above their rung, and
the card's work-years effect was carried by that defect: with the bill
conserving mass the same arm reads **−0.014 ± 0.023**. The table is kept as the
record of that engine.

800 yr, card at 200 yr, 600 yr of play after it:

| arm | ΔlnG | t | Δcolonies | Δln pop |
|---|---|---|---|---|
| **Growth card as shipped** | **+0.133 ± 0.032** | **4.18** | +57 ± 12 | +0.755 ± 0.067 |
| … with population staffing on (T-107) | +0.126 ± 0.028 | 4.48 | +123 ± 25 | +0.657 ± 0.020 |

Positive on **8/8 seeds**. The fitted growth rate of seat 0's infrastructure
rises **+2.8% ± 0.7%** (t 3.85), so on the "double the rate" reading of the
target the card is far short; on the 3-SE reading it passes as shipped.
Staffing adds colonies (Δcolonies doubles) and no work-years.

### Refuted on the way, kept because each was plausible

All measured on the `t ≈ 0` bed unless marked, which is why the first two are
refuted *as inferences* rather than as measurements:

- **"Population is not a factor of production, so the Growth card cannot reach
  work-years."** The lever moved (Δln pop **+0.632 ± 0.043**, t 14.7) and
  work-years did not (**−0.005 ± 0.099**) — true on that bed. A low-noise arm
  with the color conjunction ablated put the unstaffed card at **+0.003 ±
  0.040**, which read as proof that staffing was necessary. **It was
  confounded:** that arm also paid the price at `t ≈ 0`, which the write-alone
  arm (no card, no price, staffed) measured at **−0.24** in `ΔlnG`
  (+0.271 ± 0.133 against +0.034 ± 0.135 with the price). At legal play the
  unstaffed card is **+0.133 ± 0.032**.
- **"The idle stock sits on worlds at their population ceiling, where a rate
  card cannot reach."** The split at the horizon puts **0.0000** of standing
  stock idle on worlds at `P ≥ 0.95 K`, and **70%** idle on worlds still
  growing (`Simulation::staffing_split`).
- **"The per-color bill (T-73) throttles the Growth card."** Ablated
  (`SimConfig::ablate_color_conjunction`), the staffed write-alone arm moves
  **+0.271 ± 0.133 → +0.267 ± 0.032**: the same mean, a quarter of the standard
  error. The conjunction is a **variance amplifier** — it reorders a compounding
  run — and not a throttle on this card.
- **"The picket guesses the wrong world" (round-0 bed).** With pickets fielded
  (278 parked, 497 intercepts per run), zero fights and zero diversions. Handed
  the true destination (`SimConfig::ablate_oracle_intercept`), feasible
  intercepts fall to **18 per run** and still produce **0.1 diversions and 0
  fights**; ΔW **−955 ± 9,607**. The guess is not what binds.

### The Warfare card has no path to `W`, and each route was measured closed

Round-0 bed:

| arm | ΔW (colony-years) | t | engagements | kills / losses |
|---|---|---|---|---|
| card as shipped | −103 ± 314 | −0.33 | 0 | 0 / 0 |
| colonizer write alone (`t ≈ 0` bed) | **exactly 0.0 on 8/8 seeds** | — | 0 | 0 / 0 |
| + hostility (`engage_neutrals`) | −9,928 ± 4,961 | −2.00 | +1,955 ± 57 | **+0.6 / +2,960** |
| + hold ground (pickets, intercepts, claim target) | −520 ± 9,779 | −0.05 | 0 | 0 / 0 |
| + hold ground, `picket_reserve` 8 → 32 | **bit-identical** to the row above | — | 0 | 0 / 0 |
| + hold ground, oracle intercept | −955 ± 9,607 | −0.10 | 0 | 0 / 0 |

Work-years under hostility: **−1.79 ± 0.07 (t −25)** — seat 0 spends its mining
fleet in fights it cannot win. The mechanism for each closed route is in
`Hyades_warfare_tree.md` §8.15; the numbers are here.

### On the twelve-seat table

`examples/card_table`, cards at the round-0 barrier, 800 yr, 3 seeds (1, 7,
42). Seat-seeds are **not** independent — six share a galaxy — so every
standard error below is optimistic by an unmeasured factor:

| quantity | value | n |
|---|---|---|
| Growth `ΔlnG`, work-years | +0.199 ± 0.126 | 18 seat-seeds |
| Growth card value, `1 − t½ ratio` | +0.016 ± 0.066 | 18 |
| Warfare `ΔW_i` at 800 yr, colonies | +2.667 ± 1.489 | 18 |
| Warfare card value, `1 − t½ ratio` | −0.004 ± 0.007 | 11 of 18 defined |

The Growth point estimate is larger than the asymmetric bed's (+0.199 against
+0.133) and **three galaxies cannot resolve it**: t 1.6 on an optimistic SE.
Treating seat-seeds as independent, 3 SE at this mean needs ~64 seat-seeds, or
~11 galaxies at ~5 min each — over the ~10-minute ceiling `CLAUDE.md` §2 sets
for an ephemeral container, so it is a by-hand run. The asymmetric bed is the
one that answers the per-card question; this one answers how the cards read
beside each other. *(Run at T-123 over 11 galaxies, with the standard error
taken over galaxies: §D.2.)*

## D.2 T-123 — the port strike: armed hulls meet colony ships where they launch

**Supports:** `Hyades_warfare_tree.md` §8.16 (R-WAR16 resolved, R-WAR17/18
opened) and the T-123 entry in `hyades_todo.md`. Same bed and symbols as §D.1,
on the T-124 engine unless marked.

### Pricing the meeting site before building it

`examples/launch_census`: 3 seats, 800 yr, the eight seeds, no cards. Colony-ship
launches grouped by exact origin position (a system is a fixed point, so one
center is one position), counted after the round-0 barrier. Mean over 24
seat-seeds (estimates; the top-`k` shares are **upper bounds** on what `k`
blockaders could meet, because the ranking is taken in hindsight):

| quantity | mean | range over seat-seeds |
|---|---|---|
| launches after 200 yr, per seat | ~1,780 | 1,093–2,654 |
| distinct origins, per seat | 213.6 | 58–327 |
| homeworld's share | 6.4% | 0.0–16.4% |
| busiest 3 origins' share | 13.7% | 7.8–29.8% |
| busiest 8 origins' share | 25.5% | 16.9–52.0% |

So a destination is one of thousands of worlds and an origin is one of about two
hundred, with a quarter of the traffic through eight of them. That is what made
the port the site and not a place on the route.

### The arms

Asymmetric bed, 8 seeds, 800 yr. "Written from `t = 0`" arms seed seat 0's
Doctrine at bootstrap (the card still plays at 200 yr and charges its price); the
card arms write it at the round-0 barrier, which is the game.

| arm | engine | ΔW (colony-years) | t | seeds > 0 | rival colonies at 800 yr | kills |
|---|---|---|---|---|---|---|
| blockade, reserve 8, written from `t = 0` | T-123 | +51,169 ± 12,603 | 4.06 | 7/8 | −121.4 ± 32.1 | 321 ± 74 |
| … without the claim-target supply | T-123 | +21,545 ± 8,114 | 2.66 | 8/8 | −84.9 ± 30.0 | 253 ± 82 |
| … reserve 32 | T-123 | +73,002 ± 13,102 | 5.57 | 8/8 | −228.0 ± 46.3 | 637 ± 118 |
| **card as shipped, reserve 8** | T-123 | +18,755 ± 7,573 | 2.48 | 7/8 | −89.3 ± 30.6 | 277 ± 83 |
| card as shipped, reserve 16 | T-123 | +19,147 ± 7,923 | 2.42 | 7/8 | −94.8 ± 32.5 | 294 ± 85 |
| card as shipped, reserve 32 | T-123 | +18,993 ± 7,690 | 2.47 | 7/8 | −97.0 ± 32.8 | 323 ± 97 |
| **card as shipped, reserve 8** | **T-124** | **+24,024 ± 8,509** | **2.82** | 7/8 | **−102.6 ± 25.8** | 306 ± 84 |

Seat 0 lost **no hull** in any arm: R-WAR5's convention gives the fight to the
hull on station. The card's own work-years move −0.040 ± 0.034 (t −1.18).

**What the table decides.** Played at the barrier, the reserve is not what binds
— 8, 16 and 32 are within a tenth of a standard error. Written from `t = 0`,
more hulls do help, and the whole card is worth two to three times as much. The
inference, stated as one: hulls placed early stand at the ports that launch
first, and a cumulative launch count keeps later hulls on ports whose traffic has
moved on. That is R-WAR18; it is not established which of placement lag and
recency binds.

**The refactor is bit-identical.** `fight_at` was extracted from
`resolve_picket_fight`; the as-shipped arm before the blockade was wired into the
card reproduced T-122's per-seed ΔW on all eight seeds.

### On the twelve-seat table

`examples/card_table`, 12 seats, three arms, cards at the round-0 barrier,
800 yr, **11 independent galaxies** (1, 7, 42, 31337, 2, 3, 5, 11, 13, 17, 19),
T-124 engine. Each galaxy contributes **one** value — the mean over its six card
seats — so the standard error is over galaxies, the independent unit. §D.1's
table counted seat-seeds and was optimistic by an unmeasured factor; this one is
not. Per-galaxy `ROW` lines are printed as each finishes, so a killed run resumes
with `--seeds`.

| quantity | mean ± SE over 11 galaxies | t | galaxies > 0 |
|---|---|---|---|
| **Warfare `ΔW_i` at 800 yr**, colonies | **+28.26 ± 5.37** | **5.26** | **11/11** |
| **Warfare `∫ΔW_i dt`**, colony-years | **+8,817 ± 1,713** | **5.15** | **11/11** |
| Growth `ΔlnG`, work-years | +0.014 ± 0.042 | 0.33 | 5/11 |

| seed | Growth `ΔlnG` | Warfare `ΔW_i` at 800 yr | Warfare `∫ΔW_i dt` |
|---|---|---|---|
| 1 | −0.0749 | +67.00 | +23,102 |
| 7 | +0.0438 | +7.82 | +4,734 |
| 42 | −0.0611 | +24.64 | +6,655 |
| 31337 | −0.0020 | +54.73 | +14,053 |
| 2 | +0.2186 | +22.18 | +9,977 |
| 3 | +0.0503 | +20.82 | +4,814 |
| 5 | −0.2730 | +21.00 | +9,057 |
| 11 | +0.2227 | +9.91 | +1,949 |
| 13 | +0.0667 | +26.27 | +7,074 |
| 17 | −0.0368 | +23.09 | +7,868 |
| 19 | −0.0028 | +33.45 | +7,701 |

**The Warfare card clears 3 SE on the twelve-seat table** in both readings, on
every galaxy. **The Growth card does not move work-years** there either, which
agrees with the asymmetric bed on the conserving engine (§D.3). The fitted
doubling-time value (`1 − t½ ratio`) is printed by the harness and not used:
`W_i` has no logarithm on 11 of every 24 Warfare seat-seeds (R-TREE9).

## D.3 T-124 — a rung bill that destroyed mass, and the Growth card it was carrying

**Supports:** `Hyades_industry.md` §1.8 and R-IND23, the `infra_step_price` doc
comment, and the T-124 entry in `hyades_todo.md`.

### Found by the blockade's conservation test, on a run with no card

`mass_is_conserved_through_the_blockade` failed at 300 yr. The same bed with no
card played drifted too, and the drift was **exactly zero through 240 yr**, then
fell by **0.02921 kt** between 240 and 260 yr and by twice that between 280 and
300 — equal quanta, so one transfer and not rounding. Stepping the run and
diffing the ledger per event named it: an `UpgradeInfrastructure` on a stock at
Band **1.1113** billed **0.9 kt** (the I → II width) and raised infrastructure by
**0.87079 kt**, because the purchase sets the stock to exactly rung II.

`infra_step_price` rounded the stock to a rung and billed that rung's width. A
stock above its rung paid for mass it never received; one below would have
received mass it never paid for. `founding_infra` is a hull's cost, so colonies
start between rungs routinely.

**Fix:** bill `infra_rung_price(at + 1) − stock`. Identical for a stock on a
rung. `an_off_rung_upgrade_erects_what_it_bills` asserts both directions and
fails on the old bill with −0.02921 kt.

### The default bed barely moves

8 seeds, 3 seats, 800 yr, no cards, T-122 engine against T-124:

| quantity | Δln, T-124 − T-122 | t | seeds > 0 |
|---|---|---|---|
| colony-years, all seats | +0.0021 ± 0.0028 | 0.76 | 3/8 |
| work-years, all seats | +0.0064 ± 0.0122 | 0.52 | 4/8 |

### The Growth card moves a great deal, and a one-sided ablation says which half

The card's work-years effect went from **+0.1328 ± 0.0318** to **−0.0143 ±
0.0226**. Two variants of the bill, each keeping one half of the old error:

| variant | Growth card ΔlnG | reproduces |
|---|---|---|
| below-rung stocks still topped up for free | −0.0143 ± 0.0226 | T-124, **bit-identically** |
| above-rung stocks still overcharged | +0.1328 ± 0.0318 | T-122, **bit-identically** |

So no below-rung stock is ever upgraded in play, and the whole T-122 Growth
result rode on the overcharge. The line is named; **why** a 3% overcharge on
off-rung colonies turns into a +0.13 advantage for a population card is not
established, and no mechanism is claimed for it.

### What the Growth card does on the conserving engine

| arm | ΔlnG | t | Δln pop | Δcolonies | `u` pass → card |
|---|---|---|---|---|---|
| card as shipped | −0.014 ± 0.023 | −0.63 | +0.745 ± 0.071 | +46.4 ± 5.3 | — |
| card, staffing on (T-107) | +0.056 ± 0.062 | 0.90 | +0.746 ± 0.069 | +57.5 ± 7.6 | 0.634 → 0.686 |
| card, color conjunction ablated | +0.039 ± 0.013 | 2.94 | +0.810 ± 0.035 | +84.9 ± 23.3 | — |
| card, staffing on, conjunction ablated | **+0.121 ± 0.015** | **8.17** | +0.666 ± 0.021 | +171.4 ± 24.1 | 0.108 → 0.122 |
| write alone (no price), staffing on | +0.151 ± 0.067 | 2.25 | +0.847 ± 0.054 | +99.6 ± 23.2 | 0.634 → 0.686 |
| write alone, staffing on, conjunction ablated | +0.188 ± 0.026 | 7.13 | +0.771 ± 0.029 | +209.8 ± 29.4 | 0.108 → 0.124 |

The card moves population by three quarters of a log unit on every arm. Work-years
follow only when population staffs industry **and** a rung is not gated on the
bank holding every color at once. That is R-IND23, and both halves of it are rules
the author has to decide, not magnitudes.

**T-122's refutation of "the color conjunction throttles the Growth card"** was
measured on the overcharging bill and does not transfer: there the ablation left
the mean alone; here, with staffing on, it takes the card from +0.056 ± 0.062 to
+0.121 ± 0.015.

## D.4 T-125 — weapons as a Design, the author's 1.5–2.0x target, and where each card stands

**Supports:** `Hyades_warfare_tree.md` §8.17 (weapons are a Design; stacking;
the blockade's supply; R-WAR20), `Hyades_trees_and_card_value.md` §4.2 (the
target), `Hyades_industry.md` R-IND23 (staffing on), and the T-125 entry in
`hyades_todo.md`. Asymmetric bed as §D.1 unless marked: 3 seats, 8 independent
seeds, 800 yr, card at the round-0 barrier, mean ± standard error over seeds.
Every figure is an estimate.

| symbol | meaning | unit |
|---|---|---|
| `ΔlnG` | `ln(G_card / G_pass)`, `G = ∫ infra dt` (Growth's metric) | — |
| `ΔlnS` | `ln(S_card / S_pass)`, `S = ∫ C_0 / mean_{j≠0} C_j dt` (Warfare's metric, the author's reading) | — |
| P92 | the 92nd percentile of the per-seat ratio | — |

### The Growth card, staffing on

| multiplier | `ΔlnG` | per-seed P92 (ratio) | Δln pop | staffing `u`, card seat |
|---|---|---|---|---|
| ×1.15 (T-124 card) | +0.056 ± 0.062 | — | +0.746 | 0.686 |
| ×2 | +0.203 ± 0.076 | 0.509 (1.66x) | +3.54 | 0.892 |
| ×4 | +0.343 ± 0.111 | 0.771 (2.16x) | +4.37 | 0.978 |
| ×8 | +0.318 ± 0.091 | — | +4.51 | 0.984 |

The effect saturates past ×4 as staffing does. On the twelve-seat bed ×2 read
**P92 2.19** over 4 galaxies (median 1.18), so the card ships at **×1.6**.

**One pathology, located and not fully explained.** On seed 31337 (12 seats,
×2) seat 1's work-years fell to **0.039** of its counterfactual: its
infrastructure stops at 4.03 from year ~400 while the pass arm reaches 209. The
decision log shows why it builds nothing — **7,800 of its decisions saw an empty
candidate list with nothing left to survey**, and deepening failed the
per-color bill — so it idles. Which seats took its worlds is not established.
It is in the lower tail and does not move P92.

### The Warfare card — every lever, in the order tried

`ΔlnS`, card as shipped at the barrier:

| engine | reserve 8 | 32 | 128 | 512 |
|---|---|---|---|---|
| beams from Design, blockade as T-123 | +0.038 ± 0.011 | +0.043 ± 0.013 | +0.044 ± 0.013 | — |
| + recency and reassessment | +0.036 ± 0.011 | +0.040 ± 0.012 | +0.040 ± 0.012 | — |
| + recall of launches already seen | +0.041 ± 0.012 | +0.044 ± 0.013 | — | — |
| + **blockade built first** (`picket_first`) | +0.061 ± 0.013 | +0.092 ± 0.009 | **+0.116 ± 0.015** | +0.118 ± 0.015 |
| + stacks, allocated by launches per hull | — | — | +0.059 ± 0.007 | — |
| + stacks, recently active ports covered first | — | — | +0.066 ± 0.009 | — |
| + stacks, every seen port covered first | — | — | +0.068 ± 0.010 | — |
| + stacks, and building ahead only to cover a port (**shipped**) | — | — | **+0.094 ± 0.015** | — |

Kills per run rise with the reserve (300 → 408 before `picket_first`, 246 → 678
after); seat 0 lost no hull in any arm.

**The coverage oracle** (`ablate_strike_fraction`, strikes a fixed share of
rival launches from the barrier with no hull): **+0.265 ± 0.021 at 25%, +0.567
± 0.044 at 50%, +2.61 ± 0.20 at 100%**. So the target band needs ~30–45% of
rival launches struck.

**The census** (`examples/blockade_census`, pooled over the 8 seeds; top-k is
the share of that century's rival launches from its `k` busiest ports, `k` the
mean hulls on station — an upper bound, hindsight and lag-free):

| century | rival launches | struck | coverage | on station | in flight | top-k |
|---|---|---|---|---|---|---|
| *reserve 8, fallback supply* | | | | | | |
| 200–300 | 1,563 | 0 | 0.0% | 0.0 | 0.1 | — |
| 300–400 | 9,569 | 185 | 1.9% | 1.2 | 10.8 | 10.1% |
| 400–500 | 8,528 | 1,324 | 15.5% | 19.9 | 12.2 | 46.1% |
| *reserve 128, blockade first* | | | | | | |
| 200–300 | 1,563 | 109 | 7.0% | 0.2 | 2.1 | — |
| 300–400 | 9,429 | 1,014 | 10.8% | 3.8 | 14.9 | 17.8% |
| 400–500 | 8,810 | 2,518 | 28.6% | 36.5 | 43.4 | 66.5% |
| 500–600 | 5,442 | 1,236 | 22.7% | 69.7 | 48.1 | 88.0% |

The rivals' expansion peaks in 300–400 yr and the blockade's hulls are mostly
still in flight then. The inference, stated as one: **the binding constraint is
transit latency** — a light-crossing to see a port and a flight to reach it,
against an expansion clock that starts 100 years after the card is legal.

### The twelve-seat table (the author's bed)

`examples/card_table`, 11 galaxies, 66 card seats per tree, Growth ×1.6,
`ARMED_FRONTIER_BLOCKADERS = 128`; P92 interval 90%, bootstrapped over galaxies:

| card | median | **P92 [90%]** | P98 | min | against 1.5–2.0x |
|---|---|---|---|---|---|
| Growth | 1.161 | **1.753 [1.583, 1.932]** | 5.654 | 0.182 | **inside** |
| Warfare | 1.070 | **1.204 [1.164, 1.226]** | 1.235 | 0.972 | **below** (R-WAR20) |

### Consequences of loadouts that are not about either card

- **Rock fights kill nothing.** Every miner is a Systems hull and unarmed;
  `engage_neutrals` makes contact but neither side can shoot. T-122's
  "hostility costs its player −1.79 work-years" was R-WAR5's convention arming
  the arriver with missiles it never built, and does not describe this engine.
- `slag_conserves_the_mass_of_what_it_destroyed` now draws its kills from a seat
  carrying the card's writes, and bounds slag below by the lightest hull rather
  than equating it to miners, because what dies is colony ships with settlers
  and cargo aboard.

## D.5 T-126 — release-binary throughput on the combat bed: compiler knobs, then the profile

**Supports:** the T-126 entry in `hyades_todo.md` and `CLAUDE.md` §7's
throughput table. `examples/combat_bench`: the twelve-seat card bed with both
cards played at the barrier and engagements on, run to 400 yr (200 years past
the barrier; 1,038 and 1,357 fights on seeds 1 and 7), not to completion. Every
variant produced **the same event count and fight count** as the base binary,
so each comparison is of one bit-identical run.

### Compiler knobs — none beats the shipped profile

The shipped `[profile.release]` is already `opt-level = 3`, `lto = true`,
`codegen-units = 1`, `panic = "abort"`. Seven variants, two seeds × two rounds
interleaved; `±` is the half-range of the four runs (an estimate of the
run-to-run spread, not a standard error):

| variant | yr/s | vs base |
|---|---|---|
| **base** | **14.61 ± 0.23** | — |
| `lto = "thin"` | 14.62 ± 0.27 | +0.1% |
| `lto = false`, 16 codegen units | 14.47 ± 0.06 | −1.0% |
| `opt-level = 2` | 14.47 ± 0.12 | −1.0% |
| `-C target-cpu=x86-64-v3` | 14.76 ± 0.10 | +1.0% |
| `-C target-cpu=native` | 14.76 ± 0.23 | +1.0% |
| PGO (`llvm-profdata`, trained on seed 3) | 14.60 ± 0.33 | −0.1% |

Every difference is inside the spread. Nothing is checked in: the `target-cpu`
rows would also make the binary unportable for a gain four runs cannot resolve.

### The profile — one logarithm per planet per survey decision

Callgrind on a 260-yr run: **`log` from libm was ~41% of all instructions, and
`fill_survey_candidates` most of the rest (~85% together).** The scan walks every
unvisited planet on each survey decision and read each world's biosphere Band as
`f.bio_max.in_bands()` — a fresh `ln` — while `Factors::bio_max_band` holds the
same value, cached and kept in step by `set_bio_max`. Reading the cache:

| binary | seed 1 yr/s | seed 7 yr/s | ns/event |
|---|---|---|---|
| base | 14.67, 14.93 | 14.76, 14.63 | 118,286–127,283 |
| **cached Band** | **30.87, 31.21** | **29.82, 30.10** | **58,013–60,481** |

**2.07x, bit-identical.** `CLAUDE.md` §4 already names this pattern ("convert at
the edges — never inside a loop over entities"), and `bio_max_band` was written
for exactly this reason (R-O70) — at a different call site.

**Then the walk itself.** A second profile put the scan's loop and its `Vec`
pushes at ~60% of what remained. `visited` only grows, so each seat now keeps its
unvisited worlds in planet order and prunes them with a stable `retain` on every
call — T-101's compaction, bit-identical for the same reason:

| binary | seed 1 yr/s | seed 7 yr/s | ns/event |
|---|---|---|---|
| cached Band | 31.46, 30.21 | 30.01, 29.34 | 58,180–61,799 |
| **+ compacted scan** | **37.42, 37.07** | **35.65, 35.45** | **48,989–50,356** |

**Cumulative: 14.61 → 36.40 yr/s (2.49x) on the combat bed**, same events and
fights on both seeds. What is left in the scan is materializing a `SurveyView`
per unvisited world for a policy that keeps one of them (`CLAUDE.md` §4, "do not
materialize a collection you only `min_by` over"); removing it changes the
`Autopilot::choose_survey_target` interface and is not done here.


## D.6 T-127 — the host libm made native and wasm32 runs diverge; the engine's own transcendentals

**Supports:** netcode §6 H4a, the T-127 entry in `hyades_todo.md`, and
`src/transcendental.rs`. Author's request: remove the expensive math-library
calls (`ln` and its siblings) from the simulation, and say why they are there.

### What each transcendental was for (inventory at T-127)

| call site | function | what it computes |
|---|---|---|
| `units::Qty::band` | `ln` | the Band reading of a mass — `n + ln(m / rung_n) / ln(step_n)`; a Band *is* a logarithm |
| `units::Qty::at_band` | `powf` | its inverse, `rung_n · step_n^(b − n)` |
| `Simulation::settler_target` | `ln` | the time a seed of `S` saves the child on its own logistic, `ln(S/(K − S) · (K − x₀)/x₀)` — the closed form's inverse — at up to 32 grid points per colonization decision |
| `logistic_step` | `exp` | the closed-form logistic across one tick, `e^(−rΔ)` (T-94) |
| `sys_contract_due`, two freight routing scores | `exp` | decay with elapsed or travel time, `e^(−λt)` |
| `veins`, `crowding_factor`, `mining_crew_for` | `powf` | vein count geometric in Band; the crowding exponent β |
| intercept cone | `cos` | cosine of the cone half-angle |
| `combat::StationKeeping` | `sin`, `cos` | a random orbital plane and the orbit offset (Rodrigues rotation) — reached by every simulation fight |
| `galaxy` generation | `ln`, `exp`, `sin`, `cos`, `powf` | exponential and Gamma radial draws, Gaussian hotspot density, ring positions, Weibull population-band quantiles |
| `Rng::gaussian` | `ln`, `cos` | Box–Muller |
| `math::exp_decay` fallback | `exp` | outside its fitted range (T-102) |
| `math` flight kinematics | `powi(2)` | a square — now `q * q`, bit-identical |

`sqrt` stays on `std`: IEEE 754 specifies it exactly, and wasm has it as an
instruction. So do `floor`, `round` and `abs`.

**Cost before T-127:** after T-126, libm was **0.74%** of instructions on the
combat bed (callgrind, 12 seats, seed 1, 300 yr) and **~3.9%** on the standard
three-seat bed (400 yr), nearly all of it `ln` from `settler_target`. T-126 had
already removed the ~41% that the survey scan's `ln` cost.

### Native against wasm32 — per function

A scratch crate evaluated each function on 2,000,000 inputs built from exact
arithmetic (so both targets see the same inputs), compiled natively (glibc) and
for `wasm32-unknown-unknown` (Rust's libm, run under node 22), and compared the
result bits:

| function | inputs | results that differ | max difference |
|---|---|---|---|
| `ln` | `2^−30 … 2^31` | **1.92%** | 1 ulp |
| `exp` | `[−30, 10]` | **9.76%** | 1 ulp |
| `powf` | `x ∈ [0.5, 30.5]`, `y ∈ [−1, 3]` | **9.71%** | 1 ulp |
| `sin` | `[0, 8π)` | **3.11%** | 1 ulp |
| `cos` | `[0, 8π)` | **3.11%** | 1 ulp |

### Native against wasm32 — whole runs

The same scratch crate ran `Simulation::with_baseline` for one seed on both
targets and compared two FNV-style digests: one over every planet's position,
habitability, biosphere and mineral mass (the galaxy), one over the report
(events, and per seat colonies, outposts and the population's bits). The combat
arms play the Warfare card on even seats and Growth on odd seats at the barrier,
with engagements on (`examples/combat_bench`'s bed).

| seats | seed | horizon | before: galaxy | before: report | after: galaxy | after: report |
|---|---|---|---|---|---|---|
| 2 | 1 | 90 | differs | same | same | same |
| 3 | 7 | 60 | differs | same | same | same |
| 6 | 13 | 40 | differs | same | same | same |
| 12 | 99 | 26 | differs | same | same | same |
| 18 | 4 | 20 | differs | same | same | same |
| 3 | 1 | 800 | differs | **differs** — 306,273 vs 305,951 events | same | same |
| 3 | 7 | 800 | differs | **differs** — 360,348 vs 359,938 events; 2,874 vs 2,881 colonies | same | same |
| 3 | 42 | 800 | differs | **differs** — 352,979 vs 353,068 events; 2,872 vs 2,879 colonies | same | same |
| 12 | 1 | 300 | differs | **differs** — 105,798 vs 105,813 events | same | same |
| 12 | 1 | 300, combat | — | — | same | same, 199 fights |
| 12 | 7 | 300, combat | — | — | same | same, 255 fights |

The short arms — the horizons `tests/determinism.rs` runs — agree on the report
while their galaxies already differ in the last bit, so **the determinism suite
could not have caught this** even had it run on both targets. It takes a few
hundred simulated years for the last-bit differences to reach a count.

### Accuracy against the host (`src/transcendental.rs` tests)

| function | range | bound against the host |
|---|---|---|
| `ln` (table, run time) | 60 binades | ≤ 1 ulp |
| `ln` | within 0.3 of 1 | ≤ 2 ulp (differs on ~24% of samples; the sum of `ln c` and `ln(1 + r)` loses up to one bit) |
| `ln_const` (fdlibm, compile time) | 60 binades | ≤ 1 ulp |
| `exp` | `[−40, 20]` | ≤ 1 ulp |
| `sin`, `cos` | `[−60, 60]` | ≤ 1 ulp |
| `pow` | `x ∈ [10^−3, 10^3]`, `y ∈ [−4, 4]` | ≤ `4 + 2·|y ln x|` ulp |

Each is a bound over the sampled inputs, not a proof. The combat goldens
(`tests/balance.rs`, release) pass unchanged.

### Throughput

Per call (min of 7 passes over 2,000,000 inputs, release, this container):

| | host | engine |
|---|---|---|
| `ln` | 6.8 ns | **10.3 ns** (table); fdlibm form 11.3–11.8 ns |
| `exp` | 7.2 ns | 14.8 ns |
| `pow` | 20.5 ns | 17.2 ns |
| `sin` + `cos` | 30.6 ns | 25.3 ns (one reduction for both) |

The first run-time `ln` was fdlibm's, and replacing its division with a
128-cell table bought **nothing** (11.8 → 12.1 ns). The saturating `as usize`
conversion used to index the table cost ~2.5 ns per call: `1.5·2^52` rounding
plus a 256-entry table, indexed by the low eight bits so the bounds check
disappears, gave 10.3 ns.

**Instruction count against wall time disagreed on the standard bed**, and the
split was only resolved by timing where the two runs are the same run. At
400 yr the old and new engines reach 431 colonies on seed 1 (population equal
to six decimals) and 928 / 927 on seed 7; by 800 yr the last-bit differences
have moved event counts by up to 0.3%, so an 800-yr comparison measures a
different workload as well as a different cost.

| standard bed, 3 seats | seed 1 | seed 7 |
|---|---|---|
| instructions, 400 yr (callgrind) | 10.251 G → 10.193 G (**−0.56%**) | — |
| `ns/event`, 400 yr, min of 7, old | 30,197 | 25,443 |
| engine transcendentals, no prune | 31,054 (**+2.8%**) | 26,750 (**+5.1%**) |
| **+ the `settler_target` prune (below)** | **30,642 (+1.5%)** | **26,177 (+2.9%)** |

Fewer instructions and more time is a latency cost: the engine's `ln` and `exp`
are longer dependency chains than glibc's FMA-specialized routines
(`__ieee754_log_fma`). **The ablation that located it:** the new engine with
only `settler_target`'s `ln` pointed back at the host recovered 29% of the gap
on seed 1 and 64% on seed 7 (800 yr, min of 5).

**So the lever was the call count, not the call.** `settler_target` takes `ln`
at every grid point to find an argmax. `ln` is concave, so its tangent at an
earlier point bounds it from above, and a point whose bound cannot beat the
best so far cannot win. Skipping those points takes **~48% fewer logarithms**
(seed 1, 800 yr: 5.90 M taken, 5.44 M skipped) and is **bit-identical** — all
eleven digests above reproduce, and
`the_pruned_endowment_scan_picks_what_the_full_scan_picks` holds the pruned scan
to the full one over 20,000 random inputs.

**Combat bed** (`examples/combat_bench`, 12 seats, 400 yr), final engine against
the old, three interleaved rounds, mean `ns/event`: seed 1 **63,185 → 62,174
(−1.6%)**, seed 7 **60,970 → 61,439 (+0.8%)**; 29.55 → 30.02 and 28.64 → 28.48
yr/s. The runs differ (1,038 → 1,028 and 1,357 → 1,386 fights), and the two
seeds move in opposite directions, so the combat bed shows no cost that three
rounds can resolve.

### What the change did to the runs

Every run's bits move (the galaxy is generated with the new functions), so this
is a disturbance of the kind T-102 measured, not a behavior change. Standard
bed, 800 yr, colonies, new minus old: **+4.5 ± 2.2 (mean ± SE, n = 8)**, 5 up,
2 down, 1 tied; population moves by −0.31% to +0.30%. That is 2.0 SE on eight
seeds and is not read as an effect: nothing in the change has a direction. The
T-125 card table (appendix §D.4) was measured on the old bits and is not
re-measured here.

### Test budget

`tests/determinism.rs` was **63.7–64.5 s on the old and new binaries alike**
(two interleaved runs each) against the 60-second rule — an inherited breach, on
this container, of a target T-126 measured at 48.2 s. One test,
`full_run_reports_are_bit_identical`, was **64.9 s** of it on its own: five seat
counts in sequence. Split into one test per seat count, the harness runs them in
parallel and the target is **53.8 s**, with every assertion and every galaxy
unchanged.

`tests/smoke.rs::snapshot_is_consistent_with_report` compared a world's biomass
against its ceiling read back from a Band with an absolute 1e-9 kt tolerance.
At 620,113.69 kt that is ~8 ulp, and a Band round trip (`ln` then `exp`) carries
~1 part in 10^15; the host libm's rounding had kept it inside. The tolerance is
now relative (1e-12).


## D.7 T-129 — Band readings off the run path: static kilotons, and a reading without a logarithm

**Supports:** `Hyades_mineral_cost_curve.md` §2.6's T-129 note and the T-129
entry in `hyades_todo.md`. Author's direction: *"Band readings are not
required. Translate statically into kt readings."* Asked which form each
continuous consumer should take, the author answered: `rank` — *"do the math
statically for each Band range in kt"*; `veins` and `i_star` — *"make no
change"*; the snapshot — *"a cheaper approximation of Band readings that does
not require a transcendental function"*.

### Where the conversions were (counted, seed 1, 3 seats, 400 yr)

A temporary `#[track_caller]` counter on every conversion, on T-127's engine
(43,012 events):

| calls | site | kind |
|---|---|---|
| 322,600 | `capacity_of`: `population_mass(K)` | round trip — `K` is a minimum of two masses |
| 235,350 | `Factors::infra_band` | reading |
| 228,890 | `staffing`: `at_band(infra_band)` | round trip cost → mass |
| 203,342 | `infra_rung_of`: `round(band)` | threshold |
| 54,443 | `veins` | reading (unchanged by instruction) |
| 16,713 | `mineral_bands` memo misses | reading (`rank`) |
| 12,082 | `founding_infra_band` | reading |
| 8,444 | production tick: `population_mass(K)` | round trip |
| 1,524 × 2 | seed floor; `population_mass(k_potential)` | constant; round trip |
| 6,725 × 3 | world construction | static |

After T-129 (43,149 events): `at_band` runs only at world construction;
`staffing`'s map runs 227,359 times and the rung test 208,998 times, neither
taking a reading; the remaining readings are `veins` (54,326), `mineral_bands`
(16,698), `founding_infra_band` (12,002) and `infra_band` (6,446), all through
the new reading.

### The static translations

- **`K` as a mass.** `population_mass` is monotone, so `KT(min(hab, band(bio_max)))
  = min(KT(hab), bio_max)`; `KT(hab)` is stored when the world is built. The
  one place the round trip was not the identity is the floor, and `k_mass`
  keeps it: a `bio_max` at or below the bottom rung admits no people.
- **The nearest rung.** A reading rounds up at a segment's geometric
  midpoint, so the rung is the count of `x² ≥ rung_k² · F_k` that hold.
  Against `round` of the exact reading over 200,000 amounts at a foreign
  anchor: no disagreement (the test allows two, at a midpoint).
- **Cost → mass at equal Band position.** `mass_n · (x / cost_n)^(3/2)` within
  segment `n ≥ I`; against the exact round trip over 200,000 amounts, relative
  difference `< 1e-12`. The `Empty` segment's exponent (`ln 1000 / ln 5`) goes
  through `transcendental::pow`, and **it did not run**: over 800 yr on seeds 1
  and 7, the segment counts were `[0, 895,188, 216,837, 49,813]` and
  `[0, 957,301, 237,688, 47,092]`.

### The reading

`n + log₂(m / rung_n) · (1 / log₂ F_n)`, the per-segment constants evaluated at
compile time, and `log₂` from the exponent bits plus `f · Q(f)` on the mantissa
normalized to `[√½, √2)`. `Q` is a Chebyshev interpolant of `log₂(1 + f) / f`,
fitted in plain Python (no numpy in the container). Maximum error over the
interval, in `log₂`, and the worst case in Bands (the cost ladder's `F = 5`):

| degree of `f·Q` | `log₂` | Band |
|---|---|---|
| 5 | 2.8e-5 | 1.2e-5 |
| 6 | 4.2e-6 | 1.8e-6 |
| **7 (shipped)** | **6.3e-7** | **2.7e-7** |
| 8 | 9.6e-8 | 4.2e-8 |

The `f · Q` form makes the reading exact at every rung. **The first version
was not exact at `Band IV`**: a mass there sat inside segment III, where the
ratio to its rung is `F₃ = 252.98` rather than a power of two, and it read
3.9999999977. `IV` is now a segment start, extrapolated with `F₃` above it.
Held against the exact reading over eighteen decades on both ladders: **within
3e-7 Band**.

### Cost and effect

- **Instructions**, seed 1, 400 yr (callgrind): 10.060 G over 43,012 events →
  9.845 G over 43,149, **233,891 → 228,172 per event (−2.4%)**.
- **`ns/event`**, standard bed, 400 yr, min of 7 interleaved: seed 1 **30,523 →
  29,087 (−4.7%)**, seed 7 **26,270 → 24,011 (−8.6%)** against T-127; against the
  engine before T-127, 30,312 and 25,456, so both seeds are now faster than
  with the host libm. Seed 1's run had diverged by 400 yr (447 colonies against
  431), so its figure compares slightly different workloads.
- **Combat bed**, 12 seats, 400 yr, three interleaved rounds: 61,935 → 63,167
  and 61,556 → 61,838 `ns/event`, with a spread of up to 5% within each build —
  no difference three rounds can resolve.
- **The runs move**, because readings enter `rank` and the deepening headroom:
  standard bed, 800 yr, colonies against T-127 **−2.75 ± 2.05 (mean ± SE,
  n = 8)**, 3 up, 4 down, 1 tied; population −0.02% to +0.80%. 1.3 SE; not
  read as an effect.
- **Native against wasm32:** all eleven arms of §D.6 reproduce bit-for-bit. The
  five short arms are bit-identical to T-127 as well; the six long arms move.

### Tests that changed

Five `units` tests asserted that a reading inverts `at_band` to 1e-9 Band or a
tonne; they now assert the reading's bound, in Bands or as the relative mass
error it implies (`3e-7 · ln 1000`). One `sim` test asserted a reading equal to
1.5 exactly; it now asserts the bound and that `k_mass` is the new pristine
mass exactly. `smoke::snapshot_is_consistent_with_report` compared biomass
against a ceiling rebuilt from a Band; the snapshot now carries
`bio_max_mass`, and the comparison is of two masses with **no tolerance**.


## D.8 T-130 — `exp` and `ln` on the run path as four-multiply minimax polynomials

**Supports:** autopilot spec §3.4, netcode §6 H4a, the T-130 entry in
`hyades_todo.md`, and the run-path section of `src/transcendental.rs`. Author's
direction: *"Replace exp and ln with the best polynomial approximation over the
input range that can be achieved with a four multiply budget."*

**How multiplies are counted:** every floating-point multiply in the function,
range reduction included. Additions, comparisons, bit operations and integer
conversions are free; divisions are not used.

### Measured input ranges (per call site)

A temporary `#[track_caller]` recorder on `ln`, `exp` and `pow`, standard bed
(3 seats, 800 yr, seeds 1 and 7) and combat bed (12 seats, both cards at the
barrier, 400 yr, seeds 1 and 7):

| site | function | calls per run | argument range (union) |
|---|---|---|---|
| freight routing scores (two sites) | `exp` | 1.6–40.3 M | [−3.60, −0.0114] |
| `settler_target` | `ln` | 4.9–14.1 M | [13.8, 5.70e6] |
| `veins` | `pow(10, y)` | 0.20–0.55 M | `y ∈ [−1, 3]`: `exp` of [−2.30, 6.91] |
| `crowding_factor` | `pow(x, ½)` | 0.10–0.53 M | `x ∈ [0.40, 1000]`, `y = ½` always |
| `mining_crew_for` | `pow(x, 2)` | 7 k–81 k | `y = 2` always |
| `logistic_step` | `exp` | 30 k–200 k | −0.0873 and −0.1397 only |
| contract decay | `exp` | 60–5 k | [−0.70, −0.0118] |
| `math::exp_decay` fallback | `exp` | 2 | −2.0117, below its fitted −2 |
| `Qty::at_band` | `exp` | 40 k–84 k | world construction only |

### Candidates, by Remez exchange

Fitted in plain Python (weighted Remez, exact rational solve), maximum error on a
20,001-point grid:

| scheme (≤ 4 multiplies) | interval | error |
|---|---|---|
| `2^f`, degree 3, after `t = x·log₂e` (1 multiply) | `f ∈ [−½, ½]`, any `x` | **7.48e-5** relative |
| `eˣ` direct, degree 4 | freight `[−3.6, −0.0114]` | 8.66e-3 relative |
| `eˣ` direct, degree 4 | centrality `[−2, 0]` | 5.03e-4 relative |
| `eˣ` direct, degree 4 | contract `[−0.8, 0]` | **5.30e-6** relative |
| `eˣ` direct, degree 4 | logistic `[−0.2, 0]` | **5.21e-9** relative |
| `eˣ` direct, degree 4 | veins `[−2.31, 6.91]` | 0.476 relative |
| `log₂(1+f)`, degree 4, exponent from bits, `ln 2` folded into the caller | `f ∈ [√½−1, √2−1]` | **8.76e-5** absolute |
| `ln(1+f)`, degree 3, plus `e·ln 2` (1 multiply) | same | 4.42e-4 absolute |

The best candidate per site, in bold, is what shipped: range-reduced `exp` for the
freight scores and centrality; dedicated degree-4 fits for the logistic step
and the contract decay; `log₂` with `ln 2` folded into `settler_target`'s own
factor. Two sites needed no approximation: `x^½` is `sqrt` and `x²` is `x·x`,
both exact. The `veins` power (`10^y`) goes through `log2_fast` and
`exp2_fast`: about 2.6e-4 relative at `y = 3`.

### Cost and effect

- **Per call** (fastest of 9 passes over 2,000,000 inputs drawn from each site's
  range): `exp` 10.98 ns (T-127) → **3.49 ns**; `ln` 7.63 ns → `log2_fast`
  **4.23 ns**; the logistic `exp` 3.80 ns → **1.70 ns**. The host libm, for
  reference: 5.66 and 5.38 ns.
- **Instructions per event**, seed 7, 400 yr (callgrind): 176,258 → 174,786
  (**−0.84%**), on 63,472 and 63,568 events.
- **`ns/event`**, seed 7, 400 yr, 15 interleaved rounds: paired ratio
  **0.986 ± 0.013** (mean ± SE), below 1 in 9 of 15 — not resolved. The
  freight scores' call count grows later in a run (40.3 M at 800 yr on seed 7),
  and no 800-yr comparison of cost was made, because by then the runs differ.
- **Combat bed**, three interleaved rounds: 50,792 → 51,150 and 50,519 → 50,445
  `ns/event` — not resolved.
- **The runs move**: standard bed, 800 yr, colonies against T-129 **−0.25 ±
  3.02 (mean ± SE, n = 8)**, 2 up, 5 down, 1 tied; population −0.85% to +0.66%.
- **Native against wasm32:** all eleven arms of §D.6 reproduce bit-for-bit.

### What was removed or restated

- `math::exp_decay` (T-102's degree-7 polynomial, 5.4e-7) and its tests are
  deleted; `rank`'s centrality calls `exp_fast`.
- T-127's tangent-bound prune in `best_endowment` is deleted. It relied on the
  logarithm being concave, which an approximation holds only to within its
  error, and the bound cost a division.
- `refining_the_logistic_step_changes_nothing` asserted composition to 1e-9
  with a step argument of −5.24, outside the logistic fit's range; it now uses
  the engine's own step and asserts composition within `n` times the fit's
  5.2e-9 (the step's relative sensitivity to `e^(−rΔ)` is below one).
- `veins_are_a_decade_per_band_and_crowding_pays_at_scale` asserted 1,000 veins
  at `Band IV` to 1e-6; it now asserts `pow_fast`'s bound, 2.6e-4. Measured:
  999.908.

---

## D.9 T-131 — the Technology objective: pricing a static per-role rating, and the proposal it replaces

*Supports `Hyades_technology_tree.md` §4. Harness: `examples/capability_probe`,
release build, `SimConfig::new(1)`, `CombatConfig::default()`, beam family only
(the one built). Deterministic apart from the wall times.*

### The pool at equal spend

Budget `B` = ten General Systems hulls' price (13.154 kt). `N = round(B / m)`.

| hull | dry kt | beams | structure kJ | `N` |
|---|---|---|---|---|
| LSV | 0.0201 | 0 | 20.1 | 654 |
| MSV | 0.1092 | 0 | 109.2 | 120 |
| GSV | 1.3154 | 0 | 1,315.4 | 10 |
| LCV | 0.0200 | 1 | 20.0 | 658 |
| LCU | 0.0200 | 1 | 20.0 | 658 |
| GCV | 1.0995 | 317 | 1,099.5 | 12 |
| GCU | 1.0995 | 317 | 1,099.5 | 12 |
| LOU | 0.0200 | 1 | 20.0 | 658 |
| ROU | 0.1000 | 12 | 100.0 | 132 |
| GOU | 1.0219 | 440 | 1,021.9 | 13 |

LCV, LCU and LOU are one object in every field a bed reads, and so are GCV and
GCU: ten hull names, **seven distinct Designs** (three unarmed, four armed).
`design_loadout` ignores the class, so no class adds a candidate.

### Cost of one combat match

Point blank, LCV against LCV, `N` a side: 0.000 s (10), 0.000 s (50), 0.002 s
(100), 0.008–0.009 s (250), **0.032–0.049 s (500)** — three runs. Every one of
these fights ended with both sides destroyed, so the figures are a **lower
bound** on what a fight that runs the whole 0.5-yr engagement horizon (1,000
ticks at `dt = 0.0005`) costs.

### Beam reach, stationary fleets

Ten LCVs a side, fleets not closing, mean over seeds 1–3:

| separation ly | 0 | 1e-4 | 3e-4 | 1e-3 | 3e-3 | 1e-2 | 1e-1 |
|---|---|---|---|---|---|---|---|
| survivors of 20 | 0.00 | 0.00 | 0.00 | 0.00 | 3.00 | 15.33 | 20.00 |

A beam's hit test compares a light-time extrapolation of the target against
where it actually is, and station-keeping bends the path away from the
extrapolation, so hits thin out with range and vanish by 0.1 ly. Three seeds; the
transition sits between 1e-3 and 1e-1 ly and is not resolved more finely than the
sampled points.

### The short-range round robin

Every armed Design against every armed Design (4 distinct, 7 names, 49
matches), point blank, seed 1: **every share is 0.500, and surviving dry mass
summed over all 49 matches is 0.0000 kt.** Each match destroyed both fleets.
Whole round robin: 0.6–0.7 s over three runs.

The mechanism is inferred from the magnitudes and not instrumented: one mount
delivers 40 shots × 50 kJ = 2,000 kJ per tick, a hundred times a Limited hull's
structure, and fire is simultaneous (warfare §8.17), so each side's first tick
of fire exceeds the other's whole structure.

### Superseded: the power-mean proposal (R-TREE4)

*Carried in `Hyades_trees_and_card_value.md` §2.3.5 and Technology §4.2 through
Rev 1; never ratified.*

Capability was proposed as a vector over three axes measured in situ —
**projection** (deliverable combat mass at range), **defense** (combat mass
within response time of owned colonies) and **acquisition** (ore delivered per
year) — aggregated by a power mean `Q = (Σ_a s_a (q_a / q_ref,a)^ρ)^(1/ρ)`, with
`ρ` measured to decide how strongly a weak axis should dominate (`ρ = 1` a sum
that farms the cheapest axis, `ρ → −∞` a hard minimum with high card-value
variance, `ρ = −1` recommended as a start).

**What it was wrong about, or left unanswered, relative to its replacement:**

- **It measured capability against the live game.** Projection and defense were
  functions of where the rival was and what it flew, so a Technology card's
  value would have carried the variance of the matchup. The static rating
  removes the opponent from the stock entirely.
- **It needed four sets of placeholders** — the axes, their weights `s_a`, their
  references `q_ref,a`, and `ρ` — before one number could be read. The static
  rating has one scale (Elo) and one anchor per role.
- **Two of its three axes needed combat reach the engine did not model**, so it
  could not be built even as a proposal.

**What carries over:** its aggregation argument. A plain sum over hulls could
still farm whichever role is cheapest to be good at; the replacement answers it
with per-role anchors (every strength reads "multiples of the anchor Design in
this role") and leaves role weighting open rather than assuming a sum is safe
(Technology §4.7).

---

## D.10 T-132 — the damage model: beam power over time, structure on hull volume

*Supports `Hyades_warfare_tree.md` §8.18 and resolves Technology R-TECH14.
Harnesses: `examples/capability_probe` (`SimConfig::new(1)`, release) and
`examples/combat_bench` (twelve seats, the Warfare card on even seats and the
Growth card on odd seats at the round-0 barrier, engagements on, 450 yr). Old
and new binaries built from the same tree with and without the change, same
seeds.*

### Superseded: per-tick shot energy and structure on dry mass

*Warfare §8.17 and §8.17.1 through T-131.* A hull's structure was its dry mass
times `hull_hp_kj_per_kt = 1,000` — **1 J per kilogram**, so a 20,000-tonne
Limited hull had 20 kJ, the energy of a rifle round. A beam mount fired
`laser_shots_per_tick = 40` shots of `beam_shot_energy_kj = 50` each tick
(0.0005 yr, 4.4 hours), 2,000 kJ per tick: one hundred Limited hulls' structure
per mount per tick.

**What it got wrong:**

- **The rate was denominated in the integration step.** Damage per tick means
  damage per year scales with `1/dt`, so a numerical requirement set how fast
  hulls died. `CLAUDE.md`'s "a rate is per *something*" in a new place.
- **Every armed fight ended in its first tick.** 49 of 49 equal-spend
  point-blank matches destroyed both fleets (§D.9), so no Design difference and
  no hull difference could act.
- **`laser_shots_per_tick` was the arena's point-defense rate** — tuned against
  missiles in the laser-versus-missile sweep — reused as a hull-killing fire
  rate it was never calibrated for.
- **The structure magnitude had no stated derivation**, which is what the author
  flagged.

### The pool under the new model

`P = 50 MW`, `σ = 10¹² kJ per hull unit³`, `H = 0.5 yr` (1,000 ticks of 0.18
days). One-mount kill time is `S / P`.

| hull | dry kt | `r³` | beams | structure TJ | one mount kills in | `N` at equal spend |
|---|---|---|---|---|---|---|
| LSV | 0.0201 | 0.1066 | 0 | 106.6 | 24.7 days | 654 |
| MSV | 0.1092 | 1.0980 | 0 | 1,098.0 | 254.2 days | 120 |
| GSV | 1.3154 | 32.6228 | 0 | 32,622.8 | 7,551.6 days | 10 |
| LCV / LCU | 0.0200 | 0.0410 | 1 | 41.0 | 9.5 days | 658 |
| GCV / GCU | 1.0995 | 12.9958 | 317 | 12,995.8 | 3,008.3 days | 12 |
| LOU | 0.0200 | 0.0194 | 1 | 19.4 | 4.5 days | 658 |
| ROU | 0.1000 | 0.1691 | 12 | 169.1 | 39.1 days | 132 |
| GOU | 1.0219 | 6.0199 | 440 | 6,019.9 | 1,393.5 days | 13 |

### The short-range round robin (seed 1, point blank, equal spend)

Row's share of surviving dry mass against the column / fight length in ticks:

| | LCV | LCU | GCV | GCU | LOU | ROU | GOU |
|---|---|---|---|---|---|---|---|
| **LCV** | 0.000/82 | 0.000/82 | 0.000/9 | 0.000/9 | 1.000/26 | 0.000/22 | 0.000/6 |
| **GCV** | 1.000/9 | 1.000/9 | 0.000/74 | 0.000/74 | 1.000/5 | 1.000/8 | 1.000/30 |
| **LOU** | 0.000/26 | 0.000/26 | 0.000/5 | 0.000/5 | 1.000/49 | 0.000/11 | 0.000/3 |
| **ROU** | 1.000/22 | 1.000/22 | 0.000/8 | 0.000/8 | 1.000/11 | 0.000/32 | 0.000/5 |
| **GOU** | 1.000/6 | 1.000/6 | 0.000/31 | 0.000/31 | 1.000/3 | 1.000/5 | 1.000/28 |

(LCU and GCU rows equal LCV and GCV.) Order on seed 1: GCV > GOU > ROU > LCV >
LOU, every pairing decided. Fights last 3–82 ticks; the mirror diagonal
28–82. Surviving dry mass over all 49 matches: 515.5 kt. Wall time 17.1 s. One
seed: the order is an estimate on one geometry, not a rating.

**Mirror matches are decisive.** Identical LCV fleets, point blank, seed 1:
survivors 0 / 8 at 10 a side, 0 / 16 at 50, 0 / 43 at 100, 0 / 28 at 250, 0 / 164
at 500. Across seeds 1–3 at 10 a side, side 0 won one of three. *Inference:* the
station-keeping draws decide which fleet the other side's fire control predicts
worse, and the square law amplifies the difference; confidence moderate, and a
per-ship hit-fraction count would confirm or refute it.

### Design law #2 in the engine

One large hull against `N` of a smaller one, point blank, wins out of seeds 1–3:

| | N = 5 | 10 | 20 | 30 | 40 | 50 | 70 |
|---|---|---|---|---|---|---|---|
| 1 GOU vs N ROU | 3/3 | 3/3 | 3/3 | 3/3 | 3/3 | 0/3 | 0/3 |

| | N = 2 | 4 | 6 | 8 | 10 | 14 | 20 |
|---|---|---|---|---|---|---|---|
| 1 ROU vs N LOU | 3/3 | 3/3 | 3/3 | 3/3 | 3/3 | 0/3 | 0/3 |

Crossovers: GOU/ROU between 40 and 50 (the square-law estimate from the hull
table was 36), ROU/LOU between 10 and 14 (estimate 10.2). The law's target is
6–45.

### Beam reach, stationary fleets

Ten LCVs a side, seeds 1–3: mean survivors of 20 / mean ticks — 7.00 / 59 at 0
ly, 6.33 / 64 at 1e-4, 3.33 / 93 at 3e-4, 3.67 / 91 at 1e-3, 4.67 / 79 at 3e-3,
**15.33 / 1,000 at 1e-2, 20.00 / 1,000 at 1e-1**.

### Cost

Point-blank LCV mirror, one run each: 0.001 s (10 a side, 57 ticks), 0.013 s
(50, 71), 0.047 s (100, 66), 0.333 s (250, 84), **1.463 s (500, 68)**.

### What it did to the twelve-seat card bed

`combat_bench 450 1,7,42`, old binary against new:

| seed | fights | hulls destroyed | colonies | yr/s | ns/event |
|---|---|---|---|---|---|
| 1 | 1,580 → 1,479 | 1,580 → **356** | 2,243 → 2,425 | 31.35 → 29.19 | 46,961 → 49,830 |
| 7 | 2,030 → 1,941 | 2,030 → **127** | 2,368 → 2,618 | 27.85 → 25.29 | 48,352 → 52,611 |
| 42 | 2,798 → 2,728 | 2,798 → **369** | 2,158 → 2,378 | 29.82 → 25.82 | 48,472 → 54,778 |

Every destroyed hull on either binary was on the attacking side. Kills fell
77–94% and colonies rose 8–11%. *Inference:* the lost kills are Medium colony
ships a lone Limited blockader can no longer finish inside one engagement (one
mount needs 254 days; the engagement is 183) — warfare R-WAR21. Confidence high
on the direction, since `a_lone_limited_picket_cannot_finish_a_medium_colony_ship`
pins the arithmetic; a per-fight tally by hull type would confirm the share.

`ns/event` rose 6–13%: the workload changed (fights now run many ticks), so by
`CLAUDE.md` §2's reading table this is more work per event, not a regression to
profile — and the run carries 8–11% more colonies.

---

## References

- `CLAUDE.md` §2 — how to search, how to read a gradient, the six traps, and the
  rule that sends entries here
- `docs/Hyades_autopilot_colonization_growth.md` — the Expansion/Growth spec §A
  supports
- `docs/Hyades_politics_trade_and_intelligence.md` — the Politics spec §B supports
- `docs/Hyades_industry.md` §6.8–6.26 — the industry branch's own experiment
  record, which has **not** been moved here yet and is the largest remaining
  instance of the problem this file solves
- `docs/hyades_todo.md` — the open register; an appendix entry is evidence, a
  todo entry is work
