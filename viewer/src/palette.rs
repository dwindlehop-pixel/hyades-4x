//! **The tactical mode's palette** (`docs/Hyades_interface.md` §6.2).
//!
//! [`SOURCE`] is the author's Hyades palette, verbatim (`hyades_palette.js`):
//! Solarized's eight base tones and eight accents, each accent with two tints
//! (`3`, `2`) and one shade (`1`). The tactical palette is [`SOURCE`] through
//! a tone map ([`ToneMap`]) that rotates it toward a 21st-century design
//! language in dialogue with 1970s Earth Day art.
//!
//! [`STATUS`] colors are deliberately **off** the palette — damage, a hit,
//! the drive, braking, a wreck, a laden hold, an encounter, selection — so
//! they read as simulation information: nothing in the palette is near them.
//!
//! **[`PALETTE_STATUS`] is `proposed`: the tone map's parameters, the role and
//! seat assignments and every status color await the author's approval**
//! (R-UI1).

use crate::color::{from_oklch, to_oklch, Lch, Rgb};

pub const PALETTE_STATUS: &str = "proposed";

/// The author's palette, by name, as 8-bit sRGB.
pub const SOURCE: [(&str, [u8; 3]); 40] = [
    ("hy_base03", [0, 43, 54]),
    ("hy_base02", [7, 54, 66]),
    ("hy_base01", [88, 110, 117]),
    ("hy_base00", [101, 123, 131]),
    ("hy_base0", [131, 148, 150]),
    ("hy_base1", [147, 161, 161]),
    ("hy_base2", [238, 232, 213]),
    ("hy_base3", [253, 246, 227]),
    ("hy_magenta3", [234, 131, 182]),
    ("hy_magenta2", [222, 87, 153]),
    ("hy_magenta", [211, 54, 131]),
    ("hy_magenta1", [201, 13, 105]),
    ("hy_violet3", [197, 199, 241]),
    ("hy_violet2", [149, 153, 220]),
    ("hy_violet", [108, 113, 196]),
    ("hy_violet1", [76, 80, 171]),
    ("hy_blue3", [115, 181, 228]),
    ("hy_blue2", [74, 158, 218]),
    ("hy_blue", [38, 139, 210]),
    ("hy_blue1", [9, 122, 202]),
    ("hy_cyan3", [120, 211, 204]),
    ("hy_cyan2", [74, 184, 176]),
    ("hy_cyan", [42, 161, 152]),
    ("hy_cyan1", [13, 151, 141]),
    ("hy_green3", [194, 215, 53]),
    ("hy_green2", [163, 184, 18]),
    ("hy_green", [133, 153, 0]),
    ("hy_green1", [103, 118, 0]),
    ("hy_yellow3", [240, 197, 63]),
    ("hy_yellow2", [222, 171, 14]),
    ("hy_yellow", [181, 137, 0]),
    ("hy_yellow1", [146, 110, 0]),
    ("hy_orange3", [254, 149, 105]),
    ("hy_orange2", [230, 111, 60]),
    ("hy_orange", [203, 76, 22]),
    ("hy_orange1", [166, 53, 6]),
    ("hy_red3", [255, 135, 132]),
    ("hy_red2", [240, 90, 86]),
    ("hy_red", [220, 50, 47]),
    ("hy_red1", [181, 25, 21]),
];

/// **The tone map ("Earthrise", proposed).** In OKLCH:
///
/// | parameter | effect |
/// |---|---|
/// | `ink`, `paper` | lightness is compressed into `[ink, paper]`: the darkest tone is an ink, not black; the brightest a paper, not white |
/// | `warm_chroma`, `cool_chroma` | chroma is scaled by `warm_chroma` at 70° (ochre) falling to `cool_chroma` at 250° (blue): the muted, warm-leaning saturation of 1970s print |
/// | `pull`, `anchors` | hue is pulled a share `pull` of the way to the nearest earth anchor, degrees: terracotta, harvest ochre, avocado, a faded sky |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneMap {
    pub ink: f64,
    pub paper: f64,
    pub warm_chroma: f64,
    pub cool_chroma: f64,
    pub pull: f64,
    pub anchors: [f64; 4],
}

pub const EARTHRISE: ToneMap = ToneMap {
    ink: 0.02,
    paper: 0.95,
    warm_chroma: 0.92,
    cool_chroma: 0.6,
    pull: 0.3,
    anchors: [38.0, 78.0, 118.0, 228.0],
};

fn hue_diff(to: f64, from: f64) -> f64 {
    (to - from + 540.0).rem_euclid(360.0) - 180.0
}

/// One color through the tone map.
pub fn tone_map(c: Rgb, p: &ToneMap) -> Rgb {
    let Lch { l, c, h } = to_oklch(c);
    let warm = 0.5 + 0.5 * (h - 70.0).to_radians().cos();
    let c2 = c * (p.cool_chroma + (p.warm_chroma - p.cool_chroma) * warm);
    let nearest = p.anchors.iter().copied().fold(p.anchors[0], |best, a| {
        if hue_diff(a, h).abs() < hue_diff(best, h).abs() {
            a
        } else {
            best
        }
    });
    let h2 = (h + p.pull * hue_diff(nearest, h)).rem_euclid(360.0);
    from_oklch(Lch { l: p.ink + (p.paper - p.ink) * l, c: c2, h: h2 })
}

/// The interface's roles, each a tone-mapped palette color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Roles {
    pub ground: Rgb,
    pub panel: Rgb,
    pub grid: Rgb,
    pub hex: Rgb,
    pub text_dim: Rgb,
    pub text: Rgb,
    pub text_bright: Rgb,
    pub world: Rgb,
    pub world_dim: Rgb,
    /// The three basics (galaxy §4.2).
    pub cyan: Rgb,
    pub magenta: Rgb,
    pub yellow: Rgb,
}

/// The whole tactical palette: every [`SOURCE`] color tone-mapped, and the
/// interface's roles assigned to them.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub map: ToneMap,
    pub named: Vec<(&'static str, Rgb)>,
    pub roles: Roles,
}

impl Palette {
    pub fn new(map: ToneMap) -> Palette {
        let named: Vec<(&'static str, Rgb)> =
            SOURCE.iter().map(|&(n, [r, g, b])| (n, tone_map(Rgb::from_ints(r, g, b), &map))).collect();
        let get = |n: &str| named.iter().find(|(k, _)| *k == n).map(|(_, c)| *c).unwrap();
        let roles = Roles {
            ground: get("hy_base03"),
            panel: get("hy_base02"),
            grid: get("hy_base01"),
            hex: get("hy_base02"),
            text_dim: get("hy_base0"),
            text: get("hy_base2"),
            text_bright: get("hy_base3"),
            world: get("hy_base0"),
            world_dim: get("hy_base01"),
            cyan: get("hy_cyan"),
            magenta: get("hy_magenta"),
            yellow: get("hy_yellow"),
        };
        Palette { map, named, roles }
    }

    pub fn get(&self, name: &str) -> Rgb {
        self.named.iter().find(|(k, _)| *k == name).map(|(_, c)| *c).unwrap_or(self.roles.world)
    }

    /// **A seat's color.** Each archetype's seats draw from its own families
    /// (galaxy §3) — Blue from blue and violet, Red from red, magenta and
    /// orange, Green from green, cyan and yellow. Seat `i` has archetype
    /// `i % 3`, so the first three seats are the three supers' own colors.
    pub fn seat(&self, seat: usize) -> Rgb {
        const FAMILIES: [[&str; 6]; 3] = [
            ["hy_blue", "hy_violet2", "hy_blue3", "hy_violet3", "hy_cyan2", "hy_blue2"],
            ["hy_red", "hy_magenta2", "hy_orange3", "hy_red3", "hy_magenta3", "hy_orange"],
            ["hy_green", "hy_yellow3", "hy_cyan", "hy_green3", "hy_yellow", "hy_cyan3"],
        ];
        self.get(FAMILIES[seat % 3][(seat / 3) % 6])
    }

    /// **The role a hull's Doctrine has it on**, as an accent.
    pub fn role(&self, kind: &str) -> Rgb {
        self.get(match kind {
            "Scout" => "hy_base3",
            "Colonizer" => "hy_green3",
            "Miner" => "hy_yellow3",
            "Freighter" => "hy_orange3",
            "Picket" => "hy_violet3",
            "Sentry" => "hy_magenta3",
            "Scrapped" => "hy_base01",
            _ => "hy_base0",
        })
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new(EARTHRISE)
    }
}

/// Simulation status, off the palette (proposed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Took damage since the last frame.
    Hit,
    /// Carries damage: a ring that closes as damage nears structure.
    Damage,
    /// Drive lit, gaining speed.
    Drive,
    /// Drive at a high acceleration.
    DriveHot,
    /// Drive lit, shedding speed.
    Braking,
    /// A wreck, coasting.
    Wreck,
    /// A hold with cargo or settlers aboard.
    Laden,
    /// An encounter: fire, a missile round.
    Combat,
    /// The inspector's selection.
    Selected,
}

pub const STATUS: [(Status, &str); 9] = [
    (Status::Hit, "#ff2d6f"),
    (Status::Damage, "#ffb703"),
    (Status::Drive, "#4df3ff"),
    (Status::DriveHot, "#b8ff3d"),
    (Status::Braking, "#a77bff"),
    (Status::Wreck, "#9a7b5c"),
    (Status::Laden, "#fff36b"),
    (Status::Combat, "#ff4fe1"),
    (Status::Selected, "#f2fff6"),
];

pub fn status(s: Status) -> Rgb {
    Rgb::from_hex(STATUS.iter().find(|(k, _)| *k == s).map(|(_, h)| *h).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{contrast, delta_e, to_oklch};

    #[test]
    fn the_source_palette_is_the_authors_verbatim() {
        let get = |n: &str| SOURCE.iter().find(|(k, _)| *k == n).unwrap().1;
        assert_eq!(get("hy_base03"), [0, 43, 54]);
        assert_eq!(get("hy_base3"), [253, 246, 227]);
        assert_eq!(get("hy_magenta1"), [201, 13, 105]);
        assert_eq!(get("hy_red1"), [181, 25, 21]);
        for fam in ["magenta", "violet", "blue", "cyan", "green", "yellow", "orange", "red"] {
            for s in ["3", "2", "", "1"] {
                let n = format!("hy_{fam}{s}");
                assert!(SOURCE.iter().any(|(k, _)| *k == n), "{n}");
            }
        }
    }

    #[test]
    fn the_palette_awaits_the_authors_approval() {
        assert_eq!(PALETTE_STATUS, "proposed");
    }

    #[test]
    fn the_tone_map_lands_in_gamut_with_no_pure_black_or_white() {
        let p = Palette::default();
        for (n, c) in &p.named {
            assert!(c.in_gamut(), "{n}");
            assert_ne!(c.to_hex(), "#000000", "{n}");
            assert_ne!(c.to_hex(), "#ffffff", "{n}");
        }
        assert_eq!(Palette::default(), p, "deterministic");
    }

    #[test]
    fn the_tone_map_keeps_the_base_tones_in_lightness_order() {
        let p = Palette::default();
        let order =
            ["hy_base03", "hy_base02", "hy_base01", "hy_base00", "hy_base0", "hy_base1", "hy_base2", "hy_base3"];
        for w in order.windows(2) {
            assert!(to_oklch(p.get(w[1])).l > to_oklch(p.get(w[0])).l, "{} above {}", w[1], w[0]);
        }
    }

    #[test]
    fn the_tone_map_warms_the_palette() {
        let p = Palette::default();
        let ratio = |n: &str| {
            let [r, g, b] = SOURCE.iter().find(|(k, _)| *k == n).unwrap().1;
            to_oklch(p.get(n)).c / to_oklch(Rgb::from_ints(r, g, b)).c
        };
        assert!(ratio("hy_blue") < ratio("hy_orange"), "blue mutes more than orange");
        assert!(ratio("hy_violet") < ratio("hy_yellow"), "violet mutes more than yellow");
    }

    #[test]
    fn text_reads_against_the_ground_and_the_panel() {
        let r = Palette::default().roles;
        assert!(contrast(r.text, r.ground) >= 4.5, "WCAG AA on the ground");
        assert!(contrast(r.text, r.panel) >= 4.5, "WCAG AA on a panel");
        assert!(contrast(r.text_dim, r.ground) >= 3.0, "a dim label at 3:1");
    }

    #[test]
    fn eighteen_seats_get_eighteen_colors_legible_and_apart() {
        let p = Palette::default();
        let seats: Vec<Rgb> = (0..18).map(|i| p.seat(i)).collect();
        for (i, s) in seats.iter().enumerate() {
            let k = contrast(*s, p.roles.ground);
            assert!(k >= 3.0, "seat {i} {} on the ground: {k:.2}", s.to_hex());
        }
        for i in 0..18 {
            for j in i + 1..18 {
                assert!(delta_e(seats[i], seats[j]) >= 0.03, "seats {i} and {j}");
            }
        }
    }

    #[test]
    fn the_three_archetypes_take_their_own_supers_family() {
        let p = Palette::default();
        assert_eq!((p.seat(0), p.seat(1), p.seat(2)), (p.get("hy_blue"), p.get("hy_red"), p.get("hy_green")));
    }

    #[test]
    fn every_doctrine_role_has_an_accent_apart_from_the_ground() {
        let p = Palette::default();
        for r in ["Scout", "Colonizer", "Miner", "Freighter", "Picket", "Sentry", "Reserve"] {
            assert!(contrast(p.role(r), p.roles.ground) >= 3.0, "{r}");
        }
    }

    #[test]
    fn status_colors_sit_off_the_palette_apart_and_legible() {
        let p = Palette::default();
        for (s, hex) in STATUS {
            let c = Rgb::from_hex(hex);
            let nearest = p.named.iter().map(|(_, q)| delta_e(c, *q)).fold(f64::INFINITY, f64::min);
            assert!(nearest >= 0.04, "{s:?} is off the palette ({nearest:.3})");
            assert!(contrast(c, p.roles.ground) >= 3.0, "{s:?} reads on the ground");
        }
        for (i, (a, x)) in STATUS.iter().enumerate() {
            for (b, y) in &STATUS[i + 1..] {
                let d = delta_e(Rgb::from_hex(x), Rgb::from_hex(y));
                assert!(d >= 0.06, "{a:?} and {b:?} apart ({d:.3})");
            }
        }
    }

    #[test]
    fn the_tone_map_parameters_move_the_palette() {
        let softer = Palette::new(ToneMap { cool_chroma: 0.3, ..EARTHRISE });
        assert_ne!(softer.get("hy_blue"), Palette::default().get("hy_blue"));
    }
}
