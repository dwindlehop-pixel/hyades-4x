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
pub const BLOOM_WEIGHT: f32 = 0.6;
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
        for (p, o) in self.px.iter().zip(out.px.chunks_exact_mut(4)) {
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
            if *color != s.palette.roles.world_dim {
                let strength = if ring.is_some() { 0.5 } else { 0.12 };
                out.territory.push(light(p[0], p[1], glow, linear(*color), strength));
            }
            out.scene.push(light(p[0], p[1], halo, star, 0.35));
            out.scene.push(light(p[0], p[1], core, star, if ring.is_some() { 4.0 } else { 1.5 }));
        }
    }
    for op in ops {
        let Op::Hull { s: sp, at, outline, vector, hit, .. } = op else { continue };
        let p = pos(*sp);
        let wreck = *outline == s.palette.status(Status::Wreck);
        if let Some((end, c)) = vector {
            // The plume trails opposite the vector for a drive, ahead of it
            // when braking. The vector's end is in tactical pixels; keep its
            // offset from the hull and apply it at the hull's exact position.
            let k = crate::tactical::PIXEL / PIXEL;
            let e = [p[0] + (end[0] - at[0]) as f64 * k, p[1] + (end[1] - at[1]) as f64 * k];
            let tail = if *c == s.palette.status(Status::Braking) { e } else { [2.0 * p[0] - e[0], 2.0 * p[1] - e[1]] };
            streak(&mut out.scene, p, tail, hull_r, linear(*c), 2.5);
        }
        let (c, k) = if wreck { (linear(s.palette.status(Status::Wreck)), 0.6) } else { (linear(*outline), 2.0) };
        out.scene.push(light(p[0], p[1], hull_r, c, k));
        if *hit {
            out.scene.push(light(p[0], p[1], 3.0 * hull_r, linear(s.palette.status(Status::Hit)), 6.0));
        }
    }
    out
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
    use crate::tactical::{plan, to_raster};

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
        let s =
            crate::tactical::Scene { replay: &replay, view: &view, camera: &camera, palette: &palette, selected: None };
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

    #[test]
    fn every_entity_is_a_light_at_its_tactical_position() {
        let replay = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        let palette = Palette::default();
        let view = replay.view_at(10.0);
        let s =
            crate::tactical::Scene { replay: &replay, view: &view, camera: &camera, palette: &palette, selected: None };
        let ops = plan(&s);
        let (mut hdr, mut out) = (Hdr::new(1, 1), Raster::new(1, 1));
        render(&s, &ops, &mut hdr, &mut out, &ToneLut::new());
        assert_eq!((out.w, out.h), (320, 200));
        let lum = |p: [u8; 3]| p[0] as u32 + p[1] as u32 + p[2] as u32;
        let dark = lum(out.get(0, 0).unwrap());
        for h in &view.hulls {
            let at = to_raster(camera.project(h.row.pos), PIXEL);
            assert!(lum(out.get(at[0], at[1]).unwrap()) > dark + 60, "hull {}", h.hull.id);
        }
        for p in &replay.planets {
            let at = to_raster(camera.project(p.pos), PIXEL);
            assert!(lum(out.get(at[0], at[1]).unwrap()) > dark + 60, "world {}", p.id);
        }
    }
}
