//! **The framebuffer** both renderers draw into: 8-bit RGBA, row-major, the
//! layout a canvas `ImageData` takes. Every drawing call clips to the buffer,
//! so a caller may hand it coordinates anywhere.

use crate::color::Rgb;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Raster {
    pub w: usize,
    pub h: usize,
    /// `w × h × 4` bytes, RGBA, alpha 255.
    pub px: Vec<u8>,
}

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster { w, h, px: vec![255; w * h * 4] }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.px.resize(w * h * 4, 255);
    }

    pub fn clear(&mut self, c: Rgb) {
        let [r, g, b] = c.to_ints();
        for p in self.px.as_chunks_mut::<4>().0 {
            p.copy_from_slice(&[r, g, b, 255]);
        }
    }

    fn at(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            None
        } else {
            Some((y as usize * self.w + x as usize) * 4)
        }
    }

    pub fn get(&self, x: i64, y: i64) -> Option<[u8; 3]> {
        self.at(x, y).map(|i| [self.px[i], self.px[i + 1], self.px[i + 2]])
    }

    pub fn set(&mut self, x: i64, y: i64, c: Rgb) {
        if let Some(i) = self.at(x, y) {
            let [r, g, b] = c.to_ints();
            self.px[i..i + 3].copy_from_slice(&[r, g, b]);
        }
    }

    /// A one-pixel line, endpoints included (Bresenham).
    pub fn line(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, c: Rgb) {
        // A line far off the buffer is clipped to a box around it first, so a
        // zoomed-in hex edge thousands of pixels long costs what it shows.
        let pad = (self.w + self.h) as i64;
        if (x0 < -pad && x1 < -pad)
            || (y0 < -pad && y1 < -pad)
            || (x0 > pad + self.w as i64 && x1 > pad + self.w as i64)
            || (y0 > pad + self.h as i64 && y1 > pad + self.h as i64)
        {
            return;
        }
        let n = (x1 - x0).abs().max((y1 - y0).abs());
        if n > 4 * pad {
            // Too long to walk: split at the midpoint and drop off-buffer halves.
            let (mx, my) = ((x0 + x1) / 2, (y0 + y1) / 2);
            self.line(x0, y0, mx, my, c);
            self.line(mx, my, x1, y1, c);
            return;
        }
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Lays `c` over the pixel at opacity `a` (0 to 1), in 8-bit sRGB.
    pub fn blend(&mut self, x: i64, y: i64, c: Rgb, a: f64) {
        if let Some(i) = self.at(x, y) {
            let a = a.clamp(0.0, 1.0);
            for (k, v) in c.to_ints().into_iter().enumerate() {
                let old = self.px[i + k] as f64;
                self.px[i + k] = (old + (v as f64 - old) * a).round() as u8;
            }
        }
    }

    /// **An antialiased line** (Xiaolin Wu) between points whose integer
    /// coordinates are pixel centers: each pixel the line crosses is laid
    /// over at `alpha` times the share of a one-pixel-wide line covering it,
    /// so a line finer or fainter than a pixel draws as a faint one and moves
    /// by less than a pixel. Clipped to the buffer first.
    pub fn line_aa(&mut self, a: [f64; 2], b: [f64; 2], c: Rgb, alpha: f64) {
        let (w, h) = (self.w as f64, self.h as f64);
        let Some(([mut x0, mut y0], [mut x1, mut y1])) = clip_segment(a, b, [-1.0, -1.0, w, h]) else { return };
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            (x0, y0, x1, y1) = (y0, x0, y1, x1);
        }
        if x0 > x1 {
            (x0, y0, x1, y1) = (x1, y1, x0, y0);
        }
        let grad = if x1 > x0 { (y1 - y0) / (x1 - x0) } else { 0.0 };
        let mut plot = |x: i64, y: i64, cover: f64| {
            if steep {
                self.blend(y, x, c, alpha * cover)
            } else {
                self.blend(x, y, c, alpha * cover)
            }
        };
        let (xs, xe) = (x0.round() as i64, x1.round() as i64);
        for x in xs..=xe {
            // Each end column covers only its share of the pixel it is in.
            let reach = if xs == xe {
                x1 - x0
            } else if x == xs {
                xs as f64 + 0.5 - x0
            } else if x == xe {
                x1 - (xe as f64 - 0.5)
            } else {
                1.0
            };
            let y = y0 + grad * (x as f64 - x0);
            let (yi, f) = (y.floor() as i64, y - y.floor());
            plot(x, yi, (1.0 - f) * reach);
            plot(x, yi + 1, f * reach);
        }
    }

    /// The pixels whose centers lie within half a pixel of radius `r`.
    pub fn circle(&mut self, cx: i64, cy: i64, r: f64, c: Rgb) {
        let ri = r.ceil() as i64 + 1;
        for y in -ri..=ri {
            for x in -ri..=ri {
                let d = ((x * x + y * y) as f64).sqrt();
                if (d - r).abs() <= 0.5 {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }

    /// The pixels whose centers lie within radius `r`.
    pub fn disc(&mut self, cx: i64, cy: i64, r: f64, c: Rgb) {
        let ri = r.ceil() as i64;
        for y in -ri..=ri {
            for x in -ri..=ri {
                if ((x * x + y * y) as f64) <= r * r {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }

    /// Paints `c` wherever `mask` (`w` wide, row-major) is set, with the
    /// mask's `(ax, ay)` cell at `(x, y)`.
    #[allow(clippy::too_many_arguments)]
    pub fn stamp(&mut self, mask: &[bool], w: usize, ax: usize, ay: usize, x: i64, y: i64, c: Rgb) {
        for (i, &on) in mask.iter().enumerate() {
            if on {
                self.set(x + (i % w) as i64 - ax as i64, y + (i / w) as i64 - ay as i64, c);
            }
        }
    }

    /// A decimal number in a 3×5 pixel font, its top-left at `(x, y)`.
    /// Returns the x just past the last digit's right edge.
    pub fn number(&mut self, x: i64, y: i64, n: u32, c: Rgb) -> i64 {
        // Rows top to bottom, three bits each, high bit on the left.
        const DIGITS: [[u8; 5]; 10] = [
            [7, 5, 5, 5, 7],
            [2, 6, 2, 2, 7],
            [7, 1, 7, 4, 7],
            [7, 1, 3, 1, 7],
            [5, 5, 7, 1, 1],
            [7, 4, 7, 1, 7],
            [7, 4, 7, 5, 7],
            [7, 1, 1, 2, 2],
            [7, 5, 7, 5, 7],
            [7, 5, 7, 1, 7],
        ];
        for (k, d) in n.to_string().bytes().enumerate() {
            for (row, bits) in DIGITS[(d - b'0') as usize].iter().enumerate() {
                for col in 0..3 {
                    if bits & (4 >> col) != 0 {
                        self.set(x + 4 * k as i64 + col, y + row as i64, c);
                    }
                }
            }
        }
        x + 4 * n.to_string().len() as i64 - 1
    }

    /// Pixels of exactly color `c`.
    pub fn count(&self, c: Rgb) -> usize {
        let [r, g, b] = c.to_ints();
        self.px.as_chunks::<4>().0.iter().filter(|p| p[0] == r && p[1] == g && p[2] == b).count()
    }
}

/// The part of segment `a`–`b` inside the box `[x0, y0, x1, y1]`
/// (Liang–Barsky), or `None` when none of it is.
pub fn clip_segment(a: [f64; 2], b: [f64; 2], bx: [f64; 4]) -> Option<([f64; 2], [f64; 2])> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-d[0], a[0] - bx[0]), (d[0], bx[2] - a[0]), (-d[1], a[1] - bx[1]), (d[1], bx[3] - a[1])] {
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

#[cfg(test)]
mod tests {
    use super::*;

    const INK: Rgb = Rgb { r: 1.0, g: 0.0, b: 0.0 };
    const BLACK: Rgb = Rgb { r: 0.0, g: 0.0, b: 0.0 };

    fn black(w: usize, h: usize) -> Raster {
        let mut r = Raster::new(w, h);
        r.clear(BLACK);
        r
    }

    #[test]
    fn set_and_get_address_one_pixel_and_clip_outside() {
        let mut r = black(4, 3);
        r.set(3, 2, INK);
        assert_eq!(r.get(3, 2), Some([255, 0, 0]));
        assert_eq!(r.count(INK), 1);
        r.set(-1, 0, INK);
        r.set(4, 0, INK);
        r.set(0, 3, INK);
        assert_eq!(r.count(INK), 1, "off the buffer draws nothing");
        assert_eq!(r.get(9, 9), None);
        assert_eq!(r.px[(2 * 4 + 3) * 4 + 3], 255, "opaque");
    }

    #[test]
    fn a_line_includes_both_ends_and_is_connected() {
        let mut r = black(20, 20);
        r.line(2, 3, 15, 9, INK);
        assert_eq!((r.get(2, 3), r.get(15, 9)), (Some([255, 0, 0]), Some([255, 0, 0])));
        assert_eq!(r.count(INK), 14, "one pixel per step along the longer axis");
    }

    #[test]
    fn a_line_far_off_the_buffer_costs_nothing_and_a_long_one_still_lands() {
        let mut r = black(10, 10);
        r.line(-1_000_000_000, 5, 1_000_000_000, 5, INK);
        assert_eq!(r.count(INK), 10, "the visible run of a line a billion pixels long");
        r.line(5_000_000, 5_000_000, 6_000_000, 5_000_000, INK);
        assert_eq!(r.count(INK), 10);
    }

    /// **An antialiased line puts its weight where the line is** (T-174): a
    /// horizontal line half-way between two rows lights both at half; one on
    /// a row lights it fully; the light along a line sums to its length; and
    /// a fainter alpha lays less over the ground.
    #[test]
    fn an_antialiased_line_shares_its_light_between_the_pixels_it_crosses() {
        let mut r = black(20, 10);
        r.line_aa([2.0, 4.5], [12.0, 4.5], INK, 1.0);
        assert_eq!((r.get(6, 4).unwrap()[0], r.get(6, 5).unwrap()[0]), (128, 128), "half to each row");
        let mut r = black(20, 10);
        r.line_aa([2.0, 4.0], [12.0, 4.0], INK, 1.0);
        assert_eq!((r.get(6, 4).unwrap()[0], r.get(6, 5).unwrap()[0]), (255, 0), "on the row, all of it");
        let total: f64 = r.px.as_chunks::<4>().0.iter().map(|p| p[0] as f64 / 255.0).sum();
        assert!((total - 10.0).abs() < 0.6, "the light sums to the length: {total}");
        let mut r = black(20, 10);
        r.line_aa([2.0, 4.0], [12.0, 4.0], INK, 0.25);
        assert_eq!(r.get(6, 4).unwrap()[0], 64, "a quarter of the way to the ink");
        let mut r = black(20, 10);
        r.line_aa([-1e9, 4.0], [1e9, 4.0], INK, 1.0);
        assert_eq!(r.get(19, 4).unwrap()[0], 255, "a line far past the buffer is clipped to it");
    }

    #[test]
    fn a_circle_lies_on_its_radius_and_a_disc_fills_it() {
        let mut r = black(21, 21);
        r.circle(10, 10, 5.0, INK);
        for y in 0..21 {
            for x in 0..21 {
                if r.get(x, y) == Some([255, 0, 0]) {
                    let d = (((x - 10) * (x - 10) + (y - 10) * (y - 10)) as f64).sqrt();
                    assert!((d - 5.0).abs() <= 0.5);
                }
            }
        }
        assert!(r.get(10, 10) == Some([0, 0, 0]), "a circle is hollow");
        r.disc(10, 10, 2.0, INK);
        assert_eq!(r.get(10, 10), Some([255, 0, 0]));
    }

    #[test]
    fn a_number_is_drawn_digit_by_digit_in_the_small_font() {
        let mut r = black(12, 6);
        r.number(0, 0, 17, INK);
        assert_eq!(r.count(INK), 7 + 8, "a 1 is seven pixels, a 7 eight");
        assert_eq!((r.get(1, 0), r.get(4, 0)), (Some([255, 0, 0]), Some([255, 0, 0])), "each digit's top row");
        assert_eq!(r.get(3, 0), Some([0, 0, 0]), "a column between digits");
    }

    #[test]
    fn a_stamp_places_its_anchor_on_the_point() {
        let mut r = black(10, 10);
        let mask = [false, true, false, true, true, true];
        r.stamp(&mask, 3, 1, 1, 5, 5, INK);
        assert_eq!(r.count(INK), 4);
        assert_eq!(r.get(5, 5), Some([255, 0, 0]), "the anchor cell");
        assert_eq!(r.get(5, 4), Some([255, 0, 0]));
        assert_eq!(r.get(4, 4), Some([0, 0, 0]));
    }
}
