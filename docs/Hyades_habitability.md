# Hyades — Habitability Distribution (draft Rev 1)

**Status:** draft, proposed by Claude for Jonathan's ratification. New spec.
**Not implemented in code** — this is spec-first, matching how
`Hyades_loadout.md` and `Hyades_vehicle_roles.md` started; the current single
`habitability: f64` planet field is what this replaces, once ratified.

**The brief:** *"Rocky planets in the circumstellar habitable zone with
gravity & radiation in the range of human tolerance is the requirement...
cards will both change the range as well as change the value of a specific
planet... we don't need to model this with fidelity, just a plausible set of
functions... playability is more important."*

---

## 1. Ontology (same discipline as `Hyades_vehicle_roles.md` §1)

Three things, currently collapsed into one stored scalar:

| Term | ECS binding | Mutated by |
|---|---|---|
| **Habitability profile** | a Component on the planet — physical values, mostly fixed | terraforming cards |
| **Tolerances** | a Component on the player — acceptable ranges per dimension | tolerance-widening cards |
| **Effective habitability** | a **Query**: `habitability_for(planet, player)` — computed, never stored | nothing directly; it's derived |

This is the same split `Hyades_vehicle_roles.md` §9 made for Doctrine:
physical reality is data, a player's current capability is data, and the
number that actually matters — *"can this specific empire live here"* — is
computed fresh from both, never cached. That's what makes it possible for
the same planet to be a live prospect for one empire and worthless to
another, and for both of those to change independently over the game.

---

## 2. The four dimensions

Three continuous, one categorical. **Deliberately a *set* of independent
functions, not one composite score** — per the brief, so cards can move one
axis without moving the others (*"increase radiation resistance... but not
gravity tolerance... different players unlock different sites"*).

### 2.1 Composition — hard gate, not graded

`composition: enum { Rocky, GasGiant, IceGiant, Molten }`, categorical draw.
Only `Rocky` is ever colonizable, full stop — *"rocky planets... is the
requirement"* reads as necessary, not a matter of degree. Nothing else on
this list is checked if this fails.

- **Generation:** weighted random draw. Starting weight: **55% Rocky**
  (§5 explains the target). No spatial correlation with the other
  dimensions in this first pass — flagged as **R-H2** in case a light
  correlation (rockier near the HZ center, more gas/ice giants at the
  extremes, mirroring real solar-system structure) is worth the complexity;
  skipped for now since "we don't need fidelity."
- Terraformable in principle like any other value here (a card could turn a
  moon "effectively rocky"), but this is the one dimension where that reads
  as a rare, expensive, late-game effect rather than a routine tech — noted,
  not specially coded.

### 2.2 Circumstellar habitable zone (HZ) position — signed

`hz_offset: f64`, **signed**: 0 = centered in the zone, negative = too close
(hot), positive = too far (cold). Signed rather than a single "distance from
ideal" scalar specifically so heat tolerance and cold tolerance can be
**separate, independently unlockable axes** — doubling the number of
interesting card-differentiation points for the cost of one sign bit. Flag
**R-H3**: confirm this reading, since it's an interpretive choice, not
stated outright.

- **Generation:** Gaussian, mean 0, spread `σ_hz` (tunable).
- **Tolerance:** a player's `hz_tolerance: (f64, f64)` — a `(too_hot_max,
  too_cold_max)` pair, not a single number, so heat and cold resistance can
  diverge by card.

### 2.3 Gravity

`gravity: f64`, units of standard g. **Generation:** log-normal (`gravity =
exp(Gaussian(ln 1.0, σ_g))`) — planetary surface gravity comes from mass and
radius combining multiplicatively, and this guarantees positivity for free
without clamping, which a plain Gaussian wouldn't. Not fidelity — just the
shape that avoids an obviously wrong tail (negative gravity).

- **Tolerance:** `gravity_tolerance: (f64, f64)`, e.g. default `(0.7, 1.4)`
  — two-sided, so "high-g adaptation" and "low-g adaptation" can be separate
  cards too.

### 2.4 Radiation

`radiation: f64`, abstract units, `0` = none, unbounded above. **Generation:**
exponential — most systems quiet, a long thin tail of dangerous outliers
(flare stars, proximity to energetic remnants), which is the qualitative
shape real background radiation environments have without needing an actual
stellar-astrophysics model.

- **Tolerance:** `radiation_tolerance: f64` — a single upper threshold, not a
  range (unlike gravity/HZ, more radiation is never *better*, so there's no
  symmetric "too little" failure mode worth modeling).

---

## 3. Aggregation — reusing Liebig's law, not inventing a new one

Each continuous dimension maps to a `0–4` suitability sub-score (the same
scale `habitability`/`biosphere`/`infrastructure` already use), via a smooth
falloff from the tolerance band — full marks inside a comfortable core,
tapering toward `0` as the value exits the tolerated range. **R-H4:** I'd
default to a smoothstep-style continuous taper (a planet just outside
tolerance is marginal, not instantly worthless) over a hard cliff, since a
graded frontier reads as more interesting than a binary one — flagged, not
asserted, since a hard cliff is a legitimate alternative with its own
appeal (every tolerance-widening card has a crisp, visible "now this world
counts" moment).

```
habitability_for(planet, player) =
    0                                              if planet.composition != Rocky
    min(hz_score, gravity_score, radiation_score)  otherwise
```

Composition is the gate; the three continuous scores combine by **the same
`min()` this codebase already uses for `K = min(hab, bio, infra)`**
(`Hyades_simulation_model.md` §2a). No new aggregation rule — the weakest
dimension caps the world, exactly like Liebig's law already caps carrying
capacity elsewhere. A world with perfect gravity and radiation but a wildly
wrong HZ offset is still marginal, same as a world with everything else
right but no built infrastructure is still capped today.

**What this replaces:** the single stored `habitability: f64` field on
`Planet` becomes `habitability_for(planet, player)`, called wherever
`planet.habitability` is read today (`K`, `K_potential`, ranking). **R-H6:**
this makes `K` genuinely player-relative for the first time — a colony's
ceiling would depend on its *owner's current tolerances*, not a fixed
planet fact, which means **a colony you already hold could grow its own `K`
later** as you unlock better tolerances elsewhere in the tree. That reads
as a real, positive feature (investment in tolerance tech pays off on
existing holdings, not just on opening new sites) rather than a side
effect worth suppressing — flagged for confirmation because it's a
non-obvious consequence, not because it looks wrong.

**Biosphere stays separate.** `biosphere` (ecosystem richness) isn't a
fifth habitability dimension — it's already an independent factor in
`K = min(...)`, and stays that way. **R-H5**, noted only so the two aren't
accidentally conflated during implementation.

---

## 4. Card interaction — two distinct effects, cleanly separated

Directly parallels `Hyades_vehicle_roles.md`'s Loadout (ship-fit, mutable by
docking) vs. Doctrine (player tuning, mutable by... itself) split:

- **Tolerance-widening (a player Component change).** *"Increase the
  radiation resistance of a population"* → raises that player's
  `radiation_tolerance` only. Every other player's access to that same
  world is unaffected. This is how different players end up with
  genuinely different colonizable maps from the same galaxy.
- **Terraforming (a planet Component change).** Directly mutates a specific
  planet's `gravity` / `radiation` / `hz_offset` (or, rarely, `composition`
  — §2.1). Universal: once done, the change is visible to every player
  identically, the same way a Colonizer recycling into infrastructure is a
  permanent fact about the world, not a fact about who did it.

Both read from and write to Components already described in §1 — no new
ECS primitive needed, just two more card-effect targets (player vs. planet),
which is the same shape every other card-effect in the design already has.

---

## 5. Tuning for playability — the actual point

*"The plausibility has to give a playable game space. Playability is more
important."* Concretely, this means picking distribution parameters so that:

1. **A generous fraction of the galaxy is colonizable from the start** —
   enough that "fully colonize the habitable systems given sufficient time"
   (this conversation) is a real, reachable endgame, not something the
   distribution itself forecloses. The last MC pass found the *economy*
   already under-colonizing relative to that bar (only ~88 of ~212 systems
   at the most generous setting tried) — this spec should not make that
   worse by being stingy on top of it.
2. **Real gradient survives** — not everything is equally good, so finding
   and fighting over the best sites stays meaningful. `min()`-aggregation
   already does most of this work: a world needs to clear *all three*
   continuous dimensions plus the rocky gate to be excellent, so peak-tier
   worlds stay comparatively rare even if each individual dimension is
   generously distributed.
3. **Tolerance-widening should compound over a game**, the same way
   expansion itself compounds (`Hyades_vehicle_roles.md`'s closing note on
   the production-queue redesign) — start with access to a meaningful but
   incomplete slice of the rocky galaxy, and have the accessible fraction
   grow visibly as tolerance cards land. That's the mechanism that actually
   delivers "given sufficient time, colonize everything habitable": not
   just economic throughput, but the *set of what counts as habitable*
   getting larger for you specifically as you invest.

**Starting parameters (R-H1, all MC-tunable placeholders):**

| Parameter | Starting value | Reasoning |
|---|---|---|
| Rocky fraction | 55% | Comfortably over half, so composition alone isn't the dominant filter |
| `σ_hz` | tuned so ~75% of Rocky worlds clear default HZ tolerance | leaves room for tolerance cards to matter |
| Gravity tolerance | `(0.7, 1.4)` g | ~70–75% of a log-normal(1.0) draw by construction |
| Radiation tolerance | set so ~75% of Rocky worlds clear it | exponential's thin tail means most worlds clear a modest threshold by design |

Combined, roughly `0.55 × 0.75³ ≈ 0.23` — **~23% of all systems** meaningfully
habitable under default (no-card) tolerances. Against ~212 total systems in
a 3-seat game, that's ~49 systems — deliberately *not* the ceiling, just the
opening slice; tolerance cards are what's supposed to grow it toward "most
of the rocky galaxy" over a full game. **This ratio is the actual lever to
tune, not the individual thresholds** — flagged plainly as a first guess,
validated the same way `fleet_size_tuning.rs` validated the mineral economy,
once this is implemented: sweep the fraction, check colonization outcomes
at the horizon against the "fully colonize given time" bar directly.

---

## 6. Open ratification points

**R-H1** starting distribution parameters (table above) — first guess,
needs an MC sweep once implemented. **R-H2** should composition correlate
spatially with HZ position, or stay independent. **R-H3** confirm the
signed-HZ-offset (hot/cold as separate axes) reading. **R-H4** smooth taper
vs. hard cliff at the tolerance boundary. **R-H5** confirm biosphere stays
a separate factor, not folded into this. **R-H6** confirm player-relative
`K` (an owned colony's ceiling can grow as its owner's tolerances improve)
is a wanted feature, not an oversight to close off.

**Out of scope here:** actual card text/costs for tolerance-widening or
terraforming effects (Growth/Politics tree content, per `hyades_todo.md`'s
adjacent note that Growth/Production/Politics will carry nonconsensual-
interaction cards — this is the consensual, own-empire-improvement side of
the same trees); implementation in code, pending ratification.
