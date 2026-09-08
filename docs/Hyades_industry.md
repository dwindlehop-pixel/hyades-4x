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

`Factors::k()` is `k_potential().min(infra)` and `k_potential()` is
`hab.min(bio_max_band)`, so the change is **deleting one `.min()`** — after which
`k()` and `k_potential()` are the same function. Three real consequences, none of
them cosmetic:

- **`founding_infra` (R-O76) stops capping the founding seed.** A coloniser's
  recycled hull currently sets the new colony's `K`, so a Medium founds at
  `Band I` and a General at `Band II`. With Infrastructure out of `K`, founding
  capacity is `k_potential` alone and both hulls seed to the world's own ceiling.
  The recycled hull still lands as **industrial stock**, which is now its whole
  job, and R-O76's measured result — that seed depth does not pay — must be
  **re-measured**, because the thing it measured no longer exists.
- **The deepen/expand trade changes meaning.** `deepen_headroom` is
  `k_potential − infra` (R-O68), which under the amendment is not a headroom at
  all. Deepening no longer raises a ceiling; it raises **rates**. The branch was
  already provably dead at the shipped `reinvest_bias` (R-O68), so this is a
  chance to rebuild it rather than repair it.
- **`medium_min_level` and the production gates are untouched** — they read
  population level, not infrastructure.

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

### 3.3 Placeholder magnitudes

**Not measured.** Chosen to land near current behaviour so the first engine
change reads against a known bed.

| Constant | Placeholder | Meaning |
|---|---|---|
| `t_lead` | **2.0 yr** | irreducible per-hull lead time — tooling and crew, the part that does not scale |
| `F_slip` | **0.1 kt/yr** | one slip's throughput; set so a General hull lands near today's 10 yr |

| Hull | dry mass | `t_build` at one slip | today |
|---|---|---|---|
| Limited Systems | 0.02 kt | **2.2 yr** | 10 yr |
| Medium Systems | 0.10 kt | **3.0 yr** | 10 yr |
| General Systems | 1.00 kt | **12.0 yr** | 10 yr |

**This will move the bed.** Scouts and colonisers get dramatically cheaper in
time while General hulls get slightly dearer — a direct accelerant on the
expansion loop whose time constant T-51/R-O68 identified as the binding limiter.
Guard is `examples/colony_years`; the prediction is *up*, and if it is not, find
the mechanism before tuning the value (`CLAUDE.md` §2).

---

## 4. The mining ramp — crowding, and the grade curve

### 4.1 Mining is sublinear in capacity, and the reason is physical

**Two miners cannot work the same vein.** Capacity beyond the first must open
another one, and the best veins are opened first — so the marginal miner is
always working ore poorer than the last. Aggregate output is therefore **not**
the sum of individual outputs.

This is Lanchester's intuition with the sign reversed. Lanchester's square law
makes concentrated force *superlinear* (`N²`) because fire concentrates;
extraction is *sublinear* because sites are **exclusive**. Same lesson — an
aggregate is not `N ×` an individual — opposite direction, different cause.

**The grade curve is the mechanism, and it is empirical.** Lasky's law: in a
mineralised body, cumulative tonnage rises roughly exponentially as average grade
falls arithmetically — there is always more poor ore than rich ore, in a regular
relationship (Lasky 1950). Take the `k`-th best vein to have grade
`g(k) = g₀ · k^(−α)`. Then `n` units of capacity extract

```
Σ_{k=1..n} g₀·k^(−α)  ≈  g₀ · n^(1−α)/(1−α)        for α < 1
```

so output scales as `n^β` with **`β = 1 − α`**, the *crowding exponent*.

**Placeholder `β = 1/2`** — the square-root law, the exact mirror of Lanchester's
square. It reads plainly at the table: **doubling your miners gives you ~1.41×
the ore, not 2×.** It is a placeholder and needs Monte-Carlo ratification like
everything else here.

### 4.2 One law, two sources of capacity

The law applies to **all** extraction capacity at a site, whatever supplies it:

```
extraction(kt/yr) = ε · S · (n_crew + u_infra)^β
```

- `S` — remaining ore in the ground, kt. Output scales with the stock, so a body
  depletes asymptotically rather than cliff-edging.
- `n_crew` — miner hulls on station (the outpost route).
- `u_infra` — units of Infrastructure allocated to extraction (the colony route).
- `ε` — the per-unit rate constant.

**Outposts and colonies mine by the same physics and differ only in how they buy
capacity.** Keeping one law for both is deliberate: an asymmetry here — outposts
flat, colonies improvable — would make "outpost or colony?" a question about
*rate shape*, when it should be a question about **commitment**. Both routes
improve; they differ in what the improvement costs you and what it exposes.

| | Outpost | Colony |
|---|---|---|
| Buys capacity with | miner hulls | Infrastructure |
| Cost to establish | a Limited hull and a crew | a coloniser, and a habitable world |
| Improvable? | yes — add hulls | yes — add works |
| Vulnerable to | losing hulls | losing Infrastructure |
| Good for | a rich rock you do not intend to hold | a world you are building on |

### 4.3 What this changes that is already ratified

**T-57 ratified `miners_per_outpost = 3` under a *linear* law** — extraction was
`crew × outpost_mining_fraction`, measured at **+2.74% colony-years** with every
seed positive, and five miners were rejected for costing 3.5 years of doubling
time. Under `β = 1/2` the third miner is worth `√3 − √2 = 0.32` of the first
rather than a full unit, so **the ratified crew size is a number measured against
a law that no longer holds and must be re-ratified.** Expect the optimum to fall.

**It also gives T-66 its first real lever.** T-66 records that hauling is the
engine's largest single cost, because a `Band IV` body is effectively
inexhaustible and the hauling loop has no notion of demand. Sublinear extraction
attacks that directly: piling capacity onto one fabulous rock stops paying, so
the fleet spreads instead of queueing, and the marginal freighter trip stops
being worth taking.

---

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

**A cost-modifying card rotates the colour mix. It never lowers the total.**

Efficiency — more rate per kilotonne — is a *Design* write on a named coefficient
(§6). Mix is a *rotation* that preserves magnitude. Keeping the two separate is
what stops the card list becoming a discount race, and it is what makes "shift
the mix away from pure Yellow" a real strategic move rather than a worse version
of "make it cheaper".

---

## 6. The layering algebra — how Design and Doctrine writes compose

**Three write kinds, three algebras, one acceptance test.**

| Write kind | Written by | Algebra | Bounded by |
|---|---|---|---|
| **Design — efficiency** | permanent tree cards | **product** of factors on a named coefficient | nothing intrinsic; priced per tier |
| **Doctrine — allocation** | revisable tree cards | **simplex** — shares that renormalise | sums to 1 by construction |
| **Works — mix** | either | **rotation** of the colour vector | total kt preserved (§5.4) |

Each algebra is chosen for a property, not for convenience:

- **Products commute.** Two Design cards in either order give the same
  coefficient. Order-dependence in a permanent, tier-gated tree is unanalysable
  at Monte-Carlo scale and unlearnable at the table.
- **Simplices are bounded.** No combination of Doctrine cards exceeds the stock.
  Renormalisation *is* the bound — there is no clamp to tune and none to hide
  behind, which is precisely the failure T-64 records for the population
  logistic, where a clamp turned a broken model into a high-scoring one.
- **Rotations preserve magnitude.** No card is a discount; the mix moves, the
  bill does not.

**The acceptance test is commutativity, and it belongs in the engine rather than
in this document:** for any set of cards and any two orderings, the resulting
`(coefficients, allocation, mix)` must be identical. **R-IND4:** write it as a
property test over the tier-0 card list *before* the second industrial card
exists, not after.

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

> **R-IND7 — an addition to the thesis, recorded incomplete.** *"An addition to
> the economic thesis is much of the minerals, supers, and apex…"* — the note
> this document was given breaks off there, and the rest is not guessable without
> inventing design. The likely shape, from context, is a claim about how much of
> an empire's refined mass ends up **locked in standing fleets and works rather
> than available to spend** — which would sharpen §8 considerably, because it
> makes the shortage structural rather than geographic. **Not written up until
> the sentence is finished.**

Supers and apex above all of this are **R-IND6**, deliberately deferred: the
mid- and late-game boosts that raise the works ceiling should not be designed
until the basic ramp is measured, because their whole job is to bend a curve that
does not exist yet.

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
| 5 | Sublinear extraction, `n^β`, one law for crews and works (§4.2) | — | **T-71** |
| 6 | Re-ratify `miners_per_outpost` under the crowding exponent (§4.3) | 5 | **T-72** |
| 7 | Works: colour-differentiated infrastructure price (§5.1) | 4 | **T-73** |
| 8 | Extraction and fabrication rates from Infrastructure × allocation (§2) | 4 | **T-74** |
| 9 | `Doctrine` allocation vector + the commutativity property test (§6) | 8 | **T-75** |
| 10 | Development freight and the balanced-exchange default (§7) | 7 | **T-76** |

**Item 1 first, and alone.** It is one deleted `.min()`, it unblocks every card
that attacks Infrastructure, and it invalidates R-O76's measured result — so it
wants its own measurement rather than being folded into a larger change.

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
| **R-IND7** | The economic-thesis addition, recorded incomplete. | §8 |
| **R-IND8** | What an empire inherits when it captures developed Infrastructure. | §9 |

---

## References

- Traulsen, A. & Nowak, M. A. (2006). Evolution of cooperation by multilevel
  selection. *PNAS* 103(29):10952–10955. — the narrative thesis §1.2 leans on.
- Lasky, S. G. (1950). How tonnage and grade relations help predict ore reserves.
  *Engineering and Mining Journal* 151(4):81–85. — the grade-tonnage relation
  §4.1 derives the crowding exponent from.
- Lanchester, F. W. (1916). *Aircraft in Warfare: The Dawn of the Fourth Arm.* —
  the square law §4.1 mirrors.
- May, R. M. (1976). Simple mathematical models with very complicated dynamics.
  *Nature* 261:459–467. — the discrete logistic above `K`, §1.1.
- *Stars!* (1995), Mare Crisium. — the three-installation model §1.3 departs
  from, and the `-f` archetype that is the reason why.
