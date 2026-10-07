# Hyades — Experiments Appendix

*The measurement record behind the specs. **Nothing here is normative.** A spec
says what the engine must do; this file says what was run, on which bed, and
what it showed — including the runs that were wrong and the design that was
retracted.*

---

## 0. Why this file exists, and how to use it

`AGENTS.md` §6 splits a spec into **decisions the engine must honor** and
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

That is the plateau `AGENTS.md` §2 records as a measurement artifact: 2048 reads
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
argument `AGENTS.md` §2 makes about a denominator the game can play.

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

*Seven shapes, all of them live in this project at some point. `AGENTS.md` §2
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
~11 galaxies at ~5 min each — over the ~10-minute ceiling `AGENTS.md` §2 sets
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

**Fix:** bill `infra_price_at_band(at + 1) − stock`. Identical for a stock on a
rung. `an_off_band_upgrade_erects_what_it_bills` asserts both directions and
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

**Supports:** the T-126 entry in `hyades_todo.md` and `AGENTS.md` §7's
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

**2.07x, bit-identical.** `AGENTS.md` §4 already names this pattern ("convert at
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
per unvisited world for a policy that keeps one of them (`AGENTS.md` §4, "do not
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
| 203,342 | `infra_band_of`: `round(band)` | threshold |
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
  hulls died. `AGENTS.md`'s "a rate is per *something*" in a new place.
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
`AGENTS.md` §2's reading table this is more work per event, not a regression to
profile — and the run carries 8–11% more colonies.

---

## D.11 T-133 — pricing an encounter before building one

*Supports `Hyades_warfare_tree.md` §8.19.4–8.19.5. Analytic, from the engine's
flip-and-burn (`math::accel_leg_distance`) for a laden Medium colony ship at
0.241 ly/yr² (R-WAR9); one 50 MW Limited mount on target every tick, which is an
upper bound on damage because fire control misses some ticks; one mount wrecks
the hull in 254.2 days (§D.10); wreck curve `p₀ = 0.02`, `x½ = 1` (placeholders).*

| reach `R` | voyage | where | exposure | `D/S` | `P(wreck)` |
|---|---|---|---|---|---|
| 3e-3 ly | any | leaving a port or arriving at a world | 57.64 days | 0.2268 | 0.0470 |
| 3e-3 ly | 6.16 ly (cruise 0.819 c) | mid-voyage pass through the shooter | 2.68 days | 0.0105 | 0.0208 |
| 3e-3 ly | 25 ly (cruise 0.968 c) | mid-voyage pass | 2.26 days | 0.0089 | 0.0207 |
| 1e-2 ly | any | leaving or arriving | 105.28 days | 0.4142 | 0.0928 |
| 1e-2 ly | 6.16 ly | mid-voyage pass | 8.92 days | 0.0351 | 0.0229 |
| 1e-2 ly | 25 ly | mid-voyage pass | 7.54 days | 0.0297 | 0.0224 |

6.16 ly is the median nearest-neighbor spacing (T-115). Wreck curve checkpoints:
`P(0⁺) = 0.020`, `P(0.5) = 0.125`, `P(1) = 0.5`, `P(2) = 0.98`.

---

## D.12 T-133, second landing — named Designs, `σ` per Design class, the wreck roll and the pass

*Supports `Hyades_warfare_tree.md` §8.19, §8.18.2; `Hyades_technology_tree.md`
§1.6. Harnesses: `examples/build_digest`, `examples/capability_probe`,
`examples/combat_bench`; release builds, same seeds, old binary against new.*

### Naming every Design changed no behavior

`build_digest 300 1,7` on the default configuration (combat off), before and
after the class names: events 24,382 / 29,624, colonies 144 / 391 and the raw
population bits `40ce889698d20e05` / `40cb70b8a4741cf6` identical on both seeds.
The names reach only `role_of` and the roster, and every hull resolves to the
role it resolved to before.

### Structure under `σ` per Design class

`10¹¹` kJ per hull unit³ for the Systems Designs, `10¹²` for the armed ones;
one 50 MW mount's kill time: Meadow LSV 10.7 TJ, 2.5 days; Delta MSV 109.8 TJ,
25.4 days; Range GSV 3,262.3 TJ, 755.2 days; Cairn LCV 41.0 TJ, 9.5 days; Scarp
GCV 12,995.8 TJ, 3,008.3 days. The armed round robin and design law #2's
crossovers (§D.10) are unchanged: armed `σ` did not move.

### An encounter, priced (`capability_probe` section 6)

A laden Delta colony ship (0.241 ly/yr², 6.16 ly leg) past `N` Cairn pickets,
fire distance 0.01 ly, 105.3 days in reach either leaving or arriving; energy
absorbed over structure and the wreck roll's odds, seeds 1 / 2 / 3:

| | leaving a port | arriving at a world |
|---|---|---|
| N = 1 | 3.197 / 2.522 / 4.045, odds 1.000 | 3.197 / 2.450 / 3.995, odds 0.999–1.000 |
| N = 2 | 6.395 / 5.051 / 7.839, odds 1.000 | 6.316 / 4.965 / 8.004, odds 1.000 |
| N = 3 | stops at ≈ 8.1 (odds exactly 1.0) | stops at ≈ 8.1 |

A lone mount delivering 2.45–4.05 structures over 105 days means fire control
held on roughly 60–100% of ticks. **A first version of the resolver capped every
case at 1.006 structures** — it stopped assigning mounts to a target past its
structure even when there was no other target — which held the odds at ≈ 0.51
whatever the stack. Fixed before landing: leftover mounts fire on the nearest.

### The twelve-seat card bed (`combat_bench 450 1,7,42`)

| seed | fights | hulls destroyed | colonies | ns/event: pre-T-132 → T-132 → now |
|---|---|---|---|---|
| 1 | 1,580 | 1,580 | 2,243 | 46,961 → 49,830 → 52,635 |
| 7 | 2,030 | 2,030 | 2,368 | 48,352 → 52,611 → 53,531 |
| 42 | 2,798 | 2,798 | 2,158 | 48,472 → 54,778 → 55,826 |

Fights, kills, colonies and event counts are **identical to the pre-T-132
binary**: every port strike on this bed wrecks its colony ship, as the old
one-tick fight did. *Inference:* the wreck odds round to one in every strike at
these placeholders, so the outcome counts cannot tell the two models apart; the
difference is that a survivor is now possible and the time on target is
kinematic. The workload is the same and `ns/event` is 12–15% higher than
pre-T-132 — `AGENTS.md` §2's second row, a per-unit cost: each pass integrates
~580 ticks where the old fight ended in one. One run per seed. The pass stops
once every reachable roll is exactly certain in `f64`, which needs about eight
structures absorbed and so rarely fires here (seed 1: 54,510 → 52,635, one run
each, inside run-to-run spread).

### Test budget, and one flaky guard

Determinism 34.8 s, smoke 25.2 s, unit 2.8 s. `tests/telemetry.rs` failed on
this container 5 times in 11 on the new code and once in 11 on the committed
code (ratios 0.931–1.087 committed, 0.964–1.162 new). **Instruction counts under
callgrind say it is not this change:** full logging costs 0.072% of
instructions on the committed binary and 0.067% on the new one (8,585,169,018
→ 8,591,349,885 and 8,588,060,585 → 8,593,793,787), on the test's own bed. The
threshold is unchanged; the guard's wall-clock samples are what flaked.

---

## D.13 T-133, third landing — pricing fire as events on the main loop

*Supports `Hyades_warfare_tree.md` §8.19.7. Arithmetic from §D.12's twelve-seat
card bed (`combat_bench 450 1,7,42`): 1,580 / 2,030 / 2,798 encounters per run
against 305,643 / 334,189 / 311,366 events, each encounter a laden Delta colony
ship 105.3 days within 0.01 ly of one shooter. An upper bound: an encounter that
ends early in a wreck schedules fewer discharges.*

| discharge period | discharges per encounter | added events (seeds 1 / 7 / 42) | added share |
|---|---|---|---|
| 1 day | 105 | 166k / 213k / 294k | +54% / +64% / +94% |
| 0.18 days (the combat tick) | 576 | 910k / 1.17M / 1.61M | +298% / +350% / +518% |

**Roll at the threshold crossing, priced:** with discharges small against
structure, damage crosses `θ` by a fraction of one discharge, so the roll's odds
at that moment are `p₀` to within that fraction — 0.020 at the placeholders,
for every weapon and every hull.

---

## D.14 T-133, fourth landing — the wreck point, and what fire control costs

*Supports `Hyades_warfare_tree.md` §2.2, §8.19.5 and §8.19.7 (R-WAR24, R-WAR28,
R-WAR29).*

**Superseded: the logistic past a fractional threshold** (T-133 second landing,
§D.12). `P(wreck) = p₀ / (p₀ + (1 − p₀)·e^(−κ·(x − θ)))` with `x = D / S`,
`θ = 0.25`, `p₀ = 0.02`, `x½ = 1`, rolled once per hull when an encounter ended.
What it was wrong about, by the author's ruling: the threshold was a quarter of
the structure where it is the structure; the odds scaled with damage as a
multiple of the structure where they scale with damage above it; and the roll
was taken once where it repeats with further damage. Priced before the ruling
(§D.13): a roll taken at the threshold crossing would have had odds `p₀` every
time, whatever the weapon. The candidates R-WAR28 listed — roll when the
encounter ends, roll at each multiple of `θ`, roll at the crossing with odds
from the damage rate — are all retired by the wreck point, which is the first
candidate's odds taken on every hit.

**The wreck point against the logistic, on the twelve-seat card bed**
(`combat_bench 400 1,7,42`, one run per seed, old and new binaries built from
consecutive commits):

| seed | fights old → new | colony ships wrecked old → new | share wrecked, new | colonies old → new | `ns/event` old → new |
|---|---|---|---|---|---|
| 1 | 1,064 → 1,090 | 1,064 → 1,062 | 97.4% | 1,422 → 1,448 | 55,074 → 53,030 |
| 7 | 1,367 → 1,341 | 1,367 → 1,312 | 97.8% | 1,549 → 1,552 | 52,416 → 52,770 |
| 42 | 1,792 → 1,827 | 1,792 → 1,800 | 98.5% | 1,403 → 1,406 | 54,235 → 53,975 |

The per-event cost moved −3.7% to +0.7%, one run each, which does not resolve a
difference. `capability_probe` section 6 at the placeholders: one Cairn picket
delivers 2.45–4.05 structures to a Delta colony ship over 105.3 days; the odds
at those damages are 0.878–1.000.

**What fire control costs today** (callgrind, `combat_bench 400 1`, release
with symbols; instruction counts, a proxy for time that assumes fire control and
the rest of the run retire instructions at the same rate — not measured):

| quantity | value | kind |
|---|---|---|
| program total | 100.90 G instructions | measured |
| `encounter_at`, inclusive (fire control, the roll, slag) | 0.774 G — **0.77%** | measured |
| `resolve_pass`, inclusive | 0.763 G | measured |
| combat ticks walked by `resolve_pass` (temporary counter, seeds 1 / 7 / 42) | 245,027 / 342,790 / 435,636 over 1,090 / 1,341 / 1,827 passes | measured |
| instructions per tick of fire control, one shooter against one ship | ≈ 3,110 | estimate: the two rows above divided |
| binary-heap work per event | ≈ 117 instructions | estimate: 25.1 M heap instructions over 214,887 events |
| survey candidate scan (`fill_survey_candidates`), inclusive — the `SurveyView` per unvisited world left open under T-126 (§D.5) | 60.1% | measured |
| production decisions (`sys_build_decision`), inclusive | 28.1% | measured |

**The budget, as arithmetic.** Fire control may take at most a quarter of run
time, so its instructions `F` may be at most a third of everything else `O`:
`F ≤ O / 3 = 33.4 G` on this run. A discharge event costs `c_d ≈ 3,110 + 117 ≈
3,230` instructions (estimate). An encounter lasts at most 105.3 days per
shooter (§D.11, an upper bound), so with period `τ` days the run spends
`N_enc · 105.3 / τ · c_d` on fire control, and the budget admits:

| period `τ` | fire control's share at this bed's 1,090 encounters (bound) | encounters the budget admits |
|---|---|---|
| 0.011 days | 25% | 1,090 |
| 0.18 days (the combat tick) | 2.0% | 17,900 |
| 1 day | 0.37% | 98,100 |
| 3 days | 0.12% | 294,000 |

*Inference:* at this bed's encounter count the budget does not bind at any
period above 0.011 days. What will set it is the encounter count once stage 4
finds encounters along every trajectory, and that count is not measured.

**What the upper end of the range is for.** An encounter's exposure is
resolved to within one period, so a period `τ` misstates an exposure `E` by up
to `τ / E`. The shortest exposures priced in §D.11 are 2–9 days for a hull
passing mid-voyage at 0.82–0.97 c, and 58–105 days leaving or arriving: a 5%
bound on that error puts `τ ≤ 0.1` days for a Design meant to hit passing hulls
and `τ ≤ 2.9` days for one that fires at departures and arrivals. The wreck
point makes the period otherwise immaterial to the outcome (§8.19.5).

---

## D.15 T-133, fifth landing — engagement range from weapon accuracy

*Supports `Hyades_warfare_tree.md` §8.19.1, §8.19.2 and §8.19.4 (R-WAR23
resolved, R-WAR27, R-WAR29). Author's ruling: engagement range depends on weapon
accuracy and is a function of Design/hull/class and sometimes role.*

**The derivation, checked numerically before it was built.** Fire control hits a
hull holding station on a circle of radius `ρ` at angular rate `ω` when
`ρ · g(ω d) ≤ ε`, with `d` the light-crossing, `ε` the accuracy and
`g(θ) = √((1 − cos θ)² + (θ − sin θ)²)` (monotone; checked on `θ ∈ (0, 10]`). Over
the spread every simulated hull draws from — `ρ` uniform on 5e-5–2e-4 ly, period
uniform on 0.02–0.08 yr, a 200 × 200 grid — at the arena's `ε = 6e-5` ly:

| distance | share of targets fire control holds against |
|---|---|
| 1e-3 ly | 100% |
| 3e-3 ly | 98.5% |
| 5e-3 ly | 81.5% |
| 1e-2 ly | 29.7% |
| 3e-2 ly | 0% |

Quantiles of the per-target reach: 5% 3.55e-3, 25% 5.63e-3, **median 8.04e-3**,
75% 1.05e-2, 95% 1.48e-2 ly. Against the midpoint target (`ρ = 1.25e-4`, period
0.05) the reach is **7.90e-3 ly**, which is what the engine uses. The median
reach against accuracy: 3.93e-3 / 5.58e-3 / 7.95e-3 / 1.14e-2 / 1.69e-2 ly at
`ε` = 1.5e-5 / 3e-5 / 6e-5 / 1.2e-4 / 2.4e-4 — about `ε^0.51`, the small-angle
`θ ≈ √(2ε/ρ)`. This agrees with §D.10's measured beam reach between stationary
fleets (fights resolve at 3e-3 ly and mostly fail at 1e-2).

**Exposure at the derived range**, laden Medium colony ship at 0.241 ly/yr²:
93.6 days leaving or arriving (`√(2R/a)`), 7.05 days passing on a 6.16 ly
voyage and 5.96 days on a 25 ly one (`§D.11` scaled by `R`).

**What it did, against the 0.01 ly placeholder** (`combat_bench 400 1,7,42`,
one run per seed, consecutive builds): seed 1 fights 1,090 → 1,081, colony ships
wrecked 1,062 → 1,055, colonies 1,448 → 1,449; seeds 7 and 42 reproduce every
printed count (1,341 / 1,312 / 1,552 and 1,827 / 1,800 / 1,406). `capability_probe`
section 6: one Cairn delivers 3.197 / 2.522 / 3.686 structures leaving a port,
against 3.197 / 2.522 / 4.045 at 0.01 ly. *Inference:* the placeholder sat near
the edge of where the arena's accuracy holds, so the extra reach delivered
little energy; accuracy is now the lever that moves the range.

**The determinism gate did not cover combat until this landing.** Every test in
`tests/determinism.rs` ran with `engagements_enabled` off and no card played.
`combat_runs_are_bit_identical` runs the card bed's protocol on six seats and 600
planets, the first barrier pulled to 60 yr, 400 yr: 152 fights (150 wrecked) on
seed 1 and 312 (310 wrecked) on seed 7 in release, 3.0 s for both seeds twice in
debug. The target went 33.0 → 34.1 s. It compares two runs in one process on one
target; native against wasm32 is still the one-off check of §D.6.

---

## D.16 T-133, sixth landing — fire on the main loop, and no harness code in the engine

*Supports `Hyades_warfare_tree.md` §8.19.3, §8.19.6 and §8.19.7 (R-WAR25,
R-WAR26, R-WAR29, R-WAR30 and R-L2 resolved; R-WAR34–R-WAR36 opened) and
`AGENTS.md` §6's harness rule. Author's ruling for this landing: "Harnesses and
test beds cannot have special sim code. The only thing that can vary is the
galaxy generation."*

**Superseded: the recommended encounter** (§8.19.3 before this landing): one
wreck roll when an encounter ended, `resolve_beam_engagement` kept for a pitched
battle, detection scheduling whole encounter windows. What it was wrong about:
the author ruled that the roll repeats on every hit (the wreck point, §D.14) and
that fights run as events concurrently with everything else (§8.19.7), so no
window is resolved ahead of time and no resolver survives in the simulation.

**Retired with the ruling, and what each had measured** (the records stand in
their own sections):

| removed | kind | its measurement |
|---|---|---|
| `SimConfig::engagements_enabled` | master switch | none — it kept the card-free corpus valid, which the unarmed default now does by construction |
| `ablate_oracle_intercept` | oracle | §D.1 (T-122: information was never the constraint) |
| `ablate_strike_fraction` | oracle | §D.4 (T-125: the card needs ~30–45% coverage) |
| `ablate_color_conjunction` | engine variant | §D.1 |
| `ablate_picket_founding_cost` | engine variant | warfare §8.10 (T-116) |
| `examples/engagement_census` | harness | warfare §7.3; its war arm armed nobody, so under discharge events it fires no shot |
| `examples/capability_probe` | harness | §D.9, §D.10, §D.12; it called the deleted resolvers directly |
| slag booked at the nearer of a wreck's destination and home | interim rule | none; superseded by the author's ruling that a wreck keeps its course |

**The card-free run is bit-identical to the engine before this landing.**
`build_digest 400 1,7,42` against the build of d99a790: 42,418 / 63,568 /
61,828 events, 428 / 928 / 842 colonies and population bits `40f4d4aae5b1d064`
/ `40f26050ce22e49a` / `40f2017db8cca76e`, identical on all three seeds. No
card-free Design is armed, so the fire path is not entered.

**Two event storms during the build, both found by the run stalling at a fixed
clock:** detection admitted a hull at exactly its reach and the discharge
dropped it `1e-12` beyond, so one boundary hull was found and dropped at one
instant forever (fixed by dropping only a further margin out); and
`Simulation::position_at` kept its own flight arithmetic, ignoring the braking
prefix, so detection and fire disagreed about a braking hull's position.

**What fire costs, with detection on every trajectory** (callgrind,
`combat_bench`, twelve seats, both cards at the 200-yr barrier, seed 1, release
with symbols; instruction counts):

| horizon | program total | fire code, exclusive (a lower bound) | detection (`track_changed`), inclusive | discharge `fix` | aim sort |
|---|---|---|---|---|---|
| 250 yr | 56.6 G | 4.4% | 2.5% | — | — |
| 400 yr | 266.4 G | **30.1%** | **26.1%** (121 M pair searches) | 11.4% (73 M calls) | 8.4% (3.5 M sorts) |

*Inference:* the budget (fire control at most 25% of run time) was met while
the armed fleets were small and was broken by 400 yr, and the largest share was
the pairwise detection search, not the discharges. Three changes that leave
every result identical — a bounding-box reject before the search, skipping the
position fix for a target whose reference distance already exceeds the reach
plus both station-keeping radii, and an unstable sort on unique keys — were
checked against the binary before them
(`combat_bench 400`, seeds 1 / 7 / 42): every printed count is identical —
4,904,545 / 7,005,020 / 5,204,808 events, 26,752 / 63,158 / 32,948
encounters, 12,845 / 11,999 / 10,903 wrecks, 1,800 / 5,059 / 2,983 retargets,
24,290 / 39,969 / 27,724 withdrawals, 2,018 / 2,214 / 2,531 colonies — and
`ns/event` fell 8,797 → 6,662, 8,658 → 6,654 and 10,817 → 8,660 (−24%, −23%,
−20%; one run each, so these are estimates).

**After those changes, and the discharge change below** (same bed, seed 1,
400 yr): the program total fell **266.4 G → 223.8 G → 201.0 G** instructions.
Fire code is **16.5% exclusive** (a lower bound), and its inclusive rows —
the fire handlers inlined into the event loop, 16.3%, plus detection
(`track_changed`), 8.2%, which overlap where a course change re-runs
detection — sum to **24.5%**, an upper estimate. The survey candidate scan is
now the largest single cost at 30.7% (T-126's open item), production
decisions 18.3%. *Inference:* fire control is inside the 25% budget on this bed
at 400 yr, by a margin smaller than the estimate's own spread; one seed, one
horizon, and instructions taken as a proxy for time. What would change it: a
longer horizon or another seed putting the inclusive sum past 25%.

**Throughput on the combat bed** (`combat_bench 400`, one run per seed per
binary, interleaved; the d99a790 binary runs the retired site model, so this
compares two different mechanics on one protocol):

| seed | d99a790: yr/s, events, `ns/event` | this landing: yr/s, events, `ns/event` |
|---|---|---|
| 1 | 34.43, 214,888, 54,067 | 12.24, 4,904,545, 6,662 |
| 7 | 32.96, 228,354, 53,151 | 8.58, 7,005,020, 6,654 |
| 42 | 33.99, 216,317, 54,396 | 8.87, 5,204,808, 8,660 |

This is the first row of `AGENTS.md` §2's reading table taken to an extreme:
events rose 22.8–30.7× and the cost per event fell 6.3–8.1×, so the
simulation is doing more work, each unit cheaper. Throughput stays 3.4–4.9×
above T-24's floor of 2.5 yr/s on this bed.

**Where the fire goes, and what it does not reach.** On all three seeds **no
colony ship was wrecked** (0 of 12,845 / 11,999 / 10,903 wrecks), against
1,055 / 1,312 / 1,800 colony ships struck per run under the retired site model.
By role at the wreck (seed 1, `combat_bench` after the discharge change):
**12,843 Reserve**, 1 Miner, 1 Scout, 0 Colonizer. A hull is Reserve when it
has stood down or withdrawn under fire (§8.19.7), so the fire lands on idle and
already-fleeing hulls, while 1,800 colony ships turned away on the news or a
first hit and none was wrecked. *Inference:* target priority is nearest-first
(warfare §8.17.2), and a blockader standing on a rival port has that port's
parked hulls nearer than a colony ship leaving it, so its fire goes to them.
The roles support it; the distance from the shooter at each wreck, which would
confirm it, is not measured.

**The discharge change** (position-only aiming, `(distance, entity)` keys, the
nearest few selected lazily) reproduces every printed count on seeds 1 / 7 /
42 against the build before it; `ns/event` 6,662 → 6,351, 6,654 → 6,099,
8,660 → 7,971, measured beside the card-table runs on a loaded machine, so the
instruction profile is the figure to read.

**The determinism gate plays the shipped protocol.** `combat_runs_are_bit_identical`
no longer pulls the first barrier to 60 yr. At the shipped 200 yr, six seats,
600 planets, probed in debug: 250 yr gives seed 1 one encounter and fails the
floor; 275 yr gives 380 and 326 encounters (5.7 s for both seeds twice); 300 yr
gives 745 encounters, 47 wrecks and 466 course changes (seed 1) and 493, 53 and
82 (seed 7), 12.7 s, and ships.

**Wrecks keep their course** (the author's ruling, R-WAR34): a wreck is no
longer booked as slag at the nearer of its destination and its home, but coasts
at the velocity it had, with the ledger carrying wrecks as their own store.
Slag is inert, so nothing else moved: `combat_bench 400` reproduces every
printed count on seeds 1 and 7 against the build before it, and on the unit
bed the wrecks' summed mass equals the logged total exactly.

**Test targets** (debug, `cargo test`, idle machine, one run each, the build of
d99a790 against this one): unit 3.24 → 4.53 s, determinism 48.69 → 50.83 s,
smoke 33.56 → 34.42 s, telemetry 22.74 → 22.66 s. All inside the 60-second
rule; the determinism target's +2.1 s is its combat arm running at the shipped
barrier.

**The Warfare card** (`card_table`, 11 galaxies, 800 yr):

| card | median | **P92 [90%]** | P98 | mean `ln` ratio per galaxy | galaxies above 1 |
|---|---|---|---|---|---|
| Growth | 1.172 | **1.889 [1.742, 2.669]** | 3.749 | +0.150 ± 0.059 (t 2.54) | 10/11 |
| Warfare | 1.087 | **1.334 [1.261, 1.410]** | 1.508 | +0.093 ± 0.019 (t 4.98) | 11/11 |

Against §D.4's table (T-125, before T-132's damage model and this landing):
Warfare's P92 1.204 [1.164, 1.226] → 1.334 [1.261, 1.410], the intervals
disjoint; Growth's 1.753 [1.583, 1.932] → 1.889 [1.742, 2.669]. Every card
landed at the barrier on every seat. Each galaxy took 791–965 s for its three
arms, run as three parallel shards.

*Inference:* the Warfare card still acts on its rivals, and no longer by
wrecking colony ships (none were wrecked on the `combat_bench` seeds above).
What remains is colony ships turned away from worlds and ports a rival holds
under fire, and the rivals' idle hulls wrecked at their ports. Which of the two
carries the ratio is not measured; a census of rival colony-years lost to
retargets against minerals lost as wrecked hulls would split it. Two intervening
changes (T-132's damage model and this landing) separate the two tables, so the
difference is not attributable to this landing alone.

---

## D.17 The first Design rating — fleets generated with the galaxy, and simultaneous fire

*Supports `Hyades_technology_tree.md` §4.4.5–§4.4.7 and §4.9.1 (R-TECH19 opened,
R-WAR36 resolved) and `Hyades_warfare_tree.md` §8.19.3 (the `Hits` event).
`examples/design_rating`: two seats, 200 planets, the two fleets generated at one
point 60 ly off the disk, equal spend `B` = 13.15 kt (ten General Systems hulls),
horizon 1 yr, every pair on every seed with the seats swapped.*

**Seat order decided the fight before the fix** (seed 1, default Doctrine, the
share is seat 0's dry mass holding the field):

| seat 0 v seat 1 | share, discharges landing at once | share, `Hits` after every discharge due |
|---|---|---|
| Tor v Scarp | 1.000 (Scarps all withdrew) | 1.000 (2.88 kt held) |
| Scarp v Tor | **1.000** (Tors all withdrew) | **0.000** (3.28 kt of Tors held) |
| Cairn v Scarp | 1.000 | 1.000 (11.30 kt held) |
| Scarp v Cairn | **0.000** (Cairns 11.24 kt held) | 0.000 (11.36 kt held) |

Both fleets open fire at one instant and share the 0.25-day period, so every
volley is a tie in time; applying each as it fired let seat 0's land first. With
the fix the seatings agree to within 0.40 kt. *Inference:* every simultaneous
exchange in the engine was being ordered by entity sequence until this landing.

**Under the default Doctrine the faster fleet leaves.** In the right-hand column
every Scarp withdrew in every match — a Scarp that can outrun its neutral
attacker breaks off on the first hit (warfare §8.19.6's third ending) — so the
bed measured Doctrine, not Design. Seated hostile on both sides (Technology
§4.4.6), four seeds (1, 7, 42, 31337), 24 matches:

| row v column, mean share | Tor | Cairn | Scarp |
|---|---|---|---|
| Tor | — | 0.000 | 0.000 |
| Cairn | 1.000 | — | 0.000 |
| Scarp | 1.000 | 1.000 | — |

Bradley–Terry with one virtual draw per pair, Cairn anchored: **Tor −383.4,
Cairn 0, Scarp +383.4 Elo**, bootstrap interval a point. Every pair is completely
separated, so the gaps are the prior's (8.5 to 0.5 per pair). The losing fleet
is removed whole — e.g. Scarp v Cairn: 658 Cairns wrecked or withdrawn, the 12
Scarps untouched in mass. 12.2 s for the whole round robin.

**Defeat is withdrawal more than wreck.** The generated-fleet unit test (110
Cairns against 2 Scarps and 110 moving Tors, default Doctrine, half a year) ends
with 65 and 25 hulls withdrawn and **none wrecked**: mounts are allocated to
cover a target's remaining structure, so a hull is pushed past its structure and
leaves (§8.19.6's second ending) before leftover mounts reach its wreck point.
That is why the judge is read as holding the field (R-TECH19).

**What the `Hits` event did to the combat bed** (`combat_bench 400`, seed 1):
events 4,904,545 → 8,571,068 (one `Hits` per landing discharge), encounters
26,752 → 26,652, wrecks 12,845 → 12,639, colonies 2,018 → 2,025, 14.11 →
13.84 yr/s. The card-free digest is unchanged (no card, no fire).

**The Warfare card re-measured after the fix** (`card_table`, 11 galaxies,
800 yr, three shards pooled; the 90% interval is a bootstrap over galaxies in
the harness's seed order):

| card | median | **P92 [90%]** | P98 | mean `ln` ratio per galaxy | galaxies above 1 |
|---|---|---|---|---|---|
| Growth | 1.172 | **1.889 [1.742, 2.110]** | 3.749 | +0.150 ± 0.059 (t 2.54) | 10/11 |
| Warfare | 1.098 | **1.309 [1.267, 1.370]** | 1.541 | +0.091 ± 0.020 (t 4.63) | 11/11 |

Growth's 66 seat ratios reproduce §D.16's to the printed digit (no Growth seat
fires a shot); its interval's upper end differs from §D.16's 2.669 through the
bootstrap replicates, not the data. Warfare's P92 moved 1.334 → 1.309, inside
both intervals, so making fire simultaneous is not resolved as a change to the
card at 11 galaxies. Every card landed on time on every seat.

---

## D.18 The role beds — generated mission fleets, and two faults they exposed

*Supports `Hyades_galaxy_and_autopilot.md` §3.1 (mission dispatch),
`Hyades_autopilot_colonization_growth.md` §8.1 (a hauler stands down when its
rock is settled) and `Hyades_technology_tree.md` §4.4 (the role beds).
`examples/design_rating`, two seats, the standard field (6,723 planets on seed
1), spend `B` = 13.15 kt, a surveyed start of 20 ly for the picket, colonizer,
miner and freighter beds.*

**The first miner bed scored ore no generated miner dug.** At a 40-yr horizon
four different hulls scored identically on each seat (seat 0 44.42 kt, seat 1
809.67 kt, seed 1). An outpost yields once per `mining_tick_years` = 50 yr after
its crew lands, so no generated crew had yielded yet; the score was colony
centers mining rocks the autopilot had since settled, a quantity no hull choice
reaches. Two changes: the bed runs 160 yr (three ticks), and a generated miner
goes only to a world its seat's autopilot ranks a mining outpost, which the
baseline never settles. After both, seed 1, seat 0 at 160 yr: Meadow 103,439 kt,
Delta 44,103 kt, Range 362 kt.

**The first freighter bed never finished a match.** Traced by stepping the
event loop: from 40.218 yr on seed 1, two freighters' `FreighterArrive` events
fired at that one clock reading, alternating, for as long as the run was left
going. Each hauler's rock (planet 886) had been settled by its own seat, so the
delivery router picked the center the hauler stood on, the leg had zero length,
and the return leg to the same rock did too. Nothing ended the pair, because
only an exhausted rock did. A baseline game does not reach the state because
the baseline mines only worlds below its colonization bar; a generated miner
fleet sent to every scanned rock did. The engine now stands a hauler down when
its own empire has settled its rock (`a_hauler_whose_rock_is_settled_stands_down`;
with the guard removed that test fails). Card-free runs are bit-identical across
the guard (`build_digest 400 1,7,42`: popbits 40f4d4aae5b1d064,
40f26050ce22e49a, 40f2017db8cca76e, as before).

**Cost after both fixes, seed 1, one core:** short-range 11.7 s, long-range
24.6 s, picket 11.4 s, colonizer 1.8 s, miner 3.8 s, freighter 4.5 s, scout
3.9 s for each bed's 30 matches.

**The first table over every role** (`design_rating all`, seeds 1, 7, 42,
31337, 2, 3, 5, 11, both seatings: 240 matches a bed, 1,680 in all, 7.5 min on
one core). Raw record: `data/design_rating_matches.tsv`; ratings:
`data/design_ratings.tsv`. Elo points, the anchor at 0, 90% interval from 1,000
bootstrap resamples of the seeds; *prior* marks a rating whose pairs are all
separated completely, so its gap is the virtual draw's (8.5 to 0.5 per pair):

| Design | short-range | long-range | picket | colonizer | miner | freighter | scout |
|---|---|---|---|---|---|---|---|
| Meadow=Spur (LSV) | −731.6 *prior* | −72.1 [−73.1, −71.2] | −574.9 [−580.3, −570.0] | −820.1 [−821.5, −818.6] | **0** | −302.5 [−307.6, −297.3] | **0** |
| Tor (LCV) | −286.8 *prior* | −49.0 [−49.6, −48.6] | 0.0 | −820.1 | −0.1 [−0.4, 0.2] | −726.4 [−732.0, −720.8] | 0.0 |
| Cairn (LCV) | **0** | **0** | **0** | −820.1 | −0.1 [−0.4, 0.2] | −726.4 | 0.0 |
| Delta=Ford (MSV) | −732.1 *prior* | −70.9 [−74.3, −67.1] | −574.9 | **0** | −79.5 [−100.4, −54.6] | **0** | −111.3 [−113.5, −109.1] |
| Range=Strait (GSV) | −731.5 *prior* | −86.1 [−88.4, −83.9] | −574.9 | −278.2 [−279.8, −276.6] | −280.0 [−386.2, −187.4] | +101.6 [+95.0, +109.3] | −407.9 [−412.5, −404.0] |
| Scarp (GCV) | +351.9 *prior* | +295.9 [+292.0, +299.9] | +122.6 [+97.3, +148.1] | −254.4 [−256.0, −252.7] | −273.9 [−377.3, −183.3] | +12.9 [−4.8, +29.0] | −383.3 [−388.2, −379.0] |

Matches decided outright (one side scoring zero): short-range 192, long-range
47, picket 144, colonizer 144, miner 0, freighter 128, scout 0, of 240 each.

What each column reads, from the share matrices:

- **Short-range.** Every armed pair is decided the same way on every seed:
  Scarp over Cairn over Tor. The three unarmed Designs draw one another (both
  sides hold the field) and lose every match to an armed one. Every gap is the
  prior's.
- **Long-range** (`D_long` = 0.03 ly, closing at 0.45 c). 47 of 240 decided.
  Scarp takes 0.644 of the field against Cairn and 0.886 against Delta; every
  Limited armed Design takes 0.52–0.58 against an unarmed one. At a relative
  0.9 c the fleets cross the widest beam reach (7.9e-3 ly each way) in 6.4 days
  (an upper bound on time in range if neither has braked). *Inference:* this bed
  mostly measures what one pass delivers (R-TECH12, R-L2). Confidence moderate;
  a sweep of the closing speed, or a count of discharges per match, would settle
  it.
- **Picket.** Tor and Cairn draw each other exactly; Scarp takes 0.699 of the
  denials against either. Unarmed Designs deny nothing.
- **Colonizer.** Limited hulls found nothing (a Limited hold carries no founding
  seed — capability, not competence). Delta takes 0.885 of the colonies against
  Range and 0.865 against Scarp.
- **Miner.** Crews count hulls, one Limited hull one unit (T-71), so equal spend
  favors the cheapest hull: Limited over Medium (0.629) over General (0.849).
  The three Limited Designs are one object to the miner (same tier, same mass;
  mining reads no weapon).
- **Freighter** (Meadow miners on both seats). Range delivers 0.665 of Delta's
  share, Scarp ties Delta (0.522); a Limited Systems hull delivers 0.08 of a
  Medium's, and a Limited Contact hull delivers nothing (its volume is weapons
  and drive).
- **Scout.** The ranking is hull count. `launch_survey` flew every hull at
  `doctrine.survey_accel_g`, a flat constant (the defect recorded in warfare §8.9.7),
  so no hull property but its price reached the bed. *(Removed since: §D.19.)*

**Superseded: R-TECH10's recommendation (Technology §4.7.3).** It read: *"A
hull's role changes during a game; its Design does not. If a hull carried its
current role's strength, a Doctrine write that retasked hulls toward whichever
role rates them highest would raise `Q_i` without building anything — the
invariance rule's failure case. Recommend `c(d) = max_r γ(d, r)`, the best role
the Design can fill."* What it was wrong about, per the author's ruling: a hull
working a role it is poor at is not delivering capability that year, so a
Design's best-role strength overstates what the fleet did. Retasking toward
strength is capability used, not a metric farm.

---

## D.19 Every leg flies its Design's drive — `survey_accel_g` and `civilian_accel_g` removed

*Supports the author's ruling — "survey_accel_g must be removed. No overwriting
the Design of a ship by the sim" — as recorded in
`Hyades_autopilot_colonization_growth.md` §2.1, `Hyades_loadout.md` §5 and
`Hyades_warfare_tree.md` §8.9.7.*

**What was overwritten.** Thirteen sites flew or priced a hull at a flat
rate in place of its Design's drive: the two survey legs at
`Doctrine::survey_accel_g · G`, and the scrap leg, a colony ship's bounce home,
both Reserve re-taskings, a new freighter's first leg, a parked hull's motion,
the settler travel discount, the delivery and pickup routers and the Exchange's
freight leg at `SimConfig::civilian_accel_g · G` (1 g). The laden legs already
read the hull (`laden_accel`, with `civilian_accel_g` as a 1.0 throttle). Every
site now reads `laden_accel` or, for a forecast, the same `thrust_to_mass` of
the hull that will fly: the settler discount prices a colony ship laden to its
seed capacity (`colony_ship_accel`), the routers price the hauler's own laden
rate, and the Exchange leg is flown by the seller's standing Freighter Design
in as many full loads as the lot needs. Both constants are deleted.

The drive ladder the sites now read (T-96): empty, 1.00 / 2.37 / 5.06 g for
Limited / Medium / General Systems hulls, 0.911 g for a Limited Contact hull;
a laden Medium colony ship 0.234 g. So a Limited scout flies where it did, and
a Medium hull on an empty leg flies 2.37 times as fast as before.

**The armed-scout write is no longer inert.** `scout_hull_offensive` reproduced
its baseline bit-identically while the survey leg ignored the hull
(§8.9.7). At 400 yr, 3 seats, the write now moves the run: 43,773 → 41,860
events on seed 1 and 61,545 → 62,491 on seed 7.

**Runs move** (`build_digest 400`, 3 seats): colonies at 400 yr
428 → 452, 928 → 891 and 842 → 726 on seeds 1, 7 and 42.

**Test targets, unloaded, two interleaved runs each:** `tests/determinism.rs`
44.1 / 44.3 s before, 42.0 / 42.1 s after. Readings of 59.3 and 60.4 s were
taken while other runs shared the four cores and are not a property of the
change.

**The objectives** (`work_years`, 3 seats, 4,000 yr, seeds 1, 7, 42, 31337, 2,
3, 5, 11, paired against the pre-change binary):

| arm | work-years | colony-years |
|---|---|---|
| first landing (Exchange leg as one hull's sequential full loads) | **−18.26% ± 4.71**, 1/8 positive | +0.03% ± 0.09 |
| shipped (Exchange leg as one laden voyage) | **+2.26% ± 4.69**, 5/8 positive | +0.03% ± 0.09 |

Throughput on the shipped arm +1.50% ± 1.25 yr/s, `ns/event` −1.40% ± 1.18.

**How the −18% was found, because no single intuitive site carried it.**
Ablations in scratch builds, each restoring one group of sites to the old flat
1 g, measured against the first landing: the routers' forecasts +3.55% ± 3.52,
the settler discount +6.37% ± 2.41, the flight legs +5.80% ± 3.82 — each
leaving −13 to −16% against the old binary. A census of mineral flows at
1,500 yr (four seeds) named the channel: freight deposited fell 18–28% and
infrastructure builds 8–14% on every seed, and none of the three ablations
restored either. The Exchange leg was the remaining changed site, and it had
been rewritten twice — to the seller's Freighter drive, and to `n` sequential
full loads of one hull. Restoring it either to the flat 1 g or to a single
laden voyage at the Design's drive returned freight and builds to the old level
(seed 7: 279,886 kt old, 277,663 and 280,825 kt in the two arms). So the
sequential loads were the cause, not the drive; the engine ships the single
voyage. *Inference:* a trade that settles over many hull-trips arrives too late
to fund the next rung, and the bank that waits is the one deepening; confidence
moderate, and a per-contract settlement-delay histogram would test it.

**The ratio-scale table** (`design_rating all`, the same eight seeds, both
seatings; R-TECH8's ratio scale, anchor 1; `data/design_ratings.tsv`):

| Design | short-range | long-range | picket | colonizer | miner | freighter | scout |
|---|---|---|---|---|---|---|---|
| Meadow=Spur (LSV) | 0.0148 *prior* | 0.660 [0.656, 0.664] | 0.0365 [0.0354, 0.0376] | 0.0086 | **1** | 0.181 [0.175, 0.188] | **1** |
| Tor (LCV) | 0.192 *prior* | 0.754 [0.752, 0.756] | 1.000 | 0.0086 | 0.9995 [0.998, 1.001] | 0.0157 | 0.989 [0.986, 0.991] |
| Cairn (LCV) | **1** | **1** | **1** | 0.0086 | 0.9995 | 0.0157 | 0.989 |
| Delta=Ford (MSV) | 0.0148 *prior* | 0.665 [0.652, 0.679] | 0.0365 | **1** | 0.633 [0.561, 0.730] | **1** | 0.582 [0.574, 0.590] |
| Range=Strait (GSV) | 0.0148 *prior* | 0.609 [0.601, 0.617] | 0.0365 | 0.193 [0.189, 0.197] | 0.200 [0.109, 0.340] | 1.81 [1.65, 1.98] | 0.120 [0.117, 0.122] |
| Scarp (GCV) | 7.58 *prior* | 5.49 [5.37, 5.62] | 2.03 [1.75, 2.35] | 0.221 [0.217, 0.226] | 0.206 [0.114, 0.347] | 1.12 [1.02, 1.23] | 0.127 [0.124, 0.130] |

Against §D.18's table (the same seeds, before this change), the combat and
colonizer orders are unchanged; the scout bed now separates the LCV Designs
from the LSV (0.989, their drive 0.911 g against 1.00), which a flat survey rate
could not. Intransitivity (R-TECH7): no cyclic triad in any bed; largest
residual 0.20 (long-range, Cairn against Scarp).

---

## D.20 T-134 — the Exchange at a spatial equilibrium, the collection capacity it was missing, and one holding per (empire, planet)

*Supports `Hyades_matching.md` §8 and politics §2.14/§2.16. Bed: `examples/work_years`, 3 seats, 4,000 yr, seeds 1, 7, 42, 31337 and the replication set 2, 3, 5, 11; paired log-ratios with standard errors across the 8 seeds unless stated. Every ablation below was a scratch build, measured against the shipped binary, and none landed (`AGENTS.md` §6).*

**The optimal clearing, alone, regressed.** `clear_spatial` reproduced scipy's
LP optimum on 9 seed-1 books to every printed digit and passed every equilibrium
condition on 40 random books, and it cost **−18.89% ± 4.28% work-years, 0/8
seeds up** (89.75M → 74.55M). The Exchange switched off scored 71.30M, so the
shipped greedy wave was worth +25.9% and the optimal clearing kept +4.6% of it,
**while moving the same tonnage** (seed 1 at 1,500 yr: 146,792 → 147,804 kt).

**Five mechanisms refuted by ablation before the cause was found:**

| hypothesis | ablation | result |
|---|---|---|
| sellers now keep their reservation | reservations zeroed | seed 2, 1,500 yr: 5.43M → 5.92M (shipped 7.25M) |
| asks post the bill's share of a bank | asks post only the spare | 72.95M (8 seeds) |
| the buyer's collection leg is unpriced | leg cost + collection leg | 75.71M |
| settlement timed on the whole ask | timed on the sold lot | 76.26M |
| trade itself is harmful | Exchange off | 71.30M — worse |
| deliveries concentrated on few venues | each lot split over 4 venues | loaded 5.5% / 11.1% — unchanged |

**The census that found it** (per-lot tracking of Exchange ore through the
buyer's holding at the venue, proportional attribution on each load, seeds 2
and 7 at 1,500 yr). Every venue is served by one buyer hauler (p10–p90 = 1) with
a laden round trip of ~100–200 yr. Loaded fraction, lots delivered before 1,100 yr:

| lot size | shipped greedy (s2 / s7) | optimal clearing (s2 / s7) |
|---|---|---|
| 10–100 kt | 54.3% / 70.1% | 68.1% / 74.2% |
| 100–1,000 kt | 31.9% / 47.8% | 33.7% / 62.9% |
| ≥ 1,000 kt | none | **1.0% / 4.2%** (117k / 105k kt) |

At equal lot size the optimal clearing's lots are collected as fast or faster;
the whole deficit sits in lots of ≥ 1,000 kt, which the greedy wave never made
because a fill was one bid (≤ ~380 kt). The optimal clearing sends a buyer's
whole demand for a color through a few cheapest legs, mostly to rocks already
rich in that color (91% of kt on seed 2). Refuted along the way by the same
census: the delivered color being unwanted by the venue hauler's destination
(similar shares, similar rates) and longer round trips (medians 142 vs 157 yr).

**The 2×2 that proved it** — capacity = what the buyer's based haulers move in a
400-yr round less what already waits, per color, spilling to the next shared rock:

| | no cap | cap |
|---|---|---|
| greedy wave | 89.75M | 140.87M, **+45.35% ± 3.14%**, 8/8 |
| optimal clearing | 74.55M, −18.89% ± 4.28%, 0/8 | 139.33M, **−1.27% ± 2.84%** against greedy+cap, 4/8 |

The cap carries the whole effect; once it is present the clearing rule does not
move work-years. Loaded fraction under the cap: 36.9% / 37.3% (seeds 2 / 7),
against 18.0% / 24.8% for the shipped wave.

**The landed stages** (engine code, not scratch):

| stage | what | result |
|---|---|---|
| A | one holding per (empire, planet) | **bit-identical** to stage 1 on seeds 1, 2, 7 at 1,500 yr (work-years, colony-years, works, colonies, vehicles, events) |
| B | capacity as route capacities in the LP, then placement | **+47.29% ± 3.04%**, 8/8 against the shipped wave |
| B+C | plus asks from every holding away from a yard | **+51.27% ± 1.71%**, 8/8; against B **+3.98% ± 2.09%, 6/8 — not resolved** |

Colony-years moved by −0.00% under B+C.

**Throughput.** Stage A's first version kept every holding in one ordered map
and was bit-identical at **+19% to +27% per event**; indexing a holding at an
owned planet by the planet removed it (stage A per event 10,353–11,776 ns
against stage 1's 12,175–13,734, 6/6 pairs). Stage B+C's first version summed
each route's room over a sorted venue list per ask — 3.46 s of a 12.8 s run
spent building routes, against 0.010 s solving and 0.013 s placing — and cost
−17% yr/s; computing the shared-rock set and its room once per (seller, buyer,
color) is bit-identical and runs **145.1–156.8 yr/s against the shipped
binary's 128.4–135.7** (3 seats, 1,500 yr, seeds 1 and 7, 3 interleaved rounds;
10,874–11,993 against 12,645–13,116 ns/event). 12-seat combat bench:
11.51–11.59 against 11.67–11.97 yr/s with 9% more events at lower ns/event.

**Two habits from it.** *A proven-optimal allocation is optimal for the model it
was given* — the LP treated ore dropped at a rock as delivered, and the engine
only moves it at one hauler's rate; the census, not the objective, named the
missing term. And *ablate the pieces of a 2×2 apart before crediting either*:
the clearing rule looked like the cause of a −19% and the cure of a +44%, and it
was neither.

## D.21 T-134 stage 2 — the internal duty exchange, and self-trade

*Supports `Hyades_matching.md` §9, `Hyades_warfare_tree.md` §8.20 and roles
§4.5b. Card-free bed: `examples/work_years`, 3 seats, 4,000 yr, the 8 seeds of
§D.20, paired log-ratios against the branch binary before stage 2 (`c5df349`),
standard errors across seeds. Combat bed: `examples/combat_bench`, 12 seats,
both cards at the barrier, 450 yr. Every scratch arm below was measured against
the shipped binary and did not land.*

**Self-trade — the question asked of stage 1's clearing.** An arm that let an
empire's asks fill its own bids scored **−2.51% ± 1.59% work-years, 2/8 seeds
up**, and colony-years **+0.00% ± 0.01%**. Not resolved at 2 SE; the sign leans
harmful. An inference, held at about 60% confidence: a self-trade moves ore an
empire could spend at a center out to its own pile at a shared rock, where it
waits for a hauler. A per-lot census of self-filled lots by where they were
spent would change that. The shipped clearing drops self-routes.

**Why self-trade moves nothing — a census of the book** (`examples/holding_demand`,
3 seats, seeds 1 and 7, every barrier to 2,000 yr; `Simulation::book_census`
reads the book as posted and the fills as struck). There is demand for what the
Holdings hold, and it is small against them: from year 600 on, bids are 1–19%
of what is held away from yards (seed 1, 1,000 yr, Cyan: 42,627 kt bid against
1,806,461 kt held away; seed 1, 1,800 yr, Yellow: 348,940 against 1,869,490).
Centers also bank more in colors they are not short of than all the bids
combined in most rows. Bids are small because a center bids only its shortfall
against its next rung (the same caveat `unmet_color_demand` carries, §6.20).

Fills against bids, shipped (no self-routes) and a scratch build that allows them:

| seed, year, color | bid (kt) | filled, shipped | filled, self allowed | of which self | own haulers' spare room |
|---|---|---|---|---|---|
| 1, 600, C | 11,252 / 13,628 | 11,252 | 13,628 | 1,035 | 13,628 |
| 1, 1,000, C | 42,627 / 43,528 | 42,627 | 43,528 | 4,192 | 43,528 |
| 1, 1,800, C | 226,513 / 216,004 | 22,699 | 20,323 | 8,736 | 20,346 |
| 7, 1,000, Y | 62,025 / 57,538 | 62,025 | 57,538 | 21,388 | 57,538 |
| 7, 1,800, Y | 223,234 / 211,417 | 35,069 | 35,315 | 11,456 | 35,338 |

Two regimes, and self-trade adds nothing in either. **Early**, every bid fills
from rival sellers, and self-fills (7–40% of the volume) displace rivals. **Late**,
fills stop short of bids at the buyer's own haulers' spare room at the rocks it
shares with the seller — with self-trade allowed, fill equals that room to
within 0.4% in every row where it falls short of the bid — and a self-route consumes the same room. Across
rows the fill total moves −11% to +13% between the builds (one run per seed).
An inference: self-trade reassigns fills and burns `exp(−λt)` of what it moves,
which is where the −2.51% ± 1.59% work-years above would come from; a
per-lot account of burned mass would test it.

**The miner freight run alone** (matching §9.1): work-years **+0.63% ± 3.32%,
3/8 up**; colony-years −0.01% ± 0.02%; colonies identical on all 8 seeds. The
run fires: 2,998 and 1,817 runs on seeds 1 and 7 over 1,500 yr, most of them
after year 500.

**With the colony ship's run before embarking** (§9.2): work-years **+0.80% ±
1.89%, 3/8 up**; colony-years **+0.03% ± 0.02%, 7/8 up**; colonies identical on
all 8 seeds. Over 1,000 yr on seeds 1 and 7, miners make 2,723 and 1,460 runs and
colony ships 1,511 and 1,341. What triggers the colony run was measured before
it was built: 22.2% and 22.8% of colony ships launch with fewer settlers than the
hold carries (6,201 and 5,776 launches, seeds 1 and 7, 1,000 yr). Per-seed
work-years swing from −3.8% to +11.5%, which reads as a reordering of a
compounding run rather than a gradient; no default moved on this number.

**The picket sortie** (warfare §8.20). The first predicate — both hulls
standing when the encounter begins — was censused with a scratch print on the
combat bed, seed 1: **0 of 41,770** encounter starts had both standing. By role
pair, 3,240 were picket against picket, the rest a picket against a scout,
miner, freighter, colony ship or withdrawing hull. For picket-against-picket
starts, the shooter's side had another post within 1 ly in 1,228 and the
target's side within 2 ly in 11. With the shipped predicate (both armed, one
standing) and the 2 ly placeholder reach: **3 sorties on seed 1, 0 on seed 7**;
a scratch build at 8 ly flew 105 on seed 1.

**Superseded by the author's ruling on "nearby"** (warfare §8.20): the reach is
now belief about arriving before the battle is decided, with no distance
constant. The 2 ly and 8 ly counts above are a record of the retired rule.
`a_picket_joins_a_battle_it_can_reach_in_time_and_returns_to_its_post` sets
the fight's believed length to twice and half the picket's light-plus-flight
time and asserts one sortie and none. Its first run failed on a real defect —
the arrival check read `world.position`, which a parked hull does not carry, so
every flight time was infinite; it now reads the hull's position through its
motion.

**The belief rule on the card bed.** Two interleaved rounds against the fixed
reach, seeds 1 and 7, 450 yr: **0 sorties on both seeds** (3 and 0 before), and
`ns/event` 3,522–3,568 against 3,461–3,553 on seed 1 and 4,561–4,655 against
4,706–4,777 on seed 7 — within the run-to-run spread. A scratch census of every
pitched encounter on seed 1 (3,366; 3,165 Cairn against Cairn, 201 Cairn against
Tor): believed fight length p50 0.019 yr, maximum 0.026 yr; distance from the
battle to the nearest post of either side p05 5.94 ly, p50 14.06 ly, p95 49.7 ly;
encounters with any post nearer than the fight's length, in light-years: 0.
So the predicate cannot be true at current beam and structure magnitudes.

**Throughput.**

| bed | before stage 2 | stage 2 | reading |
|---|---|---|---|
| 3 seats, 1,500 yr, seeds 1 and 7, 3 interleaved rounds, ns/event | 11,815–12,124 and 11,881–12,170 | 12,116–13,857 and 11,575–12,403 | min-of-3 +2.5% and −2.6%: not resolved; events +0.9% and +1.1% |
| combat bench, seed 1, 2 interleaved rounds | 11.17 / 11.34 yr/s, 3,205 / 3,157 ns/event, 12,574,563 events | 10.87 / 10.92 yr/s, 3,408 / 3,393 ns/event, 12,144,877 events | about −3% yr/s: fewer events, each dearer |

Callgrind on the combat bed (seed 1, 300 yr) puts the new code's own
instructions at `best_delivery_center` 0.30% (regular haulers included),
`embark` 0.06% and `call_to_battle` 0.002%, against `fill_survey_candidates` at
45.5%. An inference: the combat bed's per-event rise is the changed run — which
hulls fly where, and so which survey scans run — and not the cost of the side
duties. A per-function comparison against the old binary's profile would test
it.

## D.22 R-MX8 — a center's abundance hauled to a center with demand

*Supports `Hyades_matching.md` §8.5. Card-free bed: `examples/work_years`,
3 seats, 4,000 yr, the 8 seeds of §D.20, paired log-ratios against `main` at
`c831aad`, standard errors across seeds. All four binaries ran at once on a
4-core container, so the `yr/s` column compares like with like and not with
other entries.*

**The objective.** Colony-years **+1.29% ± 0.22, 8/8 seeds up** (5.9 SE).
Work-years **+6.22% ± 3.62, 5/8 up** — 1.7 SE, not resolved; per seed −2.1% to
+28.9%, which reads as a reordering of a compounding run rather than a gradient.
Colonies are identical on all 8 seeds, because the bed is saturated at `k_high`
(`AGENTS.md` §7). Vehicles +10.1% ± 2.3, 7/8 up.

An inference: the colony-years gain comes from ore reaching young colonies'
first rungs sooner. Confidence about 70%. A per-colony census of the time from
founding to first build in both arms would confirm or refute it.

**Mechanism check** (a scratch counter over `FreighterTransfer` loads, keyed on
whether the planet was owned by the loading empire at that time; 3 seats,
1,500 yr). Loads at owned centers: **8.19% and 7.25% of loaded tonnage** on
seeds 1 and 7 (6,050 and 5,996 loads), against **0.00%** on `main`, which is
the control that says the counter reads what it claims to. On 6 seats, seed
31337, the first such load is at 32.4 yr. `examples/bank_mix` (seed 1, 800 yr),
`main` → R-MX8: freight's share of banked ore 8.88% → 9.97%, of which some is
ore banked twice (a center-to-center load re-enters a bank); median payable
fraction 0.054 → 0.059; infrastructure builds 523 → 650.

**What it cost, and where.** The first build ran `ns/event` **+32.4% ± 2.7, 8/8
seeds**, with events +2.6% — the "slower per unit" row of `AGENTS.md` §2's
reading table. Callgrind (seed 1, 1,000 yr, the three freight functions kept
out of line in a scratch build): program 43.5 G → 58.4 G instructions, of which
`next_pickup` went 5.1 G → 17.4 G — the offer priced for every owned center on
every milk-run stop (7.4 G) and the walk over every planet to find them
(about 4.9 G). Two changes, each bit-identical to the first build at 1,000 yr
on seeds 1 and 7 (events, colonies and population to the printed digit):

| build, 1,000 yr, seeds 1 and 7, 2 interleaved rounds | `ns/event` | `yr/s` | events |
|---|---|---|---|
| `main` | 14,915–16,633 | 128.4–136.9 | 468,107 / 512,219 |
| R-MX8, first build | 18,366–21,232 | 86.7–98.8 | 543,252 / 551,095 |
| + the buyer's side read once, abundance tested before the leg | 18,084–19,926 | 91.6–100.3 | same |
| + `owned_planets`, a per-seat index of owned planets | **14,276–15,080** | **122.1–127.1** | same |

The index also serves `best_delivery_center`, which walked the galaxy on `main`
too (7.8 G of `main`'s 43.5 G). At 1,000 yr the per-event cost is at or below
`main`'s, and the remaining `yr/s` gap is the +16% and +8% more events the
changed run does. **On the full bed** (8 seeds, 4,000 yr, the final build
reproducing the first build's work-years, colony-years, colonies and events on
every seed): `ns/event` **+2.18% ± 1.06, 6/8 up**; `yr/s` −4.64% ± 0.93;
events +2.62% ± 0.22. The residual per-event cost is 2.1 SE and is not
located; a callgrind pair at 4,000 yr would locate it.

**Test budget.** `no_nan_or_infinity_reaches_replicated_state` went 37.4 →
44.4 s at 200 yr and the determinism target 53.9–55.0 → 62.8–62.9 s. Trimmed to
150 yr (probe in the test's comment), the target runs 56.1–58.4 s; at 130 yr it
is 55.0–56.2 s, so that test no longer sets the target's time —
`stepping_in_any_granularity_reaches_the_same_state` (35.6 s on `main`, 36.0 s
here) and scheduling do. The target is under 60 s and above the 54 s the
tolerance band asks a fix to reach; `main` itself runs it at 53.9–55.0 s.

## D.23 T-138 — supers, apex, the refined Exchange, priced production, and a reachable Band IV

*Supports galaxy §3.0 and §4.5, technology §3.2 and §3.7, trees §4.6, matching
§8.7 and §10.4. Card-free bed: `examples/work_years`, 3 seats, 4,000 yr, the 8
seeds of §D.20, against the R-MX8 build (`e7e883b`), all four binaries at once.*

**Population Band IV was unreachable.** The top population edge
(`PopBands::from_weibull(1.4, 4.0)`) is the `Band IV` mass, 715,541.75 kt, and
so was every homeworld's `K` (habitability and pristine biosphere both `Band
4.0`); no world had a ceiling above it (0 of the 200-planet test galaxy, 19 at
it). The closed-form logistic approaches its ceiling without reaching it, so no
world could hold population `Band IV` and the synthesis gate could never open;
a homeworld stood at 61% of it at 600 yr. The same comparison feeds
`industrial_signature`, which could never be true either.

**Calibrating the ceiling to the author's schedule** (a scratch harness, never
landed: seat 0's homeworld population against the `Band IV` edge every 10 yr;
seeds 1, 7, 42, 31337; 1,600 yr; rounds at 200 / 600 / 1,000 / 1,400 yr):

| ceiling (Band) | card-free, every seat | Growth card ×1.6 at round 1, seat 0 | Expansion card ×8 at round 1, seat 0 |
|---|---|---|---|
| 4.05 | 693–745 yr | 503–522 yr | identical to card-free |
| 4.1 | 653–705 yr | 483–502 yr | identical to card-free |
| 4.2 | 632–685 yr | 473–482 yr | identical to card-free |
| 4.3 | 623–675 yr | 463–482 yr | identical to card-free |

At 4.2 a growth-dedicated build crosses before round two's card selection and
every card-free seat crosses between rounds two and three, as the ruling asks.
The ceiling moves the crossing by about 70 yr over 4.05–4.3. The Expansion card
draws no population, so no build in the engine drains a homeworld faster than
the default and the ruling's round-four case cannot be measured yet.

**Card-free, the whole layer is inert** — the 4.2 ceiling and every refined
mechanism together: work-years, colony-years, colonies, vehicles and events
**bit-identical on 8/8 seeds**; only the population figure moves (seed 1 at
1,000 yr: 64,267,894 → 68,593,963 kt). An inference, at about 90% confidence:
nothing a card-free run decides reads a homeworld's population above the
`Band III` edge. A census of the decisions that read population would confirm it.

**Throughput.** The first build ran `ns/event` +5.49% ± 1.05 (8/8) on that bed
with identical events — the new code's own cost. With fast paths for a game
with no standing order and for a hauler with no refined want, interleaved at
1,000 yr (3 rounds, seeds 1 and 7, events identical): 15,017–15,287 against
15,110–15,477 and 14,053–14,861 against 14,285–14,788 `ns/event`; min-of-3
+0.6% and +1.7%, inside the run-to-run spread — not resolved. Test targets:
unit 5.6 s, determinism 51.2 s, smoke 35.2 s, telemetry 25.9 s (the
determinism target was 56–58 s at §D.22).

**The mechanism bed** (`a_super_billed_design_is_synthesized_traded_and_built_with_mass_conserved`,
3 seats, 200 planets, 1,500 yr, regrowth off, every seat's colonizer and miner
Designs billed 25% Red — the state a tier-3 Design write leaves): 3.04 kt of
Red synthesized, 393 hulls built carrying Red, mass conserved to 1e-9. Two
defects found on the way, both fixed before landing:

- **A blocked Design stopped the yard.** A seat whose colonizer owed Red it
  could not get chose that order at every decision, declined it, and built
  nothing else — its banks did not move for centuries. Fixed by quoting a
  Design the center cannot pay as unpayable and choosing again (§10.4).
- **Quoting hid the demand.** With the Design quoted unpayable up front, the
  center never ordered it, never declined, and recorded no need: 0.03 kt made,
  one Red hull, nothing for freight or the Exchange to answer. Fixed by
  choosing on plain prices first and recording the need when the choice is
  blocked; needs are re-recorded at each decision so a center that stops
  wanting the order stops bidding for it.

No Red was traded in that bed: every homeworld held some Magenta and Yellow,
including the two archetypes whose native super is not Red, so each forge made
its own (technology §3.2's "exactly one" is a gradient as built; R-G4). Trade is
pinned by `a_forge_sells_the_super_it_can_make_and_makes_it_at_settlement`.

## D.24 T-139 — missiles, point defense, sentries and the supply line

*Supports technology §9, warfare §8.22 and matching §10.5. Bed: a scratch
harness, never landed, 3 seats, cards at the shipped first barrier (200 yr),
regrowth off where the mass ledger is read. All figures are single runs on one
seed unless stated.*

**The magnitudes, priced before tuning.** A Limited Offensive hull has dry mass
0.020 kt, 0.910 g empty and structure 1.94·10¹⁰ kJ; a Cairn has 4.10·10¹⁰. One
50 MW beam fires to 7.9e-3 ly against a station-keeping hull. A round from the
LOU reaches 0.0238 ly at burnout under the arena's multiplier and burn time,
three times as far.

**Defect 1: rearming by the parked test.** At first a hull counted as "at a
center" only while parked. A sentry under way during its yard delay was
therefore sent to rearm at its own center: zero-length legs, at one instant.
Seed 1, missile card on seats 0 and 2, Warfare card on seat 1: **1,093,717
events by 425 yr, with 1,158 returns to rearm**, against 38,133 events at
300 yr. Deciding "at a center" by position took the returns to 0.

**Defect 2: point defense absorbed every round.** With one round per tube per
salvo and a round's structure at one discharge (10⁹ kJ), **3 of 9,607** rounds
hit. A round spends about 2.9 days inside a beam's point-defense range, and one
mount stopped one round per 0.25-day discharge. Two placeholders changed: a
salvo is the arena's burst of 4 per tube, and a round's structure is four
discharges (4·10⁹ kJ). Then a lone beam hull stops about 3 rounds a window, and
a stack stops many.

**Loop 1: blockaders rebuilt for a port a sentry guards.** Census of the
logged events, 340–390 yr, same seed: **5,117 pickets spawned, 5,055 changed
course and 5,340 parked as reserve** on the Warfare seat. A blockader that sees
a sentry it can outrun withdraws (R-WAR26's third ending). `picket_first` then
read the port as uncovered and built another. Fix: the seat records ports where
a sentry turned a picket away, and blockades them no more. Wall time at 800 yr
went from more than 600 s (the run was stopped) to 154 s.

**Loop 2: sentries fed to blockade stacks.** Census, 750–800 yr: **538
sentries spawned**. Stacks standing at the port wrecked each sentry, and the
center replaced it. 32.5 kt of rounds were fabricated against 1.25 kt fired;
the rest went down with the sentries. Fix: a center counts the sentries it has
ordered. Then 875,000 events and 19.2 s at 800 yr.

**Detection narrowed to what a sentry fires on.** An armed hull starting a
trajectory scanned every hull in the world, and a sentry fires only on armed
ones. The sentry's own scan and the scans that consult sentries now walk only
armed hulls (`open_fire`). Missile stats and colony counts were identical before
and after; the missile-only arm went 7.7 → 7.4 s, single runs.

**Proportional sentries (the author's ruling).** Holding plus population per
center, seed 1, card-free:

| at | centers | p10 | p50 | p90 | p99 | homeworlds |
|---|---|---|---|---|---|---|
| 300 yr | 303 | 1.8 kt | 21.1 | 240 | 6,447 | 2,547–3,904 |
| 800 yr | 3,057 | 17.8 kt | 371 | 5,198 | 47,952 | 1.90–1.98 M |

At `ρ = 1e-5` and 0.030 kt per sentry, a center orders one per 3,000 kt. That
is about one at a homeworld at 300 yr and about 660 at 800 yr. The sweep is
T-140.

**Arms at 800 yr, seed 1, sequential runs:**

| arm | wall | events | colonies (seats 0 / 1 / 2) |
|---|---|---|---|
| no card | 6.6 s | 384,448 | 936 / 1,018 / 1,103 |
| missile card, seat 0 | 6.5 s | 386,441 | 935 / 1,019 / 1,103 |
| Warfare card, seat 1 | 12.0 s | 2,740,602 | 996 / 1,127 / 1,528 |
| missile, Warfare, missile | 19.4 s | 3,187,036 | 991 / 1,148 / 1,496 |

**Reach census, 4 seeds** (1, 7, 42, 31337; 3 seats; 800 yr; the Warfare card
on seat 1 in both arms, the missile card on seat 0 in one; runs concurrent, so
no wall times):

| seed | rounds launched | hit / stopped / missed | seat 1 hulls wrecked by seat 0 | seat 0 colonies, missile arm − Warfare-only arm |
|---|---|---|---|---|
| 1 | 306 | 27 / 229 / 50 | 4 | 967 − 996 = −29 |
| 7 | 135 | 31 / 78 / 26 | 3 | 996 − 1,006 = −10 |
| 42 | 170 | 30 / 96 / 44 | 5 | 847 − 889 = −42 |
| 31337 | 172 | 34 / 103 / 35 | 4 | 1,414 − 1,448 = −34 |

Mean difference **−28.8 ± 6.9** colonies (SE over 4 seeds), 4/4 down. Colony
count is Expansion's metric. This is recorded as a cost the card's seat paid at
these placeholders, not as the card's value; nothing reads Technology's own
metric (R-TECH1).

**Bit-identity, card-free.** `examples/build_digest` at 1,500 yr on seeds 1, 7,
42, 31337, 2, 3, 5, 11, against the pushed head `1e9aec6`: event counts, colony
counts and population bits identical on 8/8.

**Mass.** With regrowth off at 800 yr the ledger moves by −1.2e-5 kt card-free.
With cards it moves by −0.5 kt (Warfare), −0.8 kt (missile) and −2.1 kt (both,
three plays): exactly the card prices paid, and nothing else.

**The determinism arm.** `combat_runs_are_bit_identical` now rotates Warfare,
Growth and missile cards over six seats. At 300 yr seed 1 resolves **no** round,
because proportional sentries are few that early. Probed: 350 yr gives 7 and 90
rounds on seeds 1 and 7 (14.1 s); 400 yr gives 13 and 178 (22.1 s); 450 yr
gives 13 and 178 (27.2 s). 350 yr ships. The whole target is 54.2 s.

**Test targets after the landing:** unit 5.9 s (295 tests), determinism
54.2 s, smoke 35.7 s, telemetry 27.5 s.

**After the author's rulings on R-WAR45 and R-WAR46.** Everything above in
this entry was measured under the earlier rules: point defense decided at
impact, outside the beam's firing cycle, and sentries counted as ordered. Two
changes followed the rulings.

- **Point defense commits a beam's mounts** from when a round enters range
  until it is destroyed, and the beam's discharges in that interval deliver
  nothing at hulls. `point_defense_takes_a_beam_off_its_target_while_it_defends`
  pins it: no damage on the dueled hull inside the interval, and damage again
  after.
- **Sentries are counted standing or ordered**, so a loss leaves a place that
  a priced build fills.

Same bed as the arms table (seed 1, 3 seats, 800 yr, sequential single runs):

| arm | wall | events | sentries spawned, 750–800 yr | seat 0 hulls wrecked by seat 1 | colonies (seats 0 / 1 / 2) |
|---|---|---|---|---|---|
| missile card, seat 0 | 6.0 s | 386,222 | 261 | 0 | 930 / 1,021 / 1,103 |
| missile, Warfare, missile | 43.7 s | 3,683,069 | 3,147 | 4,800 | 817 / 1,189 / 1,512 |

Rounds in the second arm: 326 launched, 73 hit, 170 stopped, 83 missed.
Against the ordered-count rule on the same arm, seat 0 has 174 fewer colonies
(817 against 991) and spawns 3.8x the sentries. That is the rebuild loop of
"Loop 2" above: the price gate does not bind, because a sentry costs 0.03 kt
against banks of thousands. One seed, so this is a reading and not an
estimate. It is the measurement behind R-WAR47.

Card-free runs are bit-identical to `539389d` on seeds 1, 7, 42, 31337, 2, 3, 5
and 11 at 1,200 yr. Test targets on this session's machine: unit 4.5 s (298
tests), determinism 41.8 s, smoke 25.1 s, telemetry 21.9 s. That machine ran
faster than the one above, so the two sets of times are not comparable.

**R-WAR47: the loss price, set by Monte Carlo** (the author's ruling). A
center that has lost `L` sentries orders as many as its budget buys at
`c_s · (1 + κ · L)`; the sweep chooses `κ`
(`Doctrine::sentry_loss_price`). Harness: `examples/sentry_price_sweep`.
Bed: 3 seats, seat 0 plays the missile card and seat 1 the first Warfare
card at the 200-yr barrier, seat 2 none, 800 yr. The only thing varied is
seat 0's `κ`. Objective: seat 0's colony-years and work-years, each divided
by the same seed's `κ = 0` arm (common random numbers), and their geometric
mean.

*The first sweep counted only sentries wrecked as sentries, and every arm
was bit-identical.* `L` was 0 on seeds 1 and 7 with 5,589 and 2,472 sentries
ordered. A census of the last 50 years on seed 1 found the path the count
missed:

| log row, 750–800 yr, seed 1 | count |
|---|---|
| sentry withdrawals (`CourseChanged`, role Sentry, reason Withdraw) | 1,922 |
| Reserve hulls wrecked | 1,928 |
| sentries spawned | 2,115 |
| sentries wrecked as sentries | 0 |

A sentry fired on by a neutral it can outrun withdraws (R-WAR26), stands down
to Reserve at its own center, holds fire on neutrals and is wrecked there.
The counter read the role after the re-role. `L` now counts a sentry leaving
its post by either exit (`leave_missile_post`, reached only from a wreck or a
withdrawal, both answers to fire). R-WAR48 asks whether the withdrawal
should happen at all.

*Sweep, after the fix.* Mean over seeds of the per-seed change against
`κ = 0`, ± one standard error. Seeds 1, 7, 42, 31337 are the standard bed;
2, 3, 5, 11 are the replication set.

| `κ` | colony-years, standard 4 | work-years, standard 4 | colony-years, replication 4 | work-years, replication 4 |
|---|---|---|---|---|
| 0.03 | −0.06% ± 0.16 | +0.06% ± 0.41 | — | — |
| 0.1 | +0.91% ± 1.03 | +2.56% ± 1.50 | +4.18% ± 2.25 | −1.45% ± 1.07 |
| 0.3 | +2.29% ± 2.53 | −0.26% ± 1.67 | — | — |
| 1 | +3.00% ± 2.55 | +2.06% ± 2.56 | +5.54% ± 2.77 | −3.11% ± 0.91 |
| 3 | +3.84% ± 3.12 | +1.65% ± 2.55 | +5.45% ± 2.71 | −1.91% ± 1.05 |
| 10 | +4.38% ± 3.59 | +2.50% ± 2.28 | +5.90% ± 2.94 | −2.47% ± 1.21 |
| 30 | +4.59% ± 3.60 | +2.38% ± 2.32 | +5.87% ± 2.94 | −3.26% ± 0.80 |
| 100 | +4.55% ± 3.61 | +2.38% ± 2.33 | +5.89% ± 2.94 | −3.26% ± 0.80 |
| 1000 | +4.55% ± 3.61 | +2.38% ± 2.33 | +5.88% ± 2.94 | −3.26% ± 0.80 |

Pooled over all eight seeds:

| `κ` | colony-years | work-years | geometric mean | seeds positive | seat 1's colony-years | events |
|---|---|---|---|---|---|---|
| 0.1 | +2.55% ± 1.30 | +0.55% ± 1.14 | +1.51% ± 0.71 | 7/8 | −1.12% ± 0.66 | −7.2% ± 2.2 |
| 1 | +4.27% ± 1.81 | −0.53% ± 1.59 | +1.80% ± 1.28 | 5/8 | −1.43% ± 0.65 | −9.3% ± 2.8 |
| 3 | +4.65% ± 1.94 | −0.13% ± 1.44 | +2.19% ± 1.37 | 6/8 | −1.17% ± 0.65 | −10.0% ± 3.4 |
| 10 | +5.14% ± 2.17 | +0.02% ± 1.52 | +2.50% ± 1.46 | 6/8 | −1.17% ± 0.66 | −10.5% ± 3.3 |
| 30–1000 | +5.2% ± 2.2 | −0.44% ± 1.56 | +2.31% ± 1.51 | 5/8 | −1.20% ± 0.69 | −10.6% ± 3.4 |

Sentries ordered per run fall from 3,782 at `κ = 0` to 548 at `κ = 3` on the
standard four, and losses from 2,847 to 83. Wall time per run on the
standard four, uncontended: 37.8 s at `κ = 0`, 20.0 s at `κ = 3`.

*Reading.* The sweep separates `κ = 0` from `κ > 0`: colony-years rise by
2.4 standard errors at every `κ ≥ 1`, and work-years do not move at any.
It does not separate the values of `κ` from 1 to 1000, which differ by less
than one standard error; above 10 the arms converge, because a center then
replaces almost nothing after its first loss. Most of the standard bed's
gain is seed 1 (+15.1% colony-years at `κ = 10`); the replication set gains
on three or four of four seeds at every `κ ≥ 0.1`. `κ = 3` ships: it sits
inside the plateau and short of the limit where a loss ends replacement,
which would turn R-WAR46's pricing into a ban. It is a choice of a point in
a plateau the sweep measured, not an optimum it found. **The author ratified
`κ = 3` after PR #12 merged.**

Card-free runs are bit-identical to `0887203` on seeds 1, 7, 42, 31337 at
1,000 yr (events, every seat's stocks, the mass ledger). Test targets, this
session's machine, timed beside `0887203`'s binary: determinism 67.9 s →
63.3 s, smoke 44.3 s → 42.6 s; unit 7.1 s (300 tests), telemetry 32.6 s.
The determinism target is over 60 s on both binaries on this machine and
inside the 72-s band; it was 41.8 s on the previous session's.

## D.25 R-MX10 — synthesis and refined trade, confirmed in a run

*Supports matching §8.7, §9.6 and §10.6. Bed: the §D.23 mechanism bed (3 seats,
200 planets, 1,500 yr, regrowth off, test configuration), every seat's
colonizer (`Delta`) and miner (`Meadow`) Designs billed 25% refined — the state
a tier-3 Design write leaves — on seeds 11, 3, 7 and 42. A scratch census test,
never landed, read the synthesis log, the refined books and every contract.*

**Arms.** Red: 25% Red. R+G+B: 25% split equally over the three supers. Apex:
25% apex. Non-native: each seat billed in another archetype's native super.
Lockout: the R+G+B and apex arms on the same galaxy with seat 2's homeworld
ceiling at `Band 3.5`, so it can never reach population `Band IV` and forge —
a galaxy variant, which is what a bed may vary (T-133 ruling).

**Synthesis and building, 24 of 24 runs.** Every seat at `Band IV` makes what
its bills owe, and the yard builds hulls of it:

| arm | refined kt made per seat (seed 11) | hulls carrying refined (4 seeds) |
|---|---|---|
| Red | 1.97 / 0.66 / 0.62 Red | 291–428 |
| R+G+B | 0.28 / 0.37 / 0.24 of each | 335–464 each |
| apex | 1.58 / 0.77 / 0.41 apex, and the supers it draws | 319–418 |
| non-native | 0.83 / 0.64 / 0.63 of the billed super | 70–299 per super |
| lockout, R+G+B / apex | seat 2 makes nothing | 253–406 / 272–364 |

First synthesis at 605 or 655 yr on every seat that can forge. Mass drift at
most 9.8e-16 of the total.

**Trade, almost none.** Refined kilotonnes delivered between empires: 0.0050
(seed 11, Red), 0.0369 (seed 7, R+G+B), 0.0251 (seed 7, R+G+B, lockout), and 0
in the other 21 runs. Refined bids posted per run 22–255; asks 0–9. Seat 2 in
the lockout arms received refined material in 1 of 8 runs.

*Mechanism, instrumented at posting.* A forge offers capacity from basics
above its next works bill. At every barrier, on every seed, each forge held
less than that bill in at least one basic — a typical forge 0.3 / 27 / 14 kt of
Cyan / Magenta / Yellow against a bill of 260 / 130 / 390 kt — so its capacity
was zero. Apex is never offered: capacity asks cover the three supers only,
and a forge synthesizes only what it owes. Every homeworld holds all three
basics, so every forge makes every super from its own holding, and the only
window with one forge and no other (605–655 yr on seed 11) contains no round
barrier. Open as R-MX16.

**A defect found and fixed.** A capacity ask was sized per precursor against
that color's share of the works bill; settlement (`synthesis_plan`) keeps the
whole bill as a total. On seed 11 at 1,000 yr seat 1's forge held 8.163 /
7.584 / 0.028 kt against a bill of 6.333 / 3.167 / 9.500 kt and offered
2.43 kt of Blue; its total, 15.78 kt, was below the 19.0 kt bill before any
draw, and both Blue contracts struck from the ask defaulted at settlement. The
ask is now capped by the total above the whole bill, and each super's draw
comes off its precursors' room before the next super is counted (Red and Green
share Yellow). `a_forge_offers_only_capacity_it_can_settle` fails on the old
rule (2.11 kt of Red offered that settlement cannot make) and passes on the
new. Card-free runs are bit-identical to `e3e87d0` on seeds 1, 7, 42 and
31337 at 1,000 yr: a card-free game posts no refined bid, so no ask clears.

## D.26 R-MX16 priced, the trio homeworld, and hex-scale color

*Supports matching §10.6, galaxy §3, §4.3 and §4.5, industry §1.7. Beds: the
§D.25 census (3 seats, 200 planets, 1,500 yr, Designs billed 25% refined, 4
seeds × 6 arms) for trade; `examples/work_years` (3 seats, standard galaxy, 4
seeds, 1,500 yr) against `1e87f9d` for the card-free economy; a scratch slant
census over 10-ly cells (never landed).*

**The share of supers forged through trade** — refined kilotonnes delivered
between empires over refined kilotonnes synthesized, pooled over 24 runs:

| engine | share | runs above 0 |
|---|---|---|
| works bill held (§D.25) | 0.05% (0.07 of 128 kt) | 3 |
| works bill priced | 0.71% (0.91 of 129 kt) | 12 |
| + hotspots aligned to homeworlds (not landed) | 0.36% (0.69 of 188 kt) | 8 |
| + homeworld poor deposit at trace (not landed) | 0.45% (0.85 of 188 kt) | 9 |
| priced + hex-scale color + trio (landed) | **0.44%** (0.26 of 58 kt) | 2 |

*Mechanism, measured.* A forge synthesizes any super whose basics its holding
has, and an empire's freight brings every basic to its centers for its works.
At the barriers on the aligned-hotspot galaxy a homeworld held a median 8 kt
of its poor basic (1.7 kt with the trio) and its empire a median 1,546 kt
(639 kt), against 0.3–3 kt of super demand per seat per run. Each forge
supplies its own empire's shortfall before it offers, and refined trade clears
only at a barrier — twice per run after the first forge at 605–655 yr. No
geography that leaves every hue within an empire's reach changes this; a
hard rule on what a forge may make would (R-G4).

**Hex-scale color.** Share of 10-ly cells whose top color holds 80% of the ore
(one color) or whose top two hold 90% (two), 4 seeds:

| field | one color | two colors | neither | ore in slanted cells |
|---|---|---|---|---|
| three hotspots (before) | 36.0% | 14.9% | 49.0% | 99.9% |
| color sites, no floor | 33.4% | 15.0% | 51.6% | 99.7% |
| sites, one noise draw per world, no floor | 21.7% | 1.7% | 76.7% | 99.8% |
| sites + floor 0.15 | 39.2% | 4.6% | 56.2% | 99.7% |
| sites + floor 0.3 | 66.0% | 5.6% | 28.4% | 99.7% |
| **sites + floor 0.5 (landed)** | **79.4%** | **5.3%** | **15.3%** | 99.9% |

By ore mass the old field already slanted (one rich world dominates a cell);
by cell count half the cells had no slant because the envelope leaves most of
the disk with almost no ore in any color. The floor gives every site its hue.
12 seats reads within a point of 3 seats in every row.

**Card-free economy** (paired by seed against `1e87f9d`): work-years +16.36% ±
11.66 (not resolved, seeds −10.6% to +43.3%), colony-years +0.19% ± 0.46,
colonies +1.24% ± 0.54, vehicles +19.8% ± 1.9, yr/s −2.45% ± 0.68 with
`ns/event` −0.03% ± 0.93 — the simulation does more, each event costs the
same. The opening is later: `empires_expand_beyond_the_homeworld` founds no
colony by 80 yr and one by 100 yr on seed 42 (it now runs 120 yr).

**A defect the trio exposed.** `endowment_minerals` priced the destination's
build-out (industry §1.7, `I* = (K_c + D)/2`) on the founding center; a
homeworld's own deposit had kept it positive. It now prices the target. On the
card-free bed above it is inert: the runs are identical to the last event with
and without it.

**Two allocation defects of the priced ask, found by its test.** Counting the
supers' capacity in sequence left at most one basic for apex, so apex was never
offered; each basic is now shared in thirds among the two supers and apex that
draw it. `a_forge_offers_what_its_standing_order_leaves_at_its_price` pins the
thirds, the settlement of every offer together, and the standing order's hold.

## D.27 Color theory: 1:1 recipe-pair sites, and slant by absolute threshold

*Superseded in generation by §D.28* — the author: "1:1 is wrong for galaxy
generation." Pair sites are removed; the forge's 1:1 recipe rule (galaxy §4.2)
and slant by absolute threshold stand. The record below is of the pair-site
galaxy.

*Supports galaxy §3, §4.2 and §4.3, technology §3.2. The author's rulings:
forges produce supers by color theory, two basics 1:1; slant is measured by an
absolute threshold per mineral, traces not counting. Beds as §D.26; the slant
census is a scratch harness over 10-ly cells and single worlds, 4 seeds,
never landed.*

**Generation.** A color site draws two hues in proportion to the hotspots'
weights; the same hue twice is a primary site, two hues a pair site holding
both basics of a recipe at one peak, so a world beside it holds them 1:1.

**Slant by absolute threshold** (mineral present at ≥ the threshold; share of
cells, 3 seats; 12 seats within a point):

| threshold | field | none | one | two | three |
|---|---|---|---|---|---|
| `Band I` (1.0 kt) | single-hue sites (`6d751d5`) | 41.6% | 48.6% | 8.5% | 1.3% |
| `Band I` | **pair sites** | 41.5% | 40.9% | **14.5%** | 3.0% |
| `Band II` (31.6 kt) | single-hue sites | 80.3% | 17.6% | 1.9% | 0.1% |
| `Band II` | **pair sites** | 80.9% | 13.8% | **4.6%** | 0.7% |

Single worlds at `Band I`: two minerals present on 13.0% (4.5% before), three
on 1.4% (0.2%). About 41% of cells hold no mineral at `Band I` in either field
— their worlds carry traces only, far from a site or high above the midplane.

**Super trade:** 0.25% of refined mass synthesized crosses between empires
(0.22 of 90.5 kt, 24 runs).

**Card-free economy against `6d751d5`** (4 seeds, 1,500 yr, paired by seed):
work-years **+22.74% ± 4.84** (4/4 up), colony-years **−6.66% ± 1.79** (4/4
down), colonies −4.97% ± 0.58, vehicles +8.4% ± 2.0, `ns/event` +7.2% ± 1.9.
*Inference, not tested:* a pair world gives one center two colors from one
deposit, so the three-color works bill is payable more often and deepening
wins against founding more often. A census of payable works bills with and
without pair sites would settle it; open.

**The determinism combat arm moved to seeds 1 and 2 at 450 yr**: on this
galaxy seed 7 launches no missile round at 500 yr (nor do 5 and 17 of 8 seeds
probed), while seeds 1 and 2 launch 92 and 47 at 450 yr.

## D.28 A world's total ore is its richest color

*Supports galaxy §4.3. The author's ruling: limit a world's total minerals to
the largest across its colors, shared in the rolled proportions. Beds as §D.27,
against the single-hue galaxy of `6d751d5` and the pair-site galaxy of
`4bdd26d`.*

**Slant at `Band I` (1.0 kt)**, mineral present at ≥ the threshold, 3 seats,
4 seeds (12 seats within a point):

| galaxy | worlds: none | one | two | three | cells: none | one | two | three |
|---|---|---|---|---|---|---|---|---|
| single-hue sites (`6d751d5`) | 57.1% | 38.3% | 4.5% | 0.2% | 41.6% | 48.6% | 8.5% | 1.3% |
| pair sites (`4bdd26d`) | 57.3% | 28.4% | 13.0% | 1.4% | 41.5% | 40.9% | 14.5% | 3.0% |
| **single-hue + cap** | **58.0%** | 38.3% | 3.7% | 0.1% | 42.1% | 48.9% | 7.8% | 1.2% |

At `Band II` (31.6 kt): worlds with none 90.8% (90.7% before). The cap moves
worlds with no `Band I` mineral by +0.9 points: most worlds were already
dominated by one color, where the cap is nearly the sum.

**Card-free economy against `6d751d5`** (4 seeds, 1,500 yr, paired):
colony-years **+1.68% ± 0.48** (4/4), colonies +1.29% ± 0.11, work-years
−0.90% ± 4.08 (not resolved), `ns/event` −2.09% ± 0.46. Against the pair-site
galaxy: colony-years +9.04% ± 1.73, work-years −19.17% ± 2.23. *Inference, not
tested:* capped worlds hold less ore, so §4.4 leaves a few more of them
habitable.

**Super trade:** 0.27% of refined mass synthesized crosses between empires
(0.20 of 74.4 kt, 24 runs; 18 runs at 0).

**The determinism combat arm moved to seeds 5 and 9**: of seeds 1–12, only 5,
8, 9 and 11 launch a missile round by 450 yr on this galaxy (38, 7, 105, 7).

## D.29 Combat bit-identity as three tests over six sets of initial conditions

*Supports `AGENTS.md` §4 (determinism) and galaxy §3.1. The author's ruling:
make the combat determinism coverage several tests with different initial
conditions. Replaces `combat_runs_are_bit_identical`, whose seeds had to be
re-picked after each of §D.24, §D.27 and §D.28 because galaxy changes moved
which seeds launched a missile round at all.*

Each test runs its scenario twice and compares every combat log record (time
bits and event), the event count, and each seat's colonies, outposts and
population bits. Counts from one run per scenario, release build:

| test | scenario | encounters | wrecks | course changes | rounds hit / stopped / missed | refilled at a center |
|---|---|---|---|---|---|---|
| `beam_fights_are_bit_identical` | Cairn and Tor stacks parked together, seed 31, 3 yr | 800 | 20 | 20 | — | — |
| | Tor stack closing at 0.3 c from 0.03 ly, seed 32, 3 yr | 800 | 20 | 22 | — | — |
| `missile_defense_is_bit_identical` | Butte sentries vs a parked Cairn stack at 0.015 ly, seed 41, 3 yr | 118 | 10 | 14 | 37 / 53 / 271 | 44 |
| | vs a Tor stack closing at 0.2 c from 0.045 ly, seed 42, 3 yr | 100 | 10 | 10 | 10 / 46 / 254 | 36 |
| `a_card_play_game_is_bit_identical` | 6 seats, 600 planets, cards 15/3/13, seed 1, 350 yr | 3,334 | 423 | 1,906 | 0 | 2 |
| | same, seed 7 | 4,505 | 504 | 1,996 | 0 | 2 |

Two engine additions to fleet seeding were needed (galaxy §3.1): a **Sentry**
fleet stands at its seat's homeworld as the center's sentries, and a generated
missile Design starts with a full magazine.

**Not reached by any seeded bed:** the ammo run and the flight home. Both
start from a post or a voyage, and a generated picket has neither. Probed
before this was settled: Mesa pickets against an armed stack hold fire (one
picket's burst does not exceed the stack's point defense, and a postless
picket counts no co-located rounds); against unarmed Meadow and Delta targets
they fire 8–16 rounds and the targets are wrecked or withdraw before any
magazine empties (40 and 80 yr). Sentries fire only on armed hulls, so an
unarmed target draws no fire from them. The unit test
`a_dry_missile_picket_is_resupplied_by_ammo_run_or_by_return` covers the two
paths outside bit-identity.

**Cost:** determinism target 43.1 s debug, one run, against ~43 s before the
change (one run each; not resolved).

## D.30 Forging is a forge's purpose

*Supports galaxy §4.5 and matching §8.7, §10.4, §10.6, §10.7. The author's
ruling: forging is a high-priced activity, outweighing almost anything but
immediate survival; once a center clears population `Band IV` its primary
purpose is to forge supers and apex. Bed: `examples/forge_census` — card-free,
standard galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337 — run on this engine
and on `a50ef75` (common random numbers, paired by seed and seat).*

**What it replaced.** Synthesis ran on demand only: when an order a forge was
paying owed a super it lacked, when a contract it had sold came due (capacity
asks, sold from basics above what its standing order left and priced at the
precursors' willingness to pay over the yield, R-MX16), or when a hauler loaded
at a forge for a sister center. A card-free game posts no refined bid, so a
card-free forge synthesized nothing: every seat on `a50ef75` reads 0.00 kt of
every refined material. It was wrong about the forge's purpose, which the
author has now ruled is to forge.

**Economy, paired** (new against `a50ef75`):

| quantity | empire total per seed (n = 4) | per seat (n = 12) |
|---|---|---|
| colony-years | +0.001% ± 0.002 | +0.000% ± 0.013, range −0.08 to +0.09 |
| work-years | +0.66% ± 0.63 | +0.85% ± 0.84, range −2.21 to +9.43 |
| basic kt delivered between empires | +0.45% ± 0.22 | — |

None of the three is resolved at two standard errors (estimates). *Inference:*
a homeworld that stops building at 610–845 yr costs nothing measurable because
colonies carry the expansion loop by then; the mechanism is not instrumented.

**Forge output.** First synthesis at 610–845 yr on every seat. Supers forged
per seat 57.2–6,009.9 kt (7,495.4 kt over 12 seat-runs), apex 3,090.4 kt.
Every forge makes all three supers. The seat's native super is 4.7–93.0% of its
super mass (median 22.7%, pooled 30.9%): a forge makes whatever pair its
freight and the Exchange bring, not its archetype's. **No super or apex is
delivered between empires on any run**: a forge bids for no super, and no
center posts a refined bid card-free (R-MX18).

**Super-billed Designs.** On §D.25's bed (Delta and Meadow billed 25% Red,
200 planets, seed 11, 1,500 yr) each seat holds 1–4 centers; the forges make
Red and no Red-billed hull is built, because the forge no longer builds and no
hauler carries its Red to a colony within the horizon (a scratch census, never
landed: Red held outside the forge 0.00 kt at every century). Before, the
forge built 291–428 such hulls itself (§D.25). Open as R-MX17;
`a_super_billed_design_is_forged_with_mass_conserved` now pins forging and
conservation only.

**Test targets after the change:** unit 5.2 s (302 tests), determinism 43.1 s,
smoke 12.4 s, telemetry 26.5 s (debug, one run each).

## D.31 Forges build super-billed Designs and bid for supers; decisions without a cadence

*Supports galaxy §4.5, matching §8.7 and §10.7, autopilot §6.1a. The author's
rulings: a Design paid in supers is priced higher than forging (R-MX17); forges
bid on the supers they have demand for (R-MX18); no decision has a cadence of
its own; each decision is a tree with a short circuit for the common case, and
some are conditioned on an event. Bed: `examples/forge_census` (card-free,
standard galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337).*

**R-MX17.** On §D.25's bed (Delta and Meadow billed 25% Red) the forge's first
tick had turned its whole stock into apex (0.50 kt of each basic into 0.495 kt
apex at 600 yr) before its yard's next decision at 605 yr, and the
homeworld's income afterwards rounds to zero, so no Red-billed hull was built.
Two defects stood behind it, each found by tracing the decision:
`ProductionContext::price_of` keys on the hull, so a sentry read the picket's
price and a Red-billed miner read the scout's, both priced out at a forge; and
forging ran ahead of the yard. The yard's quote now reads the Design's own
price (`design_price`) and the forge runs after the yard. The bed builds
Red-billed hulls again (`a_super_billed_design_is_forged_and_built_with_mass_conserved`).

**The retry floor.** Moving the forge behind the yard tied forging to T-88's
`decision_retry_years = 50`, which the author ruled is not design. Deleted.
Asking every center with a free berth on every tick doubled the cost of seed 1
(12.3 s → 25.2 s, events +0.9%). Census of 773k decisions on seed 1: 726,129
exited before the scan (bank below every price, or empty pool), 19,228 scanned
and declined over 7.36 M candidates, 13,644 scanned and committed. Callgrind
(800 yr): `commit_one_build` 63% of instructions, `view_of` + `rank` 41.5%. Of
the scanned declines, 17,470 held the price of a colonizer and a mining pair:
the policy preferred deepening and the rung lacked a color, so it waited on a
color. Conditioning every decline on every scan woke 206,695 of 210,500 asks at
800 yr (scouts add a world on almost every tick; nearly every scanned world can
rank as a mining outpost at full pressure). Conditioning a saving decline on
money, level, works and cards only skipped 197,772 asks and let 9,042 through.

| engine, seed 1, 800 yr | time |
|---|---|
| 50-yr retry floor | 4.9 s |
| every tick | 7.5 s |
| conditioned on events | 5.0 s |

Over 4 seeds at 1,500 yr, conditioned against every tick: colony-years
−0.24% ± 0.04 (4/4 lower), work-years +6.07% ± 4.45 (not resolved). Against
`a50ef75`: colony-years +0.12% ± 0.14, work-years +0.92% ± 4.06 (neither
resolved).

**R-MX18.** Supers delivered between empires: 271.9 / 247.1 / 228.2 / 63.4 kt
on seeds 1 / 7 / 42 / 31337 — 810.6 of 22,040 kt forged (3.7%), against 0
before forges bid. A seat's native super is 29.6% of what it forges, pooled
(per seat 0.4–95.9%, median 17.3%). The recommended ask at the precursors' cost
was not adopted: it equals an ordinary center's bid for a super and never
clears once the bid is discounted by transit, which a unit test showed on the
first run.

**Test targets:** unit 4.0 s, determinism 35.9 s, smoke 10.6 s, telemetry
23.2 s (debug, one run each). `shrinking_the_economy_tick_does_not_multiply_decisions`
is retired: it pinned the retry floor.

## D.32 Empire-scale color: the hex width, one color site per hex

*Supports galaxy §1 and §4.3, R-G1, R-G5. The author's rulings: color varies
at the scale of an empire; a modest integer number of hexes per player (3–6 at
3 seats, 6–12 at 6 and 12, 3–6 at 18); planet count untouched; a human-scale
hex, about 70 ly a side and about 120 ly across. Beds: `examples/hex_census`
(card-free, standard galaxy, 1,500 yr, seeds 1, 7, 42, 31337), scratch
generation-only probes (never landed), `examples/forge_census` and
`examples/bank_mix` against `82e5579`.*

**Hexes per player.** Counted two ways, with flat-top hexes centered on the
galactic center. *Proxy* — the hexes holding 90% of the worlds nearest each
homeworld, from generation alone: every count falls with seat count at a fixed
width (at 100 ly across: 5.92 / 6.00 / 5.04 / 5.96 per player at 3 / 6 / 12 /
18 seats), so no width can put 3 seats at or under 6 and 6 seats at or over 6.
*Owned territory* — the hexes holding 90% of the worlds each empire owns at
1,500 yr (75–97% of them nearest the empire's own homeworld), measured on the
100-ly field:

| width across flats | 3 seats | 6 seats | 12 seats | 18 seats |
|---|---|---|---|---|
| 100 ly | 7.58 (2/12) | 7.08 (23/24) | 7.46 (44/48) | 7.99 (14/72) |
| 110 ly | 6.58 (5/12) | 5.96 (16/24) | 6.52 (40/48) | 7.08 (23/72) |
| 116 ly | 5.83 (9/12) | 5.58 (13/24) | 6.00 (36/48) | 6.51 (34/72) |
| 120 ly | 5.58 (11/12) | 5.25 (10/24) | 5.75 (33/48) | 6.26 (39/72) |
| 130 ly | 4.83 (12/12) | 4.75 (3/24) | 5.27 (18/48) | 5.61 (57/72) |

Mean per player, and seats inside the target. The proxy ranked 12 seats
lowest; owned territory does not, because empires reach past their own cell.
6 seats need a width of at most ~110 ly, 3 and 18 seats at least ~114 and
~125, so no width meets all four. 116 ly minimizes the worst miss; the author
chose a side of 70 ly (121.2 ly across). Re-measured on the 70-ly field, 3
seats read 5.83 per player at 121.2 ly, 10/12 inside the target.

**Each hue's strongest site at `Band IV`.** With one site per 100-ly hex and
the peak `mineral_peak · (floor + (1 − floor) · w)`, the site that landed
nearest its hotspot held most of a 3-seat galaxy's ore in one hue: 93 / 99 / 95
/ 95% on the four seeds, by mass. Normalizing each hue to its strongest site
gives 53 / 59 / 57 / 79%; the old 10-ly field read 39–59%.

**Color per empire.** Kilotonnes generated on the worlds each empire owns at
1,500 yr: on the old field 9 of 12 seats already held 86–99% of it in one
color, and every empire owned at least 5 worlds holding `Band I` (1 kt) of
every color. On the 100-ly field 4 of 12 seats owned no `Band I` world in one
color. An empire's forge draws on outposts and freight, not only on the worlds
it owns, which is why the old field forged every super everywhere.

**The card-free economy at a 70-ly side**, paired by seed against `82e5579`,
3 seats, 1,500 yr: colony-years **−19.39% ± 1.28** (4/4 lower), work-years
−28.1% ± 11.2 (+4.7% to −45.3%, not resolved). Supers forged 65,284 kt against
22,040; a seat's native super 37.0% of its forging against 29.6%; supers
crossing between empires 5.17% against 3.68%; seats making all three supers
(each at least 1% of the largest) 9/12 against 10/12. Basics traded between
empires fell from 258k–338k kt per seed to 146k–254k. `bank_mix`, seed 1,
800 yr: works purchases 977 → 574, hulls 23,583 → 24,795, payable fraction
median 0.050 → 0.043.

**Why colony-years fell — an inference, not proven.** Worlds with
`k_potential ≥ k_high` (3.2), summed over the four 3-seat galaxies: 13,801 on
the old field, 11,680 on the 70-ly field (−15.4%); mean ore Band per world
0.43 → 0.62–0.69. §4.4's anticorrelation lowers habitability as ore rises, and
the standard bed is limited by the worlds `k_high` admits. Varying only the
site spacing on the new code:

| hex side (= site spacing) | admitted | mean ore Band |
|---|---|---|
| 5.8 ly | 13,560 | 0.456 |
| 11.5 ly | 13,524 | 0.460 |
| 23.1 ly | 13,303 | 0.488 |
| 46.2 ly | 12,478 | 0.561 |
| 70.0 ly | 11,680 | 0.651 |

Why a wider spacing raises the mean ore Band is not established. An arm with
the anticorrelation held at the old field's ore levels would settle how much
of the −19% it carries.

**Superseded at §D.33** (the author: the hex is a human-legible interface, and
"color sites are of course randomly spaced"). One color site per hex made
`hex_side_ly` the color-site spacing, so every number above beyond the
hexes-per-player table measures a 70-ly site spacing with a 61-ly width, not
the hex.

## D.33 Color sites placed at random: spacing and width swept

*Supports galaxy §4.3, T-146. The author's rulings: the hex is read by no
generation; color sites are randomly placed, with their own spacing (one site
per spacing² of area) and width (a Gaussian σ on the Band). The author's
expectation, to be measured: sites spanning more than one 70-ly hex create
trade in supers. Bed: `examples/forge_census` with `FC_SITE_SPACING` and
`FC_SITE_SIGMA` (card-free, standard galaxy, 3 seats, 1,500 yr, seeds 1, 7,
42, 31337), paired by seed against `82e5579` (sites on a 10-ly jittered
lattice, width 5 ly).*

| spacing / width, ly | colony-years | work-years | native share | supers crossing | seats making all three | admitted |
|---|---|---|---|---|---|---|
| `82e5579` (lattice 10 / 5) | — | — | 29.6% | 3.68% | 10/12 | 13,801 |
| 10 / 5 | +3.1% ± 0.6 | −8.0% ± 4.6 | 34.3% | 1.04% | 12/12 | 14,211 |
| 10 / 10 | −16.6% ± 0.3 | +58.1% ± 9.9 | 29.6% | 19.27% | 11/12 | 12,000 |
| 35 / 17.5 | +3.3% ± 2.0 | −34.1% ± 6.6 | 39.5% | 0.89% | 12/12 | 14,263 |
| 35 / 35 | −23.5% ± 2.9 | +40.8% ± 17.1 | 32.4% | 1.69% | 11/12 | 11,342 |
| 70 / 35 | −0.9% ± 4.7 | −46.4% ± 7.5 | 31.0% | 5.34% | 10/12 | 13,814 |
| 70 / 70 | −36.6% ± 4.5 | +128.7% ± 56.7 | 33.0% | 3.33% | 12/12 | 9,914 |
| 140 / 70 | −13.6% ± 2.6 | −56.9% ± 10.0 | 65.8% | 0.33% | 10/12 | 12,319 |
| 140 / 140 | −55.2% ± 1.8 | +164.0% ± 75.5 | 59.1% | 3.78% | 10/12 | 7,814 |
| 210 / 105 | −10.3% ± 10.0 | −82.0% ± 1.5 | 60.9% | 0.00% | 10/12 | 12,615 |
| 210 / 210 | −50.0% ± 6.4 | −9.7% ± 41.6 | 49.7% | 1.00% | 7/12 | 7,943 |

Colony-years and work-years are paired differences, mean ± one standard error
over 4 seeds. Native share and supers crossing are pooled over the four runs;
admitted is the worlds whose `min(hab, bio)` reaches `k_high` (Band 3.2),
summed over the four galaxies. *Seats making all three:* each super at least
1% of the seat's largest.

**Supers crossing, per seed**, is dominated by the run whose forges make the
most: `82e5579` reads 84.2 / 70.2 / 1.1 / 33.5% on seeds 1 / 7 / 42 / 31337,
and every arm spans similar ranges up to 140-ly spacing. The pooled share is
therefore weighted toward one or two runs, and four seeds do not resolve it.
At 140 and 210 ly, three of four seeds read under 1% in all four arms.

**What moves together.** At width = spacing, sites overlap and the field
carries more ore: admitted worlds fall (to 7,814 at 140/140), colony-years
fall and work-years rise, through §4.4's anticorrelation and the works bill.
At width = half the spacing, admitted stays within 12,319–14,263 and
work-years fall as spacing grows (−8% at 10 ly to −82% at 210 ly). A seat's
native share rises past 140 ly (59–66%) while supers crossing between empires
falls.

**The author's floor: at least 5% of worlds hold two colors** (a color counts
at `Band I`, 1 kt). The shipped 10 / 5 field gives 4.20% at 3 seats and
3.95% at 12; `82e5579` gave 3.65%. Generation only, 3 seats, the four seeds:
a width near 0.55 of the spacing clears it at 10–70 ly (10 / 5.5: 5.37%;
35 / 19.2: 5.67%; 70 / 38.5: 5.61%) and 0.4 at 140 ly (140 / 56: 5.53%),
with admitted worlds 13,345–13,924 against `82e5579`'s 13,801. At width =
spacing the share is 15–34% and admitted falls to 7,814–12,000.

**Two arms at the floor, ore held** (the author chose 70 and 140 ly):

| spacing / width, ly | colony-years | work-years | native share | supers crossing | seats making all three | admitted |
|---|---|---|---|---|---|---|
| 70 / 38.5 | −5.0% ± 5.1 | −41.7% ± 11.1 | 42.1% | 5.11% | 10/12 | 13,345 |
| 140 / 56 | −0.3% ± 2.7 | −71.7% ± 4.3 | 54.9% | 0.92% | 11/12 | 13,725 |

Supers crossing, per seed (1 / 7 / 42 / 31337): 0.0 / 61.2 / 5.6 / 51.3% at
70 ly, 0.0 / 70.0 / 0.8 / 0.0% at 140 ly, against 84.2 / 70.2 / 1.1 / 33.5%
on `82e5579`. With habitable worlds held, colony-years do not resolve from
zero; work-years fall with spacing and a seat's native share rises.

**Inference, stated as one:** at large spacing forges specialize toward one
super and do not trade the others in; the refined books and freight, not the
geography, carry the trade that does not happen. Confidence about 60%: four
seeds per arm, and the ore quantity moves with the spacing in every arm. An
arm holding the admitted count and the mean ore Band at `82e5579`'s while the
spacing grows would separate the two; a census of refined bids and asks that
do not clear at 140 ly would test the inference directly.

## D.34 Demand for supers in the card-free bed

*Supports T-146 and R-M5. The author's question: is there sufficient demand
for supers in the test bed? Bed: `examples/super_census` (card-free, standard
galaxy, 3 seats, 1,500 yr, seeds 1, 7, 42, 31337) on three color fields;
every figure is summed over the four runs.*

**Where super demand can come from.** A refined bid is a center's declined
order's refined shortfall, or a forge's want for the supers that complete a
balanced set for apex (R-MX18). An order is billed in refined material only by
a Design write (`Roster::bill_for` returns basics otherwise), so in a
card-free game every super bid is a forge's, and its only use is apex. Apex
has no consumer in the engine: its book carried no bid at any barrier in any
run.

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| basics mined | 6,257,724 kt | 20,764,703 kt | 27,529,826 kt |
| basics drawn into supers | 93,503 kt (1.49%) | 44,805 kt (0.22%) | 28,950 kt (0.11%) |
| supers forged | 62,335 kt | 29,870 kt | 19,300 kt |
| supers drawn into apex | 48,061 kt (77.1%) | 12,806 kt (42.9%) | 4,981 kt (25.8%) |
| apex made, all held at the horizon | 24,031 kt | 6,403 kt | 2,491 kt |
| supers held at the horizon | 14,267 kt | 17,033 kt | 14,297 kt |
| supers delivered between empires | 651 kt (1.04%) | 1,527 kt (5.11%) | 178 kt (0.92%) |

**The books at the two barriers that follow the first forges** (barriers at
200, 600, 1,000 and 1,400 yr; the first forge stands at 605–765 yr), supers
summed over the three books:

| field | barrier | bid | asked | per-book min(bid, ask) | filled | bid in books with no ask | asked in books with no bid |
|---|---|---|---|---|---|---|---|
| 10 / 5 | 1,000 | 8,708 | 5,935 | 479 | 479 | 5,344 | 0 |
| 10 / 5 | 1,400 | 16,260 | 9,999 | 187 | 172 | 8,018 | 8,022 |
| 70 / 38.5 | 1,000 | 5,316 | 3,272 | 856 | 856 | 3,797 | 964 |
| 70 / 38.5 | 1,400 | 14,026 | 9,775 | 719 | 671 | 9,696 | 5,326 |
| 140 / 56 | 1,000 | 6,553 | 3,850 | 126 | 126 | 5,515 | 2,917 |
| 140 / 56 | 1,400 | 16,662 | 9,807 | 66 | 52 | 15,102 | 6,571 |

Kilotonnes; "per-book min" is taken per seed and book, then summed. The
clearing fills 79–100% of what each book could match. What does not trade is
volume posted where the other side is absent: a forge bids for the supers it
holds least of and asks only the one it holds most of, so where a seed's
forges hold the same super most, that book has asks and no bids and the other
two have bids and no asks.

**Answer to the question: no.** The card-free bed carries no final demand for
supers — no order is billed in them, and the one use, apex, is bid for by no
one — so a measurement of trade in supers on it measures forges completing
sets for a material nothing consumes. A bed in which Designs are billed in
supers through a Design write (as §D.25's 25%-refined bills were) is what can
test it; the refined books also clear only twice after the first forge in a
1,500-yr run, at the 400-yr round cadence.

## D.35 The twin bed: Designs paid in supers, built preferentially

*Supports galaxy §3.1 and T-146. The author's direction: build alternate
test hulls that exactly match the basic hulls but are paid in supers, and build
them preferentially. Bed: `examples/super_census` with `SC_TWINS=1` (twin bill
a third each of Red, Green and Blue; card-free otherwise; standard galaxy,
3 seats, 1,500 yr, seeds 1, 7, 42, 31337), summed over the four runs.*

**First build, and why it carried no demand.** A twin want recorded only at a
decision that chose a hull lasted until the center's next decision, and most
decisions choose no hull: at the barriers the orders wanted 0.06–0.45 kt of
supers in all, on seed 1, and 33 of 23,254 hull orders were paid in supers.
The want now outlives decisions that choose no hull (a center keeps wanting the
twin of the hull Design it last chose), which raises it to ~25 kt per super at
each barrier on seed 1.

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hull orders built | 91,010 (10,858 kt) | 93,250 (13,326 kt) | 92,672 (12,030 kt) |
| paid in supers | 293 (35.4 kt, 0.33%) | 229 (31.2 kt, 0.23%) | 205 (29.6 kt, 0.25%) |
| supers forged | 49,515 kt | 32,329 kt | 15,254 kt |
| drawn into apex | 28,506 kt (58%) | 11,804 kt (37%) | 3,536 kt (23%) |
| supers held at the horizon | 20,214 kt | 18,684 kt | 11,332 kt |
| supers delivered between empires | 725 kt (1.46%) | 1,648 kt (5.10%) | 348 kt (2.28%) |
| orders' want at 1,000 / 1,400 yr | 297 / 300 kt | 246 / 240 kt | 271 / 265 kt |
| supers at forges at 1,000 / 1,400 yr | 9,178 / 17,010 kt | 4,801 / 15,286 kt | 4,926 / 10,279 kt |

The supers forged are 1.3–4.6× all the kilotonnes of hulls built in the run,
and at each barrier the forges hold 16–57× what the orders want; 0.23–0.33% of
hull kilotonnes are paid in supers.

**Inference, stated as one:** within an empire, supers do not move from forges
to the yards that want them. A hauler takes refined material from a center
only as a stop on its own route to the center it serves, and the forges are a
few homeworlds among hundreds of wanting yards. Confidence about 70%; a census
of hauler stops at forges, and of refined kilotonnes delivered to centers by
freight, would test it directly.

**Card-free runs are unchanged**: `forge_census` on seeds 1 and 7 at 800 yr
reproduces `12e59f9` to every printed digit and event count.

**The freight census** (the same bed; `super_census` now reads the refined
part of every freight transfer, `LogEvent::FreighterTransfer::refined`).
Forges are the worlds at population `Band IV` at the horizon — 3 per run, the
homeworlds. Summed over the four seeds:

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hauler pickups at forges | 585 of 328,634 (0.18%) | 905 of 320,533 (0.28%) | 1,236 of 243,139 (0.51%) |
| supers loaded at forges | 36.0 kt | 50.5 kt | 70.0 kt |
| supers loaded elsewhere (rocks, other centers) | 195.3 kt | 372.7 kt | 98.2 kt |
| supers delivered to worlds other than forges | 166.9 kt | 260.2 kt | 156.7 kt |
| hull kilotonnes paid in supers | 35.4 kt | 31.2 kt | 29.6 kt |
| supers held at forges at 1,400 yr | 17,010 kt | 15,286 kt | 10,279 kt |

**Measured:** freight takes 36–70 kt of supers out of the forges over a whole
run, against 10,279–17,010 kt standing at them at the last barrier — under 0.7%.
This confirms §D.35's inference that supers do not reach the yards from the
forges within an empire.

**A second gap, an inference:** 157–260 kt of supers reach yards by freight,
yet 30–35 kt of hulls are paid in them. A twin owes all three supers at once,
and a delivery carries what one forge, rock or Exchange fill held, usually one
super — the same conjunction as a works bill over the three basics (T-91).
Confidence about 60%; the composition of each yard's refined holding when its
twin is declined would test it.


## D.36 Supply runs for supers, and the Growth card on the twin bed

*Supports galaxy §4.5 (supply runs) and T-146. The author's rulings: forges
deliver to their own empire's yards; haulers buy at rival forges; both are
Doctrine. Then: play the Growth card on every seat of the twin bed. Bed:
`examples/super_census` with `SC_TWINS=1`, and `SC_CARD=3` (the Inscrutable
Growth card, `growth_rate` × 1.6, 0.5 kt) played on every seat at the first
barrier, 200 yr; 3 seats, 1,500 yr, seeds 1, 7, 42, 31337, summed.*

**Supply runs, no card.** On seed 1 (10 / 5 ly field): 1,784 runs from own
forges carrying 56.2 kt, 840 to rival forges buying 17.7 kt for 26.6 `$`;
hull kilotonnes paid in supers 11.5 → 17.6. Over the four seeds, 0.33–0.46% of
hull kilotonnes are paid in supers on the three fields. **When hulls are built
is the limit, measured:** on seed 1, 2,345 of 2,812 kt of hulls (83%) are
built before 600 yr, and the first forge forges at 605–765 yr; from 700 yr the
empires build 999 / 1,108 / 1,024 kt of hulls on the three fields and pay
4.3% / 4.2% / 3.0% of it in supers. Card-free runs are bit-identical to
`9a30e39` (`forge_census`, seeds 1 and 7, 800 yr).

**The Growth card on every seat** against no card, same bed and seeds:

| color field (spacing / width, ly) | 10 / 5 | 70 / 38.5 | 140 / 56 |
|---|---|---|---|
| hull kilotonnes paid in supers | 51.7 → 125.4 | 50.6 → 160.4 | 41.2 → 94.5 |
| seeds higher with the card | 4/4 | 4/4 | 4/4 |
| hull kilotonnes built | 11,183 → 11,414 | 13,667 → 13,597 | 12,345 → 12,087 |
| supers forged | 59,931 → 119,252 kt | 32,865 → 93,341 kt | 19,042 → 58,638 kt |
| supers delivered between empires | 724 → 1,733 kt | 1,494 → 3,768 kt | 247 → 1,227 kt |
| supplied from own forges | 224.9 → 290.3 kt | 203.9 → 279.8 kt | 226.2 → 236.1 kt |
| bought at rival forges | 32.0 → 23.4 kt | 39.5 → 16.8 kt | 28.7 → 17.6 kt |
| first forge, per seat | 460–670 yr | 460–540 yr | 455–690 yr |

The card brings the first forge from 605–765 yr to 455–690 yr and raises the
hull kilotonnes paid in supers by 2.3–3.2× on every field; the share of hull
kilotonnes paid in supers is 0.8–1.2%.

**Why forging starts near 600 yr, derived.** A homeworld starts at population
`Band II` (31.6 kt) under a ceiling of `Band 4.2` (2,163,979 kt); a forge needs
`Band IV` (715,542 kt), 22,644× the start. The logistic runs at
`r = growth_rate / rate_reference_years = 0.873 / 50 = 0.01746` per year (a
doubling time of 39.7 yr), so `t = ln[x₁(K − x₀) / (x₀(K − x₁))] / r` =
`ln(33,830) / 0.01746` = **597 yr**. Measured card-free: 605–765 yr. The
author set the 4.2 ceiling for this timing (§D.23: a growth-dedicated build
across before round two's selection at 600 yr, most builds by round three).
With the card `r` is 0.0279, at which the same formula from the starting
population gives 373 yr; played at 200 yr, measured 455–690 yr.

**What spreads the first forge across seats: emigration from the homeworld,
proven by ablation.** Twin bed, card-free, 900 yr, the four seeds (a scratch
harness, never landed). The first forge follows the homeworld's crossing of
`Band IV` by 0–15 yr in every seat, so the spread is population, not basics.
Population at 200 yr runs 17–907 kt against the logistic's ~1,036 kt; every
colony ship a homeworld launches carries 1 kt of its people, and the seats
that sent 174–270 ships before 200 yr cross at 721–761 yr or not by 900 yr,
two of them falling from 39 → 26 kt and 17 → 10 kt between 200 and 300 yr.
Seats that sent 8–148 cross at 605–641 yr. **Ablation:** with a homeworld's
population not debited for the settlers it launches (a scratch build), all 12
seats cross at 601 yr and forge at 600–603 yr, population 952 kt at 200 yr in
every seat.

**Why seats launch so differently — measured as far as the early freight.**
Same bed, 200 yr, the four seeds (scratch harnesses, never landed):

- **Not the neighborhood's colonizable worlds.** Worlds `k_high` admits
  within 25 ly of a homeworld: 46–76 per seat, with no order matching the
  ships launched (seed 1: 64 → 8 ships, 66 → 270).
- **The homeworld's spending.** It launches colony ships out of what it
  spends, and in 10 of 12 seats what it spends in 200 yr is within 3 kt of the
  freight delivered to it (2.7 to 190.6 kt). Seed 1 seat 0 spent 5.7 kt and
  sent 8 ships; the others spent 49–135 kt.
- **Not the ore within 25 ly**, by total or by color: it is at least 98% one
  color in 10 of 12 seats and ranges 32,685–1,028,834 kt with no order
  matching the freight.
- **The first freight home.** Its time runs 37–136 yr and its size 0.30–31.23
  kt; delivered home by 100 yr runs 0–69 kt. The seats with 42–69 kt by 100 yr
  launched 100–270 ships before 200 yr; those with 0–2 kt launched 8–116.
  Traced on seed 1: seat 1 crewed a 932-kt rock 1.2 ly from home with five
  miners at 5 yr and landed 31.23 kt at 70 yr, then put ten new crews out;
  seat 0 crewed its two companions and a 1.3-kt rock one miner each, its
  haulers carried 0.1–0.9 kt a load, mostly to a colony, and the first 0.91 kt
  reached home at 114 yr.

**Inference, stated as one:** the spread is the early economy compounding on
the first outposts — a rich rock in reach of the first decision gets a large
crew and a General hauler (31.6-kt hold, T-98), and a seat without one hauls
in Medium holds (0.91 kt) for its first century. Confidence about 60%. The
hold and crew of each seat's first outposts, and an arm with every homeworld
given one equal rich rock in reach, would settle it.

## D.37 Spread of colony count between empires

*Supports galaxy §2 (the author's target: card-free, a standard deviation of
about 20 colonies between empires) and T-147. Bed: `examples/colony_spread`
(card-free, standard galaxy, 3 seats, 1,500 yr); the population standard
deviation of the seats' colony counts within each galaxy.*

**Random starts** (the shipped generator), 12 galaxies (seeds 1, 7, 42, 31337,
2, 3, 5, 11, 13, 17, 19, 23): 499, 19, 54, 255, 381, 36, 125, 74, 49, 366,
469, 354 — median 190, mean 223. On the first four, the mean is 34 at 200 yr,
135 at 400, 181 at 800 and 207 at 1,500: colony counts are nearly final by
800 yr, so the spread is decided by the race for territory.

**Symmetric starts** (`GalaxyConfig::rotational_symmetry`, now `Ground::ColorRotated`), same 12 seeds: 40,
32, 50, 163, 74, 32, 59, 39, 27, 78, 440, 20 — median 45, mean 88. Seed 19
ends 810 / 1,812 / 967 on identical ground.

**What else differs between seats on identical ground** (scratch arms, never
landed): the default works mix is 2 : 1 : 3 in Cyan, Magenta, Yellow
(`WORKS_MIX_DEFAULT`), so the archetypes' bills differ; an even 2 : 2 : 2 mix
on the symmetric galaxies gives 24, 17, 3, 111 on the first four seeds (mean
39, against 71). The simulation's own seed changes nothing in a card-free run
(seed 31337's galaxy under six simulation seeds: 1,289 / 1,326 / 964 every
time), so what remains is deterministic: seat order in tie-breaks and the
last bits of rotated coordinates.

**Inference, stated as one:** the early economy compounds (appendix §D.36:
the first outposts decide a seat's first-century income), and contested
territory lets an early lead take worlds the others would have taken, so a
small difference grows into a runaway. Confidence about 60%. A census of
contested foundings by seat over time, on a symmetric galaxy, would test it;
meeting the target likely needs a check on that runaway as well as fair
ground.

## D.38 The cause of the spread between empires: an empty hauler's routing loop

*Supports industry §6.11 (freight routed by demand price), galaxy §2 and
T-147. The author's direction: find the cause before fixing symptoms; then the
ruling, pricing based on demand. Beds: `examples/colony_spread` (card-free,
standard galaxy, 3 seats, 1,500 yr, 12 seeds), scratch harnesses and one
scratch ablation (never landed), `examples/forge_census` against `ccca4f4`.*

**Where the spread is decided.** Worlds nearest each homeworld that `k_high`
admits: 1,099–1,243 per seat, near-equal in every galaxy. The five losing seats
of the 12 random galaxies lost 457–769 of their own nearest worlds to
neighbors and had founded 3–9 colonies by 200 yr against 19–151 elsewhere. On
identical ground (seed 19, symmetric) the seats stopped mirroring at 5 yr — a
scout's target, two candidates at nearly equal distance reordered by rounding —
and ended 810 / 1,811 / 967.

**The mechanism, traced (symmetric seed 19).** The two losing homeworlds held
0.01 kt from 20 yr to 170 yr; one mined 636 kt off-world in that time and 2.7 kt
reached it. Its haulers shuttled between a companion and a colony loading
0.000 kt: a hauler loaded against its last destination's color deficit, which
was zero for the companion's color, and then chose its next destination by
completion of shortfall — zero at every center for an empty hold — with ties
broken by entity id, which is a colony's (homeworlds are generated last). The
loop never broke, and the homeworld, which has only trace ore of its own, had
no income.

**Ablation** (scratch: an empty hauler goes to its home center): seed 19's
seats reach 88 / 130 / 137 colonies at 190 yr, against 7 / 129 / 8; the 12
random galaxies spread median 55, mean 99 (against 190, 223).

**Demand pricing, landed** (the author's ruling):

| ground | spread at 1,500 yr, 12 galaxies | median | mean |
|---|---|---|---|
| random, completion routing (before) | 19–499 | 190 | 223 |
| random, demand pricing | 27–110 | 74.5 | 73.5 |
| identical, colors stepped (`rotational_symmetry`), completion routing | 20–440 | 45 | 88 |
| identical, colors stepped (`Ground::ColorRotated`), demand pricing | 4–29 | 21 | 19.3 |

The two demand-pricing rows were first recorded as median 78 / mean 72
(13–111) and mean 20.4 (5–30). Those runs came from a build before `721127c`
was final. The rows above were measured on `721127c` and reproduced
bit-for-bit after `Ground` replaced `rotational_symmetry` (seeds 1, 7, 13).

On identical ground demand pricing meets the target; what remains on random
ground is the ground (the third kind of ground is below). Paired against `ccca4f4` (4 seeds, 1,500 yr):
colony-years **+2.82% ± 0.39** (4/4 higher), work-years +3.15% ± 9.44 (not
resolved). Cost: seeds 1 and 7 at 800 yr run 6.4–6.6 s against 4.2–4.6 s, with
7–10% more events and ~34% more time per event; the simulation carries more
colonies and haulers early, an inference not profiled.

### The third kind of ground: identical, colors not stepped

*The author's direction: try a third kind of ground. `Ground::Identical`
turns one wedge to every seat with the same colors, so each archetype starts
beside the same deposits as every other; homeworlds and companions are the
same on every kind. Bed: `examples/colony_spread` with `CS_GROUND`, card-free,
3 seats, 1,500 yr, the 12 seeds above, one build.*

| seed | 1 | 7 | 42 | 31337 | 2 | 3 | 5 | 11 | 13 | 17 | 19 | 23 | mean | median |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `Random` | 27 | 76 | 73 | 63 | 82 | 69 | 85 | 67 | 86 | 95 | 50 | 110 | 73.5 | 74.5 |
| `Identical` | 21 | 87 | 33 | 76 | 24 | 64 | 88 | 27 | 51 | 16 | 11 | 77 | 48.0 | 42 |
| `ColorRotated` | 22 | 27 | 19 | 11 | 20 | 20 | 29 | 8 | 4 | 27 | 24 | 22 | 19.3 | 21 |

Paired by galaxy (difference in spread, mean ± standard error over 12 seeds,
an estimate): `Identical` − `ColorRotated` **+28.5 ± 8.5** (higher on 9/12);
`Random` − `Identical` **+25.7 ± 8.3** (higher on 9/12). Both differences are
about 3 standard errors from zero.

Which seat finishes ahead on `Identical` varies between galaxies: seat 1 in
five, seat 2 in five, seat 0 in two. **Inference, stated as one:** with the
colors the same for every seat, each archetype's two rich basics meet a
different share of the wedge's deposits, so the archetype whose colors match
the wedge starts ahead; which archetype that is depends on the wedge's colors,
so it changes between galaxies. Confidence about 60%. A census of the
deposit within reach of each homeworld by color, against its archetype's
bill and against the seat's colony count, would test it.

## D.39 Identical ground: what separates the seats, and pricing by an empire's color gaps

*Supports galaxy §2 and T-147. The author's direction: bring `Ground::Identical`'s
spread toward `ColorRotated`'s; the thought, an empire with mineral gaps should
raise prices. Beds: `examples/ground_census` (deposits within 40 ly of each
homeworld, colonies and stockpile by color per seat over time) and
`examples/colony_spread` (card-free, 3 seats, 1,500 yr, the 12 seeds of §D.37),
against scratch builds of `HEAD` that never land.*

**Where the seats differ.** On `Identical` ground the deposit within 40 ly of
each homeworld differs between seats only by the two companions (about 2,800 kt
each, against 10⁵–10⁶ kt of the wedge's main color). Seats mirror exactly to
50 yr and part between 100 and 200 yr. The archetype enters the simulation in
two places: the companions' colors (galaxy §3) and `scarcity_for`, a fixed
rank weight of 2 on the archetype's poor color in every mineral score.

**Arms** (mean spread over 12 galaxies; colonies summed over seats move by
at most 0.5 of 3,475 on `Identical` and 3,547 on `Random` in every arm):

| arm (scratch) | `Random` | `Identical` | `ColorRotated` |
|---|---|---|---|
| shipped: archetype weight `[1, 1, 2]` on the poor color | 73.5 | 48.0 | 19.3 |
| weight off: `[1, 1, 1]` | 47.8 | 32.1 | 33.6 |
| live weight from the empire's gap to its next bills, gain 1 | — | 28.5 | — |
| same, gain 3 | — | 31.6 | — |
| live weight from holdings, gain 1 | 60.0 | 24.3 | 31.5 |
| same, gain 3 | — | 35.1 | — |
| weight off, every seat's companions Cyan + Magenta | — | 17.3 | — |
| live weight from holdings, gain 1, same companions | — | 19.4 | — |

The two live weights, each normalized so the three sum to 4 as `[1, 1, 2]`
does: *gap to next bills*, `1 + g·Σ deficit_c / Σ bill_c` over the empire's
centers; *holdings*, `1 + g·(1 − cover_c / max cover)` with `cover_c` the
empire's holding of color `c` (owned worlds and outposts) over its works-mix
weight. At the start, with the companions' colors mined and the third empty,
the holdings weight at gain 1 is `[1, 1, 2]` on the empty color — the
archetype weight, read from the bank.

Paired by galaxy (mean ± standard error of the per-galaxy difference, an
estimate): on `Identical`, weight off −16.1 ± 8.4 against shipped (1.9 SE);
holdings weight −23.6 ± 9.7 (2.4 SE); holdings against weight off −7.5 ± 5.5
(1.4 SE, not resolved). On `ColorRotated`, weight off +14.2 ± 5.2 (2.8 SE) and
holdings weight +12.0 ± 4.7 (2.6 SE): both raise the spread there. On
`Random`, weight off −25.8 ± 7.6 (3.4 SE, 11/12 lower).

**Inference, stated as one:** the fixed weight compensates each archetype for
the color its companions lack. It fits the ground only when the ground's colors
turn with the archetype (`ColorRotated`); on `Identical` and `Random` ground it
sends each seat after a different color of the same deposits. Removing both
per-seat differences on `Identical` ground (weight and companions) reaches the
target. Confidence about 70%: the same-companion arms rest on 12 galaxies, and
a second set of 12 seeds the arms were not chosen against would change it if
their spread came out above 30.

## D.40 Holdings-based pricing, landed; the free upgrade at whole Band IV

*Supports autopilot spec §3.9 (holdings-based pricing, the author's ruling),
galaxy §2 and T-147. Beds: `examples/colony_spread` (card-free, 3 seats,
1,500 yr, the 12 seeds of §D.37); single 800-yr runs on seeds 1 and 7 timed
against `ac086e9`'s parent, two runs each; a scratch event census by kind and
century (never landed).*

**A decision storm, found by removing the archetype weight.** With the weight
off, seed 7 ran 893,303 events at 26.6 µs each against the shipped engine's
404,959 at 15.9 µs. Counted by kind and century, build decisions ran ~13,000
per century to 500 yr, then 87,041 and 375,509, and one center — seat 0's
homeworld — committed `UpgradeInfrastructure` 370,903 times between 600 and
700 yr, each holding a berth for the 2-yr lead time. The center stood at
800 kt, whole Band IV, where the ladder ends (`Qty::whole_band(n)` is the
Band IV amount for every `n ≥ 4`), so `infra_step_price` billed zero for an
upgrade that moved nothing; the decision's headroom test compared against the
homeworld's `K` of Band 4.2 and kept choosing it. **Fixed at the source:** the
decision deepens only while a whole Band above exists, and the yard declines a
zero bill (`a_center_on_the_top_whole_band_does_not_deepen`). The shipped
engine was exposed to it wherever a center reached whole Band IV.

**Landed against the arm measured in §D.39.** Seed 1 reproduces the scratch
holdings arm's colony counts exactly (1,050 / 1,077 / 1,049). Spread at
1,500 yr, 12 galaxies (mean; scratch arm in brackets): `Random` 60.1 (60.0),
`Identical` 24.3 (24.3, every galaxy equal), `ColorRotated` 31.7 (31.5; seed 7
14 against 12, the Band IV guard).

**Cost** (an estimate from two runs per seed): per event +2.0% on seed 1
(16,446 against 16,117 ns) and +7.6% on seed 7 (≈17,100 against 15,897 ns),
with events within 1.5% of the shipped engine. The cost is `color_prices`
summing the empire's holdings at every production decision and outpost
ranking — O(worlds and outposts held) per call. A running total kept at each
of the ~90 sites that write holdings would make it O(1); not done.

## D.41 Color-centered homeworlds on three grounds

*Supports galaxy §3 (`Homeworlds::ColorCentered`, built, off by default) and
T-147. The author's direction: homeworlds generated at random with under
`Band I` of every basic, no trio, each equidistant from a region's center of
each color, on `ColorRotated`, `Identical` and `Random` ground, with a minimum
threshold of each color if needed. Beds: `examples/ground_census` (deposit by
color within reach of each homeworld, richest world of each color, colonies,
outposts and stock over time) and `examples/colony_spread` (card-free, 3
seats, 1,500 yr, the 12 seeds of §D.37), holdings-based pricing in both.*

**Equidistance from the field's own sites cannot carry a threshold.** A
search over every Cyan–Magenta–Yellow triple of sites, for the circumcenter
nearest each seat's nominal point inside the seat's own sector (built, then
replaced): with no threshold on the sites' peaks, seeds 7 and 42 had a sector
with no triple at a common distance up to 30 ly; where one existed the colors
within 30 ly of it differed 10³–10⁴-fold (seed 31337: 549 / 4,815 /
968,944 kt), because a site's peak runs from Band 2.0 far from its hue's
hotspot to Band IV near it. With every peak at least Band 3.0, no seat on
seeds 1, 7, 42 or 31337 had a triple in its sector at any distance from 5 to
30 ly. Without the sector rule, two and three seats took the same point.
**Inference, stated as one:** sites of all three hues at strength coexist only
where the three hotspots overlap, near the galactic center.

**Built instead: planted sites.** One site of each hue is planted at
`homeworld_site_distance_ly` (10 ly) from the homeworld, 120° apart, at peak
`homeworld_site_band`; on identical ground seat 0's are planted inside its
wedge and turned with it. Within 20 ly of seat 0 the planted colors read
43–1,991 kt at Band 3.0 and 1,632–21,727 kt at Band 4.0, against 24,459–293,852
kt of the sector's own dominant hue; Cyan, whose site sits on the outward side
of the triangle where the disk is thinner, was the weakest on every seed.

**Spread at 1,500 yr** (12 galaxies; mean, median; colonies summed over seats):

| homeworlds | `Random` | `Identical` | `ColorRotated` |
|---|---|---|---|
| trio (§D.40) | 60.1, 64.3; 3,547.2 | 24.3, 18.8; 3,474.9 | 31.7, 25.4; 3,475.2 |
| color-centered, planted Band 3.0 | 322.3, 338.8; 3,521.5 | 70.6, 31.4; 3,449.1 | 129.8, 80.2; 3,449.2 |
| color-centered, planted Band 4.0 | 191.8, 154.7; 3,510.7 | 81.8, 41.2; 3,439.4 | 86.7, 60.5; 3,439.9 |

**The start is what differs** (seed 2, `Identical`, seat 0). With the trio:
2 outposts at 25 yr, 24 colonies at 100, 74 at 200. Color-centered at Band
4.0: 3 outposts at 25 yr and 8 at 100, one colony until ~110 yr, 10 at 200;
the home bank held 0.0 kt through 100 yr. Within 5 ly of the homeworld the
trio holds 2,828 kt of each companion color at 2.0 ly; the color-centered
homeworld holds 1–7 kt of its two scarce colors there, and its richest world
of each color lies 4–14 ly out. The seats stay identical to 125 yr and part
by 400 yr (41 / 94 / 40 colonies at Band 3.0). **Inference, stated as one:**
the outposts a color-centered empire opens first are small or far, the start
runs a century longer, and small differences have that century to compound.
Confidence about 60%; an ablation planting a world at each site's center
would test it.

## D.42 The first forge's date: planted outposts, starting fleets, and the starting population

*Supports galaxy §3 and T-147. The author's direction: plant three mining
outposts, 120° apart, within the color regions, `Band I`, single color,
identical, 5 ly out; add starting freighters, miners and colony ships to
accelerate the time to forge; the target, a mean of 400 yr with a 5-yr standard
deviation. Bed: `examples/forge_time` (card-free, 3 seats, each homeworld's
population read once per simulated year; the economy ticks every 5 yr), on
`Homeworlds::ColorCentered` with the outposts; starting fleets through
`FleetSeeding` (one miner, one freighter and one Medium colonizer fleet per
seat, 0.3 kt each, a 15-ly surveyed start).*

**What sets the date.** `is_forge` is population at `Band IV` and nothing else,
and population follows the logistic toward the homeworld's `K` (`Band 4.2`,
2,163,979 kt) at `growth_rate / rate_reference_years` = 0.01746 per yr. From the
shipped start (`Band 2.0`, 31.6 kt) that is 597.3 yr to `Band IV`
(715,542 kt); minerals do not enter it. A colony ship's settlers are debited
from the homeworld (`embark`), so emigration can only delay it.

**First forge, `Identical` ground** (mean, standard deviation over all seats,
mean standard deviation within a galaxy; seeds 1, 7, 42, 31337):

| arm | mean | sd | within |
|---|---|---|---|
| trio (9 of 12 seats forged by 800 yr) | 688.4 | 17.7 | 12.0 |
| color-centered, outposts | 624.0 | 15.2 | 1.8 |
| + starting fleets (6 of 12 seats forged by 800 yr) | 676.5 | 4.2 | 4.1 |
| outposts, start `Band 2.75` | 408.2 | 2.5 | 0.5 |
| + starting fleets | 410.7 | 0.5 | 0.5 |
| outposts, start `Band 2.8`, with or without fleets | 395.7 | 0.5 | 0.5 |

The logistic alone predicts 404.2 yr at `Band 2.75` and 391.3 at `Band 2.8`.

**At `Band 2.785` with the starting fleets** (seeds 1, 7, 42, 31337, 2, 3, 5,
11): first forge **400.7 yr, standard deviation 0.5 yr** on `Random`,
`Identical` and `ColorRotated` ground alike — seat 0 at 400 and seats 1 and 2
at 401 in every galaxy, the order in which their economy ticks fall within a
year. Spread of colonies between empires at 1,500 yr on the same configuration
(12 galaxies, mean): `Random` 131.9, `Identical` 25.9, `ColorRotated` 55.0,
against 60.1 / 24.3 / 31.7 with the trio (§D.40).

## D.43 Variation across empires in the tree stocks and in supers forged

*Supports T-147. The author's direction, after ratifying a starting population
of `Band II .785`: reduce the variation across empires in tree metrics and
supers forged. Bed: `examples/empire_spread` (card-free, 3 seats, 1,500 yr, the
12 seeds of §D.37, trio homeworlds). Per seat: Expansion `∫ C dt`, Growth
`∫ V dt` (works), Production `∫ F dt` (fleet volume), supers and apex forged.
The spread is the coefficient of variation between the seats of a galaxy
(standard deviation over mean), averaged over galaxies.*

| ground | Expansion | Growth | Production | supers | apex |
|---|---|---|---|---|---|
| `Random` | 0.070 | 0.423 | 0.040 | 0.125 | 0.119 |
| `Identical` | 0.028 | 0.256 | 0.042 | 0.095 | 0.264 |
| `ColorRotated` | 0.033 | 0.536 | 0.042 | 0.151 | 0.143 |

**Growth is bimodal.** A seat ends at 250,000–550,000 kt-years of works or at
1.2–3.1 million. The mechanism, traced on seed 1, `ColorRotated` (`ES_TRACE`,
`ES_WATCH`):

1. A colony at whole Band III with a ceiling above III saves for whole Band
   IV, a 780-kt step (the cost ladder's III → IV factor is 40).
2. Demand pricing values a hold at `Σ_c min(cargo_c, want_c) · price`, so the
   center with the largest unmet bill draws its empire's freight: one center
   took 60–82% of a seat's deliveries in the century before its purchase.
3. The works bill is split by the works mix (2 : 1 : 3 Cyan, Magenta, Yellow)
   and paid per color. World 2857 completed every color at 969 kt banked and
   bought whole Band IV at 304 yr; world 2115 held 1,292 kt at 370 yr and could
   not, one color short.
4. From about 400 yr the homeworld forges draw 45–96% of each seat's freight,
   and further Band IV purchases stop (seat 0's next came at 1,459 yr).

A seat that completes a Band IV bill before its forge stands holds its works at
800 kt on that world for the rest of the run; one that does not stays near
20 kt per world.

**Two scratch arms** (never landed), paired over the 12 galaxies (arm minus
shipped; mean ± standard error; levels as the mean relative change):

| arm | ground | Growth cv | Growth level | supers cv | supers level |
|---|---|---|---|---|---|
| forge premium 1 (was 10) | `Random` | −0.216 ± 0.063 | +361% ± 76 | +0.175 ± 0.037 | −60.1% ± 2.0 |
| | `Identical` | −0.047 ± 0.056 | +37% ± 11 | +0.042 ± 0.015 | −26.2% ± 3.5 |
| infrastructure bill paid in total, any colors | `Random` | −0.151 ± 0.091 | +912% ± 158 | −0.064 ± 0.037 | +28.6% ± 6.3 |
| | `Identical` | −0.170 ± 0.087 | +2,781% ± 296 | −0.034 ± 0.021 | −0.8% ± 5.0 |

Expansion and Production move by under 0.02 in every arm. **Inference,
stated as one:** both arms reduce Growth's spread by moving every empire's
level, not by evening the race — the color conjunction on a lumpy bill is a
cliff the shipped economy sits against, and the spread is which empires cross
it before forging begins. Confidence about 65%; the per-galaxy differences
carry standard errors near half their size.

## D.44 Infrastructure bought in fractions of a Band

*Supports T-147. The author's direction: buy infrastructure in a unit smaller
than a whole Band, at every stage, to smooth the variation between empires —
try 0.5, 0.25, 0.125 and 0.0625 Band. Built as `sim::INFRA_STEP_BANDS`
(shipped 1.0, bit-identical to the whole-Band engine); each arm a scratch
build with the constant changed. Bed: `examples/empire_spread`, `Random`
ground, the 12 seeds of §D.37, 3 seats, 1,500 yr. Paired over galaxies, arm
minus shipped, mean ± standard error; levels are the mean relative change of
the galaxy total.*

| step (Band) | Expansion cv | Growth cv | Production cv | supers cv | apex cv | Growth level | Production level | supers level |
|---|---|---|---|---|---|---|---|---|
| 1.0 (shipped) | 0.070 | 0.423 | 0.040 | 0.125 | 0.119 | — | — | — |
| 0.5 | +0.002 ± 0.004 | −0.059 ± 0.092 | +0.001 ± 0.010 | +0.039 ± 0.042 | +0.048 ± 0.029 | +2.4% ± 16.2 | −10.1% ± 1.1 | −7.8% ± 5.1 |
| 0.25 | +0.020 ± 0.017 | −0.156 ± 0.088 | +0.042 ± 0.019 | +0.007 ± 0.044 | +0.019 ± 0.032 | −26.5% ± 16.7 | −24.0% ± 1.9 | −32.7% ± 5.1 |
| 0.125 | +0.066 ± 0.033 | −0.136 ± 0.095 | +0.107 ± 0.023 | +0.083 ± 0.028 | +0.099 ± 0.037 | −51.3% ± 19.0 | −32.1% ± 2.4 | −41.9% ± 5.7 |
| 0.0625 | −0.029 ± 0.010 | −0.162 ± 0.084 | +0.076 ± 0.027 | +0.048 ± 0.052 | +0.053 ± 0.018 | −54.3% ± 18.7 | −31.4% ± 1.7 | −42.7% ± 5.1 |

Every finer step lowers Growth's spread by 0.06–0.16, and none of those
differences reaches 2 standard errors (the largest, at 0.0625, is 1.9). At
0.25 Band and below the spread in Production, supers and apex rises, and the
levels of Production and supers fall by 24–43%.

**One mechanism is measured, in §D.45:** at 0.0625 Band, pricing each color by
its own shortfall raises Growth's level +86.7% ± 7.2 over the 0.0625-Band arm
alone, 12 of 12 galaxies. A smaller step is a smaller bill, and the
total-based pressure that priced a color reached zero once a bank passed it.
Against the shipped engine the combined arm still moves Production −28.4% ±
2.0, supers −39.1% ± 4.7 and apex −32.8% ± 4.0, so that mechanism is not the
whole of the loss. **Inference, stated as one:** the remaining loss comes
from more, smaller purchases displacing hull builds at the yard. Confidence
about 40%; a per-decision census of what the yard builds at each step size
would settle it.

## D.45 The color a saving center lacks was priced at zero

*Supports T-147 and R-P19. The author's question: "Help the saving colony
finish its colors. Why aren't haulers being directed towards the potential
forge?" Bed: `examples/empire_spread` (now printing color stalls, the
shortfall census and freight to forges), 3 seats, 1,500 yr, the 12 seeds of
§D.37.*

**The case.** Seed 1, `ColorRotated`, world 2115 (`ES_WATCH=2115`): bank
C/M/Y 145.7 / 671.2 / 854.1 kt at 400 yr against a 780-kt whole-Band-IV bill
that asks about 260 / 130 / 390 at the works mix. It held 1,292–1,672 kt from
370 yr to the end of the run without paying. Its ceiling is Band III .850, so
its population can never reach Band IV and it is not a potential forge; the
forge in each seat is the homeworld, standing from about 400 yr.

**The mechanism, from the code and then by ablation.** Every price a center
posts — its Exchange bid, freight's delivery score and the center-to-center
offer test — was `base · demand · mineral_pressure`, and `mineral_pressure`
was `1 − bank total / bill total`. At 1,671 kt against 780 it read 0, so Cyan
was priced at 0: no bid, and a hold of Cyan worth nothing to the center that
lacked it. The scratch arm priced each color by its own shortfall,
`1 − held_c / bill_c`, and changed nothing else.

**Color stalls** — a center holding at least its next bill's total and unable
to pay it in every color — are not rare: about 1,300 centers per galaxy on
`Random` and `ColorRotated` (1,800 on `Identical`) and about 1 million
center-years per galaxy, about 740 years per stalled center. Paired, per-color
arm against shipped (log-ratio mean, standard error):

| ground | stalled center-years | stalled centers |
|---|---|---|
| `Random` | −15.2% ± 2.2, 12/12 lower | −18.0% ± 2.9, 12/12 |
| `ColorRotated` | −12.9% ± 2.4, 11/12 | −14.4% ± 3.0, 11/12 |
| `Identical` | −4.0% ± 1.2, 12/12 | −4.1% ± 1.0, 11/12 |

**Tree metrics,** same pairing (cv difference; level relative change):

| ground | Growth cv | Growth level | supers cv | supers level | apex cv | apex level |
|---|---|---|---|---|---|---|
| `Random` | +0.057 ± 0.088 | +1.9% ± 16.7 | +0.037 ± 0.037 | +15.2% ± 5.3 | +0.068 ± 0.029 | +5.5% ± 5.7 |
| `ColorRotated` | −0.174 ± 0.119 | +25.8% ± 16.7 | +0.006 ± 0.044 | +13.4% ± 4.8 | +0.042 ± 0.043 | −3.7% ± 4.2 |
| `Identical` | −0.005 ± 0.098 | +9.3% ± 14.1 | −0.019 ± 0.016 | +2.0% ± 3.5 | +0.004 ± 0.052 | −16.3% ± 6.6 |

Expansion and Production move by under 0.011 in cv and under 1% in level on
every ground. No Growth-spread difference reaches 2 standard errors. Supers
forged rise by 2.8 and 2.9 standard errors on `Random` and `ColorRotated`;
apex's spread on `Random` rises by 2.3 and its level on `Identical` falls by
2.5.

**What the per-color price does not fix — delivery.** Shortfall census at 500,
1,000 and 1,500 yr, 4 seeds, `Random`: the color shortfall summed over an
empire's centers is 1.3–7.6 Mt per color, and the same empire holds 39–634 Mt
of that color above its centers' own bills: 98.0–100% of the shortfall is
covered color by color, on both engines. On seed 1, 62–95% of that surplus
is ore waiting at outposts, and center banks above their own bills hold
7–60 Mt per color, 4.6–10.8 times the whole shortfall. Per-color pricing did not lower the summed shortfall.
**Freight to forges**, seed 1, `ColorRotated`, both engines: a seat's forge
takes 40–70% of its deliveries in the 400s and 72–96% in the 500s. A forge's
price is full on every color times `forge_premium` = 10, and its want is a
balanced set up to the most it holds of any color, so it rises as it fills.

**Inference, stated as one:** the remaining stalls are set by where haulers
go, and the forge's fixed 10× price with a want that grows as it fills is what
outbids the stalled centers once a forge stands. Confidence about 60%, from
one galaxy's freight trace; the forge share on all 12 galaxies, and an arm
whose forge price falls as its holding rises, would settle it. Landed: the
per-color price (`Simulation::color_pressure_of`), bit-identical to the
measured arm on seeds 1 and 7.

## D.46 A forge's price falls with what it holds

*Supports T-147 and galaxy §4.5. The author's ruling: "forge price should vary
with its holding." Built as `forge_premium · B / (B + H)` — `H` everything the
forge holds, every tier, kt; `B` the price of a whole Band IV works stock, kt.
Bed: `examples/empire_spread`, 3 seats, 1,500 yr, the 12 seeds of §D.37,
against the engine with per-color prices (§D.45). Paired over galaxies: cv
difference and relative level change, mean ± standard error.*

| ground | Growth level | Growth cv | supers level | supers cv | apex level | apex cv | stalled center-years |
|---|---|---|---|---|---|---|---|
| `Random` | +172.1% ± 7.1 | −0.266 ± 0.081 (11/12 lower) | −53.3% ± 5.0 | −0.084 ± 0.024 | −52.2% ± 3.8 | +0.057 ± 0.028 | +10.4% ± 1.4 |
| `ColorRotated` | +162.3% ± 10.1 | −0.129 ± 0.072 | −56.9% ± 5.8 | −0.087 ± 0.024 | −55.2% ± 6.8 | +0.099 ± 0.053 | +11.7% ± 1.7 |
| `Identical` | +23.5% ± 3.7 | −0.018 ± 0.036 | −33.4% ± 3.8 | +0.002 ± 0.010 | −38.7% ± 6.7 | +0.108 ± 0.027 | +9.2% ± 2.8 |

Expansion and Production move by under 0.5% in level and 0.003 in cv.

**The mechanism, by trace** (seed 1, `ColorRotated`, seat 0, `ES_TRACE`): the
forge's share of its seat's freight is 81% in the 500s on both engines, then
falls under the holding-priced premium to 61% in the 800s, 27% in the 1000s and
2–5% from 1,200 yr, against 67–85% throughout on the old engine; freight
delivered elsewhere rises (in the 1,200s, 7,517 kt against 5,554). The basics a
forge stops drawing go to ordinary centers, which buy Bands (Growth) and then
wait on larger bills (stalls). **Inference, stated as one:** forging falls by
half because nothing card-free consumes supers or apex, so every forge's holding
only grows and its price only falls. Confidence about 75%; a bed with final
demand for supers (T-146's twin bed) would show whether the price recovers when
a forge's holding is drawn down.

**And it moved the Exchange.** `clearing_strikes_escrowed_contracts_without_moving_the_world`
failed under this change alone: a forge that has sold supers keeps them back
from apex, apex synthesis leaves slag, so the seller holds more and its price
reads it. The test now pins the premium at zero, which prices a forge at
nothing whatever it holds, and states that channel.

## D.47 A hauler priced against the shipping backlog

> **Superseded measurement (§D.48).** The tables below were taken on an engine
> with two defects in this change: a backlog hauler was sized to the rock's
> mining rate, so a pile the Exchange dropped at a small rock got Limited
> hulls and seed 1 built hundreds of thousands of them after the 1,000-year
> barrier; and a Limited freighter order resolved to the miner role. Read the
> tables as a record of that engine; §D.49 measures the corrected one.

*Supports T-147, R-P19 and roles §4.4. The author's direction: "price of
building a new hauler should increase with high demand in shipping", read (the
author's choice) as the hauler's value rising so more are built; the bill stays
the hull's dry mass (design law #11). Built: a freighter Design ordered on its
own; per rock, the wanted ore beyond the holds based there; one trip valued at
the empire's want-weighted prices times the share of haulers sent to that rock
not lost there; the cost valued at the building center's own prices; built in
the fallback slot ahead of survey; Reserve first. Bed as §D.46.*

**First build: it ran away.** The backlog was read off the pile, which cannot
see haulers flying to it: seed 1 built **152,874** freighters by 1,500 yr
against 5,620, events 1.0 M → 3.55 M (`hauler_census`, scratch). Subtracting the
holds of the haulers based at each rock bounded it at **33,217**, events 1.43 M,
ore waiting at outposts 302 Mt against 764 Mt.

**Hauler alone** (paired against §D.45's engine):

| ground | Growth level | Production level | supers level | apex level | supers cv | apex cv | stalled center-years |
|---|---|---|---|---|---|---|---|
| `Random` | +155.4% ± 15.6 | +585.2% ± 5.3 | +416.6% ± 7.9 | +455.6% ± 6.0 | −0.060 ± 0.026 | +0.006 ± 0.033 | +25.0% ± 2.3 |
| `ColorRotated` | +104.4% ± 13.1 | +526.6% ± 4.8 | +357.8% ± 7.0 | +479.6% ± 5.9 | −0.079 ± 0.032 | −0.071 ± 0.026 | +24.3% ± 2.9 |
| `Identical` | −29.8% ± 9.0 | +370.9% ± 7.9 | +55.3% ± 17.5 | −32.0% ± 12.5 | −0.010 ± 0.016 | −0.121 ± 0.055 | +24.4% ± 3.1 |

**Forge price and hauler together** (what lands):

| ground | Growth level | Growth cv | Production level | supers level | apex level | stalled center-years |
|---|---|---|---|---|---|---|
| `Random` | +921.6% ± 18.4 | −0.263 ± 0.068 (11/12 lower) | +693.0% ± 6.0 | +64.0% ± 6.2 | +67.6% ± 4.6 | +31.3% ± 3.3 |
| `ColorRotated` | +633.8% ± 13.9 | −0.175 ± 0.061 (9/12 lower) | +618.0% ± 5.4 | +52.8% ± 5.7 | +72.6% ± 5.6 | +30.2% ± 2.6 |
| `Identical` | +8.5% ± 11.8 | −0.120 ± 0.088 | +391.0% ± 6.3 | +2.0% ± 14.3 | −58.3% ± 13.8 | +38.6% ± 2.8 |

Expansion falls 0.8–1.0% (standard error 0.1–0.2) on every ground and arm.
Production is `∫` fleet volume, so it counts the haulers themselves. No hull is
armed card-free, so the survival share is 1 there; the landed engine
reproduces the measured combined arm bit for bit on seeds 1 and 7.

**The loss in the price.** On the determinism card bed (six seats, Warfare,
Growth and missile cards, 350 yr), seed 7 ran 3.43 M events against 0.97 M:
haulers sent to rocks among the Warfare card's pickets, wrecked or withdrawn,
and replaced while the backlog stood (in the last 50 years, 616 freighters
spawned, encounters 11,063 → 19,397, withdrawals 1,338 → 2,324). Valuing a trip
at the share of haulers that came back (R-WAR47's rule) took it to 2.64 M
events and 459 freighters spawned. **Inference, stated as one:** the rest is
haulers flying among rival pickets, which is the conflict R-P19 asks for,
paid in fire events. Confidence about 55%; a census of encounters by role
would settle it.

**Cost.** Telemetry bed (3 seats, 500 planets, 600 yr, seed 1), release:
29,753 → 44,360 events, 1,455–1,871 → 2,300–2,933 ns per event, 12,455 → 22,147
instructions per event — routing decisions (`best_delivery_center`,
`offer_from`) are each `O(centers)` and there are six times the haulers to make
them. Pricing all three colors from one read of the bill took instructions
1,029 M → 982 M with the run bit-identical; per-color calls had re-read it. Test
targets, unloaded, old → new before the scenery changes: unit 5.9 → 20.4 s,
determinism 40.3 → 105.4 s, telemetry 32.8 → 68.0 s; after (card bed 600 → 400
planets, telemetry 800 → 400): 19.4 / 33.5 / 9.3 / 29.5 s.

## D.48 Three defects the forge-price sweep surfaced

*Supports T-147. Found running `examples/forge_sweep` (twin bed) at low forge
premiums; each is fixed, and each is pinned by a test.*

1. **NaN from an empty bank.** `Minerals::try_take_total` admits an amount up
   to 1e-9 against a bank of exactly zero, and divided by the zero total. An
   order paid wholly in supers leaves its basic part as a rounding residue of
   `price − owed` (~1e-18); a forge that had synthesized its last basic held
   exactly zero; every color became `0 · ∞`. Seen as `supers inf apex inf` in 3
   of 80 sweep runs, first at 1,421.88 yr on seed 42 (premium 0.8), located by
   checking `Simulation::mass_ledger` after every event. The defect predates
   T-147; the twin bed at a low premium is what drains a forge to zero. Fixed:
   an empty bank pays a crumb with nothing (`an_empty_bank_pays_a_crumb_with_nothing`).
2. **A Limited freighter order resolved to the miner role.** `role_of` read
   the freighter Design only on the Medium and General hulls, and the T-98
   sizing builds Limited haulers too. Fixed by reading it on every Systems
   hull (`a_hauler_is_built_where_its_trip_is_worth_its_minerals`).
3. **A barrier storm.** With (2) fixed, seed 1 at the shipped premium built
   25,301 → 648,743 freighters between 1,000 and 1,060 yr, while 707 Exchange
   contracts settled and ore held at rocks rose 222 → 274 Mt. The backlog
   hauler was sized by `freighter_hull`, which reads the rock's mining rate, so
   a delivered pile at a small rock got a Limited hull whose hold is a sliver
   of the pile. Fixed: a backlog hauler takes the Systems hull, of those the
   center can pay for now, that lifts the most of the backlog per kilotonne
   (`Simulation::backlog_hull`). After the fix, seed 1 holds 27,000–31,700
   freighters to 1,500 yr at premiums 10 and 0.6, 87 s a run.

## D.49 The forge's price, swept on the twin bed

*Supports T-147. The author's ruling: forging cannot be evaluated without
demand from the alt Designs paid in supers, and the forge-price decisions are
made to maximize the tree metrics on the standard bed with those Designs, then
made default. Bed: `examples/forge_sweep` — the standard galaxy (`Random`
ground, trio homeworlds), 3 seats, 1,500 yr, every seat seeded twin Designs
billed a third each in Red, Green and Blue. Objective: the geometric mean over
Expansion, Growth and Production of each tree's stock divided by its value at
the old default (`premium 10, floor 0, scale 1`) on the same seed (`AGENTS.md`
§2's composite). Engine: after §D.48's fixes; the first two passes ran on the
defective engine and are discarded.*

Screen, seeds 1, 7, 42, 31337 (mean ± standard error of the composite):

| premium | floor | scale | composite | Growth | Production | supers | apex |
|---|---|---|---|---|---|---|---|
| 0.3 | 0 | 1 | +9.5% ± 1.8 | +33.8% | −1.8% | −93.3% | −93.5% |
| 1 | 0 | 1 | +9.0% ± 2.1 | +31.5% | −1.4% | −85.9% | −92.3% |
| 0.6 | 0 | 0.3 | +8.5% ± 2.2 | +27.2% | +0.5% | −93.0% | −94.3% |
| 0.6 | 0.3 | 1 | +8.4% ± 2.2 | +28.8% | −0.9% | −92.4% | −93.4% |
| 1 | 0.5 | 1 | +8.4% ± 2.4 | +28.0% | −0.4% | −80.5% | −85.1% |
| 0.6 | 0 | 1 | +8.1% ± 2.0 | +29.0% | −1.8% | −92.0% | −93.1% |
| 0.6 | 0 | 3 | +7.0% ± 2.1 | +25.3% | −2.1% | −91.5% | −93.3% |
| 3 | 0 | 1 | +5.5% ± 0.9 | +19.0% | −1.1% | −61.2% | −67.8% |
| 3 | 0.5 | 1 | +3.5% ± 1.0 | +12.2% | −1.0% | −37.0% | −37.6% |
| 10 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |

Expansion moves by under 0.25% in every arm. Replication on seeds 2, 3, 5, 11
(chosen against nothing) and the pooled eight:

| config | seeds 1, 7, 42, 31337 | seeds 2, 3, 5, 11 | pooled, n = 8 |
|---|---|---|---|
| premium 0.3 | +9.47% ± 1.82, 4/4 | +7.99% ± 1.26, 4/4 | **+8.73% ± 1.06, 8/8** |
| premium 1 | +9.01% ± 2.12, 4/4 | +8.04% ± 0.89, 4/4 | +8.52% ± 1.08, 8/8 |
| premium 1, floor 0.5 | +8.37% ± 2.45, 4/4 | +7.24% ± 1.52, 4/4 | +7.80% ± 1.35, 8/8 |

**Shipped: premium 0.3, floor 0, scale 1** — the highest pooled composite;
premium 1 is inside its noise. The surface is a plateau below a premium of
about 1 and falls above it: every arm that raises supers and apex lowers
Growth, Expansion does not move, and Production moves by under 2.1% because
the hulls paid in supers are a small part of the fleet (183–234 kt of hull per
seat at the old default). **Inference, stated as one:** the tree composite
prices forging only through those hulls, so it prefers basics spent on works
to basics spent on supers; a final demand for supers larger than the twin
hulls' would move the optimum up. Confidence about 70%; a bed whose Designs
bill more of their price in supers would settle it.

## D.50 Why empires buy a different number of whole Band IV works

*Supports T-147. Bed: the twin bed of §D.49 at the new default (premium 0.3),
3 seats, 1,500 yr, card-free; scratch census builds of `examples/empire_spread`
reading the snapshot at the horizon and the Production log. Seeds 1, 7, 42,
31337 on `Random` and `Identical` ground.*

**Growth per seat tracks the count of whole Band IV works purchases**: the
correlation (Pearson r) between a seat's Growth stock and its count of
purchases is 0.94–0.97 across the grounds measured. The count decomposes as

> purchases = centers eligible (works at Band III, ceiling above III) × share of them that completed the purchase

and each factor varies, by different causes on the two grounds.

| ground | seed | eligible per seat | bought per seat | completion | unbought, holding the 780 kt total but short a color | unbought, short in total |
|---|---|---|---|---|---|---|
| Random | 1 | 197 / 199 / 167 | 72 / 70 / 65 | 37% / 35% / 39% | 44 / 41 / 44 | 81 / 88 / 58 |
| Random | 7 | 214 / 218 / 211 | 125 / 104 / 70 | 58% / 48% / 33% | 28 / 40 / 62 | 61 / 74 / 79 |
| Identical | 1 | 44 / 41 / 45 | 5 / 0 / 1 | 11% / 0% / 2% | 32 / 37 / 40 | 7 / 4 / 4 |
| Identical | 7 | 22 / 35 / 17 | 3 / 2 / 2 | 14% / 6% / 12% | 19 / 32 / 15 | 0 / 1 / 0 |

Across the four seeds per ground, eligible centers run 167–260 per seat on
random ground and 12–45 on identical ground; completion runs 29–58% and 0–11%.

**Random ground: the missing color is in the same empire.** Seed 7's three
seats are equally eligible (214 / 218 / 211) and complete 58 / 48 / 33%. The
unbought centers that hold the total lack one color, and the empire holds that
color above its bills elsewhere: seed 7 seat 0 is short 15.0 Mt of Cyan at its
stalled centers and holds 106.7 Mt of Cyan above bills; seat 1 is short
22.2 Mt of Yellow and holds 110.2 Mt above bills; seed 42 seat 0 is short
35.5 Mt of Yellow and holds 131.4 Mt. Every seat measured (6 of 6) holds at
least 2.7x its shortfall in each color it is short. The limit on random ground
is delivery inside the empire, not supply.

**Identical ground: the missing color is absent from every seat.** Unbought
centers hold 2.5–6.3 Mt, 3–8x the 780 kt bill, and every seat on a seed lacks
the same color: seed 1 is short 8.3–12.4 Mt of Cyan per seat against 1–7 kt
of Cyan held above bills; seed 7 is short 1.7–5.1 Mt of Magenta against
0–1 kt. Because all seats share the gap, no seat has Cyan (seed 1) or Magenta
(seed 7) to sell, and an Exchange trade cannot fill it.

**Inference, stated as one:** on random ground the spread in Growth is a spread
in how much of each empire's own surplus color reaches its stalled Band III
centers, so a remedy acts on freight priority or price inside the empire; on
identical ground the spread comes from which seat happens to complete the few
purchases the shared supply allows, and no change to trade can raise the count
for every seat. Confidence about 75% for the random-ground half, from 6 seats on
two seeds; a census of freight deliveries of the short color to the stalled
centers, per seat, would raise or refute it.

## D.51 The color a nearly-paid center lacks is outbid by its own empire

*Supports politics §2.11's completion term and T-147. Bed: §D.50's (twin bed,
new defaults, 3 seats, card-free, `Random` ground); a scratch census (never
landed) reading each seat at 600 and 900 yr, seeds 1, 7 and 42. A **stalled** center has
works at Band III, holds its Band IV bill's total and cannot pay it in every
color; its **color** is the one it is shortest of.*

**Where the color is.** Summed over each seat's stalled centers, the piles of
their color held by the same empire within 10 ly come to 0–8 kt, and within
25 ly to 2–3,169 kt. Within 50 ly they reach 836–37,930 kt, against a summed
shortfall of 515–6,672 kt. The median distance from a stalled center to one
pile covering its shortfall is 26–81 ly (on one seat at 600 yr no single pile
covered it); to a center holding that color above
its own bill, 11–42 ly. Freighters based at rocks of that color within 25 ly
number 638–2,505 per seat. **Inference:** the color is mined out near each
stalled center by the haulers based there, and what remains lies one to two
leg lengths away.

**What the stalled center bids for it.** Comparing the stalled center's price
for its color (`willingness_to_pay`) with the price every other center of the
same empire that wants that color posts:

| seed | t (yr) | seats: share of the other centers wanting it that bid more | share of their want |
|---|---|---|---|
| 7 | 600 | 70% / 82% / 96% | 67% / 65% / 87% |
| 1 | 600 | 78% / 92% / 91% | 70% / 91% / 76% |
| 1 | 900 | 78% / 77% / 86% | 68% / 70% / 79% |

The stalled centers' mean price for their color is 0.67–1.84 against a full
price of 1–3 per color (`doctrine_demand` 2 : 1 : 3, Cyan : Magenta :
Yellow). **Inference:** under `1 − held_c / bill_c` a center's price for a
color falls as it fills, so a center that lacks one color bids less for it
than a center just starting its bill does, and the freight score, the
hauler's backlog and the Exchange all rank it below those centers. The
ablation that tests this is §2.11's completion term (sweep below).

**Smoke test.** `completion_exponent = 0` reproduces the shipped engine to
every printed digit (seed 1, 1,500 yr).

**The sweep** (`examples/forge_sweep`, `FS_COMPLETION`; twin bed, 3 seats,
1,500 yr; composite as in §D.49, against `γ = 0` on the same seed; cv is the
coefficient of variation of a tree stock across the three seats, averaged
over seeds). Screen on seeds 1, 7, 42, 31337:

| γ | composite | Growth | Production | Growth cv | apex cv |
|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0.156 | 0.350 |
| 0.25 | +4.5% ± 1.9 | +10.2% | +3.4% | 0.191 | 0.582 |
| 0.5 | +4.3% ± 2.1 | +12.6% | +0.6% | 0.180 | 0.497 |
| 0.75 | +5.0% ± 1.2 | +14.6% | +0.9% | 0.139 | 0.448 |
| 1 | +2.8% ± 1.0 | +9.9% | −0.8% | 0.143 | 0.514 |
| 1.5 | −2.8% ± 1.8 | −5.2% | −3.1% | 0.084 | 0.623 |

Expansion moves by under 0.3% in every arm. Replication on seeds 2, 3, 5, 11
(composite as the log of the summed stocks, so the screen column differs from
the table above in the last digit):

| γ | seeds 1, 7, 42, 31337 | seeds 2, 3, 5, 11 | pooled, n = 8 | Growth cv, paired change, n = 8 |
|---|---|---|---|---|
| 0.5 | +4.17% ± 2.05, 3/4 | +6.89% ± 0.98, 4/4 | +5.53% ± 1.17, 7/8 | +0.026 ± 0.024 |
| **0.75** | +4.83% ± 1.22, 4/4 | +6.46% ± 1.01, 4/4 | **+5.65% ± 0.79, 8/8** | −0.001 ± 0.018 |
| 1 | +2.80% ± 1.01, 4/4 | +5.49% ± 0.41, 4/4 | +4.14% ± 0.72, 8/8 | +0.007 ± 0.021 |

**Shipped `γ = 0.75`.** `0.5` is inside its noise on the composite; `1` is
1.5 points lower, about two standard errors.

**The mechanism check** (the census above, seeds 1 and 7, `γ = 0` against
`0.75`):

| | `γ = 0` | `γ = 0.75` |
|---|---|---|
| other centers wanting the color that bid more than the stalled center | 70–96% | 12–40% |
| their share of the want for that color | 62–91% | 12–46% |
| stalled center's mean price for its color | 0.67–1.84 | 2.59–4.31 |
| stalled centers per seat at 1,200 yr, seed 7 | 30 / 30 / 40 | 19 / 19 / 19 |
| stalled centers per seat at 1,200 yr, seed 1 | 34 / 42 / 34 | 30 / 11 / 34 |
| centers at Band III short of the total, 1,200 yr, seed 7 | 56 / 61 / 62 | 67 / 67 / 69 |

The price moved what it was written to move: a stalled center now outbids
most of its empire for the color it lacks, and color stalls fall on both
seeds. Growth rises 16% pooled. **The spread between seats does not move**
(Growth cv −0.001 ± 0.018). **Inference, stated as one:** the term raised
every seat's completions by a similar factor, and what separates seats on
random ground after it is the count of Band III centers short of the bill's
total (an income limit) and the count that reach Band III at all, neither of
which a price between a seat's own centers reaches. Confidence about 60%;
a census of income per Band III center by seat, at `γ = 0.75`, would test it.

## D.52 Income per Band III center, and why a center short one color stays short

*Supports T-147. Bed: the twin bed of §D.49 at `completion_exponent = 0.75`,
3 seats, 1,500 yr, card-free, seeds 1 and 7, `Random` and `ColorRotated`
ground. Scratch census builds (never landed) counting, per center, the
kilotonnes that enter its bank by source and color while its works stand at
Band III. **Bought:** reached Band IV works by the horizon. **Short one
color:** holds its Band IV bill's total and cannot pay it in every color.
**Short in total:** holds less than the total.*

**Income per Band III center, by class** (kt/yr per center, summed over
colors; ranges over the 12 seats):

| class | centers per seat | years at Band III, mean | freight in | own mining | Exchange in |
|---|---|---|---|---|---|
| bought | 79–137 | 195–282 | 8.6–18.2 | 0.1–3.5 | 0 |
| short one color | 7–34 | 358–528 | 2.5–7.1 | 0.0–7.9 | 0 |
| short in total | 73–140 | 339–493 | 0.6–0.9 | 0.0–0.9 | 0 |

Freight is the income of a Band III center: its own planet yields nothing on
most seats, and the Exchange delivers to rocks, never to a center. A center
short one color receives that color at **0.24–0.47 kt/yr** (per center, since
reaching Band III), against a shortfall of 100–300 kt.

**Where that color is** (§D.51's census): mined out within 25 ly of the
stalled center; 50–80 ly away in piles; 11–42 ly away at centers holding it
above their own bills.

**Who gets it.** Each time a hauler prices a pile holding a stalled center's
missing color (`best_delivery_center`, at the pile), the stalled center won
10–55% of the pile tonnage priced, cumulative to 1,500 yr (seeds 1 and 7
`Random`, seed 1 `ColorRotated`); another center also wanting the color won
45–90%, a center not wanting it under 1.3%. The winner averaged 46–53 ly from
the pile against 50–63 ly to the nearest stalled center.

**The shipping backlog is empty.** From 600 yr on, every seat's backlog
(`Simulation::refresh_shipping`) reads 0 kt in every color while 6–94 Mt of ore
waits at outposts, and each seat has ordered 14,000–17,000 haulers by 1,500 yr.
Hauler count is not short.

**Two arms refuted.**

- *Price a pile at what the hold carries* (a pile capped per material at the
  hold's room before pricing): centers built **−8.2%** (`Random`) and
  **−5.0%** (`ColorRotated`), tree composite −2.80% ± 1.52 and −3.80% ± 0.75
  (seeds 1 and 7). Priced whole, the center wanting most wins and the hold is
  filled; priced at the hold, centers with small wants win and the last stop
  fills the hold beyond their want.
- *A works supply run* (an idle Reserve hauler fetches a stalled center's
  missing color from its empire's pile or a center whose abundance passes
  R-MX8's test; a Doctrine flag): 75 runs loaded **1.2 kt** by 600 yr on
  seed 7, because Reserve holds only small hulls, and choosing the largest
  Reserve hold reproduced the run exactly. Not landed.

**The rate, priced.** A General hold is 31.2 kt and a Band IV bill is 130–390
kt per color, so a missing color needs 4–12 holds. Over 50 ly a laden hull
flies at nearly `c`, so a round trip is at least 100 years whatever its drive.
**Inference:** one hauler serving one stalled center delivers about 0.3 kt/yr,
which is what the census measures; the delivery rate to a stalled center is
set by how many hauler round trips end there, and the levers on that are the
voyage discount `λ` (who wins a pile) and the stops a leg may make (what one
trip assembles). Confidence about 70%; §D.53 tests both.

## D.53 The voyage discount and the stops a leg may make, swept together

*Supports T-147, politics §1.4/§1.8 and industry §6.20. The author's
direction: sweep the levers behind the appendix's shifts larger than 2x, and
commit what improves the centers built. Bed: `examples/forge_sweep`
(`FS_LAMBDA`, `FS_STOPS`; twin bed, 3 seats, 1,500 yr, `completion_exponent =
0.75`). Score: the tree composite (§D.49) and **centers built** — owned worlds
whose works stand at Band IV at the horizon — both against `λ = 0.01`, `2`
stops on the same seed and ground. Screen on seeds 1, 7, 42, 31337, on
`Random` and `ColorRotated` ground. Of the appendix's >2x shifts, `λ`
(§A, 2.7x coverage) and the forge premium (§D.43, done at §D.49) are knobs;
the color-site spacing (§D.41) is the ground, and the backlog hauler (§D.47)
and the hull-sized hauler (T-98) are mechanisms already shipped.*

Mean over both grounds (each cell four seeds per ground):

| `λ` \ stops | 1 | 2 | 3 | 4 | 5 | 6 | 8 |
|---|---|---|---|---|---|---|---|
| 0.0025 | | −9.65% / −25.9% | | | | | |
| 0.005 | | −3.85% / −12.1% | | | | | |
| 0.01 | −43.26% / −65.0% | **0** | +2.98% / +10.3% | +4.58% / +14.9% | +3.36% / +14.2% | +4.42% / +16.4% | +2.62% / +13.8% |
| 0.02 | | +1.33% / +6.8% | | +6.86% / +25.0% | +7.69% / +26.4% | +7.57% / +29.3% | |
| 0.03 | | −1.67% / +3.1% | | +7.94% / +28.5% | +7.50% / +30.8% | +8.34% / +29.8% | |
| 0.04 | | | | +6.50% / +27.5% | +7.63% / +29.9% | **+9.12% / +30.7%** | |
| 0.05 | | −4.54% / −0.5% | | | | +7.94% / +26.8% | |

(composite / centers built.) Expansion falls with stops — about −0.9% at 3,
−1.7% at 4, −2.7% at 5, −3.6% at 6, −5.0% at 8 — and supers and apex forged
fall 60–85% in every arm with four or more stops. **The two levers interact:**
a sharper discount alone gains nothing past `0.02`, and more stops alone peak
near +4.6%; together they reach +9%.

**Replication** on seeds 2, 3, 5, 11, both grounds (16 runs per arm):

| arm | seeds 1, 7, 42, 31337 | seeds 2, 3, 5, 11 | pooled, n = 16 | centers built, pooled |
|---|---|---|---|---|
| `λ 0.04`, 6 stops | +9.12% ± 0.87, 8/8 | +8.47% ± 1.36, 8/8 | +8.79% ± 0.78, 16/16 | +28.2% ± 1.6 |
| `λ 0.03`, 4 stops | +7.94% ± 0.87, 8/8 | +6.88% ± 1.19, 8/8 | +7.41% ± 0.72, 16/16 | +25.5% ± 1.5 |

Paired, `0.04 / 6` minus `0.03 / 4`: composite +1.38% ± 0.60 (12/16), centers
built +2.76% ± 1.01, Expansion −1.40% ± 0.23 (16/16 lower).

**A mass leak the sweep exposed.** At `λ = 0.04` with three or more stops,
`a_twin_bed_builds_from_supers_with_mass_conserved` lost about 1.0 kt.
Checking the ledger after every event named the first loss: a `DutyArrive` at
723.5 yr, where cargo fell 0.1595 kt with nothing gained elsewhere. The hauler
had been parked in Reserve at 689.8 yr *laden*: a milk run of three or more
stops can come back to its own base, and the retirement test there (an
exhausted rock, nothing loaded at this stop) ignored what the earlier stops had
put in the hold; the side run that took it from Reserve wrote over the hold. At
two stops a leg's second stop is never its base, so the shipped engine could
not reach it. **Fixed:** a hauler retires only with an empty hold. `λ 0.01`,
2 stops reproduces the prior binary to every printed digit (seed 1).

**Shipped `λ = 0.04`, 6 stops**, measured on the fixed engine, 16 runs:

| ground | composite | centers built | Growth | Production | Expansion | supers | apex | Growth cv, paired |
|---|---|---|---|---|---|---|---|---|
| `Random` | +8.55% ± 1.23, 8/8 | +29.7% ± 2.4 | +26.2% ± 2.6 | +6.3% ± 1.1 | −3.7% ± 0.5 | −80.6% ± 8.3 | −81.6% ± 8.2 | −0.002 ± 0.020 |
| `ColorRotated` | +9.34% ± 1.02, 8/8 | +28.8% ± 1.7 | +28.0% ± 1.7 | +6.0% ± 1.5 | −2.5% ± 0.7 | −78.3% ± 14.1 | −75.8% ± 11.5 | −0.040 ± 0.021 |
| both | **+8.95% ± 0.78, 16/16** | **+29.3% ± 1.4** | | | | | | |

**Cost.** Seed 1, 800 yr, one run each: 589,595 → 712,729 events (+21%) and
44,247 → 64,443 ns/event (+46%), 26.1 → 45.9 s. More work and dearer work;
the dearer part is `next_pickup`'s scan of every pile and center, run up to
five times per leg. Test targets on this container: unit 27.3 s, smoke 13.2 s,
telemetry 46.6 s, determinism 49.0 s.

**Inference, stated as one:** a sharper discount sends each pile's color to the
nearest center that wants it, and more stops let one leg assemble the colors a
bill lacks from several piles; together a leg finishes more bills per round
trip, and the forges, which want basics without end, receive less. Confidence
about 60%; a per-leg census of how many bills each delivery completes, at both
settings, would test it. The supers and apex lost are the cost the tree
composite does not price (§D.49's caveat).

## D.54 Freight routed by `$` at every stop: four arms, none landed

*Supports T-147. The author's direction: the missing unlock is dynamic
routing priced in `$`, not stop count; reprice the next leg at each stop,
deciding whether to take on cargo; throughput is not the goal until the
scheduling is found. Bed: §D.53's (twin bed, 3 seats, 1,500 yr, `λ = 0.04`),
scored against the shipped engine (`max_pickup_stops = 6`) on the same seed.
Each arm is a scratch build (never landed); the patch is kept outside the
repository. A scratch census (`freight_census`) splits freight into deliveries
× kt per delivery and reports the fleet every 250 yr.*

**Arm 1 — the next pile by the empire's posted price.** After each delivery a
hauler goes to the pile whose one hold is worth most at its empire's
want-weighted color prices, discounted by the empty leg; a claim book keeps
haulers off a pile already claimed, and the pile becomes its base.

| stops | composite | centers built | runs |
|---|---|---|---|
| 6 | −6.87% ± 1.18 | −27.0% ± 3.1 | 0/8 positive |
| 2 | −23.33% ± 2.06 | −58.5% ± 4.0 | 0/8 |

Decomposition (seeds 1 and 7, `Random`, per seat, against shipped):
deliveries +65–80%, kt per delivery 6.0–8.1 → 3.0–5.4, total kt delivered 0
to −22%, ore left at outposts −30% to −75%, leg lengths unchanged (30–39 ly).
The stockpiles move; each hold carries half as much.

**Arm 2 — the same, with the four best piles priced at their real buyer
over both legs.** −6.11% ± 1.40, centers built −28.9% ± 3.3, 0/8.

**Arm 3 — the next leg priced at every stop** (the author's direction):
deliver now, or detour to one of the three best piles and deliver the larger
load — whichever is worth more at its best buyer's prices, discounted over
every leg; each pile loads only what that buyer wants; stop cap 16. Seed 1,
500 yr, against shipped: Growth 54k → 371k, 53k → 231k, 71k → 83k kt-years
per seat; Expansion +6% to +51%; centers built 3 → 9; cost 69 s against 6 s.
At 1,500 yr (4 seeds, `Random`): **composite −41.50% ± 3.59, centers built
down about two thirds, 0/4.** The time series (seed 1) says when:

| per seat | shipped | arm 3 |
|---|---|---|
| freight, 250–500 yr | 5.5–10.2 Mt | 19.2–23.5 Mt |
| freight, 750–1,000 yr | 112–133 Mt | 66–82 Mt |
| freighters at 1,000 yr | 9.1k–9.9k | 12.4k–15.8k |
| miners at 1,000 yr | 1.8k–2.6k | 1.6k–1.7k |
| kt per delivery, 750–1,000 yr | 5.9–6.5 | 2.1–3.2 |
| ore at outposts, 1,000 yr | 53–95 Mt | 15–46 Mt |

**Arm 3 with two fleet-size rules.** The hauler order priced a per-rock
backlog net of the haulers *based* at each rock, and a roaming hauler moves
its base every trip.
- *The whole fleet against the whole stock:* freighters fall to about 2,200
  per seat, ore at outposts climbs to 128–158 Mt by 1,000 yr, freight 22–27 Mt
  per 250 yr.
- *A new hauler priced by the route it would fly* (the order fires when a
  fresh hauler's best move from the center is worth more than the hull):
  freighters reach 26k–30k per seat by 750 yr, 0.6–2.9 kt per delivery.

**Inference, stated as one:** pricing each leg at its buyer routes freight
better than the welded base — arm 3 delivers two to four times as much in
its first 500 years — but nothing in these arms prices the *size of the
fleet* correctly once haulers roam. A route's value says what one more trip
earns now, the backlog compares a stock with holds per trip, and the claim
book reserves only piles that haulers are flying to; none of them sees the
rate at which mining refills the piles, which is what a marginal hauler
competes for. Confidence about 60%; a fleet-size rule priced against the
piles' refill rate (or a per-cycle assignment of haulers to piles and
buyers), measured on the same bed, would test it.

## References

- `AGENTS.md` §2 — how to search, how to read a gradient, the six traps, and the
  rule that sends entries here
- `docs/Hyades_autopilot_colonization_growth.md` — the Expansion/Growth spec §A
  supports
- `docs/Hyades_politics_trade_and_intelligence.md` — the Politics spec §B supports
- `docs/Hyades_industry.md` §6.8–6.26 — the industry branch's own experiment
  record, which has **not** been moved here yet and is the largest remaining
  instance of the problem this file solves
- `docs/hyades_todo.md` — the open register; an appendix entry is evidence, a
  todo entry is work
