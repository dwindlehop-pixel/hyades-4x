# Hyades — todo / parking lot

Everything known to be outstanding, in **one list ordered from specific to
vague**. The `Hyades_*.md` docs are specs with ratification points; this file is
the register of what has not been settled or not been built, including the
pre-spec ideas that are only worth not losing.

**How to read the order.** The list runs from *the change is known and only the
work remains* down to *we cannot yet say what this is*. Position in the list is
therefore a claim about **how much design is left**, not about priority or
sequencing — a Band A item can be low value and a Band E item can be the most
important thing in the game. Read the band, then decide.

| Band | Meaning | What is missing |
|---|---|---|
| **A** | Ready to build | nothing — the change is specified |
| **B** | Decided, needs a design pass | the shape, not the direction |
| **C** | Open question with a concrete test | the answer, but you know how to get it |
| **D** | Direction only | the formulation |
| **E** | Parking lot | the ability to state the problem |

**Identifiers.** Every entry carries a permanent `T-nn`. **IDs are never reused
and never renumbered** — a new entry takes the next free number wherever it
lands in the order, and an entry that graduates into a spec is struck through
here with a pointer rather than deleted. So the IDs will stop being sorted, and
that is the point: `T-14` means one thing forever, while its position in this
file tracks how well understood it currently is.

R-codes are cited where one exists. An entry with no R-code is engine work with
no open design question attached.

---

## Band A — ready to build

### T-01. Wire `matching.rs` into `lib.rs`

The Exchange (order-book matching) is built and tested but not exported, and
`sim.rs` still calls `most_needed_center` directly. Swap the call sites.
`most_needed_center` is **retained permanently as a test oracle** (design law
#5, R-MX6): single-supply degenerate matching provably reduces to it, so it is
the thing that proves the Exchange right, not dead code.

Also the main lever on the O(P) freighter-routing scan, which now runs against
6,725 planets rather than the 600 `Hyades_matching.md` assumed.

### T-02. Restore `examples/bench_hex_size.rs`, sweeping entity count

Cited by `galaxy.rs` (×3), `tests/smoke.rs`, `tests/determinism.rs` and
`Hyades_matching.md` — and **absent from this tree**, so the throughput claims
resting on it are currently uncheckable. Rewrite it to sweep *entity count*, not
just hex size: entity count is the first-order cost now that the snowball is the
shipped behaviour (design law #14), and hex size is not.

### T-03. Slag as a bank entry — R-O59

Synthesis wastage degrades rather than vanishing. Inert by default; a tier-1
card makes it refinable. Wastage-reduction and slag-refining then both become
mass-recovery plays, and the mass sits on the board in between instead of
leaving the ledger. Standing-layer §9.3, roadmap item 13.

### T-04. Magazine mass on ordnance families — R-O60 / R-XM6

Ordnance is mass; energy weapons are not. Expended rounds leave the fleet, so a
fleet that has been shooting accelerates *better* — which is a live observable
under §6.2, not just bookkeeping. Standing-layer §9.4, roadmap item 14.

### T-05. `min_time_search` as a reachability-cone query — R-O31

The same function run in reverse: instead of "how long to reach this target",
"what is reachable within this time". Prune candidates through the existing BSP
tree. Standing-layer roadmap item 6.

### T-06. Split three long functions

`sys_production_tick` (~149 lines), `apply_build_with` (~148), `production_choice`
(~137). Pure refactor, no behaviour change — verify by A/B on seed 1.

### T-07. Recalibrate `centrality_scale`

Still tuned to an old ~25 ly galaxy extent; the galaxy is now hundreds of ly, so
the centrality term is effectively saturated. `k_high` was the other half of
this pair and is resolved at 3.2 (R-AC17); this one is not. One of the two
nearest levers on T-19.

---

## Band B — decided, needs a design pass

### T-08. `on_refit` retrofit realization — R-O47b / R-O55

**No retroactive refits** is ratified (design law #12): a Design write never
reaches a hull already in the field by fiat. Value accrues as hulls reach a
friendly port, so a fleet-wide change lands staggered by transit time and the
recall is itself a signal. What needs designing is the realization mode —
whether a mobile foundry counts, and what a partially-refitted fleet's
acceleration distribution looks like while the change is in flight. Roadmap
item 15.

### T-09. Throttle fraction on `Doctrine`; observe `a` from trajectory — R-O40

A ship may fly below peak acceleration and never above it, so an observed `a` is
a **lower bound** — which is where surprise attack comes from (design law #10).
Requires deriving the observable from the trajectory rather than reading the
stat block. Roadmap item 4; T-10 depends on it.

### T-10. Route intercept and accept/decline through *believed* `a_max` — R-O41

**Half done.** `src/belief.rs` holds the estimator and the decision:
`BeliefAMax` folds light-lagged observations into the *maximum* acceleration a
target has ever been caught making — one-sided, because a ship may fly below
peak and never above it — and `decide_engagement` resolves accept/decline
against it. Belief is monotone, so masking is **spend-once**; a 4,851-case sweep
pins that the decision can only err by optimism, which *is* the surprise attack
rather than a bug to guard.

**What remains is the wiring, and it is blocked on T-30.** There is no
accept/decline site in the engine: `combat::resolve_engagement` is a pure
tactical resolver over two fully-specified fleets, and no round/command layer
exists for a "do I take this fight" decision to live in. Belief is also
*supplied* by the caller rather than harvested from trajectories, which is
T-09's half. Neither changes the estimator.

Rule to hold when the wiring lands: the decision reads a `BeliefAMax` and
**never** the other side's `Combatant::max_accel`. Missile terminal guidance is
deliberately exempt — §6.2 puts close range as where the degeneracy breaks.

### T-11. Diplomatic fields on `Doctrine` — R-O27 / R-A3

Trade lanes, partners, pact state. **No field list has been specified**, which
is the whole of the work. Roadmap item 3.

### T-12. R-MC9c — layer HP, weapon count, AoE and magazines onto combat

All as `CombatConfig` fields plus slot-derived stats: HP pools (a GOU must not
die to one hit); weapon *count* scaling with hull slots while per-hit damage
stays a single global constant (this is what keeps design law #2 intact);
missile AoE for dense LOU swarms; magazines, so LOUs are better laser platforms
than missile platforms.

**Balance-preservation constraint:** keep the AoE radius *below* baseline ROU
formation spacing, so the AoE term is identically zero in the ROU-vs-ROU case
and the existing laser-vs-missile balance is untouched. Verify against
`--example laser_vs_missile`.

### T-13. Counter-graph matrix partition — card contract §8.3

Define Red-class positions against Blue/Green-edge positions, so design law #1
(mineral substitution lives in the counter-graph, not the mineral ladder) has
mechanical grip rather than being a statement of intent.

### T-14. R-SIM1 — light survey view

`Autopilot::choose_survey_target` takes `&[SurveyView]` and reads a fraction of
each entry. Sizing the view to the query is the largest remaining per-ship win
(`view_of` alone was 18% of engine instructions), but it adds a second view type
to the fog-of-war contract (`Hyades_simulation_model.md` §1/§2b). **Deliberately
not taken yet** — it is a contract decision, not a free optimization.

### T-30. The round/command layer — netcode B1

Sim §3's four phases (income → hidden simultaneous command → resolution →
aftermath) and `Hyades_netcode.md`'s protocol clock `r` have **no engine
counterpart**. `Simulation::run()` drains a discrete-event queue to a horizon;
there is no round boundary, no simultaneous order, and no `apply_orders`.

This is the **largest single gap between the engine and the shipped design**,
and it is simulation work rather than network work. Netcode §5's barrier, §5.1's
order coercion, §5.2's timeout-as-message and §11's single inbound entry point
all attach here and nowhere else. Direction is fully specified; the shape inside
the engine is not.

When it lands, give the engine **exactly one** inbound entry point
(`apply_orders`) rather than enforcing the one-directional seam by convention —
netcode H9/§2.1. Today the seam is trivially safe because there is no inbound
path at all; that is the property to preserve, not to rediscover.

### T-31. A card system — netcode B2

Cards are the game's entire action layer and the engine has none. Blocks:
`card_id` in the wire payload (netcode §4.2), roster enforcement (T-25),
counter-graph disruption (T-13), and every "a card does X" clause across the
specs. R-C1 (the closed list of legal `target_rule` kinds) is the first thing
inside it, and R-NET4's field widths are blocked on R-C1 in turn.

### T-32. The state digest — netcode B3

Netcode §8.1's Merkle root over `galaxy`, `players`, `vehicles`, `event_queue`,
`rng_cursor`, `exchange_books`, `counter_graph`. **Two leaves have no engine
representation at all** — `exchange_books` waits on T-01 and `counter_graph` on
T-13 — and the other five have no canonical encoding.

Two rules from the spec worth carrying into the implementation: digest the
authoritative state and *not* `Snapshot` (a projection lets divergence hide in
what it drops, and the event queue and RNG cursor especially must be inside),
and exchange leaf vectors on mismatch so a desync localizes to a subsystem
rather than to a round number.

### T-15. Production-queue redesign — roles §10, deferred

Expand whenever affordable, rather than through a competing bias dial. Partly
overtaken by R-O29: the `BuildOrder` match no longer names missions and role
assignment has moved into `Autopilot::assign_role`, so production choice can
become "what does my current Role's System say to build". The dial
(`expand_bias`) survives and is what this would replace.

---

## Band C — open question with a concrete test

### T-33. `Knowledge` stores membership, not observations — netcode B4

`Knowledge::scanned` is a `BTreeSet<PlanetId>`, so it records *that* a world was
scanned and never *what was seen*. Every subsequent read through `view_of`
re-reads current ground truth — `factors`, `density`, `population`, `owner` — so
a 500-year-old scan yields today's values with **zero lag**.

That is netcode §2.1's causal failure exactly: an agent acting on information
that has not reached it. Note what it is *not* — it is not a desync risk, since
every client computes the same wrong thing, so no checkpoint will ever catch it.
It is a game-correctness bug that §2.1 promotes to a design-law violation.

The fix is to store observed values with an as-of round, which is the same
per-player-per-planet storage R-SIM4 (T-26) flags as expensive at fleet scale —
so the two should be designed together. Concrete test once it lands: a scan,
then a change to the world, then a read must return the *pre-change* value until
light has had time to carry the update.

### T-34. Colonization filters on instantaneous global ownership — netcode B5

The colonization candidate loop skips a world when `world.owner.contains(e)`,
whether or not the acting player has observed the claim. The survey path makes
the same kind of call but documents it and defers to R-SIM3; the colonization
path does neither, and it is the one that matters — it reads a rival's state
with no observation behind it. Narrower than T-33 and fixable independently.

### T-16. R-O63 — the biosphere regrowth magnitude

`SimConfig::biosphere_regen_rate` defaults to `0.10` of the remaining deficit
per cycle, a **placeholder**. It decides how long a razed world stays razed, and
therefore whether biological warfare is a strategy or a rounding error. Testable
directly: sweep it and measure how long a zeroed biosphere suppresses `K`.

### T-17. R-O65 — should `hull_thrust_to_mass` be flat within a family?

The shell model says empty-hull acceleration is size-independent (thrust and dry
mass both scale with area), but the code still carries a 1.2 / 1.1 / 1.0 ladder
across Systems sizes. Not flattened when R-O58 landed, because it is an MC-tuned
combat surface and CLAUDE.md §6 requires ratification before those move. It
reaches only `arena`/`combat` — civilian motion runs on `civilian_accel_g` — so
this is a one-line change plus a balance re-run.

### T-18. R-O64 — confirm the reinterpretation of roles §6's cargo ladder

The 0 / 1 / 2 capacity ladder was *confirmed*, but as a **unit count**, and the
shell model makes capacity a mass on a cubic ladder. The engine keeps the
ordinal content (Limited zero, strictly increasing) and takes magnitudes from
geometry. Flagged because it reinterprets something a spec calls confirmed —
needs a yes rather than a re-derivation.

### T-19. Does the 1 : 2.2 : 4 radius ladder beat 1 : 3 : 9?

Standing-layer §9.2 predicted radii of 1 : 2.2 : 4. Under the shell model the
radius ladder is *derived* from the cost ladder, so that prediction is now a
**tuning target in existing knobs**: it asks for `limited_fleet_size = 16` and
`medium_fleet_size = 3.31` against the shipped 9 and 3. Offline search question,
answerable by `min_time_search`.

### T-20. Raise coverage inside a fixed 4,000-year run

Currently **15.5%** of colonizable worlds (1,044 of 6,725, seed 1) — down from
17.6% when laden ships stopped flying for free (R-O57/R-O58). **4,000 years is
the run length; the coverage reached within it is the objective.** Do not extend
the horizon: doubling it doubles every trial and the 60-second rule already had
to absorb the snowball once. T-07 and T-21 are the nearest levers.

### T-21. R-SIM2 — survey scan cost

The survey scan is still O(planets) per arrival. The trigger is right (arrival
driven) but the per-evaluation cost is not local, which is exactly the product
CLAUDE.md §4 warns about. Note the recorded negative result before retrying: an
incrementally maintained unvisited frontier cut the scanned count 39% and came
out *slower*, because swap-removal traded a sequential walk for random access.
Locality beat count. Measure, do not assume.

### T-22. R-ARENA2, 3, 4, 6, 7 — arena calibration placeholders

Station-keeping radius/period ranges (2); whether a Systems Vehicle's cargo
should count against its combat mass (3); the position/interception treatment
(4); the tactical-range impossibility claim (6); and the weapon constants the
arena exists to calibrate (7). Design law #4 makes the Ship Testing Arena the
required harness for these — they cannot be derived analytically.

### T-23. R-MX1–5 — Exchange design calls

Whether market pressure ever surfaces diegetically (1, feeds T-28); market-tick
cadence and per-fill light-lag (2); distance-discounted price as a doctrine knob
versus price-then-distance (3); mining-bid concurrency quantity (4); one Book
per (empire, commodity) versus per-commodity merge (5).

### T-35. Netcode ratification points with concrete tests

Open `R-NET` calls that name their own experiment: **R-NET5** (Merkle leaf
partition and checkpoint cadence), **R-NET7** (headless replay throughput vs.
match length — decides whether snapshot-assisted catch-up is needed),
**R-NET10** (enable `simd128` in the pinned build), **R-NET15** (state-digest
cost at an 18-seat galaxy — planet count there is *unmeasured*, do not
extrapolate from 12). R-NET15 pairs naturally with T-24, which has the same
unmeasured 18-seat corner.

### T-36. Netcode policy calls needing a decision, not a measurement

**R-NET6** (does a defaulted round consume the action or refund it — fires in
~30% of rounds at 18 seats, so it is a balance decision rather than a corner
case), **R-NET8** (even split under `SimpleMajority`; the spec recommends Halt,
since a seat-order tiebreak rewards whoever bribed seat 0), **R-NET16**
(match-start admission threshold), **R-NET17** (liveness beacon as a second
eclipse tell, against putting per-round data back on a server that currently
carries none), **R-NET18** (`m_ingress` and spectator gossip degree `d`).

### T-24. Throughput watch — the 12-seat × 8-kyr corner is unmeasured

Confirmed floor is **2.5 simulated-years/real-second**. Measured: 3 seats/4 kyr
456 yr/s, 3 seats/8 kyr 79 yr/s, 12 seats/4 kyr 128 yr/s. Degradation is
**superlinear in duration** and roughly linear in seat count, so the worst case
is long horizons rather than wide tables; extrapolating puts 12 seats × 8 kyr
near 20 yr/s, an ~8× margin. Measure it. Treat approaching the floor as the
trigger to **optimize, not to shrink the scenario**.

---

## Band D — direction only

### T-25. Enforce the starting roster — R-O42, blocked on cards

§7.1 ratifies a starting roster of LSV + LCV only, and the engine seeds exactly
that. `SimConfig::enforce_roster` **defaults off**, because there is no card
system and therefore no unlock path: the colonizer and freighter ride on the
Medium hull the starting roster excludes, so enforcement forbids every expansion
build permanently — 3 colonies and 18 vehicles against 1,183 and 4,778 over
4,000 years, pinned as a test.

Not an argument against §7.1; an ordering constraint. **Blocked on the card
layer existing at all**, which is the vague part — no card system is specified
in engine terms yet.

### T-37. Everything in netcode outside the crate

Topology (§3), the 144-byte frame (§4), genesis assembly (§7), relay and
reconnection (§9), server posture (§10), client hardening (§11). **None of it
blocks on the engine** and none of it lives in this repo — it is client and
service work whose only contract with `hyades-engine` is determinism, one
inbound entry point, and no host access, all of which hold today (see the
netcode engine-status block). Listed so it is tracked somewhere; it needs a home
before it needs a design.

### T-26. R-SIM4 — departure-traffic confidence

R-SIM3 settled that occupancy is inferable at range, and the pop-4 industrial
signature is implemented exactly with no new state. The **graded** signal is
not: repeated sightings of ships leaving a world should raise confidence it is
held. That needs accumulated light-lagged observations per player per planet,
which is precisely the storage the simulation model warns about at fleet scale.
The mechanism is agreed; the representation is not.

### T-27. R-XM5, 6, 7 — exotic matter

Cited as open in CLAUDE.md §7 and referenced from the standing layer (R-XM6 is
answered in passing — yes, an ammunition system exists — via R-O60). **No
definitions for R-XM5 or R-XM7 exist anywhere in this tree.** Recover them from
`Exotic_matter_technology_inspiration.md` or restate them before treating them
as tracked work. Related ratified ground: exotic synthesis is pair production,
because conservation holds for negative and imaginary mass too (design law #11).

### T-28. R-ARENA1 and R-ARENA5 — cited, undefined

Both appear in CLAUDE.md's open-R-code list and **nowhere else in the tree** —
no definition in `docs/`, none in `src/`. Either recover them from history or
retire the numbers. Listed here so the gap is tracked rather than silently
inherited; numbers are never reused, so retiring them costs nothing.

---

## Band E — parking lot

Ideas flagged during design sessions as real but **not yet articulable enough to
spec formally**. A place to not lose the idea, not a place to design it
prematurely. When an entry becomes articulable it graduates into the relevant
spec doc and is struck through here.

### T-29. Nonconsensual role change — a "bidding system"

*Raised in design, explicitly flagged as not yet spec-able:*

> I envision a sort of bidding system for role change (a pirate far away
> can't demand tribute, but one co-located can) but I cannot articulate it
> yet so it's useless to spec out.

What's known so far:

- **Co-location is required** for one civilization to induce a role change
  on another's entity — consistent with `Hyades_vehicle_roles.md` §5's
  Fleet rule (same role + co-located). A distant threat can't compel
  anything; only a present one can. (This is the one piece concrete enough
  to already be reflected in `Hyades_vehicle_roles.md` §4.4's Tribute
  entry.)
- **Growth, Production, and Politics will all have cards** that induce
  nonconsensual role change on an opponent's entity — not confined to one
  tree.
- **The user story is economic, not (only) traitorous:** *"the other
  civilization offered my entity a better deal than it could get
  elsewhere."* Treason/defection is in the design space, explicitly, but
  isn't the primary intended flavor.
- **The main intended form is softer:** mineral or ship trades that are not
  in the affected civilization's strategic interest — an incentive/bidding
  mechanic that talks a civ (or its autopilot) into a bad trade, rather than
  outright capture or defection.
- **A literal "bidding system"** is the working name for the mechanism that
  would decide when such an offer succeeds — not designed yet.

**Not to be spec'd until it can actually be articulated.** Revisit when
there's more shape to it. R-MX1 (does Exchange pressure ever surface
diegetically) is the nearest thing to a handle on it.

---

## Graduated

Entries that have landed. Kept as stubs so the IDs stay unique and the pointer
survives.

*(none yet — this file was restructured after R-O29/R-O44 and R-O57/R-O58 had
already landed, so those are recorded in
`Hyades_standing_layer_and_observation.md` §11 rather than here.)*
