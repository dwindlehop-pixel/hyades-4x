//! **Transcendentals built from `+ − × ÷` and bit operations only** (T-127).
//!
//! The engine may not call the platform's `f64::ln`, `exp`, `powf`, `sin`,
//! `cos` or any other libm function. Those are the host libm natively and a Rust port of musl
//! compiled into the module on `wasm32-unknown-unknown`, and the two are
//! different algorithms. Measured on 2,000,000 inputs per function (appendix
//! §D.6): they disagree in the last bit on **1.9%** of `ln` calls, **9.8%** of
//! `exp`, **9.7%** of `powf` and **3.1%** of `sin`/`cos`. That is enough to
//! make a native run and a browser run of the same seed diverge — they did,
//! on every one of four arms tried at 800 yr (3 seats) and 300 yr (12 seats),
//! which is the native-against-wasm gap `Hyades_netcode.md` §6 H4a names.
//!
//! IEEE 754 specifies `+`, `−`, `×`, `÷` and `sqrt` exactly — correctly
//! rounded — so a function composed of nothing else returns the same bits on
//! every conforming target. Rust never contracts `a * b + c` into a fused
//! multiply-add on its own, and nothing here asks for one; **do not add
//! `mul_add`**, which rounds once where the separate operations round twice.
//!
//! The algorithms are FreeBSD msun's (`e_log.c`, `e_exp.c`, `k_sin.c`,
//! `k_cos.c`, `e_rem_pio2.c`), which are Sun's fdlibm, carried with the notice
//! their license asks for:
//!
//! > Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//! > Developed at SunPro, a Sun Microsystems, Inc. business.
//! > Permission to use, copy, modify, and distribute this software is freely
//! > granted, provided that this notice is preserved.
//!
//! Each constant is written as its bit pattern, which is how fdlibm documents
//! them, so there is no decimal-to-binary conversion to get wrong. The
//! accuracy of each function against the host libm is measured by the tests
//! below rather than asserted from the source; every bound is a bound over
//! the sampled inputs, not a proof.
//!
//! `clippy::disallowed_methods` (`clippy.toml`) rejects the libm methods
//! anywhere in the library, so a new call site fails CI instead of quietly
//! reopening the gap.

/// Build an `f64` from its bit pattern. Exact.
#[inline(always)]
const fn b(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// The high 32 bits of `x` (sign, exponent, top of the mantissa), as fdlibm's
/// `GET_HIGH_WORD` reads them.
#[inline(always)]
const fn high(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

/// `x` with its high word replaced (fdlibm's `SET_HIGH_WORD`).
#[inline(always)]
const fn with_high(x: f64, hi: i32) -> f64 {
    f64::from_bits(((hi as u32 as u64) << 32) | (x.to_bits() & 0xffff_ffff))
}

const LN2_HI: f64 = b(0x3fe6_2e42_fee0_0000); // 6.93147180369123816490e-01
const LN2_LO: f64 = b(0x3dea_39ef_3579_3c76); // 1.90821492927058770002e-10
const TWO54: f64 = b(0x4350_0000_0000_0000); // 2^54

const LG1: f64 = b(0x3fe5_5555_5555_5593);
const LG2: f64 = b(0x3fd9_9999_9997_fa04);
const LG3: f64 = b(0x3fd2_4924_9422_9359);
const LG4: f64 = b(0x3fcc_71c5_1d8e_78af);
const LG5: f64 = b(0x3fc7_4664_96cb_03de);
const LG6: f64 = b(0x3fc3_9a09_d078_c69f);
const LG7: f64 = b(0x3fc2_f112_df3e_5244);

/// **Natural logarithm by fdlibm `e_log.c`, for constants.** One division on
/// the critical path, which is why run-time callers use [`ln`]; this one is a
/// `const fn` so that a logarithm of a constant — `LN_TABLE`, the Band
/// ladder's `Scale::LN_STEPS` — is evaluated by the compiler, whose float
/// arithmetic is the same IEEE 754 as every target's. It is also the second
/// reference the tests hold [`ln`] against.
pub const fn ln_const(x: f64) -> f64 {
    let mut x = x;
    let mut hx = high(x);
    let lx = x.to_bits() as u32;
    let mut k: i32 = 0;
    if hx < 0x0010_0000 {
        // x < 2^-1022: zero, negative, or subnormal.
        if ((hx & 0x7fff_ffff) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = high(x);
    }
    if hx >= 0x7ff0_0000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000f_ffff;
    let i = (hx + 0x95f64) & 0x0010_0000;
    // Normalize x or x/2 into [sqrt(2)/2, sqrt(2)).
    x = with_high(x, hx | (i ^ 0x3ff0_0000));
    k += i >> 20;
    let f = x - 1.0;
    let dk = k as f64;
    if (0x000f_ffff & (2 + hx)) < 3 {
        // -2^-20 <= f < 2^-20
        if f == 0.0 {
            return if k == 0 { 0.0 } else { dk * LN2_HI + dk * LN2_LO };
        }
        let r = f * f * (0.5 - 0.333_333_333_333_333_3 * f);
        return if k == 0 { f - r } else { dk * LN2_HI - ((r - dk * LN2_LO) - f) };
    }
    let s = f / (2.0 + f);
    let z = s * s;
    let mut i = hx - 0x6147a;
    let w = z * z;
    let j = 0x6b851 - hx;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)
    }
}

/// Cells per unit of mantissa in [`LN_TABLE`].
const LN_CELLS: f64 = 128.0;
/// `1.5 · 2^52`: adding it to a value below `2^51` rounds that value to the
/// nearest integer and leaves the integer in the low mantissa bits — an IEEE
/// round-to-nearest, where `as usize` would be a saturating conversion with
/// its own range checks.
const ROUND_TO_INT: f64 = 6_755_399_441_055_744.0;

/// `(c, 1/c, ln c)` at the cell center `c = j / 128`, indexed by `j`. A
/// mantissa normalized into `[√2/2, √2)` rounds to `j ∈ [91, 181]`; the table
/// is 256 long so that the index, taken as the low eight bits, needs no bounds
/// check, and the unused entries are never read. Built at compile time by
/// [`ln_const`], so it is the same bits on every target by construction and
/// there are no constants to transcribe.
const LN_TABLE: [(f64, f64, f64); 256] = {
    let mut t = [(1.0, 1.0, 0.0); 256];
    let mut j = 91;
    while j <= 181 {
        let c = j as f64 / LN_CELLS;
        t[j] = (c, 1.0 / c, ln_const(c));
        j += 1;
    }
    t
};

/// Taylor coefficients of `ln(1 + r)` from `r³`.
const T3: f64 = 1.0 / 3.0;
const T5: f64 = 1.0 / 5.0;
const T6: f64 = -1.0 / 6.0;
const T7: f64 = 1.0 / 7.0;

/// **Natural logarithm, without a division.** Same special values as
/// `f64::ln`: `ln(0) = −∞`, `ln(negative) = NaN`, `ln(+∞) = +∞`.
///
/// `x = 2^k · m` with `m ∈ [√2/2, √2)`, then `m` is split at the nearest
/// 1/128th, `c`: `ln x = k ln 2 + ln c + ln(1 + r)` with `r = (m − c)/c`.
/// `m − c` is exact (Sterbenz: the two are within a factor of two), so the
/// only rounding before the polynomial is one multiply by the tabled `1/c`,
/// and `|r| ≤ 2^−7.5` needs the Taylor series only to `r⁸`. Where `c = 1` the
/// table entry is exactly zero and `r = m − 1` exactly, so a result near zero
/// is not a difference of two large numbers. Elsewhere near one the sum
/// can lose a bit, so the tested bound against the host is 1 ulp over sixty
/// binades and 2 ulp within 0.3 of one.
///
/// It replaced [`ln_const`] at run time because `settler_target` calls it
/// about five million times per 400 simulated years on the standard bed, and
/// the division was most of its cost (appendix §D.6).
pub fn ln(x: f64) -> f64 {
    let mut x = x;
    let mut hx = high(x);
    let mut k: i32 = 0;
    if hx < 0x0010_0000 {
        // x < 2^-1022: zero, negative, or subnormal.
        if x == 0.0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= TWO54;
        hx = high(x);
    }
    if hx >= 0x7ff0_0000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    let hm = hx & 0x000f_ffff;
    // fdlibm's normalization: halve m when it is at or above ~√2.
    let i = (hm + 0x95f64) & 0x0010_0000;
    let m = with_high(x, hm | (i ^ 0x3ff0_0000));
    k += i >> 20;
    let (c, inv_c, ln_c) = LN_TABLE[((m * LN_CELLS + ROUND_TO_INT).to_bits() & 0xff) as usize];
    let r = (m - c) * inv_c;
    let r2 = r * r;
    // ln(1 + r) = r + r²·(−1/2 + r/3 − r²/4 + r³/5 − r⁴/6 + r⁵/7 − r⁶/8), Estrin.
    let q = (-0.5 + r * T3) + r2 * ((-0.25 + r * T5) + r2 * ((T6 + r * T7) + r2 * -0.125));
    let dk = k as f64;
    (dk * LN2_HI + ln_c) + (dk * LN2_LO + (r + r2 * q))
}

const O_THRESHOLD: f64 = b(0x4086_2e42_fefa_39ef); // 709.78...
const U_THRESHOLD: f64 = b(0xc087_4910_d52d_3051); // -745.13...
const INV_LN2: f64 = b(0x3ff7_1547_652b_82fe);
const TWOM1000: f64 = b(0x0170_0000_0000_0000); // 2^-1000
const P1: f64 = b(0x3fc5_5555_5555_553e);
const P2: f64 = b(0xbf66_c16c_16be_bd93);
const P3: f64 = b(0x3f11_566a_af25_de2c);
const P4: f64 = b(0xbebb_bd41_c5d2_6bf1);
const P5: f64 = b(0x3e66_3769_72be_a4d0);

/// **`e^x`** — fdlibm `e_exp.c`. Same special values as `f64::exp`.
pub fn exp(x: f64) -> f64 {
    let hx0 = high(x);
    let negative = hx0 < 0;
    let hx = hx0 & 0x7fff_ffff;
    if hx >= 0x4086_2e42 {
        // |x| >= 709.78...
        if hx >= 0x7ff0_0000 {
            if x.is_nan() {
                return x + x;
            }
            return if negative { 0.0 } else { x };
        }
        if x > O_THRESHOLD {
            return f64::INFINITY;
        }
        if x < U_THRESHOLD {
            return 0.0;
        }
    }
    let mut x = x;
    let (hi, lo, k) = if hx > 0x3fd6_2e42 {
        // |x| > 0.5 ln2: reduce to x = k ln2 + r, |r| <= 0.5 ln2.
        let (hi, lo, k) = if hx < 0x3ff0_a2b2 {
            // and |x| < 1.5 ln2
            if negative {
                (x + LN2_HI, -LN2_LO, -1)
            } else {
                (x - LN2_HI, LN2_LO, 1)
            }
        } else {
            let k = (INV_LN2 * x + if negative { -0.5 } else { 0.5 }) as i32;
            let t = k as f64;
            (x - t * LN2_HI, t * LN2_LO, k)
        };
        x = hi - lo;
        (hi, lo, k)
    } else if hx < 0x3e30_0000 {
        // |x| < 2^-28
        return 1.0 + x;
    } else {
        (0.0, 0.0, 0)
    };
    let t = x * x;
    let twopk = if k >= -1021 {
        f64::from_bits(((0x3ff + k) as u64) << 52)
    } else {
        f64::from_bits(((0x3ff + k + 1000) as u64) << 52)
    };
    let c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    if k == 0 {
        return 1.0 - ((x * c) / (c - 2.0) - x);
    }
    let y = 1.0 - ((lo - (x * c) / (2.0 - c)) - hi);
    if k >= -1021 {
        if k == 1024 {
            return y * 2.0 * b(0x7fe0_0000_0000_0000);
        }
        y * twopk
    } else {
        y * twopk * TWOM1000
    }
}

/// **`x^y` for `x ≥ 0`**, as `exp(y · ln x)`.
///
/// Not fdlibm's `e_pow.c`, which carries `ln x` in extra precision; the
/// error here grows with `z = |y · ln x|` — the rounding of the product is
/// magnified `z`-fold by `exp` — and the test below holds it to `4 + 2z` units
/// in the last place over its sample. The engine's arguments keep `z` to a
/// handful: a ladder step to a fractional Band is `z ≤ ln 1000 ≈ 6.9`. `pow(x, 0) = 1`
/// and `pow(0, y > 0) = 0` exactly, as `powf` gives them. A negative base is a
/// caller error in this engine and returns `NaN`.
pub fn pow(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 1.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    if x == 0.0 {
        return if y > 0.0 { 0.0 } else { f64::INFINITY };
    }
    if x < 0.0 {
        return f64::NAN;
    }
    exp(y * ln(x))
}

const S1: f64 = b(0xbfc5_5555_5555_5549);
const S2: f64 = b(0x3f81_1111_1110_f8a6);
const S3: f64 = b(0xbf2a_01a0_19c1_61d5);
const S4: f64 = b(0x3ec7_1de3_57b1_fe7d);
const S5: f64 = b(0xbe5a_e5e6_8a2b_9ceb);
const S6: f64 = b(0x3de5_d93a_5acf_d57c);

/// fdlibm `k_sin.c`: `sin(x + y)` for `|x| <= π/4`, `y` the tail of `x`.
#[inline]
fn k_sin(x: f64, y: f64, has_tail: bool) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let v = z * x;
    if !has_tail {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}

const C1: f64 = b(0x3fa5_5555_5555_554c);
const C2: f64 = b(0xbf56_c16c_16c1_5177);
const C3: f64 = b(0x3efa_01a0_19cb_1590);
const C4: f64 = b(0xbe92_7e4f_809c_52ad);
const C5: f64 = b(0x3e21_ee9e_bdb4_b1c4);
const C6: f64 = b(0xbda8_fae9_be88_38d4);

/// fdlibm `k_cos.c` (the FreeBSD form): `cos(x + y)` for `|x| <= π/4`.
#[inline]
fn k_cos(x: f64, y: f64) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * r - x * y))
}

const INV_PIO2: f64 = b(0x3fe4_5f30_6dc9_c883);
const PIO2_1: f64 = b(0x3ff9_21fb_5440_0000);
const PIO2_1T: f64 = b(0x3dd0_b461_1a62_6331);
const PIO2_2: f64 = b(0x3dd0_b461_1a60_0000);
const PIO2_2T: f64 = b(0x3ba3_198a_2e03_7073);
const PIO2_3: f64 = b(0x3ba3_198a_2e00_0000);
const PIO2_3T: f64 = b(0x397b_839a_2520_49c1);

/// **Largest `|x|` the trigonometric reduction is accurate for**, `2^20 · π/2`.
///
/// fdlibm switches to Payne–Hanek past this; the engine's angles are fractions
/// of a turn plus at most a few dozen radians of station-keeping phase, so the
/// switch is not carried. Past the bound the result is still bit-identical on
/// every target — only its accuracy degrades.
pub const TRIG_ACCURATE_MAX: f64 = 1_647_099.0;

/// fdlibm `e_rem_pio2.c`'s medium path: `x = n·π/2 + (y0 + y1)`, three-stage
/// Cody–Waite reduction with `π/2` split into 33-bit pieces.
fn rem_pio2(x: f64) -> (i32, f64, f64) {
    let ix = high(x) & 0x7fff_ffff;
    let fnn = (x * INV_PIO2).round();
    let n = fnn as i32;
    let mut r = x - fnn * PIO2_1;
    let mut w = fnn * PIO2_1T;
    let j = ix >> 20;
    let mut y0 = r - w;
    let i = j - ((high(y0) >> 20) & 0x7ff);
    if i > 16 {
        let t = r;
        w = fnn * PIO2_2;
        r = t - w;
        w = fnn * PIO2_2T - ((t - r) - w);
        y0 = r - w;
        let i = j - ((high(y0) >> 20) & 0x7ff);
        if i > 49 {
            let t = r;
            w = fnn * PIO2_3;
            r = t - w;
            w = fnn * PIO2_3T - ((t - r) - w);
            y0 = r - w;
        }
    }
    let y1 = (r - y0) - w;
    (n, y0, y1)
}

/// **`(sin x, cos x)`** — fdlibm's kernels after `rem_pio2`. One reduction
/// serves both, which is what every caller in the engine wants.
pub fn sin_cos(x: f64) -> (f64, f64) {
    let ix = high(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        // |x| <~ π/4
        if ix < 0x3e46_a09e {
            // |x| < 2^-27 · sqrt(2): sin x = x, cos x = 1 to the last bit.
            return (x, 1.0);
        }
        return (k_sin(x, 0.0, false), k_cos(x, 0.0));
    }
    if ix >= 0x7ff0_0000 {
        return (f64::NAN, f64::NAN);
    }
    let (n, y0, y1) = rem_pio2(x);
    let (s, c) = (k_sin(y0, y1, true), k_cos(y0, y1));
    match n & 3 {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    }
}

/// `cos x`. See [`sin_cos`].
#[inline]
pub fn cos(x: f64) -> f64 {
    sin_cos(x).1
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // the host libm is the reference being compared against
mod tests {
    use super::*;
    use crate::rng::Rng;

    /// Distance in units in the last place between two finite `f64` of the
    /// same sign.
    fn ulps(a: f64, b: f64) -> u64 {
        if a == b {
            return 0;
        }
        let (a, b) = (a.to_bits() as i64, b.to_bits() as i64);
        (a - b).unsigned_abs()
    }

    /// Max ULP distance from the host libm over `n` samples, and how many
    /// samples differed at all.
    fn against(
        n: usize,
        mut draw: impl FnMut(&mut Rng) -> f64,
        ours: fn(f64) -> f64,
        host: fn(f64) -> f64,
    ) -> (u64, usize) {
        let mut rng = Rng::new(0x7127);
        let (mut worst, mut differ) = (0, 0);
        for _ in 0..n {
            let x = draw(&mut rng);
            let d = ulps(ours(x), host(x));
            worst = worst.max(d);
            differ += (d > 0) as usize;
        }
        (worst, differ)
    }

    /// Mantissas uniform, exponents uniform over sixty binades.
    fn wide(r: &mut Rng) -> f64 {
        f64::from_bits(((1023 - 30 + r.below(61) as u64) << 52) | ((r.unit() * (1u64 << 52) as f64) as u64))
    }

    #[test]
    fn ln_const_is_within_one_ulp_of_the_host_over_sixty_binades() {
        let (worst, _) = against(200_000, wide, ln_const, f64::ln);
        assert!(worst <= 1, "ln_const differs from the host by {worst} ulp");
    }

    #[test]
    fn ln_is_within_one_ulp_of_the_host_over_sixty_binades() {
        let (worst, differ) = against(1_000_000, wide, ln, f64::ln);
        assert!(worst <= 1, "ln differs from the host by {worst} ulp ({differ} samples differ)");
    }

    /// Where the result is small, `ln c` and `ln(1 + r)` can have opposite
    /// signs and the sum loses up to one bit — a factor of two, because `c` is
    /// the *nearest* cell, so `|ln(1 + r)| ≤ |ln c| / 2`. Measured: 2 ulp, on
    /// about a quarter of samples within 0.3 of one. The bound is the claim.
    #[test]
    fn ln_is_within_two_ulp_of_the_host_near_one() {
        let (worst, differ) = against(1_000_000, |r| 1.0 + r.range(-0.3, 0.3) * r.unit() * r.unit(), ln, f64::ln);
        assert!(worst <= 2, "ln near 1 differs from the host by {worst} ulp ({differ} samples differ)");
    }

    #[test]
    fn exp_is_within_one_ulp_of_the_host_over_the_engines_range() {
        let (worst, _) = against(200_000, |r| r.range(-40.0, 20.0), exp, f64::exp);
        assert!(worst <= 1, "exp differs from the host by {worst} ulp");
    }

    #[test]
    fn sin_and_cos_are_within_one_ulp_of_the_host_over_eight_turns_and_a_fight() {
        let draw = |r: &mut Rng| r.range(-60.0, 60.0);
        let (ws, _) = against(200_000, draw, |x| sin_cos(x).0, f64::sin);
        let (wc, _) = against(200_000, draw, cos, f64::cos);
        assert!(ws <= 1 && wc <= 1, "sin {ws} ulp, cos {wc} ulp");
    }

    #[test]
    fn pow_error_stays_within_a_few_ulp_where_the_engine_uses_it() {
        // Ladder steps to a fractional Band, vein counts to the crowding
        // exponent, a Weibull quantile to 1/k: bases up to ~10^3, |y| <= 4.
        let mut rng = Rng::new(0x9001);
        let mut worst = 0;
        for _ in 0..200_000 {
            let (x, y) = (rng.range(1e-3, 1e3), rng.range(-4.0, 4.0));
            let (ours, host) = (pow(x, y), x.powf(y));
            // Ulps grow with |y ln x|, as documented; the bound is 4 + 2|y ln x|.
            let allow = 4 + (2.0 * (y * x.ln()).abs()).ceil() as u64;
            let d = ulps(ours, host);
            assert!(d <= allow, "pow({x}, {y}): {d} ulp against an allowance of {allow}");
            worst = worst.max(d);
        }
        assert!(worst > 0, "no sample differed — the comparison is not reaching the function");
    }

    #[test]
    fn special_values_match_the_host() {
        for x in [0.0, -0.0, -1.0, 1.0, f64::INFINITY, f64::NEG_INFINITY, f64::MIN_POSITIVE, 5e-324, f64::MAX] {
            let (o, h) = (ln(x), x.ln());
            assert!(o.to_bits() == h.to_bits() || (o.is_nan() && h.is_nan()), "ln({x}) = {o}, host {h}");
        }
        for x in [0.0, -0.0, 1.0, -1.0, 709.0, 709.8, -745.0, -746.0, f64::INFINITY, f64::NEG_INFINITY, 1e-300] {
            let (o, h) = (exp(x), x.exp());
            assert!(ulps(o, h) <= 1 || (o.is_nan() && h.is_nan()), "exp({x}) = {o}, host {h}");
        }
        assert!(ln(f64::NAN).is_nan() && exp(f64::NAN).is_nan() && sin_cos(f64::NAN).0.is_nan());
        assert_eq!(pow(7.3, 0.0), 1.0);
        assert_eq!(pow(0.0, 2.5), 0.0);
        assert_eq!(pow(1.0, 123.0), 1.0);
        assert_eq!(sin_cos(0.0), (0.0, 1.0));
    }
}
