# §D. Mechanisms — combat, Designs and their ratings

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

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

---

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
