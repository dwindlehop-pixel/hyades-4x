//! **Playback** (`docs/Hyades_interface.md` §3): a clock over a replay's span
//! that plays at a signed rate, so a negative rate rewinds, seeks anywhere in
//! the span, and steps frame by frame.

/// Years of game time per second of wall time a replay opens at.
pub const DEFAULT_RATE: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timeline {
    /// The first frame's time, years.
    pub t0: f64,
    /// The last frame's time, years.
    pub t1: f64,
    /// The time shown, years.
    pub t: f64,
    /// Years of game time per wall-clock second; negative rewinds.
    pub rate: f64,
    pub playing: bool,
}

impl Timeline {
    /// Paused at `t0`.
    pub fn new(t0: f64, t1: f64, rate: f64) -> Timeline {
        Timeline { t0, t1: t1.max(t0), t: t0, rate, playing: false }
    }

    /// Plays from where it stands. At the end it is heading for, it starts
    /// over from the other end, so play after a finished run replays it.
    pub fn play(&mut self) {
        if self.rate >= 0.0 && self.t >= self.t1 {
            self.t = self.t0;
        } else if self.rate < 0.0 && self.t <= self.t0 {
            self.t = self.t1;
        }
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn toggle(&mut self) {
        if self.playing {
            self.pause()
        } else {
            self.play()
        }
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate;
    }

    /// Moves to `t`, held inside the span.
    pub fn seek(&mut self, t: f64) {
        self.t = t.clamp(self.t0, self.t1);
    }

    /// Advances by `dt` wall-clock seconds at the rate. Reaching either end
    /// stops there and pauses.
    pub fn tick(&mut self, dt: f64) {
        if !self.playing {
            return;
        }
        let t = self.t + self.rate * dt;
        if t >= self.t1 || t <= self.t0 {
            self.playing = false;
        }
        self.seek(t);
    }

    /// Moves to the next frame time after `t` (`dir > 0`) or the last one
    /// before it (`dir < 0`), and pauses. At an end it stays.
    pub fn step(&mut self, frames: &[f64], dir: i32) {
        self.playing = false;
        let next = if dir > 0 {
            frames.iter().copied().find(|&f| f > self.t)
        } else {
            frames.iter().rev().copied().find(|&f| f < self.t)
        };
        if let Some(f) = next {
            self.seek(f);
        }
    }

    /// Where `t` sits in the span, 0 to 1 — what a scrubber shows.
    pub fn fraction(&self) -> f64 {
        if self.t1 > self.t0 {
            (self.t - self.t0) / (self.t1 - self.t0)
        } else {
            0.0
        }
    }

    pub fn seek_fraction(&mut self, f: f64) {
        self.seek(self.t0 + f.clamp(0.0, 1.0) * (self.t1 - self.t0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tl() -> Timeline {
        Timeline::new(0.0, 100.0, 10.0)
    }

    #[test]
    fn a_timeline_opens_paused_at_its_start() {
        let mut t = tl();
        assert_eq!((t.t, t.playing), (0.0, false));
        t.tick(1.0);
        assert_eq!(t.t, 0.0, "a paused clock does not move");
    }

    #[test]
    fn playing_advances_by_rate_times_wall_time() {
        let mut t = tl();
        t.play();
        t.tick(0.5);
        assert_eq!(t.t, 5.0);
        t.set_rate(40.0);
        t.tick(0.25);
        assert_eq!(t.t, 15.0);
    }

    #[test]
    fn a_negative_rate_rewinds() {
        let mut t = tl();
        t.seek(50.0);
        t.set_rate(-20.0);
        t.play();
        t.tick(1.0);
        assert_eq!(t.t, 30.0);
    }

    #[test]
    fn playback_stops_and_pauses_at_either_end() {
        let mut t = tl();
        t.play();
        t.tick(100.0);
        assert_eq!((t.t, t.playing), (100.0, false));
        t.set_rate(-10.0);
        t.play();
        t.tick(100.0);
        assert_eq!((t.t, t.playing), (0.0, false));
    }

    #[test]
    fn play_at_the_end_it_heads_for_starts_over() {
        let mut t = tl();
        t.seek(100.0);
        t.play();
        assert_eq!((t.t, t.playing), (0.0, true));
        t.pause();
        t.seek(0.0);
        t.set_rate(-10.0);
        t.play();
        assert_eq!(t.t, 100.0, "rewinding from the start replays from the end");
    }

    #[test]
    fn seek_is_held_inside_the_span() {
        let mut t = tl();
        t.seek(-5.0);
        assert_eq!(t.t, 0.0);
        t.seek(250.0);
        assert_eq!(t.t, 100.0);
        t.seek(42.0);
        assert_eq!(t.t, 42.0);
    }

    #[test]
    fn step_lands_on_the_neighboring_frame_and_pauses() {
        let frames = [0.0, 10.0, 20.0, 30.0];
        let mut t = Timeline::new(0.0, 30.0, 10.0);
        t.play();
        t.seek(12.0);
        t.step(&frames, 1);
        assert_eq!((t.t, t.playing), (20.0, false));
        t.step(&frames, 1);
        assert_eq!(t.t, 30.0);
        t.step(&frames, 1);
        assert_eq!(t.t, 30.0, "no frame after the last");
        t.seek(12.0);
        t.step(&frames, -1);
        assert_eq!(t.t, 10.0);
        t.step(&frames, -1);
        assert_eq!(t.t, 0.0, "from a frame time, back goes to the frame before");
        t.step(&frames, -1);
        assert_eq!(t.t, 0.0);
    }

    #[test]
    fn a_scrubber_fraction_round_trips() {
        let mut t = Timeline::new(20.0, 60.0, 1.0);
        t.seek_fraction(0.25);
        assert_eq!(t.t, 30.0);
        assert_eq!(t.fraction(), 0.25);
        assert_eq!(Timeline::new(5.0, 5.0, 1.0).fraction(), 0.0, "a one-frame replay");
    }
}
