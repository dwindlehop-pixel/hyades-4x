# Hyades — the game interface (Rev 1)

The presentation of a Hyades game: how a run leaves the engine, how it is
played back, and how it is drawn. **This spec carries ratified and open
decisions only** (`AGENTS.md` §6). The measurements behind it are in
`docs/experiments/` §D.58.

Status key: **RATIFIED** — the author's ruling; **BUILT** — implemented to a
ratified decision; **OPEN** — a decision not yet made, with a recommendation
where one is attached. Every magnitude marked *placeholder* is unratified.

Code: the engine's recorder is `src/replay.rs` and `Simulation::snapshot_at`;
the viewer is the `viewer/` crate (`hyades-viewer`); the web client is `web/`;
deployment is `.github/workflows/pages.yml`.

---

## 1. The author's rulings

RATIFIED (T-149, the author's request that opened this spec):

1. **The presentation is decoupled from the simulation.** It reads what the
   engine records and nothing reaches back (design law #15).
2. **Old games can be replayed, rewound, sought and their logs filtered.**
3. **Two modes.** A **juicy** mode for enjoying the game, and a **tactical**
   mode for the minutiae of the simulation — replay analysis, Monte-Carlo
   overview and simulation debugging.
4. **Juicy mode's style is found, not specified; its lighting scheme is the
   heart of its visual language; it needs levels of detail** to be legible at
   every scale.
5. **Tactical mode evokes a 16-color EGA game without being one.** It uses more
   than sixteen colors, none of them mapped to the EGA palette, with full 8 bits
   per channel — the way modern pixel-art games evoke a lower bit depth through
   palette design rather than hardware limits.
6. **The tactical palette is the author's `hyades_palette.js`, rotated to a tone
   map that evokes a 21st-century design language in dialogue with 1970s Earth
   Day art. Its hue and tone require the author's approval** (R-UI1).
7. **Many colors off that palette show key simulation information** — receiving
   damage, acceleration and other status.
8. **In tactical mode every entity is drawn at its position in the galaxy view,
   and its Design and Doctrine are reflected in its glyph and colors.**
9. **The viewer is written in Rust.**
10. **The viewer uses GPU acceleration.**
11. **Continuous deployment to GitHub Pages** from `main`.
12. **Test-driven:** the interface is specified by tests written before the
    code they test.

RATIFIED (T-150, the author's rulings after the first deployment):

13. **The palette is ratified on a live design, not on a palette sheet.** The
    author judges colors as the game is drawn in them, so the viewer carries a
    live editor (§6.2.1) and a ratification names the settings it was made on.
14. **The game interface and networking may link upstream packages.** The
    engine stays dependency-free (`AGENTS.md` §4); the viewer links the engine
    only as a test dependency, and nothing an interface package computes enters
    replicated state.

RATIFIED (T-151, the author's ruling after the first phone review):

15. **The replay viewer is a menu option of the game client, not a separate
    page.** The site opens on the client's menu; replays, the palette sheet and
    (when it is built) a new game are its screens (§8.1).

RATIFIED (T-152, the author's rulings on the tactical mode):

16. **A glyph appears only centered on its hull's actual location.** Where
    glyphs overlap, a display order by role, hull and distance from the
    homeworld decides which is on top; no glyph is offset and no line joins a
    glyph to a place (§6.4).
17. **Magnitudes are drawn with Band math, not with a logarithm of the
    viewer's own.** A holding's bar reads the engine's Band reading of it
    (§6.5).
18. **Armed hulls stay prominent.** Scouts and unladen, unarmed Systems and
    Contact hulls are drawn unobtrusively; a hull with a beam mount or a
    missile tube never is, laden or not (§6.6).

RATIFIED (T-154, the author's rulings after the `beams` replay):

19. **A seat's identity never overlaps the identity of a material.** The
    basics are cyan, magenta and yellow, the supers red, green and blue, the
    apex platinum; no seat is drawn in any of them (§6.2).
20. **The display reflects smooth, continuous acceleration and
    deceleration**, as the sim flies it: no hull moves in steps or eases in
    and out at each frame (§3, §4).

RATIFIED (T-155, the author's ruling):

21. **The game client uses the ratified fictional names for resources**
    (`Hyades_galaxy_and_autopilot.md` §4.1): Cage Ice, Rosepeter and Voltslate
    for the basics, Strange Matter for the apex. The replay keeps the engine's
    color names (`enums.material`, §3), and the client maps them. The supers'
    names are placeholders (T-142), so the client shows a super by its color —
    Red, Green, Blue — until they are ratified.

RATIFIED (T-158, the author's ruling):

22. **A wreck fades to a pinpoint after 250 ms** of wall time (§6.6, §7.1).

RATIFIED (T-159, the author's rulings):

23. **The juicy mode displays hexes** (§7.1).
24. **The juicy mode gives a visual indication of the works trend within a
    hex** (§7.1). *Amended by ruling 25: it shows work-years, not deltas.*

RATIFIED (T-161 – T-163, the author's rulings):

25. **A hex's brightness shows each seat's work-years in it, not a delta**:
    the seat's color, a bright line on a well-established hex, one line per
    seat where seats are intermixed, and a special line where a world holds
    `Band IV` works (§6.5, §7.1).
26. **The tactical mode shows hex brightness too** (§6.5).
27. **In the juicy mode a stack's owner reads while it is under attack, and
    a stack's loss of a hull is visible** (§7.1).
28. **Panning a paused tactical view changes no glyph** — not its size, its
    order, or its shape (§6.4; the bug report behind T-160).

RATIFIED (T-172, the author's ruling):

29. **A count counts only hulls still on their primary role.** A hull the
    autopilot has sent home is counted apart, in a distinct color; a wreck is
    in no count (§6.4, §7.1).

RATIFIED (T-171, the author's ruling):

30. **Whether hulls are one stack is a proximity threshold that follows the
    zoom**, so counts never sit on top of each other at any zoom (§6.4; the
    author's report on `beams`, where a lattice of stack squares put counts
    over one another and regrouped hulls as the view zoomed).

RATIFIED (T-173 – T-178, the author's rulings):

31. **The client targets 30 or 60 frames a second, by platform.** Game years
    per second do not change with the frame rate; frames per game year vary
    with the playback rate (§4).
32. **A path is drawn only for a large stack, and only its next five years
    ahead; a hull's past is drawn only while it is selected.** A path's color
    shows the drive's acceleration, shifted toward blue where the drive gains
    speed and toward red where it sheds it (§6.7). *Retires the routes of
    T-152.*
33. **Tactical mode antialiases, so a faint line can be thinner than a
    pixel** (§6.7).
34. **The juicy mode shows the same paths, in its own style** (§7.1).
35. **The `Band IV` indicator stands still until it is removed** (§6.5,
    §7.1; the author's report of juicy-mode jitter).
36. **The palette is ratified in play**: a replay shows a large gamut of the
    defined palette as the game uses it, and both modes draw in the ratified
    palette (§6.2, §8.4).

RATIFIED (T-179, the author's ruling):

37. **The juicy background is `ink`** — the tone map's darkest tone (§6.2,
    §7.1).

RATIFIED (T-180, T-181, the author's rulings):

38. **The demo games ship as recipes the browser records through the
    engine** — a seed, its settings and its card plays, not a recorded file:
    the engine is bit-deterministic, native and wasm32 alike, so the recipe
    is the replay (§2, §8.1). The author's choice over keeping recorded files.
39. **The `palette` and `forge` demos carry more diversity of built hulls and
    roles, more years, and more resources** (§6.2).

RATIFIED (T-183, the author's ruling — *"I don't want to run the replays
before viewing. I want live simulation while I'm viewing"*):

40. **A demo runs live while it is watched**: the viewer opens on the run's
    first frame and plays while the engine runs ahead of it; nothing is
    recorded before viewing (§2). Resolves R-UI5.

---

## 2. The seam

BUILT. The engine records a run as a **replay** — a self-describing JSON
document (§3) — and the viewer reads replays and nothing else:

```
Simulation ──snapshot_at(t)──▶ record_run ──JSON──▶ Replay ──▶ View ──▶ tactical / juicy
 (engine)                       (engine)                       (viewer crate, wasm32)
```

**The demo games run where they are watched, live** (rulings 38 and 40,
T-180, T-183): each is a recipe in the engine (`hyades_engine::demos` — a
galaxy, its seeded fleets, the cards played at the round barrier and how to
frame the record), and the client runs it through **the engine's browser
module** (`sim/`, `hyades-sim`, compiled to wasm32) in a web worker. The
worker takes the first frame and posts **the replay so far**
(`replay::Recorder::head`, a version-1 replay with `"live": true`), which the
viewer opens on and plays; it then keeps the engine running and posts **what
the run added** about every 100 ms (`Recorder::chunk`, §3.1), which the viewer
appends while it plays. The clock plays up to the last frame that has arrived
and waits there, playing, for the next (`Timeline::ready`); the status line
says how far the engine has run. The module and the native engine write the
same bytes for every demo (appendix §D.63.1), and a run read piece by piece is
the run read whole (`viewer/tests/contract.rs`, appendix §D.64).

- The viewer crate **does not link the engine**. Its only dependency on it is a
  dev-dependency for the contract test (`viewer/tests/contract.rs`), which
  records a run and checks that the viewer reads every world, every hull of
  every frame — owner, hull, Design, role, position — and every event.
- The engine gains no presentation code: `snapshot_at` and `next_event_time`
  are reads of state the engine already holds, and the recorder is pure data
  (no filesystem, no clock).
- The engine runs ahead of playback as fast as it can, so a run a viewer
  catches up with waits only while the engine is slower than the playback
  rate; it is not held to the clock.

---

## 3. The replay format, version 1

BUILT. A replay is one JSON object. Every table names its own fields, so a
reader looks a field up by name and a later version may add fields without
breaking it.

| key | content |
|---|---|
| `format`, `version` | `"hyades-replay"`, `1`. A reader refuses another format or version |
| `meta` | `label`, `seed`, `seats`, `planets`, `horizon_years`, `frame_years`, `hex_side_ly`, `hex_origin` `[x, y]` (ly), `ground`, and optionally `focus` `[x, y, radius]` (ly) — where a viewer opens |
| `seats` | per seat: `archetype` (its native super), `home` (world id) |
| `enums` | `kind` (role), `hull` (the docs' codes, LSV … GOU), `design` (the class names, Meadow … Unnamed), `category` (log categories), `material` (the eight on the Exchange's books: Cyan, Magenta, Yellow, Red, Green, Blue, Apex, Ordnance) |
| `planet_fields`, `planets` | static per world: `id, x, y, z, hab, bio_max, cyan, magenta, yellow, home` — Bands |
| `frame_planet_fields` | per world per frame: `owner` (−1 none), `pop`, `works` — Bands — and `works_kt`, works as a mass, kt to four significant figures (T-159): masses add across worlds where Band readings do not |
| `vehicle_fields` | per hull per frame (positions, of hulls and of worlds, to 4 decimals of a light year since T-154 — at 2, a fight moved in 0.01-ly steps): `id, kind, x, y, z, vx, vy, vz, accel, burn, damage, flags, dest, cargo, settlers, cargo_cyan … cargo_ordnance` — ly, ly/yr, ly/yr², burn ∈ {−1, 0, +1}, damage as a share of structure, flags bit 0 in flight, bit 1 wrecked and bit 2 heading home off its mission (T-172: withdrawn under fire and not yet home), dest a world id or −1, cargo and settlers kt, then the cargo by material, kt, to four significant figures (T-152) |
| `holding_fields` | per holding per frame: `planet, seat, cyan … ordnance` — each material a **Band reading on the cost ladder** (a holding is a stock that can be spent), `null` where none of it is held (T-152) |
| `frames` | `{t, owner[], pop[], works[], vehicles[[…]], holdings[[…]]}` — `holdings` lists every non-empty holding, an empire's at worlds it owns and at rocks it only mines, by planet and then seat |
| `hull_fields`, `hulls` | static per hull: `id, owner, hull, design, beams, tubes, wrecked_at` — written once, after the frames; `wrecked_at` the year it was wrecked to six decimals, or `null` (T-158) |
| `event_fields`, `events`, `events_truncated` | `[t, category, kind, seat, text]` for every event the run's log filter collected, in time order; past the recorder's cap the list stops and the flag says so |

Decisions in the format:

- **Frame `k` is the theater at exactly `k · frame_years`**: every event at or
  before that year applied, every hull where its motion has it then. A frame is
  never taken at "the first event after" its year (§D.58.1 is the defect this
  replaced).
- **A replay may open densely** (T-177): the recorder's `dense = (until,
  step)` takes frames every `step` years up to `until`, then every
  `frame_years` (`replay::frame_times`), so one replay holds a fight frame by
  frame and decades of traffic after it. Each frame's `t` is in the file and
  the viewer reads frames by `t`, so the format is unchanged; `meta.frame_years`
  is the spacing after the opening.
- **Every hull in the theater is in every frame**, a wreck included (it coasts,
  `damage = 1`). A hull recycled for its minerals is not: its mass is in a bank.
- **A wreck carries the time it was wrecked** (T-158), and the viewer shows
  the wreck from that instant rather than from the next frame: a fight's frame
  is 0.02 yr, over a second of wall time at the rate a fight opens at.
- **A number that is not finite is a fatal error** in the recorder (design law
  #16), not a value in the file.
- **One seed records one replay, byte for byte** — a replay is a pure function
  of the run.
- **Cards played** (T-180): a recording carries `(year, orders)` pairs
  (`ReplayConfig::plays`), each applied through `apply_orders` — the one
  inbound channel, as a client would — once the clock reaches its year, which
  a demo sets to the first round barrier.
- Size: OPEN (**R-UI6**). A demo is no longer downloaded (ruling 38), so what
  bounds it is memory: the viewer holds 262 MB after loading the 600-yr,
  18-seat `palette` (42.8 MB of text) and 178 MB after `forge` (appendix
  §D.63.3). Recommendation: a version 2 that writes only the hulls whose state
  changed since the previous frame, when a demo or a replay the author wants
  to keep would pass ~500 MB of viewer memory.

### 3.1 A live run (T-183)

A run the engine is still running reaches the viewer in pieces, in the same
version:

- **The head** is a replay of every frame taken so far — every hull seen and
  every event logged — with `"live": true`. A viewer opens it as it would a
  file. Its field lists are the only ones; the pieces after it carry rows only.
- **A chunk** is one JSON object: `frames` (the frames since the last piece,
  in time order), `hulls` (each hull first seen since, and each whose
  `wrecked_at` became known since, as a whole row that replaces the row with
  its id), `events` (the events logged since), `events_truncated`, and `done`,
  `true` on the last. A frame that does not follow the last is refused.
- **The head and its chunks, read in order, are the whole replay**:
  `Recorder::head` and `Recorder::chunk` share the frame, hull and event
  writers with `Recorder::finish`. The engine and viewer tests check it on
  every demo (appendix §D.64).

---

## 4. Playback

BUILT (`viewer/src/timeline.rs`).

- The clock plays at a **signed rate** in game years per wall-clock second; a
  negative rate rewinds. Reaching either end stops and pauses there.
- **Play at the end it is heading for starts over** from the other end.
- **Seek** holds the clock inside the replay's span; **step** moves to the next
  or previous frame time and pauses.
- Between frames, a hull is drawn on the **cubic Hermite curve** through its
  position and velocity at both frames (`replay::hermite`), and its velocity is
  that curve's (ruling 20). The curve meets both frames' positions and
  velocities, so speed changes smoothly through a frame, and a constant
  acceleration — a drive leg — is reproduced exactly. Every other quantity is
  the earlier frame's.
- A replay opens paused at its start, at a rate that plays it through in **60
  seconds** (*placeholder*), framed on its `focus` when it has one.
- **Frame pacing** (ruling 31; T-173): the page draws at most
  `targetFps` frames a second — **60, or 30 where the primary pointer is
  coarse** (`(pointer: coarse)`, a phone or tablet), or as the link's `fps=30`
  / `fps=60` says — skipping display refreshes faster than that. Each drawn
  frame advances the clock by the wall time since the previous one, capped at
  1 s (a tab back from the background does not jump), so **game years per
  second are the rate whatever the frame rate**, and frames per game year are
  `fps / |rate|`. The status line shows the measured frames in the last
  second against the target, and frames per game year while playing.

---

## 5. The log view

BUILT (`viewer/src/logview.rs`). The replay's events, filtered by every one of:

- **category** (a set), **seat** (a set, and separately whether to admit events
  that name no seat), **kind** (exactly one, or all), **text** (a substring of
  the event's text or kind, ignoring case);
- a **time window** against the clock: the whole log, the log up to now, or
  within a stated span either side of now (the shell offers ±1 frame).

The log **follows the clock**, keeping the last admitted event at or before it
in view; scrolling stops following. **Clicking a row seeks the clock to its
event and pauses.**

---

## 6. The tactical mode

### 6.1 The invariant

BUILT, and pinned by tests (`tactical.rs`,
`every_world_and_every_hull_is_planned_once_at_its_position`,
`every_hull_is_drawn_or_counted_in_a_drawn_stack`; and by the contract test on
a recorded run). **Every world and every hull of the instant shown is laid out
at its projected position**, and **every hull is either drawn or counted in a
stack that is drawn** (§6.4). The plan is computed and tested apart from the
pixels; painting is a second step.

The view is **top-down onto the galaxy plane** (galaxy `x` right, `y` up);
height `z` is not projected and is shown in the inspector. A third axis on
screen is OPEN (**R-UI7**).

### 6.2 The palette — OPEN, awaiting the author's approval (R-UI1)

The tactical palette is the author's 40 colors (`hyades_palette.js`: eight base
tones, eight accents each with tints `3`, `2` and shade `1`), held verbatim in
`viewer/src/palette.rs::SOURCE`, passed through a tone map. **Every value below
is a proposal**, to be ratified on the live viewer (ruling 13, §6.2.1). The
client's **Palette** screen (§8.1) is a reference sheet of the same values —
each source color beside its mapped color, every role, seat and status color —
and shows a link's settings too.

**The palette is seen in play** (rulings 36 and 39, T-177, T-181):

- **`palette`** is eighteen seats — every seat color the largest fair table
  uses — with a picket stack of each on a ring beside seat 0's homeworld, half
  of them closing on the middle to brake, fight, take hits, wreck and withdraw
  past a sentry post; then **six centuries** of every seat's traffic: scouts,
  miners, freighters and colonizers, settlers, holdings and established hex
  lines. Each seat plays one of six cards at the 200-yr barrier (Warfare,
  missile, Scarp unlock, Growth, Production, ecology, by turns), so seats
  build Cairn pickets and Butte sentries; every seat's twin Designs are billed
  0.15 in each super and 0.05 in the apex, so the three supers and the apex
  are held from 330–350 yr and carried. Its first year is framed every
  0.02 yr (§3); watch it at ×1/16.
- **`forge`** is six seats over **eight centuries**, with a surveyed start
  (10 ly) and a parade of every hull and named Design on its role generated
  with the galaxy — Spur and Tor scouts, Delta, Range and Scarp colonizers,
  Meadow miners, Ford and Strait freighters, Cairn and Mesa pickets, Butte
  sentries, and the four hulls no Design names — plus the same cards and the
  same super-billed twins. Forges stand from 330–400 yr, every one of the
  eight materials is carried, and a world holds `Band IV` works from 740 yr.

Measured coverage is in appendix §D.63.2. **No yard built a Range or Scarp
colonizer** in either demo; the two in `forge` are generated with the galaxy.
The autopilot takes the General rung only when a center can afford it and it
delivers more settlers per mineral than the Medium one
(`BaselineAutopilot::production_choice`).

**The proposed tone map, "Earthrise"**, in OKLCH (Björn Ottosson, *A perceptual
color space for image processing*, 2020):

| symbol | name | proposed value | what it does |
|---|---|---|---|
| `ink`, `paper` | lightness range | 0.02, 0.95 | lightness `L` is compressed into `[ink, paper]`: the darkest tone is an ink and the brightest a paper, not pure black or white |
| `warm_chroma`, `cool_chroma` | chroma scale | 0.92, 0.6 | chroma `C` is scaled by a factor running from `warm_chroma` at hue 70° (ochre) to `cool_chroma` at 250° (blue): the muted, warm-leaning saturation of 1970s offset print |
| `pull`, `anchors` | hue pull | 0.3; 38°, 78°, 118°, 228° | hue `h` moves a share `pull` of the way to the nearest anchor — terracotta, harvest ochre, avocado, faded sky |

Out-of-gamut results are brought in by reducing chroma at fixed lightness and
hue. `ink = 0.02` is the highest value tried at which every one of the first
eighteen seat colors still reaches 3:1 contrast on the ground (§D.58.3).

#### 6.2.1 The live editor

BUILT (T-150). The viewer's side panel tunes every value in this section while
the game is drawn — the tone map's six parameters and four anchors, the glyph
dimming of an unarmed glyph's hollow body (`fill`, proposed 0.7, §6.3), and any of the 40 palette colors
or 12 status colors set by hand (a hand-set color replaces the tone map's
result for that name, and every role and seat drawn from it follows). Each
change redraws both modes.

**The settings are one line of text** (`viewer/src/palette.rs::Settings`),
owned by the module — the page sends it and reads back the canonical form:

```
ink=0.02 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228 fill=0.7 hy_red=#c83a2c Hit=#ff2d6f
```

Keys left out keep the proposal; a key that is unknown, a value out of range
(`ink` and `paper` 0–1 with `ink` below `paper`, chroma scales 0–2, `pull` and
`fill` 0–1, anchors 0–360°) or a malformed color refuses the whole line and
names the key. The line rides in the page's link (`#palette=…`), so a tuned
palette can be reopened, sent and ratified as it was seen; the replay picker
and reloads keep it. **To ratify, the author sends the line or the link**; the
values then replace the proposal in `palette.rs` and this section, and
`PALETTE_STATUS` becomes `ratified`.

**The proposed assignments:**

| role | source color |
|---|---|
| ink — the juicy background (ruling 37) | none: a neutral at OKLCH lightness `ink`, chroma 0 — `#000000` at the proposed `ink = 0.02`, `#222222` at 0.25 |
| ground, panel, grid, hex lines | base03, base02, base01, base02 |
| text (dim, normal, bright) | base0, base2, base3 |
| a world (unowned) | base01 |
| the basics cyan, magenta, yellow | cyan, magenta, yellow |
| seat `i` | **off the palette** (ruling 19): OKLCH hue `SEAT_HUES[i mod 4]` = 300°, 160°, 335°, 280°, lightness `SEAT_LIGHTNESS[⌊i/4⌋ mod 5]` = 0.70, 0.60, 0.78, 0.65, 0.74, chroma 0.13 brought into gamut |
| Doctrine roles | Scout base3, Colonizer green3, Miner yellow3, Freighter orange3, Picket violet3, Sentry magenta3, Reserve base0, Scrapped base01 |

Seats take the hues between the materials' (magenta 8°, red 30°, rounds 39°,
yellow 83°, green 118°, cyan 200°, blue 240° on the proposed palette): every
seat is at least 25° of hue and 0.08 in OKLab from every material, and every
two seats are at least 0.03 apart (`no_seat_shares_an_identity_with_a_basic_super_or_the_apex`,
appendix §D.58.7). An empire no longer wears its archetype's super; that
identity is the material's alone.

**The proposed status colors — off the palette on purpose.** Each is at least
0.04 from every palette color and 0.06 from every other status color in OKLab,
and reads at 3:1 on the ground (`status_colors_sit_off_the_palette_apart_and_legible`):

| status | color | drawn as |
|---|---|---|
| Hit | `#ff2d6f` | a ring around a hull damaged since the previous frame |
| Damage | `#ffb703` | a bar beside the hull, its length the share of structure lost |
| Drive | `#4df3ff` | the drive vector of a hull burning |
| DriveHot | `#b8ff3d` | the drive vector at or above 1 g (`HOT_ACCEL`, *placeholder*) |
| Braking | `#a77bff` | the drive vector of a hull braking |
| Wreck | `#9a7b5c` | a wreck's body |
| Laden | `#fff36b` | the fill of a laden hull whose replay does not carry its cargo by material |
| Combat | `#ff4fe1` | the log's combat rows |
| Selected | `#f2fff6` | the selection's corner brackets |
| Retreat | `#ffaba1` | the count of a place's hulls heading home, and their juicy cluster (T-172): OKLCH L 0.88, C 0.16, 25° clamped to sRGB, the point farthest from every other status, the materials, the seats and the text |
| BlueShift | `#006bf0` | the end a path's color moves toward where the drive gains speed (§6.7, T-174): OKLCH L 0.56, C 0.22, 259° clamped to sRGB |
| RedShift | `#ff664b` | the end a path's color moves toward where the drive sheds speed (§6.7, T-174): OKLCH L 0.70, C 0.21, 32° clamped to sRGB |

**Materials** are drawn in the palette's own hue of their names — Cyan
`hy_cyan`, Magenta `hy_magenta`, Yellow `hy_yellow`, Red `hy_red`, Green
`hy_green`, Blue `hy_blue` — with Strange Matter **platinum** (ruling 19:
OKLCH L 0.84, C 0.012, h 250, `palette::PLATINUM`), rounds
`hy_orange` and settlers `hy_base2` (proposed, part of R-UI1). They color a
holding's bars and a hold's stripes (§6.5, §6.6).

**Both modes draw only in the palette** (ruling 36): every color the tactical
raster, the juicy light list and the page chrome use is a role, seat, status
or material color of `Palette`, so the live editor and a ratification reach
both — the juicy background included, which is the `ink` role (ruling 37,
§7.1).

What is tested and holds whatever is approved: text at 4.5:1 on the ground and
on a panel (WCAG 2 AA), dim text at 3:1, eighteen seats each at 3:1 on the
ground and at least 0.03 apart in OKLab, and every role accent at 3:1.

### 6.3 The glyph grammar — OPEN (R-UI3)

Proposed, and built as proposed (`viewer/src/glyph.rs`), revised at T-153
after the author could not tell armed from unarmed hulls, nor role at a glance:

| what | read from | drawn as |
|---|---|---|
| hull family | the hull code | Systems a square, Contact a diamond, Offensive a triangle |
| hull size | the hull code | Limited 9, Medium 11, General 13 tactical pixels across |
| **armed or not** | the hull's beam and tube mounts | **an armed body is solid in the seat's color; an unarmed body is hollow**, the seat's color 70% toward the ground (`fill`, *placeholder*) — and a two-pixel spike ahead for beams, a pixel either side for tubes |
| **Doctrine role** | the role | **a 3×3 mark at the body's center, one shape per role**: Colonizer `+`, Miner `│`, Freighter `─`, Picket `×`, Sentry `□`, Scout a dot, Reserve the four corners (`glyph::role_mark`) — in the role's accent on a hollow body and in the ground's color on a solid one, so it reads on either; a triangle's mark sits a pixel low, where its interior is |
| cargo | the hold, by material | an unarmed body striped whole, an armed body's bottom two rows (§6.6) |
| Design class | the class | a 4-bit tag under the shape: the class's index plus one; Unnamed has none |
| seat | the owner | the outline in the seat's color |

Every Design reads differently on every hull, the three families differ at
every size, and every role's mark differs from every other
(`every_design_reads_differently_on_every_hull`,
`every_role_mark_differs_from_every_other`). The side panel's legend draws each
mark from the module (`hv_text(11)`), so it is the mark the renderer stamps. A
tactical pixel is two screen pixels (*placeholder*), scaled up without
smoothing.

### 6.4 Stacks and the display order — OPEN (R-UI4)

RATIFIED (T-152, the author's ruling): **a glyph is drawn only centered on its
hull's own position.** Nothing is offset from where it stands and no leader
line joins a glyph to a place; where glyphs overlap, a **display order**
decides which is drawn on top. The fan this replaced is in appendix §D.58.6.

Proposed and built, every magnitude a *placeholder*:

- **Places** — RATIFIED (ruling 30, T-171): **hulls whose footprints overlap
  on screen are one place**, joined transitively (`tactical::places`). A
  footprint (`tactical::footprint`) is measured in tactical pixels about the
  hull's own pixel: below the galaxy level, its glyph with the damage bar to
  the left and, unless it is a wreck, room to the right for its place's two
  counts at `COUNT_ROOM_DIGITS` = 2 digits each (placeholder); at the galaxy
  level, the largest marker; for a quiet hull (§6.6), its pixel and the ring
  about it. Because the reach is a fixed number of pixels, in light-years it
  shrinks as the view zooms in: a fight parts into its hulls and a galaxy
  gathers into clusters, zooming in only parts places, and **no count is
  drawn over another place's glyph or count**. A place with a count of 100 or
  more draws it up to four pixels past its footprint. The square lattice this
  replaced (3 / 5 / 8 tactical pixels at System / Sector / Galaxy) put hulls
  a pixel apart in two places wherever a square's edge fell between them, and
  moved its edges as the view zoomed; appendix §D.59.
- **Stacks.** In one place, hulls **with one owner, Design, role, wreck state
  and quietness** (§6.6) are one glyph, drawn where its lowest-id hull
  stands. The glyph carries the worst damage, any hit, the sum of its
  members' cargo and any selection. **A wreck never stacks with a live hull,
  at any level**, so a galaxy-level marker is sized by live hulls alone
  (T-172).
- **A place's counts** — RATIFIED (ruling 29, T-172): beside the glyph drawn
  on top of a place, the number of its hulls **still on their role**, in the
  bright text color, then the number **heading home** in the retreat color
  (`Status::Retreat`); a wreck is in neither. The first is drawn when it is
  more than one or when hulls are also heading home, the second when any are.
  The engine marks a hull heading home from the moment it withdraws until it
  arrives home or is wrecked (`VehicleSnapshot::withdrawing`, flags bit 2).
  The first count starts `COUNT_OFFSET_PX` = 6 tactical pixels past the
  glyph's half body, clear of the hit ring and the selection brackets.
  **Places are found in pixels of the galaxy at the view's scale, not of the
  screen** (T-160): a pan regroups nothing, and only a zoom changes the
  stacks. On a screen-fixed lattice a paused view flickered as it was
  panned — hulls crossed square edges, and stacks, counts, marker sizes and
  the glyph drawn on top all changed.
- **Positions are rounded on a lattice fixed to the galaxy** (T-160,
  `tactical::pixel_of`): a point's tactical pixel is the galaxy rounded at the
  view's scale, plus the pan as a whole number of pixels. A pan moves
  everything drawn by the same number of pixels, or none. Rounding each
  glyph's own screen position let two stacks less than a pixel apart share a
  pixel or not as the view moved, so their outlines merged and parted.
- **The display order** (`tactical::display_order`), bottom to top: wrecks,
  then quiet hulls (§6.6), then the rest; within each, by role — Reserve and
  Scrapped, Scout, Miner, Freighter, Colonizer, Sentry, Picket, so the fighting
  roles are on top; then by hull size, the smaller over the larger so both stay
  visible; then by distance from the seat's homeworld, the frontier over home.
- **A place's counts** cover every hull in the place, whatever its owner, and
  are drawn once, in a 3×5 pixel font, beside the glyph on top.
- **At the Galaxy level a place's hulls are grouped by owner and by whether
  they are armed**, and each group is drawn as a **marker**: a square in the
  seat's color, solid for armed hulls and hollow for unarmed, 3, 5, 7 or 9
  tactical pixels across for 1, 2–9, 10–99 and 100 or more hulls, centered on
  its lowest-id hull. A group of quiet hulls is one dim pixel.
- A pick selects the nearest drawn glyph within a radius (§8.3), the one on top
  where two are equally near; **the inspector lists what the place is made
  of** — the count of each Design, hull and role — and then the hull picked.

### 6.5 Worlds, hexes and levels of detail

BUILT, magnitudes *placeholders*. **A colony is a disc in its owner's seat
color, a pixel wider than an unowned world** at every level (T-152, the
author's direction: colonies by seat, by color); an unowned world is a dim dot;
a homeworld is a pixel wider again, with a ring.

**Holdings** (T-152, the author's direction): under each world, one group per
seat holding there — a base line in the seat's color under one vertical bar per
material on the Exchange's books, in book order (Cyan, Magenta, Yellow, Red,
Green, Blue, Apex, Ordnance). **A bar is two tactical pixels per Band** of the
holding's cost-ladder reading (`BAR_PX_PER_BAND`), at least one pixel for
anything held and capped at Band V, 10 pixels — the replay carries the Band
reading, so the bar is the engine's own Band math (§3). Holdings are drawn at
the Sector and System levels; at the Galaxy level they would cover the field.
The inspector lists a selected world's holdings by seat and material, as Bands.

*Superseded (T-152 → T-174):* every hull in flight drew a **route** to its
destination world. With hundreds of hulls in flight the lines covered the
field, and the author found the view illegible; paths (§6.7) replace them.
Appendix §D.62.2.

The command
view's hexes (flat-top, `hex_side_ly`, one centered on `hex_origin` — galaxy
§2) are drawn when a hex is at least 6 tactical pixels across, and **only the
active ones: a hex holding a world or a hull** (`tactical::active_hexes`). A
point's hex is found by axial coordinates with cube rounding
(`Replay::hex_of`). The level of
detail is set by the camera's scale: **Galaxy** below 2 screen pixels per ly,
**Sector** to 40, **System** above; worlds grow a pixel per level.

**Hex brightness** — RATIFIED that a hex shows each seat's work-years in it
(rulings 25, 26); the form is proposed (R-UI4, T-162), every magnitude a
*placeholder*:

- **Work-years** (`Replay::fill_hex_works`, `tactical::hex_lines`): for each
  hex and seat, the works on the seat's worlds there, as a mass (`works_kt`,
  §3), integrated over the replay by the trapezoid between frames and run
  smoothly between them.
- **A line per seat with works standing in the hex**, nested inward
  `HEX_LINE_STEP_PX` = 4 screen pixels apart, the seat with the most
  work-years outermost. Its color is the seat's, from 25% of the way from the
  ground (`SEAT_LINE_FLOOR`) at no work-years to the full color at
  `ESTABLISHED_WORK_YEARS` = 100 kt·yr — a kiloton of works, `Band II`,
  standing a century. **An established line** is tinted 35% toward the bright
  text color (`ESTABLISHED_TINT`): the bright line.
- **A `Band IV` world**: the hex's edge is dashed in the bright text color
  (`BAND_IV_DASH` = 3 on, 2 off) in place of the grid's line. The dashes are
  counted from the hex's own corner, so they stand still as the view pans,
  follows or zooms, until the world drops below `Band IV` (ruling 35). No
  world in the 400-yr `expansion` replay reaches it at this commit (appendix
  §D.62.3); the `forge` replay starts six homeworlds there.

### 6.6 Quiet hulls and cargo — OPEN (R-UI4)

Proposed and built (T-152, the author's direction):

- **Quiet hulls.** A scout, and an unladen, unarmed hull of the Systems or
  Contact family, is **one tactical pixel in its seat's dimmed color** — no
  glyph, no drive vector, no count — and is drawn under every other live hull.
  A hit still rings it and a selection still brackets it. **An armed hull is
  never quiet** — RATIFIED (ruling 18, T-152): the `beams` replay's whole
  fight is unladen Contact pickets, which the unarmed condition keeps visible
  (appendix §D.58.6).
- **Cargo.** A laden hull's body is striped vertically, one stripe per
  material in its hold and then its settlers, each as wide as its share of the
  cargo's mass, in book order, in the material colors (§6.2) — the whole body of
  an unarmed hull, and the bottom two rows of an armed one, whose solid body
  says it is armed (a sentry carries rounds). The inspector lists the hold by
  material, kt, and a world's ore and holdings by material, all by in-game
  name (ruling 21). The page's glyph legend keys each stripe color to its
  material's name, from the module (`hv_text(12)`).
- **Wrecks** — RATIFIED (ruling 22, T-158). A wreck's glyph is drawn in the
  wreck color, unstriped, and fades toward the ground over
  `WRECK_FADE_SECONDS` = **0.25 s of wall time** at the playback rate, leaving
  a pinpoint: one pixel in the wreck color, which it stays. The fade is read
  from the clock — `1 − (t − wrecked_at) / (|rate| · 0.25)` — so playing,
  rewinding and seeking show the same frame at the same instant. A wreck in a
  replay without wreck times is a pinpoint.

### 6.7 Paths — OPEN (R-UI4)

RATIFIED that paths are drawn for large stacks only, five years ahead, with a
hull's past only while it is selected, colored by the drive with a blue and a
red shift, and antialiased (rulings 32–34, T-174). The magnitudes are
proposed, each a *placeholder* in `viewer/src/paths.rs`:

| symbol | name | proposed value | what it does |
|---|---|---|---|
| `PATH_MIN_STACK` | a large stack | 5 hulls | a drawn stack of at least this many hulls that is not a wreck draws its lead hull's path ahead |
| `PATH_AHEAD_YEARS` | how far ahead | 5 yr (ruling 32) | the path ahead runs from now to five years on, or to the hull's wreck if sooner |
| `PATH_SAMPLES`, `PATH_PAST_SAMPLES` | samples | 64, 128 | instants sampled along a path ahead and along a selected hull's past |
| `PATH_ALPHA` | opacity | 0.7 | a path's opacity at its hull; ahead it fades to 0 at its far end, the past to a quarter at the hull's first frame |
| `PATH_COAST` | the coasting gray | 0.45 | a coasting stretch is this share of the way from the ground to the text color |
| `PATH_ACCEL_HALF` | half shift | 0.25 g | the acceleration at which a burning stretch is half way from the coasting gray to its shift |

- **What is drawn**: the path ahead of every large stack, and for a selected
  hull its past from its first frame and, if its stack drew none, its path
  ahead. Small stacks and wrecks draw nothing.
- **Where**: the replay's own motion — the same Hermite curve between frames
  a hull is drawn on (§4; `Replay::track`), so a path and its hull cannot
  disagree. A path ahead reads frames the clock has not reached; a replay is
  recorded whole, so the future is in it.
- **Color** (`paths::drive_color`): each stretch takes its frame's drive —
  acceleration `a` (ly/yr²) and burn sign. Coasting is the gray; a burn gaining
  speed is moved toward BlueShift and one shedding it toward RedShift by
  `a / (a + PATH_ACCEL_HALF)` of the way (§6.2). So the color says how hard,
  and the hue which way relative to the velocity.
- **Tactical**: lines from Xiaolin Wu's antialiasing algorithm (Wu 1991), each
  pixel's share of the line blended over what is drawn, so a faint line can be
  thinner than a pixel (ruling 33). Points are placed on the same pixel lattice
  as glyphs (`tactical::subpixel_of`), so a path meets its glyph's center.
  Drawn over the hex grid and under every glyph.
- **Juicy**: the same paths as a string of lights one hull radius apart,
  radius half a hull's, at 0.6 times the path's opacity, in its colors
  (ruling 34, §7.1).

---

## 7. The juicy mode

### 7.1 The method

BUILT. **Every entity is a light** added into a linear high-dynamic-range
buffer; bright light blooms into its neighbors; one tone curve brings the sum
to display range. The lights come from the tactical plan (§6.1), so the two
modes cannot disagree about where anything is.

| entity | light |
|---|---|
| a world | a faint star: a small white core and a fainter halo; an unowned world at half an owned one's |
| an owned world | the star, plus a wide dim tint in its seat's color, stronger at a homeworld — at the Galaxy level an empire reads as a faint colored nebula |
| a hull | a point in its seat's color; a quiet hull (§6.6) a quarter as bright |
| a burning drive | a plume behind the hull (ahead of it when braking), in the drive's status color |
| a hit | a flash in the hit color |
| a wreck | a dim ember, narrowing to a pinpoint (radius 0.5 px) as its tactical glyph fades (§6.6) |
| a hex | a dim line of lights just inside its edge, in the hex color; brighter and whiter as the works in it grow, darker as they fall |
| a path (§6.7) | a string of small lights along it, one hull radius apart, in its colors and fading as it does (`PATH_LIGHT_SPACING` = 1, `PATH_LIGHT_R` = 0.5 hull radius, `PATH_LIGHT` = 0.6, at most `PATH_MAX_LIGHTS` = 2,000 a frame) |

A light of intensity `I` and radius `r` adds `I / (1 + d²/r²)²` at distance
`d`, cut at `4r`. The bloom is the buffer averaged in 4×4 blocks, box-blurred
(radius 2, three passes) and added back at weight 0.35. The tone curve is
`t = 1 − e^(−x)` per channel, laid over the background — **the palette's
`ink`** (ruling 37, T-179) — as `g + (1 − g)·t` in linear light, where `g` is
the ink's linear value in that channel; then sRGB encoding. No light shows the
ink, and no light, however bright, passes white. Every magnitude here is a
*placeholder* (`juicy.rs`'s constants); the light sizes grow with the level of
detail (`light_scale`).

**The hulls carry the scene and the worlds recede** (proposed, R-UI2): the
author found that bright worlds made everything illegible.
`a_hull_outshines_any_world_but_a_homeworld` pins that a lone hull renders
brighter than any lone world other than a homeworld (appendix §D.58.5).

**Hexes** — RATIFIED that juicy mode shows hexes and each seat's
work-years in them (rulings 23, 25); the form is proposed (R-UI2, T-159,
T-162):

- **Which hexes, and when**: the tactical grid's — every hex a world or a hull
  stands in, while a hex is at least `MIN_HEX_PX` tactical pixels across
  (`tactical::hexes_shown`, one predicate for both modes).
- **The grid's line**: lights of radius 1 px every 1.5 px, inset 2 px from the
  edge so a shared edge reads as two lines, one per hex, in the hex color at
  `HEX_GLOW` = 0.1; dashed in the bright text color at `HEX_BAND_IV_GLOW` =
  0.6 where a world holds `Band IV` works. Only the part of a line on screen
  is lit, so a hex many screens wide costs what one does. **The dots and
  dashes are counted from the edge's own corner** and only those on screen
  lit (ruling 35, T-176): counted from where the screen cut the edge, they
  slid along it whenever the view moved, which was the `Band IV` jitter the
  author reported (appendix §D.62.3).
- **A line per seat**, nested inward as in tactical mode (§6.5), in the same
  color, at `HEX_SEAT_GLOW` = 0.25 times its brightness — twice that
  (`HEX_ESTABLISHED`) once established.
- *Superseded (T-159 → T-162):* the line first showed the works **trend** — a
  hex's works against four frames earlier, brightening and whitening as they
  grew. The author ruled that the brightness shows work-years, not deltas
  (ruling 25). Appendix §D.59.

**Stacks below the galaxy level are clusters** (ruling 27; proposed, R-UI2,
T-163): each seat's hulls in a place (§6.4) are a sunflower spiral of dots,
one per hull, `CLUSTER_DOT_SPACING` = 2 hull radii apart, each a hull's light
in the seat's color; past `CLUSTER_MAX_DOTS` = 64 the dots brighten instead.
A lone hull is one dot where it stands, so losing a hull loses a dot.
Clusters that would overlap on screen are one place; in a place each seat's
hulls are one cluster, and where seats share it the clusters stand apart
around the place's middle, `CLUSTER_GAP` = 3 hull radii apart at their
nearest. **A hit is a ring** of lights about the cluster in the hit color
(`HIT_RING` = 1.2 each), not a flash over it, so the seat's color shows
inside the ring. **Hulls heading home are a cluster of their own in the
retreat color** (ruling 29), so a seat's cluster counts only its hulls on
their role. Wrecks and quiet hulls are single lights where they stand;
at the galaxy level every hull is a light where it stands. Tactical mode is
unchanged: there a glyph stands on its hull (ruling 16).

### 7.2 Style — OPEN (R-UI2)

RATIFIED that it is to be found rather than specified, with lighting at its
heart. What is built is a first pass to look at, not a proposal for the style.
Recommendation: settle it by looking at the deployed site against recorded
games, and record each decision here as it is made.

### 7.3 On the GPU

BUILT. The module computes the **light list** (`juicy::lights`); the shell
draws it with **WebGL2** (`web/gpu.js`): instanced additive quads into a
half-float target, the block average, the separable blur, the territory glow
into the bloom target, and a composite pass with the tone curve over the ink.
The constants come from the module, so the two renderers share them; the ink
moves with the live editor, so the shell reads it every frame
(`hv_juicy_ground`).

**The module's CPU renderer (`juicy::rasterize`) is the reference** and the
fallback where a browser has no WebGL2 or no renderable half-float target. The
browser test renders one frame both ways and fails if they differ by more than
a mean of 1 code, a 99th percentile of 4 or a maximum of 24 (§D.58.4).

Tactical mode is rasterized by the module pixel for pixel; the GPU scales it
up, nearest-neighbor.

Rendering juicy at the device's pixel density rather than at CSS pixels is
OPEN — part of R-UI2.

---

## 8. The client and the site

BUILT (T-149, T-150; the client, phone layout and touch at T-151).

### 8.1 The client

The site is one page, the game client, showing **one screen at a time**
(ruling 15):

| screen | what it holds | link |
|---|---|---|
| **Menu** | New game, Replays, Palette | `./` |
| **New game** | the relay test (T-169): make a link, open one, Open games, the room, the match, the relays and the log; saves the diagnostics and the match record (`docs/Hyades_sessions_discovery_and_security.md` §11) | `?view=new`, or a room link `#j=<payload>` |
| **Replays** | the demo games, each run live in the browser when opened (§2), and "Open a replay file…" | `?view=replays` |
| **Palette** | the palette sheet (§6.2) | `?view=palette` |
| **Viewer** | one replay, with a back button to the menu (and Esc) | `?replay=<name>` |

The browser's back and forward buttons move between screens. The palette's
settings ride in the link's hash on every screen (§6.2.1), so a tuned palette
survives moving between them; the palette writes its own key and keeps the
hash's others, because a room link rides there too (`#j=`). The former
`palette.html` forwards to the Palette screen with its hash.

### 8.2 Layout

On a wide screen the viewer has a header, the theater, a side panel (the
selection, the seats, the palette editor, the keys) and a footer (the transport
and the log). The header's **Panel** and **Log** tabs hide and show the side
panel and the log.

**On a phone (760 CSS pixels wide or less)** the theater takes the screen
between a one-row header and the transport. The side panel and the log open as
**sheets over the bottom of the theater**, one at a time, so opening one does
not resize it; a selection is shown in a small box at the theater's top left,
which opens the panel. Nothing scrolls sideways. The browser test asserts on a
Pixel 7 emulation that the theater is at least 60% of the screen's height
(*placeholder*) and that nothing scrolls sideways.

### 8.3 Input

- **Keys**: Space, J/K/L, ←/→, T (mode), F (fit everything), G (the replay's
  focus), C (center on the selection), Esc (menu).
- **Direction** (T-157): ▶ plays forward and ◀ backward. Each pauses when the
  replay is already playing its way and otherwise turns the replay and plays,
  so neither direction can be left stuck. Space pauses whichever way the
  replay is playing, and plays forward when it is paused; L and J play forward
  and backward, K pauses.
- **Pointer**: the wheel zooms about the pointer; one pointer drags the view;
  **two fingers pinch** to zoom about their midpoint and pan with it. A press
  that moves no farther than a slop (4 CSS pixels for a mouse, 8 for a pen, 10
  for a finger) is a **pick**, which selects the nearest drawn stack within a
  radius (10 CSS pixels for a mouse, 14 for a pen, 24 for a finger — a finger
  covers more of the screen than a cursor), else the nearest world. Every
  magnitude is a *placeholder*.
- **Robustness**: the viewer's module and the engine's module are fetched up to
  four times on a server error (5xx) or a dropped connection, 0.5, 1 and 2 s
  apart; a load that still fails says why and offers "Try again". A WebGL2
  error or a lost context moves drawing to the module's CPU renderer for the
  rest of the session (§7.3). A demo opens on its first frame while the
  engine runs the rest (§2); a run that fails says why and offers "Try again".

### 8.4 The parts

- **`viewer/`** — Rust, no dependencies today (ruling 14 permits them; R-UI8), compiled to `wasm32-unknown-unknown`
  as a module with no imports. Its plain C interface (`viewer/src/ffi.rs`,
  `hv_*`) takes and returns numbers; bytes cross through one input buffer, the
  framebuffer and one text buffer. **Everything the page shows as text is
  formatted by the module.**
- **`net/`** — `hyades-net`, Rust with upstream packages (ruling 14), the
  sessions spec's transport as a state machine plus a `wasm-bindgen` layer
  (R-UI8's first use of it). It does not link the engine. Built by
  `web/build.sh` into `net/` beside the page; the `wasm-bindgen` CLI must match
  `net/Cargo.toml`'s exact pin (`web/install-wasm-bindgen.sh` installs it).
- **`web/`** — `index.html` (the client's screens), `style.css`, `shell.js`
  (screens and the link, input, layout, the log as a virtual list), `gpu.js`
  (§7.3), `newgame.js` (the New game screen; loaded only when that screen
  opens, so the replay viewer never fetches the networking module), and
  `palette.html`, which forwards to the Palette screen. A
  Content-Security-Policy meta tag pins scripts to the site and connections to
  the pinned relays and loopback (sessions spec §4.5 rule 2, §8). The page
  chrome takes its colors from the module's palette, so chrome and canvas
  cannot differ.
- **Controls**: play forward/pause, play backward/pause (§8.3), step, a rate multiplier, a scrubber, and
  the input of §8.3; the side panel's palette editor tunes the palette live
  (§6.2.1).
- **`sim/`** — `hyades-sim`, the engine's browser module (§2, T-180): links
  the engine and nothing else, compiled to wasm32 as a module with no imports,
  with a plain C interface (`hs_demos`, `hs_begin`, `hs_step`, `hs_head`,
  `hs_chunk`). `web/sim-worker.js` runs it live (§2): the shell compiles the
  module once, gives each run a worker of its own, and ends the worker when
  the run is done or another replay opens, because a module's memory never
  shrinks. The viewer's interface takes a piece with `hv_append`.
- **`web/build.sh <out> [--quick]`** assembles the site: the two modules, the
  page and the worker, and `demos.json`, which says whether the site's demos
  are the quick ones the tests open. No replay is recorded at build time;
  `examples/record_replay` writes the demos to files for opening as files.

**Tests**: the viewer crate's unit tests; the contract test (§2);
`web/test/smoke.mjs <site>`, which runs every demo live through the engine's
module as the worker does — opening the viewer module on the head, appending
each chunk and drawing the newest frame — and drives the viewer headless over
each in both modes; and `web/test/browser.mjs`, which opens the site in headless
Chrome, goes from the menu through Replays to a demo, checks it plays as it
opens while the engine runs it, waits for the run to arrive whole, checks it renders on
WebGL2 without a console error, checks the glyph legend shows every role's
mark and keys the materials by in-game name, clicks a drawn hull and checks it is selected,
compares GPU and CPU juicy on the proposed palette and over a visible ink, and checks the background is the ink on both (§7.3), plays, filters the log and seeks from a row,
drives the palette editor (a link's settings open with the page; a slider
redraws the canvas and rewrites the link; a hand-set color reaches the settings
line; reset returns to the proposal), returns to the menu and opens the
Palette screen, follows the old `palette.html` link, answers the engine's module
503 once and checks the demo runs on the second request, loses the WebGL context and checks
the CPU renderer draws and a click still selects, and on a Pixel 7 emulation checks the
theater's size, that nothing scrolls sideways, and that a tap beside a drawn
hull selects it. `web/test/relay-match.mjs` runs a loopback Nostr relay that
rate-limits writes with a backoff hint, and plays a three-seat match in three
headless tabs through the New game screen (sessions spec §11).

**Deployment**: `.github/workflows/pages.yml` builds and tests the site on
every push to `main` and publishes it to GitHub Pages. The repository's Pages
source must be set to **GitHub Actions** once (Settings → Pages). CI's `viewer`
job runs the same build and tests on every pull request.

---

## 9. Register

| code | decision | status | what would settle it |
|---|---|---|---|
| **R-UI1** | the tactical palette: the tone map's parameters, the role and seat assignments, the status colors, the glyph fill (§6.2) | OPEN — proposal built, live editor built, `PALETTE_STATUS = "proposed"`; the `palette` and `forge` replays show it in play (ruling 36, T-177) | the author's ratification on the live viewer: a settings line or link (§6.2.1); its values then replace the proposal and `PALETTE_STATUS` becomes `ratified` |
| **R-UI2** | juicy mode's style (§7.2), including its resolution on high-density screens | OPEN — first pass built; worlds dimmed at T-151 after the author found them illegible; hexes and the works trend added at T-159 with placeholder magnitudes (§7.1); paths added at T-175; the background is the palette's `ink` (ruling 37, T-179) | the author's judgment against recorded games |
| **R-UI3** | the glyph grammar (§6.3) | OPEN — revised at T-153: armed bodies solid and unarmed hollow; a 3×3 role mark per role; glyphs a pixel wider each side; a legend | the author's review on the deployed site |
| **R-UI4** | stacking and the tactical layer: the room a footprint keeps for counts (ruling 30 ratified places as overlapping footprints), the Galaxy-level owner markers, the display order, the place count, paths, holding bars, quiet hulls and cargo stripes (§6.4–§6.7) | OPEN — revised at T-152: glyphs only where their hulls stand (ratified), the fan replaced by a display order; holdings, colonies by color, quiet traffic and cargo composition added at the author's direction; armed hulls stay prominent (ruling 18); routes replaced by paths at T-174 (rulings 32–33), their magnitudes proposed | the author's review on the deployed site; a census of stack sizes on a long replay if counts prove unreadable |
| **R-UI5** | watching a game live (§2) | **RESOLVED** (ruling 40, T-183): a demo runs live while it is watched — the head and its chunks (§3.1) | — |
| **R-UI6** | replay size (§3) | OPEN — recommendation attached; demos are no longer downloaded (ruling 38), so viewer memory bounds them | the first demo or replay the author wants to keep that would pass ~500 MB of viewer memory |
| **R-UI7** | showing height `z` in the galaxy view (§6.1) | OPEN | the author's ruling on whether the view stays top-down |
| **R-UI8** | which upstream packages the interface takes up now that it may (ruling 14) | OPEN — recommendation attached; `wasm-bindgen`/`web-sys` taken up by `net/` at T-169 (the author's choice), the viewer still on its C interface | a need the current code cannot meet. Recommendation: none yet; the first candidates are `wasm-bindgen`/`web-sys` (to retire the hand-written C interface and `shell.js`'s byte copying) and `wgpu` (to move `gpu.js` into Rust), each taken when a change would otherwise be built on the hand-rolled path |
| **R-UI9** | the client's phone layout and touch magnitudes: the 760-pixel breakpoint, the sheets, the tap slop and pick radius (§8.2–8.3), and which platforms target 30 frames a second (§4) | OPEN — proposal built; 30 fps where the primary pointer is coarse (T-173) | the author's use of the deployed site on a phone |

## References

- `AGENTS.md` §4 (design law #15, #16) and §5
- `docs/Hyades_netcode.md` §2.1 — the presentation seam is read-only
- `docs/Hyades_galaxy_and_autopilot.md` §2 — the command view's hexes
- `docs/Hyades_vehicle_roles.md` §7.1 — the Design class names
- Björn Ottosson, *A perceptual color space for image processing* (2020) — OKLab
- W3C, *Web Content Accessibility Guidelines 2.1*, §1.4.3 and §1.4.11 — the 4.5:1 and 3:1 contrast thresholds
- Xiaolin Wu, *An efficient antialiasing technique*, Computer Graphics 25(4) (SIGGRAPH 1991), 143–152 — the antialiased line (§6.7)
- You-Dong Liang and Brian A. Barsky, *A new concept and method for line clipping*, ACM Transactions on Graphics 3(1) (1984), 1–22 — clipping a line to the screen (§6.7, §7.1)
