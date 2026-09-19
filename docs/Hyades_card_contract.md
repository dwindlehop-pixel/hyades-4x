# Hyades — Card Design Contract & Balancing
*The normative spec for what a Hyades card **is**, how its effect realizes in the simulation theater, and how the whole list is balanced by Monte Carlo without a live playerbase. Governs every card (round-1 and beyond) in `hyades_opening_actions_r1.json` and successors. Companion to `Hyades_simulation_model.md` (sim §), `Hyades_command_cards.md` (cmd §), `Hyades_galaxy_and_autopilot.md` (world model), and `Hyades_autopilot_colonization_growth.md` (the first detailed autopilot). Calls flagged **R-Cn**. **Rev 2:** cards issue **instant global orders** executed at light-speed via relativistic event-scheduling (§2, correcting the earlier "no instant" framing); the always-false `arbitrary_input` flag is dropped (the rule in §1 stands).*

---

## 1. The cardinal rule — no arbitrary input

**A card is `(costs, target)`. Playing it is: meet the costs, place it on a target. Nothing else.** Strategic play must stay this simple. Cards that demand **arbitrary input are verboten.**

- **Forbidden:** free-numeric input ("send *how many* ships?"), freeform coordinates, "choose any N of M," sliders, or any choice whose option set is not small, finite, and enumerable from the theater state.
- **Allowed targets** are drawn from a **determinable finite set**: `self/global`, one neighbor empire, one held hex, one frontier hex, all-future-production (self), a single finite menu (e.g., a bias vector ∈ {high-K, a hue, a heading, open space, mines}), or `none`.

This is not only ergonomic; it is the precondition for §6–§7 (deterministic value, Monte-Carlo balance). **R-C1:** ratify the closed list of legal `target_rule` kinds.

---

## 2. What a card does — an instant global order, executed at light-speed

A properly designed card **issues an instant, global order**: the moment it resolves, that directive is your empire's standing intent everywhere at once. What is **not** instant is the order's *consequences in the theater* — because the simulation is **relativistic**.

- **The outcome is an autopilot order**, set instantly. Named order families: **production, expansion, trade, research**, plus **combat/posture, synthesis, fortification** (§5).
- **Effects propagate at light-speed via event-based scheduling.** When an observation at one place must produce a response at another **N light-years away, the response is queued N years in the future** — nothing outruns light. A scan result must *reach* a production center before it can build; a reaction to a detected fleet is scheduled by the distance to the responder. The simulation is a **discrete-event schedule whose every causal edge carries a light-travel delay.**
- **The lag is the counterplay window.** Because consequences ripple outward at *c*, an opponent who sees the order land has the light-cone gap to respond before its value matures. This — not a slow progress bar — is where counterplay lives.
- **Realization modes** (`realization`): `standing` (bites once the order's light reaches the actors, then each round), `over_time` (value accrues as the light-cone widens), `on_new_production` (future builds only). **R-C2:** the realization/lag curve per order family.

**Information model.** The **command view is omniscient** — the player sees the true theater state, for planning. The **simulation runs on fog of war**: units act only on what they have **scanned**, **stealth** hides the rest, and the light-lag above delays every reaction. The strategic space is exactly that gap — the player sees sooner than the empire can react, and spends cards to close it (still bounded by *c*). Full detail in `Hyades_autopilot_colonization_growth.md` §1.

---

## 3. Replay & scaling

Some cards are **singletons** (played once, ever); others may be **played again — at most once per round — for additional effect.** The marginal effect is **diminishing, snowballing, or constant**, and which one depends on the **tree, the whole-game strategy, the node depth, and the theater state** (so the same scaling tag can behave differently in different games).

- `replay` ∈ {`singleton`, `once_per_round`}; `scaling` ∈ {`diminishing`, `snowballing`, `constant`, `contextual`, `n/a`}.
- Pattern (per the example, §4): a shallow card may be a **singleton**, with its **repeatable / escalating** versions appearing **deeper in the tree** ("additional acceleration upgrades in deeper trees"). **R-C3:** the house rules for which trees default to which scaling.

---

## 4. Worked example (for extrapolation)

**Fusion-Torch Efficiency** *(Technology / The Long Dawn, hard-SF engine tech — placeholder name)*
- **Outcome:** new hulls built from now on accelerate faster (an `on_new_production` order on the production autopilot).
- **Open tuning knobs (flagged, not yet set):** the **magnitude** (the illustrative ×2 is probably too large), a possible **class restriction** (limit the thrust gain to certain hull classes), and **replay** — likely a **singleton**, with further acceleration upgrades living **deeper** in the tree rather than re-playing this one.
- **Why it obeys the contract:** target is `self/global` (no arbitrary input); value realizes over time (you must build the hulls); the best target is computable (§6). **R-C4:** lock the magnitude, class scope, and singleton/upgrade structure once the hull model (§8) exists.

---

## 5. The autopilot order model

Cards set **orders**; the theater **executes** them in real time. Each order family has: a **state it edits**, an **execution loop** (how the sim carries it out per tick/round), a **precedence rule** (how it combines with standing defaults and other orders), and a **realization curve**.

| Order family | Edits | Realizes as |
|---|---|---|
| **Production** | build-mix, hull pattern, foundry priority | future builds change over `on_new_production` |
| **Expansion** | settle/claim/explore bias vector, headings | fleets re-route and settle `over_time` |
| **Trade** | standing lanes, partners, commodity flow | flow each round once the lane is `standing` |
| **Research** | which class/aspect the ramp redraws next | counter-graph shifts `over_time` (the slow bomb) |
| **Combat/posture** | hold↔ready, formation, engagement rule | posture changes `standing`; effects on contact |
| **Synthesis** | pop-Band-IV conversion targets, wastage handling | output each round once a pop-Band-IV forge exists |
| **Fortification** | per-hex hardening | builds up `over_time` |

**R-C5:** the precedence algebra — when two orders touch the same state (e.g., two expansion biases, or a card vs. The Compass), how do they compose (override / sum / most-recent / weighted)? This is load-bearing for determinism.

---

## 6. Deterministic value & the computable best target

Every card has a **deterministic value function** `V(card, target, theater_state)` over its finite legal targets, and the engine can compute the **argmax target** — *where the card's value is largest.* The player may pick a different target for whole-game-strategy reasons, but the value itself is **deterministic**.

- **`value_model`** (per card) names the quantity `V` measures (e.g., Δ projected output, Δ expected combat losses avoided, Δ counter-coverage) and how the best target is found (argmax over the finite set).
- **Determinism coexists with yomi.** Determinism is in **resolution**, not **information**: orders are still **hidden and simultaneous** (cmd's meso layer), so you are betting on the opponent's concealed order. The engine's argmax assumes a *believed* opponent state; the real game is choosing well under that uncertainty. So `V` being deterministic does **not** collapse the bluff. **R-C6:** the theater-state vector `V` reads from, and whether `V` is exact or an estimator.

---

## 7. Monte-Carlo balancing — no live players for the first pass

Because there is **no arbitrary input** and the **best-target value is statically determinable**, full games can be **simulated**: give each seat an **autopilot policy** (baseline: greedy-`V` — play the affordable card whose argmax value is highest; later, parameterized or tree-biased policies), run many **seeds × matchups × archetype/color assignments**, and read **outcome distributions**.

- **Balance targets (to define):** win-rate parity across the six sagas, the three homeworld archetypes, and node depths; no dominant single-tree line (cmd's combo-cost principle); a healthy elimination-clock distribution. **The autopilot's behavior is judged quantitatively** — a tree that wins too often under greedy-`V` is over-tuned.
- This makes balance a **measured** property of the card list + autopilot, re-runnable on every revision (and every content-hash bump). **R-C7:** the balance metric set and pass/fail thresholds.

---

## 8. What the sim & autopilot docs still need (authoring agenda)

To author the full list and run §7, these must be specified — most belong in `Hyades_simulation_model.md` and `Hyades_galaxy_and_autopilot.md`:

1. **Hull / class taxonomy + stats** — so "all new hulls," class restrictions, and acceleration are well-defined (Banks-style naming still open). *(blocks R-C4 and most Technology/Warfare/Production cards.)*
2. **Production model** — build rates, the pop→class gating table (R-P2), foundry output, the mobile-dock exception.
3. **Counter-graph numbers** — the weapon/defense/acceleration interaction matrix, so combat resolves deterministically and `V` for combat cards is computable (sim §5).
4. **Autopilot order semantics + precedence** (R-C5) and **realization curves** (R-C2) per family.
5. **Trade / diplomacy resolution** — how lanes, partner-K lift, and pacts resolve each round (R-A3).
6. **The `V` inputs** — the theater-state vector, and the **MC autopilot policy** (R-C6, R-C7).
7. **Synthesis logistics** — supply-chain transit vs. abstract adjacency (R-M5), wastage handling.

Until 1–4 exist, cards beyond round 1 can be *named and slotted* (saga beats) but not given final outcomes, magnitudes, or costs. Round 1 is authorable now because its cards set **standing/posture orders** that need only the order families, not the hull/combat numbers.

---

## 9. Ratification points
- **R-C1** closed list of legal `target_rule` kinds · **R-C2** realization curves per order family · **R-C3** scaling house-rules per tree · **R-C4** lock the engine-efficiency example (magnitude/class/singleton) once hulls exist · **R-C5** order precedence algebra · **R-C6** `V` state-vector + exact-vs-estimator · **R-C7** balance metrics + thresholds

---

## 10. Write capability is partitioned by tree, and the partition is enforced in the engine

**Author's ruling: only Warfare cards may carry Doctrine that kills population,
and this is enforced at the simulation level.** That is a constraint on the
*card layer's data*, not on card text, so it belongs here with the rest of the
contract.

### 10.1 The rule

> **A standing-layer write whose realization can reduce a player's population
> may appear only on a card whose `tree` is `Warfare`.** Every other tree's
> cards are population-safe by construction.

Three reasons it is a contract clause rather than an authoring convention:

- **It is what makes the trees mean anything.** Trees §2 gives each tree its own
  objective; §2.3.2 makes Warfare's the only one that scores by *lowering*
  someone else's stock. If a Growth card can crater a biosphere, five trees have
  a Warfare mouth and Warfare has no distinguishing act.
- **Its absence is silent.** Nothing in `CardEffect` or `DoctrineWrite`
  distinguishes a write that raises a ceiling from one that lowers it — both are
  an `f64` — so a mis-slotted card compiles, runs, and reports a plausible
  number. This is `CLAUDE.md` §4's *"a quantity carries its unit in the type"*
  applied to a capability instead of a unit.
- **`Hyades_standing_layer_and_observation.md` §5 already says Doctrine and
  Design are written *only* by tree cards.** This says *which* tree, which is
  the half that was missing.

### 10.2 The predicate is on the write's value, not on its variant

The obvious implementation — a list of population-lethal `DoctrineWrite`
variants — is wrong, and the shipped card list shows why. `TIER0` card 5 is
**Growth / LessGuarded**, carrying `WriteDoctrine(BiosphereRegen(1.5))`. The
variant is population-lethal: `biosphere_regen_bonus` scales the rate the
biosphere regrows toward `bio_max`, and since T-94 an over-capacity world decays
*toward* its ceiling rather than overshooting to zero, so a factor below `1.0`
starves growth and, sustained, lowers the standing stock population is drawn
out of (design law #11). At `1.5` it does the opposite and is harmless.

So the predicate reads the **argument**:

```text
lethal(WriteDoctrine(GrowthRate(f)))     =  f < 1.0
lethal(WriteDoctrine(BiosphereRegen(f))) =  f < 1.0
lethal(WriteDoctrine(SurveyVehicles(_))) =  false
lethal(WriteDoctrine(ReinvestBias(_)))   =  false
lethal(UnlockDesign(..))                 =  false      // Design adds a roster entry
lethal(DiscloseScans)                    =  false
lethal(WriteWorks(..))                   =  false      // works are industrial, §1.1
```

`lethal` is **total over `CardEffect`**, so a new variant must state its answer
rather than defaulting to permitted. That is the property worth having: the check
fails closed when the card layer grows.

### 10.3 Where it is enforced, and why there are two places

Two enforcement points, for two different failure modes, and both are cheap:

1. **At the card list** — a `const` assertion over `TIER0` that no non-Warfare
   card carries a lethal effect. This is a **static** property of shipped data,
   so it costs nothing at runtime and fails the build rather than a match. It is
   the same shape as the `compile_fail` doctest that stops a rung being written
   as a bare number (`CLAUDE.md` §4).
2. **In `Order::coerce`** — a play of a lethal card from a non-Warfare tree
   coerces to `Order::pass`, like any other illegality. Redundant while the list
   is static, and **not** redundant once cards are data: §7's Monte-Carlo
   balancing revises the list every run, and netcode §5.1 requires every client
   to reach the same verdict from state it already holds. Legality here is a
   pure function of `(card.tree, card.effect)`, both of which every client has,
   so the coercion is free of message exchange and cannot desync.

**Coerce, never reject** (§1, netcode §5.1). A lethal card in the wrong tree is
not an error to surface; it is an illegal order, and an illegal order becomes a
pass.

### 10.4 What the rule does *not* say

Stated so the boundary is not re-argued later:

- **It does not make Warfare cards population-lethal.** It licenses them. The
  first Warfare card specified (`Hyades_warfare_tree.md` §7) carries no lethal
  write at all — its
  lethality is to hulls — and it is still a Warfare card for reasons of its own.
- **It does not cover Design.** A roster entry adds a hull; hulls kill people
  by being used, which is combat, not a standing-layer write. Gating Design by
  tree would forbid Technology from unlocking an Offensive hull, which §8.3's
  cross-tree ladder disruption requires it to be able to do.
- **It does not cover indirect economic harm.** A Politics card that publishes a
  rival's holdings gets their colonies attacked; that is the design working
  (politics §5.3). The rule is about a *write* whose own realization reduces
  population, not about consequences downstream of other players' choices.
- **It is not a claim that population loss is currently costly.** It is not:
  `Hyades_industry.md` §1.8 records that no output function in the engine reads
  population, so a population kill costs its target nothing per year. The gate
  and the thing it gates are separate work items (T-109 and T-107).

**R-TREE11** carries the two calls this leaves open: whether `lethal` should be
a *sign* test as above or a declared capability flag per card, and whether the
partition is exclusive (only Warfare) or a floor (Warfare must carry some,
others may not) — the same shape as R-O33's lean-as-a-ratio rule, which this
deliberately does not follow, because a ratio cannot express a prohibition.
