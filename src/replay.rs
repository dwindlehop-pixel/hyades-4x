//! **Replays** — a run recorded for the presentation layer
//! (`docs/Hyades_interface.md` §2).
//!
//! The presentation never reaches into the simulation (design law #15). It
//! reads a replay: the galaxy as generated, a [`Snapshot`] taken every
//! `frame_years`, and the logged events, written as one JSON document. A
//! viewer seeks, rewinds and filters within the document; nothing it does
//! reaches back. The format is versioned, self-describing (every row carries
//! a field list in the header) and a pure function of the run, so one seed
//! records one byte-identical replay.
//!
//! Pure data: nothing here touches the filesystem or the clock — a harness
//! writes the string where it wants (`examples/record_replay`).

use crate::galaxy::Galaxy;
use crate::sim::Simulation;
use crate::snapshot::Snapshot;

/// The format's name and version, written into every replay.
pub const FORMAT: &str = "hyades-replay";
/// **Version 1.** A reader refuses a version it does not know.
pub const VERSION: u32 = 1;

/// How a run is recorded.
#[derive(Clone, Debug)]
pub struct ReplayConfig {
    /// Years between frames. A viewer interpolates between them.
    pub frame_years: f64,
    /// The most logged events written; past it the replay says it was
    /// truncated.
    pub max_events: usize,
    /// A human label for the replay's index.
    pub label: String,
    /// Where a viewer opens: a circle on the galaxy plane, `(x, y, radius)`
    /// in ly, around the action a replay was recorded to show. `None` opens
    /// on the whole galaxy.
    pub focus: Option<(f64, f64, f64)>,
}

/// Vehicle kinds, in the order a replay indexes them.
const KINDS: [&str; 8] = ["Scout", "Colonizer", "Miner", "Freighter", "Picket", "Sentry", "Reserve", "Scrapped"];
/// Hull types by the docs' abbreviation, in the order a replay indexes them.
const HULLS: [&str; 10] = ["LSV", "MSV", "GSV", "LCV", "LCU", "GCV", "GCU", "LOU", "ROU", "GOU"];
/// Design classes, in the order a replay indexes them.
const DESIGNS: [&str; 12] =
    ["Meadow", "Spur", "Tor", "Cairn", "Delta", "Range", "Scarp", "Ford", "Strait", "Butte", "Mesa", "Unnamed"];
/// The materials on the Exchange's books, in [`Material`] order — what a
/// hold carries and a holding keeps.
///
/// [`Material`]: crate::resources::Material
const MATERIALS: [&str; 8] = ["Cyan", "Magenta", "Yellow", "Red", "Green", "Blue", "Apex", "Ordnance"];
/// Log categories, in [`LogCategory::ALL`] order.
const CATEGORIES: [&str; 7] = ["Production", "Mining", "Vehicles", "Population", "Scanning", "Cards", "Combat"];

/// A planet as generated: `id`, position (ly), habitability and pristine
/// biosphere (Band readings), its deposit by color (Band readings), and
/// whether it is a homeworld.
const PLANET_FIELDS: [&str; 10] = ["id", "x", "y", "z", "hab", "bio_max", "cyan", "magenta", "yellow", "home"];
/// Per frame, one array per field, aligned to `planets`: the owning seat
/// (`-1` unowned), population and works as Band readings.
const FRAME_PLANET_FIELDS: [&str; 3] = ["owner", "pop", "works"];
/// What a hull is for its whole life, once per hull: its id, owning seat,
/// hull type and Design (indices into `enums`), beam mounts and missile tubes.
const HULL_FIELDS: [&str; 6] = ["id", "owner", "hull", "design", "beams", "tubes"];
/// One hull per row per frame: its id, the role its Doctrine has it on
/// (`kind`), position ly, velocity ly/yr, acceleration ly/yr², `burn` the
/// drive's sense, `damage` a share of structure, `flags` bit 0 in flight and
/// bit 1 wrecked, `dest` a planet id or `-1`, cargo and settlers kt, and the
/// cargo by material, kt, in [`MATERIALS`] order.
const VEHICLE_FIELDS: [&str; 23] = [
    "id",
    "kind",
    "x",
    "y",
    "z",
    "vx",
    "vy",
    "vz",
    "accel",
    "burn",
    "damage",
    "flags",
    "dest",
    "cargo",
    "settlers",
    "cargo_cyan",
    "cargo_magenta",
    "cargo_yellow",
    "cargo_red",
    "cargo_green",
    "cargo_blue",
    "cargo_apex",
    "cargo_ordnance",
];
/// Per frame, one row per non-empty holding: the planet, the holding seat,
/// and each material in [`MATERIALS`] order as a Band reading on the cost
/// ladder — a holding is a stock that can be spent — or `null` where it holds
/// none of that material.
const HOLDING_FIELDS: [&str; 10] =
    ["planet", "seat", "cyan", "magenta", "yellow", "red", "green", "blue", "apex", "ordnance"];
const EVENT_FIELDS: [&str; 5] = ["t", "category", "kind", "seat", "text"];

/// Record `sim` from where it stands to its horizon, a frame every
/// `cfg.frame_years`, with whatever events its log filter collects.
///
/// Frame `k` is the theater at `k · frame_years` exactly: every event at or
/// before that year applied, every hull where its motion has it then
/// ([`Simulation::snapshot_at`]). A run whose events stop early still has a
/// frame at every year to its horizon, with hulls coasting or at rest.
pub fn record_run(galaxy: &Galaxy, mut sim: Simulation, cfg: &ReplayConfig) -> String {
    let mut out = String::with_capacity(1 << 20);
    header(&mut out, galaxy, &sim, cfg);
    out.push_str(",\"frames\":[");
    let frames = (sim.horizon_years() / cfg.frame_years).floor() as usize;
    // Every hull seen, by id: what it is, written once after the frames.
    let mut hulls: std::collections::BTreeMap<u64, crate::snapshot::VehicleSnapshot> = Default::default();
    for k in 0..=frames {
        let t = k as f64 * cfg.frame_years;
        while sim.next_event_time().is_some_and(|n| n <= t) {
            sim.step();
        }
        if k > 0 {
            out.push(',');
        }
        let snap = sim.snapshot_at(t);
        for v in &snap.vehicles {
            hulls.entry(v.id).or_insert(*v);
        }
        frame(&mut out, &snap);
    }
    out.push_str("],\"hull_fields\":");
    strings(&mut out, &HULL_FIELDS);
    out.push_str(",\"hulls\":[");
    for (i, v) in hulls.values().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "[{},{},{},{},{},{}]",
            v.id,
            v.owner,
            index(&HULLS, v.hull),
            index(&DESIGNS, v.design),
            v.beams,
            v.tubes
        ));
    }
    out.push(']');
    events(&mut out, &sim, cfg);
    out.push('}');
    out
}

fn header(out: &mut String, galaxy: &Galaxy, sim: &Simulation, cfg: &ReplayConfig) {
    let g = &galaxy.config;
    out.push_str("{\"format\":");
    string(out, FORMAT);
    out.push_str(&format!(",\"version\":{VERSION},\"meta\":{{\"label\":"));
    string(out, &cfg.label);
    out.push_str(&format!(",\"seed\":{},\"seats\":{},\"planets\":{}", g.seed, g.players, galaxy.planets.len()));
    out.push_str(",\"horizon_years\":");
    write_num(out, sim.horizon_years(), 3);
    out.push_str(",\"frame_years\":");
    write_num(out, cfg.frame_years, 3);
    out.push_str(",\"hex_side_ly\":");
    write_num(out, g.hex_side_ly, 3);
    let (ox, oy) = g.hex_grid_origin();
    out.push_str(",\"hex_origin\":[");
    write_num(out, ox, 3);
    out.push(',');
    write_num(out, oy, 3);
    out.push_str("],\"ground\":");
    string(out, &format!("{:?}", g.ground));
    if let Some((x, y, r)) = cfg.focus {
        out.push_str(",\"focus\":[");
        for (i, v) in [x, y, r].into_iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_num(out, v, 4);
        }
        out.push(']');
    }
    out.push_str("},\"seats\":[");
    for (i, id) in galaxy.homeworlds.iter().enumerate() {
        let p = &galaxy.planets[id.0 as usize];
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"archetype\":");
        string(out, &p.archetype.map_or("None".to_string(), |a| format!("{:?}", a.native_super())));
        out.push_str(&format!(",\"home\":{}}}", id.0));
    }
    out.push_str("],\"enums\":{\"kind\":");
    strings(out, &KINDS);
    out.push_str(",\"hull\":");
    strings(out, &HULLS);
    out.push_str(",\"design\":");
    strings(out, &DESIGNS);
    out.push_str(",\"category\":");
    strings(out, &CATEGORIES);
    out.push_str(",\"material\":");
    strings(out, &MATERIALS);
    out.push_str("},\"planet_fields\":");
    strings(out, &PLANET_FIELDS);
    out.push_str(",\"planets\":[");
    for (i, p) in galaxy.planets.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{}", p.id.0));
        for x in [p.position.x, p.position.y, p.position.z] {
            out.push(',');
            write_num(out, x, 2);
        }
        for x in [p.habitability.bands(), p.biosphere.bands()] {
            out.push(',');
            write_num(out, x, 2);
        }
        for b in crate::resources::Basic::ALL {
            out.push(',');
            write_num(out, band_of(p.minerals.get(b).kilotons()), 2);
        }
        out.push_str(if p.is_homeworld { ",1]" } else { ",0]" });
    }
    out.push_str("],\"frame_planet_fields\":");
    strings(out, &FRAME_PLANET_FIELDS);
    out.push_str(",\"vehicle_fields\":");
    strings(out, &VEHICLE_FIELDS);
    out.push_str(",\"holding_fields\":");
    strings(out, &HOLDING_FIELDS);
}

/// A [`Minerals`] in [`MATERIALS`] order, kt.
///
/// [`Minerals`]: crate::resources::Minerals
fn materials(m: &crate::resources::Minerals) -> [f64; 8] {
    [m.cyan, m.magenta, m.yellow, m.red, m.green, m.blue, m.apex, m.ordnance]
}

/// A mass's Band reading, with nothing read as `0` rather than as the
/// logarithm of nothing.
fn band_of(kt: f64) -> f64 {
    if kt > 0.0 {
        crate::units::Kilotons::new(kt).band().bands()
    } else {
        0.0
    }
}

fn frame(out: &mut String, snap: &Snapshot) {
    out.push_str("{\"t\":");
    write_num(out, snap.time_years, 3);
    out.push_str(",\"owner\":[");
    for (i, p) in snap.planets.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&p.owner.map_or("-1".to_string(), |o| o.to_string()));
    }
    out.push_str("],\"pop\":[");
    for (i, p) in snap.planets.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_num(out, band_of(p.population.kilotons()), 2);
    }
    out.push_str("],\"works\":[");
    for (i, p) in snap.planets.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_num(out, p.infrastructure.bands(), 2);
    }
    out.push_str("],\"vehicles\":[");
    for (i, v) in snap.vehicles.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{}", v.id, index(&KINDS, &format!("{:?}", v.kind))));
        for x in [v.position.x, v.position.y, v.position.z] {
            out.push(',');
            write_num(out, x, 2);
        }
        for x in [v.velocity.x, v.velocity.y, v.velocity.z, v.accel] {
            out.push(',');
            write_num(out, x, 4);
        }
        out.push_str(&format!(",{},", v.burn));
        write_num(out, v.damage, 3);
        let flags = u8::from(v.in_flight) | (u8::from(v.wrecked) << 1);
        let dest = v.destination.map_or(-1, |d| d.0 as i64);
        out.push_str(&format!(",{flags},{dest},"));
        write_num(out, v.cargo.total().kilotons(), 3);
        out.push(',');
        write_num(out, v.settlers.kilotons(), 3);
        for x in materials(&v.cargo) {
            out.push(',');
            write_sig(out, x);
        }
        out.push(']');
    }
    out.push_str("],\"holdings\":[");
    for (i, h) in snap.holdings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{}", h.planet.0, h.owner));
        for x in materials(&h.minerals) {
            out.push(',');
            if x > 0.0 {
                write_num(out, crate::units::Price::new(x).band().bands(), 2);
            } else {
                out.push_str("null");
            }
        }
        out.push(']');
    }
    out.push_str("]}");
}

fn events(out: &mut String, sim: &Simulation, cfg: &ReplayConfig) {
    out.push_str(",\"event_fields\":");
    strings(out, &EVENT_FIELDS);
    out.push_str(",\"events\":[");
    let mut truncated = false;
    for (written, r) in sim.log().iter().enumerate() {
        if written == cfg.max_events {
            truncated = true;
            break;
        }
        if written > 0 {
            out.push(',');
        }
        out.push('[');
        write_num(out, r.time, 3);
        let cat = crate::log::LogCategory::ALL.iter().position(|&c| c == r.category).unwrap_or(0);
        out.push_str(&format!(",{cat},"));
        let debug = format!("{:?}", r.event);
        let kind: String = debug.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        string(out, &kind);
        out.push_str(&format!(",{},", r.event.player().map_or(-1, |p| p as i64)));
        string(out, &r.event.to_string());
        out.push(']');
    }
    out.push_str(&format!("],\"events_truncated\":{truncated}"));
}

/// `name`'s place in `list`, the last entry when it is not there.
fn index(list: &[&str], name: &str) -> usize {
    list.iter().position(|&n| n == name).unwrap_or(list.len() - 1)
}

fn strings(out: &mut String, list: &[&str]) {
    out.push('[');
    for (i, s) in list.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(out, s);
    }
    out.push(']');
}

/// A JSON string, escaped.
fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Write `x` with at most `dp` decimals, trailing zeros dropped and no
/// negative zero. **A non-finite number is fatal** (design law #16): a NaN in
/// a replay is a NaN in the state it was taken from.
pub fn write_num(out: &mut String, x: f64, dp: usize) {
    assert!(x.is_finite(), "non-finite number in a replay: {x}");
    let mut s = format!("{x:.dp$}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    out.push_str(&s);
}

/// Write `x` to four significant figures: in decimals from 0.001 to below a
/// million, in exponent form outside that. A holding runs from tonnes to
/// millions of kilotonnes, which a fixed number of decimals cannot carry.
/// Non-finite is fatal, as in [`write_num`].
pub fn write_sig(out: &mut String, x: f64) {
    assert!(x.is_finite(), "non-finite number in a replay: {x}");
    if x == 0.0 {
        out.push('0');
        return;
    }
    let e = format!("{x:.3e}");
    let (mantissa, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if (-3..6).contains(&exp) {
        write_num(out, x, (3 - exp).max(0) as usize);
    } else {
        out.push_str(mantissa.trim_end_matches('0').trim_end_matches('.'));
        out.push_str(&format!("e{exp}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::GalaxyConfig;
    use crate::log::{LogCategory, LogFilter};
    use crate::sim::SimConfig;

    /// A JSON value, parsed by the test's own reader so the contract is
    /// checked on the bytes a viewer receives.
    #[derive(Debug, Clone, PartialEq)]
    enum J {
        Null,
        Bool(bool),
        Num(f64),
        Str(String),
        Arr(Vec<J>),
        Obj(Vec<(String, J)>),
    }

    impl J {
        fn get(&self, k: &str) -> &J {
            match self {
                J::Obj(v) => &v.iter().find(|(key, _)| key == k).unwrap_or_else(|| panic!("no key {k}")).1,
                _ => panic!("not an object"),
            }
        }
        fn arr(&self) -> &Vec<J> {
            match self {
                J::Arr(v) => v,
                _ => panic!("not an array: {self:?}"),
            }
        }
        fn num(&self) -> f64 {
            match self {
                J::Num(x) => *x,
                _ => panic!("not a number: {self:?}"),
            }
        }
        fn str(&self) -> &str {
            match self {
                J::Str(s) => s,
                _ => panic!("not a string: {self:?}"),
            }
        }
    }

    fn parse(s: &str) -> J {
        let b = s.as_bytes();
        let mut i = 0;
        let v = value(b, &mut i);
        ws(b, &mut i);
        assert_eq!(i, b.len(), "trailing bytes after the document");
        v
    }

    fn ws(b: &[u8], i: &mut usize) {
        while *i < b.len() && b[*i].is_ascii_whitespace() {
            *i += 1;
        }
    }

    fn value(b: &[u8], i: &mut usize) -> J {
        ws(b, i);
        match b[*i] {
            b'{' => {
                *i += 1;
                let mut out = Vec::new();
                ws(b, i);
                if b[*i] == b'}' {
                    *i += 1;
                    return J::Obj(out);
                }
                loop {
                    ws(b, i);
                    let J::Str(k) = value(b, i) else { panic!("object key") };
                    ws(b, i);
                    assert_eq!(b[*i], b':');
                    *i += 1;
                    out.push((k, value(b, i)));
                    ws(b, i);
                    match b[*i] {
                        b',' => *i += 1,
                        b'}' => {
                            *i += 1;
                            return J::Obj(out);
                        }
                        c => panic!("bad object byte {}", c as char),
                    }
                }
            }
            b'[' => {
                *i += 1;
                let mut out = Vec::new();
                ws(b, i);
                if b[*i] == b']' {
                    *i += 1;
                    return J::Arr(out);
                }
                loop {
                    out.push(value(b, i));
                    ws(b, i);
                    match b[*i] {
                        b',' => *i += 1,
                        b']' => {
                            *i += 1;
                            return J::Arr(out);
                        }
                        c => panic!("bad array byte {}", c as char),
                    }
                }
            }
            b'"' => {
                *i += 1;
                let mut s = String::new();
                while b[*i] != b'"' {
                    if b[*i] == b'\\' {
                        *i += 1;
                        match b[*i] {
                            b'n' => s.push('\n'),
                            b't' => s.push('\t'),
                            b'u' => {
                                let h = std::str::from_utf8(&b[*i + 1..*i + 5]).unwrap();
                                s.push(char::from_u32(u32::from_str_radix(h, 16).unwrap()).unwrap());
                                *i += 4;
                            }
                            c => s.push(c as char),
                        }
                    } else {
                        s.push(b[*i] as char);
                    }
                    *i += 1;
                }
                *i += 1;
                J::Str(s)
            }
            b't' => {
                *i += 4;
                J::Bool(true)
            }
            b'f' => {
                *i += 5;
                J::Bool(false)
            }
            b'n' => {
                *i += 4;
                J::Null
            }
            _ => {
                let start = *i;
                while *i < b.len() && (b[*i] == b'-' || b[*i] == b'.' || b[*i] == b'e' || b[*i].is_ascii_digit()) {
                    *i += 1;
                }
                J::Num(
                    std::str::from_utf8(&b[start..*i]).unwrap().parse().unwrap_or_else(|_| panic!("number at {start}")),
                )
            }
        }
    }

    /// Two seats on a 150-world field to 120 yr: expansion has started, so
    /// frames carry hulls in flight and a colony or two.
    fn small(seed: u64, filter: LogFilter) -> (Galaxy, Simulation) {
        let mut g = GalaxyConfig::new(2, seed);
        g.planet_count = 150;
        let galaxy = Galaxy::generate(g).unwrap();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = 120.0;
        let mut sim = Simulation::with_baseline(galaxy.clone(), cfg);
        sim.set_log_filter(filter);
        (galaxy, sim)
    }

    fn rc() -> ReplayConfig {
        ReplayConfig { frame_years: 10.0, max_events: 100_000, label: "test".into(), focus: None }
    }

    /// **Every hull of every frame is in the replay, with every field the
    /// header names** — the tactical mode's invariant (§6) starts here.
    #[test]
    fn a_replay_carries_every_hull_and_world_of_every_frame() {
        let (galaxy, sim) = small(5, LogFilter::none());
        let doc = parse(&record_run(&galaxy, sim, &rc()));
        assert_eq!(doc.get("format").str(), FORMAT);
        assert_eq!(doc.get("version").num(), VERSION as f64);
        let planets = doc.get("planets").arr();
        assert_eq!(planets.len(), galaxy.planets.len());
        let pf = doc.get("planet_fields").arr().len();
        assert!(planets.iter().all(|p| p.arr().len() == pf));
        let vf = doc.get("vehicle_fields").arr().len();
        let hulls = doc.get("hulls").arr();
        let hf = doc.get("hull_fields").arr().len();
        assert!(hulls.iter().all(|h| h.arr().len() == hf));

        // The same run, frame by frame, beside the replay.
        let (_, mut twin) = small(5, LogFilter::none());
        let frames = doc.get("frames").arr();
        assert_eq!(frames.len(), 13, "frames at 0, 10, …, 120 yr");
        let mut hulls_seen = 0;
        let mut holdings_seen = 0;
        let mut laden_seen = 0;
        let fields: Vec<String> = doc.get("vehicle_fields").arr().iter().map(|f| f.str().to_string()).collect();
        assert_eq!(doc.get("holding_fields").arr().len(), 10);
        assert_eq!(doc.get("enums").get("material").arr().len(), 8);
        for (k, f) in frames.iter().enumerate() {
            let t = k as f64 * 10.0;
            while twin.next_event_time().is_some_and(|n| n <= t) {
                twin.step();
            }
            let snap = twin.snapshot_at(t);
            assert_eq!(f.get("t").num(), t, "frame {k} falls on its own year");
            let vs = f.get("vehicles").arr();
            assert_eq!(vs.len(), snap.vehicles.len(), "frame {k}: every hull");
            assert!(vs.iter().all(|v| v.arr().len() == vf), "frame {k}: every field");
            let ids: Vec<f64> = vs.iter().map(|v| v.arr()[0].num()).collect();
            let want: Vec<f64> = snap.vehicles.iter().map(|v| v.id as f64).collect();
            assert_eq!(ids, want, "frame {k}: by id, in entity order");
            for (v, s) in vs.iter().zip(&snap.vehicles) {
                let hull = hulls.iter().find(|h| h.arr()[0].num() == s.id as f64).expect("every hull is in the table");
                assert_eq!(hull.arr()[1].num(), s.owner as f64, "its owner");
                assert_eq!(doc.get("enums").get("design").arr()[hull.arr()[3].num() as usize].str(), s.design);
                assert!((v.arr()[2].num() - s.position.x).abs() <= 0.005, "frame {k}: at its position");
            }
            assert_eq!(f.get("owner").arr().len(), galaxy.planets.len());
            // Cargo by material sums to the cargo, within the written figures.
            let at = |name: &str| fields.iter().position(|f| f == name).unwrap();
            for (v, s) in vs.iter().zip(&snap.vehicles) {
                let row = v.arr();
                let mix: f64 = (at("cargo_cyan")..=at("cargo_ordnance")).map(|i| row[i].num()).sum();
                let total = s.cargo.total().kilotons();
                assert!((mix - total).abs() <= 1e-3 * total.max(1.0), "frame {k}: hull {} {mix} vs {total}", s.id);
                laden_seen += usize::from(mix > 0.0);
            }
            // Every holding, by planet and seat, with what it holds.
            let rows = f.get("holdings").arr();
            assert_eq!(rows.len(), snap.holdings.len(), "frame {k}: every holding");
            for (row, h) in rows.iter().zip(&snap.holdings) {
                let row = row.arr();
                assert_eq!((row[0].num(), row[1].num()), (h.planet.0 as f64, h.owner as f64));
                for (cell, kt) in row[2..].iter().zip(materials(&h.minerals)) {
                    if kt > 0.0 {
                        let band = crate::units::Price::new(kt).band().bands();
                        assert!((cell.num() - band).abs() <= 0.005, "frame {k}: the Band reading of {kt} kt");
                    } else {
                        assert_eq!(cell, &J::Null, "frame {k}: none held is null, not Band 0");
                    }
                }
            }
            holdings_seen += rows.len();
            hulls_seen += vs.len();
        }
        assert!(hulls_seen > 20, "the bed puts hulls in the frames: {hulls_seen}");
        assert!(laden_seen > 0, "the bed puts cargo in a hold");
        assert!(holdings_seen > 13, "every frame holds at least the homeworlds' banks: {holdings_seen}");
    }

    /// **One seed records one replay, byte for byte** (`AGENTS.md` §4): a
    /// replay is a pure function of the run, so it can be regenerated in CI
    /// and diffed.
    #[test]
    fn a_replay_is_byte_identical_for_one_seed() {
        let filter = LogFilter::none().with(LogCategory::Vehicles);
        let (g1, s1) = small(9, filter);
        let (g2, s2) = small(9, filter);
        let (a, b) = (record_run(&g1, s1, &rc()), record_run(&g2, s2, &rc()));
        assert!(!a.is_empty());
        assert_eq!(a, b);
    }

    /// **The log rides along, as the filter collected it** (§4): every event
    /// is in a recorded category, in time order, and a cap below the count
    /// truncates and says so.
    #[test]
    fn a_replay_carries_the_events_its_log_collected() {
        let filter = LogFilter::none().with(LogCategory::Vehicles);
        let (galaxy, sim) = small(5, filter);
        let doc = parse(&record_run(&galaxy, sim, &rc()));
        let cats = doc.get("enums").get("category").arr();
        let events = doc.get("events").arr();
        assert!(events.len() > 10, "the bed logs vehicle events: {}", events.len());
        let mut last = 0.0;
        for e in events {
            let row = e.arr();
            let t = row[0].num();
            assert!(t >= last, "in time order");
            last = t;
            assert_eq!(cats[row[1].num() as usize].str(), "Vehicles");
            assert!(!row[2].str().is_empty() && !row[4].str().is_empty(), "a kind and a text");
        }
        assert_eq!(doc.get("events_truncated"), &J::Bool(false));

        let (galaxy, sim) = small(5, filter);
        let capped = ReplayConfig { max_events: 5, ..rc() };
        let doc = parse(&record_run(&galaxy, sim, &capped));
        assert_eq!(doc.get("events").arr().len(), 5);
        assert_eq!(doc.get("events_truncated"), &J::Bool(true));
    }

    #[test]
    fn numbers_are_written_short_and_exact_to_their_decimals() {
        let mut s = String::new();
        for (x, dp) in [(1.0, 2), (-0.5, 2), (1.23456, 3), (2.0e-9, 2), (-1.0e-9, 2), (120.04, 1)] {
            write_num(&mut s, x, dp);
            s.push(' ');
        }
        assert_eq!(s, "1 -0.5 1.235 0 0 120 ");
    }

    #[test]
    fn significant_figures_carry_a_holding_from_tonnes_to_millions_of_kilotonnes() {
        let mut s = String::new();
        for x in [0.0, 1234.5678, 0.012345, 2.5, 123_456_789.0, 0.000_012_34, -0.5, 999_999.0] {
            write_sig(&mut s, x);
            s.push(' ');
        }
        assert_eq!(s, "0 1235 0.01235 2.5 1.235e8 1.234e-5 -0.5 1e6 ");
        for w in s.split_whitespace() {
            assert!(w.parse::<f64>().is_ok(), "{w} reads back as a number");
        }
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn a_non_finite_number_is_fatal() {
        write_num(&mut String::new(), f64::NAN, 2);
    }
}
