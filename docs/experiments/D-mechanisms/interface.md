# §D. Mechanisms — the replay viewer

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

## D.58 The replay viewer: frames, sizes, the palette's ink, and the GPU against the CPU

*Supports `docs/Hyades_interface.md` and T-149. Bed: the three replays
`examples/record_replay` writes — `expansion` (3 seats, seed 7, 300 planets
requested, 400 yr), `beams` (2 seats, seed 32, a 20-hull Tor
stack closing on a 20-hull Cairn stack), `sentries` (3 seats, seed 42, missile
sentries against a Tor raider). Timings are single-container readings, not
benchmarks: each is a median of 5 frames per run, on a shared container, and
carries no error bar.*

### D.58.1 Frames fell on event times, and repeated once a run's events stopped

The first recorder stepped the run until its clock reached a frame's year and
snapshotted at the clock — the time of the **first event at or after** that
year. Two defects followed. Frame times drifted onto event times (the `beams`
replay's frames 1–5 read 0.023, 0.082, 0.082, 0.082, 0.1 yr against 0.02 …
0.1), and once the run's queue held no event before the horizon, every
remaining frame repeated the last event's time: `beams` ended at **1.644 yr**
and `sentries` at **0.257 yr**, against a 3-year horizon, with the last
frames identical. Replaced by `Simulation::snapshot_at(t)` after applying every
event at or before `t`; both replays now end at 3.000 yr and every frame falls
on `k · 0.02`. `a_replay_carries_every_hull_and_world_of_every_frame` asserts
the frame time exactly.

### D.58.2 Replay sizes and load times

| replay | frames | size | load in the wasm module |
|---|---|---|---|
| `expansion`, 400 yr, a frame every 5 yr | 81 | 8,863 kB (2.1 MB gzip) | 237–276 ms |
| `beams`, 3 yr, every 0.02 yr | 151 | 674 kB | 14–17 ms |
| `sentries`, 3 yr, every 0.02 yr | 151 | 561 kB | 11–15 ms |

The first `expansion` replay, on a larger and longer scenario, was 37.6 MB;
writing each hull's static fields once in a table after the frames, leaving out
hulls recycled for their minerals, and shortening the scenario brought it to
the row above. The count of worlds is what galaxy generation produces, which
exceeds `planet_count` (150 requested gives 159 on the contract test's bed).

### D.58.3 The tone map's ink against the seat colors

Seat 1's color (Red archetype, `hy_red` through the tone map) on the ground,
WCAG 2 contrast, by the `ink` parameter: **2.91 at 0.07, 2.98 at 0.04**, and at
or above 3.0 at 0.02 and at 0.0. The other seventeen tested seats pass at all
four. So 0.02 is the highest of the four values tried at which every seat
reaches 3:1; values between 0.02 and 0.04 were not tried.

### D.58.4 What a frame costs, and the GPU against the CPU

Rendered by the wasm module under Node at 960 × 600 screen pixels
(`web/test/smoke.mjs`):

| | tactical | juicy, first CPU version (one run) | juicy, after the two optimizations below (range over five runs) |
|---|---|---|---|
| `expansion` at 400 yr | 2.4–6.0 ms | 85.2 ms | 27.1–38.9 ms |
| `beams` | 0.1–0.4 ms | 66.2 ms | 15.7–19.6 ms |
| `sentries` | 0.1–0.3 ms | 61.3 ms | 16.1–17.4 ms |

The CPU juicy renderer's passes, native release, 960 × 600, 50 repetitions each
(`viewer/examples/juicy_phases`): before — upsample 8.61 ms, resolve 5.95 ms,
blur 1.50, downsample 1.03; after precomputing the bilinear weights per row and
column and tabulating the tone curve indexed by the light's float bits —
upsample 1.91–2.53 ms, resolve 2.03 ms. These are one run each on a shared
container; the blur's two readings (1.47 and 1.89 ms) with no change to it
bound the run-to-run spread at about ±0.4 ms.

**GPU against CPU** (`web/test/browser.mjs`, headless Chromium with SwiftShader,
the `sentries` replay at 30% of its span, 690 lights, the canvas's size in a
1280 × 800 window): per-pixel maximum channel difference **mean 0.17 codes, 99th
percentile 2, maximum 6**, over one frame, two runs with identical results.
SwiftShader is a software rasterizer; the difference on a hardware GPU has not
been measured. The test's tolerance (mean < 1, 99th percentile ≤ 4, maximum
≤ 24) leaves room for a hardware GPU's half-float rounding.

The GPU's frame time has not been measured: SwiftShader's would describe the
CPU it runs on, and no hardware GPU is available to the test.

### D.58.5 The first deployment on a phone, and the overlay that took every tap (T-151)

**The theater's size.** The deployed site under Playwright's Pixel 7 emulation
(412 × 839 CSS pixels): the canvas was **623 × 148 CSS pixels**, wider than the
screen, with sideways scroll. The header, the side panel and the log were laid
out below and beside it as on a wide screen. After the client restructure
(§8 of `Hyades_interface.md`), the same emulation gives **412 × 700** (83% of the
height) with no sideways scroll, and opening the panel or the log does not
change it (`web/test/browser.mjs` asserts ≥ 60% and no scroll).

**Selection.** Picking through the module (`hv_pick` at a drawn hull's position)
selected the hull; a mouse click at the same point selected nothing, and a
`pointerup` listener on the canvas never fired. `#message`, the full-stage
overlay that carries loading and error text, had its own `display: grid`, which
overrides the `hidden` attribute's `display: none`; so a hidden message still
covered the canvas and received every pointer event. The deployed `style.css`
carried the same rule. A `[hidden] { display: none !important; }` rule fixed
it; the browser test now clicks and taps a hull the module reports drawn
(`hv_text(10)`). Inference: this overlay is why the author could not select
anything on a phone. Confidence: high for the overlay, since the same rule and
the same page structure were deployed; it would be lower if a tap on the
deployed site had ever selected anything.

**Juicy legibility.** The test `a_hull_outshines_any_world_but_a_homeworld`
renders the viewer's three-seat test replay at 320 × 200 and compares the
dimmest lone hull against the brightest lone non-home world, as the sum of the
8-bit channels at their pixels. Under the first lights (world core 1.5, halo
0.35, bloom weight 0.6): **hull 341, world 562** — a world outshone every hull.
Under the dimmed lights (world core 0.3, an unowned world at half that, halo
0.03, territory 0.12 at a homeworld and 0.025 at a colony, bloom weight 0.35):
**hull 341, world 278**. One frame of one replay; a bound on nothing beyond it.

**The 503.** Not reproduced. The session's container cannot reach the deployed
site (its proxy refuses the connection), and the local server the browser test
runs never answers 503. The page builds no URL that differs between modes, so a
503 in juicy mode came from the server. Inference: it was GitHub Pages
answering while a deployment replaced the site; confidence low, and a 503
reproduced outside a deployment window would refute it. The client now fetches
the module, the index and a replay up to four times on a 5xx or a dropped
connection, 0.5, 1 and 2 s apart, and shows the status with a "Try again"
button if all fail; the browser test serves one 503 and checks the second
request loads.

### D.58.6 The fan, retired; quiet traffic and the armed exception (T-152)

**Superseded design — the fan** (T-151, `Hyades_interface.md` §6.4 before
T-152). Unalike stacks in one place were drawn side by side to the right of the
place, 22 tactical pixels apart, each joined back to the place by a one-pixel
leader line in the grid color, at most four, the rest merged into a marker.
What it was wrong about: a glyph was drawn where its hull was not, and the
leader lines — one per offset glyph, redrawn every frame as stacks formed and
split — read to the author as "weird distracting glitches with the little
lines". Replaced by drawing every glyph at its own position and ordering the
overlap (`tactical::display_order`).

**The literal quiet rule hid a fight.** Taken as written, "unladen Systems and
Contact vehicles are unobtrusive" made every unladen Contact hull one dim
pixel. The `beams` replay's two fleets are LCV pickets with one beam mount each
(20 Cairn, 20 Tor), never laden. At the replay's end, tactical mode drew 16
stacks and **one** pixel that was not the ground, out of 144,000; the module
smoke test failed on it ("1 colors"). Exempting armed hulls draws the surviving
fleet as one glyph with a count of 18 at the replay's midpoint, and the smoke
test passes on all three replays. The author then ruled that armed hulls stay
prominent (`Hyades_interface.md` ruling 18).

**Holdings in the recorded replays.** In `expansion` (60 yr, three seats, 150
worlds requested) the only holdings are the three homeworlds' banks: Band 2.00
of each basic at year 0, read on the cost ladder, and Band 0.00 of each from
year 30, because a homeworld spends its bank as fast as it fills. No outpost
holding appears inside 60 years. The engine–viewer contract test's bed (three
seats, 150 worlds, 80 yr, seed 7) does record holdings at worlds their holder
does not own, and asserts at least one.

**What the new fields cost in size.** The three replays `web/build.sh --quick`
records, before and after cargo by material and holdings were written: `beams`
274,915 → 322,279 bytes (+17%), `expansion` 234,967 → 282,410 (+20%),
`sentries` 221,107 → 261,268 (+18%). One build each; the recording is
deterministic, so these are exact for this commit.

### D.58.7 Steps in a fight, the `beams` setup, and the seat hues (T-154)

**The steps were the replay's precision.** Positions were written to two
decimals of a light year. The `beams` fight spans 0.03 ly — the Tor stack
starts 0.03 ly from the Cairns and fire opens at about 0.009 ly — so every hull
moved in 0.01-ly jumps whatever the sim did, and the viewer's straight-line
interpolation between equal positions read as stops. Velocities, written to
four decimals, were smooth all along: a withdrawing Cairn's recorded `vx`
falls −0.018, −0.036, −0.055 … ly/yr per 0.02-yr frame, a constant 0.94
ly/yr². Positions are now written to four decimals, and the viewer
interpolates on the cubic Hermite curve through both frames' positions and
velocities. Sampled every 0.005 yr through the viewer on the re-recorded
replay, that Cairn's finite-difference speed tracks its interpolated velocity
within 0.003 ly/yr from 0.12 to 0.48 yr (−0.016 against −0.018 at the start,
−0.335 against −0.335 at the end). One hull, one replay.

**What each stack in `beams` is doing.** The replay recorder seeds both fleets
in open space in the picket role (`examples/record_replay.rs`): 20 Cairns
parked 0.5 ly from seat 0's homeworld, and 20 Tors 0.03 ly beyond them moving
toward them at 0.3 ly/yr. A seeded station-role fleet takes its position and
velocity — "parked at rest, or shedding the velocity from there" — so the Tors
brake at 0.94 ly/yr² with no destination; their stopping distance, 0.048 ly,
carries them through the Cairns. Neither stack has a post (`dest` −1 in every
frame), a target or an objective. With no diplomacy (T-11) each seat regards
the other as neutral, and the default Doctrine fires on neutrals only in the
picket role, so both open fire when in reach. A hull returning fire on a
neutral breaks off only if it believes it can outrun the shooter, and a hull
past its structure heads home (`Standing::under_fire`, warfare §8.19): all 20
Tors turned for home past their structure at t ≈ 0.094–0.108 and were wrecked;
two Cairns, at 1.21 and 1.16 of structure, withdrew; the other 18 (damage 0.95,
0.21, 0.03 and fifteen untouched) received no further event and stayed parked
for the rest of the run. *Superseded for the replay by §D.58.8: the Tors'
starting speed is now one they can shed in the gap.*

**Seat hues.** On the proposed palette the materials sit at OKLCH hues magenta
8°, red 30°, rounds 39°, yellow 83°, green 118°, cyan 200°, blue 240°, and the
old seat 0–2 colors were the blue, red and green materials themselves. A fifth
seat hue at 145° came within 0.063 of green at L 0.6; a fourth at 270° within
0.072 of blue at L 0.65. Swept over 265°–290°, the worst seat-to-material
distance rose 0.065 → 0.090 and the worst seat-to-seat distance fell 0.040 →
0.023; 280° keeps both bounds (0.086, 0.040). Lightness stays at or below 0.78,
because a pale violet at 0.84 came within 0.069 of platinum.

### D.58.8 The `beams` Tors close at a speed they can shed (T-156)

At the author's report that the Tors' starting speed was too large for them to
match velocity with the Cairns: at 0.3 ly/yr and a proper braking acceleration
of 0.94 ly/yr², a Tor needs 0.048 ly to stop and starts 0.03 ly out, so no
course it could fly ends at rest beside the Cairns. The recorder now reads the
Tors' braking acceleration off the hull at `t = 0` (0.9399 ly/yr²) and sets
their speed to the one that braking sheds in exactly the gap,
`v = √(γ² − 1)/γ` with `γ = 1 + a·L`: **0.2326 ly/yr**. The determinism
suite's arm keeps 0.3 ly/yr, because it pins the braking-through mechanism.

Re-recorded, seed 32, 3 yr, frames every 0.02 yr (one run):

| | before (0.3 ly/yr) | after (0.2326 ly/yr) |
|---|---|---|
| Tors at rest beside the Cairns possible | no — 0.018 ly overshoot | yes — rest at the Cairns' place |
| fire opens | the stacks overlapping | t ≈ 0.13 yr, the Tors 0.006 ly out, closing at 0.11 ly/yr |
| Tors wrecked | 20, past structure at t ≈ 0.094–0.108 | 20, all by t 0.18 |
| Cairns past structure, withdrawn home | 2 | 9 (worst 1.48 of structure) |
| Cairns standing at the site at the end | 18 | 11 |

The Tors still brake on the course they started on and close into the Cairns'
reach while still moving, because a picket with no post has no decision that
holds it off at range; under fire it holds that course (warfare §8.19).

---

## D.59 Hexes, the clipped galaxy, a pan that moved glyphs, and counts over counts (T-159 – T-163, T-171)

**The works trend, superseded (T-159 → T-162).** The first juicy hex line
showed a hex's works against four frames earlier, as a share of the earlier
sum over a doubling: whiter and up to 3× brighter rising, 0.3× falling. In
the 400-yr `expansion` replay works were flat to ~120 yr (9.12 kt in every
frame, 1,045.73 kt at 400 yr), so the quick 60-yr replay drew every hex flat,
and at 380 yr two hexes read bright. The author ruled that the brightness
shows each seat's work-years, not deltas (interface ruling 25).

**The clipped galaxy (T-161).** Worlds kept inside the prescribed hexes, of
those the unclipped field generates, at seeds 1 and 7:

| seats | hexes | kept |
|---|---|---|
| 2 | 10 | 6,236 and 6,242 of 6,729 (92.7%, 92.8%) |
| 3 | 12 | 6,396 and 6,398 of 6,731 (95.0%, 95.1%) |
| 6 | 19 | 9,626 and 9,620 of 10,059 (95.7%, 95.6%) |
| 12 | 37 | 13,765 and 13,773 of 14,056 (97.9%, 98.0%) |
| 18 | 61 | 18,502 and 18,507 of 18,718 (98.8%, 98.9%) |

The first rule kept only the homeworld hexes and their neighbors, which at 12
and 18 seats left the hexes inside the homeworld ring empty: 72.4% and 33.7%
kept on seed 1. The author confirmed no empty center, and the enclosed hexes
were added.

Clip on against off, the standard bed (3 seats, 800 yr, seeds 1, 7, 42,
31337, paired; a scratch harness sampling every 5 yr):

| | per seed | mean ± SE |
|---|---|---|
| colonies at 800 yr | +0.10, −0.23, +0.06, +0.06% | −0.00% ± 0.08 |
| colony-years | +0.09, −0.05, +0.07, −0.05% | +0.02% ± 0.04 |
| work-years | +1.29, +1.33, +0.24, +0.28% | +0.78% ± 0.30 |

Colonies and colony-years are flat. Work-years reads 2.6 SE on four seeds,
4/4 positive; no replication set was run, so it is not established as an
effect. Run cost: 71–95 s per run either way.

Test targets, the old binary (`main` at 7fadb4a) beside the new on one
container, two interleaved rounds: determinism 37.7 and 38.4 s against 37.4
and 36.5 s; smoke 10.4 and 10.8 s against 10.3 and 10.3 s. A reading of 28.4 s
for determinism earlier in the session was on the previous container.

**A pan that moved glyphs (T-160).** Two lattices fixed to the screen. Stacks
were screen cells, so a pan moved hulls between stacks; the regression test
(hulls a third of a cell apart, the camera moved by tenths of a cell) failed
at one tenth. Glyph pixels were each glyph's screen position rounded, so two
stacks less than a pixel apart shared a pixel or not as the view moved — the
author's "the pointy part of the glyph gets separated or smushed together" in
`beams`; with the stack lattice fixed and per-glyph rounding restored, the
same test failed at four tenths of a cell.

**Juicy `beams` (T-163).** Twenty hulls at one point were twenty lights summed
at one spot, which the tone curve saturated, so a loss changed little, and a
hit was a flash of intensity 6 over three hull radii in the hit color, which
covered the seat color. The first cluster pass separated seats only within one
stack square; the two stacks fell in neighboring squares and the Tor dots
drew inside the Cairn cluster. Grouping clusters by screen overlap separated
them: at t 0.152 yr the 20 Cairns and the remaining Tors draw apart, and at
0.180 yr the Tor cluster is gone.

**Counts over counts, and places that merged as the view zoomed in (T-171).**
A scratch probe (not landed) loaded each `--quick` replay, centered the view
on the mean of its hulls, and swept 41 instants × 31 scales from 200 px/ly up
by ×1.25 to 1.6e5 px/ly (and from 0.3 px/ly on `expansion`, to cover the
Galaxy and Sector levels). At each it counted pairs where a drawn count's box
overlapped another place's glyph or count, and the zoom-in steps where the
number of drawn places fell. Before the numbers below, two errors in the
probe were corrected: it gave a count to places that draw none (all wrecks,
or a quiet hull on top), and it took a digit as four pixels wide, not three.

| replay | square lattice: overlapping pairs / zoom-in drops | overlapping footprints: overlapping pairs / zoom-in drops |
|---|---|---|
| `beams` | 6,050 / 77 | 0 / 1 |
| `sentries` | 714 / 60 | 0 / 2 |
| `expansion`, from 200 px/ly | 0 / 5 | 0 / 1 |
| `expansion`, from 0.3 px/ly | 0 / 86 | 0 / 2 |

A first footprint gave a quiet hull its one pixel and left 21 drops on
`beams`: quiet traffic one pixel apart at a homeworld, whose pixels coincide
or not as the galaxy rounds at each scale. A footprint of the pixel and the
ring about it took that to 1. Inference: the drops that remain are the same
rounding at a footprint's edge, one pixel either way; confidence moderate,
and printing the places at those steps would test it. At 8,882 px/ly,
t 0.200 yr, the lattice drew the `beams` stacks' glyphs and counts over one
another; the footprints draw one place, 7 hulls on their role and 13 heading
home. The hit ring, radius half body + 4, covered a count's first column
there before and after the change; the counts now start two pixels later.

## D.62 Frame pacing, paths in place of routes, the `Band IV` jitter, and the palette in play (T-173 – T-178)

*Supports `docs/Hyades_interface.md` rulings 31–36, §4, §6.2, §6.5, §6.7 and
§7.1. Bed: the replays `examples/record_replay` writes at this commit. Every
timing is a single-container reading on a shared 4-core container with no
error bar; the A/B rows are two interleaved rounds of the same probe, and the
spread between rounds is the only variance estimate.*

### D.62.1 What a frame costs, and the pacing

**Per-frame module cost**, wasm in Node (a scratch probe: four seek points
in each replay, 60 ticks of 1/60 s each, the module's work for one frame —
`hv_render` in tactical, `hv_prepare_lights` for the GPU juicy path, the CPU
renderer for the fallback), `main` against this branch, round 1 / round 2,
median ms:

| replay | tactical, `main` | tactical, branch | juicy prep, `main` | juicy prep, branch | juicy CPU, `main` | juicy CPU, branch |
|---|---|---|---|---|---|---|
| `beams` | 0.40 / 0.43 | 0.37 / 0.43 | 0.17 / 0.24 | 0.18 / 0.19 | 30.5 / 30.2 | 29.1 / 30.1 |
| `sentries` | 0.37 / 0.39 | 0.47 / 0.32 | 0.13 / 0.13 | 0.11 / 0.11 | 29.3 / 30.3 | 29.7 / 29.8 |
| `expansion` | 6.29 / 5.35 | 3.70 / 4.15 | 6.65 / 6.67 | 3.51 / 4.45 | 44.2 / 43.5 | 41.7 / 46.2 |
| `palette` | 8.09 / 8.06 | 4.28 / 4.20 | 7.91 / 7.90 | 4.35 / 4.28 | 36.7 / 38.6 | 36.4 / 35.1 |

p95, `expansion` tactical: 9.53 / 10.18 on `main`, 6.13 / 6.35 on the branch;
`palette` tactical: 24.8 / 26.0 against 13.5 / 16.6. The branch draws routes
no longer and caches a seat's glyph fill per frame instead of mixing it per
hull. Inference: the busy replays' tactical and juicy-prep frames cost
0.5–0.7× what they did, and the CPU juicy fallback is unchanged at 29–46 ms,
which caps it near 22–34 fps on this container whatever the pacing; moderate
confidence, and a profile of the CPU renderer on a phone would change it.

**In the browser** (headless Chromium, SwiftShader software WebGL2, a scratch
probe pressing Space through the shell and reading its status line after
3 s; 60 fps target unless stated):

| replay | tactical fps | juicy fps | yr/s at the rate |
|---|---|---|---|
| `beams` (0.05 yr/s) | 49 | 18 | 0.050 / 0.049 |
| `expansion` (6.67 yr/s) | 45 | 3 | 7.18 / 4.22 |
| `palette` (1.67 yr/s) | 60 | 12 | 1.81 / 1.66 |
| `forge` (1.00 yr/s) | 51 | 6 | 1.000 / 0.950 |
| `expansion`, 30 fps target | 31 | 4 | 7.22 / 5.41 |
| `palette`, 30 fps target | 31 | 11 | 1.80 / 1.62 |

Two readings from it. **The excess yr/s in tactical is a fixed offset**: on
`expansion` it is +1.66 yr at 1.5 s and +1.67 yr at 6 s of play (7.78 and
6.94 yr/s), which is one 0.25 s step at 6.67 yr/s — the first frame after
pressing play counts the wall time a slow frame took. **The juicy shortfall
was the cap**: at 3–6 fps a frame lasts longer than the 0.25 s cap on `dt`,
so the clock lost the rest; the cap is now 1 s. SwiftShader runs the GPU
passes on the CPU, so the juicy fps here bound nothing about a device with a
GPU; the measurement that is missing is the same probe on one.

### D.62.2 Routes, superseded (T-152 → T-174)

Routes drew a line from every hull in flight to its destination world, in the
seat's color 90% of the way to the ground. On `expansion` hundreds of hulls are
in flight at once, and the lines covered the field; the author found the view
illegible (ruling 32). A route also said nothing about the drive: a hull
braking into a world and one coasting past it drew the same line. Paths
(interface §6.7) draw only large stacks, only five years ahead, and color the
drive.

### D.62.3 The `Band IV` jitter

The juicy hex line is a row of dots and, on a `Band IV` hex, a row of dashes.
Both were counted from **where the screen cut the edge**, not from the edge's
corner, so whenever the view moved — a pan, the camera following a selection,
a zoom — the cut point moved and every dot slid along the edge by up to one
spacing. That is the jitter. With the camera still, a scratch probe on
`expansion` at 30% of the replay moved no dot in 200 steps of 0.2 yr, and on
`forge` none in 200; the dots move only with the view.

The fix counts from the edge's corner and lights the dots whose positions fall
on screen. `a_hexs_dots_move_with_the_hex_when_the_view_pans` pans by
(0.37, 0), (11.3, −7.9) and (−23, 15.5) pixels at zoom 1 and asserts every dot
moves with the hex; it fails on the old code and passes on the new.

**No world in the 400-yr `expansion` replay reaches `Band IV` works at this
commit** (the most is a `Band III` reading), so the dashes the author saw came
from an earlier recording; `forge` starts six homeworlds at `Band IV`
population and works to show them.

### D.62.4 The palette in play

Sizes at this commit (`examples/record_replay`): `palette` 12.8 MB, 101
frames, 18 seats, 600 planets requested, 100 yr — 50 frames at 0.02 yr over
the first year, then every 2 yr; `forge` 0.47 MB, 61 frames, 6 seats, 60 yr.
A first `palette` at 40 yr with every frame 0.1 yr apart was 51.7 MB, which is
what the dense opening (interface §3) is for.

What each replay exercises (a scratch census over every row of every frame),
with the year each first appears:

| | `palette` | `forge` | `expansion` |
|---|---|---|---|
| seats with hulls | 18 | 6 | 3 |
| roles | Picket, Sentry, Scout 0; Reserve, Scrapped 0.1; Miner, Freighter 8; Colonizer 72 | Scout | Scout 0; Miner, Freighter 10; Colonizer 75; Reserve 120 |
| carried | Ordnance 0; the three basics 68; settlers 72 | — | the three basics 70; settlers 75 |
| held | the three basics | the basics; Strange Matter 5 | the basics; Red, Green, Strange Matter 400 |
| drive, braking, hot burn, hits, wrecks, heading home | all | drive, braking, hot | — |
| hex lines, established | yes | yes | yes |
| `Band IV` works | no | yes | no |

Nothing shipped shows the Blue super, or any super or the apex as cargo.

**Why `forge` is its own replay.** Starting `palette`'s homeworlds developed
was tried first, at 100 yr: works `Band IV` with population `Band IV`, `II .785`,
`III .5` and `III .9`. Every arm reached `Band IV` works, and none flew a
colonizer or carried a basic in 100 yr: the `II .785` arm flew miners and
freighters, and the other three no miner, freighter or colonizer at all. Inference: a
developed homeworld spends its first century deepening and forging before
expanding, so a replay cannot open with `Band IV` and show the expansion
traffic within a shippable horizon; low confidence in the mechanism, which a
census of the production decisions on that bed would settle.

## D.63 The demo games as recipes, their coverage, and the viewer's memory (T-180 – T-182)

*Supports `docs/Hyades_interface.md` rulings 38–39, §2, §3 and §6.2. Bed: the
five demos in `hyades_engine::demos` at this commit. Timings are single runs
on a shared 4-core container unless a range is given, and a range is two
runs; no error bar is available for them.*

### D.63.1 One recipe, one replay, on both targets

Every full demo recorded through the wasm32 engine module (`hyades_sim.wasm`
in Node, a frame at a time in steps of 4) and through the native engine
(`examples/record_replay`) gives **byte-identical replays**, five of five:

| demo | frames | replay | native record | wasm32 record (Node) | open in headless Chromium |
|---|---|---|---|---|---|
| `expansion` | 81 | 11.6 MB | 0.5 s | 0.85–0.98 s | 1.5–2.6 s |
| `beams` | 151 | 1.0 MB | 0.0 s | 0.07 s | 0.3–0.4 s |
| `palette` | 111 | 42.8 MB | 3.3 s | 4.75–4.94 s | 5.9–6.9 s |
| `forge` | 81 | 28.7 MB | 3.2 s | 5.67 s | 6.0–6.7 s |
| `sentries` | 151 | 0.8 MB | 0.1 s | 0.08 s | 0.3 s |

"Open" is from loading the page at `?replay=<name>` to the first drawn frame:
fetching and compiling the module, recording in a worker, handing the replay
over and parsing it. The module is 1.0 MB; its memory reaches 148 MB while
recording `palette` or `forge` and is released when the shell ends the
worker. The three demos #29 shipped unchanged record byte for byte what the
old example wrote.

### D.63.2 What each demo shows

A scratch census over every hull row of every frame. Hull and Design pairs
seen:

| demo | pairs |
|---|---|
| `palette` (600 yr, 18 seats, 300 planets requested) | LSV Spur, LSV Meadow, LSV Ford, LSV Unnamed, MSV Delta, MSV Ford, GSV Strait, LCV Tor, LCV Cairn, LOU Butte — 10 |
| `forge` (800 yr, 6 seats, 300 planets requested) | the ten above, LCU, GCU, ROU and GOU Unnamed, GSV Range, GCV Scarp, LOU Mesa — 17 |

Materials carried: all eight in both (`palette`: Blue 51 rows, Green 51, Red
34, Strange Matter 37; `forge`: Blue 909, Green 1,273, Red 712, Strange Matter
778). Held from: the supers and the apex at 330–350 yr in `palette`, 330–400 yr
in `forge`. `Band IV` works: `forge` from 740 yr, `palette` none. Roles: all
eight in `forge`. Cards landed: 18 of 18 and 6 of 6 at 200 yr.

How the demos got there, arm by arm:

| arm | size | what it showed, or why it failed |
|---|---|---|
| `palette` 400 yr, 600 planets, 5-yr frames, survey cards among the six | 74.6 MB | 669,285 of ~800,000 hull rows are Spur scouts; the survey cards add scouts |
| `palette` 400 yr, 300 planets, 10-yr frames, no survey cards | 24.2 MB | 18 cards land; supers held from 330 yr; **no scout is built after 200 yr** (the field is surveyed), so the Warfare card has no scouts to move onto Tor, and colonizers stay on Delta |
| `palette` 600 yr, the same plus super-billed twins (shipped) | 42.8 MB | supers and the apex carried |
| `forge` works and population at `Band IV`, 60 yr (#29) | 0.47 MB | forging takes priority over building; only the starting scouts move |
| `forge` works at `Band IV`, population default, 600 yr | 2.8 MB | **0 cards land**: each seat holds 0.008–0.009 kt of basics from 50 to 250 yr against a 0.5 kt card. Inference: works four Bands above the people are mostly unstaffed, so the homeworld mines almost nothing; not measured directly |
| `forge` default start, 800 yr, cards and twins | 32.1 MB | 6 cards land, all eight materials carried, `Band IV` from 420 yr, Butte sentries and Cairn pickets built; no Range, Scarp, Tor, Mesa, LCU, GCU, ROU or GOU |
| the same plus the parade and a 10-ly survey (shipped) | 28.7 MB | all 17 pairs; `Band IV` from 740 yr |

No yard built a Range or Scarp in any arm. The General colonizer rung is taken
only when a center can pay for it and it delivers more settlers per mineral
than the Medium one (`BaselineAutopilot::production_choice`); the census
counts builds, and the comparison itself was not instrumented.

### D.63.3 The viewer's memory

Wasm memory of the viewer module after `hv_load` (a scratch Node probe; it
does not move in the first render of either mode):

| demo | before | after |
|---|---|---|
| `palette` | 454 MB | 262 MB |
| `forge` | 305 MB | 178 MB |
| `expansion` | 129 MB | 74 MB |

Natively, with a counting allocator on `palette`: the parse tree was 492 MB
and the peak 657 MB; after, 110 MB and 275 MB, with the built replay itself
123 MB in both. Two causes. **A `Value` per number**: storing a numbers-only
array as `Vec<f64>` at first changed nothing measurable, because the
`Vec<Value>` → `Vec<f64>` collect reused the original allocation in place and
kept 32 bytes of capacity a number; parsing straight into a `Vec<f64>` and
shrinking it to size is what took the tree from 492 to 110 MB. **A copy of the
input**: `hv_load` copied the input buffer into a `String` before parsing,
another 43 MB for `palette`; it now parses in place and releases the buffer.
The replay holds 453,905 hull rows for `palette`, 286,195 for `forge` and
121,794 for `expansion`.
