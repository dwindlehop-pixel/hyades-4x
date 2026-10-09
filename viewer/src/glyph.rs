//! **Hull glyphs** (`docs/Hyades_interface.md` §6.3, proposed — R-UI3).
//!
//! A glyph reads a hull's Design off its pixels:
//!
//! | what | from | drawn as |
//! |---|---|---|
//! | hull family | `LSV`… → Systems, Contact, Offensive | square, diamond, triangle |
//! | hull size | Limited, Medium, General | 9, 11, 13 px across |
//! | armament | beam mounts, missile tubes | a spike ahead, ears either side — and the renderer fills an armed body solid |
//! | Design class | the class's index | a 4-bit tag in a row under the shape |
//! | Doctrine role | the role a hull is on | a 3×3 mark at [`Glyph::pip`] ([`role_mark`]) |
//!
//! Three layers come back as masks so the renderer can color them apart: the
//! **outline** and the **inner** body in the seat's color, and the **marks**
//! (armament and tag) in bright text. The role mark is stamped at the pip,
//! which sits where every shape has a 3×3 interior.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Systems,
    Contact,
    Offensive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Limited,
    Medium,
    General,
}

/// The family of a hull code, as `hyades_engine::sim::HullType::code` writes it.
pub fn family(code: &str) -> Family {
    match code.get(1..2) {
        Some("C") => Family::Contact,
        Some("O") => Family::Offensive,
        _ => Family::Systems,
    }
}

pub fn size(code: &str) -> Size {
    match code.get(0..1) {
        Some("M") | Some("R") => Size::Medium,
        Some("G") => Size::General,
        _ => Size::Limited,
    }
}

/// The Design classes, in the replay's `enums.design` order. The tag is the
/// index plus one, so every named class has at least one tag pixel and
/// `Unnamed` (the last) is drawn with none.
pub const DESIGNS: [&str; 12] =
    ["Meadow", "Spur", "Tor", "Cairn", "Delta", "Range", "Scarp", "Ford", "Strait", "Butte", "Mesa", "Unnamed"];

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub w: usize,
    pub h: usize,
    /// The cell drawn at the hull's position.
    pub ax: usize,
    pub ay: usize,
    pub outline: Vec<bool>,
    pub inner: Vec<bool>,
    pub marks: Vec<bool>,
    /// The center of the role mark: the anchor, or a pixel below it on a
    /// triangle, whose 3×3 interior sits low.
    pub pip: (usize, usize),
}

/// **A role's mark** (proposed, R-UI3): a 3×3 mask, row-major, distinct by
/// shape so it reads without color. `None` for a role with no mark.
pub fn role_mark(role: &str) -> Option<[bool; 9]> {
    const X: bool = true;
    const O: bool = false;
    Some(match role {
        "Colonizer" => [O, X, O, X, X, X, O, X, O],
        "Miner" => [O, X, O, O, X, O, O, X, O],
        "Freighter" => [O, O, O, X, X, X, O, O, O],
        "Picket" => [X, O, X, O, X, O, X, O, X],
        "Sentry" => [X, X, X, X, O, X, X, X, X],
        "Scout" => [O, O, O, O, X, O, O, O, O],
        "Reserve" => [X, O, X, O, O, O, X, O, X],
        _ => return None,
    })
}

impl Glyph {
    fn blank(w: usize, h: usize, ax: usize, ay: usize) -> Glyph {
        let blank = vec![false; w * h];
        Glyph { w, h, ax, ay, outline: blank.clone(), inner: blank.clone(), marks: blank, pip: (ax, ay) }
    }

    fn put(layer: &mut [bool], w: usize, x: i64, y: i64) {
        if x >= 0 && y >= 0 && (x as usize) < w {
            if let Some(c) = layer.get_mut(y as usize * w + x as usize) {
                *c = true;
            }
        }
    }
}

/// The glyph for a hull of `code` built to `design`, with its mounts.
pub fn glyph(code: &str, design: &str, beams: u32, tubes: u32) -> Glyph {
    let s: i64 = match size(code) {
        Size::Limited => 9,
        Size::Medium => 11,
        Size::General => 13,
    };
    let r = s / 2;
    // Two columns either side for ears, two rows above for the spike, two
    // rows below for the gap and the tag.
    let (w, h) = ((s + 4) as usize, (s + 4) as usize);
    let (ax, ay) = (r + 2, r + 2);
    let mut g = Glyph::blank(w, h, ax as usize, ay as usize);
    let inside = |x: i64, y: i64| -> bool {
        match family(code) {
            Family::Systems => x.abs() <= r && y.abs() <= r,
            Family::Contact => x.abs() + y.abs() <= r,
            // Apex up: row y (from -r to r) is half as wide as it is far
            // from the apex.
            Family::Offensive => y.abs() <= r && 2 * x.abs() <= y + r,
        }
    };
    for y in -r..=r {
        for x in -r..=r {
            if !inside(x, y) {
                continue;
            }
            let edge = !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let layer = if edge { &mut g.outline } else { &mut g.inner };
            Glyph::put(layer, w, ax + x, ay + y);
        }
    }
    if family(code) == Family::Offensive {
        g.pip = (ax as usize, ay as usize + 1);
    }
    if beams > 0 {
        let top = (-r..=r).find(|&y| inside(0, y)).unwrap_or(-r);
        for k in 1..=2 {
            Glyph::put(&mut g.marks, w, ax, ay + top - k);
        }
    }
    if tubes > 0 {
        let row = r / 2;
        let edge = (0..=r).rev().find(|&x| inside(x, row)).unwrap_or(r);
        for side in [-1, 1] {
            Glyph::put(&mut g.marks, w, ax + side * (edge + 2), ay + row);
        }
    }
    let tag = DESIGNS.iter().position(|&d| d == design).map_or(0, |i| (i + 1) % DESIGNS.len());
    for bit in 0..4 {
        if tag & (1 << bit) != 0 {
            Glyph::put(&mut g.marks, w, ax - 3 + 2 * bit as i64, ay + r + 2);
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    const HULLS: [&str; 10] = ["LSV", "MSV", "GSV", "LCV", "LCU", "GCV", "GCU", "LOU", "ROU", "GOU"];

    fn count(m: &[bool]) -> usize {
        m.iter().filter(|&&b| b).count()
    }

    #[test]
    fn a_hull_code_names_its_family_and_size() {
        assert_eq!((family("LSV"), size("LSV")), (Family::Systems, Size::Limited));
        assert_eq!((family("GCU"), size("GCU")), (Family::Contact, Size::General));
        assert_eq!((family("ROU"), size("ROU")), (Family::Offensive, Size::Medium));
        assert_eq!((family("LOU"), size("GOU")), (Family::Offensive, Size::General));
    }

    #[test]
    fn every_glyph_has_an_outline_around_a_filled_inside_with_room_for_a_role_mark() {
        for h in HULLS {
            let g = glyph(h, "Unnamed", 0, 0);
            assert!(count(&g.outline) > 0 && count(&g.inner) > 0, "{h}");
            for i in 0..g.w * g.h {
                assert!(!(g.outline[i] && g.inner[i]), "{h}: layers are disjoint");
            }
            let (px, py) = g.pip;
            for dy in 0..3 {
                for dx in 0..3 {
                    assert!(g.inner[(py + dy - 1) * g.w + px + dx - 1], "{h}: the 3×3 at the pip is inside the body");
                }
            }
        }
    }

    #[test]
    fn every_role_mark_differs_from_every_other() {
        let roles = ["Colonizer", "Miner", "Freighter", "Picket", "Sentry", "Scout", "Reserve"];
        let marks: Vec<[bool; 9]> = roles.iter().map(|r| role_mark(r).unwrap()).collect();
        for i in 0..marks.len() {
            for j in i + 1..marks.len() {
                assert_ne!(marks[i], marks[j], "{} and {}", roles[i], roles[j]);
            }
        }
        assert_eq!(role_mark("Scrapped"), None);
    }

    #[test]
    fn bigger_hulls_draw_bigger() {
        for fam in [["LSV", "MSV", "GSV"], ["LCV", "LCU", "GCV"], ["LOU", "ROU", "GOU"]] {
            let n: Vec<usize> = fam.iter().map(|h| count(&glyph(h, "Unnamed", 0, 0).outline)).collect();
            assert!(n[0] < n[2], "{fam:?}: {n:?}");
            assert!(n[0] <= n[1] && n[1] <= n[2], "{fam:?}: {n:?}");
        }
    }

    #[test]
    fn the_three_families_differ_at_every_size() {
        for (a, b, c) in [("LSV", "LCV", "LOU"), ("MSV", "MSV", "ROU"), ("GSV", "GCV", "GOU")] {
            let (x, y, z) = (glyph(a, "Unnamed", 0, 0), glyph(b, "Unnamed", 0, 0), glyph(c, "Unnamed", 0, 0));
            assert_ne!(x.outline, z.outline, "{a} {c}");
            if a != b {
                assert_ne!(x.outline, y.outline, "{a} {b}");
                assert_ne!(y.outline, z.outline, "{b} {c}");
            }
        }
    }

    #[test]
    fn every_design_reads_differently_on_every_hull() {
        for h in HULLS {
            let all: Vec<Glyph> = DESIGNS.iter().map(|d| glyph(h, d, 0, 0)).collect();
            for i in 0..all.len() {
                for j in i + 1..all.len() {
                    assert_ne!(all[i], all[j], "{h}: {} and {}", DESIGNS[i], DESIGNS[j]);
                }
            }
            assert_eq!(count(&all[11].marks), 0, "{h}: Unnamed carries no tag");
        }
    }

    #[test]
    fn armament_adds_marks_and_unarmed_adds_none() {
        let bare = glyph("LCV", "Unnamed", 0, 0);
        let beam = glyph("LCV", "Unnamed", 2, 0);
        let tube = glyph("LOU", "Unnamed", 0, 1);
        assert_eq!(count(&bare.marks), 0);
        assert_eq!(count(&beam.marks), 2, "a two-pixel spike");
        assert_eq!(count(&tube.marks), 2, "an ear either side");
        assert!(beam.marks[(beam.ay - 5) * beam.w + beam.ax], "the spike sits ahead of the apex");
        assert_eq!(beam.outline, bare.outline, "marks leave the shape alone");
    }
}
