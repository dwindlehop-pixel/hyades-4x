//! **The log view** (`docs/Hyades_interface.md` §4): the replay's events,
//! filtered by category, seat, kind, text and a time window around the
//! playback clock.

use crate::replay::Event;

/// Which events a time window admits, relative to the clock `now`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Window {
    /// Every event in the replay.
    All,
    /// Events at or before `now` — the log as it stood then.
    UpToNow,
    /// Events within this many years of `now`, either side.
    Around(f64),
}

/// A filter over the log. Every condition must hold.
#[derive(Clone, Debug, PartialEq)]
pub struct LogQuery {
    /// Bit `i` admits category `i` (the replay's `enums.category`).
    pub categories: u32,
    /// Bit `i` admits seat `i`.
    pub seats: u64,
    /// Admit events that name no seat.
    pub unseated: bool,
    /// An event kind to match exactly; empty matches every kind.
    pub kind: String,
    /// Text to find in the event's kind or text, ignoring case; empty matches.
    pub text: String,
    pub window: Window,
}

impl Default for LogQuery {
    fn default() -> Self {
        LogQuery {
            categories: u32::MAX,
            seats: u64::MAX,
            unseated: true,
            kind: String::new(),
            text: String::new(),
            window: Window::All,
        }
    }
}

impl LogQuery {
    pub fn admits(&self, e: &Event, now: f64, needle: &str) -> bool {
        let category = e.category < 32 && self.categories & (1 << e.category) != 0;
        let seat = match e.seat {
            Some(s) => s < 64 && self.seats & (1 << s) != 0,
            None => self.unseated,
        };
        let window = match self.window {
            Window::All => true,
            Window::UpToNow => e.t <= now,
            Window::Around(h) => (e.t - now).abs() <= h,
        };
        category
            && seat
            && window
            && (self.kind.is_empty() || e.kind == self.kind)
            && (needle.is_empty() || e.text.to_lowercase().contains(needle) || e.kind.to_lowercase().contains(needle))
    }
}

/// The indices of the events `q` admits at clock `now`, in log order.
pub fn select(events: &[Event], q: &LogQuery, now: f64) -> Vec<usize> {
    let needle = q.text.to_lowercase();
    (0..events.len()).filter(|&i| q.admits(&events[i], now, &needle)).collect()
}

/// The position in `selected` of the last event at or before `now` — the row
/// the log scrolls to as the clock moves. `None` when every selected event is
/// later.
pub fn current(events: &[Event], selected: &[usize], now: f64) -> Option<usize> {
    selected.partition_point(|&i| events[i].t <= now).checked_sub(1)
}

/// Every event kind in the log, sorted — the kind filter's choices.
pub fn kinds(events: &[Event]) -> Vec<String> {
    let mut k: Vec<String> = events.iter().map(|e| e.kind.clone()).collect();
    k.sort();
    k.dedup();
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(t: f64, category: usize, kind: &str, seat: Option<usize>, text: &str) -> Event {
        Event { t, category, kind: kind.into(), seat, text: text.into() }
    }

    fn log() -> Vec<Event> {
        vec![
            ev(1.0, 2, "VehicleSpawned", Some(0), "P0 builds a Spur"),
            ev(2.0, 6, "HullWrecked", Some(1), "P1 hull wrecked by P0"),
            ev(3.0, 2, "ColonyFounded", Some(0), "P0 founds a colony at planet#2"),
            ev(4.0, 5, "CardPlayed", None, "round 1 opens"),
            ev(5.0, 6, "MissileResolved", Some(2), "P2 salvo hits P1"),
        ]
    }

    #[test]
    fn the_default_query_admits_everything() {
        assert_eq!(select(&log(), &LogQuery::default(), 0.0), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn a_category_mask_admits_only_its_categories() {
        let q = LogQuery { categories: 1 << 6, ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![1, 4]);
    }

    #[test]
    fn a_seat_mask_admits_its_seats_and_unseated_events_are_a_separate_switch() {
        let q = LogQuery { seats: 1 << 0, ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![0, 2, 3]);
        let q = LogQuery { seats: 1 << 0, unseated: false, ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![0, 2]);
    }

    #[test]
    fn kind_matches_exactly_and_text_matches_any_case_in_text_or_kind() {
        let q = LogQuery { kind: "HullWrecked".into(), ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![1]);
        let q = LogQuery { text: "SPUR".into(), ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![0]);
        let q = LogQuery { text: "missile".into(), ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![4], "the kind is searched too");
    }

    #[test]
    fn the_window_follows_the_clock() {
        let q = LogQuery { window: Window::UpToNow, ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 2.5), vec![0, 1]);
        let q = LogQuery { window: Window::Around(1.0), ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 3.0), vec![1, 2, 3]);
    }

    #[test]
    fn conditions_combine() {
        let q = LogQuery { categories: 1 << 2, seats: 1, text: "colony".into(), ..LogQuery::default() };
        assert_eq!(select(&log(), &q, 0.0), vec![2]);
    }

    #[test]
    fn the_current_row_is_the_last_selected_event_at_or_before_the_clock() {
        let events = log();
        let sel = vec![0, 2, 4];
        assert_eq!(current(&events, &sel, 0.5), None);
        assert_eq!(current(&events, &sel, 3.0), Some(1));
        assert_eq!(current(&events, &sel, 4.9), Some(1));
        assert_eq!(current(&events, &sel, 9.0), Some(2));
    }

    #[test]
    fn kinds_are_listed_once_each_in_order() {
        let mut events = log();
        events.push(ev(6.0, 6, "HullWrecked", Some(0), "again"));
        assert_eq!(
            kinds(&events),
            vec!["CardPlayed", "ColonyFounded", "HullWrecked", "MissileResolved", "VehicleSpawned"]
        );
    }
}
