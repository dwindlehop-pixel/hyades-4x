//! **The tactical mode** (`docs/Hyades_interface.md` §6): every entity at its
//! position, as a glyph of its Design in its seat's and Doctrine's colors,
//! with the simulation's state in colors that are not on the palette.
//!
//! Drawn into a coarse framebuffer — one tactical pixel is [`PIXEL`] screen
//! pixels — that the shell scales up without smoothing.
//!
//! Rendering is two steps so the invariant can be tested apart from the
//! pixels: [`plan`] lays out one [`Op`] per world and per hull at its
//! projected position, and [`paint`] draws them.

use crate::camera::{Camera, Lod};
use crate::color::{mix, Rgb};
use crate::glyph::{self, glyph, role_mark, Family, Glyph, Size};
use crate::palette::{Palette, Status, PEOPLE};
use crate::raster::Raster;
use crate::replay::{Replay, View};
use std::collections::BTreeMap;

/// Screen pixels per tactical pixel.
pub const PIXEL: f64 = 2.0;
/// A drive at or above this acceleration, ly/yr² (about 1 g), draws hot.
pub const HOT_ACCEL: f64 = 1.0;
/// Hexes narrower than this many tactical pixels are not drawn: the grid
/// would be a fill.
pub const MIN_HEX_PX: f64 = 6.0;
/// How far a route's color is moved from its seat's toward the ground: a
/// route is the faintest thing drawn (proposed, R-UI4).
pub const ROUTE_DIM: f64 = 0.9;
/// Tactical pixels of a holding's bar per whole Band (proposed, R-UI4).
pub const BAR_PX_PER_BAND: f64 = 2.0;
/// A holding's tallest bar, tactical pixels: Band V.
pub const BAR_MAX: u8 = 10;

/// Hulls whose positions fall in one square this many tactical pixels wide
/// are in one place (proposed, R-UI4). The square grows as the view zooms
/// out, so a galaxy reads as clusters and a fight as single hulls.
pub fn stack_px(lod: Lod) -> i64 {
    match lod {
        Lod::Galaxy => 8,
        Lod::Sector => 5,
        Lod::System => 3,
    }
}

/// A marker's side, tactical pixels: larger for more hulls.
pub fn marker_px(count: usize) -> i64 {
    match count {
        0..=1 => 3,
        2..=9 => 5,
        10..=99 => 7,
        _ => 9,
    }
}

/// **A holding's bar height** from its Band reading on the cost ladder:
/// [`BAR_PX_PER_BAND`] per Band, at least one pixel for anything held — a
/// trace below Band Empty reads negative — and none for nothing.
pub fn bar_height(band: Option<f64>) -> u8 {
    match band {
        None => 0,
        Some(b) => (b * BAR_PX_PER_BAND).round().clamp(1.0, BAR_MAX as f64) as u8,
    }
}

/// Something the viewer can select.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    World(u32),
    Hull(u64),
}

/// What a hull's glyph is built from: hull code, Design class, armed with
/// beams, armed with tubes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphKey {
    pub hull: usize,
    pub design: usize,
    pub beams: bool,
    pub tubes: bool,
}

// Most ops in a plan are hulls, so boxing the large variant would add an
// allocation per hull and save no memory.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    World {
        id: u32,
        /// Screen position, pixels.
        s: [f64; 2],
        /// The tactical pixel `s` falls in.
        at: [i64; 2],
        r: f64,
        /// The owning seat's color, or the dim world color.
        color: Rgb,
        /// A homeworld's ring.
        ring: Option<Rgb>,
        /// Every seat's holding here: its color and one bar height per
        /// material on the books ([`bar_height`]). Empty at the galaxy level.
        holdings: Vec<(Rgb, [u8; 8])>,
        selected: bool,
    },
    Hull {
        id: u64,
        s: [f64; 2],
        /// The tactical pixel the hull stands in, where its glyph is drawn.
        at: [i64; 2],
        /// Hulls this glyph stands for: its stack's size on the stack's
        /// lowest id, 0 on the others, which are not drawn.
        count: u32,
        /// The ids of the hulls this glyph stands for, on the drawn one.
        members: Vec<u64>,
        /// On the glyph drawn on top in its place, the ids of every hull in
        /// the place when there is more than one; empty otherwise. Its count
        /// is drawn beside that glyph.
        place: Vec<u64>,
        /// Drawn as a marker, a square in the seat's color sized by count: one
        /// owner's hulls in one place at the galaxy level.
        marker: bool,
        /// **Drawn as one dim pixel**: a scout, or an unladen, unarmed hull of
        /// the Systems or Contact family — traffic, kept out of the way. An
        /// armed hull is never quiet: a picket of the Contact family is what a
        /// fight is made of (ruling 18).
        quiet: bool,
        /// Where it is drawn among overlapping glyphs: higher on top. See
        /// [`display_order`].
        order: u64,
        key: GlyphKey,
        /// The seat's color, or the wreck color.
        outline: Rgb,
        /// The outline's color, dimmed toward the ground.
        inner: Rgb,
        /// The Doctrine role's accent.
        core: Rgb,
        /// The role's 3×3 mark ([`role_mark`]), drawn at the glyph's pip.
        mark: Option<[bool; 9]>,
        marks: Rgb,
        /// The drive vector's far end and color.
        vector: Option<([i64; 2], Rgb)>,
        /// The destination's tactical pixel and the route's faint color.
        route: Option<([i64; 2], Rgb)>,
        /// Share of structure lost, 0 to 1.
        damage: f64,
        hit: bool,
        laden: bool,
        /// What the hold carries, kt, by material and then people
        /// ([`PEOPLE`]): the glyph's fill is striped in these shares.
        cargo: [f64; 9],
        selected: bool,
    },
}

/// Everything a frame of either mode is drawn from.
pub struct Scene<'a> {
    pub replay: &'a Replay,
    pub view: &'a View,
    pub camera: &'a Camera,
    pub palette: &'a Palette,
    pub selected: Option<Pick>,
}

/// A screen point to the tactical pixel it falls in.
pub fn to_raster(s: [f64; 2], pixel: f64) -> [i64; 2] {
    [(s[0] / pixel).floor() as i64, (s[1] / pixel).floor() as i64]
}

/// **Which of two overlapping glyphs is drawn on top** (proposed, R-UI4):
/// a live hull over a wreck and a prominent hull over a [quiet](Op::Hull)
/// one; then by role, the fighting roles highest; then the smaller hull over
/// the larger, so both stay visible; then the hull farther from its seat's
/// homeworld, the frontier, over the one nearer home.
pub fn display_order(wrecked: bool, quiet: bool, role: &str, size: Size, from_home_ly: f64) -> u64 {
    let tier = if wrecked {
        0
    } else if quiet {
        1
    } else {
        2
    };
    let role = match role {
        "Scout" => 1,
        "Miner" => 2,
        "Freighter" => 3,
        "Colonizer" => 4,
        "Sentry" => 5,
        "Picket" => 6,
        _ => 0,
    };
    let size = match size {
        Size::General => 0,
        Size::Medium => 1,
        Size::Limited => 2,
    };
    let distance = (from_home_ly.max(0.0) * 16.0).min((1u64 << 48) as f64 - 1.0) as u64;
    (tier << 60) | (role << 56) | (size << 52) | distance
}

/// One [`Op`] per world and one per hull in the view, each at its position.
pub fn plan(s: &Scene) -> Vec<Op> {
    let (r, f, pal) = (s.replay, &s.replay.frames[s.view.frame], s.palette);
    let screen = |p: [f64; 3]| s.camera.project(p);
    let lod = s.camera.lod();
    let base = match lod {
        Lod::Galaxy => 0.0,
        Lod::Sector => 1.0,
        Lod::System => 2.0,
    };
    let mut ops = Vec::with_capacity(r.planets.len() + s.view.hulls.len());
    for p in &r.planets {
        let owner = f.owner.get(p.id as usize).copied().flatten();
        let color = owner.map_or(pal.roles.world_dim, |o| pal.seat(o));
        let sp = screen(p.pos);
        let holdings = if lod == Lod::Galaxy {
            Vec::new()
        } else {
            let first = f.holdings.partition_point(|h| h.planet < p.id);
            f.holdings[first..]
                .iter()
                .take_while(|h| h.planet == p.id)
                .map(|h| (pal.seat(h.seat), h.bands.map(bar_height)))
                .collect()
        };
        ops.push(Op::World {
            id: p.id,
            s: sp,
            at: to_raster(sp, PIXEL),
            // A colony is a disc in its seat's color, a pixel wider than an
            // unowned world at every level.
            r: base + if owner.is_some() { 1.0 } else { 0.0 } + if p.home { 1.0 } else { 0.0 },
            color,
            ring: if p.home { Some(color) } else { None },
            holdings,
            selected: s.selected == Some(Pick::World(p.id)),
        });
    }
    let first_hull = ops.len();
    for h in &s.view.hulls {
        let row = &h.row;
        let sp = screen(row.pos);
        let here = to_raster(sp, PIXEL);
        let key =
            GlyphKey { hull: h.hull.hull, design: h.hull.design, beams: h.hull.beams > 0, tubes: h.hull.tubes > 0 };
        let role = r.kinds.get(row.kind).map_or("", String::as_str);
        let code = r.hulls.get(h.hull.hull).map_or("", String::as_str);
        let armed = h.hull.beams > 0 || h.hull.tubes > 0;
        let quiet = !row.wrecked
            && (role == "Scout"
                || (!h.laden && !armed && matches!(glyph::family(code), Family::Systems | Family::Contact)));
        let seat = pal.seat(h.hull.owner);
        let (outline, core, marks) = if row.wrecked {
            (pal.status(Status::Wreck), pal.roles.grid, pal.roles.grid)
        } else {
            (seat, pal.role(role), pal.roles.text_bright)
        };
        let inner = mix(outline, pal.roles.ground, pal.fill_dim());
        let home = r.seats.get(h.hull.owner).and_then(|st| r.planets.get(st.home as usize)).map(|p| p.pos);
        let from_home = home.map_or(0.0, |o| {
            let d = [row.pos[0] - o[0], row.pos[1] - o[1], row.pos[2] - o[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        });
        let route = match (row.in_flight && !row.wrecked, row.dest.and_then(|d| r.planets.get(d as usize))) {
            (true, Some(to)) => Some((to_raster(screen(to.pos), PIXEL), mix(seat, pal.roles.ground, ROUTE_DIM))),
            _ => None,
        };
        let mut cargo = [0.0; 9];
        cargo[..8].copy_from_slice(&row.mix);
        cargo[PEOPLE] = row.settlers;
        ops.push(Op::Hull {
            id: h.hull.id,
            s: sp,
            at: here,
            count: 1,
            members: Vec::new(),
            place: Vec::new(),
            marker: false,
            quiet,
            order: display_order(row.wrecked, quiet, role, glyph::size(code), from_home),
            key,
            outline,
            inner,
            core,
            mark: role_mark(role),
            marks,
            vector: if quiet { None } else { vector(s, h.row, here) },
            route,
            damage: row.damage.clamp(0.0, 1.0),
            hit: h.hit,
            laden: h.laden,
            cargo,
            selected: s.selected == Some(Pick::Hull(h.hull.id)),
        });
    }
    stack(&mut ops[first_hull..], s);
    ops
}

/// **Stacks** (`docs/Hyades_interface.md` §6.4): hulls in one place with one
/// owner, Design, role, wreck state and quietness are one glyph with a count,
/// carrying the worst damage, any hit, the sum of their cargo and any
/// selection, drawn where its lowest-id hull stands. At the galaxy level a
/// place is wider and a stack is every hull of one owner there, drawn as a
/// marker sized by count. Glyphs in one place overlap in [`display_order`];
/// the one on top carries the place's count.
fn stack(ops: &mut [Op], s: &Scene) {
    let lod = s.camera.lod();
    let cell_px = stack_px(lod);
    // At the galaxy level a group is one owner's armed or unarmed hulls in a
    // place; below it, one owner's hulls of one Design, role, wreck state and
    // quietness.
    type Group = (i64, i64, usize, bool, Option<(GlyphKey, usize, bool, bool)>);
    let mut groups: BTreeMap<Group, Vec<usize>> = BTreeMap::new();
    for (i, h) in s.view.hulls.iter().enumerate() {
        let Op::Hull { at, key, quiet, .. } = ops[i] else { continue };
        let cell = (at[0].div_euclid(cell_px), at[1].div_euclid(cell_px));
        let kind = (lod != Lod::Galaxy).then_some((key, h.row.kind, h.row.wrecked, quiet));
        groups.entry((cell.0, cell.1, h.hull.owner, key.beams || key.tubes, kind)).or_default().push(i);
    }
    let mut places: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for ((cx, cy, ..), mut members) in groups {
        members.sort_unstable();
        let (mut damage, mut hit, mut laden, mut selected, mut quiet, mut order) =
            (0.0f64, false, false, false, true, 0);
        let mut cargo = [0.0; 9];
        let mut ids = Vec::with_capacity(members.len());
        for &m in &members {
            if let Op::Hull {
                id,
                damage: d,
                hit: h,
                laden: l,
                selected: sel,
                count,
                quiet: q,
                order: o,
                cargo: c,
                ..
            } = &mut ops[m]
            {
                (damage, hit, laden, selected) = (damage.max(*d), hit | *h, laden | *l, selected | *sel);
                (quiet, order) = (quiet & *q, order.max(*o));
                for k in 0..9 {
                    cargo[k] += c[k];
                }
                *count = 0;
                ids.push(*id);
            }
        }
        let lead = members[0];
        if let Op::Hull {
            count,
            members: mem,
            marker,
            damage: d,
            hit: h,
            laden: l,
            selected: sel,
            quiet: q,
            order: o,
            cargo: c,
            ..
        } = &mut ops[lead]
        {
            (*count, *marker, *d, *h, *l, *sel) =
                (members.len() as u32, lod == Lod::Galaxy, damage, hit, laden, selected);
            (*q, *o, *c) = (quiet, order, cargo);
            *mem = ids;
        }
        places.entry((cx, cy)).or_default().push(lead);
    }
    for leads in places.into_values() {
        // The glyph drawn last, as [`drawing_order`] sorts them.
        let top = *leads.iter().max_by_key(|&&i| (op_order(&ops[i]), op_id(&ops[i]))).unwrap();
        let all: Vec<u64> = leads
            .iter()
            .flat_map(|&i| match &ops[i] {
                Op::Hull { members, .. } => members.clone(),
                Op::World { .. } => Vec::new(),
            })
            .collect();
        if all.len() > 1 {
            if let Op::Hull { place, .. } = &mut ops[top] {
                *place = all;
            }
        }
    }
}

fn op_order(op: &Op) -> u64 {
    match op {
        Op::Hull { order, .. } => *order,
        Op::World { .. } => 0,
    }
}

fn op_id(op: &Op) -> u64 {
    match op {
        Op::Hull { id, .. } => *id,
        Op::World { id, .. } => *id as u64,
    }
}

/// The drive vector: along the velocity, or toward the destination when the
/// hull is burning from rest; longer the faster it goes. Colored by the drive
/// — burning, burning hot, braking — and dim while it coasts.
fn vector(s: &Scene, row: crate::replay::Row, here: [i64; 2]) -> Option<([i64; 2], Rgb)> {
    if !row.in_flight {
        return None;
    }
    let speed = row.vel[0].hypot(row.vel[1]);
    let dir = if speed > 0.0 {
        [row.vel[0] / speed, row.vel[1] / speed]
    } else {
        let to = s.replay.planets.get(row.dest? as usize)?.pos;
        let d = [to[0] - row.pos[0], to[1] - row.pos[1]];
        let n = d[0].hypot(d[1]);
        if n == 0.0 || row.burn == 0 {
            return None;
        }
        [d[0] / n, d[1] / n]
    };
    let color = match row.burn {
        b if b > 0 && row.accel >= HOT_ACCEL => s.palette.status(Status::DriveHot),
        b if b > 0 => s.palette.status(Status::Drive),
        b if b < 0 => s.palette.status(Status::Braking),
        _ => s.palette.roles.grid,
    };
    let len = 3.0 + 6.0 * speed.min(1.0);
    Some(([here[0] + (dir[0] * len).round() as i64, here[1] - (dir[1] * len).round() as i64], color))
}

/// The drawn hull ops (`count ≥ 1`), bottom first in [`display_order`], ties
/// by id.
pub fn drawing_order(ops: &[Op]) -> Vec<usize> {
    let mut drawn: Vec<usize> = (0..ops.len()).filter(|&i| matches!(ops[i], Op::Hull { count: 1.., .. })).collect();
    drawn.sort_by_key(|&i| (op_order(&ops[i]), op_id(&ops[i])));
    drawn
}

/// Draws the hex grid, the routes, the worlds with their holdings, and then
/// the hulls in [`display_order`], with glyphs from `cache` (kept between
/// frames).
pub fn paint_with(s: &Scene, ops: &[Op], r: &mut Raster, cache: &mut GlyphCache) {
    let pal = s.palette;
    r.clear(pal.roles.ground);
    hex_grid(s, r);
    for op in ops {
        if let Op::Hull { at, route: Some((to, c)), .. } = op {
            r.line(at[0], at[1], to[0], to[1], *c);
        }
    }
    for op in ops {
        if let Op::World { at, r: rad, color, ring, holdings, selected, .. } = op {
            r.disc(at[0], at[1], *rad, *color);
            if let Some(c) = ring {
                r.circle(at[0], at[1], rad + 2.0, *c);
            }
            bars(r, pal, *at, *rad, holdings);
            if *selected {
                brackets(r, *at, *rad as i64 + 3, pal.status(Status::Selected));
            }
        }
    }
    for i in drawing_order(ops) {
        let Op::Hull {
            at,
            count,
            place,
            key,
            outline,
            inner,
            core,
            mark,
            marks,
            vector,
            damage,
            hit,
            laden,
            cargo,
            selected,
            marker,
            quiet,
            ..
        } = &ops[i]
        else {
            continue;
        };
        if *quiet {
            r.set(at[0], at[1], *inner);
            if *hit {
                r.circle(at[0], at[1], 3.0, pal.status(Status::Hit));
            }
            if *selected {
                brackets(r, *at, 3, pal.status(Status::Selected));
            }
            continue;
        }
        let armed = key.beams || key.tubes;
        if *marker {
            // Solid for armed hulls, hollow for the rest, as a glyph's body.
            let side = marker_px(*count as usize);
            let h = side / 2;
            for y in -h..=h {
                for x in -h..=h {
                    let edge = x.abs() == h || y.abs() == h;
                    r.set(at[0] + x, at[1] + y, if edge || armed { *outline } else { *inner });
                }
            }
            if *hit {
                r.circle(at[0], at[1], (h + 3) as f64, pal.status(Status::Hit));
            }
            if *selected {
                brackets(r, *at, h + 3, pal.status(Status::Selected));
            }
            continue;
        }
        if let Some((end, c)) = vector {
            r.line(at[0], at[1], end[0], end[1], *c);
        }
        let g = cache.get(s.replay, *key);
        let (gw, ax, ay) = (g.w, g.ax, g.ay);
        r.stamp(&g.outline, gw, ax, ay, at[0], at[1], *outline);
        // The body: solid when armed, hollow otherwise — armed and unarmed
        // read apart before anything else. A hold's cargo stripes the whole
        // body of an unarmed hull and the bottom two rows of an armed one.
        let solid = armed;
        r.stamp(&g.inner, gw, ax, ay, at[0], at[1], if solid { *outline } else { *inner });
        if *laden {
            let last = (0..g.h).rev().find(|&y| (0..g.w).any(|x| g.inner[y * g.w + x])).unwrap_or(0);
            stripes(r, pal, g, *at, cargo, if solid { last.saturating_sub(1) } else { 0 });
        }
        if let Some(m) = mark {
            let (px, py) = (at[0] + g.pip.0 as i64 - ax as i64, at[1] + g.pip.1 as i64 - ay as i64);
            r.stamp(m, 3, 1, 1, px, py, if solid { pal.roles.ground } else { *core });
        }
        r.stamp(&g.marks, gw, ax, ay, at[0], at[1], *marks);
        let half = ax as i64 - 2;
        if *damage > 0.0 {
            let n = (damage * (2 * half + 1) as f64).ceil() as i64;
            for k in 0..n {
                r.set(at[0] - half - 3, at[1] + half - k, pal.status(Status::Damage));
            }
        }
        if *hit {
            r.circle(at[0], at[1], (half + 4) as f64, pal.status(Status::Hit));
        }
        if *selected {
            brackets(r, *at, half + 5, pal.status(Status::Selected));
        }
        if place.len() > 1 {
            r.number(at[0] + half + 4, at[1] - 2, place.len() as u32, pal.roles.text_bright);
        }
    }
}

/// **A hold's composition**: the glyph's body from row `from` down, in
/// vertical stripes, one per material carried, each as wide as its share of
/// the cargo's mass, in book order and then people. A hold whose composition
/// the replay does not carry is filled in the laden color.
fn stripes(r: &mut Raster, pal: &Palette, g: &Glyph, at: [i64; 2], cargo: &[f64; 9], from: usize) {
    let total: f64 = cargo.iter().sum();
    let cols: Vec<usize> = (0..g.w).filter(|&x| (from..g.h).any(|y| g.inner[y * g.w + x])).collect();
    for (k, &x) in cols.iter().enumerate() {
        let color = if total > 0.0 {
            let f = (k as f64 + 0.5) / cols.len() as f64 * total;
            let mut acc = 0.0;
            let m = (0..9).find(|&m| {
                acc += cargo[m];
                cargo[m] > 0.0 && f <= acc
            });
            pal.material(m.unwrap_or_else(|| (0..9).rev().find(|&m| cargo[m] > 0.0).unwrap()))
        } else {
            pal.status(Status::Laden)
        };
        for y in from..g.h {
            if g.inner[y * g.w + x] {
                r.set(at[0] + x as i64 - g.ax as i64, at[1] + y as i64 - g.ay as i64, color);
            }
        }
    }
}

/// **Holdings**: under a world, one group per seat holding there — a base in
/// the seat's color under one vertical bar per material on the books, in book
/// order, each as tall as [`bar_height`] reads its Band. Under, because a
/// hull's count and damage are drawn either side of the place.
fn bars(r: &mut Raster, pal: &Palette, at: [i64; 2], rad: f64, holdings: &[(Rgb, [u8; 8])]) {
    let x0 = at[0] - 4;
    let base = bar_base(at, rad);
    for (g, (seat, heights)) in holdings.iter().enumerate() {
        let x = x0 + 10 * g as i64;
        r.line(x, base, x + 7, base, *seat);
        for (m, &h) in heights.iter().enumerate() {
            for k in 1..=h as i64 {
                r.set(x + m as i64, base - k, pal.material(m));
            }
        }
    }
}

/// The row a world's holding bars stand on: clear of the world and of a
/// General glyph's tag row beneath a hull parked there.
pub fn bar_base(at: [i64; 2], rad: f64) -> i64 {
    at[1] + rad as i64 + 4 + BAR_MAX as i64
}

/// Four corner brackets `d` pixels out from `at`, in the selection color.
fn brackets(r: &mut Raster, at: [i64; 2], d: i64, c: Rgb) {
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (x, y) = (at[0] + sx * d, at[1] + sy * d);
        r.line(x, y, x - sx * 2, y, c);
        r.line(x, y, x, y - sy * 2, c);
    }
}

/// The hex a galaxy point stands in, as lattice coordinates `(i, j)`: its
/// center is `hex_origin + i·a + j·b` with `a = w·(cos 30°, sin 30°)`,
/// `b = w·(0, 1)` and `w = √3 · hex_side_ly` (flat-top hexes, galaxy §2).
pub fn hex_of(replay: &Replay, p: [f64; 3]) -> (i64, i64) {
    let side = replay.meta.hex_side_ly;
    let o = replay.meta.hex_origin;
    let (x, y) = (p[0] - o[0], p[1] - o[1]);
    // Axial coordinates of a flat-top hex: q = i, r = j.
    let q = (2.0 / 3.0 * x) / side;
    let r = (-x / 3.0 + 3f64.sqrt() / 3.0 * y) / side;
    let cube = [q, r, -q - r];
    let mut rd = cube.map(f64::round);
    let diff = [0, 1, 2].map(|k| (rd[k] - cube[k]).abs());
    if diff[0] > diff[1] && diff[0] > diff[2] {
        rd[0] = -rd[1] - rd[2];
    } else if diff[1] > diff[2] {
        rd[1] = -rd[0] - rd[2];
    }
    (rd[0] as i64, rd[1] as i64)
}

/// The center of hex `(i, j)`, ly.
pub fn hex_center(replay: &Replay, (i, j): (i64, i64)) -> [f64; 2] {
    let w = 3f64.sqrt() * replay.meta.hex_side_ly;
    let o = replay.meta.hex_origin;
    [o[0] + i as f64 * w * 3f64.sqrt() / 2.0, o[1] + i as f64 * w / 2.0 + j as f64 * w]
}

/// **The hexes worth drawing**: those a world or a hull stands in at this
/// instant. Empty space draws no grid.
pub fn active_hexes(s: &Scene) -> std::collections::BTreeSet<(i64, i64)> {
    let worlds = s.replay.planets.iter().map(|p| p.pos);
    let hulls = s.view.hulls.iter().map(|h| h.row.pos);
    worlds.chain(hulls).map(|p| hex_of(s.replay, p)).collect()
}

/// The command view's active hexes (flat-top, a side of `hex_side_ly`, one
/// centered on `hex_origin`), all six edges each.
fn hex_grid(s: &Scene, r: &mut Raster) {
    let side = s.replay.meta.hex_side_ly;
    if side <= 0.0 || 3f64.sqrt() * side * s.camera.scale / PIXEL < MIN_HEX_PX {
        return;
    }
    for h in active_hexes(s) {
        let c = hex_center(s.replay, h);
        let vertex = |k: i32| {
            let a = (60.0 * k as f64).to_radians();
            to_raster(s.camera.project([c[0] + side * a.cos(), c[1] + side * a.sin(), 0.0]), PIXEL)
        };
        for k in 0..6 {
            let (p, q) = (vertex(k), vertex(k + 1));
            r.line(p[0], p[1], q[0], q[1], s.palette.roles.hex);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::tests::TINY;

    fn paint(s: &Scene, ops: &[Op], r: &mut Raster) {
        paint_with(s, ops, r, &mut GlyphCache::default());
    }

    fn render(s: &Scene, r: &mut Raster) {
        paint(s, &plan(s), r);
    }

    struct Fixture {
        replay: Replay,
        camera: Camera,
        palette: Palette,
    }

    fn fixture() -> Fixture {
        let replay = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(replay.planets.iter().map(|p| p.pos), 20.0);
        Fixture { replay, camera, palette: Palette::default() }
    }

    fn raster(c: &Camera) -> Raster {
        Raster::new((c.width / PIXEL).ceil() as usize, (c.height / PIXEL).ceil() as usize)
    }

    #[test]
    fn every_world_and_every_hull_is_planned_once_at_its_position() {
        let f = fixture();
        for t in [0.0, 2.5, 7.0, 10.0] {
            let view = f.replay.view_at(t);
            let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
            let ops = plan(&s);
            let mut worlds: Vec<u32> = Vec::new();
            let mut hulls: Vec<u64> = Vec::new();
            for op in &ops {
                match op {
                    Op::World { id, at, .. } => {
                        assert_eq!(*at, to_raster(f.camera.project(f.replay.planets[*id as usize].pos), PIXEL));
                        worlds.push(*id);
                    }
                    Op::Hull { id, at, .. } => {
                        let h = view.hulls.iter().find(|h| h.hull.id == *id).expect("a planned hull is in the view");
                        assert_eq!(*at, to_raster(f.camera.project(h.row.pos), PIXEL));
                        hulls.push(*id);
                    }
                }
            }
            assert_eq!(worlds, (0..f.replay.planets.len() as u32).collect::<Vec<_>>(), "t {t}");
            let mut want: Vec<u64> = view.hulls.iter().map(|h| h.hull.id).collect();
            want.sort();
            hulls.sort();
            assert_eq!(hulls, want, "t {t}");
        }
    }

    #[test]
    fn every_planned_entity_leaves_pixels_at_its_position() {
        let f = fixture();
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let mut r = raster(&f.camera);
        let ops = plan(&s);
        paint(&s, &ops, &mut r);
        let ground = f.palette.roles.ground.to_ints();
        for op in &ops {
            let at = match op {
                Op::World { at, .. } | Op::Hull { at, .. } => *at,
            };
            assert_ne!(r.get(at[0], at[1]), Some(ground), "{op:?}");
        }
    }

    #[test]
    fn a_hull_wears_its_seat_on_its_body_and_its_role_at_its_center() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let freighter = ops.iter().find(|o| matches!(o, Op::Hull { id: 5, .. })).unwrap();
        let Op::Hull { outline, inner, core, mark, key, laden, .. } = freighter else { unreachable!() };
        assert_eq!(*outline, f.palette.seat(0));
        assert_eq!(*inner, mix(f.palette.seat(0), f.palette.roles.ground, f.palette.fill_dim()));
        assert_eq!(*core, f.palette.role("Freighter"));
        assert_eq!(*mark, role_mark("Freighter"));
        assert_eq!(f.replay.designs[key.design], "Ford");
        assert_eq!(f.replay.hulls[key.hull], "MSV");
        assert!(*laden);
    }

    /// One hull of `id` at the frame of `t`, zoomed so a glyph is drawn whole,
    /// alone in its view, painted; and where it stands.
    fn alone(f: &Fixture, t: f64, id: u64, tweak: impl Fn(&mut crate::replay::HullView)) -> (Raster, [i64; 2]) {
        let mut view = f.replay.view_at(t);
        view.hulls.retain(|h| h.hull.id == id);
        tweak(&mut view.hulls[0]);
        let mut cam = f.camera;
        cam.scale = crate::camera::SYSTEM_PX_PER_LY * 2.0;
        cam.center = [view.hulls[0].row.pos[0], view.hulls[0].row.pos[1]];
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let mut r = raster(&cam);
        render(&s, &mut r);
        (r, to_raster(cam.project(view.hulls[0].row.pos), PIXEL))
    }

    /// Hull 6 unarmed and moved onto an LOU, the Offensive family's Limited
    /// hull, so it is drawn as a glyph and not as quiet traffic.
    fn unarmed_lou(h: &mut crate::replay::HullView) {
        (h.hull.beams, h.hull.hull) = (0, 7);
    }

    #[test]
    fn an_armed_body_is_solid_and_an_unarmed_body_hollow() {
        let f = fixture();
        let seat = f.palette.seat(1).to_ints();
        let hollow = mix(f.palette.seat(1), f.palette.roles.ground, f.palette.fill_dim()).to_ints();
        // Hull 6, a picket on an LCV with a beam mount, at t 0: a diamond.
        let (armed, at) = alone(&f, 0.0, 6, |_| {});
        assert_eq!(armed.get(at[0] + 2, at[1]), Some(seat), "solid in the seat's color");
        // Unarmed, it would be quiet traffic on its Contact hull; on an
        // Offensive hull (an LOU triangle) it is a glyph, hollow.
        let (bare, at) = alone(&f, 0.0, 6, unarmed_lou);
        assert_eq!(bare.get(at[0] + 2, at[1] + 2), Some(hollow), "hollow: the seat's color dimmed");
    }

    #[test]
    fn an_armed_hull_carrying_rounds_stays_solid_with_its_cargo_along_the_bottom() {
        let f = fixture();
        let (r, at) = alone(&f, 0.0, 6, |h| {
            h.laden = true;
            h.row.mix[7] = 0.2;
        });
        let rounds = f.palette.material(7).to_ints();
        assert_eq!(r.get(at[0] + 2, at[1]), Some(f.palette.seat(1).to_ints()), "the body stays solid");
        assert_eq!(r.get(at[0], at[1] + 3), Some(rounds), "the diamond's bottom interior row is the cargo");
        assert_ne!(r.get(at[0] + 2, at[1] + 1), Some(rounds), "and only the bottom two rows");
    }

    #[test]
    fn the_role_is_a_mark_at_the_center_and_reads_on_either_body() {
        let f = fixture();
        // An armed picket: an X in the ground color on its solid body.
        let (r, at) = alone(&f, 0.0, 6, |_| {});
        let ground = f.palette.roles.ground.to_ints();
        for (dx, dy) in [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)] {
            assert_eq!(r.get(at[0] + dx, at[1] + dy), Some(ground), "the X at {dx},{dy}");
        }
        assert_eq!(r.get(at[0], at[1] - 1), Some(f.palette.seat(1).to_ints()), "and the body between its arms");
        // Unarmed, the same mark in the role's accent on the hollow body —
        // a pixel low on a triangle, whose interior sits low.
        let (r, at) = alone(&f, 0.0, 6, unarmed_lou);
        assert_eq!(r.get(at[0], at[1] + 1), Some(f.palette.role("Picket").to_ints()));
        assert_eq!(r.get(at[0] - 1, at[1]), Some(f.palette.role("Picket").to_ints()));
    }

    #[test]
    fn at_the_galaxy_level_armed_and_unarmed_hulls_are_separate_markers() {
        let f = fixture();
        let mut view = crowd(&f, 2);
        let mut armed = view.hulls[0];
        (armed.hull.id, armed.row.id, armed.hull.beams) = (400, 400, 1);
        view.hulls.push(armed);
        let cam = galaxy_camera(&f);
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let marker_of = |id| {
            ops.iter().find_map(|o| match o {
                Op::Hull { members, count: 1.., marker: true, .. } if members.contains(&id) => Some(members.clone()),
                _ => None,
            })
        };
        assert_eq!(marker_of(400), Some(vec![400]), "the armed hull is its own marker");
        assert!(!marker_of(5).unwrap().contains(&400), "apart from the unarmed Fords");
    }

    #[test]
    fn the_drive_vector_reads_burn_and_points_where_the_hull_goes() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        // Hull 5 burns at rest toward world 2, which lies up and to the right.
        let Some(Op::Hull { at, vector: Some((end, color)), .. }) =
            ops.iter().find(|o| matches!(o, Op::Hull { id: 5, .. }))
        else {
            panic!("a burning hull has a vector")
        };
        assert_eq!(*color, f.palette.status(Status::Drive));
        assert!(end[0] > at[0] && end[1] < at[1], "{at:?} → {end:?}");
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        assert!(
            matches!(ops.iter().find(|o| matches!(o, Op::Hull { id: 9, .. })), Some(Op::Hull { vector: None, .. })),
            "a scout burns quietly: no vector"
        );
        // Hull 5 has stopped: no drive, no vector.
        assert!(matches!(
            ops.iter().find(|o| matches!(o, Op::Hull { id: 5, .. })),
            Some(Op::Hull { vector: None, .. })
        ));
    }

    #[test]
    fn a_wreck_is_drawn_in_the_wreck_color_and_a_hit_rings() {
        let f = fixture();
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let Some(Op::Hull { outline, hit, damage, .. }) = ops.iter().find(|o| matches!(o, Op::Hull { id: 6, .. }))
        else {
            panic!()
        };
        assert_eq!(*outline, f.palette.status(Status::Wreck));
        assert!(*hit);
        assert_eq!(*damage, 1.0);
        let mut r = raster(&f.camera);
        paint(&s, &ops, &mut r);
        assert!(r.count(f.palette.status(Status::Hit)) > 0, "the hit ring is drawn");
        assert!(r.count(f.palette.status(Status::Damage)) > 0, "the damage bar is drawn");
    }

    #[test]
    fn the_hex_grid_is_drawn_when_hexes_are_wide_enough_and_not_when_they_are_a_fill() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let hex = f.palette.roles.hex;
        let mut cam = f.camera;
        cam.scale = 1.0;
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let mut r = raster(&cam);
        paint(&s, &[], &mut r);
        assert!(r.count(hex) > 0, "a 121-ly hex is 60 tactical pixels across");
        cam.scale = MIN_HEX_PX / 121.0;
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        paint(&s, &[], &mut r);
        assert_eq!(r.count(hex), 0);
    }

    /// A view with copies of hull 5 at its place: `alike` more of the same
    /// Design, and one built to another.
    fn crowd(f: &Fixture, alike: u64) -> View {
        let mut view = f.replay.view_at(0.0);
        let five = *view.hulls.iter().find(|h| h.hull.id == 5).unwrap();
        for k in 0..alike {
            let mut c = five;
            (c.hull.id, c.row.id, c.row.damage) = (100 + k, 100 + k, 0.1 * k as f64);
            view.hulls.push(c);
        }
        let mut other = five;
        (other.hull.id, other.hull.design) = (200, 4);
        view.hulls.push(other);
        view
    }

    #[test]
    fn every_hull_is_drawn_or_counted_in_a_drawn_stack() {
        let f = fixture();
        let view = crowd(&f, 4);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let total: u32 = ops.iter().map(|o| if let Op::Hull { count, .. } = o { *count } else { 0 }).sum();
        assert_eq!(total as usize, view.hulls.len());
        let lead = |id| {
            ops.iter().find_map(|o| match o {
                Op::Hull { id: i, count, damage, cargo, .. } if *i == id => Some((*count, *damage, *cargo)),
                _ => None,
            })
        };
        let (n, damage, cargo) = lead(5).unwrap();
        assert_eq!(n, 5, "five alike Fords are one glyph");
        assert!((damage - 0.3).abs() < 1e-12, "carrying the worst damage among them");
        assert_eq!(cargo[0], 15.0, "and the cargo of all five");
        assert_eq!(lead(101).unwrap().0, 0, "a stacked hull is not drawn on its own");
        assert_eq!(lead(200).unwrap().0, 1, "the unalike hull is its own glyph");
    }

    #[test]
    fn every_glyph_is_drawn_where_its_hull_stands_and_overlaps_in_display_order() {
        let f = fixture();
        let mut view = f.replay.view_at(0.0);
        let five = *view.hulls.iter().find(|h| h.hull.id == 5).unwrap();
        for d in 0..7 {
            let mut c = five;
            (c.hull.id, c.row.id, c.hull.design) = (300 + d as u64, 300 + d as u64, d);
            view.hulls.push(c);
        }
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        for op in &ops {
            if let Op::Hull { id, at, .. } = op {
                let h = view.hulls.iter().find(|h| h.hull.id == *id).unwrap();
                assert_eq!(*at, to_raster(f.camera.project(h.row.pos), PIXEL), "hull {id} where it stands");
            }
        }
        let here = to_raster(f.camera.project(five.row.pos), PIXEL);
        let order: Vec<usize> = drawing_order(&ops)
            .into_iter()
            .filter(|&i| matches!(&ops[i], Op::Hull { at, .. } if *at == here))
            .collect();
        assert_eq!(order.len(), 8, "the Ford and seven other Designs, one glyph each");
        let orders: Vec<u64> = order.iter().map(|&i| op_order(&ops[i])).collect();
        assert!(orders.windows(2).all(|w| w[0] <= w[1]), "bottom first");
        let top = &ops[*order.last().unwrap()];
        let Op::Hull { place, .. } = top else { unreachable!() };
        assert_eq!(place.len(), 8, "the glyph on top carries the place's count");
        let tops = ops.iter().filter(|o| matches!(o, Op::Hull { place, .. } if !place.is_empty())).count();
        assert_eq!(tops, 1, "one count per place");
    }

    #[test]
    fn the_display_order_puts_the_fight_and_the_frontier_on_top() {
        let o = |w, q, role, size, d| display_order(w, q, role, size, d);
        assert!(o(false, false, "Picket", Size::Medium, 1.0) > o(false, false, "Freighter", Size::Medium, 1.0));
        assert!(o(false, false, "Freighter", Size::Medium, 1.0) > o(false, false, "Miner", Size::Medium, 1.0));
        assert!(o(false, false, "Miner", Size::General, 0.0) > o(false, true, "Picket", Size::Limited, 99.0));
        assert!(o(false, true, "Scout", Size::General, 0.0) > o(true, false, "Picket", Size::Limited, 99.0));
        assert!(o(false, false, "Freighter", Size::Limited, 1.0) > o(false, false, "Freighter", Size::General, 1.0));
        assert!(o(false, false, "Freighter", Size::Medium, 30.0) > o(false, false, "Freighter", Size::Medium, 20.0));
    }

    #[test]
    fn a_scout_and_unladen_traffic_are_one_dim_pixel_and_a_laden_hull_is_a_glyph() {
        let f = fixture();
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let quiet = |id| ops.iter().any(|o| matches!(o, Op::Hull { id: i, quiet: true, .. } if *i == id));
        assert!(quiet(9), "the scout");
        assert!(!quiet(5), "the laden freighter");
        assert!(!quiet(6), "a wreck is not traffic");
        let view0 = f.replay.view_at(0.0);
        let s0 = Scene { replay: &f.replay, view: &view0, camera: &f.camera, palette: &f.palette, selected: None };
        assert!(
            plan(&s0).iter().any(|o| matches!(o, Op::Hull { id: 6, quiet: false, .. })),
            "an armed Contact picket, unladen, is not traffic"
        );
        let mut view0 = f.replay.view_at(0.0);
        view0.hulls.iter_mut().find(|h| h.hull.id == 5).unwrap().laden = false;
        let s = Scene { replay: &f.replay, view: &view0, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        assert!(ops.iter().any(|o| matches!(o, Op::Hull { id: 5, quiet: true, vector: None, .. })), "unladen MSV");
        let mut r = raster(&f.camera);
        paint(&s, &ops, &mut r);
        let at = to_raster(f.camera.project(view0.hulls[0].row.pos), PIXEL);
        let inner = mix(f.palette.seat(0), f.palette.roles.ground, f.palette.fill_dim());
        assert_eq!(r.get(at[0], at[1]), Some(inner.to_ints()), "one pixel in the dimmed seat color");
        // The same hull laden is a glyph: the MSV square's corner is its
        // outline, in the seat's color (clear of the homeworld's ring, which
        // passes 4 pixels out on the axes and not at the corners).
        let mut laden = raster(&f.camera);
        render(&Scene { view: &f.replay.view_at(0.0), ..s }, &mut laden);
        let seat = f.palette.seat(0).to_ints();
        assert_eq!(laden.get(at[0] + 5, at[1] + 5), Some(seat), "laden: the glyph's corner");
        assert_ne!(r.get(at[0] + 5, at[1] + 5), Some(seat), "quiet: no glyph");
    }

    #[test]
    fn a_hull_in_flight_draws_a_faint_route_to_its_destination() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let Some(Op::Hull { route: Some((to, c)), .. }) = ops.iter().find(|o| matches!(o, Op::Hull { id: 5, .. }))
        else {
            panic!("the freighter flies to world 2")
        };
        assert_eq!(*to, to_raster(f.camera.project(f.replay.planets[2].pos), PIXEL));
        assert_eq!(*c, mix(f.palette.seat(0), f.palette.roles.ground, ROUTE_DIM));
        let mut r = raster(&f.camera);
        paint(&s, &ops, &mut r);
        assert!(r.count(*c) > 10, "the route is drawn");
        assert!(
            matches!(ops.iter().find(|o| matches!(o, Op::Hull { id: 6, .. })), Some(Op::Hull { route: None, .. })),
            "a hull at rest has no route"
        );
    }

    #[test]
    fn a_colony_is_a_disc_in_its_seats_color_wider_than_an_unowned_world() {
        let f = fixture();
        let (v0, v1) = (f.replay.view_at(0.0), f.replay.view_at(10.0));
        let world2 = |view: &View| {
            let s = Scene { replay: &f.replay, view, camera: &f.camera, palette: &f.palette, selected: None };
            plan(&s).into_iter().find_map(|o| match o {
                Op::World { id: 2, r, color, .. } => Some((r, color)),
                _ => None,
            })
        };
        let (r0, c0) = world2(&v0).unwrap();
        let (r1, c1) = world2(&v1).unwrap();
        assert_eq!(c0, f.palette.roles.world_dim, "unowned at t 0");
        assert_eq!(c1, f.palette.seat(0), "P0's colony at t 10");
        assert!(r1 > r0);
    }

    #[test]
    fn a_holding_bar_is_two_pixels_a_band_and_one_for_a_trace() {
        assert_eq!(bar_height(None), 0, "nothing held");
        assert_eq!(bar_height(Some(-0.4)), 1, "a trace below Band Empty still shows");
        assert_eq!(bar_height(Some(2.1)), 4);
        assert_eq!(bar_height(Some(3.25)), 7);
        assert_eq!(bar_height(Some(9.0)), BAR_MAX, "capped at Band V");
    }

    #[test]
    fn a_worlds_holdings_are_drawn_beside_it_one_bar_per_material() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let mut cam = f.camera;
        cam.scale = crate::camera::SECTOR_PX_PER_LY * 2.0;
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let held = |id| {
            ops.iter().find_map(|o| match o {
                Op::World { id: i, holdings, .. } if *i == id => Some(holdings.clone()),
                _ => None,
            })
        };
        assert_eq!(held(0).unwrap(), vec![(f.palette.seat(0), [4, 4, 7, 0, 0, 0, 0, 0])]);
        assert_eq!(held(2).unwrap(), vec![(f.palette.seat(1), [1, 0, 0, 0, 0, 0, 0, 0])], "a rival's mine");
        let mut r = raster(&cam);
        paint(&s, &ops, &mut r);
        let at = to_raster(cam.project(f.replay.planets[0].pos), PIXEL);
        let Some(Op::World { r: rad, .. }) = ops.iter().find(|o| matches!(o, Op::World { id: 0, .. })) else {
            panic!()
        };
        let (x0, base) = (at[0] - 4, bar_base(at, *rad));
        assert_eq!(r.get(x0 + 2, base - 7), Some(f.palette.material(2).to_ints()), "yellow's bar, seven tall");
        assert_ne!(r.get(x0 + 2, base - 8), Some(f.palette.material(2).to_ints()));
        assert_eq!(r.get(x0, base), Some(f.palette.seat(0).to_ints()), "the base in the holder's color");
        assert!(base - (BAR_MAX as i64) > at[1] + *rad as i64, "the bars clear the world");
        // The galaxy level draws none.
        let s =
            Scene { replay: &f.replay, view: &view, camera: &galaxy_camera(&f), palette: &f.palette, selected: None };
        assert!(plan(&s).iter().all(|o| !matches!(o, Op::World { holdings, .. } if !holdings.is_empty())));
    }

    #[test]
    fn a_laden_glyph_is_striped_in_its_cargos_materials_by_mass() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let mut cam = f.camera;
        cam.scale = crate::camera::SECTOR_PX_PER_LY * 2.0;
        cam.center = [0.0, 0.0];
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let mut r = raster(&cam);
        render(&s, &mut r);
        let (cyan, yellow) = (r.count(f.palette.material(0)), r.count(f.palette.material(2)));
        assert!(cyan > 0 && yellow > 0, "3 kt of cyan and 1.5 of yellow: {cyan} {yellow}");
        assert!(cyan > yellow, "the larger share is the wider stripe: {cyan} {yellow}");
    }

    #[test]
    fn a_stack_shows_its_count() {
        let f = fixture();
        let (one, many) = (crowd(&f, 0), crowd(&f, 4));
        let text = f.palette.roles.text_bright;
        let mut counts = Vec::new();
        for view in [&one, &many] {
            let s = Scene { replay: &f.replay, view, camera: &f.camera, palette: &f.palette, selected: None };
            let mut r = raster(&f.camera);
            render(&s, &mut r);
            counts.push(r.count(text));
        }
        assert!(counts[1] > counts[0], "the digit 5 is drawn: {counts:?}");
    }

    #[test]
    fn a_selection_is_bracketed() {
        let f = fixture();
        let view = f.replay.view_at(0.0);
        let s = Scene {
            replay: &f.replay,
            view: &view,
            camera: &f.camera,
            palette: &f.palette,
            selected: Some(Pick::Hull(5)),
        };
        let mut r = raster(&f.camera);
        render(&s, &mut r);
        assert!(r.count(f.palette.status(Status::Selected)) > 0);
    }

    #[test]
    fn glyphs_are_built_once_per_key() {
        let mut cache = GlyphCache::default();
        let k = GlyphKey { hull: 1, design: 7, beams: false, tubes: false };
        let a = cache.get(&Replay::from_json(TINY).unwrap(), k).clone();
        assert_eq!(cache.len(), 1);
        let _ = cache.get(&Replay::from_json(TINY).unwrap(), k);
        assert_eq!(cache.len(), 1);
        assert_eq!(a, glyph("MSV", "Ford", 0, 0));
    }

    #[test]
    fn a_point_lands_in_the_hex_whose_center_is_nearest() {
        let r = Replay::from_json(TINY).unwrap();
        let side = r.meta.hex_side_ly;
        for h in [(0, 0), (1, 0), (0, 1), (-2, 3), (5, -4)] {
            let c = hex_center(&r, h);
            assert_eq!(hex_of(&r, [c[0], c[1], 0.0]), h, "a center is its own hex");
            for k in 0..6 {
                // Just inside each vertex, still this hex.
                let a = (60.0 * k as f64).to_radians();
                let p = [c[0] + 0.95 * side * a.cos(), c[1] + 0.95 * side * a.sin(), 0.0];
                assert_eq!(hex_of(&r, p), h, "{h:?} vertex {k}");
            }
        }
        let a = hex_center(&r, (0, 0));
        let b = hex_center(&r, (1, 0));
        assert!(((b[0] - a[0]).hypot(b[1] - a[1]) - 3f64.sqrt() * side).abs() < 1e-9, "neighbors one width apart");
    }

    #[test]
    fn only_hexes_holding_a_world_or_a_hull_are_active() {
        let f = fixture();
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let active = active_hexes(&s);
        let mut want: std::collections::BTreeSet<(i64, i64)> =
            f.replay.planets.iter().map(|p| hex_of(&f.replay, p.pos)).collect();
        want.extend(view.hulls.iter().map(|h| hex_of(&f.replay, h.row.pos)));
        assert_eq!(active, want);
        assert!(active.len() <= f.replay.planets.len() + view.hulls.len());
    }

    /// A camera far enough out that the view is a galaxy.
    fn galaxy_camera(f: &Fixture) -> Camera {
        let mut c = f.camera;
        c.scale = crate::camera::SECTOR_PX_PER_LY / 2.0;
        c.center = [10.0, 0.0];
        c
    }

    #[test]
    fn at_the_galaxy_level_one_owners_hulls_in_a_place_are_one_marker() {
        let f = fixture();
        let view = crowd(&f, 4);
        let cam = galaxy_camera(&f);
        let s = Scene { replay: &f.replay, view: &view, camera: &cam, palette: &f.palette, selected: None };
        let ops = plan(&s);
        let drawn: Vec<&Op> = ops.iter().filter(|o| matches!(o, Op::Hull { count: 1.., .. })).collect();
        let total: u32 = drawn.iter().map(|o| if let Op::Hull { count, .. } = o { *count } else { 0 }).sum();
        assert_eq!(total as usize, view.hulls.len(), "every hull is counted");
        assert!(drawn.iter().all(|o| matches!(o, Op::Hull { marker: true, .. })), "the galaxy draws markers");
        let five = drawn.iter().find(|o| matches!(o, Op::Hull { members, .. } if members.contains(&5))).unwrap();
        let Op::Hull { count, members, .. } = five else { unreachable!() };
        assert_eq!(*count, 6, "the five Fords and the Delta are one owner's cluster");
        assert_eq!(members.len(), 6);
        let mut r = raster(&cam);
        paint(&s, &ops, &mut r);
        assert!(r.count(f.palette.seat(0)) > 0, "the marker is in the seat's color");
    }
}

/// Glyphs by key, built on first use.
#[derive(Default)]
pub struct GlyphCache(BTreeMap<GlyphKey, Glyph>);

impl GlyphCache {
    pub fn get(&mut self, replay: &Replay, k: GlyphKey) -> &Glyph {
        self.0.entry(k).or_insert_with(|| {
            let name = |list: &[String], i: usize| list.get(i).cloned().unwrap_or_default();
            glyph(&name(&replay.hulls, k.hull), &name(&replay.designs, k.design), k.beams as u32, k.tubes as u32)
        })
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
