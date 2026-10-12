//! **Paths** (`docs/Hyades_interface.md` §6.7, T-174, the author's ruling):
//! where each large stack is going over the next [`PATH_AHEAD_YEARS`], and,
//! for the selected hull only, where it has been. Both modes draw the same
//! paths, each in its own style.
//!
//! **A path's color is its drive** ([`drive_color`]): a dim gray where the
//! hull coasts, moved toward [`Status::BlueShift`] where the drive gains
//! speed and toward [`Status::RedShift`] where it sheds it, the farther the
//! harder it burns. A path fades toward its far end, and ends where its hull
//! is wrecked.

use crate::color::{mix, Rgb};
use crate::palette::{Palette, Status};
use crate::replay::TrackPoint;
use crate::tactical::{Op, Pick, Scene};

/// Hulls a stack needs for its path to be drawn (proposed, R-UI4).
pub const PATH_MIN_STACK: u32 = 5;
/// How far ahead a path reaches, years (the author's ruling).
pub const PATH_AHEAD_YEARS: f64 = 5.0;
/// Instants sampled along a path ahead, and along the selected hull's past.
pub const PATH_SAMPLES: usize = 64;
pub const PATH_PAST_SAMPLES: usize = 128;
/// A path's opacity where it leaves its hull (proposed, R-UI4).
pub const PATH_ALPHA: f64 = 0.7;
/// The coasting gray: this share of the way from the ground to the text
/// color (proposed, R-UI4).
pub const PATH_COAST: f64 = 0.45;
/// The acceleration at which a path's color is half way from the coasting
/// gray to its shift, ly/yr² — a quarter g, about a laden Medium hull's
/// burn (proposed, R-UI4).
pub const PATH_ACCEL_HALF: f64 = 0.25 * 1.0323;

/// One instant of a path: where, in what color, how opaque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathPoint {
    pub pos: [f64; 3],
    pub color: Rgb,
    pub alpha: f64,
}

/// A path's instants in time order; a segment takes its first point's
/// color and opacity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub points: Vec<PathPoint>,
}

/// **A drive's color**: the coasting gray, moved toward the blue shift where
/// the burn gains speed (`burn > 0`) and toward the red shift where it sheds
/// it (`burn < 0`), by `a / (a + PATH_ACCEL_HALF)` of the way.
pub fn drive_color(pal: &Palette, accel: f64, burn: i8) -> Rgb {
    let coast = mix(pal.roles.ground, pal.roles.text, PATH_COAST);
    if burn == 0 || accel <= 0.0 {
        return coast;
    }
    let end = pal.status(if burn > 0 { Status::BlueShift } else { Status::RedShift });
    mix(coast, end, accel / (accel + PATH_ACCEL_HALF))
}

/// A track as a path, faded from `PATH_ALPHA` at `near` to `PATH_ALPHA·far_alpha`
/// at `far`. Points that share a drive share a color, which is worked out
/// once for them.
fn path(pal: &Palette, track: Vec<TrackPoint>, near: f64, far: f64, far_alpha: f64) -> Path {
    let mut last: Option<(f64, i8, Rgb)> = None;
    let span = far - near;
    let points = track
        .into_iter()
        .map(|p| {
            let color = match last {
                Some((a, b, c)) if a == p.accel && b == p.burn => c,
                _ => drive_color(pal, p.accel, p.burn),
            };
            last = Some((p.accel, p.burn, color));
            let share = if span != 0.0 { ((p.t - near) / span).clamp(0.0, 1.0) } else { 0.0 };
            PathPoint { pos: p.pos, color, alpha: PATH_ALPHA * (1.0 - share * (1.0 - far_alpha)) }
        })
        .collect();
    Path { points }
}

/// **The paths to draw at this instant**: ahead, for every drawn stack of at
/// least [`PATH_MIN_STACK`] hulls that is not a wreck, from its lead hull;
/// and for the selected hull, its past and, if its stack drew none, its way
/// ahead.
pub fn paths(s: &Scene, ops: &[Op]) -> Vec<Path> {
    let t = s.view.t;
    // Ahead ends where the hull is wrecked: a wreck coasts on, but on no
    // course of its own.
    let ahead = |id: u64| {
        let end = s.replay.hull_table.get(&id).and_then(|h| h.wrecked_at).map_or(f64::INFINITY, |w| w.max(t));
        let to = (t + PATH_AHEAD_YEARS).min(end);
        path(s.palette, s.replay.track(id, t, to, PATH_SAMPLES), t, t + PATH_AHEAD_YEARS, 0.0)
    };
    let selected = match s.selected {
        Some(Pick::Hull(id)) => Some(id),
        _ => None,
    };
    let mut out = Vec::new();
    let mut selected_ahead = false;
    for op in ops {
        let Op::Hull { id, count, members, wreck: None, .. } = op else { continue };
        if *count < PATH_MIN_STACK {
            continue;
        }
        selected_ahead |= selected.is_some_and(|sel| sel == *id || members.contains(&sel));
        out.push(ahead(*id));
    }
    if let Some(sel) = selected {
        let start = s.replay.first_frame.get(&sel).map_or(t, |&i| s.replay.frames[i].t);
        out.push(path(s.palette, s.replay.track(sel, start, t, PATH_PAST_SAMPLES), t, start, 0.25));
        if !selected_ahead {
            out.push(ahead(sel));
        }
    }
    out.retain(|p| p.points.len() > 1);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;
    use crate::color::delta_e;
    use crate::replay::tests::TINY;
    use crate::replay::Replay;

    #[test]
    fn a_drive_shifts_blue_gaining_speed_and_red_shedding_it_the_more_the_harder() {
        let p = Palette::default();
        let coast = drive_color(&p, 0.0, 0);
        assert_eq!(drive_color(&p, 3.0, 0), coast, "no burn, no shift");
        let (blue, red) = (p.status(Status::BlueShift), p.status(Status::RedShift));
        let soft = drive_color(&p, 0.1, 1);
        let hard = drive_color(&p, 3.0, 1);
        assert!(delta_e(hard, blue) < delta_e(soft, blue), "a harder burn is bluer");
        assert!(delta_e(soft, blue) < delta_e(coast, blue), "any burn is bluer than coasting");
        assert!(delta_e(drive_color(&p, 3.0, -1), red) < delta_e(hard, red), "braking is redder");
        let half = drive_color(&p, PATH_ACCEL_HALF, -1);
        assert!((delta_e(coast, half) - delta_e(half, red)).abs() < 0.02, "half way at PATH_ACCEL_HALF");
    }

    /// **Only large stacks draw a path ahead, and only the selected hull its
    /// past** (T-174): in the fixture no stack is large, so nothing draws
    /// until a hull is selected; selected, it draws its past from its first
    /// frame, fading toward the start, and its way ahead, fading out.
    #[test]
    fn only_large_stacks_and_the_selection_draw_paths() {
        let r = Replay::from_json(TINY).unwrap();
        let mut camera = Camera::new(320.0, 200.0);
        camera.fit(r.planets.iter().map(|p| p.pos), 20.0);
        let palette = Palette::default();
        let view = r.view_at(5.0);
        let scene =
            |selected| Scene { replay: &r, view: &view, camera: &camera, palette: &palette, selected, rate: 1.0 };
        let s = scene(None);
        assert!(paths(&s, &crate::tactical::plan(&s)).is_empty(), "small stacks draw nothing");
        let s = scene(Some(Pick::Hull(5)));
        let drawn = paths(&s, &crate::tactical::plan(&s));
        assert_eq!(drawn.len(), 2, "its past and its way ahead");
        let (past, ahead) = (&drawn[0], &drawn[1]);
        assert!(past.points.last().unwrap().alpha > past.points[0].alpha, "the past fades toward its start");
        assert!(ahead.points[0].alpha > ahead.points.last().unwrap().alpha, "ahead fades out");
        assert!((ahead.points.last().unwrap().alpha).abs() < 1e-12);
        // Five copies of hull 5 at its place are a large stack: it draws ahead
        // with nothing selected.
        let mut crowd = r.view_at(5.0);
        let five = *crowd.hulls.iter().find(|h| h.hull.id == 5).unwrap();
        for k in 0..4 {
            let mut c = five;
            (c.hull.id, c.row.id) = (100 + k, 100 + k);
            crowd.hulls.push(c);
        }
        let s = Scene { replay: &r, view: &crowd, camera: &camera, palette: &palette, selected: None, rate: 1.0 };
        assert_eq!(paths(&s, &crate::tactical::plan(&s)).len(), 1, "a stack of five draws its way ahead");
    }
}
