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
  all. Deepening no longer raises a ceiling; it raises **rates**. *(Updated at
  §6.18: R-O68's units fault is closed — the trade is now a return per kilotonne
  on both sides — and the branch is still cold at the shipped `reinvest_bias`,
  now for the price reason R-O85 carries. `k_potential` remains what gates the
  staircase, so this bullet's point stands: the quantity gating deepening is a
  population ceiling, and what deepening buys is a rate.)*
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

> **It was false in the engine for two landings, and is true again (R-O88).**
> `slips` read the *rate* `F`, and T-74 made `F` a Michaelis–Menten hyperbola
> bounded by `fab_cap`, so the axis was `fab_cap / F_slip = 2` berths — closed,
> not short: a yard on 10¹² kt of infrastructure still had two, and homeworlds
> are *generated* at rung II, already past the only step it had. **`slips` now
> reads the fabrication share of the infrastructure *stock*** and is unbounded,
> so `F` in the formula above is a stock and `F_slip` is the stock one berth
> occupies. The per-berth *rate* is where `fab_cap` lives now — §6.19b.

**Turnaround has a soft floor, and it is not a second tuned curve — it falls out
of the first.** Throughput divides among the active slips, so a hull of dry mass
`m` occupies its berth for

```
t_build(m, F) = t_lead + m / (F / slips(F))
```

As `F` grows, `slips` grows with it, `F / slips → F_slip` **from below**, and so

```
t_build → t_lead + m / F_slip        (approached from above, never reached)
```

> **Corrected at T-69.** This paragraph previously read "`F / slips → F_slip`
> from above", which is the reciprocal of what the `1 +` in `slips` actually
> does and the opposite of the floor claim in the next paragraph. Per-berth
> throughput is `F / (1 + floor(F/F_slip))`, which is *strictly less* than
> `F_slip` for every finite `F`; time is its reciprocal, so `t_build` sits
> strictly **above** the floor and descends toward it. The `1 +` is therefore
> load-bearing, not a boundary convenience: **dropping it inverts the design
> property.** Measured while implementing T-69 — with `slips = max(1, floor(·))`
> a rung-II centre turns a Medium hull around in **2.55 yr against a 3.0 yr
> floor**, so a rich empire buys faster single hulls, which §3.2 exists to
> forbid. `industry_buys_concurrency_and_never_undercuts_the_turnaround_floor`
> pins the inequality rather than any number, because a number-by-number
> schedule test passes under both forms.

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

| Hull | dry mass | `t_build` floor = `t_lead + m/F_slip` | before T-68 |
|---|---|---|---|
| Limited Systems | 0.02 kt | **2.2 yr** | 10 yr |
| Medium Systems | 0.10 kt | **3.0 yr** | 10 yr |
| General Systems | 1.00 kt | **12.0 yr** | 10 yr |

> **What this column means changed at T-69.** T-68 shipped `t_build = t_lead +
> m/F` with no slips, and a rung-I centre hit these three numbers exactly. Once
> slips divide the throughput they become the **asymptote** — the limit an
> arbitrarily industrialised yard descends toward and never reaches (§3.2). At
> the T-74 anchor a rung-I centre sits at `F = F_slip` with two berths, so it
> turns a Limited hull around in 2.4 yr rather than 2.2, and buys a second
> concurrent build for the difference. The schedule is still the design target;
> it is now a floor rather than a reading.

The schedule reads the way the design wants it to: a scout is a season's work, a
coloniser is quick enough to spam, and a General hull is a **twelve-year
commitment** that an opponent has time to notice and answer.

**This will move the bed.** Scouts and colonisers get dramatically cheaper in
time while General hulls get slightly dearer — a direct accelerant on the
expansion loop whose time constant T-51/R-O68 identified as the binding limiter.
*(T-51 closed at §6.18; the limiter's mechanism resolved to the infra price
ladder rather than to the comparison — R-O85.)*
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

### 4.3a Extraction saturates once — here, not in §6.3 (R-IND18, decided)

**§4.3 and §6.3 both claim to set the extraction rate, and applying both would
be wrong.** §6.3's Michaelis–Menten curve `rate_e = cap_e · u_e/(u_e + half_e)`
saturates in capacity; §4.3's `W(n, S) = N(S)^(1−β) · n^β` also saturates in
capacity. Composing them saturates twice.

**Decision: extraction saturates through §4.3, and §6.3's curve governs
fabrication and warding.** Two reasons, and the first is the one that matters:

- **MM has no deposit term.** `half_ext` is a per-planet constant, so an MM
  extraction curve makes the crowding knee identical on a `Band I` rock and a
  `Band IV` seam — which is *precisely* the fault §4.2 exists to correct, and it
  would kill the "hundreds or thousands of miners at a high-value outpost" the
  ramp is for. `N(S)` is not an optional refinement of the shape; it is the
  shape.
- **Double saturation makes a Production card inert.** Under both curves,
  `cap_ext` is a ceiling above a function that already cannot reach it, so
  raising it moves nothing measurable — the same dead-branch shape R-O68 found
  in `reinvest_bias`, arrived at from a different direction. *(§6.18 measured
  the same saturation on the fabrication side: `fab_cap = 0.2` puts the whole
  infra ladder inside one knee, which is R-O85.)*

**Both knobs survive with their tree meanings intact**, mapped onto §4.3's two
parameters instead of MM's:

| §6.2 field | Where it lands in §4.3 | Tree meaning, unchanged |
|---|---|---|
| `cap[Extraction]` | scales `ε`, the per-unit rate constant | Production's **asymptote** — more ore per vein-year, the highest peak |
| `half[Extraction]` | divides `n`, the effective capacity on station | Growth/Expansion's **knee** — more work per kilotonne invested |

So `extraction = ε · cap_ext · S · W(n / half_ext, S)`, and the tall/wide axis
reads the same as it does for fabrication.

**§6.3's text is amended by this section**, and the amendment is narrow: `rate_e`
is the rate law for **fabrication and warding**; extraction's is here. The two
employments are not obliged to share a functional form — fabrication works a
stock the empire built and can always add to, extraction works a body that is
finite, heterogeneous and shared with rivals.

**R-IND18** is what remains open: `ε`, `β` and `VEINS_PER_BAND` are all
placeholders, and the composite above has never been measured. The guard is
Growth's work-years with colony-years alongside (§6.12), and the prediction is
**concentration** — fewer, larger, closer outposts (§4.5) — which is a claim about
site count and must be read from a census, not inferred from the objective.

### 4.3b The law is normalised by `N`, because §4.3 double-counts the deposit (R-IND19)

§4.3 writes `extraction = ε · S · W(n, S)`. **`W` rises with `N`, `N` rises with
`S`, and `S` is already there — so output goes as richness squared**, and the
section's own guarantee that "output tracks the stock, so a body depletes
asymptotically rather than cliff-edging" fails. It is not a marginal effect: at
`ε = 0.238` a full crew on a `Band III` body computes **307% of everything
present** in one tick, and only a clamp stops it.

**The engine multiplies the stock by `W(n,S) / N(S)`, which reduces to
`(n/N)^β`** — the share of the body's veins a crew effectively works.

**Every claim §4.3 makes survives**, because dividing by `N` is a change of
scale and `ε` absorbs it, while §4.3's assertions are all ratios:

| §4.3 says | under `ε·S·W` | under `ε·S·(n/N)^β` |
|---|---|---|
| 1000 miners on `Band IV` lift **31.6x** what one does | 1000 / 31.6 = 31.6x | 1 / 0.0316 = **31.6x** |
| a full crew pays **linearly** in richness | `ε·S·N` — quadratic | `ε·S` — **linear** |
| a poor rock **saturates immediately** | `N = 1`, yes | `N = 1`, **yes** |
| richness is worth going to | yes | **yes** — a lone miner works a smaller *share* of a rich body and lifts far more ore |

So the disagreement is confined to the **absolute scale**, which no measurement
has ever fixed either way (R-IND18), and the engine takes the reading that keeps
a body finite. `ε` is `outpost_mining_fraction = 0.238` unchanged, which anchors
the change: **a lone miner on a `Band I` body extracts exactly what it did
before T-71**, since `N = 1` there and the factor is 1.

`veins_are_a_decade_per_band_and_crowding_pays_at_scale` asserts the table and
the 31.6x through the normalised form, so the design's numbers are pinned to the
thing the engine multiplies by rather than to an intermediate.

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

> **Landed (T-71 + T-72, then superseded by T-87).** `miners_per_outpost` is
> **removed**, not retained at its ratified value; `Doctrine::miner_vein_fraction`
> replaced it with `max(1, round(f · N(S)))` and was itself removed at T-87, when
> crew stopped being a policy at all (§6.16). A knob nothing reads is worse than no knob — a
> sweep moves it, measures nothing and reports a flat gradient, which is exactly
> how `cargo_unit_size` came to look inert (design law #14). The ratified figures
> survive as the record in the retired field's doc comment, and as the reason
> `f`'s default has to be re-ratified rather than inherited.
>
> **Measuring T-71 without T-72 measures a half-built mechanism.** With crowding
> in and crews still a flat 3, every crew is under-sized against a vein count
> spanning 1 to 1,000 — three miners on a `Band III` body work 17% of a full
> crew's share where they used to work three times one miner's. The engine's
> response is to open *more sites*, which is the exact opposite of this
> section's prediction and is entirely an artifact of the missing half. The
> measured signature of that half-built state is in §6.14.
>
> **`f` is gone too (T-87).** It shipped at 0.07 and was replaced within the
> session by a model rather than a value: crew is derived from the founding
> centre's unmet mineral demand, §4.3's law inverted, with **no parameter at
> all**. The sign inverts with it — a richer rock wants a *smaller* crew,
> because it meets the same demand with fewer hands. §6.16 has the law and the
> measurement; it beats `f = 0.07` on both metrics on both seeds while carrying
> 34% fewer vehicles.

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

> **Amended by §4.3a: `rate_e` here is fabrication and warding.** Extraction
> saturates through §4.3's crowding law instead, because that law carries the
> deposit term `N(S)` and this one does not — an MM extraction curve would make
> crowding identical on every rock, which is the fault §4.2 exists to correct.
> `cap_ext` and `half_ext` keep their tree meanings; §4.3a says where they land.
> The engine ships the MM form for extraction today (T-74) and §4.3a is T-71.

**The ceiling is per *planet*, and the empire total is not capped.** An empire
scales by holding more worlds, which is why Expansion's cheap, low-ceiling works
are a strategy and not a handicap.

> ~~"and why `slips` (§3.2) growing linearly in `F` does not contradict a bounded
> `F`: the bound is per yard, and an empire has many yards."~~ **Withdrawn — it
> is a non-sequitur (R-O88).** §3.2's claim is about **one centre's berth
> count**; "an empire has many yards" is about the **empire total**. The two
> sentences are not about the same quantity, and the reconciliation they
> appeared to perform did not happen.
>
> **Resolved by denomination instead.** `cap_e` for fabrication is now the
> ceiling **one berth** can reach, not one planet: `berth_rate = cap · u/(u +
> half)`, and a planet's total is `slips × berth_rate`, unbounded in the stock.
> Every word of this section's reading of `cap` and `half` survives — Production
> raises the ceiling, Growth and Expansion lower the knee — it is simply a
> statement about a berth. §5.3's table then reads directly: **Production buys
> fast berths, Expansion buys many slow ones.** See §6.19b.

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
  that branch is **cold at the shipped `reinvest_bias = 0.5`** — R-O68 first
  measured it dead on a units fault, and closing that fault (§6.18) left it cold
  for a *price* reason instead (R-O85). The crossover sits at `b` between 0.96
  and 0.98, so the reader is live in principle and never consulted in practice.

Measured on the guard (`examples/colony_years`, 3 seats, 4,000 yr): seed 1
**10,558,680.0 → 10,558,680.0** and seed 7 **10,474,864.5 → 10,474,864.5**,
colonies and first-founding identical. Pinned by
`infrastructure_reaches_every_decision_through_an_integer`, which asserts the two
representations disagree in the Band and agree in the rung — so if a future
change makes a *continuous* reader of infrastructure live, the test fails and
points at the reason. It survived R-O68's own fix — `examples/deepen_census`
reports the whole run bit-identical below `b = 0.96` — and it is what will fail
if R-O85 ever prices the rungs low enough for that reader to start deciding.

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

### 6.10 The default mix is `3:2:1` Y:C:M, and Sole is an asymptote (R-IND15 resolved)

**Ratified by the author.** The works default is the §5.1 *Default* point
`3:2:1`, assigned **Yellow : Cyan : Magenta** — `WORKS_MIX_DEFAULT`, stored in
`Basic` order as `[2, 1, 3]`, normalising to `C 0.333 / M 0.167 / Y 0.500`.

Yellow-primary because **Production is Yellow** (`Hyades_galaxy_and_autopilot.md`
§4.8), so the ordinary cost of developing any world already leans toward the
colour one tree is about, and every empire feels the pull of a colour it may not
hold. The `(1,1,1)` the first implementation shipped was a placeholder and was
wrong on the spec's own terms: §5.1 says a works ratio is *"never a true
1:1:1"*.

**`1:0:0` is reachable only by deep Production cards, and that needs no
mechanism.** The author's ruling is that Sole sits several layers into the
Production tree — and it falls straight out of `mix_w` being an **additive
weight**. A card adding `k` to Yellow moves the share to `(3 + k) / (6 + k)`:

| added Yellow weight | Yellow share |
|---|---|
| 0 (default) | 0.500 |
| +3 | 0.667 |
| +9 | 0.800 |
| +24 | 0.900 |
| +54 | 0.950 |

Diminishing returns are steep and the limit is never attained. **So "Sole" is
the deep end of a ladder rather than a discrete state**, which is exactly the
behaviour the ruling asks for, with no gate, no special case and nothing to
enforce. How much weight one deep Production card adds — and therefore how many
layers "deep" means — is **R-IND16**, open.

### 6.11 T-81 — freight routes by colour

`most_needed_center` scored need on a **total**: how far a centre was from
affording its next rung, with no colour term at all. §6.9 measured what that
cost once bills became colour-payable. The routing score is now

```text
deficit[c] = max(0, works_bill[c] − bank[c])
relief     = Σ_c min(cargo[c], deficit[c]) / Σ_c cargo[c]
score      = relief · exp(−λ · t_transit)
```

so a hauler carrying Yellow goes where Yellow is what is missing.

**It replaces `mineral_pressure_of` in the score rather than multiplying it.**
The two ask the same question at different resolutions — *how far is this centre
from affording its next rung*, on a total versus per colour — and multiplying
would double-count. `mineral_pressure_of` survives for the deepen/expand
decision, where a total is what a centre weighs.

Both terms are dimensionless fractions in `[0, 1]`, deliberately: R-O68 is this
project's standing lesson on what a mixed-unit comparison does to a branch — a
Band difference against an unbounded score, a constant absorbing the mismatch,
and a path that could never fire. *(Closed at §6.18; the lesson stands, and its
sequel is that fixing the units revealed the branch was cold on its merits.)*

**Degenerate case, stated rather than discovered:** a centre that can already pay
every colour of its next bill scores `0`, and if every centre can, the choice
falls to the entity-id tie-break. That is what the pressure formula already did
when nothing was short, so it is not new behaviour.

#### T-81 measured — it does not work, and it is counterproductive

| | infrastructure builds | banks skewed |
|---|---|---|
| pre-T-73 | 1,032 | — |
| T-73 | 57 | 1,494 / 1,515 |
| **T-73 + T-81 + `3:2:1`** | **31** | **1,533 / 1,551** |

Bank composition did not move, and deepening fell further. **Two mechanisms,
both measured.**

**The supply is single-coloured.** `examples/bank_mix` measures every mineral
source: **6,725 sources, mean dominant-colour share 0.789**, with **38% of
sources ≥95% one colour** and 57% ≥80%.

| dominant share | sources |
|---|---|
| <40% | 427 |
| 40–60% | 1,160 |
| 60–80% | 1,315 |
| 80–95% | 1,267 |
| **≥95%** | **2,556** |

That is T-62 being taken seriously — each colour is an independent Gaussian over
Bands, so in kilotons one dominates a rock by orders of magnitude — and
`MineralField::extract` takes proportionally, so a **freighter's cargo inherits
the skew**. No routing rule over single-coloured cargoes can assemble a
three-coloured bank.

**And routing by colour-need anti-concentrates, which is why it made things
worse.** Paying a three-colour bill requires ore to **converge** on one centre.
`relief` sends each colour to wherever *that colour* is scarcest — by
construction a different centre for each colour. Before T-81 every hauler went to
the neediest centre by total, so ore accumulated somewhere it could eventually
hold all three. Scattering it by colour is the opposite of what the bill needs.

**The corrected formulation, and the denominator is the whole of it.** Score the
*completion of the bill*, not the relief of one colour:

```text
short_before = Σ_c max(0, bill[c] − bank[c])
short_after  = Σ_c max(0, short_before[c] − cargo[c])
completion   = (short_before − short_after) / short_before   · exp(−λ·t)
```

**Divide by what is left to find, not by the bill.** The first written form of
R-IND17 used `Σ bill` and it does not concentrate — worked on paper before
implementing, per `CLAUDE.md`'s rule about probing a scaling relationship first:

| destination, given a Yellow cargo | `÷ Σ bill` | `÷ short_before` |
|---|---|---|
| needs only Yellow | 0.500 | **1.000** |
| needs Yellow and Magenta | 0.500 | 0.750 |
| needs everything | 0.500 | 0.500 |
| needs only Magenta | 0.000 | 0.000 |

`÷ Σ bill` ties the nearly-payable centre with the empty one — both absorb the
same absolute shortfall — so it reproduces exactly the scattering it was written
to fix. `÷ short_before` puts the centre that this cargo *finishes* at `1.0`, so
ore concentrates where it can actually be spent. **R-IND17** (landed;
`a_hauler_routes_to_the_colour_that_is_missing` asserts the concentration
property directly).

> **A note on the guard, which is about method rather than industry.**
> Colony-years tracks deepening **inversely, on both seeds**, across four
> landings:
>
> | | infra builds | seed 1 | seed 7 |
> |---|---|---|---|
> | pre-works | 1,032 | 10,558,680 | 10,474,865 |
> | T-73 | 57 | 10,606,309 | 10,583,150 |
> | T-81 | 31 | **10,633,441** | **10,599,130** |
> | R-IND17 | 66 | 10,582,211 | 10,546,759 |
>
> It peaks exactly where development is most broken, and **falls when R-IND17
> restores some** — which was predicted before the run and is what makes this an
> account rather than a pattern.
> It rose *because* development was being switched off: minerals denied to
> infrastructure buy hulls, and on a `k_high`-bound bed that takes worlds
> earlier. **The guard was rewarding the breakage.**
>
> `Hyades_trees_and_card_value.md` §2 says why: colony-years is **Expansion's**
> objective. The objective for an industry change is Growth's **work-years**,
> which would have flagged T-73 on the day it landed instead of three commits
> later through a build-mix census. The six-objective work is not only for card
> balance — it is what engine changes should be guarded against too.

#### R-IND17 measured — real, partial, and it settles the design question

| | infrastructure builds | banks skewed |
|---|---|---|
| pre-T-73 | **1,032** | — |
| T-73 | 57 | 1,494 / 1,515 (98.6%) |
| T-73 + T-81 (relief) | 31 | 1,533 / 1,551 (98.8%) |
| T-73 + **R-IND17** (completion) | **66** | 1,366 / 1,391 (98.2%) |

**The concentration argument is validated directionally**: 31 → 66 against T-81's
relief term, and 57 → 66 against no colour routing at all. Scoring the delivery
that *finishes* a bill really does assemble more payable bills than scoring the
delivery that relieves the scarcest colour.

**And it recovers about 6% of what T-73 cost.** 66 against 1,032. Bank
composition barely moved — 98.8% → 98.2% skewed — which is the expected shape:
R-IND17 routes the same single-coloured cargoes *better*, it does not make them
diverse. The supply constraint dominates and no routing rule can lift it.

**So the design question is settled by measurement rather than argument: a
colour-payable works bill cannot be made to work by freight alone on this
field.** It needs the relief valves the spec already names and the engine does
not have — the **Exchange** (§8.1, T-77) for buying colours from empires that
have them, and design law #1's **counter-graph**, where Red is the general key,
for substituting. Until one exists, T-73 is a constraint with no answer, and
§6.7's plan for stages 3–5 to land **inert** is not being met by stage 5.

**Recommendation, not yet actioned:** gate the colour bill behind a `SimConfig`
flag defaulting **off**, so the works layer lands inert as §6.7 intended and
switches on with T-77. The mechanism, its tests and this measurement all stay;
only the default changes.

**What this does not fix, and should not be asked to.** Even perfect internal
routing cannot give an empire a colour its own ground does not hold. That is
`§8.1`'s subject and the Exchange's job (T-77), and design law #1's counter-graph
— Red as the general key — is the other half. **T-81's premise was that freight
could answer the colour constraint on its own; that premise is now measured
false.**

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

### 6.12 T-74 measured — the curve landed, and the two seeds disagree

`fabrication_rate` and `extraction_rate` now read the infrastructure stock
through §2's Michaelis–Menten curve instead of a flat constant:

```
rate(u) = cap · u / (u + half),     u = infra · alloc_share(employment)
```

with `cap` the asymptote a Production card raises and `half` the knee a
Growth or Expansion card lowers. **The anchor is what makes it a pivot rather
than a retune**: `fab_cap` and `ext_cap` are set to exactly twice the flat
constants they replace, and `works_knee` is one rung's price split three ways,
so a **rung-I centre under default doctrine sits exactly on the knee and
fabricates and mines at precisely the rates that shipped**. Nothing below or
above rung I did before; everything does now.

Measured on `examples/work_years`, 3 seats, 4,000 yr, against the immediately
preceding commit:

| | work-years | colony-years | final works | colonies |
|---|---|---|---|---|
| seed 1, before | 1,366,140.0 | 10,593,850 | 426.80 | 3,340 |
| seed 1, after | **1,385,302.5** (+1.40%) | 10,717,850 (+1.17%) | 429.50 | 3,340 |
| seed 7, before | 1,596,222.5 | 10,558,625 | 502.70 | 3,349 |
| seed 7, after | **1,446,100.0** (−9.41%) | 10,688,675 (+1.23%) | 449.40 | 3,349 |

**The seeds disagree on Growth's objective and agree on everybody else's**, and
that is the finding rather than the mean of the two. Colony count is identical
on both seeds — the bed is `k_high`-bound, so it would be — and colony-years
rises ~1.2% on both. Work-years is the only metric that splits, and it splits
hard.

**This is not ratified and no number here should be quoted as a result.** Two
seeds cannot separate a −9.41% treatment effect from seed noise on a bed whose
four-seed spread is routinely wider than that (`CLAUDE.md` §2), and the standard
CRN bed is four seeds for exactly this reason. What the pair does establish is
the *sign structure*: the change is a **reallocation, not a gain** — the same
deepen-versus-expand trade R-O66 and §6.9 both landed on, now visible as one
metric falling while the other rises on the same run. `final works` moving
502.7 → 449.4 on seed 7 with colony-years up says the industry that was not
built became colonies.

**Open, and it is T-78's job**: run the four-seed bed and decide whether the
curve costs Growth anything real. Until then `fab_cap = 0.2` and
`ext_cap = 0.30` are **placeholders anchored to the flat constants**, not tuned
values — the anchor is the ratified part, the asymptote is not.

### 6.12a T-69 — concurrency has to be *spent*, not merely bought

`slips(F)` gives a yard more berths as its fabrication throughput rises, and
`build_time` divides that throughput among them. The first implementation did
exactly that and **committed one build per decision anyway.**

That combination is strictly worse than having no slips at all, and the way it
is worse is instructive: the extra berths sat idle while the one in use ran at
`F/slips` instead of `F`. All of concurrency's cost, none of its benefit. It
would have measured as a clean regression, on the correct guard, with a
completely wrong mechanism available to explain it — "slips slow a yard down"
is true of that code and false of the design.

`sys_build_decision` now **fills every free berth** before returning, and
`a_rich_yard_fills_every_berth_it_has_in_one_decision` pins it against `slips`
rather than against the number two, so the assertion survives a ratification of
`fab_cap` or `slip_throughput`.

**Measured** (`examples/work_years`, 3 seats, 4,000 yr, against T-74):

| | work-years | colony-years | final works | colonies |
|---|---|---|---|---|
| seed 1, before | 1,385,302.5 | 10,717,850 | 429.50 | 3,340 |
| seed 1, after | **1,450,642.5** (+4.72%) | 10,921,700 (+1.90%) | 441.30 | 3,340 |
| seed 7, before | 1,446,100.0 | 10,688,675 | 449.40 | 3,349 |
| seed 7, after | **1,620,105.0** (+12.03%) | 10,925,525 (+2.21%) | 494.60 | 3,349 |

**Both seeds up on both metrics**, which is what §6.7 stage 7 predicted, and the
two metrics agreeing is the part worth noting — §6.12's T-74 pair disagreed, and
the guard-inversion §6.9 documented is precisely a case of one rising while the
other falls. Colony *count* is identical on both seeds because the bed is
`k_high`-bound; the gain is entirely in *when*, which is what work-years and
colony-years are for on a saturated bed.

Two seeds is still not a ratification. What is solid here is the direction and
the mechanism, both of which were stated before the run.

**The delta is slips and nothing else, by ablation.** Forcing `slips ≡ 1` with
everything else in the commit in place reproduces the pre-T-69 numbers
**bit-for-bit** — 1,385,302.5 and 1,446,100.0 work-years, 10,717,850 and
10,688,675 colony-years — so the berth bookkeeping, the `Vec` of completion
times and the fill loop are all inert at one berth, and the gain is not some
other line in the change. `CLAUDE.md` §2 puts ablation first among the three
kinds of proof; this is the cheap version of it, and it also rules out the
awkward possibility that a refactor moved the bed by accident.

**It costs ~25% of throughput, and for once that is not entity count.**

| | vehicles | yr/s | margin vs T-24's floor |
|---|---|---|---|
| `slips ≡ 1`, seed 1 | 24,470 | 9.7 | 3.9x |
| T-69, seed 1 | 25,523 | **7.3** | 2.9x |
| `slips ≡ 1`, seed 7 | 25,984 | 9.0 | 3.6x |
| T-69, seed 7 | **25,200** | **6.6** | 2.6x |

Seed 7 carries **fewer** vehicles under T-69 and is 27% slower, which forecloses
the usual explanation. Entity count is the first-order cost in this engine and
it did not move; what moved is **decision count** — every commit now schedules
its own `BuildDecision`, so a yard running `k` berths raises `k` events where it
raised one, and the fill loop re-runs the candidate scan per berth. That the
vehicle count *fell* while work-years rose is the same trade the table above
shows from the other side: a yard with spare berths buys the infrastructure it
could not previously fit in.

At 3 seats / 4 kyr the margin is **2.6x**, and T-24's degradation is superlinear
in duration, so the 12-seat x 8-kyr corner is now firmly under the floor on any
extrapolation. That corner has still never been measured and extrapolating it
again would be the fourth time that table was wrong about a number nobody ran:
**T-66 is the next engine job**, and its first step is the measurement.

**Where the gain actually comes from is `t_lead`, not `F`.** Aggregate
throughput is unchanged by construction — `slips · (F/slips) = F` — so if the
mass term were the whole story, slips would be exactly neutral. It is not,
because the **lead time is per-build and is not divided**. At the T-74 anchor a
rung-I yard runs two berths at `F_slip/2`:

| | one build at a time | two berths |
|---|---|---|
| `t_build`, Limited hull | 2.2 yr | 2.4 yr |
| hulls per year | 0.455 | **0.833** |

So a yard turns each hull around *slightly slower* and produces them **1.8x
faster**, and the whole of that is `t_lead` being amortised across berths rather
than paid serially. That is the mechanism, it is arithmetic rather than a
hypothesis, and it predicts the effect vanishes for large hulls — where the mass
term dominates the lead time — which is the design's "a General hull stays a
twelve-year commitment" (§3.2) arriving for free.

### 6.14 T-71 + T-72 measured — two of §4.5's three predictions, and the third was never crowding's to keep

§4.5 predicted **fewer, larger, closer** outposts. `examples/crew_census`,
3 seats, 4,000 yr, three configurations — the middle one is T-71 with crews
still a flat 3, i.e. the half-built state, kept deliberately because it is what
separates the law from the policy:

| seed 1 | sites worked | mean crew | max crew | ore/site | mean distance |
|---|---|---|---|---|---|
| T-69 (no crowding) | 2,494 | 5.97 | 9 | 6,491.4 kt | 66.74 ly |
| T-71 only (flat crews) | 2,494 | 8.03 | 9 | 6,424.5 kt | 71.75 ly |
| **T-71 + T-72** | **2,494** | **26.22** | **900** | **6,491.4 kt** | **59.95 ly** |

| seed 7 | sites worked | mean crew | max crew | ore/site | mean distance |
|---|---|---|---|---|---|
| T-69 (no crowding) | 2,516 | 5.50 | 9 | 4,680.5 kt | 61.12 ly |
| T-71 only (flat crews) | 2,516 | 7.73 | 9 | 4,646.2 kt | 69.28 ly |
| **T-71 + T-72** | **2,516** | **24.00** | **851** | **4,680.5 kt** | **58.38 ly** |

**Larger: yes.** Mean crew 4.4x, and the maximum goes 9 → 900 — the design's
"hundreds or thousands of miners at a high-value mining outpost", arriving for
the first time. The share of sites with more than one miner falls from 100% to
~89%, which is the *other* half of §4.3 showing up: a `Band I` pebble has one
vein, so it now correctly gets one hull.

**Closer: yes.** Mean distance from the working empire's homeworld falls 10.2%
and 4.5%. Concentration is what §4.5 said would give T-66 its lever.

**Fewer: no, and the reason is that it was never crowding's to deliver.** Site
count is **identical to the digit** in all three configurations, on both seeds.
Which rocks get worked is decided by `rank`, and neither the crowding law nor
the crew policy is an input to it — so §4.5's first prediction was about a
function neither change touches. **Amend §4.5 to "larger and closer"**; making
it *fewer* is a ranking change and belongs with T-66 rather than here.

**And ore per site returns to the baseline exactly** — 6,491.4 and 4,680.5 kt,
the same digits as T-69. That is not a coincidence and it is the best available
check on the whole change: a rock is a **finite stock** mined to
`density_floor`, so the total it yields cannot depend on how it was crewed.
§4.5 says this in words — "a bigger crew does not raise what a field yields in
total, it brings that total forward" — and the census confirms the arithmetic
held through both changes. Mass conservation (design law #11) has no exception
for mining.

**What T-71 alone bought was nothing, and it is worth keeping the row.** With
the law in and crews still flat, site count and ore/site are unmoved while mean
crew rises 35% and cross-player contention rises 33% (1,671 → 2,214 on seed 1).
Every crew is under-sized against a vein count spanning 1 to 1,000, so each one
works its rock more slowly, rocks stay unexhausted longer, and more hulls pile
onto the same sites for longer. **A pure cost with no compensating gain**, which
is exactly what §6.7's stage order predicts for a stage measured without its
successor — and why the two landed as one change. The first hypothesis for that
row was "the empire compensates by opening more sites"; the census refuted it in
one column, which is what a census is for.

### 6.15 The crew fraction cannot be ratified on this bed, and that is the finding

`miner_vein_fraction` shipped first at **0.3** — the value that gives a `Band II`
body T-57's ratified crew of three and a `Band IV` seam three hundred, i.e. the
design's "hundreds or thousands of miners at a high-value mining outpost". It
loses, decisively, on both objectives:

| | work-years | colony-years | vehicles |
|---|---|---|---|
| seed 1, T-69 | 1,450,642.5 | 10,921,700 | 25,523 |
| seed 1, `f = 0.3` | 1,295,800.0 (**−10.67%**) | 10,569,400 (−3.23%) | **83,826** (×3.28) |
| seed 7, T-69 | 1,620,105.0 | 10,925,525 | 25,200 |
| seed 7, `f = 0.3` | 1,250,405.0 (**−22.82%**) | 10,319,975 (−5.54%) | **74,998** (×2.98) |

**Both seeds, both metrics, and the mechanism is not in doubt** — §6.14's census
shows ore per site returning to the baseline *exactly*. A deposit is a finite
stock mined to `density_floor`, so a crew cannot raise what it yields; it can
only bring that yield **forward**. Crew is therefore bought at a cost linear in
hulls to buy a benefit that is purely a time shift, and crowding makes the trade
strictly worse than the law T-57 ratified under: three miners on a `Band III`
body now return `(3/100)^½ / (1/100)^½ = 1.73x` one miner's share, where the old
linear law returned `3x`.

**So the honest reading is not "mining is overpriced", it is "this bed cannot
value mining at all."** `examples/reach_limit` established that the binding
constraint on the standard bed is **`k_high`**, not the economy, and R-AC17
ruled minerals out specifically. Bringing ore forward is worth something only
where minerals are what you run out of. On a bed where they are not, the
objective-optimal crew is **one hull everywhere**, and ratifying `f` here would
ratify *"do not mine"* — a statement about the test bed, not about the design.
`CLAUDE.md` §7 already says this in general form: **before tuning another
economic knob, check whether the thing being optimised is what is actually
scarce.**

**The sweep is monotone**, which is what settles it — every crew size tested is
worse than the one below, on both seeds and both metrics:

| `f` | max crew | vehicles (s1) | work-years s1 | work-years s7 | colony-years s1 | colony-years s7 |
|---|---|---|---|---|---|---|
| — (flat 3, T-69) | 9 | 25,523 | — | — | — | — |
| **0.07 (shipped)** | **204** | 37,706 | −1.00% | −6.26% | −0.97% | −2.30% |
| 0.3 | 900 | 83,826 | −10.67% | −22.82% | −3.23% | −5.54% |

There is no interior optimum to find: the objective's preferred crew is **one
hull on every rock**, because on this bed a crew buys only a time shift of a
resource the empire is not short of.

**Decision: `f = 0.07`, and it is a design call made against the metric, not
with it.** It gives a `Band IV` seam **204 miners where it had 9** — the
order-of-magnitude, richness-scaled crew this stage exists to make possible — at
a measured mean cost of **−3.6% work-years and −1.6% colony-years** on a bed that
provably cannot price the benefit. `f = 0.3` delivers the literal "hundreds or
thousands" and costs 10–23%; that is a larger bill than should be paid on
evidence this weak, and it is a decision for the author rather than for a
measurement that cannot see the other side of the ledger.

What is *not* a judgement call, and is the reason shipping anything here is
defensible: **ore per site is invariant to the digit** across the whole sweep
(§6.14). The crew redistributes when ore arrives; it cannot change how much
there is. So the worst case for a wrong `f` is a fleet sized wrong, not an
economy sized wrong.

**R-IND18 is narrowed, not answered.** What it needs is not more seeds but a
**different bed**: one where minerals bind. Two candidates, neither built — a
mineral-scarce galaxy configuration, or the 8-kyr multi-metric bed of
`Hyades_trees_and_card_value.md` §3, where Production's fleet-years objective
values hulls directly and would price a crew the way colony-years cannot.

### 6.16 T-87 — crew stops being a parameter, and the sign inverts

**Author's ruling: "miner crew count should not be a parameter but should fall
out of mineral demand. As the simulation shows, building mining outposts without
regard to mineral demand slows each tree's objectives."** §6.15 had already
measured that from the inside — the crew sweep was monotone in the wrong
direction and there was no interior optimum — and had shipped a compromise
value. The ruling replaces the value with a model.

**Both retired knobs were the same mistake twice.** `miners_per_outpost` (T-57,
a flat hull count) and `miner_vein_fraction` (T-72, a share of the rock) were
each a number someone had to choose against an objective that could not price
it. Neither had a term for the only thing that decides whether a mine is worth
opening: **whether anyone can spend what it produces.**

**The law.** Demand, in kilotons per year, is what the founding centre can
consume and currently cannot get:

```
D = fabrication_rate(centre) · mineral_pressure(centre)
```

Both terms already existed and both already ran on this decision path.
`fabrication_rate` is the rate the yard turns minerals into mass — the only
sink that consumes ore — and `mineral_pressure` is `1.0` when the centre is
broke for its next rung and `0.0` when it is comfortable. Supply is §4.3's law
read forwards, so the crew is that law inverted:

```
supply(n) = ε · S · (n/N)^β / T          kt/yr,  T = mining_tick_years
n*        = N · (D · T / (ε · S))^(1/β)   clamped to [1, N]
```

**The sign inverts, and that is the whole finding.** Deposit mass grows as
`N^{3/2}` while the demand target does not grow at all, so `n*` **falls** as the
body gets richer — a rich rock meets the same demand with fewer hands. Under a
flat count the crew was richness-blind; under a vein fraction it *rose* with
richness. Both were backwards, and §6.15's monotone loss is what that costs.

**Measured** (3 seats, 4,000 yr), against the value T-87 replaces and against
the flat-3 bed before crowding landed:

| | work-years s1 | work-years s7 | colony-years s1 | colony-years s7 | vehicles s1 |
|---|---|---|---|---|---|
| T-69, flat crew of 3 | 1,450,642.5 | 1,620,105.0 | 10,921,700 | 10,925,525 | 25,523 |
| `f = 0.07` (T-72, shipped) | 1,436,087.5 | 1,518,692.5 | 10,816,000 | 10,673,925 | 37,706 |
| `f = 0.3` | 1,295,800.0 | 1,250,405.0 | 10,569,400 | 10,319,975 | 83,826 |
| **T-87, demand-derived** | **1,495,212.5** | **1,540,712.5** | **10,888,100** | **10,983,925** | **24,801** |

**It beats the value it replaces on every metric and every seed** — +4.12% and
+1.45% work-years, +0.67% and +2.90% colony-years — while carrying **34% fewer
vehicles**. Against the pre-crowding flat-3 bed it is a wash on work-years
(+3.07% / −4.90%, seeds disagreeing as they have throughout) and slightly ahead
on colony-years, with a smaller fleet on both seeds. **A parameter was deleted
and nothing got worse**, which is the outcome that justifies a model change over
a retuning.

**The census says why, and one column changed character.**

| seed 1 | sites | mean crew | max crew | ore/site | distance |
|---|---|---|---|---|---|
| T-69, flat 3 | 2,494 | 5.97 | 9 | 6,491.4 kt | 66.74 ly |
| `f = 0.07` | 2,494 | 7.46 | 204 | 6,491.4 kt | 63.20 ly |
| **T-87** | 2,494 | **2.95** | **12** | **5,818.9 kt** | 70.78 ly |

Mean crew is now **below the flat 3 it replaced**, and the maximum is 12 rather
than 204: demand-driven sizing says almost every rock wants one or two hulls,
because one miner on a rich seam already lifts orders of magnitude more than a
rung-I yard can absorb.

**And ore per site stops being invariant.** It was 6,491.4 kt to the digit
across T-69, T-71 and T-72 — every worked rock was mined to `density_floor`, so
the total could not depend on crewing. At 5,818.9 it is not: **the empire now
leaves ore in the ground it has no use for.** That is not a leak, it is the
mechanism working. A finite stock mined by a crew sized to demand is a stock
that stops being mined when demand stops.

**R-IND20, open: demand is read at the founding centre, not empire-wide.** An
outpost feeds the whole empire through freight, so the correct demand is the
empire's unmet total. That is an `O(planets)` scan on a decision path
(`CLAUDE.md` §4) and would need the `holdings_centroid` memo treatment. The
centre that pays for the pair is the defensible local proxy; the difference is
what R-IND20 is for.

**R-IND18 is closed as a crew question and survives as a rate question.** There
is no crew magnitude left to ratify. `ε`, `β` and `veins_per_band` are still
placeholders, and §6.15's point stands about which bed could price them — but
they now set *how fast a given crew works*, not *how many hulls to buy*, which
is a much smaller blast radius.

### 6.13 T-75b — the write path, and why it lands before any card uses it

`CardEffect::WriteWorks(WorksWrite)` is the card layer's entry into §6.2's state.
Two components per empire:

| Component | What it holds |
|---|---|
| `works_writes` | the multiset — every `(CardId, WorksWrite)` this empire has played |
| `works` | the **fold** of that multiset, in `CardId` order |

A play appends to the first and re-derives the second. It never multiplies a
coefficient into the live state, and the reason is §6.5's: a running product
accumulates in **play** order, and float multiplication is not associative, so
two empires holding the same cards would hold state differing in its last bits.
That is a desync, not a rounding difference — and it is invisible to a test of
the fold alone, because a broken `apply_card_effect` can bypass the fold
entirely while every fold test still passes. So the property is asserted on the
state *the simulation reads*:
`playing_works_cards_in_any_order_leaves_the_same_empire_state` plays six cards
in five permutations and compares `to_bits()`.

**No tier-0 card carries the effect yet, deliberately.** The eighteen tier-0
cards are the balance scaffolding every measurement so far has run against;
reassigning one to a works effect changes what those measurements measured.
`no_tier0_card_writes_works_yet_and_that_is_on_purpose` pins the count at zero,
so the first works card is a change that has to edit a number rather than one
that slips in. That also keeps the layering **inert** in §6.7's sense: the state
exists, the fold runs, the pipeline reads it, and with nothing played every
coefficient is `1.0`.

**R-IND4 is now satisfied at both levels** — `cards::works_fold_is_order_independent`
on the algebra, and the engine test above on the state — and §6.5's requirement
that it exist *before the second industrial card* is met with room to spare,
since the first one does not exist either.

---

---

### 6.17 The ore is idle because Doctrine never asks for it — not because nothing wants it

**This section replaces a wrong conclusion, and the correction is the useful
part.** The first version measured that 91–98% of held ore sits idle while unmet
demand is ~1/1000 of the pile, and concluded *the mineral economy has no demand
side*. **The author rejected that and was right:** "there's uses for minerals but
the Doctrine needs to shift to demand them."

**Measured** (`examples/infra_ceiling`, 600 planets / 1,500 yr):

| | seed 1 | seed 7 |
|---|---|---|
| mean infrastructure | **Band 1.051** | **Band 1.047** |
| mean ceiling `k` | **Band 3.595** | **Band 3.611** |
| colonies **at** their ceiling | **0 (0.0%)** | **0 (0.0%)** |
| unbuilt headroom across the empire | **756 Bands** | **769 Bands** |
| banked ore available to build it | 19,619 kt | 57,464 kt |
| survey coverage | 100% per player | 100% per player |

**Colonies sit at 29% of their own ceiling and not one is capped.** The sink is
not missing; it is 756 Bands wide and the empire is standing next to it holding
the money. Survey is saturated, so scouts are not a sink either — both of the
obvious uses are checked, and one of them is wide open.

**Why the first measurement missed it, and it is the artifact list's oldest
shape.** `unmet_colour_demand` sums `colour_deficit`, which is the shortfall
against a centre's **next rung only**. A centre that can afford its next rung
reports **zero demand** — even with three more Bands of headroom above it. So the
metric measured *demand the policy had already decided to express*, and the
policy expresses almost none. **A metric that reads a decision's output cannot
tell you what the decision declined to ask for.**

#### The mechanism, and both sides of it are now measured (R-O68 closed)

`production_choice` preferred depth when `b · deepen_headroom ≥ (1 − b) · score`.
At the shipped `reinvest_bias = 0.5` that reduced to **`headroom ≥ score`** — and
the two sides were in different units, which is the fault R-O68 named:

| side | what it is | measured |
|---|---|---|
| `deepen_headroom` | a **Band difference**, bounded by 4 | mean **2.55** (3.60 − 1.05) |
| `score` | `rank`'s **unbounded weighted score** | p05 4.40 / median **6.17** / max 12.16 |

**Headroom loses every comparison it is ever in.** 2.55 against a median 6.17 is
not a close call: the deepen branch was *arithmetically unreachable* at any bias
below ~0.8, which is what `examples/score_scale` measured and what the ceiling
census confirms from the other end. **70 infrastructure builds against 18,373
hull builds** on the standard bed (`examples/bank_mix`) is the same fact counted
a third way.

### 6.18 R-O68 resolved — the comparison is a return per kilotonne, and the branch is still cold (R-O85 opened)

**The fix.** Both sides of the deepen-vs-expand test are now `rank` score per
kilotonne committed, and neither half introduces a constant:

```text
expand = score / outward_cost                 // this candidate, at its price
deepen = w_k · min(1, headroom) / infra_cost  // one rung, at its price
```

`w_k` is the weight `rank` already puts on one Band of `k_potential` (§3 of
`Hyades_autopilot_colonization_growth.md`), and it is the right converter because
the two moves trade in one commodity: expansion **acquires** a world's Bands of
ceiling, deepening **realises** a Band of them here. `min(1, ·)` is what a rung
actually delivers, since `apply_build` steps to the next whole rung whatever the
headroom is — a last partial step pays a full price for less than a Band.

The comparison is then an odds ratio: depth wins when `b/(1 − b) ≥ expand/deepen`.
That crossover is **state-dependent**, which is the graded region the old form had
nowhere — a centre facing a cheap next rung and a mediocre candidate deepens where
one facing an expensive rung and a hub does not.

**Measured** (`examples/deepen_census`, 600 planets / 1,500 yr, 3 seats):

| `reinvest_bias` | infra builds | hull builds | colonies | mean infra | colony-years |
|---|---|---|---|---|---|
| **seed 1** | | | | | |
| 0.00 / 0.50 / 0.90 / 0.95 / 0.96 | 88 | 484,136 | 3,309 | 1.028 | 2,540,752.7 |
| 0.97 | 88 | 477,697 | 3,311 | 1.027 | 2,541,959.0 |
| 0.98 | **326** | 484,686 | 3,308 | **1.099** | 2,534,733.4 |
| 0.99 | 216 | 317,278 | 3,323 | 1.066 | **1,846,450.9** |
| 1.00 | 0 | 564 | **3** | 2.000 | 0.0 |
| **seed 7** | | | | | |
| 0.50 / 0.95 | 83 | 512,499 | 3,334 | 1.026 | 2,608,344.6 |
| 0.99 | 248 | 393,004 | 3,343 | 1.075 | **1,934,059.8** |

Three things it says, and the first is the guard:

- **The run is bit-identical to the old form everywhere below `b = 0.96`** —
  same build mix, same colony count, same colony-years to the decimal, on both
  seeds. This is a units fix and not a behaviour change, which is exactly the
  property a units fix should have.
- **The usable range of the dial grew, and that is the whole behavioural
  payoff.** Under the old form the cliff sat between 0.5 and 0.9 with *nothing
  working past it*: seed 1 at `b = 0.9` reached 70 colonies, seed 7 at `b = 0.95`
  reached **3** — the homeworlds alone. Under the new form 0.97 and 0.98 are
  working empires that genuinely deepen (326 infra builds against 88, mean infra
  Band 1.028 → 1.099) at a cost of −0.24% colony-years, and the collapse moves to
  `b = 1.0`, where `w_expand` is identically zero by definition.
- **The branch is still cold at the shipped `0.5`, and the reason is now a
  price.** The empirical crossover between 0.96 and 0.98 is an odds ratio of
  **24–49**, which is the infra ladder's own ratio read from the other side: the
  rung above the founding one costs **0.9 kt** where a Medium coloniser costs
  **0.1 kt**, and the coloniser brings a whole world with its own ceiling, its own
  ore and its own yard. Expansion *ought* to win. **The dead branch was the right
  answer reached for a wrong reason.**

#### R-O85 — infrastructure is priced as if it were the scarce thing

What the fix exposes is not a tuning question, and it is the reason §6.17's sink
stays unspent:

At the shipped constants, printed off the engine's own functions:

| rung | stock to stand there | step to the next | `fabrication_rate` | `slips` |
|---|---|---|---|---|
| 0 | 0.02 kt | 0.08 | 0.0333 kt/yr | 1 |
| **I** | **0.10 kt** | **0.90** | 0.1000 | 2 |
| II | 1.00 kt | **19.0** | 0.1818 | 2 |
| III | 20.0 kt | **780** | 0.1990 | 2 |
| IV | 800 kt | — | 0.2000 | 2 |

For scale: a Limited hull is 0.02 kt, a **Medium coloniser 0.10 kt**, a General
hull 1.00 kt, a mining pair 0.12 kt.

- **The step above the founding rung costs nine colonisers**, and every colony is
  founded at rung I (`founding_infra`), so that is the step every colony in the
  empire is looking at.
- **`fabrication_rate` saturates by rung II.** `fab_cap = 0.2` with
  `works_knee = rung(1)/3` and an even three-way `alloc_w` puts the whole ladder
  inside one hyperbola's knee: the 19-kt step buys **+0.017 kt/yr** and the 780-kt
  step buys **+0.001**.
- **`slips` is pinned at 2 from rung I onward**, for the same reason.
  `slips(F) = 1 + ⌊F / slip_throughput⌋` with `slip_throughput = 0.1` and
  `F < fab_cap = 0.2`, so **no amount of infrastructure ever buys a third berth.**
  T-69's "industry buys more ships at once" is bounded at two by the ratio of two
  constants, and nothing in the engine says so out loud.
- So rungs III and IV are, today, almost pure cost. The only thing that keeps
  scaling past rung II is `extraction_rate`, which is linear in the stock — and
  extraction is not what binds on a bed holding 19,619 kt of unspent ore.

That is the sink's real shape: it is 756 Bands wide and priced at a ladder whose
top half buys nothing the empire is short of. Fixing it means moving MC-tuned
surfaces — `fab_cap` and the knee, the infra ladder's anchor, or giving
infrastructure a consumer that does not saturate — and every one of those needs
ratification (`CLAUDE.md` §6), so none is taken here.

### 6.19 R-O87 — `reinvest_bias` cannot move work-years, and that is an identity

**The obvious next move after R-O68 was to tune the bias against Growth's own
objective rather than Expansion's.** `examples/work_years` already argues that
colony-years scores anything mineral-allocating with the sign reversed, and
`reinvest_bias` is *the* mineral-allocating knob. So: sweep it against
work-years, `∫ Σ_p infra_p dt`.

**It does not move.** And before the measurement, the reason it cannot:

| route | what it bills | works it adds | works per mineral |
|---|---|---|---|
| **deepen** | `infra_step_price / eta_works` | `infra_step_price` (the stock moves to the next rung) | `eta_works` |
| **found** | `hull_cost(coloniser)` | `founding_infra = hull_cost` | **1** |

The second row is design law #11 arriving somewhere nobody was looking for it.
A recycled hull's minerals *are* the new colony's works stock (T-70) because a
hull's mass is its cost (R-O57) — so at the card-free `eta_works = 1` the two
routes are worth the same to the metric, **to the last bit**, at every rung a
centre can stand on. `a_mineral_buys_the_same_works_whether_it_deepens_or_founds`
pins it. The bias is choosing between equals, and everything downstream breaks
the tie *for expansion*: a colony mines, grows and builds, while a rung past II
buys almost no fabrication and no extra berth at all (§6.18's R-O85 table).

**Measured** (`examples/work_years`, 4,000 yr, 3 seats, paired per seed under
CRN). A 1,500-year screen put the best point at `b = 0.972`, on a narrow ridge:
flat and bit-identical below 0.965, and off a cliff above 0.98 (−6.8% at 0.98,
−35% at 0.99, and at 1.0 the empire never expands at all — 3 colonies).
Confirmed at the objective horizon:

| bed | mean | SE | verdict | seeds positive |
|---|---|---|---|---|
| standard four (1, 7, 42, 31337) | **+2.33%** | 0.96 | 2.4 SE — clears the bar | **4/4** |
| four it was not chosen against (2, 3, 5, 11) | **−1.70%** | 2.42 | 0.7 SE | 1/4 |
| **pooled, eight seeds** | **+0.32%** | **1.42** | **0.22 SE — flat** | 5/8 |

**The first row is a coin landing on its edge, and the second row is what
proved it.** `CLAUDE.md` §2 already carried that warning from `survey_reserve`
— "a 2.4-SE reading on four seeds is not a finding" — and this is the first time
the project has *refuted* one rather than merely doubted it. The refutation cost
four runs and is far stronger than more seeds on the same bed would have been:
a replication set cannot inherit whatever made the original four agree.

The neighbourhood says the same thing from another direction. On the standard
bed at 4,000 yr, `0.968` scores **−0.20% ± 0.76** and `0.975` scores **+2.24% ±
2.08** — adjacent values swinging the full magnitude of the "effect" in both
directions. That is a chaotic reordering of a compounding run, not a gradient,
and no value on it is a place to stand.

**So `reinvest_bias` stays at 0.5** — held rather than defaulted. Colony-years
and colony count are unmoved across the whole comparison too (+0.009% and
identical per seed), so this is not a trade being declined; there is no trade.

**What would make it worth sweeping again**, and it is the useful half of the
result: `eta_works`. It divides the deepening bill and nothing else, so a
Production card genuinely does make a mineral buy more works, and it is the
tie-break the baseline policy has no access to. The identity test fails the day
that lands, which is exactly when someone should re-read this section.

### 6.19a The identity was half an argument — the other half favours deepening, and something else is eating it

**Objection, and it was right:** a colony is founded with a recycled hull and
*cannot keep improving that way*, so §6.19's identity — which says the two routes
buy the same immediate works — says nothing about what they buy afterwards.
Deepening delays the first ship and buys build rate forever, and over 1,500 years
an integral should reward that.

**Priced off the engine's own functions** (`t_lead = 2.0`, a Medium coloniser is
0.10 kt):

| rung | stock | `F` kt/yr | slips | `t_build` | hull/yr | next step costs |
|---|---|---|---|---|---|---|
| I | 0.10 kt | 0.1000 | 2 | 4.000 yr | 0.500 | 0.90 kt = **9 colonisers** |
| **II** | 1.00 kt | 0.1818 | 2 | 3.100 yr | **0.645 (+29.0%)** | 19 kt = 190 colonisers |
| III | 20.0 kt | 0.1990 | 2 | 3.005 yr | 0.666 (+3.2%) | 780 kt = 7,800 colonisers |
| IV | 800 kt | 0.2000 | 2 | 3.000 yr | 0.667 (+0.2%) | — |

So the objection is arithmetically right about the rung that matters: **+29% for
nine colonisers pays back in 62 years** and should be worth ~200 extra hulls over
the horizon. §6.19's "everything downstream breaks the tie for expansion" was
asserted, not measured, and it is wrong.

It is wrong about slips, and that is the engine's fault rather than the
argument's: `slips(F) = 1 + ⌊F / slip_throughput⌋` with `F < fab_cap = 0.2` and
`slip_throughput = 0.1` gives **two berths at every rung, forever** (R-O85).

**Three facts eat the +29%, and none of them is the identity**
(`examples/founding_tree`, 3 seats, 1,500 yr):

| | shipped `b = 0.5` | deepening **ablated entirely** | `b = 0.99` |
|---|---|---|---|
| colonies the homeworld founds **itself** | **33.3** | **32.3** | **35.7** |
| year its first one lands | **61.6** | **61.6** | **61.6** |
| hulls the homeworld built | 176 | 178 | 213 |
| the homeworld's final rung | **2.000** | **2.000** | **2.000** |
| infrastructure builds, empire-wide | 620 | **0** | 469 |
| work-years | 976,147 | 258,752 | 617,858 |
| **fleet-years** `∫ vehicles dt` | **21,802,650** | 20,584,525 | **17,506,750** |

1. **A homeworld never deepens, in any arm.** `galaxy.rs` generates homeworlds at
   `Band::new(2.0)` — rung II, exactly where fabrication saturates — so the rung
   worth +29% is one they are *born with*, and the next costs 190 colonisers for
   +3.2%. The ablated arm has **zero** infrastructure builds and still reports
   rung 2.000. This is also why the first founding lands at **61.6 yr in all
   three arms**: there is no delayed-first-ship trade to make.
2. **The yard is not the constraint.** Homeworld utilisation is **18.8%** — it
   builds 176 hulls where rung II could have built ~930 over the same span.
3. **A declined build costs thirty years of yard time.** Mean gap from one
   production decision to the next at a homeworld: **1.5 yr after a committed
   build, 29.6 yr after an `Idle`** (seed 7: 1.6 and 31.8). `commit_one_build`
   returning `None` leaves the yard free and *schedules nothing* — the next
   attempt is the economy tick, `cycle_years = 50`. With 18% of decisions
   idling, **81% of the homeworld's production timeline is spent waiting out
   that cadence**, which reconciles exactly with the 18.8% utilisation.

**So build rate governs about a fifth of a centre's timeline, and deepening is
the only thing this knob can buy.** +29% on 19% of the timeline is +5.5% at the
absolute best, available only to a centre below rung II, against a 50-year retry
that has nothing to do with infrastructure. That is the bottleneck, and it is
**T-88** — already open, and now quantified.

**Was work-years the wrong metric?** Not wrong, but not the informative one, and
**no objective would have been**: the knob's only mechanism is throttled to a
fifth of the timeline before it reaches any metric. Fleet-years — the objection's
own suggestion — is the better-shaped instrument for this question and it is
carried above; it agrees, and more sharply than work-years does (deepening at
`b = 0.99` costs **−20%** of fleet-years where it costs −37% of work-years and
−0.3% of colonies).

**What this changes.** §6.19's conclusion stands — `reinvest_bias` is held at 0.5
and no measurement supports moving it — but its *reasoning* was half an argument,
and the half it was missing pointed the other way. **The order to fix things in
is now T-88 first, then R-O85** (`fab_cap`, the knee, or a berth count that
actually grows), and only then re-sweep this knob. Sweeping it before the retry
cadence is fixed measures the cadence.

### 6.19b R-O88 — there is no build-wide axis, and a yard cannot use the rate it is allowed

**Author's objection: "we made slips and latency part of the building, so there's
no saturation on Infra."** That is what §3.2 says and it is not what the engine
does. Two ratified sections disagree and the engine implements the intersection
of them rather than either.

| | claim |
|---|---|
| **§3.2** (T-69) | `slips(F) = 1 + ⌊F / F_slip⌋` — "the *build wide* axis, and **it scales without limit, as asked**" |
| **§6.3** (T-74) | `F = cap · u/(u + half)` — Michaelis–Menten, so `F < fab_cap` at **every** finite infrastructure |

`slips` reads `F`. So the second bounds the first, and the bound is the ratio of
two constants chosen independently in different landings:

> **`fab_cap / slip_throughput = 0.2 / 0.1 = 2` is the entire build-wide axis.**

**Closed, not short.** A yard standing on **10¹² kt** of infrastructure still has
two berths, because the asymptote is never reached. And homeworlds are
*generated* at `Band::new(2.0)` — rung II — so every homeworld is **born past the
only step the axis has**. Pinned by
`the_build_wide_axis_is_two_berths_wide_and_closed`.

**§6.3's reconciliation does not reconcile anything.** It reads: *"`slips` (§3.2)
growing linearly in `F` does not contradict a bounded `F`: the bound is per yard,
and an empire has many yards."* §3.2's claim is about **one centre's berth
count**; "an empire has many yards" is about the **empire total**. Those are
different quantities, so the sentence answers a question nobody asked. Withdrawn
above.

#### The second defect is independent of the cap: `slips` ignores `t_lead`

A berth's cycle is `t_lead + m/per_berth` and **only the second term
fabricates**, so provisioning berths from a rate alone leaves most of each cycle
idle. Against the `F` the same rung *allows* (rung II, `F = 0.1818 kt/yr`):

| hull | dry mass | `t_build` | sustained output | **as % of `F`** | berths to saturate `F` | lead as % of cycle |
|---|---|---|---|---|---|---|
| Limited | 0.02 kt | 2.22 yr | 0.0180 kt/yr | **10%** | 20.2 | 90% |
| Medium | 0.10 kt | 3.10 yr | 0.0645 kt/yr | **35%** | 5.6 | 65% |
| General | 1.00 kt | 13.00 yr | 0.1538 kt/yr | 85% | 2.4 | 15% |

**A rung-II yard building Medium hulls emits a third of the throughput it is
allowed**, and building Limited hulls, a tenth. This survives whatever happens to
the cap: even with `F` unbounded, `1 + ⌊F/F_slip⌋` under-provisions by
`1 + t_lead · F_slip / m` — a factor of **11 for a Limited hull, 3 for a
Medium**. The small hulls are worst hit, which is precisely backwards for a
design whose expansion loop runs on Limited and Medium hulls.

#### Decided: option C — split `F`'s two roles

**Author's call.** `fab_cap` now bounds the rate **per berth** — the *quality*
axis — and `slips` scales with the fabrication share of the infrastructure
**stock** — the *quantity* axis. Neither tree is capped on the axis the other is
strong in, and §5.3's table reads off the engine directly.

```text
u          = infra · alloc_w[Fabrication] / Σ alloc_w     // the stock both axes buy from
slips(u)   = 1 + ⌊u / infra_per_slip⌋                     // quantity — unbounded
berth_rate = fab_cap · u / (u + half)                     // quality  — saturating
t_build    = t_lead + m / berth_rate                      // one hull sits in one berth
fabrication_rate = slips × berth_rate                     // the planet's total, for demand
```

**`fab_cap` went 0.2 → 0.1, and that is a re-denomination rather than a retune.**
The old code divided a planet-wide rate by a `slips` that was *always exactly 2*
at every rung a centre can occupy, so halving the ceiling reproduces the old
per-berth rate **bit-for-bit** —
`turnaround_is_unchanged_and_only_the_berth_count_opened` asserts it at every
playable rung. Two things follow for free: §3.3's approved schedule now reads off
one constant (`t_lead + m / fab_cap` is 2.2 / 3.0 / 12.0 yr, the table as
approved), and **turnaround did not move at all** — the only thing that changed
is how many hulls a yard can have in the water.

**`slip_throughput` is deleted.** Its meaning ("one slip's throughput, kt/yr") is
what `fab_cap` bounds now, and the berth *size* is derived rather than stored:
one berth occupies one Limited hull's worth of stock, so a slipway is sized like
the smallest thing it can lay down. That anchor is a **placeholder** — the form
is what R-O88 settles, not the unit — and it is chosen so a rung-I yard keeps
exactly the two berths it already had:

| rung | fabrication stock | berths, before | **berths, after** |
|---|---|---|---|
| I | 0.033 kt | 2 | **2** |
| II | 0.333 kt | 2 | **17** |
| III | 6.67 kt | 2 | **334** |
| IV | 267 kt | 2 | **13,334** |

**Measured** (`examples/founding_tree`, 3 seats, 1,500 yr, `b = 0.5`):

| | seed 1 before | seed 1 after | seed 7 before | seed 7 after |
|---|---|---|---|---|
| **fleet-years** `∫ vehicles dt` | 21,802,650 | **27,503,725 (+26.1%)** | 20,913,800 | **27,962,225 (+33.7%)** |
| work-years | 976,147 | 1,112,797 (+14.0%) | 875,395 | 824,095 (−5.9%) |
| colonies | 3,308 | 3,296 | 3,331 | 3,325 |
| homeworld's first founding | 61.6 yr | **58.5 yr** | 57.4 | 57.4 |
| throughput | 88.7 yr/s | **110.7 yr/s** | 86.5 | 113.5 |
| ns/event | 69,355 | **52,278** | 70,952 | 54,066 |

**The fleet is a quarter to a third larger and the engine got *faster*** — 88.7 →
110.7 yr/s with 23% more vehicles, because a quarter of the events it used to
process were decisions that declined and stalled. Colony count and colony-years
barely move, which is the expected answer on a bed `k_high` already saturates
(§6.17): more yard cannot buy worlds that the classifier does not admit.

**And it largely dissolves T-88 as a side effect, which nothing predicted.** The
retry cadence bit because a declined build left the yard with nothing scheduled;
with berths instead of two, some *other* berth clears and re-triggers the centre
long before the economy tick does. Measured at a homeworld:

| | before | after |
|---|---|---|
| gap after a committed build | 1.5 yr | **0.1 yr** |
| **gap after an `Idle`** | **29.6 yr** | **7.6 yr** |
| decisions taken | 216 | 388 |
| share that idled | 18% | 47% |

T-88 is still worth doing for the reason it was opened — economic *granularity*,
a 50-year Euler step on a logistic — but it is no longer the dominant throttle,
and the 81%-of-timeline figure in its register entry is superseded.

**Cost, and where it landed.** Not throughput, which improved; the **test
targets**, which is where entity count always lands first (`CLAUDE.md` §2). Unit
went 19 → 54 s and determinism 30 → 58 s, both fixed in the same commit and both
by the lever that section prescribes — `paired_cfg` 250 → 120 yr with an
`events_processed` floor guarding the trim, and `full_run_reports_are_bit_identical`
given a **per-seat-count** horizon instead of a uniform one, which is cheaper
*and* covers more (the 2-seat arm went from 812 events to ~2,800). Final: unit
27.6 s, determinism 36.6 s, smoke 23.1 s.

**What is still open.** The berth anchor is a placeholder (R-IND3's family), and
R-O85's price ladder is untouched: rungs III and IV now buy enormous concurrency
but still cost 19 kt and 780 kt, so whether anyone can afford the axis is the
next question. Order is now **R-O85, then re-sweep** — T-88 having dropped down
the list.

#### What follows#### What follows#### What follows#### What follows

- **R-IND21 stays withdrawn.** There is no need to invent a sink.
- **R-O86 landed alongside this and moved mean infrastructure 1.027 → 1.462**
  (`Hyades_autopilot_colonization_growth.md` §6b) by unblocking the
  `outward == None` deepen fallback, which a survey pre-emption had been
  swallowing. It does **not** change R-O85's crossover or reach the ceiling —
  1.462 against 3.612, zero colonies at cap — so the price argument below stands
  exactly as measured.
- **R-O68 is closed**, and closing it did *not* recover the flat mineral-side
  results. `outpost_mining_fraction`, both crew policies (§6.15) and the Exchange
  (politics §10.6a) were measured on a bed that cannot spend minerals, and they
  still are — the cause has moved from the comparison to the price ladder, not
  gone away. Re-measuring them is blocked on R-O85, not on T-51.
- **T-76 (development freight) is the author's own answer to the same
  diagnosis** — `develop_bias` is a Doctrine knob that demands minerals for
  development, and it reaches the world through a different path than either the
  comparison or the ladder. It is next.

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

> **Amended: inter-empire settlement happens at a shared outpost**
> (`Hyades_politics_trade_and_intelligence.md` §10.6, author's ruling). Goods
> change hands at a worked rock both parties already call at — caravan trade —
> and the buyer's own freighter carries them home on the leg it was already
> flying. **A foreign hull near a colony is a card, not the default.** §8.1 is
> untouched: the mass still crosses real space and is still attackable; the
> voyage is the existing haulage leg rather than a new cross-empire one.

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
| 3 | Slips: concurrency linear in `F` (§3.2) | 2 | ~~**T-69**~~ — **done**, §6.12a |
| 4 | Infrastructure stored as kilotons; the Band is a reading (§1.3) | 1 | **T-70** |
| 5 | Extraction law: `N(S)` veins per deposit, `W = N^(1−β)·n^β`, one law for crews and works (§4.3) | — | ~~**T-71**~~ — **done**, normalised per §4.3b |
| 6 | `miners_per_outpost` becomes a target *fraction of `N(S)`*, not a hull count (§4.5) | 5 | ~~**T-72**~~ — **done**; the old knob is removed, not retained |
| 7 | Works: colour-differentiated infrastructure price (§5.1) | 4 | **T-73** |
| 8 | Extraction and fabrication rates from Infrastructure × allocation (§2) | 4 | ~~**T-74**~~ — **done**, §6.12; extraction's half superseded by §4.3a |
| 9a | `Works` struct + the CardId-ordered fold + the commutativity property test (§6.7) | 4 | **T-75a** |
| 9b | `Doctrine` allocation vector wired to the fold (§6.2) | 9a | ~~**T-75b**~~ — **done**, §6.13 |
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
| ~~**R-IND4**~~ | ~~Commutativity as a property test over the card list~~ — **resolved (T-75a/T-75b).** Two tests: the fold's algebra (`cards`) and the empire state the simulation reads (`sim`). The second is the load-bearing one — a broken `apply_card_effect` passes every test of the first. | §6, §6.13 |
| ~~R-IND5~~ | ~~May a development route target a rival's world?~~ — **resolved: balanced-value exchange needs no pact; deficits, pacts, pact-breaking and smuggling are cards.** | §7 |
| **R-IND6** | Supers and apex raising the works ceiling — deferred until the basic ramp is measured. | §8 |
| ~~R-IND7~~ | ~~The economic-thesis addition~~ — **resolved: minerals, supers and apex traverse real space**, so they can be attacked, diverted, stolen and blockaded. Nothing teleports; a trade is a voyage. | §8.1 |
| **R-IND8** | What an empire inherits when it captures developed Infrastructure. | §9 |
| **R-IND9** | The extraction tail past `N(S)` — flat, or a shallow seam at floor grade. | §4.3 |
| **R-IND19** | §4.3's `ε·S·W` double-counts the deposit — output goes as richness squared and a rich body is stripped in one tick. **Decided: the engine uses `W/N`, i.e. `(n/N)^β`,** which preserves every ratio §4.3 asserts and differs only in an absolute scale `ε` absorbs. Open only in whether the spec's own formula should be rewritten or annotated. | §4.3b |
| ~~**R-O68**~~ | ~~The deepen/expand comparison is between incommensurable quantities~~ — **resolved (T-51).** Both sides are now `rank` score per kilotonne committed: `score / outward_cost` against `w_k · min(1, headroom) / infra_cost`. `reinvest_bias` is an odds ratio with a state-dependent crossover. Bit-identical below `b = 0.96`; the old form's cliff at 0.9 moved to 1.0. | §6.18 |
| **R-O86** | ~~Both survey tests read the wrong quantity~~ — **resolved.** `candidate_count` has median **0** and max **164** against a ratified `survey_reserve` of 1024, so the reserve test is a constant `true`; and `candidates.is_empty()` pre-empted the only live deepen path. Worse, `apply_build_with` spent the minerals *before* `launch_survey` declined to spawn anything: **1,779,509 hull builds against 18,093 hulls** at the 4,000-yr horizon, i.e. 99.0% of production was mass destroyed (design law #11). Fixed with `survey_frontier`; colony count identical, colony-years +0.007%, **5.6x throughput**. | autopilot §6b |
| ~~**R-O87**~~ | ~~Tune `reinvest_bias` against work-years rather than colony-years~~ — **resolved: there is nothing to tune.** Deepening and founding buy **exactly the same works per mineral** at `eta_works = 1` (design law #11 via R-O57/T-70), so the knob is works-neutral by identity. The best screen point scored +2.33% ± 0.96 on the standard four seeds (4/4 positive) and **−1.70% ± 2.42 on four it was not chosen against**; pooled over eight, **+0.32% ± 1.42**. Held at **0.5**. `eta_works` is the lever this is not. **§6.19a corrects the reasoning**: the identity is about stock, the *flow* argument favours deepening (+29% hull/yr for 9 colonisers), and what eats it is a homeworld already at rung II, `slips` pinned at 2, and a declined build costing **29.6 yr** of yard time against 1.5 yr after a build (T-88). | §6.19, §6.19a |
| ~~**R-O88**~~ | ~~There is no build-wide axis~~ — **resolved, option C.** `fab_cap` bounds the rate **per berth** (quality); `slips` reads the fabrication share of the **stock** (quantity, unbounded). `fab_cap` 0.2 → 0.1 is a re-denomination: per-berth turnaround is **bit-identical** at every playable rung, and §3.3's schedule now reads off one constant. `slip_throughput` deleted; berth size derived from the Limited hull (**placeholder anchor**). Berths at rung II: 2 → **17**. Fleet-years **+26–34%**, throughput 88.7 → 110.7 yr/s, colony count flat. Also drops the `t_lead` defect — there is no per-planet rate left to fail to reach — and takes T-88's after-idle gap 29.6 → 7.6 yr. | §3.2, §6.3, §6.19b |
| **R-O85** | **Infrastructure is priced as if it were the scarce thing.** The step above the founding rung costs nine colonisers (0.9 kt vs 0.10 kt); `fabrication_rate` saturates by rung II so the 19-kt and 780-kt steps buy +0.017 and +0.001 kt/yr; and `slips` is pinned at **2** from rung I onward because `fab_cap / slip_throughput = 2`. So the 756-Band sink is real and priced out of reach. Every candidate fix moves an MC-tuned surface and needs ratification. | §6.18 |
| ~~**R-IND21**~~ | ~~The mineral economy has no demand side~~ — **withdrawn, and it was the wrong diagnosis.** Colonies sit at Band 1.05 against a ceiling of 3.60 with **zero** at cap and 756 Bands unbuilt: the sink is enormous and Doctrine never asks for it. The cause was read as R-O68's dead deepen branch; §6.18 refined it — the branch is cold on its merits and the ladder is what prices the sink out (R-O85). | §6.17 |
| **R-IND18** | Magnitudes for the composed extraction law — `ε`, `β`, `VEINS_PER_BAND`, and how `cap_ext`/`half_ext` land on it. The *form* is decided (§4.3a); nothing about its size is measured. | §4.3a |
| ~~**R-O74**~~ | ~~Founding settlers are conjured~~ — **resolved.** Settlers are debited from the founding centre's population and the rest of the hold is loaded from its bank; a contested coloniser unloads both halves back home. | §1.7 |
| **R-IND12** | How much a coloniser carries. **Model settled, magnitudes open.** Settlers are priced in time — what the seed saves the destination against what it costs the origin to regrow — discounted by transit; minerals are sized by the destination's intended build-out. The supply-side `endowment_fraction` is retired. | §1.7 |
| **R-IND13** | The works-value rung `I*`. Placeholder is the Band midpoint of capacity and abundance, i.e. the geometric mean of the two masses — the cheapest form with the required positive cross partial. The real function is §5's and needs T-73/T-74. | §1.7, §5 |
| ~~**R-IND15**~~ | ~~The identity works mix~~ — **resolved.** `3:2:1` Yellow : Cyan : Magenta, the §5.1 *Default* point, Yellow-primary because Production is Yellow. The `(1,1,1)` first shipped was a placeholder and contradicted §5.1's "never a true 1:1:1". | §6.10 |
| **R-IND17** | Score freight by *completion of the bill* rather than relief of one colour — `(short_before − short_after) / Σ bill`. T-81's relief term anti-concentrates and was measured counterproductive. | §6.11 |
| **R-IND16** | How much colour weight one deep Production card adds — and therefore how many layers "deep" is, given that Sole is an asymptote approached at `(3+k)/(6+k)`. | §6.10 |
| **R-IND14** | Whether the travel discount should be hyperbolic (`1/(1+n)`, current, no new constant) or exponential (needs a time constant). | §1.7 |
| ~~**R-IND10**~~ | ~~Who bears the loss when a carrier is destroyed?~~ **resolved — the register was stale, `Hyades_politics_trade_and_intelligence.md` §3.3 already answered it.** Escrow returns to the buyer minus the burn: the buyer loses the burn, the seller loses the cargo, the loss is shared. That is what makes escorting worth paying for. | §8.1, politics §3.3 |
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
