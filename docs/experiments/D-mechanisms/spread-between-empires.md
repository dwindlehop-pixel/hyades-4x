# §D. Mechanisms — the spread between empires, and pricing by holdings

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

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

---

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

---

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

---

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

---

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

---

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

---

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

---

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

---

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

---

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

---

## D.57 The spread of colony-years between empires on random ground

*Supports galaxy §2 and T-147. The author's direction: reduce the variation in
colony-years per empire on random ground — by Exchange pricing, by sweeping
`trade_decay_lambda`, or by constraints on galaxy generation — until it is no
more than 1.5x the lowest variation measured on identical or color-rotated
ground; then, the variation in whole Band IV works per empire. Bed: the twin
bed (`examples/forge_sweep`, 3 seats, 1,500 yr, card-free, shipped freight
doctrine), seeds 1, 7, 42, 31337, 2, 3, 5, 11. The spread is the coefficient of
variation (standard deviation over mean) of a per-seat stock across the seats
of one galaxy, averaged over the eight galaxies, ± its standard error; "paired"
differences are per galaxy against random ground on the same seed. Every arm
but the shipped one is a scratch configuration of galaxy generation or of
`SimConfig` (beds vary only the galaxy).*

**The target, as read.** The lowest mean spread of colony-years on the two
symmetric grounds is identical ground's, 0.055 ± 0.012, so the target is
**0.083**. The pooled reading (one coefficient of variation over all 24
seat-runs of a ground) puts random ground at 0.108 against rotated ground's
0.073, under 1.5x already; it mixes the spread between galaxies, which every
seat of one game shares, into the spread between seats, so it is not the
reading used.

**Baseline** (shipped engine):

| ground | spread of colony-years | spread of Band IV centers |
|---|---|---|
| random | 0.098 ± 0.016 | 0.137 ± 0.040 |
| identical | 0.055 ± 0.012 | 0.787 ± 0.202 (2.7 Band IV centers per seat) |
| color-rotated | 0.060 ± 0.005 | 0.126 ± 0.022 |

So the Band IV spread on random ground is 1.09x color-rotated ground's and
meets the 1.5x reading; colony-years is the one above it.

**When the seats part** (`examples/seat_race`, the same bed to 400 yr): a
seat's colony count at 150 yr, relative to its galaxy's mean, correlates with
its colony-years at 1,500 yr at r = 0.84 (24 seats). At 50 yr the spread of
colony count is 0.27 on random ground and 0.02 on both symmetric grounds; by
150 yr the symmetric grounds have reached 0.16–0.17 too, and they fall back
to 0.10–0.12 by 400 yr while random ground holds 0.20. What each seat's region
holds at generation (`examples/seat_ground`: worlds `k_high` admits, their
distance-weighted count, deposits by color within 8–40 ly and over the whole
region) correlates with the 150-yr count at |r| ≤ 0.53. Each seat's region is
93–99% one color.

**Arms on random ground** (paired against random ground):

| arm | spread of colony-years | Δ | spread of Band IV centers | Δ | Growth | composite |
|---|---|---|---|---|---|---|
| `trade_decay_lambda` 0.02 | 0.094 ± 0.014 | −0.004 ± 0.011 | 0.128 ± 0.050 | −0.009 ± 0.017 | −0.6% | −0.3% ± 0.4 |
| `trade_decay_lambda` 0.08 (5 seeds) | 0.110 ± 0.029 | −0.006 ± 0.016 | 0.152 ± 0.060 | +0.007 ± 0.012 | −3.5% | +0.6% ± 0.4 |
| fair start 20 ly, colors stepped | 0.097 ± 0.014 | −0.002 ± 0.025 | 0.129 ± 0.035 | −0.009 ± 0.024 | +8.8% | +5.6% ± 2.9 |
| **fair start 35 ly, colors stepped** | **0.081 ± 0.023** | −0.017 ± 0.031 | 0.153 ± 0.024 | +0.016 ± 0.043 | **+12.1%** | +5.2% ± 8.1 |
| fair start 35 ly, same colors | 0.050 ± 0.013 | −0.048 ± 0.019 | 0.220 ± 0.063 | +0.083 ± 0.075 | −73.2% | −37.9% ± 6.2 |
| color sites kept 20 ly from homeworlds | 0.113 ± 0.032 | +0.015 ± 0.025 | 0.187 ± 0.048 | +0.049 ± 0.026 | −19.2% | −14.5% ± 3.5 |
| color sites kept 35 ly from homeworlds | 0.075 ± 0.015 | −0.023 ± 0.022 | 0.211 ± 0.033 | +0.073 ± 0.054 | −67.3% | −48.8% ± 3.4 |
| color-centered homeworlds (equidistant sites) | 0.335 ± 0.063 | +0.237 ± 0.077 | 0.194 ± 0.050 | +0.056 ± 0.016 | −2.2% | −1.0% ± 0.8 |

The fair start (`GalaxyConfig::fair_start_ly`) makes every seat's wild worlds
within the radius seat 0's, carried to the seat and color-stepped as the
archetypes step; "same colors" is a scratch build that does not step them. The
color clearance was a scratch `GalaxyConfig` field, never landed: no random
color site within the distance of any homeworld.

**What the stepped fair start leaves.** On seeds 2 and 3 its spread rises to
0.185 and 0.186: the seats part by 150 yr inside their copied starts (seed 3:
43 / 26 / 61 colonies, none in a rival's region before 200 yr). The starts
differ only in color, against one works mix for every seat (2 : 1 : 3 Cyan :
Magenta : Yellow, ratified); seats whose start is Magenta average 0.945 of
their galaxy's mean colony-years and Cyan 1.050 (7 seats each, about 2
standard errors apart; Yellow 1.005, which the works mix alone would rank
first).

**The homeworlds placed by hex** (the author's rulings, galaxy §2): at 3 seats
with the 25-ly inset, the ground census of all eight seeds and the twin-bed
runs of seeds 1 and 7 to 300 yr are bit-identical to the ring they replace, so
every 3-seat number above holds under the new placement.

**Inference, stated as one:** the spread on random ground is set by the first
few foundings, and what separates them is not any one stock near the
homeworld but the order in which a seat's ground lets it found — which no
freight price reaches, since the race is decided before 150 yr when freight
is small. Confidence about 60%. Fixing the ground near home either removes
color from the economy (same colors, clearance: Growth −19% to −73%) or leaves
the color asymmetry against the shared works mix; a stepped fair start on
eight more seeds would say whether its 0.081 is under the target or at it.
