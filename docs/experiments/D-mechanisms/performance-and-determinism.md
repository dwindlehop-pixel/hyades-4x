# §D. Mechanisms — throughput and cross-target determinism

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

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

---

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

---

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

---

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
