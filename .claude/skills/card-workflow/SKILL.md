---
name: card-workflow
description: Design, implement, test, name and voice one Hyades card, and balance it within its own tree. Use when designing a new card, reworking an existing card, or measuring a card's value. Does not cover cross-tree balance or counter-graph design, which are a separate workflow.
---

# Card workflow — one card, from intent to a measured, named card

This is the path the Growth card (`TIER0[3]`) and the two Warfare cards (the
armed frontier, `TIER0[15]`, and its port strike) took, with every step that was
learned by getting it wrong written in as a gate. Run the stages in order. Each
ends in a **gate**: a check that must pass, or a stop that names what is missing.
A stage that is skipped is recorded as skipped, with the reason, in the PR.

**Scope.** In scope: one card's design, its engine work, its tests, its
measurement against its own tree's metric, its name and flavor text, and
**within-tree balance** (its slant, its tier-mates, its combos inside the tree).
**Out of scope:** cross-tree balance (tier equality across trees, trees §4.3's
first requirement) and the design of the counter-graph. Take counter-graph edges
as given input. If the card needs an edge that does not exist, or its value can
only be read against another tree's metric, **stop and hand off** — do not
invent the edge or borrow the metric.
When the card's write moves its run the wrong way, or costs more throughput
than it is worth, root-cause it through `.claude/skills/design-doctrine-workflow`
and return here to measure the card.

**Read first:** `AGENTS.md` §2 (measurement habits), §5 (design laws), §6
(working agreement); `docs/Hyades_trees_and_card_value.md` §1 (voice), §2 (the
six objectives), §4 (card value); `docs/Hyades_card_contract.md` §10 (which tree
may write what); the card's own tree spec and its register.

---

## Stage 0 — Brief

Write down, before anything else:

- **Tree, tier, slant** (`Inscrutable` / `Balanced` / `LessGuarded`, design
  law #9), and the `TIER0` slot or new id.
- **The author's intent, quoted.** Every later stage checks against these words.
- **The counter-graph edges the card lives on** (given; see scope).
- **Every design law, ratified decision and R-code it touches**, by number.

**Gate:** the brief names the tree's metric (Stage 1) and at least one sentence
of the author's own words. No author intent → ask; do not design from a guess.

## Stage 1 — Story and objective

- **Where in the saga** (`Hyades_galaxy_and_autopilot.md` §7): which beat of the
  tree's arc this card is, and the mode of love the empire believes it is
  practicing.
- **The tree's metric** (trees §2.3 and §4.2's table): Growth reads work-years,
  Warfare reads `S_i`, Expansion colony count. The value target is **1.5x–2.0x
  its own tree's metric at P92** on the twelve-seat bed (trees §4.2).
- **If the tree has no harness-readable metric** (Technology as of T-131:
  a per-role Design rating with no stock harness), say so, name the missing
  harness, and measure the card's *reach* (Stage 4) without a value claim.

**Gate:** never score a card on another tree's metric because it is the one
that exists. Colony count scored development regressions as improvements four
times running (`AGENTS.md` §2).

## Stage 2 — Intent as algebra

Write each property the intent claims as an equation over engine quantities, and
**define every symbol in a table** (symbol, name, unit, where it is set).

- Check the set is not over-determined. R-O95: "worse at freight, faster empty,
  slower laden" is three claims and two degrees of freedom, because
  `a_empty / a_laden = 1 + C / M_dry` exactly. One line of algebra refuted a
  third of the first Warfare card's intent before any code was written.
- Check each quantity has the unit the type system says (`Band`, `Kilotons`,
  `Price`, R-O66).

**Gate:** every claimed property is an equation, or is marked as a judgment the
engine cannot express.

## Stage 3 — Writes

Decompose the card into writes, and route each through the standing layer:

- **Design writes** (`UnlockDesign`, a Design field) are permanent and reach
  only hulls built after the play (**design law #12, no retroactive refits**).
- **Doctrine writes** (`DoctrineWrite`) are revisable policy.
- **Works writes** (`WorksWrite`) fold in `CardId` order, never play order
  (float multiplication is not associative; a play-order product desyncs).
- **Every consumer asks `autopilot::Standing`** — add a question to the
  resolver; never `if doctrine.flag { A } else { B }` at a call site (T-117).
  A write moving a role onto a hull is read at the build order, the price and
  the tasking, and three readings diverge.
- **Tree partition** (card contract §10): the write must be one its tree may
  carry. A population-lethal Doctrine write is Warfare's only.
- **Coerce, never reject** (`Order::coerce`): an unaffordable play becomes a
  pass, silently.
- **Leave room** for later cards: a new decision is a `Standing` question with
  the write as its input, so the next card writes the same field.

**Gate:** a table of writes → field → the `Standing` question that reads it →
every call site that asks it.

## Stage 4 — Reach audit (before building anything that measures)

Five checks, each cheap, each of which has saved a bed:

1. **Consumers of each write** (T-120). `grep` the written state for readers
   and read what guards them. The first Warfare card wrote the `Roster`, whose
   only consumer returned `true` while `enforce_roster` was off: the card
   reached no decision, and a twelve-seat bed measured noise for it.
2. **Defaults against the spec** (T-121). If the spec states a default, assert
   the default. For a lock ("X is behind this card"), enumerate every role,
   ladder and roster and assert both halves: shut by default, open after play.
3. **Price the mechanic** (T-115, T-123). A probe of a few printed columns —
   travel time against light, the interception window, where the other side
   cannot avoid being — decides feasibility before tuning. A probe inherits
   the code's assumptions: after finding a defect, re-derive the probe.
4. **Census every trigger predicate** (T-134 stage 2). Count how often each
   condition the card's mechanic fires on is true on the bed. "Both hulls
   standing when an encounter begins" was true 0 of 41,770 times.
5. **Oracle ceiling** (T-125). In a scratch build, hand the mechanic the
   quantity it delivers (a coverage fraction, a delivered tonnage) and read the
   metric. It separates "the idea cannot reach the target" from "the
   implementation does not" — which want opposite next steps. Scratch builds
   never land.

**Gate:** all five answered with numbers, or the card goes back to Stage 2.

## Stage 5 — Implement

- **No special sim code for a bed** (T-133 ruling): no `SimConfig` switch,
  ablation or oracle in the engine; a bed varies only the galaxy and plays
  cards through `apply_orders` at the protocol's barrier. A fleet a bed needs
  is generated with the galaxy (`Galaxy::generate_with`).
- **Conservation** (law #11): `Simulation::mass_ledger` over a run that walks
  the new path. A transfer with one end missing is the usual leak; a `max`
  where a sum belongs is the usual cause.
- **No absorbing zero** (T-112): a cost charged against a stock that
  multiplies production must not reach zero.
- **Determinism**: if no card-free run reaches the new path, add an arm to
  `tests/determinism.rs` that does. No host `ln`/`exp` (`clippy.toml`).
- **Throughput**: time the test targets on the old and new binary; report
  `yr/s` beside `ns/event` (`AGENTS.md` §2's reading table).

**Gate:** CI's four commands pass, and every test target is under 60 s.

## Stage 6 — Tests

- Pin the mechanism **in both directions** — it fires when it should and not
  when it should not. A test that asserts only the positive case passes on a
  door that was never shut.
- **Assert the play landed** (T-120): `apply_orders` coerces an unaffordable
  play to a pass, so a bed that assumes the play measures nothing.
- **A bit-identical result is a structural claim** (T-113): the write is inert
  or unreachable. Look at the data the decision reads.
- When you cut a horizon to fit the budget, **assert the mechanism still
  fires** (a floor on its count), and probe past the value you ship.

## Stage 7 — Measure

- **At earliest legal play: the protocol barrier, never `t = 0`** (T-122).
  Playing at `t ≈ 0` charged every card its price out of the bootstrap bank,
  and that price was the largest effect either first card had.
- **A pure-price control**: an inert card at the same cost, same seats. It
  separates what the card does from what it costs.
- **Common random numbers**, then **replication on seeds the result was not
  chosen against** (R-O87). A 2.4-SE result on four seeds has been refuted
  twice by four fresh ones.
- **Swap the seats** in any head-to-head (T-133): event order at one instant
  once decided every exchange.
- **Write the mechanism check before measuring** (T-90): the quantity the card
  claims to move, read directly. An objective answers "did anything change";
  only the mechanism check answers "did the thing I described change".
- **Report the mix beside the mean** (R-IND11), and decompose a total into
  population × rate (R-O89).
- **When the metric is flat, measure the utilization of what the card buys**
  (R-O87): a knob behind a throttle reads flat.
- **Re-measure after any conservation fix** (T-124): a defect invisible in the
  baseline was the whole of the first Growth card's effect.
- The bed is `examples/card_table` (twelve seats, P92 over seats, interval
  bootstrapped over galaxies). Report P92 with its interval, the median and P98.

**Gate:** P92 in 1.5x–2.0x with its interval, or a stated reason and an R-code.

## Stage 8 — Within-tree balance

- **Slant triad** (design law #9): value rises with commitment and the σ→value
  curve is convex; the cost ratios are std §2's (Floor 5:4:3, Default 3:2:1,
  Peak 4:2:1).
- **Tier-mates in the same tree**: the new card's P92 against theirs, each with
  an interval. A difference inside 2 SE is not a finding.
- **Dispersion at earliest legal play** (trees §4.3): median absolute
  deviation, and the P92−median gap, beside the tree's other tier-mates.
- **Combos inside the tree are measured jointly, never summed** (trees §4.5).
- **Every card in the tree has a situation where it is the best play.** Name
  it. A card with none is dominated and is redesigned, not repriced.

Cross-tree tier equality is out of scope: record the P92 for the cross-tree
workflow and stop there.

## Stage 9 — Name and voice

Register rules are trees §1.4 and are not negotiable: **sincere, never
winking**; bureaucratic and founder registers; expected-value language heaviest
where the act is worst; warm hard SF, never imperial or naval (Warfare is the
Hollywood Western). **Every card keeps a plain `role` subtitle** naming its
mechanic.

- **Name** the card as a beat of its tree's saga (galaxy §7's arc table).
- **Flavor text in the Magic: The Gathering form** — one or two lines, a quote
  or a line of in-world document, attributed where attribution adds to it.
- **The voice reads its context.** The same card is a different beat in a
  different game. Write a **voice table**: the default reading, then one row
  per context that changes it, each a condition over state the command layer
  shows every player — cards this empire has played, cards the target has
  played, the relation between them, which tree reached a milestone first.
  Examples from the author: Privateers against a peaceful empire read
  differently from Privateers against one that has already declared you an
  enemy; a Growth card is biological by default and reads differently if
  Production reached an android workforce first. A context may change the
  flavor, the name, or both; it never changes the role subtitle or the rules.
- **Selection is presentation-only** (design law #15): the variant is chosen
  from `Snapshot` state and nothing flows back into the simulation.
- **Flavor text is the author's own** (`AGENTS.md` §6). Everything this stage
  writes is a **DRAFT for the author**, labeled so in the spec. Never overwrite
  a line the author wrote.

**Gate:** a voice table with a default row and at least one context row, every
cell marked DRAFT until the author approves it.

## Stage 10 — Record

- The tree spec gets the card's section: ratified decisions and open ones only,
  each open one with what would settle it.
- `docs/experiments/` (its `AGENTS.md` says where) gets every run, refuted hypothesis and
  scratch arm, linked from the decision it supports.
- `docs/hyades_todo.md` gets the T-code's state: closed, advanced or opened.
- `AGENTS.md` gets a lesson only if one was learned that changes how to work.
- The PR body lists T-codes closed, advanced and opened, and every ratified
  decision **satisfied or contradicted** — a contradiction argued is a
  ratification; one unannounced is a defect.

---

## Quick reference — harnesses

| harness | what it answers |
|---|---|
| `examples/card_table` | a card's P92 ratio on its tree's metric, twelve seats, bootstrapped over galaxies |
| `examples/card_probe` | a single card's effect on one bed, for iteration |
| `examples/work_years` | Growth's metric and colony-years, 8-seed CRN with a replication set (`WY_SEEDS`) |
| `examples/combat_bench` | throughput and fight counts with both first cards played |
| `examples/tree_gradient` | a knob's elasticity against every measurable tree |
| `examples/holding_demand` | the Exchange book in kilotonnes: bids, asks, holdings, haul room, fills |
