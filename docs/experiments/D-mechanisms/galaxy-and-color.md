# §D. Mechanisms — color on the galaxy

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

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

---

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

---

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

---

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

---

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

---

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
