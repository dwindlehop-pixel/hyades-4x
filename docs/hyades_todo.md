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

*(Unrelated to the "Band I–IV" magnitude-tier terminology in
`Hyades_mineral_cost_curve.md` §2.6 — this is a lettered readiness scale for
todo entries; that one is a numbered multiplicative-quantity ladder for
game state. Same word, different concept, kept apart by alphabet on
purpose.)*

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

**Reclassified: this is not cleanup, it is the first step of the Politics
tree.** `Hyades_politics_trade_and_intelligence.md` §1 — the Exchange is the
substrate the whole trade system is built on, and it is already written,
deterministic and `HashMap`-free. Wiring it in is the prerequisite for T-38.

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

Was believed saturated — "tuned to an old ~25 ly extent, the galaxy is now
hundreds of ly." **The measurement says otherwise, and in the opposite
direction.** A partial sweep (before it was aborted for T-43's reason) read
25 → 36.1%, 75 → 37.4%, against the shipped 150 giving 35.3% at the same
configuration. So the term is *not* dead, it is an active lever, and the
optimum is **below** the shipped value rather than above it. Finish the sweep
and ratify; the premise recorded here was backwards.

### T-43. The cost ladder can no longer be swept one leg at a time

**A search-harness correctness bug, found by the search reporting a result that
was an artifact.** Since R-O58 the cost ladder *is* the capacity ladder: hull
radius is `sqrt(cost / cost_Limited)` and capacity is `(r − 1)³` normalised to
the Medium hull. So `medium_fleet_size` is not a price — it also sets how much
bigger a Medium hull is than a Limited one, and therefore the whole contents
ladder.

Sweeping it alone over `[3, 4, 6, 8, 12]` reported **8.0 as optimal (25.1% vs
15.2% at the default)** and read 12.0's collapse to 0.3% as an economic cliff.
Neither reading survives: at 8.0 against `limited_fleet_size = 9` a General hull
holds **~36,000×** a Medium's load, and at 12.0 the Medium hull is *smaller*
than the Limited one, the normaliser is zero and **every hull carries nothing.**
The "cliff" was the normaliser, not the economy.

It stayed invisible because the freighter is an MSV and the Medium hull is the
normalisation unit, so `cap_M` is pinned at `cargo_unit_size` and the haul per
trip never moves. **A nonsense ladder that does not perturb the objective is the
worst kind**, and it is why `SimConfig::hull_ladder_fault` is now a hard check
that `Simulation::new` panics on rather than a comment.

**The guard immediately found two more contaminated measurements**, both of
which were in CI and passing:

- `coverage_time`'s "cheaper colonizers" comparison ran `medium_fleet_size = 6.0`
  (General hull ~700× a Medium's load). Now 4.0, and the comparison it exists to
  make is unaffected: 1,216 / 1,331 against the baseline's 1,044 / 1,093.
- `coverage_trace` swept `[3.0, 8.0, 12.0]` and reported this knob as "the one
  that DID move it" — with two of its three points degenerate and 12.0 zeroing
  every hull's capacity. Now `[2.5, 3.0, 4.0]`; the *direction* of that verdict
  survives (106 → 117 → 129 colonies, monotone), the magnitude does not.

Three contaminated measurements from one coupling is the argument for the guard
being a panic rather than a lint.

> **Retracted — this was a fourth artifact of the same root cause, and the
> escalation that came with it was wrong.** Capacity used to be normalised
> against the *live* Medium radius, which pinned `cap_Medium` at
> `cargo_unit_size` for **every** cost ladder. So raising `medium_fleet_size`
> made colonizers cheaper while the freighter's haul never shrank — a free
> lunch, and the reason the objective ran monotonically to whatever upper bound
> it was given. There was no tension between coverage and the shell model;
> there was a bug in the denominator.
>
> | `medium_fleet_size` | 2.0 | 3.0 | 4.0 | 5.0 | 6.0 | 8.0 |
> |---|---|---|---|---|---|---|
> | `cap_M`, old (live normaliser) | 5.00 | 5.00 | 5.00 | 5.00 | 5.00 | 5.00 |
> | `cap_M`, fixed reference | 17.97 | 5.00 | 1.59 | 0.51 | 0.14 | 0.003 |
>
> Normalising against a **constant** reference radius (√3, the Medium hull at
> the reference ladder) fixes it: nothing diverges, narrow ladders are legal and
> meaningful, and the soft `r_M < 1.25` guard is deleted. Only the *inverted*
> ladder is still refused, and for a naming reason rather than a physical one —
> a "Medium" hull cheaper than a "Limited" one is smaller than it.
>
> Re-measured under the corrected model, `medium_fleet_size` is still the
> largest single elasticity (+32.7 ± 3.6 points per ln), so cheaper colonizers
> genuinely do win — but now they win *against* a shrinking freighter hold,
> which is a real tradeoff rather than a free one.

**The original (contaminated) sweep, kept for the record.** Over the range then
considered valid the objective was *monotone to the upper bound* every time:

| `medium_fleet_size` | 2.0 | 2.5 | 3.0 (shipped) | 4.0 | 5.0 |
|---|---|---|---|---|---|
| mean coverage | 26.3% | 33.8% | 38.7% | 43.8% | **45.3%** |

**Replicated on the 4-seed bed**: 25.1 / 32.3 / 38.3 / 42.7 / **45.4%**, against
the ten-seed run's 45.3% at the same point. Agreement to 0.1 points on this axis
says the seed trim (commit `ec5ee23`) cost nothing here, which is worth knowing
before trusting it elsewhere.

5.0 was simply the largest value the old guard permitted. The monotonicity was
the free lunch above, not a statement about hull pricing.

What remains: sweep `(medium_fleet_size, limited_fleet_size)` **jointly**, or
better, sweep the *radius* ladder directly and derive cost from it — radius is
the primitive under R-O58 and the cost ladder is the derived thing, so the
search is currently optimising the wrong parameterisation.

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

**Field list supplied**, which was the whole of the blocker:
`Hyades_politics_trade_and_intelligence.md` §7 specifies a `Diplomacy` struct —
per-mineral `demand`, `trade_budget`, `denial_premium`, `escrow_ratio`,
`excluded`, and the two disclosure willingnesses. What remains is implementing
it, and it should land with T-38 rather than alone, since every field is a term
in a pricing formula that does not exist yet.

Note what §7 deliberately **omits**: no `ally`, no `pact`, no `treaty`. A
bilateral agreement object would be exactly the confederate the anti-collusion
thesis is trying to make unnecessary, and it would need a consent handshake —
a second inbound channel across the seam (design law #15). Blocs are emergent,
not stored.

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

**Round layer landed; simultaneity has not.** `EventKind::RoundBoundary` is a
scheduled event that chains its own successor (not a tick sweep, not a wall
clock — net §1.1), `Simulation::apply_orders` is the **single inbound channel**
required by design law #15, orders apply in seat-index order (net §5 P2), and
illegal orders coerce to `pass` rather than being rejected (net §5.1).
Cadence is `years_to_first_round` (200) and `years_per_round` (400), both
MC/playtest surfaces (R-P12).

**Behaviour-neutral**, which is the property that matters: `choose_card`
defaults to `None`, so seed 1 / 3 seats / 4 kyr still gives 1,044 colonies and
every coverage number in the tree — and the offline search resting on them —
stays valid. Pinned as a test.

**What remains:** the *hidden simultaneous* half. Today orders are collected
from autopilots at the barrier and applied immediately; there is no commit
phase, no reveal phase, no salt, and no timeout vote. That is the meso layer —
the yomi channel the whole competitive frame rests on — and it is carried as
**T-42**. Also missing: sim §3's income/aftermath phases as distinct beats.

### T-31. A card system — netcode B2

**Placeholder tier-0 layer landed** (`src/cards.rs`): the 18-card grid (3 slants
× 6 trees, std §1), `CardId` as its own index so the wire protocol's `card_id`
needs no lookup table, the closed `Target` set, the coercion rule, and three
effect families that write live engine state — `DiscloseScans`,
`WriteDoctrine`, `UnlockDesign`. Warfare's three are
`CardEffect::NotYetImplemented`, *counted* rather than silently inert, because
`sim` never calls `combat::resolve_engagement`.

**Placeholder means the flavour, the costs, and the slot assignments** — card
text is the author's own and none is written. See T-41.

Still blocked behind it: R-C1 (the closed list of legal `target_rule` kinds) and
therefore R-NET4's field widths. Tiers 1+ and the reach economy do not exist.

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

### T-42. Hidden simultaneous orders — commit/reveal in the engine

The other half of T-30, and **the meso layer the competitive frame rests on**.
Today the barrier collects orders and applies them in the same instant; there is
no commit beat, so there is nothing hidden and no yomi.

Needed: a commit phase holding a salted hash per seat, a reveal phase gated on
holding every live seat's commit (net §5 — a *log-derived* condition, never a
timer, which is what makes simultaneity fair regardless of latency), and the
timeout-vote path. Note net §5.2's finding that at 18 seats a timeout fires in
~30% of rounds, so it is a routine transition to present well, not an error
path.

The engine can carry the phases without any network: commit/reveal is a
*simulation* structure that the protocol then mirrors.

### T-38. The Exchange, `$`, and the trade economy

`Hyades_politics_trade_and_intelligence.md` §§2–4. Cross-empire order books on
top of `matching.rs` (T-01), `$` as a non-mass claim (R-P1), willingness-to-pay
derived from Doctrine demand × shortfall × counterparty risk, escrow with
settlement on delivery, and the **transit burn** that makes the travel-time
discount and the `$` sink the same mechanism.

The verb set is specified (§4). The magnitudes are not: `λ`, the income rate and
the Politics depth multiplier are all R-P2, and all three are MC surfaces.

Reputation ships in **both** forms (R-P4, resolved): public consensus by
default, per-observer for anyone who buys the **Audit** card. That is worth
building in this order — public first, because per-observer is the same
per-pair storage T-33 flags as expensive, and making it opt-in is what bounds
the cost to the players who chose to pay it.

### T-39. Shared intelligence, tiers 1–3

Tier 0 (planetary scan data) has landed as a card effect. Tiers 1–3 — movement,
Doctrine, Design — need the observation storage T-33 describes, because you
cannot *sell* an observation the engine does not keep. Ordering is by the
standing layer's own leak asymmetry: Design never goes stale, so it is the most
valuable and the most damaging to have published (politics §5.1).

**Disclose (other) is the interesting one** — transparency as an attack, and the
mechanism that makes "my ally told me their Design" worth nothing.

### T-40. The trade ↔ mobilization counter-graph coupling

`Hyades_politics_trade_and_intelligence.md` §8. Neither direction may be a
modifier; both must fall out of simulation state:

- **Early trade blunts later mobilization** through the supply chain — mobilizing
  against your supplier cuts the minerals the mobilization is made of.
- **Early mobilization blunts later trade** through the risk premium — an armed
  fleet is visible (std §6.2) and visibly armed players get worse prices.

Both accumulate, which is what makes the counter-graph edge *time-dependent*.
Needs T-38 for the price side and combat-in-sim for the mobilization side.
R-P9 asks whether the risk premium is bounded; if it is not, early arming could
become unplayable.

### T-15. Production-queue redesign — roles §10, deferred

Expand whenever affordable, rather than through a competing bias dial. Partly
overtaken by R-O29: the `BuildOrder` match no longer names missions and role
assignment has moved into `Autopilot::assign_role`, so production choice can
become "what does my current Role's System say to build". The dial
(`expand_bias`) survives and is what this would replace.

---

## Band C — open question with a concrete test


### T-58. Band arithmetic removed from the type, and the two bugs it was hiding

**`Band` has no `Add`, `Sub`, `AddAssign`, `SubAssign` or `Mul`.** `I + II +
III + IV` is not `X`: Band numerals are positions on a multiplicative ladder, so
summing or differencing them yields a number with no meaning. The gap between
two Bands is a **mass**, and `Kilotons` is what that is for.

- To move along the ladder: **`Band::up(rungs)`**, documented as *multiplying*
  the mass rather than adding to it.
- To ask how much more stuff: **`Measure::gap_to(other) -> Kilotons`**, which
  names the unit in its return type.

Only four sites in the engine used Band arithmetic, and each was worth the look.
Two were real bugs; two were *scores* — `hub_value` (a `k_potential` reading
discounted by distance, compared against another reading) and a `k_high` sweep —
where scaling a log-scale reading is legitimate but was worth spelling out.

**Bug 1: `founding_infra` summed Band numerals.** R-O76 computed a hull's
infrastructure budget as `b(b+1)/2` — the cumulative `1+2+3+4 = 10` of the
linear price ladder — and concluded a General hull founds at `Band IV`. Redone
in kilotons, ten times the hull mass is `log_F(10) = 0.67` of a rung at
`F = 31.62`:

| hull | cost | infra mass | founding infra |
|---|---|---|---|
| Limited | 0.02 | 0.2 kt | `Band 0.53` |
| Medium | 0.10 | 1.0 kt | **`Band I`** (the anchor) |
| General | 1.00 | 10.0 kt | **`Band 1.67`**, not `Band IV` |

A General colonizer is now **`K`-limited rather than hold-limited**: its
`Band II` hold is capped by the `Band 1.67` of infrastructure its own hull left
behind. Same conclusion as stage 4c — the extra hold is unusable — reached by
derivation instead of by a cap. R-V9 still holds, but through the *hold*: a
Limited hull's seed capacity is below `colony_seed_pop` whatever infrastructure
it would leave.

**Bug 2, recorded not fixed — R-O80: the infra price is Band-additive.**
`round(infra) + 1` minerals per rung is a linear count, and it is now named
(`infra_step_price`) rather than inlined at two call sites, because naming it is
what makes it visible. Priced correctly as `KT(b+1) − KT(b)` the `I → II` step
costs **≈31 minerals against the 2 charged today**, and `II → III` about 2,800 —
an economy in which nothing ever deepens. Either infrastructure anchors its own
`Band I` far below population's (§2.6 permits exactly that) or it is not a
Band-laddered quantity at all and `Band` is the wrong type for it. A design call
with a large measurement attached; the shipped ladder stands until it is made.

### T-59. Mining crews and outpost ore are per player

**Directed this conversation.** `mine_crew` is keyed `(player, outpost)` and ore
lives in `outpost_stock: BTreeMap<(u32, u64), Minerals>`.

The rock's `stockpile` component was one pile per *planet*, and outposts are
never claimed — `claim_planet` is for colonies — so two empires working the same
body filled and drew from the same heap: either could haul away what the other's
miners dug. `examples/crew_census` found this is not hypothetical, it is the
norm: **53.7% of worked rocks carried more than one crew on seed 1, and every
one of the 1,185 was cross-player.**

**The body is shared; the pile is not.** `density` stays per-planet because a
rock is one physical object and every crew on it depletes the same ore — a
contested field runs out sooner for everybody, which is the mechanic. What each
empire *lifts* is its own. Sharing a pile is a **card**, not the default.

Measured on seeds 1 and 7: 8,795,729 and 9,124,999 against 8,762,145 and
9,131,535 — +0.38% and −0.07%, mixed sign and small, which is what a fairness
correction with no systematic direction should look like.

### T-57. How many miners per mining outpost? — **the term now exists**

**Implemented this conversation.** `mine_operator: BTreeMap<u64, Entity>` is now
`mine_crew: BTreeMap<u64, Vec<Entity>>`, extraction is **per miner** rather than
per rock, and `Doctrine::miners_per_outpost` (default 1) is the knob:

```rust
let crew = self.mine_crew.get(&outpost.0).map_or(1, |c| c.len().max(1));
let amt = density.metallicity() * (crew as f64 * cfg.outpost_mining_fraction).min(1.0);
```

`outpost_mining_fraction` changes meaning from "the fraction of remaining
density a *rock* yields per tick" to "the fraction **one miner** works", capped
at the whole field because a rock cannot yield more than it holds. A rock is a
finite stock, so a bigger crew does not raise what a field yields in total — it
brings that total **forward**, which is what the expansion loop is short of
(`CLAUDE.md` §7: the residual is worlds scanned and not reached in time).

#### Crew 1 is not bit-identical, and the cause is **demonstrated**

The arithmetic at `crew = 1` is the old expression exactly, so this should have
reproduced the bed. It does not:

| seed | before | after | |
|---|---|---|---|
| 1 | 8,481,134.1 | 8,485,264.2 | +0.05% |
| 7 | 8,717,150.7 | 8,761,142.3 | **+0.50%** |
| 42 | 8,690,707.4 | 8,736,414.2 | +0.53% |
| 31337 | 8,791,088.4 | 8,809,169.8 | +0.21% |

**A pre-existing bug is being fixed, and `examples/crew_census` counts it rather
than arguing it.** `mine_operator.insert(outpost.0, vehicle)` *overwrote*, so a
second miner reaching an already-worked rock replaced the first in the map: its
extraction stopped being counted, and on exhaustion `remove()` returned only the
last hull — the earlier one was **leaked**, still `Role::Miner`, parked forever,
never re-tasked and never scrapped. `mine_crew` pushes, so both count and both
are released.

The census says this is not an edge case:

| seed | outposts worked | with >1 miner | max crew | cross-player |
|---|---|---|---|---|
| 1 | 2,205 | **1,185 (53.7%)** | 3 | **1,185 of 1,185** |
| 7 | 2,188 | **1,132 (51.7%)** | 3 | **1,132 of 1,132** |

Over half of all worked rocks had more than one miner on them, and **every
single one of those is cross-player** — not one is the same empire
double-working a rock, which is what `targeted` exists to prevent and evidently
does. Rival empires land on the same rock constantly, because `mine_crew` and
the outpost's stockpile are keyed by **outpost alone** while `targeted` and
`exploited` are per **player**.

**R-O79 (new, open): a rock is shared, and T-57 makes that sharing matter
more.** Two empires' miners now jointly accelerate a stockpile they both draw
from — `sys_freighter_arrive` loads from `sh.outpost`'s single stockpile,
whoever arrives. That was already the model; per-miner extraction turns it from
a curiosity into a rate. Whether extraction and the stockpile should be
per-player is a design question with a real answer either way (a contested rock
is a legitimate mechanic), and it is not settled by making the crew a `Vec`.

#### The sweep — **ratified at 3**

Standard four-seed CRN bed, 4,000 yr, read against the **new** crew-1 baseline
(8,697,997.6), not the pre-T-57 figure. Every seed positive at every crew size:

| crew | colony-years | vs 1 | per extra miner | doubling |
|---|---|---|---|---|
| 1 | 8,697,998 | — | — | 270.3 yr |
| 2 | 8,877,142 | +2.06% | +2.06% | 262.4 yr |
| **3** | **8,936,603** | **+2.74%** | +1.37% | **260.7 yr** |
| 5 | 8,965,467 | +3.08% | +0.77% | 264.2 yr |

**3, not 5, and the doubling column is the reason.** Five miners buy 0.34 more
points of colony-years and give back 3.5 years of doubling time: the extra hulls
compete for the same build slots the expansion loop needs, so past three the
crew starts costing what it is meant to buy. The marginal return per miner
halves at 3 and halves again at 5 — a rock is a finite stock, so a crew cannot
raise what a field yields, only bring it forward, and there is a limited amount
of forward to be had.

Colony *count* is identical (3,481.0) at every crew size, which is the saturated
bed doing what `CLAUDE.md` §7 says it does: on a bed taking ~99% of what
`k_high` admits, *when* is the only thing left to measure.

**Throughput is not measured here, deliberately.** Three miners per outpost adds
roughly 4,400 hulls on ~2,200 rocks, and vehicle count is T-24's first-order
cost — but this container returned 218/208 and then 107/98 yr/s for *identical
code* earlier in the same session, so any A/B taken on it would be measuring
load, not the change. It needs a same-machine run, which is the same outstanding
job T-24 already carries for the whole throughput table.

#### The original entry, kept because the diagnosis was the useful part

**Extraction does not depend on the miner at all.** `sys_mining_tick` is

```rust
let amt = density.metallicity() * cfg.outpost_mining_fraction;   // per mining_tick_years
```

— a property of the *rock*, not of who is standing on it. `mine_operator` is a
`BTreeMap<u64, Entity>`, so an outpost has exactly one operator by data
structure, and that operator's only mechanical effect is to keep the tick
scheduled and to be released to Reserve when the rock dies (R-O67 recycling).
A second miner on the same body would extract nothing extra; a *better* miner
would extract nothing extra either.

So "the right number of miners per outpost" is not a value to tune — **there is
no term in the model for it to tune.** That is the finding, and it is the same
shape as λ before its ratification (`CLAUDE.md` §7: freighter routing had *no*
distance component, so no amount of sweeping would have found one). Before
sweeping, check whether the term is absent.

**What a real answer needs, roughly in order:**

1. **Decide whether extraction is rock-limited or labour-limited.** Today it is
   purely rock-limited (`density × fraction`). A labour term —
   `min(rock_rate, miners × per_miner_rate)` — is the smallest change that makes
   the count matter, and it immediately raises the design question of whether a
   rich rock should reward concentration. Design law #3 says consolidation wins
   under geometry; it is not obvious that mining should agree.
2. **Decide what a miner's hull buys.** `role_hull_type` puts Miner on LSV
   because "the engine already deposits extraction into the outpost's own
   stockpile, so a Limited miner needs no cargo" (roles §4.3). If extraction
   becomes labour-limited, hull size plausibly enters — and then this is not an
   independent question from T-56's cost ladder, because miners are the most
   numerous vehicle class in the run.
3. **Then measure.** The instrument exists: `examples/mining_probe -- census`
   already reports outposts, mean rock lifetime (808 yr) and the fraction of
   outpost-years spent on an exhausted rock (39% before recycling). Extend it
   with a miners-per-outpost sweep once there is something for the sweep to move.

**Why it matters beyond mining.** Miner + freighter pairs dominate vehicle
count, so this is simultaneously a throughput question (T-24) and an economy
question. And `outpost_mining_fraction` is MC-ratified at 0.238 with a
+14.5 ± 5.8 elasticity — the weakest of the gradient step's four knobs — so any
change to what that fraction *means* consumes that ratification and needs a
joint re-measure, not an addition.

**Do not treat the current one-miner-per-outpost as a ratified answer.** It is
a data-structure consequence that nobody has measured against an alternative.

### T-56. R-MC3a / R-MC3b — hull geometry with units, and the ladder that makes General hulls worth building

**Stage 1 of 4: analysis and ratification candidates. No code in this entry.**

#### "What unit?" — the answer, and it is not flattering

`hull_radius(h) = √(cost_fraction(h) / cost_fraction(Limited))`, and
`cargo_capacity` subtracts **1** from it. That `1` is `r_L`. So the shell
thickness is **one Limited-hull radius, identical for every class**, and
"the Limited hull is all shell and no hold" is a *definition* rather than a
result. There is no thickness parameter, and:

- **η (R-MC3a, shape efficiency) appears nowhere in the engine.** §2.1 builds a
  whole cylinder→ellipsoid→spheroid argument for super-linear value growth and
  the code implements none of it.
- **`r_eq` (R-MC3b) is not a parameter either** — it is back-derived from cost.

Both open R-codes are simply unrepresented, which is why neither could be
ratified against the code.

#### Corrected geometry — exact, and one dial falls out

Per class: `r` (equal-volume radius), **`φ` = hold fraction = (r−t)/r**, `η`
(shape, R-MC3a).

```
V_total = (4/3)πr³      V_hold = (4/3)π(φr)³      V_shell = (4/3)πr³(1−φ³)
cost = dry mass ∝ V_shell / η          capacity ∝ (φr)³
        ⇒   cost = capacity · (1−φ³)/(φ³η)          — r cancels entirely
```

Define **carry efficiency `E(φ,η) = capacity/cost = φ³η/(1−φ³)`**. Then:

- **Design law #3 holds ⟺ `E` rises with class ⟺ `φ` rises with class.** Size
  does not enter. **`φ` is the entire economic dial**, which is exactly the
  "hull thickness per class" lever this was asked to find.
- **R-MC3a (η) and φ set the ladders; R-MC3b (`r_eq`) sets only absolute
  size.** That separation is new and it makes the two R-codes independently
  ratifiable — `r_eq` matters for combat cross-section and the arena harness,
  not for the cost/cargo ladders at all.

#### Candidate ladder

`F_pop = F_cargo = 100`, `F_cost = 8`, town 3,200 at 300 kg/person
(`KT(I) = 0.96 kt`), **Pop Band IV = 3.2 billion** — inside the 1–10 B target,
with `F₃ = F = 100`, nowhere near four orders of magnitude.

Each cargo Band up multiplies carry efficiency by `F/F_cost = 12.5`, anchored on
today's Medium (`E = hold/dry = 4.27`):

| class | cargo Band | cost Band | `E` | `φ` | `t/r` | hold | dry mass |
|---|---|---|---|---|---|---|---|
| Limited | Empty | — | 0 | 0 | 1.0 | 0 kt | — |
| Medium | **I** | I | 4.27 | 0.9412 | 0.0588 | 0.96 kt | 0.225 kt |
| General (early) | **II** | II | 53.3 | 0.9940 | 0.0060 | 96 kt | 1.80 kt |
| General (mid) | **III** | III | 667 | 0.9995 | 0.00048 | 9,600 kt | 14.4 kt |
| General (late, card) | **IV** | IV | 8,332 | 0.99996 | 0.00004 | 960,000 kt | 115 kt |

**The design rule this produces is legible: each Band of hull is roughly an
order of magnitude thinner-skinned than the last.**

#### The Sleeper Service check

Canon: a GSV carrying tens of thousands of LOU/ROU-class hulls in its hold.
By dry mass, this ladder gives a General (III) a hold of **42,662 Medium
hulls** — General (II) holds 427, General (IV) holds 4.3 million. The
mid-game General lands precisely on "tens of thousands", and it was not tuned
to; it fell out of `F = 100` with cost tracking cargo Band-for-Band.

#### The binding constraint, stated so it can be argued with

`E` is bounded by `φ < 1`, so **how far cargo can outrun cost per Band is set by
the thinnest shell you will accept**:

| min `t/r` | max `E` | vs Medium | max `F` at `F_cost = 8` |
|---|---|---|---|
| 0.05 | 5.8 | 1.4x | 9.4 |
| 0.01 | 31.7 | 7.4x | 21.8 |
| **0.001** | **323** | **75.6x** | **69.6** |
| 0.0001 | 3,233 | 758x | 220 |

`F = 100` with cost tracking cargo Band-for-Band needs `t/r ≈ 5×10⁻⁴` by Band
III. That is a very thin skin — and it is what the fiction already asserts about
a GSV, which is mostly volume and field.

#### Consequences to expect, and the falsification test

Raising General cost while raising its hold much faster should make **General
colony ships the correct play**, accelerating the colony doubling rate and
raising total colony-years. **If increasing General hull cost degrades those
metrics, the ladder is wrong** — that is the stated acceptance test, and
`examples/colony_years` is the instrument. Note the autopilot will not use
General hulls at all until Doctrine is taught to (`role_hull_type` pins
Colonizer to Medium today), so the ladder change and the Doctrine change must
be measured *separately* or the result is uninterpretable.

#### Stage 1b — `V_reserved`, the role axis, and two corrections to Stage 1

**Correction 1: Stage 1 used superseded `η`.** It took §2.1's *illustrative*
0.73 / 0.85 / 0.97. §2.2 supersedes those with a **ratified `η(role, size)`
table**, and the anchor is stronger than the number I used: **a General Systems
Vehicle is a literal sphere, `η = 1.000` exactly.**

| role | General | Medium/Rapid | Limited |
|---|---|---|---|
| Systems | **1.000** | 0.98 | 0.86 |
| Contact | 0.96 | *(no Medium tier)* | 0.75 |
| Offensive | 0.93 | 0.73 (Gangster anchor, real data) | 0.64 |

**Correction 2: `φ` needs both axes, not just role.** With `cost = V(1−φ³)/η`
and `cargo = Vφ³ − V_res`, a `φ` fixed within a role makes both quantities `∝ V`,
so **the cost ratio is forced to equal the cargo ratio.** Measured on the
role-only draft: the Systems GSV/MSV cost ratio came out **82.8** against
`F_cost ≤ 8`. So `φ(role, size)` — the same two-axis shape §2.2 already gives
`η`. Role sets the *level* (Offensive thick, Systems thin, Contact between);
size sets the *slope* within it.

**`V_reserved` needs a second term, which §2.3 does not have.** §2.3 defines it
as "engines, structure, crew — the non-cargo baseline every hull needs
**regardless of size**", i.e. purely absolute. That is right for the core and
wrong for the role: a General Offensive amortises a fixed core and comes out
**cargo-rich — 33.2 against a GSV's 133.4, 25%, not "little to zero"**. The fix
is that a warship's reserved volume *is* weapons, armour and magazines, and
those scale with the hull:

```
V_reserved(role, V) = a_role + b_role · V
```

| role | `a` (V_LSV) | `b` (of V) | why |
|---|---|---|---|
| Systems | 0.25 | **0.00** | cargo *is* the payload |
| Contact | 0.40 | 0.35 | sensor/probe fit — between |
| Offensive | 0.55 | **0.92** | weapons + armour + magazines scale with the hull, so cargo stays ~0 at every size |

#### The candidate ladder, all three axes

Systems row solved for `F_cargo = 100`, `F_cost = 8`. Volumes in `V_LSV`;
`r_eq` (∛V): LSV 1.000, MSV 1.239, **GSV 5.15**.

| hull | role | size | `φ` | `t/r` | `η` | cargo | cost | `E = cargo/cost` |
|---|---|---|---|---|---|---|---|---|
| LimitedSystems | Systems | Limited | 0.900 | 0.100 | 0.86 | **0.479** | 0.315 | 1.52 |
| MediumSystems | Systems | Medium | 0.9412 | 0.059 | 0.98 | 1.334 | 0.322 | 4.14 |
| GeneralSystems | Systems | General | 0.9937 | 0.0063 | **1.000** | 133.4 | 2.578 | 51.7 |
| LimitedContactVehicle | Contact | Limited | 0.820 | 0.180 | 0.75 | 0.000 | 0.598 | 0 |
| GeneralContactVehicle | Contact | General | 0.985 | 0.015 | 0.96 | 82.1 | 6.291 | 13.1 |
| LimitedOffensive | Offensive | Limited | 0.600 | 0.400 | 0.64 | **0.000** | 1.225 | 0 |
| RapidOffensive | Offensive | Rapid | 0.620 | 0.380 | 0.73 | **0.000** | 1.982 | 0 |
| GeneralOffensive | Offensive | General | 0.900 | 0.100 | 0.93 | **0.000** | 39.70 | 0 |

**Offensive cargo is 0.000 at every size by mechanism, not by a floor** — the
`b` term consumes the interior. That is what "little to zero cargo" asked for,
and it now falls out rather than being clamped.

**`LimitedSystems` carries 0.479 rather than zero** — the requested "Limited
costs Band I but you get more per Band-I cost". Note this **may retire R-V9's
"a Colonizer must be Medium or larger"**, which rested on Limited having
literally no hold; it now depends on whether 0.479 clears the colony seed.

#### Two independent validations

- **Design law #2's window.** GOU cost 39.70 against ROU 1.982 is **20.0x**, so
  a cost-parity fleet is 20 ROUs to one GOU — inside the ratified 6–45 target,
  and not tuned to.
- **Design law #3.** Systems carry-efficiency rises 1.52 → 4.14 → 51.7 with
  size, monotonically, so consolidation wins by construction rather than by
  assertion.

#### What this opens

- **R-MC12 gets an answer shape.** Cross-role sizing is no longer "same size,
  less efficient" vs "smaller": role now differs in `φ` and `V_reserved` at the
  *same* `r_eq`, which is a third lane and the one this ladder takes.
- **The `b` term is new** and extends §2.3 rather than restating it; it needs
  ratifying alongside `a`.
- **R-MC11** (shape as a function of loadout, not a per-`HullType` lookup) is
  the natural successor: `φ` and `b` are exactly "how much of the interior is
  role payload", which *is* slot composition.


#### Stage 1c — `V_reserved` per size **and** hull, the cost anchoring, and R-MC15

**`docs/Hyades_mineral_cost_curve.md` §2.3 and §2.6 are now written** (this
stage is that doc change). Stage 1b's candidate table above is **superseded**
by it: 1b measured volumes in `V_LSV` and left cost free, which let the Limited
Systems hull keep a 0.479 hold — 36% of a Medium's, not the "tiny fraction of
`Band I`" the ladder is supposed to produce. Stage 1c re-anchors on the
directed rule that hull cost is `general_vehicle_cost` divided by the existing
fleet-size knobs, which removes the freedom and pins the row.

**What changed from 1b, and why each move was forced:**

| | Stage 1b | Stage 1c | forced by |
|---|---|---|---|
| dial | `φ` (hold *fraction*) | **`τ` (absolute shell thickness)**, `φ = 1 − τ/r` | `τ` is what "hull thickness per size/class" names, and it makes `cost ∝ area` the constant-`τ` special case rather than a separate claim |
| cost | free per hull | `1` / `1/medium_fleet_size` / `1/limited_fleet_size` | directed this conversation — no new cost field |
| `a_role` | 0.25 / 0.40 / 0.55 (in `V_LSV`) | 0.09 / 0.15 / 0.22 (in `V = r³`) | unit change, then re-solved against the 1%-of-`Band I` target |
| `b_role` | 0.00 / 0.35 / 0.92 | 0.00 / 0.15 / **0.45** | 0.92 drove Offensive volume negative once cost was pinned; 0.45 still zeroes LOU and ROU |
| LSV cargo | 0.479 (36% of Medium) | **0.0096 kt (1.0% of `Band I`)** | the stated expectation |
| `limited_fleet_size` | free | **32** | converged: 32.31 from "share the Medium's thickness" + "carry 1% of `Band I`" |

**The Sleeper Service check survives the re-anchoring.** Dry mass is the
mineral cost (L6/R-O57), so a mid-game General at cost `Band III` masses 8
units and holds cargo `Band III` = 9,600 kt against a Medium hull's dry mass of
0.125 — **76,800 Medium hulls in the hold**, still "tens of thousands", and
again not tuned to.

**Two things this stage does *not* settle**, both recorded rather than guessed:

- **The absolute scale.** With cost ≡ dry mass, one mineral unit is a mass, and
  the ladder above makes an LOU mass 0.03125 of it. Whether that is 31 tonnes
  or 31 megatonnes is R-O72 / T-54 and is untouched here. Every *ratio* in
  §2.3 is independent of it.
- **R-V9** ("a Colonizer must be Medium or larger"). Stage 1b thought a 0.479
  hold might retire it; at 0.0096 kt a Limited Systems hull plainly cannot seed
  a colony, so R-V9 now looks like a *consequence* of the ladder rather than a
  separate rule. Confirm in Stage 3 rather than asserting it here.

**R-MC15 is the gate on Stage 2** (directed this conversation). The candidate
is `F₁ = 4`, `F₂ = 8` on the cost ladder — `medium_fleet_size = 8`,
`limited_fleet_size = 32`, `cargo_unit_size = 0.96`, `units::BAND_STEP = 100` —
with the reasoning in §2.6. Two of those four are globally MC-tuned and none of
them moves before ratification.


#### Stage 1d — R-MC15 **ratified**, and what it superseded

Ratified this conversation, and it changes three things Stage 1c had guessed at.

- **There is no `Band 0`. The bottom rung is `Band Empty`, and `Band Empty > 0`.**
  Renamed throughout `docs/` and in the `BandTier` rustdoc. The rung is a
  positive magnitude beneath `Band I`, not an absence — a quantity that is
  genuinely zero is *off* the ladder, not at its bottom.
- **The `[4, 8]` window on step factors is superseded and removed**, and with it
  the "`F₃` is unconstrained" release valve. The constraint is now on how the
  factors *grow*: `1 < F₍ₙ₊₁₎/Fₙ < 10`, on both ladders.
- **`F_mass = F_cost^(3/2)` is ratified as an identity**, so the two ladders are
  one geometry. The mass ladder's growth rule is the binding one, since a cost
  step-ratio of `x` is a mass step-ratio of `x^1.5` — cost ratios must stay under
  `10^(2/3) = 4.64`.

**Stage 1c's `limited_fleet_size` argument is withdrawn.** It pinned the value
by requiring the Limited and Medium hulls to share a shell thickness. That
requirement is explicitly *not* wanted — thickness should rise
`Limited < Medium < General` across the arc — so the convergence at 32.31 was
an artefact of a constraint that does not exist. `limited_fleet_size` is now
fixed by the ladder instead: it is the `Empty → I` cost step times
`medium_fleet_size`.

**The ratified default progression** (uniform step-ratio 2, subject to
Monte-Carlo verification that it creates no colony-years bottleneck):

| step | `F_cost` | `F_mass` | rung | cost | hold (kt) | population |
|---|---|---|---|---|---|---|
| — | — | — | `Empty` | 0.02 | 0.086 | 286 |
| `Empty → I` | 5 | 11.18 | `I` | 0.10 | 0.96 | 3,200 |
| `I → II` | 10 | 31.62 | `II` | 1.00 | 30.4 | 101,200 |
| `II → III` | 20 | 89.44 | `III` | 20.0 | 2,715 | 9.05 M |
| `III → IV` | 40 | 252.98 | `IV` | 800 | 686,900 | **2.29 B** |

⇒ `medium_fleet_size = 10`, `limited_fleet_size = 50`, `cargo_unit_size = 0.96`,
and `units::BAND_STEP` becomes **piecewise** — the ratified progression has a
different factor per rung, so `KT(b) = KT_I · BAND_STEP^(b−1)` can no longer be
a single exponential.

**The best result of the ratification is one nobody asked for.** With cost and
hold both fixed by the rung, shell thickness stops being a dial and becomes an
*output* — and it comes out **`Limited (0.0277) < Medium (0.0325) <
General (0.0339)`**, which is exactly the ordering the design arc wanted. The
mechanism is `η`: a bigger hull is rounder, gets more interior per unit of skin,
and spends the surplus on a thicker skin.

**And it runs out, which is where Supers come in.** Once the GSV is a literal
sphere there is no more shape to spend: a `Band III` General would need
`τ = 0.0342`, no thicker than the `Band II` hull. Holding a `Band III` hold at a
`τ` that *did* keep rising costs 1.5× at `τ = 0.05` and **2.94× at `τ = 0.10`**.
So a mid-game `Band III` General Systems Hull must be a Design requiring
**Supers**, and a `Band IV` one a Design requiring **apex** — the tier reset on
absolute thickness is what keeps the hull at its rung's price. This is now a
geometric consequence rather than a balance decision, and it is the mechanical
content of "Design level resets thickness."

**Sleeper Service check, re-run.** Dry mass is the mineral cost (L6/R-O57), so a
`Band III` General holds 2,715 kt against a Medium hull's dry mass of 0.10 —
**27,150 Medium hulls**, or 135,800 Limited. Still "tens of thousands," and
still not tuned to.

**R-O71 / T-53 is resolved by this**, in an amended form: capacity keeps a
ladder, but the *mass* ladder rather than the cost one, and the `3/2` tie is
what makes that a single geometry rather than two unrelated scales.
`the_cargo_ladder_is_geometric_not_banded` pins the old disagreement and needs
replacing in Stage 3 by a test on the ratified tie.

**Stage 2 is unblocked.**


#### Stage 2 — hull geometry carries its units in the type

**Landed. Behaviour-preserving, verified on colony-years:** seeds 1 and 7
reproduce **7,819,401.0** and **8,480,172.0** bit-for-bit, the same figures
R-O70 was held to.

`src/units.rs` gains `Length`, `Area` and `Volume` alongside `Band` and
`Kilotons`, with only the dimensionally sound operations defined —
`Length::cubed() → Volume`, `Area × Length → Volume`, `Volume ÷ Volume → f64`
(the one sanctioned exit, and the way a hold becomes a load), and no path at
all from a `Volume` to a `Kilotons` without a density. Two `compile_fail`
doctests pin that a length is not a volume and a hold is not a cargo.

`src/sim.rs` grows the three quantities the shell model always had and never
named:

| new | what it is | today |
|---|---|---|
| `HullType::shell_thickness` | **`τ`** — the model's one free geometric input | `1.0` for every hull, which is exactly what `cargo_capacity`'s `(r − 1)` meant |
| `HullType::hold_radius` | `r − τ`, floored at zero | unchanged arithmetic |
| `HullType::hold_volume` | `(r − τ)³` — the quantity that sits on a Band rung | unchanged arithmetic |
| `HullType::shell_volume` | `r³ − (r − τ)³` — the material bought, i.e. cost and dry mass | **not yet on the cost path**; `cost_fraction` is still the fleet-size ratio, and reconciling the two is stage 3 |

`hull_radius` now returns `Length` and `hull_dry_mass` returns `Kilotons`.
`hull_geometry_is_dimensioned_and_the_shell_closes` pins the identities:
shell + hold = the whole hull, the Limited hull is all shell *because* `r = τ`
rather than because a literal cancelled, and the capacity ratio between two
hulls is their hold-volume ratio — which fails the moment a second conversion
creeps in.

**One deliberate tripwire.** The test asserts `shell_thickness == 1.0` for
every hull. §2.3 ratifies `Limited < Medium < General`, so that assertion is
what stage 3 must knowingly change; it is there so the thickness ladder cannot
arrive as a silent side effect of some other edit.

**One float-order note, because it nearly went wrong.** `cargo_capacity` keeps
its expression shape — `k · v / v_ref`, left to right — rather than the more
natural `k · (v / v_ref)`. Multiplication is not associative in floating point
and the two differ in the last bits, which is a determinism break and a changed
golden. The comment says so at the site.


#### Stage 2b — `Band Zero`, and the anchor moved to a round kiloton

Both from the ratification pass; behaviour-preserving, colony-years unchanged
at 7,819,401.0 / 8,480,172.0.

**`BandTier::Zero`.** Ratifying `Band Empty > 0` took away the rung that used
to mean "none of this quantity", so a `> 0` check went back to being a bare
`0.0` — the exact failure this type exists to close. `Zero` is now the bottom
sentinel, the mirror of `V` at the top: nothing in a game reaches it, and its
ladder position is `Band(-1.0)`, deliberately outside `BAND_FLOOR`, **so every
rung above it kept the index it had.** `index()` is now `i8`.

One real hazard that came with it: `PopBands::level` indexed `BandTier::ALL` by
a crossing count, and adding a rung at the *bottom* of `ALL` would have shifted
every world's population level by one, silently. It now indexes `PLAYABLE`,
whose positions are the ladder's by construction, and the test asserts both
arrays' indexing conventions separately so the two cannot be confused again.

**`KT(I) = 1.0` kt.** `0.96` was a fallout of the population anchor (3,200
people × 300 kg), and it made `cargo_unit_size = 0.96`, which is an unfortunate
number for the engine's reference hold. Every *ratio* on both ladders is fixed
by the step factors, so the only freedom left is where `Band I` sits in real
kilotons — and a round kiloton makes the Medium hull's hold exactly 1.000 kt
and `cargo_unit_size` exactly 1.0, which is already the value of
`units::KILOTONS_AT_BAND_I`. The population anchor absorbs it at ~3,333 people,
and it is the anchor that can afford to: its requirement (`Pop IV` in 1–10
billion) is an order of magnitude wide, and `Pop IV` moves 2.29 B → 2.38 B.

Downstream, `a_role` re-solves to 0.079 / 0.132 / 0.194 and the Systems row
becomes:

| hull | cost | hold (kt) | `τ` | cargo (kt) | `E` |
|---|---|---|---|---|---|
| LSV | 0.02 | 0.0894 | 0.0270 | 0.0104 | 0.52 |
| MSV | 0.10 | **1.000** | 0.0317 | 0.921 | 9.21 |
| GSV | 1.00 | 31.62 | 0.0330 | 31.54 | 31.5 |

`τ` still comes out `Limited < Medium < General`, and a `Band III` hold at
`τ = 0.10` still costs **3.02×** its rung, so the Supers gate is unchanged.


#### Stage 3a — the piecewise mass ladder, and why it moved nothing

`units::BAND_STEP` is gone. In its place `MASS_LADDER` carries the ratified
factors — `11.18, 31.62, 89.44, 252.98`, each `F_cost^(3/2)` — and the
Band↔kiloton bridge is piecewise, interpolating log-linearly inside each
segment and extrapolating with the edge factor outside the playable ladder
rather than clamping (a clamp there is mass created or destroyed at the clamp,
which L6 forbids).

**Colony-years is bit-identical: 7,819,401.0 and 8,480,172.0.** A `Band II`
population now masses 31.6 kt where it massed 4.0, and a `Band III` one 2,828
where it massed 64 — a 44× change to the biomass draw that changed nothing at
all. That is a big enough non-effect to need a mechanism rather than a shrug,
and there are two, both checkable:

1. **`bio_max` round-trips through the ladder, so `k_potential` never sees
   it.** `galaxy.rs` generates `biosphere` as a **Band**; `sim.rs` converts it
   with `.in_kilotons()` into both `biomass` and `bio_max`; `Factors::new`
   converts it straight back with `.in_bands()`. `k_potential =
   min(hab, bio_max_band)` therefore reads the *generated Band*, whatever the
   ladder is. The deepening guard — which R-O66 showed is the lever that
   actually moves coverage — is structurally immune to this change.
2. **The biomass draw is slack, and that was ablated, not assumed.**
   `CLAUDE.md` §7 records deleting the draw outright and reproducing 3,294.0
   bit-for-bit. A draw that does not bind cannot be made to bind by scaling it
   when the stock it draws from scales with it.

So the ladder adoption is genuinely free at the shipped operating point, and
the place it *will* bite is a card that moves `bio_max` directly — which is
exactly the case R-O66's unit fix was about. Recorded here rather than
discovered later.

**One hazard closed on the way.** The bridge now has three interior joins, and
a discontinuity at any of them is mass created or destroyed at a rung boundary,
since the growth draw is `KT(after) − KT(before)`.
`the_piecewise_bridge_is_continuous_and_monotone_across_every_join` pins
continuity to 1e-6 at every rung from both sides and monotonicity across
`[-0.5, 5.0]`, including the extrapolated ends.


#### Stage 3b — the cost ladder adopted, and the acceptance test answered

**`medium_fleet_size` 4.45 → 10, `limited_fleet_size` 9.0 → 50,
`cargo_unit_size` 5.0 → 1.0.** Measured on the standard four-seed CRN bed at
4,000 yr with `examples/hull_ladder`, whose baseline leg reproduces the
recorded bed exactly (3,459.8 colonies) before anything is attributed to a
change.

| leg | Medium hold | colony-years | vs shipped | doubling |
|---|---|---|---|---|
| shipped (4.45 / 9.0 / 5.0) | 0.96 kt | 8,011,139 | — | 284.5 yr |
| cost ladder (10 / 50) | 24.07 kt | 8,697,322 | **+8.6%** | 265.0 yr |
| `cargo_unit_size` → 1.0 alone | 0.19 kt | 6,396,459 | −20.2% | 358.3 yr |
| **both — ratified** | 4.81 kt | 8,697,322 | **+8.6%** | **265.0 yr** |
| cost ladder, hold pinned at 0.96 kt | 0.96 kt | 8,622,168 | **+7.6%** | 269.3 yr |

**The acceptance test passes.** T-56 asks that making General hulls relatively
more expensive must not degrade the colony doubling rate or total colony-years.
Going from `1 : 4.45 : 9` to `1 : 10 : 50` raises the General hull's price from
4.45 Medium hulls to 10, and colony-years rise 8.6% with every seed up
(+8.0, +2.9, +13.9, +10.0) while the doubling time falls 19.5 years.

**And the attribution needed an ablation, which refuted the obvious reading.**
`hull_radius` is `sqrt(cost ratio)`, so `medium_fleet_size` is not a price knob
— moving it 4.45 → 10 takes the Medium hold 0.96 → 24.07 kt, 25× bigger, in the
same stroke. The expectation was therefore that the +8.6% was the hold and would
vanish once capacity was held fixed. It did not: with `cargo_unit_size` rescaled
to 0.199 so the Medium hull carries exactly what it carried before, the ladder
still returns **+7.6% and 269.3 yr**. Roughly seven of the eight points are the
price. The prediction was wrong and the run is what said so.

**Row 4 is bit-identical to row 2**, across four seeds and every digit, despite
a five-fold cut in every hold. That is not new — `SimConfig::cargo_unit_size`'s
own doc already records `binding_check`'s finding that 5, 25 and 100 are
bit-identical because `load = cap.min(avail)` (`sys_freighter_arrive`) and an
outpost never accumulates a full hold between visits. What row 4 adds is that
the threshold is still below the ratified operating point, so adopting
`cargo_unit_size = 1.0` costs nothing. Row 3 is the other side of the same
curve: at 0.19 kt the hold binds hard and the economy loses a fifth of its
colony-years.

**Three tests were silently depending on the old defaults**, and all three
failed for reasons unrelated to what they check —
`shell_model_ladders_are_derived_not_tuned`,
`only_an_inverted_hull_ladder_is_refused` and
`constructing_a_sim_on_a_degenerate_ladder_panics` each set one leg of the
ladder and inherited the other. They now pin both, and the "General is
untouched" assertion is stated as an *invariance* (same hull, before and after
narrowing) rather than an absolute kiloton threshold, which had quietly become
a test of `cargo_unit_size`.

**One rule stopped being contradicted.** `the_cargo_ladder_is_geometric_not_banded`
asserted that the colony seed does *not* fit a Medium hold — 1.0 kt of settlers
against 0.959 kt of hull — which made R-V9 ("a Colonizer must be Medium or
larger") unsatisfiable in the engine, unnoticed because capacity gates mineral
loading only. At the ratified ladder the Medium hold is 4.81 kt and the seed
fits. The assertion is inverted and R-V9 is now a consequence of the geometry.

**Still outstanding for stage 3c:** the spec's `cargo_unit_size = 1.0` is not
yet the Medium hull's hold in the engine, because capacity is normalised against
the fixed reference radius `√3`. Making the spec's number the engine's number
needs §2.3's per-`(role, size)` thickness and `η` — which is also what separates
cost from capacity so they stop being the same knob.


#### Stage 3c — the real geometry: cost is the shell, capacity is the hold

**Landed, and it is the change the whole ratification was for.** `hull_radius`
no longer square-roots the cost ratio; it *solves* the shell model:

```
cost · η = r³ − (r − τ)³      ⇒      r = τ/2 + sqrt(12·τ·cost·η − 3τ⁴) / (6τ)
```

`r = sqrt(cost / cost_Limited)` was the **constant-`τ` special case written as
if it were the law**. With a ratified per-hull thickness it stops being true,
and that is exactly what unties the two ladders: cost is the shell volume,
capacity is the hold volume, and `medium_fleet_size` is a price again rather
than a price *and* a hold. Four of the measurement artifacts in `CLAUDE.md` §2
had that coupling in common.

`HullType::geometry` carries the ratified table — `η` per §2.2, `τ` per §2.3,
and the two `V_reserved` terms — written out for all ten hulls so a new variant
cannot inherit a neighbour's shell. `REFERENCE_MEDIUM_RADIUS` and
`UNIT_SHELL_THICKNESS` are **deleted**: there is no normaliser left to put a
derived quantity in a denominator.

**The engine reproduces §2.3 exactly** (`examples/cargo_units`):

| hull | dry mass | `τ` | hold | `V_res` | cargo | cargo/dry |
|---|---|---|---|---|---|---|
| Limited | 0.0200 | 0.02700 | 0.0894 | 0.0790 | 0.0104 | 0.52 |
| Medium | 0.1000 | 0.03165 | **1.0000** | 0.0790 | 0.9210 | 9.21 |
| General | 1.0000 | 0.03299 | **31.6228** | 0.0790 | 31.5438 | 31.54 |

with the hold steps landing on `F_mass` to **+0.00% and −0.00%**. Usable cargo
does *not* walk the ladder (88.2× then 34.3×), and `V_reserved` is why — the
rung is the hold, the reserve is deducted after it, and it bites hardest at the
bottom where a Limited hull's core eats 88% of a `Band Empty` hold.

**Cost: −0.31%, which is to say free.** Four-seed CRN bed, 4,000 yr:

| | colonies | colony-years | doubling |
|---|---|---|---|
| shipped, pre-T-56 | 3,459.8 | 8,011,139 | 284.5 yr |
| stage 3b (cost ladder only) | 3,481.0 | 8,697,322 | 265.0 yr |
| **stage 3c (real geometry)** | **3,481.0** | **8,670,020** | **269.4 yr** |

So the physically correct geometry keeps essentially all of the ladder's
**+8.2%** while cutting the Medium hull's hold from 4.81 kt to 0.92 kt — which
it can, because §2.6's freight-capacity threshold (`load = cap.min(avail)`) is
below both. `tests/balance.rs` reproduces its combat goldens: `hull_dry_mass` is
still `cost_fraction × general_vehicle_cost`, so nothing in `combat.rs` moved.

**The four-decimal `τ` in the spec table is not precise enough to use.** §2.3
*solves* `τ = (cost·η + hold)^(1/3) − hold^(1/3)` from the ratified cost and the
ratified hold, so a rounded value misses its rung: at four decimals the Medium
hull's hold came out 0.9977 kt against a `Band I` of 1.000, 0.2% low — and
enough to make a Colonizer unable to carry a colony seed defined at exactly that
rung. The constants are carried at full width, and they reproduce the rungs at
the **ratified cost ladder**; a config that moves `medium_fleet_size` without
moving the mass ladder has broken `F_mass = F_cost^(3/2)` and the hold drifts
off its rung, which is the tie being visible rather than a bug.

**Three tests changed meaning, each deliberately.**

- `the_cargo_ladder_is_geometric_not_banded` → **`the_hold_ladder_is_the_mass_ladder`**.
  Its own failure message asked for this replacement ("if this has moved to the
  ratified 31.62… replace by a check on `F_mass = F_cost^(3/2)`"), and it now
  asserts the tie against the cost ladder that produced it rather than a copied
  constant. **R-O71 / T-53 closes here.**
- `only_an_inverted_hull_ladder_is_refused` — "a narrow ladder makes the Medium
  hull nearly all shell" is **no longer true**. With a real `τ` the hold is set
  by the price and the hull's own thickness, so a Medium hull priced like a
  Limited one simply *has* a Limited hull's hold: they converge (1.14×) instead
  of collapsing toward zero. Same conclusion — narrow is a real economic
  statement, not a modelling failure — reached by arithmetic with no denominator
  in it.
- `shell_model_ladders_are_derived_not_tuned` — the `1 : √3 : 3` radius
  assertions are replaced by the **inversion**: `shell_volume / η` must return
  the cost that produced the radius. That holds for every ladder; the three
  magnitudes only held for one.

**`hull_ladder_fault` gained a second failure**, and it is a different kind from
the first. `medium_fleet_size ≥ limited_fleet_size` is a naming contradiction; a
hull priced below its own skin (`4·cost·η < τ³`) is a *geometric impossibility*
whose radius solve has no real root, and design law #16 makes the resulting NaN
fatal rather than merely wrong. It is refused at construction so it can never
reach hashed state. At the ratified ladder the tightest margin is the Limited
Offensive hull's, 24× clear.

**Still open after 3c:** `role_hull_type` pins Colonizer and Freighter to Medium,
so nothing in the run ever builds a General hull. Teaching Doctrine to is stage
4, and it is the change the ladder was built to enable — measured separately, or
the result is uninterpretable.


#### Stage 4 — Doctrine can spend the ladder

Three parts, landed together because only the first is separable and none of
the others means anything alone. **The default is unchanged**
(`ColonizerHull::Medium`), so this commit is behaviour-neutral: it makes the
choice *possible* and the measurement is what decides the default.

**4a — the ordered hull is the hull that gets built.** R-O29 moved the hull
choice into `BuildOrder::Hull { hull_type, class }`, but `apply_build` kept
pricing with `role_cost(role)` and `spawn_courier` kept stamping
`role_hull_type(role)`. Those agreed **only because the role map happened to
invert `assign_role`** — nothing asserted the round trip. The first General
colonizer would have flown a Medium ship on a General ship's bill and nothing
would have complained. Now priced and spawned from `hull_type`, with
`hull_cost(hull)` beside `role_cost(role)` (which survives for the Scout and
the paired Freighter, where the role genuinely still picks the hull).
Behaviour-neutral, verified: colony-years **8,481,134.1 / 8,717,150.7**,
bit-identical, and `the_ordered_hull_is_the_hull_that_is_priced_and_flown` pins
the round trip that used to hold by luck.

**4b — a colony ship's seed is the Band its hold masses.** `Simulation::colony_seed_for`
replaces the flat `colony_seed_pop` at `spawn_courier`. This is the ratified hold
ladder doing the work: the hold rungs *are* the mass rungs, so a Medium hull
seeds `Band I` (exactly `colony_seed_pop` — neutral at the hull the baseline
builds) and a General hull seeds `Band II`.

Two consequences worth naming:

- **R-V9 becomes physics.** "A Colonizer must be Medium or larger" was a rule
  about hull types; a Limited hull's hold sits at `Band Empty`, below the floor,
  so `colony_seed_for` returns `None`. `colony_seed_pop` keeps its ratified value
  and changes job — from *the* seed to the **floor a hull must clear**.
- **R-O74 (new, open): the settlers are conjured, and 4b makes it 31× louder.**
  Nothing debits the founding center's population or biosphere for the people put
  aboard. That was already a design law #11 violation at `Band I`; at `Band II` it
  is a bigger one. Not fixed here on purpose: drawing the seed from the origin is a
  behaviour change that would dominate the measurement stage 4 exists to take.

**4c — `Doctrine::colonizer_hull`**, with `Medium` / `GeneralWhenAffordable` /
`General`. `ProductionContext` gains `general_colonizer_cost` so the policy can
weigh the two prices without the context having to know which it will pick, and
`assign_role` grows a `GeneralSystems` arm (without it a General hull would be
built and then find no mission).

#### The result: the ladder is fine, the *mechanism* was wrong

| doctrine | colonies | colony-years | vs shipped | doubling |
|---|---|---|---|---|
| Medium colonizers (shipped) | 3,481.0 | 8,670,020 | — | 269.4 yr |
| General when affordable | **3,481.0** | 8,162,619 | **−5.9%** | 281.8 yr |
| General always | 2,018.0 | 4,518,057 | **−47.9%** | 431.2 yr |
| *ablation:* `Band II` seed, Medium price | 3,481.0 | 8,445,171 | **−2.6%** | 277.5 yr |

**The middle row is the informative one.** Colony count is *identical on every
seed* (3435 / 3467 / 3471 / 3551) while colony-years falls 5.9% — the same worlds,
reached later. The extra minerals bought nothing whatsoever. `General always`
then shows the price acting alone: ten times the cost per colonizer, roughly a
tenth the colonies.

**And the ablation refutes the price explanation.** Seed depth at a *Medium
hull's* price is still **−2.6%, every seed down**. So a `Band II` seed is not
merely worthless, it is **actively harmful even when nearly free** — the General
hull's price is not what killed it.

#### The mechanism, and it is not the one predicted

The prediction below named `sys_production_tick`'s `.clamp(0.0, kb)` and mean
`K = 1.430`. The sign was right and the mechanism was wrong twice over:

1. **The `K` that matters is 1.0, not 1.43.** A founding colony gets
   `infra = Band I` and `K = min(hab, bio_max, infra)`, so *every* Band of seed
   above the first is above capacity on arrival, however good the world is. The
   1.43 figure is the *mature* mean, reached later.
2. **The clamp is not what does the damage — the discrete logistic is.** Growth
   is `s + r·s·(1 − s/K)`, whose growth term goes strongly negative above `K`. At
   the ratified `growth_rate = 0.873`, a population at `2K` does not settle back
   to `K`; it **overshoots to `0.25K` in a single step**. The clamp bounds the
   top only. So a colony founded at `Band II` is *worse off after one tick* than
   one founded at `Band I`, which sits at `K` and stays.

`a_colony_seeded_above_its_capacity_crashes_below_it` pins both halves so the
finding cannot decay back into prose.

**R-O75 (new, open): the discrete logistic can overshoot downward.** Nothing in
the design says an overfull world should lose three quarters of its people in
fifty years; a saturating step, or the closed-form logistic over the interval,
would not. It is a real modelling artifact — but it is on the hottest path in
the engine and **every ratified growth number was measured with it**, so it is
recorded rather than changed. Fixing it consumes `growth_rate`'s ratification.

**What this says about T-56's acceptance test.** The test is that making General
hulls relatively more expensive must not degrade colony-years, and **it already
passed at stage 3b: +8.6%, every seed up.** Stage 4 does not overturn that — it
says the *mechanism* by which a heavier colony ship could pay is the wrong one.
Converting hold into **seed depth** cannot work while a founding colony's `K` is
`Band I` by construction. The ladder wants hold converted into **count** — one
large ship founding several colonies, spending its hold on more `Band I` seeds
rather than one deeper one. That is a different engine change (multi-leg colony
voyages) and it is the natural stage 5.

**`ColonizerHull::Medium` stays the default**, now on evidence rather than for
staging.

#### The prediction as recorded before the run finished

**General colonizers should lose, and not because the ladder is bad.**
`sys_production_tick` grows population as

```rust
target = Band::new((s + growth * s * (1.0 - s / kb)).clamp(0.0, kb));
```

— **clamped to `K`**, whose mean on this bed is **1.430** (`CLAUDE.md` §7). A
`Band II` seed is therefore clamped to ~1.43 on the colony's *first* tick, and
because `draw` goes negative there, the surplus settlers are handed back to the
destination's biosphere. So a General hull buys skipping the ramp from 1.0 to
1.43, at `medium_fleet_size` times a Medium hull's price.

If the run agrees, the conclusion is **not** that T-56's acceptance test fails.
It is that converting hold into *seed depth* cannot pay while `K ≈ 1.43`, and
the ladder wants hold converted into **count** — one large ship founding
several colonies — which is a different engine change and the natural stage 5.
Recorded here before the numbers landed so the run adjudicates it rather than
being written up after the fact.


#### Stage 4c — carry up to `K`, and take the smallest hull that can

*Directed this conversation, and it replaces stage 4's doctrine enum with a
derivation.* The rule is now stated where the decision is made:

```rust
let needed = col.view.k_potential().min(ctx.founding_capacity_cap);
let (hull, cost) = if needed <= ctx.medium_seed_capacity { Medium } else { General };
```

and the load that actually flies is `min(hull capacity, founding K)`.
`Doctrine::colonizer_hull` is **deleted** — the hull is not a preference, it is
whatever fits the load, and the load is capped by the target rather than by the
stockpile.

**Two things this fixes at once.** It removes the failure mode stage 4b
measured — a seed above `K` crashing to a quarter of a Band — by construction
rather than by choosing not to trigger it. And it makes the colonizer's hull a
*derived* quantity, so it will start answering "General" the day something makes
a founding colony's `K` exceed one Band, with no doctrine to remember to change.

**The answer today is always Medium, and the reason is a single constant.**
`sys_colony_arrive` recycles the colony ship's hull into `FOUNDING_INFRA`
(`Band I`), and `K = min(hab, bio_max, infra)` — so a new colony's capacity is
one Band **whatever founded it**, however good the world. Every colonization
target has `k_potential ≥ k_high = 3.2`, so the binding term is never the world;
it is always the infra floor. A Medium hold carries exactly `Band I`. There is
nothing a bigger hull could deliver.

`a_colony_ship_carries_up_to_the_targets_capacity_and_no_more` asserts that
chain — capacity ladder, the cap, both hulls delivering the same seed, and R-V9
as a consequence — so the day it stops being true, it fails.

**The change that would make a General colony ship worth building** is therefore
not in the hold ladder at all: it is **scaling `FOUNDING_INFRA` with the mass of
the hull that was recycled**. That is mass-conservation-consistent (the hull's
minerals become infrastructure, which is what founding already claims to do),
and it is the only lever that raises a founding colony's `K` above one Band. It
is not made here because it moves the whole expansion economy and needs its own
measurement — **R-O76 (new, open)**.


#### Stage 4d — R-O76: founding infrastructure is the recycled hull, priced at the ladder's own rate

**Directed this conversation.** `sys_colony_arrive` has always said the colony
ship's hull *becomes* the colony's first infrastructure, and then awarded one
Band regardless of what was recycled — which pinned every new colony's `K` at
one Band and made a heavier colony ship pointless by construction (stage 4c).

The hull's price now buys infrastructure at the rate the ladder charges,
anchored so a Medium hull still yields exactly `Band I`:

```
budget = hull_cost / medium_hull_cost                (in Medium hulls)
infra  = max b with b(b+1)/2 <= budget               (the ladder, inverted)
       = floor((sqrt(8·budget + 1) − 1) / 2)
```

The ladder charges `round(infra)+1` per level, so reaching Band `b` costs
`1+2+…+b = b(b+1)/2`. At the ratified cost ladder that gives:

| hull | cost | budget | founding infra |
|---|---|---|---|
| Limited | 0.02 | 0.2 | **none** — cannot found |
| Medium | 0.10 | 1.0 | **`Band I`** (unchanged) |
| General | 1.00 | 10.0 | **`Band IV`** |

**The General hull landing exactly on the top playable rung is arithmetic, not
a fit:** `medium_fleet_size = 10` and `1+2+3+4 = 10`. And the Limited hull
buying a fifth of a Band is R-V9 arriving for the *third* time from a different
direction — first the hold ladder, then the seed floor, now the infra ladder.

**Behaviour-neutral, verified:** 8,670,020.2 colony-years on the four-seed bed,
bit-identical on every seed. The baseline still builds Medium colonizers and a
Medium hull's founding infra did not move.

**The hull choice is now a genuine economic comparison, and the criterion is
founding `K` per mineral.** Both hulls fit their load; the General also founds a
far better colony. Measured at a typical target (`k_potential ≈ 3.5`):

| hull | founding `K` | cost | `K` per mineral |
|---|---|---|---|
| Medium | 1.00 | 0.10 | **10.00** |
| General | 3.50 | 1.00 | 3.50 |

so the rule picks Medium, and does so for a reason it can state rather than
because the alternative was unreachable.

**The criterion is myopic and that is recorded, not hidden.** It scores
*founding*, and a Medium colony must then spend `2+3+4 = 9` minerals on the
ladder to reach the `Band IV` a General colony starts at — 9.1 minerals against
1.0, a 9× arbitrage the founding-only score cannot see. Whether the empire
*wants* Band IV colonies is `expand_bias`/`reinvest_bias` territory and is a
measurement, not a derivation. **R-O78 (new, open):** score the hull choice on
lifetime cost-to-`K` rather than founding `K`, and confirm against the
objective before changing the rule.

**R-O77 (new, open): the founding subsidy this scales was already there.** A
Medium hull costs 0.1 minerals and becomes a Band of infrastructure the ladder
charges 1.0 for — founding conjures 10× the minerals spent. Stage 4d preserves
that rate rather than introducing it. Same family as R-O74's conjured settlers,
and recorded rather than closed because removing it would stop colonisation
outright at the ratified hull prices.


#### Staging (each stage independently revertible)

1. **This entry** — analysis, units, candidates. Docs only. Landed in three
   commits: 1 (geometry + first ladder), 1b (`V_reserved`'s second term and the
   role axis), 1c (`V_reserved` per size *and* hull, the cost anchoring, the
   thrust law, and the R-MC15 candidate), 1d (the R-MC15 ratification and what
   it superseded). **R-MC15 is ratified, so stage 2 is unblocked.**
2. **Typed hull geometry — done.** `Length`, `Area` and `Volume` newtypes so a
   unit mismatch in hull design is a compile error, and `shell_thickness` /
   `hold_radius` / `hold_volume` / `shell_volume` as named quantities.
   Behaviour-preserving, verified on colony-years. `η` is still not in the
   engine; it arrives with the values in stage 3.
3. **Adopt the ratified ladder — done**, in three measured steps: 3a the
   piecewise mass ladder (bit-identical), 3b the cost ladder (+8.6%
   colony-years), 3c the real geometry with per-hull `τ` and `η` (−0.31% on top
   of 3b, so the correct physics is free).
4. **Doctrine: build and deploy heavier hulls when useful — landed.**
   `Doctrine::colonizer_hull` makes role→hull a policy choice, the ordered hull
   is finally the hull that is priced and flown (4a), and a colony ship's seed
   is the Band its hold masses (4b). Default unchanged, so the ladder and the
   doctrine spending it stay measured apart.


### T-55. R-O73 — the three F ladders, the quantity survey, and what to ratify

**Ratification candidates for `F_cost`, `F_cargo` and `F_pop`, worked before any
simulation.** Nothing here is implemented.

#### Glossary — every symbol, its unit, and where it comes from

Nothing below is notation invented for this entry; each row is either read out
of `src/sim.rs` or derived from those readings.

**Fixed by the engine** (constants, not choices):

| symbol | value | what it is | code |
|---|---|---|---|
| `√3` | 1.7320508 | `REFERENCE_MEDIUM_RADIUS` — the *constant* radius the capacity ladder normalises against. Constant on purpose (R-O58b): normalising against the live Medium radius made a quantity that can approach zero into a divisor. | `sim.rs` |
| `(√3−1)³` | 0.392305 | the normaliser as it actually appears | `cargo_capacity` |

**Read out of the code** (definitions, not assumptions):

| expression | meaning |
|---|---|
| `cost_fraction(h)` | General `= 1`, Medium `= 1/medium_fleet_size`, Limited `= 1/limited_fleet_size`. A hull's mineral cost as a fraction of a General's. |
| `hull_radius(h) = √(cost_fraction(h) / cost_fraction(Limited))` | cost ∝ surface area ⇒ `r ∝ √cost`. Limited is the unit radius, so `r_L = 1` exactly. |
| `r_G = √(limited_fleet_size)`, `r_M = √(limited_fleet_size / medium_fleet_size)` | the same thing, unfolded |
| `cargo_capacity(h) = cargo_unit_size · (r_h − 1)³ / (√3 − 1)³` | hold as a **mass in kilotons**; `(r−1)` is the usable interior of a unit-thickness shell |
| `hull_dry_mass(h) = cost_fraction(h) · general_vehicle_cost` | dry mass **is** the mineral cost — one number, L6/R-O57 |
| `laden_accel = base_g · G · dry / (dry + cargo)` | `a = thrust/mass`; every term kilotons |

**The four free parameters** — this is the whole of what ratification chooses:

| symbol | meaning | unit |
|---|---|---|
| `F` | the shared band width, `F_pop = F_cargo` (Band equivalence) | dimensionless ratio |
| `N₁` | people at population Band I — "a small town" | people |
| `kg/person` | mass per colonist at Band I (**300**, ratified this conversation) | kg |
| `r_M` | the Medium hull's radius, in Limited-radii. The one free *geometric* parameter. | dimensionless |

**Everything else is derived, in this order:**

```
c                  = F^(1/3)                        # per-Band radius step of the hold
r_G                = 1 + (r_M − 1)·c                 # forces F_cargo = F exactly
limited_fleet_size = r_G²
medium_fleet_size  = (r_G / r_M)²
F₁_cost            = r_M²  ( = limited_fleet_size / medium_fleet_size )
F₂_cost            = medium_fleet_size
KT(I)              = N₁ · kg_per_person / 10⁶        # kilotons at pop Band I
cargo_unit_size    = KT(I) · (√3 − 1)³ / (r_M − 1)³  # so the Medium hold IS KT(I)
dry_M              = general_vehicle_cost / medium_fleet_size
base_g             = 0.18349 · (1 + KT(I)/dry_M)     # holds laden accel at today's
Band IV population = N₁ · F³
```

`0.18349` is today's colonizer laden-acceleration factor,
`dry/(dry+cargo) = (1/4.45)/((1/4.45)+1)`, carried as the thing to preserve.

**Fully worked, and verified back through the code's formulas rather than the
derivation:**

| opt | `F` | `N₁` | `r_M` | `r_G` | `limited_fleet_size` | `medium_fleet_size` | `F₁_cost` | `F₂_cost` | `KT(I)` | **`cargo_unit_size`** | `dry_M` | `base_g` | Band IV pop |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 100 | 3,200 | 2.000 | 5.642 | 31.828 | 7.957 | 4.00 | 7.96 | 0.960 kt | **0.3766** | 0.1257 | 1.59 g | 3.20 B |
| 2 | 81 | 5,000 | 2.100 | 5.759 | 33.171 | 7.522 | 4.41 | 7.52 | 1.500 kt | **0.4421** | 0.1329 | 2.25 g | 2.66 B |
| 3 | 64 | 10,000 | 2.300 | 6.200 | 38.440 | 7.267 | 5.29 | 7.27 | 3.000 kt | **0.5357** | 0.1376 | 4.18 g | 2.62 B |

Checked for all three, through `cargo_capacity` and `laden_accel` as written:
Medium hold `== KT(I)`; General hold `/` Medium hold `== F` (Band equivalence);
laden acceleration preserved exactly; both cost steps inside `[4, 8]`.
Resulting holds — Limited `0.000`, Medium `KT(I)`, General `F · KT(I)`: option 1
gives 0 / 0.96 / 96 kt, option 3 gives 0 / 3.0 / 192 kt.

#### Correction: "grow the hold" is achieved by *lowering* `cargo_unit_size`

Earlier this entry quoted the hold as growing ×1.00 / ×1.56 / ×3.13. That is
right as a statement about **the Medium hull's hold in kilotons**, and it is
**the opposite of what happens to the knob**: `cargo_unit_size` must fall from
**5.0 to 0.38–0.54**.

The reason is that `cargo_unit_size` is not the hold — it is the hold *at the
reference radius √3*, and the new cost ladder moves the Medium hull's radius
from today's 1.422 to 2.0–2.3. The geometric factor `(r_M − 1)³ / (√3 − 1)³`
goes from 0.192 to 2.55–4.98, a 13–26x swing, so a much smaller
`cargo_unit_size` yields a larger hold. This is the same trap T-53 recorded —
the field is named for a hull whose radius it no longer describes — showing up
a second time, now in the direction of the fix.

#### The closed form that decides it
#### The closed form that decides it

Shell model with the Limited hull at unit radius (`r_L = 1`, all shell, no
hold):

```
cost ∝ r²          F₁_cost = r_M²        F₂_cost = (r_G/r_M)²
capacity ∝ (r−1)³  F_cargo = ((r_G−1)/(r_M−1))³
```

Writing `c = F_cargo^(1/3)` so `r_G = 1 + (r_M−1)c`:

```
F₂_cost = (c + (1−c)/r_M)²   — increasing in r_M, supremum c²
     ⇒   F₂_cost < F_cargo^(2/3),  always
```

**So §2.6's own floor `F_cost ≥ 4` forces `F_cargo > 4^1.5 = 8`.** The spec's
cost constraint rules out a cargo ladder inside `[4, 8]` — an internal
contradiction that has nothing to do with the population fiction, and an
independent argument for widening the bands.

#### The admissible window is narrow, and it is a *result*, not a choice

Requiring **both** cost steps in `[4, 8]`:

| `F_cargo` | legal `r_M` | `F₁_cost` | `F₂_cost` | `medium_fleet_size` | `limited_fleet_size` |
|---|---|---|---|---|---|
| 9, 12, 16 | — | \multicolumn — **no legal cost ladder** | | | |
| 27 | 2.000–2.828 | 4.00–8.00 | 4.00–5.26 | 5.26 | 42.04 |
| 40 | 2.000–2.828 | 4.00–8.00 | 4.88–6.58 | 6.58 | 52.59 |
| **64** | **2.000–2.561** | **4.00–6.56** | **6.25–8.00** | **7.27–8.00** | **38–52** |
| 81 | 2.000–2.220 | 4.00–4.93 | 7.09–8.00 | 7.52–8.00 | 33–39 |
| 100 | 2.000–2.008 | 4.00–4.03 | 7.96–8.00 | 7.96 | 31.83 |
| 144, 216 | — | **no legal cost ladder** | | | |

**`F_cargo` is pinned to roughly `[27, 100]`** by the cost constraint alone.
Below it the cost ladder cannot reach `F₂ ≥ 4`; above it `F₂` overshoots 8.
`F = 64` has the most comfortable interior window; `F = 100` is a razor's edge
at `r_M = 2`.

#### Band equivalence, in the form that survives distinct ladders

Set **`F_pop = F_cargo`**. Then a hold at cargo-Band `N` carries a population at
pop-Band `N` — the equivalence that actually matters, because it is the colony
ship. It also reproduces roles §6's `0 / 1 / 2` exactly as Bands: Limited is
cargo-`Empty` (all shell), Medium is cargo-`I` (§2.6 already calls the Medium
hold cargo's own Band I), General is cargo-`II`.

`F_cost` stays a **distinct** ladder — it must, per R-O71's proof — but it is
*derived* from the same geometry rather than chosen, so the three ladders are
one system with one free parameter. What is shared is **ordinal**: Band `N` is
the same tier everywhere, which is what the cross-quantity gates (synthesis at
pop Band IV, `medium_min_level`, roster unlocks) actually read.

#### Option sets

| # | `F` | town @300 kg/p | Band IV pop | `KT(I)` | hold | colonizer `base_g` | `r_M` | `med_fleet` | `lim_fleet` |
|---|---|---|---|---|---|---|---|---|---|
| **1** | 100 | 3,200 | 3.2 B | 0.96 kt | **×1.00** | 1.59 g | 2.00 | 7.96 | 31.83 |
| **2** | 81 | 5,000 | 2.66 B | 1.50 kt | ×1.56 | 2.25 g | 2.10 | 7.52 | 33.17 |
| **3** | 64 | 10,000 | 2.62 B | 3.00 kt | ×3.13 | 4.18 g | 2.30 | 7.27 | 38.44 |

All three satisfy "small town → many billions", keep both cost steps in
`[4, 8]`, and hold Band equivalence. **Today's ladder (4.45 / 9.0) gives
`F₁ = 2.02`, `F₂ = 4.45` — neither pair legal**, which is R-MC15 restated with a
resolution now available.

Note `base_g` rises in all three even where the hold does not: the new cost
ladder makes the Medium hull *lighter* (`dry = 1/medium_fleet_size`, so 0.126 kt
at `med_fleet = 7.96` against 0.225 today), so keeping laden acceleration at
today's 0.183 g needs more thrust, not less.

Design law #3 holds throughout — at `F = 100`, cost per kt hauled is 4.00
(Medium) against 0.318 (General), so consolidation wins by 12.6x.

#### The survey: every quantity, and the ladder it binds to

| ladder | quantities |
|---|---|
| **Planetary (`F_pop`)** | `habitability`, `biosphere`/`bio_max`, `infrastructure`, `population`, `k`/`k_potential`, `PopBands` edges, `rank.k_high`, `rank.hub_high`, `colony_seed_pop`, `medium_min_level`, `limited_min_level`, habitability §3's gravity/radiation suitability |
| **Cost (`F_cost`)** | `general_vehicle_cost` (its Band I anchor), `medium_fleet_size`, `limited_fleet_size`, `hull_dry_mass`, `homeworld_start_minerals`, **the infra upgrade price**, `scrap_recovery_fraction`'s base |
| **Cargo (`F_cargo`)** | `cargo_unit_size` (its Band I anchor = the Medium hold), `HullType::cargo_capacity` |
| **Mineral density** | `mineral_peak`, `density_floor`, `rank.mineral_high` — in-ground density, arguably the cost family's Band I before extraction; **unclassified, needs a call** |
| **On no ladder** (rates, times, fractions, weights, counts) | `horizon_years`, `cycle_years`, `build_years`, `growth_rate`, `biosphere_regen_rate`, `biosphere_regen_bonus`, `trade_decay_lambda`, `productivity_step`, `reinvest_bias`, `w_k`/`w_mineral`/`w_hub`, `centrality_scale`, `mineral_pressure_gain`, `civilian_accel_g`, `survey_accel_g`, `center_mining_fraction`, `outpost_mining_fraction`, `mining_tick_years`, `survey_reserve`, `survey_vehicles`, `max_survey_hops` |

#### What the survey turned up

**The infrastructure upgrade price is linear where the ladder says
multiplicative.** `apply_build_with` charges `round(infra) + 1` minerals — Band
I→II costs 2, II→III costs 3, III→IV costs 4. Infrastructure is a *planetary*
Band and its price is a *cost*-family quantity, so this is the one place the two
ladders touch, and it currently bridges them with a straight line. Under any
`F`, deepening one Band should cost `F` times the last one.

**This sits exactly on T-51's critical path.** The expansion-loop time constant
is the pre-`medium_min_level` staircase: mine `round(infra)+1`, deepen, grow,
repeat. Making that price multiplicative changes the staircase's shape directly,
and at `F ∈ [27, 100]` it makes the upper rungs enormously more expensive — 2 /
3 / 4 minerals today against 1 / `F` / `F²` under the ladder. **This is very
likely a larger effect on coverage than anything else in this entry, and it is
not optional if the Band ladder is taken seriously.** Measure it alone, first,
with `examples/colony_years`.


### T-54. R-O72 — the Band→kiloton bridge is wrong in two structural ways, and the scale is off by 10⁴–10⁵

**No code changed. Calibration first, by instruction.** `units.rs`'s bridge was
written from `Hyades_mineral_cost_curve.md` §2.6 and it contradicts that section
twice. Both are structural, not tuning.

**1. `Band III → IV` is unconstrained by design, and the bridge forces it to
`BAND_STEP`.** §2.6 is explicit: *"`Band III → Band IV` is unconstrained, and
does not need to be the same factor across quantities. This is a deliberate
release valve... the last Band before a quantity's absolute ceiling
(population's 'many billions') ... the natural place for a balance-tuning knob
rather than a physically-derived constant."* The `[4, 8]` constraint binds `F₁`
(I→II) and `F₂` (II→III) **only**. `KT(b) = KT_I · BAND_STEP^(b−1)` applies one
factor to every step, so it silently ratifies `F₃ = 4`. **The bridge has to be
piecewise.**

**2. Every quantity anchors its own `Band I`; the bridge has one global
anchor.** §2.6: *"Every quantity anchors its own `Band I` independently — what
has to be shared across quantities is not the absolute value at `Band I`, it is
the ratio between consecutive Bands."* It names three separate anchors — *a
small town* (population), *the cost of one General-class hull* (mineral spend),
*a Medium hull's reference cargo hold* (cargo). `KILOTONS_AT_BAND_I` is a single
constant shared by all of them, which forces the three equal.

**How far off the scale is.** `Hyades_galaxy_and_autopilot.md` §5.1: *"Pop `Band
I` ≈ small town; pop `Band IV` ≈ many billions."* That is the calibration
target, and it is not close:

| reading | Band I → Band IV ratio required |
|---|---|
| small town 5,000 → 2 billion | 400,000x |
| small town 5,000 → 10 billion | 2,000,000x |
| large town 20,000 → 2 billion | 100,000x |
| **what the code gives** | **64x** |

So `F₃` must be **O(10³–10⁶)**: 6,250–1,250,000 if `F₁ = F₂ = 4`, or
1,562–312,500 if `F₁ = F₂ = 8`. Against the 4 the code uses. **This is exactly
the freedom §2.6 reserved `F₃` for, and the bridge spent it without noticing.**

**The gigagram, checked against the descriptions.** 1 kiloton = 1,000 t = 10⁶ kg
= 1 Gg. At `KT(Band I) = 1 kt`, a Band-I population implies:

| "small town" | kg per person |
|---|---|
| hamlet, 500 | 2,000 |
| small town, 5,000 | **200** |
| large town, 20,000 | 50 |

Reference points: a human body is ~70 kg; Apollo CSM+LM ran ~15 t per crew; the
ISS is ~70 t per crew of six. **200 kg/person buys the people and roughly their
clothes** — no habitat, no life support, no common areas, on a flight measured
in decades.

**And there is a three-way squeeze that pins it, which is the useful part.**

1. Population `Band I` is a small town (galaxy §5.1).
2. A Colonizer carries `colony_seed_pop` = Band I of settlers (R-V9), in a
   **Medium** hull — the smallest that can (roles §4.2).
3. The Medium hold is **0.959 kt** at the shipped ladder (T-53).

(1) and (2) and (3) together force Band I ≲ 1 kt, hence the 200 kg/person. To
get to a physically comfortable 10–100 t/person you need Band I at 50–500 kt for
a 5,000-person town — **50–500x the hold of the hull that has to carry it.** One
of the three has to give:

- **the town shrinks** — "small town" becomes ~10–100 people, a landing party
  rather than a town, and the flavour text in galaxy §5.1 changes;
- **the hold grows** — which is the cost ladder, so it lands on T-53/R-O71 and
  R-MC15 together;
- **the seed stops being a whole Band** — `colony_seed_pop` becomes a fraction
  of Band I, and R-V9's "1 pop as cargo" is reinterpreted.

That is a design call and it is upstream of any code. **Nothing in `units.rs`
should move until it is made**, because the anchor and `F₃` are both determined
by whichever branch is taken.

**Settled — the ladder is named, and Band V is a ceiling.** `Band Empty` is now
**`BandTier::Empty`**, named rather than numbered because it is the one rung
that is a *condition* (no colony, no hold, an uncolonizable world) rather than a
magnitude. **`BandTier::V` is the maximum for comparison and clamping and is not
achievable in play** — it gives a bounds check a rung one past the end instead
of a magic number, and `band_v_is_one_past_the_playable_end` pins that it stays
unreachable. `BandTier::MAX_PLAYABLE` is `IV`, matching every spec.

**Settled — the squeeze resolves by growing the hold.** Of the three
constraints, the **Medium hold** is the one that gives: it rises to carry a real
town, and the town description in galaxy §5.1 and R-V9's "1 pop as cargo" both
stand. Still to be implemented, and it is not a one-line change:

- Since R-O58 the cost ladder **is** the capacity ladder, so this lands on
  **R-O71** (the 106x M→G step, which growing `cargo_unit_size` uniformly does
  *not* fix) and **R-MC15** (no shipped ladder satisfies `[4, 8]`) together.
- **`cargo_unit_size`'s coverage elasticity is already measured as exactly zero**
  at this operating point — `binding_check` is bit-identical at 5, 25 and 100,
  because `load = cap.min(avail)` and an outpost never accumulates a full hold
  between visits. Growing it is therefore free for the *mineral* economy. Do not
  read that as "no effect anywhere".
- **The colony seed is where it bites.** `pop_cargo` masses `KT(Band I)` and
  `laden_accel` is `dry / (dry + cargo)`. A Medium hull is 0.225 kt dry against
  a 1.0 kt seed today — 18% of base acceleration. At a 50 kt seed that is 0.45%,
  and travel time goes as roughly `1/√a`, so colony ships get ~6x slower and
  coverage falls hard. **Growing the hold without growing the hull makes the
  hold 200x the ship's own structure**, which is not a body the shell model
  describes.
- So the honest shape of the work is: raise the anchor *and* re-derive the hull
  ladder, so the carrier of a Band-I population is a hull that plausibly masses
  more than its cargo. Measure with `examples/colony_years` — colony *count* at
  the horizon will not show a slowdown that `∫ colonies dt` will.

### T-52. R-O69 follow-ups — the hostile-interrupt trigger, and the decision path's O(galaxy) scan

Two things the production decoupling (R-O69) left open, one blocked and one a
cost it exposed.

**The hostile-interrupt trigger does not exist.** The design calls for a new
decision when a build is *interrupted by hostiles* as well as when one
completes. `EventKind::BuildDecision` is the event either would raise, but
nothing can raise the second: **no combat is wired into the simulation loop** —
`combat::resolve_engagement` is never called from `sim.rs`. When it is,
interruption is this event scheduled at the moment of the strike after clearing
`building_until`, which is a scheduling call rather than a redesign. Blocked on
combat integration (T-12/T-30), not on engine work here.

~~**The decoupling exposed a pre-existing `O(galaxy)` cost in the decision path,
and made it fire twice as often.**~~ **Fixed — R-O70.** The premise was right and
the diagnosis inside it was wrong, which is the part worth keeping.

**What was actually costing the time.** Instrumented iteration counters, seed 1
at the shipped horizon — counting loops rather than guessing at them:

| loop | calls | items scanned | heavy work |
|---|---|---|---|
| `holdings_centroid` | 169,211 | **1,138 M** | — |
| production candidates | 169,211 | **1,033 M** | 328 M `view_of` + `rank` |
| survey candidates | 26,456 | 178 M | 68 M views |

**Two guesses were made before that table existed and both were wrong.** The
candidate `Vec` and a cached `ln` in `view_of` were fixed first and produced
**no speedup at all** — 60 → 58 yr/s, inside noise. Survey was then assumed to
be the hot path, on the strength of §4's worked example; it is an order of
magnitude smaller than either of the other two. A slow program is a symptom, and
CLAUDE.md §2 says a symptom needs a *proven* mechanism. Profiling is what proves
this one, and it cost one instrumented run.

**The three real fixes, all bit-identical:**

1. **`holdings_centroid` memoised.** A full `planet_entity` walk per decision,
   for a value that only moves when a colony is founded — ~3,400 changes against
   169,211 calls. The cache stores the **recomputed** value rather than a running
   sum, deliberately: an incremental sum accumulates in claim order while the
   walk accumulates in planet-id order, and float addition is not associative, so
   the centroid would differ in its last bits and every rank score with it.
   `claim_planet` is now the only sanctioned way to give a planet an owner, and a
   `debug_assert` recomputes and compares on every call in test builds — a missed
   invalidation fails loudly instead of diverging on some seeds.
2. **`Knowledge::targeted`: `BTreeSet` → bitmap.** Membership-tested **1.03
   billion times** and never iterated. This is the exact fix already applied to
   its sibling `visited`, whose doc comment records that one `BTreeSet::contains`
   was once 63% of all engine instructions — the lesson was learned and never
   carried across.
3. **`Knowledge::scanned`: `BTreeSet` → sorted `Vec`.** *Iterated* 1.03 billion
   times on the hottest path. A B-tree walk is a pointer chase over boxed nodes;
   a sorted `Vec` is a sequential read, and §4's own finding is that locality
   beats element count. Identical order, so nothing about determinism moves.

**Result: 3.3x, with the guard held exactly.** Seed 1 60 → 198 yr/s, seed 7
52 → 185 yr/s, and **colony-years identical to the decimal** (7,819,401.0 and
8,480,172.0), as are colony counts, foundings and first-founding times. Against
T-24's 2.5 yr/s floor the margin at 3 seats / 4 kyr goes ~26x → ~79x, and the
12-seat / 8-kyr corner from ~1.3x to ~3.8x. R-O69's +165.8 colonies now cost
about a quarter of the throughput rather than four times it.

**Still open on this surface.** The production candidate scan remains
`O(scanned)` — 1.03 G iterations, 328 M of them building a view and ranking it —
and is now the largest loop left. An incremental per-player ranked frontier is
the obvious next step, but §4 records that the last attempt at exactly that came
out *slower*, because swap-removal traded a sequential walk for random access.
**Measure it, keep the iteration order stable, and hold colony-years fixed** —
`examples/colony_years` exists for precisely that.

### T-53. R-O71 — the cargo ladder is geometric where §2.6 says it must be Banded

**Investigated. The reported units error is real but it is not where it was
expected, and the dimensional analysis of the acceleration term comes out
clean.**

**What is *not* wrong.** `laden_accel` is `dry / (dry + minerals + pop)` and
every term is kilotons: `dry` is `hull_dry_mass`, which L6/R-O57 made identical
to mineral cost; `minerals` is `Minerals::basic_total()`, the same unit by that
same identity — a mineral in the hold masses exactly what it massed as hull,
which is why no `cargo_mass_per_unit` coefficient exists any more; and `pop`
goes through `units::population_mass` (fixed at R-O66). There is no Band
standing in for a mass in that equation. The readings are now taken explicitly
through the types anyway, and `cargo_capacity` returns [`Kilotons`], so the
next reader does not have to redo this analysis.

**What is wrong: the ladder.** `Hyades_mineral_cost_curve.md` §2.6 requires one
step factor `F ∈ [4, 8]` to govern every Band-laddered quantity and **names
cargo capacity in that list**. Measured (`examples/cargo_units`, shipped
defaults):

| hull | dry mass | hold | step |
|---|---|---|---|
| Limited | 0.111 | 0.000 kt | — |
| Medium | 0.225 | 0.959 kt | ∞ |
| General | 1.000 | 101.96 kt | **106.35x** |

Against §2.6's `[4, 8]` that is 13–26x outside the permitted range. Under the
Band reading of roles §6's 0 / 1 / 2 — Limited at Band Empty, Medium at Band I,
General at Band II, which is the "Bands I–III worth of stuff" the design
intends — the holds would be **0.25 / 1.0 / 4.0 kt**, steps of exactly
`BAND_STEP`. Note the Medium hull lands at 0.959 kt, within 5% of `KT(Band I)`;
it is General that is two orders of magnitude off.

**Two ratified specs disagree by an order of magnitude, so nothing was
changed.** R-O58's shell model derives capacity from usable interior `(r − 1)³`
and is ratified; §2.6's Band constraint is ratified. Picking between them is a
design call and `cargo_unit_size` sits behind an MC-tuned cost ladder, so this
is flagged, pinned by `the_cargo_ladder_is_geometric_not_banded`, and left for
ratification. **R-O71.**

**A concrete inconsistency the mismatch already produces.** A Colonizer carries
`colony_seed_pop = 1.0` Band of settlers, massing `KT(I) = 1.0 kt`, in a Medium
hull whose hold is **0.959 kt** — R-V9 makes Medium the smallest hull that can
carry a colony seed, and the seed does not fit. Nothing catches it because
capacity gates mineral loading only and pop cargo is inserted directly. Asserted
in the same test so it cannot drift unnoticed.

**Also corrected: a stale doc that was misreading its own sweep.**
`SimConfig::cargo_unit_size` was documented as "what a Medium hull carries, in
kilotons". It is the hold of a hull at `REFERENCE_MEDIUM_RADIUS = √3`, which the
Medium hull has only when `medium_fleet_size == 3`; at the ratified 4.45 the
Medium hull carries **0.959 kt against the field's 5.0**, a factor of 5.2. The
constant normaliser is deliberate and correct (R-O58b) — the doc line was
stale. It matters because the `binding_check` sweep table in that same comment
has this field as its x-axis, so "the hold stops binding past roughly 1–5"
means *past roughly 0.19–0.96 kt of real Medium hold*. Since R-O58 the cost
ladder **is** the capacity ladder, and CLAUDE.md §2 says a parameter that
reaches the objective through a derived quantity cannot be swept alone.

**Resolution options, for whoever ratifies:**

1. **Band the capacity ladder** — set holds to `KT(0) / KT(I) / KT(II)` and let
   the shell model keep only its *ordinal* content, the way roles §6's slot
   count was already demoted at R-O64. Costs the 20x consolidation incentive
   that design law #3 leans on.
2. **Exempt cargo capacity from §2.6** and strike it from that list, on the
   grounds that a hold is a *volume* and the Band ladder governs magnitudes of
   stuff rather than the containers. Cheapest, and needs §2.6 amended.
3. **Re-derive `F` from the geometry** so the two agree by construction, which
   constrains the cost ladder rather than the capacity one — and `F ∈ [4, 8]`
   with `capacity ∝ (r−1)³` pins `medium_fleet_size` hard. That interacts with
   R-MC15 and T-19, so it is the one to think about before the others.

Whichever is chosen, **ablate before believing any coverage effect**: the
biomass economy looked load-bearing and turned out entirely slack, and
`cargo_unit_size`'s own elasticity is already measured as exactly zero at this
operating point.


### T-51. R-O68 — the deepen/expand trade does not exist, and it is what sets the expansion-loop time constant

**Symptom, then mechanism, as CLAUDE.md §2 requires.**

*Symptom.* Post-R-O66 the bed no longer saturates: 94.6% of the `k_high` set at
the horizon against 99.8% before, with the founding rate still near peak in the
last bucket. "The expansion loop's time constant is binding" — true, and not yet
a mechanism.

*Mechanism, proven.* `production_choice` chooses depth over expansion when
`b · deepen_headroom >= (1 − b) · score`. **Those two sides are not in the same
unit.** `deepen_headroom` is a *Band* difference, `k_potential − infra`, bounded
by 4 and in practice by `k_potential − 1`. `score` is `rank`'s weighted sum over
a Band, a mineral density and a hub figure — unbounded, and dimensionless only
by fiat. Measured (`examples/score_scale`, seed 1), colony-class candidate
scores run **p05 = 4.40, median 6.17, p95 = 8.97, max 12.16** at zero mineral
pressure, and the branch compares against the *maximum* because `outward` takes
the best candidate. Depth therefore wins only when `b >= score/(score+headroom)
≈ 0.8`.

**At the shipped `reinvest_bias = 0.5` the branch cannot fire while any
candidate exists.** It is not a convex dial; it is **inert below ~0.8 and a hard
switch above it** — a step function wearing a dial's clothes, and a search
cannot climb it because there is no graded region. Pinned by
`reinvest_bias_is_a_step_function_not_a_dial`.

**So the policy has three deepen paths and only two are live:**

| path | condition | live? |
|---|---|---|
| pre-`medium_min_level` staircase | `level < 3 && deepen_possible && can_afford_infra` | **yes — unconditional** |
| the convex dial | `b · headroom >= (1 − b) · score` | **no, at the shipped bias** |
| the no-candidate fallback | `outward == None` | yes, but only on an empty frontier |

Which means the expansion-loop time constant is set **entirely by the pre-level-3
staircase**, with no tunable trade anywhere near it. It also explains R-O66
exactly: that change moved `deepen_possible`, which gates the *staircase*, not
the dial — so all −178 colonies came through the one branch that actually runs.

**The staircase, which is the real time constant.** A colony is founded at
`colony_seed_pop` = Band 1 with `infra = 1`, so `K = min(hab, bio_max, 1) = 1`
and it starts *at* its ceiling with zero growth headroom. To produce its own
colonizer it must, serially:

1. mine `round(infra)+1` minerals (2, then 3) at `center_mining_fraction × density`
   per tick, plus whatever hauling delivers;
2. deepen 1 → 2 → 3, each step raising `K`;
3. grow logistically at `growth_rate` past the level-3 `PopBands` edge (~2.675);
4. accumulate `colonizer_cost` (← `medium_fleet_size`);
5. fly there at `civilian_accel_g` over the target distance.

Every step is quantized to `cycle_years = 50`.

**Why the current gradient probe cannot see this.** Of its nine knobs, only four
touch the chain at all (`center_mining_fraction`, `growth_rate`,
`trade_decay_lambda`, `medium_fleet_size`). Absent entirely: `cycle_years`,
`medium_min_level`, `limited_min_level`, `colony_seed_pop`, `reinvest_bias`,
`civilian_accel_g`, `limited_fleet_size`, `PopBands::top_edge`, and the infra
cost ladder — **which is hard-coded as `round(infra)+1` and is not a parameter
at all.** Worse, the gates are *discrete*: `medium_min_level` and
`limited_min_level` are `u8`, `cycle_years` quantizes everything, and a
central-difference probe has no gradient to read on any of them. So the loop's
rate limiters are largely invisible to an all-continuous elasticity method,
which is why every probe so far has ranked ecology and hull-cost knobs.

**Work, in order:**

1. **Widen the probe surface** to the continuous knobs on the chain above
   (`colony_seed_pop`, `civilian_accel_g`, `limited_fleet_size`,
   `PopBands::top_edge`, `reinvest_bias`), and add **role/hull cost** knobs
   rather than only `medium_fleet_size`. Record raw per T-50.
2. **Sweep the discrete gates separately** — `medium_min_level ∈ {2,3,4}`,
   `limited_min_level ∈ {1,2,3}`, `cycle_years ∈ {25,50,100}` — because a
   gradient cannot. Expect the level gates to be the largest single effect on
   time; they are the staircase.
3. **Decide what the deepen/expand comparison should be.** Both sides must be a
   *rate of return in one unit* — plausibly expected colonies per mineral per
   year, discounted — so that `reinvest_bias` becomes a preference over a real
   trade rather than a unit-conversion constant with a preference hidden in it.
   This is a policy redesign and `reinvest_bias` is globally MC-tuned, so it
   needs ratification (CLAUDE.md §6), not a quiet edit.
4. Only then ask whether the *structure* wants to change. The policy is already
   a decision tree; the fault is a dead branch and an incommensurable
   comparison, not the tree form. Fix the comparison before replacing the
   shape — otherwise a richer structure inherits the same broken predicate.

### T-50. Record gradient sensitivity as raw data, not prose — it is card-design input

**The elasticities are being spent and thrown away.** Every gradient probe this
project has run has ended up as a sentence in `CLAUDE.md` or a doc — "`+141.2 ±
18.1`, first" — and then been invalidated by the next step, leaving nothing
behind. That is the wrong artifact. The *ranking and magnitude* of
`∂(objective)/∂ln(knob)` is exactly the table card design needs: a card is a
perturbation of a knob, so **the elasticity is the card's effect size before the
card exists.** "Terraforming raises `hab`" is not a design until you know what a
10% move in `hab` is worth relative to a 10% move in `growth_rate`.

**What to keep.** `examples/gradient_probe.rs` already computes everything; it
just prints it. Persist the *raw* per-knob, per-seed evaluations — not the
summary — into a checked-in dataset, with:

- knob name, base value, δ, and both perturbed values;
- the objective at each, **per seed** (CRN pairing is the whole point, and a
  mean discards it);
- the operating point: full `SimConfig` + `Doctrine` at the time, plus the
  engine commit. An elasticity without its operating point is not a
  measurement, it is a rumour — this file has already recorded that mistake
  twice (R-AC18/R-AC20, and the post-gradient-step re-measure).

**Why raw and not derived.** Standard errors, elasticities and rankings are all
recoverable from the raw evaluations; the reverse is not true. It also lets a
later reader re-analyse under a *different* objective without re-running — which
matters, because this project has changed its objective twice already (fraction
→ absolute count, and the operating point moved again at R-O66).

**Format:** a plain CSV or TSV under `data/`, appended to rather than
overwritten, one row per evaluation. Zero dependencies, diffable, and readable
by whatever does the card-costing analysis later. The harness gains a
`--record <path>` flag; nothing else changes.

**Related:** T-45 (elasticity baseline) is the first dataset this should
capture, and it needs re-running at the post-R-O66 operating point anyway.

### T-49. R-O67 — population stalls when it runs out of biomass; it never dies back

**Design call, not a bug to patch.** Population responds to two ceilings and the
model corrects for only one.

- **Above `K`** (the Band ceiling) the logistic is self-correcting: `1 − s/K`
  goes negative, population declines, and the decline *returns* its mass to the
  biosphere. A razed world sheds people back down to its new ceiling. Works
  today, pinned by `drawing_the_biosphere_down_does_not_lower_the_ceiling` and
  the growth tests.
- **Out of biomass** there is no correction. The draw is capped at what is
  standing, so growth **stalls** at whatever population it reached and nobody
  starves. No Malthusian overshoot-and-crash exists; a world can sit
  indefinitely with a live `K`, a dead biosphere and a frozen population.

The reachable case today is **immigration**: `biomass + KT(pop) ≤ bio_max` holds
under growth (1:1 conversion) and regrowth (stops at the ceiling), but a
colonizer lands `colony_seed_pop` of people whose mass came off a ship, and
nothing works the overshoot back off. Bounded and small — one seed's worth per
world, 1 kt against a typical 33 kt ceiling — and `tests/smoke.rs` asserts the
bound *including* that allowance rather than hiding it.

**The question to ratify:** should starvation kill? A die-back is a sharp
weapon for Warfare and a real stake for Greening — a biosphere strike that
kills people rather than freezing them is a different card. It also needs
somewhere for the mass to go, which is slag (R-O59/T-03). Concrete test once
decided: zero a settled world's biosphere and assert the population curve.


### T-47. R-AC20 — ~~time-to-10% vs. coverage disagree on `medium_fleet_size`~~ premise withdrawn; `center_mining_fraction` still open

> **Resolved, and it was an artifact.** The two elasticities were measured at
> different operating points (coverage's `+32.7 pts/ln` at `medium_fleet_size
> = 3.0`, before the ratification moved it to 4.45; the time-to-10% number at
> 4.45). Measured directly at ±25% around the shipped value, coverage is an
> **interior optimum at 4.45** on all three seeds, so raising it further hurts
> *both* metrics — they agree. Comparing gradients across operating points is
> the mistake; "a gradient is local" (CLAUDE.md §2) has now produced a project
> artifact rather than merely warning about one. **What is still open:**
> `center_mining_fraction` (`~noise` at 1.33 SE) wants the ten-seed bed.
> **What the detour actually produced:** the `colonies@2000` screening metric
> — ρ = 0.923 against true coverage at 31× less cost, now documented in
> CLAUDE.md §2 and calibrated by `examples/proxy_metric_calibration.rs`.

The original entry, kept for the record:

While resolving R-AC3 (survey-sector strategy: no measurable effect on early
speed — `Hyades_autopilot_colonization_growth.md` §2), a gradient probe
retargeted at years-to-10%-colonized (`examples/time_to_10pct_probe.rs`)
found `medium_fleet_size` — the single largest lever for coverage-at-4,000-yr
(+32.7 ± 3.6 pts/ln, already MC-ratified at 4.45) — has the **opposite**
sign for time-to-10% (+161.5 ± 41.7 yr/ln: raising it *slows* the early
game). Plausible mechanism: a cheaper Medium hull is also a smaller one
(shell model), while the pop-seed cargo a Colonizer must carry
(`colony_seed_pop = 1.0`) is a fixed mass, so laden acceleration drops as
the hull shrinks under a fixed load — more colonizers get built, each one
slower to arrive. Not yet directly traced.

`rank.k_high` (ratified at 3.2 for coverage, R-AC17) shows the same-sign
tension more weakly (borderline 2.06 SE on the time-to-10% side).
`growth_rate` and `outpost_mining_fraction` agree in direction across both
objectives — no tension there.

**The concrete test:** define an explicit blended objective (e.g.
`α·coverage_at_horizon − β·time_to_10pct`, or "time to first elimination,"
cmd §"R-5") and re-run the gradient-step methodology against it, rather than
picking a number without a stated objective. Also: `center_mining_fraction`
came back `~noise` (1.33 SE) on the 4-seed bed and wants the ten-seed bed
(T-44's precedent) before trusting either sign.

**Not this file's call:** whether the shipped defaults should move at all is
a product decision about how much early-game feel is worth trading against
late-game sprawl — flagged, not resolved, per R-AC20.

### T-48. ~~Finish the metric change — five harnesses still divide~~ **done**

All five (`binding_check`, `colonization_ramp_trace`, `min_time_search`,
`mining_probe`, `proxy_metric_calibration`) now report absolute colony counts.

**One unit mismatch was caught in the conversion and is the reason to do these
by hand rather than by `sed`.** `proxy_metric_calibration`'s healthy-band floor
was `HEALTHY_COVERAGE = 0.25` — a *fraction*. Left unconverted it still
compiles, still runs, and puts **every** configuration in the band, because
every colony count exceeds 0.25. That would have silently deleted the
discrimination check — the one that caught `log_slope` as a collapse detector —
rather than failing it. Now `1700.0`, the same ~25% bar against the ~6,725-world
beds.

**Two numbers measured under the fraction are still quoted and are not
comparable to anything measured after this change:**

- The `colonies@2000` screen's **ρ = 0.923** (CLAUDE.md §2) was calibrated
  against the fraction as ground truth. The proxy was always a count, so the
  correspondence should if anything improve — but the figure needs a re-run
  before it is quoted as current.
- **`mining_probe`'s +1.69 ± 0.53** recycling ratification is in fractions.

Both are re-runs, not edits, and neither gates anything today.

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

### T-41. Card flavour, names, and the slant cost ratios — R-P11

The 18 tier-0 slots carry no names, and their costs (0.5 / 0.8 / 1.2) are flat
placeholders that do **not** implement std §2's ratified cost-ratio spreads
(Floor 5:4:3, Default 3:2:1, Peak 4:2:1). The ordering is asserted in a test —
inscrutable < balanced < less-guarded, which design law #9's convexity needs —
but the ratios are not.

**Flavour text is the author's own** (CLAUDE.md §6), so the names are not
Claude's to write. The cost ratios are a separate, mechanical job and can land
without them.

### T-44. Confirm `trade_decay_lambda` on the ten-seed bed

Ratified at **0.01** (half-life 69 yr) on 3 seeds — an interior optimum, 39.0%
against 14.35% at λ=0. Direction and order of magnitude are not in doubt; the
precise value is, because three seeds is thin for a shipped constant and the
neighbouring points (0.005 → 35.2%, 0.02 → 36.9%) are close enough that seed
noise could move the peak.

Re-run `lambda_routing` over the ten-seed `TEST_BED_SEEDS` and refine between
0.005 and 0.02. Cheap, offline, and it closes a ratification that currently
rests on less evidence than the rest of the shipped defaults.

### T-45. Elasticity baseline — what the knobs actually do

First run of `examples/gradient_probe.rs` (3 seats, 4 CRN seeds, ±10% central
differences, 72 evaluations). Coverage `∂/∂ln x` in percentage points:

| knob | value | d/dln x | SE | verdict |
|---|---|---|---|---|
| `medium_fleet_size` | 3.0 | **+32.7** | 3.6 | raise |
| `biosphere_regen_rate` | 0.10 | **+19.7** | 3.2 | raise |
| `outpost_mining_fraction` | 0.20 | +14.5 | 5.8 | raise |
| `center_mining_fraction` | 0.15 | +8.2 | 4.3 | ~noise |
| `growth_rate` | 0.50 | +7.3 | **1.2** | raise |
| `survey_reserve` | 1024 | +6.3 | 3.6 | ~noise |
| `trade_decay_lambda` | 0.010 | +4.8 | 2.8 | ~noise |
| `rank.centrality_scale` | 150 | +4.6 | 3.0 | ~noise |
| `cargo_unit_size` | 5.0 | +0.0 | 0.0 | **inert** |

**Acted on**: the four significant knobs were moved jointly along the
normalised gradient, α = 0.5, verified at **+10.99 ± 1.86 points** paired
(38.26% → 49.25%) and ratified. α = 1.0 collapses to 6.9% — a cliff, because the
Medium hull's hold vanishes as the cost ladder narrows — so 0.5 is deliberately
short of the edge rather than at it. Chasing the last points toward a known
cliff on a four-seed bed is exactly the boundary-hugging that produced this
project's earlier artifacts.

Four readings worth keeping:

- **Four of nine knobs are inside 2 SE.** More than half of what this project
  has been sweeping cannot be told from noise at a four-seed bed. That is the
  headline, and it applies to every sweep taken before error bars existed.
- **`trade_decay_lambda` shows no gradient** — which is what a ratified optimum
  should look like, and an independent confirmation of the λ = 0.01 ratification
  arrived at by a completely different method.
- **`growth_rate` has by far the tightest SE (1.2).** Mid-pack elasticity but
  the most *reliably* measured knob, so it is where a small change is most
  confidently an improvement.
- **`biosphere_regen_rate` is the number-two lever and it is a placeholder**
  (R-O63/T-16). The dial that decides whether biological warfare is a strategy
  or a rounding error turns out to be load-bearing for the base economy too.

### T-16. R-O63 — the biosphere regrowth magnitude

**Previously measured as the largest single lever on coverage** (+141.2 ± 18.1
under the absolute-colony-count objective). **Treat that reading as suspect
until re-measured** — see the ablation below, which finds the biomass economy
completely slack at the shipped defaults. It remains the biological-warfare
dial, and it has never been tuned.

`SimConfig::biosphere_regen_rate` defaults to `0.10` of the remaining deficit
per cycle, a **placeholder**. It decides how long a razed world stays razed, and
therefore whether biological warfare is a strategy or a rounding error. Testable
directly: sweep it and measure how long a zeroed biosphere suppresses growth.

**Re-measure before acting — the operating point moved (R-O66).** Coverage fell
**−178.5 ± 26.9 colonies (−5.1%)** across the Band/kiloton separation, so every
gradient measured before that landing is consumed and this knob should be
re-probed at the new point rather than stepped along the old direction.
~~"suppresses `K`"~~ — it no longer touches `K` at all; it governs the *rate*.

**But note what that drop was not.** Ablation says **the biomass economy is
entirely slack at the shipped defaults**: deleting the growth draw outright
reproduces 3,294.0 colonies bit-identically, as does moving the regrowth
logistic onto living mass. The −178 came from `k_potential` no longer eroding,
which freed the deepening guard. So a sweep of this knob is measuring a
constraint that currently *never binds* — expect a flat gradient, and treat a
non-flat one as suspicious until ablated. Its value is as a **design** dial
(how durable is biological damage) and as the thing a Warfare card makes bind,
not as an economic lever on the baseline.

**Note also that `biosphere_regen_rate` is a design dial, not a free economic
knob.** It sets how durable biological damage is, which is a Greening/Warfare
balance question. A coverage-driven step on it trades design surface for
colonies, and that trade is the design owner's to make.

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

**Sharper since the Band ladder (R-MC15, `Hyades_mineral_cost_curve.md`
§2.6):** this is no longer only a two-way comparison. Reading Limited/Medium/
General as `Band I`/`II`/`III`, the Band step factors must each land in
`[4, 8]` — and **none of 1:3:9, the shipped 1:4.45:9, or this section's own
1:3.31:16 target satisfies that for both steps at once** (§2.6's table).
`limited_fleet_size = 16` clears the `Band I → II` step (`F₁ ≈ 4.83`) but its
own `Band II → III` step (`F₂ = medium_fleet_size = 3.31`) falls just short
of the floor — so hitting this target does not, by itself, close R-MC15. The
search should optimize jointly against both the coverage objective and the
`[4, 8]` constraint, not just the radius prediction.

### T-20. Raise coverage inside a fixed 4,000-year run

> **The objective is now an absolute colony count** (CLAUDE.md §2). The old
> fraction let a habitability-derived denominator into the score, which
> terraforming and bombardment cards would both have moved *against* the play.
> Shipped defaults measure **3,472 ± 24 colonies** on the 4-seed CRN bed.
>
> **Every knob was re-measured against the corrected metric, and the answer is
> that the defaults stay.** Not for want of looking — three candidates were
> examined and each was rejected on its own evidence:
>
> | candidate | measured | why it was not taken |
> |---|---|---|
> | `biosphere_regen_rate` + `growth_rate` together | **+6.2 ± 1.8** (3.4 SE) | real but **+0.18%**, and unobtainable without raising the R-O63 design dial 56% |
> | `survey_reserve` → lower | probe said −23.8 ± 10.1 | **refuted by direct sweep** — 1024 is a plateau, the cliff is *below* it |
> | `medium_fleet_size` → lower | −62.5 ± 26.3 (2.4 SE) | three prior measurements put 4.45 on its peak |
>
> **The attribution is the interesting part and it reversed under the corrected
> metric.** `growth_rate` alone is **negative** (−12.0 ± 11.0) and
> `biosphere_regen_rate` alone is noise (+1.5 ± 1.8), yet together they are
> +6.2 ± 1.8. They are **complements, not substitutes** — the opposite of what
> the fraction reported. Population consumes biosphere 1:1 (design law #11), so
> raising growth without funding the regrowth starves the ecology that caps
> `K`. That is also the mechanism behind the starvation cliff, now explained
> rather than merely bounded.
>
> **So the honest reading of this objective is that it is finished as a tuning
> target.** The best available step is 0.18% and costs design surface; the
> ceiling is `k_high` (R-AC18), not the economy. Further work belongs on the
> classifier or on a new mechanism, not on these knobs.
>
> ---
>
> **Reopened by R-O66 — the bed no longer saturates, so time is binding again.**
>
> That verdict rested on a bed that finished: at the old defaults the run took
> **99.8%** of everything `k_high` admits (99.7 · 99.9 · 99.7 · 99.7), so the
> only headroom left really was the classifier. Post-R-O66 it takes **94.6%**
> (93.9 · 95.9 · 92.6 · 96.0). The missing ~5.4 points is ~190 colonies —
> essentially all of the 178 the unit fix cost — and it is *reachable* headroom,
> not classifier headroom.
>
> `examples/reach_limit.rs` on the standard bed says what is consuming it:
>
> | limiter | evidence | binding? |
> |---|---|---|
> | classification (`k_high`) | 47–48% of the galaxy permanently ineligible; the set is now exactly fixed (`gate_erosion = 0`) | **yes, on the total** |
> | **expansion-loop time constant** | founding rate ×~2 per 500 yr, peaks at 3,000–3,500 yr on all four seeds, turns over only in the last bucket | **yes, on the time** |
> | survey | 11–41 above-gate worlds unscanned per seed (0.3–1.2%) | no |
> | biomass economy | deleting the growth draw outright is bit-identical | no |
> | mineral economy | ruled out at R-AC17 | no |
>
> The residual is **126–216 worlds per seed that were scanned and simply not
> reached in time**, so this is a rate problem, not a discovery or supply
> problem.
>
> **Where to look, and it is not an economic knob.** R-O66's entire measured
> effect was a *reallocation*: a corrected `k_potential` gives centers real
> deepening headroom and they take it (mean infra 1.420 → 1.443, mean `K`
> 1.418 → 1.430). So the time constant is set by the deepen-versus-expand
> split — `expand_bias` and the `infra < k_potential` guard — which is policy,
> tunable, and has never been probed against the objective. Note the two knobs
> pull against each other by construction: deepening buys production that
> compounds, expansion buys the centers that do the producing, so the optimum is
> interior and a gradient probe is the right instrument rather than a sweep.
>
> **Do not read this as "the unit fix was a regression."** The old 99.8% was
> partly bought by an artificial cap on deepening; the bed saturated because
> centers were forbidden from investing. What changed is that the *question*
> went back to being interesting.

**3,294 colonies / ~49.0%** of the objective set (4-seed CRN mean) as of R-O66;
**~51.4%** before it, after **four** ratifications:
`trade_decay_lambda = 0.01` (a *missing term* — routing had no distance
component) took it from 14.4% to 38.3%; a verified gradient step on four knobs
took it to 49.3%; **R-AC19 (mining-pair recycling)** added +1.69 ± 0.53 on the
8-seed bed; and **`growth_rate` 0.546 → 0.873** added +2.31 ± 1.05 on the
standard four. **None came from a coordinate sweep** — two missing terms and
two measured gradients.

> **Measured — and they are strongly sub-additive, as the `k_high` finding
> predicted.** The two gains were developed on parallel branches and each was
> measured *without* the other: recycling's +1.69 (8 seeds) / +1.19 (standard
> four) against `growth_rate = 0.546`, and `growth_rate`'s +2.31 against
> recycling off. On the merged defaults, standard four seeds:
>
> | configuration | coverage | vs. neither |
> |---|---|---|
> | neither | 49.11% | — |
> | recycling only | 50.30% | +1.19 |
> | `growth_rate` only | 51.42% | +2.31 |
> | **both (shipped now)** | **51.64% ± 0.36** | **+2.53** |
> | *additive prediction* | *52.61%* | *+3.50* |
>
> **The sum falls ~1 point short of additive, and recycling's marginal
> contribution on top of `growth_rate` is +0.22 — not resolvable on this bed.**
> Symmetrically, `growth_rate` keeps only +1.34 of its +2.31 once recycling is
> on. They are substitutes competing for the same headroom, the same shape the
> `growth_rate`/`biosphere_regen_rate` pair showed.
>
> **Why, and it is the useful part:** `examples/reach_limit.rs` found the
> binding constraint here is **`k_high`, not the economy** — the bed already
> colonizes 90–100% of what the threshold *admits*, and it admits only 51–53%
> of the galaxy. Both gains are economic, and the economy is not what is short.
> Adding a third economic improvement should be expected to buy near nothing.
>
> **This is not an argument against recycling**, which was ratified on its own
> 8-seed evidence and buys a real hull economy regardless of coverage (29%
> fewer hulls for the same work, idle hull-years 2.77 M → 1.24 M). It is an
> argument that **the coverage objective is now capped by classification, not
> production** — so the next move is R-AC18 (should the Colony class have a K
> floor at all), not another economic knob. The marginal figures above sit
> inside the 4-seed noise; `SEEDS_WIDE` would settle them, but no pending
> decision turns on it.
**The third one is the methodological point:** it came from re-running the
probe *at the operating point the second step had produced*. The old direction
had been spent — of nine knobs, four measured flat, two were inside 2 SE, and
`medium_fleet_size` had gone from the biggest lever (+32.7 pts/ln at its old
value) to sitting on its peak (`+1.93 ± 2.37`, sign unresolvable). Only
`growth_rate` and `biosphere_regen_rate` survived, and `--attribution` showed
`growth_rate` earned 92% of the joint gain by itself, so the R-O63 biosphere
design dial was left untouched.

**The remaining headroom is not obviously in these knobs, and the returns are
visibly diminishing** (+23.9, +11.0, +2.3 points). Two cliffs now bound this
ray: the original one at α = 1.0 of the *old* direction (6.9% as the Medium
hull's hold vanishes), and a **starvation cliff** on `growth_rate` itself —
1.395 still works (50.99%) but 2.229 collapses to 28.46%, because population
consumes biosphere 1:1 (L6) and fast enough growth eats the ecology that caps
`K`. Guarded by `growth_rate_stays_clear_of_the_starvation_cliff`. Further
progress likely needs a new *term* rather than a better value — the λ lesson
again. **4,000 years is
the run length; the coverage reached within it is the objective.** Do not extend
the horizon: doubling it doubles every trial and the 60-second rule already had
to absorb the snowball once. T-07 and T-21 are the nearest levers.

**The limiter is now measured, and it is `k_high` — not the economy, not
survey, not the horizon** (`examples/reach_limit.rs`, standard 4-seed bed,
3 seats, 4,000 yr). The driver partitions the target set instead of sweeping
it, so every target lands in exactly one bucket and the biggest bucket is the
binding constraint:

| seed | targets | above `k_high` | colonized | **share of the reachable set** | unscanned above the gate | scanned, still unclaimed |
|---|---|---|---|---|---|---|
| 1 | 6,725 | 3,435 (51.1%) | 3,381 | **98.4%** | 0 | 54 |
| 7 | 6,725 | 3,467 (51.6%) | 3,460 | **99.8%** | 0 | 7 |
| 42 | 6,725 | 3,471 (51.6%) | 3,311 | **95.4%** | 0 | 160 |
| 31337 | 6,725 | 3,551 (52.8%) | 3,379 | **95.2%** | 0 | 172 |

Measured at the current defaults, i.e. **after** R-AC19 (mining-pair recycling,
autopilot §5a) — which moved the reachable-set share up from 97.5 / 99.6 / 89.6
/ 93.0 and closed the last unscanned worlds. It **tightens this conclusion
rather than changing it**: the +1.69 points landed inside a set that was already
90–100% taken, so the classification ceiling is now the only thing of any size
still holding coverage down.

Read the fifth column, not the fourth. The run already takes **90–100% of
everything the ranking makes takeable**; the ~50% is `rank` classifying the
low-K half of the galaxy as *Mining outpost* or *Barren*, so a colonizer is
never dispatched there — `production_choice` and `assign_role` both draw only
from `PlanetClass::Colony` / `ProductionCenter`, and `sim.rs` drops Barren
worlds before they reach the candidate list at all. Survey is not the limiter
(0–2 unscanned worlds above the gate on the whole bed), and neither is the
mineral economy.

**This also resolves the apparent contradiction with
`Hyades_autopilot_colonization_growth.md` §6**, which reports **100% of
colonizable worlds** on the same bed. Both numbers are right and the
denominators differ: that "colonizable" is the above-gate set (3,435 on seed 1
— the same count this driver measures), while the coverage objective's is
`min(hab, bio_max) > 0.01`, which is effectively every planet in the galaxy. The
gap between them is not work left undone; it is a class of world the baseline
policy declines to settle. Three of §6's four counts reproduce here exactly
(3,435 · 3,467 · 3,471); its fourth reads 3,516 against this driver's 3,551 for
seed 31337, which is **unexplained and small** — galaxy generation is
deterministic per seed and neither `max_survey_hops` nor the horizon touches
it, so the likeliest causes are a different fourth seed or a generator change
since that run.

~~**A side finding: the gate is not a fixed set.** 240 / 207 / 286 / 275
above-gate worlds per seed (1 / 7 / 42 / 31337) end the run *below* `k_high`,
because population is paid for out of biosphere (L6) and
`k_potential = min(hab, bio)` — a settled world can drop out of the class that
made it settleable.~~

**Withdrawn (R-O66): that was the unit error, not a finding.** `k_potential`
was taking a minimum of the biosphere's *standing mass in kilotons* against two
Band levels, so a world's classification fell as its own population ate it.
With Bands and kilotons separated the gate reads `bio_max`, which nothing in
the shipped engine moves, and `reach_limit`'s `gate_erosion` counter is
structurally zero — **the gate is a fixed set.** The counter is kept as a guard
rather than deleted, because the first card that lowers a world's pristine
biosphere makes the denominator playable again.

**Where the headroom actually is.** Counting mining outposts as reach, the bed
covers **81.0–84.0%** of targets — two thirds of the sub-gate worlds are
already visited, just not settled. The genuinely untouched remainder is 16–21%
of targets, and on seed 1 it is 1,139 worlds of which only 54 sit above the
gate: the rest are low-K *and* too mineral-poor to mine, i.e. Barren to
everyone, and no amount of economy or horizon reaches them.

The ceiling as a function of the threshold (galaxy alone, no run, seed 1):

| `k_high` | 1.5 | 2.0 | 2.5 | 3.0 | **3.2** | 3.5 | 4.0 |
|---|---|---|---|---|---|---|---|
| colonizable share | 99.9% | 97.8% | 88.0% | 63.5% | **51.1%** | 31.8% | 7.1% |
| minable worlds left below the gate | 9 | 106 | 478 | 1,039 | **1,255** | 1,488 | 1,684 |

3.2 sits on the steepest part of that curve — 3.0 alone would raise the ceiling
12.4 points — and the second row is why it cannot simply be lowered: `k_high`
*also* defines the Mining-outpost class, and R-AC17 is the record of what
happens when that class is starved (zero mining pairs, zero freighter legs,
expansion stalled at 41 colonies). Lowering the gate buys ceiling and sells
economy, so the two must move together or not at all. **R-AC18** puts the
design question rather than guessing at a value: should the Colony class have a
K floor at all, or should `k_high` order *preference* rather than gate
*eligibility*?

### T-47. What is left on the mining surface after R-AC19

Recycling (autopilot §5a) is landed and is worth +1.69 ± 0.53 points of
colonies. What it leaves open is smaller and specific:

- **`rank.mineral_high` is the one live knob** — +3.92 ± 1.86 on the standard
  bed, which clears 2 SE by a hair and therefore wants the 8-seed bed before
  anyone moves it. Raising it mines *fewer, richer* rocks; note that is the
  opposite direction from R-AC17's failure, so the two ends of this threshold
  are not symmetric and a step needs its own verification rather than an
  extrapolation.
- **Which rock a recycled pair is sent to is the producing center's choice,
  not the pair's.** The center picks the target by rank and the hulls fly from
  wherever the dead rock left them, which can be most of a galaxy away. Letting
  the pair pick the nearest outpost candidate itself would shorten the flights,
  but it costs a candidate scan per re-tasking — the §4 locality rule, so it
  needs measuring rather than assuming.
- **A recycled pair is only taken when a center orders one.** A pool of idle
  hulls does not itself provoke an outpost; if mining loses the score
  comparison to a colony target the center saves for the colonizer, and the
  free pair keeps waiting. Whether that is correct is a doctrine question.

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

Not an argument against §7.1; an ordering constraint.

**The block is now partly lifted.** Technology's three tier-0 cards are
`UnlockDesign` writes, and card 12 unlocks `MediumSystems` — the hull the
colonizer and freighter ride on, and the exact thing whose absence made
enforcement fatal. What is still missing is a *policy* that plays it: the
baseline autopilot passes every round by design (T-30), so enforcement would
still halt expansion until some autopilot buys the unlock. That is now a
policy question, not an architecture one.

### T-46. Habitability's gravity/radiation Bands as population-health statistics — R-H7/R-H8

`Hyades_habitability.md` §2.3–2.4 decides gravity and radiation should each
reduce to a **Band Empty–IV** population-health statistic (LD50-like — a
mortality/fertility/cardiovascular threshold crossed at each Band edge)
rather than a raw g-value or dosage number the player reasons about
directly. The reframing is decided; what it needs is a design pass: (1)
whether such a statistic actually compounds multiplicatively the way mass
and cost do, which is what the shared `[4, 8]` Band step-factor constraint
(`Hyades_mineral_cost_curve.md` §2.6) assumes of every Banded quantity —
unlike population or hull size, there is no obvious physical reason a
lethality curve should have that shape; (2) the actual threshold magnitudes
(R-H1). Blocked on habitability.md's own implementation, which is itself
still unlanded in code.

### T-37. Everything in netcode outside the crate

Topology (§3), the 144-byte frame (§4), genesis assembly (§7), relay and
reconnection (§9), server posture (§10), client hardening (§11). **None of it
blocks on the engine** and none of it lives in this repo — it is client and
service work whose only contract with `hyades-engine` is determinism, one
inbound entry point, and no host access, all of which hold today (see the
netcode engine-status block). Listed so it is tracked somewhere; it needs a home
before it needs a design.

### T-26. R-SIM4 — departure-traffic confidence

R-SIM3 settled that occupancy is inferable at range, and the pop-Band-IV
industrial signature is implemented exactly with no new state. The **graded** signal is
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
