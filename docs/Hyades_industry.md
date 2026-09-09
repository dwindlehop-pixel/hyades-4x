# Hyades — Industry: Infrastructure, the Mining Ramp, and the Production Ramp

*The normative spec for **what a planet's built capacity is, what it produces, and
how a player invests in it.** Companion to `Hyades_simulation_model.md` §2a (the
planet model, which this document amends), `Hyades_mineral_cost_curve.md` §1 and
§2.6 (card costs and the Band ladder), `Hyades_standing_layer_and_observation.md`
§5 (Doctrine and Design as state), `Hyades_politics_trade_and_intelligence.md`
(the Exchange), and `Hyades_galaxy_and_autopilot.md` §4.8 (the colour → tree
spine). Calls flagged **R-IND\<n\>**.*

*Rev 1. Nothing here has been Monte-Carlo ratified. Every magnitude is a
**placeholder** and says so. What is being agreed is **shape** — the algebra, the
invariants, and what each quantity means — because those are the parts a later
measurement cannot fix.*

---

## 0. What this settles

Three gaps, named plainly:

1. **Infrastructure is not differentiated in minerals.** `infra_step_price`
   returns a single total drawn proportionally from whatever colours a stockpile
   happens to hold. The mineral distribution the galaxy spends so much effort
   creating (§4.3–4.4, and T-62's log-normal field) therefore **does not bite on
   development at all** — only on card costs.
2. **Mining rate and production rate are not investable.** They are `SimConfig`
   constants. A planet cannot get better at either, no card can move them, and
   the autopilot has nothing to buy. There is no ramp.
3. **A production centre cannot develop a colony it did not just found.** The
   only path from one world's industry to another's Infrastructure is recycling
   the coloniser's hull on arrival (R-O76). After that the colony is on its own.

And it takes the decision that unblocks all three: **Infrastructure leaves the
carrying-capacity minimum entirely** (§1).

**One rule in here outranks the rest, and it is not about industry at all:
refined mass traverses real space** (§8.1). Minerals, supers and apex cross the
theater on hulls, under light-lag, where they can be attacked, diverted, stolen
and blockaded. Every price gradient in §5 and every market in §7 is downstream of
it — take traversal away and piracy, theft, blockade and conquest stop being
alternatives to trade and become flavour text.

---

## 1. Infrastructure is capacity, and *only* capacity

### 1.1 The amendment — `K = min(habitability, bio_max)`

**Ratified this conversation, amending `Hyades_simulation_model.md` §2a:
infrastructure is removed from the carrying-capacity minimum.** The planet model
becomes **two** ceiling factors and one industrial stock:

| Factor | Role | Attackable? |
|---|---|---|
| **Habitability** | ceiling — gravity / radiation / thermal | yes; a hab strike crashing population is **intended** |
| **Biosphere (`bio_max`)** | ceiling — pristine living capacity | yes; a bio strike crashing population is **intended** |
| **Infrastructure** | **industrial stock** — mines and fabricates (§2) | yes, and it **must not touch population** |

So `K = min(hab, bio_max)`, and Infrastructure caps nothing.

**Why.** Infrastructure is the thing this design wants to be attackable *without*
killing people — that is the whole reason it is the primary war target (§1.2).
While it sat inside `K`, it could not be: T-64's discrete logistic
`x + r·x·(1 − x/K)` goes strongly negative above `K`, so a population whose
ceiling is cut does not settle down to the new one, it **overshoots below it**.
Measured and pinned: `a_colony_seeded_above_its_capacity_crashes_below_it` has a
population at `2K` landing at `0.25K` in a single step. An Infrastructure strike
deep enough to matter was a population strike with extra steps.

Removing the term is the clean fix rather than softening the logistic, and it is
cleaner than the alternative in two ways. The crash **stays** where it is wanted:
a habitability or biosphere strike still collapses a population, because those
genuinely are the world's capacity to hold people, and a weapon that poisons a
biosphere *should* be terrible. And nothing has to be tuned — no decline rate, no
second time constant, no clamp. **A clamp that turns a broken model into a
high-scoring one is exactly the failure T-64 records**; this removes the need for
one rather than adding another.

### 1.2 Why Infrastructure is the war target

Design pillar: *love/cooperation wins over deep time* (Traulsen & Nowak, PNAS
2006). A game whose thesis is cooperation cannot have genocide as its only
military verb. *Stars!* bombing does not distinguish — population, defenses,
mines and factories die together — so the only way to reduce an enemy's output is
to reduce its enemy.

With §1.1 in place, Hyades has the other verb. An Infrastructure strike degrades
what a world **makes** and leaves who lives there **untouched**: it is a tempo
weapon, cheap to inflict and cheap to repair, and it makes both branches of
`sim §2a`'s "take it intact or crater it" real choices rather than one choice and
one euphemism. Genocide remains available — through habitability and biosphere —
and is now a *distinct, more terrible decision* rather than a side effect of
attacking a factory.

### 1.3 One stock, one price, three employments

The instinct to bundle rather than split is right, and it is worth saying why
rather than leaving it as taste. *Stars!* prices mines, factories and defenses
separately, and the consequence is a known degenerate archetype: the
**factoryless hyperexpander** (`-f`), who declines one of the three cost curves
and is rewarded for it. Three independently-priced stocks means three
independent opt-outs, and the strongest builds opt out of two.

**So: one stock, one price, three employments.** A planet buys *Infrastructure*;
what it is *doing* is a Doctrine allocation (§2), revisable and free of charge.
You cannot decline to build the thing that mines while building the thing that
fabricates, because they are the same thing pointed differently. One strike
degrades all three at once — the *Stars!* bombing feel, without the *Stars!*
population kill.

**Infrastructure is stored as a mass in kilotons.** It is built out of minerals
and minerals are masses (L6/R-O57); a Band is a *reading* of it for display and
for thresholds, never a second thing to store (T-64, `CLAUDE.md` §4).

### 1.4 What the amendment costs, in engine terms

> **Landed (T-67).** Measured on the standard bed: **+4.4% / +4.7% colony-years**
> (seed 1 9,139,231 → 9,542,958; seed 7 9,060,095 → 9,483,962) with the colony
> **count identical** on both seeds. The same worlds, taken earlier — colonies
> now grow to `min(hab, bio_max)` instead of being pinned at whatever
> infrastructure their founding hull happened to leave, so they cross the
> `PopBands` production gates sooner and the expansion loop compounds faster.
>
> **It also cost a rule, and finding out how was the useful part — see §1.5.**


`Factors::k()` is `k_potential().min(infra)` and `k_potential()` is
`hab.min(bio_max_band)`, so the change is **deleting one `.min()`** — after which
`k()` and `k_potential()` are the same function. Three real consequences, none of
them cosmetic:

- **`founding_infra` (R-O76) stops capping the founding seed — confirmed
  intended.** A coloniser's recycled hull currently sets the new colony's `K`, so
  a Medium founds at `Band I` and a General at `Band II`. With Infrastructure out
  of `K`, **hulls seed to the world's own ceiling** and the hull no longer gates
  the colony's opening population at all. The recycled hull still lands as
  **industrial stock**, which is now its whole job.

  This is a substantial buff to founding and it deletes the question R-O76 was
  asked to answer. R-O76 measured that *seed depth does not pay* — a `Band II`
  seed on a `Band I` colony crashing to `0.25` Bands in its first tick — and that
  finding was entirely about the mismatch between what a hull carried and what
  the colony could hold. Both hulls now deliver to the same ceiling, so there is
  no mismatch and **the old number describes a mechanism that no longer exists.**
  Do not carry it forward; the hull-choice question is now purely one of price
  and hold size.
- **The deepen/expand trade changes meaning.** `deepen_headroom` is
  `k_potential − infra` (R-O68), which under the amendment is not a headroom at
  all. Deepening no longer raises a ceiling; it raises **rates**. The branch was
  already provably dead at the shipped `reinvest_bias` (R-O68), so this is a
  chance to rebuild it rather than repair it.
- **`medium_min_level` and the production gates are untouched** — they read
  population level, not infrastructure.

### 1.5 R-V9 was being enforced by accident, and the amendment exposed it

**"A Colonizer must be Medium or larger" was not implemented anywhere.** It fell
out of two unrelated rules meeting: `founding_capacity` was
`k_potential.min(infra.max(founding_infra(hull)))`, a Limited hull's
`founding_infra` is `Band Empty`, and `population_mass` maps `Band Empty` to
**exactly zero** — so a Limited hull's seed was zero and the hull was refused.

Nothing named that. Take infrastructure out of `K` and the coincidence
dissolves: a Limited hull founds a colony of 0.089 kt, quietly, in a build
nobody changed on purpose. It was caught by a test that asserted the *outcome*
(`colony_seed_for(Limited, …) == None`) rather than the mechanism — which is
the argument for writing tests at that level, since a test of the mechanism
would have been deleted along with it.

**The rule is now stated where roles §4 says it lives** — as *capability, not
competence*: a hull founds nothing unless its **hold** can carry a full
`colony_seed_pop`. A Medium hull's hold is `Band I` exactly and a Limited hull's
is `Band Empty`, so the ratified rule and the geometry agree without either
propping up the other.

**The general lesson, which is not about colonisation.** An invariant that holds
because two unrelated expressions happen to intersect is not an invariant, it is
a coincidence with good luck. This one survived a units migration, a ladder
ratification and a cost-model rewrite before the amendment finally broke it.
Where a design document states a rule, the engine should contain that rule —
searchable by the words the document uses — and not a derivation that produces
the same answer for now.

### 1.6 R-IND11 measured — the gain is the seed, not the hull, and it is conjured

**The measurement.** `examples/colonizer_policy`, CRN over seeds
`[1, 7, 42, 31337]`, 3 seats, 4,000 yr. `SettlersPerMineral` against
`CheapestViable`:

| | colonies | colony-years | mean founding | mean flight | General share |
|---|---|---|---|---|---|
| `CheapestViable` | 3,365.5 | 9,609,694 | 1,145.0 yr | 113.5 yr | 0.0% |
| `SettlersPerMineral` | 3,365.5 | **10,951,730 (+13.97%)** | **746.2 yr (−398.8)** | 109.5 yr (−4.0) | 90.4% |

Colony **count is bit-identical on every seed** — 3,337 / 3,346 / 3,333 / 3,446
in both arms — so on the stated objective the policy is worth nothing. The whole
effect is in the guard: the same worlds, taken ~400 years earlier.

**The proposed mechanism was transit, and it is refuted.** The case for a
General coloniser was that a deep seed becomes a forward base sooner and so
shortens every later colonising voyage. Mean flight time (coloniser spawn →
founding) is **107.6–116.7 yr in all twenty-four measured rows** and moves −4.0
yr between the arms. A −4 yr change in flight cannot produce a −399 yr change in
founding. Forward bases are not what is happening.

**Two ablations name the cause, and they close from both sides.** Each is a
one-line change to `colony_seed_for`, run on the same bed:

| ablation | `CheapestViable` | `SettlersPerMineral` | gap |
|---|---|---|---|
| — (shipped) | 9,609,694 | 10,951,730 | +13.97% |
| **A**: every seed forced to the *General's* hold, prices untouched | **10,988,611 (+14.35%)** | 10,958,812 | −0.27% |
| **B**: every seed forced to the *Medium's* hold, prices untouched | 9,609,694 | **9,601,637** | −0.08% |

Under **A** the entire gain reproduces on a policy that **never builds a single
General hull** — cheap Medium hulls, General-sized seeds. Under **B** the policy
still buys General hulls for 91.9% of its colonisers, pays 10× each, and lands
back at the baseline. So the cause is **the seed mass alone**: not the hull, not
its price, not transit.

**A side result worth keeping.** Ablation B is a controlled 10× overpayment on
every coloniser hull for no benefit whatsoever, and it costs **−0.08%**. Minerals
are not the binding constraint at the shipped defaults — which is
`examples/reach_limit`'s finding (`k_high` binds, not the economy) arriving from
an unrelated direction.

**The dwell metric moved the wrong way, and the reason is a mix.** Time from a
colony's founding to its own first applied build *rose* 253.5 → 335.8 yr under
the faster policy. Decomposed by what that first build was, **both components
fell**: infrastructure-first 72.4 → 66.3 yr, hull-first 401.4 → 342.9 yr. A
weighted mean can only rise while both group means fall if the weights move, and
they do: the hull-first share is **55.0% → 97.4%**, measured (`hull1st`), and
per-seed monotone — 54.2/54.1/55.0/56.6 against 96.9/98.3/97.5/97.0. Solving the
two-group mean for the weight independently gives 55.05% and 97.43%, so the
decomposition accounts for the aggregate exactly rather than approximately.
Deep-seeded colonies **skip the pre-`medium_min_level` staircase** and go
straight for a hull they must save for. That is the pathway from seed mass to
earlier founding, and the aggregate was hiding it.

> **A new artifact shape for `CLAUDE.md` §2's table: an aggregate that moves
> against every one of its parts.** Nothing was wrong with the dwell
> measurement; it was a correct mean over a population whose composition the
> treatment changes. Any metric averaged over a set the intervention re-selects
> needs its mix reported beside it, or it will report the opposite of the
> mechanism — which is exactly what it did here.

**Why this does not ratify anything: the settlers are conjured (R-O74).**
Nothing debits the founding centre's population or biosphere for the people put
aboard a coloniser. So "deliver a bigger seed" is not a strategy the economy
pays for — it is **free mass**, and the +13.97% is a measurement of how much a
policy can extract from an open design-law-#11 violation. It scales with the
hold because the hold sets how much is created from nothing. Ratifying
`SettlersPerMineral` on this number would be ratifying the exploit.

**Decision.** `Doctrine::colonizer_policy` stays **`CheapestViable`**. R-IND11 is
not resolved by this measurement; it is **reframed and blocked on R-O74** — the
hull-choice question is unanswerable while founding mass is free, because any
policy that delivers more free mass wins by precisely the amount it conjures.
Draw the seed from the origin first, then re-run this harness.

> **R-O74 is now closed (§1.7), and the re-measurement is below.** The numbers
> above are kept because the *method* is what the section is for — two ablations
> refuting two plausible mechanisms — and because the +13.97% is the best
> available measurement of how large the violation was.

#### Re-measured under conservation — about 30% of it was the conjuring

Same harness, same CRN bed, settlers now drawn from the origin:

| | conjured | conserved |
|---|---|---|
| colony count | +0.00% | +0.00% |
| colony-years | +13.97% | **+9.79%** |
| mean founding | −398.8 yr | −279.8 yr |
| General share of colonisers | 90.4% | **63.3%** |
| mean flight (transit) | −4.0 yr | −0.9 yr |
| mean dwell | **+82.3 yr** | **−67.2 yr** |
| hull-first share | 55.0% → 97.4% | 55.0% → 79.4% |

Three things to take from it. **Transit stays refuted** — flight is flat in every
arm measured, conjured or not, so the forward-base story was never the mechanism.
**Seven tenths of the effect is real**: moving people from a mature centre sitting
at its `K` to a new world far below one is worth something on its own, because
`x + r·x·(1 − x/K)` is near zero at the top and fastest at `K/2`. Population
wants to be on the frontier, and that is a genuine strategy rather than an
exploit. And **the dwell metric stopped inverting**: with the mix shift smaller
(55% → 79% rather than → 97%) gate-skipping now shows in the aggregate as well
as in the decomposition, which is the §1.6 artifact box arriving at its own
prediction from the other side.

**Still not ratified.** T-68 makes a General hull a twelve-year yard commitment
against a Medium's three (§3.3), and unlike before, the hull *is* now part of the
mechanism — a General hold is what makes a large transfer possible at all. So
this is the measurement to repeat after the ramp lands, not a default to move
before it.

**And it puts a question on T-67's own number.** The amendment let hulls seed to
the world's ceiling instead of to `founding_infra`, which is the same channel —
so some part of T-67's +4.4%/+4.7% may be the same conjured mass rather than the
faster gate-crossing §1.4 attributes it to. That is a hypothesis, not a finding:
the test is ablation A run against the pre-T-67 seed rule, and it has not been
run.

**T-68 is second-order here.** The approved schedule makes a General hull a
twelve-year yard commitment against a Medium's three (§3.3), which would blunt a
General coloniser — but the hull is not what is producing the effect, so the
schedule change cannot decide R-IND11 either.

---

### 1.7 Settlers are drawn from a real population, and the hold carries a mix (R-O74, resolved)

**Ruled by the author, and it closes the block §1.6 opened:** *"Do not conjure
settlers from nothing. They must be taken from the population of some other
world. The cargo hold needn't be filled with space or pop. It can hold a
combination of pop and minerals to jumpstart production."*

#### What was wrong

`spawn_courier` wrote `pop_cargo` and debited nothing. A coloniser's founding
population was created at launch, so **design law #11 had exactly one
exemption and it was the one the expansion loop runs on.** The size of it was
measured before it was fixed, which is the useful part: the policy that shipped
the biggest seed scored **+13.97% colony-years** on a bit-identical colony
count, and two ablations put the whole effect on the seed mass rather than on
the hull, the price or transit (§1.6). It scaled with the hold because the hold
set how much was invented.

#### Terms

Defined before use, because the previous revision of this section did not and
one of its two load-bearing lines was unreadable as a result (`CLAUDE.md` §6):

| symbol | name | unit | where it comes from |
|---|---|---|---|
| `x_p` | origin's current population | kt | `World::population` |
| `K_p` | origin's carrying capacity | kt | `population_mass(k())`, and `k = min(hab, bio_max)` since T-67 |
| `K_c` | destination's carrying capacity | kt | same function, at the target |
| `x_0` | floor a colony starts from | kt | `units::POPULATION_SEED_FLOOR`, one tonne |
| `H` | the hull's hold | kt | geometry, `HullType::colony_seed_capacity` |
| `S` | settlers put aboard | kt | derived below |
| `E` | mineral endowment put aboard | kt (as `Price`) | derived below |
| `tau` | one-way transit, origin → destination | yr | `math::ship_travel_years` at civilian accel |
| `delta` | discount on a gain arriving `tau` late | — | `1 / (1 + tau / cycle_years)` |
| `r` | logistic growth rate per cycle | 1/cycle | `Doctrine::growth_rate` |
| `g(x, K)` | logistic rate, `r·x·(1 − x/K)` | kt/cycle | the engine's own growth step |
| `I_0` | founding infrastructure (the recycled hull) | Band | `founding_infra(hull)` |
| `I*` | build-out the site is worth developing to | Band | §5 works value; placeholder below |
| `D` | destination's mineral abundance | Band | `MineralField::abundance()` |

#### The rule

A coloniser is **loaded out of its origin**, and the load is one kiloton budget
because both halves mass the same (R-O32). Population is debited at launch and
credited at founding; the minerals leave the origin's stockpile and land in the
new colony's. A **contested** coloniser unloads both halves back into its home
centre when it bounces — §4.2's "nothing is lost" was a reassurance and is now
an entry, because a hull parked while laden would hold that mass out of the
economy permanently.

**The sizing is demand-side.** An earlier revision shipped a *fraction of the
parent* and the author's ruling retired it: *a fraction of hold is irrelevant.*
What decides the amount is `K` against the hold, the build-out the destination
will actually pay for, and the growth of the **combined** origin-plus-colony
system under a travel discount.

##### Settlers — priced in time

**Total growth is not what varies; timing is.** Both worlds reach their own
ceiling eventually whatever is shipped, so "the total growth of the combined
system" is a statement about *when* — which is exactly what the colony-years
guard measures. So the seed is priced in time:

```text
value(S) = T_c(S) = ln[ (S / (K_c − S)) · ((K_c − x_0) / x_0) ] / r
           the time the seed saves the destination, floor -> S

cost(S)  = T_p(S) = S / g(x_p − S, K_p)
           the time the origin needs to regrow what it gave away

choose S maximising   delta · T_c(S) − T_p(S)
```

**`r` cancels** — both terms carry `1/r` — so the split does not move when
`growth_rate` is retuned. That matters: `growth_rate` is separately ratified
(R-O84), and a policy coupled to it would tie together two things that were
measured apart.

**Two rival formulations were tried and rejected on measurement**, which is why
this one is written out rather than asserted:

| formulation | closed form? | why it was rejected |
|---|---|---|
| marginal next-cycle rate, `g(x_p − S) + delta·g(S)` | yes | ships **nothing** whenever the origin is below `K_p/2` and the destination is far — i.e. nothing in the early game, when colonising matters most |
| sum of fill times | no | strips a full origin to **99%** of itself at zero distance, *and* ships nothing from one at its ceiling when the destination is far — wrong in both directions at one operating point |

**The optimum is gridded, not solved.** Sampled over 4,000 random
configurations the objective has more than one turning point in **~4.6%** of
them, so it is not unimodal and a bisection or golden-section search would
return a local optimum in those — deterministically and invisibly, which is the
worst kind of wrong. `ENDOWMENT_GRID = 32` evaluations per launch is a few
thousand per run against a growth step that runs per planet per cycle.

**A destination-limited seed fills the world**, taken as an explicit case rather
than left to the grid: `T_c` diverges as `S → K_c`, because a colony landed at
its own ceiling has no growth left to wait through.

**The discount is `delta = 1 / (1 + tau / cycle_years)`** — a gain landing `n`
production cycles late is worth `1/(1+n)` of one landing now. Hyperbolic rather
than exponential, and chosen because it needs **no new constant**: `cycle_years`
already exists and is the natural clock for a per-cycle quantity. Whether the
form should be exponential is **R-IND14**, open.

##### Minerals — sized by the build-out, and this is where works value enters

```text
E = min( H − S , C(I_0 → I*) , bank(origin) )
```

where `C(I_0 → I*)` is the cumulative infrastructure rung price over that range.
Sending more than the destination will spend is freight for ore that then sits
in a stockpile; sending less means it waits on a freighter for something the
coloniser had room for.

**`I*` is where works value enters, and works are superadditive in `K` and `D`**
— the author's ruling: *works have higher value on a high-`K` world and on a
high mineral-density world, and the highest on the combination.* "Highest on the
combination" is a positive cross partial, and the simplest function with one is
a **product**. A product of masses is a **sum on the Band ladder**, so the
placeholder is the midpoint

```text
I* = (K_c + D) / 2          (Bands — i.e. the geometric mean of the two masses)
```

**Placeholder, flagged: R-IND13.** The real works-value function is §5's and
needs T-73/T-74. This is the cheapest form with the right cross partial, not a
claim about magnitudes.

**The mix is not a second decision.** The settlers are capped by `K_c`, so a
low-ceiling world takes few people and therefore leaves with a mineral-heavy
endowment — founded to be worked rather than to be lived on. Both halves mass
the same, so the split is invisible from outside, which is the point:
acceleration must not read out cargo *type* (design law #10).

#### Measured — and the model is inert at the shipped coloniser policy

**Colony-years is bit-identical to the supply-side model it replaces**, on both
seeds where a like-for-like baseline exists: seed 1 **10,558,680** and seed 7
**10,474,865**, against 10,558,680 and 10,474,864.5 before. Seeds 42 and 31337
have no pre-R-IND12 baseline at that operating point; their absolutes are
10,555,813 and 11,113,014, mean 10,675,593 over the four.

**That is a symptom, and the mechanism is measured rather than inferred**
(`examples/endowment --census`, all four seeds, 2,000 yr, **20,622 coloniser
launches**):

| seed | launches | hold filled to capacity | settlers only, no ore | carried minerals |
|---|---|---|---|---|
| 1 | 5,187 | **100.0%** | 99.7% | 0.3% |
| 7 | 5,000 | **100.0%** | 99.9% | 0.1% |
| 42 | 5,124 | **100.0%** | 99.8% | 0.2% |
| 31337 | 5,311 | **100.0%** | 99.9% | 0.1% |

Mean settlers **0.998–0.999 kt against a Medium hold of exactly 1.0000 kt**, and
**not one General hull flies.** So of the three caps, the **hold** binds on
essentially every launch: `hi = min(H, K_c, spare)` is `H`, the objective is
still climbing at the top of the feasible range, and the model returns *fill the
hold*. That is the same answer the retired fraction gave, so the identity is
arithmetic rather than luck.

Three things follow, and the third is the one that matters:

- **The mixed hold is implemented and exercised, just rare** — 0.1–0.3% of
  launches carry minerals, which are exactly the destinations whose `K_c` sits
  below a Medium hull's hold. The mechanism is live, not dormant code.
- **Conservation is still free.** R-O74 cost nothing at the shipped policy for
  the same reason (§1.6): a 1.0 kt hold against a centre that needs ≈14.2 kt of
  people before it may build one.
- **R-IND12 and R-IND11 are one question from two sides, and neither is
  ratifiable alone.** The endowment model only has a gradient where the hold is
  large enough for `K_c` or the origin to bind first, and that is the General
  hull — which is precisely what R-IND11 is about. A sweep of either while the
  other sits at its default is measuring a plateau.

**What this invalidates on purpose.** Every colony-years figure taken before it
— including §1.6's own +13.97% and §1.4's +4.4%/+4.7% for T-67 — was measured
on an economy that created population. They are not wrong as records of what
that engine did; they are not comparable to anything measured after. §1.6's
open question about how much of T-67's gain ran through the conjured-mass
channel is answered by re-measuring rather than by ablation now, since the
channel is closed.

---

---

## 2. The three employments, and where the allocation lives

| Employment | Produces |
|---|---|
| **Extraction** | ore out of the ground, kt/yr (§4) |
| **Fabrication** | hull mass per year, and slips (§3) |
| **Warding** | resistance to having Infrastructure razed — **not specified here** |

**The allocation is Doctrine**: a share vector `(w_ext, w_fab, w_ward)` on the
simplex, per empire, written only by tree cards
(`Hyades_standing_layer_and_observation.md` §5). It is **revisable** and costs no
minerals to change, which is the point of fusing the stock: you choose what your
industry is *for* continuously, and pay once for how much of it there is.

Three properties fall straight out of the standing-layer model:

- **It leaks through output, and the leak dies on retasking** (§5). A world
  out-producing its ore is fabricating; one hauling more than it builds is
  extracting. The allocation is inferable from behaviour at range — this layer's
  contribution to the yomi channel.
- **It is revisable, so committing it early is *weak* and committing it late is
  strong** — the opposite timing profile to a Design write (R-O37).
- **It cannot run away.** Shares renormalise, so no combination of Doctrine cards
  exceeds the stock (§6).

**R-IND2 (open):** whether `w_ward` is a third share at all, or whether warding
is a *Design* hardness coefficient on the stock. The second is cheaper to reason
about and does not force every empire to hold defensive capacity it is not using.

---

## 3. The production ramp

### 3.1 What is wrong today, measured

`SimConfig::build_years = 10.0`, flat, and a centre holds **one** build at a time
(R-O69's yard occupancy). So:

- **A Limited hull and a General hull take the same ten years**, though the
  General costs and masses **50×** as much (`limited_fleet_size = 50`). Hull size
  is free in time, which leaves the cost ladder as the only thing discouraging
  large hulls and quietly undercuts design law #3 — geometry is supposed to make
  consolidation *efficient*, not *instantaneous*.
- **An empire's industrial output is independent of its industry.** A homeworld
  at `Band II` and a fresh colony build at exactly the same rate. Nothing a
  player does moves it.

### 3.2 The model — linear slips, and a soft floor that emerges

Let `F` be a centre's **fabrication throughput** in kt/yr (§5 says how it is
bought).

**Concurrency is linear.**

```
slips(F) = 1 + floor(F / F_slip)
```

Every `F_slip` of throughput buys another berth; a centre always has at least
one. This is the "build wide" axis and it scales without limit, as asked.

**Turnaround has a soft floor, and it is not a second tuned curve — it falls out
of the first.** Throughput divides among the active slips, so a hull of dry mass
`m` occupies its berth for

```
t_build(m, F) = t_lead + m / (F / slips(F))
```

As `F` grows, `slips` grows with it, `F / slips → F_slip` from above, and

```
t_build → t_lead + m / F_slip        (approached, never reached)
```

**The soft limit is emergent, not imposed.** No amount of industry rushes one
hull below `t_lead + m/F_slip`; industry buys *more ships at once*, never
*faster ships*. Two consequences worth having on purpose:

- **Big hulls are slow for everybody.** A General hull is a long, visible
  commitment — a yard tied up for years — which is exactly the signal the
  observation model trades in. A rich empire cannot buy its way out of
  telegraphing it.
- **`m` is dry mass, which under R-O57 *is* mineral cost.** The time ladder and
  the price ladder are one ladder, with no second constant to tune and no way for
  them to drift apart.

### 3.3 The starting schedule — approved

**Approved as the schedule to start from.** Not Monte-Carlo ratified — these are
the values the first engine change ships with and measures against, not values
any sweep has confirmed.

| Constant | Value | Meaning |
|---|---|---|
| `t_lead` | **2.0 yr** | irreducible per-hull lead time — tooling and crew, the part that does not scale |
| `F_slip` | **0.1 kt/yr** | one slip's throughput |

| Hull | dry mass | `t_build` at one slip | today |
|---|---|---|---|
| Limited Systems | 0.02 kt | **2.2 yr** | 10 yr |
| Medium Systems | 0.10 kt | **3.0 yr** | 10 yr |
| General Systems | 1.00 kt | **12.0 yr** | 10 yr |

The schedule reads the way the design wants it to: a scout is a season's work, a
coloniser is quick enough to spam, and a General hull is a **twelve-year
commitment** that an opponent has time to notice and answer.

**This will move the bed.** Scouts and colonisers get dramatically cheaper in
time while General hulls get slightly dearer — a direct accelerant on the
expansion loop whose time constant T-51/R-O68 identified as the binding limiter.
Guard is `examples/colony_years`; the prediction is *up*, and if it is not, find
the mechanism before tuning the value (`CLAUDE.md` §2).

> **Landed (T-68, stage 2).** `SimConfig::build_years = 10.0` is gone, replaced
> by `build_lead_years` and `slip_throughput`; `Simulation::build_time(mass)` is
> the single expression, and it reproduces the table above exactly.
>
> **It takes the mass actually committed, not a hull table**, which is what lets
> one expression cover a hull, an infrastructure rung and a whole mining pair.
> Under R-O57 dry mass *is* mineral cost, so "what this build spent" and "how
> much stuff it is" are one number — and a mining pair drawn partly from Reserve
> is cheaper *and* quicker, because there is genuinely less to fabricate. To get
> that, `apply_build_with` now returns the committed `Price` rather than a
> `bool`: the real price is only known inside it, and re-deriving it outside
> would have been a second copy of the recycling rule.
>
> **The whole order launches as one unit.** The delay every hull in an order
> waits out is the order's own `t_build`, which is also exactly how long the yard
> is held — one number, computed once, rather than a per-ship time that could
> drift from the occupancy. Hulls taken from Reserve are already built and leave
> at once, and bootstrap survey craft are *seeded* rather than built, so they
> still launch instantly.
>
> **Measured on the guard, and the prediction held.** `examples/colony_years`,
> 3 seats, 4,000 yr: seed 1 **9,542,958 → 10,558,680 (+10.64%)**, seed 7
> **9,483,962 → 10,474,864 (+10.45%)**, with colony **count identical** on both
> (3,337 and 3,346). The same worlds, taken sooner — which is what a saturated
> bed has left to give (T-20).
>
> **And it cost an order of magnitude of throughput, which is the real result.**
> The same runs came in at **11 and 10 simulated-years per real second**, against
> the 183–217 yr/s the bed did after R-O70. T-24's floor is 2.5 yr/s, so the
> margin at 3 seats / 4 kyr is **~4x**, down from ~79x — and T-24 already records
> that degradation is *superlinear in duration*, so the 12-seat / 8-kyr corner is
> now under the floor rather than near it. The mechanism is not mysterious and
> should not be assumed either: a Medium hull went 10 yr → 3.0, so yards decide
> three to four times as often and entity count follows. **Measure the corner
> before optimising it** — that is T-66's first step and it now has a second
> reason to happen.
>
> **What it cost in test time, and why that is a finding rather than a chore.**
> The unit target went **87 s → 507 s**, and `CLAUDE.md` §2's rule applied
> exactly as written: *when a test target moves, look at what the simulation
> started doing, not at what the tests are asking.* A Medium hull dropped from 10
> yr to 3.0, so centres decide three times as often and the entity count follows.
> Four tests were paying for that in horizon they did not need — one cadence test
> was **437 s of the 507 on its own**, buying extra round barriers with a 1,400
> yr run. Shortening the *cadence* instead of the horizon gives it ten barriers
> where it had four, and the target is back to **≈55 s**. Nothing that was proven
> stopped being proven.

---

## 4. The mining ramp — crowding, scaled to the deposit

### 4.1 Mining is sublinear in crew, and the reason is physical

**Two miners cannot work the same vein.** Capacity beyond the first must open
another one, and the best veins are opened first — so the marginal miner is
always working ore poorer than the last. Aggregate output is **not** the sum of
individual outputs.

This is Lanchester's intuition with the sign reversed. Lanchester's square law
makes concentrated force *superlinear* (`N²`) because fire concentrates;
extraction is *sublinear* because sites are **exclusive**. Same lesson — an
aggregate is not `n ×` an individual — opposite direction, different cause.

### 4.2 The correction that matters: the vein count scales with the deposit

**A crowding exponent alone is wrong, and wrong in a way that would have killed
the thing this ramp is for.** Applying a bare `n^β` makes crowding identical on
every rock: with `β = 1/2` the third miner is worth `√3 − √2 = 0.32` of the
first whether the body holds one kilotonne or seven hundred thousand. Nobody
would ever put a large crew anywhere, and **the design wants hundreds or
thousands of miners working a high-value outpost.**

The missing term is that **a bigger deposit has more veins.** Lasky's law is a
statement about grade *and tonnage* together — cumulative tonnage rises roughly
exponentially as average grade falls arithmetically (Lasky 1950) — and the
relation is scale-invariant, so a large body and a small one have the same grade
*profile* over a different number of workable sites. Crowding is therefore
relative: `n` miners crowd a small rock and rattle around a large one.

**Vein count is a decade per Band**, which is the design's own language for
"an order of magnitude more of something":

```
N(S) = VEINS_PER_BAND ^ (band(S) − 1)          VEINS_PER_BAND = 10 (placeholder)
```

| deposit | mass | workable veins `N` |
|---|---|---|
| `Band I` | 1 kt | **1** |
| `Band II` | 31.6 kt | **10** |
| `Band III` | 2,828 kt | **100** |
| `Band IV` | 715,500 kt | **1,000** |

### 4.3 The law

```
extraction(kt/yr) = ε · S · W(n, S)

W(n, S) = N(S)^(1−β) · n^β          for n ≤ N(S)
```

- `S` — remaining ore, kt. Output tracks the stock, so a body depletes
  asymptotically rather than cliff-edging.
- `n` — extraction capacity at the site: **miner hulls on station plus units of
  Infrastructure allocated to extraction**, one law for both.
- `N(S)` — workable veins, §4.2.
- `β` — the crowding exponent. **Placeholder `1/2`.**
- `ε` — the per-unit rate constant.

Three properties, and they are the reason this shape was chosen:

- **A full crew pays linearly in the deposit's richness.** `W(N, S) = N`, so
  working a `Band IV` body with its full thousand miners yields a thousand
  units of work against a `Band I` body's one. Richness is worth going to, at
  scale, which is what §4.2 says was missing.
- **A large crew on a rich rock is rational.** On `Band IV`, one miner does
  `W = 31.6` and a thousand do `W = 1000` — 31.6× the ore for 1000× the hulls,
  against a body worth 715,500 kt and hulls costing 0.02 kt each. Twenty
  kilotonnes of miners to work seven hundred thousand of ore is an easy trade,
  and it stays easy for hundreds of hulls.
- **A poor rock saturates immediately.** On `Band I`, `N = 1`: the second miner
  adds nothing. Sending three hulls to a marginal rock is simply waste, and the
  autopilot should be able to see that.

**Beyond `N` the marginal miner gets the floor grade, not zero.** There is always
more poor ore (Lasky again), so the cap is a knee rather than a wall. **R-IND9:**
the exact tail past `N` — flat, or a shallow linear seam at floor grade. It
matters only for absurd crews and should be settled by what reads better in a
log, not by a sweep.

### 4.4 One law, two ways to buy capacity

Outposts and colonies mine by the same physics and differ only in how they buy
`n`. Keeping one law for both is deliberate: an asymmetry — outposts flat,
colonies improvable — would make "outpost or colony?" a question about *rate
shape*, when it should be a question about **commitment**.

| | Outpost | Colony |
|---|---|---|
| Buys capacity with | miner hulls | Infrastructure |
| Cost to establish | a Limited hull and a crew | a coloniser, and a habitable world |
| Improvable? | yes — add hulls | yes — add works |
| Vulnerable to | losing hulls | losing Infrastructure |
| Good for | a rich rock you do not intend to hold | a world you are building on |

### 4.5 What this changes that is already ratified

**T-57 ratified `miners_per_outpost = 3` under a law with no deposit term at
all** — extraction was `crew × outpost_mining_fraction`, linear in crew and
identical on every rock, measured at **+2.74% colony-years** with every seed
positive. Under §4.3 the right crew is **a property of the rock, not a constant**:
one miner on a `Band I` body, a thousand on a `Band IV`. So `miners_per_outpost`
does not survive as a scalar — it becomes a *policy* over `N(S)`, and the natural
Doctrine knob is a **target fraction of the deposit's veins** rather than a hull
count.

**It also gives T-66 its lever, in the opposite direction to the one I first
expected.** T-66 records hauling as the engine's largest single cost. Sublinear
crowding does not spread the fleet thin — it **concentrates** it, because a rich
body now genuinely rewards a thousand hulls. What that buys is fewer *sites* for
the same ore, and freight cost scales with sites and distance, not with crew. The
prediction is fewer, larger, closer outposts; it is a prediction and it must be
measured, not assumed.

## 5. Works — buying a rate with minerals, and where the colours bite

A **work** is a purchase that raises a planet's Infrastructure. It is the only
way `F` and `M` go up.

### 5.1 The works ratio — the mineral differentiation

Card costs split across a tree's colour ranking in a ratio
(`Hyades_mineral_cost_curve.md` §1.1: peak `4:2:1`, default `3:2:1`, floor
`5:4:3`, "never a true 1:1:1"). **Works get a parallel ratio system that is
allowed to go where card costs may not: all the way to a single colour.**

| Point | Ratio | Reads as |
|---|---|---|
| **Sole** | **1:0:0** | one colour only — unavailable to card costs; the Production signature |
| **Peak** | 4:2:1 | leans hard into one colour |
| **Default** | 3:2:1 | the typical case |
| **Floor** | 5:4:3 | flattest — payable by almost any homeworld |

**Why works may reach `1:0:0` and cards may not.** A card is a choice made once,
so a colour-locked card is simply *unavailable* to an archetype poor in that
colour — which design law #13 forbids. A work is a repeated purchase **with
alternatives**, so a sole-colour route is a *specialisation*: the Yellow-poor
empire cannot take the Production route efficiently, but §5.3 guarantees it has
another. The law is satisfied by the existence of the alternative, not by
flattening every price.

**This is the mechanism that makes the galaxy's mineral distribution bite on
development.** Today it bites only on card costs — and T-62 made the field
log-normal, so which colours a homeworld sits near is an enormous, almost
unexpressed fact about a game. Works are where it gets expressed.

### 5.2 Production's signature — the highest peak per planet, not the best efficiency

`Hyades_galaxy_and_autopilot.md` §4.8 puts **Production ↔ Yellow**.

**The Production tree's fabrication works are `1:0:0` in Yellow and reach the
highest `F` any single planet can attain. They are *not* the most
mineral-efficient route — they are the least.** The tree's identity is a
**ceiling**, bought at a premium and payable in one colour.

This is the tall/wide axis, and putting Production on *tall* is what makes it a
strategy rather than a discount:

- **Production is tall.** A few monstrous forges, each far past what any other
  route can reach, each expensive per kilotonne and payable only in Yellow. A
  General hull every few years from *one* world.
- **Everything else is wide.** Better rate per kilotonne, lower ceiling — so the
  same total industry costs less but needs more planets to hold it, and each one
  is a place that must be defended, supplied and held.

The premium is exactly the pressure the economic engine wants (§8): an empire
that insists on the highest ceiling is buying Yellow from someone, or taking it.

### 5.3 Growth and Expansion — efficient, capped, and payable anywhere

Both are **Cyan** (§4.8). They offer the genuine alternative design law #13
requires, along the axes named: efficiency and mix.

| Route | Mix | Rate per kt | Peak per planet |
|---|---|---|---|
| **Production** | `1:0:0` Yellow | **worst** | **highest** |
| **Growth** | `4:2:1` Cyan-primary | better | middling |
| **Expansion** | `5:4:3`, flattest | **best** | **lowest** |

Read as archetypes: **Production** builds few, enormous industrial worlds and
must source Yellow. **Growth** builds solid industry cheaply and hits a ceiling
that only cross-tree play lifts. **Expansion** gets the most industry per
kilotonne of anyone and can pay for it from any ground at all — but must hold
many worlds to use it, which is what Expansion is already for.

**R-IND3 (open):** the coefficients — how much better Expansion's per-kt is, how
much lower its ceiling, and where the crossovers sit. A Monte-Carlo question with
`colony_years` as the objective on the standard bed; not to be guessed here.

### 5.4 The rule that keeps costs honest

**A cost-modifying card moves the colour mix. It never lowers the total.**

Efficiency — more Infrastructure per kilotonne — is a *Design* write on a named
coefficient (§6.2). The mix is a **share vector**: a card adds weight to a
colour, and the split is the normalised weights, so it cannot change what the
bill sums to. Keeping the two orthogonal is what stops the card list becoming a
discount race, and it is what makes "shift the mix away from pure Yellow" a real
strategic move rather than a worse version of "make it cheaper".

---

## 6. The layering algebra — how work improvements compose

Cards accumulate all game. This section says exactly what a planet reads when an
empire has played a dozen of them, and it is written to be implementable rather
than evocative.

### 6.1 The correction: there are two algebras, not three, and "rotation" was wrong

An earlier draft of this section listed three: products for Design, a simplex for
Doctrine, and **rotation** for the colour mix. **The third is a defect, and it
falsifies the very property the section claims.** Rotations in three dimensions
do not commute — `R_x R_y ≠ R_y R_x` — so two mix cards played in opposite orders
would have produced different prices, in a document whose acceptance test is that
order does not matter.

The fix is that a colour mix is not an orientation, it is a **share vector**, and
it composes exactly the way the allocation does:

> **Both simplex-valued quantities — the employment allocation and the colour
> mix — are stored as non-negative *weights* and read as normalised shares.**
> Cards add weight. Addition commutes; normalisation bounds. Magnitude is
> preserved because a share is a share of a total set elsewhere.

So the whole layer is **two** algebras:

| Algebra | Applies to | Composition | Bounded by |
|---|---|---|---|
| **Multiplicative** | scalar coefficients — `η_works`, `F_cap`, `M_cap`, `u_half`, `ε` | `x = x_base · Π f_i` | nothing intrinsic; priced per tier |
| **Additive-weight** | simplex quantities — employment allocation, colour mix | `share_j = w_j / Σ w` | normalisation, with no clamp |

Both commute, which is the point. And the second has a property worth naming:
**there is no clamp anywhere in it.** Normalisation *is* the bound. That matters
because a clamp is exactly what let T-64's broken population logistic keep
scoring well — a bound that hides a fault rather than preventing one.

### 6.2 The state, and where it lives

Per **empire**, written only by tree cards (`Hyades_standing_layer_and_observation.md` §5):

| Field | Kind | Written by | Meaning |
|---|---|---|---|
| `eta_works` | multiplicative | Design | Infrastructure gained per kilotonne spent — *acquisition* efficiency |
| `cap_fab`, `cap_ext` | multiplicative | Design | per-planet **rate ceilings** — the asymptote |
| `half_fab`, `half_ext` | multiplicative | Design | Infrastructure at half the ceiling — the *knee* |
| `alloc_w[3]` | additive-weight | Doctrine | employment weights → `(w_ext, w_fab, w_ward)` |
| `mix_w[3]` | additive-weight | either | colour weights → the works price split |

Per **planet**: one number, `infra` in kilotonnes (§1.3). Everything else is
derived. That is deliberate — per-planet standing state is per-planet
serialisation, per-planet desync surface and per-planet card scope, and none of
those is wanted yet.

### 6.3 The pipeline — base to price, base to rate

**Buying a work:**

```
price_kt   = infra_rung_step(current_rung)      // cost ladder, T-61
           / eta_works                          // Design, multiplicative
price[c]   = price_kt · mix_w[c] / Σ mix_w      // colour split, additive-weight
```

**Running the stock:**

```
u_e        = infra · alloc_w[e] / Σ alloc_w     // employment share of the stock
rate_e     = cap_e · u_e / (u_e + half_e)       // saturating, per planet
```

The rate curve is a Michaelis–Menten hyperbola, chosen because **its two
parameters are exactly the two axes the trees are meant to differ on**:

- `cap_e` is the **asymptote** — the highest rate this planet can ever reach.
  **Production raises it.** That is "the highest peak per planet".
- `half_e` is the **knee** — the investment at which you are halfway there, so
  the initial slope is `cap_e / half_e`. **Growth and Expansion lower it.** That
  is "better efficiency per kilotonne invested".

One curve, two knobs, and the tall/wide axis falls out rather than being imposed.
A Production world climbs slowly toward a distant ceiling; an Expansion world
reaches most of a nearer ceiling almost at once.

**The ceiling is per *planet*, and the empire total is not capped.** An empire
scales by holding more worlds, which is why Expansion's cheap, low-ceiling works
are a strategy and not a handicap — and why `slips` (§3.2) growing linearly in
`F` does not contradict a bounded `F`: the bound is per yard, and an empire has
many yards.

### 6.4 Why a card cannot be a discount

`eta_works` is multiplicative on the **total**; `mix_w` is a **share** of that
total. So a mix card moves which colours the bill lands in and cannot change what
it sums to, and an efficiency card changes the bill without touching its
composition. **The two are orthogonal by construction rather than by discipline**,
which is what makes §5.4's rule ("move the mix, never lower the total")
enforceable in a type rather than in review.

### 6.5 The acceptance test

**Commutativity, as a property test in the engine, not a claim in this document.**
For any set of cards and any permutation of it, the resulting
`(eta_works, cap_*, half_*, alloc_w, mix_w)` must be **bit-identical** — not
approximately equal. Floating-point multiplication and addition are *not*
associative, so this is a real constraint on implementation and not a formality:

- **accumulate weights and factors in a canonical order** — by `CardId`, not by
  play order — so the arithmetic itself is order-independent, not merely the
  algebra;
- which means the state is a **multiset of played cards folded in card order**,
  and never a running product mutated at play time.

That is the same lesson as `holdings_centroid` (`CLAUDE.md` §4): a running total
accumulates in event order, a recomputed fold accumulates in a canonical order,
and only the second is reproducible. **R-IND4** is this test; it is cheap, and it
must exist before the second industrial card does.

### 6.6 Worked example — three cards, six orders, one answer

An empire plays a Production Design card (`cap_fab ×2`, `mix_w` +3 Yellow), a
Growth Design card (`half_fab ×0.5`, `mix_w` +2 Cyan), and a Doctrine card
(`alloc_w` +4 fabrication). Base `cap_fab = 1.0 kt/yr`, `half_fab = 2.0 kt`,
`mix_w = (1,1,1)`, `alloc_w = (1,1,1)`.

Folded in `CardId` order, whatever order they were *played*:

```
cap_fab   = 1.0 × 2      = 2.0 kt/yr
half_fab  = 2.0 × 0.5    = 1.0 kt
mix_w     = (1+2, 1, 1+3) = (3, 1, 4)  → C 37.5%, M 12.5%, Y 50%
alloc_w   = (1, 1+4, 1)   = (1, 5, 1)  → ext 14.3%, fab 71.4%, ward 14.3%
```

A planet holding `infra = 3.0 kt` fabricates at
`u_fab = 3.0 × 0.714 = 2.14 kt`, so `rate = 2.0 × 2.14/(2.14+1.0) = 1.36 kt/yr` —
enough for **14 slips** at `F_slip = 0.1`, and a General hull in
`2.0 + 1.0/(1.36/14) = 12.3 yr`. Note the last number: fourteen berths and the
big hull is *still* a twelve-year commitment (§3.2). Industry bought width, not
speed, exactly as intended.

### 6.7 The build order — a plan for landing the layering

Seven stages. **Each one is measurable on its own**, and the order is chosen so
that every stage before the last is either behaviour-neutral or has a predicted
sign with a known guard. The guard throughout is `examples/colony_years`, held to
the decimal where a stage claims to be neutral.

| # | Stage | Behaviour | Guard | T-code |
|---|---|---|---|---|
| 1 | **`K` loses its infra term** | changes; hulls seed to the world's ceiling | measure, do not predict | T-67 |
| 2 | **`t_build` tracks hull mass** | changes; predicted **up** | colony-years | T-68 |
| 3 | **Infrastructure stored as kilotonnes** | **neutral — measured, bit-identical** (§6.8) | colony-years | T-70 |
| 4 | **`Works` struct + the fold** | **neutral** — nothing reads it yet | bit-identical | T-75a |
| 5 | **Price reads `eta_works` and `mix_w`** | **changes — deepening −94.5%** (§6.9) | colony-years | T-73 |
| 6 | **Rates read the allocation and the curve** | changes; the ramp switches on | colony-years | T-71, T-74 |
| 7 | **Slips** | changes; predicted **up** | colony-years | T-69 |

Stages 3–5 are the layering proper, and they are deliberately **inert**: the
`Works` state exists, the fold runs, the pipeline reads it — and with no cards
played the coefficients are `1.0` and the weights are `(1,1,1)`, so every number
comes out where it was. **That is the whole trick.** A layering system that lands
neutral can be verified against a bit-identical bed, and only then switched on.

**Stage 4 in detail, because it is the one with a trap.**

```rust
/// Folded from the played-card multiset in *CardId* order — never mutated at
/// play time. Float multiplication is not associative, so a running product
/// accumulated in play order is a desync waiting for two clients to differ on
/// the order two simultaneous cards resolved in.
struct Works {
    eta_works: f64,          // multiplicative, base 1.0
    cap: [f64; 3],           // per employment, multiplicative, base 1.0
    half: [f64; 3],          // per employment, multiplicative, base 1.0
    alloc_w: [f64; 3],       // additive weights, base (1,1,1)
    mix_w: [f64; 3],         // additive weights, base (1,1,1)
}
```

The trap is the one `holdings_centroid` already taught (`CLAUDE.md` §4): **a
running total accumulates in event order and a recomputed fold accumulates in a
canonical order, and only the second is reproducible.** Playing cards A then B
must give bit-identical state to B then A, which for floats means the fold has to
*sort* — it cannot simply apply each card as it resolves. Recomputing on each
card play is `O(cards played)` on a list that reaches maybe a few dozen entries
per empire per game, and it runs on a round barrier rather than in an entity
loop, so the cost is irrelevant and the reproducibility is not.

**What to test, in the order the stages land:**

1. `works_fold_is_order_independent` — the R-IND4 property test. Every
   permutation of a card multiset gives **bit-identical** `Works`. This is the
   acceptance test for the whole section and it is written at stage 4, before any
   card exists that would make it fail.
2. `identity_works_reproduces_the_bed` — with no cards played, colony-years is
   unchanged to the decimal. Guards stages 3–5.
3. `a_mix_card_cannot_change_the_total` — for any `mix_w`, `Σ price[c]` equals
   `price_kt`. This is §6.4 as an assertion rather than a promise.
4. `the_rate_curve_saturates_and_is_monotone` — `rate(u)` rises with `u`, never
   exceeds `cap`, and reaches `cap/2` at `u = half`. Cheap, and it pins the two
   knobs to their stated meanings so a later retune cannot quietly swap them.
5. `an_allocation_cannot_exceed_the_stock` — `Σ u_e == infra` for every weight
   vector, including degenerate ones. Normalisation is the bound (§6.1); this is
   what proves no clamp crept in.

**Two things deliberately *not* in the plan.** No card content — the stages build
the surface cards will write to, and the first industrial card comes after the
property test, not before. And no coefficient tuning: **R-IND3's numbers are a
Monte-Carlo question and every value in §6.2 ships at identity**, so stage 6 is
measuring whether the *shape* behaves before anyone argues about magnitudes.

### 6.8 Two of the three "inert" stages are not inert, and the reason is in the code

§6.7 predicted stages 3 and 5 would be **bit-identical** against the shipped bed.
Reading the engine before building on that prediction says otherwise, in both
cases for a reason that is structural rather than a bug to be fixed.

**Stage 3 (T-70) was predicted non-neutral, and it came out bit-identical. The
prediction was wrong and the reason is worth more than the number.**

The arithmetic half of the argument was right: infrastructure was a `Band` and an
upgrade was `infra.up(1.0)`, exact addition in Band space; stored as the minerals
standing in it, an upgrade becomes `infra = infra_rung_price(n + 1)` and the rung
comes back through a `ln`. Those two really do disagree in the last bits.

The conclusion did not follow, because it assumed `BaselineAutopilot::rank` reads
infrastructure. **It does not** — `rank` scores `k_potential`, minerals and
position. Checking that took one grep and was skipped.

**Infrastructure reaches every live decision through an integer**, so a `1e-12`
difference is washed out before it can move anything:

- `infra_rung_of` **rounds**, and every pricing path is built on it —
  `infra_step_price`, `mineral_pressure_of`.
- The one continuous reader is `deepen_headroom = k_potential − infra`, and
  **R-O68 measured that branch dead at the shipped `reinvest_bias = 0.5`**: it
  cannot fire while any candidate exists.

Measured on the guard (`examples/colony_years`, 3 seats, 4,000 yr): seed 1
**10,558,680.0 → 10,558,680.0** and seed 7 **10,474,864.5 → 10,474,864.5**,
colonies and first-founding identical. Pinned by
`infrastructure_reaches_every_decision_through_an_integer`, which asserts the two
representations disagree in the Band and agree in the rung — so if a future
change makes a *continuous* reader of infrastructure live, which is exactly what
fixing R-O68 would do, the test fails and points at the reason.

**One real difference, found by that test rather than by reasoning.** Past the
top playable rung the old Band climbed without limit — `up(1.0)` on a position
has no ceiling — while the stock saturates, because the ladder does. It is
invisible in play because deepening is gated on `infra < k_potential` and
`k_potential = min(hab, bio_max)` cannot exceed the top rung. It is also the
better behaviour: an infrastructure Band above `Band IV` was a number with no
meaning.

### 6.9 T-73 measured — the constraint landed before its relief valve

**Colony-years went up** — seed 1 `10,558,680 → 10,606,309` (+0.451%), seed 7
`10,474,864.5 → 10,583,150.3` (+1.034%), colonies and first-founding identical.
A new *constraint* improving the bed is not a result to bank, so it was ablated.

**Keeping the per-colour gate and restoring proportional payment reproduces the
whole run bit-identically**, on both seeds. So the payment scheme contributes
nothing, and the entire effect is the **gate**. Instrumented (`examples/bank_mix`,
seed 1, 800 yr):

| | infrastructure builds | hull builds |
|---|---|---|
| pre-T-73 | 1,032 | 13,639 |
| T-73 | **57** | **21,098** |

**Deepening fell 94.5%.** The +1% is not colour billing paying off; it is a
deepen→expand reallocation on a bed where `k_high` binds the total, so minerals
denied to infrastructure buy hulls and worlds are taken earlier.

**The mechanism is the field, not the bill.** T-62 made the mineral field
log-normal **per colour**, so a centre's bank is dominated by one colour with
traces of the others — measured at t=800, **1,494 of 1,515 non-empty banks are
skewed**, one holding `C 477.2 / M 0.0095 / Y 1.04`. A bill that names all three
colours is unpayable there whatever the ratio: `1:1:1`, Floor `5:4:3` and Peak
`4:2:1` all demand every colour, and only a Sole `1:0:0` work matching the
dominant colour is affordable.

**So the constraint is right and it arrived before the thing that answers it.**
§5.3 promises every archetype an alternative route, and design law #13 requires
one — but *all four* ratio points in §5.1 need all three colours, so on this
field the promise is not kept by the ratios. What keeps it is **getting the
colours you lack**, and that is §8.1's whole subject: refined mass traverses real
space, on hulls. The engine has freight — `most_needed_center` routes ore by
mineral *pressure* — and **no colour term at all**.

That is the same shape as λ, the largest single ratification in this project's
history: freighter routing had no distance component, and adding the missing term
took coverage 14.4% → 38.3%. **Before tuning R-IND3's coefficients, add the
missing term** (**T-81**): route freight by what a centre's *bill* needs, not by
how broke it is overall.

**Until then T-73's identity mix is also wrong on its own terms.** `mix_w =
(1,1,1)` normalises to an even split, and §5.1's table has no `1:1:1` — it says a
works ratio is "never a true 1:1:1". The identity should be the Floor `5:4:3`,
which is the flattest *ratified* point. Left as `(1,1,1)` for now because
changing it moves the measurement, and the measurement above is the one that
matters (**R-IND15**).

---

**Stage 5 (T-73) changes a mechanic at identity, and that mechanic is the
point.** `Minerals::try_spend_total` debits an empire's bank **proportional to
holdings**. A works bill split by `mix_w` demands *specific colours*, and at
identity `(1,1,1)` that is equal thirds — which is not proportional-to-holdings
for any bank that is not already even. So there is no coefficient setting at
which the colour split reproduces the current behaviour, because the current
behaviour has no colour split at all.

That is not a defect in the plan; it is §5.1 arriving. *"This is the mechanism
that makes the galaxy's mineral distribution bite on development"* — the bite is
precisely the difference between "spend from the bank in proportion to what is
in it" and "pay this bill in these colours". A Yellow-poor empire noticing that
it cannot afford the Production route **is the feature**, and it cannot be
delivered by a change that leaves spending untouched.

**What to carry forward, and it is not about works.** A stage plan that labels
work "neutral" is making a claim about code, and the claim is checkable *before*
the work starts. Two of three were wrong here, and both were visible in ten
minutes of reading — one in an arithmetic identity, one in a single function.
**Check a neutrality claim against the code that would have to be neutral, not
against the description of the change.**

---

---

## 7. Trade, development freight, and what needs a pact

The third gap in §0, and the answer turns out to be the same mechanism as
inter-empire trade with a different owner at the far end.

### 7.1 The default transaction — balanced in value, no pact required

**An exchange between two empires whose trade balance is even *in value* requires
no pact and no card.** It is the default, always-available transaction, and it is
what makes the Exchange a market rather than a diplomacy minigame.

**Value is not kilotons.** They start equal — at first a kilotonne of Cyan is
worth a kilotonne of Yellow — and they diverge as the game develops, because
**value is set by demand, and demand is Doctrine.** An empire whose Doctrine is
pushing fabrication wants Yellow; one pushing extraction wants what its works
cost. The sole-Yellow Production route (§5.2) is therefore not just a price on a
tree — it is a **standing bid** that moves the market price of Yellow for
everybody.

That is the property worth protecting: the Exchange prices minerals off what
empires are actually *doing*, so §5's colour differentiation and
`Hyades_politics_trade_and_intelligence.md`'s market are the same system seen
from two ends.

### 7.2 What requires a card

| Transaction | Needs | Why |
|---|---|---|
| **Balanced exchange** (even value) | nothing | the default |
| **Deficit** — giving more value than you get | a card | a gift is a commitment, and commitments are what the standing layer prices |
| **Forging a pact** | a card | |
| **Breaking a pact** | a card | |
| **Smuggling** — trade that evades an embargo or a pact | a card | supported, deliberately |

The shape: **the market is free, and everything that bends it is a card.** A
player may always trade at par with anyone; running a deliberate deficit — the
classic entanglement move — is a play, not a transaction.

### 7.3 Development freight

Within an empire, the same machinery closes gap 3. Minerals are already hauled
between worlds by need (`most_needed_center`, design law #5). A **development
route** is that freight with its destination spending on *works* rather than
hulls:

- the **route** is a freighter leg, with the same distance discount and the same
  light-lag as any other;
- the **decision** is Doctrine — a `develop_bias` share of a centre's output
  aimed at raising other worlds rather than building at home;
- the **spend** happens at the destination, on its works ladder, in its colours.

Two things this gets right by construction: **it is neither instant nor free**,
so a distant colony is genuinely harder to develop than a near one — the geography
the engine is built on applies to development automatically; and **it is the same
mechanism as §7.1**, so `sim §2a`'s existing claim that "commerce can raise a
partner's infrastructure… and embargo a way to let it decay" is implemented
rather than merely asserted.

---

## 8. Why the engine needs all six minerals

The economic thesis: **the best war machine requires all three basics and all
three supers, and no single empire's territory supplies all six at scale.**
Trading partners, piracy, theft or conquest are the only ways to close the gap —
and that is the hard choice the game is about.

This document contributes three pressures, and **none of them is a rule
forbidding self-sufficiency** — each is a gradient that makes it expensive:

- **Works are colour-differentiated (§5).** Your development ramp is priced in
  colours your ground may not hold, so *growth itself* is a reason to trade, not
  only armament.
- **The highest ceiling is sole-coloured (§5.2).** The tallest industrial route
  is payable in one colour, so the empire that wants it and lacks the colour has
  a standing demand — the thing an Exchange needs to price.
- **Development is transferable (§7.3).** Because industry can be shipped, that
  demand has a supply. Without §7 the colour pressure would be a tax; with it, it
  is a market.

### 8.1 Refined mass traverses real space (R-IND7, resolved)

**Minerals, supers and apex must physically cross the theater, so they can be
attacked, diverted, stolen and blockaded.** Nothing teleports. This is the
addition that turns §8 from a set of price gradients into a *military* problem,
and it is the load-bearing half of the economic thesis.

Everything follows from taking it literally:

- **A trade is a voyage.** §7.1's balanced exchange is not a ledger entry; it is
  freight, on a leg, under light-lag, with a hull that can be intercepted. The
  Exchange prices the *contract*; the freighter carries the *goods*, and the gap
  between the two is where piracy lives.
- **A supply line is a target, and it is the softest one an empire has.**
  `Hyades_politics_trade_and_intelligence.md` §3.3 already needs escrow,
  settlement, default and theft; this is why. Blockade is not a special rule — it
  is the ordinary consequence of a fleet sitting on a route that must be flown.
- **Refining has a location.** The `3 basics → 2 supers → 1 apex` ladder
  (`Hyades_galaxy_and_autopilot.md` §4.2, cost curve §5.0) consumes precursors
  *somewhere*, so the precursors must be hauled to that somewhere. Concentrating
  synthesis is efficient and makes one place worth attacking; dispersing it is
  safe and slow. **That is a real strategic choice and it exists only because
  mass moves.**
- **The scarcity becomes positional, not just geological.** An empire can sit on
  all six colours and still be unable to *use* them, because the routes between
  the deposits and the forge run through somebody else's reach. Conversely a poor
  empire astride a corridor has something to sell that is not ore.

**Why this is what makes the thesis work.** §8's three pressures — colour-priced
works, a sole-coloured tall route, transferable development — establish that
empires *need* to trade. Traversal is what makes trading *risky*, and therefore
what makes the alternatives real: piracy, theft, blockade and conquest are all
just interventions on a route, and they are available precisely because the goods
are on a ship rather than in a spreadsheet. **Take traversal away and the
alternatives to trade collapse into flavour text**, because there would be
nothing physical to interdict.

It also disciplines this document's own proposals. Development freight (§7.3) is
attackable, so *developing a distant colony is a military exposure* and not only
a logistical cost. And T-66's hauling bill stops being pure overhead: freight is
the surface the whole war economy acts on, so the fix there must make hauling
*cheaper*, never *abstract*.

**Engine consequence:** the Exchange (T-01, `matching.rs`) must settle into a
**freight leg**, not a transfer. Matching decides *who trades what at what price*;
delivery is an ordinary voyage that can fail. **R-IND10:** what happens to a
matched contract whose carrier is destroyed — does the loss fall on buyer,
seller, or an escrow the politics spec already sketches?

### 8.2 What is deferred

Supers and apex *as a works input* — the mid- and late-game boosts that raise the
ceiling — are **R-IND6**, deliberately deferred: they should not be designed until
the basic ramp is measured, because their whole job is to bend a curve that does
not exist yet. §8.1 is not deferred with them; it is a property of how *all*
refined mass moves, and it applies from the first freighter.

---

## 9. Captured Infrastructure — R-IND8

**Open, and to be resolved in this spec rather than in code.** When an empire
takes a developed world, what does it inherit?

The two halves come apart, and the asymmetry is the interesting part:

- **Improvement costs shift trivially.** Future works on that world are priced in
  the *captor's* tree colours and Doctrine (§5), because the price is a property
  of who is building, not of what is built. This part is easy and almost
  certainly right.
- **Current production may not shift at all.** The standing stock was built by
  someone else, to someone else's Design, and the argument for it carrying on
  unchanged is that a fabricator is a fabricator. The argument against is that
  Design coefficients are the captor's, so an inherited work would silently
  perform at rates the captor never paid for — in either direction.

Three candidate resolutions, none chosen:

1. **Inherit the stock, apply the captor's coefficients.** Simplest; means
   capturing a Production world hands a Growth empire a forge it could not have
   built and cannot fully use.
2. **Inherit the stock at the *builder's* coefficients until re-worked.** Richest:
   captured industry is a distinct, decaying asset, and there is a real decision
   about whether to re-tool it.
3. **Inherit a fraction and re-tool from there.** The conquest analogue of R-O59
   slag.

The question interacts with R-IND2 (warding) and with whatever answers R-O47b's
no-retroactive-refits rule implies for installations rather than hulls — a
captured work is the same shape of problem as a Design write reaching a hull
already in the field.

---

## 10. Engine work items

Dependency order. Each is small; the order matters more than the size.

| # | Item | Blocked on | T-code |
|---|---|---|---|
| 1 | `K = min(hab, bio_max)` — drop infrastructure from the minimum (§1.1) | — | **T-67** |
| 2 | `t_build` from hull mass — replace flat `build_years` (§3.2) | — | **T-68** |
| 3 | Slips: concurrency linear in `F` (§3.2) | 2 | **T-69** |
| 4 | Infrastructure stored as kilotons; the Band is a reading (§1.3) | 1 | **T-70** |
| 5 | Extraction law: `N(S)` veins per deposit, `W = N^(1−β)·n^β`, one law for crews and works (§4.3) | — | **T-71** |
| 6 | `miners_per_outpost` becomes a target *fraction of `N(S)`*, not a hull count (§4.5) | 5 | **T-72** |
| 7 | Works: colour-differentiated infrastructure price (§5.1) | 4 | **T-73** |
| 8 | Extraction and fabrication rates from Infrastructure × allocation (§2) | 4 | **T-74** |
| 9a | `Works` struct + the CardId-ordered fold + the commutativity property test (§6.7) | 4 | **T-75a** |
| 9b | `Doctrine` allocation vector wired to the fold (§6.2) | 9a | **T-75b** |
| 10 | Development freight and the balanced-exchange default (§7) | 7 | **T-76** |
| 11 | Exchange settles into a **freight leg**, not a transfer — refined mass traverses real space (§8.1) | T-01 | **T-77** |

**Item 1 first, and alone.** It is one deleted `.min()`, it unblocks every card
that attacks Infrastructure, and it invalidates R-O76's measured result — so it
wants its own measurement rather than being folded into a larger change.

**§6.7 is the sequenced version of this table and takes precedence over it.** It
orders the same work so the layering lands **inert** — the state exists, the fold
runs, the pipeline reads it, and with no cards played every coefficient is `1.0`
and every weight vector is `(1,1,1)`, so the bed is bit-identical. Only then does
anything switch on.

**Item 2 is the other one worth doing early**, because it is a behaviour change
with a known guard and a predicted sign, and because leaving the flat-time
artifact in place contaminates every later measurement.

---

## 11. Open R-code register

| Code | Question | Where |
|---|---|---|
| ~~R-IND1~~ | ~~Population above `K` must decline rather than crash~~ — **resolved: infrastructure leaves `K` entirely**, so the crash cannot be triggered by an industrial strike. Habitability and biosphere strikes still crash population, intentionally. | §1.1 |
| **R-IND2** | Is warding a third allocation share, or a Design hardness coefficient? | §2 |
| **R-IND3** | Works coefficients — Expansion's per-kt advantage, Production's ceiling, the crossovers. MC question. | §5.3 |
| **R-IND4** | Commutativity as a property test over the card list, written before the second industrial card. | §6 |
| ~~R-IND5~~ | ~~May a development route target a rival's world?~~ — **resolved: balanced-value exchange needs no pact; deficits, pacts, pact-breaking and smuggling are cards.** | §7 |
| **R-IND6** | Supers and apex raising the works ceiling — deferred until the basic ramp is measured. | §8 |
| ~~R-IND7~~ | ~~The economic-thesis addition~~ — **resolved: minerals, supers and apex traverse real space**, so they can be attacked, diverted, stolen and blockaded. Nothing teleports; a trade is a voyage. | §8.1 |
| **R-IND8** | What an empire inherits when it captures developed Infrastructure. | §9 |
| **R-IND9** | The extraction tail past `N(S)` — flat, or a shallow seam at floor grade. | §4.3 |
| ~~**R-O74**~~ | ~~Founding settlers are conjured~~ — **resolved.** Settlers are debited from the founding centre's population and the rest of the hold is loaded from its bank; a contested coloniser unloads both halves back home. | §1.7 |
| **R-IND12** | How much a coloniser carries. **Model settled, magnitudes open.** Settlers are priced in time — what the seed saves the destination against what it costs the origin to regrow — discounted by transit; minerals are sized by the destination's intended build-out. The supply-side `endowment_fraction` is retired. | §1.7 |
| **R-IND13** | The works-value rung `I*`. Placeholder is the Band midpoint of capacity and abundance, i.e. the geometric mean of the two masses — the cheapest form with the required positive cross partial. The real function is §5's and needs T-73/T-74. | §1.7, §5 |
| **R-IND14** | Whether the travel discount should be hyperbolic (`1/(1+n)`, current, no new constant) or exponential (needs a time constant). | §1.7 |
| **R-IND10** | Who bears the loss when a matched contract's carrier is destroyed — buyer, seller, or escrow? | §8.1 |
| **R-IND11** | Is a General coloniser ever worth it, now that the hold is the only thing separating the hulls? **Measured and reframed — blocked on R-O74.** `SettlersPerMineral` scores +13.97% colony-years on identical colony counts, but two ablations put the whole effect on the *seed mass* rather than the hull, and founding settlers are conjured. Unanswerable until the seed is drawn from the origin. Default stays `CheapestViable`. | §1.6 |

---

## References

- Traulsen, A. & Nowak, M. A. (2006). Evolution of cooperation by multilevel
  selection. *PNAS* 103(29):10952–10955. — the narrative thesis §1.2 leans on.
- Lasky, S. G. (1950). How tonnage and grade relations help predict ore reserves.
  *Engineering and Mining Journal* 151(4):81–85. — the grade-tonnage relation
  §4.1 derives the crowding exponent from, and §4.2 the scale-invariance that
  makes the vein count track the deposit.
- Lanchester, F. W. (1916). *Aircraft in Warfare: The Dawn of the Fourth Arm.* —
  the square law §4.1 mirrors.
- May, R. M. (1976). Simple mathematical models with very complicated dynamics.
  *Nature* 261:459–467. — the discrete logistic above `K`, §1.1.
- *Stars!* (1995), Mare Crisium. — the three-installation model §1.3 departs
  from, and the `-f` archetype that is the reason why.
