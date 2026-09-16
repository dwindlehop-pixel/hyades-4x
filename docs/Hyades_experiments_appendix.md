# Hyades — Experiments Appendix

*The measurement record behind the specs. **Nothing here is normative.** A spec
says what the engine must do; this file says what was run, on which bed, and
what it showed — including the runs that were wrong and the design that was
retracted.*

---

## 0. Why this file exists, and how to use it

`CLAUDE.md` §6 splits a spec into **decisions the engine must honour** and
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
through the centre that built them).

**All three land within 2 SE of each other** — 2,765–2,865 yr mean. No
significant difference.

**Why, diagnosed rather than assumed** (`examples/colonization_ramp_trace.rs`):
a homeworld crosses the medium-hull gate at t≈200 yr (seed 1), and known
candidates already vastly outnumber what the treasury can afford that cycle — 98
candidates by t=200, 151 by t=400, against a coloniser costing ~0.22 minerals
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
already targeted, which in a colonised galaxy is the common case. So both survey
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

**Before optimising a hot path, check what fraction of the work it does produces
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
> of one thing, freight not moving colour (R-O89/R-O92, §B.4 and
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
and the next centre ordering a pair takes the reserved hulls nearest its target.

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
  in Reserve, so a centre too poor for a new pair sat Idle beside hulls it
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
independent, individually-real improvements mostly cancelling because they
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
biosphere — so centres ran out of deepening headroom and spent minerals on
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

**Resolving it moved the diagnosis rather than the behaviour.** Both sides are now
`rank` score per kilotonne committed. Measured (`examples/deepen_census`, seeds 1
and 7) the run is **bit-identical below `b = 0.96`**, and the branch is **still
cold at the shipped 0.5** — because an infra rung above the founding one costs
0.9 kt against a Medium coloniser's 0.10 kt, so expansion returns 24–49× per
kilotonne and *ought* to win.

**The dead branch was the right answer reached for a wrong reason.** Had the fix
landed without pricing what the branch would have chosen, the next step would
have been to tune a dial toward a decision the economics say is bad.

## A.11 R-O87 — `reinvest_bias` cannot move Growth's objective, by identity

Work-years is `∫ Σ_p infra_p dt`, and the two things `reinvest_bias` chooses
between are worth the same to it: deepening bills `infra_step_price / eta_works`
and raises works by `infra_step_price`, while founding bills the coloniser's
price and the new colony's stock is `founding_infra = hull_cost` — the recycled
hull's minerals *are* the stock, because a hull's mass is its cost. At the
card-free `eta_works = 1` those are identical to the last bit, at every rung.

Measured to match: **+0.32% ± 1.42 over eight seeds** at 4,000 yr.

**And the replication is why that number is trusted.** The standard four-seed bed
gave `b = 0.972` at **+2.33% ± 0.96 with 4/4 seeds positive** — 2.4 SE *and* a
1-in-16 sign test. On seeds 2, 3, 5, 11 it scores **−1.70% ± 2.42**, 1/4 positive.
Neighbours swing the full magnitude in both directions (0.968 → −0.20% ± 0.76,
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

Replacing it with the deciding centre's live shortfall moved the mechanism check
from **0.043 to 0.043**, cost **−3.30% ± 0.49 colony-years on 0/4 seeds**, and
was reverted.

**The premise was refutable before a line was written.** The empire's outpost
holdings are **957k / 905k / 626k kt** across the three colours — already
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
centre's mining fraction. Shrinking the tick did not integrate the same economy
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
never only an integration step: it also quantises when a centre mines, crosses a
band edge and re-decides. **Refining a step that carries more than one job
improves all of them; fixing the integrator pays for one.**

Final: **+6.57% ± 1.14 colony-years, 8/8 seeds**, throughput unchanged.
`growth_rate = 0.873` is **carried, not re-ratified** — its operating point and
its plateau map are both consumed.

---

# §B. Politics, Trade and Intelligence

*Moved out of `docs/Hyades_politics_trade_and_intelligence.md` at Rev 2. That
spec had reached 1,125 lines, roughly half of it implementation history.*

## B.1 R-P2 — λ, and a trade mechanism that paid for itself before trade existed

The transit burn was proposed as a `$` sink that happened to give the
travel-time behaviour the brief asked for. The ratification condition was
stronger: it had to also be *the* solution to freighter routing.

Before it, a laden freighter picked the highest-pressure owned centre with **no
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
stop serving genuinely needy distant centres; above it, need swamps distance again
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
default when the seller's bank is short the colour it owes, and conserve mass.
**64,642 contracts settled and 3,484 defaulted** (seed 1); **64,553 and 3,905**
(seed 7). Only *geography* rejected anything — 47 and 31 fills out of ~68,000
found no shared rock.

**Yellow dominates the flow, which is the design goal arriving:**

| colour | seed 1 delivered | seed 7 delivered | share |
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
not trading, regardless of which colour moves where.

That is checkable and must be checked before anything is tuned: compare banked
against piled holdings over time, and measure the dwell between a contract
settling and its ore reaching a bank. **A plausible mechanism attached to a real
number is the shape of every measurement artifact in this project.** Carried as
**R-P18**.

## B.4 The stage plan predicted the wrong risky stage

| # | Stage | Predicted | Actual |
|---|---|---|---|
| 1 | `$` ledger + faucet | neutral | neutral, bit-identical |
| 2 | `Commodity` gains colour; `Offer` gains an owner | neutral | neutral, bit-identical |
| 3 | Cross-empire book; centres post `wtp` | neutral | neutral, bit-identical |
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
they share more than one? The answer written first minimised the two parties'
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
classification co-extensive with a colour domain)?" The earlier draft carried a
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
| 1 | `medium_fleet_size = 8` optimal, 12 a "cliff" | the capacity normaliser going to zero | — |
| 2 | `coverage_trace`: this knob "DID move it" | two of three sample points degenerate | — |
| 3 | `coverage_time`: cheaper colonisers at 6.0 | a General hull holding ~700× a Medium's | — |
| 4 | coverage "wants" a cheaper Medium hull | `cap_Medium` pinned by a live normaliser | — |
| 5 | `survey_reserve` is "significant" at −23.8 ± 10.1 | a ±10% probe on a plateau, clearing 2 SE by luck | A.3 |
| 6 | time-to-10% and coverage disagree on a knob's sign | two gradients taken at different operating points | A.2 |
| 7 | a dwell metric *rose* under the policy that founds colonies 399 yr earlier | the mix moved: hull-first share 55.0% → 97.4%, both components fell | — |

Shapes 1–4 share a cause: **the quantity that broke was in a denominator the
shipped autopilot never exercised**, so none of them was visible in the
objective. Shape 7 is the only one where **nothing was broken** — the
measurement was correct, the population it averaged over was not the same
population, and the sign it reported was the opposite of the mechanism.

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
