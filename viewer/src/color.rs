//! Color arithmetic: sRGB, OKLab and OKLCH (Björn Ottosson, *A perceptual
//! color space for image processing*, 2020), WCAG 2 contrast, and gamut
//! mapping by chroma reduction.

/// A gamma-encoded sRGB color, channels in `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

/// A color in OKLab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// A color in OKLCH: lightness, chroma, hue in degrees on `[0, 360)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl Rgb {
    pub fn from_ints(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r: r as f64 / 255.0, g: g as f64 / 255.0, b: b as f64 / 255.0 }
    }

    /// `#rrggbb`.
    pub fn from_hex(hex: &str) -> Rgb {
        let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
        Rgb::from_ints((n >> 16) as u8, (n >> 8) as u8, n as u8)
    }

    /// The nearest 8-bit color.
    pub fn to_ints(self) -> [u8; 3] {
        let q = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        [q(self.r), q(self.g), q(self.b)]
    }

    pub fn to_hex(self) -> String {
        let [r, g, b] = self.to_ints();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    pub fn in_gamut(self) -> bool {
        const EPS: f64 = 1e-6;
        [self.r, self.g, self.b].iter().all(|&c| (-EPS..=1.0 + EPS).contains(&c))
    }
}

fn to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn to_gamma(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

pub fn to_oklab(c: Rgb) -> Lab {
    let (r, g, b) = (to_linear(c.r), to_linear(c.g), to_linear(c.b));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    Lab {
        l: 0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        a: 1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        b: 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    }
}

/// OKLab to sRGB without clipping: a channel outside `[0, 1]` is out of gamut.
fn oklab_raw(c: Lab) -> Rgb {
    let l = (c.l + 0.396_337_777_4 * c.a + 0.215_803_757_3 * c.b).powi(3);
    let m = (c.l - 0.105_561_345_8 * c.a - 0.063_854_172_8 * c.b).powi(3);
    let s = (c.l - 0.089_484_177_5 * c.a - 1.291_485_548 * c.b).powi(3);
    let lin = [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
    ];
    let g = |x: f64| if x < 0.0 { -to_gamma(-x) } else { to_gamma(x) };
    Rgb { r: g(lin[0]), g: g(lin[1]), b: g(lin[2]) }
}

pub fn from_oklab(c: Lab) -> Rgb {
    let raw = oklab_raw(c);
    Rgb { r: raw.r.clamp(0.0, 1.0), g: raw.g.clamp(0.0, 1.0), b: raw.b.clamp(0.0, 1.0) }
}

pub fn to_oklch(c: Rgb) -> Lch {
    let Lab { l, a, b } = to_oklab(c);
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    Lch { l, c: a.hypot(b), h }
}

fn lab_of(l: f64, c: f64, h: f64) -> Lab {
    let r = h.to_radians();
    Lab { l, a: c * r.cos(), b: c * r.sin() }
}

/// OKLCH to sRGB. Out of gamut, chroma is reduced by bisection until the
/// color fits, keeping lightness and hue.
pub fn from_oklch(c: Lch) -> Rgb {
    if oklab_raw(lab_of(c.l, c.c, c.h)).in_gamut() {
        return from_oklab(lab_of(c.l, c.c, c.h));
    }
    let (mut lo, mut hi) = (0.0, c.c);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if oklab_raw(lab_of(c.l, mid, c.h)).in_gamut() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    from_oklab(lab_of(c.l, lo, c.h))
}

/// OKLCH to sRGB with no gamut mapping (for testing what mapping fixes).
pub fn from_oklch_unclipped(c: Lch) -> Rgb {
    oklab_raw(lab_of(c.l, c.c, c.h))
}

fn luminance(c: Rgb) -> f64 {
    0.2126 * to_linear(c.r) + 0.7152 * to_linear(c.g) + 0.0722 * to_linear(c.b)
}

/// WCAG 2 contrast ratio, 1 to 21.
pub fn contrast(x: Rgb, y: Rgb) -> f64 {
    let (p, q) = (luminance(x), luminance(y));
    (p.max(q) + 0.05) / (p.min(q) + 0.05)
}

/// Euclidean distance in OKLab.
pub fn delta_e(x: Rgb, y: Rgb) -> f64 {
    let (p, q) = (to_oklab(x), to_oklab(y));
    ((p.l - q.l).powi(2) + (p.a - q.a).powi(2) + (p.b - q.b).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        for h in ["#002b36", "#fdf6e3", "#dc322f", "#000000", "#ffffff"] {
            assert_eq!(Rgb::from_hex(h).to_hex(), h);
        }
    }

    #[test]
    fn oklab_round_trips_srgb_to_the_8_bit_channel() {
        for h in ["#002b36", "#859900", "#d33682", "#6c71c4", "#fdf6e3"] {
            assert_eq!(from_oklab(to_oklab(Rgb::from_hex(h))).to_hex(), h);
        }
    }

    #[test]
    fn oklab_reads_white_as_l_1_and_black_as_l_0() {
        let w = to_oklab(Rgb::from_hex("#ffffff"));
        assert!((w.l - 1.0).abs() < 1e-4 && w.a.abs() < 1e-4 && w.b.abs() < 1e-4);
        assert!(to_oklab(Rgb::from_hex("#000000")).l.abs() < 1e-9);
    }

    #[test]
    fn oklch_round_trips_with_hue_in_degrees() {
        let c = to_oklch(Rgb::from_hex("#cb4b16"));
        assert!((0.0..360.0).contains(&c.h));
        assert_eq!(from_oklch(c).to_hex(), "#cb4b16");
    }

    #[test]
    fn contrast_is_the_wcag_ratio() {
        assert!((contrast(Rgb::from_hex("#000000"), Rgb::from_hex("#ffffff")) - 21.0).abs() < 1e-9);
        assert_eq!(contrast(Rgb::from_hex("#268bd2"), Rgb::from_hex("#268bd2")), 1.0);
    }

    #[test]
    fn delta_e_is_the_oklab_distance() {
        assert_eq!(delta_e(Rgb::from_hex("#123456"), Rgb::from_hex("#123456")), 0.0);
        assert!(delta_e(Rgb::from_hex("#000000"), Rgb::from_hex("#ffffff")) > 0.99);
    }

    #[test]
    fn out_of_gamut_is_brought_in_by_chroma_keeping_lightness_and_hue() {
        let vivid = Lch { l: 0.7, c: 0.4, h: 140.0 };
        assert!(!from_oklch_unclipped(vivid).in_gamut());
        let back = to_oklch(from_oklch(vivid));
        assert!((back.l - 0.7).abs() < 0.01, "lightness kept: {}", back.l);
        assert!((back.h - 140.0).abs() < 2.0, "hue kept: {}", back.h);
    }
}
