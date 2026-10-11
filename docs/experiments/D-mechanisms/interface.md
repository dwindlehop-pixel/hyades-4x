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

## D.59 Hexes, the clipped galaxy, and a pan that moved glyphs (T-159 – T-163)

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
