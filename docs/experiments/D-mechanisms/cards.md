# §D. Mechanisms — cards and the Technology objective

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

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

---

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

---

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

---

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
