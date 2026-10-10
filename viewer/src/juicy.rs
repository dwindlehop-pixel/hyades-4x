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
//! | a burning drive | a plume behind the hull, in the drive's status color, brighter with acceleration |
//! | a hit | a flash in the hit color |
//! | a wreck | a dim ember |
//! | a hex | a dim line of lights inset from its edge; inside it a line per seat with works there, brighter with the seat's work-years; dashed bright where a world holds `Band IV` works |

use crate::camera::Lod;
use crate::color::Rgb;
use crate::palette::Status;
use crate::raster::Raster;
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
/// hulls brighten the dots instead). Seats sharing a stack square stand
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

fn linear(c: Rgb) -> [f32; 3] {
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
            o.copy_from_slice(&[lut.get(p[0]), lut.get(p[1]), lut.get(p[2]), 255]);
        }
    }
}

/// [`tone`] then sRGB encoding, tabulated: two transcendentals per channel per
/// pixel were most of a frame. Indexed by the light's own float bits —
/// exponent and the top eight mantissa bits — so every octave of brightness
/// gets 256 entries and no arithmetic is spent finding one.
#[derive(Clone, Debug, PartialEq)]
pub struct ToneLut(Vec<u8>);

impl ToneLut {
    /// Below this the code is 0; past `HI` the curve is within half a code of
    /// white.
    const LO: f32 = 1.0 / (1 << 20) as f32;
    const HI: f32 = 16.0;
    const SHIFT: u32 = 15;

    pub fn new() -> ToneLut {
        let (lo, hi) = (Self::LO.to_bits(), Self::HI.to_bits());
        let n = ((hi - lo) >> Self::SHIFT) + 1;
        ToneLut(
            (0..n)
                .map(|i| {
                    let mid = f32::from_bits(lo + (i << Self::SHIFT) + (1 << (Self::SHIFT - 1)));
                    encode(tone(mid))
                })
                .collect(),
        )
    }

    pub fn get(&self, x: f32) -> u8 {
        let i = (x.max(0.0).to_bits().saturating_sub(Self::LO.to_bits()) >> Self::SHIFT) as usize;
        self.0[i.min(self.0.len() - 1)]
    }
}

impl Default for ToneLut {
    fn default() -> Self {
        ToneLut::new()
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

/// The scene's lights. `ops` is the tactical plan: the same entities at the
/// same positions, so the two modes cannot disagree about where anything is.
pub fn lights(s: &Scene, ops: &[Op]) -> Lights {
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
    hexes(s, &mut out.scene);
    // Below the galaxy level a stack is a cluster of dots, one per hull, so
    // its count reads and a hit does not hide whose it is (T-163).
    let clustered = s.camera.lod() != Lod::Galaxy;
    // Keyed by stack square, seat, and whether the hulls are heading home.
    let mut groups: std::collections::BTreeMap<((i64, i64), usize, bool), Vec<usize>> = Default::default();
    let hull_ops: Vec<&Op> = ops.iter().filter(|o| matches!(o, Op::Hull { .. })).collect();
    for (i, (op, h)) in hull_ops.iter().zip(&s.view.hulls).enumerate() {
        let Op::Hull { s: sp, at, outline, vector, hit, quiet, wreck, .. } = op else { continue };
        let p = pos(*sp);
        if clustered && wreck.is_none() && !*quiet {
            groups
                .entry((crate::tactical::stack_cell(s, h.row.pos), h.hull.owner, h.row.withdrawing))
                .or_default()
                .push(i);
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
        out.scene.push(light(p[0], p[1], r, c, k));
        if *hit {
            ring(&mut out.scene, p, 3.0 * hull_r, hull_r, linear(s.palette.status(Status::Hit)));
        }
    }
    // The clusters: each stack square's hulls of one seat about their mean
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
        let middle = mean(&all);
        for (k, ((seat, back), members)) in seats.iter().enumerate() {
            let (sin, cos) = (std::f64::consts::TAU * k as f64 / m as f64).sin_cos();
            let c = if m > 1 { [middle[0] + apart * cos, middle[1] + apart * sin] } else { mean(members) };
            // Hulls heading home are their own cluster, in the retreat color
            // (T-164): the seat's own cluster counts only hulls on their role.
            let color = linear(if *back { s.palette.status(Status::Retreat) } else { s.palette.seat(*seat) });
            let n = members.len();
            let dots = n.min(CLUSTER_MAX_DOTS);
            let k_dot = 2.0 * n as f32 / dots as f32;
            for d in 0..dots {
                let (sin, cos) = (GOLDEN_ANGLE * d as f64).sin_cos();
                // The first dot at the center: a lone hull is where it stands.
                let r = spacing * (d as f64).sqrt();
                out.scene.push(light(c[0] + r * cos, c[1] + r * sin, CLUSTER_DOT_R * hull_r, color, k_dot));
            }
            if let Some(Op::Hull { at, vector: Some((end, col)), .. }) = members.first().map(|&i| hull_ops[i]) {
                plume(&mut out.scene, s, c, *at, *end, *col, hull_r);
            }
            if members.iter().any(|&i| matches!(hull_ops[i], Op::Hull { hit: true, .. })) {
                ring(&mut out.scene, c, radius(n) + 2.0 * hull_r, hull_r, linear(s.palette.status(Status::Hit)));
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
fn hexes(s: &Scene, out: &mut Vec<Light>) {
    if !crate::tactical::hexes_shown(s) {
        return;
    }
    let lines = crate::tactical::hex_lines(s);
    let none = crate::tactical::HexLines::default();
    let (w, h) = (s.camera.width / PIXEL, s.camera.height / PIXEL);
    let mut ring = |v: [[f64; 2]; 6], color: [f32; 3], k: f32, dash: Option<(usize, usize)>| {
        for e in 0..6 {
            let Some((a, b)) = clip(v[e], v[(e + 1) % 6], w, h, 4.0 * HEX_DOT_R) else { continue };
            let n =
                (((b[0] - a[0]) * (b[0] - a[0]) + (b[1] - a[1]) * (b[1] - a[1])).sqrt() / HEX_DOT_PX).floor() as usize;
            for i in 0..=n {
                if dash.is_some_and(|(on, off)| i % (on + off) >= on) {
                    continue;
                }
                let f = if n == 0 { 0.0 } else { i as f64 / n as f64 };
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

/// The part of segment `a`–`b` inside `[−m, w + m] × [−m, h + m]`
/// (Liang–Barsky), or `None` when none of it is.
fn clip(a: [f64; 2], b: [f64; 2], w: f64, h: f64, m: f64) -> Option<([f64; 2], [f64; 2])> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-d[0], a[0] + m), (d[0], w + m - a[0]), (-d[1], a[1] + m), (d[1], h + m - a[1])] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    (t0 <= t1).then(|| ([a[0] + d[0] * t0, a[1] + d[1] * t0], [a[0] + d[0] * t1, a[1] + d[1] * t1]))
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

/// [`lights`] then [`rasterize`], at the camera's size.
pub fn render(s: &Scene, ops: &[Op], hdr: &mut Hdr, out: &mut Raster, lut: &ToneLut) {
    let (w, h) = ((s.camera.width / PIXEL).ceil() as usize, (s.camera.height / PIXEL).ceil() as usize);
    rasterize(&lights(s, ops), w, h, hdr, out, lut);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;
    use crate::palette::Palette;
    use crate::replay::{tests::TINY, Replay};
    use crate::tactical::plan;

    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

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

    #[test]
    fn the_tone_table_matches_the_curve_to_one_code() {
        let lut = ToneLut::new();
        for i in 0..2000 {
            let x = (i as f32 / 100.0).powi(2) / 10.0;
            let exact = encode(tone(x)) as i32;
            assert!((lut.get(x) as i32 - exact).abs() <= 1, "x {x}: {} against {exact}", lut.get(x));
        }
        assert_eq!((lut.get(0.0), lut.get(-1.0), lut.get(1e9)), (0, 0, 255));
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
            hexes(&s, &mut out);
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
    /// sharing a stack square stand apart, each in its own color; and a hit
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

    #[test]
    fn clipping_keeps_the_part_of_a_segment_on_screen() {
        assert_eq!(clip([-10.0, 5.0], [20.0, 5.0], 10.0, 10.0, 0.0), Some(([0.0, 5.0], [10.0, 5.0])));
        assert_eq!(clip([2.0, 2.0], [3.0, 3.0], 10.0, 10.0, 0.0), Some(([2.0, 2.0], [3.0, 3.0])));
        assert_eq!(clip([-5.0, -5.0], [-1.0, 20.0], 10.0, 10.0, 0.0), None);
        assert_eq!(clip([5.0, -5.0], [5.0, 15.0], 10.0, 10.0, 1.0), Some(([5.0, -1.0], [5.0, 11.0])));
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
        render(&s, &ops, &mut hdr, &mut out, &ToneLut::new());
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
        render(&s, &ops, &mut hdr, &mut out, &ToneLut::new());
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
}
