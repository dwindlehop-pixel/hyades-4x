# Hyades — Galaxy Generation, Materials, Population & Autopilot
*Companion to `Hyades_simulation_model.md` (sim §) and `Hyades_command_cards.md` (cmd §). **Rev 8** — adds the **information model** (omniscient command view over a fog-of-war simulation: scanning, stealth, light-lag) and points the Explore/Expand/Exploit defaults at the first detailed autopilot, `Hyades_autopilot_colonization_growth.md`. Warfare is in the **Hollywood-Western** register (Those Who Stand → … → the *believed* Beloved Republic, a win-state not a win-object). The **card design contract** lives in `Hyades_card_contract.md`. Carries rev 6's love-wins thesis, rev 5's warm-hard-SF voice, rev 4's color swap, rev 3's super-aligned homeworlds, specific-color costs. Grounds `hyades_opening_actions_r1.json`. Calls flagged **R-Gn/R-Mn/R-Pn/R-An/R-Nn/R-Cn**; carry-overs **R-7/R-9**.*

---

## 1. Two-view model — 3D theater, 2D command

The **theater is 3D** (X, Y, Z): movement, combat, encirclement, and retreat happen in a volume, so heading/velocity/formation/encirclement (sim §6) become **3-vectors** and egress can be cut in three dimensions. The **command view is 2D** (X, Y): the strategic map is the flat hex tiling, Z collapsed. Plan in 2D, render in 3D. **R-G0 / sim cross-ref:** sim §1/§6 absorb this — encirclement and retreat are volumetric.

**Hex dimensions — `RATIFIED` in part (the author's rulings; R-G1).** A hex is a flat-top hexagon **70 ly a side, 121.2 ly across flats** (`GalaxyConfig::hex_side_ly`), one centered on the galactic center; light crosses one in about 120 yr. **Color varies at the scale of an empire** (§4.3), so an empire spans a modest integer number of hexes, with the target per seat count: **3–6 hexes per player at 3 seats, 6–12 at 6 and 12, 3–6 at 18**, and the side a human-scale number (about 70 ly a side, about 120 ly across). Counted as the hexes holding 90% of the worlds an empire owns at 1,500 yr, no one width meets all four targets; the side is the author's choice near the width that minimizes the worst miss (116 ly), and 6 seats read below their target at it (**R-G5, open**; appendix §D.32). **The hex is a human-legible interface and nothing generated or simulated reads it** (the author's ruling) — changing it changes no run (`the_hex_is_read_by_no_generation`); ore color varies at the scale of the color sites (§4.3), a separate pair of parameters. **The star field's extent is sized by a separate 10-ly ring step** (`GalaxyConfig::ring_step_ly`, the previous hex side), so resizing the hex moved no planet and changed no planet count; the field's vertical scale length is 3 ring steps, 30 ly. **R-G1 remainder (open):** the command view's prism depth. *Superseded:* side `s ∈ [50, 250] ly` with depth `1×–5× s`; then a 10-ly side chosen against throughput, at which every empire's reach held every color (appendix §D.32).

---

## 2. Hex geometry — fair player counts fall out of symmetry

Homeworlds seed in **adjacent hexes forming a vertex-transitive cluster** where every player borders the **same number of other homeworlds**:

| N | Config | Neighbors each | Neutral core | Fair |
|---|---|---|---|---|
| **2** | domino | 1 | — | yes |
| **3** | tri-hex **clique** | 2 | shared vertex | yes (tightest) |
| **6** | **ring** | 2 | center hex | yes |
| **6r** (12, 18) | radius-`r` ring | 2 | inner region | yes |
| **4, 5, 7** | — | — | — | **no symmetric config** |

Cliques cap at 3, rings come in 6s, 4/5/7 have no equal-adjacency arrangement. A radius-`r` ring holds exactly **6r** cells, so the ring family is **6, 12, 18, 24, …** — note that 9 and 15 are multiples of 3 but form *no* ring, so a `% 3` rule would be the wrong predicate. The **neutral core** is equidistant from all — early contested space.

**Balance targets the 2-neighbor configurations — 3, 6, 12, 18 — where every seat borders exactly two others.** **N=2 is supported but is not a balance target** (ratified): the domino gives each player *one* neighbor, and the `p % 3` archetype cycle leaves it with Blue and Red and no Green. Both are accepted consequences of a configuration nothing is tuned around — which **resolves R-O9** as "known and accepted" rather than as a defect to fix.

**R-O12 (resolved): `Galaxy::FAIR_COUNTS` is `[2, 3, 6, 12, 18]`.** It had been truncated at 12 while `starting_hex_radius` already carried an `18 => 4.5` branch; all three of its ring radii are exactly `N/6 + 1.5`, so that branch was the third term of the family rather than a stray, and the list was simply one term short. The engine now expresses the family as that closed form instead of three magic numbers. **R-G2:** core contents (supported counts now settled).

**Spread between empires — the author's target, `RATIFIED`: card-free, the
standard deviation of colony count between the empires of one game is about
20 colonies; more would not be fun.** Measured at 1,500 yr on 12 standard
3-seat galaxies: median 190, mean 223 (appendix §D.37). **`OPEN` (T-147):**
how to meet it. The runaway was an empty hauler's routing loop (appendix
§D.38); with freight routed by demand price, `GalaxyConfig::ground` selects
among three kinds of ground, all built, `Ground::Random` the default:

| `Ground` | what each seat starts on | spread, archetype weight (§D.38) | spread, holdings-based pricing (§D.40) |
|---|---|---|---|
| `Random` | one field over the whole disk | 73.5 | 60.1 |
| `Identical` | one wedge of the disk turned to every seat, same colors | 48.0 | 24.3 |
| `ColorRotated` | the wedge with its colors stepped once per seat, as the archetypes step | 19.3 | 31.7 |

Mean spread at 1,500 yr over 12 galaxies. Holdings-based pricing is ratified
(autopilot spec §3.9); under it no ground meets the target with trio
homeworlds.

---

## 3. Homeworlds — super-aligned, identical in shape, equitable but unequal

Identical in shape (fairness): **Band 4.2 / Band 4.2 / Band I** (hab/bio/infra; §3.0 below) → `K = min = 1`; a **small town** that matures by building **infra Band I → Band IV** (§5; the Band ladder is defined generally in `Hyades_mineral_cost_curve.md` §2.6). **Rich in two tier-1 colors, poor in the third** — the two precursors of **one super** — so **mining income is imbalanced** and three rotationally-symmetric archetypes result (reflecting the §7 color swap):

| Homeworld | Rich basics | Poor | Native super @ pop Band IV | Cheap tree-pairs | Expensive pair |
|---|---|---|---|---|---|
| **Blue-type** | Cyan, Magenta | Yellow | **Blue** (C+M) | Expansion/Growth · Warfare/Technology | Production/Politics |
| **Red-type** | Magenta, Yellow | Cyan | **Red** (M+Y) | Warfare/Technology · Production/Politics | Expansion/Growth |
| **Green-type** | Yellow, Cyan | Magenta | **Green** (Y+C) | Production/Politics · Expansion/Growth | Warfare/Technology |

Same **2-rich-1-poor shape** rotated → **equal total wealth, color-shifted**. The start is **equitable but unequal**: 4 trees cheap, 2 expensive, your native super fixed. Blue-type is the militarist-expander-technologist (weak economy/diplomacy); Red-type the tall industrial-military-political power (weak at spreading); Green-type the economic-expansionist (weak on the whole war-tech axis). **N=3:** one of each. **N=6:** B-R-G alternating. **R-G3:** placement per table.

**3.0 `RATIFIED` (the author's ruling) — a homeworld's ceiling is `Band 4.2`.**
Habitability and pristine biosphere both read `Band 4.2` (`GalaxyConfig::homeworld_ceiling`),
a little past `Band IV`. At `4.0` the ceiling *was* the top population edge, and
the logistic approaches its ceiling without reaching it, so no world could ever
hold population `Band IV` and synthesis (§4.5) could never run. The target is a
schedule: a growth-dedicated build reaches population `Band IV` before round
two's card selection, most builds by round three, a high-tempo proactive build
around round four. Measured on seeds 1, 7, 42, 31337: card-free homeworlds cross
at 632–685 yr (round two is at 600 yr, round three at 1,000), and a Growth card at
the first barrier brings its homeworld across at 473–482 yr. No build in the
engine drains a homeworld faster than the default, so the round-four case is not
yet measurable. Appendix §D.23. **Placeholder** magnitude; the schedule is the
ruling.

**`RATIFIED` (the author's ruling) — a homeworld is a trio.** The habitable world, where the seat's population grows and its forge will stand, holds only a trace of every basic; beside it, one companion world is rich in each of the archetype's two rich basics. **Every forge's precursors therefore arrive by freight** (§4.5). Built: companions at `Band 3.0` in their one color, trace in the others, habitability and biosphere `Band 0.5`, `2 ly` from the homeworld on either side along the ring (`GalaxyConfig::homeworld_companion_*`, all **placeholders**); they start wild, and the opening's outposts take them. Card-free, 4 seeds, 1,500 yr: colony-years and colonies unchanged within two standard errors; the first colony comes later (appendix §D.26).

**Eventual self-synthesis.** At **pop Band IV** a homeworld synthesizes its archetype's super from its trio's two rich basics, which are that super's recipe. **R-G4 — resolved (the author's ruling: "forges have to produce supers according to color theory").** "Exactly one super" is a gradient, not a rule: a forge makes any super whose two basics it holds 1:1 (§4.2), and an archetype is native to one super only in that its trio supplies that recipe. Measured with the trio, hex-scale color, the ore cap and a priced works bill: **0.27% of refined mass synthesized crosses between empires** (24 runs, appendix §D.28) — empires make their own supers from basics their own freight brings. With forging a forge's purpose and forges bidding for supers (§4.5), card-free: every forge still makes all three supers, and 3.7% of supers forged cross between empires (appendix §D.31).

---

### 3.1 Fleets generated with the galaxy (T-133 follow-up) — `RATIFIED`

*Author's ruling:* "fleets can be optionally generated at Galaxy generation,
with a position and velocity. Equal cost mineral spend per fleet makes sense."
`Galaxy::generate_with(config, FleetSeeding)`: one spend for every fleet, and per
fleet a seat, a Design `(hull, class)`, a role, a position and a velocity (`< c`).
The engine builds `round(spend / dry mass)` unpaid hulls of each at bootstrap.
None by default, so an ordinary game is unchanged; this is how a test bed puts
fleets face to face while varying nothing but the galaxy (Technology §4.4.5).

**A generated hull goes to work through the engine's own launchers** —
`RATIFIED` as mechanism, magnitudes placeholders:

- `known_radius_ly` (ly; 0 by default) — a **surveyed start**: each seat has
  scanned every world within that radius of its homeworld at `t = 0`, so a
  colonizer or miner fleet has somewhere to go.
- A **Scout** hull launches a survey from its home port; a **Colonizer** hull
  flies to the nearest scanned world it can seed, one hull to a world; a
  **Miner** hull flies to a scanned world its seat's own autopilot ranks a
  mining outpost, nearest first, one to a rock before any rock takes two; a
  **Freighter** hull serves the seat's generated mining sites in turn.
- A **Picket** or **Reserve** hull stands at the fleet's position with its
  velocity; a **Sentry** hull stands at its seat's homeworld, counted there as
  the center's sentries (T-139), whatever position the fleet names; a mission
  hull with nowhere to go stands in Reserve at its home port.
- `twin_bill` (none by default) — **twin Designs paid in supers** (the author's
  direction for a bed in which supers have a final demand): every seat starts
  with a twin of every Design, the same hull, class, mass and stats, billed
  `twin_bill` (`Roster::twin`). A yard pays the twin's bill wherever it holds,
  or as a forge can make, the supers it owes for the whole order, and the
  Design's own bill otherwise. A center keeps wanting the twin of the hull
  Design it last chose — recorded as its standing order, so freight and the
  Exchange bid for it — until it can pay one. At a forge a Design counts as
  paid in supers only while its twin is payable, so R-MX17's pricing holds.
  `BUILT` for beds; appendix §D.35.
- **A generated missile Design starts with its magazine full** — its rounds
  are generated with it, as its hulls are, and drawn from no bank. A generated
  picket has no post and no voyage, so once dry it is not resupplied: the
  ammo run and the flight home both start from one.
- Every generated hull is logged once, `FleetGenerated`, with its seat, Design
  and role, which is how a bed tells its fleet from what the autopilot builds.

## 4. Materials — the ladder, the color algebra, and synthesis

### 4.1 Three tiers
**Tier 1 — basics: Cyan, Magenta, Yellow.** Mined (§4.3). · **Tier 2 — supers: Red, Green, Blue.** Synthesized, never mined. · **Apex — Strange Matter** (`RATIFIED`, the author's ruling). Synthesized from supers; metallic silver-white.

**Names and fiction (R-M1).** Basics are ordinary matter changed by extreme pressure (`Exotic_matter_technology_inspiration.md`, "materials at high pressure"): stable only at depth or at an impact site, and mined from there. Supers are categories of far-future technology built on exotic matter. The color stays the mechanical name in the engine (`resources::Material`); these are the in-game names.

| Tier | Color | Material (flavor) | Status |
|---|---|---|---|
| Basic | Cyan | **Cage Ice** — *Clathrate Hydride*: hydrogen caged in a metal lattice at core pressure; the hydrogen is propellant and the lattice is light structure | `RATIFIED` |
| Basic | Magenta | **Rosepeter** — *Erbium Polynitride*: single-bonded nitrogen above ~110 GPa, doped with erbium; the bond energy feeds weapons and power, the erbium lases | `RATIFIED` |
| Basic | Yellow | **Voltslate** — *Sodium Subchloride*: Na₃Cl, stable only under pressure; alternating sodium (conductor) and salt (insulator) layers are a circuit in the crystal | `RATIFIED` |
| Super | Blue ← C+M | **Countermass** — negative mass made by pair production (§3.5 of `Hyades_technology_tree.md`); keels and drives | placeholder (T-142) |
| Super | Red ← M+Y | **Ambient Superconductor** — an electron-pair condensate held at ordinary pressure; power, magnets, computation | placeholder (T-142) |
| Super | Green ← Y+C | **Metamaterial** — conducting layers on a cage lattice; sensors, signature control, communications | placeholder (T-142) |
| Apex | — | **Strange Matter** — converts ordinary matter it touches into itself; spent only on win conditions | `RATIFIED` |

### 4.2 Color algebra + the refining ladder
Fixed two-basic recipes: **Blue ← Cyan + Magenta · Red ← Magenta + Yellow · Green ← Yellow + Cyan.** **`RATIFIED` (the author's ruling) — a forge produces supers by color theory: the two basics of a super's recipe, in a 1:1 ratio by mass,** and nothing else makes it. A forge makes whichever super its holding has both basics for; no archetype is barred from a super (R-G4, below). **Apex ← Red + Green + Blue in equal parts** (`RATIFIED`, the author's ruling — silver-white is additive white). Ladder **3 basics → 2 supers → 1 apex** with **wastage** (cards reduce it) — named as yield fractions
**`Y_super` = 2/3** (mineral-mass → super-mass) and **`Y_apex` = 1/2**
(super-mass → apex-mass) in `Hyades_mineral_cost_curve.md` §5.0, which also
distinguishes this literal mass-conserving ratio from the separate
order-of-magnitude *value* heuristic ("3 CMY = 2 RGB = 1 Strange Matter," §5.1 of
that spec) the two used to share notation with. **No direct substitution.**
**R-M2:** ratios + wastage — this section owns the ratified values;
`Hyades_mineral_cost_curve.md` §5.0 only names them.

### 4.3 Tier-1 distribution — 3D field, XY-dominant, **Gaussian over Bands** (T-62)
**Gaussian in X & Y** (each hue's hotspot) **× exponential decay in Z from the midplane**. **`RATIFIED` (the author's ruling) — color varies at the scale of an empire.** The three hotspots set where each hue is strong across the galaxy; inside that envelope the ore sits at **color sites, placed at random** — one per `color_site_spacing_ly`² of area, independent of the hex — each a single hue drawn in proportion to the hotspots' weights at the site. A site's peak is `mineral_peak · (floor + (1 − floor) · w / w_max)`, `w` its hue's weight and `w_max` the largest weight among that hue's sites, so **each hue's strongest site is `Band IV`** (R-O82) wherever the lattice falls relative to its hotspot — without it, the one site that landed nearest its hotspot held most of a 3-seat galaxy's ore in one hue (appendix §D.32). A world's deposit in a hue is the Gaussian, of width `color_site_sigma_ly`, of its strongest site of that hue. One noise draw per world is added to every color, so the noise does not reorder a world's colors. Spacing `10 ly`, width `5 ly`, floor `0.5` — **placeholders** (`GalaxyConfig::color_site_*`); **`RATIFIED` (the author's ruling): at least 5% of worlds hold two colors** (a color counts at `Band I`, 1 kt); the shipped placeholders give 4.20% and do not yet meet it. **OPEN:** the spacing and width that put color at the scale of an empire within that floor, measured against trade in supers and the balance they move (appendix §D.33). *Superseded:* sites on a jittered lattice one 10-ly hex side apart ("a hex has a distinct slant or two"), at which each empire's reach held every color (appendix §D.26); then one site per 70-ly hex, which made the hex a generation parameter (§D.32); sites carrying a recipe pair 1:1 — the author: 1:1 is wrong for galaxy generation (appendix §D.27).

**`RATIFIED` (the author's ruling) — a world's total ore is its richest color, shared in its rolled proportions.** The three colors as the field rolls them are rescaled to sum to the largest of them, keeping their ratios: a one-color world barely changes, a balanced one keeps a third. §4.4's anticorrelation reads the capped deposit.

**Slant is measured by an absolute threshold per mineral** (the author's direction: trace amounts do not count): a mineral is present in a region when it holds at least the threshold, and the region's slant is how many are present. On the superseded 10-ly-site field, over 10-ly cells at `Band I` (1.0 kt): 42.1% none, 48.9% one, 7.8% two, 1.2% three; 58.0% of worlds hold no mineral at `Band I` (appendix §D.28). A turtler mines the **Z-column** for a modest baseline, but the mass sits near Z=0 and one hex captures only its XY footprint, so **the lion's share needs X-Y expansion**. **R-M3:** Z scale-height / ratio.

**The Gaussian is over the *Band*, not over the mass.** That is the design
statement, and it is the only reading under which the shape means anything: a
game whose every other magnitude compounds on the ladder cannot have its ore
spread linearly. The field is therefore **log-normal in kilotons** — a `Band IV`
seam holds ~715,000× a `Band I` one, where the old linear reading made it 4× —
which is the concentration the design requires: *a handful of extraordinary
worlds sitting next to each other*, and a long tail of worlds not worth the
freighter. Noise is additive on the Band, which is a proportional wobble in mass.

Storage is the mass. A Band is a **reading** of that number, never a second
thing to store: the field depletes, and a representation that cannot hold small
masses destroys ore at the bottom of the ladder (measured: three miners took
4.20× what one took instead of 3.00×). Mass is conserved without exclusions
(L6).

**`mineral_peak = Band IV`, ratified** (R-O82). The peak is the top of the
ladder, so the richest seams hold ~715,500 kt against a General hull priced at
1.0 kt. The consequence was measured and accepted rather than discovered later:
the standard bed hauls **2,256×** the ore it did under the linear field, none of
that surplus buys a colony, and the freighters moving it are the single largest
cost in the engine. Because the scale is ratified, that is an engine problem
(T-66) and not a number to walk back — the hauling loop has no notion of demand,
so an outpost on a body it can never exhaust schedules a trip per hold-full
forever.

### 4.4 Habitability ↔ metallicity, negatively correlated
Rich-mineral hexes tend **low-habitability**; habitable hexes **metal-poor** → expansion forces a **colony-vs-mine** choice. Homeworlds are the bounded exception (§3). **R-M4:** strength.

**The reading is the world's *mean* Band across the three colors** — the
geometric mean of the three masses — not the Band of their total. Both are
legitimate classifications; on a log ladder they are wildly different, because
the total is dominated by whichever color is richest. A world at
`(II, I, Empty)` reads ~`II` under the total and ~`I` under the mean, and at
`anticorrelation = 0.6` that difference burns a whole extra Band of
habitability off every such world. Measured: routing §4.4 through the total
cost **−52% colony-years** on seed 1 (10,105,286 → 4,845,144), all of it
habitability the galaxy never had. Under the mean Band, T-62 costs **−0.9%**
(seed 1 10,105,286 → 10,021,989; seed 7 10,037,745 → 9,941,898) — the residual
is the noise model, which moved from multiplicative-on-mass to
additive-on-Band, not the distribution.

### 4.5 Synthesis gates — pop Band IV + supply chain
Synthesis **only at pop Band IV** (§5.2). Each super needs **two** basics from distant hotspots → synthesis **generally demands a supply chain**; a hex where two gaussians overlap richly (synthesize **with no chain**) is **exceptionally high value** — the homeworld is the modest, archetype-locked instance. **R-M5:** supply-chain model.

**`RATIFIED` (the author's ruling) and built — forging is a forge's primary
purpose, priced above every use but survival.** A center whose population
reads `Band IV` is a forge. At each economy tick, after its yard has decided,
it synthesizes everything the yard left that synthesis can use:

| step | draws | makes |
|---|---|---|
| balanced basics | `m = min_c basic_c` of each basic | `Y_super · m` of every super |
| the pair left over | `min` of the two basics still held | `Y_super · 2 · min` of their super |
| balanced supers | `min_s super_s` of each super | `Y_apex · 3 · min` of apex |

`Y_super = 2/3`, `Y_apex = 1/2` (placeholders, R-M2); the loss is slag at the
forge (R-O59). What is left is one basic, waiting on the colors that pair it.

- **Survival first.** A forge's yard builds the sentries its Doctrine wants
  first; their price, and the rounds its sentries' magazines lack, are kept back
  from the forge (`survival_reserve`). Rounds are made from the holding.
- **Then Designs paid in supers — `RATIFIED` (R-MX17, the author's ruling: a
  Design paid in supers is priced higher than forging).** A forge's yard may
  build a Design a write bills in supers or apex; every order paid only in
  basics is quoted unpayable at a forge. The forge runs after the yard's
  decision, never ahead of it.
- **Its basics are its synthesis's.** No other build, whole Band, Exchange ask or
  hauler draws on them (`available_at`).
- **Priced above every other use.** A forge bids for the basics that complete
  a balanced set — every color up to the largest of what it holds and of its
  next whole Band's colors — at `Doctrine::forge_premium` times its price
  (`10`, **placeholder**; every other center's pressure is at most `1`), and
  its pull on its own empire's freight is scaled by the same factor.
- **Its empire first.** Supers its empire's centers wait on, and supers it has
  sold and not yet delivered, are kept back from apex.
- **It bids for the supers it has demand for — `RATIFIED` (R-MX18, the author's
  ruling).** Every super up to the largest it holds — what apex draws — is a
  refined need, bid at `forge_premium` times the refined floor; freight routes
  a refined hold to it by the same need. It asks zero for what it forged
  outside a balanced set (matching §8.7).

What consumes supers is the author's ruling (`Hyades_trees_and_card_value.md`
§4.6): **only a tier-3 card's Design writes, and apex only win conditions** —
so a card-free game forges and consumes nothing it forges. Tests:
`a_forge_forges_pairs_into_supers_and_balanced_supers_into_apex`,
`a_forge_builds_only_for_its_survival`,
`a_forge_sells_what_it_has_forged_and_keeps_it_until_delivery`,
`a_super_billed_design_is_forged_and_built_with_mass_conserved`. Measured
card-free with both rulings (appendix §D.31): colony-years and work-years
against `a50ef75` unchanged within two standard errors; 3.7% of the supers
forged cross between empires (810.6 of 22,040 kt over 4 seeds); a seat's native
super is 29.6% of what it forges, pooled. *Superseded:* synthesis on demand, for
an order or a sale only (appendix §D.30).

**Supers to the yards that want them — `RATIFIED` (the author's ruling,
T-146) and built; Doctrine, so a card can change it.** A yard whose standing
order lacks a super is sent one by an idle hauler from Reserve, which loads,
delivers and stands down at the yard (`src/sim/supply.rs`):

- `Doctrine::forge_supply_runs` (on by default) — from the nearest forge of its
  own empire with any of what it lacks to spare; free, one empire's holding
  moving between its planets.
- `Doctrine::buy_from_rival_forges` (on by default) — where no forge of its own
  can, from the nearest forge of another empire holding supers that empire
  does not need (its spare less what its empire waits on and what it has sold
  and not delivered), bought at the supers' floor price, buyer's purse to
  seller's (**placeholder price**).
- A run starts when a yard records a want and when a forge of its empire
  forges; a yard has one run in flight. `DoctrineWrite::SuperSupply` sets both
  flags; no card writes it yet. **`OPEN`:** the rival forge's stock is read
  without light-lag, as the Exchange's books are (T-145).

A card-free game has no order that lacks a super, so the runs never start in
one; on the twin bed (§3.1) they move supers to yards (appendix §D.36). Test:
`a_supply_run_brings_a_yard_its_supers_from_its_own_forge_or_a_rivals`.

**R-M5 — resolved in part.** Freight now loads against a forge's balanced-set
want and a forge out-pulls every other center, so precursors are hauled to a
forge for synthesis's sake, and a forge buys the supers it lacks (R-MX18).
**`OPEN`:** what else would make a forge buy a super from the forge native to it
rather than make it from bought basics — T-143.

### 4.6 Substitution — native only within a super's own counter-graph aspects
Each super is native across the **whole lineup — but only for the specific aspects of the counter-graph it brings.** Covering **Blue's** aspects with Red/Green/apex costs **a card each**; Blue does **not** natively cover another super's aspects. Supers are **non-interchangeable specialists**, cheap in their own region, card-expensive outside it. **R-M6:** each super's (and the apex's) aspect-set. Each super is also **dominant in different trees**, and is the counter-graph's general key in the trees it dominates and a traversal key elsewhere (`Hyades_technology_tree.md` §3.1, R-TECH25).

### 4.7 Use-domains — Stars! as a starting point only
First-pass lean: **Cyan → structure/propulsion, Magenta → weapons/energy, Yellow → economy/electronics**; supers/apex add advanced bands (§4.6). **R-M7:** full mapping + apex weapons (sim §5).

### 4.8 Color → tree mapping (the cost spine)
Card costs are **specific-color**, two-trees-per-domain (Politics↔Yellow and Technology↔Magenta after the swap):

| Color | Domain | Trees |
|---|---|---|
| **Cyan** | structure / propulsion | **Expansion** · **Growth** |
| **Magenta** | weapons / energy | **Warfare** · **Technology** |
| **Yellow** | economy / electronics | **Production** · **Politics** |

A homeworld rich in two colors is **cheap in those four trees, expensive in the other two** (§3). **T1_any is a rare exception.** **R-M8:** the swap leaves **Growth↔Cyan** as the lone soft fit (biosphere ≠ structure); **Technology↔Magenta** (military-tech axis) and **Politics↔Yellow** (economic leverage) are intended.

---

## 5. Population — Bands, Gibrat meaning, hard gate

### 5.1 Theater vs. command
Theater grows pop/infra logistically toward K (sim §2). Command reports a
discrete **Band** — `Band Empty` through **Band IV** — for which
**Weibull-quantile band** the value has crossed; the Weibull shape (`k` near
log-normal) makes bands **Gibrat-spaced** — each Band a fixed
*multiplicative* jump over the last. **Pop `Band I` ≈ small town; pop
`Band IV` ≈ many billions.** This is the founding instance of the general
**Band ladder** now defined once, for every quantity that uses this same
discrete multiplicative scale — `Hyades_mineral_cost_curve.md` §2.6, which
also states the constraint this section's band edges must satisfy: the
`Band I → II` and `Band II → III` step factors are each a rational number in
`[4, 8]`, shared with infrastructure (§5.3 below), biosphere, radiation,
gravity, cargo capacity, and hull production cost; `Band III → IV` is a free
tuning knob. **R-P1:** `k` + band edges, now scoped by that shared
constraint rather than chosen for population alone.

### 5.2 Pop is a hard production gate
Discrete Band **gates which designs build**, not speed. Default: **capitals
only at pop `Band IV`**, **synthesis only at pop `Band IV`**. **R-P2:**
pop→design table.

### 5.3 Infrastructure is the early binding constraint
At **Band IV/IV/I** (hab/bio/infra), **infra is scarce** (Liebig): K pinned
at **Band I** until built. Early game = **infra `Band I → Band IV`**
(unlocking pop-`Band IV` capitals + synthesis) — the *Stars!* maturation arc
(sim §10). Infra is also the softest wartime target. **R-P3:** rate vs.
clock, under the same shared step-factor constraint as §5.1.

---

## 6. Autopilot defaults

A card overrides exactly one default (sim §0a). The **command view is omniscient**, but the autopilot's **units act under fog of war** (scanning, stealth) and every reaction is **light-lagged** — cards issue instant global orders whose consequences propagate at *c* (contract §2). The Explore/Expand/Exploit rows below are specified in implementable detail in **`Hyades_autopilot_colonization_growth.md`**.

| Default | What the sim does unasked | Round-1 override |
|---|---|---|
| **Explore** | idle **Contact** hulls path to nearest unrevealed hex | **The Long Voyage**; **The Compass** |
| **Expand** | **Systems** hulls settle nearest *viable* world, weighing **colony-vs-mine** | **The Compass**; **The Long Voyage** |
| **Exploit** | colonies build **infra toward K**, grow pop, **mine local incl. Z-column** | **The First Hearth**, **First Furrow**, **The Open Hand** |
| **Synthesize** | at **pop Band IV**, convert per `Y_super`/`Y_apex` + wastage (`Hyades_mineral_cost_curve.md` §5.0) **when an order or a sold contract owes it** (§4.5); a matured homeworld self-makes its **one** native super | post-pop-Band-IV (Synthesis) |
| **Exterminate / defend** | **Offensive** hulls **hold**; engage in-range, **line** formation, deterministic accept/decline | **Those Who Stand**, **The Pattern**, **Open Skies** |
| **Build** | planetside at docks; capitals/synthesis **pop-Band-IV-gated**; mobile dock excepted | **The Compass**, **The First Hearth** |
| **Retreat** (sim §4) | defeated ships flee toward open/friendly space **in 3D**; survival = wreck roll | deep Warfare + positioning |
| **Fortification** | **none free** (sim §2a) | **The Aegis** |

**R-A1:** expand-bias · **R-A2:** formation/posture · **R-A3:** trade/NAP in the verb model.

---

## 7. Narrative principle — every story is how love wins

**The thesis.** Every story Hyades tells is one story: *love wins — love outcompetes everything else over a long enough sweep of history.* Not sentiment, but the hard-SF reading of deep time: cooperation is the competitively superior strategy at scale, the engine behind life's major transitions — [groups with more cooperators outcompete and displace those with fewer](https://www.pnas.org/doi/10.1073/pnas.0602530103) (Traulsen & Nowak, *PNAS* 2006; Nowak, "Five rules for the evolution of cooperation," *Science* 2006), an intuition reaching back to Darwin and Kropotkin. Love is rendered as a **force that wins**, never a feeling that is merely nice.

> **Read `Hyades_trees_and_card_value.md` §1 before writing any card text.** The six sagas below are the empires' mythmaking about themselves, not the game's posture, and the satirical target is named there.

**Emergent, never scripted.** As in *Beyond the Sun*, the path **is** the story ([The Giant Brain](https://giantbrain.co.uk/2023/07/06/somewhere-beyond-the-sun/)) — but inverted in tone: where BtS carries desperation (Earth dying, a flight from extinction), Hyades carries hope (a young people *setting out*). And the table's **myriad interactions** — yomi, trade, the wars and the truces — compose a **unique** instance of the love-wins story every game. Cards are **beats**, not a script; their meaning shifts with the company they keep. Hyades supplies the grammar; the players write the sentence. The voice is **warm hard SF**, **never imperial or naval**; every card keeps a `role` subtitle so fiction never drifts from mechanic.

**Six modes of love winning** (the warm axes). **Cyan** — love that *reaches* and *nurtures*. **Yellow** — love that *provides* and *binds*. **Magenta** — light: love that *understands*, and love that *parts with what cannot love* (light both reveals and cuts).

| Tree (color) | Saga | Mode of love | Arc: foundation → … → apex/win | Round-1 mouth |
|---|---|---|---|---|
| Expansion (C) | **The Far Shore** | reaches | The Long Voyage → Landfall → A Thousand Shores | **The Long Voyage** |
| Growth (C) | **The Greening** | nurtures | First Furrow → The Flourishing → The Myriad | **First Furrow** |
| Production (Y) | **The Hearth** | provides | The First Hearth → The Great Works → Cornucopia | **The First Hearth** |
| Politics (Y) | **The Commonwealth** | binds | Hands Across the Dark → Common Cause → The Concord | **Hands Across the Dark** |
| Technology (M) | **The Long Dawn** | understands | First Light → The Breakthrough → Transcendence | **First Light** |
| Warfare (M) | **The Hard Mercy** | parts with what cannot love | Those Who Stand → The Long Ride → The Range Wars → The Beloved Republic *(believed)* | **Those Who Stand** |

**Warfare — The Hard Mercy** is the hardest case and the one made explicit, and its register is the **Hollywood Western** (*The Magnificent Seven*, *Shane*, and the deconstruction, *Unforgiven*): the gun taken up so the garden can grow, by those who often cannot live in the world they save. A people **separates out what cannot love among them, then goes looking for more** — and that excision is itself destabilizing, the way a society churns through successive reconstitutions after a monarchy falls (the various French Republics, the Terror and Thermidor and the rest) before any stable order is found. So the arc runs **Those Who Stand → The Long Ride → The Range Wars → The Beloved Republic**: the defenders' stand, the outward search, the turbulent founding, and at last the polity that only love deserves — [Forster's "Beloved Republic"](https://www.oxfordreference.com/display/10.1093/acref/9780191843730.001.0001/q-oro-ed5-00004518) ("Only Love the Beloved Republic deserves that," *What I Believe*, 1939). Crucially the Beloved Republic is the Warfare **win-state, not the win-object** (that remains the **War Sun**, cmd) and it requires **no omniscient POV**: it is enough that the victor's civilization *believes* it has made the Beloved Republic. Whether the belief is true or is *Unforgiven*'s self-told lie is the story each unique game decides. **Amended — it is the self-told lie** (`Hyades_trees_and_card_value.md` §1.5). Everything in this section is the *empire's own voice*, sincere and never winking, and that is exactly what makes it work; what changed is that it is no longer the game's claim about what happened. Every winning game is Paul Atreides' jihad or The Mule's empire, told by the winner about themselves, and the player registers the gap.

**Framing acts** (not tree beats): *First Principles* — **The Compass** (Doctrine), **The Pattern** (Design); *Common Acts* — **The Open Hand** (Trade), **Open Skies** (Non-Aggression), **The Aegis** (Citadel). Depth-1+ beats are **illustrative first-pass arcs**. **R-N1:** confirm the Warfare mouth name (**Those Who Stand**; alts *Strap on the Iron*, *The Hard Hands*, *High Noon*) and saga (**The Hard Mercy**; alt *The Gun and the Garden*), and lock the other five arcs. **R-N2:** tier-crossing **named events** (the BtS pacing move) — the elimination drumbeat could ride these, making each excision a table-wide beat.

---

## 8. Round-1 opening actions — derived (11 cards)

Turn-1 state: co-located homeworlds at **Band IV/IV/I** (pop ~Band I, **no pop-Band-IV planet**), autopilot per §6, **3 actions** (no free action), six **mouths accessed not unlocked**, **super-aligned** banks. Menu:

- **First Principles** — `The Compass`, `The Pattern` (cross-domain → **0 minerals**).
- **Common Acts** — `The Open Hand`, `Open Skies` (peaceful → **0 minerals**), `The Aegis` (fortification → **Cyan**). **Synthesis stays out of round 1** — pop-Band-IV-gated.
- **Mouth beats** — each a **base + 1-action reach** costed in **its domain color** (§4.8): Those Who Stand · First Light **Magenta** · The Long Voyage · First Furrow **Cyan** · The First Hearth **Yellow** (Production); Hands Across the Dark **Yellow** (Politics).

**Specific-color costs make the start bite** (§3). **No T1_any.** Costs remain **placeholders** (hash churns with them). **R-7/R-9:** standing/peaceful action costs.

---

## 9. Ratification points (consolidated)

- **R-G0** sim §1/§6 absorb 3D/2D · **R-G1** hex side ratified at 70 ly (§1); prism depth open · **R-G5** the hex misses the per-seat hexes-per-player target at 6 seats (§1; open) · **R-G2** counts + core · **R-G3** archetype placement · ~~**R-G4**~~ self-synth yield + "exactly one" (resolved: color theory, no archetype barred, §3)
- ~~**R-M1**~~ names — **resolved**: Cage Ice, Rosepeter, Voltslate, Strange Matter (§4.1); super names are placeholders under T-142 · **R-M2** ratios+wastage · **R-M3** Z scale-height · **R-M4** anticorrelation · **R-M5** supply chain (resolved in part, §4.5) · **R-M6** super aspect-sets · **R-M7** use-domains+apex · **R-M8** Growth↔Cyan soft fit (rest intended)
- **R-P1** Weibull `k`+bands · **R-P2** pop→design gating · **R-P3** infra rate vs. clock
- **R-A1** expand-bias · **R-A2** formation/posture · **R-A3** trade/NAP in verb model
- **R-N1** lock the six saga arcs as modes of love winning; Warfare voice now Hollywood-Western (Those Who Stand; saga alt 'The Gun and the Garden'); confirm the believed Beloved Republic win-state · **R-N2** tier-crossing named events carrying the elimination drumbeat
- **R-7/R-9** standing/peaceful action costs · cost numbers placeholder pending R-5
