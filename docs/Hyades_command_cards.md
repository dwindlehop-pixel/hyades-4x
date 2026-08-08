# Hyades — Command Layer, Production & Strategy Trees
*Companion to `Hyades_simulation_model.md`. **Rev 8** — collapses splash and unlock into **one act**: playing a card from your accessed frontier **unlocks that node** (the +1 reach cost *is* the unlock). There is no separate, reversible probe — every play is a real commitment, so a feint costs something and reads identically to the opening of a deep dive. Zero distinction is what fuels the yomi engine.*

---

## 0. What rev 8 changes (from rev 7)

1. **Splash and unlock are one act — zero distinction.** Playing a card from an accessed node **unlocks it**: the **+1 reach cost *is* the unlock** (paid once per node; thereafter its cards are base cost and its children become the frontier). There is **no separate, reversible "splash."**
2. **Why:** a free, non-committal probe would hand opponents a clean signal filter — ignore the probes, read only the unlocks. With one act, a feint costs a real (if shallow) commitment and is **mechanically indistinguishable** from the start of a deep dive, so intent can only be read from the **cards themselves**. That is the yomi engine.
3. **The clock is now intrinsic.** With **three actions** and a reach costing **base + 1**, you can afford to open **about one new node a round** — the lead-time throttle falls out of the cost rather than a separate "one unlock per round" rule.

*(Carried: the tree data structure with per-tree independent depth; the accessed frontier including sibling branches; tempo-trends-with-depth; per-tree depth/branching balance knobs; the Growth area-under-the-curve win-objective; the macro-only card model; production; the battle-royale backbone; the combo backbone; elimination-by-manner.)*

---

## 1. The card model *(carried — condensed)*

The **simulation is the micro** (target acquisition, defense, formation, repositioning, encirclement, retreat); the player reads it to run the **yomi engine** and pulls **macro** levers. Hyades is a **digital board game** (Inis / 878 Vikings line), **not a CCG** — it borrows the *strategic vocabulary* (archetypes, tempo, combo) but **no card-game machinery** (no traps, no deck-building, no hand/card-advantage, no filler). The card test: **inject a spatial verb that reshapes board state at empire scale** — **War Sun is the gold standard**; abstract threshold "cards" are banned. Two layers of agency: a **standing layer** (doctrine + design, §2) and a **played layer** (build a board-object · a consequential cross-player act · unlock/play a tree node). **There is no free action and no free probe:** three a round, and reaching a not-yet-unlocked node costs **base + 1** and **unlocks it** (§5) — so every play is a real commitment and every turn carries real opportunity cost. Cards **tell the civilization's story**; descending a tree authors it and the theater renders it.

---

## 2. Production & fleet design — the counter-as-yomi engine *(carried — condensed)*

The sim builds and deploys a **mix** of ships by board state; you shape **what** and **how**. **Taxonomy** (Banks-inspired; naming open): roles **Systems** (economy / mobile foundry), **Contact** (expand / scout / diplomacy), **Offensive** (warhulls / counter-graph) × sizes **Heavy / Standard / Light** × **armed / demilitarized** (any hull re-arms or stands down — Banks's dROU ⇄ ROU; absorbs *Militarize* and *Retrofit*). **Three macro levers:** **Doctrine** (build-mix bias), **Design** (the counter-graph — pulse/beam/torpedo/missile/exotic × armor/shields × acceleration — *this is the yomi*: read the sim, hypothesize rivals' builds, design the counter), **Tech** (Research redefines what each class *is*, so the counter-graph evolves). You never fly a ship; the fleet that wins is the one **designed right for what it meets, decided rounds in advance.**

---

## 3. The battle-royale backbone *(carried — condensed)*

A **battle-royale auto-battler** ([Auto battler](https://en.wikipedia.org/wiki/Auto_battler)), not a Euro game. Elimination is progressive and front-loaded — first out at **~⅓ length** (10–20 min of 30–45), **dissolved / assimilated / destroyed** by manner (§9). Win by a **win-object** (skilled: Platinum, telegraphed, guarantees the win) or **last-to-lose** (casual: no win-object needed). Every tree must **survive** the drumbeat, **accelerate** it against others (the genre's "contribute to an elimination, get paid" rule — [Dota Auto Chess](https://en.wikipedia.org/wiki/Dota_Auto_Chess); cf. #36), and **reach its win-object before the drumbeat reaches it.** Snowball checked by ganging the telegraphed leader.

---

## 4. Tree structure — archetypes, the combo backbone, and topology

**Archetype palette.** Trees are described in modern Magic terms ([deck types](https://en.wikipedia.org/wiki/Magic:_The_Gathering_deck_types)) — each a **blend of two** of **Aggro** (proactive, high tempo) · **Control** (reactive, low tempo) · **Midrange** (flexible, mid tempo) · **Ramp** (accelerate to a late payoff). **Interactivity is the house style** — every tree leans interactive; **Expansion and Growth are the most linear**, **Politics the most interactive** (value depends heavily on board state), **Production interactive through mass** (*quantity is a quality of its own*).

**The combo backbone.** The connective tissue is **combo cards** (not a garnish): power scales with **(1) tree breadth** (a domain payoff — how many *other* trees you hold board state in) and **(2) cross-tree board state** (a cheap Warfare card lethal only on Production's mass; a Politics card scaling with Expansion's spread). No tree is a "combo tree" — **combo runs *between* trees.** Balance law: **a winning strategy plays from more trees than it descends deeply** — if a single tree can win without cross-tree play, the combo cards are undercosted.

**Each tree is a *tree* (the data structure).** Not a ladder: depth is **per-tree and independent** — a **mouth** (root), internal **nodes** that **branch**, and **leaves**, the deepest being the **win-object**.

**Tempo trends with depth (not a hard axis).** A tree's two archetypes split roughly by depth — the **higher-tempo one nearer the mouth** (early pressure), the **lower-tempo one deeper** — with the **win-object purely the deeper archetype** (Warfare's deepest nodes, the War Sun included, are pure Midrange). But the higher-tempo archetype is **not confined to the shallows:** a **deep aggro node is allowed**, it just must **buy its tempo with less total value than the midrange option at that depth.** The deeper (win-object) archetype per tree: Warfare **Midrange** · Production **Ramp** · Politics **Control** · Growth **Ramp** · Expansion **Control** · Technology **Control**.

**Depth and branching are per-tree balance knobs.** The **budget** sets the totals — **≈3 cards/node**, 6 trees, **~120 compelling / 360 ceiling** → ~7–20 nodes/tree, an **average branching of ~2 (lean) to ~3 (rich)** at depth ~6 (≈180–250 cards). But neither is uniform — **each tree's shape follows its two archetypes:**
- An **early+late tree** (Warfare Aggro+Midrange, Growth Aggro+Ramp, Expansion Aggro+Control) forks a **shallow cash-out branch** (the aggro payoff — reached fast, lower value) away from a **deep investment branch** (the late archetype, ending at the win-object). The fork is a **tempo asymmetry.**
- A **late+late tree** (Technology Ramp+Control, Politics Control+Midrange, Production Midrange+Ramp) has **no shallow cash-out** — both choices want the descent — so it forks **deeper, into two flavors of late game,** both requiring investment.

**Depth is tuned independently per tree:** a tree too strong in playtest gets **deeper** (more lead time to its win), one too weak gets **shallower**. Branching fills the trees, leaves room for cross-tree combos, gives each branch a **distinct board signature** (the yomi tell), and lets each path tell its own story.

---

## 5. Doorways & the action economy — traversing the tree

**Accessed vs unlocked.** Within each tree a node is **unlocked** (you've descended into it; its cards cost normal price) or **accessed** (its parent is unlocked but it is not — the frontier). **The accessed frontier is every child of an unlocked node** — and because the tree branches, that includes the **sibling branches at equivalent depth** you passed over, not just the next node down. The six **mouths are accessed at game start.**

**One act — reaching a node unlocks it.** To play a card from an accessed node, pay its **base cost + 1** (the +1 once **per node, not per card**). That play **unlocks** the node: thereafter its cards are base cost and **its** children become the new frontier. There is **no separate, reversible "splash"** — every reach is a real, permanent commitment, and you can never reach a node whose parent isn't unlocked, so you only ever advance one step past your frontier.

**The clock is intrinsic to the cost.** Three actions a round, and a reach costs **base + 1**, so you can open **about one new node a round** — no separate cap needed. A win-object sits **4–10 nodes deep**, so a full descent is **~4–10 rounds**; across a 30–45-minute game you take only **one or two trees to full depth**, plus shallow reaches elsewhere. *(This is the corrected arithmetic behind the retired "bank an action" line — there is no banking and no free probe; reaching costs an action and commits.)*

**Routing rule:** peaceful presence is free (co-locate, pass, scout, escort; **trade in minerals/supers is deliberately frictionless**). A **consequential act on another player's hex/assets/population** requires a node **deep enough for that act**, and the act's **method selects the tree** (force → Warfare; settle-into/assimilate → Politics; seize value → Production; raid/steal → a **cross-tree** card on two trees). **Everything manifests immediately** in the theater — no hidden commitment.

**The yomi tell is the cards themselves.** Because every reach commits, there is no risk-free probe to filter out — a shallow reach and the first step of a deep dive are the **same act**. **Deep cards are sharp and committal**, so a rival's play *is* the signal of how far and where they've descended. **Misdirection** is a real, costed shallow commitment into a tree you won't deepen — a believable feint precisely because it isn't free.

---

## 6. The trees — archetype blends + win-objectives *(table ratified; objects are placeholders)*

**Win-objective** = the condition that wins (skilled play); the **object** in parentheses is a placeholder Platinum board-piece at the deepest leaf, telegraphing it. Objectives are ratified; objects are first-pass.

| Tree (mouth) | Blend (shallow → deep) | Interactivity / tempo | Win-objective *(placeholder object)* | Inflicts | Drumbeat |
|---|---|---|---|---|---|
| **Warfare** (Mobilize) | Aggro → Midrange | interactive · high tempo | **destroy the enemy in a decisive fleet battle** *(War Sun)* | **destroyed** | **accelerates** it; strong early, fades if it can't close |
| **Technology** (Research) | Ramp → Control | interactive · low tempo, late bomb | **reach a late stage of unmatched supremacy** *(Ascension Engine)* | **destroyed / dissolved** | survives via quality; the wolverine; mustn't die before it lands |
| **Politics** (Statecraft) | Midrange → Control | **most interactive** · mid tempo | **forge a winning alliance** *(Concord)* | **dissolved / assimilated** | survives by not being the target; **redirects** the drumbeat |
| **Growth** (Cultivate) | Aggro → Ramp | **more linear** · high → late payoff | **maximize the area under the curve — productive planets at unmatched pace** *(cumulative Bloom, not a single city-world)* | **assimilated** | survives by mass + dispersal; vulnerable to ganging |
| **Expansion** (Survey) | Aggro → Control | **more linear** · high → hold | **control too much of the galaxy** *(Beacon Lattice)* | **dissolved / assimilated** | early presence aids survival; thin economy risks late collapse |
| **Production** (Industrialize) | Midrange → Ramp | interactive **through mass** · low → overwhelming | **build something no rival can afford** *(Orbital / Ringworld)* | **assimilated** | feeds by absorbing worlds intact |

---

## 7. The macro opening *(carried — condensed; no Station, no micro)*

Set/adjust the **standing layer** — **Doctrine** (build-mix + expansion bias; folds the old per-colony Settle/Pioneer/Develop/Prospect into policy the sim executes — R-3) and **Design** (your opening counter-graph bet). **Discrete plays:** build a board-object (early: a **Citadel** — the macro form of card-granted fortification), **Open Trade** (liquid, low-friction), **Non-Aggression**, **Transmute** (mineral-ladder valve), or **reach an accessed node** (base + 1 — this unlocks it).

---

## 8. The Warfare tree, worked — a tree, shallow to deep *(the reconception in one tree)*

Warfare (**Aggro shallow → Midrange deep**) inflicts **destroyed**, accelerates the drumbeat, and ends at **War Sun**. Doctrine, designs, board-objects, and consequential acts — never per-fleet micro. ≈3 cards/node.

- **Mouth + shallow Aggro nodes (cheap to reach):** `Mobilize` *(the war-footing tip — visible the instant it resolves)* · basic **Offensive designs** *(counter-graph picks; out-design rivals, don't out-fly them)* · **Levée en Masse** *(combo card scaling with your **Systems**-hull count)*. **Reach example:** a Producer reaches Warfare's shallow node for **base + 1** to play `Levée en Masse` off its mass — a real but small commitment (the node is now unlocked), partial teeth without a deep descent.
- **Deep Midrange nodes (soft-locked behind the descent):** **Advance** *(commit force to a hex an opponent holds)* → **Blockade** *(sever a system's supply while held)* → **Bombard** *(crater a world to lower **K**, sim §2a)*. The sim fights the engagements.
- **Deeper still:** **Privateer** *(a cross-tree card on the Politics **and** Warfare trees — the home of piracy)*.
- **Win-object (deepest leaf):** **War Sun** — `Platinum + supers` on a deep base; build and ignite the capital-killer (#19).

Absent by design: Concentrate Fire, Encircle, Run Them Down, Picket — those are **what the sim does** once your designs and doctrine equip it.

---

## 9. Elimination by manner *(carried — condensed)*

**Destroyed** (Warfare): worlds → husks, fleets → a **wreck-field** survivors **salvage** (denial, not transfer). **Assimilated** (Production / Growth / Politics): worlds + population folded in **intact** — the victor **gains the base**. **Dissolved** (Politics / collapse): the empire **fragments** to neutral/rebel hexes — a scramble the **whole table** feeds on. Each accelerates the next elimination, checked by ganging the telegraphed leader.

---

## 10. Arc — recapitulate, then depart from *Stars!* *(carried — condensed)*

**Recapitulate** ([Stars! SG Ch.1](https://wiki.starsautohost.org/wikinew/ssg/ssg01.htm)): the strategy archetypes (expansion / production / growth / tech) and the "mature fast, then face a fleet you can't match" tension. **Depart:** Platinum **win-objects**, embraced **progressive elimination**, the front-loaded **drumbeat** — explicitly **not** a Euro game.

---

## 11. Open decisions to ratify

- **R-1 · Tree table** (§6) — blends + win-**objectives** ratified; **objects** are placeholders (the Growth object is now a cumulative *Bloom*, distinct from Production's concentration). Redirect object flavor if desired.
- **R-2 · The action economy & branching** (§4–5) — *resolved:* **splash and unlock are one act** — reaching a node costs **base + 1** and unlocks it (no reversible probe; zero distinction, to fuel the yomi engine); the +1 is **per node, not per card**; the clock is **intrinsic to the cost** (~one new node/round); **depth (4–10) and branching are per-tree balance knobs** shaped by the archetype pairing; deep aggro is allowed but **trades value for tempo**; the win-object is the deeper archetype. **Still open:** the **combo-strength target** (single-tree play must be losing) and the **actual per-tree depth/branch numbers** — playtest outputs (R-5).
- **R-3 · How far folded into doctrine** (§7) — Settle/Pioneer/Develop/Prospect → standing doctrine; defense as a board-object (Citadel). Confirm the abstraction, and ship-class **naming**.
- **R-4 · Where backbone + production live** (§2–3) — migrate these how-it-works premises into the sim model?
- **R-5 · The full clock math** — three actions, one unlock/round, depth 4–10, ≈3 cards/node, and win-object build times must net out to a **first elimination ~10–20 min** and a **full game 30–45**. The first thing the sims (sim §5) should pin.
- **R-6 · Sim-model line** — the peaceful-default + consequential-act-not-routing rule wants one line in sim §0a/§3.

---

## References

- *Stars!* Strategy Guide, Ch. 1 — *Whole Game Strategy*: https://wiki.starsautohost.org/wikinew/ssg/ssg01.htm
- Sirlin, *Playing to Win* — *Yomi: Spies of the Mind*: https://www.sirlin.net/ptw-book/7-spies-of-the-mind
- *Magic: The Gathering* deck types (Aggro/Control/Midrange/Ramp; combo): https://en.wikipedia.org/wiki/Magic:_The_Gathering_deck_types
- Auto battler (battle-royale cadence; ⅓-length elimination; 30–45 min): https://en.wikipedia.org/wiki/Auto_battler
- Dota Auto Chess (the "contribute to an elimination, get paid" rule): https://en.wikipedia.org/wiki/Dota_Auto_Chess
- The Culture — ship classification (role taxonomy, militarize/demilitarize): https://en.wikipedia.org/wiki/The_Culture
- Tree (data structure) — depth, branching factor, nodes/leaves: https://en.wikipedia.org/wiki/Tree_(data_structure)
- Liebig's law of the minimum (K = min): https://en.wikipedia.org/wiki/Liebig%27s_law_of_the_minimum
- Logistic function (pop fill; P(wreck); area under the curve): https://en.wikipedia.org/wiki/Logistic_function
- Lanchester's laws (deterministic attrition): https://en.wikipedia.org/wiki/Lanchester%27s_laws
- Intransitivity (the counter-graph): https://en.wikipedia.org/wiki/Intransitivity
- List of military strategies and concepts: https://en.wikipedia.org/wiki/List_of_military_strategies_and_concepts
