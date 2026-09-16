# Hyades — Production: Works, Yards, and the Hull Ladder

*The design space for **Production**, the tree that turns minerals into hulls.
Its objective is **fleet-years in mass** (`Hyades_trees_and_card_value.md`
§2.3.4). Companion to `Hyades_industry.md` (the economy and the layering
algebra), `Hyades_mineral_cost_curve.md` (the two ladders),
`Hyades_vehicle_roles.md` (what a hull is for) and `Hyades_loadout.md` (what it
is fitted with). New calls continue the **R-PROD n** series.*

**Rev 1, new.** Carries **ratified decisions and open decisions only**
(`CLAUDE.md` §6). More of this tree is built than any other unlanded tree —
`Works`, the employment split, the fold, the colour-denominated bill, the berth
model and the cost ladder all ship — so the ratified half is substantial and the
open half is mostly *cards*: nothing writes `Works` yet except a test.

---

## 0. What this tree is, and what it is not

**Production is the rate at which an empire converts banked minerals into hull
mass.** It is not the minerals (that is Expansion's outposts and freight), not
the population (Growth), and not what the hulls can *do* (Technology).

Three axes, and they are separate purchases:

| Axis | Bought by | Engine term |
|---|---|---|
| **Efficiency** — a mineral buys more works | `eta_works` | divides the bill |
| **Quality** — one berth builds faster | `fab_cap` | bounds the per-berth rate |
| **Quantity** — more berths | `slips` | reads the fabrication stock, unbounded |

**The three-way split is the whole design surface**, and `Hyades_industry.md`
§5.3 assigns them: **Production buys fast berths; Expansion buys many slow
ones.** A tree that could buy all three would have no shape.

**The *Stars!* lineage, and where Hyades departs.** *Stars!* production is a
per-planet queue fed by factories, with **miniaturization** making older
components cheaper as a field advances. Hyades keeps the queue and the
miniaturization *idea* — `eta_works` is exactly "a mineral buys more" — and drops
the rest: there is no factory count to micro, no per-item build queue the player
touches, and the ramp is continuous rather than integer-factoried. What survives
is the shape *Stars!* got right: **capacity is a stock you invest in, and
investing in it competes with spending it.**

---

## 1. The industrial stock

**1.1 `RATIFIED` — infrastructure is capacity and *only* capacity.**
`K = min(hab, bio_max)`; infrastructure is **not** a term in it
(`Hyades_industry.md` §1.1). It is the industrial stock: it mines, it fabricates,
and it can be razed — and **razing it must not move population**, which is the
whole reason it left the minimum. While it sat in `K`, every industrial strike
was a population strike, because the logistic overshoots *below* a cut ceiling.

**1.2 `RATIFIED` — one stock, one price, three employments.**
`Employment::{Extraction, Fabrication, Warding}`. The stock is bought once, in
kilotons of works, and *allocated*; there is no separate mine-building and
yard-building economy.

**1.3 `RATIFIED` — infrastructure is the war target.** It is the only stock that
is simultaneously valuable, visible, and destructible without killing people.
That is what gives Warfare something to hit that is neither hulls nor a
population (`Hyades_warfare_tree.md` §4).

**1.4 `RATIFIED` — the rung ladder is scale-free.** Cost and output are both
geometric in the stock, so **a rung pays for itself in 1.8 years at every rung**.
**R-O85 is resolved as false**: infrastructure was never priced out of reach.
Counted per decision on the standard bed, 0% of decisions are gated at the
ceiling, 0.7–0.9% are outbid, and **98.3% simply cannot pay the bill** — of which
**43.8–46.6% of *all* decisions hold the total and lack a colour.**

**The binding constraint on this tree is therefore colour composition, not
price**, which is what R-O89 and R-O92 then moved.

**1.5 `OPEN` — R-IND2: is Warding a third employment share at all?** Or is it a
Design hardness coefficient on the stock, which is cheaper to reason about and
does not force every empire to hold defensive capacity it is not using?
*Recommend* the coefficient, and note that the answer changes what a Warfare card
can target.

---

## 2. The works bill

**2.1 `RATIFIED` — a bill is payable in named colours, and that is a
conjunction.**

```text
total   = infra_step_price(stock) / eta_works      // Design, multiplicative
bill[c] = total × mix_share(c)                     // Doctrine, a simplex
```

**A bill denominated in a total is an unexpressed fact about a game.** The
conjunction is why `can_afford_infra` is a per-colour test, and why inferring it
from a total reported 44.7% "outbid" where the truth is 0.9%. **A reconstruction
that looks arithmetically equivalent is not, when the thing it reconstructs is a
conjunction over three colours.**

**2.2 `RATIFIED` — the default mix is `3:2:1` Yellow : Cyan : Magenta, and Sole
is an asymptote.** **R-IND15.** `WORKS_MIX_DEFAULT`, normalised, in `Basic`
order. A card moves weight *between* colours; it cannot move the total, which
only `eta_works` sets.

**This is measured downstream, not merely asserted:** trade flow reproduces the
`3:2:1` mix on both seeds without anything in the market being told about it
(`Hyades_politics_trade_and_intelligence.md` §2.11, appendix §B.3).

**2.3 `RATIFIED` — Production's signature is the highest peak per planet, not the
best efficiency.** The tallest industrial route in the game runs on a
**sole-Yellow** price (`Hyades_industry.md` §5.2), which makes every empire
pursuing it a standing bid that moves the price of Yellow *for everyone* —
including empires that never touch this tree. Growth and Expansion get the
efficient, capped, payable-anywhere routes instead (§5.3).

**2.4 `RATIFIED` — a card cannot be a discount.** A Production card writes a
*factor*, and the factors compose by the layering algebra (§3). A flat subtraction
would make order matter and would let a stack of small cards drive a price
negative.

---

## 3. The layering algebra

**3.1 `RATIFIED` — there are two algebras, not three.** A **product** (Design:
`eta_works`, `cap`, `half` — multiplicative, commutative, order-free) and a
**simplex** (Doctrine: `mix_w`, `alloc` — weights that renormalise). "Rotation"
was a third candidate and was wrong. **Commutativity is the acceptance test**
(`Hyades_industry.md` §6.5).

**3.2 `RATIFIED` — the write is recorded, not applied.** A card play appends
`(CardId, WorksWrite)` and `Works` is **re-derived** by `Works::fold`, which
accumulates in `CardId` order.

**Applying the factor to a running product at play time would accumulate in
*play* order, and float multiplication is not associative** — two players who
played the same cards in different orders would hold state differing in its last
bits, which is a desync. Same lesson as `holdings_centroid`. **T-75b.**

**3.3 `RATIFIED` — the base is the identity.** `eta_works = 1.0`, `cap = [1;3]`,
`half = [1;3]`, and `mix_w` normalised to the default mix. **An empire that has
played no works card sits exactly on the shipped constants**, so the layer is
inert until a card exists and every measurement taken before cards is still
valid.

**3.4 `OPEN` — R-PROD1: which of the six `Works` fields each tier writes, and by
how much.** The algebra is settled; the *card* surface is not. Constraint from
`Hyades_trees_and_card_value.md` §4.3: tier-1 cards must have the **lowest**
dispersion of any tier, which argues for early Production cards writing
`eta_works` (a single scalar with a smooth effect) rather than `mix_w` (whose
value depends entirely on what the empire's geography already holds).

---

## 4. Yards and berths

**4.1 `RATIFIED` — `slips` is quantity and `fab_cap` is quality, and they are
different variables.** `slips` reads the **fabrication share of the stock** and is
unbounded; `fab_cap = 0.1` bounds the rate **per berth**.

Before **R-O88** one symbol did both jobs and the build-wide axis was **closed at
two berths** — 10¹² kt of infrastructure still bought two — while §3.2 of the
industry spec said the axis "scales without limit". Berths at rung II went
**2 → 17**; **fleet-years +26–34%** on both seeds, and throughput *improved*
because a quarter of all events had been decisions that declined and stalled.

**When two ratified claims collide, check whether one symbol is carrying two
meanings before you pick a winner.**

**4.2 `RATIFIED` — the re-denomination was engineered to be bit-identical.**
`fab_cap` 0.2 → 0.1 reproduces the old per-berth rate *exactly*, because the old
`slips` was always exactly 2 — so turnaround did not move at all and the only
behavioural change in the landing is berth count. **That is what makes the
measurement readable**, and it is the precedent for any future re-denomination.

**4.3 `RATIFIED` — build time tracks hull mass.**
`t_build = build_lead_years + m / fab_cap`, `build_lead_years = 2.0`. A Medium
hull went from 10 yr to 3.0 at **T-68**, which bought +10.5% colony-years with
colony count unmoved — and cost ~19× throughput, because centres then decide
three to four times as often.

**4.4 `RATIFIED` — a yard fills every berth.** Filling all berths amortises
`build_lead_years` across slips, so a yard produces ~1.8× the hulls (**T-69**).
The cost is **decision count**, not entity count: every commit schedules its own
`BuildDecision`, so a yard with `k` berths raises `k` events where it raised one.
This is the one measured case where entity count — the table's standing
first-order cost — pointed the wrong way.

**4.5 `RATIFIED` — extraction saturates once**, in the mining law and not again in
the fabrication pipeline (**R-IND18**). Two saturations on the same stock would
double-count the diminishing return.

**4.6 `OPEN` — T-74: the fabrication *rate* is not a measured quantity in the
engine.** `Factors::infra` is kilotons of works — what production is bought with,
a leading indicator rather than the thing itself. Three things are waiting on it:
Growth's work-years objective in its true form, the `$` faucet
(`Hyades_politics_trade_and_intelligence.md` §1.6, R-P16), and any Production card
whose value is a *rate* rather than a stock.

---

## 5. The hull ladder

**5.1 `RATIFIED` — cost is surface area and value is volume, and the ratio is the
law.** Design law #3. Radius is *derived* from the cost ladder by solving
`cost · η = r³ − (r − τ)³`; the Limited hull is the unit radius, so it is all
shell and no hold.

**This law was silently false in the engine until R-O58**, because cost was on
area (correct) and capacity was the abstract slot count 0/1/2, which is
near-linear — so fragmenting was *cheaper* per unit hauled, 0.067 against 0.100.
Neither half looked wrong alone and both were individually ratified. **A law about
a ratio is not checked by checking its numerator and its denominator separately.**
`shell_model_ladders_are_derived_not_tuned` asserts the ratio.

**5.2 `RATIFIED` — dry mass *is* mineral cost.** Design law #11, R-O57. No
independent `dry_mass` constant can be correct under conservation, and the
reconstruction that existed was costing 30× — one mineral massed 6.0 as hull and
0.2 as cargo.

**5.3 `RATIFIED` — there are two ladders and they are tied by geometry.**
`F_mass = F_cost^(3/2)` (**R-MC15**): cost tracks surface area and the hold tracks
volume, so a General hull costs 10× a Medium while holding 31.6×. The same amount
reads `Band I` as a mass and `Band II` as a cost, and that is geometry, not a
units bug. `Qty::on_scale` is the only crossing.

**5.4 `RATIFIED` — the shipped ladder.** `general_vehicle_cost = 1.0` kt,
`medium_fleet_size = 10.0`, `limited_fleet_size = 50.0`. **Confirmed** —
+8.6% colony-years against the previous ladder, doubling 284.5 → 265.0 yr, every
seed positive, and an ablation puts roughly seven of the eight points on the
**price** rather than the hold.

**This discharges design law #6 for the ladder:** 1 : 3 : 9 was scaffolding to be
replaced rather than a target, and it has been.

**5.5 `RATIFIED` — the hull a role flies is a decision, not a constant.**
`freighter_hull` scores candidates on `load / round_trip / hull_cost` under a
**liquidity cap** — only hulls the centre can pay for now. **+170.1% ± 16.2
work-years, 8/8 seeds** (R-O94). Without the cap the same change is +51%
work-years and **−17% colony-years**.

**A score of the form `value / cost` is a rate, true in steady state, and says
nothing about the years spent saving for an indivisible purchase.** When a
decision picks among lumpy purchases, price the wait as well as the return.

**5.6 `OPEN` — R-PROD2: what a Production card does to the ladder, if anything.**
The ladder is geometry, so a card cannot move `F_mass = F_cost^(3/2)` without
breaking design law #3. What it *can* move is `eta_works` (the bill), `fab_cap`
(the rate) and the roster (which hulls exist at all — but that is Technology).
**A Production card that made big hulls cheaper would be a Technology card wearing
the wrong colour.**

---

## 6. Objective and guards

**6.1 `RATIFIED` — the objective is fleet-years in *mass*.**

```text
P_i = ∫₀^T F_i(t) dt      // F = fleet dry mass owned by i, kt
```

**Mass, never hull count.** Counting hulls rewards fragmentation and would put
this objective in direct contradiction with design law #3. Since dry mass *is*
mineral cost, fleet-years is also "minerals committed to hulls, integrated" — one
quantity, two readings, no second ladder.

> **A caveat that is load-bearing:** R-O88's ratified "+26–34% fleet-years" was
> taken on the **count**, before `VehicleSnapshot` carried `dry_mass`. It is
> **not** re-denominated, because that would invalidate the figure without
> re-running the comparison.

**6.2 `RATIFIED` — colony-years is not a guard for this tree.** It is **monotone
inverse** for anything that changes how minerals are spent: it *rises* as
development collapses and *falls* as it recovers, on both seeds, across four
industry landings. The build-mix census (`examples/bank_mix`) is what actually
caught every defect on that branch. Appendix §B.5.

**6.3 `RATIFIED` — the mechanism check is the payable fraction.** `bank_mix`
reports what share of banked ore can actually pay a rung. It held at **0.043
through three interventions** that each claimed to move it and moved it to 0.043,
and finally moved to 0.052 under R-O92. **An objective answers "did anything
change"; only a mechanism check answers "did the thing I described change".**

**6.4 `OPEN` — R-PROD3: Production's saturation point on the measurement bed.**
`Hyades_trees_and_card_value.md` §3.2 requires a per-metric saturation measurement
as the *first* measurement. Colony count saturates by ~1,500 yr on this bed;
fleet-years has not been checked, and a card measured past its metric's knee is
measured against nothing.

---

## 7. What is not here

**7.1 Capitals and synthesis.** A production centre that reaches **pop-Band IV**
unlocks capitals and synthesis (world model §5.2). Nothing of this exists in the
engine and it is the natural depth-3 of this tree. **R-PROD4 — open**, and it is
the join with Technology: synthesis makes *supers*, which are a Technology
commodity.

**7.2 Slag.** Wastage degrades to slag rather than vanishing (design law #11), and
slag is not a bank entry yet. **R-O59 / T-03 — open.**

**7.3 Captured infrastructure.** What an occupier gets when it takes a developed
world. **R-IND8 — open**, and it is the join with Warfare.

---

## 8. Register

### Ratified

| Code | Decision |
|---|---|
| R-IND1 | `K = min(hab, bio_max)`; infrastructure is capacity only |
| R-IND15 | the default works mix is `3:2:1` Y:C:M; Sole is an asymptote |
| R-IND18 | extraction saturates once, in the mining law |
| R-IND19 | the deposit law is normalised by `N` — output is not richness squared |
| R-O57 | dry mass *is* mineral cost |
| R-O58 | the shell model — cost on area, hold on volume |
| R-O85 | **resolved false** — the rung ladder is scale-free; colour is the constraint |
| R-O88 | `slips` is quantity, `fab_cap` is quality |
| R-O94 | a hauler's hull is a forecast under a liquidity cap |
| R-MC15 | `F_mass = F_cost^(3/2)` |
| T-68 | `t_build` tracks hull mass |
| T-69 | a yard fills every berth |
| T-75b | works writes are recorded and folded in `CardId` order |
| — | the cost ladder: 1.0 / 10.0 / 50.0, +8.6% colony-years |

### Open

| Code | Question | What would settle it |
|---|---|---|
| R-IND2 | is Warding an employment share or a Design hardness coefficient? | a design pass; changes what Warfare can target |
| R-IND8 | captured infrastructure | a design pass, with Warfare |
| R-O59 / T-03 | slag as a bank entry | engine work |
| R-PROD1 | which `Works` fields each card tier writes | the dispersion constraint, then MC |
| R-PROD2 | what a Production card may do to the hull ladder | a decision — probably "nothing" |
| R-PROD3 | fleet-years' saturation point on the bed | one cheap measurement |
| R-PROD4 | capitals and synthesis at pop-Band IV | a design pass, with Technology |
| T-74 | fabrication rate as a measured quantity | engine work; blocks R-P16 and Growth |
| T-92 | 85% of bank inflow is a centre mining its own planet | a census, then a mechanism |

---

## References

- `Hyades_industry.md` §1 (the stock), §3 (the ramp), §5 (works and colours),
  §6 (the layering algebra and the whole measurement branch)
- `Hyades_experiments_appendix.md` §A, §B — the measurement record
- `Hyades_mineral_cost_curve.md` §2.3 (the shell model), §2.6 (the Band ladder)
- `Hyades_trees_and_card_value.md` §2.3.4 (fleet-years), §3.2 (saturation first),
  §4.3 (the dispersion constraint)
- `Hyades_vehicle_roles.md` §6 · `Hyades_loadout.md` §3.5 (MECH tooling)
- `src/cards.rs` — `Works`, `Employment`, `WorksWrite`, `Works::fold`
- CLAUDE.md design laws #3, #6, #11
