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

/// The slot after the eight materials: people aboard (see [`Palette::material`]).
pub const PEOPLE: usize = 8;

/// How much a glyph's fill is moved from its seat's color toward the ground,
/// so the edge reads first (proposed).
pub const DEFAULT_FILL_DIM: f64 = 0.45;

/// **Everything the author tunes**, as one value with a one-line text form
/// (`docs/Hyades_interface.md` §6.2): the tone map, the glyph fill's dimming,
/// and colors set by hand — a [`SOURCE`] name (`hy_red`) replaces that color
/// after the tone map, a [`Status`] name (`Hit`) replaces that status color.
///
/// The text form is `key=value` pairs separated by spaces or `&`:
/// `ink=0.02 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228
/// fill=0.45 hy_red=#c83a2c Hit=#ff2d6f`. It is what the live editor puts in
/// the page's link and what the author sends back to ratify.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub map: ToneMap,
    pub fill_dim: f64,
    /// `(name, color)`, sorted by name, one per name.
    pub overrides: Vec<(String, Rgb)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { map: EARTHRISE, fill_dim: DEFAULT_FILL_DIM, overrides: Vec::new() }
    }
}

fn number(v: f64) -> String {
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let m = &self.map;
        let anchors: Vec<String> = m.anchors.iter().map(|a| number(*a)).collect();
        write!(
            f,
            "ink={} paper={} warm={} cool={} pull={} anchors={} fill={}",
            number(m.ink),
            number(m.paper),
            number(m.warm_chroma),
            number(m.cool_chroma),
            number(m.pull),
            anchors.join(","),
            number(self.fill_dim)
        )?;
        for (name, c) in &self.overrides {
            write!(f, " {name}={}", c.to_hex())?;
        }
        Ok(())
    }
}

impl Settings {
    /// Reads the text form. Keys left out keep their defaults; an unknown key,
    /// a malformed color or a value out of its range is an error naming it.
    pub fn parse(text: &str) -> Result<Settings, String> {
        let mut out = Settings::default();
        for pair in text.split(|c: char| c.is_whitespace() || c == '&').filter(|p| !p.is_empty()) {
            let (key, value) = pair.split_once('=').ok_or_else(|| format!("'{pair}' is not key=value"))?;
            let num = |lo: f64, hi: f64| -> Result<f64, String> {
                let x: f64 = value.parse().map_err(|_| format!("{key}: '{value}' is not a number"))?;
                if x.is_finite() && (lo..=hi).contains(&x) {
                    Ok(x)
                } else {
                    Err(format!("{key}: {value} is outside {lo} to {hi}"))
                }
            };
            match key {
                "ink" => out.map.ink = num(0.0, 1.0)?,
                "paper" => out.map.paper = num(0.0, 1.0)?,
                "warm" => out.map.warm_chroma = num(0.0, 2.0)?,
                "cool" => out.map.cool_chroma = num(0.0, 2.0)?,
                "pull" => out.map.pull = num(0.0, 1.0)?,
                "fill" => out.fill_dim = num(0.0, 1.0)?,
                "anchors" => {
                    let a: Vec<f64> = value
                        .split(',')
                        .map(|x| x.parse::<f64>().ok().filter(|h| h.is_finite() && (0.0..=360.0).contains(h)))
                        .collect::<Option<_>>()
                        .ok_or_else(|| format!("anchors: '{value}' is not four hues from 0 to 360"))?;
                    out.map.anchors = a.try_into().map_err(|_| format!("anchors: '{value}' is not four hues"))?;
                }
                name if SOURCE.iter().any(|(n, _)| *n == name)
                    || STATUS.iter().any(|(st, _)| status_name(*st) == name) =>
                {
                    let hex = value.trim_start_matches('#');
                    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(format!("{name}: '{value}' is not a #rrggbb color"));
                    }
                    out.overrides.retain(|(n, _)| n != name);
                    out.overrides.push((name.into(), Rgb::from_hex(hex)));
                }
                _ => return Err(format!("'{key}' is not a palette setting")),
            }
        }
        if out.map.ink >= out.map.paper {
            return Err(format!("ink {} must be below paper {}", out.map.ink, out.map.paper));
        }
        out.overrides.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    }

    fn color(&self, name: &str) -> Option<Rgb> {
        self.overrides.iter().find(|(n, _)| n == name).map(|(_, c)| *c)
    }
}

/// The name a [`Status`] takes in [`Settings`] and in the palette sheet.
pub fn status_name(s: Status) -> String {
    format!("{s:?}")
}

/// The whole tactical palette: every [`SOURCE`] color tone-mapped (or set by
/// hand), the interface's roles assigned to them, and the status colors.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub settings: Settings,
    pub named: Vec<(&'static str, Rgb)>,
    pub roles: Roles,
    /// In [`STATUS`] order.
    pub status: [Rgb; 9],
}

impl Palette {
    pub fn new(map: ToneMap) -> Palette {
        Palette::with(Settings { map, ..Settings::default() })
    }

    pub fn with(settings: Settings) -> Palette {
        let named: Vec<(&'static str, Rgb)> = SOURCE
            .iter()
            .map(|&(n, [r, g, b])| {
                (n, settings.color(n).unwrap_or_else(|| tone_map(Rgb::from_ints(r, g, b), &settings.map)))
            })
            .collect();
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
        let status = STATUS.map(|(st, hex)| settings.color(&status_name(st)).unwrap_or_else(|| Rgb::from_hex(hex)));
        Palette { settings, named, roles, status }
    }

    /// A status color.
    pub fn status(&self, s: Status) -> Rgb {
        self.status[STATUS.iter().position(|(k, _)| *k == s).unwrap()]
    }

    /// The glyph fill's share of the way from the seat's color to the ground.
    pub fn fill_dim(&self) -> f64 {
        self.settings.fill_dim
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

    /// **A material on the Exchange's books** (`crate::replay::MATERIALS`
    /// order), and at [`PEOPLE`] the settlers a colonizer carries: the color a
    /// holding's bar and a hold's stripe are drawn in. The basics and supers
    /// are the palette's own hues of their names; Strange Matter is the pale
    /// violet, rounds the orange, people the paper (proposed, part of R-UI1).
    pub fn material(&self, i: usize) -> Rgb {
        self.get(match i {
            0 => "hy_cyan",
            1 => "hy_magenta",
            2 => "hy_yellow",
            3 => "hy_red",
            4 => "hy_green",
            5 => "hy_blue",
            6 => "hy_violet3",
            7 => "hy_orange",
            _ => "hy_base2",
        })
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
        Palette::with(Settings::default())
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
    fn settings_print_and_read_back_and_the_default_is_the_proposal() {
        let d = Settings::default();
        assert_eq!(d.to_string(), "ink=0.02 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228 fill=0.45");
        assert_eq!(Settings::parse(&d.to_string()), Ok(d.clone()));
        assert_eq!(Settings::parse(""), Ok(d), "every key left out keeps its default");
        let s = Settings::parse("pull=0.5&Hit=#00ff00  hy_red=C83A2C ink=0.1").unwrap();
        assert_eq!((s.map.pull, s.map.ink), (0.5, 0.1));
        assert_eq!(
            s.to_string(),
            "ink=0.1 paper=0.95 warm=0.92 cool=0.6 pull=0.5 anchors=38,78,118,228 fill=0.45 Hit=#00ff00 hy_red=#c83a2c"
        );
        assert_eq!(Settings::parse(&s.to_string()), Ok(s));
    }

    #[test]
    fn a_setting_out_of_range_or_unknown_is_refused_by_name() {
        for (bad, says) in [
            ("ink=2", "ink"),
            ("paper=0.01", "below paper"),
            ("anchors=1,2,3", "anchors"),
            ("anchors=1,2,3,400", "anchors"),
            ("hy_red=#12345", "hy_red"),
            ("hy_rouge=#123456", "hy_rouge"),
            ("pull", "key=value"),
            ("warm=NaN", "warm"),
        ] {
            let e = Settings::parse(bad).unwrap_err();
            assert!(e.contains(says), "{bad}: {e}");
        }
    }

    #[test]
    fn a_color_set_by_hand_reaches_every_role_and_seat_that_uses_it() {
        let red = Rgb::from_hex("#c83a2c");
        let p = Palette::with(Settings::parse("hy_red=#c83a2c hy_base03=#101010 Wreck=#123456").unwrap());
        assert_eq!(p.seat(1), red, "seat 1 is the Red archetype's own color");
        assert_eq!(p.roles.ground.to_hex(), "#101010");
        assert_eq!(p.status(Status::Wreck).to_hex(), "#123456");
        assert_eq!(p.status(Status::Hit).to_hex(), "#ff2d6f", "others keep the proposal");
        assert_eq!(p.get("hy_blue"), Palette::default().get("hy_blue"));
    }

    #[test]
    fn the_tone_map_parameters_move_the_mapped_colors() {
        let a = Palette::default();
        let b = Palette::with(Settings::parse("ink=0.15").unwrap());
        assert!(to_oklch(b.roles.ground).l > to_oklch(a.roles.ground).l + 0.05);
        let c = Palette::with(Settings::parse("cool=0.2").unwrap());
        assert!(to_oklch(c.get("hy_blue")).c < to_oklch(a.get("hy_blue")).c);
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
