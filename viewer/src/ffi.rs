//! **The plain C interface the web shell calls** (`docs/Hyades_interface.md`
//! §8). No bindings generator: every function takes and returns numbers, and
//! bytes cross through two buffers the module owns.
//!
//! - **In:** [`hv_input`] returns a buffer of the asked length; the shell
//!   copies a replay or a filter string into it, then calls the function that
//!   reads it ([`hv_load`], [`hv_filter_text`], [`hv_filter_kind`]).
//! - **Out:** [`hv_render`] returns the framebuffer (RGBA, [`hv_frame_w`] ×
//!   [`hv_frame_h`]); [`hv_text`] returns UTF-8 text of [`hv_text_len`] bytes.
//!
//! A pointer is valid until the next call that writes the same buffer.

use crate::app::{Mode, Viewer};
use crate::logview::Window;
use crate::palette::{self, Palette, Settings};
use std::cell::RefCell;
use std::fmt::Write as _;

thread_local! {
    static VIEWER: RefCell<Option<Viewer>> = const { RefCell::new(None) };
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static TEXT: RefCell<String> = const { RefCell::new(String::new()) };
    static ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    /// The palette the author is tuning; kept across replays.
    static PALETTE: RefCell<Option<Palette>> = const { RefCell::new(None) };
}

fn palette() -> Palette {
    PALETTE.with(|p| p.borrow_mut().get_or_insert_with(Palette::default).clone())
}

fn with<R: Default>(f: impl FnOnce(&mut Viewer) -> R) -> R {
    VIEWER.with(|v| v.borrow_mut().as_mut().map(f).unwrap_or_default())
}

fn input() -> String {
    INPUT.with(|b| String::from_utf8_lossy(&b.borrow()).into_owned())
}

/// A buffer of `len` bytes for the shell to write into.
#[no_mangle]
pub extern "C" fn hv_input(len: usize) -> *mut u8 {
    INPUT.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        b.resize(len, 0);
        b.as_mut_ptr()
    })
}

/// Reads the replay in the input buffer onto a screen `w × h`. Returns 0, or
/// 1 with the reason in text 3.
#[no_mangle]
pub extern "C" fn hv_load(w: f64, h: f64) -> i32 {
    match Viewer::load(&input(), w, h) {
        Ok(mut v) => {
            v.palette = palette();
            VIEWER.with(|x| *x.borrow_mut() = Some(v));
            0
        }
        Err(e) => {
            ERROR.with(|x| *x.borrow_mut() = e);
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn hv_resize(w: f64, h: f64) {
    with(|v| v.resize(w, h))
}

#[no_mangle]
pub extern "C" fn hv_tick(dt: f64) {
    with(|v| v.tick(dt))
}

/// Draws and returns the framebuffer.
#[no_mangle]
pub extern "C" fn hv_render() -> *const u8 {
    with(|v| Some(v.render().px.as_ptr())).unwrap_or(std::ptr::null())
}

/// Works out the juicy mode's lights for the GPU renderer. Layer 0 is the
/// scene, layer 1 the territory; each is [`hv_lights_len`] lights of
/// [`crate::juicy::LIGHT_FLOATS`] `f32`s from [`hv_lights_ptr`]:
/// `x, y, radius` in screen pixels, then linear `r, g, b` times intensity.
#[no_mangle]
pub extern "C" fn hv_prepare_lights() {
    with(|v| {
        v.prepare_lights();
    })
}

#[no_mangle]
pub extern "C" fn hv_lights_ptr(layer: i32) -> *const f32 {
    with(|v| {
        let l = if layer == 1 { &v.lights.territory } else { &v.lights.scene };
        Some(l.as_ptr() as *const f32)
    })
    .unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn hv_lights_len(layer: i32) -> usize {
    with(|v| if layer == 1 { v.lights.territory.len() } else { v.lights.scene.len() })
}

/// The juicy mode's constants the GPU renderer must share with the CPU one:
/// 0 bloom downsample, 1 bloom weight, 2 exposure, 3 blur radius, 4 blur
/// passes.
#[no_mangle]
pub extern "C" fn hv_juicy_constant(which: i32) -> f64 {
    use crate::juicy::*;
    match which {
        0 => BLOOM_DOWNSAMPLE as f64,
        1 => BLOOM_WEIGHT as f64,
        2 => EXPOSURE as f64,
        3 => BLUR_RADIUS as f64,
        4 => BLUR_PASSES as f64,
        _ => 0.0,
    }
}

#[no_mangle]
pub extern "C" fn hv_frame_w() -> usize {
    with(|v| v.out.w)
}

#[no_mangle]
pub extern "C" fn hv_frame_h() -> usize {
    with(|v| v.out.h)
}

/// 0 tactical, 1 juicy.
#[no_mangle]
pub extern "C" fn hv_set_mode(m: i32) {
    with(|v| v.mode = if m == 1 { Mode::Juicy } else { Mode::Tactical })
}

#[no_mangle]
pub extern "C" fn hv_mode() -> i32 {
    with(|v| (v.mode == Mode::Juicy) as i32)
}

/// 1 plays, 0 pauses, anything else toggles.
#[no_mangle]
pub extern "C" fn hv_play(on: i32) {
    with(|v| match on {
        1 => v.timeline.play(),
        0 => v.timeline.pause(),
        _ => v.timeline.toggle(),
    })
}

#[no_mangle]
pub extern "C" fn hv_playing() -> i32 {
    with(|v| v.timeline.playing as i32)
}

/// Years per second; negative rewinds.
#[no_mangle]
pub extern "C" fn hv_set_rate(rate: f64) {
    with(|v| v.timeline.set_rate(rate))
}

#[no_mangle]
pub extern "C" fn hv_rate() -> f64 {
    with(|v| v.timeline.rate)
}

#[no_mangle]
pub extern "C" fn hv_seek(t: f64) {
    with(|v| v.timeline.seek(t))
}

#[no_mangle]
pub extern "C" fn hv_seek_fraction(f: f64) {
    with(|v| v.timeline.seek_fraction(f))
}

/// One frame forward (`dir > 0`) or back.
#[no_mangle]
pub extern "C" fn hv_step(dir: i32) {
    with(|v| {
        let frames: Vec<f64> = v.replay.frames.iter().map(|f| f.t).collect();
        v.timeline.step(&frames, dir)
    })
}

#[no_mangle]
pub extern "C" fn hv_time() -> f64 {
    with(|v| v.timeline.t)
}

/// Years between the replay's frames.
#[no_mangle]
pub extern "C" fn hv_frame_years() -> f64 {
    with(|v| v.replay.meta.frame_years)
}

#[no_mangle]
pub extern "C" fn hv_fraction() -> f64 {
    with(|v| v.timeline.fraction())
}

/// Zooms by `factor` about screen point `(x, y)`.
#[no_mangle]
pub extern "C" fn hv_zoom(x: f64, y: f64, factor: f64) {
    with(|v| v.camera.zoom_at([x, y], factor))
}

#[no_mangle]
pub extern "C" fn hv_pan(dx: f64, dy: f64) {
    with(|v| v.camera.pan(dx, dy))
}

/// Frames every world and hull.
#[no_mangle]
pub extern "C" fn hv_fit() {
    with(|v| v.fit())
}

/// Frames the replay's focus.
#[no_mangle]
pub extern "C" fn hv_focus() {
    with(|v| v.focus())
}

/// Selects what is within `radius` screen pixels of `(x, y)`; 1 when
/// something is.
#[no_mangle]
pub extern "C" fn hv_pick(x: f64, y: f64, radius: f64) -> i32 {
    with(|v| v.pick(x, y, radius).is_some() as i32)
}

/// Centers the camera on the selection, keeping the zoom.
#[no_mangle]
pub extern "C" fn hv_follow() {
    with(|v| {
        let view = v.view();
        let at = match v.selected {
            Some(crate::tactical::Pick::Hull(id)) => view.hulls.iter().find(|h| h.hull.id == id).map(|h| h.row.pos),
            Some(crate::tactical::Pick::World(id)) => v.replay.planets.get(id as usize).map(|p| p.pos),
            None => None,
        };
        if let Some(p) = at {
            v.camera.center = [p[0], p[1]];
        }
    })
}

/// Bit `i` admits log category `i`.
#[no_mangle]
pub extern "C" fn hv_filter_categories(mask: u32) {
    with(|v| v.query.categories = mask)
}

/// Bit `i` admits seat `i`; `unseated` admits events with no seat.
#[no_mangle]
pub extern "C" fn hv_filter_seats(mask: u32, unseated: i32) {
    with(|v| {
        v.query.seats = mask as u64;
        v.query.unseated = unseated != 0;
    })
}

/// 0 all, 1 up to now, 2 within `half` years of now.
#[no_mangle]
pub extern "C" fn hv_filter_window(kind: i32, half: f64) {
    with(|v| {
        v.query.window = match kind {
            1 => Window::UpToNow,
            2 => Window::Around(half),
            _ => Window::All,
        }
    })
}

/// The text filter, from the input buffer.
#[no_mangle]
pub extern "C" fn hv_filter_text() {
    let s = input();
    with(|v| v.query.text = s)
}

/// The kind filter, from the input buffer; empty admits every kind.
#[no_mangle]
pub extern "C" fn hv_filter_kind() {
    let s = input();
    with(|v| v.query.kind = s)
}

#[no_mangle]
pub extern "C" fn hv_log_seek(row: usize) {
    with(|v| v.seek_log_row(row))
}

/// **Sets the palette** from the settings text in the input buffer
/// ([`Settings`]'s text form) and redraws with it. Returns 0, or 1 with the
/// reason in text 3 and the palette unchanged.
#[no_mangle]
pub extern "C" fn hv_palette_set() -> i32 {
    match Settings::parse(&input()) {
        Ok(settings) => {
            let p = Palette::with(settings);
            PALETTE.with(|x| *x.borrow_mut() = Some(p.clone()));
            with(|v| v.palette = p);
            0
        }
        Err(e) => {
            ERROR.with(|x| *x.borrow_mut() = e);
            1
        }
    }
}

/// Text `which`, its length in [`hv_text_len`]:
///
/// | which | text |
/// |---|---|
/// | 0 | the status line |
/// | 1 | the inspector |
/// | 2 | `rows` log rows from `first` (`first < 0` follows the clock) — see [`Viewer::log`] |
/// | 3 | the last load error |
/// | 4 | the legend: one line per seat, `P# \t #rrggbb \t archetype` |
/// | 5 | the log's categories, one per line |
/// | 6 | the log's event kinds, one per line |
/// | 7 | the palette sheet, for the author's approval ([`palette_sheet`]) |
/// | 8 | the replay's label and seed |
/// | 9 | the palette's settings, in their text form |
/// | 10 | every drawn hull glyph's screen position — see [`Viewer::drawn`] |
/// | 11 | the role marks, for a legend: `role \t mark \t accent`, the mark nine `0`/`1` row-major |
#[no_mangle]
pub extern "C" fn hv_text(which: i32, first: i32, rows: usize) -> *const u8 {
    let s = match which {
        3 => ERROR.with(|e| e.borrow().clone()),
        7 => palette_sheet(&palette()),
        9 => palette().settings.to_string(),
        10 => with(|v| v.drawn()),
        11 => role_legend(&palette()),
        _ => with(|v| match which {
            0 => v.status(),
            1 => v.inspector(),
            2 => v.log(usize::try_from(first).ok(), rows),
            4 => v
                .replay
                .seats
                .iter()
                .enumerate()
                .map(|(i, s)| format!("P{i}\t{}\t{}\n", v.palette.seat(i).to_hex(), s.archetype))
                .collect(),
            5 => v.replay.categories.join("\n"),
            6 => crate::logview::kinds(&v.replay.events).join("\n"),
            8 => format!("{} · seed {} · {} seats", v.replay.meta.label, v.replay.meta.seed, v.replay.meta.seats),
            _ => String::new(),
        }),
    };
    TEXT.with(|t| {
        *t.borrow_mut() = s;
        t.borrow().as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn hv_text_len() -> usize {
    TEXT.with(|t| t.borrow().len())
}

/// **The palette sheet**, tab-separated, for the live editor and the approval
/// page: a header line `status \t settings`, then sections `source`, `role`,
/// `seat` and `status`, each line `section \t name \t #source \t #shown`.
pub fn palette_sheet(p: &Palette) -> String {
    let mut out = format!("{}\t{}\n", palette::PALETTE_STATUS, p.settings);
    for &(name, [r, g, b]) in palette::SOURCE.iter() {
        let src = crate::color::Rgb::from_ints(r, g, b);
        let _ = writeln!(out, "source\t{name}\t{}\t{}", src.to_hex(), p.get(name).to_hex());
    }
    let ro = &p.roles;
    for (name, c) in [
        ("ground", ro.ground),
        ("panel", ro.panel),
        ("grid", ro.grid),
        ("hex", ro.hex),
        ("text_dim", ro.text_dim),
        ("text", ro.text),
        ("text_bright", ro.text_bright),
        ("world", ro.world),
        ("world_dim", ro.world_dim),
        ("cyan", ro.cyan),
        ("magenta", ro.magenta),
        ("yellow", ro.yellow),
    ] {
        let _ = writeln!(out, "role\t{name}\t\t{}", c.to_hex());
    }
    for kind in ["Scout", "Colonizer", "Miner", "Freighter", "Picket", "Sentry", "Reserve", "Scrapped"] {
        let _ = writeln!(out, "role\t{kind}\t\t{}", p.role(kind).to_hex());
    }
    for i in 0..18 {
        let _ = writeln!(out, "seat\tP{i}\t\t{}", p.seat(i).to_hex());
    }
    for (s, hex) in palette::STATUS.iter() {
        let _ = writeln!(out, "status\t{}\t{hex}\t{}", palette::status_name(*s), p.status(*s).to_hex());
    }
    out
}

/// **The role marks** ([`crate::glyph::role_mark`]) with their accents, so
/// the page's legend shows the marks the renderer stamps.
fn role_legend(p: &Palette) -> String {
    let mut out = String::new();
    for role in ["Colonizer", "Miner", "Freighter", "Picket", "Sentry", "Scout", "Reserve"] {
        if let Some(m) = crate::glyph::role_mark(role) {
            let bits: String = m.iter().map(|&b| if b { '1' } else { '0' }).collect();
            let _ = writeln!(out, "{role}\t{bits}\t{}", p.role(role).to_hex());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::tests::TINY;

    #[test]
    fn the_role_legend_lists_each_mark_as_the_renderer_stamps_it() {
        let legend = text(11, 0, 0);
        assert_eq!(legend.lines().count(), 7);
        assert!(legend.contains("Picket\t101010101\t#"), "{legend}");
        assert!(legend.contains("Freighter\t000111000\t#"));
    }

    fn text(which: i32, first: i32, rows: usize) -> String {
        let p = hv_text(which, first, rows);
        let n = hv_text_len();
        String::from_utf8(unsafe { std::slice::from_raw_parts(p, n) }.to_vec()).unwrap()
    }

    fn put(s: &str) {
        let p = hv_input(s.len());
        unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len()) };
    }

    #[test]
    fn the_shell_loads_plays_renders_and_reads_text_through_the_c_interface() {
        put("not json");
        assert_eq!(hv_load(320.0, 200.0), 1);
        assert!(!text(3, 0, 0).is_empty(), "the reason is readable");
        put(TINY);
        assert_eq!(hv_load(320.0, 200.0), 0);
        let px = hv_render();
        assert!(!px.is_null());
        assert_eq!((hv_frame_w(), hv_frame_h()), (160, 100));
        hv_set_mode(1);
        hv_render();
        assert_eq!((hv_frame_w(), hv_mode()), (320, 1));
        hv_set_rate(5.0);
        hv_play(1);
        hv_tick(1.0);
        assert_eq!(hv_time(), 5.0);
        hv_step(1);
        assert_eq!((hv_time(), hv_playing()), (10.0, 0));
        assert!(text(0, 0, 0).contains("frame 2/2"));
        assert!(text(4, 0, 0).starts_with("P0\t#"));
        put("wrecked");
        hv_filter_text();
        assert!(text(2, -1, 10).starts_with("1\t"), "one event matches");
        hv_log_seek(0);
        assert_eq!(hv_time(), 9.5);
        assert_eq!(text(5, 0, 0).lines().count(), 7);
    }

    #[test]
    fn the_palette_is_set_from_text_kept_across_replays_and_refused_whole_when_wrong() {
        put("ink=0.1 Hit=#00ff00");
        assert_eq!(hv_palette_set(), 0);
        assert_eq!(
            text(9, 0, 0),
            "ink=0.1 paper=0.95 warm=0.92 cool=0.6 pull=0.3 anchors=38,78,118,228 fill=0.7 Hit=#00ff00"
        );
        put(TINY);
        assert_eq!(hv_load(320.0, 200.0), 0);
        assert!(text(7, 0, 0).contains("status\tHit\t#ff2d6f\t#00ff00"), "a replay opens in the tuned palette");
        hv_render();
        put("ink=7");
        assert_eq!(hv_palette_set(), 1);
        assert!(text(3, 0, 0).contains("ink"));
        assert!(text(9, 0, 0).starts_with("ink=0.1 "), "a refused setting changes nothing");
    }

    #[test]
    fn the_palette_sheet_lists_every_source_color_and_says_it_is_proposed() {
        let sheet = palette_sheet(&Palette::default());
        assert!(sheet.starts_with("proposed\t"));
        assert_eq!(sheet.lines().filter(|l| l.starts_with("source\t")).count(), 40);
        assert_eq!(sheet.lines().filter(|l| l.starts_with("status\t")).count(), palette::STATUS.len());
        assert_eq!(sheet.lines().filter(|l| l.starts_with("seat\t")).count(), 18);
    }
}
