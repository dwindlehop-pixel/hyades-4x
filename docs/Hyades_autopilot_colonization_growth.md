# Hyades — Expansion and Growth

*The autopilot the simulation runs for **The Far Shore** (Expansion) and **The
Greening** (Growth): survey, planet classification, colonization, mining
outposts, freight, and the production decision. Companion to
`Hyades_simulation_model.md`, `Hyades_galaxy_and_autopilot.md` (world model),
`Hyades_industry.md` (the economy this spends), `Hyades_trees_and_card_value.md`
(what each tree is scored on) and `Hyades_card_contract.md` (card law). Calls
flagged **R-ACn**.*

**Rev 4.** Rewritten to carry **ratified decisions and open decisions only**
(`CLAUDE.md` §6). The measurement record that used to be inlined here — R-AC3's
survey sweep, R-AC19's recycling passes, R-O66's ablations, R-O68's dead branch,
R-O86's 99% waste, the `survey_reserve` plateau, and the withdrawn R-AC20 sign
conflict — is in **`Hyades_experiments_appendix.md` §A**, linked per decision.
Rev 4 also reconciles the spec with everything landed through T-102: the
`K = min(hab, bio_max)` amendment, the five-year economy tick, the closed-form
logistic, the shell/drive model, and freight's two color fixes.

---

## 0. How to read this file

Every numbered item below is one of exactly two things:

- **`RATIFIED`** — the engine must honor it. Magnitude and unit are stated, and
  the value is marked **confirmed** (Monte-Carlo ratified on a named bed) or
  **placeholder** (shipped because something had to be).
- **`OPEN`** — not settled. States what would settle it. A recommendation with
  no ratification behind it is an open decision with a recommendation attached,
  not a third category.

Evidence is a link, never an inlined table. Where a decision has a measurement
behind it, the appendix section is named in the same line.

---

## 1. Information model

**1.1 `RATIFIED` — the command view is omniscient over realized state; the
simulation is not.** Planets, structures and fleets are fully visible to the
*player* for planning. Hidden simultaneous *orders* are not realized state and
stay concealed until they resolve. The *simulation* acts only on light-lagged,
player-relative knowledge (design law #15). **R-AC1 resolved.**

**1.2 `RATIFIED` — the simulation is continuous 3D with no hexes.** Each star
system is one point. Cards target hexes (the player decides hex by hex); the
autopilot targets planets (execution). A hex-targeted card resolves onto the
planets the hex contains. **R-AC2 resolved.**

**1.3 `RATIFIED` — three scan tiers.**

| Tier | What it yields | Requires |
|---|---|---|
| **Remote** | biosphere, habitability | spectroscopy at interstellar range |
| **Close** | ownership, infrastructure, mineral density | a survey craft on site |
| **Inferential** | a pop-Band-IV world radiates the waste heat of billions, so it is legible as *occupied* from home — but not *whose* | nothing; off by default |

Acting on the inferential tier is `Doctrine::survey_avoids_inhabited`, **default
off** — early game, an empire flies out and finds out. **R-SIM3.**

**1.4 `RATIFIED` — every reaction is light-lagged.** Cards issue instant global
orders; consequences propagate at `c`. A response to an observation `N` light-years
away is queued `N` years in the future.

**1.5 `OPEN` — R-SIM4: departure traffic as a second inferential signal**,
graded by repeat sightings. No formulation.

---

## 2. Survey

**2.1 `RATIFIED` — the opening is six Light Vehicles on the six cube-face
headings** (±X/±Y/±Z), free starting units. `Doctrine::survey_vehicles = 6`.
**Placeholder magnitude.** A survey craft flies at its own Design's drive, like
every hull (§8.4): an empty LSV at 1.00 g, an empty LCV at 0.911 g. There is no
survey rate in Doctrine — the author's ruling that the sim does not overwrite a
ship's Design removed `survey_accel_g` (appendix §D.19).

**2.2 `RATIFIED` — every later survey craft is a paid build** from a production
center, charged to that center's stockpile. Each chain ends after
`max_survey_hops = 120` worlds and the craft scraps at the nearest friendly
colony.

This is the mechanism that makes expansion compound. Without replenishment,
total exploration is the **fixed product** `players × survey_vehicles ×
max_survey_hops`, which no economy knob can move.

**2.3 `RATIFIED` — the survey candidate list is fog-limited by construction.**
`Autopilot::choose_survey_target` receives `SurveyView { id, position,
habitability, biosphere, industrial_signature }` — §1.3's remote tier plus the
one inferential signal. It is **not** filtered on ownership, which is a close-scan
fact. Knowledge is one set per *player*, not per craft.

**2.4 `RATIFIED` — `SurveyStrategy::OpeningSectors`.** The six bootstrap craft
keep a soft cube-face preference for their whole hop chain; every later paid
Scout pools globally. **R-AC3 resolved: no significant difference between the
three candidates** (all within 2 SE on a 4-seed CRN bed), because survey was
never the binding constraint — appendix §A.1. Ships unchanged, since every
existing coverage number was measured against it.

**2.5 `RATIFIED` — a scout is built only when there is somewhere to scout.**
The precondition is `ProductionContext::survey_frontier` — worlds no craft has
been dispatched to, the set `launch_survey` actually picks from, counted `O(1)`
off `VisitedMask`. `apply_build_with` declines a Scout order on an empty
frontier.

**R-O86, and it is the largest single engine speedup this project has
measured** — 5.6× throughput, colony count *identical*, colony-years +0.007% —
because 99.0% of all production was issuing builds that created no object.
Appendix §A.4.

**2.6 `RATIFIED but inert` — `Doctrine::survey_reserve = 1024`.** The frontier
size a center tries to keep ahead of itself before spending an otherwise-idle
cycle on a scout. **The direction is ratified and the magnitude does not reach
the simulation**: it is compared against `candidate_count`, whose median is 0 and
whose maximum over a run is 164, so every value above ~200 is bit-identical.
**R-AC16 resolved; superseded in effect by 2.5.** Appendix §A.3.

**2.7 `OPEN` — R-AC4: scan dwell time and close-scan range.** Both are currently
zero and unbounded respectively. Settled by a decision, not a measurement.

---

## 3. Planet ranking and classification

**3.1 `RATIFIED` — every close-scanned planet gets a numeric rank, and its class
follows from where the score and its components land.**

| Class | Signature | Handled by |
|---|---|---|
| **Production center** | high `k_potential` **and** hub value | colony vehicle (§4) |
| **Colony** | high `k_potential`, weak hub value | colony vehicle (§4) |
| **Mining outpost** | high mineral density, low `k_potential` — *can out-rank a colony* | miner + freighter (§5) |
| **Barren** | low on all | ignored |

Because rank is numeric, all targeting is a deterministic **argmax**
(card-contract §6), and **cards re-rank retroactively**: terraforming lifts
`hab`/`bio_max` and a Barren world becomes a colony; prospecting raises mineral
value and a colony becomes an outpost.

**3.2 `RATIFIED` — `k_potential = min(hab, bio_max)`, a minimum over Bands,
reading the biosphere's *pristine* ceiling.** Infrastructure is **not** a term
(`Hyades_industry.md` §1.1): it is the industrial stock, it can be razed, and
razing it must not move population. **R-O66** — the earlier `min(hab, bio, infra)`
was a `min` across incompatible units that typechecked and read as plausible
ecology. Appendix §A.9.

**3.3 `RATIFIED` — the rank components and weights.** `RankWeights`, all
**placeholder magnitudes except `k_high`**:

| field | value | meaning |
|---|---|---|
| `w_k` | 1.0 | weight on `k_potential` |
| `w_mineral` | 0.8 | weight on the per-color Band sum |
| `w_hub` | 1.2 | weight on hub value |
| `k_high` | **Band 3.2** | at/above ⇒ high-K (center or colony); below ⇒ a mineral-rich world is an outpost |
| `mineral_high` | 2.0 | at/above ⇒ a low-K world is an outpost |
| `hub_high` | Band 0.8 | at/above ⇒ a high-K world is a production center |
| `centrality_scale` | 150.0 ly | decay scale of centrality to holdings |
| `mineral_pressure_gain` | 1.0 | gain on the scarcity term |

**`k_high = 3.2` is confirmed and is a knife-edge, not a slope** — ±25% collapses
coverage in *both* directions. **R-AC17 resolved**; appendix §A.7, §A.2.

**3.4 `RATIFIED` — centrality is evaluated by `transcendental::exp_fast`,
four multiplies, relative error 7.5e-5** (T-130, author's four-multiply
budget). It supersedes T-102's `math::exp_decay` (degree 7, 5.4e-7, eight
multiplies), which is deleted: under the budget a single degree-4 fit over the
measured `[−2, 0]` reaches 5.0e-4, and the range-reduced `2^f` form reaches
7.5e-5. Built from IEEE-exact operations, so bit-identical native and wasm32.
Appendix §D.8.

**3.5 `RATIFIED` — the mineral term reads Bands, memoized on the field's own
bits.** `rank` scores ore as `Σ_c scarcity_c · Band(m_c)`; `PlanetView` carries
`mineral_bands: [f64; 3]` rather than a `MineralField`, because nothing in the
seam read the masses (T-100).

**3.6 `OPEN` — R-AC5: the rank formula's shape and the class thresholds**, as
distinct from the weights above. Five of six mining-side knobs measure as noise
(appendix §A.5), which says what is left on this surface is *terms*, not values.

**3.7 `OPEN` — R-AC6: do provisional ranks exist?** Should ranking read remote
data (hab/bio) before a close scan, giving ranks that firm up on visit? Today it
does not.

**3.8 `OPEN` — R-AC18: should the Colony class carry a `K` floor at all**, or
should `k_high` order *preference* rather than gate *eligibility*? `k_high`
permanently excludes 47–48% of the galaxy, which is the whole of T-20's gap
between "colonisable" and the coverage objective's denominator. Lowering the gate
to widen the ceiling directly shrinks the Mining-outpost class that funds
expansion — R-AC17 run backwards. **The two knobs must move together.**

**3.9 `OPEN` — the scarcity vector is written once at game start and never
again.** `scarcity_c` comes from the homeworld archetype, so selection can say
*mine more* and never *mine Cyan*. Replacing it with the deciding center's live
shortfall was **implemented, measured and reverted** (−3.30% ± 0.49 colony-years,
0/4 seeds) — the decision was blind and had nothing to see. Appendix §A.12. The
defect is real and remains; what is open is whether it matters anywhere.

---

## 4. Colonization

**4.1 `RATIFIED` — production centers first, colonies next, always by descending
rank.** `ExpandBias::ProductionCentersFirst`. A colony vehicle targets the
highest-rank production-center-class planet; if none remain unclaimed, the
highest-rank colony-class planet. On arrival it founds a colony, which begins its
own production schedule (§6).

**4.2 `RATIFIED` — a colonizer's hold is one kiloton budget carrying a mix of
settlers and minerals.** Whatever volume the people do not fill leaves with
minerals **out of the founding center's own bank** and lands in the new colony's
stockpile. That is a transfer a parent paid for, not a grant. **R-O74** —
supersedes "no mineral seed for colonies".

**4.3 `RATIFIED` — settlers are conserved.** Founding *moves people that already
exist*; it does not conjure them. Until R-O74 landed, a colonizer's settlers were
written into its hold with nothing debited anywhere — one exemption from design
law #11, on the exact path the expansion loop runs on. **An exemption from
conservation is not a modeling shortcut, it is a free resource, and a search
will find it and call it a strategy.**

**4.4 `RATIFIED` — `colony_seed_pop = BandTier::I`.** Typed as a rung, not a
number. **Placeholder magnitude.**

**4.5 `RATIFIED` — `ColonizerPolicy::CheapestViable`.** The cheapest hull that
can found at all: ten Mediums make ten colonies, each with its own `K` and its
own growth curve, where one General makes one colony starting further up a curve
it would have climbed anyway. **`BiggestSeedPerMineral` is measured and
deliberately not shipped** — it scores +13.97% colony-years on a bit-identical
colony count, and two ablations put the entire effect on the **seed mass**
(`Hyades_industry.md` §1.6), not on the hull, its price or transit.

**4.6 `OPEN` — R-AC7: colonizer cost, build time and founding infrastructure** as
independent magnitudes. Today all three are derived — cost from the hull ladder,
build time from hull mass, and `founding_infra = hull_cost` because a hull's mass
*is* its cost (design law #11).

**4.7 `OPEN` — R-AC8: contested claims.** What happens when two empires target
the same planet. Presumed resolved by arrival time under light-lag; not
specified, and **design law #15 is at risk here** — T-34 records that
colonization currently filters on instantaneous global ownership.

---

## 5. Mining outposts and freight

**5.1 `RATIFIED` — an outpost is worked, not colonized, and it is unowned.** No
`owner` component is ever set on a worked rock. `outpost_stock` is keyed
`(player, rock)`, so **each empire holds its own pile at the same body** and each
player's crew works the shared rock into that player's own pile. Measured on the
standard bed, **2,226 of 2,494 worked sites are cross-player.**

**5.2 `RATIFIED` — a mining pair is a miner plus a freighter**, built by the
nearest production center. The miner extracts; the freighter hauls to a center.

**5.3 `RATIFIED` — crew size is derived from demand, not configured.**
`miners_per_outpost` was ratified at 3 (+2.74% colony-years, every seed positive)
and **retired at T-72**; its replacement retired at T-87. The measurement no
longer applies because it was taken under a law with no deposit term at all.

**5.4 `RATIFIED` — extraction is sublinear in crew and scales with the deposit.**
`outpost_mining_fraction = 0.238` is the fraction **one miner** works;
`crowding_beta = 0.5`; `veins_per_band = 10.0`; `mining_tick_years = 50.0`;
`density_floor = 0.01`. Law and normalization in `Hyades_industry.md` §4.3/§4.3b.
**Placeholder magnitudes except the crowding law's shape.**

**5.5 `RATIFIED` — an exhausted pair goes to Reserve and is re-tasked, not
stranded.** `SimConfig::recycle_mining_pairs`, **default on**. The next center
ordering a pair takes the reserved hulls nearest its target — no minerals, no
build delay, only the flight. **+1.69 ± 0.53 over eight seeds, positive on all
eight.** **R-AC19 resolved**; appendix §A.6.

Two things it fixed that are separately load-bearing: the miner and the freighter
now use **the same exhaustion predicate** (they did not, so haulers flew empty
round trips for the rest of the match), and the build decision **prices the pair
it is actually going to buy** (it quoted the full price even when hulls sat in
Reserve).

**5.6 `RATIFIED` — freight routes by distance-discounted need.**
`argmax` over owned centers of `mineral_pressure(center) · exp(−λ · t_transit)`,
`trade_decay_lambda = 0.01` (half-life 69 yr). **`λ = 0` reduces exactly to
`most_needed_center`**, which design law #5 keeps as the single-supply oracle, so
one function checks two independent degeneracies. **Confirmed on 3 seeds** —
thin for a ratified constant; appendix §B.1.

> **Cross-tree conflict, open:** λ is +0.002 on Expansion and **−0.348 on
> Growth**. It is the largest ratification in this project's history and it was
> measured on coverage alone.

**5.7 `RATIFIED` — a hold is filled against the destination's color deficit**,
not in proportion to the pile the hauler happens to be standing on. **+8.40% ±
1.86 work-years, 8/8 seeds, replicated on four the candidate was not chosen
against**, on *the same tonnage* and the same trips — only the colors in the
hold changed. **R-O89.**

**5.8 `RATIFIED` — one outbound leg may visit two piles.**
`max_pickup_stops = 2`. A hold used to be filled from one map entry, so **every
delivery was mono-colored by construction** and no routing fix could reach it.
**+55.13% ± 4.65 work-years, 8/8 seeds** (**R-O92**), and the intermediate stop
caps each color at what is wanted **and** at its proportional share of the
hold — the second cap is worth +10.2% on its own and is inert at today's
magnitudes, load-bearing at the ones development reaches.

**Two is a peak, not merely better than one**: one stop through six scores
184k / **284k** / 261k / 236k / 190k work-years.

**5.9 `RATIFIED` — a hauler's hull is sized to its rock, under a liquidity
cap.** `freighter_hull` scores candidate hulls on `load / round_trip / hull_cost`
where load is `min(supply, demand)`, and **considers only hulls the center can
pay for now**. **+170.1% ± 16.2 work-years, 8/8 seeds; +9.54% colony-years, 7/8**
(**R-O94/T-98**). Without the liquidity term the same change is +51% work-years
and **−17% colony-years on 1/8 seeds** — a development gain bought out of the
expansion loop.

The mechanism generalises: **a score of the form `value / cost` is a rate, true
in steady state, and says nothing about the years spent saving for an indivisible
purchase.**

**5.10 `OPEN` — R-AC9: "nearest production center" under light-lag** — true
nearest or nearest-known. Today true nearest, which is a design law #15 concern
of the same family as T-34.

**5.11 `OPEN` — R-AC10: mining rate, freighter capacity and cadence** as ratified
magnitudes rather than derived placeholders.

**5.12 `OPEN` — freight is 1.73% → 14.70% of bank inflow and the rest is a center
mining its own single-colored planet straight into its own bank** (T-92). That
is the next constraint on the mineral economy and it is not a freight problem.

---

## 6. The production decision — Growth

**6.1 `RATIFIED` — a center's *economy* is cadence-driven and its *decision* is
not.** Mining and growth are rates over an interval, and an interval is what a
rate needs: `cycle_years = 5.0`. The **decision** is an event — `BuildDecision`,
raised when the yard clears `build_years` after a build was committed — because a
decision is not a rate. **R-O69**, +165.8 colonies (+5.0%).

A declined build schedules `decision_retry_years = 50.0`.

**6.2 `RATIFIED` — every rate is denominated per `rate_reference_years = 50.0`,
not per tick.** `tick_scale` multiplies each rate by `cycle_years /
rate_reference_years`, which is exactly `1.0` at the cadence they were ratified
at — so refining the tick is bit-identical on the shipped bed and **no
Monte-Carlo-tuned magnitude moves.** **T-88.** Before it, shrinking the tick did
not integrate the same economy more finely, it ran a fifty-times-faster one.

**If a knob's doc comment says `1/cycle`, changing the cycle changes the knob.**

**6.3 `RATIFIED` — population growth is the closed-form logistic, not an Euler
step.**

```text
x(t+Δ) = K·x / ( x + (K − x)·e^(−rΔ) )
```

The ceiling is constant across a tick, so the step is autonomous and solvable —
and `settler_target` had been pricing colonization off this solution's *inverse*
since R-IND11, so the policy and the economy were following different curves.
**T-94/R-O93**, +6.57% ± 1.14 colony-years, 8/8 seeds, at no throughput cost.

Two things it retires: the **`r < 2` bifurcation ceiling** was a property of the
Euler map and not of the model (`e^(−rΔ) ∈ (0,1)` at every positive `r`, so the
map is monotone at any rate), and the **undershoot below `K`** — an over-capacity
world now decays *toward* the ceiling rather than overshooting to zero. The
collapse was design content; the undershoot was truncation error whose severity
was a function of `cycle_years`, so **the outcome of an attack on a world's
habitability was being set by a performance knob.**

**6.4 `RATIFIED` — every logistic runs on the mass, not on the Band reading.**
`growth_rate = 0.873` per `rate_reference_years`, **confirmed** (R-O84, carried
through T-94 rather than re-ratified). Population growth is paid for out of
biosphere; biosphere regrows logistically toward `bio_max` at
`biosphere_regen_rate = 0.127`, **and that knob is bit-identically inert at the
current operating point** — since `K = min(hab, bio_max)` the ceiling is the
pristine biosphere and regrowth only refills the standing stock.

**6.5 `RATIFIED` — tier gates.** A center's development level decides which hull
classes it may build: `limited_min_level = BandTier::II`,
`medium_min_level = BandTier::II`, all classes at Band IV. Both are config;
**placeholder magnitudes**, and both are `u8`-valued, which is why an
all-continuous gradient probe has never ranked them.

**6.6 `RATIFIED` — the preference order within a decision.**

1. **Deepen** toward `k_potential`, if `reinvest_bias` favors depth and the
   upgrade is funded.
2. **Expand** — colonizer or mining pair, by rank (§§4–5) — if funded.
3. **Survey**, as a *fallback* for a decision that would otherwise be Idle, when
   there is unexplored galaxy left (§2.5) and a scout is affordable.
4. **Idle** — save toward whichever of the above was preferred but unfunded.

**Survey is a fallback, never a pre-emption**, and that ordering is load-bearing:
an earlier revision gave survey priority whenever the frontier was thin, which
made `survey_reserve` non-monotonic — on seed 1 a reserve of 256 reached 1,047
colonies while 4,096 collapsed to **3**, because every center scouted every cycle
and none ever colonized.

**6.7 `RATIFIED` — deepening uses headroom, not whole levels.** The guard is
`infra < k_potential`, not `infra + 1 <= k_potential`. Blocking the last partial
step strands a center below the population bands permanently: a world with
`k_potential = 2.86` sat at infra 2, which pinned `K` at 2, which pinned
population at 2, which never crossed the level-3 band edge — it could never build
anything and accumulated minerals it could not spend. **1,050 of 2,435 Idle
decisions on seed 1 were centers in exactly that state.**

**6.8 `RATIFIED` — `reinvest_bias = 0.5`, held rather than tuned.** Both sides of
the comparison are now `rank` score per kilotonne committed — `score /
outward_cost` against `w_k · min(1, headroom) / infra_cost` — so it is an odds
ratio with a state-dependent crossover.

**The knob cannot move Growth's own objective, by identity** (R-O87): a mineral
buys the same works whether it deepens or founds, at every rung, at the card-free
`eta_works = 1`. Measured **+0.32% ± 1.42 over eight seeds**. The lever it is
*not* is `eta_works`, which divides the deepening bill and nothing else.
Appendix §A.11.

**The branch is still cold at 0.5, and that is correct**: an infra rung above the
founding one costs 0.9 kt against a Medium colonizer's 0.10 kt, so expansion
returns 24–49× per kilotonne. **R-O68 resolved** — the dead branch was the right
answer reached for a wrong reason. Appendix §A.10.

**6.9 `RATIFIED` — `productivity_step = 0.20`.** The share of effort ploughed
back into productivity versus spent reaching outward. **R-AC11 resolved;
placeholder magnitude**, and a doctrine parameter a Greening card retunes.

**6.10 `RATIFIED` — berths are quantity and `fab_cap` is quality.**
`slips` reads the *fabrication share of the stock* and is unbounded; `fab_cap =
0.1` bounds the rate **per berth**. Before R-O88 one variable did both jobs and
the build-wide axis was **closed at two berths** — 10¹² kt of infrastructure still
bought two. Berths at rung II went 2 → 17; **fleet-years +26–34%** — measured as
a hull *count*, which is neither the mass nor the volume denomination the
objective has since carried (R-PROD5); kept as measured, R-TREE10 re-runs it.

**When two ratified claims collide, check whether one symbol is carrying two
meanings before you pick a winner.**

**6.11 `RATIFIED` — a yard fills every berth.** Filling all berths amortizes
`build_lead_years = 2.0` across slips, so a yard produces ~1.8× the hulls. Every
commit schedules its own `BuildDecision`, so a yard with `k` berths raises `k`
events where it raised one — which is a **decision-count** cost, not an
entity-count one.

**6.12 `OPEN` — R-AC12: how a center splits output across multiple pending
targets.** One build per decision versus a genuine parallel allocation across
filled berths.

**6.13 `OPEN` — the expansion loop's time constant.** The limiter is the
**unconditional pre-`medium_min_level` staircase**: found at `K = 1` with no
headroom, then serially mine `round(infra)+1` minerals, deepen, grow past the
level-3 `PopBands` edge, afford the colonizer, fly. **Several of those gates are
integers and one — the infra cost ladder — is not a parameter at all**, which is
why every continuous gradient probe has ranked ecology and hull-cost knobs
instead: *the rate limiters are invisible to the instrument.* Sweep the gates
discretely. **T-51 carries the work**; appendix §A.8.

---

## 7. Defense — the hook only

**7.1 `RATIFIED` — default posture is expand in all directions, defending if
pressed.** A hostile detected within reach schedules a defensive reaction,
light-lagged by the distance to the responder.

**7.2 `OPEN` — R-AC13: the trigger and reaction for "if pressed" at the
colonization layer** — e.g. a colonizer re-routing away from a detected threat.
The substance belongs to `Hyades_warfare_tree.md`.

---

## 8. The civilian hulls this tree uses

**8.1 `RATIFIED` — the four civilian roles.** Scout (survey), Colonizer (founds),
Miner (extracts), Freighter (hauls), plus **Reserve** (a standing mission that
ended) and **Scrapped** (a *completable* mission that ended — only an exhausted
Scout, at `scrap_recovery_fraction = 0.5`). A miner/hauler pair's mission ends
when its rock is exhausted **or when its own empire settles the rock** — a colony
mines itself — and the hauler goes to Reserve (appendix §D.18; before this a
hauler routed to the center it stood on flew legs of zero length forever).

**8.2 `RATIFIED` — role eligibility is permissive; competence varies.**
`assign_role` declines on no viable target, never on hull type. Competence is a
degree (an LSV scouts badly); capability is a fact (a Limited hull has no cargo
hold, so a Limited Colonizer founds nothing). **R-O44 resolved.**

**8.3 `RATIFIED` — the hull a role flies is a decision, not a constant.** For
freighters it is §5.9's forecast. `role_hull_type`'s old rationale — *"pick the
cheaper"* — was correct under the pre-R-O58 ladder and backwards for every landing
since; it survived because the General hull's *turnaround* made it a bad idea for
an unrelated reason, which T-96 removed. **Check whether a constant's stated
reason still holds after you fix something else.**

**8.4 `RATIFIED` — acceleration is `thrust / (dry_mass + cargo_mass)`, and thrust
comes from mounted drive.** `drive_specific_thrust = 18.21`,
`structural_drive_fraction = 0.05`, `drive_volume_fraction = 0.01` — **placeholder
magnitudes**, ratified in shape (R-MC16/T-96). Drive and cargo both scale `r³`, so
the shell term shrinks away and a laden General hull is no longer paying the law's
own cost advantage back in turnaround.

**The load-state broadcast survives, which is what the observation model needs**:
a General hull still drops 5.06 g empty to 0.23 g laden, a 22× swing, against a
Limited hull's 1.00 → 0.70.

**8.5 `OPEN` — R-AC14: civilian hull stats as ratified magnitudes** — accel,
capacity, cost, build time, pop-gate to produce. Most are now derived from the
shell model rather than configured, which is a better answer than the one this
R-code asked for; what remains open is the three drive constants in 8.4.

---

## 9. Card hooks — what Far Shore and Greening cards write

**9.1 `RATIFIED` — a card writes standing-layer state and nothing else.**
Doctrine, the Roster, per-player knowledge, and `Works`. An illegal order is
**coerced to the default order, never rejected** (net §5.1).

**9.2 `RATIFIED` — the two trees' write surfaces.**

| Tree | Writes |
|---|---|
| **The Far Shore** (Expansion) | survey volume / speed / range, directional or sector bias, colonization priority, claim rate, contest resolution, `survey_avoids_inhabited` |
| **The Greening** (Growth) | `productivity_step`, `growth_rate`, **K-ceiling lifts (terraforming `hab`/`bio_max`)** — which retroactively re-rank planets per §3.1 — and carrying-capacity effects |

**9.3 `RATIFIED` — Expansion is scored on colony-years and Growth on
work-years**, not on a shared colony count (`Hyades_trees_and_card_value.md`
§2.3.1, §2.3.3). Colony count at a horizon is a weak invariant — a change that
founds the same worlds a century later scores identically — and **colony-years is
measurably *inverted* as a guard for anything that changes how minerals are
spent** (appendix §B.5).

**9.4 `OPEN` — R-AC15: lock §§2–6 enough to author the first depth-1 beats** of
both trees with real outcomes and costs.

**9.5 `OPEN` — a terraforming card makes the coverage denominator playable
again.** `gate_erosion` is structurally zero today only because nothing mutates
`bio_max`. The counter is already in place — the objective is an absolute colony
count, not a fraction — and the guard is kept rather than deleted.

---

## 10. Register

### Ratified

| Code | Decision | Confirmed? |
|---|---|---|
| R-AC1 | omniscient command view, light-lagged simulation | — |
| R-AC2 | continuous 3D, no hexes in the sim | — |
| R-AC3 | `SurveyStrategy::OpeningSectors`; no measurable difference between candidates | 4-seed CRN |
| R-AC11 | `productivity_step = 0.20` | placeholder |
| R-AC16 | `survey_reserve = 1024` — direction ratified, magnitude inert | superseded by R-O86 |
| R-AC17 | `k_high = Band 3.2` — a knife-edge, not a slope | 4-seed, ±25% both directions |
| R-AC19 | recycle exhausted mining pairs | 8 seeds, 3.2 SE |
| R-O44 | permissive role eligibility | — |
| R-O66 | `k_potential = min(hab, bio_max)`, Bands | ablated |
| R-O68 | deepen/expand compared as return per kilotonne | bit-identical below 0.96 |
| R-O69 | the production *decision* is an event | +5.0% colonies |
| R-O74 | settlers are conserved; the hold carries a mix | ablated |
| R-O84 | `growth_rate = 0.873` | 4-seed, carried through T-94 |
| R-O86 | a scout needs somewhere to scout | 5.6× throughput, objective flat |
| R-O87 | `reinvest_bias = 0.5`, held | 8 seeds, +0.32% ± 1.42 |
| R-O88 | `slips` is quantity, `fab_cap` is quality | +26–34%, on a hull *count* |
| R-O89 | freight loads against the destination's color deficit | 8/8 seeds, replicated |
| R-O92 | one outbound leg visits two piles | 8/8 seeds |
| R-O93 | the population logistic is solved, not stepped | 8/8 seeds |
| R-O94 | a hauler's hull is a forecast under a liquidity cap | 8/8 and 7/8 seeds |
| R-MC16 | thrust is drawn from mounted drive | placeholder magnitudes |
| R-P2 | `trade_decay_lambda = 0.01` for internal routing | 3 seeds — thin |

### Open

| Code | Question | What would settle it |
|---|---|---|
| R-AC4 | scan dwell time and close-scan range | a decision |
| R-AC5 | the rank formula's *shape* and class thresholds | terms, not a sweep — §A.5 |
| R-AC6 | provisional remote ranks before a close scan | a decision |
| R-AC7 | colonizer cost / build time / founding infra as independent magnitudes | MC, once they stop being derived |
| R-AC8 | contested-claim resolution | a decision; blocked with T-34 |
| R-AC9 | "nearest center" under light-lag | a decision; design law #15 |
| R-AC10 | mining and freighter rates | MC |
| R-AC12 | multi-target output split across filled berths | a design pass |
| R-AC13 | "if pressed" at the colonization layer | `Hyades_warfare_tree.md` |
| R-AC14 | the three drive constants of §8.4 | the arena |
| R-AC15 | lock enough to author depth-1 beats | §§2–6 stable |
| R-AC18 | should the Colony class carry a `K` floor at all | joint sweep with T-20's ceiling curve |
| R-AC20 | `center_mining_fraction`'s sign | a ten-seed bed |
| R-SIM4 | departure traffic as an inferential signal | a formulation |
| T-51 | the expansion loop's integer gates | a discrete sweep |
| T-92 | 85% of bank inflow is a center mining its own planet | a census, then a mechanism |
| — | λ is +0.002 on Expansion and −0.348 on Growth | the composite objective |

---

## References

- `Hyades_experiments_appendix.md` §A — the measurement record behind every
  ratified item above
- `Hyades_industry.md` §1 (the `K` amendment), §4 (the mining law), §6 (the
  layering algebra and the freight branch), §8.1 (refined mass traverses real
  space)
- `Hyades_trees_and_card_value.md` §2.3.1, §2.3.3 — this tree's two objectives
- `Hyades_galaxy_and_autopilot.md` — the world model
- `Hyades_vehicle_roles.md` §4 — role definitions and the standing/completable
  distinction
- `Hyades_card_contract.md` §6 — deterministic argmax targeting
- `CLAUDE.md` design laws #3, #11, #14, #15
