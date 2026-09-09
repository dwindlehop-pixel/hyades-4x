# Hyades — The Trees: Tone, Objectives, and Card Value

*The normative spec for **what the six trees are about, what each one is trying to
maximise, and how a card's power is measured and compared.** Companion to
`Hyades_galaxy_and_autopilot.md` §7 (the sagas and the colour spine, which §1
here **amends**), `Hyades_card_contract.md` (the card data model),
`Hyades_standing_layer_and_observation.md` §5 (Doctrine and Design as state),
`Hyades_industry.md` §5 (works) and §8.1 (refined mass traverses real space), and
`Hyades_politics_trade_and_intelligence.md` (the Exchange). Calls flagged
**R-TREE\<n\>**.*

*Rev 1. **Nothing here is Monte-Carlo ratified.** §2 settles what each tree is
measured on and §4 settles how a card's value is defined; every magnitude,
threshold and coefficient is a **placeholder** and says so. What is being agreed
is shape, because that is the part a later measurement cannot fix.*

---

## 0. What this settles

Three things, and the first is a correction rather than an addition.

1. **The tone.** Every winning game is a story about how love wins — told by the
   winner, about themselves, and the player can see what actually happened. It is
   **satire**, and §1 says what the satirical target is and what that forbids.
2. **Six objectives, not one.** Every measurement in this project so far has run
   against a single global objective (colony count, with colony-years as the
   guard). That cannot balance six trees, because five of them are not trying to
   do that. §2 defines one objective per tree, **per player**.
3. **What a card is worth.** Not an average. Card value is a *distribution* over
   galaxies, opponents and moments, and the design target is its **92nd
   percentile**, with the median and 98th as secondary reads. §4 states the
   contract and the tier constraints.

---

## 1. The story is satire, and the empire is the unreliable narrator

### 1.1 The frame

**Every winning game in Hyades is Paul Atreides' jihad, or The Mule's empire.** A
charismatic power sweeps a known universe, and the civilisation that did it
tells itself — sincerely, at length, in its own official voice — that this was
love: that it reached, nurtured, provided, bound, understood, and parted only
with what could not love. The mechanics record conquest, extraction and
progressive elimination. Both are true statements about the same game.

**The load-bearing comparison is Helldivers.** Super Earth's satire works because
Super Earth is *never winking*: the Eagle Sweat commercial is a real commercial,
Managed Democracy is described with pride, and the player supplies the irony. The
game does not tell you the joke; it performs the thing straight and trusts you.

So: **love-winning is in-game mythmaking by the player's own empire**, and the
player registers the gap.

### 1.2 The gap is the design, and it must be legible

A satire the player cannot see is just propaganda. The gap only exists if both
halves reach them:

- **The empire's voice is sincere, warm, and everywhere it belongs** — card
  names, flavour, tree sagas, victory text, UI chrome. It believes itself.
- **The simulation's readouts are plain and unvarnished, and are never laundered
  through that voice.** A biosphere reduced from `Band III` to `Band Empty` reads
  as a biosphere reduced from `Band III` to `Band Empty`. Numbers do not get
  euphemisms.

**The chronicle is the mechanism** (R-TREE1). At the end of a game the winner's
empire writes its own history, generated from what actually happened, in its own
register — and the euphemism is *derived from* the atrocity rather than replacing
it. An orbital bombardment that removed eight Bands of living mass is filed as an
**Ecological Simplification Initiative**, with the expected-value arithmetic that
justified it, next to the number. The player did the thing, and reads what their
civilisation says it did.

That is the whole joke, and it is the only place the design is allowed to make
it. Everywhere else, straight.

### 1.3 The satirical target

**Tech founders planning a multiplanetary civilisation, and the use of effective
altruism to justify atrocity.** Specifically the register in which:

- expansion is framed as a moral obligation to unborn trillions, so that anything
  standing on the land is an *obstacle to the light cone*;
- harm is netted out in expectation, at a scale where the arithmetic stops being
  checkable and starts being a licence;
- the language of care — flourishing, stewardship, the long term, the garden — is
  the working vocabulary of an extraction operation;
- and the people running it are **sincere**, which is what makes it worth
  satirising rather than merely condemning.

The target is an ideology and its powerful adherents. It is not the players, and
it is not any real person by name.

### 1.4 Register rules for card and flavour text

Concrete, so this is checkable rather than a vibe:

1. **Never wink.** No card may signal that it knows it is in a satire. The
   moment one does, the player stops supplying the irony and the gap closes.
2. **Bureaucratic and founder registers, both sincere.** Programme names, launch
   language, mission statements, initiative titles. Warm hard SF, never imperial
   or naval — `Hyades_galaxy_and_autopilot.md` §7's rule survives intact and is
   in fact what makes the satire land.
3. **Expected-value language is the empire's native idiom** and is used most
   heavily where the act is worst. Bathos is the tool: the same careful moral
   arithmetic applied to a shipping schedule and to a sterilised world.
4. **The `role` subtitle stays.** Every card names its mechanic plainly, so the
   fiction cannot drift free of what the card does — the gap is between the
   empire's *framing* and the outcome, never between the card's text and its
   rules.
5. **Flavour text is the author's own** (`CLAUDE.md` §6). This section constrains
   register; it does not licence anyone to rewrite the author's lines.

### 1.5 What this amends

`Hyades_galaxy_and_autopilot.md` §7 says the game "carries hope (a young people
*setting out*)", and leaves Warfare's Beloved Republic deliberately open —
*"whether the belief is true or is Unforgiven's self-told lie is the story each
unique game decides."*

**That ambiguity is now settled, and in one direction: it is the self-told lie,
and the game is on the player's side of knowing it.** The hope is real *as the
empire's sincere belief about itself*; the satire is the distance between that
belief and the board. §7's six sagas, six modes of love, and the Western register
for The Hard Mercy all stand unchanged — they are the empire's voice, which is
exactly what this section says they should be. What changes is that they are no
longer the game's own claim about what happened.

### 1.6 What this forbids

- **Mechanically rewarding cruelty while narratively condemning it.** That is the
  ordinary grimdark move and it is not this. Here cruelty is narratively
  *celebrated*, as altruism, by the people who did it.
- **A neutral or omniscient narrator anywhere in the shipped fiction.** There is
  no voice above the empires telling the player what to think. The simulation's
  numbers are the only unaligned witness, which is why §1.2 forbids dressing
  them.
- **Winking, in any card, ever.** Stated twice on purpose.

---

## 2. Six objectives, one per tree

### 2.1 Why one global objective was wrong

Every ratification in this project has been measured against **absolute colony
count at a fixed horizon**, with colony-years as the guard. That is the right
objective for exactly one tree — Expansion — and it is actively misleading for
the other five. A Warfare card that ends a neighbour's colony *lowers* the global
count. A Production card that converts minerals into hulls does not move it at
all. Measured on the shipped objective, half the design scores zero or negative.

Worse, it is a **single-player** objective in a table game. Ending a rival's
expansion is worth as much as growing your own, and no aggregate over all seats
can express that.

So: **one objective per tree, evaluated per player.**

### 2.2 Terms

Defined before use (`CLAUDE.md` §6), because §3 and §4 both index on them.

| symbol | name | unit | where it comes from |
|---|---|---|---|
| `i`, `j` | player (seat) indices | — | `PlayerId` |
| `T` | measurement horizon | yr | §3.1, currently 8,000 |
| `C_i(t)` | colonies owned by `i` at time `t` | count | `SimReport::players[i].colonies` |
| `V_i(t)` | works owned by `i` | kt of works | `Hyades_industry.md` §5 — **not yet built** |
| `F_i(t)` | fleet dry mass owned by `i` | kt | `hull_dry_mass` summed over owned hulls |
| `Q_i(t)` | capability of `i` | dimensionless | §2.3.5, **proposed here** |
| `w_ij` | Warfare's neighbour weight of `j` from `i`'s view | dimensionless, `Σ_j w_ij = 1` | §2.3.2, fixed at game start |
| `φ_ij(t)` | share of `j`'s output accruing to `i` | dimensionless, `Σ_i φ_ij ≤ 1` | §2.3.6, from delivered freight |
| `κ` | Politics coupling constant | dimensionless, `0 < κ < 1` | §2.3.6, placeholder |
| `X_i` | a tree's stock for player `i` | tree-specific | one of `C`, `V`, `F`, `Q` |
| `g` | exponential growth rate of a stock | 1/yr | §2.4 |
| `t₂` | doubling time of a stock, `ln 2 / g` | yr | §2.4 — **the numeraire** |

**Every objective is an integral of a stock over the horizon** — an
"`X`-years" — which is the form colony-years already has and the reason it is
the guard the project trusts (`CLAUDE.md` §7: count at a horizon is a weak
invariant; the integral falls the moment anything slows down).

### 2.3 The six

#### 2.3.1 Expansion — colony-years, absolute

```text
E_i = ∫₀^T C_i(t) dt
```

The shipped objective, unchanged, now correctly scoped to one tree.

#### 2.3.2 Warfare — colony-years relative to the table, neighbour-weighted

```text
W_i = ∫₀^T [ C_i(t) − Σ_{j≠i} w_ij · C_j(t) ] dt
```

**Ending a neighbour's colony is worth exactly what founding your own is**, and
slowing or reversing a neighbour's expansion scores continuously rather than only
on kills. `w_ij` weights near neighbours heavily — a rival across the theatre is
someone else's problem.

Proposed form, reusing the hyperbolic shape R-IND12 already uses and a quantity
the engine already computes:

```text
w_ij ∝ 1 / (1 + d_ij / λ_w)        normalised so Σ_{j≠i} w_ij = 1
```

with `d_ij` the distance between the two empires' holdings centroids and `λ_w` a
placeholder length scale (**R-TREE2**).

**`d_ij` is measured once, at game start, from homeworld positions — and this is
an anti-farm rule, not an optimisation.** A live `d_ij` would be a term the
Warfare player can move without fighting: expand *away* from a strong rival and
their weight falls, scoring the same as having beaten them. `CLAUDE.md` §2's
invariance rule in its exact form — *what could a card do to move this without
moving the world?* Freezing `w_ij` at setup answers "nothing".

> **Warfare's objective is relative, and that is not a violation of the
> invariance rule.** The rule forbids a metric the optimised thing can move
> *without moving the world*. Destroying a rival's colony moves the world. The
> distinction is the whole reason `w_ij` is frozen and `C_j` is not.

#### 2.3.3 Growth — work-years

```text
G_i = ∫₀^T V_i(t) dt
```

**Blocked on the works pipeline** (`Hyades_industry.md` §5, T-73/T-74): works do
not exist in the engine yet. Until they do, the honest interim stock is
**infrastructure in kilotons** once T-70 lands — infrastructure is the industrial
stock works are bought with, so it is a leading indicator of the real quantity
rather than a different one. Flagged so the substitution is not forgotten
(**R-TREE3**).

#### 2.3.4 Production — fleet-years, in mass

```text
P_i = ∫₀^T F_i(t) dt
```

**Mass, never hull count.** Counting hulls rewards fragmentation and would put
this objective in direct contradiction with design law #3, which says
consolidation wins under geometry alone. Since R-O57 dry mass *is* mineral cost,
so fleet-years is also "minerals committed to hulls, integrated" — one quantity,
two readings, no second ladder.

Measurable today.

#### 2.3.5 Technology — capability-years

```text
T_i = ∫₀^T Q_i(t) dt
```

`Q_i` is the metric the author left open. **Proposed definition, and the
reasoning matters more than the constants:**

Capability is *not* fleet mass — that is Production. It is what a fleet **can
do**, so it is a per-mass effectiveness times the mass, and Technology raises the
first factor. It is naturally a **vector** over the axes the author named:

| axis | meaning | measurable as |
|---|---|---|
| projection | deliverable combat mass at range | combat mass × reach under `a_max` within a response window |
| defence | combat mass within response time of owned colonies | same, evaluated against own holdings |
| acquisition | ore delivered per year | freighter deliveries — **already logged** |

**Aggregate them with a power mean, not a sum and not a hard minimum**
(**R-TREE4**):

```text
Q = ( Σ_a  s_a · (q_a / q_ref,a)^ρ  ) ^ (1/ρ)
```

- A **sum** (`ρ = 1`) lets Technology farm whichever axis is cheapest and ignore
  the rest.
- A **hard Liebig minimum** (`ρ → −∞`) matches the economic thesis — the best war
  machine needs all three basics and all three supers, which is a minimum — and
  forces the cross-tree engagement design law #7 requires.
- But a hard minimum makes a Technology card's value depend entirely on whether
  it happens to raise the *currently weakest* axis, which is enormous variance.
  §4.3 requires tier-1 cards to have the **lowest** dispersion of any tier, and a
  Liebig capability may make that impossible for Technology specifically.

So `ρ` is a **measured knob**, and "how Liebig is capability" becomes one number
to ratify rather than a binary to argue about. Start near `ρ = −1` (harmonic
mean: strongly penalises a weak axis without zeroing the card that missed it).
Weights `s_a`, references `q_ref,a` and `ρ` are all placeholders.

#### 2.3.6 Politics — own colony-years plus an earned share of others'

```text
Pol_i = ∫₀^T [ C_i(t) + κ · Σ_{j≠i} φ_ij(t) · C_j(t) ] dt
```

*If a Politics player's allies and trading partners are growing, that improves
the Politics player's position.*

**`φ_ij` must be earned, not declared** — this is the objective most obviously
farmable, and the naive form fails immediately: if `φ` keys on "has a pact", a
Politics player signs with everyone and scores the whole table for free.

The resolution comes from a rule that already exists and is not about Politics at
all. **`Hyades_industry.md` §8.1: refined mass traverses real space.** So define
`φ_ij(t)` as **the share of `j`'s exported output that actually reached `i`** —
freight delivered, on hulls, under light-lag. Three properties fall out and none
of them had to be legislated:

- **`Σ_i φ_ij ≤ 1` automatically.** `j` cannot ship more than it produces, so the
  metric is self-limiting without an arbitrary cap.
- **It is rivalrous.** Two Politics players courting the same partner compete for
  the same finite export stream.
- **A blockaded pact scores nothing**, because nothing arrived. Piracy, theft and
  interdiction bite on the Politics objective directly rather than by analogy.

`κ` bounds how much of a partner's success is *yours* and is a placeholder
(**R-TREE5**). The author's note that Politics' objective may need Monte-Carlo
clarification stands: this is a concrete proposal to measure, not a settled
answer.

### 2.4 Commensurability — the doubling time is the numeraire

**The six objectives are in six different units, and §4 requires tier-1 cards to
be worth "about the same" across trees.** Colony-years cannot be compared to
kilotons of fleet-years. Something has to make them commensurable, and the author
named it: the **gradient**, *e.g. years to double*.

That is the right answer, and it is worth stating why it works. While a stock is
compounding, `X(t) ≈ X₀·e^{g·t}`, and its doubling time is `t₂ = ln 2 / g` — **in
years, for every tree.** So:

> **Card value is the fractional reduction in the doubling time of its tree's own
> stock.** Dimensionless, comparable across all six trees, and independent of the
> units of the stock it acts on.

Two consequences follow immediately and both are load-bearing:

- **A doubling time only exists while the stock compounds.** Once a stock
  saturates — and colony count now saturates by ~1,500 yr (`examples/horizon_cost`)
  — `g → 0`, `t₂ → ∞`, and the measure stops meaning anything. So the value of a
  card must be read in the **exponential regime**, not at the horizon.
- **Which is the same requirement as "balance at earliest possible play"**
  (§4.4). The author's two rules are not two rules: measuring on a gradient
  *forces* early evaluation, because that is the only window where a gradient is
  defined.

**Estimate `g` by regression on `ln X_i(t)` over the compounding window, not from
two endpoints.** Endpoint estimates are dominated by whichever end is noisier and
they hide saturation instead of revealing it; a fit exposes curvature, which is
the signal that the window has been chosen wrong.

### 2.5 The invariance audit

`CLAUDE.md`'s rule — *what could a card do to move this metric without moving the
world?* — applied to all six before any of them is used:

| objective | farmable? | the answer |
|---|---|---|
| Expansion | no | absolute count of a thing that must be founded |
| Warfare | **only if `w_ij` is live** | frozen at setup (§2.3.2) |
| Growth | no, once works exist | works must be bought with minerals |
| Production | **yes, if counted in hulls** | counted in mass (§2.3.4) |
| Technology | **yes, if aggregated by sum** | power mean with `ρ < 1` (§2.3.5) |
| Politics | **yes, if `φ` keys on pacts** | `φ` is delivered freight (§2.3.6) |

Four of the six had a live farm in their obvious formulation. That ratio is the
argument for doing this audit as part of defining a metric rather than after
something scores suspiciously well.

---

## 3. The measurement bed

### 3.1 The 8-kyr requirement, and what it costs

The author's instruction is to measure all six on an **8,000-year** bed. The
requirement is sound — colony count saturates early, but fleet mass, works and
capability plausibly do not, and a horizon that truncates five of six stocks
mid-compounding would bias every comparison toward Expansion.

**It is also, at today's throughput, the binding constraint on the entire card
programme, and that has to be said plainly rather than discovered later.**
Measured post-T-68 (`examples/horizon_cost`, seed 1, 3 seats): 4,000 yr costs
~400 s. `CLAUDE.md` §7 records that 8 kyr cost **5.8×** the 4 kyr run at an
earlier operating point, so one 8-kyr seed is on the order of **35–40 minutes**.

A single card's value distribution needs enough samples for a stable **92nd
percentile** — realistically dozens, not a handful. Six trees × the card set ×
tiers × timings × that sample count puts the full programme in the range of
**years of single-machine compute**. It is not a scheduling problem to be
absorbed; it is a blocker.

**So T-66 (throughput) is a prerequisite for this programme, not a parallel
nicety** — and it now has a second, larger reason to happen than the T-24 floor.
Three things soften the cost and none of them removes it:

- **Common random numbers**, already the project's practice: the same galaxies
  for every card, compared card-by-card within seed. This is free and it is worth
  more here than anywhere else, because the quantity being estimated is a *tail*.
- **Screen on a truncated horizon, ratify on the objective** (`CLAUDE.md` §2),
  with the screen **re-calibrated** — the ρ figures in that file predate three
  landings and must not be assumed to carry.
- **Measure where each stock actually saturates first** (§3.2). If fleet-years
  saturates at 3,000 yr, the 8-kyr bed is only needed for the stocks that do not.

### 3.2 Per-metric saturation is the first measurement, and it is cheap

Before any card is measured: **one 8-kyr run per seed, instrumented for all six
stocks, plotting each against time.** That single run answers, for each tree:

- where its stock leaves the exponential regime (which sets the window §2.4's
  regression is fitted over);
- where it saturates (which sets the horizon that tree actually needs);
- and whether it saturates at all inside 8 kyr.

This is a handful of runs to potentially cut the whole programme's horizon for
four or five of the six trees. It is the highest-leverage measurement available
and it should be done first (**T-78**).

### 3.3 Instrumentation

All six stocks are **per player, sampled on a fixed cadence**, so the integrals
and the log-regressions are computed from one time series per seat per tree.
Sampling on a fixed grid rather than on events is deliberate: event-driven
sampling would weight the series by activity, which is exactly what the metric is
trying to measure.

Three of the six are measurable today (`C`, `F`, and the freight flows `φ` needs);
`V` waits on works (T-73/T-74) and `Q` on the capability definition (R-TREE4).

---

## 4. Card value

### 4.1 Value is a distribution, and the design does not target its mean

A card's value depends on the galaxy, the opponents, the board state, and above
all *when* it is played. It is a random variable, and:

> **Card value = the fractional reduction in the doubling time of its tree's
> stock, for the player who played it, against a counterfactual run in which it
> was not played** — same seed, same everything else (CRN).

### 4.2 The percentile contract

**Design to the 92nd percentile.** The median and the 98th are secondary reads.

The reasoning, stated because it is a real design position rather than a
convention: a card is chosen by a player who thinks the moment is right, so the
realised distribution is not the unconditional one — it is *conditioned on
someone choosing to play it*. Balancing on the mean balances a card as though it
were played at random, which no card ever is. P92 asks *"how good is this when it
is working?"*, which is the case that decides whether a card warps the game. P98
is the blow-out check; the median is the floor check.

### 4.3 Tier equality, and the dispersion constraint

Two requirements, and they are not the same requirement:

1. **Tier-1 cards have similar P92 to each other**, across all six trees — made
   possible by §2.4's common numeraire.
2. **Tier-1 cards have lower dispersion than any other tier.** Higher tiers may
   and should swing more.

**Use a robust dispersion measure, not the variance** (**R-TREE6**). The value
distribution is skewed by construction — that is why P92 rather than the mean —
and variance on a skewed distribution is dominated by the same tail the design is
already reading separately. **Median absolute deviation** is the robust analogue
and is what "median variance" should be operationalised as; the P92−median gap is
a second, directional read of the same thing.

Both requirements are evaluated **at earliest legal play** (§4.4). Dispersion is
expected to be an increasing function of how late a card is played — the board
has more state to interact with — so a dispersion bound stated without a timing
is not a bound at all.

### 4.4 Balance at earliest legal play

**The measurement point is the earliest turn on which a card can legally be
played.** Three reasons, and the third is the one that makes it non-negotiable:

- It is the moment the card's effect has the longest time to compound, so it is
  where a mispriced card does the most damage.
- It is the least state-dependent moment available, which is what makes the
  tier-1 dispersion bound achievable at all.
- **It is the only moment where the gradient exists** (§2.4). Play a card late
  enough and its tree's stock has saturated, `g → 0`, and the measure is
  undefined. Balancing at earliest play is not a simplifying choice; it is forced
  by the numeraire.

**Timing swing is explicitly not a balance problem.** A card played at the right
or wrong moment should swing hard — that is the meso layer the design is built
on. What is being controlled is the value of the *correct* play, not the spread
between a good play and a bad one.

### 4.5 What this method cannot do

Stated so it is not discovered as a surprise:

- **A tail quantile is expensive.** P92 has far higher sampling variance than a
  mean at the same sample count. Every tier-1 equality claim needs an error bar,
  and `CLAUDE.md` §2's rule — *anything inside 2 SE of zero is not a finding* —
  applies to differences between cards exactly as it does to knobs.
- **Counterfactual value is not additive.** Two cards each worth +5% at P92 are
  not worth +10% together; the project has already measured this once, where
  `growth_rate` and mining-pair recycling combined to +2.53 rather than +3.50
  because they competed for headroom that was not economic. Combo cards
  (design law #7) are precisely where this bites, and they must be measured
  *jointly*, never summed.
- **It says nothing about whether a card is interesting.** It is a power
  measurement. Legibility (design law #9), flavour, and whether a card places a
  behaviour-rich object on the board are design judgements this method does not
  make.

---

## 5. R-code register

| Code | Question | Where |
|---|---|---|
| **R-TREE1** | The end-of-game chronicle: generated from real events, in the empire's register, with the euphemism derived from the act. Scope and surface. | §1.2 |
| **R-TREE2** | `λ_w`, Warfare's neighbour length scale — and whether centroid distance at setup is the right proximity measure. | §2.3.2 |
| **R-TREE3** | Growth's interim stock until works exist. Infrastructure in kilotons is proposed; confirm or replace when T-73/T-74 land. | §2.3.3 |
| **R-TREE4** | Capability: the axis set, the reference scales `q_ref,a`, the weights `s_a`, and above all `ρ` — how Liebig capability is. | §2.3.5 |
| **R-TREE5** | Politics' coupling `κ`, and whether delivered freight is the whole of `φ_ij` or only its economic half (shared intelligence is the other candidate). | §2.3.6 |
| **R-TREE6** | Dispersion measure for the tier-1 constraint. MAD proposed over variance; and the numeric bound for "similar P92". | §4.3 |
| **R-TREE7** | Whether all six trees need the 8-kyr horizon, or only those whose stock has not saturated. Answered by T-78. | §3.2 |

---

## References

- Herbert, F. (1965). *Dune.* — the winning game's shape: a jihad the winner
  experiences as deliverance.
- Asimov, I. (1952). *Foundation and Empire.* — The Mule: empire forged from the
  ruins of a fall, by someone the galaxy loves.
- Forster, E. M. (1939). *What I Believe.* — "Only Love the Beloved Republic
  deserves that", which `Hyades_galaxy_and_autopilot.md` §7 takes Warfare's
  win-state from, and which §1.5 here re-reads as the empire's own claim.
- Arrowhead Game Studios (2015, 2024). *Helldivers*, *Helldivers 2.* — the
  never-winking satirical register §1.4 rule 1 is taken from.
