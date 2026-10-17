//! **The viewer** (`docs/Hyades_interface.md` §2): one replay, its playback
//! clock, its camera, the log filter, the selection and the framebuffer. The
//! web shell drives it through [`crate::ffi`]; everything it shows as text it
//! reads from here, so the shell formats nothing.

use crate::camera::Camera;
use crate::juicy::{self, Hdr, Lights, ToneLut, Tween};
use crate::logview::{self, LogQuery};
use crate::palette::Palette;
use crate::raster::Raster;
use crate::replay::{Replay, View};
use crate::tactical::{self, GlyphCache, Op, Pick, Scene};
use crate::timeline::Timeline;
use std::fmt::Write as _;

/// A replay opens at a rate that plays it through in this many seconds…
pub const OPENING_PLAY_SECONDS: f64 = 60.0;
/// …or slower, so that no hull crosses more than this many screen pixels a
/// wall second at the zoom it opens on (T-184, proposed): a hull at light
/// speed, the fastest any can fly, then moves 4 pixels a frame at 60 frames
/// a second. Faster, a hull jumps across the screen from frame to frame.
pub const OPENING_SCREEN_SPEED: f64 = 240.0;
/// Light speed, ly/yr: no hull flies faster.
const C: f64 = 1.0;
/// A pick with a mouse takes the nearest entity within this many screen
/// pixels; a touch passes a wider radius, a fingertip's.
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
    /// The juicy hulls' eased offsets (T-184); the wall seconds ticked, and
    /// the wall time they were last eased to; the game time they were last
    /// eased at, and the game time played since by ticks.
    tween: Tween,
    wall: f64,
    eased_at: f64,
    eased_t: f64,
    played: f64,
}

impl Viewer {
    /// Reads a replay and frames the whole galaxy on a screen `w × h`.
    pub fn load(json: &str, w: f64, h: f64) -> Result<Viewer, String> {
        let replay = Replay::from_json(json)?;
        // A live run (T-183) spans its horizon; it has arrived to its last frame.
        let t0 = replay.t0();
        let t1 = if replay.live { replay.meta.horizon_years.max(replay.t1()) } else { replay.t1() };
        let mut timeline = Timeline::new(t0, t1, 1.0);
        timeline.arrived(replay.t1(), !replay.live);
        let mut v = Viewer {
            timeline,
            camera: Camera::new(w, h),
            palette: Palette::default(),
            mode: Mode::Tactical,
            query: LogQuery::default(),
            selected: None,
            out: Raster::new(1, 1),
            lights: Lights::default(),
            hdr: Hdr::new(1, 1),
            lut: ToneLut::new(Palette::default().roles.ink),
            glyphs: GlyphCache::default(),
            tween: Tween::default(),
            wall: 0.0,
            eased_at: 0.0,
            eased_t: f64::NAN,
            played: 0.0,
            replay,
        };
        v.focus();
        v.timeline.set_rate(v.opening_rate());
        Ok(v)
    }

    /// **The rate a replay opens at**: the whole run in
    /// [`OPENING_PLAY_SECONDS`], or slower, so that a hull at light speed
    /// crosses no more than [`OPENING_SCREEN_SPEED`] pixels a second at the
    /// current zoom (T-184).
    pub fn opening_rate(&self) -> f64 {
        let whole = (self.timeline.t1 - self.timeline.t0) / OPENING_PLAY_SECONDS;
        let legible = OPENING_SCREEN_SPEED / (self.camera.scale * C);
        whole.min(legible).max(1e-6)
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

    /// **Appends what a live run added** ([`Replay::append`], T-183) and lets
    /// the clock play on to it.
    pub fn append(&mut self, chunk: &str) -> Result<(), String> {
        self.replay.append(chunk)?;
        self.timeline.arrived(self.replay.t1(), !self.replay.live);
        Ok(())
    }

    pub fn resize(&mut self, w: f64, h: f64) {
        self.camera.resize(w, h);
    }

    /// `dt` wall seconds have passed: the clock plays on if it is playing,
    /// and the juicy mode's lights ease on whether or not it is.
    pub fn tick(&mut self, dt: f64) {
        let before = self.timeline.t;
        self.timeline.tick(dt);
        self.played += self.timeline.t - before;
        self.wall += dt;
    }

    /// Whether a juicy light is still easing to its place (T-184): the next
    /// frame differs with nothing else changed, so the shell draws it.
    pub fn animating(&self) -> bool {
        self.mode == Mode::Juicy && self.tween.moving()
    }

    /// The juicy mode's lights at the current instant, each hull's eased on
    /// by the wall time since they were last worked out — unless the clock
    /// moved by anything but playing (a seek, a step, a log row), which is not
    /// motion: then every light is where its layout puts it, with no trail.
    fn eased_lights(&mut self, view: &View, ops: &[Op]) -> Lights {
        let jumped = self.eased_t.is_nan() || (self.timeline.t - self.eased_t - self.played).abs() > 1e-9;
        let dt = if jumped { f64::INFINITY } else { self.wall - self.eased_at };
        (self.eased_at, self.eased_t, self.played) = (self.wall, self.timeline.t, 0.0);
        let mut tween = std::mem::take(&mut self.tween);
        let lights = juicy::lights_eased(&self.scene(view), ops, &mut tween, dt);
        self.tween = tween;
        lights
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
        Scene {
            replay: &self.replay,
            view,
            camera: &self.camera,
            palette: &self.palette,
            selected: self.selected,
            rate: self.timeline.rate.abs(),
        }
    }

    /// Draws the current instant in the current mode into [`Self::out`].
    pub fn render(&mut self) -> &Raster {
        // The juicy background is the palette's ink, which the editor moves.
        if self.mode == Mode::Juicy && self.lut.ground() != juicy::linear(self.palette.roles.ink) {
            self.lut = ToneLut::new(self.palette.roles.ink);
        }
        let (view, ops) = self.planned();
        let lights = (self.mode == Mode::Juicy).then(|| self.eased_lights(&view, &ops));
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
            Mode::Juicy => {
                let (w, h) = juicy::frame_size(&self.camera);
                juicy::rasterize(&lights.unwrap_or_default(), w, h, &mut hdr, &mut out, &self.lut);
            }
        }
        (self.out, self.hdr, self.glyphs) = (out, hdr, glyphs);
        &self.out
    }

    /// Every drawn hull glyph or marker as `id \t x \t y \t count`, in screen
    /// pixels at the glyph's center — where a pick at that point lands.
    pub fn drawn(&self) -> String {
        let (_, ops) = self.planned();
        let mut out = String::new();
        for op in &ops {
            if let Op::Hull { id, at, count: count @ 1.., .. } = op {
                let p = tactical::PIXEL;
                let _ = writeln!(out, "{id}\t{}\t{}\t{count}", (at[0] as f64 + 0.5) * p, (at[1] as f64 + 0.5) * p);
            }
        }
        out
    }

    /// Works out the juicy mode's lights at the current instant into
    /// [`Self::lights`], for a renderer outside the module to draw.
    pub fn prepare_lights(&mut self) -> &Lights {
        let (view, ops) = self.planned();
        self.lights = self.eased_lights(&view, &ops);
        &self.lights
    }

    /// Selects the drawn hull, else the world, nearest screen point `(x, y)`
    /// within `radius` screen pixels — of glyphs at one distance, the one drawn
    /// on top; clears the selection when nothing is that near.
    pub fn pick(&mut self, x: f64, y: f64, radius: f64) -> Option<Pick> {
        let (_, ops) = self.planned();
        let near = |s: [f64; 2]| (s[0] - x).hypot(s[1] - y);
        let p = tactical::PIXEL;
        // Drawn hulls bottom first, so of equal distances the last wins.
        let hull = tactical::drawing_order(&ops)
            .into_iter()
            .filter_map(|i| match &ops[i] {
                Op::Hull { id, at, .. } => Some((near([(at[0] as f64 + 0.5) * p, (at[1] as f64 + 0.5) * p]), *id)),
                Op::World { .. } => None,
            })
            .filter(|(d, _)| *d <= radius)
            .reduce(|a, b| if b.0 <= a.0 { b } else { a })
            .map(|(_, id)| Pick::Hull(id));
        let world = || {
            ops.iter()
                .filter_map(|op| match op {
                    Op::World { id, s, .. } => Some((near(*s), Pick::World(*id))),
                    Op::Hull { .. } => None,
                })
                .filter(|(d, _)| *d <= radius)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, p)| p)
        };
        self.selected = hull.or_else(world);
        self.selected
    }

    /// One line: the clock, the frame, the rate, the level of detail, the mode.
    pub fn status(&self) -> String {
        let t = &self.timeline;
        format!(
            "t {:.3} yr of {:.3} · frame {}/{}{} · {}{:.3} yr/s · {:?} · {:?}{}",
            t.t,
            t.t1,
            self.replay.frame_at(t.t) + 1,
            self.replay.frames.len(),
            if self.replay.live { format!(" · simulated to {:.0} yr", t.ready) } else { String::new() },
            if t.waiting() {
                "⧗ "
            } else if t.playing {
                "▶ "
            } else {
                "❚❚ "
            },
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
                let _ = write!(out, "ore (Band):");
                for (m, b) in p.ore.iter().enumerate() {
                    let _ = write!(out, "\n  {:<15}{b:.2}", crate::replay::MATERIALS[m]);
                }
                let first = f.holdings.partition_point(|h| h.planet < id);
                for h in f.holdings[first..].iter().take_while(|h| h.planet == id) {
                    let _ = write!(out, "\nheld by P{} (Band, cost ladder):", h.seat);
                    for (m, b) in h.bands.iter().enumerate() {
                        if let Some(b) = b {
                            let _ = write!(out, "\n  {:<15}{b:.2}", crate::replay::MATERIALS[m]);
                        }
                    }
                }
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
                // The makeup of the place the hull's glyph tops, or of its stack.
                let members = ops.iter().find_map(|o| match o {
                    Op::Hull { id: i, place, .. } if *i == id && place.len() > 1 => Some(place.clone()),
                    Op::Hull { id: i, members, .. } if *i == id && members.len() > 1 => Some(members.clone()),
                    _ => None,
                });
                if let Some(members) = members {
                    let _ = writeln!(out, "here       {} hulls:", members.len());
                    let mut kinds: std::collections::BTreeMap<(String, String, String), usize> = Default::default();
                    for m in &members {
                        if let Some(h) = view.hulls.iter().find(|h| h.hull.id == *m) {
                            let k = (
                                name(&r.designs, h.hull.design),
                                name(&r.hulls, h.hull.hull),
                                name(&r.kinds, h.row.kind),
                            );
                            *kinds.entry(k).or_default() += 1;
                        }
                    }
                    for ((design, hull, role), n) in kinds {
                        let _ = writeln!(out, "  {n:>4} × {design} on {hull}, {role}");
                    }
                    let _ = writeln!(out, "shown      hull {id}:");
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
                for (m, kt) in row.mix.iter().enumerate().filter(|(_, kt)| **kt > 0.0) {
                    let _ = writeln!(out, "  {:<15}{kt:.4} kt", crate::replay::MATERIALS[m]);
                }
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
        assert_eq!(v.pick(hull[0] + 2.0, hull[1], PICK_RADIUS), Some(Pick::Hull(6)));
        let world = v.camera.project(v.replay.planets[2].pos);
        assert_eq!(v.pick(world[0], world[1], PICK_RADIUS), Some(Pick::World(2)));
        assert_eq!(v.pick(-500.0, -500.0, PICK_RADIUS), None);
    }

    #[test]
    fn a_wider_radius_picks_what_a_mouse_radius_misses_and_a_stack_lists_its_makeup() {
        let mut v = viewer();
        v.timeline.seek(10.0);
        let hull = v.camera.project([21.0, 0.0, 0.0]);
        assert_eq!(v.pick(hull[0] + 20.0, hull[1], PICK_RADIUS), None);
        assert_eq!(v.pick(hull[0] + 20.0, hull[1], 24.0), Some(Pick::Hull(6)), "a fingertip's radius");
        let drawn = v.drawn();
        let row: Vec<&str> = drawn.lines().find(|l| l.starts_with("6\t")).unwrap().split('\t').collect();
        let (x, y): (f64, f64) = (row[1].parse().unwrap(), row[2].parse().unwrap());
        v.selected = None;
        assert_eq!(v.pick(x, y, PICK_RADIUS), Some(Pick::Hull(6)), "a drawn position picks its hull");
    }

    #[test]
    fn the_inspector_reads_the_selection() {
        let mut v = viewer();
        assert!(v.inspector().starts_with("Nothing selected"));
        v.selected = Some(Pick::Hull(5));
        let text = v.inspector();
        assert!(text.contains("Ford on MSV") && text.contains("Freighter") && text.contains("burning"), "{text}");
        assert!(
            text.contains("Cage Ice       3.0000 kt") && text.contains("Voltslate      1.5000 kt"),
            "cargo by material: {text}"
        );
        v.timeline.seek(10.0);
        v.selected = Some(Pick::Hull(6));
        assert!(v.inspector().contains("WRECK"));
        v.selected = Some(Pick::World(2));
        let text = v.inspector();
        assert!(text.contains("owner      P0") && text.contains("Cage Ice       2.50"), "{text}");
        v.timeline.seek(0.0);
        v.selected = Some(Pick::World(2));
        let text = v.inspector();
        assert!(
            text.contains("held by P1") && text.contains("Cage Ice       -0.40"),
            "a rival's holding at a rock: {text}"
        );
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

    /// **A replay opens at a rate no hull outruns on screen** (T-184): the
    /// whole run in a minute, or slower if a hull at light speed would cross
    /// more than `OPENING_SCREEN_SPEED` pixels a second at the opening zoom.
    #[test]
    fn a_replay_opens_at_a_rate_no_hull_outruns_on_screen() {
        let mut v = viewer();
        let whole = (v.timeline.t1 - v.timeline.t0) / OPENING_PLAY_SECONDS;
        assert_eq!((v.opening_rate(), v.timeline.rate), (whole, whole), "framed wide, the whole run in a minute");
        v.camera.scale *= 1000.0;
        let legible = OPENING_SCREEN_SPEED / v.camera.scale;
        assert!(
            legible < whole && v.opening_rate() == legible,
            "framed close, no faster than light crosses the screen"
        );
    }

    /// **A seek is not motion** (T-184): after the clock jumps, every juicy
    /// light is where its layout puts it and none trails, though wall time
    /// has passed for an easing to run on.
    #[test]
    fn after_a_seek_the_juicy_lights_are_where_their_layout_puts_them() {
        let mut v = viewer();
        v.mode = Mode::Juicy;
        v.timeline.seek(5.0);
        v.prepare_lights();
        v.tick(1.0 / 60.0);
        v.timeline.seek(15.0);
        let eased = v.prepare_lights().clone();
        let (view, ops) = v.planned();
        assert_eq!(eased, juicy::lights(&v.scene(&view), &ops), "a seek snaps");
        assert!(!v.animating());
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
        juicy::rasterize(&l, 320, 200, &mut hdr, &mut out, &ToneLut::new(Palette::default().roles.ink));
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
