---
name: design-doctrine-workflow
description: Root-cause and fix the behavior and the engine cost of a Design or Doctrine change — a new or changed autopilot decision, a Doctrine field or default, a Design (hull, drive, bill, loadout), or an engine constant a decision reads. Use when such a change moves an objective the wrong way, turns around over a run, costs more throughput than it is worth, or must be approximated more cheaply. Does not cover measuring a card's value (card-workflow) or cross-tree balance.
---

# Design and Doctrine workflow — from a direction to a landed, explained change

This is the path T-147's freight routing in `$` took (appendix §D.53–§D.56):
a joint sweep of two levers, four routing arms that all lost, the census that
named why, the fix that reversed the sign, and a cheaper doctrine that kept
most of the gain at 2.4x the throughput. Every step it learned by getting it
wrong is written in as a gate. The Design half draws on §D.19 (every leg flies
its Design's drive, −18% traced by elimination) and T-98 (the hauler's hull).

Run the stages in order. Each ends in a **gate**: a check that must pass, or a
stop that names what is missing. A stage that is skipped is recorded as
skipped, with the reason, in the PR.

**Scope.** In scope: a change to the standing layer's state — a **Doctrine**
field or default (revisable policy), a **Design** (a hull, its drive, its bill,
its loadout; permanent, design law #12), or a `SimConfig` constant that a
decision reads — and the engine code that consumes it. The work is to find what
the change does to the run, why, and what it costs, and to fix it. **Out of
scope:** a card's value against its tree (`.claude/skills/card-workflow`
Stage 7 — hand off, and come back here when the card's write misbehaves),
cross-tree balance, and performance work with no behavior change (`AGENTS.md`
§4: profile first).

**Read first:** `AGENTS.md` §2 (measurement habits — every subsection is a
gate below), §4 (per-evaluation cost), §5 (design laws), §6 (working
agreement: the standing layer answers questions; fix an allocation with a
price; no harness code in the engine); `docs/Hyades_standing_layer_and_observation.md`
(what Design and Doctrine are); the spec that owns the decision.

---

## Stage 0 — Brief

Write down, before anything else:

- **The author's direction, quoted.** Each phase of T-147 began with one, and
  each fixed what "done" meant: *sweep the levers behind the >2x shifts*; *the
  missing unlock is dynamic routing priced in `$` … throughput is not the goal
  until the scheduling is found*; *explain the turnaround after 500 yr, then
  fix it*; *approximate it with a cheaper doctrine at under 5% loss of
  work-years*. Every later stage checks against these words.
- **The state written**: Doctrine field, Design, or `SimConfig` constant, and
  the `Standing` question that reads it (`AGENTS.md` §6).
- **The phase**: behavior first, cost second. Do not trade objective for
  throughput while the behavior is unexplained; a cheap approximation of a
  wrong decision is a wrong decision.
- **The objective stack**, all read on every arm:
  1. the tree composite (geometric mean of tree stocks against the reference,
     `AGENTS.md` §2) and each tree beside it;
  2. the quantity the direction names (centers built, work-years);
  3. **the mechanism quantity** — what the change claims to move, read
     directly (Stage 1);
  4. **outputs the composite does not price** — supers and apex forged fell
     60–85% in every §D.53 arm with four or more stops, arms that raised the
     composite;
  5. **cost**: events, `ns/event`, wall time (Stage 7).
- **The acceptance bound in the author's terms** (e.g. "under 5% loss").
- **Every design law, ratified decision and R-code the change touches**, by
  number.

**Gate:** the direction is quoted and the objective stack names a mechanism
quantity. No direction → ask; do not choose the objective from a guess.

## Stage 1 — The decision as algebra, and what it replaces

- **Write the decision as a score over engine quantities** and define every
  symbol in a table (symbol, name, unit, where it is set). T-147's leg score is
  the cargo's value at its buyer's posted prices, discounted by `e^(−λ·d)` over
  every leg — a sentence that cannot be checked until each term has a unit.
- **List what the decision may read** (design law #15). A decision handed
  ground truth cannot be wrong, so it cannot be deceived (T-120).
- **Allocate with a price, not a cap or an order** (politics §0, R-P19). A
  remedy that adds a quota is out of the remedy space; say so and find the
  price.
- **Inventory what the old rule did, including by accident, before replacing
  it.** Census the old rule's outputs by source, sink and kind. Three of this
  project's regressions were a constraint or a channel that the old rule
  provided without naming it:
  - §D.55: the per-stop planner priced outpost piles only. The welded-base
    rule it replaced also moved center-to-center freight (R-MX8), which grows
    from 0.5–3 Mt to the size of outpost freight by 1,000 yr. The planner
    dropped it: composite −41.50% ± 3.59.
  - §D.55 again: the full-hold top-up carried forge colors incidentally; the
    planner removed it and apex fell to 0 on 24 of 24 seat-runs.
  - T-134: the greedy wave bounded each fill by one center's shortfall, which
    the optimal clearing removed (−18.9%).
- **For a Design change, list every site that reads the Design.** §D.19
  rewrote thirteen sites at once; the cause of its −18% was one of them.

**Gate:** a symbol table; a list of the decision's inputs; a census table of
the old rule's outputs by kind, with each kind marked "priced by the new
decision" or "dropped, on purpose: why".

## Stage 2 — Bed, reference and census

- **The reference is the shipped binary**, built once and kept beside the
  scratch builds. Every arm is a paired difference against it on the same seed
  and ground.
- **Arms are scratch builds; the patch lives outside the repository**
  (T-133's ruling: no switch, ablation or oracle lands in the engine). A bed
  varies the galaxy and sets the Doctrine fields a card writes; the harness
  environment variables of `examples/forge_sweep` are the pattern.
- **Seeds:** common random numbers on 1, 7, 42, 31337; replication on 2, 3, 5,
  11; a second ground (`Ground::ColorRotated` beside `Random`).
- **A scratch census harness** that reports per seat, per 250-yr bucket, the
  decision's output decomposed as **population × rate** (T-147: freighters ×
  deliveries per freighter × kt per delivery; loads split by source — outposts
  against centers). Flush a line per seed and per bucket (`AGENTS.md` §2,
  "Always run sweeps unbuffered").
- **The horizon is the objective's, not a screen's.** Arm 3 of §D.54 raised one
  seat's Growth 54k → 371k kt-years at 500 yr, and scored the composite
  −41.50% ± 3.59 at 1,500. A
  horizon before the regime changes ranks arms backwards; screen shorter only
  after the time series says the arms do not cross later.

**Gate:** the reference binary runs the bed and the census prints every
bucket. Any arm that should be inert at its default reproduces the reference
**to every printed digit** (§D.53's leak fix, `λ 0.01 / 2 stops`, seed 1).

## Stage 3 — Arms

- **One lever per arm.** Two changes aimed at one diagnosis are ablated apart
  before either is believed (R-O89's 2×2: +8.4% and −52.3% ran together to
  −41.2%).
- **Sweep interacting levers jointly.** §D.53: a sharper discount alone gained
  nothing past `λ = 0.02`, more stops alone peaked near +4.6%, and together
  they reached +9%. A one-at-a-time sweep would have shipped neither.
- **Probe past the value you intend to ship** and report where it breaks.
- **Weigh the ledger on every arm** (`Simulation::mass_ledger`, law #11). A
  sweep reaches states the default does not: at three or more stops a hauler
  could retire laden at its own base and lose 0.16 kt per trip (§D.53).
  Checking the ledger after every event names the first loss and its event.
- **For a Design change touching many sites, ablate by site group.** §D.19
  restored one group at a time to the old value; each left −13 to −16%, and
  the remaining site was the cause by elimination.

**Gate:** each arm has one stated lever, the ledger closes on each, and every
leak found is fixed in the reference as well as the arm.

## Stage 4 — Read

- Paired mean ± standard error over seeds, and the sign count (n/N). Inside
  2 SE is not a result; a 2.4-SE reading on four seeds has been refuted twice
  by four fresh ones (R-O87, `rank.w_mineral`).
- **The mechanism quantity beside the objective** (T-90). The objective says
  whether anything changed; the mechanism quantity says whether the thing
  described changed.
- **The time series**, not only the horizon total: when the arms diverge.
- **The mix beside the mean** (R-IND11) and **population × rate** (R-O89).
- **The unpriced outputs** (Stage 0, item 4), every arm.

**Gate:** a table per arm with all five rows. An arm that wins on the
objective and loses an unpriced output is recorded with both.

## Stage 5 — Root cause (when the sign is wrong, or the arms turn around)

Stop only at a line of code shown to produce the number (`AGENTS.md` §2,
"Never leave an identified symptom without a proven mechanism").

1. **Bucket by time.** §D.54's arm 3 delivered 2–4x shipped's freight to
   500 yr and 0.5–0.7x after. The bucket where the arms cross names the regime
   to look at.
2. **Decompose the total.** kt delivered = deliveries × kt per delivery; fleet
   = freighters at time t. A per-unit rate that holds while the total falls is
   a population problem, and the two want opposite fixes.
3. **List hypotheses and ablate each in a scratch arm.** Record each refuted
   one with its number. §D.55: filling the hold on departure and keeping the
   base each moved kt per delivery by about 1 kt — refuted.
4. **Rule out upstream.** §D.55: outposts mined 75–84 Mt per seat against
   shipped's 77–88 Mt, so extraction was not it.
5. **Split by source and kind.** The split that named §D.55's cause was loads
   at outposts against loads at centers: arm 3 loaded 0.001–0.002 Mt at
   centers where shipped loaded 21–25 Mt. If Stage 1's inventory was complete,
   this step reads it back.
6. **Oracle ceiling** (T-125) when the question is whether the idea can reach
   the target at all: hand the decision the quantity it delivers in a scratch
   build and read the objective.
7. **The fix must reverse the sign on four seeds**, then replicate (Stage 6).
8. **State the inference in its own sentence, with a confidence and the
   measurement that would change it.** §D.54's inference (fleet size) was
   stated at 60% with its test; §D.55 ran the test and refuted it. An
   inference written that way can be refuted; one written as a finding gets
   built on.

**Gate:** the cause is a named line or rule, an ablation removing it reverses
the effect, and every refuted hypothesis is in the appendix with its number.

## Stage 6 — Confirm, and ask whether the exact rule is optimal

- **Replicate** on the seeds and ground the arm was not chosen against. Report
  the pooled result and per ground.
- **Variants inside 2 SE of each other are a tie; choose the cheaper.** §D.55:
  stop cap 6 against 16 was −0.44% ± 0.33 and ran 20–28% fewer events.
- **Ask whether the exact decision is optimal for the population that runs
  it.** Many agents reading the same fresh price commit to the same buyer.
  §D.56's census — the share of kilotonnes delivered beyond the buyer's
  shortfall on arrival — read **0.930** for the exact planner. Netting each
  commitment against a kept table took work-years +46.78% ± 5.50 above the
  exact planner. Census **what agents commit against what the shared state can
  absorb on arrival** before treating "exact" as the ceiling. (0.930 bounds the
  waste rather than estimating it: some overshoot banks toward the next bill.)

**Gate:** pooled over ≥ 12 runs on two grounds with its sign count, and the
overshoot (or the equivalent coordination census) reported for any decision
that many agents make against shared state.

## Stage 7 — Cost

Start only once Stages 5–6 have explained the behavior.

- **Cost = events × `ns/event`**, both printed, beside `yr/s` (`AGENTS.md` §2's
  reading table). §D.53's arm was +21% events and +46% `ns/event`: more work
  and dearer work, which want different fixes.
- **Wall time on an idle machine, one run at a time, interleaved against the
  reference over at least two rounds.** A reading taken while another run
  shares the cores is not a property of the change.
- **Ask whether the change altered the amount of work** before reading either
  column (T-111).
- **Time an exact optimization even when it is bit-identical.** §D.56's
  pruning of the buyer search was bit-identical and slower, 263.9 s against
  183.8 s, because its bound ignored the voyage discount.
- **Each cheap lever alone, then together:** a table of objective against wall
  for every lever (`freight_shortlist`, `freight_price_age_years`,
  `max_pickup_stops` in §D.56). Choose on that frontier against the author's
  bound, and report the choice against **both** references — the exact
  decision and the engine before the change. §D.56 shipped at −3.8% against
  the best netted arm for 2.2x its throughput, and is still 1.6x slower than
  the engine before the planner; both numbers go in the record.
- **Time the test targets** on the old and new binary (60 s, band to 72 s,
  fix to ≤ 54 s; `AGENTS.md` §2).

**Gate:** a lever table with objective and wall per row, the chosen row
replicated, and every test target timed beside the old binary.

## Stage 8 — Land

- **A revisable policy is a Doctrine field**, read through a `Standing`
  question, with its default as a named constant whose doc comment carries
  the measurement and says **Placeholder** or **chosen by Monte Carlo**. An
  engine constant stays in `SimConfig`. A Design change goes through the
  roster and `design_for`, never a call-site branch (T-117).
- **The landed source reproduces the scratch arm** to every printed digit on
  one seed (§D.55: seed 1, 600 yr).
- **Tests:** pin the mechanism in both directions (it fires when it should and
  not when it should not); a ledger test over a run that walks the new path; a
  determinism arm if no card-free run reaches it; a non-vacuity floor on any
  test whose horizon was cut.
- CI's four commands pass (`AGENTS.md` §2, "CI gates").

**Gate:** the bit-identity check, the ledger test, and CI green locally.

## Stage 9 — Record

- **Appendix, one section per phase**, headed with the T-code and spec it
  supports, the author's direction, the bed, the score, and "arms are scratch
  builds; the patch is kept outside the repository". Arms that lost are
  recorded with their numbers under a title that says none landed (§D.54).
  Each section ends with its inference, its confidence, and its test — and a
  later section that refutes it is linked from it.
- **The spec** carries the decision and its open questions only (`AGENTS.md`
  §6); unpriced outputs that fell become an open question with what would
  settle it (§D.55's apex → galaxy §4.5).
- **`docs/hyades_todo.md`**: the T-code's state. **`AGENTS.md`**: a lesson only
  if it changes how to work.
- **The PR body** lists T-codes closed, advanced and opened, and every ratified
  decision satisfied or contradicted.

---

## Quick reference — harnesses

| harness | what it answers |
|---|---|
| `examples/forge_sweep` | tree composite, per-tree stocks and centers built per seat on the twin bed; Doctrine and `SimConfig` levers by environment variable |
| `examples/work_years` | Growth's metric and colony-years, 8-seed CRN with a replication set, `ns/event` beside `yr/s` |
| `examples/tree_gradient` | a knob's elasticity against every measurable tree |
| `examples/empire_spread` | the tree stocks per seat on a 25-year grid |
| `examples/holding_demand` | the Exchange book in kilotonnes: bids, asks, holdings, haul room, fills |
| `examples/horizon_cost` | cost and colony count by horizon — whether a shorter screen is faithful |
| `examples/mass_audit` | the mass ledger by store over a run |
| `examples/build_digest` | colonies and events at a horizon, for bit-identity checks between binaries |

## Quick reference — the record this workflow is drawn from

| section | stage it illustrates |
|---|---|
| §D.19 | Stage 3: a Design change ablated by site group; the cause found by elimination |
| §D.53 | Stage 3: joint sweep of interacting levers; a leak only a sweep reaches |
| §D.54 | Stages 2 and 5: a 500-yr screen that ranked backwards; an inference stated with its test |
| §D.55 | Stages 1 and 5: the old rule's channel the new decision dropped, named by a split by source |
| §D.56 | Stages 6 and 7: the overshoot census; netting; the lever frontier against wall time |
