//! **The camera** (`docs/Hyades_interface.md` §5): a top-down orthographic
//! view of the galaxy plane, in screen pixels. Galaxy `x` runs right and `y`
//! runs up; `z` is height off the plane and is not projected.
//!
//! The camera is in **screen** pixels for both modes, so switching mode keeps
//! the view; a renderer whose framebuffer is coarser divides by its pixel size.

/// Below this many screen pixels per light-year the view is a galaxy.
pub const SECTOR_PX_PER_LY: f64 = 2.0;
/// At or above this many screen pixels per light-year the view is a system.
pub const SYSTEM_PX_PER_LY: f64 = 40.0;
/// Zoom limits, screen pixels per light-year. The top shows a beam fight
/// (hulls a few hundredths of a light-year apart) across the screen.
pub const MIN_SCALE: f64 = 0.01;
pub const MAX_SCALE: f64 = 200_000.0;

/// The level of detail a scale calls for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lod {
    /// The whole theater: hexes, empires, flows.
    Galaxy,
    /// A hex or a few: worlds and fleets.
    Sector,
    /// A neighborhood of a world: single hulls and fire.
    System,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// The galaxy point at the screen's center, ly.
    pub center: [f64; 2],
    /// Screen pixels per light-year.
    pub scale: f64,
    pub width: f64,
    pub height: f64,
}

impl Camera {
    pub fn new(width: f64, height: f64) -> Camera {
        Camera { center: [0.0, 0.0], scale: 1.0, width, height }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    /// Galaxy point to screen pixel.
    pub fn project(&self, p: [f64; 3]) -> [f64; 2] {
        [
            self.width / 2.0 + (p[0] - self.center[0]) * self.scale,
            self.height / 2.0 - (p[1] - self.center[1]) * self.scale,
        ]
    }

    /// Screen pixel to the galaxy plane.
    pub fn unproject(&self, s: [f64; 2]) -> [f64; 2] {
        [
            self.center[0] + (s[0] - self.width / 2.0) / self.scale,
            self.center[1] - (s[1] - self.height / 2.0) / self.scale,
        ]
    }

    /// Zooms by `factor`, keeping the galaxy point under screen pixel `s`
    /// where it is.
    pub fn zoom_at(&mut self, s: [f64; 2], factor: f64) {
        let before = self.unproject(s);
        self.scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let after = self.unproject(s);
        self.center[0] += before[0] - after[0];
        self.center[1] += before[1] - after[1];
    }

    /// Drags the view by a screen displacement: what was under the pointer
    /// stays under it.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.center[0] -= dx / self.scale;
        self.center[1] += dy / self.scale;
    }

    /// Frames every point, with `margin` screen pixels clear on each side.
    pub fn fit(&mut self, points: impl IntoIterator<Item = [f64; 3]>, margin: f64) {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in points {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if lo[0] > hi[0] {
            return;
        }
        self.center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let span = [(hi[0] - lo[0]).max(1e-9), (hi[1] - lo[1]).max(1e-9)];
        let room = [(self.width - 2.0 * margin).max(1.0), (self.height - 2.0 * margin).max(1.0)];
        self.scale = (room[0] / span[0]).min(room[1] / span[1]).clamp(MIN_SCALE, MAX_SCALE);
    }

    pub fn lod(&self) -> Lod {
        if self.scale < SECTOR_PX_PER_LY {
            Lod::Galaxy
        } else if self.scale < SYSTEM_PX_PER_LY {
            Lod::Sector
        } else {
            Lod::System
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
    }

    #[test]
    fn the_center_projects_to_the_middle_and_y_runs_up() {
        let mut c = Camera::new(200.0, 100.0);
        c.center = [10.0, 5.0];
        c.scale = 4.0;
        assert_eq!(c.project([10.0, 5.0, 99.0]), [100.0, 50.0]);
        assert_eq!(c.project([11.0, 6.0, 0.0]), [104.0, 46.0], "galaxy +y is screen up");
    }

    #[test]
    fn unproject_inverts_project() {
        let mut c = Camera::new(640.0, 400.0);
        c.center = [-3.5, 12.25];
        c.scale = 7.5;
        for p in [[0.0, 0.0], [100.0, -40.0], [-3.5, 12.25]] {
            let s = c.project([p[0], p[1], 0.0]);
            assert!(close(c.unproject(s), p));
        }
    }

    #[test]
    fn zoom_keeps_the_point_under_the_pointer() {
        let mut c = Camera::new(640.0, 400.0);
        let s = [100.0, 300.0];
        let before = c.unproject(s);
        c.zoom_at(s, 8.0);
        assert_eq!(c.scale, 8.0);
        assert!(close(c.unproject(s), before));
        c.zoom_at(s, 1e12);
        assert_eq!(c.scale, MAX_SCALE, "zoom is bounded");
    }

    #[test]
    fn a_drag_moves_the_content_with_the_pointer() {
        let mut c = Camera::new(640.0, 400.0);
        c.scale = 2.0;
        let p = [10.0, 10.0, 0.0];
        let s = c.project(p);
        c.pan(30.0, -20.0);
        assert!(close(c.project(p), [s[0] + 30.0, s[1] - 20.0]));
    }

    #[test]
    fn fit_frames_every_point_inside_the_margin() {
        let mut c = Camera::new(640.0, 400.0);
        let pts = [[-100.0, -20.0, 0.0], [300.0, 80.0, 5.0], [0.0, 0.0, 0.0]];
        c.fit(pts, 10.0);
        for p in pts {
            let s = c.project(p);
            assert!((10.0 - 1e-9..=630.0 + 1e-9).contains(&s[0]) && (10.0 - 1e-9..=390.0 + 1e-9).contains(&s[1]));
        }
        assert!((c.scale - 1.55).abs() < 1e-12, "the wider axis binds: 620 px over 400 ly");
    }

    #[test]
    fn the_level_of_detail_rises_with_scale() {
        let mut c = Camera::new(640.0, 400.0);
        c.scale = 1.0;
        assert_eq!(c.lod(), Lod::Galaxy);
        c.scale = SECTOR_PX_PER_LY;
        assert_eq!(c.lod(), Lod::Sector);
        c.scale = SYSTEM_PX_PER_LY;
        assert_eq!(c.lod(), Lod::System);
    }
}
