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
use crate::palette::{status, Palette, Status};
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
/// Hulls whose positions fall in one square this many tactical pixels wide
/// are in one place: alike, they stack; unalike, they fan out.
pub const STACK_PX: i64 = 3;
/// How far apart unalike stacks in one place are drawn, tactical pixels: a
/// General glyph, a gap, and a three-digit count.
pub const FAN_PX: i64 = 22;
/// A glyph's fill is its seat's color this far toward the ground, so the
/// edge reads first.
pub const FILL_DIM: f64 = 0.45;

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
            (status(Status::Wreck), pal.roles.grid, pal.roles.grid)
        } else {
            (pal.seat(h.hull.owner), pal.role(role), pal.roles.text_bright)
        };
        let inner = mix(outline, pal.roles.ground, FILL_DIM);
        ops.push(Op::Hull {
            id: h.hull.id,
            s: sp,
            at: here,
            draw: here,
            count: 1,
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
/// joined to the place by a leader line.
fn stack(ops: &mut [Op], s: &Scene) {
    type Group = (i64, i64, usize, GlyphKey, usize, bool);
    let mut groups: BTreeMap<Group, Vec<usize>> = BTreeMap::new();
    for (i, h) in s.view.hulls.iter().enumerate() {
        let Op::Hull { at, key, .. } = ops[i] else { continue };
        let cell = (at[0].div_euclid(STACK_PX), at[1].div_euclid(STACK_PX));
        groups.entry((cell.0, cell.1, h.hull.owner, key, h.row.kind, h.row.wrecked)).or_default().push(i);
    }
    // The largest stack in a place keeps the place; the rest fan out by size,
    // then by key.
    let mut places: BTreeMap<(i64, i64), Vec<Vec<usize>>> = BTreeMap::new();
    for ((cx, cy, ..), members) in groups {
        places.entry((cx, cy)).or_default().push(members);
    }
    for (_, mut stacks) in places {
        stacks.sort_by_key(|m| std::cmp::Reverse(m.len()));
        for (slot, members) in stacks.iter().enumerate() {
            let (mut damage, mut hit, mut laden, mut selected) = (0.0f64, false, false, false);
            for &m in members {
                if let Op::Hull { damage: d, hit: h, laden: l, selected: sel, count, .. } = &mut ops[m] {
                    (damage, hit, laden, selected) = (damage.max(*d), hit | *h, laden | *l, selected | *sel);
                    *count = 0;
                }
            }
            if let Op::Hull { at, draw, count, damage: d, hit: h, laden: l, selected: sel, .. } = &mut ops[members[0]] {
                *draw = [at[0] + slot as i64 * FAN_PX, at[1]];
                (*count, *d, *h, *l, *sel) = (members.len() as u32, damage, hit, laden, selected);
            }
        }
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
        b if b > 0 && row.accel >= HOT_ACCEL => status(Status::DriveHot),
        b if b > 0 => status(Status::Drive),
        b if b < 0 => status(Status::Braking),
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
                brackets(r, *at, *rad as i64 + 3);
            }
        }
    }
    for op in ops {
        if let Op::Hull { at, draw, count: 1.., vector, .. } = op {
            if draw != at {
                r.line(at[0], at[1], draw[0], draw[1], pal.roles.grid);
            }
            if let Some((end, c)) = vector {
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
            ..
        } = op
        else {
            continue;
        };
        let g = cache.get(s.replay, *key);
        let (gw, ax, ay) = (g.w, g.ax, g.ay);
        r.stamp(&g.outline, gw, ax, ay, at[0], at[1], *outline);
        r.stamp(&g.inner, gw, ax, ay, at[0], at[1], *inner);
        r.stamp(&g.core, gw, ax, ay, at[0], at[1], *core);
        r.stamp(&g.marks, gw, ax, ay, at[0], at[1], *marks);
        let half = ax as i64 - 2;
        if *laden {
            r.set(at[0], at[1], status(Status::Laden));
        }
        if *damage > 0.0 {
            let n = (damage * (2 * half + 1) as f64).ceil() as i64;
            for k in 0..n {
                r.set(at[0] - half - 3, at[1] + half - k, status(Status::Damage));
            }
        }
        if *hit {
            r.circle(at[0], at[1], (half + 4) as f64, status(Status::Hit));
        }
        if *selected {
            brackets(r, *at, half + 5);
        }
        if *count > 1 {
            r.number(at[0] + half + 4, at[1] - 2, *count, pal.roles.text_bright);
        }
    }
}

/// Four corner brackets `d` pixels out from `at`, in the selection color.
fn brackets(r: &mut Raster, at: [i64; 2], d: i64) {
    let c = status(Status::Selected);
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (x, y) = (at[0] + sx * d, at[1] + sy * d);
        r.line(x, y, x - sx * 2, y, c);
        r.line(x, y, x, y - sy * 2, c);
    }
}

/// The command view's hexes (flat-top, a side of `hex_side_ly`, one centered
/// on `hex_origin`), each drawing its three upper edges so every edge is drawn
/// once.
fn hex_grid(s: &Scene, r: &mut Raster) {
    let side = s.replay.meta.hex_side_ly;
    let width = 3f64.sqrt() * side;
    if side <= 0.0 || width * s.camera.scale / PIXEL < MIN_HEX_PX {
        return;
    }
    let o = s.replay.meta.hex_origin;
    // Lattice: center(i, j) = o + i·a + j·b, a = width·(cos 30°, sin 30°), b = width·(0, 1).
    let (ax, ay) = (width * 3f64.sqrt() / 2.0, width / 2.0);
    let corners = [[0.0, 0.0], [s.camera.width, 0.0], [0.0, s.camera.height], [s.camera.width, s.camera.height]]
        .map(|c| s.camera.unproject(c));
    let (mut i0, mut i1, mut j0, mut j1) = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);
    for c in corners {
        let i = (c[0] - o[0]) / ax;
        let j = (c[1] - o[1] - i * ay) / width;
        i0 = i0.min(i.floor() as i64 - 1);
        i1 = i1.max(i.ceil() as i64 + 1);
        j0 = j0.min(j.floor() as i64 - 2);
        j1 = j1.max(j.ceil() as i64 + 2);
    }
    let vertex = |cx: f64, cy: f64, k: i32| {
        let a = (60.0 * k as f64).to_radians();
        to_raster(s.camera.project([cx + side * a.cos(), cy + side * a.sin(), 0.0]), PIXEL)
    };
    for i in i0..=i1 {
        for j in j0..=j1 {
            let (cx, cy) = (o[0] + i as f64 * ax, o[1] + i as f64 * ay + j as f64 * width);
            for k in 0..3 {
                let (p, q) = (vertex(cx, cy, k), vertex(cx, cy, k + 1));
                r.line(p[0], p[1], q[0], q[1], s.palette.roles.hex);
            }
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
        assert_eq!(*inner, mix(f.palette.seat(0), f.palette.roles.ground, FILL_DIM));
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
        assert_eq!(*color, status(Status::Drive));
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
        assert_eq!(*outline, status(Status::Wreck));
        assert!(*hit);
        assert_eq!(*damage, 1.0);
        let mut r = raster(&f.camera);
        paint(&s, &ops, &mut r);
        assert!(r.count(status(Status::Hit)) > 0, "the hit ring is drawn");
        assert!(r.count(status(Status::Damage)) > 0, "the damage bar is drawn");
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
        assert!(r.count(status(Status::Selected)) > 0);
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
