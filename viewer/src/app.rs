//! **The viewer** (`docs/Hyades_interface.md` §2): one replay, its playback
//! clock, its camera, the log filter, the selection and the framebuffer. The
//! web shell drives it through [`crate::ffi`]; everything it shows as text it
//! reads from here, so the shell formats nothing.

use crate::camera::Camera;
use crate::juicy::{self, Hdr, Lights, ToneLut};
use crate::logview::{self, LogQuery};
use crate::palette::Palette;
use crate::raster::Raster;
use crate::replay::{Replay, View};
use crate::tactical::{self, GlyphCache, Op, Pick, Scene};
use crate::timeline::Timeline;
use std::fmt::Write as _;

/// A replay opens at a rate that plays it through in this many seconds.
pub const OPENING_PLAY_SECONDS: f64 = 60.0;
/// A pick takes the nearest entity within this many screen pixels.
pub const PICK_RADIUS: f64 = 10.0;
/// Fit leaves this many screen pixels clear at each edge.
pub const FIT_MARGIN: f64 = 24.0;
/// ly/yr² per g.
const G: f64 = 1.0323;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Tactical,
    Juicy,
}

pub struct Viewer {
    pub replay: Replay,
    pub palette: Palette,
    pub timeline: Timeline,
    pub camera: Camera,
    pub mode: Mode,
    pub query: LogQuery,
    pub selected: Option<Pick>,
    pub out: Raster,
    /// The juicy mode's lights at the current instant, for the GPU renderer.
    pub lights: Lights,
    hdr: Hdr,
    lut: ToneLut,
    glyphs: GlyphCache,
}

impl Viewer {
    /// Reads a replay and frames the whole galaxy on a screen `w × h`.
    pub fn load(json: &str, w: f64, h: f64) -> Result<Viewer, String> {
        let replay = Replay::from_json(json)?;
        let (t0, t1) = (replay.t0(), replay.t1());
        let rate = ((t1 - t0) / OPENING_PLAY_SECONDS).max(1e-6);
        let mut v = Viewer {
            timeline: Timeline::new(t0, t1, rate),
            camera: Camera::new(w, h),
            palette: Palette::default(),
            mode: Mode::Tactical,
            query: LogQuery::default(),
            selected: None,
            out: Raster::new(1, 1),
            lights: Lights::default(),
            hdr: Hdr::new(1, 1),
            lut: ToneLut::new(),
            glyphs: GlyphCache::default(),
            replay,
        };
        v.focus();
        Ok(v)
    }

    /// Frames the replay's focus ([`crate::replay::Meta::focus`]), or
    /// everything when it has none.
    pub fn focus(&mut self) {
        match self.replay.meta.focus {
            Some([x, y, r]) => self.camera.fit([[x - r, y - r, 0.0], [x + r, y + r, 0.0]], FIT_MARGIN),
            None => self.fit(),
        }
    }

    /// Frames every world and every hull of the first frame.
    pub fn fit(&mut self) {
        let hulls = self.replay.frames[0].rows.iter().map(|r| r.pos);
        let pts: Vec<[f64; 3]> = self.replay.planets.iter().map(|p| p.pos).chain(hulls).collect();
        self.camera.fit(pts, FIT_MARGIN);
    }

    pub fn resize(&mut self, w: f64, h: f64) {
        self.camera.resize(w, h);
    }

    pub fn tick(&mut self, dt: f64) {
        self.timeline.tick(dt);
    }

    pub fn view(&self) -> View {
        self.replay.view_at(self.timeline.t)
    }

    /// The frame's tactical plan and its view.
    fn planned(&self) -> (View, Vec<Op>) {
        let view = self.view();
        let s = self.scene(&view);
        let ops = tactical::plan(&s);
        (view, ops)
    }

    fn scene<'a>(&'a self, view: &'a View) -> Scene<'a> {
        Scene { replay: &self.replay, view, camera: &self.camera, palette: &self.palette, selected: self.selected }
    }

    /// Draws the current instant in the current mode into [`Self::out`].
    pub fn render(&mut self) -> &Raster {
        let (view, ops) = self.planned();
        let (mut out, mut hdr, mut glyphs) =
            (std::mem::take(&mut self.out), std::mem::take(&mut self.hdr), std::mem::take(&mut self.glyphs));
        let s = self.scene(&view);
        match self.mode {
            Mode::Tactical => {
                let w = (self.camera.width / tactical::PIXEL).ceil() as usize;
                let h = (self.camera.height / tactical::PIXEL).ceil() as usize;
                out.resize(w.max(1), h.max(1));
                tactical::paint_with(&s, &ops, &mut out, &mut glyphs);
            }
            Mode::Juicy => juicy::render(&s, &ops, &mut hdr, &mut out, &self.lut),
        }
        (self.out, self.hdr, self.glyphs) = (out, hdr, glyphs);
        &self.out
    }

    /// Works out the juicy mode's lights at the current instant into
    /// [`Self::lights`], for a renderer outside the module to draw.
    pub fn prepare_lights(&mut self) -> &Lights {
        let (view, ops) = self.planned();
        self.lights = juicy::lights(&self.scene(&view), &ops);
        &self.lights
    }

    /// Selects the hull, else the world, nearest screen point `(x, y)` within
    /// [`PICK_RADIUS`]; clears the selection when nothing is that near.
    pub fn pick(&mut self, x: f64, y: f64) -> Option<Pick> {
        let (_, ops) = self.planned();
        let near = |s: [f64; 2]| (s[0] - x).hypot(s[1] - y);
        let best = |want_hull: bool| {
            ops.iter()
                .filter_map(|op| match op {
                    Op::Hull { id, draw, count: 1.., .. } if want_hull => {
                        let s = [(draw[0] as f64 + 0.5) * tactical::PIXEL, (draw[1] as f64 + 0.5) * tactical::PIXEL];
                        Some((near(s), Pick::Hull(*id)))
                    }
                    Op::World { id, s, .. } if !want_hull => Some((near(*s), Pick::World(*id))),
                    _ => None,
                })
                .filter(|(d, _)| *d <= PICK_RADIUS)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, p)| p)
        };
        self.selected = best(true).or_else(|| best(false));
        self.selected
    }

    /// One line: the clock, the frame, the rate, the level of detail, the mode.
    pub fn status(&self) -> String {
        let t = &self.timeline;
        format!(
            "t {:.3} yr of {:.3} · frame {}/{} · {}{:.3} yr/s · {:?} · {:?}{}",
            t.t,
            t.t1,
            self.replay.frame_at(t.t) + 1,
            self.replay.frames.len(),
            if t.playing { "▶ " } else { "❚❚ " },
            t.rate,
            self.camera.lod(),
            self.mode,
            if self.replay.events_truncated { " · log truncated" } else { "" }
        )
    }

    /// What the selection is, field per line.
    pub fn inspector(&self) -> String {
        let r = &self.replay;
        let f = &r.frames[r.frame_at(self.timeline.t)];
        let mut out = String::new();
        match self.selected {
            None => out.push_str("Nothing selected. Click a hull or a world."),
            Some(Pick::World(id)) => {
                let Some(p) = r.planets.get(id as usize) else { return out };
                let owner = f.owner.get(id as usize).copied().flatten();
                let _ = writeln!(out, "World {id}{}", if p.home { " · homeworld" } else { "" });
                let _ = writeln!(out, "owner      {}", owner.map_or("none".into(), |o| format!("P{o}")));
                let _ = writeln!(out, "position   {:.2}, {:.2}, {:.2} ly", p.pos[0], p.pos[1], p.pos[2]);
                let _ = writeln!(out, "population Band {:.3}", f.pop.get(id as usize).copied().unwrap_or(0.0));
                let _ = writeln!(out, "works      Band {:.3}", f.works.get(id as usize).copied().unwrap_or(0.0));
                let _ = writeln!(out, "hab        Band {:.3}", p.hab);
                let _ = writeln!(out, "bio_max    Band {:.3}", p.bio_max);
                let _ = write!(out, "ore C/M/Y  Band {:.2} / {:.2} / {:.2}", p.ore[0], p.ore[1], p.ore[2]);
            }
            Some(Pick::Hull(id)) => {
                let view = self.view();
                let Some(h) = view.hulls.iter().find(|h| h.hull.id == id) else {
                    let _ = write!(out, "Hull {id} is not in the theater at this time.");
                    return out;
                };
                let name = |list: &[String], i: usize| list.get(i).cloned().unwrap_or_default();
                let row = &h.row;
                let speed = (row.vel[0].powi(2) + row.vel[1].powi(2) + row.vel[2].powi(2)).sqrt();
                let _ = writeln!(out, "Hull {id} · P{}", h.hull.owner);
                let (_, ops) = self.planned();
                let n = ops.iter().find_map(|o| match o {
                    Op::Hull { id: i, count, .. } if *i == id => Some(*count),
                    _ => None,
                });
                if let Some(n @ 2..) = n {
                    let _ = writeln!(out, "stack      {n} alike hulls here; this one shown");
                }
                let _ =
                    writeln!(out, "design     {} on {}", name(&r.designs, h.hull.design), name(&r.hulls, h.hull.hull));
                let _ = writeln!(out, "role       {}", name(&r.kinds, row.kind));
                let _ = writeln!(out, "mounts     {} beam, {} tube", h.hull.beams, h.hull.tubes);
                let _ = writeln!(out, "position   {:.3}, {:.3}, {:.3} ly", row.pos[0], row.pos[1], row.pos[2]);
                let _ = writeln!(out, "speed      {:.4} c", speed);
                let drive = match row.burn {
                    1 => "burning",
                    -1 => "braking",
                    _ => "coasting",
                };
                let _ = writeln!(out, "drive      {drive} at {:.3} g", row.accel / G);
                let _ = writeln!(
                    out,
                    "damage     {:.0}% of structure{}",
                    row.damage * 100.0,
                    if row.wrecked { " · WRECK" } else { "" }
                );
                let _ = writeln!(out, "cargo      {:.4} kt · settlers {:.4} kt", row.cargo, row.settlers);
                let _ = write!(out, "bound for  {}", row.dest.map_or("—".into(), |d| format!("world {d}")));
            }
        }
        out
    }

    /// The filtered log as tab-separated lines, `rows` of them, starting at
    /// row `first` — or, when `first` is `None`, ending at the clock's row so
    /// the log follows playback. The first line is a header:
    /// `count \t current row \t first row shown`. Each event line is
    /// `row \t t \t category \t seat \t kind \t text`.
    pub fn log(&self, first: Option<usize>, rows: usize) -> String {
        let sel = logview::select(&self.replay.events, &self.query, self.timeline.t);
        let current = logview::current(&self.replay.events, &sel, self.timeline.t);
        let start = first.unwrap_or_else(|| (current.map_or(0, |c| c + 1)).saturating_sub(rows)).min(sel.len());
        let mut out = format!("{}\t{}\t{}\n", sel.len(), current.map_or("-".into(), |c| c.to_string()), start);
        for (row, &i) in sel.iter().enumerate().skip(start).take(rows) {
            let e = &self.replay.events[i];
            let cat = self.replay.categories.get(e.category).map_or("?", String::as_str);
            let seat = e.seat.map_or("-".into(), |s| format!("P{s}"));
            let text = e.text.replace(['\t', '\n'], " ");
            let _ = writeln!(out, "{row}\t{:.3}\t{cat}\t{seat}\t{}\t{text}", e.t, e.kind);
        }
        out
    }

    /// Seeks to the time of filtered row `row`.
    pub fn seek_log_row(&mut self, row: usize) {
        let sel = logview::select(&self.replay.events, &self.query, self.timeline.t);
        if let Some(&i) = sel.get(row) {
            self.timeline.pause();
            self.timeline.seek(self.replay.events[i].t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logview::Window;
    use crate::replay::tests::TINY;

    fn viewer() -> Viewer {
        Viewer::load(TINY, 320.0, 200.0).unwrap()
    }

    #[test]
    fn a_loaded_replay_is_framed_paused_at_its_start_and_plays_in_a_minute() {
        let mut v = viewer();
        v.fit();
        assert_eq!((v.timeline.t, v.timeline.playing), (0.0, false));
        assert!((v.timeline.rate * OPENING_PLAY_SECONDS - 10.0).abs() < 1e-12);
        for p in &v.replay.planets {
            let s = v.camera.project(p.pos);
            assert!(s[0] >= FIT_MARGIN - 1e-9 && s[0] <= 320.0 - FIT_MARGIN + 1e-9);
        }
        assert!(Viewer::load("{}", 1.0, 1.0).is_err());
    }

    #[test]
    fn a_replay_with_a_focus_opens_on_it_and_fit_frames_everything() {
        let mut v = viewer();
        assert_eq!(v.camera.center, [20.0, 0.0], "TINY's focus");
        let near = v.camera.scale;
        v.fit();
        assert!(v.camera.scale < near);
        v.focus();
        assert_eq!(v.camera.scale, near);
    }

    #[test]
    fn rendering_fills_a_framebuffer_sized_to_the_mode() {
        let mut v = viewer();
        let r = v.render();
        assert_eq!((r.w, r.h), (160, 100), "tactical is a two-pixel grid");
        v.mode = Mode::Juicy;
        let r = v.render();
        assert_eq!((r.w, r.h), (320, 200));
        v.mode = Mode::Tactical;
        assert_eq!(v.render().w, 160, "and back");
    }

    #[test]
    fn a_pick_takes_the_nearest_hull_before_a_world_and_misses_far_off() {
        let mut v = viewer();
        v.timeline.seek(10.0);
        let hull = v.camera.project([21.0, 0.0, 0.0]);
        assert_eq!(v.pick(hull[0] + 2.0, hull[1]), Some(Pick::Hull(6)));
        let world = v.camera.project(v.replay.planets[2].pos);
        assert_eq!(v.pick(world[0], world[1]), Some(Pick::World(2)));
        assert_eq!(v.pick(-500.0, -500.0), None);
    }

    #[test]
    fn the_inspector_reads_the_selection() {
        let mut v = viewer();
        assert!(v.inspector().starts_with("Nothing selected"));
        v.selected = Some(Pick::Hull(5));
        let text = v.inspector();
        assert!(text.contains("Ford on MSV") && text.contains("Freighter") && text.contains("burning"), "{text}");
        v.timeline.seek(10.0);
        v.selected = Some(Pick::Hull(6));
        assert!(v.inspector().contains("WRECK"));
        v.selected = Some(Pick::World(2));
        let text = v.inspector();
        assert!(text.contains("owner      P0") && text.contains("ore C/M/Y  Band 2.50"), "{text}");
    }

    #[test]
    fn the_log_follows_the_clock_and_obeys_the_filter() {
        let mut v = viewer();
        v.timeline.seek(9.6);
        let log = v.log(None, 10);
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines[0], "3\t1\t0", "three events, the clock at row 1");
        assert_eq!(lines.len(), 4);
        assert!(lines[2].contains("HullWrecked") && lines[2].contains("Combat") && lines[2].contains("P1"));
        let log = v.log(None, 1);
        assert_eq!(log.lines().nth(1).unwrap().split('\t').next(), Some("1"), "the window ends at the clock's row");
        v.query = LogQuery { window: Window::UpToNow, text: "colony".into(), ..LogQuery::default() };
        assert_eq!(v.log(None, 10).lines().next(), Some("0\t-\t0"));
    }

    #[test]
    fn a_log_row_seeks_the_clock_to_its_event() {
        let mut v = viewer();
        v.timeline.play();
        v.seek_log_row(2);
        assert_eq!((v.timeline.t, v.timeline.playing), (9.9, false));
    }

    #[test]
    fn prepared_lights_are_what_the_cpu_renderer_draws() {
        let mut v = viewer();
        v.timeline.seek(10.0);
        let l = v.prepare_lights().clone();
        assert!(!l.scene.is_empty() && !l.territory.is_empty());
        v.mode = Mode::Juicy;
        let cpu = v.render().clone();
        let (mut hdr, mut out) = (Hdr::default(), Raster::default());
        juicy::rasterize(&l, 320, 200, &mut hdr, &mut out, &ToneLut::new());
        assert_eq!(cpu, out);
    }

    #[test]
    fn the_status_line_names_the_clock_frame_mode_and_level() {
        let mut v = viewer();
        v.timeline.seek(10.0);
        let s = v.status();
        assert!(s.contains("frame 2/2") && s.contains("Tactical") && s.contains("t 10.000 yr"), "{s}");
    }
}
