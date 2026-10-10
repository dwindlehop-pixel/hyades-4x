# Hyades — implemented register

The entries below were built, measured or closed, and were moved here from
Band A of `docs/hyades_todo.md` so that Band A lists only work that is ready to
build and not yet done. Each entry is moved **verbatim**, in the order it held in
Band A; its `T-nn` is permanent and is not reused (the register's own rule).
Nothing here is normative: the decisions an entry produced live in the spec it
cites, and its measurements in `docs/Hyades_experiments_appendix.md`.

**Where the open work went.** An entry here may name questions it left open.
They are not open here — each one lives in a spec's register or as its own
`T-nn` in `docs/hyades_todo.md`, as the table says. Band A entries that group
several codes (T-67 … T-77, T-78 … T-80, T-82 … T-86) stay in Band A whole,
because each still holds an open code (T-76; T-78, T-79, T-80; T-86); the codes
inside them marked **LANDED** are implemented.

**The landing ledger** (below the table) is the per-landing account of
T-codes moved and ratified decisions implemented or contradicted, moved here
from `docs/hyades_todo.md` with the entries it describes.

**Moving an entry.** When a Band A entry closes, move it here unchanged, add its
row below, and leave its code in the Band A pointer line of
`docs/hyades_todo.md`.

| code | status | open follow-ups, and where they live |
|---|---|---|
| **T-154** | Built: seats off every material's hue, platinum apex, positions to 0.0001 ly, Hermite interpolation | R-UI1 — `Hyades_interface.md` §9; the `beams` setup and the picket under-fire rule are put to the author |
| **T-153** | Built: armed bodies solid and unarmed hollow; a 3×3 mark per role; a glyph legend from the module | R-UI3 — `Hyades_interface.md` §9 |
| **T-152** | Built: glyphs only where their hulls stand, a display order, routes, holdings by material (Band bars), colonies by seat color, quiet traffic, cargo stripes | R-UI4 — `Hyades_interface.md` §9 |
| **T-151** | Built: one client with a menu; phone layout and touch; Galaxy-level stacks, active hexes, dimmer worlds; the overlay that took every tap removed | R-UI2, R-UI4, R-UI9 — `Hyades_interface.md` §9 |
| **T-150** | Built: the live palette editor; the author's rulings on ratifying live and on upstream packages | R-UI1 (awaits ratification on the live viewer), R-UI8 — `Hyades_interface.md` §9 |
| **T-149** | Built: the replay viewer, its two modes, and deployment to Pages | R-UI1 … R-UI7 — `Hyades_interface.md` §9 |
| **T-133** | Closed (sixth landing): fire on the main loop | R-WAR35, R-WAR37 — warfare register |
| **T-132** | Closed (engine): damage as beam power over time, structure on volume | R-WAR19, R-WAR21 — warfare register |
| **T-130** | Closed: `exp`/`ln` as four-multiply polynomials | — |
| **T-127** | Closed: no host libm in the engine | T-128 (Band A) |
| **T-129** | Closed: Band readings off the run path | — |
| **T-126** | Closed: combat-bed throughput | — |
| **T-125** | Closed (engine): weapons are a Design | R-WAR20 (Warfare card below the 1.5–2.0x target) — warfare register |
| **T-124** | Closed: the rung bill conserves mass | — |
| **T-123** | Closed: the port strike (R-WAR16 resolved) | R-WAR17 — warfare register |
| **T-122** | Landed: where each first card's effect goes | R-IND22 — industry register; T-108 (Band A) |
| **T-121** | Built: the default is unarmed; the Warfare card is the key (R-WAR13 resolved) | — |
| **T-120** | Built: a picket guesses from the bearing (R-WAR10 resolved) | R-WAR14 — warfare register |
| **T-119** | Built: hull composition; the founder pays the floor (R-IND21, R-IND22) | — |
| **T-118** | Built: the mass ledger and its conservation test | — |
| **T-117** | Built, behavior-neutral: the standing layer resolver | — |
| **T-116** | Measured: why `W_0` was negative | — |
| **T-115** | Built: two responders, the moving picket, the armed scout | R-WAR8, R-WAR12 — warfare register |
| **T-114** | Resolved: `tests/determinism.rs` measured 33.6 s at T-134 against the 60 s budget | — |
| **T-113** | Built: the cheap picket, the erected hold, held ground | R-WAR6, R-WAR7 — warfare register |
| **T-112** | Built: pickets and light-lagged diversion; its engagement gate was retired at T-133 | R-WAR6 — warfare register |
| **T-111** | Landed: combat in the simulation loop; superseded by T-133 | — |
| **T-109** | Built: only Warfare may carry a population-lethal write | R-TREE11 — trees register |
| **T-110** | Built as `TIER0[15]` (T-121, T-123, T-125), not in the form written here | T-97 (Band A); R-WAR20 — warfare register |
| **T-96** | Done (R-MC16): the drive is a mass | — |
| **T-101** | Done: the candidate scan prunes what it rejected | — |
| **T-100** | Done: `rank`'s logarithms memoized | — |
| **T-98** | Done (R-O94): the hauler's hull is a forecast | T-99 (Band A) |
| **T-94** | Done (R-O93): the logistic in closed form | T-95 (Band A) |
| **T-91** | Done (R-O92): the milk run | T-92, T-93 (Band A) |
| **T-90** | Implemented and refuted; code reverted | — |
| **T-81** | Landed and measured false; kept as the record | — |
| **T-88** | Closed: decisions decoupled from the economy tick | — |
| **T-01** | Closed with T-83: `matching.rs` wired into `lib.rs` | — |

---

## The landing ledger

**`AGENTS.md` §6 requires every PR to account for the `T-nn` it moved and every
ratified decision it implemented or contradicted.** That rule landed part-way
through the `claude/works` branch, so **this section is it applied retroactively
to the 21 commits that predate it** — the point of a rule like this is a
continuous record, and a record with a hole at its start is one nobody can trust
the rest of.

Grouped by landing rather than by commit, because a landing is the unit a PR
describes.

| Landing | Closed | Advanced | Opened | Ratified decisions **implemented** | Ratified decisions **contradicted** |
|---|---|---|---|---|---|
| **Works 3–4** — infra as a stock, CardId-ordered fold | **T-70**, **T-75a** | T-73 | — | R-O80 (the infra ladder *is* the mineral ladder); design law #16 (fold refuses non-finite); R-IND4 at the algebra level | — |
| **T-70 measured** | — | T-70 | — | — | **My own prediction, not a spec's** — I asserted `rank` reads infrastructure and that bit-identity was therefore impossible. It does not; one grep would have shown it. Infrastructure reaches every live decision through an integer |
| **Works 5 (T-73)** — color-payable bills | **T-73** | T-81 | R-IND15 | §5.1 ("never a true 1:1:1"); design law #13 | — |
| **T-73 measured** | — | — | — | — | **§6.7's neutrality claim for stage 5.** The stage was planned inert and was not: it changes *what a purchase costs*. Deepening fell 94.5% |
| **T-81 + 3:2:1 default** | — | T-81 | R-IND16 | **R-IND15 resolved** — `3:2:1` Y:C:M, Yellow-primary because Production is Yellow | The placeholder `(1,1,1)`, which contradicted §5.1 on the spec's own terms |
| **T-81 measured false** | **T-81** (kept as the record, not reverted) | — | R-IND17 | design law #1 (substitution is the counter-graph's, not the ladder's) | **T-81's own premise.** Relief routing *anti-concentrates*: builds fell 57 → 31. A three-color bill needs colors arriving together, and relief scatters them |
| **R-IND17** — bill-completion routing | **R-IND17** | T-77 | — | — | **The spec's own formula.** §6.11 divided by `Σ bill`, which ties a center needing one color with one needing everything. Corrected to `short_before` before implementing |
| **Politics §10 spec** | — | T-82, T-86 | R-P16, R-P10 resolved | **R-P10 decided** (clear per round: a continuous book makes price a function of event ordering) | R-IND10 marked resolved — **the register was stale**, §3.3 had already answered it |
| **T-74** — MM rate curve | **T-74** | — | — | §6.3's two-parameter curve, anchored so a rung-I center reproduces the flat constants | — |
| **T-69** — slips | **T-69**, **T-75b** | — | R-IND18 | §3.2's soft floor; **R-IND4 resolved** at both levels | **§3.2's prose.** "`F/slips → F_slip` from above" is the reciprocal of what `slips` does; the `1 +` is load-bearing and dropping it inverts the design property. Also **§3.3's schedule** becomes an asymptote rather than a reading |
| **T-71 + T-72** — crowding | **T-71**, **T-72** | — | R-IND19, R-IND20 | §4.2's vein ladder; §4.3's ratios; design law #16 (vein clamp) | Three: **§4.3's `ε·S·W`** double-counts the deposit (R-IND19); **§6.3 vs §4.3** both claimed extraction (§4.3a); **T-57's ratified crew of 3** does not survive a law with a deposit term |
| **T-87** — crew from demand | **T-87** | T-52 | R-IND20, **T-88** | Author's ruling: crew is not a parameter | **`miner_vein_fraction` itself**, shipped one landing earlier. And **§4.5's "fewer"** — site count is identical to the digit; that half was never crowding's to deliver |
| **Exchange 1 (T-82)** | **T-82** | T-85 | R-P17 | R-P1 (`$` is a claim, zero mass); design law #16 at the single writer; **the §10.6 outpost amendment** | **R-P16's placeholder.** §10.3 shipped the faucet against infra *because* T-74 had not landed. It had. Resolved, not shipped |
| **Exchange 2 (T-83)** | **T-83**, **T-01** | — | — | §3.1's cross-empire book shape; §10.5's canonical order (why `Basic: Ord`) | **§10.0's audit.** It called `matching.rs` "built, not wired" — it was never in `lib.rs`'s module list, so it did not compile and its tests never ran in CI |
| **Exchange 3 (T-84)** | **T-84** | T-85, T-86 | — | §3.2's `wtp`; **§10.4's field list** — which is what T-11/R-O27 was holding open | — |

**Since the rule landed**, each commit carries its own accounting in its message;
these rows keep the table continuous so the ledger can be read in one place.

| Landing | Closed | Advanced | Opened | Ratified decisions **implemented** | Ratified decisions **contradicted** |
|---|---|---|---|---|---|
| **Exchange 4 (T-85)** — clearing into escrowed contracts | **T-85** | T-77, T-86 | — | §10.6's two-location contract and FTL debt (the author's own amendment); design law #15 (settlement strikes at the round barrier, a protocol sync point) | **R-P17's venue rule.** One venue per pair is wrong when the two shippers are light-years apart; revised to per-shipper nearest |
| **Exchange 5 (T-77)** — settlement | **T-77** | T-86 | **R-P18** | §10.6a's freight leg; §8.1 (refined mass traverses real space) | — |
| **§10.6a corrected** | — | — | — | — | **My own screen.** A 400-planet bed reported 776 contracts / 327 kt and I concluded volume did not matter; the full bed is 64,642 / 29,460 kt. A conclusion of the form "X does not matter" cannot be drawn from a screen at all |
| **R-IND21 opened, then withdrawn** | — | T-51 | R-IND21, then withdrawn | — | **R-IND21, by me, one commit later.** The measurement was right and the conclusion backwards: `unmet_color_demand` reads the next rung only, so it measures demand the policy already decided to express |
| **T-51 / R-O68** — the deepen/expand trade | **T-51**, **R-O68** | — | **T-89**, **R-O85** | Both sides of the comparison in one unit, using `rank`'s own `w_k` rather than a new constant; design law #16 (the `per_kt` floor exists because `0.0 * inf` is `NaN`); `AGENTS.md` §2's ablation-before-explanation — the fix was measured against the old binary on both seeds before it was believed | **This file's own T-51 prescription.** Item 3 said fixing the units would make `reinvest_bias` "a preference over a real trade". It did not: the trade is real now and expansion still wins it by 24–49x, because an infra rung costs nine colonizers. The dead branch was the right answer reached for a wrong reason, and the cause moved to R-O85. Also **`reinvest_bias_is_a_step_function_not_a_dial`**, the characterization test that existed to stop this changing silently — replaced, deliberately, by the test that pins the new form |
| **R-O86** — a scout needs somewhere to scout | **R-O86** | T-89 | — | Design law #11, restored on the engine's busiest path — `apply_build_with` was debiting the bank and holding the yard for a hull `launch_survey` then declined to spawn; `AGENTS.md` §4's "hand a decision only the fields it reads" (`survey_frontier` is `O(1)` off a running integer count, not a walk) | **R-AC16's magnitude.** `survey_reserve = 1024` is compared against a quantity with median **0** and maximum **164**, so the test is a constant `true` and every value above ~200 is bit-identical. The direction the ratification argued is fine; the number never reached the simulation. Also **the code's own justification** for the `candidates.is_empty()` pre-emption — "no candidates means every other branch below returns Idle" — which is false and was swallowing the only live deepen path |
| **R-O87** — `reinvest_bias` against work-years | **R-O87** | T-89 | — | `AGENTS.md` §2's 2-SE bar, enforced against my own candidate rather than someone else's; design law #11 surfacing as the works-per-mineral identity that makes the knob neutral | **Nothing ratified** — and that is the point: the candidate cleared the bar on the standard bed (+2.33% ± 0.96, 4/4 seeds) and was refuted by an independent seed set (−1.70% ± 2.42). `reinvest_bias` is **held at 0.5**, not re-ratified. The contradiction is of my own screen, not of a spec |
| **§6.19a** — the identity was half an argument | — | **T-88** (quantified), T-89 | — | `AGENTS.md` §2's "never leave a symptom without a proven mechanism", applied to my own flat result: the *flow* side was priced off the engine's functions and the bottleneck named and measured rather than asserted | **My own §6.19 reasoning.** "Everything downstream breaks the tie for expansion" was asserted and is wrong — rung I→II is **+29% hull/yr for nine colonizers**, payback 62 yr. The conclusion (hold 0.5) survives; the argument for it did not. Author's objection, correct |
| **R-O88** — there is no build-wide axis | — | T-88, T-89 | **R-O88** | A characterization test pinning a contradiction rather than a number, so neither spec section can drift further apart silently | **Two ratified sections at once.** §3.2's "`slips` scales without limit" is false in the engine (`fab_cap / slip_throughput = 2` is the whole axis, and it is closed — 10¹² kt still buys two berths). §6.3's reconciliation of the two is a **non-sequitur** and is withdrawn: it defends a per-center claim with an empire-wide fact. Author's objection, correct |
| **R-O88 resolved** — option C, split `F`'s two roles | **R-O88** | T-89 | — | §3.2's unbounded build-wide axis and §6.3's tree meanings for `cap`/`half`, both made true at once by denominating the ceiling per berth; `AGENTS.md` §2's "when a change raises entity count, check the test horizons in the same commit" — unit 19→54 s and determinism 30→58 s, both fixed in this commit with `events_processed` floors guarding the trims | **`SimConfig::slip_throughput` deleted** and `fab_cap` 0.2 → 0.1. Neither is a retune: per-berth turnaround is bit-identical at every playable rung, and §3.3's approved schedule now reads off `fab_cap` directly. Also supersedes **T-88's own headline measurement** — the after-idle gap is 29.6 → 7.6 yr, so 81%-of-timeline no longer holds |
| **R-O85 resolved** — the ladder is fine, freight is not | **R-O85** | **T-76** (promoted to first) | — | `AGENTS.md` §2's "measure the utilization of whatever the knob buys", applied to a *price* — the ladder is scale-free (1.8-yr payback at every rung) so there is nothing to ratify; and logging the decision's own predicate rather than reconstructing it from totals, which inverted the answer | **R-O85's own premise.** Infrastructure is not priced out of reach: the bed banks 1,714,697 kt against a 19 kt rung. Counted per decision, **0%** gated and **0.7–0.9%** outbid — so R-O68's crossover, which three sections circled, is consulted in one decision per hundred and cannot have been the cause of anything. **43.8–46.6% of all decisions hold the total and lack a color** |
| **R-O89** — freight loads what the destination is short of | **R-O89** | **T-76** (the load half; the routing half stays open) | — | `AGENTS.md` §2's **replicate on seeds the candidate was not chosen against** — +8.21% ± 3.06 on the standard bed, **+8.60% ± 2.61 on seeds 2/3/5/11**, pooled **+8.40% ± 1.86, 8/8**; and its **ablate before you explain** — the 2×2 ran before either arm was believed, and it is what saved the +8.4% from being buried under the −52.3% they scored together. T-73's color-payable bill and design law #1's color semantics, finally reaching the load leg | **§6.19c's instruction not to re-sweep anything mineral-side is lifted** — `outpost_mining_fraction`, both crew policies, `reinvest_bias` and the Exchange were each measured flat against this wall and are now re-measurable (none is re-measured here). Also **my own first attempt**: need-routing the *pickup* leg is −52.3%, and transit, per-hull throughput and hull recycling are each measured not to be why — the residual is left open under T-76 rather than given a story |
| **Tree gradient** — rank knobs against every tree | — | **T-45** (superseded and broadened 9 → 32 knobs), **T-50** (first raw dataset landed), T-78 | **R-O90**, **R-TREE8**, **R-TREE9** | Trees §2.1's "one global objective is wrong for five of six trees", finally applied to the *tuning* loop and not only to card costing; §2.3.4's mass reading of Production, which needed `VehicleSnapshot::dry_mass`; T-50's "persist the raw per-seed evaluations, not the summary" | **T-45's whole table.** Nine knobs at the pre-R-O66 operating point, ranked on coverage. `medium_fleet_size` has **flipped sign** (+32.7 "raise" → −3.93 and a cliff), `cargo_unit_size` went from **inert** to third-largest — design law #14's corollary about knobs that look dead while expansion is broken, confirmed — and `biosphere_regen_rate`, the **+141.2 ± 18.1 headline lever**, is now **bit-identically zero** because T-67 took infrastructure out of `K`. Also **trees §2.3.2 and §2.3.6**: Warfare is unreadable on the standard bed and Politics has no objective distinct from Expansion |
| **T-88** — granularity without decision rate | **T-88** | T-52 (named, not fixed), T-24 | the idle census (see T-88) | The author's directive that opened it — *"move all decision making to trigger off an event so the economy tick can be made 1/year"*; `AGENTS.md` §4's "entities evaluate on their own arrival events, never on a sweep", now true of the production decision as well as the ship; §2's "when a change raises event count, check the test horizons in the same commit" (all four targets rescaled here); and its replicate-on-fresh-seeds rule, run on both stage B and the new default | **`cycle_years` 50 → 5**, a ratified default moved on measurement plus an explicit directive. **And a units defect in three Monte-Carlo-tuned rates**: `growth_rate`, `biosphere_regen_rate` and `outpost_mining_fraction` were applied per *tick* irrespective of tick length, so the first sweep's +58.6% was mostly artifact. `tick_scale` fixes the denomination without moving a magnitude (bit-identical at the old cadence). **Every gradient measured before this is consumed**, including `data/tree_gradient.tsv` and T-45's table |
| **T-90** — live local scarcity | **T-90** (refuted), **R-O91** (answered) | T-76 | **T-91** | `AGENTS.md` §2's rule that a fix needs a *mechanism* check beside the objective — written down before the measurement and it is what refused this one; and "probe past the value you intend to ship", which is what separated *inert axis* from *gain too small* | **My own diagnosis from the previous landing.** §7.4 said outpost selection being color-blind was why banks are mono-colored. The term is genuinely a constant and correcting it changes nothing: the empire already mines a balanced mix, and **a hold is filled from exactly one rock**, so every delivery is mono-colored however the rocks are chosen. Code reverted; the diagnosis is marked refuted where it was written rather than quietly dropped |
| **T-91 / R-O92** — the milk run | **T-91**, **R-O92** | T-76 | **T-92**, **T-93** | `AGENTS.md` §2's **replicate on seeds the candidate was not chosen against** (8/8, error bar *tightened*) and **probe past the value you intend to ship** — 4 and 6 are where it breaks and that is why 2 is known to be a peak rather than a direction; the mechanism check written down in advance, which had refused T-90 and passed this; R-O89's welded pickup site left alone rather than re-litigated | **§6.23's closing sentence**, by me — *"no loading rule and no delivery rule reaches that, because both act on ore that has already been mined from the wrong rocks"*. A loading rule does reach it, and is +55%. The sentence was right about *which ore* and wrong that nothing could mix it, because it assumed the voyage shape it was written under. **And T-91's own premise is now half-refuted**: the atomicity was real, and removing it exposed that freight is only 1.73% of what enters a bank (T-92) |
| **T-94 / R-O93** — the logistic is solved, not stepped | **T-94**, **R-O93** | T-24 | **T-95** | `AGENTS.md` §2's **replicate on seeds the candidate was not chosen against** (8/8, 5.7 SE); its `yr/s` **and** `ns/event` rule, which is what shows the `exp` is free rather than assumed to be; design law #16 at the denominator | **Design law #11's `r < 2` ceiling**, which T-64 derived from the conjugacy to the logistic map and which is a property of the *Euler step*, not of the model — the closed form is monotone at any rate. **R-O84's ratification of `growth_rate = 0.873`**, whose operating point and plateau map are both consumed; the value is carried, not re-measured. And **the undershoot below `K`**, which T-67 cited as supporting evidence for taking infrastructure out of the minimum — T-67's conclusion stands on its own (razing works must not move people) but that particular argument was resting on truncation error |
| **T-96 / R-MC16** — the drive is a mass, not a stat | **T-96**, **R-MC16** | R-O65 (still blocked), T-24 | **T-97**, **T-98** | R-MC16's own words — volume is the ENG ceiling, realized thrust is a Design quantity paid for in minerals; design law #11 (the drive masses and costs what it occupies) and #3 (the consolidation advantage stops being repaid in turnaround); R-O57 (cost stays exactly dry mass); `AGENTS.md` §2's replication rule and its instruction to write the mechanism check down first — the round-trip ratio, which moved 1.252 → 1.011 | **R-O58's `a_empty` is size-independent**, which was true only while thrust *was* dry mass — empty acceleration now rises with size (1.00 / 2.37 / 5.06 g), which is the point. **§3.3's build schedule** 2.2/3.0/12.0 → 2.201/3.092/15.154, because `t_build` tracks hull mass and a hull now masses its drive. **The founding-infra rung coincidence** drifts +0.003/+0.038/+0.119 Bands — R-O80's claim about the two ladders is untouched (the shell still prices exactly at its rung) and R-O87's works identity survives, because both sides moved together |
| **T-98 / R-O94** — the hauler's hull is a forecast | **T-98**, **R-O94** | **T-92** (freight 1.73% -> 14.70% of bank inflow), T-24 | **T-99** | `AGENTS.md`'s **ablate them apart before you believe either** — the liquidity term is the whole difference between +170%/+9.5% and +51%/-17%, and landed as one change the honest response would have been to revert; the mechanism check written down in advance (freight's share of bank inflow, not the objective); design law #3 in both directions — its cost basis *and* its named counterweight, indivisibility; `AGENTS.md` section 4's `O(1)` rule at the decision | **`role_hull_type(Role::Freighter)`'s stated rationale**, "spec: MSV/GSV, picking the cheaper" — correct under the pre-R-O58 ladder and backwards since R-O58 made cost per kilotonne hauled 0.109 against 0.032. It survived because the General hull's turnaround made it a bad idea for an unrelated reason, which T-96 removed. **And colony count falls 6.1%** — recorded rather than left to be found, and not a defect: colony-years rise 9.5% on the same bed |
| **T-149** — the game interface: replay, viewer, Pages | **T-149** | — | **R-UI1 … R-UI7** | Design law #15 (the viewer reads a replay and does not link the engine; `snapshot_at` and `next_event_time` are reads); design law #16 (the recorder refuses a non-finite number); `AGENTS.md` §4's determinism (one seed, one replay, byte for byte) and its 60-second rule (every test target measured, the longest 34.5 s) | **The recorder's own first frame rule**, one commit earlier: frames at the first event after their year drifted onto event times and repeated once a run's events stopped (appendix §D.58.1); replaced by `snapshot_at` |
| **T-150** — the palette is tuned and ratified live | **T-150** | R-UI1 | **R-UI8** | The author's rulings 13 (ratify on a live design) and 14 (the interface and networking may link upstream packages; the engine stays dependency-free); design law #15 (the palette is presentation state and never reaches the engine) | **`Hyades_interface.md` §9's own settle-by for R-UI1** ("approval of `palette.html`"), one landing old — the author ruled a sheet cannot ratify a palette; replaced by a settings line or link from the live editor. **`AGENTS.md` §4's zero-dependency rule**, narrowed by the author's ruling to the engine |
| **T-151** — the replay viewer is a menu option of one client; phones | **T-151** | R-UI2, R-UI4 | **R-UI9** | The author's ruling 15 (the replay viewer is a menu option of the game client); design law #15 (the client reads replays and the palette is presentation state); `AGENTS.md` §2's rule to assert that a mechanism fires (the browser test now clicks and taps a hull the module reports drawn — before, nothing tested a pick through the page, and an invisible overlay took every one) | **`Hyades_interface.md` §6.4's stack rule** (one glyph per owner, Design, role and wreck state in a 3-pixel square at every level), replaced at the author's report that tactical stacked too many glyphs; **§6.5's hexes** (every hex drawn), replaced by active hexes at the author's report; **§7.1's world lights** (core 1.5, halo 0.35, bloom 0.6), dimmed at the author's report that glowing worlds made everything illegible — all placeholders, none ratified; **`palette.html` as a page** (§6.2), now a screen |
| **T-152** — the tactical layer: glyphs in place, routes, holdings, cargo | **T-152** | R-UI4, R-UI1 (material colors proposed) | — | The author's rulings 16 (a glyph only centered on its hull's location), 17 (Band math, not a log) and 18 (armed hulls stay prominent); design law #15 (the snapshot gains holdings as a read; nothing flows back); `AGENTS.md` §4 (a Band is a reading the engine takes — the replay carries it, the viewer does not re-derive it) | **`Hyades_interface.md` §6.4's fan** (unalike stacks offset with leader lines), moved to appendix §D.58.6; **§6.5's world dot** (a colony now a pixel wider than an unowned world); **§6.2's Laden status** (a center pixel, now the fill only where a replay lacks cargo by material); **the author's literal quiet rule**, narrowed to unarmed hulls because it drew the `beams` fight as one pixel — then ratified as ruling 18 |
| **T-153** — armed and role at a glance | **T-153** | R-UI3, R-UI1 (`fill` 0.45 → 0.7, proposed) | — | Ruling 18 (armed hulls stay prominent — now by body as well as by quietness); design law #15 (the legend is presentation, read from the module) | **`Hyades_interface.md` §6.3's role plus and armament-only marks**, replaced at the author's report that neither armed state nor role read at a glance; **the proposed `fill` 0.45**, moved to 0.7 so a hollow body reads against a solid one — both placeholders |
| **T-154** — seats apart from materials; smooth motion | **T-154** | R-UI1 | — | The author's rulings 19 (a seat never wears a material's identity; the apex is platinum) and 20 (smooth continuous acceleration in the display); design law #15 | **`Hyades_interface.md` §6.2's seat families** (each archetype's seats from its own super's hues — the first three seats were the blue, red and green materials), replaced; **§6.2's Strange Matter `hy_violet3`**, now platinum; **§4's straight-line interpolation** and **§3's two-decimal positions**, which together drew a fight in 0.01-ly steps (appendix §D.58.7) |

**Two things this retrospective surfaced that no individual commit had said out
loud.**

**Ten contradictions in twenty-one commits, and every one was load-bearing.**
Four were spec formulas or prose that were simply wrong (§3.2's reciprocal,
§4.3's double-count, §6.11's divisor, §10.0's audit); three were ratified values
that stopped meaning anything under a changed model; two were stage-neutrality
claims that did not hold; one was a stale register entry. **None of them was a
reason to stop, and all of them would have been invisible a month later.** That
ratio is the argument for the rule: contradiction is the normal case in a
project that measures things, so the cost of announcing one has to be near zero
or it will not happen.

**The neutrality claims failed twice, the same way, and the tell was available
both times.** §6.7 planned works stages 3–5 inert and stage 5 was not; §10.7 plans
Exchange stages 1–3 inert and they were, because §10.7 was written *after* that
lesson and says so. The distinguishing property is stated in both places now: a
stage that changes **what a purchase costs** is never neutral. Check a
neutrality claim against the code that would have to be neutral, not against the
description of the change.

---

## Entries moved from Band A

### T-164. Counts are of hulls on their role; retreat apart; no wrecks

*Opened and built in one landing at the author's ruling: "Only count hulls
that are still embarking on their primary role. If autopilot decides to
retreat, that's a distinct count color. If a ship is wrecked, the wreck is not
part of a count."* The engine marks a withdrawing hull from `fire::withdraw`
until it arrives home or is wrecked (`World::withdrawing`,
`VehicleSnapshot::withdrawing`, replay flags bit 2). A tactical place shows
its count of hulls on their role and, in a new retreat color (`#ffaba1`), its
count heading home; wrecks never stack with live hulls, so galaxy markers are
sized by live hulls; juicy draws hulls heading home as their own cluster in
the retreat color (`Hyades_interface.md` ruling 29, §3, §6.2, §6.4, §7.1).

### T-163. Juicy stacks show their owner and their count under fire

*Opened and built in one landing at the author's report on the `beams`
replay: "I can't tell which seat owns which stack in juicy mode on beams when
they are being attacked. I also can't tell when the stack decreases in
number."* Twenty hulls at one point were twenty lights summed into one blob
that the tone curve saturated, and a hit flash covered it. Below the galaxy
level a stack is now a spiral of dots, one per hull in its seat's color;
seats whose clusters would overlap stand apart; a hit is a ring about the
cluster (`Hyades_interface.md` ruling 27, §7.1).

### T-162. Hex brightness shows each seat's work-years, in both modes

*Opened and built in one landing at the author's rulings: "The juicy hex
brightness is important, but it should not show deltas. The color should show
the work-years of the seat (with a bright line on well established hexes).
Multiple lines for intermixed hexes. Special line for Band IV works. I need a
tactical hex brightness, also."* Each hex carries a line per seat with works
in it, in the seat's color, brightening with the seat's work-years there to an
established mark, then tinted bright; the edge is dashed bright where a world
holds `Band IV` works. Work-years are integrated per hex and seat when a
replay is read. T-159's trend is superseded (`Hyades_interface.md` rulings
25–26, §6.5, §7.1; magnitudes placeholders under R-UI2 and R-UI4).

### T-161. The galaxy is its prescribed hexes

*Opened and built in one landing at the author's rulings: "there's no purpose
in single planet hexes. The three seat galaxy should have 3 starting hexes
plus 9 hexes. Planets outside the prescribed hexes should be clipped and not
generated. No need to change the math, just discard planets generated that
fall outside the target regions", and "No empty center confirmed".*
Generation discards every wild world outside the homeworld hexes, the ring
around them and what the homeworld ring encloses
(`GalaxyConfig::kept_hexes`, `hex_rings_kept` = 1); a kept world is bit for
bit the world it was (`Hyades_galaxy_and_autopilot.md` §1; appendix §D.59).
AGENTS.md §4's "no hexes in the engine" is amended: the simulation reads no
hex, and generation reads two.

### T-160. A paused tactical view no longer flickers as it is panned

*Opened and built in one landing at the author's report: "The tactical view
flickers when paused and panning. Glyphs change in size and ordering."*
Two causes, both a lattice fixed to the screen. Stacks were cells of the
screen, so a pan moved hulls between them and changed counts, marker sizes and
the glyph drawn on top; and each glyph's pixel was its own screen position
rounded, so in `beams` two stacks less than a pixel apart shared a pixel or
not as the view moved — the author's second report, "the pointy part of the
glyph gets separated or smushed together". Both now tile the galaxy at the
view's scale, with the pan a whole number of pixels (`tactical::stack_cell`,
`tactical::pixel_of`; `Hyades_interface.md` §6.4, ruling 28).
`panning_keeps_every_stack_and_the_drawing_order` fails on the old stack
lattice at a tenth of a cell and on per-glyph rounding at four tenths of one.

### T-159. Juicy hexes and the works trend within each

*Opened and built in one landing at the author's rulings: "Juicy needs to
display hexes" and "Juicy should provide a visual indication of the works
trend within a hex."* The replay writes each world's works as a mass
(`works_kt`), which the viewer sums per hex — hexes stay a presentation
concept. Juicy draws each active hex as a dim line of lights inset from its
edge, lit only where it is on screen; the line brightens and whitens as the
hex's works grow over the last four frames and darkens as they fall
(`Hyades_interface.md` rulings 23–24, §3, §7.1; magnitudes placeholders under
R-UI2).

### T-158. Wrecks fade to pinpoints after 250 ms

*Opened and built in one landing at the author's ruling: "Wrecks should fade
to pinpoints after 250 ms."* The snapshot carries the time a hull was wrecked
(`VehicleSnapshot::wrecked_at`, replacing the `wrecked` flag, which is now a
method), and the replay's hull table writes it (`wrecked_at`). The viewer shows
a wreck from that instant, and the tactical glyph fades toward the ground over
0.25 s of wall time at the playback rate, leaving one pixel in the wreck
color; the juicy ember narrows to a pinpoint with it (`Hyades_interface.md`
ruling 22, §3, §6.6, §7.1).

### T-157. Play forward after playing backward

*Opened and built in one landing at the author's report: "Playing replay
backwards seems to lock the viewer into negative timeflow. I can't get the
viewer to play forward again."* ▶ and Space toggled play without resetting the
direction ◀ had set, so they paused and resumed the backward run; only the L
key turned it. ▶ now plays forward and ◀ backward, each pausing when already
playing its way; Space pauses, or plays forward (`Hyades_interface.md` §8.3).
The browser test plays backward, presses ▶ and checks the clock runs forward;
it fails on the previous `shell.js`.

### T-156. The `beams` Tors can match the Cairns' velocity

*Opened and built in one landing at the author's report: "the Tor initial
delta V is too large. They need the option to match velocity with the Cairn
stack without overshooting."* The replay recorder sets the Tors' starting
speed from their own braking acceleration so that they come to rest at the
Cairns' place: 0.2326 ly/yr in place of 0.3 (appendix §D.58.8). The
determinism suite keeps 0.3 ly/yr.

### T-155. The client names resources by their ratified fiction

*Opened and built in one landing at the author's direction: "The game client
should use the ratified fictional names for resources."* The inspector and a
new material key on the page's glyph legend show Cage Ice, Rosepeter,
Voltslate and Strange Matter (`Hyades_galaxy_and_autopilot.md` §4.1;
`Hyades_interface.md` ruling 21, §6.6). The supers keep their colors as names
until T-142 ratifies theirs. The replay format and the engine's log keep the
engine's color names.

### T-154. Seats apart from the materials; smooth motion

*Opened and built in one landing at the author's report on the `beams`
replay: "The player/seat distinctions cannot overlap with the CMY/RGB/Platinum
identity of basics/supers/apex" and "the animation shows discrete steps with
separate slow in/slow out … The sim and the display should both reflect smooth
continuous acceleration and deceleration."* Seats take OKLCH hues between the
materials', tested at least 25° and 0.08 in OKLab from every material
(`Hyades_interface.md` §6.2, ruling 19); Strange Matter is platinum. The steps
were two-decimal positions on a fight 0.03 ly across; positions are written to
four decimals and the viewer interpolates on the Hermite curve through each
frame's position and velocity (§3, §4, ruling 20). The sim's motion was
already continuous: constant-acceleration legs with a braking prefix
(appendix §D.58.7). The fight's own behavior — the Tors closing to fire
distance and the 18 Cairns staying — follows the replay's setup and the
ratified under-fire rule, and is put to the author.

### T-153. Armed or not, and role, at a glance

*Opened and built in one landing at the author's report on the T-152
deployment: "I can't distinguish the glyphs between armed and unarmed ships. I
can't tell role at a glance."* Armament was a two-pixel spike or two ear
pixels; role was a plus at the center in a pale accent, and the accents are
all pale tints. A glyph's body is now solid when the hull is armed and hollow
when it is not, galaxy markers split the same way, and the role is a 3×3 mark
distinct by shape — so neither reading depends on color
(`Hyades_interface.md` §6.3). Glyphs grew a pixel each side to hold the mark in
every shape. The side panel's legend draws the marks from the module. Open:
R-UI3.

### T-152. The tactical layer: glyphs where their hulls stand, routes, holdings and cargo

*Opened and built in one landing at the author's direction after the T-151
deployment: "The tactical view has weird distracting glitches with the little
lines. Let's define a display order based on role, hull, and distance from
homeworld rather than utilize the offset line. Glyphs should only appear
centered on their actual location. The tactical view needs to show routes as
very faint lines. I want to see colony by seat via color. I want to see
Holdings by vertical line per item on the book in each glyph. I also want to
have scouts and unladen systems/contact vehicles to be unobtrusive. Loaded
vehicles should reflect the composition of their cargo graphically." — and,
during it, "Use Band math instead of log."* The engine's snapshot gains every
empire's holdings (`Snapshot::holdings`); the replay carries each hull's cargo
by material and each holding as cost-ladder Band readings (`Hyades_interface.md`
§3). Tactical mode draws every glyph at its own position in a display order
(§6.4), faint routes and colonies in their seat's color (§6.5), holding bars at
two pixels per Band (§6.5), quiet traffic as one dim pixel and laden holds as
stripes of their materials (§6.6). Armed hulls are exempt from quietness: the
direction as written drew the `beams` fight as one pixel (appendix §D.58.6),
and the author ruled that armed hulls stay prominent (ruling 18). Open: R-UI4.

### T-151. The replay viewer is a menu option of one client; the client works on a phone

*Opened and built in one landing at the author's reports after the first phone
review: "Juicy gets a 503 on sentries", "Glowing planets make everything
illegible", "Tactical is stacking too many glyphs", "The galaxy viewport is way
too small on my phone", "The galaxy view is showing a huge number of empty
hexes. It should only show active hexes", "I can't select any entity on my
phone", and "Replay viewer should be a menu option outside the game client, not
separate".* The site opens on a menu (New game, shown as not built; Replays;
Palette); the viewer is a screen with a back button, and the link names the
screen (`Hyades_interface.md` §8.1, ruling 15). On a phone the theater fills
the screen and the panel and log open as sheets over it (§8.2); a tap picks
over a wider radius than a click, two fingers pinch, and loads retry on a
server error (§8.3). At the Galaxy level a place's hulls are one marker per
owner and a fan stops at four (§6.4); only hexes holding a world or a hull are
drawn (§6.5); worlds are faint and hulls carry the juicy scene (§7.1). A hidden
overlay that took every pointer event is removed; it is the likely reason
nothing could be selected (appendix §D.58.5). The 503 was not reproduced.
Open: R-UI2, R-UI4, R-UI9.

### T-150. The palette is ratified on a live design — a live editor, and the settings it ratifies on

*Opened and built in one landing at the author's direction after the first
deployment: "I can't ratify based on a palette. I need to ratify on a live
design." and "For the game interface and networking, I relax my prohibition
against linking against upstream packages."* The viewer's side panel tunes the
tone map, the glyph fill's dimming and any palette or status color by hand
while the game is drawn; the settings are one line of text the module parses
(`palette::Settings`), carried in the page's link. Both rulings are in
`docs/Hyades_interface.md` §1 (13, 14); the editor is §6.2.1. Open: R-UI1
(the author's ratification) and R-UI8 (which packages, if any, to take up).

### T-149. The game interface — replay, rewind, seek, filter; a tactical and a juicy mode; deployed from `main`

*Opened and built in one landing at the author's request; it never sat in Band
A.* The decisions are in `docs/Hyades_interface.md`, the measurements in the
appendix §D.58. What landed: the replay format (engine, `src/replay.rs`,
`Simulation::snapshot_at`), the `hyades-viewer` crate (playback, the log
filter, the camera and levels of detail, Design glyphs, the tactical renderer
with stacks, the juicy light list with a CPU reference renderer, the C
interface for wasm32), the web shell with a WebGL2 juicy renderer, the
palette sheet for the author's approval, a module smoke test, a browser test,
CI's `viewer` job and the Pages workflow. Open: R-UI1 (the palette awaits
approval) through R-UI7.

### T-133. No engagements and no fight sites — encounters along trajectories, decided by Doctrine

**Closed (sixth landing).** Every stage below is built and the harness-only
sim code is gone (the author's ruling: *"Harnesses and test beds cannot have
special sim code. The only thing that can vary is the galaxy generation."*).
Opened and resolved R-WAR34 (the author's ruling: a wrecked hull continues on
its course), opened R-WAR35 (belief
event A reads a course change before its light arrives) and R-WAR36 (the arena
against the harness ruling); resolved R-WAR22, R-WAR25, R-WAR26, R-WAR30 and
R-L2; R-WAR29's budget is measured with detection on every trajectory
(appendix §D.16).

*History, kept as the record of how it got here:* `Hyades_warfare_tree.md` §8.19
carries the rulings (no engagements; no hardwired fight sites; being fired upon
is not consent; a pitched battle needs both sides' Doctrine; the wreck roll
resolves combat and transport together; the colony ship tries to found before it
is destroyed) and the recommended mechanism (R-WAR25). Priced in appendix §D.11.

**What the engine does today, and has to stop doing:** it fights at three
hardwired sites — a shared rock (`sys_engagement`), a held world
(`resolve_picket_fight`) and a blockaded port (`strike_at_port`) — through a
stationary resolver that holds both sides in place up to
`engagement_horizon_years`, and a colony ship that survives turns back.

**Where it stands (second landing):** stages 1–3 are built, and stage 5 in an
interim form — the port and the held world still decide *where* the engine looks
for fire, but nobody is held there any longer: a colony ship flies through the
fire, takes a wreck roll, and flies on or founds. The author's answers are
recorded (R-WAR24's threshold rule, R-WAR26's three endings, R-WAR27's hold
fire, `σ` per Design class) and every Design the engine builds is named
(R-O42b). Measurements in appendix §D.12.

**Amended by the author (third landing): fights run on the main event loop,
concurrently with production and travel, and the loop processes weapons
discharge and course adjustment** (warfare §8.19.7). That retires the pass
resolver built in stage 2 as an off-clock integration. The stages below are
re-cut accordingly; stages 1 and 3 stand.

**Fourth landing: the author ruled the wreck roll, the survivor and the course
adjustment** (warfare §8.19.2, §8.19.5, §8.19.7). The threshold is the structure,
a soft maximum of hit points; the roll repeats with further damage, with odds
from damage above it; a colony ship that survives fire leaves; course adjustment
is flight from a moving start on belief events, decided per fleet; fire control
may take at most 25% of run time (R-WAR29). Stage 1 is rebuilt to the ruling,
the arriving survivor leaves, and the fire-control share is measured: **0.77% of
instructions** on the twelve-seat card bed, so at today's encounter count the
budget admits any discharge period above 0.011 days; a period of at most 5%
of the shortest exposure the Design must resolve is recommended (0.3–4.7 days
at the arena's accuracy), and the budget has to be re-measured once stage 4 sets
the encounter count (R-WAR29, appendix §D.14).

**Fifth landing: engagement range depends on weapon accuracy** (the author's
ruling, R-WAR23 resolved). Accuracy is per Design class, the range is derived
from it against the reference target (7.90e-3 ly at the arena's accuracy), and
the 0.01 ly placeholder is gone. Which roles narrow the range is R-WAR33
(appendix §D.15).

**Stages, in order, each measured before the next:**

1. ~~**The wreck roll**~~ — **done, rebuilt at the fourth landing**: a Weibull
   wreck point per hull past its structure (`combat::wreck_point_kj`), checked on
   every hit, damage carried across encounters (`hull_damage`); run-path
   arithmetic, no division. Placeholders `x₀ = 1`, `γ = ½` (R-WAR24).
2. ~~**The pass resolver**~~ — **done**: `combat::resolve_pass`, fire between
   hulls on arbitrary paths; a hull at its wreck point stops firing and is no
   longer a target, and the pass stops once no outcome can change.
3. ~~**Fire distances**~~ — **done**: on every Design's loadout, both the
   engagement range derived from the Design's accuracy (7.90e-3 ly at the
   arena's; R-WAR23 resolved, appendix §D.15); `Standing::fire_distance` holds
   fire where Doctrine ignores one, and is where a role would narrow the range
   (R-WAR33); default holds fire on neutrals except in the picket role.
4. ~~**Encounter detection**~~ — **done**: on every trajectory change, the first
   entry into fire distance against every rival that fires or is fired on —
   closed form against a hull at rest, conservative advancement between two
   moving hulls — scheduled as events (warfare §8.19.3).
5. ~~**Retire the three sites**~~ — **done**: `sys_engagement`,
   `resolve_picket_fight`, `strike_at_port` and `encounter_at` are deleted; the
   port and the held world are where Doctrine sends armed hulls, not where the
   engine looks for fire.
6. ~~**Discharge events**~~ — **done**: `combat::resolve_pass` is deleted; each
   shooter discharges on the main loop at its Design's period (R-WAR29), and
   each discharge that lands is checked against the target's wreck point.
7. ~~**Course adjustment events**~~ — **done**: flight from a moving start
   (`Motion::brake`), belief events A and B, one decision per fleet per shooter
   (`FleetKey`, R-WAR32's interim), colonists that retarget (R-WAR30).
8. ~~**Pitched battles through the same events**~~ — **done**: R-WAR26's three
   endings, each on the event that raises it; `resolve_beam_engagement` is
   deleted, and a Technology bed must place fleets through the engine (R-WAR36).
9. ~~**Re-measure**~~ — **done** (appendix §D.16): on the twelve-seat table, 11 galaxies, the
   Warfare card's P92 is **1.309 [1.267, 1.370]** with fire simultaneous
   (§D.17; 1.334 [1.261, 1.410] before it, 1.204 [1.164, 1.226] at T-125), still below the 1.5–2.0x target (R-WAR20), and Growth's is 1.889
   [1.742, 2.669]; fire control is ~16.5–24.5% of instructions on the card bed,
   inside the 25% budget (R-WAR29); test targets all under 60 s; card-free runs
   bit-identical to the engine before. The port strike wrecks no colony ship
   under nearest-first targeting (R-WAR37).

---

### T-132. The damage model — beam power over time, structure on hull volume

**Closed (engine); opened R-WAR21 and R-WAR22.** Author's specification, quoted
in `Hyades_warfare_tree.md` §8.18: kilojoules; damage not per tick; a fight's
duration scaled so Design improvements and hull distinctions can act; realism
yields to fun; structure on **hull volume** (the author's choice between volume
and dry mass). Appendix §D.10.

- **The model:** a beam mount delivers power `P` while on target, `P · dt` per
  tick, so a fight's length does not depend on the step (tested). Structure is
  `σ · r³`. One target per mount per tick. The simulation's resolver no longer
  reads the arena's `laser_shots_per_tick`; the arena is unchanged
  (`tests/balance.rs` passes).
- **Placeholders (R-WAR19):** `P = 50 MW`, `σ = 10¹² kJ per hull unit³` — one
  mount wrecks a Limited Contact hull in 9.5 days. Chosen so equal-spend mirror
  fights last 28–82 ticks and every fight between the smallest hulls ends inside
  the 1,000-tick engagement.
- **Measured:** the short-range round robin decides every pairing on seed 1
  (it destroyed both fleets in all 49 before), resolving Technology R-TECH14;
  one GOU beats 40 ROUs and loses to 50; one ROU beats 10 LOUs and loses to 14.
- **What it cost the Warfare card:** a lone Limited picket cannot finish a
  Medium colony ship in one engagement (254 days against 183), so on the
  twelve-seat card bed hulls destroyed fell 77–94% and colonies rose 8–11%
  (seeds 1, 7, 42). §8.17.4's covering rule lost its premise. **R-WAR21** is
  the author's choice of remedy; **R-WAR20** needs re-measuring after it.
- **Opened R-WAR22:** damage does not persist past an engagement, which was
  invisible while every fight ended in one tick.
- **Test budget:** determinism 47.1 s old against 46.9 s new, one uncontended
  pair on this container (a second pair read 46.7 against 51.9 with the new run
  overlapping other builds); smoke 33.3 s, telemetry 23.0 s, balance 50.6 s.

---

### T-130. `exp` and `ln` on the run path: four-multiply minimax polynomials

**Closed.** Author's direction: *"Replace exp and ln with the best polynomial
approximation over the input range that can be achieved with a four multiply
budget."* Appendix §D.8; autopilot §3.4; netcode H4a.

- **Measured every run-path site's input range first**, then fitted by Remez
  and kept the best scheme per site: `exp_fast` (range-reduced `2^f`, 7.5e-5
  relative) for the freight scores and `rank`'s centrality; dedicated degree-4
  fits for the logistic step (5.2e-9) and the contract decay (5.3e-6);
  `log2_fast` (8.8e-5 absolute) for `settler_target`, with `ln 2` folded into
  its own factor; `pow_fast` takes `y = ½` and `y = 2` exactly (`sqrt`, `x·x`).
- `math::exp_decay` deleted (contradicts the old §3.4, now rewritten);
  T-127's tangent-bound prune deleted.
- **Cost:** per call 45–68% cheaper; per event −0.84% instructions, and wall
  time not resolved (0.986 ± 0.013 over 15 paired rounds). Runs move: colonies
  −0.25 ± 3.02 over 8 seeds. Native and wasm32 still identical.
- **Still accurate, not on the run path:** galaxy generation, `Rng::gaussian`,
  `Qty::at_band` at world construction, `ln_const`, and the cost ladder's
  `Empty` segment in `mass_at_same_band_from` (measured never reached).

---

### T-127. The engine calls no host libm — native and wasm32 runs had diverged

**Closed.** Author's request: remove the expensive math-library calls such as
`ln` from the simulation, and say why they are used. Appendix §D.6; netcode §6
H4a.

- **Why they are there:** a Band is a logarithm of a mass (`band`, `at_band`);
  the closed-form logistic needs `e^(−rΔ)` and its inverse (`logistic_step`,
  `settler_target`); decay with travel time (`e^(−λt)`); crowding and vein
  exponents (`pow`); station-keeping orbits in every fight (`sin`, `cos`); and
  galaxy generation and `Rng::gaussian` (exponential, Gamma, Box–Muller).
- **The finding that mattered more than cost:** the host libm and the wasm32
  build's libm disagree in the last bit on 1.9–9.8% of inputs, and native and
  wasm32 runs of one seed **diverged** — on all four arms tried at 800 yr
  (3 seats) and 300 yr (12 seats). `AGENTS.md` §4's "native and wasm32" claim
  was false, and the determinism suite's short horizons could not see it.
- **Landed:** `src/transcendental.rs` — `ln` (division-free, 256-entry table
  built at compile time), `ln_const` (fdlibm, `const fn`), `exp`, `pow`,
  `sin_cos`, all from `+ − × ÷` and bit operations. Every call site moved;
  `Scale::LN_STEPS` evaluates the ladder's logarithms at compile time.
  `clippy.toml` bans the host functions and `mul_add`; `src/lib.rs` denies the
  lint (it was denied before, with nothing configured). Eleven arms now
  reproduce bit-for-bit native and wasm32, two of them combat beds.
- **Cost:** `ns/event` +1.5% / +2.9% on the standard bed (400 yr, min of 7),
  after a bit-identical prune of `settler_target`'s scan that takes ~48% fewer
  logarithms. `Volume::cbrt` had no callers and is deleted.
- **Also:** `tests/determinism.rs` was 64 s on the old binary too; its full-run
  test is now one test per seat count (53.8 s). One smoke tolerance became
  relative.
- **Opened:** T-128, T-129 (closed since).

---

### T-129. Band readings off the run path — static kilotons, and a reading without a logarithm

**Closed.** Author's direction: *"Band readings are not required. Translate
statically into kt readings."* Mineral cost curve §2.6 (T-129 note); appendix
§D.7.

- **Static translations:** `K` is a stored mass (`Factors::k_mass`); the rung
  test compares squared midpoints (`Qty::nearest_whole_band_from`); `staffing`'s
  cost → mass map is a per-segment power law with the `3/2` tie
  (`Price::mass_at_same_band_from`); the seed floor is a rung's own mass
  (`units::population_mass_at_tier`). `at_band` now runs only where the world
  is built.
- **The reading** (`Qty::band`) has no logarithm: static per-segment constants
  and a degree-7 polynomial on the mantissa bits, within 3e-7 Band, exact at
  every rung. `rank`, the views and the snapshot read through it.
- **Unchanged on the author's instruction:** `veins` (54 k readings and 54 k
  `pow` calls per 400 yr on the standard bed — at run time, not in galaxy
  generation) and `i_star`.
- **Cost:** `ns/event` −4.7% / −8.6% against T-127 at 400 yr (min of 7),
  instructions −2.4% per event. Runs move: colonies −2.75 ± 2.05 over 8 seeds.
- `PlanetSnapshot::bio_max_mass` added; `units::population_at_mass` deleted (no
  callers, before or after).
- **Still transcendental at run time:** `settler_target`'s `ln` (the
  logistic's inverse, pruned by half at T-127), `logistic_step`'s and the
  freight scores' `exp`, and `veins`' `pow`. None is a Band reading.

---

### T-126. Release throughput on the combat bed — the knobs were already right; the profile was not

**Closed.** Author's request: experiment with compiler knobs and check in any
that raise yr/s on the standard bed with shots fired. `examples/combat_bench`
(12 seats, both cards at the barrier, 400 yr). Appendix §D.5.

- **Knobs: none checked in.** Thin/no LTO, opt-level 2, `target-cpu`
  x86-64-v3 and native, and PGO all landed inside the base binary's ±2%
  run-to-run spread, every one bit-identical. The shipped profile (fat LTO, one
  codegen unit, abort, O3) stays.
- **Profile: 2.49x, bit-identical.** Callgrind put ~41% of instructions in libm
  `log`, all from `fill_survey_candidates` reading `bio_max.in_bands()` per
  planet per survey decision while `Factors::bio_max_band` held the same value.
  Reading the cache: 14.6 → 30.3 yr/s. Then a per-seat unvisited list, pruned by
  stable `retain`: → 36.4 yr/s. Determinism target 56.5 → 48.2 s.
- **Open:** the scan still builds a `SurveyView` per unvisited world for a policy
  that keeps one; removing it changes `Autopilot::choose_survey_target`.

---

### T-125. Weapons are a Design; the author's 1.5–2.0x target; Growth in the band, Warfare below it

**Closed (engine), advanced (Warfare's magnitude).** Author's specifications,
each landed: loadout is a function of the Design fixed at construction; lasers
only, for the Warfare card only; armed colonizers go to the frontier as pickets
after founding; pickets stack like miners; staffing on by default; every card at
**1.5–2.0x its tree's metric at P92** on the twelve-seat bed. Warfare §8.17;
appendix §D.4.

**Landed:**

- `combat::Loadout`, `Armed`, `hull_hp_kj`, `resolve_beam_engagement` —
  simultaneous fire, damage in kJ against structure (dry mass × 1,000 kJ/kt),
  unarmed ships deal nothing. `design_loadout(hull, class)`: Systems hulls
  unarmed, Contact/Offensive mount beams slot-organically. Stamped once at
  construction (`World::loadout`). Every simulation fight uses it; the arena
  keeps its tuned resolver. `SimConfig::engagement_volley_period_years` deleted
  (only the sim's missile path read it).
- `ArmedFrontier` also writes `picket_after_founding` and `picket_first`;
  `stays_armed_after_founding` is the one predicate for credit and dispatch, and
  is true only for armed hulls.
- Stacks: held ground and blockades hold several hulls; cover every seen port
  before stacking; build ahead of expansion only to cover a port.
- Blockade supply and timing: `picket_first`, `recall_seen_launches`, recency
  ranking with `BlockadeReassess` (reusing `intercept_reassess_years`), in-flight
  hulls counted against the reserve. `ARMED_FRONTIER_BLOCKADERS` 8 → 128.
- `population_staffs_industry` on by default (R-IND23 resolved by the author).
- Growth card `GrowthRate(1.15)` → **(1.6)**, tuned on the twelve-seat bed.
- `examples/card_table` reports per-seat ratios with P92 and a bootstrap over
  galaxies; `examples/blockade_census`; `card_probe` reports `ΔlnS` and has
  coverage-oracle arms (`SimConfig::ablate_strike_fraction`, ablation only).
- Tests: `a_ship_fires_what_its_design_mounts_and_nothing_else`,
  `a_shot_weaker_than_the_hull_does_not_kill_it`,
  `only_the_warfare_card_arms_a_hull`, `pickets_stack_on_held_ground_like_a_crew`,
  `a_new_blockader_recalls_only_launches_whose_light_has_arrived`,
  `the_blockade_is_built_first_only_while_a_port_is_uncovered`.

**Measured, twelve seats, 11 galaxies:** Growth **P92 1.753 [1.583, 1.932]** —
inside the band. Warfare **P92 1.204 [1.164, 1.226]** — below it. The coverage
oracle says the mechanic reaches the band at ~30–45% of rival launches struck;
the census says the blockade reaches ~10% in the rivals' expansion peak because
its hulls are still in flight (**R-WAR20**, the author's choice).

**Opened:** R-WAR19 (beam and structure magnitudes), R-WAR20 (Warfare below
target, latency-bound). **Resolved:** R-WAR5 (for the simulation), R-WAR18
(implemented, null), R-IND23. **Open, not coded:** the Growth-seat collapse on
seed 31337 (appendix §D.4) — which seats took its worlds.

---

### T-124. A rung bill that destroyed mass — and the Growth card it was carrying

**Closed.** `infra_step_price` billed the width of the rung a stock *rounds* to
and the purchase set the stock to the next rung, so a stock founded above its
rung paid 0.0292 kt per purchase that was never erected (design law #11). Now it
bills what the stock is short of the next rung. Found by T-123's conservation
test on a run **with no card played**; localized by re-running to increasing
horizons (zero drift through 240 yr, equal quanta after) and then diffing the
ledger per event. `an_off_band_upgrade_erects_what_it_bills` pins both
directions and fails on the old bill. Appendix §D.3.

- **The default bed is inside noise:** colony-years +0.0021 ± 0.0028,
  work-years +0.0064 ± 0.0122 (8 seeds).
- **The first Growth card is not.** Its work-years effect was **+0.133 → −0.014**,
  and a one-sided ablation reproduces each end bit-identically: the overcharge
  on above-rung stocks carried it. **T-122's Growth result is retracted.** Why a
  3% overcharge produced a +0.13 advantage for a population card is not
  established.
- **What the card needs now is R-IND23:** population reaches work-years only
  with staffing on (T-107) **and** the per-color conjunction lifted — +0.121 ±
  0.015 (t 8.2) with both, +0.056 ± 0.062 with staffing alone. Both are
  ratified or default-off rules, so the author decides.

---

### T-123. The port strike — armed hulls meet colony ships where they launch (R-WAR16, resolved)

**Closed.** The author chose "armed hulls strike colony ships"; the census chose
the site. `examples/launch_census` found a seat's launches spread over a mean of
213.6 origins with 25.5% through its busiest eight, where a destination is one
of thousands — so the meeting site is the rival's **port**. Warfare §8.16;
appendix §D.2.

**Landed:**

- `Doctrine::picket_blockades` — a picket goes to the rival port its seat has
  **seen** launch the most (`EventKind::LaunchSeen`, `distance / c` after the
  drive lights), holds it, and strikes each colony ship launched there at range
  zero (`strike_at_port`). Counted against `picket_reserve`.
- `fight_at` — the one fight between hulls at a site, extracted from
  `resolve_picket_fight` and shared by the port strike; bit-identical.
- `DoctrineWrite::ArmedFrontier` writes the blockade, the claim-target supply and
  `picket_reserve ≥ ARMED_FRONTIER_BLOCKADERS = 8` (placeholder).
- Tests: `a_blockader_strikes_the_colony_ship_leaving_its_port` (the strike, its
  ledger, the light-lagged sighting, the placement rule) and
  `mass_is_conserved_through_the_blockade` (a whole run, non-vacuous — 36
  strikes by 300 yr).
- `examples/card_table` takes `--seeds`, prints one `ROW` per galaxy and reports
  the standard error over **galaxies**, which are the independent unit.

**Measured, card as shipped, asymmetric bed:** ΔW **+24,024 ± 8,509
colony-years (t 2.82, 7/8)**, rival colonies −102.6 ± 25.8 (t −3.98), 306 ± 84
colony ships struck, none lost. **Twelve-seat table, 11 galaxies: `ΔW_i` +28.26
± 5.37 colonies (t 5.26), `∫ΔW_i dt` +8,817 ± 1,713 (t 5.15), 11/11** — the
3-SE confirmation the author asked for. Growth on the same table: +0.014 ± 0.042
(t 0.33).

**Opened:** R-WAR17 (a strike forfeits the rival's target, because `targeted`
is monotone); R-WAR18 (placement has no recency — written from `t = 0` the same
writes score t 4.06–5.57).

---

### T-122. Where each first card's effect goes — and the largest constraint was the bed

Isolate what stands between each first card and its own tree's metric, then
remove it. Target: an effect at ≥ 3 standard errors on the tree's metric (the
"double the rate" reading is reported beside it). Measurements in appendix
§D.1; decisions in warfare §8.15, industry §1.8, trees §2.4.

**The bed.** `examples/card_probe`: asymmetric (seat 0 plays, others pass),
paired against the same seed passing, **8 independent seeds** — so the standard
error counts galaxies. On the twelve-seat table six seats shared a galaxy and
were counted as six replicates. Arms are ablations: an engine change on both
halves, or a Doctrine write on the card half.

**The first constraint was mine.** Both harnesses played cards at `t ≈ 0`,
inside the card-free opening (`years_to_first_round` = 200 yr, netcode §1),
charging the 0.5 kt price to the 3 kt bootstrap bank. That price was the
largest effect either card had — ~550 colonies on seed 31337, reproduced by a
pure-price control — and at the round-0 barrier it is invisible. **Both
harnesses now play at the barrier**, and T-120/T-121's tables are marked as
measured inside the opening.

~~**Growth — passes as shipped.** Played legally, `TIER0[3]` moves work-years
**+0.133 ± 0.032, t 4.18, 8/8 seeds**.~~ **Retracted at T-124:** that result
was carried by a rung bill that destroyed mass, and on the conserving engine
the card reads **−0.014 ± 0.023** (appendix §D.3). Growth is open again under
R-IND23.

**Warfare — no path to `W`.** At legal play `TIER0[15]` is inert (ΔW −103 ±
314). Four gates in series, each closed by ablation (warfare §8.15): arming
changes no fight; the initiator always loses (R-WAR5); rock fights cannot move
`W` even when won (§7.3); held-ground fights never happen — not because the
guess is wrong (an oracle arm changes nothing). **Removing them needs a new
mechanic, and which one is the author's call: R-WAR16.**

**Landed:**

- `SimConfig::population_staffs_industry` — **T-107, off by default**, with
  `P_req` anchored to the homeworld's generated Band-for-Band staffing (industry
  §1.8). Instruments `Simulation::staffing` and `staffing_split`.
- Two ablation knobs, each pinned by a test that it changes only what it names:
  `ablate_color_conjunction` and `ablate_oracle_intercept`.
- `Sighting` bundles what a picket observed, so the oracle could be threaded
  without an eighth argument.

**Refuted on the way, recorded in §D.1:** "staffing is necessary for the Growth
card" (confounded by the `t ≈ 0` price); "the idle stock is on worlds at their
ceiling" (0.0000 is); "the color conjunction throttles Growth" (it multiplies
the standard error by ~4 and leaves the mean alone); "the picket guesses the
wrong world" (the oracle does no better).

**Open:** ~~R-WAR16 (the mechanic)~~ resolved at T-123; R-IND22 (`u`'s shape); T-108 (the
profitability test — with staffing on, 70% of standing stock is idle at 600 yr).

---

### T-121. The default is unarmed, and the Warfare card is the key (R-WAR13, resolved)

Author's specification. Full write-up in `Hyades_warfare_tree.md` §8.14.

| role | before | after |
|---|---|---|
| Scout | LCV / `Tor` | **LSV / `Tor`** |
| Picket | LOU / `Unnamed` | **LCV / `Unnamed`** |
| Colonizer, both rungs | MSV, GSV | unchanged — already unarmed |
| seeded roster | LSV(Meadow) + LCV(Tor) | **LSV(Meadow)** |

Every Warfare `Doctrine` field already defaulted off, so the *Doctrine* half was
unarmed and the *Design* half was not — which is exactly why §8.13 measured the
card as a price. **R-O42 / standing-layer §7.1 had already ratified "100% LSV in
the Scout role" and the engine had never implemented it**, so the doctrine half
of this is a correction rather than a change.

#### The card is a bundle now

`Card::effect` → `Card::effects: &'static [CardEffect]`, applied in slice order,
which is what §8.2's *"one Design write, three Doctrine writes"* describes.
`TIER0[15]` carries `UnlockDesign(LCV, Tor)`, `UnlockDesign(GCV, Unnamed)` and
`WriteDoctrine(ArmedFrontier)`. The new write sets `scout_hull_offensive` and
`colonizer_general_contact` together — one decision, because splitting them
would let a player buy the cheap half of a slant (design law #9).

§8.2's third write, `picket_after_founding`, is **out**: §8.10 measured it at
−287.8 `W_0`, so it is R-WAR6's design question.

#### What it contradicts

**R-O42 / §7.1's roster half** — LSV + LCV becomes LSV alone. §7.1's own
argument (the opening fleet must be one indistinct object at range) survives and
is better served by one design than two. Recorded in both specs.

#### Three duplications removed

`scout_hull` was public and read in three places — the build branch, the
affordability test, `launch_survey` — the shape T-117 named and T-116 paid for.
It is private; `Standing::scout_order()` is *derived* from `design_for`;
`launch_survey` asks `design_for`; `price_of` maps the Limited tier explicitly
and says why `picket_cost` and `light_vehicle_cost` are equal today.

#### Re-measured, and three seeds cannot resolve either card

`examples/card_table`, same arms and fit as T-120, re-run against the armed card:

| quantity | T-120 | T-121 |
|---|---|---|
| Growth card value | +0.1134 ± 0.0592 (n=18) | **−0.0705 ± 0.1447** (n=18) |
| Warfare card value | −0.0371 ± 0.1609 (n=9) | **−0.4961 ± 0.2842** (n=8) |
| Warfare `ΔW_i`, colonies | −32.879 ± 46.075 | −29.364 ± 39.010 |

Estimates, not bounds; `n` counts seat-seeds over **3 independent seeds**.
Against 2 SE this bed resolves neither card — Warfare is 1.7 SE from zero, its
raw `ΔW_i` 0.75 SE, Growth 0.5 SE. The missing measurement is seed count.

**The columns are different beds** (the default Design moved), so they must not
be differenced; only the within-column comparison is paired. **R-WAR15** is
open on which of the card's three writes reaches the simulation, and it wants
an ablation per write rather than a story.

#### One build order broke, and no test could have caught it

Merging the picket and the armed scout onto one shell broke ordering a picket by
*naming its hull*: `hull_order(LCV)` stamps `Tor` and `role_of` reads that back
as a scout — T-116's defect, a yard paying for one thing and getting another.
`Standing::order_for(role)` replaces it, derived from `design_for`;
`scout_order` is one line of it.

The branch sits behind `picket_reserve > 0` or `picket_claims_target`, both
default off and neither written by `ArmedFrontier`, so the suite stayed green
across the change. Found by enumerating the call sites of the hull that moved.
`a_build_order_round_trips_to_the_role_that_asked_for_it` now pins both the
round trip and the defect.

#### `ASSIGNABLE`'s order is load-bearing now

Scout and Miner both mount `LimitedSystems`, so `role_of`'s second pass —
first role that *mounts* the hull — is what resolves a class neither has named.
Miner precedes Scout, matching `competent_role`, so the two cannot disagree.
Nothing production-built reaches that pass; the totality test does.

---

### T-120. A picket guesses, and can be feinted (R-WAR10, resolved)

§8.9.8 left `offer_interception` being *handed* the colony ship's destination,
which is not what a picket can see. Light delivers a departure point, a
departure time and a **bearing**; the destination is an inference. Full write-up
in `Hyades_warfare_tree.md` §8.12.

#### The guess

```text
t_see   = depart + |origin − station|
bearing = the direction the hull is actually moving
guess   = a world this picket has scanned, within `intercept_cone` of that
          bearing, reachable before the ship arrives
```

Candidates come from the picketing empire's **own `scanned` set** — a world it
never surveyed is not a world it can guess at (design law #15). It backs the
world **nearest the origin along the bearing**, because a colony ship is slow
and expensive and the near world on a line is the cheaper errand. A prior, not
knowledge.

#### Deception is a move now, not a wish

Two worlds on one bearing are indistinguishable at range, so a colony ship aimed
at the far one puts a picket on the near one for **free**.
`a_picket_backs_the_near_world_on_a_bearing_and_can_be_feinted` constructs that
and asserts the picket takes the bait.

`EventKind::PicketReassess` is the other half: every
`intercept_reassess_years` the picket re-reads the trajectory at the
**light-lagged** position — what it sees is where the quarry was when the light
left it, which is why a ship that has already turned still looks, for
`distance` years, like it is going where it was going. Re-aiming happens from
where the picket is, so the distance already flown is not refunded: a picket
that took the feint has paid for it.

#### Two magnitudes, both placeholders — R-WAR12

`intercept_cone_radians` (0.15 rad ≈ 8.6°) and `intercept_reassess_years`
(25 yr). Neither is physical. The cone is a **legibility** knob — wider makes a
decoy bearing likelier to catch the world the ship actually wants — and the
cadence sets how long a feint stays bought. They are the first magnitudes in
this tree whose job is to price a **bluff**, so they want a bed on which the
yomi channel is readable rather than `W_0`, which a bluff does not move
directly.

#### What it does not do

`BaselineAutopilot` aims at the world it wants; nothing in the engine chooses a
deceptive bearing. The channel exists and no policy uses it — which is the
right order (mechanism before policy) and is stated so the absence is not read
as a measurement.

#### Part 2 — the head-to-head, and the card has nothing to tune

`examples/card_table` is §8.4's bed: 12 seats, three arms (`Pass`,
`GrowthOnly`, `Both`), cards at earliest legal play, `g` fitted by least
squares on `ln X(t)` over `[150, 600]` yr as trees §2.4 requires. Each card is
read against the arm differing from it by that card alone. Full write-up in
`Hyades_warfare_tree.md` §8.13.

| quantity | value | n | mean `R²` |
|---|---|---|---|
| Growth card value, `1 − t½ ratio` | **+0.1134 ± 0.0592** | 18 | 0.911 |
| Warfare card value, `1 − t½ ratio` | **−0.0371 ± 0.1609** | 9 | 0.791 |
| Warfare `ΔW_i` at 600 yr, colonies | −32.879 ± 46.075 | 18 | — |

Estimates, not bounds. `n` counts seat-seeds and six seats share one galaxy, so
independent replicates number **3**; and `W_i` has no logarithm where it is
negative, so the Warfare row is computed on a subset selected by the quantity
being measured.

**The Warfare card reaches no decision, proved in the code rather than
inferred from the numbers.** `TIER0[15]` writes
`UnlockDesign(LimitedOffensive, Unnamed)`; the Roster's only consumer outside
tests is `roster_permits`, which returns `true` before the lookup while
`enforce_roster` is off; and `role_hull_type(Role::Picket)` already hands that
hull to everyone. The card's whole channel is its 0.5 kt price.

So the answer to *"tune it to match Growth"* is that there is no magnitude to
move — the card needs an **effect** first, which is a design decision. Two new
open codes came out of it: **R-WAR13** (the tree has no playable card doing what
§8 specifies — `DoctrineWrite` has no Warfare variant) and **R-WAR14**
(`inert_card_plays` counts `NotYetImplemented` only, so a write into a
component with no live consumer reads as working).

Two traps the bed exposed, both pinned as tests:
`a_tier_zero_card_is_affordable_at_round_zero` (an unaffordable order coerces
to a pass **silently**, so a bed cannot tell "did nothing" from "never
played"), and `the_first_warfare_card_is_a_price_and_not_yet_an_effect`.

---

### T-119. A hull is made of something, and the founder pays for the floor

**Both ratified by the author, both closing T-118's open exceptions.** Design
law #11 now holds **with no exceptions**, card played or not.

#### R-IND22 — the founding center pays the top-up

The absorbing-zero floor (§8.7) used to raise a thin colony to `Band Empty` out
of nothing, which was the last place the engine created mass. It is a
**transfer** now: the parent center that dispatched the colonizer is billed for
the shortfall out of its own bank, exactly as it is billed for the settlers and
the endowment (R-O74).

A parent too poor to pay leaves its child at whatever it could afford — **the
guard degrades rather than conjuring**, so a bankrupt empire cannot found its
way to free infrastructure. `mass_is_conserved_under_the_warfare_card` asserts
an *equality* now where it carried a `colonies × floor` allowance.

#### R-IND21 — a hull carries its own composition

Cost is one scalar (R-O57), so every path that returned a hull's mass to the
world guessed an even color split — in the one dimension the mineral economy is
actually constrained by (§6.19c).

`World::hull_minerals` records what the bank handed over.
`Minerals::try_take_total` is the capture point: it is `try_spend_total` that
**returns the withdrawal** rather than discarding it, because the bank's mix
moves the instant it returns and a reconstruction afterwards reads the wrong
proportions. `composition_of(vehicle, want)` scales the record to any amount, so
one store answers "what does half of this hull recover" and "what does all of it
slag".

**Better material makes better loot, with no rule beyond the composition.** A
hull bought with supers or apex returns supers or apex, because that is what it
is made of. Nothing spends supers on a hull yet, so
`a_hull_returns_the_minerals_it_was_built_from` asserts it directly — an
emergent test would sit at zero.

Three consumers read it: scrap salvage, the founding ceiling's overflow, and
(through the same helper) anything that wrecks a hull. A hull nobody paid for —
the seed scouts the galaxy is generated with — falls back to an even split, and
`BuiltHull::unpaid` names that case rather than letting it be the default.

#### What it cost

Four CRN seeds, 3 seats, 800 yr, against T-118: colonies 2,949 / 2,851 / 2,898 /
2,954 → 2,949 / 2,848 / 2,890 / 2,961, a mean of **−0.25** with 2/4 seeds down,
and population identical to five significant figures. **Flat**, and it should
be: the default bed has no picket doctrine so the floor almost never binds, and
every hull is bought with basics either way — the composition record differs
from an even split only by however lopsided the paying bank happened to be.

The changes are for correctness and for what they unblock, not for the
objective. What they unblock is supers and apex meaning something when a hull is
built from them.

#### A dispatcher takes a hull whole

`BuiltHull { hull, mix }` replaces passing the two separately. They are one
fact: a composition is meaningless without the type it scales to, and a type
without a composition is the guess this replaces. It also kept `spawn_courier`
and `launch_survey` inside clippy's argument limit without an `allow`.

---

### T-118. Mass conservation is enforced, and it was leaking in three places

**Design law #11 is the engine's most load-bearing invariant and it was asserted
in prose.** `Simulation::mass_ledger` weighs every store — in-ground, banked, at
outposts, in cargo, infrastructure, hulls, population, settlers, biomass, slag —
and `mass_is_conserved_with_regrowth_off` runs with the one legitimate source
(biosphere regrowth) switched off and asserts the total does not move.

It found three leaks on the first run, on a 200-year 3-seat bed. The ledger is
broken out per store rather than totalled because "mass changed" says nothing
about which transfer lost an end — and each of these was exactly that.

#### 1. Half of every scrapped hull vanished

`sys_scrap_arrive` credited `role_cost(Role::Scout) * scrap_recovery_fraction`
to a bank and **dropped the rest**. Two defects in one line: the recovered
amount was priced off the *Scout* hull whatever actually scrapped, and
`scrap_recovery_fraction = 0.5` meant half of every hull left the ledger. Design
law #11 is explicit that *wastage degrades to slag rather than vanishing*; the
unrecovered half is slag at the site now, and a hull broken up with nowhere to
deliver salvage becomes slag entirely rather than disappearing.

#### 2. A recycled colonizer was counted twice

The hull that founds a colony *becomes* its infrastructure, and the comment at
that line already said "recycled hull, now inert infrastructure" — but the
entity was only parked, keeping its role and hull type. So its dry mass stood as
a live hull **and** as the `founding_infra` credited from it. It is
`Role::Scrapped` now, the engine's existing retirement marker, which retires an
object that was already inert.

#### 3. Founding overwrote the infrastructure already standing

`f.infra = f.infra.max(founded_at)` — so a wild world's existing infrastructure
was **thrown away** whenever the recycled hull was worth more. Localized in time
by running the same seed to increasing horizons: at 38→40 yr, hulls fell
**0.10921** (one Medium hull retiring) while infrastructure rose only
**0.0892**, and the 0.0200 already on the world was gone. It is `+=` now — the
same `max`-where-a-sum-belongs defect T-117 fixed on the other credit of the
same line, which is twice in two landings and is why the ledger exists.

#### What it cost

Four CRN seeds, 3 seats, 800 yr: colonies **+0.37%** (2,946→2,949, 2,849→2,851,
2,861→2,898, 2,955→2,954) and population **+1.1%**, 3/4 and 2/4 seeds positive
— inside noise at four seeds. `ns/event` flat. Colonies start marginally richer
because founding no longer discards what was already there.

#### The one remaining exception, filed rather than hidden

**R-IND22**: the floor rung *creates* mass. A colony founded below `Band Empty`
is raised to it because zero infrastructure is an absorbing state (§8.7), so
mass appears from nowhere — bounded by one floor rung per colony, and binding
only under the picket doctrine. `mass_is_conserved_under_the_warfare_card`
**asserts that bound** rather than waiving it: that bed may gain mass, never
lose it, and never more than `colonies × floor`. The conservative alternative is
for the founding center to pay the top-up out of its own bank, making it a
transfer.

**R-IND21**: a hull has no mineral composition. Cost is one scalar (R-O57), so
when a hull's minerals return to a bank — scrap salvage, the founding ceiling's
overflow — the engine splits them evenly across the three colors because nothing
records what it was built from. That is a guess in the one dimension the mineral
economy is actually constrained by (§6.19c).

#### Two tests changed meaning, and both were right to

`combat_off_is_bit_identical` asserted "no slag anywhere" as a proxy for "no
combat". Slag is now what *any* mass degrades into, so a peaceful galaxy
accumulates it from ordinary retirement; the assertion that still means
something is that no engagement resolved.
`slag_conserves_the_mass_of_what_it_destroyed` asserted standing slag **equals**
what combat logged; it is `>=` now, with the exact reconciliation kept on the
combat portion.

---

### T-117. The standing layer answers; it is not switched on

**Architecture, verified behavior-neutral.** T-113, T-115 and T-116 each added a
Doctrine write and each added the *same three branches* to carry it — the build
order, the price the production context reads, and `assign_role`'s match on hull
type. Full write-up in `Hyades_warfare_tree.md` §8.11.

`autopilot::Standing` is a borrowed reading of Design and Doctrine that answers
instead of exposing flags: `design_for(role)`, `colonizer_ladder()`,
`mounts(role, hull)`, `role_of(hull, class)`, `recycles_on_founding()`.
`ProductionContext::price_of(hull)` replaces picking the right price field by
re-deriving the write.

**`role_of` is derived from `design_for`, not written out** — an exact design
match, then the hull alone for a class the layer has not named, then a
competence table (R-O44) for a hull no write has claimed. So a card that mounts
the colony role on a Contact hull changes one function.
`role_of_inverts_design_for_every_role` pins it across every combination of the
writes; they compose, and a resolver correct one write at a time is not one.

#### What it caught

- **`assign_role` was not total** — the two large Offensive hulls hit
  `_ => None`, so a hull the yard had been charged for could come back with no
  mission. That shape has already cost real work twice (`launch_survey` at
  T-116, `apply_build_with` at R-O86).
  `every_hull_has_a_role_under_every_doctrine` requires totality now.
- **A `max` standing where a sum belonged** — a colony credited its founding
  stock from the recycled hull and *then* from the hold with `max`, so one of
  the two vanished. It is `+=` now; the shipped `founding_infra_share = 0.0`
  is why nothing moved.

#### The hold erects in ratio — R-IND20

A rung is billed per color (`works_bill` splits by `works.mix_share(c)`), so
minerals in a colony ship's hold are worth what their **scarcest** color allows:

```text
erectable = eta_works · min over c of ( aboard[c] / mix_share(c) )
```

`sim::erectable_from`. A hold that is all Cyan erects **nothing** — the same
conjunction `Hyades_industry.md` §6.19c measured as the mineral economy's
binding constraint, arriving at founding rather than at a production decision.
What the ratio cannot use is banked, so `consumed + remainder == aboard`. The
scalar-total version it replaces let a single-color hold stand up as
infrastructure **no center could have bought with the same minerals**.

#### Cost

**Bit-identical on four CRN seeds** against the pre-refactor binary — 2,946 /
2,849 / 2,861 / 2,955 colonies and 4,788,955 / 9,786,675 / 8,831,325 /
9,456,757 kt. Two configurations resolve differently than the switch did (a
`Tor`-classed LOU with the scout write off; the two large Offensive hulls) and
nothing builds either pairing today, which is why the bed does not move.

---

### T-116. Why `W_0` is negative — one cost, measured by ablation

**The question T-112 through T-115 kept not answering.** Six arms agreed on the
sign; none said why. `examples/warfare_why` (new) decomposes it. Full write-up
in `Hyades_warfare_tree.md` §8.10.

#### The table (4 CRN seeds, 3 seats, 800 yr, seat 0 alone playing)

| arm | own | neighbors | `W_0` | MSV | Gen | founded | transfer |
|---|---|---|---|---|---|---|---|
| peace | — | — | — | 6,889 | **0** | 3,562 | — |
| Design write alone | +0.0 | +0.0 | **+0.0** | 6,889 | **0** | 3,562 | — |
| `SettlersPerMineral` alone | +287.8 | −107.1 | **+394.9** | 8,836 | 469 | 4,713 | — |
| …+ the Design write | +220.0 | −93.1 | **+313.1** | 7,415 | 639 | 4,442 | — |
| **Doctrine write alone** | −197.2 | +90.5 | **−287.8** | 5,665 | 0 | 2,773 | **0.46** |
| …founding rung restored *(ablation)* | −1.0 | +1.0 | **−2.0** | 6,492 | 0 | 3,558 | 1.00 |
| the card as §8.2 specifies it | −197.2 | +90.5 | −287.8 | 5,665 | 0 | 2,773 | 0.46 |
| **both halves, both live** | −337.5 | +153.6 | **−491.1** | 4,162 | 637 | 2,212 | 0.46 |

#### The answer

**One cost.** Restoring the founding rung takes the Doctrine write from −287.8
to **−2.0** — so pickets, engagements, diversions and kills are together worth
about two colonies. The chain:

```text
colonizer keeps its hull
  → colony starts at the floor rung, 0.020 kt not 0.109        (5.5x)
  → it produces less, forever
  → the empire builds 18% fewer colony ships   (6,889 → 5,665)
  → founds 22% fewer colonies                  (3,562 → 2,773)
  → neighbors pick up 46% of them              (transfer 0.46)
  → W_0 = −(1 + 0.46) x the loss
```

**And it corrects §8.6's arithmetic in the direction that matters.** That bound
(`ΔW_0 = −k(1 − w_0A)`) assumed the card *denies*. It does not: neighbors
**gain** 0.46 per colony it costs its own player, so the realized form is
`−k(1 + t)` and there is no denial term at all. The card is not a bad trade, it
is not a trade.

**The two halves are multiplicatively bad**: +313.1 and −287.8 compose to −491.1,
an interaction of −516. `founding_infra(hull)` **is** `hull_cost(hull)` (T-70,
R-O57), so the Doctrine write's price is the mass of whatever hull the colony
ship is — and the Design write mounts it on one costing **10.07x** a Medium.
**The card's price is proportional to the thing its other half makes bigger.**

**R-WAR11 (restated at T-118 — the first version was wrong).** It said the
engine cannot charge a card a price. It can: `apply_orders` checks
`empire_can_afford` and calls `empire_spend(p, c.cost)`. The gap is that **a card
has two costs and only one is priceable** — `Card::cost`, bounded, and the
mechanic's own, which here is every colony founded afterwards starting 5.5x
thinner forever. The second cannot be priced because its size depends on how
much the player goes on to expand, and a difference objective charges it twice.
Not Warfare-specific; it belongs in the card contract.

Note also that §8.10's arms set Doctrine fields directly rather than playing the
card through `apply_orders`, so `Card::cost` was never charged: the measured
`W_0` is the effect **without** its price.

#### Two findings that are not about this card

- **The Design write is unreachable at the shipped colonizer policy.**
  `CheapestViable` keys on negated price, a Medium hull costs 0.1092 kt against
  any General hull's ~1.1–1.3, and `settler_target`'s zero cases do not depend
  on the hold — so the two options are admitted together and the cheap one
  always wins. **Zero General colonizers across four seeds.** Pinned by
  `the_cheapest_viable_policy_never_names_a_general_colonizer`. And the
  **Contact ladder has no Medium rung**, so a card mounting the colony role on
  a Contact hull is forced to the tier nothing selects.
- **R-IND11 is unblocked and may have flipped.** `SettlersPerMineral` is
  **+394.9 `W_0`** unilaterally — more than any card measured. `Hyades_industry.md`
  §1.6 blocks it on R-O74 (conjured settlers); **R-O74 closed at §1.7.** The
  measurement here is *competitive* (seat 0 alone changes policy), and a default
  is a global question, so the missing measurement is a symmetric re-run.

#### Also landed

`GeneralContactVehicle` is plumbed as a colonizer option
(`Doctrine::colonizer_general_contact`), substituting **within** the option set
so the Medium hull stays. Ladder read out by `examples/hull_compare`: cost
1.0995 vs the GSV's 1.3154 kt, seed hold **12.04 vs 31.62**, settlers per
kilotonne **10.95 vs 24.04** — cheaper and much worse at carrying people, which
is §8.2's stated trade. Its 10.95 still beats a Medium's 9.16, so it is a middle
rung rather than a dominated one.

**And a plumbing defect fixed on the way**: `launch_survey` spawned a fresh
entity with a hardcoded `role_hull_type(Role::Scout)`, discarding the hull
`apply_build_with` had just charged for. Invisible only because every Limited
hull shares a price and a mass (§8.9.7) — it is the third independent reason
`scout_hull_offensive` measured inert. `LogEvent::VehicleSpawned` now carries
the hull, because `role` and `hull` are not recoverable from each other in
either direction.

---

### T-115. Two responders, a moving picket, an armed scout — and a mechanic priced before it was believed

**Built, gated off, measured.** Full write-up in `Hyades_warfare_tree.md` §8.9.
Four corrections, three of them to things §8.6–8.8 got *wrong* rather than left
untuned, plus one contradiction found and deliberately not fixed.

The author's specification: *"Pickets should move to the frontier if the site
they patrol is colonized. LOU can replace LSV as scout in this Doctrine… I think
the enemy colony ships should be reacting in addition to enemy home centers.
Also, pickets should actively close the distance to targets they can intercept
prior to founding a colony."*

#### The bed, re-measured (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

Every row is measured *after* R-WAR9, so this table and T-112/T-113's are from
different engines. `own`/`neighbors` are % against the peace arm on the same
seed; `W_0` is in colonies. Neighbors should **fall** and `W_0` should **rise**.

| arm | own | neighbors | `W_0` | diverts | pickets |
|---|---|---|---|---|---|
| T-112 colonizer pickets | −21.46 ± 4.41 | +9.25 ± 2.78 | −281 ± 72 | 15 | 4,974 |
| + hold erects infra | −21.25 ± 4.48 | +9.16 ± 2.74 | −276 ± 70 | 14 | 5,012 |
| + cheap LOU pickets | −21.02 ± 4.55 | +8.86 ± 2.52 | −267 ± 63 | 21 | 5,079 |
| LOU pickets alone | −2.87 ± 2.11 | +1.06 ± 0.58 | −29 ± 19 | 1 | 188 |
| + claims its target | −2.64 ± 2.38 | +1.76 ± 0.80 | −49 ± 25 | 2 | 333 |
| **+ the LOU scouts** | **−2.87 ± 2.11** | **+1.06 ± 0.58** | **−29 ± 19** | **1** | **188** |
| + and intercepts | −2.29 ± 1.99 | +0.85 ± 0.62 | −22 ± 20 | 0 | 195 |

The direction is unchanged and the magnitudes roughly doubled: a colonizer that
keeps its hull costs its own player **21.5%** of its colonies where it cost 9.6,
because a slower expansion loop makes a spent colonizer dearer. The last three
arms are inside 2 SE on every column, and the `LOU scouts` row reproducing the
one above it to every digit is its own finding below.

#### The colony ship hears it itself — and that broke the predicate

§8.6 scheduled one warning, by distance to the ship's home center, on card
contract §2. The rule is right and the reading was too narrow: the home center
issues an *order*, the crew *notices*. The ship-side instant is the root of
`f(t) = (t − now) − |p(t) − world|`, found by 48 bisections because the
trajectory has no elementary inverse and bisection uses only IEEE-exact
operations (design law #16).

**Adding it made the warning arrive every time**, because light outruns a
sub-light hull by construction — so *"did the warning arrive"* stopped having
two outcomes, and left alone it would have turned every ship back and deleted
the arrival fight. The gate that replaces it costs no constant: a leg is a 1 g
brachistochrone, so **before turnover the hull's velocity still points somewhere
it can choose and after turnover it arrives whatever it decides.** The
counterplay window is now a *margin* rather than a race.

#### Interception is a forward-deployment mechanic, and it fires on about half the field

`Doctrine::picket_intercepts`: a picket leaves station when
`depart + |origin − station| + flight(station → world) < colony_arrival`. Every
term already exists — no reaction time, no interception radius. Cost is
`O(pickets held)` per launch, one `is_empty()` where nobody plays the card.

**Priced before it was believed** (`examples/intercept_probe`, new), with the
colony ship flown at the rate its load implies (R-WAR9 below):

| range | light | laden colony ship | **window** |
|---|---|---|---|
| 5 ly | 5.000 yr | 10.395 yr | 5.395 |
| 25 ly | 25.000 | 32.253 | 7.253 |
| 200 ly | 200.000 | 208.139 | **8.139** |

A laden Medium colonizer makes **0.241 ly/yr²** against an empty hull's 2.446,
so its overhead over light **saturates at ≈8.14 yr** — and that overhead is the
whole window an interceptor has. An empty LOU picket (0.940 ly/yr²) covers
**6.29 ly** in it, against a median nearest-neighbor spacing of **6.16 ly**. So
roughly **half the field is interceptable from a neighboring station**, with a
margin at the median of 0.13 years.

**And the sign of one term is the design.** A station 0.5 ly *beyond* the
contested world loses the race; the same 0.5 ly on the *near* side wins it,
because the far side pays the distance twice — once in light to hear the launch,
once in flight.

**And it fires** — `LogEvent::PicketIntercept` counts **81 interceptions across
eight seeds** (≈10 a run) against 195 picket arrivals, so roughly 40% of the
pickets that exist end up breaking for a contested world. The event exists
because nothing else can see the mechanic: an interception *moves* a hull rather
than building one, so the picket column barely registers it.

**Zero diversions, and that follows from the turnover gate rather than being a
second failure.** A picket that arrives just ahead of a colony ship arrives when
that ship is long past turnover, so the warning lands on a hull that cannot
turn — interception converts an uncontested founding into a **fight at the
destination**. `W_0` goes −29 ± 19 → −22 ± 20, which is a 7-colony paired
improvement with error bars three times its size: **not resolved at eight
seeds**, and the arm is still supply-starved at 24 pickets a run.

**The first version of this section said 1.92 yr and "fewer than a tenth".**
That was the *empty* hull's overhead, measured with the R-WAR9 defect below —
which had already been found and written up one subsection later, and used
anyway. **A probe inherits every assumption of the code it measures.**

#### Two supply writes, and they pull in opposite directions

§8.8 measured the cheap picket's problem as supply (95 hulls across eight
seeds), because its branch sits behind a survey test R-O86 measured a constant
`true`. `picket_claims_target` (T-113) found the picket a *new* branch;
**`scout_hull_offensive` puts it in the branch that already runs constantly** —
the armed hull takes the survey slot. Cadence untouched, hull changed, price
paid at the yard.

A scout built this way carries **`Class::Tor`**, which is how `assign_role`
tells a scouting LOU from a picketing one. The class *is* the design
(R-O28/R-O42b), so it needs no second piece of state.

**Note on the specification**: the author wrote *"LOU can replace LSV as
scout"*. The engine's scout rides the **LCV** (`role_hull_type(Role::Scout)`,
`Class::Tor`); the LSV is the mining hull. The write replaces the LCV.

#### A picket whose world is settled goes back to the frontier

Its product is a colony somebody does not found; once the world it stands on
*is* a colony there is nothing left to deny, and a parked hull the empire keeps
counting throttles `picket_reserve` against its own replacements.
`vacate_picket` moves the map and the count together, because every path that
removes a picket has to move both. This is precisely the situation T-113's
held-ground preference *creates*.

#### The contradiction, found by needing a number — **R-WAR9, fixed**

Implementing interception meant needing the colony ship's speed, which is how
`spawn_courier` turned out to compute the leg's acceleration as
`civilian_accel_g · G` **before** the hold is loaded, never re-reading it —
every `laden_accel` call site in the engine was freight. `AGENTS.md` §7 records
standing-layer item 5 (R-O32) as **done** (*"a laden colony ship flew like an
empty hull"*); it had closed it for the arena and not for this dispatcher.

The leg now reads `laden_accel` after loading, which also stops it pretending
every hull mounts the same drive (T-96). `launch_picket` and the interception's
own feasibility test read the same function, so the decision and the leg cannot
disagree about who wins a race. Guarded by
`the_colonization_leg_is_flown_at_the_rate_its_own_load_implies`.

**It invalidates every transit-dependent magnitude measured before it**, this
entry's own §8.6–8.8 arms included: a colony ship is 20% slower over 25 ly and
56% slower over 5, so the expansion loop is slower everywhere. The §8.9 census
is measured after the fix; the earlier ones are not, and comparing across them
is comparing two engines.

#### The armed scout is bit-identically inert, and the cost ladder is why

`scout_hull_offensive` reproduces its baseline **to every printed digit** — 188
pickets, −2.87% own, +1.06% neighbors, all eight seeds, and it did so both
before and after the R-WAR9 fix. Cause:
**`hull_dry_mass(LCV) == hull_dry_mass(LOU) == 0.020 kt`**, because under R-O57
cost *is* dry mass and `hull_dry_mass` reads the cost *tier*, which groups every
Limited hull. The two are the same object economically, so swapping them changes
nothing the simulation reads.

Not a wiring defect, and the write is kept: it goes live when hull types carry
differentiated cost (**R-O64**, **R-L0**). It does weaken §8.6's framing —
*"a picket must cost less than a colony"* holds (0.020 against 0.109 kt), but
the cheap *armed* hull is not cheaper than the cheap unarmed one, so arming the
frontier is free and the trade §8.6 describes is not the one being made.

#### What is still not modeled — **R-WAR10**

`offer_interception` is handed the colony ship's **destination**. Light delivers
a *trajectory* — departure point, time, heading — and a destination is an
inference from it; inferring it wrong is what makes a feint possible, which is
yomi content rather than a gap. That is the whole of R-WAR10.

**The obvious larger idea was refuted by one probe, which is why it is recorded
here rather than built.** *"Meet the ship on its path instead of racing it to a
world"* is a *narrower* set, not a wider one, because the window is
**back-loaded**: the slack a picket lives on is `t_ship(along) − along`, and it
accumulates as the hull decelerates through the back half of its
brachistochrone. On a 25 ly voyage the tolerable perpendicular offset runs
**1.24 ly at 12.5% of the track → 1.95 at half → 4.96 at the destination**. The
destination is where the entire window is, and the shipped criterion already
takes it.

#### What ships

`scout_hull_offensive` and `picket_intercepts` both default false (**R-WAR8**).
The ship-side warning, the turnover gate and the picket's return to the frontier
are unconditional but reachable only where a picket exists, so
`picketing_off_is_bit_identical` still pins the default galaxy.

---

### T-114. `tests/determinism.rs` is over the 60-second budget, and it arrived that way at T-112

**Measured, not suspected.** `cargo test --test determinism` is **69.08 s at
`HEAD` (T-112)** and **67.36 s** with T-113 applied, on the same machine in the
same session. The budget is 60 s per target (`AGENTS.md` §2), so the target is
over it and **T-113 did not put it there** — the ~1.7 s difference is inside
this machine's run-to-run variance and is not read as a saving.

T-112 raised entity count (pickets that never scrap, engagements, the round
layer) without checking test horizons in the same landing, which is exactly the
habit `AGENTS.md` §2 records: *"when a change raises entity count, check the
test horizons in the same commit."*

**What it needs, in the order §2 prescribes.** Not a uniform horizon cut —
`full_run_reports_are_bit_identical` already carries **per-seat-count** horizons
because cost goes as seats × years, and the arm that pays for the target is not
the arm a uniform cut would trim. Time each test by name first (the shell loop
in §2, since `--report-time` is nightly-only), then reduce the **galaxy** before
the horizon where the assertion is about a mechanism rather than about scale.
`full_run_reports_are_bit_identical` is the one test here whose question *is*
scale (R-NET14: a divergence at any seat count is a desync), so its galaxies
stay full-size.

**And any horizon that is cut needs a guard that the mechanism still fired** —
a shortened run leaves every assertion vacuously true and the suite green, which
is why `positions_never_exceed_lightspeed` counts moving entities and
`full_run_reports_are_bit_identical` floors its event count at 1,000.

---

### T-113. Claiming instead of denying — the cheap picket, the erected hold, and settling held ground

> **Measured before R-WAR9 (T-115)**, like T-112 above: the colonization leg was
> flown unladen. The mechanisms below hold; the magnitudes are from an engine
> that no longer exists.


**Built, gated off, measured on the asymmetric bed.** Full write-up in
`Hyades_warfare_tree.md` §8.8. Three Doctrine writes and one unconditional
preference; **one of the four moved the objective**, and it needed an engine fix
before it could move anything at all.

The author's specification: *"Add LOU to the Doctrine and Design as a cheap
picket to claim the frontier to support the Contact Vehicles. Load colony ships
with a mix of minerals for Infra and population so they do not suffer from the
no recycling policy,"* and *"Warfare tree should colonize worlds with a
picket."*

#### What it measures (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

| arm | own | neighbors | `W_0` | pickets |
|---|---|---|---|---|
| T-112 + settle held ground | −8.30 ± 2.69 | +4.07 ± 1.09 | −120 ± 34 | 5,415 |
| + hold erects infra (`share = 0.5`) | −8.33 ± 2.56 | +4.04 ± 1.04 | −119 ± 32 | 5,411 |
| + cheap LOU pickets (`reserve = 128`) | −9.01 ± 2.62 | +4.42 ± 1.12 | −129 ± 34 | 5,402 |
| LOU pickets alone | −1.57 ± 1.21 | +0.93 ± 0.38 | −21 ± 12 | 95 |
| LOU pickets + claims its target | −1.62 ± 4.47 | +1.10 ± 1.83 | −35 ± 61 | 188 |

*(Pre-R-WAR9. The same arms re-measured are in the T-115 entry above.)*

Neighbors should **fall** and `W_0` should **rise**. Neither does. The last two
arms are inside 2 SE on every column.

#### The one that moved, and the defect behind it

Settling held ground is worth **+1.25 points of own colonies and +13 on `W_0`**
(arm 1 was −9.55 / +4.40 / −133 at T-112). It recovers roughly a seventh of the
card's self-harm and does not change its sign.

**It measured as exactly zero first.** `sim::commit_one_build` reduces the
scanned pool to the per-class argmax before the policy sees it (R-O70) — exact
for a consumer reading the argmax of a **class**. This preference reads the
argmax of a **subset**, and `max(S)` does not carry `max(S′)` for `S′ ⊂ S`, so a
held world reached the policy only when it already won its class outright. The
arm reproduced the arm without it *to every printed digit*, which is the tell:
**a behavioral change that reproduces the baseline exactly is inert or
unreachable, not small.** Six slots now — per-class winner, plus per-class
winner among held ground — pinned by
`the_candidate_reduction_carries_held_ground_without_duplicating_it` and
`a_colonizer_prefers_held_ground_to_a_better_unheld_world`. Recorded in
`AGENTS.md` §"A decision can be provably blind".

#### The cheap picket is not built, and the reason is one predicate

§8.6 said a picket must cost less than a colony. It now does
(`HullType::LimitedOffensive`), and the hull is fielded **95 times across eight
seeds**. The branch sits in the production fallback behind
`wants_survey = survey_frontier > 0 && candidate_count < survey_reserve`, and
R-O86 measured the second term a **constant `true`** (median `candidate_count`
0, max 164, against a reserve of 1024) — so it is reachable only once the survey
frontier is exhausted. It is behind survey on purpose: ahead of it, the first
version cost its own player **24.5%** of its colonies.

`picket_claims_target` opens the other state where a cheap hull is the best use
of a cycle — the center has named an outward world and cannot pay for it —
gated on the target not already being held **or claimed**
(`Simulation::picket_inbound`, so one hull per world rather than one per
decision for a whole voyage). Supply **95 → 188**, objective unmoved. The state
is rare: expansion here is not bound by the colonizer's price.

#### Erecting the hold is flat because the hold is empty

`examples/founding_stock` (new) prints the distribution of the founding stock
against the floor rung (`Band Empty` = 0.020 kt), seed 1, seat 0's own
foundings:

| `founding_infra_share` | foundings | median | mean | max | above floor |
|---|---|---|---|---|---|
| 0.00 | 650 | 0.020 | 0.020 | **0.020** | **0.0%** |
| 0.50 | 649 | 0.020 | 0.051 | 0.313 | 21.0% |
| 1.00 | 649 | 0.020 | 0.087 | 0.625 | 22.5% |

At share 0 the *maximum* is the floor. At 1.00 — the whole hold erected — the
median is still the floor and coverage gains 1.5 points over 0.50, so the axis
is **exhausted, not undertuned**. `settler_target` sizes people to the
destination's carrying capacity (R-IND12) and minerals take what volume is left,
which is usually none. **R-WAR7**: loading a mix is a *reservation against the
hold*, which is a change to `settler_target`.

#### What ships

All three writes default off (`picket_reserve = 0`,
`picket_claims_target = false`, `founding_infra_share = 0.0`);
`picketing_off_is_bit_identical` still pins the default galaxy. The held-ground
preference is unconditional but self-gating — `held_by_me` is false everywhere
no picket exists — and the six-slot reduction costs one `is_empty()` on a bed
with no pickets.

TIER0 card 15 (Warfare / Inscrutable) is now `UnlockDesign(LimitedOffensive,
Class::Unnamed)` rather than `NotYetImplemented`; two inert cards remain.

#### What is still open

- **R-WAR7** — a `settler_target` that reserves mineral volume.
- **R-WAR6** — amended: the cheaper hull is built and cost is not what binds.
  The one remaining exit from §8.6's arithmetic is a denial covering more than
  one world, which needs a spatial object the engine does not have.
- The in-flight guard's own value is **not resolved at eight seeds** (−1.62
  against −4.87 own colonies, both error bars wider than the gap). Kept on
  waste grounds, not measured grounds.

---

### T-112. Denial: pickets, light-lagged diversion — and the card is refuted as specified

> **Measured before R-WAR9 (T-115).** The colonization leg was flown at the
> empty-hull rate, so every magnitude below is from an engine that no longer
> exists. Re-measured after the fix the headline arm reads **−21.46% own /
> +9.25% neighbors / −281 `W_0`** against the −9.55 / +4.40 / −133 recorded
> here. The direction is unchanged; the magnitudes are not.


**Built, gated off, measured on the asymmetric bed, and it does the opposite of
what it is for.** Full write-up in `Hyades_warfare_tree.md` §8.6–8.7.

The author's specification: *"A Contact Vehicle that has founded a colony
without recycling itself should travel to a nearby site and prevent it from
being colonized. A threatened Colony Ship should divert from its course… I
expect to greatly decrease the expansion of the Warfare player's neighbors."*

#### What it does (`examples/denial_census`, 8 seeds, 3 seats, 800 yr)

Seat 0 plays the card; seats 1–2 at the default doctrine — the asymmetric bed
R-TREE8 requires, and the first measurement in this project to use one.

| | mean | seeds |
|---|---|---|
| **neighbor colonies** | **+4.40% ± 1.09** (4.0 SE) | 1/8 negative |
| own colonies | −9.55% ± 2.61 (3.7 SE) | 7/8 negative |
| `W_0 = C_0 − mean(C_j)` | **−133.2 colonies ± 35.3** (3.8 SE) | 1/8 positive |

The neighbors expand **more**, at 4.0 SE with 7/8 seeds agreeing.

#### Three things establish why, and only the first is a magnitude

- **Pickets are fielded and never used.** 493–793 pickets produce **3–12
  diversions** per run — one to two percent utilization. Placement is the
  mechanism: the picket takes the world nearest the colony just founded, which
  is the picketing empire's *own* frontier.
- **Aiming at the neighbor is worse, and structurally so.** Nearest-to-rival
  placement gives **24–32 pickets and zero diversions** on every seed: a world
  this empire has scanned and a rival has not yet claimed barely exists.
  Colonization is exclusive (R-V3) and worlds go unscanned → owned without
  pausing. **There is no contested frontier to stand on.**
- **A 1:1 trade would still lose, and that is arithmetic.** Spending `k`
  colonizers to deny `k` neighbor colonies gives
  `ΔW_0 = −k + w_0A·k = −k(1 − w_0A)`, and `Σ_j w_0j = 1` by construction, so
  `ΔW_0 < 0` at any table wider than two seats **however well it is aimed**. The
  realized ratio is **−0.97 denied per spent**, where even +1.00 would not have
  been enough.

#### The defect that was mine, and the shape worth keeping

The first arm charged the card honestly — roles §4.2 makes a colony's `Band I`
stock the *recycled hull*, so a hull that leaves leaves nothing — and debited the
founding rung to **zero**. That took seat 0 from **769 colonies to 10** and
raised the neighbors **+20.0%**.

**`employment_rate` returns exactly `0.0` for a stock of zero**, and
`fabrication_rate` is `slips × berth_rate`, so a colony founded at infra 0 can
never mine, never build and never recover. **Zero is not a price, it is an
absorbing state.** Floored at the ladder's own floor rung (`Band Empty`, design
law #11/T-63) and pinned by
`a_picketed_founding_still_leaves_a_workable_colony`.

Three arms, and the monotonicity is what identifies self-harm rather than denial
— **the neighbors' gain tracks the card-player's loss one for one across a
25-point range**:

| founding rung | own | neighbors | `W_0` |
|---|---|---|---|
| zero | −57.2% | **+20.0%** | −688 |
| **floor (shipped)** | −9.6% | **+4.4%** | −133 |
| unchanged `Band I` | +2.6% | −0.7% | +31 |

Even the arm that charges the card *nothing* moves the neighbors −0.71% ± 0.54,
inside noise.

#### What it did unblock

- **An accept/decline at range**, on a genuinely light-lagged edge: a picket's
  warning reaches an inbound colonizer's home center at
  `established_at + distance/c`, and whether it beats the ship is the
  counterplay window. §7.4's *"only miners ever fight"* is amended.
- **`Role::Picket`**, the first role whose product is something that does not
  happen, and the population-destroying path design law #11 needed
  (`destroy_free_hulls` debits settlers and endowment with the hull).

#### What would have to change

- **A picket must cost less than a colony** — a colonizer is the most expensive
  object in the expansion loop, spent on an object that produces nothing.
- **Or deny more than one world per hull** — a blockade over an approach rather
  than a point clears the `1 − w_0A` bound.
- **Or run ahead of the wave** — picketing *after* founding puts hulls where
  expansion has already been. That is a production order, not a founding side
  effect, and it is a different card.

**R-WAR6** carries the magnitudes. The mechanic ships gated off
(`SimConfig::engagements_enabled` + `Doctrine::picket_after_founding`, both
false); `picketing_off_is_bit_identical` pins that the default galaxy is
untouched.

---

### T-111. Combat is wired into the simulation loop — and the occasion was always there

**Landed.** `combat::resolve_engagement` is called from `sim.rs`. Specified in
`Hyades_warfare_tree.md` §7; the arena still seeds no production and the
dependency is `sim → combat`, never `sim → arena`.

**The framing this corrects is the useful part.** §3.4 read *"there is no round
or command layer for a 'do I take this fight' decision to live in"*, and cited
T-30. Both halves had stopped being true:

- **The round layer landed** (T-30, partially — what is still missing is hidden
  simultaneity, which is T-42). The entry was citing a blocker that had shipped.
- **The premise that nothing meets was never checked, and it is false.**
  `examples/contact_census`, 3 seats, 1,500 yr: **68–75% of occupied sites are
  worked by more than one empire**, ~4,200 contacts per run, up to three seats on
  one rock. Mining is non-exclusive (roles §4.3) and has been since outposts
  existed. The engine was producing thousands of co-locations a run and nothing
  was looking at them.

**One census refuted a blocker that had stood since the standing layer shipped**,
and it cost one run. `AGENTS.md` §2's rule — *check whether the thing upstream
was ever short* — applied to a design claim instead of a knob.

#### What it is

| piece | where | status |
|---|---|---|
| trigger | `sys_mining_arrive` raises `EventKind::Engagement` on a shared rock | `O(seats)`, reads the existing `mine_crew` index |
| hostility | `Doctrine::engage_neutrals`, default **false** | R-O27/T-11's first field; the rest of the list stays open |
| master gate | `SimConfig::engagements_enabled`, default **false** | keeps the measurement corpus valid |
| accept/decline | `belief::resolve_engagement_choice` | wired at its **degenerate end** — see below |
| resolution | `combat::resolve_engagement` | the same call the arena makes |
| losses | destroyed hull mass → `World::slag` at the site | design law #11; inert per R-O59, advances **T-03** |

#### Measured (`examples/engagement_census`, 8 seeds, 3 seats, 800 yr)

| | mean | seeds positive |
|---|---|---|
| colony count | **+0.43% ± 0.17** (2.5 SE) | 5/8 |
| colony-years | **−0.74% ± 0.26** (2.8 SE) | 3/8 |

3,701–4,131 engagements and **5,488–12,819 hulls destroyed** per run, 110–258 kt
of slag. **Neither aggregate is a finding** — a 5/8 or 3/8 sign test is noise —
and the magnitude is the point: *an empire can lose its whole mining fleet
several times over and its expansion does not notice.* That is §6.19c's and
R-O92's conclusion reached by destroying the hulls rather than by counting them.

**Throughput rose on all eight seeds with `ns/event` falling**, which
`AGENTS.md` §2's table reads as "a real optimization". It is not: the workload
shrank because 11,345 hulls stopped existing. The table assumes a fixed
workload; this is the row it does not cover, and it is now recorded there.

#### Two defects it surfaced, one in my own code

- **`Rng::fork` takes `&self` and does not advance.** Forking both fleets on the
  ship index alone handed attacker `i` and defender `i` **bit-identical**
  `thrust_factor` draws, so `own == theirs`, `can_disengage` (a strict `>`) was
  false forever, and the census read **100% committed across 8,127
  engagements**. The fleet index is now in the label and it reads 49.4–51.6%.
  *A column printed only for completeness is what caught it* — R-O86's lesson
  again, and the reason to print the distribution rather than the verdict.
- **`resolve_engagement` carried two arena assumptions.** `laser_ships[0]`
  panics on an empty side, which a scenario cannot reach and the sim can; and
  `carrier_accel` reads ship 0 under a comment saying *"same hull both sides"*,
  true of a one-hull sweep and false of two empires. The first is fixed (a
  walkover), the second is **R-WAR5** — changing it moves every golden in
  `tests/balance.rs`.

#### What is still open

- **R-WAR5** — which side carries which weapon. The defender takes the lasers
  and the arriver the missiles; a convention that decides outcomes. Needs R-L0.
- **The belief layer's interesting half.** Zero range means belief *is* truth, so
  masking and surprise are untested. Needs T-33 and an engagement at range.
- **Only miners fight.** Freighters in transit, colonizers under way and
  colonies themselves are untouchable, so §4.4's commerce raiding and blockade
  and §4.1's infrastructure strike are all still unreachable. Each needs a
  detection query rather than an arrival — which is where R-AC13 and the posture
  cards actually live.
- **No magnitude is ratified.** Warfare's objective is an algebraic zero on the
  symmetric bed (R-TREE8), so nothing here was tuned against a galaxy.

---

### T-109. The write-capability partition — only Warfare may kill population

**Author's ruling, enforced at the simulation level.** Specified in
`Hyades_card_contract.md` §10; **R-TREE10** carries what is open.

Two enforcement points, for two failure modes:

1. **A `const` assertion over `TIER0`** that no non-Warfare card carries a
   population-lethal effect. Static, costs nothing at runtime, fails the build.
2. **`Order::coerce`** returns `Order::pass` for a lethal card outside Warfare.
   Redundant while the card list is static and **not** redundant once cards are
   data (contract §7 revises the list every Monte-Carlo run). Legality is a pure
   function of `(card.tree, card.effect)`, both of which every client holds, so
   the coercion needs no message and cannot desync (netcode §5.1).

**The predicate reads the write's argument, not its variant**, and the shipped
list is why: `TIER0` card 5 is **Growth**/`BiosphereRegen(1.5)`, and the variant
is lethal at any factor below `1.0` while `1.5` is harmless. Make `lethal` total
over `CardEffect` so a new variant must state its answer rather than defaulting
to permitted.

Independent of T-107 as *engine work* — the gate compiles either way — and
dependent on it for the gate to matter.

---

### T-110. The first Warfare card — the armed colonizer

Specified in `Hyades_warfare_tree.md` §7 — the first concrete answer to
**R-WAR2** — with the roles amendment in `Hyades_vehicle_roles.md` §4.2. One Design write and three Doctrine writes; the
Design half moves the Colonizer role from the Medium Systems hull to a Contact
class, and the third Doctrine write stops the colonizer recycling itself into
the colony it founds.

**One third of the stated intent was refuted by the engine's own algebra and is
withdrawn.** The intent was *worse cargo per kilotonne of cost, lower laden
acceleration, higher dry acceleration*. Since T-96 thrust does not vary with the
load and since R-O57 cost is dry mass, so

```text
a_dry / a_laden  =  1 + C / M_dry
```

exactly — the empty-to-laden swing **is** the cargo efficiency (R-O95, pinned by
`sim::tests::the_acceleration_swing_is_the_cargo_efficiency`). Cutting cargo
efficiency narrows the swing, so a hull cannot be worse at freight, faster empty
and slower laden at once. **Decided: keep the first two and take the narrowed
signature as the compensation** — an armed colonizer is worse freight and
*quieter*, which is a Warfare property bought with no combat constant
(standing layer §9.2).

**Blocked, in this order** — none of it is parallel:

| # | blocker | code |
|---|---|---|
| 1 | no combat in the simulation loop; `combat::resolve_engagement` is never called from `sim.rs` | T-12 / T-30, via T-52 |
| 2 | `Doctrine` has no diplomatic stance field | R-O27 / T-11 |
| 3 | `drive_volume_fraction` is global, not per-`Class` | T-97, itself blocked on T-08 |
| 4 | Warfare's objective is an algebraic zero on the standard bed, and `w_ij` is unset | R-TREE8, R-WAR3 |
| 5 | population is not a factor of production | **T-107** |

**The evaluation is against a Growth card that raises the growth rate**
(`TIER0` slots 3 and 4), on the asymmetric bed R-TREE8 calls for, and it needs
two things settled first. **R-TREE12**: Warfare's stock is a difference that can
go negative, so it has no doubling time and §4's numeraire does not apply —
decided as the fractional *increase* the card causes in the **target's**
doubling time. And **the Growth card's own ratified numbers do not transfer**:
`growth_rate` was measured on colony count, which is Expansion's objective, and
T-94/T-95 consumed the surface outright. Re-measure on work-years at the 3-kyr
bed.

**Prediction, written down so the measurement can refuse it:** Growth compounds
and this card does not, so Growth's P92 should exceed Warfare's at earliest
legal play with Warfare's dispersion higher — and §4.3 requires tier-1
dispersion to be the lowest of any tier, so if that holds, the pair does not
belong at tier 1.

---

### ~~T-96. A laden GSV is the most sluggish hull in the game~~ — **DONE (R-MC16, §2.3)**

> **The drive is a mass, not a stat.** Civilian motion priced thrust as
> `civilian_accel_g × dry_mass`, and dry mass is the *shell* — so thrust scaled
> `r²` while the load scaled `r³`, laden acceleration fell as `1/r`, and design
> law #3's consolidation advantage was being paid back in turnaround. A laden GSV
> flew at **0.317×** a laden MSV and took **1.25×** as long door to door.
>
> `thrust = k · (δ·shell + ρ·φ·(hold − V_reserved))`. Three quantities and they
> are the three the drive needs: **`k = 18.21`** specific thrust (kt·g per kt of
> drive — one unit of engine, one proportionate unit of acceleration),
> **`δ = 0.05`** the share of a hull's own structure that is drive, **`φ = 0.01`**
> the share of usable interior a Design mounts. Volume comes out of the hold and
> its mass out of the mineral budget, so speed is paid for in cargo, mass and
> price at once.
>
> **`k` is derived, not chosen**: solved so an empty Limited Systems hull still
> flies at 1 g, which is what makes the change landable — scouts and miners fly
> small hulls mostly empty and stay where they were (LCV 0.911 g).
>
> **`δ` is load-bearing.** An LCV's reserved core (0.138) exceeds its entire hold
> (0.026), so a purely volumetric drive gives it **zero thrust** and a scout never
> moves. At `δ = 0` an LSV drops from 1.00 g to 0.06 g.
>
> **Result.** GSV ÷ equal-cost MSV fleet: laden acceleration **0.317 → 0.814**,
> empty **1.00 → 2.14**, round trip **1.252 → 1.011**. Empty acceleration now
> rises with size (1.00 / 2.37 / 5.06 g) — the ocean-liner statement, falling out
> of the same `r³`-over-`r²` that gives design law #3 its cost advantage, with
> nothing tuned to produce it. The General hull's disadvantage is **removed
> rather than reversed**, and its 2.7× throughput-per-mineral advantage survives.
>
> Objective, pooled over eight seeds including four it was not chosen against:
> **+11.33% ± 4.84 work-years, 7/8 (2.3 SE)**, colony-years +1.02% ± 0.59 (5/8,
> marginal), throughput −4.7%. Read the mechanism first: the round-trip ratio is
> what this change is about and it moved decisively; the objective agrees in sign
> at a bar this file calls a coin on its edge, and one seed is strongly negative.
>
> **Nothing builds a GSV freighter yet**, so none of that gain is the General
> hull being used — it is the whole fleet flying faster laden. Freight hull
> choice is the follow-on (T-98), and it must be **ablated apart** from this one.
>
> **Successor: T-97** — `φ` has to move onto `Class` or design law #10's inverse
> problem collapses.

---

### ~~T-101. 82% of the candidate scan re-derives a permanent answer~~ — **DONE**

> **Both filters are monotone, so a rejected entry is rejected forever.**
> `Knowledge::targeted` is only ever inserted and a planet's `owner` is only ever
> set — there is no `remove` for either anywhere in the engine — so the 206 M of
> 251.6 M scan steps that failed one of them were re-deriving a permanent answer,
> once per production decision, for the rest of the run.
>
> `ScannedSet` now carries a **live pool** beside `ids`: the scanned worlds not
> yet targeted and not yet owned, in the same ascending order. Entries are
> appended when a world is first scanned and **compacted out in place** during
> the scan that observes them rejected, so the set pays for its own maintenance
> and there is no invalidation hook to forget.
>
> **Compaction, not swap-removal, and that is the whole difference from the
> attempt that failed.** R-O70 tried an incrementally-maintained frontier before:
> it cut the scanned count 39% and came out *slower*, because swap-removal
> scrambled the order and traded a sequential walk for random access across three
> component stores. A retain-style compaction preserves ascending order, so the
> walk stays sequential and the survivors stay a sorted subsequence of `ids` —
> which is also what makes the result bit-identical, since the scan visits the
> same worlds in the same order and simply skips the ones it would have skipped.
>
> **Measured, four interleaved pairs in one session** (the arms must be
> interleaved — this container's speed drifts enough between sessions to swamp
> the effect):
>
> | | seed 1 | seed 7 |
> |---|---|---|
> | T-100 | 95.4–97.4 | 98.6–100.1 |
> | **T-101** | **106.2–107.3** | **111.1–111.5** |
>
> **+10.6% / +11.9%**, bit-identical — 357,786 events, 2,907 colonies and total
> population to the last digit on seed 1.
>
> **And the interesting number is the one that did *not* show up.** Scan steps
> fell **251.6 M → 45.5 M, −82%**, with the ranked count unchanged at 45.44 M —
> so the pruning removed exactly the dead entries and nothing else. Deleting 82%
> of a loop's iterations bought **11%**, because the iterations deleted were two
> bitmap lookups apiece. **Iteration count is not cost.** The residual is T-102.

---

### ~~T-100. `rank` recomputes three logarithms per scanned world~~ — **DONE**

> **The production candidate scan is the engine, and one line of it was the
> cost.** Instrumented on the standard bed, 800 yr, 3 seats: the scan is reached
> **56,706** times, walks **251.6 M** entries (4,437 per decision), and **45.4 M**
> of those survive the two filters and reach `view_of` + `rank` — 703 scan steps
> and **127 rank calls per event**.
>
> `rank` scores ore as `Σ_c scarcity_c · Band(m_c)`, and `Band(·)` is a `ln`. So
> that is ~136 M logarithms for a quantity that changes only when the rock is
> mined. **Ablated** — the three conversions replaced by a constant — throughput
> goes **80.2 → 187.4 yr/s**; ablating the `sqrt` + `exp` in `centrality` beside
> it gives **89.0**, so the two are not comparable and only one was worth a
> cache. That ablation is why no effort went into the second.
>
> Three changes, all **bit-identical** (357,786 events, 2,907 colonies,
> population to the last digit on seed 1):
>
> - **A memo for the three Band readings**, keyed on the field's own bits rather
>   than invalidated at each `density` write. The obvious design is an
>   invalidation hook; the obvious failure of it is the write somebody adds
>   later — `world.density.get_mut` is reached directly in eight places, most of
>   them tests. Three `f64` compares against three `ln`s, correct by
>   construction instead of by everyone remembering.
> - **`view_of` used `f.bio_max.in_bands()`** where `Factors::bio_max_band` is
>   the same value already kept in step. R-O70 put that field there precisely to
>   keep a `ln` off the hot path, and the caller was still recomputing it.
> - **`PlanetView` no longer carries a `MineralField`.** Nothing in the seam read
>   the masses — `rank` converted them and threw them away — so the copy was a
>   memory-traffic tax on the hottest path for a field with no reader
>   (`AGENTS.md` §4: hand a decision only the fields it reads).
>
> Measured against the old binary, three runs each: seed 1 **79.0–81.9 →
> 104.6–114.4 yr/s**, seed 7 **86.6–91.7 → 107.9–118.2**. About **+30%**, for
> **0%** disturbance to any simulation metric.
>
> **What is left, and it is most of it.** The ablation ceiling is ~187 yr/s and
> this reaches ~114, so roughly half the available win is still on the table. The
> residual is the scan itself: 206 M of the 251.6 M steps are rejected by the two
> filters, and the survivors still pay a `PlanetView` construction and a full
> `rank`. **T-101.**

---

### ~~T-98. Freight never builds a General hull~~ — **DONE (R-O94, §6.26)**

> **The hull is a forecast now.** `freighter_hull` sizes it to
> `min(supply_rate, demand_rate) x round_trip` and ranks candidates on delivered
> kilotonnes per year per kilotonne of hull — design law #3's own quantity.
> Supply is the rock's yield as `sys_mining_tick` computes it; demand is the
> destination's fabrication throughput; both are `O(1)`, and the round trip is
> solved rather than assumed because the load depends on it and it on the load.
>
> **+170.1% ± 16.2 work-years, 8/8 seeds (10.5 SE)**, colony-years
> **+9.54% ± 4.19 (7/8)**, throughput **+73%** with `ns/event` −46%.
>
> **The liquidity term is most of the value and the ablation is the reason that
> is known.** Capping candidates at what the center can pay for *now* costs no
> new constant — the budget is the bank, which the decision already lives under.
> Without it the same rule scores **+51% work-years and −17% colony-years on 1/8
> seeds**, and `all_fair_counts_run_and_expand` founds nothing in forty years:
> a thin-banked center buys one General hauler instead of twelve Mediums and
> stops expanding while it saves. That is design law #3's *indivisibility as a
> liability*, and a steady-state rate cannot see it.
>
> **Mechanism check — this is T-92's ceiling moving.** Freight's share of bank
> inflow **1.73% → 14.70%**, ore ever collected 0.21% → 2.30%, payable fraction
> 0.052 → 0.083, infrastructure builds 335 → 601. Bigger haulers on rich rocks
> move 14x the tonnage, and because the milk run mixes colors *within* a hold
> that tonnage is payable. The two compose; neither does this alone. **T-92 is
> advanced, not closed** — 85% of inflow is still the center's own ground.
>
> **Cost: colony count −6.1%** while colony-years rise 9.5%. Worlds are taken
> earlier and the tail is shorter. Same shape as R-O66's −178: a
> deepen-versus-expand reallocation on better information, and a question for
> `expand_bias` and T-20.
>
> **Successor: T-99** — demand is a ceiling, not a share.

---

### ~~T-94. The population logistic is a forward Euler step at `r·Δ = 0.873`~~ — **DONE (R-O93, §6.25)**

> **It has a closed form.** `x(t+Δ) = K·x / (x + (K − x)·e^(−rΔ))` — the ceiling
> is constant across a tick (`K = min(hab, bio_max)`, the *pristine* biosphere),
> so the population step is autonomous and solvable. `settler_target` had been
> pricing colonization off this solution's inverse since R-IND11, so the engine
> was already carrying both the true curve and a crude walk of it.
>
> Euler was **79% low** at `cycle_years = 50` and still **21% low** at T-88's
> refined 5. **+6.57% ± 1.14 colony-years, 8/8 seeds (5.7 SE)**, work-years
> noise, **throughput unchanged** — one `exp` per center per tick is below the
> run-to-run variance, and `ns/event` fell.
>
> Retires three things, and the third is the one to remember: design law #11's
> **`r < 2` ceiling** was a property of the Euler map; the **clamp at `K`** stops
> being load-bearing; and the **undershoot below `K`** was truncation error whose
> severity was a function of `cycle_years` — so the outcome of an attack on a
> world's habitability was being set by a performance knob. The *collapse*
> survives (31.62 → 8.88 kt in one tick, converging from above); only the
> undershoot goes.
>
> **Successor: T-95.** Exact @ 50 beats Euler @ 50 by +33% work-years at the same
> cost, and is still 29% below exact @ 5 — so the tick has a second job.

---

### ~~T-91. A hold is loaded from one rock, so every delivery is one color~~ — **DONE (R-O92, §6.24)**

> **The milk run.** An outbound leg now visits `SimConfig::max_pickup_stops`
> piles before turning for its destination; the final stop fills the hold as
> R-O89 does, and every earlier one takes each color capped at what is still
> wanted **and** at its proportional share of the hold. `Shuttle` gained an
> immutable `base` (the hauler's own miner's rock, which every leg starts and
> ends at) and a per-leg `stops` counter.
>
> **Ratified at 2: +55.13% ± 4.65 work-years, 8/8 seeds (11.9 SE)**, replicated
> on four seeds it was not chosen against, colony-years +2.1% and +3.8% on the
> two beds, ~12% throughput. **Two is a peak** — 1/2/3/4/6 score
> 184k / **284k** / 261k / 236k / 190k work-years on the standard bed — and the
> share cap is worth **+10.2%** over the obvious `min(want, room)`, because a
> geometric bill outgrows a hold and past that point `min(want, room)` *is*
> `room`.
>
> **The stated acceptance criterion fired, and passed.** The payable fraction had
> sat at **0.043** through T-81, R-O89's pickup arm and T-90; it is **0.052** at
> two stops, with infrastructure builds 268 → 335. Writing it down before the
> measurement is what made it a criterion rather than a rationalisation, and it
> has now refused one change and passed one.
>
> **What it does not reach is T-92**, and that is the finding worth carrying:
> freight is **1.73%** of everything that ever enters a bank. The lesson is the
> opposite of "the channel was too small to matter" — mass was never binding, a
> **conjunction** makes the minority color the whole constraint, and a channel
> carrying 1.7% of the mass carried all of the scarcity.

---

### ~~T-90. Outpost selection is color-blind — R-O91~~ — **IMPLEMENTED AND REFUTED**

> **The term is a real defect and it is not the cause.** `local_scarcity` weighted
> a world's colors by the deciding center's own shortfall against its next rung,
> reducing exactly to the old constant when nothing is short. Measured:
> **payable fraction 0.043 → 0.043**, dead share **99.7% → 99.7%**, work-years
> **+1.12% ± 4.31 (2/4 — noise)**, colony-years **−3.30% ± 0.49 (0/4)**.
> **Reverted.**
>
> **And not because the gain was too small**, which is the other thing that
> result could have meant and is worth four runs to separate. Swept 0 / 1 / 4 /
> 16, the payable fraction reads 0.043 / 0.043 / 0.057 / 0.045 and the dead share
> is 99.7% at every one.
>
> **The mechanism is in §7.4 and it is provable from the code**: the empire
> already mines a balanced mix, a rock is one color, and a hold is filled from
> exactly one rock. Selection had nothing to fix. The successor is **T-91**
> above.



**The third missing term, and it is one step further back than the last two.**
λ gave freighter routing a *distance* component it did not have (coverage
14.4% → 38.3%); R-O89 gave the freight *load* leg a *color* component it did not
have (+8.4% work-years). This is the decision that puts a mining pair on a rock
in the first place, and it has a color term that reads a **constant**:
`RankWeights`-scored `scarcity_c` is written once at game start from the
homeworld archetype and never again, while `mineral_pressure` is live but
**scalar**. The empire can say *mine more*; it cannot say *mine Cyan*.

**Measured consequence** (`examples/bank_mix`, `examples/decision_census`,
`Hyades_industry.md` §6.23/§7.4):

- **79.8% of production decisions return `Idle`**, and **100% of those are
  "wants to deepen, cannot pay the bill"** with a mean bank of **561 kt**. Not
  one idle decision in the run had `can_afford_infra == true`.
- **99.7% of 360,517 banked kt cannot pay a balanced rung at any price.**
- The median bank's dominant-color share is **0.871** — *worse* than the
  field's 0.789 — so freight **concentrates** the geology rather than mixing it.

**Why not pickup re-routing.** Measured at R-O89 and it costs **−52.3%**, with
transit, per-hull throughput and hull recycling each refuted as the reason and
the cause still open. Outpost selection is a different intervention: made once at
build time, it leaves the 1:1 miner↔hauler pairing intact, and it is where
`AGENTS.md` §4 says this class of decision belongs.

**R-O91 is what has to be settled first** — whose shortfall the live term reads
(a purely local one makes every center chase the same color at once; the empire
aggregate is already balanced at 171x–2,356x and would say nothing), and whether
reading ore on hand makes the term farmable under §2's invariance rule. Scarcity
read from *bills outstanding* rather than ore on hand is the candidate that is
not.

**Acceptance is the mechanism, not just the objective.** Work-years is the
metric; `bank_mix`'s payable fraction is the check. A change that does not move
**0.043** has not done what it claims, whatever the objective reports.

---

### T-81. Freight has no color term — **LANDED AND MEASURED FALSE**

**A missing term, not a tuning question** — the same shape as λ, the largest
single ratification in this project's history (freighter routing had no *distance*
component; adding it took coverage 14.4% → 38.3%).

T-73 made a works bill payable **in named colors**. T-62 made the mineral field
log-normal **per color**. Together, a center's bank is dominated by one color
with traces of the others — 1,494 of 1,515 non-empty banks measured skewed at
t=800, one holding `C 477.2 / M 0.0095 / Y 1.04` — so a bill naming all three is
unpayable there. Measured effect: **infrastructure builds fell 1,032 → 57**, a
94.5% cut, with the minerals going to hulls instead.

`most_needed_center` routes ore by mineral **pressure** — how broke a center is
overall — and carries **no color term at all**. Route by what a center's *bill*
needs and the constraint gets the answer §5.3 promises and design law #13
requires. Note that all four of §5.1's ratio points (`1:0:0`, `4:2:1`, `3:2:1`,
`5:4:3`) demand every color except the first, so the alternative routes do **not**
supply that answer on their own; freight does, which is §8.1's subject.

**Landed with the ratified `3:2:1` Y:C:M default mix** (R-IND15 resolved,
`Hyades_industry.md` §6.10) as two separate commits so the two effects can be
attributed apart. Routing now scores `relief · exp(−λ·t)` where `relief` is the
fraction of the cargo aboard that lands on a color the destination cannot
otherwise buy; it *replaces* `mineral_pressure_of` in the score rather than
multiplying it, since the two are the same question at different resolutions.

**It does not work and it is counterproductive** (`Hyades_industry.md` §6.11).
Banks did not mix (1,533 of 1,551 still skewed) and infrastructure builds fell
further, **57 → 31**. Two measured mechanisms: the **supply is single-colored**
(6,725 sources, mean dominant-color share **0.789**, 38% at ≥95%), so a
freighter's cargo inherits the skew; and **relief routing anti-concentrates** —
paying a three-color bill needs ore to converge on one center, and sending each
color to wherever that color is scarcest scatters it by construction.

Live proposal is **R-IND17**: score the *completion of the bill*
(`(short_before − short_after) / Σ bill`) rather than the relief of one color, so
a center holding two colors and missing the third attracts the third and ore
concentrates. Not ratified.

**And the premise was too strong.** No internal routing can give an empire a
color its own ground does not hold — that is §8.1 and the Exchange (T-77), with
design law #1's counter-graph as the other half.

---

## ~~T-88 — decouple economic *granularity* from decision *rate*~~ — **CLOSED**

> **Landed.** The retry is severed from the economy tick, the per-cycle rates
> are scaled by the tick, and `cycle_years` is ratified at **5.0**.
>
> **The severance works and the numbers say so.** At `cycle_years = 1`, economy
> ticks go **48,707 → 3,602,083 (74x)** while decisions go **97,197 → 128,726
> (1.32x)**. That ratio *is* the task. Two mechanisms do it: `wake_on_minerals`,
> which decides the moment a freighter deposits (a saving center is waiting for
> exactly one thing, and ore landing is the event that changes whether it can
> buy anything), and `decision_after`, a per-center floor evaluated on the tick
> as the catch-all beneath it. The wake path is also **cheaper than the cadence
> it replaces** — ~26,800 deposits against ~99,000 ticks over 1,500 yr — so
> responsiveness went up and decision count went down.
>
> Landed in stages so each is attributable:
>
> | stage | what | measured |
> |---|---|---|
> | A | `decision_after` floor, `decision_retry_years = 50` | **bit-identical** — the gate passes every tick at equal values |
> | B | `wake_on_minerals` on freight deposit | **+6.81% ± 2.36 work-years, 7/8 seeds** (replicated) |
> | C | skip the candidate scan when nothing is affordable | behavior-identical, and **−3.3%** — see below |
> | D | `tick_scale`: per-cycle rates scale with the tick | bit-identical at `cycle_years = 50` |
> | E | `cycle_years` **50 → 5** | **+21.00% ± 4.19 work-years, 8/8 seeds** |
>
> **The sweep found a units defect before it found an answer.** `growth_rate` is
> documented `1/cycle` and the logistic stepped it once per tick *regardless of
> the tick's length*, as do `biosphere_regen_rate` and the center's mining
> fraction. So the first sweep reported **+58.6%** at `cycle_years = 1` — which
> measured a fifty-times-faster economy, not a better-integrated one. `tick_scale`
> multiplies every per-cycle rate by `cycle_years / rate_reference_years`, which
> is exactly `1.0` at the shipped cadence, so **no Monte-Carlo-tuned magnitude
> moves** and the re-denomination lands as a no-op.
>
> **What survives the correction is still large, and it is a genuine Euler
> result.** The logistic advances by `r·dt` per tick and `r·dt = 0.873` at the
> shipped values — stable under design law #11's `r < 2` bound and nowhere near
> accurate. A homeworld's population at 300 yr:
>
> | `cycle_years` | 50 | 25 | 10 | 5 |
> |---|---|---|---|---|
> | homeworld population | 1,143 | 2,275 | 3,516 | 4,297 |
> | ratio to next coarser | — | 1.99 | 1.55 | 1.22 |
>
> The 50-year step **under-integrates by ~4x**. Refining it is worth
> **+21.00% ± 4.19 work-years (8/8 seeds)** and **+19.9% colony-years**, and it
> **saturates at 5** — `cycle_years = 1` scores the same +20.3% on the original
> bed for another 10% of throughput. Shipped at **5**.
>
> **Cost: 93.3 → 71.1 yr/s** on the standard bed; at 3 seats / 4 kyr, **68.9
> yr/s** with **22,394 ns/event** (against ~174,000 at T-87 — the event mix is
> now dominated by cheap economy ticks, which is what the table's `ns/event`
> column is for).
>
> **Every gradient measured before this is consumed.** The operating point moved
> a long way; `data/tree_gradient.tsv` and T-45's table are pre-T-88 and must be
> re-run, not stepped along.
>
> **Opened by it:** the idle census below, which is the freight question.

### What the decision census found, and it is not what was expected

`examples/decision_census` counts the work rather than the aggregate. On the
standard bed at 1,500 yr, seed 1:

| | before T-88 | after |
|---|---|---|
| decisions | 77,961 | 97,197 |
| **idle share** | 71.3% | **79.8%** |
| candidate-scan steps | 5,096,496 | 10,304,207 |
| per decision | 65 | 106 |

**The scan is the engine's largest loop and 80% of it produces nothing** — which
is R-O86's rule ("ask what fraction of a busy path produces nothing") pointing at
T-52. The obvious fix was to skip the scan when the center cannot afford
anything; it is **behavior-identical and bought −3.3%**, which refutes the
premise: those centers can afford *something*.

Breaking the idles down by their own logged inputs says what they are, and it is
a single bucket:

> **100% of idle decisions are "wants to deepen, cannot pay the bill", with a
> mean bank of 561 kt.**

Not one idle decision in the run had `can_afford_infra == true`. That is R-O85's
color conjunction — the bank holds the total and lacks a color — measured from
a completely different direction, and it is the reason R-O89's freight fix did
not produce more than it did. See §"Why freight is not providing serious upside".



**Author's directive: "move all decision making to trigger off an event so the
economy tick can be made 1/year without adversely affecting years/second. I want
to improve the granularity of the economic simulation, not the rate of decision
making."**

**Measured, and it is the largest throttle left in the economy**
(`examples/founding_tree`, 3 seats, 1,500 yr, at a homeworld):

| | |
|---|---|
| mean gap, decision → next decision, **after a committed build** | **1.5 yr** (seed 7: 1.6) |
| mean gap, **after an `Idle`** | **29.6 yr** (seed 7: 31.8) |
| share of decisions that idle | 18% (seed 7: 11%) |
| **yard utilization** | **18.8%** (seed 7: 28.6%) |

`commit_one_build` returning `None` leaves the yard free and **schedules
nothing** — the next attempt is the economy tick. So a single declined build
costs up to `cycle_years` of yard time, sixteen builds' worth at rung II's
3.1-year `t_build`, and **81% of a homeworld's production timeline is spent
waiting out that cadence.**

That is not merely a fidelity problem, it is why the industrial knobs read flat.
`reinvest_bias` can only buy *build rate* (R-O87/§6.19a), and build rate governs
the other 19% of the timeline — so a +29% rung improvement is worth +5.5% at the
absolute best before anything else attenuates it. **Fix this before R-O85 and
before re-sweeping any economic knob**; a sweep run today is measuring the retry
cadence.

> **Superseded by R-O88, which largely dissolved this as a side effect.** The
> retry cadence bit because a declined build left the yard with *nothing*
> scheduled. With berths instead of two, some other berth clears and re-triggers
> the center long before the economy tick does: the after-idle gap measured
> **29.6 → 7.6 yr** and the after-build gap **1.5 → 0.1 yr**
> (`Hyades_industry.md` §6.19b). The 81% figure above is the pre-R-O88 number and
> should not be re-cited.
>
> **T-88 is still worth doing for the reason it was opened** — economic
> *granularity*, a 50-year Euler step on a logistic, which is a fidelity argument
> and not a throughput one. It is no longer the dominant throttle, so the order
> is now **R-O85, then re-sweep**, with this below both.

`cycle_years = 50` is doing two unrelated jobs and they want opposite values:

| job | wants | why |
|---|---|---|
| **Economic integration step** — mine, grow, mint `$` | **small** (1 yr) | every one is a *rate over an interval*, and a 50-year step is a coarse Euler step on a logistic. Fidelity is the whole reason to shrink it. |
| **Decision retry cadence** — a saving center re-checks whether it can afford anything | large, or better, **not a cadence at all** | a decision is triggered by a *change in situation*, and `AGENTS.md` §4 says so: entities evaluate on their own arrival events, never on a sweep. |

Dropping `cycle_years` to 1 today multiplies **both**, and the decision half is
what costs throughput. R-O69 already moved the production decision *off* the
tick for a center with a build under way — but `sys_production_tick` still calls
`sys_build_decision` directly when the yard is free (`src/sim.rs`, "the mining
step doubles as its retry"). That call is the remaining coupling.

**The work:** make the retry event-driven too. A saving center's situation
changes when *minerals arrive* — `FreighterArrive`, the local extraction that
crosses an affordability threshold, or a `Reserve` hull becoming available — not
when a fixed clock ticks. Raise `BuildDecision` from those, and the economy tick
becomes purely economic: mine, grow, mint, reschedule.

**Then `cycle_years` is free to fall to 1.0**, and what it buys is real: T-64's
logistic is conjugate to the logistic map, so step size is not a cosmetic choice
— a 50-year step is why `growth_rate` is a *step function of itself* (T-64/R-O84,
the objective is piecewise constant in `r` because what matters is how many
50-year cycles a center takes to cross a `PopBands` edge). **A 1-year step would
give that surface a gradient a probe can actually read**, which is worth more
than the value it would find.

**Guard: `ns/event` beside `yr/s`** (`AGENTS.md` §2). This is exactly the change
that decomposition exists for — the *right* outcome is `yr/s` down and
`ns/event` flat or better, which is "the simulation is doing more, each unit
costs the same". Reading the aggregate alone would call a fidelity improvement a
regression. Colony-years and work-years are the behavior guards; a 50x finer
integration step **will** move them and that is not a bug, so expect to
re-ratify `growth_rate` after it rather than treating the shift as a defect.

---

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

---
