# Hyades — Galaxy Generation, Materials, Population & Autopilot
*Companion to `Hyades_simulation_model.md` (sim §) and `Hyades_command_cards.md` (cmd §). **Rev 8** — adds the **information model** (omniscient command view over a fog-of-war simulation: scanning, stealth, light-lag) and points the Explore/Expand/Exploit defaults at the first detailed autopilot, `Hyades_autopilot_colonization_growth.md`. Warfare is in the **Hollywood-Western** register (Those Who Stand → … → the *believed* Beloved Republic, a win-state not a win-object). The **card design contract** lives in `Hyades_card_contract.md`. Carries rev 6's love-wins thesis, rev 5's warm-hard-SF voice, rev 4's color swap, rev 3's super-aligned homeworlds, specific-color costs. Grounds `hyades_opening_actions_r1.json`. Calls flagged **R-Gn/R-Mn/R-Pn/R-An/R-Nn/R-Cn**; carry-overs **R-7/R-9**.*

---

## 1. Two-view model — 3D theater, 2D command

The **theater is 3D** (X, Y, Z): movement, combat, encirclement, and retreat happen in a volume, so heading/velocity/formation/encirclement (sim §6) become **3-vectors** and egress can be cut in three dimensions. The **command view is 2D** (X, Y): the strategic map is the flat hex tiling, Z collapsed. Plan in 2D, render in 3D. **R-G0 / sim cross-ref:** sim §1/§6 absorb this — encirclement and retreat are volumetric.

**Hex dimensions.** Hexagonal **prism**: side `s ∈ [50, 250] ly`, depth `1×–5× s` (**start 3×**). Light crosses a hex in 50–250 yr, so under the *c*-cap (sim §1a) a round spans **long epochs** and intercepts are slow. **R-G1:** final `s` and depth.

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

**Balance targets the 2-neighbour configurations — 3, 6, 12, 18 — where every seat borders exactly two others.** **N=2 is supported but is not a balance target** (ratified): the domino gives each player *one* neighbour, and the `p % 3` archetype cycle leaves it with Blue and Red and no Green. Both are accepted consequences of a configuration nothing is tuned around — which **resolves R-O9** as "known and accepted" rather than as a defect to fix.

**R-O12 (resolved): `Galaxy::FAIR_COUNTS` is `[2, 3, 6, 12, 18]`.** It had been truncated at 12 while `starting_hex_radius` already carried an `18 => 4.5` branch; all three of its ring radii are exactly `N/6 + 1.5`, so that branch was the third term of the family rather than a stray, and the list was simply one term short. The engine now expresses the family as that closed form instead of three magic numbers. **R-G2:** core contents (supported counts now settled).

---

## 3. Homeworlds — super-aligned, identical in shape, equitable but unequal

Identical in shape (fairness): **`4 / 4 / 1`** (hab/bio/infra) → `K = min = 1`; a **small town** that matures by building **infra 1 → 4** (§5). **Rich in two tier-1 colors, poor in the third** — the two precursors of **one super** — so **mining income is imbalanced** and three rotationally-symmetric archetypes result (reflecting the §7 color swap):

| Homeworld | Rich basics | Poor | Native super @ pop-4 | Cheap tree-pairs | Expensive pair |
|---|---|---|---|---|---|
| **Blue-type** | Cyan, Magenta | Yellow | **Blue** (C+M) | Expansion/Growth · Warfare/Technology | Production/Politics |
| **Red-type** | Magenta, Yellow | Cyan | **Red** (M+Y) | Warfare/Technology · Production/Politics | Expansion/Growth |
| **Green-type** | Yellow, Cyan | Magenta | **Green** (Y+C) | Production/Politics · Expansion/Growth | Warfare/Technology |

Same **2-rich-1-poor shape** rotated → **equal total wealth, color-shifted**. The start is **equitable but unequal**: 4 trees cheap, 2 expensive, your native super fixed. Blue-type is the militarist-expander-technologist (weak economy/diplomacy); Red-type the tall industrial-military-political power (weak at spreading); Green-type the economic-expansionist (weak on the whole war-tech axis). **N=3:** one of each. **N=6:** B-R-G alternating. **R-G3:** placement per table.

**Eventual self-synthesis.** At **pop-4** a homeworld self-synthesizes **exactly one** super — its archetype's — in **modest** quantity, **no supply chain**. The **corrected R-G4:** the homeworld is a **bounded** exception to the habitability↔metallicity anticorrelation (§4.4) — habitable *and* modestly mineralized in two colors, **enough for one modest super, not super-rich**. **R-G4:** modest yield; confirm "exactly one super" is hard.

---

## 4. Materials — the ladder, the color algebra, and synthesis

### 4.1 Three tiers
**Tier 1 — basics: Cyan, Magenta, Yellow.** Mined (§4.3). · **Tier 2 — supers: Red, Green, Blue.** Synthesized, never mined (R-M1). · **Apex.** Synthesized from supers; metallic silver-white, *Platinum* a placeholder name (R-M1).

### 4.2 Color algebra + the 3:2:1 ladder
Fixed two-basic recipes: **Blue ← Cyan + Magenta · Red ← Magenta + Yellow · Green ← Yellow + Cyan.** Ladder **3 basics → 2 supers → 1 apex** with **wastage** (cards reduce it). **No direct substitution.** **R-M2:** ratios + wastage.

### 4.3 Tier-1 distribution — 3D field, XY-dominant
**Gaussian in X & Y** (each hue's hotspot) **× exponential decay in Z from the midplane**. A turtler mines the **Z-column** for a modest baseline, but the mass sits near Z=0 and one hex captures only its XY footprint, so **the lion's share needs X-Y expansion**. **R-M3:** Z scale-height / ratio.

### 4.4 Habitability ↔ metallicity, negatively correlated
Rich-mineral hexes tend **low-habitability**; habitable hexes **metal-poor** → expansion forces a **colony-vs-mine** choice. Homeworlds are the bounded exception (§3). **R-M4:** strength.

### 4.5 Synthesis gates — pop-4 + supply chain
Synthesis **only at pop-4** (§5.2). Each super needs **two** basics from distant hotspots → synthesis **generally demands a supply chain**; a hex where two gaussians overlap richly (synthesize **with no chain**) is **exceptionally high value** — the homeworld is the modest, archetype-locked instance. **R-M5:** supply-chain model.

### 4.6 Substitution — native only within a super's own counter-graph aspects
Each super is native across the **whole lineup — but only for the specific aspects of the counter-graph it brings.** Covering **Blue's** aspects with Red/Green/apex costs **a card each**; Blue does **not** natively cover another super's aspects. Supers are **non-interchangeable specialists**, cheap in their own region, card-expensive outside it. **R-M6:** each super's (and the apex's) aspect-set.

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

## 5. Population — integer levels, Gibrat meaning, hard gate

### 5.1 Theater vs. command
Theater grows pop/infra logistically toward K (sim §2). Command reports **integer level 0–4** = which **Weibull-quantile band** the value has crossed; the Weibull shape (`k` near log-normal) makes bands **Gibrat-spaced** — each level a fixed *multiplicative* jump. **Pop 1 ≈ small town; pop 4 ≈ many billions.** **R-P1:** `k` + band edges.

### 5.2 Pop is a hard production gate
Discrete level **gates which designs build**, not speed. Default: **capitals only at pop 4**, **synthesis only at pop 4**. **R-P2:** pop→design table.

### 5.3 Infrastructure is the early binding constraint
At `4/4/1`, **infra is scarce** (Liebig): K pinned at 1 until built. Early game = **infra 1 → 4** (unlocking pop-4 capitals + synthesis) — the *Stars!* maturation arc (sim §10). Infra is also the softest wartime target. **R-P3:** rate vs. clock.

---

## 6. Autopilot defaults

A card overrides exactly one default (sim §0a). The **command view is omniscient**, but the autopilot's **units act under fog of war** (scanning, stealth) and every reaction is **light-lagged** — cards issue instant global orders whose consequences propagate at *c* (contract §2). The Explore/Expand/Exploit rows below are specified in implementable detail in **`Hyades_autopilot_colonization_growth.md`**.

| Default | What the sim does unasked | Round-1 override |
|---|---|---|
| **Explore** | idle **Contact** hulls path to nearest unrevealed hex | **The Long Voyage**; **The Compass** |
| **Expand** | **Systems** hulls settle nearest *viable* world, weighing **colony-vs-mine** | **The Compass**; **The Long Voyage** |
| **Exploit** | colonies build **infra toward K**, grow pop, **mine local incl. Z-column** | **The First Hearth**, **First Furrow**, **The Open Hand** |
| **Synthesize** | at **pop-4**, convert per 3:2:1 + wastage; a matured homeworld self-makes its **one** native super | post-pop-4 (Synthesis) |
| **Exterminate / defend** | **Offensive** hulls **hold**; engage in-range, **line** formation, deterministic accept/decline | **Those Who Stand**, **The Pattern**, **Open Skies** |
| **Build** | planetside at docks; capitals/synthesis **pop-4-gated**; mobile dock excepted | **The Compass**, **The First Hearth** |
| **Retreat** (sim §4) | defeated ships flee toward open/friendly space **in 3D**; survival = wreck roll | deep Warfare + positioning |
| **Fortification** | **none free** (sim §2a) | **The Aegis** |

**R-A1:** expand-bias · **R-A2:** formation/posture · **R-A3:** trade/NAP in the verb model.

---

## 7. Narrative principle — every story is how love wins

**The thesis.** Every story Hyades tells is one story: *love wins — love outcompetes everything else over a long enough sweep of history.* Not sentiment, but the hard-SF reading of deep time: cooperation is the competitively superior strategy at scale, the engine behind life's major transitions — [groups with more cooperators outcompete and displace those with fewer](https://www.pnas.org/doi/10.1073/pnas.0602530103) (Traulsen & Nowak, *PNAS* 2006; Nowak, "Five rules for the evolution of cooperation," *Science* 2006), an intuition reaching back to Darwin and Kropotkin. Love is rendered as a **force that wins**, never a feeling that is merely nice.

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

**Warfare — The Hard Mercy** is the hardest case and the one made explicit, and its register is the **Hollywood Western** (*The Magnificent Seven*, *Shane*, and the deconstruction, *Unforgiven*): the gun taken up so the garden can grow, by those who often cannot live in the world they save. A people **separates out what cannot love among them, then goes looking for more** — and that excision is itself destabilizing, the way a society churns through successive reconstitutions after a monarchy falls (the various French Republics, the Terror and Thermidor and the rest) before any stable order is found. So the arc runs **Those Who Stand → The Long Ride → The Range Wars → The Beloved Republic**: the defenders' stand, the outward search, the turbulent founding, and at last the polity that only love deserves — [Forster's "Beloved Republic"](https://www.oxfordreference.com/display/10.1093/acref/9780191843730.001.0001/q-oro-ed5-00004518) ("Only Love the Beloved Republic deserves that," *What I Believe*, 1939). Crucially the Beloved Republic is the Warfare **win-state, not the win-object** (that remains the **War Sun**, cmd) and it requires **no omniscient POV**: it is enough that the victor's civilization *believes* it has made the Beloved Republic. Whether the belief is true or is *Unforgiven*'s self-told lie is the story each unique game decides.

**Framing acts** (not tree beats): *First Principles* — **The Compass** (Doctrine), **The Pattern** (Design); *Common Acts* — **The Open Hand** (Trade), **Open Skies** (Non-Aggression), **The Aegis** (Citadel). Depth-1+ beats are **illustrative first-pass arcs**. **R-N1:** confirm the Warfare mouth name (**Those Who Stand**; alts *Strap on the Iron*, *The Hard Hands*, *High Noon*) and saga (**The Hard Mercy**; alt *The Gun and the Garden*), and lock the other five arcs. **R-N2:** tier-crossing **named events** (the BtS pacing move) — the elimination drumbeat could ride these, making each excision a table-wide beat.

---

## 8. Round-1 opening actions — derived (11 cards)

Turn-1 state: co-located homeworlds at `4/4/1` (pop ~1, **no pop-4 planet**), autopilot per §6, **3 actions** (no free action), six **mouths accessed not unlocked**, **super-aligned** banks. Menu:

- **First Principles** — `The Compass`, `The Pattern` (cross-domain → **0 minerals**).
- **Common Acts** — `The Open Hand`, `Open Skies` (peaceful → **0 minerals**), `The Aegis` (fortification → **Cyan**). **Synthesis stays out of round 1** — pop-4-gated.
- **Mouth beats** — each a **base + 1-action reach** costed in **its domain color** (§4.8): Those Who Stand · First Light **Magenta** · The Long Voyage · First Furrow **Cyan** · The First Hearth **Yellow** (Production); Hands Across the Dark **Yellow** (Politics).

**Specific-color costs make the start bite** (§3). **No T1_any.** Costs remain **placeholders** (hash churns with them). **R-7/R-9:** standing/peaceful action costs.

---

## 9. Ratification points (consolidated)

- **R-G0** sim §1/§6 absorb 3D/2D · **R-G1** hex `s`+depth · **R-G2** counts + core · **R-G3** archetype placement · **R-G4** self-synth yield + "exactly one"
- **R-M1** rename tier-2/apex · **R-M2** ratios+wastage · **R-M3** Z scale-height · **R-M4** anticorrelation · **R-M5** supply chain · **R-M6** super aspect-sets · **R-M7** use-domains+apex · **R-M8** Growth↔Cyan soft fit (rest intended)
- **R-P1** Weibull `k`+bands · **R-P2** pop→design gating · **R-P3** infra rate vs. clock
- **R-A1** expand-bias · **R-A2** formation/posture · **R-A3** trade/NAP in verb model
- **R-N1** lock the six saga arcs as modes of love winning; Warfare voice now Hollywood-Western (Those Who Stand; saga alt 'The Gun and the Garden'); confirm the believed Beloved Republic win-state · **R-N2** tier-crossing named events carrying the elimination drumbeat
- **R-7/R-9** standing/peaceful action costs · cost numbers placeholder pending R-5
