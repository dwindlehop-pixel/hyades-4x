# Hyades — to-do overview

**The to-do register lives in GitHub issues.** This file is the overview: what
the bands mean, which labels carry them, and which issue each `T-nn` became.
The detail of an entry — its history, its measurements, what it is waiting on,
and the discussion — is in the issue, and nowhere else.

- All open to-do: [`label:todo`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22todo%22)
- Open decisions to ratify (R-codes): [`docs/hyades_decisions.md`](hyades_decisions.md)
- What was built before the move: [`docs/hyades_done.md`](hyades_done.md), a frozen archive

## Where the old register went

Each issue opens with a permalink to its entry in the markdown register at
commit `27e06ca`, and an entry longer than an issue comfortably holds keeps its
head and tail there, with the full text one click away. The register's header,
including the record of three `T-nn` collisions and why no grep could catch
them, is at [`docs/hyades_todo.md` @ `27e06ca`](https://github.com/dwindlehop-pixel/hyades-4x/blob/27e06ca881170dcb29fc6e9305006725130fa023/docs/hyades_todo.md#L1-L91).
Issue numbers retire that problem: GitHub assigns them, once.

## Bands

The list runs from *the change is known and only the work remains* down to *we
cannot yet say what this is*. A band is a claim about **how much design is
left**, not about priority. Read the band, then decide.

| Band | Label | Meaning | What is missing |
|---|---|---|---|
| **A** | [`band A: ready to build`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22band+A%3A+ready+to+build%22) | Ready to build | nothing — the change is specified |
| **B** | [`band B: needs a design pass`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22band+B%3A+needs+a+design+pass%22) | Decided, needs a design pass | the shape, not the direction |
| **C** | [`band C: open question`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22band+C%3A+open+question%22) | Open question with a concrete test | the answer, but you know how to get it |
| **D** | [`band D: direction only`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22band+D%3A+direction+only%22) | Direction only | the formulation |
| **E** | [`band E: parking lot`](https://github.com/dwindlehop-pixel/hyades-4x/issues?q=is%3Aissue+is%3Aopen+label%3A%22band+E%3A+parking+lot%22) | Parking lot | the ability to state the problem |

*(Unrelated to the "Band I–IV" magnitude-tier terminology in
`Hyades_mineral_cost_curve.md` §2.6. This is a lettered readiness scale for
work; that one is a ladder of quantities in game state.)*

## Rules

- **New work is a new issue.** Label it `todo` and one band label. It is
  identified by its issue number; **no new `T-nn` codes are assigned**, so the
  collisions this register suffered three times cannot recur.
- **The existing `T-nn` codes are permanent.** Each one is the prefix of its
  issue's title, and the table below maps it. Cite either form; prefer the issue
  link in new text.
- **Moving between bands is a label change**, with a comment saying what
  changed in how well the entry is understood.
- **Closing an entry is closing its issue** as completed, from the PR that lands
  it (`Closes #nn`), or as not planned with the reason. The PR body still lists
  the issues it closed, advanced and opened (`AGENTS.md` §6).
- **Dependencies are GitHub issue dependencies.** When an entry cannot start
  until another lands, mark it *blocked by* that issue in the issue's
  Relationships panel, and say why in the issue body. When two entries only
  interact, mention one in the other: GitHub then shows the link on both.
- **A decision that has to be ratified is not a to-do.** It is a decision issue
  (`decision: needs ratification`), listed in
  [`hyades_decisions.md`](hyades_decisions.md).

## Index

Issues are listed in the order the old register gave them, which ran from
specific to vague within each band. A grouped entry (`T-82 … T-86`) is one
issue covering a staged plan whose stages are tracked inside it.

### Band A — Ready to build

| Code | Issue | Title |
|---|---|---|
| T-147 | [#32](https://github.com/dwindlehop-pixel/hyades-4x/issues/32) | Bring the card-free spread between empires to about 20 colonies |
| T-146 | [#33](https://github.com/dwindlehop-pixel/hyades-4x/issues/33) | Make the hex presentation-only; give color its own spacing |
| T-145 | [#34](https://github.com/dwindlehop-pixel/hyades-4x/issues/34) | Raise the yomi of super and apex forging |
| T-144 | [#35](https://github.com/dwindlehop-pixel/hyades-4x/issues/35) | Decisions without a cadence: a short-circuit tree, conditioned on events |
| T-143 | [#36](https://github.com/dwindlehop-pixel/hyades-4x/issues/36) | Every empire forges every super — make forging a forge's purpose, then trade supers |
| T-141 | [#37](https://github.com/dwindlehop-pixel/hyades-4x/issues/37) | Decide whether spent-round debris should be cut |
| T-140 | [#38](https://github.com/dwindlehop-pixel/hyades-4x/issues/38) | Sweep the sentry ratio and its thresholds |
| T-139 | [#39](https://github.com/dwindlehop-pixel/hyades-4x/issues/39) | Supply lines, and the first missile Design |
| T-138 | [#40](https://github.com/dwindlehop-pixel/hyades-4x/issues/40) | Supers, apex and the refined Exchange; production priced |
| T-137 | [#41](https://github.com/dwindlehop-pixel/hyades-4x/issues/41) | Voice tables need played cards and relations in `Snapshot` |
| T-136 | [#42](https://github.com/dwindlehop-pixel/hyades-4x/issues/42) | The first Technology card — the drive |
| T-135 | [#43](https://github.com/dwindlehop-pixel/hyades-4x/issues/43) | The second Warfare card — Privateers |
| T-134 | [#44](https://github.com/dwindlehop-pixel/hyades-4x/issues/44) | The Exchange at a spatial equilibrium; one holding per (empire, planet) |
| T-131 | [#45](https://github.com/dwindlehop-pixel/hyades-4x/issues/45) | The Technology objective — a static rating of every Design, earned head-to-head per role |
| T-128 | [#46](https://github.com/dwindlehop-pixel/hyades-4x/issues/46) | A native-against-wasm32 parity check in CI |
| T-103 | [#47](https://github.com/dwindlehop-pixel/hyades-4x/issues/47) | The rest of the specs still carry their own experiment history |
| T-104 | [#48](https://github.com/dwindlehop-pixel/hyades-4x/issues/48) | Three trees have specs and two of them cannot be measured |
| T-107 | [#49](https://github.com/dwindlehop-pixel/hyades-4x/issues/49) | Population is not a factor of production — `Y` must read `P` |
| T-108 | [#50](https://github.com/dwindlehop-pixel/hyades-4x/issues/50) | Development is a profitability test — R-IND22 |
| T-95 | [#51](https://github.com/dwindlehop-pixel/hyades-4x/issues/51) | `cycle_years` is doing two jobs, and only one of them is integration |
| T-97 | [#52](https://github.com/dwindlehop-pixel/hyades-4x/issues/52) | `drive_volume_fraction` is a Design quantity in a config field |
| T-92 | [#53](https://github.com/dwindlehop-pixel/hyades-4x/issues/53) | A center's bank is 98.3% its own single-colored planet |
| T-93 | [#54](https://github.com/dwindlehop-pixel/hyades-4x/issues/54) | `next_pickup` scans every pile the player works |
| T-102 | [#55](https://github.com/dwindlehop-pixel/hyades-4x/issues/55) | The surviving 45.4 M `view_of` + `rank` calls |
| T-99 | [#56](https://github.com/dwindlehop-pixel/hyades-4x/issues/56) | The demand term is a ceiling, not this hauler's share |
| T-66 | [#57](https://github.com/dwindlehop-pixel/hyades-4x/issues/57) | Hauling is the engine's largest cost, and the scale is ratified (R-O82 closed) |
| T-82 … T-86 | [#58](https://github.com/dwindlehop-pixel/hyades-4x/issues/58) | The Exchange — `Hyades_politics_trade_and_intelligence.md` §10 |
| T-78 … T-80 | [#59](https://github.com/dwindlehop-pixel/hyades-4x/issues/59) | The trees: tone, six objectives, card value — `Hyades_trees_and_card_value.md` |
| T-67 … T-77 | [#60](https://github.com/dwindlehop-pixel/hyades-4x/issues/60) | The industrial layer — `Hyades_industry.md` |
| T-02 | [#61](https://github.com/dwindlehop-pixel/hyades-4x/issues/61) | ~~Restore `examples/bench_hex_size.rs`, sweeping entity count~~ — closed |
| T-03 | [#62](https://github.com/dwindlehop-pixel/hyades-4x/issues/62) | Slag as a bank entry — R-O59 |
| T-04 | [#63](https://github.com/dwindlehop-pixel/hyades-4x/issues/63) | Magazine mass on ordnance families — R-O60 / R-XM6 |
| T-05 | [#64](https://github.com/dwindlehop-pixel/hyades-4x/issues/64) | `min_time_search` as a reachability-cone query — R-O31 |
| T-06 | [#65](https://github.com/dwindlehop-pixel/hyades-4x/issues/65) | ~~Split three long functions~~ — closed |
| T-07 | [#66](https://github.com/dwindlehop-pixel/hyades-4x/issues/66) | Recalibrate `centrality_scale` |
| T-43 | [#67](https://github.com/dwindlehop-pixel/hyades-4x/issues/67) | The cost ladder can no longer be swept one leg at a time |

### Band B — Decided, needs a design pass

| Code | Issue | Title |
|---|---|---|
| T-142 | [#68](https://github.com/dwindlehop-pixel/hyades-4x/issues/68) | Revise the super names once Design is specified |
| T-08 | [#69](https://github.com/dwindlehop-pixel/hyades-4x/issues/69) | `on_refit` retrofit realization — R-O47b / R-O55 |
| T-09 | [#70](https://github.com/dwindlehop-pixel/hyades-4x/issues/70) | Throttle fraction on `Doctrine`; observe `a` from trajectory — R-O40 |
| T-10 | [#71](https://github.com/dwindlehop-pixel/hyades-4x/issues/71) | Route intercept and accept/decline through *believed* `a_max` — R-O41 |
| T-11 | [#72](https://github.com/dwindlehop-pixel/hyades-4x/issues/72) | Diplomatic fields on `Doctrine` — R-O27 / R-A3 |
| T-12 | [#73](https://github.com/dwindlehop-pixel/hyades-4x/issues/73) | R-MC9c — layer HP, weapon count, AoE and magazines onto combat |
| T-13 | [#74](https://github.com/dwindlehop-pixel/hyades-4x/issues/74) | Counter-graph matrix partition — card contract §8.3 |
| T-14 | [#75](https://github.com/dwindlehop-pixel/hyades-4x/issues/75) | R-SIM1 — light survey view |
| T-30 | [#76](https://github.com/dwindlehop-pixel/hyades-4x/issues/76) | The round/command layer — netcode B1 |
| T-31 | [#77](https://github.com/dwindlehop-pixel/hyades-4x/issues/77) | A card system — netcode B2 |
| T-32 | [#78](https://github.com/dwindlehop-pixel/hyades-4x/issues/78) | The state digest — netcode B3 |
| T-42 | [#79](https://github.com/dwindlehop-pixel/hyades-4x/issues/79) | Hidden simultaneous orders — commit/reveal in the engine |
| T-38 | [#80](https://github.com/dwindlehop-pixel/hyades-4x/issues/80) | The Exchange, `$`, and the trade economy |
| T-39 | [#81](https://github.com/dwindlehop-pixel/hyades-4x/issues/81) | Shared intelligence, tiers 1–3 |
| T-40 | [#82](https://github.com/dwindlehop-pixel/hyades-4x/issues/82) | The trade ↔ mobilization counter-graph coupling |
| T-15 | [#83](https://github.com/dwindlehop-pixel/hyades-4x/issues/83) | Production-queue redesign — roles §10, deferred |
| T-164 | [#84](https://github.com/dwindlehop-pixel/hyades-4x/issues/84) | Sessions, discovery and security — ratify the spec |
| T-165 | [#85](https://github.com/dwindlehop-pixel/hyades-4x/issues/85) | The engine as a browser module, and the transcript verifier |
| T-167 | [#86](https://github.com/dwindlehop-pixel/hyades-4x/issues/86) | The rendezvous layer and the link flow |

### Band C — Open question with a concrete test

| Code | Issue | Title |
|---|---|---|
| T-169 | [#87](https://github.com/dwindlehop-pixel/hyades-4x/issues/87) | Run the relay field test |
| T-64 | [#88](https://github.com/dwindlehop-pixel/hyades-4x/issues/88) | One quantity type; the logistic runs on people (R-O83 closed) |
| T-63 | [#89](https://github.com/dwindlehop-pixel/hyades-4x/issues/89) | `Band Empty` is the ladder's floor, one metric tonne wide (R-MC15 amended) |
| T-62 | [#90](https://github.com/dwindlehop-pixel/hyades-4x/issues/90) | Mineral generation is Gaussian over Bands (R-O81 closed) |
| T-61 | [#91](https://github.com/dwindlehop-pixel/hyades-4x/issues/91) | The infrastructure ladder is the mineral ladder (R-O80 closed) |
| T-60 | [#92](https://github.com/dwindlehop-pixel/hyades-4x/issues/92) | The founding subsidy is removed (R-O77 closed), and what it broke |
| T-58 | [#93](https://github.com/dwindlehop-pixel/hyades-4x/issues/93) | Band arithmetic removed from the type, and the two bugs it was hiding |
| T-59 | [#94](https://github.com/dwindlehop-pixel/hyades-4x/issues/94) | Mining crews and outpost ore are per player |
| T-57 | [#95](https://github.com/dwindlehop-pixel/hyades-4x/issues/95) | How many miners per mining outpost? — the term now exists |
| T-56 | [#96](https://github.com/dwindlehop-pixel/hyades-4x/issues/96) | R-MC3a / R-MC3b — hull geometry with units, and the ladder that makes General hulls worth building |
| T-55 | [#97](https://github.com/dwindlehop-pixel/hyades-4x/issues/97) | R-O73 — the three F ladders, the quantity survey, and what to ratify |
| T-54 | [#98](https://github.com/dwindlehop-pixel/hyades-4x/issues/98) | R-O72 — the Band→kiloton bridge is wrong in two structural ways, and the scale is off by 10⁴–10⁵ |
| T-52 | [#99](https://github.com/dwindlehop-pixel/hyades-4x/issues/99) | R-O69 follow-ups — the hostile-interrupt trigger, and the decision path's O(galaxy) scan |
| T-53 | [#100](https://github.com/dwindlehop-pixel/hyades-4x/issues/100) | R-O71 — the cargo ladder is geometric where §2.6 says it must be Banded |
| T-51 | [#101](https://github.com/dwindlehop-pixel/hyades-4x/issues/101) *(closed)* | R-O68 — the deepen/expand trade does not exist, and it is what sets the expansion-loop time constant |
| T-89 | [#102](https://github.com/dwindlehop-pixel/hyades-4x/issues/102) | R-O85 — infrastructure is priced as if it were the scarce thing |
| T-50 | [#103](https://github.com/dwindlehop-pixel/hyades-4x/issues/103) | Record gradient sensitivity as raw data, not prose — it is card-design input |
| T-65 | [#104](https://github.com/dwindlehop-pixel/hyades-4x/issues/104) | R-O67 — population stalls when it runs out of biomass; it never dies back |
| T-47 | [#105](https://github.com/dwindlehop-pixel/hyades-4x/issues/105) *(closed)* | R-AC20 — time-to-10% vs. coverage disagree on `medium_fleet_size` |
| T-48 | [#106](https://github.com/dwindlehop-pixel/hyades-4x/issues/106) *(closed)* | Finish the metric change — five harnesses still divide |
| T-33 | [#107](https://github.com/dwindlehop-pixel/hyades-4x/issues/107) | `Knowledge` stores membership, not observations — netcode B4 |
| T-34 | [#108](https://github.com/dwindlehop-pixel/hyades-4x/issues/108) | Colonization filters on instantaneous global ownership — netcode B5 |
| T-41 | [#109](https://github.com/dwindlehop-pixel/hyades-4x/issues/109) | Card flavour, names, and the slant cost ratios — R-P11 |
| T-44 | [#110](https://github.com/dwindlehop-pixel/hyades-4x/issues/110) | Confirm `trade_decay_lambda` on the ten-seed bed |
| T-45 | [#111](https://github.com/dwindlehop-pixel/hyades-4x/issues/111) | Elasticity baseline — what the knobs actually do |
| T-16 | [#112](https://github.com/dwindlehop-pixel/hyades-4x/issues/112) | R-O63 — the biosphere regrowth magnitude |
| T-17 | [#113](https://github.com/dwindlehop-pixel/hyades-4x/issues/113) | R-O65 — should `hull_thrust_to_mass` be flat within a family? |
| T-18 | [#114](https://github.com/dwindlehop-pixel/hyades-4x/issues/114) | R-O64 — confirm the reinterpretation of roles §6's cargo ladder |
| T-19 | [#115](https://github.com/dwindlehop-pixel/hyades-4x/issues/115) | Does the 1 : 2.2 : 4 radius ladder beat 1 : 3 : 9? |
| T-20 | [#116](https://github.com/dwindlehop-pixel/hyades-4x/issues/116) | Raise coverage inside a fixed 4,000-year run |
| T-49 | [#117](https://github.com/dwindlehop-pixel/hyades-4x/issues/117) | What is left on the mining surface after R-AC19 |
| T-21 | [#118](https://github.com/dwindlehop-pixel/hyades-4x/issues/118) | R-SIM2 — survey scan cost |
| T-22 | [#119](https://github.com/dwindlehop-pixel/hyades-4x/issues/119) | R-ARENA2, 3, 4, 6, 7 — arena calibration placeholders |
| T-23 | [#120](https://github.com/dwindlehop-pixel/hyades-4x/issues/120) | R-MX1–5 — Exchange design calls |
| T-35 | [#121](https://github.com/dwindlehop-pixel/hyades-4x/issues/121) | Netcode ratification points with concrete tests |
| T-36 | [#122](https://github.com/dwindlehop-pixel/hyades-4x/issues/122) | Netcode policy calls needing a decision, not a measurement |
| T-24 | [#123](https://github.com/dwindlehop-pixel/hyades-4x/issues/123) | Throughput watch — the 12-seat × 8-kyr corner is unmeasured |
| T-166 | [#124](https://github.com/dwindlehop-pixel/hyades-4x/issues/124) | wasm32 throughput at 18 seats, on a phone and a laptop |

### Band D — Direction only

| Code | Issue | Title |
|---|---|---|
| T-25 | [#125](https://github.com/dwindlehop-pixel/hyades-4x/issues/125) | Enforce the starting roster — R-O42, blocked on cards |
| T-46 | [#126](https://github.com/dwindlehop-pixel/hyades-4x/issues/126) | Habitability's gravity/radiation Bands as population-health statistics — R-H7/R-H8 |
| T-37 | [#127](https://github.com/dwindlehop-pixel/hyades-4x/issues/127) | Everything in netcode outside the crate |
| T-26 | [#128](https://github.com/dwindlehop-pixel/hyades-4x/issues/128) | R-SIM4 — departure-traffic confidence |
| T-27 | [#129](https://github.com/dwindlehop-pixel/hyades-4x/issues/129) | R-XM5, 6, 7 — exotic matter |
| T-28 | [#130](https://github.com/dwindlehop-pixel/hyades-4x/issues/130) | R-ARENA1 and R-ARENA5 — cited, undefined |
| T-148 | [#131](https://github.com/dwindlehop-pixel/hyades-4x/issues/131) | A neutron star as the third star of a system |

### Band E — Parking lot

| Code | Issue | Title |
|---|---|---|
| T-29 | [#132](https://github.com/dwindlehop-pixel/hyades-4x/issues/132) | Nonconsensual role change — a "bidding system" |
