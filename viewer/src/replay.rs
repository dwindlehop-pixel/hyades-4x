//! **A replay, read** (`docs/Hyades_interface.md` §2): the header, the
//! galaxy, the hull table, the frames and the events of one recorded run —
//! and the state of the theater at any instant between its frames.

use crate::json::Value;
use std::collections::BTreeMap;

pub const FORMAT: &str = "hyades-replay";
pub const VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub label: String,
    pub seed: u64,
    pub seats: usize,
    pub horizon_years: f64,
    pub frame_years: f64,
    pub hex_side_ly: f64,
    pub hex_origin: [f64; 2],
    pub ground: String,
    /// Where the replay opens: `[x, y, radius]`, ly.
    pub focus: Option<[f64; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Seat {
    /// The native super of the seat's archetype: `Blue`, `Red`, `Green`.
    pub archetype: String,
    pub home: u32,
}

/// A world as generated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Planet {
    pub id: u32,
    pub pos: [f64; 3],
    pub hab: f64,
    pub bio_max: f64,
    /// Deposit by basic — Cyan, Magenta, Yellow — as Band readings.
    pub ore: [f64; 3],
    pub home: bool,
}

/// What a hull is for its whole life.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hull {
    pub id: u64,
    pub owner: usize,
    /// Index into [`Replay::hulls`].
    pub hull: usize,
    /// Index into [`Replay::designs`].
    pub design: usize,
    pub beams: u32,
    pub tubes: u32,
    /// When it was wrecked, years; `None` while it stands, and in a replay
    /// recorded before the field (T-158).
    pub wrecked_at: Option<f64>,
}

/// One hull in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub id: u64,
    /// Index into [`Replay::kinds`]: the role its Doctrine has it on.
    pub kind: usize,
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub accel: f64,
    pub burn: i8,
    pub damage: f64,
    pub in_flight: bool,
    pub wrecked: bool,
    pub dest: Option<u32>,
    pub cargo: f64,
    pub settlers: f64,
    /// The cargo by material, kt, in [`MATERIALS`] order; all zero in a
    /// replay that does not carry it.
    pub mix: [f64; 8],
}

/// The materials on the Exchange's books, in the order a replay writes them,
/// by their in-game names (galaxy §4.1). The replay carries the engine's
/// color names; the client shows the ratified ones. The supers' names are
/// placeholders (T-142), so a super shows its color until they are ratified.
pub const MATERIALS: [&str; 8] =
    ["Cage Ice", "Rosepeter", "Voltslate", "Red", "Green", "Blue", "Strange Matter", "Ordnance"];

/// One empire's materials at one planet, in [`MATERIALS`] order: each a Band
/// reading on the cost ladder, `None` where it holds none of that material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Holding {
    pub planet: u32,
    pub seat: usize,
    pub bands: [Option<f64>; 8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub t: f64,
    /// Per planet, in [`Replay::planets`] order: the owning seat.
    pub owner: Vec<Option<usize>>,
    pub pop: Vec<f64>,
    pub works: Vec<f64>,
    /// Per planet, works as a mass, kt; empty in a replay recorded before the
    /// field (T-159).
    pub works_kt: Vec<f64>,
    /// Hulls in id order.
    pub rows: Vec<Row>,
    /// Every non-empty holding, by planet and then by seat.
    pub holdings: Vec<Holding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub t: f64,
    pub category: usize,
    pub kind: String,
    pub seat: Option<usize>,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Replay {
    pub meta: Meta,
    pub seats: Vec<Seat>,
    pub kinds: Vec<String>,
    pub hulls: Vec<String>,
    pub designs: Vec<String>,
    pub categories: Vec<String>,
    pub planets: Vec<Planet>,
    pub hull_table: BTreeMap<u64, Hull>,
    pub frames: Vec<Frame>,
    pub events: Vec<Event>,
    pub events_truncated: bool,
}

/// A hull as the viewer shows it at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HullView {
    pub hull: Hull,
    pub row: Row,
    /// Took damage since the frame before (or was wrecked since).
    pub hit: bool,
    /// Carries cargo or settlers.
    pub laden: bool,
}

/// The theater at one instant.
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    pub t: f64,
    /// The frame at or before `t`.
    pub frame: usize,
    pub hulls: Vec<HullView>,
}

/// A row of numbers read by field name.
struct Fields(Vec<String>);

impl Fields {
    fn of(v: &Value, key: &str) -> Result<Fields, String> {
        let list = v.get(key).and_then(Value::as_arr).ok_or_else(|| format!("no {key}"))?;
        Ok(Fields(list.iter().filter_map(|x| x.as_str().map(String::from)).collect()))
    }

    fn at(&self, name: &str) -> Result<usize, String> {
        self.0.iter().position(|f| f == name).ok_or_else(|| format!("no field {name}"))
    }

    /// The numbers of `row`, indexed by `names`.
    fn read<const N: usize>(&self, row: &Value, names: [&str; N]) -> Result<[f64; N], String> {
        let row = row.as_arr().ok_or("a row is not an array")?;
        let mut out = [0.0; N];
        for (i, n) in names.iter().enumerate() {
            out[i] = row.get(self.at(n)?).and_then(Value::as_f64).ok_or_else(|| format!("{n} is not a number"))?;
        }
        Ok(out)
    }

    /// The numbers of `row` by `names`, `None` for a `null` or a field the
    /// header does not name: for fields a replay gained after its version was
    /// set, and for values that may be absent.
    fn read_opt<const N: usize>(&self, row: &Value, names: [&str; N]) -> [Option<f64>; N] {
        let row = row.as_arr().unwrap_or(&[]);
        names.map(|n| self.at(n).ok().and_then(|i| row.get(i)).and_then(Value::as_f64))
    }
}

fn num(v: &Value, key: &str) -> Result<f64, String> {
    v.get(key).and_then(Value::as_f64).ok_or_else(|| format!("no number {key}"))
}

fn strings(v: &Value) -> Vec<String> {
    v.as_arr().unwrap_or(&[]).iter().filter_map(|x| x.as_str().map(String::from)).collect()
}

fn seat_of(x: f64) -> Option<usize> {
    if x >= 0.0 {
        Some(x as usize)
    } else {
        None
    }
}

impl Replay {
    pub fn from_json(text: &str) -> Result<Replay, String> {
        Replay::from_value(&crate::json::parse(text)?)
    }

    pub fn from_value(v: &Value) -> Result<Replay, String> {
        if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(format!("not a {FORMAT} document (format)"));
        }
        let version = num(v, "version")?;
        if version != VERSION as f64 {
            return Err(format!("replay version {version} is not one this viewer reads ({VERSION})"));
        }
        let m = v.get("meta").ok_or("no meta")?;
        let origin = m.get("hex_origin").and_then(Value::as_arr).unwrap_or(&[]);
        let meta = Meta {
            label: m.get("label").and_then(Value::as_str).unwrap_or("").into(),
            seed: num(m, "seed")? as u64,
            seats: num(m, "seats")? as usize,
            horizon_years: num(m, "horizon_years")?,
            frame_years: num(m, "frame_years")?,
            hex_side_ly: num(m, "hex_side_ly")?,
            hex_origin: [
                origin.first().and_then(Value::as_f64).unwrap_or(0.0),
                origin.get(1).and_then(Value::as_f64).unwrap_or(0.0),
            ],
            ground: m.get("ground").and_then(Value::as_str).unwrap_or("").into(),
            focus: m.get("focus").and_then(Value::as_arr).and_then(|f| match f {
                [x, y, r] => Some([x.as_f64()?, y.as_f64()?, r.as_f64()?]),
                _ => None,
            }),
        };
        let seats = v
            .get("seats")
            .and_then(Value::as_arr)
            .unwrap_or(&[])
            .iter()
            .map(|s| Seat {
                archetype: s.get("archetype").and_then(Value::as_str).unwrap_or("").into(),
                home: s.get("home").and_then(Value::as_f64).unwrap_or(0.0) as u32,
            })
            .collect();
        let enums = v.get("enums").ok_or("no enums")?;
        let list = |k: &str| enums.get(k).map(strings).unwrap_or_default();

        let pf = Fields::of(v, "planet_fields")?;
        let mut planets = Vec::new();
        for row in v.get("planets").and_then(Value::as_arr).ok_or("no planets")? {
            let [id, x, y, z, hab, bio, c, mg, ye, home] =
                pf.read(row, ["id", "x", "y", "z", "hab", "bio_max", "cyan", "magenta", "yellow", "home"])?;
            planets.push(Planet {
                id: id as u32,
                pos: [x, y, z],
                hab,
                bio_max: bio,
                ore: [c, mg, ye],
                home: home != 0.0,
            });
        }

        let hf = Fields::of(v, "hull_fields")?;
        let mut hull_table = BTreeMap::new();
        for row in v.get("hulls").and_then(Value::as_arr).ok_or("no hulls")? {
            let [id, owner, hull, design, beams, tubes] =
                hf.read(row, ["id", "owner", "hull", "design", "beams", "tubes"])?;
            let [wrecked_at] = hf.read_opt(row, ["wrecked_at"]);
            let id = id as u64;
            hull_table.insert(
                id,
                Hull {
                    id,
                    owner: owner as usize,
                    hull: hull as usize,
                    design: design as usize,
                    beams: beams as u32,
                    tubes: tubes as u32,
                    wrecked_at,
                },
            );
        }

        let vf = Fields::of(v, "vehicle_fields")?;
        let hf = Fields::of(v, "holding_fields").unwrap_or(Fields(Vec::new()));
        const MIX: [&str; 8] = [
            "cargo_cyan",
            "cargo_magenta",
            "cargo_yellow",
            "cargo_red",
            "cargo_green",
            "cargo_blue",
            "cargo_apex",
            "cargo_ordnance",
        ];
        const HELD: [&str; 8] = ["cyan", "magenta", "yellow", "red", "green", "blue", "apex", "ordnance"];
        let mut frames = Vec::new();
        for f in v.get("frames").and_then(Value::as_arr).ok_or("no frames")? {
            let col = |k: &str| -> Vec<f64> {
                f.get(k).and_then(Value::as_arr).unwrap_or(&[]).iter().filter_map(Value::as_f64).collect()
            };
            let mut rows = Vec::new();
            for r in f.get("vehicles").and_then(Value::as_arr).ok_or("a frame has no vehicles")? {
                let [id, kind, x, y, z, vx, vy, vz, accel, burn, damage, flags, dest, cargo, settlers] = vf.read(
                    r,
                    [
                        "id", "kind", "x", "y", "z", "vx", "vy", "vz", "accel", "burn", "damage", "flags", "dest",
                        "cargo", "settlers",
                    ],
                )?;
                let flags = flags as u32;
                rows.push(Row {
                    id: id as u64,
                    kind: kind as usize,
                    pos: [x, y, z],
                    vel: [vx, vy, vz],
                    accel,
                    burn: burn as i8,
                    damage,
                    in_flight: flags & 1 != 0,
                    wrecked: flags & 2 != 0,
                    dest: if dest >= 0.0 { Some(dest as u32) } else { None },
                    cargo,
                    settlers,
                    mix: vf.read_opt(r, MIX).map(|x| x.unwrap_or(0.0)),
                });
            }
            rows.sort_by_key(|r| r.id);
            let mut holdings = Vec::new();
            for h in f.get("holdings").and_then(Value::as_arr).unwrap_or(&[]) {
                let [planet, seat] = hf.read(h, ["planet", "seat"])?;
                holdings.push(Holding { planet: planet as u32, seat: seat as usize, bands: hf.read_opt(h, HELD) });
            }
            frames.push(Frame {
                t: num(f, "t")?,
                owner: col("owner").into_iter().map(seat_of).collect(),
                pop: col("pop"),
                works: col("works"),
                works_kt: col("works_kt"),
                rows,
                holdings,
            });
        }
        if frames.is_empty() {
            return Err("a replay needs at least one frame".into());
        }

        let ef = Fields::of(v, "event_fields")?;
        let (t_at, cat_at, kind_at, seat_at, text_at) =
            (ef.at("t")?, ef.at("category")?, ef.at("kind")?, ef.at("seat")?, ef.at("text")?);
        let mut events = Vec::new();
        for e in v.get("events").and_then(Value::as_arr).unwrap_or(&[]) {
            let e = e.as_arr().ok_or("an event is not an array")?;
            let n = |i: usize| e.get(i).and_then(Value::as_f64).unwrap_or(-1.0);
            let s = |i: usize| e.get(i).and_then(Value::as_str).unwrap_or("").to_string();
            events.push(Event {
                t: n(t_at),
                category: n(cat_at).max(0.0) as usize,
                kind: s(kind_at),
                seat: seat_of(n(seat_at)),
                text: s(text_at),
            });
        }

        Ok(Replay {
            meta,
            seats,
            kinds: list("kind"),
            hulls: list("hull"),
            designs: list("design"),
            categories: list("category"),
            planets,
            hull_table,
            frames,
            events,
            events_truncated: v.get("events_truncated").and_then(Value::as_bool).unwrap_or(false),
        })
    }

    /// The first frame's time.
    pub fn t0(&self) -> f64 {
        self.frames.first().map_or(0.0, |f| f.t)
    }

    /// The last frame's time.
    pub fn t1(&self) -> f64 {
        self.frames.last().map_or(0.0, |f| f.t)
    }

    /// The last frame at or before `t` (the first frame before it).
    pub fn frame_at(&self, t: f64) -> usize {
        self.frames.partition_point(|f| f.t <= t).saturating_sub(1)
    }

    /// **The theater at `t`**: every hull of the frame at or before `t`, each
    /// moved along the cubic Hermite curve through its position and velocity
    /// at that frame and the next (T-154). The curve meets both frames'
    /// positions and velocities, so a hull's speed changes smoothly across a
    /// frame instead of jumping, and a constant acceleration — the drive's
    /// whole leg — is reproduced exactly.
    pub fn view_at(&self, t: f64) -> View {
        let i = self.frame_at(t);
        let f = &self.frames[i];
        let next = self.frames.get(i + 1);
        let prev = if i > 0 { self.frames.get(i - 1) } else { None };
        let span = next.map_or(0.0, |n| n.t - f.t);
        let frac = if span > 0.0 { ((t - f.t) / span).clamp(0.0, 1.0) } else { 0.0 };
        let find = |fr: &Frame, id: u64| fr.rows.binary_search_by_key(&id, |r| r.id).ok().map(|k| fr.rows[k]);
        let hulls = f
            .rows
            .iter()
            .map(|r| {
                let mut row = *r;
                if let Some(n) = next.and_then(|n| find(n, r.id)) {
                    (row.pos, row.vel) = hermite(r.pos, r.vel, n.pos, n.vel, span, frac);
                }
                let mut hit =
                    prev.and_then(|p| find(p, r.id)).is_some_and(|p| r.damage > p.damage || (r.wrecked && !p.wrecked));
                let hull = self.hull_table.get(&r.id).copied().unwrap_or(Hull {
                    id: r.id,
                    owner: 0,
                    hull: 0,
                    design: self.designs.len().saturating_sub(1),
                    beams: 0,
                    tubes: 0,
                    wrecked_at: None,
                });
                // A wreck between frames shows from the instant it happened,
                // not from the next frame (T-158).
                if !row.wrecked && hull.wrecked_at.is_some_and(|w| w <= t) {
                    (row.wrecked, row.damage, hit) = (true, 1.0, true);
                }
                HullView { hull, row, hit, laden: r.cargo > 0.0 || r.settlers > 0.0 }
            })
            .collect();
        View { t, frame: i, hulls }
    }
}

/// **A cubic Hermite step**: the position and velocity at a share `s` of a
/// span of `dt` years from `(p0, v0)` to `(p1, v1)`.
pub fn hermite(p0: [f64; 3], v0: [f64; 3], p1: [f64; 3], v1: [f64; 3], dt: f64, s: f64) -> ([f64; 3], [f64; 3]) {
    let (s2, s3) = (s * s, s * s * s);
    let (h00, h10, h01, h11) = (2.0 * s3 - 3.0 * s2 + 1.0, s3 - 2.0 * s2 + s, -2.0 * s3 + 3.0 * s2, s3 - s2);
    let (d00, d10, d01, d11) = (6.0 * s2 - 6.0 * s, 3.0 * s2 - 4.0 * s + 1.0, -6.0 * s2 + 6.0 * s, 3.0 * s2 - 2.0 * s);
    let mut p = [0.0; 3];
    let mut v = [0.0; 3];
    for k in 0..3 {
        p[k] = h00 * p0[k] + h10 * dt * v0[k] + h01 * p1[k] + h11 * dt * v1[k];
        v[k] = if dt > 0.0 { (d00 * p0[k] + d01 * p1[k]) / dt + d10 * v0[k] + d11 * v1[k] } else { v0[k] };
    }
    (p, v)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A two-frame replay, written out by hand: two seats, three worlds, a
    /// freighter flying from (0,0,0) to (10,0,0), a picket taking damage and
    /// then wrecked, and a scout that appears only in the second frame.
    pub(crate) const TINY: &str = r#"{
      "format": "hyades-replay", "version": 1,
      "meta": {"label": "tiny", "seed": 3, "seats": 2, "planets": 3, "horizon_years": 20, "frame_years": 10,
               "hex_side_ly": 70, "hex_origin": [52.5, 30.311], "ground": "Random",
               "focus": [20, 0, 2]},
      "seats": [{"archetype": "Blue", "home": 0}, {"archetype": "Red", "home": 1}],
      "enums": {"kind": ["Scout","Colonizer","Miner","Freighter","Picket","Sentry","Reserve","Scrapped"],
                "hull": ["LSV","MSV","GSV","LCV","LCU","GCV","GCU","LOU","ROU","GOU"],
                "design": ["Meadow","Spur","Tor","Cairn","Delta","Range","Scarp","Ford","Strait","Butte","Mesa","Unnamed"],
                "category": ["Production","Mining","Vehicles","Population","Scanning","Cards","Combat"],
                "material": ["Cyan","Magenta","Yellow","Red","Green","Blue","Apex","Ordnance"]},
      "planet_fields": ["id","x","y","z","hab","bio_max","cyan","magenta","yellow","home"],
      "planets": [[0, 0, 0, 0, 4.2, 4.2, 0, 0, 0, 1], [1, 20, 0, 0, 4.2, 4.2, 0, 0, 0, 1], [2, 10, 5, 1, 3.1, 3, 2.5, 0, 0.1, 0]],
      "frame_planet_fields": ["owner","pop","works","works_kt"],
      "vehicle_fields": ["id","kind","x","y","z","vx","vy","vz","accel","burn","damage","flags","dest","cargo","settlers",
                         "cargo_cyan","cargo_magenta","cargo_yellow","cargo_red","cargo_green","cargo_blue",
                         "cargo_apex","cargo_ordnance"],
      "holding_fields": ["planet","seat","cyan","magenta","yellow","red","green","blue","apex","ordnance"],
      "frames": [
        {"t": 0, "owner": [0, 1, -1], "pop": [2.8, 2.8, 0], "works": [2, 2, 0], "works_kt": [1, 1, 0],
         "vehicles": [[5, 3, 0, 0, 0, 0, 0, 0, 0.2, 1, 0, 1, 2, 4.5, 0, 3, 0, 1.5, 0, 0, 0, 0, 0],
                      [6, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0.25, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
         "holdings": [[0, 0, 2.1, 1.8, 3.25, null, null, null, null, null],
                      [2, 1, -0.4, null, null, null, null, null, null, null]]},
        {"t": 10, "owner": [0, 1, 0], "pop": [2.9, 2.8, 1.1], "works": [2, 2, 1], "works_kt": [1, 2, 0.5],
         "vehicles": [[5, 3, 10, 0, 0, 2, 0, 0, 0, 0, 0, 0, 2, 4.5, 0, 3, 0, 1.5, 0, 0, 0, 0, 0],
                      [6, 7, 21, 0, 0, 0.1, 0, 0, 0, 0, 1, 3, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                      [9, 0, 0, 0, 0, 0, 0, 0, 0.5, 1, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
         "holdings": [[0, 0, 2.2, 1.7, 3.3, 0.5, null, null, null, null],
                      [1, 1, 1.5, 1.5, 1.5, null, null, null, null, null]]}
      ],
      "hull_fields": ["id","owner","hull","design","beams","tubes","wrecked_at"],
      "hulls": [[5, 0, 1, 7, 0, 0, null], [6, 1, 3, 3, 2, 0, 9.5], [9, 0, 0, 1, 0, 0, null]],
      "event_fields": ["t","category","kind","seat","text"],
      "events": [[2.5, 2, "VehicleSpawned", 0, "P0 builds a Spur"], [9.5, 6, "HullWrecked", 1, "P1 hull wrecked by P0"],
                 [9.9, 2, "ColonyFounded", 0, "P0 founds a colony at planet#2"]],
      "events_truncated": false
    }"#;

    fn tiny() -> Replay {
        Replay::from_json(TINY).expect("the fixture reads")
    }

    #[test]
    fn a_replay_reads_its_header_galaxy_hulls_frames_and_events() {
        let r = tiny();
        assert_eq!(r.meta.label, "tiny");
        assert_eq!((r.meta.seats, r.meta.frame_years, r.meta.hex_origin), (2, 10.0, [52.5, 30.311]));
        assert_eq!(r.meta.focus, Some([20.0, 0.0, 2.0]));
        assert_eq!(r.seats[1], Seat { archetype: "Red".into(), home: 1 });
        assert_eq!(r.planets.len(), 3);
        assert_eq!(r.planets[2].pos, [10.0, 5.0, 1.0]);
        assert_eq!(r.planets[2].ore, [2.5, 0.0, 0.1]);
        assert!(r.planets[0].home && !r.planets[2].home);
        assert_eq!(
            r.hull_table[&6],
            Hull { id: 6, owner: 1, hull: 3, design: 3, beams: 2, tubes: 0, wrecked_at: Some(9.5) }
        );
        assert_eq!(r.hull_table[&5].wrecked_at, None);
        assert_eq!(r.designs[r.hull_table[&5].design], "Ford");
        assert_eq!(r.frames.len(), 2);
        assert_eq!(r.frames[1].owner, vec![Some(0), Some(1), Some(0)]);
        let f = r.frames[0].rows[0];
        assert_eq!(
            (f.id, f.kind, f.burn, f.in_flight, f.wrecked, f.dest, f.cargo),
            (5, 3, 1, true, false, Some(2), 4.5)
        );
        assert!(r.frames[1].rows[1].wrecked);
        assert_eq!(f.mix, [3.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0], "the cargo by material");
        assert_eq!(
            r.frames[0].holdings,
            vec![
                Holding { planet: 0, seat: 0, bands: [Some(2.1), Some(1.8), Some(3.25), None, None, None, None, None] },
                Holding { planet: 2, seat: 1, bands: [Some(-0.4), None, None, None, None, None, None, None] },
            ],
            "a seat holds at a rock it does not own"
        );
        assert_eq!(r.events.len(), 3);
        assert_eq!(r.events[1].kind, "HullWrecked");
        assert_eq!(r.events[1].seat, Some(1));
        assert_eq!(r.categories[r.events[1].category], "Combat");
        assert!(!r.events_truncated);
        assert_eq!((r.t0(), r.t1()), (0.0, 10.0));
    }

    #[test]
    fn a_reader_refuses_another_format_or_a_version_it_does_not_know() {
        assert!(Replay::from_json(&TINY.replace("hyades-replay", "other")).unwrap_err().contains("format"));
        assert!(Replay::from_json(&TINY.replace("\"version\": 1", "\"version\": 2")).unwrap_err().contains("version"));
        assert!(Replay::from_json("{").is_err());
    }

    #[test]
    fn a_replay_without_the_material_fields_reads_with_none() {
        // A replay recorded before cargo by material and holdings were
        // written still opens: those read as empty.
        let older = TINY.replace("\"cargo_cyan\"", "\"unknown\"").replace("\"holdings\":", "\"ignored\":");
        let r = Replay::from_json(&older).unwrap();
        assert_eq!(r.frames[0].rows[0].mix[0], 0.0);
        assert!(r.frames.iter().all(|f| f.holdings.is_empty()));
    }

    #[test]
    fn fields_are_read_by_name_not_by_position() {
        // Swap two vehicle fields in the header and in every row: the reader
        // follows the names.
        let swapped = TINY
            .replace("[\"id\",\"kind\",\"x\"", "[\"kind\",\"id\",\"x\"")
            .replace("[5, 3, 0,", "[3, 5, 0,")
            .replace("[6, 4, 20,", "[4, 6, 20,")
            .replace("[5, 3, 10,", "[3, 5, 10,")
            .replace("[6, 7, 21,", "[7, 6, 21,")
            .replace("[9, 0, 0, 0, 0, 0, 0, 0, 0.5", "[0, 9, 0, 0, 0, 0, 0, 0, 0.5");
        assert_eq!(Replay::from_json(&swapped).unwrap().frames, tiny().frames);
    }

    #[test]
    fn the_frame_at_an_instant_is_the_last_one_at_or_before_it() {
        let r = tiny();
        assert_eq!(r.frame_at(-1.0), 0);
        assert_eq!(r.frame_at(0.0), 0);
        assert_eq!(r.frame_at(9.99), 0);
        assert_eq!(r.frame_at(10.0), 1);
        assert_eq!(r.frame_at(50.0), 1);
    }

    #[test]
    fn a_hull_between_frames_follows_its_acceleration_and_not_a_straight_line() {
        // Hull 5 leaves rest at 0 and reaches 10 ly at 2 ly/yr after 10 yr: a
        // constant 0.2 ly/yr², which the interpolation reproduces exactly.
        let v = tiny().view_at(2.5);
        let f = v.hulls.iter().find(|h| h.row.id == 5).unwrap();
        assert!((f.row.pos[0] - 0.5 * 0.2 * 2.5 * 2.5).abs() < 1e-12, "{:?}", f.row.pos);
        assert!((f.row.vel[0] - 0.2 * 2.5).abs() < 1e-12, "and its velocity: {:?}", f.row.vel);
        assert!(f.laden, "it carries 4.5 kt");
        assert_eq!(v.frame, 0);
    }

    #[test]
    fn interpolated_motion_has_no_kink_at_a_frame() {
        // The velocity just before a frame equals the frame's own: speed
        // changes smoothly through it rather than stepping.
        let (p, v) = hermite([0.0; 3], [1.0, 0.0, 0.0], [3.0, 0.0, 0.0], [5.0, 0.0, 0.0], 1.0, 1.0);
        assert_eq!((p[0], v[0]), (3.0, 5.0));
        let (p, v) = hermite([0.0; 3], [1.0, 0.0, 0.0], [3.0, 0.0, 0.0], [5.0, 0.0, 0.0], 1.0, 0.0);
        assert_eq!((p[0], v[0]), (0.0, 1.0));
        // Constant velocity is a straight line at that velocity.
        let (p, v) = hermite([1.0; 3], [2.0; 3], [3.0; 3], [2.0; 3], 1.0, 0.25);
        assert!(p.iter().all(|x| (x - 1.5).abs() < 1e-12) && v.iter().all(|x| (x - 2.0).abs() < 1e-12));
    }

    #[test]
    fn a_hull_is_shown_from_the_frame_it_first_appears_in() {
        let r = tiny();
        assert!(!r.view_at(9.0).hulls.iter().any(|h| h.row.id == 9), "not before its frame");
        assert!(r.view_at(10.0).hulls.iter().any(|h| h.row.id == 9));
    }

    #[test]
    fn every_hull_of_the_frame_is_in_the_view() {
        let r = tiny();
        for (i, f) in r.frames.iter().enumerate() {
            let v = r.view_at(f.t);
            assert_eq!(v.hulls.len(), f.rows.len(), "frame {i}");
        }
    }

    #[test]
    fn a_hit_is_damage_taken_since_the_frame_before() {
        let r = tiny();
        let picket = |t: f64| *r.view_at(t).hulls.iter().find(|h| h.row.id == 6).unwrap();
        assert!(!picket(0.0).hit, "nothing before the first frame to compare with");
        let later = picket(10.0);
        assert!(later.hit && later.row.wrecked, "0.25 then wrecked");
        assert!(!r.view_at(10.0).hulls.iter().find(|h| h.row.id == 5).unwrap().hit);
    }

    #[test]
    fn a_wreck_shows_from_the_instant_it_happened_not_the_next_frame() {
        let r = tiny();
        let picket = |t: f64| *r.view_at(t).hulls.iter().find(|h| h.row.id == 6).unwrap();
        assert!(!picket(9.49).row.wrecked && picket(9.49).row.damage == 0.25, "standing until 9.5");
        let w = picket(9.5);
        assert!(w.row.wrecked && w.row.damage == 1.0 && w.hit, "wrecked at 9.5, a frame early");
    }
}
