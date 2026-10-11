# Hyades — experiments record

*The measurement record behind the specs. **Nothing here is normative.** A spec
says what the engine must do; this directory says what was run, on which bed, and
what it showed — including the runs that were wrong and the design that was
retracted. How it is organized, how to add to it and where the data lives:
[`AGENTS.md`](AGENTS.md).*

**Citing.** A spec cites an entry by its identifier — `appendix §D.24` — and the
identifier never changes when an entry moves between files. This table resolves
an identifier to its file.

| section | file | what it holds |
|---|---|---|
| §A | [`A-expansion-growth.md`](A-expansion-growth.md) | Expansion and Growth, moved out of `Hyades_autopilot_colonization_growth.md` at Rev 4 |
| §B | [`B-politics-trade.md`](B-politics-trade.md) | Politics, Trade and Intelligence, moved out of `Hyades_politics_trade_and_intelligence.md` at Rev 2 |
| §C | [`C-measurement-artifacts.md`](C-measurement-artifacts.md) | the seven shapes of measurement artifact, collected |
| §D | [`D-mechanisms/`](D-mechanisms/) | mechanisms from T-122 onward, one file per subject (below) |

## Table of contents

### [`A-expansion-growth.md`](A-expansion-growth.md)

- **§A.1** — R-AC3 — survey strategy has no measurable effect
- **§A.2** — R-AC20 — a sign conflict that was an artifact of two operating points
- **§A.3** — R-AC16 / `survey_reserve` — a threshold above the range of what it thresholds
- **§A.4** — R-O86 — both survey tests read the wrong quantity, and 99% of production built nothing
- **§A.5** — The mining knobs are measured and are nearly all noise
- **§A.6** — R-AC19 — mining-pair recycling, and three passes to measure it
- **§A.7** — R-AC17 — `k_high`, and "the snowball is the design"
- **§A.8** — Reach-limit — two limiters that bind in sequence
- **§A.9** — R-O66 — the unit fix cost 178 colonies, and the obvious explanation was wrong
- **§A.10** — R-O68 — a dead branch, proved three ways, and fixing it changed nothing
- **§A.11** — R-O87 — `reinvest_bias` cannot move Growth's objective, by identity
- **§A.12** — T-90 — a decision that was provably blind, with nothing to see
- **§A.13** — T-94 — what the Euler logistic was hiding, and what the sweep that found it was measuring
- **§A.14** — R-PROD5 — fleet-years in mass is provably indifferent to design law #3

### [`B-politics-trade.md`](B-politics-trade.md)

- **§B.1** — R-P2 — λ, and a trade mechanism that paid for itself before trade existed
- **§B.2** — The faucet and sink — four models, and why three were rejected
- **§B.3** — T-77 — settlement landed, and the screen that preceded it was wrong twice
- **§B.4** — The stage plan predicted the wrong risky stage
- **§B.5** — Colony-years is inverted as a guard for anything that changes how minerals are spent
- **§B.6** — R-P17 — the venue question was wrong, not merely unanswered
- **§B.7** — R-IND10 — a register entry that was already answered
- **§B.8** — R-P8 — a question dissolved by a better representation

### [`C-measurement-artifacts.md`](C-measurement-artifacts.md)

- **§C** — Cross-cutting: measurement artifacts, collected

### [`D-mechanisms/cards.md`](D-mechanisms/cards.md)

- **§D.1** — T-122 — isolating the constraints on the first Growth and Warfare cards
- **§D.2** — T-123 — the port strike: armed hulls meet colony ships where they launch
- **§D.3** — T-124 — a rung bill that destroyed mass, and the Growth card it was carrying
- **§D.4** — T-125 — weapons as a Design, the author's 1.5–2.0x target, and where each card stands
- **§D.9** — T-131 — the Technology objective: pricing a static per-role rating, and the proposal it replaces

### [`D-mechanisms/performance-and-determinism.md`](D-mechanisms/performance-and-determinism.md)

- **§D.5** — T-126 — release-binary throughput on the combat bed: compiler knobs, then the profile
- **§D.6** — T-127 — the host libm made native and wasm32 runs diverge; the engine's own transcendentals
- **§D.7** — T-129 — Band readings off the run path: static kilotons, and a reading without a logarithm
- **§D.8** — T-130 — `exp` and `ln` on the run path as four-multiply minimax polynomials

### [`D-mechanisms/combat.md`](D-mechanisms/combat.md)

- **§D.10** — T-132 — the damage model: beam power over time, structure on hull volume
- **§D.11** — T-133 — pricing an encounter before building one
- **§D.12** — T-133, second landing — named Designs, `σ` per Design class, the wreck roll and the pass
- **§D.13** — T-133, third landing — pricing fire as events on the main loop
- **§D.14** — T-133, fourth landing — the wreck point, and what fire control costs
- **§D.15** — T-133, fifth landing — engagement range from weapon accuracy
- **§D.16** — T-133, sixth landing — fire on the main loop, and no harness code in the engine
- **§D.17** — The first Design rating — fleets generated with the galaxy, and simultaneous fire
- **§D.18** — The role beds — generated mission fleets, and two faults they exposed
- **§D.19** — Every leg flies its Design's drive — `survey_accel_g` and `civilian_accel_g` removed
- **§D.24** — T-139 — missiles, point defense, sentries and the supply line
- **§D.29** — Combat bit-identity as three tests over six sets of initial conditions

### [`D-mechanisms/exchange-and-freight.md`](D-mechanisms/exchange-and-freight.md)

- **§D.20** — T-134 — the Exchange at a spatial equilibrium, the collection capacity it was missing, and one holding per (empire, planet)
- **§D.21** — T-134 stage 2 — the internal duty exchange, and self-trade
- **§D.22** — R-MX8 — a center's abundance hauled to a center with demand
- **§D.47** — A hauler priced against the shipping backlog
- **§D.53** — The voyage discount and the stops a leg may make, swept together
- **§D.54** — Freight routed by `$` at every stop: four arms, none landed
- **§D.55** — Why per-stop routing fell behind after 500 years: it never loaded at a center
- **§D.56** — A cheaper freight doctrine: a kept price table, netted as haulers commit

### [`D-mechanisms/supers-and-forging.md`](D-mechanisms/supers-and-forging.md)

- **§D.23** — T-138 — supers, apex, the refined Exchange, priced production, and a reachable Band IV
- **§D.25** — R-MX10 — synthesis and refined trade, confirmed in a run
- **§D.30** — Forging is a forge's purpose
- **§D.31** — Forges build super-billed Designs and bid for supers; decisions without a cadence
- **§D.34** — Demand for supers in the card-free bed
- **§D.35** — The twin bed: Designs paid in supers, built preferentially
- **§D.36** — Supply runs for supers, and the Growth card on the twin bed
- **§D.42** — The first forge's date: planted outposts, starting fleets, and the starting population
- **§D.46** — A forge's price falls with what it holds
- **§D.48** — Three defects the forge-price sweep surfaced
- **§D.49** — The forge's price, swept on the twin bed

### [`D-mechanisms/galaxy-and-color.md`](D-mechanisms/galaxy-and-color.md)

- **§D.26** — R-MX16 priced, the trio homeworld, and hex-scale color
- **§D.27** — Color theory: 1:1 recipe-pair sites, and slant by absolute threshold
- **§D.28** — A world's total ore is its richest color
- **§D.32** — Empire-scale color: the hex width, one color site per hex
- **§D.33** — Color sites placed at random: spacing and width swept
- **§D.41** — Color-centered homeworlds on three grounds

### [`D-mechanisms/spread-between-empires.md`](D-mechanisms/spread-between-empires.md)

- **§D.37** — Spread of colony count between empires
- **§D.38** — The cause of the spread between empires: an empty hauler's routing loop
- **§D.39** — Identical ground: what separates the seats, and pricing by an empire's color gaps
- **§D.40** — Holdings-based pricing, landed; the free upgrade at whole Band IV
- **§D.43** — Variation across empires in the tree stocks and in supers forged
- **§D.44** — Infrastructure bought in fractions of a Band
- **§D.45** — The color a saving center lacks was priced at zero
- **§D.50** — Why empires buy a different number of whole Band IV works
- **§D.51** — The color a nearly-paid center lacks is outbid by its own empire
- **§D.52** — Income per Band III center, and why a center short one color stays short
- **§D.57** — The spread of colony-years between empires on random ground

### [`D-mechanisms/interface.md`](D-mechanisms/interface.md)

- **§D.58** (with subsections §D.58.1–§D.58.8) — The replay viewer: frames, sizes, the palette's ink, and the GPU against the CPU
- **§D.59** — Hexes, the clipped galaxy, and a pan that moved glyphs (T-159 – T-163)

### [`D-mechanisms/sessions.md`](D-mechanisms/sessions.md)

- **§D.60** — Relay load against published relay limits (T-164, R-SES16)
- **§D.61** — The relay field test: protocol (T-169, R-SES16, R-SES17, R-SES18)

## References

- `AGENTS.md` §2 — how to search, how to read a gradient, the six traps, and the
  rule that sends entries here
- `docs/Hyades_autopilot_colonization_growth.md` — the Expansion/Growth spec §A
  supports
- `docs/Hyades_politics_trade_and_intelligence.md` — the Politics spec §B supports
- `docs/Hyades_industry.md` §6.8–6.26 — the industry branch's own experiment
  record, which has **not** been moved here yet and is the largest remaining
  instance of the problem this record solves
- `docs/hyades_todo.md` — the open register; an appendix entry is evidence, a
  todo entry is work
