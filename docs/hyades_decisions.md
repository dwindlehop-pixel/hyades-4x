# Hyades — open decisions overview

**Every open decision (R-code) is a GitHub issue** labeled
[`decision: needs ratification`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22decision%3A+needs+ratification%22) and one
`spec: <name>` label per spec that lists it. This file is the overview. The spec
register is where a decision is *stated*; the issue is where it is *discussed and
ratified*: its source text, the spec sections that cite it, what would settle it,
and what it is blocked by.

- To-do work: [`docs/hyades_todo.md`](hyades_todo.md)
- A code that appears in several specs has **one** issue, listing every spec.

## Rules

- **A new open decision is a new issue**: title `R-XX: <question>`, labels
  `decision: needs ratification` and `spec: <name>`, a body that says what would
  settle it. Add one line for it to the spec's register, linking the issue, and a
  row here.
- **Ratifying is three edits in one PR**: the spec section states the decision
  (`AGENTS.md` §6), the register row is marked resolved, and the issue is closed
  as completed with a comment that states the ruling. A question that stops
  applying is closed as not planned, with the reason.
- **Blocks are GitHub issue dependencies.** A decision that cannot be settled
  until another issue lands is marked *blocked by* it; a related issue that does
  not block is mentioned in the body, which links the two both ways.

## Index by spec

The spec column is where the code is defined; other specs that cite it are on
the issue.

### Expansion and Growth (autopilot, colonization, growth) — [`Hyades_autopilot_colonization_growth.md`](Hyades_autopilot_colonization_growth.md) · [`spec: autopilot`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+autopilot%22)

| Code | Issue | Question |
|---|---|---|
| R-AC4 | [#133](https://github.com/dwindlehop-pixel/hyades-4x/issues/133) | Scan dwell time and close-scan range |
| R-AC5 | [#134](https://github.com/dwindlehop-pixel/hyades-4x/issues/134) | The rank formula's shape and class thresholds |
| R-AC6 | [#135](https://github.com/dwindlehop-pixel/hyades-4x/issues/135) | Provisional remote ranks before a close scan |
| R-AC7 | [#136](https://github.com/dwindlehop-pixel/hyades-4x/issues/136) | Colonizer cost / build time / founding infra as independent magnitudes |
| R-AC8 | [#137](https://github.com/dwindlehop-pixel/hyades-4x/issues/137) | Contested-claim resolution |
| R-AC9 | [#138](https://github.com/dwindlehop-pixel/hyades-4x/issues/138) | "nearest center" under light-lag |
| R-AC10 | [#139](https://github.com/dwindlehop-pixel/hyades-4x/issues/139) | Mining and freighter rates |
| R-AC12 | [#140](https://github.com/dwindlehop-pixel/hyades-4x/issues/140) | Multi-target output split across filled berths |
| R-AC13 | [#141](https://github.com/dwindlehop-pixel/hyades-4x/issues/141) | "if pressed" at the colonization layer |
| R-AC14 | [#142](https://github.com/dwindlehop-pixel/hyades-4x/issues/142) | The three drive constants of §8.4 |
| R-AC15 | [#143](https://github.com/dwindlehop-pixel/hyades-4x/issues/143) | Lock enough to author depth-1 beats |
| R-AC18 | [#144](https://github.com/dwindlehop-pixel/hyades-4x/issues/144) | Should the Colony class carry a K floor at all |
| R-AC20 | [#145](https://github.com/dwindlehop-pixel/hyades-4x/issues/145) | center_mining_fraction's sign |

### Simulation model — [`Hyades_simulation_model.md`](Hyades_simulation_model.md) · [`spec: simulation model`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+simulation+model%22)

| Code | Issue | Question |
|---|---|---|
| R-SIM2 | [#146](https://github.com/dwindlehop-pixel/hyades-4x/issues/146) | Expose nearest-unvisited survey query as an engine primitive? |
| R-SIM4 | [#147](https://github.com/dwindlehop-pixel/hyades-4x/issues/147) | Departure traffic as an inferential signal (per-player observation state) |

### Card contract — [`Hyades_card_contract.md`](Hyades_card_contract.md) · [`spec: card contract`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+card+contract%22)

| Code | Issue | Question |
|---|---|---|
| R-C1 | [#148](https://github.com/dwindlehop-pixel/hyades-4x/issues/148) | Closed list of legal target_rule kinds |
| R-C2 | [#149](https://github.com/dwindlehop-pixel/hyades-4x/issues/149) | Realization curves per order family |
| R-C3 | [#150](https://github.com/dwindlehop-pixel/hyades-4x/issues/150) | Scaling house-rules per tree |
| R-C4 | [#151](https://github.com/dwindlehop-pixel/hyades-4x/issues/151) | Lock the engine-efficiency example (magnitude, class, singleton) once hulls exist |
| R-C5 | [#152](https://github.com/dwindlehop-pixel/hyades-4x/issues/152) | Order precedence algebra |
| R-C6 | [#153](https://github.com/dwindlehop-pixel/hyades-4x/issues/153) | The V state-vector, and exact versus estimator |
| R-C7 | [#154](https://github.com/dwindlehop-pixel/hyades-4x/issues/154) | Balance metrics and thresholds |

### Command cards — [`Hyades_command_cards.md`](Hyades_command_cards.md) · [`spec: command cards`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+command+cards%22)

| Code | Issue | Question |
|---|---|---|
| R-1 | [#155](https://github.com/dwindlehop-pixel/hyades-4x/issues/155) | Tree table: objects are placeholders; redirect object flavor? |
| R-2 | [#156](https://github.com/dwindlehop-pixel/hyades-4x/issues/156) | Action economy: combo-strength target and per-tree depth and branch numbers |
| R-3 | [#157](https://github.com/dwindlehop-pixel/hyades-4x/issues/157) | How far cards fold into doctrine; ship-class naming |
| R-4 | [#158](https://github.com/dwindlehop-pixel/hyades-4x/issues/158) | Where backbone and production live: migrate premises into the sim model? |
| R-5 | [#159](https://github.com/dwindlehop-pixel/hyades-4x/issues/159) | The full clock math: first elimination 10–20 min, full game 30–45 |
| R-6 | [#160](https://github.com/dwindlehop-pixel/hyades-4x/issues/160) | Sim-model line for the peaceful-default rule |

### Galaxy and autopilot — [`Hyades_galaxy_and_autopilot.md`](Hyades_galaxy_and_autopilot.md) · [`spec: galaxy`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+galaxy%22)

| Code | Issue | Question |
|---|---|---|
| R-7, R-9 | [#161](https://github.com/dwindlehop-pixel/hyades-4x/issues/161) | Standing and peaceful action costs (numbers placeholder pending R-5) |
| R-G0 | [#162](https://github.com/dwindlehop-pixel/hyades-4x/issues/162) | Sim §1/§6 absorb the 3D/2D split |
| R-G1 | [#163](https://github.com/dwindlehop-pixel/hyades-4x/issues/163) | Galaxy prism depth (hex side is ratified at 70 ly) |
| R-G2 | [#164](https://github.com/dwindlehop-pixel/hyades-4x/issues/164) | Galaxy planet counts and core |
| R-G3 | [#165](https://github.com/dwindlehop-pixel/hyades-4x/issues/165) | Archetype placement |
| R-G5 | [#166](https://github.com/dwindlehop-pixel/hyades-4x/issues/166) | The hex misses the per-seat hexes-per-player target at 6 seats |
| R-G6 | [#167](https://github.com/dwindlehop-pixel/hyades-4x/issues/167) | Star-field scale against the hex-placed homeworld ring at 6+ seats |
| R-G7 | [#168](https://github.com/dwindlehop-pixel/hyades-4x/issues/168) | A neutron star as a system's third star: an energy site |
| R-M2 | [#169](https://github.com/dwindlehop-pixel/hyades-4x/issues/169) | Mineral ratios and wastage |
| R-M3 | [#170](https://github.com/dwindlehop-pixel/hyades-4x/issues/170) | Mineral Z scale-height |
| R-M4 | [#171](https://github.com/dwindlehop-pixel/hyades-4x/issues/171) | Mineral anticorrelation |
| R-M5 | [#172](https://github.com/dwindlehop-pixel/hyades-4x/issues/172) | Mineral supply chain (resolved in part) |
| R-M6 | [#173](https://github.com/dwindlehop-pixel/hyades-4x/issues/173) | Super aspect-sets |
| R-M7 | [#174](https://github.com/dwindlehop-pixel/hyades-4x/issues/174) | Mineral use-domains and apex |
| R-M8 | [#175](https://github.com/dwindlehop-pixel/hyades-4x/issues/175) | Growth and Cyan soft fit |
| R-P1 (population) | [#176](https://github.com/dwindlehop-pixel/hyades-4x/issues/176) | Weibull k and population bands |
| R-P2 (population) | [#177](https://github.com/dwindlehop-pixel/hyades-4x/issues/177) | Population to design gating |
| R-P3 (population) | [#178](https://github.com/dwindlehop-pixel/hyades-4x/issues/178) | Infrastructure rate versus the clock |
| R-A1 | [#179](https://github.com/dwindlehop-pixel/hyades-4x/issues/179) | Autopilot expand-bias |
| R-A2 | [#180](https://github.com/dwindlehop-pixel/hyades-4x/issues/180) | Autopilot formation and posture |
| R-A3 | [#181](https://github.com/dwindlehop-pixel/hyades-4x/issues/181) | Trade and non-aggression pacts in the verb model |
| R-N1 | [#182](https://github.com/dwindlehop-pixel/hyades-4x/issues/182) | Lock the six saga arcs as modes of love winning |
| R-N2 | [#183](https://github.com/dwindlehop-pixel/hyades-4x/issues/183) | Tier-crossing named events carrying the elimination drumbeat |

### Habitability — [`Hyades_habitability.md`](Hyades_habitability.md) · [`spec: habitability`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+habitability%22)

| Code | Issue | Question |
|---|---|---|
| R-H1 | [#184](https://github.com/dwindlehop-pixel/hyades-4x/issues/184) | Habitability starting distribution parameters (needs an MC sweep) |
| R-H2 | [#185](https://github.com/dwindlehop-pixel/hyades-4x/issues/185) | Should planet composition correlate spatially with habitable-zone position? |
| R-H3 | [#186](https://github.com/dwindlehop-pixel/hyades-4x/issues/186) | Confirm the signed habitable-zone offset reading (hot and cold as separate axes) |
| R-H4 | [#187](https://github.com/dwindlehop-pixel/hyades-4x/issues/187) | Smooth taper or hard cliff at the tolerance boundary |
| R-H5 | [#188](https://github.com/dwindlehop-pixel/hyades-4x/issues/188) | Confirm biosphere stays a separate factor from habitability |
| R-H6 | [#189](https://github.com/dwindlehop-pixel/hyades-4x/issues/189) | Confirm player-relative K is a wanted feature |
| R-H7 | [#190](https://github.com/dwindlehop-pixel/hyades-4x/issues/190) | Reframe gravity's Band sub-score as a population-health statistic |
| R-H8 | [#191](https://github.com/dwindlehop-pixel/hyades-4x/issues/191) | Reframe radiation's Band sub-score as a population-health statistic |

### Industry — [`Hyades_industry.md`](Hyades_industry.md) · [`spec: industry`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+industry%22)

| Code | Issue | Question |
|---|---|---|
| R-IND2 | [#192](https://github.com/dwindlehop-pixel/hyades-4x/issues/192) | Is warding a third allocation share, or a Design hardness coefficient? |
| R-IND3 | [#193](https://github.com/dwindlehop-pixel/hyades-4x/issues/193) | Works coefficients: Expansion's per-kt advantage, Production's ceiling, the crossovers |
| R-IND6 | [#194](https://github.com/dwindlehop-pixel/hyades-4x/issues/194) | Supers and apex raising the works ceiling |
| R-IND8 | [#195](https://github.com/dwindlehop-pixel/hyades-4x/issues/195) | What an empire inherits when it captures developed infrastructure |
| R-IND9 | [#196](https://github.com/dwindlehop-pixel/hyades-4x/issues/196) | The extraction tail past N(S): flat, or a shallow seam at floor grade? |
| R-IND19 | [#197](https://github.com/dwindlehop-pixel/hyades-4x/issues/197) | Rewrite or annotate §4.3's ε·S·W formula (the engine already uses W/N) |
| R-O90 | [#198](https://github.com/dwindlehop-pixel/hyades-4x/issues/198) | One freighter hold buys exactly one infrastructure whole Band: make it deliberate or remove the cliff? |
| R-IND22 | [#199](https://github.com/dwindlehop-pixel/hyades-4x/issues/199) | The development profitability test's three unknowns |
| R-IND18 | [#200](https://github.com/dwindlehop-pixel/hyades-4x/issues/200) | Magnitudes for the composed extraction law (ε, β, VEINS_PER_BAND, cap_ext, half_ext) |
| R-IND12 | [#201](https://github.com/dwindlehop-pixel/hyades-4x/issues/201) | How much a colonizer carries (model settled, magnitudes open) |
| R-IND13 | [#202](https://github.com/dwindlehop-pixel/hyades-4x/issues/202) | The works-value whole Band I |
| R-IND16 | [#203](https://github.com/dwindlehop-pixel/hyades-4x/issues/203) | How much color weight one deep Production card adds |
| R-IND14 | [#204](https://github.com/dwindlehop-pixel/hyades-4x/issues/204) | Travel discount: hyperbolic or exponential? |
| R-IND11 | [#205](https://github.com/dwindlehop-pixel/hyades-4x/issues/205) | Is a General colonizer ever worth it, now that the hold is the only thing separating the hulls? |

### Interface — [`Hyades_interface.md`](Hyades_interface.md) · [`spec: interface`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+interface%22)

| Code | Issue | Question |
|---|---|---|
| R-UI1 | [#207](https://github.com/dwindlehop-pixel/hyades-4x/issues/207) | The tactical palette |
| R-UI2 | [#208](https://github.com/dwindlehop-pixel/hyades-4x/issues/208) | Juicy mode's style, including high-density screens |
| R-UI3 | [#209](https://github.com/dwindlehop-pixel/hyades-4x/issues/209) | The glyph grammar (§6.3) |
| R-UI4 | [#210](https://github.com/dwindlehop-pixel/hyades-4x/issues/210) | Stacking and the tactical layer |
| R-UI6 | [#211](https://github.com/dwindlehop-pixel/hyades-4x/issues/211) | Replay size (§3) |
| R-UI7 | [#212](https://github.com/dwindlehop-pixel/hyades-4x/issues/212) | Showing height z in the galaxy view (§6.1) |
| R-UI8 | [#213](https://github.com/dwindlehop-pixel/hyades-4x/issues/213) | Which upstream packages the interface takes up now that it may (ruling 14) |
| R-UI9 | [#214](https://github.com/dwindlehop-pixel/hyades-4x/issues/214) | The client's phone layout and touch magnitudes |

### Loadout — [`Hyades_loadout.md`](Hyades_loadout.md) · [`spec: loadout`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+loadout%22)

| Code | Issue | Question |
|---|---|---|
| R-L0 | [#215](https://github.com/dwindlehop-pixel/hyades-4x/issues/215) | Concrete per-hull slot tables |
| R-L1 | [#216](https://github.com/dwindlehop-pixel/hyades-4x/issues/216) | Shield behavior with no in-combat rounds |
| R-L3 | [#217](https://github.com/dwindlehop-pixel/hyades-4x/issues/217) | Where unlocked-tier state lives: player component or resource |
| R-L4 | [#218](https://github.com/dwindlehop-pixel/hyades-4x/issues/218) | Do steering aids and hull size affect cruise acceleration, or only combat? |

### Matching (the Exchange) — [`Hyades_matching.md`](Hyades_matching.md) · [`spec: matching`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+matching%22)

| Code | Issue | Question |
|---|---|---|
| R-MX1 | [#219](https://github.com/dwindlehop-pixel/hyades-4x/issues/219) | Does market pressure ever surface diegetically? |
| R-MX2 | [#220](https://github.com/dwindlehop-pixel/hyades-4x/issues/220) | Market-tick cadence and per-fill light-lag |
| R-MX3 | [#221](https://github.com/dwindlehop-pixel/hyades-4x/issues/221) | Distance-discounted price versus pure price-then-distance |
| R-MX4 | [#222](https://github.com/dwindlehop-pixel/hyades-4x/issues/222) | Mining bid concurrency quantity |
| R-MX5 | [#223](https://github.com/dwindlehop-pixel/hyades-4x/issues/223) | One Book per (empire, commodity), or per-commodity global books |
| R-MX9 | [#224](https://github.com/dwindlehop-pixel/hyades-4x/issues/224) | A Mahan main fleet's quick intercept |
| R-MX10 | [#225](https://github.com/dwindlehop-pixel/hyades-4x/issues/225) | The room the Exchange leaves for cards |
| R-MX11 | [#226](https://github.com/dwindlehop-pixel/hyades-4x/issues/226) | Monopoly |
| R-MX12 | [#227](https://github.com/dwindlehop-pixel/hyades-4x/issues/227) | Monopsony |
| R-MX13 | [#228](https://github.com/dwindlehop-pixel/hyades-4x/issues/228) | Cartel |
| R-MX15 | [#229](https://github.com/dwindlehop-pixel/hyades-4x/issues/229) | The ordnance book's key and price |

### Mineral cost curve — [`Hyades_mineral_cost_curve.md`](Hyades_mineral_cost_curve.md) · [`spec: mineral cost curve`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+mineral+cost+curve%22)

| Code | Issue | Question |
|---|---|---|
| R-MC1 | [#230](https://github.com/dwindlehop-pixel/hyades-4x/issues/230) | Where each card or node sits on the mineral ratio continuum |
| R-MC2 | [#231](https://github.com/dwindlehop-pixel/hyades-4x/issues/231) | How much variance "Strange Matter ratio close to 4:2:1" permits |
| R-MC3a | [#232](https://github.com/dwindlehop-pixel/hyades-4x/issues/232) | Pin per-class semi-axis ratios that produce the η table |
| R-MC6 | [#233](https://github.com/dwindlehop-pixel/hyades-4x/issues/233) | Size the Kinetic/Potential/Latent group-level super premium |
| R-MC7 | [#234](https://github.com/dwindlehop-pixel/hyades-4x/issues/234) | Confirm Lanchester's square law as the multiplier-to-mass mechanism |
| R-MC8 | [#235](https://github.com/dwindlehop-pixel/hyades-4x/issues/235) | Which stats "effectiveness" reads off |
| R-MC11 | [#236](https://github.com/dwindlehop-pixel/hyades-4x/issues/236) | Hull shape as a function of loadout slot composition |
| R-MC14 | [#237](https://github.com/dwindlehop-pixel/hyades-4x/issues/237) | Arena sweep ranges for distance, velocity and N |

### Netcode — [`Hyades_netcode.md`](Hyades_netcode.md) · [`spec: netcode`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+netcode%22)

| Code | Issue | Question |
|---|---|---|
| R-NET4 | [#238](https://github.com/dwindlehop-pixel/hyades-4x/issues/238) | Frame payload field widths (card_id, target_kind/target_ref) |
| R-NET5 | [#239](https://github.com/dwindlehop-pixel/hyades-4x/issues/239) | Merkle leaf partition and checkpoint cadence |
| R-NET6 | [#240](https://github.com/dwindlehop-pixel/hyades-4x/issues/240) | Does a defaulted/timed-out round consume the action or refund it? |
| R-NET7 | [#241](https://github.com/dwindlehop-pixel/hyades-4x/issues/241) | Headless replay throughput versus match length; is snapshot-assisted catch-up needed? |
| R-NET8 | [#242](https://github.com/dwindlehop-pixel/hyades-4x/issues/242) | Even-split behavior under SimpleMajority |
| R-NET10 | [#243](https://github.com/dwindlehop-pixel/hyades-4x/issues/243) | Enable simd128 in the pinned build? |
| R-NET14 | [#244](https://github.com/dwindlehop-pixel/hyades-4x/issues/244) | Pin the circulant offset table in genesis |
| R-NET15 | [#245](https://github.com/dwindlehop-pixel/hyades-4x/issues/245) | Measure the state-digest cost at an 18-seat galaxy |
| R-NET16 | [#246](https://github.com/dwindlehop-pixel/hyades-4x/issues/246) | Match-start admission rule and straggler negotiation time |
| R-NET17 | [#247](https://github.com/dwindlehop-pixel/hyades-4x/issues/247) | Lobby-published liveness beacon as a second eclipse tell |
| R-NET18 | [#248](https://github.com/dwindlehop-pixel/hyades-4x/issues/248) | Ratify m_ingress (spectator links per seat) and spectator gossip degree d |

### Politics, trade and intelligence — [`Hyades_politics_trade_and_intelligence.md`](Hyades_politics_trade_and_intelligence.md) · [`spec: politics`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+politics%22)

| Code | Issue | Question |
|---|---|---|
| R-P2 | [#249](https://github.com/dwindlehop-pixel/hyades-4x/issues/249) | The base income rate, the Politics depth multiplier, and λ on a ten-seed bed |
| R-P5 | [#250](https://github.com/dwindlehop-pixel/hyades-4x/issues/250) | The event taxonomy admitting solicitation, and window_years |
| R-P7 | [#251](https://github.com/dwindlehop-pixel/hyades-4x/issues/251) | The per-tree counter list with costs — the tree's whole balance surface |
| R-P9 | [#252](https://github.com/dwindlehop-pixel/hyades-4x/issues/252) | Strength of both counter-graph effects; is the risk premium bounded? |
| R-P13 | [#253](https://github.com/dwindlehop-pixel/hyades-4x/issues/253) | Which stances may be written, by which tier, on which index |
| R-P14 | [#254](https://github.com/dwindlehop-pixel/hyades-4x/issues/254) | Does an imposed stance decay? |
| R-P15 | [#255](https://github.com/dwindlehop-pixel/hyades-4x/issues/255) | Does a two-referent Target widen target_ref past R-NET4? |

### Production tree — [`Hyades_production_tree.md`](Hyades_production_tree.md) · [`spec: production`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+production%22)

| Code | Issue | Question |
|---|---|---|
| R-PROD1 | [#256](https://github.com/dwindlehop-pixel/hyades-4x/issues/256) | Which Works fields each card tier writes |
| R-PROD2 | [#257](https://github.com/dwindlehop-pixel/hyades-4x/issues/257) | What a Production card may do to the hull ladder |
| R-PROD3 | [#258](https://github.com/dwindlehop-pixel/hyades-4x/issues/258) | Fleet-years' saturation point on the bed |
| R-PROD4 | [#259](https://github.com/dwindlehop-pixel/hyades-4x/issues/259) | Capitals and synthesis at pop-Band IV |

### Trees and card value — [`Hyades_trees_and_card_value.md`](Hyades_trees_and_card_value.md) · [`spec: trees`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+trees%22)

| Code | Issue | Question |
|---|---|---|
| R-TREE1 | [#260](https://github.com/dwindlehop-pixel/hyades-4x/issues/260) | The end-of-game chronicle |
| R-TREE2 | [#261](https://github.com/dwindlehop-pixel/hyades-4x/issues/261) | λ_w, Warfare's neighbor length scale, and the right proximity measure |
| R-TREE3 | [#262](https://github.com/dwindlehop-pixel/hyades-4x/issues/262) | Growth's interim stock until works exist |
| R-TREE5 | [#263](https://github.com/dwindlehop-pixel/hyades-4x/issues/263) | Politics' coupling κ, and whether delivered freight is the whole of φ_ij |
| R-TREE6 | [#264](https://github.com/dwindlehop-pixel/hyades-4x/issues/264) | Dispersion measure for the tier-1 constraint |
| R-TREE7 | [#265](https://github.com/dwindlehop-pixel/hyades-4x/issues/265) | Whether all six trees need the full horizon, or only those whose stock has not saturated. |
| R-TREE8 | [#266](https://github.com/dwindlehop-pixel/hyades-4x/issues/266) | Warfare needs an asymmetric bed |
| R-TREE9 | [#267](https://github.com/dwindlehop-pixel/hyades-4x/issues/267) | The composite objective |
| R-TREE10 | [#268](https://github.com/dwindlehop-pixel/hyades-4x/issues/268) | Re-measure the composite with Production in volume |
| R-TREE11 | [#269](https://github.com/dwindlehop-pixel/hyades-4x/issues/269) | The write-capability partition |
| R-TREE12 | [#270](https://github.com/dwindlehop-pixel/hyades-4x/issues/270) | Warfare card value: single highest-weight neighbor or weighted mean? |
| R-TREE13 | [#271](https://github.com/dwindlehop-pixel/hyades-4x/issues/271) | The voice table needs state Snapshot does not carry |

### Sessions, discovery and security — [`Hyades_sessions_discovery_and_security.md`](Hyades_sessions_discovery_and_security.md) · [`spec: sessions`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+sessions%22)

| Code | Issue | Question |
|---|---|---|
| R-SES1 | [#272](https://github.com/dwindlehop-pixel/hyades-4x/issues/272) | Nostr as the rendezvous; relay count; a second backend |
| R-SES2 | [#273](https://github.com/dwindlehop-pixel/hyades-4x/issues/273) | Identity: per-tab keys for casual play, persistent backed-up keys for roster play; name plus fingerprint |
| R-SES3 | [#274](https://github.com/dwindlehop-pixel/hyades-4x/issues/274) | No TURN; frames over the rendezvous as the last transport |
| R-SES5 | [#275](https://github.com/dwindlehop-pixel/hyades-4x/issues/275) | Link format, originator sequencing, spectator queue, rematch host and queue order |
| R-SES6 | [#276](https://github.com/dwindlehop-pixel/hyades-4x/issues/276) | Roster links: the seat table fixed in the link, a short watch link for chat |
| R-SES7 | [#277](https://github.com/dwindlehop-pixel/hyades-4x/issues/277) | Practice modes as a static table; room announcements; no ranking of the list |
| R-SES8 | [#278](https://github.com/dwindlehop-pixel/hyades-4x/issues/278) | Match record format and verification steps |
| R-SES9 | [#279](https://github.com/dwindlehop-pixel/hyades-4x/issues/279) | Light, unverified spectators |
| R-SES10 | [#280](https://github.com/dwindlehop-pixel/hyades-4x/issues/280) | The verifier's six verdicts |
| R-SES11 | [#281](https://github.com/dwindlehop-pixel/hyades-4x/issues/281) | A per-round inputs hash |
| R-SES12 | [#282](https://github.com/dwindlehop-pixel/hyades-4x/issues/282) | Ejected seats keep replaying and showing their roots |
| R-SES18 | [#283](https://github.com/dwindlehop-pixel/hyades-4x/issues/283) | Late timeout votes can make clients resolve a round differently (§11) |
| R-SES19 | [#284](https://github.com/dwindlehop-pixel/hyades-4x/issues/284) | The relay test's departures from this spec and netcode |
| R-SES13 | [#285](https://github.com/dwindlehop-pixel/hyades-4x/issues/285) | Retention of old engine builds on Pages |
| R-SES15 | [#286](https://github.com/dwindlehop-pixel/hyades-4x/issues/286) | The streamer threat model's six rules |
| R-SES16 | [#287](https://github.com/dwindlehop-pixel/hyades-4x/issues/287) | Relay-load magnitudes (the rules are ratified) |
| R-SES17 | [#288](https://github.com/dwindlehop-pixel/hyades-4x/issues/288) | Robustness to rate limits that change |

### Standing layer and observation — [`Hyades_standing_layer_and_observation.md`](Hyades_standing_layer_and_observation.md) · [`spec: standing layer`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+standing+layer%22)

| Code | Issue | Question |
|---|---|---|
| R-O64 | [#289](https://github.com/dwindlehop-pixel/hyades-4x/issues/289) | Roles §6's 0/1/2 cargo was a unit count, not a mass ladder |
| R-O65 | [#290](https://github.com/dwindlehop-pixel/hyades-4x/issues/290) | Flatten hull_thrust_to_mass's 1.2/1.1/1.0 Systems ladder? |
| R-O3 | [#291](https://github.com/dwindlehop-pixel/hyades-4x/issues/291) | Write the "intra-round ordering is immaterial" assumption into the contract |
| R-O6 | [#292](https://github.com/dwindlehop-pixel/hyades-4x/issues/292) | Is a pass observable? |
| R-O11 | [#293](https://github.com/dwindlehop-pixel/hyades-4x/issues/293) | Where the directed counter-cycle lives; write f(r) into contract §5 |
| R-O13 | [#294](https://github.com/dwindlehop-pixel/hyades-4x/issues/294) | Is a target part of the move, or a sub-decision? |
| R-O19 | [#295](https://github.com/dwindlehop-pixel/hyades-4x/issues/295) | Size the convexity of the σ→value curve |
| R-O21 | [#296](https://github.com/dwindlehop-pixel/hyades-4x/issues/296) | Fix the σ = T × concentration(spread) mapping |
| R-O22 | [#297](https://github.com/dwindlehop-pixel/hyades-4x/issues/297) | Tier-1 reach σ — a fourth spread point beyond Peak, or Peak at higher T? |
| R-O23 | [#298](https://github.com/dwindlehop-pixel/hyades-4x/issues/298) | σ_hi as a function of adjacency share 2/(N−1) |
| R-O24 | [#299](https://github.com/dwindlehop-pixel/hyades-4x/issues/299) | Actions/turn ramp as σ_hi rising with round |
| R-O27 | [#300](https://github.com/dwindlehop-pixel/hyades-4x/issues/300) | Doctrine has no diplomatic fields |
| R-O49b | [#301](https://github.com/dwindlehop-pixel/hyades-4x/issues/301) | Set k per ladder whole Band and m per Design card, arena-measured |
| R-O51 | [#302](https://github.com/dwindlehop-pixel/hyades-4x/issues/302) | Ratify the three-way σ decomposition |
| R-O52 | [#303](https://github.com/dwindlehop-pixel/hyades-4x/issues/303) | Concealment by offset: cost the combo that holds acceleration constant |
| R-O53 | [#304](https://github.com/dwindlehop-pixel/hyades-4x/issues/304) | The observable-channel enumeration |
| R-O54 | [#305](https://github.com/dwindlehop-pixel/hyades-4x/issues/305) | Is planetside development quieter at long range than a burn? |

### Technology tree — [`Hyades_technology_tree.md`](Hyades_technology_tree.md) · [`spec: technology`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+technology%22)

| Code | Issue | Question |
|---|---|---|
| R-TECH1 | [#306](https://github.com/dwindlehop-pixel/hyades-4x/issues/306) | The Technology objective is specified and not built |
| R-TECH2 | [#307](https://github.com/dwindlehop-pixel/hyades-4x/issues/307) | Does capability saturate on the bed? |
| R-TECH5 | [#308](https://github.com/dwindlehop-pixel/hyades-4x/issues/308) | Fit Bradley–Terry by maximum likelihood on the Elo scale (recommended) |
| R-TECH9 | [#309](https://github.com/dwindlehop-pixel/hyades-4x/issues/309) | Weight hulls by dry mass, not count or volume (recommended) |
| R-TECH11 | [#310](https://github.com/dwindlehop-pixel/hyades-4x/issues/310) | Equal-spend budget B = ten General hulls (recommended) |
| R-TECH12 | [#311](https://github.com/dwindlehop-pixel/hyades-4x/issues/311) | D_long, and a closing long-range bed |
| R-TECH15 | [#312](https://github.com/dwindlehop-pixel/hyades-4x/issues/312) | The rating table's storage and staleness stamp |
| R-TECH18 | [#313](https://github.com/dwindlehop-pixel/hyades-4x/issues/313) | An unlock reaches no build |
| R-TECH19 | [#314](https://github.com/dwindlehop-pixel/hyades-4x/issues/314) | The offensive beds' judge: dry mass holding the field |
| R-TECH3 | [#315](https://github.com/dwindlehop-pixel/hyades-4x/issues/315) | The card surface beyond UnlockDesign |
| R-TECH26 | [#316](https://github.com/dwindlehop-pixel/hyades-4x/issues/316) | Drawing energy from a neutron star (§7.1.3; galaxy R-G7, T-148) |
| R-TECH4 | [#317](https://github.com/dwindlehop-pixel/hyades-4x/issues/317) | A miniaturization analogue inside this tree? |
| R-TECH20 | [#318](https://github.com/dwindlehop-pixel/hyades-4x/issues/318) | Magnetar matter as a Technology option (§7.1.2) |
| R-TECH22 | [#319](https://github.com/dwindlehop-pixel/hyades-4x/issues/319) | The drive card's slot, slant and factor f |
| R-TECH24 | [#320](https://github.com/dwindlehop-pixel/hyades-4x/issues/320) | The missile card's slot, slant and sentry ratio |
| R-TECH25 | [#321](https://github.com/dwindlehop-pixel/hyades-4x/issues/321) | Which super dominates which tree |
| R-TECH23 | [#322](https://github.com/dwindlehop-pixel/hyades-4x/issues/322) | Where the drive state lives |

### Warfare tree — [`Hyades_warfare_tree.md`](Hyades_warfare_tree.md) · [`spec: warfare`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22spec%3A+warfare%22)

| Code | Issue | Question |
|---|---|---|
| R-IND20 | [#206](https://github.com/dwindlehop-pixel/hyades-4x/issues/206) | Should founding erect infrastructure from the hold at all? |
| R-WAR11 | [#323](https://github.com/dwindlehop-pixel/hyades-4x/issues/323) | May a Doctrine write impose a cost that scales with its holder's own activity? |
| R-WAR17 | [#324](https://github.com/dwindlehop-pixel/hyades-4x/issues/324) | Does a colony ship struck at its port forfeit its destination? |
| R-WAR14 | [#325](https://github.com/dwindlehop-pixel/hyades-4x/issues/325) | Sim::inert_card_plays counts NotYetImplemented only |
| R-WAR12 | [#326](https://github.com/dwindlehop-pixel/hyades-4x/issues/326) | The two interception guess magnitudes (cone radius, reassess period) |
| R-WAR8 | [#327](https://github.com/dwindlehop-pixel/hyades-4x/issues/327) | The two supply writes' magnitudes |
| R-WAR7 | [#328](https://github.com/dwindlehop-pixel/hyades-4x/issues/328) | A colonizer's hold is nearly all settlers: what share is erected as stock? |
| R-WAR6 | [#329](https://github.com/dwindlehop-pixel/hyades-4x/issues/329) | The denial magnitudes |
| R-WAR20 | [#330](https://github.com/dwindlehop-pixel/hyades-4x/issues/330) | The first Warfare card is below the author's 1.5–2.0x P92 target |
| R-WAR19 | [#331](https://github.com/dwindlehop-pixel/hyades-4x/issues/331) | The beam and structure magnitudes |
| R-WAR21 | [#332](https://github.com/dwindlehop-pixel/hyades-4x/issues/332) | A lone Limited picket cannot finish a Medium colony ship |
| R-WAR24 | [#333](https://github.com/dwindlehop-pixel/hyades-4x/issues/333) | The wreck curve's magnitudes |
| R-WAR27 | [#334](https://github.com/dwindlehop-pixel/hyades-4x/issues/334) | Accuracy per Design |
| R-WAR29 | [#335](https://github.com/dwindlehop-pixel/hyades-4x/issues/335) | The discharge period |
| R-WAR31 | [#336](https://github.com/dwindlehop-pixel/hyades-4x/issues/336) | Repair |
| R-WAR32 | [#337](https://github.com/dwindlehop-pixel/hyades-4x/issues/337) | The stored fleet |
| R-WAR35 | [#338](https://github.com/dwindlehop-pixel/hyades-4x/issues/338) | Belief event A reads a course change before its light arrives |
| R-WAR37 | [#339](https://github.com/dwindlehop-pixel/hyades-4x/issues/339) | Target priority under fire anywhere |
| R-WAR38 | [#340](https://github.com/dwindlehop-pixel/hyades-4x/issues/340) | The armed survey Design's structure |
| R-WAR39 | [#341](https://github.com/dwindlehop-pixel/hyades-4x/issues/341) | What a picket may believe about a battle it did not fight in |
| R-WAR40 | [#342](https://github.com/dwindlehop-pixel/hyades-4x/issues/342) | The Privateer card's slot and slant |
| R-WAR41 | [#343](https://github.com/dwindlehop-pixel/hyades-4x/issues/343) | Does $ change hands at the coerced price π_v? |
| R-WAR42 | [#344](https://github.com/dwindlehop-pixel/hyades-4x/issues/344) | The GCU Design: class name and hold, loadout and drive split |
| R-WAR43 | [#345](https://github.com/dwindlehop-pixel/hyades-4x/issues/345) | Belief about rival mineral production |
| R-WAR44 | [#346](https://github.com/dwindlehop-pixel/hyades-4x/issues/346) | The missile magnitudes |
| R-WAR48 | [#347](https://github.com/dwindlehop-pixel/hyades-4x/issues/347) | Whether a sentry withdraws from its own center |
| R-WAR33 | [#348](https://github.com/dwindlehop-pixel/hyades-4x/issues/348) | Which roles narrow a Design's engagement range |
| R-WAR1 | [#349](https://github.com/dwindlehop-pixel/hyades-4x/issues/349) | Elimination: nothing eliminates anyone |
| R-WAR2 | [#350](https://github.com/dwindlehop-pixel/hyades-4x/issues/350) | The card surface |
| R-WAR4 | [#351](https://github.com/dwindlehop-pixel/hyades-4x/issues/351) | The armed colonizer's magnitudes (drive fraction φ) |
| R-WAR3 | [#352](https://github.com/dwindlehop-pixel/hyades-4x/issues/352) | w_ij, the neighbor weight |

## Listed as open in a spec, not migrated

These codes still read as open in a spec register, but another part of `docs/`
or `AGENTS.md` records them as ratified, resolved or reduced to engine work, so
they got no decision issue. Each is a candidate for marking resolved in its
register.

| Code | Where it is listed | Why it has no decision issue |
|---|---|---|
| R-O9 | standing layer §13 | `AGENTS.md` §7, item 9: N=2's missing Green is accepted, not open |
| R-O12 | standing layer §13 | `AGENTS.md` §7, item 9: `FAIR_COUNTS` is `[2, 3, 6, 12, 18]` |
| R-O28 | standing layer §13 | `AGENTS.md` §7, item 2: the `Roster` component landed |
| R-O34 | standing layer §13 | design law #13 states L1 as a law |
| R-O47b | standing layer §13, technology register | design law #12 rules out retroactive refits; the build is T-08 ([#69](https://github.com/dwindlehop-pixel/hyades-4x/issues/69)) |
| R-O95 | standing layer §13 | answered by an identity; the remainder is R-WAR4 |
| R-NET11 | netcode §12 | design law #16 |
| R-NET13 | netcode §12 | design law #15; the audit it asks for is T-33 ([#107](https://github.com/dwindlehop-pixel/hyades-4x/issues/107)) and T-34 ([#108](https://github.com/dwindlehop-pixel/hyades-4x/issues/108)) |
| R-MX6 | matching §7 | design law #5 keeps `most_needed_center` permanently |
| R-L2 | loadout §7 | resolved by T-133 (repeated passes), warfare register |
| R-MC13 | mineral cost curve roundup | the arena exists (`src/arena.rs`) |
| R-O59 | production register | engine work only: T-03 ([#62](https://github.com/dwindlehop-pixel/hyades-4x/issues/62)) |
| R-O60 | technology register | engine work only: T-04 ([#63](https://github.com/dwindlehop-pixel/hyades-4x/issues/63)) |
| R-O40 | warfare register | engine work only: T-09 ([#70](https://github.com/dwindlehop-pixel/hyades-4x/issues/70)) |
| R-O31 | warfare register | engine work only: T-05 ([#64](https://github.com/dwindlehop-pixel/hyades-4x/issues/64)) |
| R-MC9c | warfare register | engine work only: T-12 ([#73](https://github.com/dwindlehop-pixel/hyades-4x/issues/73)) |

Rows a register already strikes through or marks resolved, ratified or closed
were not migrated either.
