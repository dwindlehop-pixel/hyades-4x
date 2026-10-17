//! **The juicy mode** (`docs/Hyades_interface.md` §7): the theater as light.
//! Its style is to be found rather than specified (R-UI2); what is fixed is
//! the method — every entity is a light added into a linear high-dynamic-range
//! buffer, bright light blooms into its neighbors, and one tone curve brings
//! the sum into display range — and that the lights change with the level of
//! detail.
//!
//! | entity | light |
//! |---|---|
//! | a world | a star: a white core, a halo that widens with the level of detail |
//! | an owned world | the star plus a wide, dim glow in its seat's color — at the galaxy level an empire reads as a colored nebula |
//! | a hull | a point in its seat's color |
//! | a hull that moved | a fading trail back to where it was a frame ago (T-184) |
//! | a burning drive | a plume behind the hull, in the drive's status color, brighter with acceleration |
//! | a hit | a flash in the hit color |
//! | a wreck | a dim ember |
//! | a hex | a dim line of lights inset from its edge; inside it a line per seat with works there, brighter with the seat's work-years; dashed bright where a world holds `Band IV` works |

use crate::camera::Lod;
use crate::color::Rgb;
use crate::palette::Status;
use crate::raster::{clip_segment, Raster};
use crate::tactical::{Op, Scene};

/// Screen pixels per juicy pixel.
pub const PIXEL: f64 = 1.0;
/// The bloom works at this fraction of the resolution.
pub const BLOOM_DOWNSAMPLE: usize = 4;
/// How much of the blurred light is added back.
pub const BLOOM_WEIGHT: f32 = 0.35;
/// Light intensities (proposed, R-UI2). Worlds are faint points and an
/// empire's glow is a tint, so the hulls — the things that move and fight —
/// carry the scene; the author found brighter worlds made everything
/// illegible.
pub const WORLD_CORE: f32 = 0.3;
/// An unowned world's core, as a share of an owned one's.
pub const UNOWNED_SHARE: f32 = 0.5;
pub const HOME_CORE: f32 = 0.9;
pub const WORLD_HALO: f32 = 0.03;
pub const TERRITORY_HOME: f32 = 0.12;
pub const TERRITORY_WORLD: f32 = 0.025;
/// A quiet hull's light (a scout, unladen traffic), against 2.0 for the rest.
pub const QUIET_HULL: f32 = 0.5;
/// **A stack below the galaxy level is a cluster** (T-163, proposed, R-UI2):
/// a dot of radius `CLUSTER_DOT_R` hull radii per hull, `CLUSTER_DOT_SPACING`
/// hull radii apart in a sunflower spiral, up to `CLUSTER_MAX_DOTS` (more
/// hulls brighten the dots instead). Seats sharing a place (§6.4) stand
/// `CLUSTER_GAP` hull radii apart at their nearest.
pub const CLUSTER_DOT_R: f64 = 1.0;
pub const CLUSTER_DOT_SPACING: f64 = 2.0;
pub const CLUSTER_MAX_DOTS: usize = 64;
pub const CLUSTER_GAP: f64 = 3.0;
/// The sunflower spiral's turn between dots, radians: 360°/φ².
pub const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
/// A hit is a ring of lights of radius `HIT_RING_R` hull radii at intensity
/// `HIT_RING` each (T-163, proposed): a ring, not a flash, so the hull's own
/// color shows inside it.
pub const HIT_RING_R: f64 = 0.5;
pub const HIT_RING: f32 = 1.2;
/// A wreck's ember, and the radius it narrows to once its glyph has faded
/// (T-158), juicy pixels.
pub const WRECK_EMBER: f32 = 0.6;
pub const WRECK_PINPOINT_R: f64 = 0.5;
/// **A hex's lines** (T-159, T-162; proposed, R-UI2): lights every
/// `HEX_DOT_PX` juicy pixels of radius `HEX_DOT_R`. The grid's line stands
/// `HEX_INSET_PX` inside the edge, so neighbors each keep their own, at
/// `HEX_GLOW` in the hex color. Each seat with works in the hex adds a line
/// [`crate::tactical::HEX_LINE_STEP_PX`] further in, in its color at
/// `HEX_SEAT_GLOW` × its brightness — `HEX_ESTABLISHED` times that once the
/// seat is established. A hex holding `Band IV` works has its grid line
/// dashed in the bright text color at `HEX_BAND_IV_GLOW`.
pub const HEX_DOT_PX: f64 = 1.5;
pub const HEX_DOT_R: f64 = 1.0;
pub const HEX_INSET_PX: f64 = 2.0;
pub const HEX_GLOW: f32 = 0.1;
pub const HEX_SEAT_GLOW: f32 = 0.25;
pub const HEX_ESTABLISHED: f32 = 2.0;
pub const HEX_BAND_IV_GLOW: f32 = 0.6;
/// The bloom's box blur: radius in bloom pixels, and passes (three approach
/// a Gaussian).
pub const BLUR_RADIUS: usize = 2;
pub const BLUR_PASSES: usize = 3;
/// Scene-linear light that maps to about 63% of display white.
pub const EXPOSURE: f32 = 1.0;

/// A linear-light buffer, RGB per pixel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hdr {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[f32; 3]>,
}

/// A color as linear light, the form lights and the background add in.
pub(crate) fn linear(c: Rgb) -> [f32; 3] {
    let f = |v: f64| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    [f(c.r) as f32, f(c.g) as f32, f(c.b) as f32]
}

fn encode(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0) as f64;
    let g = if v <= 0.003_130_8 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
    (g * 255.0).round() as u8
}

/// The tone curve: `1 − e^(−x · EXPOSURE)` per channel. Zero stays black,
/// it rises monotonically, and no light, however bright, passes white.
pub fn tone(x: f32) -> f32 {
    1.0 - (-x.max(0.0) * EXPOSURE).exp()
}

/// **The toned light over the background** (T-179), linear, per channel:
/// `ground + (1 − ground) · tone(x)`. No light is the background — the
/// palette's ink — and no light, however bright, passes white.
pub fn over(ground: f32, x: f32) -> f32 {
    ground + (1.0 - ground) * tone(x)
}

impl Hdr {
    pub fn new(w: usize, h: usize) -> Hdr {
        Hdr { w, h, px: vec![[0.0; 3]; w * h] }
    }

    pub fn energy(&self) -> f32 {
        self.px.iter().map(|p| p[0] + p[1] + p[2]).sum()
    }

    pub fn get(&self, x: i64, y: i64) -> [f32; 3] {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            [0.0; 3]
        } else {
            self.px[y as usize * self.w + x as usize]
        }
    }

    /// Adds a light of `color × intensity` at `(x, y)` (pixel units, may be
    /// fractional) falling off as `1 / (1 + (d/r)²)²`, cut at `4r`.
    pub fn splat(&mut self, x: f64, y: f64, r: f64, color: [f32; 3], intensity: f32) {
        let r = r.max(0.35);
        let reach = (4.0 * r).ceil() as i64;
        let (cx, cy) = (x.floor() as i64, y.floor() as i64);
        for py in (cy - reach).max(0)..=(cy + reach).min(self.h as i64 - 1) {
            for px in (cx - reach).max(0)..=(cx + reach).min(self.w as i64 - 1) {
                let (dx, dy) = (px as f64 + 0.5 - x, py as f64 + 0.5 - y);
                let q = (dx * dx + dy * dy) / (r * r);
                if q > 16.0 {
                    continue;
                }
                let k = intensity / ((1.0 + q) * (1.0 + q)) as f32;
                let p = &mut self.px[py as usize * self.w + px as usize];
                for c in 0..3 {
                    p[c] += color[c] * k;
                }
            }
        }
    }

    /// The mean of each `k × k` block.
    pub fn downsample(&self, k: usize) -> Hdr {
        let (w, h) = (self.w.div_ceil(k), self.h.div_ceil(k));
        let mut out = Hdr::new(w, h);
        for y in 0..self.h {
            for x in 0..self.w {
                let p = self.px[y * self.w + x];
                let o = &mut out.px[(y / k) * w + x / k];
                for c in 0..3 {
                    o[c] += p[c] / (k * k) as f32;
                }
            }
        }
        out
    }

    /// A separable box blur of radius `r`, applied `passes` times (three
    /// passes approach a Gaussian). Light that would leave the buffer is held
    /// at the edge, so the sum is kept.
    pub fn blur(&mut self, r: usize, passes: usize) {
        let (w, h) = (self.w, self.h);
        let mut tmp = vec![[0.0f32; 3]; self.px.len()];
        for _ in 0..passes {
            for horizontal in [true, false] {
                let (src, dst) = if horizontal { (&self.px, &mut tmp) } else { (&tmp, &mut self.px) };
                let (len, lines) = if horizontal { (w, h) } else { (h, w) };
                let idx = |line: usize, i: usize| if horizontal { line * w + i } else { i * w + line };
                for line in 0..lines {
                    for i in 0..len {
                        dst[idx(line, i)] = [0.0; 3];
                    }
                    for i in 0..len {
                        let v = src[idx(line, i)];
                        // Spread v over the window around i, clamped into the line.
                        let share = 1.0 / (2 * r + 1) as f32;
                        for d in 0..=2 * r {
                            let j = (i + d).saturating_sub(r).min(len - 1);
                            let o = &mut dst[idx(line, j)];
                            for c in 0..3 {
                                o[c] += v[c] * share;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Adds `small` (a downsampled buffer `k` times coarser), bilinearly
    /// upsampled, times `weight`. The weights depend only on the column and
    /// the row, so they are worked out once per each.
    pub fn add_upsampled(&mut self, small: &Hdr, k: usize, weight: f32) {
        if small.w == 0 || small.h == 0 {
            return;
        }
        let taps = |n: usize, len: usize| -> Vec<(usize, usize, f32)> {
            (0..n)
                .map(|i| {
                    let f = ((i as f64 + 0.5) / k as f64 - 0.5).max(0.0);
                    let i0 = (f.floor() as usize).min(len - 1);
                    (i0, (i0 + 1).min(len - 1), (f - i0 as f64).min(1.0) as f32)
                })
                .collect()
        };
        let (cols, rows) = (taps(self.w, small.w), taps(self.h, small.h));
        for (y, &(y0, y1, ty)) in rows.iter().enumerate() {
            let (r0, r1) = (&small.px[y0 * small.w..][..small.w], &small.px[y1 * small.w..][..small.w]);
            let out = &mut self.px[y * self.w..][..self.w];
            for (p, &(x0, x1, tx)) in out.iter_mut().zip(&cols) {
                let (a, b, c, d) = (r0[x0], r0[x1], r1[x0], r1[x1]);
                for ch in 0..3 {
                    let top = a[ch] + (b[ch] - a[ch]) * tx;
                    let bot = c[ch] + (d[ch] - c[ch]) * tx;
                    p[ch] += weight * (top + (bot - top) * ty);
                }
            }
        }
    }

    /// Tone-maps into an 8-bit raster of the same size, through [`ToneLut`].
    pub fn resolve(&self, out: &mut Raster, lut: &ToneLut) {
        out.resize(self.w, self.h);
        for (p, o) in self.px.iter().zip(out.px.as_chunks_mut::<4>().0) {
            o.copy_from_slice(&[lut.get(0, p[0]), lut.get(1, p[1]), lut.get(2, p[2]), 255]);
        }
    }
}

/// [`over`] then sRGB encoding, tabulated per channel: two transcendentals
/// per channel per pixel were most of a frame. Indexed by the light's own
/// float bits — exponent and the top eight mantissa bits — so every octave of
/// brightness gets 256 entries and no arithmetic is spent finding one. Built
/// for one background, and rebuilt when the palette's ink changes.
#[derive(Clone, Debug, PartialEq)]
pub struct ToneLut {
    ground: [f32; 3],
    tables: [Vec<u8>; 3],
}

impl ToneLut {
    /// Below this the code is 0; past `HI` the curve is within half a code of
    /// white.
    const LO: f32 = 1.0 / (1 << 20) as f32;
    const HI: f32 = 16.0;
    const SHIFT: u32 = 15;

    /// The table over `ink`, the background color.
    pub fn new(ink: Rgb) -> ToneLut {
        let ground = linear(ink);
        let (lo, hi) = (Self::LO.to_bits(), Self::HI.to_bits());
        let n = ((hi - lo) >> Self::SHIFT) + 1;
        let table = |g: f32| -> Vec<u8> {
            (0..n)
                .map(|i| {
                    let mid = f32::from_bits(lo + (i << Self::SHIFT) + (1 << (Self::SHIFT - 1)));
                    encode(over(g, mid))
                })
                .collect()
        };
        ToneLut { ground, tables: ground.map(table) }
    }

    /// The background this table was built over, linear.
    pub fn ground(&self) -> [f32; 3] {
        self.ground
    }

    /// The display code of light `x` in channel `ch` (0 red, 1 green, 2 blue).
    pub fn get(&self, ch: usize, x: f32) -> u8 {
        let t = &self.tables[ch];
        let i = (x.max(0.0).to_bits().saturating_sub(Self::LO.to_bits()) >> Self::SHIFT) as usize;
        t[i.min(t.len() - 1)]
    }
}

/// Light sizes and strengths at a level of detail: star core radius, star
/// halo radius, territory glow radius (pixels), hull light radius.
pub fn light_scale(lod: Lod) -> (f64, f64, f64, f64) {
    match lod {
        Lod::Galaxy => (0.6, 1.5, 24.0, 0.6),
        Lod::Sector => (0.9, 3.0, 40.0, 1.0),
        Lod::System => (1.5, 6.0, 64.0, 1.6),
    }
}

/// **A light**: a position and a radius in juicy pixels, and a linear color
/// already scaled by the light's intensity. Its falloff is [`Hdr::splat`]'s.
/// `repr(C)`, six `f32`s, so the web shell can hand a slice of them to the GPU
/// as an instance buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub color: [f32; 3],
}

/// Floats per [`Light`].
pub const LIGHT_FLOATS: usize = 6;

/// **What is lit**, the one list both renderers draw: the CPU one here
/// ([`rasterize`]) and the GPU one in the web shell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lights {
    /// Stars, hulls, plumes and hits, at full resolution.
    pub scene: Vec<Light>,
    /// Empires' glow, wide and dim. Positions and radii are at full
    /// resolution; it is drawn [`BLOOM_DOWNSAMPLE`] times coarser, beside the
    /// bloom, below everything else.
    pub territory: Vec<Light>,
}

fn light(x: f64, y: f64, r: f64, color: [f32; 3], intensity: f32) -> Light {
    Light { x: x as f32, y: y as f32, r: r as f32, color: color.map(|c| c * intensity) }
}

/// A streak of lights from `a` to `b`, fading toward `b`, carrying
/// `intensity` in all.
fn streak(out: &mut Vec<Light>, a: [f64; 2], b: [f64; 2], r: f64, color: [f32; 3], intensity: f32) {
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / r.max(0.5)).ceil().max(1.0) as usize;
    for i in 0..=n {
        let f = i as f64 / n as f64;
        let fade = 1.0 - f as f32;
        out.push(light(a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, r, color, intensity * fade / n as f32));
    }
}

/// **How long a hull's light takes to move to a new place in its cluster**
/// (T-184, proposed, R-UI2): wall seconds for the offset from the hull to
/// close by a factor of e. A cluster is laid out afresh every frame, and a
/// hull joining or leaving one, or two clusters meeting, moves every dot in
/// them at once; eased, the dots slide there instead of jumping.
pub const TWEEN_SECONDS: f64 = 0.12;
/// An offset within this many juicy pixels of where it is going is there.
pub const TWEEN_SNAP_PX: f64 = 0.05;
/// **A hull's trail** (T-184, proposed, R-UI2): a light that moved more than
/// `TRAIL_MIN_PX` juicy pixels since the last frame draws a fading line of
/// light back to where it was, carrying `TRAIL_SHARE` of its own light, in
/// at most `TRAIL_MAX_SAMPLES` lights — a frame-long exposure, so a fast
/// hull reads as motion rather than as a dot that jumps. No trail is longer
/// than `TRAIL_MAX_SHARE` of the frame's larger side (a seek is not motion),
/// and a frame draws at most `TRAIL_MAX_LIGHTS` trail lights.
pub const TRAIL_MIN_PX: f64 = 2.0;
pub const TRAIL_SHARE: f32 = 1.0;
pub const TRAIL_MAX_SAMPLES: usize = 24;
pub const TRAIL_MAX_SHARE: f64 = 0.5;
pub const TRAIL_MAX_LIGHTS: usize = 8000;

/// A hull's light as last shown: its offset from the hull (juicy pixels), and
/// where the hull stood then (ly).
type Shown = ([f64; 2], [f64; 3]);

/// **Where each hull's light stands off the hull, as shown** (T-184): one
/// offset per hull, in juicy pixels, eased toward the offset its cluster puts
/// it at. Offsets, not positions, so the camera moving moves every light at
/// once and only a change of layout is eased.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tween {
    /// Per hull: the offset shown, and where the hull stood then (ly).
    offsets: std::collections::BTreeMap<u64, Shown>,
    /// Some offset shown was not yet where it is going.
    moving: bool,
}

impl Tween {
    /// Whether a light is still on its way to its place: another frame would
    /// show it moved with nothing else changed.
    pub fn moving(&self) -> bool {
        self.moving
    }

    /// The offset last shown for hull `id`, juicy pixels.
    pub fn offset(&self, id: u64) -> Option<[f64; 2]> {
        self.offsets.get(&id).map(|e| e.0)
    }

    /// The offsets shown after `dt` wall seconds, each eased from the one
    /// shown before toward its `target`, with where the hull stood when it
    /// was last shown (ly): a hull not shown before starts at its target with
    /// no past, and a hull not in `targets` is forgotten. With an infinite
    /// `dt` every offset is its target and no hull has a past.
    fn ease(&mut self, targets: &[(u64, [f64; 2], [f64; 3])], dt: f64) -> Vec<([f64; 2], Option<Shown>)> {
        let keep = if dt.is_finite() { (-dt.max(0.0) / TWEEN_SECONDS).exp() } else { 0.0 };
        let mut next = std::collections::BTreeMap::new();
        self.moving = false;
        let shown = targets
            .iter()
            .map(|&(id, t, at)| {
                let was = self.offsets.get(&id).copied().filter(|_| dt.is_finite());
                let from = was.map_or(t, |w| w.0);
                let mut o = [t[0] + (from[0] - t[0]) * keep, t[1] + (from[1] - t[1]) * keep];
                if (o[0] - t[0]).hypot(o[1] - t[1]) < TWEEN_SNAP_PX {
                    o = t;
                } else {
                    self.moving = true;
                }
                next.insert(id, (o, at));
                (o, was)
            })
            .collect();
        self.offsets = next;
        shown
    }
}

/// The scene's lights with no easing: every light where its layout puts it.
/// `ops` is the tactical plan: the same entities at the same positions, so
/// the two modes cannot disagree about where anything is.
pub fn lights(s: &Scene, ops: &[Op]) -> Lights {
    lights_eased(s, ops, &mut Tween::default(), f64::INFINITY)
}

/// **The scene's lights, each hull's eased** `dt` wall seconds on from where
/// `tween` last showed it ([`Tween`]).
pub fn lights_eased(s: &Scene, ops: &[Op], tween: &mut Tween, dt: f64) -> Lights {
    let mut out = Lights::default();
    let (core, halo, glow, hull_r) = light_scale(s.camera.lod());
    let star = linear(s.palette.roles.text_bright);
    let pos = |s: [f64; 2]| [s[0] / PIXEL, s[1] / PIXEL];
    for op in ops {
        if let Op::World { s: sp, color, ring, .. } = op {
            let p = pos(*sp);
            let owned = *color != s.palette.roles.world_dim;
            if owned {
                let strength = if ring.is_some() { TERRITORY_HOME } else { TERRITORY_WORLD };
                out.territory.push(light(p[0], p[1], glow, linear(*color), strength));
            }
            let share = if owned { 1.0 } else { UNOWNED_SHARE };
            out.scene.push(light(p[0], p[1], halo, star, WORLD_HALO * share));
            out.scene.push(light(p[0], p[1], core, star, if ring.is_some() { HOME_CORE } else { WORLD_CORE * share }));
        }
    }
    hex_lights(s, &mut out.scene);
    path_lights(s, ops, hull_r, &mut out.scene);
    // Below the galaxy level a stack is a cluster of dots, one per hull, so
    // its count reads and a hit does not hide whose it is (T-163).
    let clustered = s.camera.lod() != Lod::Galaxy;
    // Keyed by the tactical place (§6.4), seat, and whether the hulls are
    // heading home.
    let mut groups: std::collections::BTreeMap<(usize, usize, bool), Vec<usize>> = Default::default();
    let hull_ops: Vec<&Op> = ops.iter().filter(|o| matches!(o, Op::Hull { .. })).collect();
    // Each hull's light: its hull's id and place, where its layout puts it,
    // and its light at that place — eased together once all are laid out.
    let mut hulls: Vec<(u64, [f64; 2], [f64; 3], Light)> = Vec::new();
    for (i, (op, h)) in hull_ops.iter().zip(&s.view.hulls).enumerate() {
        let Op::Hull { s: sp, at, outline, vector, hit, quiet, wreck, site, .. } = op else { continue };
        let p = pos(*sp);
        if clustered && wreck.is_none() && !*quiet {
            groups.entry((*site, h.hull.owner, h.row.withdrawing)).or_default().push(i);
            continue;
        }
        if let Some((end, c)) = vector {
            plume(&mut out.scene, s, p, *at, *end, *c, hull_r);
        }
        // A quiet hull — a scout, unladen traffic — is a faint point, as in
        // tactical mode. A wreck is a dim ember that narrows to a pinpoint as
        // its tactical glyph fades (T-158).
        let (c, k, r) = match wreck {
            Some(share) => (
                linear(s.palette.status(Status::Wreck)),
                WRECK_EMBER,
                WRECK_PINPOINT_R + (hull_r - WRECK_PINPOINT_R).max(0.0) * share,
            ),
            None => (linear(*outline), if *quiet { QUIET_HULL } else { 2.0 }, hull_r),
        };
        hulls.push((h.hull.id, p, h.row.pos, light(p[0], p[1], r, c, k)));
        if *hit {
            ring(&mut out.scene, p, 3.0 * hull_r, hull_r, linear(s.palette.status(Status::Hit)));
        }
    }
    // The clusters: each place's hulls of one seat about their mean
    // position. Clusters that would overlap on screen are one place; in a
    // place each seat's hulls are one cluster, and where seats share it each
    // seat's cluster is set to its own side.
    let spacing = CLUSTER_DOT_SPACING * hull_r;
    let radius = |n: usize| spacing * (n.min(CLUSTER_MAX_DOTS) as f64).sqrt();
    let mean = |members: &[usize]| {
        let mut c = [0.0, 0.0];
        for &i in members {
            if let Op::Hull { s: sp, .. } = hull_ops[i] {
                let p = pos(*sp);
                (c[0], c[1]) = (c[0] + p[0], c[1] + p[1]);
            }
        }
        [c[0] / members.len() as f64, c[1] / members.len() as f64]
    };
    /// A seat, and whether its hulls here are heading home.
    type Side = (usize, bool);
    let found: Vec<(Side, Vec<usize>, [f64; 2])> =
        groups.into_iter().map(|((_, seat, back), m)| ((seat, back), m.clone(), mean(&m))).collect();
    // Places: clusters joined while any two are within reach of each other.
    let mut place: Vec<usize> = (0..found.len()).collect();
    fn root(p: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        p[i] = r;
        r
    }
    for a in 0..found.len() {
        for b in a + 1..found.len() {
            let (pa, pb) = (found[a].2, found[b].2);
            let d = ((pa[0] - pb[0]) * (pa[0] - pb[0]) + (pa[1] - pb[1]) * (pa[1] - pb[1])).sqrt();
            if d < radius(found[a].1.len()) + radius(found[b].1.len()) + CLUSTER_GAP * hull_r {
                let (ra, rb) = (root(&mut place, a), root(&mut place, b));
                place[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    let mut places: std::collections::BTreeMap<usize, std::collections::BTreeMap<Side, Vec<usize>>> =
        Default::default();
    for (i, (seat, members, _)) in found.iter().enumerate() {
        let r = root(&mut place, i);
        places.entry(r).or_default().entry(*seat).or_default().extend(members);
    }
    for seats in places.values() {
        let widest = seats.values().map(|m| radius(m.len())).fold(0.0, f64::max);
        let m = seats.len();
        let apart = if m > 1 { (widest + CLUSTER_GAP * hull_r) / (std::f64::consts::PI / m as f64).sin() } else { 0.0 };
        let all: Vec<usize> = seats.values().flatten().copied().collect();
        let at = |i: usize| match hull_ops[i] {
            Op::Hull { s: sp, .. } => pos(*sp),
            Op::World { .. } => unreachable!("hull_ops holds hulls"),
        };
        let middle = mean(&all);
        for (k, ((seat, back), members)) in seats.iter().enumerate() {
            let (sin, cos) = (std::f64::consts::TAU * k as f64 / m as f64).sin_cos();
            let c = if m > 1 { [middle[0] + apart * cos, middle[1] + apart * sin] } else { mean(members) };
            // Hulls heading home are their own cluster, in the retreat color
            // (T-172): the seat's own cluster counts only hulls on their role.
            let color = linear(if *back { s.palette.status(Status::Retreat) } else { s.palette.seat(*seat) });
            let n = members.len();
            let dots = n.min(CLUSTER_MAX_DOTS);
            let k_dot = 2.0 * n as f32 / dots as f32;
            // A dot per hull, in id order, so a hull keeps its dot while the
            // cluster's membership holds (past the most dots, the rest
            // brighten them).
            let mut by_id: Vec<usize> = members.clone();
            by_id.sort_by_key(|&i| s.view.hulls[i].hull.id);
            for (d, &i) in by_id.iter().take(dots).enumerate() {
                let (sin, cos) = (GOLDEN_ANGLE * d as f64).sin_cos();
                // The first dot at the center: a lone hull is where it stands.
                let r = spacing * (d as f64).sqrt();
                let dot = light(c[0] + r * cos, c[1] + r * sin, CLUSTER_DOT_R * hull_r, color, k_dot);
                hulls.push((s.view.hulls[i].hull.id, at(i), s.view.hulls[i].row.pos, dot));
            }
            if let Some(Op::Hull { at, vector: Some((end, col)), .. }) = members.first().map(|&i| hull_ops[i]) {
                plume(&mut out.scene, s, c, *at, *end, *col, hull_r);
            }
            if members.iter().any(|&i| matches!(hull_ops[i], Op::Hull { hit: true, .. })) {
                ring(&mut out.scene, c, radius(n) + 2.0 * hull_r, hull_r, linear(s.palette.status(Status::Hit)));
            }
        }
    }
    let targets: Vec<(u64, [f64; 2], [f64; 3])> =
        hulls.iter().map(|(id, p, at, l)| (*id, [l.x as f64 - p[0], l.y as f64 - p[1]], *at)).collect();
    let longest = TRAIL_MAX_SHARE * s.camera.width.max(s.camera.height) / PIXEL;
    let mut trail_lights = 0;
    for ((_, p, _, l), (o, was)) in hulls.iter().zip(tween.ease(&targets, dt)) {
        let now = [p[0] + o[0], p[1] + o[1]];
        out.scene.push(Light { x: now[0] as f32, y: now[1] as f32, ..*l });
        // Where this light was drawn last frame, through this frame's camera,
        // so a pan or a zoom draws no trail.
        let Some((o0, at0)) = was else { continue };
        let q = pos(s.camera.project(at0));
        let then = [q[0] + o0[0], q[1] + o0[1]];
        let len = (now[0] - then[0]).hypot(now[1] - then[1]);
        if len > TRAIL_MIN_PX && len <= longest && trail_lights < TRAIL_MAX_LIGHTS {
            let n = ((len / l.r.max(0.5) as f64).ceil() as usize).clamp(1, TRAIL_MAX_SAMPLES);
            for i in 1..=n.min(TRAIL_MAX_LIGHTS - trail_lights) {
                let f = i as f64 / (n + 1) as f64;
                let k = TRAIL_SHARE * 2.0 * (1.0 - f as f32) / (n + 1) as f32;
                let c = l.color.map(|c| c * k);
                out.scene.push(Light {
                    x: (now[0] + (then[0] - now[0]) * f) as f32,
                    y: (now[1] + (then[1] - now[1]) * f) as f32,
                    r: l.r,
                    color: c,
                });
                trail_lights += 1;
            }
        }
    }
    out
}

/// A drive's plume from `p`: behind the vector for a drive, ahead of it when
/// braking. The vector's end is in tactical pixels from the glyph's pixel
/// `at`; its offset is applied at `p`.
fn plume(out: &mut Vec<Light>, s: &Scene, p: [f64; 2], at: [i64; 2], end: [i64; 2], c: Rgb, r: f64) {
    let k = crate::tactical::PIXEL / PIXEL;
    let e = [p[0] + (end[0] - at[0]) as f64 * k, p[1] + (end[1] - at[1]) as f64 * k];
    let tail = if c == s.palette.status(Status::Braking) { e } else { [2.0 * p[0] - e[0], 2.0 * p[1] - e[1]] };
    streak(out, p, tail, r, linear(c), 2.5);
}

/// A hit: a thin ring of lights of radius `radius` about `c`, so what was hit
/// still shows inside it in its own color.
fn ring(out: &mut Vec<Light>, c: [f64; 2], radius: f64, hull_r: f64, color: [f32; 3]) {
    let n = ((std::f64::consts::TAU * radius / (1.5 * hull_r)).ceil() as usize).max(8);
    for i in 0..n {
        let (sin, cos) = (std::f64::consts::TAU * i as f64 / n as f64).sin_cos();
        out.push(light(c[0] + radius * cos, c[1] + radius * sin, HIT_RING_R * hull_r, color, HIT_RING));
    }
}

/// **The active hexes as lines of light** (T-159, T-162): the grid's line,
/// inset so a shared edge reads as two lines, one per hex; inside it a line
/// per seat with works in the hex ([`crate::tactical::hex_lines`]); and the
/// grid's line dashed bright where a world holds `Band IV` works. Only the
/// part of a line on screen is lit, so a hex many screens wide costs what
/// one does.
pub fn hex_lights(s: &Scene, out: &mut Vec<Light>) {
    if !crate::tactical::hexes_shown(s) {
        return;
    }
    let lines = crate::tactical::hex_lines(s);
    let none = crate::tactical::HexLines::default();
    let (w, h) = (s.camera.width / PIXEL, s.camera.height / PIXEL);
    // Dots are counted from each edge's own corner, along the whole edge,
    // and only those on screen are lit: a dot belongs to its place on the
    // hex, so a pan or a follow carries it with the hex, and the dashes of a
    // `Band IV` edge stand still (T-176). Counting from where the screen cut
    // the edge slid them along it whenever the view moved.
    let m = 4.0 * HEX_DOT_R;
    let mut ring = |v: [[f64; 2]; 6], color: [f32; 3], k: f32, dash: Option<(usize, usize)>| {
        for e in 0..6 {
            let (a, b) = (v[e], v[(e + 1) % 6]);
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let Some((p, q)) = clip_segment(a, b, [-m, -m, w + m, h + m]) else { continue };
            let n = (len / HEX_DOT_PX).floor().max(1.0);
            let along = |c: [f64; 2]| (c[0] - a[0]).hypot(c[1] - a[1]) / len;
            let (first, last) = ((along(p) * n).ceil() as usize, (along(q) * n).floor() as usize);
            for i in first..=last {
                if dash.is_some_and(|(on, off)| i % (on + off) >= on) {
                    continue;
                }
                let f = i as f64 / n;
                out.push(light(a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, HEX_DOT_R, color, k));
            }
        }
    };
    for hex in crate::tactical::active_hexes(s) {
        let corners = crate::tactical::hex_corners(s, hex).map(|p| [p[0] / PIXEL, p[1] / PIXEL]);
        let l = lines.get(&hex).unwrap_or(&none);
        let Some(edge) = crate::tactical::inset_corners(corners, HEX_INSET_PX) else { continue };
        if l.band_iv {
            let c = linear(s.palette.roles.text_bright);
            ring(edge, c, HEX_BAND_IV_GLOW, Some(crate::tactical::BAND_IV_DASH));
        } else {
            ring(edge, linear(s.palette.roles.hex), HEX_GLOW, None);
        }
        for (n, seat) in l.seats.iter().enumerate() {
            let d = HEX_INSET_PX + crate::tactical::HEX_LINE_STEP_PX * (n + 1) as f64 / PIXEL;
            let Some(v) = crate::tactical::inset_corners(corners, d) else { break };
            let c = linear(crate::tactical::seat_line_color(s.palette, seat));
            let k = HEX_SEAT_GLOW * seat.brightness as f32 * if seat.established { HEX_ESTABLISHED } else { 1.0 };
            ring(v, c, k, None);
        }
    }
}

/// Path lights (T-175, proposed, R-UI2): one every `PATH_LIGHT_SPACING`
/// hull radii along a path, `PATH_LIGHT_R` hull radii wide, in the path's
/// drive color at `PATH_LIGHT` times its opacity — the same paths tactical
/// draws ([`crate::paths`]), as a faint line of light. At most
/// `PATH_MAX_LIGHTS` in a frame, however many paths are drawn, and none off
/// screen.
pub const PATH_LIGHT_SPACING: f64 = 1.0;
pub const PATH_LIGHT_R: f64 = 0.5;
pub const PATH_LIGHT: f32 = 0.6;
pub const PATH_MAX_LIGHTS: usize = 2000;

/// **The paths, as lights**: marched along each path at an even spacing that
/// carries across its segments, so a path reads as one line of light.
fn path_lights(s: &Scene, ops: &[Op], hull_r: f64, out: &mut Vec<Light>) {
    let (w, h) = (s.camera.width / PIXEL, s.camera.height / PIXEL);
    let m = 4.0 * PATH_LIGHT_R * hull_r;
    let at = |p: [f64; 3]| {
        let q = s.camera.project(p);
        [q[0] / PIXEL, q[1] / PIXEL]
    };
    let step = PATH_LIGHT_SPACING * hull_r;
    let mut lit = 0;
    for path in crate::paths::paths(s, ops) {
        let mut carry = 0.0;
        for seg in path.points.windows(2) {
            let (a, b) = (at(seg[0].pos), at(seg[1].pos));
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let color = linear(seg[0].color);
            let k = PATH_LIGHT * seg[0].alpha as f32;
            let mut d = carry;
            while d < len && lit < PATH_MAX_LIGHTS {
                let f = d / len;
                let p = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
                if p[0] > -m && p[1] > -m && p[0] < w + m && p[1] < h + m {
                    out.push(light(p[0], p[1], PATH_LIGHT_R * hull_r, color, k));
                    lit += 1;
                }
                d += step;
            }
            carry = d - len;
        }
    }
}

/// **The CPU renderer**: adds `lights` into `hdr` (`w × h`), blooms them and
/// resolves into `out`. The reference the GPU renderer is checked against,
/// and the fallback where the browser has no WebGL2.
pub fn rasterize(l: &Lights, w: usize, h: usize, hdr: &mut Hdr, out: &mut Raster, lut: &ToneLut) {
    if hdr.w != w || hdr.h != h {
        *hdr = Hdr::new(w, h);
    }
    hdr.px.iter_mut().for_each(|p| *p = [0.0; 3]);
    for g in &l.scene {
        hdr.splat(g.x as f64, g.y as f64, g.r as f64, g.color, 1.0);
    }
    let mut bloom = hdr.downsample(BLOOM_DOWNSAMPLE);
    bloom.blur(BLUR_RADIUS, BLUR_PASSES);
    bloom.px.iter_mut().for_each(|p| *p = p.map(|c| c * BLOOM_WEIGHT));
    let k = BLOOM_DOWNSAMPLE as f64;
    for g in &l.territory {
        bloom.splat(g.x as f64 / k, g.y as f64 / k, g.r as f64 / k, g.color, 1.0);
    }
    hdr.add_upsampled(&bloom, BLOOM_DOWNSAMPLE, 1.0);
    hdr.resolve(out, lut);
}

/// The juicy frame's size for a camera, juicy pixels.
pub fn frame_size(camera: &crate::camera::Camera) -> (usize, usize) {
    ((camera.width / PIXEL).ceil() as usize, (camera.height / PIXEL).ceil() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;
    use crate::palette::Palette;
    use crate::replay::{tests::TINY, Replay};
    use crate::tactical::plan;

    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

    /// The scene drawn by the CPU renderer, every light where its layout
    /// puts it.
    fn render(s: &Scene, ops: &[Op], hdr: &mut Hdr, out: &mut Raster, lut: &ToneLut) {
        let (w, h) = frame_size(s.camera);
        rasterize(&lights(s, ops), w, h, hdr, out, lut);
    }

    #[test]
    fn the_tone_curve_keeps_black_rises_and_never_passes_white() {
        assert_eq!(tone(0.0), 0.0);
        let xs = [0.01, 0.1, 0.5, 1.0, 4.0, 100.0, 1e6];
        for w in xs.windows(2) {
            assert!(tone(w[0]) < tone(w[1]) || tone(w[1]) == 1.0);
        }
        assert!(xs.iter().all(|&x| tone(x) <= 1.0));
        assert!(tone(10.0) > 0.99);
    }

    /// **The table is the curve over the background, to one code** (T-179):
    /// no light is the background's own code in every channel, and bright
    /// light is white whatever the background.
    #[test]
    fn the_tone_table_matches_the_curve_over_the_ink_to_one_code() {
        let ink = Rgb::from_hex("#1a2433");
        let lut = ToneLut::new(ink);
        let g = linear(ink);
        for (ch, &gc) in g.iter().enumerate() {
            for i in 0..2000 {
                let x = (i as f32 / 100.0).powi(2) / 10.0;
                let exact = encode(over(gc, x)) as i32;
                let got = lut.get(ch, x) as i32;
                assert!((got - exact).abs() <= 1, "channel {ch}, x {x}: {got} against {exact}");
            }
            assert_eq!(lut.get(ch, 1e9), 255, "bright light is white");
        }
        let none = [0, 1, 2].map(|ch| lut.get(ch, 0.0));
        assert_eq!(none, [0x1a, 0x24, 0x33], "no light is the ink");
        assert_eq!([0, 1, 2].map(|ch| lut.get(ch, -1.0)), none);
    }

    /// **The background is the palette's ink**, a neutral at the tone map's
    /// `ink` lightness, and the editor's `ink` moves it (T-179).
    #[test]
    fn the_juicy_background_is_the_palettes_ink() {
        use crate::palette::Settings;
        let dark = Palette::default().roles.ink;
        let light = Palette::with(Settings::parse("ink=0.2").unwrap()).roles.ink;
        let l = |c: Rgb| crate::color::to_oklch(c).l;
        assert!((l(dark) - 0.02).abs() < 0.01 && (l(light) - 0.2).abs() < 0.01, "{} {}", l(dark), l(light));
        assert!((light.r - light.g).abs() < 1e-6 && (light.g - light.b).abs() < 1e-6, "a neutral");
        let lut = ToneLut::new(light);
        let mut out = Raster::new(1, 1);
        Hdr::new(2, 2).resolve(&mut out, &lut);
        let code = |v: f64| (v * 255.0).round() as u8;
        let want = [code(light.r), code(light.g), code(light.b)];
        let got = out.get(1, 1).unwrap();
        assert!(
            got.iter().zip(want).all(|(&a, b)| a.abs_diff(b) <= 1),
            "an empty sky resolves to the ink: {got:?} {want:?}"
        );
    }

    #[test]
    fn a_light_peaks_where_it_stands_falls_off_and_lights_add() {
        let mut h = Hdr::new(21, 21);
        h.splat(10.5, 10.5, 2.0, WHITE, 1.0);
        let at = |x: i64| h.get(x, 10)[0];
        assert_eq!(at(10), 1.0);
        assert!(at(10) > at(12) && at(12) > at(14) && at(14) > at(18));
        assert_eq!(h.get(0, 0)[0], 0.0, "cut beyond four radii");
        let one = h.energy();
        h.splat(10.5, 10.5, 2.0, WHITE, 1.0);
        assert!((h.energy() - 2.0 * one).abs() < 1e-3 * one);
    }

    #[test]
    fn blur_spreads_light_and_keeps_its_sum() {
        let mut h = Hdr::new(16, 16);
        h.px[8 * 16 + 8] = [9.0, 0.0, 0.0];
        let before = h.energy();
        h.blur(2, 3);
        assert!((h.energy() - before).abs() < 1e-4, "{} {}", h.energy(), before);
        assert!(h.get(8, 8)[0] < 9.0 && h.get(10, 9)[0] > 0.0);
        let mut edge = Hdr::new(8, 8);
        edge.px[0] = [1.0, 1.0, 1.0];
        let e = edge.energy();
        edge.blur(3, 2);
        assert!((edge.energy() - e).abs() < 1e-5, "light at the edge is held");
    }

    #[test]
    fn a_downsample_keeps_the_mean_and_an_upsample_restores_a_flat_field() {
        let mut h = Hdr::new(8, 8);
        h.px.iter_mut().for_each(|p| *p = [0.5, 0.25, 1.0]);
        let s = h.downsample(4);
        assert_eq!((s.w, s.h), (2, 2));
        assert_eq!(s.get(1, 1), [0.5, 0.25, 1.0]);
        let mut back = Hdr::new(8, 8);
        back.add_upsampled(&s, 4, 2.0);
        assert!(back.px.iter().all(|p| (p[0] - 1.0).abs() < 1e-6 && (p[2] - 2.0).abs() < 1e-6));
    }

    #[test]
    fn lights_widen_with_the_level_of_detail() {
        let (g, s, y) = (light_scale(Lod::Galaxy), light_scale(Lod::Sector), light_scale(Lod::System));
        assert!(g.1 < s.1 && s.1 < y.1, "star halo");
        assert!(g.3 < s.3 && s.3 < y.3, "hull light");
    }

    #[test]
    fn every_hull_and_world_has_a_light_where_it_stands() {
        let replay = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let palette = Palette::default();
        let view = replay.view_at(10.0);
        let s = crate::tactical::Scene {
            replay: &replay,
            view: &view,
            camera: &camera,
            palette: &palette,
            selected: None,
            rate: 1.0,
        };
        let l = lights(&s, &plan(&s));
        let lit = |p: [f64; 3]| {
            let q = camera.project(p);
            l.scene.iter().any(|g| (g.x as f64 - q[0]).abs() < 1e-3 && (g.y as f64 - q[1]).abs() < 1e-3)
        };
        assert!(view.hulls.iter().all(|h| lit(h.row.pos)));
        assert!(replay.planets.iter().all(|p| lit(p.pos)));
        assert_eq!(l.territory.len(), 3, "the three owned worlds glow");
        assert_eq!(std::mem::size_of::<Light>(), LIGHT_FLOATS * 4);
    }

    /// **Hexes are lines of light in juicy mode too, with a line per seat**
    /// (T-159, T-162): every active hex is lit when the tactical grid would be
    /// drawn and not when it would be a fill; each seat with works in a hex
    /// adds a line in its color, brighter once established; a `Band IV` world
    /// dashes the grid line bright; and a view zoomed until one hex is many
    /// screens wide lights only what is on screen.
    #[test]
    fn hexes_are_lines_of_light_with_a_line_per_seat() {
        let replay = Replay::from_json(TINY).unwrap();
        let palette = Palette::default();
        let hex_lights = |r: &Replay, camera: &Camera, t: f64| {
            let view = r.view_at(t);
            let s =
                crate::tactical::Scene { replay: r, view: &view, camera, palette: &palette, selected: None, rate: 1.0 };
            let mut out = Vec::new();
            hex_lights(&s, &mut out);
            out
        };
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let grid = linear(palette.roles.hex).map(|c| c * HEX_GLOW);
        let is = |g: &Light, c: [f32; 3]| g.color.iter().zip(c).all(|(a, b)| (a - b).abs() < 1e-6);
        let lit = hex_lights(&replay, &camera, 10.0);
        assert!(lit.iter().any(|g| is(g, grid)), "the grid's line");
        let view = replay.view_at(10.0);
        let scene = crate::tactical::Scene {
            replay: &replay,
            view: &view,
            camera: &camera,
            palette: &palette,
            selected: None,
            rate: 1.0,
        };
        let lines = crate::tactical::hex_lines(&scene);
        let mut seats = std::collections::BTreeSet::new();
        for l in lines.values().flat_map(|l| &l.seats) {
            let k = HEX_SEAT_GLOW * l.brightness as f32;
            let c = linear(crate::tactical::seat_line_color(&palette, l)).map(|c| c * k);
            assert!(lit.iter().any(|g| is(g, c)), "seat {}'s line, in its color at its brightness", l.seat);
            seats.insert(l.seat);
        }
        assert_eq!(seats.len(), 2, "both seats hold works in the fixture");
        let sum = |l: &[Light]| l.iter().map(|g| g.color.iter().sum::<f32>()).sum::<f32>();
        let mut established = replay.clone();
        for f in &mut established.frames {
            for w in &mut f.hex_works {
                w.work_years += 10.0 * crate::tactical::ESTABLISHED_WORK_YEARS;
            }
        }
        assert!(sum(&hex_lights(&established, &camera, 10.0)) > 1.5 * sum(&lit), "established lines are bright");
        let mut band_iv = replay.clone();
        band_iv.frames[1].hex_works.iter_mut().for_each(|w| w.band_iv = true);
        let dashed = linear(palette.roles.text_bright).map(|c| c * HEX_BAND_IV_GLOW);
        let iv = hex_lights(&band_iv, &camera, 10.0);
        assert!(iv.iter().any(|g| is(g, dashed)) && !lit.iter().any(|g| is(g, dashed)), "Band IV dashes the edge");

        let mut far = camera;
        far.scale = crate::tactical::MIN_HEX_PX * crate::tactical::PIXEL / 121.0 * 0.9;
        assert!(hex_lights(&replay, &far, 10.0).is_empty(), "too small to draw: no lights");

        let mut near = camera;
        near.scale *= 1.0e4;
        let close = hex_lights(&replay, &near, 10.0);
        let screen = (320.0f64 + 200.0) * 2.0 / HEX_DOT_PX;
        assert!(close.len() as f64 <= 4.0 * screen, "a hex wider than the screen lights the screen: {}", close.len());
        assert!(close.iter().all(|g| (-5.0..=325.0).contains(&g.x) && (-5.0..=205.0).contains(&g.y)));
    }

    /// **Below the galaxy level a stack is a cluster of dots** (T-163): one
    /// dot per hull in its seat's color, so losing a hull loses a dot; seats
    /// sharing a place stand apart, each in its own color; and a hit
    /// is a ring about the cluster, so the seat still shows inside it.
    #[test]
    fn a_stack_is_a_dot_per_hull_and_seats_stand_apart() {
        let replay = Replay::from_json(TINY).unwrap();
        let palette = Palette::default();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        camera.scale *= 40.0; // below the galaxy level
        assert_ne!(camera.lod(), Lod::Galaxy);
        let base = replay.view_at(0.0);
        let five = *base.hulls.iter().find(|h| h.hull.id == 5).unwrap();
        camera.center = [five.row.pos[0], five.row.pos[1]];
        let crowd = |zero: u64, one: u64, hit: bool| {
            let mut view = base.clone();
            view.hulls.clear();
            for k in 0..zero + one {
                let mut c = five;
                (c.hull.id, c.row.id, c.hull.owner, c.hit) = (100 + k, 100 + k, usize::from(k >= zero), hit && k == 0);
                view.hulls.push(c);
            }
            view
        };
        let lights_of = |view: &crate::replay::View| {
            let s = crate::tactical::Scene {
                replay: &replay,
                view,
                camera: &camera,
                palette: &palette,
                selected: None,
                rate: 1.0,
            };
            lights(&s, &plan(&s)).scene
        };
        let hull_r = light_scale(camera.lod()).3;
        let dots = |l: &[Light], seat: usize| {
            let c = linear(palette.seat(seat)).map(|c| c * 2.0);
            l.iter()
                .filter(|g| (g.r as f64 - CLUSTER_DOT_R * hull_r).abs() < 1e-6)
                .filter(|g| g.color.iter().zip(c).all(|(a, b)| (a - b).abs() < 1e-5))
                .map(|g| [g.x as f64, g.y as f64])
                .collect::<Vec<_>>()
        };
        assert_eq!(dots(&lights_of(&crowd(6, 0, false)), 0).len(), 6, "six hulls, six dots");
        // Two of six heading home: a cluster of their own, in the retreat color.
        let mut back = crowd(6, 0, false);
        for h in back.hulls.iter_mut().take(2) {
            h.row.withdrawing = true;
        }
        let l = lights_of(&back);
        let retreat = linear(palette.status(Status::Retreat)).map(|c| c * 2.0);
        let homeward = l
            .iter()
            .filter(|g| (g.r as f64 - CLUSTER_DOT_R * hull_r).abs() < 1e-6)
            .filter(|g| g.color.iter().zip(retreat).all(|(a, b)| (a - b).abs() < 1e-5))
            .count();
        assert_eq!((dots(&l, 0).len(), homeward), (4, 2), "four on their role in the seat's color, two heading home");
        assert_eq!(dots(&lights_of(&crowd(5, 0, false)), 0).len(), 5, "one lost, one dot fewer");
        let both = lights_of(&crowd(6, 4, true));
        let (a, b) = (dots(&both, 0), dots(&both, 1));
        assert_eq!((a.len(), b.len()), (6, 4));
        let mean = |v: &[[f64; 2]]| {
            [v.iter().map(|p| p[0]).sum::<f64>() / v.len() as f64, v.iter().map(|p| p[1]).sum::<f64>() / v.len() as f64]
        };
        let (ca, cb) = (mean(&a), mean(&b));
        let reach = |v: &[[f64; 2]], c: [f64; 2]| {
            v.iter().map(|p| ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt()).fold(0.0, f64::max)
        };
        let gap = ((ca[0] - cb[0]).powi(2) + (ca[1] - cb[1]).powi(2)).sqrt();
        assert!(gap > reach(&a, ca) + reach(&b, cb), "the two seats' clusters do not overlap: {gap}");
        let hit = linear(palette.status(Status::Hit)).map(|c| c * HIT_RING);
        let ring: Vec<&Light> =
            both.iter().filter(|g| g.color.iter().zip(hit).all(|(x, y)| (x - y).abs() < 1e-5)).collect();
        assert!(ring.len() >= 8, "a hit is a ring of lights");
        assert!(
            ring.iter().all(|g| {
                let d = ((g.x as f64 - ca[0]).powi(2) + (g.y as f64 - ca[1]).powi(2)).sqrt();
                d > reach(&a, ca)
            }),
            "outside the hit seat's dots"
        );
    }

    /// **A cluster laid out afresh slides to its new layout** (T-184): when a
    /// rival's hulls arrive and the seat's cluster steps aside, each hull's
    /// light moves no more per frame than the easing allows, closes on its new
    /// place, and is there within a second — where an uneased frame puts it.
    #[test]
    fn a_cluster_laid_out_afresh_slides_to_its_new_layout() {
        let replay = Replay::from_json(TINY).unwrap();
        let palette = Palette::default();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        camera.scale *= 40.0;
        let base = replay.view_at(0.0);
        let five = *base.hulls.iter().find(|h| h.hull.id == 5).unwrap();
        camera.center = [five.row.pos[0], five.row.pos[1]];
        let crowd = |zero: u64, one: u64| {
            let mut view = base.clone();
            view.hulls.clear();
            for k in 0..zero + one {
                let mut c = five;
                (c.hull.id, c.row.id, c.hull.owner) = (100 + k, 100 + k, usize::from(k >= zero));
                view.hulls.push(c);
            }
            view
        };
        fn at<'a>(r: &'a Replay, v: &'a crate::replay::View, c: &'a Camera, p: &'a Palette) -> Scene<'a> {
            Scene { replay: r, view: v, camera: c, palette: p, selected: None, rate: 1.0 }
        }
        let scene = |view| at(&replay, view, &camera, &palette);
        let (alone, met) = (crowd(6, 0), crowd(6, 4));
        let mut tween = Tween::default();
        let s = scene(&alone);
        lights_eased(&s, &plan(&s), &mut tween, 1.0 / 60.0);
        assert!(!tween.moving(), "a first frame shows every light at its place");
        let before: Vec<[f64; 2]> = (100..106).map(|id| tween.offset(id).unwrap()).collect();
        let s = scene(&met);
        let ops = plan(&s);
        let settled = lights(&s, &ops);
        let mut target = Tween::default();
        lights_eased(&s, &ops, &mut target, f64::INFINITY);
        let gap = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
        let widest =
            (100..106).map(|id| gap(before[id as usize - 100], target.offset(id).unwrap())).fold(0.0, f64::max);
        assert!(widest > 5.0, "the seat's cluster steps aside: {widest:.1} px");
        let step = 1.0 - (-1.0 / 60.0 / TWEEN_SECONDS).exp();
        let mut last = before.clone();
        let mut frames = 0;
        while tween.moving() || frames == 0 {
            let l = lights_eased(&s, &ops, &mut tween, 1.0 / 60.0);
            frames += 1;
            for id in 100..106u64 {
                let (now, goal, was) = (tween.offset(id).unwrap(), target.offset(id).unwrap(), last[id as usize - 100]);
                let most = step * gap(was, goal) + TWEEN_SNAP_PX;
                assert!(gap(now, was) <= most, "frame {frames}: a light moves at most the easing's share");
                assert!(gap(now, goal) <= gap(was, goal), "frame {frames}: and closes on its place");
                last[id as usize - 100] = now;
            }
            if !tween.moving() {
                assert_eq!(l, settled, "settled, the frame is the uneased one");
            }
            assert!(frames <= 60, "there within a second");
        }
        assert!(frames > 5, "and it took some frames: {frames}");
    }

    /// **A hull that moved draws a trail back to where it was; a pan does
    /// not** (T-184): the trail is where the light was last frame, through
    /// this frame's camera, so only the hull's own motion leaves one.
    #[test]
    fn a_moving_hull_draws_a_trail_and_a_pan_does_not() {
        let replay = Replay::from_json(TINY).unwrap();
        let palette = Palette::default();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        camera.scale *= 40.0;
        let mut view = replay.view_at(0.0);
        view.hulls.retain(|h| h.hull.id == 5);
        camera.center = [view.hulls[0].row.pos[0], view.hulls[0].row.pos[1]];
        let lit = |view: &crate::replay::View, camera: &Camera, tween: &mut Tween| {
            let s = Scene { replay: &replay, view, camera, palette: &palette, selected: None, rate: 1.0 };
            lights_eased(&s, &plan(&s), tween, 1.0 / 60.0).scene.len()
        };
        let mut tween = Tween::default();
        let still = lit(&view, &camera, &mut tween);
        assert_eq!(lit(&view, &camera, &mut tween), still, "a hull standing still draws no trail");
        let mut panned = camera;
        panned.center[0] += 40.0 / camera.scale;
        assert_eq!(lit(&view, &panned, &mut tween), still, "a pan draws no trail");
        let mut moved = view.clone();
        moved.hulls[0].row.pos[0] += 30.0 / camera.scale;
        let trailed = lit(&moved, &panned, &mut tween);
        assert!(trailed > still, "a hull that moved 30 px draws a trail: {still} → {trailed} lights");
        moved.hulls[0].row.pos[0] += 10.0 * camera.width / camera.scale;
        assert_eq!(lit(&moved, &panned, &mut tween), still, "a jump of ten screens is not motion");
    }

    #[test]
    fn clipping_keeps_the_part_of_a_segment_on_screen() {
        let clip = |a, b, w: f64, h: f64, m: f64| clip_segment(a, b, [-m, -m, w + m, h + m]);
        assert_eq!(clip([-10.0, 5.0], [20.0, 5.0], 10.0, 10.0, 0.0), Some(([0.0, 5.0], [10.0, 5.0])));
        assert_eq!(clip([2.0, 2.0], [3.0, 3.0], 10.0, 10.0, 0.0), Some(([2.0, 2.0], [3.0, 3.0])));
        assert_eq!(clip([-5.0, -5.0], [-1.0, 20.0], 10.0, 10.0, 0.0), None);
        assert_eq!(clip([5.0, -5.0], [5.0, 15.0], 10.0, 10.0, 1.0), Some(([5.0, -1.0], [5.0, 11.0])));
    }

    /// **A hex's dots stand still on it as the view pans** (T-176): after any
    /// pan, every dot well inside the screen in both views is where the pan
    /// moved it, wherever the screen cuts the hex's edges. Counted from the
    /// cut, the dots slid along an edge as the view moved — the `Band IV`
    /// dashes most visibly.
    #[test]
    fn a_hexs_dots_move_with_the_hex_when_the_view_pans() {
        let mut replay = Replay::from_json(TINY).unwrap();
        replay.frames.iter_mut().flat_map(|f| f.hex_works.iter_mut()).for_each(|w| w.band_iv = true);
        let palette = Palette::default();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        camera.zoom_at([160.0, 100.0], 1.0);
        let view = replay.view_at(10.0);
        let dots = |c: &Camera| {
            let s = crate::tactical::Scene {
                replay: &replay,
                view: &view,
                camera: c,
                palette: &palette,
                selected: None,
                rate: 1.0,
            };
            let mut out = Vec::new();
            hex_lights(&s, &mut out);
            out.iter().map(|g| [g.x as f64, g.y as f64]).collect::<Vec<_>>()
        };
        let (w, h) = (camera.width / PIXEL, camera.height / PIXEL);
        let inside = |p: [f64; 2]| p[0] > 8.0 && p[1] > 8.0 && p[0] < w - 8.0 && p[1] < h - 8.0;
        let key = |p: [f64; 2]| ((p[0] * 16.0).round() as i64, (p[1] * 16.0).round() as i64);
        let base = dots(&camera);
        for (dx, dy) in [(0.37, 0.0), (11.3, -7.9), (-23.0, 15.5)] {
            let mut moved = camera;
            moved.pan(dx, dy);
            let after: std::collections::BTreeSet<_> = dots(&moved).into_iter().map(key).collect();
            let expected: Vec<[f64; 2]> =
                base.iter().map(|p| [p[0] + dx / PIXEL, p[1] + dy / PIXEL]).filter(|p| inside(*p)).collect();
            assert!(expected.len() > 20, "dots on screen in both views");
            let kept = expected.iter().filter(|p| after.contains(&key(**p))).count();
            assert_eq!(kept, expected.len(), "panned ({dx}, {dy}): every dot where the pan moved it");
        }
    }

    #[test]
    fn a_hull_outshines_any_world_but_a_homeworld() {
        // The author found bright worlds made the scene illegible: the things
        // that move and fight carry it, and a world is a faint point.
        let replay = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let palette = Palette::default();
        let view = replay.view_at(10.0);
        let s = crate::tactical::Scene {
            replay: &replay,
            view: &view,
            camera: &camera,
            palette: &palette,
            selected: None,
            rate: 1.0,
        };
        let ops = plan(&s);
        let (mut hdr, mut out) = (Hdr::new(1, 1), Raster::new(1, 1));
        render(&s, &ops, &mut hdr, &mut out, &ToneLut::new(palette.roles.ink));
        let lum = |p: [f64; 3]| {
            let at = camera.project(p).map(|c| (c / PIXEL).floor() as i64);
            out.get(at[0], at[1]).unwrap().iter().map(|&c| c as u32).sum::<u32>()
        };
        let apart = |p: [f64; 3], others: &[[f64; 3]]| {
            let q = camera.project(p);
            others.iter().all(|&o| {
                let r = camera.project(o);
                (q[0] - r[0]).hypot(q[1] - r[1]) > 8.0
            })
        };
        let hull_at: Vec<[f64; 3]> = view.hulls.iter().map(|h| h.row.pos).collect();
        let world_at: Vec<[f64; 3]> = replay.planets.iter().map(|p| p.pos).collect();
        let worlds: Vec<u32> =
            replay.planets.iter().filter(|p| !p.home && apart(p.pos, &hull_at)).map(|p| lum(p.pos)).collect();
        let hulls: Vec<u32> = view
            .hulls
            .iter()
            .filter(|h| !h.row.wrecked && apart(h.row.pos, &world_at))
            .map(|h| lum(h.row.pos))
            .collect();
        assert!(!worlds.is_empty() && !hulls.is_empty(), "the scene has a lone world and a lone hull");
        let (brightest_world, dimmest_hull) = (worlds.iter().max().unwrap(), hulls.iter().min().unwrap());
        assert!(dimmest_hull > brightest_world, "hull {dimmest_hull} against world {brightest_world}");
    }

    #[test]
    fn every_entity_is_a_light_at_its_tactical_position() {
        let replay = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let palette = Palette::default();
        let view = replay.view_at(10.0);
        let s = crate::tactical::Scene {
            replay: &replay,
            view: &view,
            camera: &camera,
            palette: &palette,
            selected: None,
            rate: 1.0,
        };
        let ops = plan(&s);
        let (mut hdr, mut out) = (Hdr::new(1, 1), Raster::new(1, 1));
        render(&s, &ops, &mut hdr, &mut out, &ToneLut::new(palette.roles.ink));
        assert_eq!((out.w, out.h), (320, 200));
        let lum = |p: [u8; 3]| p[0] as u32 + p[1] as u32 + p[2] as u32;
        let dark = lum(out.get(0, 0).unwrap());
        let px = |p: [f64; 3]| camera.project(p).map(|c| (c / PIXEL).floor() as i64);
        for h in &view.hulls {
            let at = px(h.row.pos);
            assert!(lum(out.get(at[0], at[1]).unwrap()) > dark + 60, "hull {}", h.hull.id);
        }
        for p in &replay.planets {
            let at = px(p.pos);
            assert!(lum(out.get(at[0], at[1]).unwrap()) > dark + 60, "world {}", p.id);
        }
    }

    /// **Juicy draws the paths tactical draws, as lights** (T-175): none while
    /// no stack is large and nothing is selected; for a selected hull, lights
    /// along its past and its way ahead, never farther apart than the
    /// spacing, and none off screen.
    #[test]
    fn juicy_draws_a_selected_hulls_path_as_a_string_of_lights() {
        let replay = Replay::from_json(TINY).unwrap();
        let palette = Palette::default();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let view = replay.view_at(5.0);
        let hull_r = light_scale(camera.lod()).3;
        let path_of = |selected| {
            let s = Scene { replay: &replay, view: &view, camera: &camera, palette: &palette, selected, rate: 1.0 };
            let mut out = Vec::new();
            path_lights(&s, &plan(&s), hull_r, &mut out);
            out
        };
        assert!(path_of(None).is_empty(), "no large stack, no selection: no path");
        let l = path_of(Some(crate::tactical::Pick::Hull(5)));
        assert!(l.len() > 2, "the selected hull's path is lit: {}", l.len());
        assert!(l.iter().all(|g| (g.r as f64 - PATH_LIGHT_R * hull_r).abs() < 1e-6));
        let step = PATH_LIGHT_SPACING * hull_r;
        let gaps = l.windows(2).map(|w| ((w[1].x - w[0].x) as f64).hypot((w[1].y - w[0].y) as f64));
        assert!(gaps.clone().all(|d| d <= step + 1e-3), "lights no farther apart than the spacing");
        assert!(gaps.filter(|&d| d > 0.5 * step).count() > l.len() / 2, "and mostly a spacing apart");
        let (w, h, m) = (camera.width / PIXEL, camera.height / PIXEL, 4.0 * PATH_LIGHT_R * hull_r);
        assert!(l
            .iter()
            .all(|g| (g.x as f64) > -m && (g.y as f64) > -m && (g.x as f64) < w + m && (g.y as f64) < h + m));
    }
}
