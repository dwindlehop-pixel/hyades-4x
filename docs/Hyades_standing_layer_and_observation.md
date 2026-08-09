# Hyades — The Standing Layer, Observation, and the Tier-0 Card Model

**Rev 1.** Consolidates every decision ratified in the round-1 decision-space
thread. Supersedes `Hyades_opening_decision_space.md` rev 1 and rev 2 in full,
and amends `Hyades_command_cards.md` (cmd §), `Hyades_galaxy_and_autopilot.md`
(galaxy §), `Hyades_card_contract.md` (contract §), `Hyades_vehicle_roles.md`
(roles §), `Hyades_mineral_cost_curve.md` (cost §), and
`Hyades_simulation_model.md` (sim §) where noted. New calls continue the
**R-O n** series. Cost magnitudes remain placeholders (R-7/R-9).

---

## 0. The six structural changes

1. **Every card is a tree card.** The First Principles class (The Compass, The
   Pattern) and the Common Acts class (The Open Hand, Open Skies, The Aegis)
   are deleted. Doctrine and Design become *state written by tree cards*, not
   cards themselves.
2. **Tier 0 is ungated.** All six mouths are open from game start; their cards
   cost 1 action and carry no reach surcharge. The `base + 1` unlock applies to
   descending *below* a mouth. Tier-1 reaches are legal in round 1.
3. **Three slants per tree at tier 0** — inscrutable, balanced, less guarded —
   carried numerically by the cost-ratio spread.
4. **σ (slant) is committed production**, resolving into three separable
   components: what it costs (σ_commit), how far it moves your fleet's
   **acceleration signature** (σ_kinetic, the long-range observable), and how
   far it rotates the standing layer (σ_vector, close-range).
5. **The counter-graph is a ladder by default and is disrupted by cards.** It
   is per-player state with a total-order initial condition, not a fixed
   global object.
6. **Mass is conserved, and cost and dry mass are one number.** A hull is a
   shell: dry mass scales with surface area, contents with volume. Exotic
   technology changes the *sign* of mass, not the total.

---

## 1. Action economy and the round-1 space

Three actions per round, no free action (cmd §5, carried). Costs:

| Act | Actions | Note |
|---|---|---|
| Play a tier-0 card | 1 | no gate, no surcharge |
| Reach a tier-1 node (first card from it) | 2 | `base + 1`; the +1 is **per node, not per card** |
| Play a further card from an unlocked node | 1 | |

Two mouths' worth of breadth is therefore free at tier 0, and the `+1` no
longer taxes breadth — which matters because cmd §4's balance law
(*a winning strategy plays from more trees than it descends deeply*) was
previously priced against itself.

**Round-1 legal openings**, with 6 trees × 3 tier-0 cards, the colour filter
leaving 4 affordable trees (n = 12 affordable tier-0 cards), branching 2 and
≈3 cards per tier-1 node:

| Shape | Count |
|---|---|
| three tier-0 cards | `C(12,3)` = 220 |
| reach + one outside tier-0 card | `8 × 3 × 12` = 288 |
| reach + a second card from the same node (2 + 1 actions) | `8 × C(3,2)` = 24 |
| **total** | **532** |

**The mineral budget is now the sole throttle on breadth.** Actions no longer
restrain how many trees you touch per round, so R-7/R-9 carries considerably
more weight than it did.

*Supersedes:* galaxy §8's derived 11-card round-1 list, which becomes 18
tier-0 cards (3 slants × 6 trees) plus tier-1 reaches, and nothing else.
Galaxy §6's autopilot table lists The Compass as the round-1 override for
Explore/Expand/Build and The Pattern for Exterminate/defend; all four rows now
take tree-card overrides.

---

## 2. The slant triad

Each tree carries three tier-0 cards along one axis. **Legibility is not a
separate dial — it is σ read from the other side of the table**, because a
rational observer's inference from a play is exactly "how much would this only
be worth playing if you meant it."

| Stop | σ | Cost ratio (cost §1.1) | Spread a/c | Value is… |
|---|---|---|---|---|
| **Inscrutable** | low | Floor 5:4:3 | 1.67 | useful whether or not you descend this tree |
| **Balanced** | mid | Default 3:2:1 | 3.00 | partly conditional |
| **Less guarded** | high | Peak 4:2:1 | 4.00 | strongly conditional; playing it and not continuing is a real loss |
| **Tier-1 reach** | max | beyond Peak (**R-O22**) | >4 | worthless unless you continue |

This **answers R-MC1 at tier 0** — cost §1.1 left "where a given node sits on
this continuum" as an open per-card call; the three tier-0 cards occupy the
three named points by construction. Cost §1.1's own guidance already said peak
for signature single-tree cards and floor for cross-tree combo cards, which is
the less-guarded/inscrutable distinction under different names.

**Concealment type differs by vector** (§5): a Design write conceals
**spatially** — the opponent must come to you to read the fit. A Doctrine
write conceals **temporally** — the opponent can read it for free, but must
wait for trajectories to resolve. The inscrutable slant therefore sits most
naturally on Design, where the gate is stronger.

**The separating condition (R-O19).** If concealment is cheap, everyone opens
inscrutable, nothing transmits, and the yomi channel carries no signal. The
σ→value curve must be **convex**: the inscrutable card must be more than
proportionally weak, so a player with a true plan prefers to broadcast. This is
Spence's single-crossing condition ([Spence 1973](https://doi.org/10.2307/1882010);
[signalling game](https://en.wikipedia.org/wiki/Signaling_game),
[separating equilibrium](https://en.wikipedia.org/wiki/Separating_equilibrium)).
§4 gives it a physical cause rather than a tuned penalty.

---

## 3. σ — three components (R-O51)

σ is **committed production share**, and mineral consumption is its transitive
consequence. Formally `σ = T × concentration(spread)`, where `T` is the card's
total mineral cost — so σ is **continuous**, not quantised into three levels
(**R-O21**). It resolves into three components which are correlated but
**separable**, and that separability is load-bearing:

| Component | What it is | Observable at | Sampling |
|---|---|---|---|
| **σ_commit** | minerals and production the card consumes | not directly observable | — (drives E3, §4) |
| **σ_kinetic** | how far the card shifts your fleet's **acceleration signature** | long range, passive | continuous — a distribution shift, not a census |
| **σ_vector** | how far Doctrine or Design rotates | close scan, or repeated observation | distributional — SPRT over many samples |

**The slant magnitude a rival reads is σ_kinetic, not mass and not hull count.**
Acceleration is the long-range observable (§6.2); a hull census is not something
anyone is handed. *"Those hulls are mining, not colonising"* is σ_vector: a
distribution over destinations needing several looks or proximity.

**The convexity coupling (R-O30), re-derived on acceleration.** A fleet's
acceleration distribution carries role information in its **mean and variance**:
laden haulers sit low, war hulls sit high and cluster tightly. As commitment
grows the distribution shifts and narrows, so posterior entropy over taskings
falls continuously with σ_kinetic. **This is the physical cause of §2's required
convexity**, and it is better behaved than the earlier mass-count formulation
because it is continuous rather than threshold-triggered.

**Concealment is an offsetting move in another latent (R-O52).** `a = thrust /
(dry_mass + cargo_mass)` is one scalar over three latents, so σ_commit and
σ_kinetic come apart: arming a fleet adds mass and *lowers* `a` — loud — unless
the same player also buys thrust, in which case the signature is unchanged while
the commitment is large. **Concealment is therefore a combo property, not a card
property**, which puts it in exactly the same class as the ladder-disruption
combos of §8.3 and satisfies cmd §4's combo backbone without new machinery. A
player who arms and does not re-engine is loud; a player who does both is
invisible at range and has paid twice for it.

**σ_vector is measurable in the engine.** `Doctrine` is a real numeric vector
in `autopilot.rs` (`productivity_step`, `growth_rate`, `survey_vehicles`,
`survey_accel_g`, `expand_bias`, `reinvest_bias`, and the ranking weights
`w_k` / `w_mineral` / `w_hub` / `centrality_scale` / `mineral_pressure_gain`),
so a card's σ_vector is a computable distance between pre- and post-card
doctrine states. **Design has no engine component at all** — this half is
blocked on **R-O28**.

---

## 4. The elite predicate and the selectivity target

**E3 — an opening is elite iff its total committed production Σ falls inside a
viable band `[σ_lo, σ_hi]`.**

- **Under-commit** (Σ < σ_lo): tempo forfeited. Suboptimal, not lost.
- **Over-commit** (Σ > σ_hi): builds you cannot feed → mineral starvation.
  **This is the immediate-loss mode**, and it makes tree progress a genuine
  high-variance bet rather than an asserted one.

Because σ is continuous, the band is a continuous dial. This replaced an
earlier gate-card predicate whose selectivity was quantised at `3/n` and could
not reach the target at all.

**Target: 30–40% selectivity (R-O5).** Measured at the recommended
configuration: **532 legal round-1 openings, 35.3% elite.** The band is robust
to the tier-1 σ value across the swept range 5.0–6.5, so **R-O22 is not
load-bearing**.

**Player-count dependence rides the same dial (R-O23).** Neighbour count is
pinned at 2 for every fair count ≥ 3 by vertex-transitivity, but **adjacency
share** `2/(N−1)` is not:

| N | Neighbours | Adjacent share | Eliminations before a win |
|---|---|---|---|
| 2 | 1 | 100% | 1 |
| 3 | 2 | 100% | 2 |
| 6 | 2 | 40% | 5 |
| 12 | 2 | 18% | 11 |
| 18 | 2 | 12% | 17 |

`σ_hi` scales with adjacency share. At N=3 an early descent projects onto the
whole table and only two eliminations stand in the way, so a high-Σ opening
converts. At N=18 the identical Σ is over-subscription against a payoff
reaching 12% of the table, while being telegraphed to seventeen rivals —
cmd §3's ganging condition. **One parameter, no player-count-conditional card
lists.** If playtesting needs an actions/turn ramp, that ramp is `σ_hi` rising
with round number — the same dial (**R-O24**).

---

## 5. Doctrine and Design as state

Neither is a card. Both are state written **only** by tree cards.

| | Definition | Timing | Leak |
|---|---|---|---|
| **Design** | **the roster** — which hulls and classes exist for you at all | **permanent**; strictly earlier-is-better, D1's decay `f(r)` applies | **never goes stale** — one close scan and they know forever |
| **Doctrine** | **policy over the roster** — what mix is ordered, what roles are assigned | **revisable**; best played when informed | **dies on retasking** |

They have **opposite timing profiles** (**R-O37**), which belongs in
contract §5 as a property of the order families. The asymmetric leak decay
(**R-O39**) is what makes scan investment worth different amounts against
different targets.

**Consequence — the strongest card shape in the game (R-O38).** An inscrutable
Design card is permanent *and* proximity-gated. It must be priced explicitly
rather than left to fall out of the general σ curve.

**Consequence — the information game switches on gradually.** Starting from
LSV and LCV only, one class each, the long-range observable carries almost no
information: everyone's fleet looks identical. Legibility grows as rosters
diverge. Early inscrutability is total *by construction*, which is a far better
source for it than card design.

**The lean is a ratio, not a card count (R-O33).** Per tree and per tier:
`Σ Design-write > floor` and `Σ Doctrine-write > floor`, with the tree's lean
being the ratio of the two totals. Individual cards may be pure — a
pure-Design card is fine so long as the tier's Doctrine total still clears the
floor. Expressing the lean as magnitude rather than count is what lets Warfare
and Technology both lean strongly Design without any archetype losing access
(§10).

**Pure-vector openings are a real archetype.** Three Design cards from three
trees is a counter-graph commitment that builds nothing and is invisible
without a scout. Three Doctrine cards is an economic opening, free to read but
only after trajectories resolve.

**Politics has nothing to write to (R-O27).** The `Doctrine` struct has no
diplomatic fields — no trade lanes, partners, or pact state. Same gap as
galaxy §6's R-A3.

---

## 6. The observation model

### 6.1 What is visible, and at what cost

**This enumeration is open, not closed (R-O53.)** It currently covers the
**kinetic channel** only, because that is the one the engine can already
support. At least two further channels are expected and are not yet specified:

- **Structural** — planetside **infrastructure** and **orbitals**. Static, so
  no trajectory inference applies and there is nothing to throttle; persistent,
  so it cannot be masked the way a burn can. Infrastructure already exists in
  the sim as a `Planet` field and galaxy §5.3 calls it the early binding
  constraint and the softest wartime target. Orbitals are Production's
  win-object, and cmd §3 requires win-objects to be **telegraphed**, so they
  should sit at the maximally-legible end by design.
- **Economic** — mineral drawdown and exchange pressure. This is the leak
  channel that makes off-archetype commitments louder than on-archetype ones
  (you must source your poor colour externally), and it is what turns R-MX1
  ("does pressure ever surface diegetically?") from cosmetic into load-bearing.

Nothing below should be read as a complete capability list. New channels are
expected to arrive with new board objects, and each needs its own
range / latency / maskability profile rather than inheriting the kinetic one.

| Channel | Reveals | Cost | Maskable? |
|---|---|---|---|
| Kinetic — long range, passive, light-lagged | **acceleration signature**; hull class | free | yes, one-sided (§6.2) |
| Kinetic — repeated observation over rounds | destination distribution → tasking | patience | by trajectory, until the cone narrows |
| Kinetic — close scan | **fit, cargo, flight plan** | proximity — a Contact hull must go there | no |
| **Structural** — infrastructure, orbitals | *to be specified* | *open* | likely **not** — static and persistent |
| **Economic** — drawdown, exchange pressure | colour of spend → domain of commitment | *open* (R-MX1) | by trading, at a price |

**A question the structural channel raises (R-O54).** If planetside development
is *quieter* at long range than a burning fusion torch — which is physically
plausible, since infrastructure is not self-luminous — then Growth and
Production players are structurally more inscrutable than Warfare players,
independent of any card choice. That is an asymmetry to price or design against,
and it is not covered by L1 because it does not align with a colour domain.

**Scouting is the counter to inscrutability.** That makes Contact hulls a
standing strategic investment rather than an early-game formality (Banks-
correct: GCUs did espionage) and hands the Expansion tree an information
payoff on top of its territorial one. Even a perfect close scan returns a read
stale by the light-travel time — contract §2's counterplay window, now with a
concrete thing inside it.

### 6.2 Acceleration is the visible σ, not mass (R-O32, R-O40)

**Colony cargo mass ≡ mineral cargo mass.** This removes the cargo-type leak at
its root: a laden hull is a laden hull and the hold's contents do not show in
the burn.

**Acceleration is a one-sided signal.** A ship may fly below peak; it may never
fly above it. Observed `a` is therefore a **lower bound** on capability.
Masking has a quantified price: under-burning means arriving later, the order
realises later, and under D1 that costs `f(Δt)` — the separating condition of
§2 with a physical unit attached.

**The inverse problem is under-determined at range.** `a = total_thrust /
(dry_mass + cargo_mass)` is one observable over three latents, so improved
thrust, reduced hull mass, and reduced cargo mass are indistinguishable.
Breaking the degeneracy requires **angular size at close range**, which yields
dry mass independently and unlocks thrust from there.

*Engine:* `laden_accel` exists. Needed: a doctrine-set **throttle fraction**
per vehicle, and an observation path that derives acceleration from the actual
trajectory rather than reading the stat block.

### 6.3 Destination inference — the reachability cone (R-O31)

The cheap primitive is the **reachability cone**, and it is the same function
`min_time_search.rs` already computes. Isaacs' isotropic-rocket minimum-time
intercept, solved by bisection over the quartic, answers *"how long to reach
point P."* The set of destinations a ship could be heading for by time `t` is
exactly the set with min-time ≤ `t`. **The uncertainty cone and the intercept
solution are one function evaluated in opposite directions**, which satisfies
the constraint that any solution must also serve combat intercept — by
construction, not coincidence. The candidate set prunes through the existing
slab-allocated BSP tree.

The cone narrows as the burn resolves, and light-lag guarantees you are
watching the early, ambiguous phase. A hull bound for a metal-rich
low-habitability world is mining; for a habitable one, colonising; for your
homeworld, conquering — but the hypotheses do not separate until enough arc has
arrived.

### 6.4 Believed kinematics drive combat (R-O41)

An observer must assume a maximum acceleration for a target, but observation
only supplies a **lower bound**. A ship that has been masking has a true
reachable set *larger* than the observed one, so its actual destination can lie
outside the cone the observer computed. **Surprise attack falls out of the
physics** rather than being bolted on. On the combat side, an intercept solved
against a believed `a_max` can simply fail against a target that was
under-burning, and sim §4's deterministic accept/decline runs on believed
kinematics, not true ones.

### 6.5 Detection latency

An observer is running a hypothesis test: baseline policy versus shifted
policy. The optimal sequential test is Wald's SPRT, whose expected sample size
to reach a decision threshold is inversely proportional to the Kullback–Leibler
divergence between the hypotheses ([Wald 1945](https://doi.org/10.1214/aoms/1177731118);
[Wald & Wolfowitz 1948](https://doi.org/10.1214/aoms/1177730197) proved it
optimal). So:

> **rounds-to-detection ≈ threshold / KL(your policy ‖ baseline)**, and σ_vector
> *is* that divergence.

"Inscrutable" therefore means **sub-threshold per observation**, not hidden.
Close range raises per-observation signal-to-noise and cuts the required sample
count.

*Amends:* cmd §5's "the yomi tell is the cards themselves." The tell is the
**fleet and the mineral drawdown**, delayed by light-lag and scoped by
scanning.

---

## 7. Hull, class, and role are three separate things (R-O29)

`BuildOrder::ColonyVehicle { .. }` names the mission in the build order, which
leaks doctrine at range for free. Split it:

- **`BuildOrder::Hull { hull_type, class }`** — production makes an object.
- **Role** is a component assigned *after* production, and **reassignable**.

Roles §4 already keys eligibility to hull type plus loadout, so roles-as-
components is the existing model; the build order is the leak.

**Role eligibility becomes permissive with varying competence (R-O44).**
Roles §4.1 currently restricts Scout to GCV/GCU/LCV/LCU. That restriction leaks
role from hull, which is exactly what this split exists to prevent. An LSV
scouts badly — slow, no dedicated sensor fit — but legally.

### 7.1 Starting state (R-O42)

- **Roster at game start: LSV and LCV only, one class each.**
- **Default doctrine: 100% LSV in the Scout role.**
- Class names follow the Banks convention already in `Hulls & classes`, which
  scales the landform to the hull (Ocean/Plate/System at GSV; Desert/Steppe/
  Plains at MSV; Delta/Escarpment/Mountain/Ridge/River for Contact). Limited
  sizes want small landforms. **Proposed, flavour subject to your authorship:**
  **LSV — Meadow-class** (alts: Fen, Holm, Croft, Hollow; *Furrow* is taken by
  the Growth mouth card), **LCV — Tor-class** (alts: Spur, Cairn, Shoal,
  Gully; avoid *Scree*, Banks's own LCU class).

This makes the opening genuinely opaque: at turn 0 a scout, a settler and a
hauler are the same object. The first real Doctrine decision is when to shift
the mix off 100% LSV.

### 7.2 The armament ladder (R-O43, R-O45)

**LSV unarmed → LCV lightly armed → LCU modestly armed → LOU heavily armed and
armoured.** This is a continuous ladder, not an armed/unarmed toggle, so
Vehicle⇄Unit is **not** a free stand-down: **LCU is a roster-add (a Design
unlock)**, after which toggling between forms may be free. *Resolves R-V8.*
Consistent with `Hyades_loadout.md` §2 — a Systems Vehicle mounting a defensive
beam is just a slot filled.

**Fiction anchor.** At the Battle of Pulo Aura, 14 February 1804, an East India
Company convoy of armed merchantmen drove off a much stronger French squadron
under Linois by presenting themselves as disguised ships of the line. Two years
later Linois was captured at the Action of 13 March 1806 by a battle squadron
he had mistaken for a merchant convoy. The **bidirectional** failure — armed
merchantmen read as warships, then warships read as merchantmen — is exactly
the yomi structure, and it is historical rather than invented.
([Pulo Aura](https://en.wikipedia.org/wiki/Battle_of_Pulo_Aura);
[Linois's expedition](https://en.wikipedia.org/wiki/Linois's_expedition_to_the_Indian_Ocean))

**Mechanically:** an LCU's value is not its guns, it is that at range it cannot
be distinguished from an LOU. Under §6.2 that is already true — an LCU at peak
burn and a laden LOU under-burning present the same acceleration. The bluff is
not a special rule; it is the observation model.

---

## 8. The counter-graph: ladder by default, disrupted by cards

**The counter-graph is per-player state with a total-order initial condition
(R-O48).** At turn 0 it is a strict ladder ordered by hull size and armament.
Cards are the **only** source of intransitivity.

This answers sim §5's open question — DAG or intransitive loops? — as **both,
sequentially**: a ladder at turn 0, cycles by the mid-game, player-authored
rather than designed in.

Three consequences:

1. **The counter-graph is per-matchup, not global.** Each player disrupts the
   ladder differently, so "what beats what" depends on which two players are
   fighting. This is what makes cmd §2's yomi real — you infer an opponent's
   private graph and design against it rather than consulting a chart — and it
   is what makes the close-scan investment of §6 pay. It also gives cost §4's
   legibility goal its proper form: the correct counter-build should be
   readable from what is cheap **for you** against what you believe is in
   **their** roster.
2. **It is a pacing statement.** Early game the graph is a ladder, so there is
   no counter-play and the game is pure economy — precisely the *Stars!* arc
   cmd §10 recapitulates: mature fast, then meet a fleet you cannot match.
3. **Hull size and armament ordering are fine as a default**, because the
   ladder is the substrate to be disrupted rather than a defect to avoid.

### 8.1 Two Design write types (R-O47)

| Kind | Write | σ_kinetic | Reads as |
|---|---|---|---|
| **Retrofit** — e.g. arm all Systems Vehicle designs | modifies existing roster entries | **high unless offset** — added weapon mass lowers `a` fleet-wide | a sudden across-the-board drop in acceleration; nothing new was built, but the whole fleet got slower |
| **Roster-add** — e.g. create an armed Systems Vehicle class | adds a hull+class | high | must be built before it matters; the new hulls carry their own signature |

**Correcting an earlier claim.** A retrofit is *not* inherently invisible. Under
§6.2 the observable is acceleration, and bolting weapons onto every Systems
Vehicle adds mass to every one of them — which is about the loudest single
signal available, precisely because it is fleet-wide and simultaneous. The
retrofit becomes invisible only when **paired with a thrust or hull-mass write
that holds `a` constant** (R-O52). That is the concealment combo, and it means
the strongest-and-quietest Design play costs two cards rather than one — which
is the right price for it.

Whether a retrofit applies retroactively to hulls already in the field or
`on_new_production` only (contract §5) remains the biggest lever on its strength
(**R-O47b**): retroactive means the whole fleet's signature changes in one tick,
lagless, with no build to watch for.

### 8.2 Numbers versus Design — the exchange rate (R-O49)

"Taking down an LOU with LSVs should require numbers or a sufficiently large
Design advantage" is [Lanchester's square law](https://en.wikipedia.org/wiki/Lanchester%27s_laws).
With `k` the baseline quality ratio between rungs and `m` the multiplier Design
buys, `N` LSVs beat one LOU when **`N > √(k/m)`**:

| k | m=1 | m=2 | m=4 | m=8 | m=16 |
|---|---|---|---|---|---|
| 9 | 3 | 3 | 2 | 2 | 1 |
| 16 | 4 | 3 | 2 | 2 | 1 |
| 25 | 5 | 4 | 3 | 2 | 2 |
| 100 | 10 | 8 | 5 | 4 | 3 |
| 400 | 20 | 15 | 10 | 8 | 5 |

**Each doubling of Design quality cuts the required fleet by `1/√2`.** A Design
step is worth a fixed *fraction* of fleet size, not a fixed count, which is why
the substitution stays meaningful at every scale and why the right combo can
lift an LSV into contention with an LCU without breaking the LOU relationship.

Setting `k` per rung is the design decision; **the Ship Testing Arena is where
`q` gets measured**, since per-class values cannot be derived analytically.
Ratified by Monte Carlo (R-C7). The square law is the imperial-scale
abstraction; the individual-vehicle arena calibrates its parameters.

The strategic counterweight is cost §2.5's **indivisibility as liability**: one
LOU is a single point of failure and can be in one place; N LSVs degrade
gradually and can be in N places. **The square law prices the fight;
indivisibility prices the campaign.**

### 8.3 Ladder disruption is inherently cross-tree (R-O50)

Arming a Systems hull is a Magenta-domain Design write applied to a Cyan- or
Yellow-domain object. Ladder-disruption combos are therefore natural instances
of cmd §4's combo backbone rather than special cases, and they satisfy its
balance law — a winning strategy plays from more trees than it descends
deeply — without additional machinery.

---

## 9. Mass, cost, and conservation

### 9.1 The law (R-O57)

**Mass is conserved.** Minerals spent become hull; hull destroyed becomes
wreck-field; hull scrapped returns to the bank. Population is the one exclusion
— biological growth converts local planetary material, so worlds are an open
reservoir for biomass and a closed one for minerals.

**Cost and dry mass are one number.** This is the direct consequence and it
deletes a placeholder: `hull_dry_mass` no longer needs independent
reconstruction or reconciliation against the sweep-tuned propulsion values — it
*is* the mineral cost, in mass units.

### 9.2 The shell model (R-O58)

Conservation exposed a contradiction between two ratified models: cost §1 uses
**surface area as the cost basis**, while the propulsion model had **mass ∝
volume**. If minerals become hull, both cannot hold — a General hull would mass
more than was paid for it.

**Resolution: a hull is a shell.** Dry hull mass ∝ **surface area** — the
material actually bought. Contents (cargo, population, ordnance, fuel) scale
with **volume**. Thrust continues to scale with surface area. Then:

| Quantity | Scaling | Consequence |
|---|---|---|
| `a_empty = T / (k·S)` | T ∝ S, so **size-independent** | empty hulls of every class accelerate alike |
| `a_laden = T / (k·S + ρ·V)` | falls as `1/L` | large hulls suffer badly when full |
| spread `= 1 + (ρ/k)(r/3)` | **linear in radius** | the empty-to-laden range widens with size |

Everything previously ratified survives: cost stays area-based, capacity stays
volume-based, so cost §2's isoperimetric super-linear value argument is
untouched — and now has a physical cause rather than an assumed one.

**A prediction to verify before adoption.** If the measured per-class
acceleration ranges are that empty-to-laden spread:

| Class | Measured range | Implied `(ρ/k)(r/3)` | Relative radius |
|---|---|---|---|
| Limited | 1.82 | 0.82 | 1.00 |
| Medium | 2.79 | 1.79 | 2.18 |
| General | 4.30 | 3.30 | 4.02 |

Radius ratios ≈ **1 : 2.2 : 4**. The *shape* is right — the model predicts
wider ranges for larger hulls, which is what was measured — but the ratios must
be checked against the sweep-tuned values before this is treated as settled.

> **R-O58b checked against the engine — the prediction does NOT currently hold.**
> Computed from `sim.rs` as shipped (`hull_dry_mass` tier 1/2/3,
> `hull_thrust_to_mass` 1.2/1.1/1.0 for Systems hulls, `cargo_mass_per_unit`
> 0.2, one `cargo_unit_size` hauled), the empty-to-laden spreads are
> **2.00 : 1.50 : 1.33** for Limited : Medium : General — *narrowing* with hull
> size, the exact opposite of the 1.82 : 2.79 : 4.30 above.
>
> The cause is that the engine implements neither half of the shell model.
> `hull_dry_mass` scales with a size **tier** (1, 2, 3), a volume-like proxy
> rather than surface area; and there is **no per-hull cargo capacity at all** —
> `cargo_unit_size` is a single flat constant, so a General hull hauls the same
> load as a Limited one. A fixed load against a rising dry mass is necessarily a
> *shrinking* penalty, which is precisely the inverted ordering measured.
>
> So the 1.82/2.79/4.30 figures did not come from this tree, and §9.2 cannot be
> adopted as a re-derivation of existing numbers. It is a **behavioural change
> requiring both halves at once** — dry mass onto area, capacity onto volume —
> after which the per-class propulsion the laser-vs-missile balance rests on has
> to be re-certified (CLAUDE.md §7's flagged placeholder, and `tests/balance.rs`
> goldens). Work items 11 and 12 are therefore coupled and are not independently
> landable.

**A non-combat source of small-fleet value.** A laden General hull is
dramatically slower than an empty one; a laden Limited hull barely differs. In
the kinetic channel, **large hulls broadcast their load state and small ones do
not.** Consolidation always wins under geometry alone and the counterweight has
so far had to come from combat effects; this is a counterweight that is not
combat at all — small fleets are harder to read.

### 9.3 Slag (R-O59)

Synthesis wastage is not destroyed, it is **degraded**. Slag is **useless by
default**, and a **tier-1 card makes it refinable**. Wastage-reduction and
slag-refining are therefore both mass-recovery plays, and the mass sits on the
board in the meantime rather than vanishing from the ledger.

### 9.4 Ordnance is mass; energy weapons are not (R-O60)

The weapon families split on a **logistical** axis, orthogonal to the
range/kinematic axis in `Hyades_loadout.md` §3.2:

| Family | Magazine | Consequence |
|---|---|---|
| **Torpedo, Missile** | **mass carried; expended and lost** | requires supply lines; ammunition depletes |
| **Beam, Pulse** | none | no tail; power-limited instead |
| **Exotic** | open | — |

Four consequences:

1. **An ordnance fleet has a logistics tail** — freighters or an accompanying
   foundry hauling munitions. That tail is a visible, attackable board object,
   and it gives cmd §8's **Blockade** a concrete target rather than an abstract
   one.
2. **A durable intransitivity, not a ladder rung.** Under Lanchester (§8.2), a
   fleet out of ammunition has collapsed `q`. So **ordnance wins the opening
   exchange and energy wins the long one** — a genuine cycle that pairs with
   the range/kinematics counter to give a two-axis counter-graph.
3. **The magazine state is readable kinetically.** Full magazines are mass:
   heavier, slower. A fleet that has expended its ordnance accelerates better.
   **Post-battle acceleration reports ammunition state for free**, straight
   into the σ_kinetic channel — no new mechanic.
4. This is a non-ladder asymmetry available early, but only once weapons exist
   at all, which is after the LSV/LCV opening (§7.1).

*Folds into R-XM6 (does an ammunition system exist) — answered: yes, for
ordnance families only.*

### 9.5 The specific-strength ladder (R-O61)

Higher mineral tiers carry more capability per unit mass. Since dry mass is
cost and thrust scales with area, a super-built hull delivers **higher
acceleration at equal capability** than a basics-built one.

**Technology therefore becomes legible through the kinetic channel by physics
rather than by rule** — which is what the observation model wants, and a
cleaner reading of the "3 CMY = 2 RGB = 1 Platinum" order-of-magnitude value
heuristic than a conversion ratio.

### 9.6 Negative and imaginary mass (R-O62)

**Conservation is not violated.** It holds for negative and imaginary masses
too. What exotic technology changes is the *sign* — and possibly the *phase* —
of a component's mass, not the total.

- **Consequence: exotic synthesis is pair production.** Producing a −M
  component while conserving the total requires producing +M alongside it. The
  counterweight is dead mass that must be dumped, parked, or used. Exotic
  technology arrives with an equal-and-opposite disposal problem built in.
- **Degree, not tier.** Negative-mass components are a **factor in RGB-tier
  designs**, not the defining element of Platinum-tier ones. Platinum is more
  of it, not the only source of it.
- **Surprising acceleration.** With `a = T / (m_pos + m_neg)` and `m_neg < 0`,
  effective mass falls toward zero and acceleration rises steeply — divergently
  near cancellation. This is the "surprising characteristics" and it is a
  continuum, matching the degree framing above.
- **Exotic is the one thing that cannot be hidden kinetically.** Angular size
  gives volume and surface area, which under §9.2 gives expected dry mass and
  therefore an acceleration envelope for the class. A hull accelerating
  *outside* that envelope is carrying exotic mass. The observer's inverse
  problem (§6.2) is normally under-determined; exotic resolves it by pushing
  the observation outside the physically possible range. A sufficiently extreme
  anomaly is readable even without angular size, since it exceeds the maximum
  for *any* known class.
- **Imaginary mass** is treated as a separate conserved channel — real and
  imaginary parts each conserved — with physical interpretation and magnitudes
  left open.

---

## 10. Design laws

**L1 — The co-extension law (R-O34).** *No **categorical** strategic
classification may be co-extensive with a colour domain.* Archetypes are
defined by colour pairs, so any categorical classification aligned with the
colour partition locks out exactly the archetype poor in that colour. Known
instance: **Latent is exactly the Cyan domain**, so Red-type (Cyan-poor) is
locked out of a whole tempo group — R-O8 / R-G6, still open. The law binds
*categorical* classifications only; continuous ones expressed as magnitude
(such as the Doctrine/Design lean, §5) are exempt, which is why Warfare and
Technology may both lean Design.

**L2 — The conservation rule (R-O33).** Per tree and per tier, both vectors
must clear a floor; the lean is the ratio of the totals. Individual cards may
be pure.

**L3 — Convexity / separating condition (R-O19).** The σ→value curve must be
convex, or the pooling equilibrium is the game. §3's mass-entropy coupling is
its physical cause.

**L4 — Legibility is σ from the other side.** Not a separate stat. This is why
the tier-1 reach is automatically both maximum slant and maximum legibility.

**L5 — Hard counters are timing-dependent, earlier strictly better, at an equal
gradient for Kinetic, Potential and Latent (D1).** The decay applies to
**commitment time**, not realization time: a Potential counter still pays off
late; committing it in round 1 beats committing it in round 4 by the same
factor a Kinetic counter would lose. One shared decay function `f(r)`. Per
R-O37 this applies to **Design writes only**. Belongs in contract §5.

**L6 — Mass is conserved (R-O57, §9).** Minerals spent are hull dry mass; there
is no second number. Wastage degrades to slag rather than vanishing; ordnance
expended leaves the fleet lighter; wrecks retain their mass. The single
exclusion is population, which converts local planetary biomass. Negative and
imaginary masses are **not** an exception — conservation holds for them, which
is why exotic synthesis is pair production (§9.6).

---

## 11. Engine work items

| # | Change | Blocks / R-code |
|---|---|---|
| 1 | Split `BuildOrder::ColonyVehicle` (and mining/freighter builds) into `BuildOrder::Hull { hull_type, class }` + separate role assignment | R-O29 |
| 2 | Add a **Design/roster component** — which hull types and classes a player has unlocked | **R-O28** (blocks σ_vector for Design entirely) |
| 3 | Add **diplomatic fields** to `Doctrine` — trade lanes, partners, pact state | R-O27 / R-A3 |
| 4 | Add a **throttle fraction** to `Doctrine`; derive observed acceleration from trajectory, not the stat block | R-O40 |
| 5 | Equalise colony cargo mass and mineral cargo mass | R-O32 |
| 6 | Expose `min_time_search` as a **reachability cone** query (same function, reverse direction); prune candidates via the existing BSP tree | R-O31 |
| 7 | Route intercept and sim §4 accept/decline through **believed `a_max`** | R-O41 |
| 8 | Rewrite roles §4 eligibility lists as **permissive with competence** | R-O44 |
| 9 | `Galaxy::FAIR_COUNTS` is `[2, 3, 6, 12]` and rejects 18, while galaxy §2 lists 6r (12, 18) as fair and `starting_hex_radius` already carries an `18 => 4.5` branch | R-O12 |
| 10 | Seed the starting roster: LSV + LCV, one class each; default doctrine 100% LSV Scout | R-O42 |
| 11 | **Derive `hull_dry_mass` from mineral cost** — delete it as an independent field. Resolves the flagged placeholder rather than reconciling it | R-O57 |
| 12 | Re-base hull mass on **surface area** (shell), contents on volume; verify the 1 : 2.2 : 4 radius prediction against the sweep-tuned propulsion values before adopting | R-O58 |
| 13 | Track **slag** as a bank entry: inert by default, refinable once the tier-1 card is played | R-O59 |
| 14 | **Magazine mass** on ordnance families only; expended rounds leave the fleet, raising `a` | R-O60 / R-XM6 |
| 15 | Retrofit realization: add an **`on_refit`** mode — value accrues as hulls reach a friendly port; a mobile foundry may or may not count | R-O47b / R-O55 |

---

## 12. Superseded and retracted

| Claim | Status |
|---|---|
| galaxy §8 — the 11-card round-1 list | **superseded** by 18 tier-0 cards + tier-1 reaches |
| galaxy §6 — Compass/Pattern as round-1 overrides (4 rows) | **superseded**; tree cards take those rows |
| cmd §5 — "the yomi tell is the cards themselves" | **amended**: the tell is the fleet and the drawdown, lagged and scoped |
| sim §5 — is the counter-graph a DAG or does it carry cycles? | **answered**: ladder at turn 0, player-authored cycles thereafter (R-O48) |
| roles §4.1 — Scout eligible only for Contact hulls | **superseded** by permissive eligibility (R-O44) |
| Opening-space rev 1 §4.2 — "81% of the space is strictly worse" | **retracted**; the no-descent block is the low-variance half of the elite set |
| Opening-space rev 1 §7.1 — neighbour count carries no variation | **amended**: true of the count, false of adjacency share |
| Opening-space rev 2 — the `3/n` selectivity ladder and gate-card predicate | **superseded** by E3 |
| Common Act symmetry work (R-O7, R-O14, R-O15, R-O20) | **moot** — the class is deleted |
| R-O1, R-O2 (Compass/Pattern menus), R-O18 (two-gate predicate) | **moot** — no such cards |
| R-O26 (Doctrine/Design concealment asymmetry) | **resolved** by §6.2's acceleration model |
| R-O35 (which Magenta tree flips to Doctrine-lean), R-O36, R-O46 | **withdrawn** — lean is magnitude, not count |
| R-V8 (Vehicle⇄Unit toggle) | **resolved**: roster-add, then free toggle (R-O45) |
| R-MC1 (position on the ratio continuum) | **answered at tier 0**; open for deeper tiers |

---

## 13. Open R-code register

**Ratified this thread:** R-O5 (30–40% target), R-O16 (tier-1 reaches legal at
round 1), R-O4 (siblings exposed — the +1 is per node), R-O29, R-O30, R-O31,
R-O32, R-O33, R-O37, R-O38, R-O39, R-O40, R-O41, R-O42, R-O43 (LSV unarmed),
R-O45, R-O47 *(principle ratified; realization is now `on_refit` — see below)*,
R-O48, R-O49 *(principle ratified; magnitudes open)*, R-O50, R-O57, R-O58
*(pending the radius check, R-O58b)*, R-O59, R-O60, R-O61, R-O62.

**Retrofit, ratified separately:** no deep-space retrofit by default; refit at a
friendly port only; a card enables field refit for minerals. This answers
**R-O47b** with a third realization mode — **`on_refit`**, absent from
contract §5 — under which a fleet-wide Design change lands staggered by transit
time rather than in one tick. Two consequences: the **recall is itself a signal**
in the kinetic channel and cannot be offset by a thrust write, and refitting
hulls are **out of position**, making the retrofit a vulnerability window rather
than a power spike.

**Open, this thread:**

| Code | Question |
|---|---|
| R-O3 | write the "intra-round ordering is immaterial" assumption into the contract |
| R-O6 | is a pass observable? (decides weak vs. strict dominance of unspent actions) |
| R-O8 | Latent is co-extensive with Cyan → Red-type's (2,2,0) profile. **The outstanding instance of L1** |
| R-O9 | N=2 seats only Blue and Red; Green never appears |
| R-O11 | where the directed counter-cycle lives; write `f(r)` into contract §5 |
| R-O12 | `FAIR_COUNTS` rejects 18 |
| R-O13 | is a target part of the move, or a sub-decision? (survives for tier-0 slants that carry targets) |
| R-O19 | size the convexity of the σ→value curve |
| R-O21 | fix the `σ = T × concentration(spread)` mapping |
| R-O22 | tier-1 reach σ — a fourth spread point beyond Peak, or Peak at higher `T`? *(shown not load-bearing)* |
| R-O23 | `σ_hi` as a function of adjacency share `2/(N−1)` |
| R-O24 | actions/turn ramp as `σ_hi` rising with round |
| R-O27 | `Doctrine` has no diplomatic fields |
| R-O28 | Design has no engine component; blocks σ_vector measurement |
| R-O34 | ratify L1 as a stated law |
| R-O42b | confirm class names (Meadow / Tor, or your alternatives) |
| R-O47b | does a retrofit apply retroactively, or `on_new_production` only? |
| R-O49b | set `k` per ladder rung and `m` per Design card, arena-measured |
| **R-O51** | ratify the three-way σ decomposition — σ_commit / σ_kinetic / σ_vector — with σ_kinetic derived from the acceleration signature, not mass or hull count |
| **R-O52** | concealment-by-offset: cost the thrust-and-armament combo that holds `a` constant while σ_commit is large. Concealment is a combo property, not a card property |
| **R-O53** | the observable-channel enumeration is **open**. Specify the structural channel (infrastructure, orbitals) and the economic channel (drawdown, exchange pressure), each with its own range / latency / maskability profile |
| **R-O54** | is planetside development quieter at long range than a burn? If so, Growth and Production are structurally more inscrutable than Warfare, independent of card choice |

**Pre-existing, depended on:** R-5 (full clock math), R-7/R-9 (round-1 cost
numbers — now the *sole* throttle on breadth), R-C1, R-C2, R-C5, R-C7,
R-M8, R-MC1 (deeper tiers), R-MC3b (arena harness), R-MX1 (does exchange
pressure surface diegetically — it is the mineral leak channel), R-CG28,
R-G6 (merges with R-O8), R-A3 (merges with R-O27).

---

## References

- Wald, A. (1945). "Sequential Tests of Statistical Hypotheses." *Annals of
  Mathematical Statistics* 16(2), 117–186. https://doi.org/10.1214/aoms/1177731118
- Wald, A. & Wolfowitz, J. (1948). "Optimum Character of the Sequential
  Probability Ratio Test." *Ann. Math. Statist.* 19(3), 326–339.
  https://doi.org/10.1214/aoms/1177730197
- Spence, A. M. (1973). "Job Market Signaling." *Quarterly Journal of
  Economics* 87(3), 355–374. https://doi.org/10.2307/1882010
- Shannon, C. E. (1950). "Programming a Computer for Playing Chess."
  *Philosophical Magazine* 7th ser., 41(314), 256–275 — ~30 legal moves per
  position. https://vision.unipv.it/IA1/ProgrammingaComputerforPlayingChess.pdf
- Gobet, F. (1998). "Chess players' thinking revisited." *Swiss Journal of
  Psychology* 57, 18–32, Table 1 — base moves by skill: Grandmasters 4.2,
  Masters 3.2, Class A 6.5. Retained as the live-count reference; the
  selectivity target has since moved to 30–40%.
  https://bura.brunel.ac.uk/bitstream/2438/822/4/1791.pdf
- Battle of Pulo Aura (1804) and the 1806 reversal.
  https://en.wikipedia.org/wiki/Battle_of_Pulo_Aura ·
  https://en.wikipedia.org/wiki/Linois's_expedition_to_the_Indian_Ocean
- Lanchester's laws. https://en.wikipedia.org/wiki/Lanchester%27s_laws
- Signalling game · separating equilibrium.
  https://en.wikipedia.org/wiki/Signaling_game ·
  https://en.wikipedia.org/wiki/Separating_equilibrium
- Vertex-transitive graph (why adjacency is pinned at 2).
  https://en.wikipedia.org/wiki/Vertex-transitive_graph
- Intransitivity. https://en.wikipedia.org/wiki/Intransitivity

## Appendix — reproduction

`r1_space_v3.py` (gate-card model, superseded), `sigma_band.py` (E3 band
sweep), `arch_symmetry.py` (L1 instances). Zero-dependency Python 3.
