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
use crate::glyph::{glyph, Glyph};
use crate::palette::{Palette, Status};
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
/// How far apart unalike stacks in one place are drawn, tactical pixels: a
/// General glyph, a gap, and a three-digit count.
pub const FAN_PX: i64 = 22;
/// At most this many stacks are drawn side by side in one place; the rest
/// are drawn together as one marker in the last place.
pub const FAN_MAX: usize = 4;

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

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    World {
        id: u32,
        /// Screen position, pixels.
        s: [f64; 2],
        /// The tactical pixel `s` falls in.
        at: [i64; 2],
        r: f64,
        color: Rgb,
        /// A homeworld's ring.
        ring: Option<Rgb>,
        selected: bool,
    },
    Hull {
        id: u64,
        s: [f64; 2],
        at: [i64; 2],
        /// Where the glyph is drawn: `at`, or beside it when unalike stacks
        /// share the place.
        draw: [i64; 2],
        /// Hulls this glyph stands for: its stack's size on the stack's
        /// lowest id, 0 on the others, which are not drawn.
        count: u32,
        /// The ids of the hulls this glyph stands for, on the drawn one.
        members: Vec<u64>,
        /// Drawn as a marker, a square in the seat's color sized by count:
        /// a cluster at the galaxy level, or stacks past [`FAN_MAX`] in one
        /// place. Its members can differ in Design and role.
        marker: bool,
        key: GlyphKey,
        /// The seat's color, or the wreck color.
        outline: Rgb,
        /// The outline's color, dimmed toward the ground.
        inner: Rgb,
        /// The Doctrine role's accent.
        core: Rgb,
        marks: Rgb,
        /// The drive vector's far end and color.
        vector: Option<([i64; 2], Rgb)>,
        /// Share of structure lost, 0 to 1.
        damage: f64,
        hit: bool,
        laden: bool,
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

/// One [`Op`] per world and one per hull in the view, each at its position.
pub fn plan(s: &Scene) -> Vec<Op> {
    let (r, f, pal) = (s.replay, &s.replay.frames[s.view.frame], s.palette);
    let screen = |p: [f64; 3]| s.camera.project(p);
    let base = match s.camera.lod() {
        Lod::Galaxy => 0.0,
        Lod::Sector => 1.0,
        Lod::System => 2.0,
    };
    let mut ops = Vec::with_capacity(r.planets.len() + s.view.hulls.len());
    for p in &r.planets {
        let owner = f.owner.get(p.id as usize).copied().flatten();
        let color = owner.map_or(pal.roles.world_dim, |o| pal.seat(o));
        let sp = screen(p.pos);
        ops.push(Op::World {
            id: p.id,
            s: sp,
            at: to_raster(sp, PIXEL),
            r: base + if p.home { 1.0 } else { 0.0 },
            color,
            ring: if p.home { Some(color) } else { None },
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
        let (outline, core, marks) = if row.wrecked {
            (pal.status(Status::Wreck), pal.roles.grid, pal.roles.grid)
        } else {
            (pal.seat(h.hull.owner), pal.role(role), pal.roles.text_bright)
        };
        let inner = mix(outline, pal.roles.ground, pal.fill_dim());
        ops.push(Op::Hull {
            id: h.hull.id,
            s: sp,
            at: here,
            draw: here,
            count: 1,
            members: Vec::new(),
            marker: false,
            key,
            outline,
            inner,
            core,
            marks,
            vector: vector(s, h.row, here),
            damage: row.damage.clamp(0.0, 1.0),
            hit: h.hit,
            laden: h.laden,
            selected: s.selected == Some(Pick::Hull(h.hull.id)),
        });
    }
    stack(&mut ops[first_hull..], s);
    ops
}

/// **Stacks** (`docs/Hyades_interface.md` §6.4): hulls in one place with one
/// owner, Design, role and wreck state are one glyph with a count, carrying
/// the worst damage and any hit, cargo or selection among them. The largest
/// stack in a place is drawn there; unalike stacks fan out to its right, each
/// joined to the place by a leader line, up to [`FAN_MAX`] — past it the rest
/// are one marker. At the galaxy level a place is wider and a stack is every
/// hull of one owner there, drawn as a marker sized by count.
fn stack(ops: &mut [Op], s: &Scene) {
    let lod = s.camera.lod();
    let cell_px = stack_px(lod);
    type Group = (i64, i64, usize, Option<(GlyphKey, usize, bool)>);
    let mut groups: BTreeMap<Group, Vec<usize>> = BTreeMap::new();
    for (i, h) in s.view.hulls.iter().enumerate() {
        let Op::Hull { at, key, .. } = ops[i] else { continue };
        let cell = (at[0].div_euclid(cell_px), at[1].div_euclid(cell_px));
        let kind = (lod != Lod::Galaxy).then_some((key, h.row.kind, h.row.wrecked));
        groups.entry((cell.0, cell.1, h.hull.owner, kind)).or_default().push(i);
    }
    // The largest stack in a place keeps the place; the rest fan out by size,
    // then by key.
    let mut places: BTreeMap<(i64, i64), Vec<Vec<usize>>> = BTreeMap::new();
    for ((cx, cy, ..), members) in groups {
        places.entry((cx, cy)).or_default().push(members);
    }
    for (_, mut stacks) in places {
        stacks.sort_by_key(|m| std::cmp::Reverse(m.len()));
        if stacks.len() > FAN_MAX {
            let rest: Vec<usize> = stacks.drain(FAN_MAX - 1..).flatten().collect();
            stacks.push(rest);
        }
        let n_stacks = stacks.len();
        let mut x = 0i64;
        for (slot, mut members) in stacks.into_iter().enumerate() {
            members.sort_unstable();
            let marker = lod == Lod::Galaxy
                || (slot == FAN_MAX - 1 && n_stacks == FAN_MAX && members.len() > 1 && {
                    let first = &ops[members[0]];
                    members.iter().any(|&m| !same_kind(&ops[m], first))
                });
            let (mut damage, mut hit, mut laden, mut selected) = (0.0f64, false, false, false);
            let mut ids = Vec::with_capacity(members.len());
            for &m in &members {
                if let Op::Hull { id, damage: d, hit: h, laden: l, selected: sel, count, .. } = &mut ops[m] {
                    (damage, hit, laden, selected) = (damage.max(*d), hit | *h, laden | *l, selected | *sel);
                    *count = 0;
                    ids.push(*id);
                }
            }
            let step = if lod == Lod::Galaxy { marker_px(members.len()) + 2 } else { FAN_PX };
            if let Op::Hull {
                at,
                draw,
                count,
                members: mem,
                marker: mk,
                damage: d,
                hit: h,
                laden: l,
                selected: sel,
                ..
            } = &mut ops[members[0]]
            {
                *draw = [at[0] + x, at[1]];
                (*count, *mk, *d, *h, *l, *sel) = (members.len() as u32, marker, damage, hit, laden, selected);
                *mem = ids;
            }
            x += step;
        }
    }
}

fn same_kind(a: &Op, b: &Op) -> bool {
    match (a, b) {
        (Op::Hull { key: ka, core: ca, outline: oa, .. }, Op::Hull { key: kb, core: cb, outline: ob, .. }) => {
            ka == kb && ca == cb && oa == ob
        }
        _ => false,
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

/// Draws the hex grid and then every op, worlds under hulls, with glyphs from
/// `cache` (kept between frames).
pub fn paint_with(s: &Scene, ops: &[Op], r: &mut Raster, cache: &mut GlyphCache) {
    let pal = s.palette;
    r.clear(pal.roles.ground);
    hex_grid(s, r);
    for op in ops {
        if let Op::World { at, r: rad, color, ring, selected, .. } = op {
            r.disc(at[0], at[1], *rad, *color);
            if let Some(c) = ring {
                r.circle(at[0], at[1], rad + 2.0, *c);
            }
            if *selected {
                brackets(r, *at, *rad as i64 + 3, pal.status(Status::Selected));
            }
        }
    }
    for op in ops {
        if let Op::Hull { at, draw, count: 1.., vector, marker, .. } = op {
            if draw != at {
                r.line(at[0], at[1], draw[0], draw[1], pal.roles.grid);
            }
            if let (Some((end, c)), false) = (vector, marker) {
                let (dx, dy) = (end[0] - at[0], end[1] - at[1]);
                r.line(draw[0], draw[1], draw[0] + dx, draw[1] + dy, *c);
            }
        }
    }
    for op in ops {
        let Op::Hull {
            draw: at,
            count: count @ 1..,
            key,
            outline,
            inner,
            core,
            marks,
            damage,
            hit,
            laden,
            selected,
            marker,
            ..
        } = op
        else {
            continue;
        };
        if *marker {
            let side = marker_px(*count as usize);
            let h = side / 2;
            for y in -h..=h {
                for x in -h..=h {
                    let edge = x.abs() == h || y.abs() == h;
                    r.set(at[0] + x, at[1] + y, if edge { *outline } else { *inner });
                }
            }
            if *hit {
                r.circle(at[0], at[1], (h + 3) as f64, pal.status(Status::Hit));
            }
            if *selected {
                brackets(r, *at, h + 3, pal.status(Status::Selected));
            }
            if *count > 1 && s.camera.lod() != Lod::Galaxy {
                r.number(at[0] + h + 2, at[1] - 2, *count, pal.roles.text_bright);
            }
            continue;
        }
        let g = cache.get(s.replay, *key);
        let (gw, ax, ay) = (g.w, g.ax, g.ay);
        r.stamp(&g.outline, gw, ax, ay, at[0], at[1], *outline);
        r.stamp(&g.inner, gw, ax, ay, at[0], at[1], *inner);
        r.stamp(&g.core, gw, ax, ay, at[0], at[1], *core);
        r.stamp(&g.marks, gw, ax, ay, at[0], at[1], *marks);
        let half = ax as i64 - 2;
        if *laden {
            r.set(at[0], at[1], pal.status(Status::Laden));
        }
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
        if *count > 1 {
            r.number(at[0] + half + 4, at[1] - 2, *count, pal.roles.text_bright);
        }
    }
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
        let Op::Hull { outline, inner, core, key, laden, .. } = freighter else { unreachable!() };
        assert_eq!(*outline, f.palette.seat(0));
        assert_eq!(*inner, mix(f.palette.seat(0), f.palette.roles.ground, f.palette.fill_dim()));
        assert_eq!(*core, f.palette.role("Freighter"));
        assert_eq!(f.replay.designs[key.design], "Ford");
        assert_eq!(f.replay.hulls[key.hull], "MSV");
        assert!(*laden);
    }

    #[test]
    fn the_drive_vector_reads_burn_and_points_where_the_hull_goes() {
        let f = fixture();
        let view = f.replay.view_at(10.0);
        let s = Scene { replay: &f.replay, view: &view, camera: &f.camera, palette: &f.palette, selected: None };
        let ops = plan(&s);
        // Hull 9 burns at rest toward world 2, which lies up and to the right.
        let Some(Op::Hull { at, vector: Some((end, color)), .. }) =
            ops.iter().find(|o| matches!(o, Op::Hull { id: 9, .. }))
        else {
            panic!("a burning hull has a vector")
        };
        assert_eq!(*color, f.palette.status(Status::Drive));
        assert!(end[0] > at[0] && end[1] < at[1], "{at:?} → {end:?}");
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
                Op::Hull { id: i, count, damage, draw, at, .. } if *i == id => Some((*count, *damage, *draw, *at)),
                _ => None,
            })
        };
        let (n, damage, draw, at) = lead(5).unwrap();
        assert_eq!((n, draw), (5, at), "five alike Fords are one glyph, in place");
        assert!((damage - 0.3).abs() < 1e-12, "carrying the worst damage among them");
        assert_eq!(lead(101).unwrap().0, 0, "a stacked hull is not drawn on its own");
        let (n, _, draw, at) = lead(200).unwrap();
        assert_eq!(n, 1);
        assert_eq!(draw, [at[0] + FAN_PX, at[1]], "the unalike hull fans out beside them");
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

    #[test]
    fn past_the_fan_limit_the_rest_of_a_place_is_one_marker() {
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
        let drawn: Vec<&Op> = ops.iter().filter(|o| matches!(o, Op::Hull { count: 1.., .. })).collect();
        let at_place = drawn
            .iter()
            .filter(|o| matches!(o, Op::Hull { at, .. } if *at == to_raster(f.camera.project(five.row.pos), PIXEL)))
            .count();
        assert_eq!(at_place, FAN_MAX, "no more than the limit side by side");
        assert_eq!(drawn.iter().filter(|o| matches!(o, Op::Hull { marker: true, .. })).count(), 1);
        let total: u32 = drawn.iter().map(|o| if let Op::Hull { count, .. } = o { *count } else { 0 }).sum();
        assert_eq!(total as usize, view.hulls.len());
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
