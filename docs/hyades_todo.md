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

### T-47. R-AC18 — ~~time-to-10% vs. coverage disagree on `medium_fleet_size`~~ premise withdrawn; `center_mining_fraction` still open

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
late-game sprawl — flagged, not resolved, per R-AC18.

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

**Now known to be the second-largest lever on coverage** (+19.7 ± 3.2, T-45),
which raises its priority considerably: it is not only the biological-warfare
dial, it is a first-order economic parameter that has never been tuned.

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

**~49%** of colonizable worlds (3,348 of 6,725, seed 1) after two ratifications:
`trade_decay_lambda = 0.01` (a *missing term* — routing had no distance
component) took it from 14.4% to 38.3%, and a verified gradient step on four
knobs took it to 49.3%. Neither came from a coordinate sweep.

**The remaining headroom is not obviously in these knobs.** The gradient step
found a cliff at α = 1.0 (coverage collapses to 6.9% as the Medium hull's hold
vanishes), so this ray is close to exhausted. Further progress likely needs a
new *term* rather than a better value — the λ lesson again. **4,000 years is
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
reduce to a **Band 0–IV** population-health statistic (LD50-like — a
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
