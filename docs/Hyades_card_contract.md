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
| **Synthesis** | pop-4 conversion targets, wastage handling | output each round once a pop-4 forge exists |
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
