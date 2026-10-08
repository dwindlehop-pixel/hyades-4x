# Hyades — the game interface (Rev 1)

The presentation of a Hyades game: how a run leaves the engine, how it is
played back, and how it is drawn. **This spec carries ratified and open
decisions only** (`AGENTS.md` §6). The measurements behind it are in
`docs/Hyades_experiments_appendix.md` §D.58.

Status key: **RATIFIED** — the author's ruling; **BUILT** — implemented to a
ratified decision; **OPEN** — a decision not yet made, with a recommendation
where one is attached. Every magnitude marked *placeholder* is unratified.

Code: the engine's recorder is `src/replay.rs` and `Simulation::snapshot_at`;
the viewer is the `viewer/` crate (`hyades-viewer`); the web shell is `web/`;
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

---

## 2. The seam

BUILT. The engine records a run as a **replay** — a self-describing JSON
document (§3) — and the viewer reads replays and nothing else:

```
Simulation ──snapshot_at(t)──▶ record_run ──JSON──▶ Replay ──▶ View ──▶ tactical / juicy
 (engine)                       (engine)                       (viewer crate, wasm32)
```

- The viewer crate **does not link the engine**. Its only dependency on it is a
  dev-dependency for the contract test (`viewer/tests/contract.rs`), which
  records a run and checks that the viewer reads every world, every hull of
  every frame — owner, hull, Design, role, position — and every event.
- The engine gains no presentation code: `snapshot_at` and `next_event_time`
  are reads of state the engine already holds, and the recorder is pure data
  (no filesystem, no clock).
- Watching a game **as it runs** — the engine in the browser beside the viewer,
  streaming frames — is OPEN (**R-UI5**). Recommendation: the same replay
  format, appended to frame by frame, so the viewer has one input.

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
| `enums` | `kind` (role), `hull` (the docs' codes, LSV … GOU), `design` (the class names, Meadow … Unnamed), `category` (log categories) |
| `planet_fields`, `planets` | static per world: `id, x, y, z, hab, bio_max, cyan, magenta, yellow, home` — Bands |
| `frame_planet_fields` | per world per frame: `owner` (−1 none), `pop`, `works` — Bands |
| `vehicle_fields` | per hull per frame: `id, kind, x, y, z, vx, vy, vz, accel, burn, damage, flags, dest, cargo, settlers` — ly, ly/yr, ly/yr², burn ∈ {−1, 0, +1}, damage as a share of structure, flags bit 0 in flight and bit 1 wrecked, dest a world id or −1, cargo and settlers kt |
| `frames` | `{t, owner[], pop[], works[], vehicles[[…]]}` |
| `hull_fields`, `hulls` | static per hull: `id, owner, hull, design, beams, tubes` — written once, after the frames |
| `event_fields`, `events`, `events_truncated` | `[t, category, kind, seat, text]` for every event the run's log filter collected, in time order; past the recorder's cap the list stops and the flag says so |

Decisions in the format:

- **Frame `k` is the theater at exactly `k · frame_years`**: every event at or
  before that year applied, every hull where its motion has it then. A frame is
  never taken at "the first event after" its year (§D.58.1 is the defect this
  replaced).
- **Every hull in the theater is in every frame**, a wreck included (it coasts,
  `damage = 1`). A hull recycled for its minerals is not: its mass is in a bank.
- **A number that is not finite is a fatal error** in the recorder (design law
  #16), not a value in the file.
- **One seed records one replay, byte for byte** — a replay is a pure function
  of the run.
- Size: OPEN (**R-UI6**). A 400-year, 3-seat, 159-world replay at a 5-year
  frame is 8.9 MB (2.1 MB compressed, §D.58.2). Recommendation: a version 2
  that writes only the hulls whose state changed since the previous frame,
  when a replay the author wants to keep passes ~50 MB.

---

## 4. Playback

BUILT (`viewer/src/timeline.rs`).

- The clock plays at a **signed rate** in game years per wall-clock second; a
  negative rate rewinds. Reaching either end stops and pauses there.
- **Play at the end it is heading for starts over** from the other end.
- **Seek** holds the clock inside the replay's span; **step** moves to the next
  or previous frame time and pauses.
- Between frames, a hull is drawn on the straight line between its two frame
  positions. Every other quantity is the earlier frame's.
- A replay opens paused at its start, at a rate that plays it through in **60
  seconds** (*placeholder*), framed on its `focus` when it has one.

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
page `palette.html` is a reference sheet of the same values — each source color
beside its mapped color, every role, seat and status color — and opens a link's
settings too.

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
fill's dimming (`fill`, proposed 0.45, §6.3), and any of the 40 palette colors
or 9 status colors set by hand (a hand-set color replaces the tone map's
result for that name, and every role and seat drawn from it follows). Each
change redraws both modes.

**The settings are one line of text** (`viewer/src/palette.rs::Settings`),
owned by the module — the page sends it and reads back the canonical form:

```
ink=0.02 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228 fill=0.45 hy_red=#c83a2c Hit=#ff2d6f
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
| ground, panel, grid, hex lines | base03, base02, base01, base02 |
| text (dim, normal, bright) | base0, base2, base3 |
| a world (unowned) | base01 |
| the basics cyan, magenta, yellow | cyan, magenta, yellow |
| seat `i` | archetype `i mod 3`'s family, step `⌊i/3⌋`: Blue — blue, violet2, blue3, violet3, cyan2, blue2; Red — red, magenta2, orange3, red3, magenta3, orange; Green — green, yellow3, cyan, green3, yellow, cyan3 |
| Doctrine roles | Scout base3, Colonizer green3, Miner yellow3, Freighter orange3, Picket violet3, Sentry magenta3, Reserve base0, Scrapped base01 |

The first three seats are the three supers' own colors, so on a 3-seat table
an empire wears its archetype.

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
| Laden | `#fff36b` | a pixel at the center of a hull carrying cargo or settlers |
| Combat | `#ff4fe1` | the log's combat rows |
| Selected | `#f2fff6` | the selection's corner brackets |

What is tested and holds whatever is approved: text at 4.5:1 on the ground and
on a panel (WCAG 2 AA), dim text at 3:1, eighteen seats each at 3:1 on the
ground and at least 0.03 apart in OKLab, and every role accent at 3:1.

### 6.3 The glyph grammar — OPEN (R-UI3)

Proposed, and built as proposed (`viewer/src/glyph.rs`):

| what | read from | drawn as |
|---|---|---|
| hull family | the hull code | Systems a square, Contact a diamond, Offensive a triangle |
| hull size | the hull code | Limited 7, Medium 9, General 11 tactical pixels across |
| armament | the hull's beam and tube mounts | a two-pixel spike ahead for beams, a pixel either side for tubes |
| Design class | the class | a 4-bit tag under the shape: the class's index plus one; Unnamed has none |
| seat | the owner | the outline in the seat's color, the fill the same color 45% toward the ground |
| Doctrine role | the role | a plus at the center in the role's accent |

Every Design reads differently on every hull, and the three families differ at
every size (`every_design_reads_differently_on_every_hull`). A tactical pixel
is two screen pixels (*placeholder*), scaled up without smoothing.

### 6.4 Stacks — OPEN (R-UI4)

Proposed and built: hulls whose positions fall in one 3-tactical-pixel square
(*placeholder*) **with one owner, Design, role and wreck state** are one glyph
with a count beside it in a 3×5 pixel font. The glyph carries the worst damage
and any hit, cargo or selection among its members. Unalike stacks in one place
fan out to the right 22 tactical pixels apart (*placeholder*), each joined to
the place by a leader line; the largest stack keeps the place. A pick selects a
stack's lowest-id hull, and the inspector says how many it stands for.

### 6.5 Worlds, hexes and levels of detail

BUILT, magnitudes *placeholders*. A world is a dot in its owner's seat color
(an unowned world dim), a homeworld one pixel larger with a ring. The command
view's hexes (flat-top, `hex_side_ly`, one centered on `hex_origin` — galaxy
§2) are drawn when a hex is at least 6 tactical pixels across. The level of
detail is set by the camera's scale: **Galaxy** below 2 screen pixels per ly,
**Sector** to 40, **System** above; worlds grow a pixel per level.

---

## 7. The juicy mode

### 7.1 The method

BUILT. **Every entity is a light** added into a linear high-dynamic-range
buffer; bright light blooms into its neighbors; one tone curve brings the sum
to display range. The lights come from the tactical plan (§6.1), so the two
modes cannot disagree about where anything is.

| entity | light |
|---|---|
| a world | a star: a white core and a halo |
| an owned world | the star, plus a wide dim glow in its seat's color — at the Galaxy level an empire reads as a colored nebula |
| a hull | a point in its seat's color |
| a burning drive | a plume behind the hull (ahead of it when braking), in the drive's status color |
| a hit | a flash in the hit color |
| a wreck | a dim ember |

A light of intensity `I` and radius `r` adds `I / (1 + d²/r²)²` at distance
`d`, cut at `4r`. The bloom is the buffer averaged in 4×4 blocks, box-blurred
(radius 2, three passes) and added back at weight 0.6. The tone curve is
`1 − e^(−x)` per channel, then sRGB encoding. Every magnitude here is a
*placeholder*; the light sizes grow with the level of detail
(`light_scale`).

### 7.2 Style — OPEN (R-UI2)

RATIFIED that it is to be found rather than specified, with lighting at its
heart. What is built is a first pass to look at, not a proposal for the style.
Recommendation: settle it by looking at the deployed site against recorded
games, and record each decision here as it is made.

### 7.3 On the GPU

BUILT. The module computes the **light list** (`juicy::lights`); the shell
draws it with **WebGL2** (`web/gpu.js`): instanced additive quads into a
half-float target, the block average, the separable blur, the territory glow
into the bloom target, and a composite pass with the tone curve. The constants
come from the module, so the two renderers share them.

**The module's CPU renderer (`juicy::rasterize`) is the reference** and the
fallback where a browser has no WebGL2 or no renderable half-float target. The
browser test renders one frame both ways and fails if they differ by more than
a mean of 1 code, a 99th percentile of 4 or a maximum of 24 (§D.58.4).

Tactical mode is rasterized by the module pixel for pixel; the GPU scales it
up, nearest-neighbor.

Rendering juicy at the device's pixel density rather than at CSS pixels is
OPEN — part of R-UI2.

---

## 8. The viewer and the site

BUILT.

- **`viewer/`** — Rust, no dependencies today (ruling 14 permits them; R-UI8), compiled to `wasm32-unknown-unknown`
  as a module with no imports. Its plain C interface (`viewer/src/ffi.rs`,
  `hv_*`) takes and returns numbers; bytes cross through one input buffer, the
  framebuffer and one text buffer. **Everything the page shows as text is
  formatted by the module.**
- **`web/`** — `index.html`, `style.css`, `shell.js` (input, layout, the log as
  a virtual list), `gpu.js` (§7.3) and `palette.html` (§6.2). The page chrome
  takes its colors from the module's palette, so chrome and canvas cannot
  differ.
- **Controls**: play/pause, rewind, step, a rate multiplier, a scrubber; Space,
  J/K/L, ←/→, T (mode), F (fit everything), G (the replay's focus), C (center
  on the selection); wheel zooms about the pointer, drag pans, click selects; a
  replay file can be opened from disk; the side panel's palette editor tunes the
  palette live (§6.2.1).
- **`web/build.sh <out> [--quick]`** assembles the site: the module, the page,
  and replays recorded by `examples/record_replay` at that commit (expansion,
  a beam fight, a missile-sentry fight).

**Tests**: the viewer crate's unit tests; the contract test (§2);
`web/test/smoke.mjs`, which drives the module headless over every replay in
both modes; and `web/test/browser.mjs`, which opens the site in headless
Chrome, checks it renders on WebGL2 without a console error, compares GPU and
CPU juicy (§7.3), plays, filters the log and seeks from a row, and drives the
palette editor (a link's settings open with the page; a slider redraws the
canvas and rewrites the link; a hand-set color reaches the settings line;
reset returns to the proposal).

**Deployment**: `.github/workflows/pages.yml` builds and tests the site on
every push to `main` and publishes it to GitHub Pages. The repository's Pages
source must be set to **GitHub Actions** once (Settings → Pages). CI's `viewer`
job runs the same build and tests on every pull request.

---

## 9. Register

| code | decision | status | what would settle it |
|---|---|---|---|
| **R-UI1** | the tactical palette: the tone map's parameters, the role and seat assignments, the status colors, the glyph fill (§6.2) | OPEN — proposal built, live editor built, `PALETTE_STATUS = "proposed"` | the author's ratification on the live viewer: a settings line or link (§6.2.1); its values then replace the proposal and `PALETTE_STATUS` becomes `ratified` |
| **R-UI2** | juicy mode's style (§7.2), including its resolution on high-density screens | OPEN — first pass built | the author's judgment against recorded games |
| **R-UI3** | the glyph grammar (§6.3) | OPEN — proposal built | the author's review on the deployed site |
| **R-UI4** | stacking: the stack square, the fan spacing, what a stack carries (§6.4) | OPEN — proposal built | the author's review; a census of stack sizes on a long replay if counts prove unreadable |
| **R-UI5** | watching a game live (§2) | OPEN — recommendation attached | a decision to run the engine in the browser; the recorder's API already takes a running `Simulation` |
| **R-UI6** | replay size (§3) | OPEN — recommendation attached | the first replay the author wants to keep that passes ~50 MB |
| **R-UI7** | showing height `z` in the galaxy view (§6.1) | OPEN | the author's ruling on whether the view stays top-down |
| **R-UI8** | which upstream packages the interface takes up now that it may (ruling 14) | OPEN — recommendation attached | a need the current code cannot meet. Recommendation: none yet; the first candidates are `wasm-bindgen`/`web-sys` (to retire the hand-written C interface and `shell.js`'s byte copying) and `wgpu` (to move `gpu.js` into Rust), each taken when a change would otherwise be built on the hand-rolled path |

## References

- `AGENTS.md` §4 (design law #15, #16) and §5
- `docs/Hyades_netcode.md` §2.1 — the presentation seam is read-only
- `docs/Hyades_galaxy_and_autopilot.md` §2 — the command view's hexes
- `docs/Hyades_vehicle_roles.md` §7.1 — the Design class names
- Björn Ottosson, *A perceptual color space for image processing* (2020) — OKLab
- W3C, *Web Content Accessibility Guidelines 2.1*, §1.4.3 and §1.4.11 — the 4.5:1 and 3:1 contrast thresholds
