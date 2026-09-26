//! **Fights on the main event loop** (T-133, `Hyades_warfare_tree.md` §8.19).
//!
//! There are no engagements and no fight sites (the author's ruling). A hull is
//! fired on wherever its trajectory takes it within the fire distance of a hull
//! whose Doctrine fires on it, and fire is a sequence of events interleaved with
//! every production decision, mining tick and arrival in the galaxy:
//!
//! | event | when | what it does |
//! |---|---|---|
//! | [`EventKind::EncounterBegin`] | a hull enters a shooter's fire distance — found by detection when either starts a trajectory or takes station | adds it to the shooter's reach and starts the shooter's discharges |
//! | [`EventKind::Discharge`] | every Design period while anything it fires on is in reach | fire control against the target's position now; energy for one period if it holds; the wreck check |
//! | [`EventKind::ThreatSeen`] | light from an armed hull's approach reaches a hull it will fire on (belief event A) | the target's fleet decides |
//! | a hit (inside a discharge) | an enemy fired on this fleet (belief event B) | the target's fleet decides |
//!
//! A decision is a **course change** from where the hull is and how it is
//! moving ([`Motion::come_to_rest`]): a colony ship that believes it would be
//! wrecked before founding seeks a new destination, a hull past its structure
//! or off a post it cannot hold under fire heads home, and a hull that returns
//! fire on an enemy stands.
use super::*;
use crate::combat::{StationKeeping, STATION_PERIOD, STATION_RADIUS};

/// Fork label for each hull's wreck-point draw (T-133), XORed with a label from
/// the entity so it cannot coincide with an encounter's fork.
const WRECK_DRAW_LABEL: u64 = 0x5752_4543_4B5F_5054;

/// Fork label for each hull's station-keeping draw (T-133).
const STATION_LABEL: u64 = 0x5354_4154_494F_4E00;

/// **How far past its reference trajectory a hull can be**: two
/// station-keeping radii, one for each end of a shot. Detection runs on
/// reference trajectories and widens the fire distance by this, so a hull
/// whose station-keeping carries it into reach is never missed.
const STATION_MARGIN: f64 = 2.0 * STATION_RADIUS.1;

/// **Steps one detection between two moving hulls may take on one event**
/// before it resumes on an [`EventKind::EncounterSeek`].
const SEEK_STEPS: usize = 96;

/// **Which trajectory a hull is on** (T-133). `gen` changes on every course, so
/// an event predicted from an earlier one can tell it is stale; `since` and
/// `at` are where and when the current one began — the light a distant
/// observer sees it by.
#[derive(Clone, Copy, Debug)]
pub(super) struct Track {
    pub(super) gen: u32,
    pub(super) since: f64,
    pub(super) at: Vec3,
    /// The box every position on this trajectory lies in — the corners of
    /// its stretches. Detection rejects a pair whose boxes, widened by the
    /// shooter's reach, do not meet, before it searches for an entry time.
    lo: Vec3,
    hi: Vec3,
}

/// The box around every position a trajectory takes: each stretch is a
/// straight segment, so its end points bound it.
fn bounds(m: &Motion) -> (Vec3, Vec3) {
    let mut pts = [m.origin, m.dest, m.origin, m.origin];
    if let Some(b) = m.brake {
        pts[2] = b.from;
        pts[3] = b.stop();
    }
    let (mut lo, mut hi) = (pts[0], pts[0]);
    for p in &pts[1..] {
        lo = Vec3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
        hi = Vec3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
    }
    (lo, hi)
}

/// Whether two boxes come within `reach` of each other on every axis — the
/// necessary condition for two trajectories inside them to come within
/// `reach`, so a `false` is exact.
fn boxes_within(a: &Track, b: &Track, reach: f64) -> bool {
    let gap = |alo: f64, ahi: f64, blo: f64, bhi: f64| (blo - ahi).max(alo - bhi);
    gap(a.lo.x, a.hi.x, b.lo.x, b.hi.x) <= reach
        && gap(a.lo.y, a.hi.y, b.lo.y, b.hi.y) <= reach
        && gap(a.lo.z, a.hi.z, b.lo.z, b.hi.z) <= reach
}

/// **Who decides together** (T-133, R-WAR30 and R-WAR32's interim): a mining
/// crew at one rock, a picket stack at one world or port, or a single hull.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum FleetKey {
    Hull(u64),
    Crew(u32, u64),
    Stack(u32, u64),
}

/// One straight stretch of a trajectory, and how to read the time the hull is
/// `x` ly along it.
#[derive(Clone, Copy, Debug)]
struct Stretch {
    from: Vec3,
    dir: Vec3,
    length: f64,
    /// When the hull is first at `from` — `−∞` for a leg flown from rest,
    /// which it waits at until it departs.
    present_from: f64,
    /// When it leaves this stretch; `+∞` for the last, where it stays.
    leaves: f64,
    kind: StretchKind,
}

#[derive(Clone, Copy, Debug)]
enum StretchKind {
    /// Braking to rest at `end`.
    Brake { end: f64, accel: f64 },
    /// A flip-and-burn from `depart` to `arrive`.
    Leg { depart: f64, arrive: f64, accel: f64 },
}

impl Stretch {
    /// The time the hull is `x` ly along this stretch. Closed form in both
    /// kinds ([`math::accel_leg_time`]), so exact on every target.
    fn time_at(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, self.length);
        match self.kind {
            StretchKind::Brake { end, accel } => end - math::accel_leg_time(accel, self.length - x),
            StretchKind::Leg { depart, arrive, accel } => {
                if x <= 0.5 * self.length {
                    depart + math::accel_leg_time(accel, x)
                } else {
                    arrive - math::accel_leg_time(accel, self.length - x)
                }
            }
        }
    }
}

/// The straight stretches of a trajectory, in order.
fn stretches(m: &Motion) -> [Option<Stretch>; 2] {
    let brake = m.brake.map(|b| Stretch {
        from: b.from,
        dir: b.dir,
        length: b.length,
        present_from: b.start,
        leaves: b.end,
        kind: StretchKind::Brake { end: b.end, accel: b.accel },
    });
    let length = m.origin.distance(m.dest);
    let leg = Stretch {
        from: m.origin,
        dir: m.dest.sub(m.origin).normalized(),
        length,
        present_from: m.brake.map_or(f64::NEG_INFINITY, |b| b.end),
        leaves: f64::INFINITY,
        kind: StretchKind::Leg { depart: m.depart, arrive: m.arrive, accel: m.accel },
    };
    [brake, Some(leg)]
}

/// **The first time at or after `from` a hull flying `m` is within `reach` of
/// the fixed point `p`**, or `None` if it never is on this trajectory.
///
/// Along one straight stretch the distance to a point is quasiconvex in the
/// distance flown, and the distance flown is monotone in time, so the set of
/// times in reach is one interval per stretch: its ends come from the line's
/// intersection with the sphere of radius `reach`, and the times from the
/// inverse of the flight distance. No stepping, no tolerance.
fn entry_against_point(m: &Motion, p: Vec3, reach: f64, from: f64) -> Option<f64> {
    for s in stretches(m).into_iter().flatten() {
        if s.leaves < from {
            continue;
        }
        let w = p.sub(s.from);
        if s.length <= 0.0 {
            if w.norm() <= reach {
                return Some(s.present_from.max(from));
            }
            continue;
        }
        let along = w.dot(s.dir);
        let off2 = (w.dot(w) - along * along).max(0.0);
        if off2 > reach * reach {
            continue;
        }
        let half = (reach * reach - off2).sqrt();
        let (x_in, x_out) = (along - half, along + half);
        if x_out < 0.0 || x_in > s.length {
            continue;
        }
        let t_in = if x_in <= 0.0 { s.present_from } else { s.time_at(x_in) };
        let t_out = if x_out >= s.length { s.leaves } else { s.time_at(x_out) };
        if t_out < from {
            continue;
        }
        return Some(t_in.max(from));
    }
    None
}

/// Shortest distance between two segments `[a0, a1]` and `[b0, b1]`
/// (the closest-points construction for segments).
fn segment_distance(a0: Vec3, a1: Vec3, b0: Vec3, b1: Vec3) -> f64 {
    let d1 = a1.sub(a0);
    let d2 = b1.sub(b0);
    let r = a0.sub(b0);
    let (a, e, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    let (mut s, mut t);
    if a <= 1e-300 && e <= 1e-300 {
        return r.norm();
    }
    if a <= 1e-300 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= 1e-300 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let denom = a * e - b * b;
            s = if denom > 0.0 { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
            t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
        }
    }
    a0.add(d1.scale(s)).distance(b0.add(d2.scale(t)))
}

/// A lower bound on how close two trajectories ever come: the least distance
/// between their stretches. Every position either hull takes lies on one.
fn closest_approach_bound(a: &Motion, b: &Motion) -> f64 {
    let mut best = f64::INFINITY;
    for sa in stretches(a).into_iter().flatten() {
        for sb in stretches(b).into_iter().flatten() {
            let d = segment_distance(
                sa.from,
                sa.from.add(sa.dir.scale(sa.length)),
                sb.from,
                sb.from.add(sb.dir.scale(sb.length)),
            );
            best = best.min(d);
        }
    }
    best
}

/// What a detection found.
enum Entry {
    At(f64),
    /// Still searching at this time; resume on an [`EventKind::EncounterSeek`].
    Resume(f64),
    Never,
}

use crate::autopilot::UnderFire;
use crate::log::CourseReason;

impl Simulation {
    /// **Is this hull still in play?** Built, owned, placed, not wrecked or
    /// broken up.
    pub(super) fn live_hull(&self, e: Entity) -> bool {
        self.world.hull_type.contains(e)
            && self.world.owner.contains(e)
            && self.world.motion.contains(e)
            && self.world.role.get(e).copied() != Some(Role::Scrapped)
    }

    fn gen_of(&self, e: Entity) -> u32 {
        self.world.track.get(e).map_or(0, |t| t.gen)
    }

    /// **Does the arrival scheduled for `t` still belong to this hull's leg?**
    pub(super) fn arrival_is_current(&self, e: Entity, t: f64) -> bool {
        self.world.role.get(e).copied() != Some(Role::Scrapped)
            && self.world.motion.get(e).is_none_or(|m| m.arrive == t)
    }

    /// **A hull's trajectory changed** (T-133): it starts a leg, takes station
    /// or changes course. Every such moment re-runs detection for it, which is
    /// §4's rule — a hull's situation changes when its trajectory does, and
    /// only then.
    pub(super) fn track_changed(&mut self, e: Entity) {
        let gen = self.gen_of(e).wrapping_add(1);
        let at = self.position_at(e, self.clock).unwrap_or(Vec3::ZERO);
        let (lo, hi) = self.world.motion.get(e).map_or((at, at), bounds);
        self.world.track.insert(e, Track { gen, since: self.clock, at, lo, hi });
        self.fired_on_at_destination.remove(&e);
        if self.world.loadout.get(e).is_some_and(|l| l.is_armed()) && self.live_hull(e) {
            self.armed.insert(e);
        }
        self.detect(e);
    }

    /// **How far `shooter` fires on `target`**, or `None` where its standing
    /// layer holds fire or they are one empire (`Standing::fire_distance`).
    pub(super) fn fire_reach(&self, shooter: Entity, target: Entity) -> Option<f64> {
        let (&os, &ot) = (self.world.owner.get(shooter)?, self.world.owner.get(target)?);
        if os == ot {
            return None;
        }
        let loadout = self.world.loadout.get(shooter)?;
        let role = self.world.role.get(shooter).copied().unwrap_or(Role::Reserve);
        let standing = Standing::of(self.world.doctrine.get(self.player_entity[os.0 as usize])?);
        standing.fire_distance(role, loadout, standing.regard())
    }

    /// **Encounter detection for one hull** (T-133 stage 4): against every
    /// armed rival that fires on it, and — if it is armed — against every
    /// rival hull it fires on. `O(armed hulls)` for an unarmed hull, which is
    /// what §4's locality rule asks of a path every departure runs.
    fn detect(&mut self, e: Entity) {
        if self.armed.is_empty() || !self.live_hull(e) {
            return;
        }
        let Some(&owner) = self.world.owner.get(e) else { return };
        let Some(&mine) = self.world.track.get(e) else { return };
        // The farthest `shooter` could fire, plus the station-keeping margin:
        // an upper bound on the reach `seek` will use, so the box test below
        // only ever rejects a pair `seek` would have found `Never` for.
        let farthest = |sim: &Simulation, shooter: Entity| {
            sim.world.loadout.get(shooter).map_or(0.0, |l| l.fire_enemy_ly.max(l.fire_neutral_ly)) + STATION_MARGIN
        };
        let near = |sim: &Simulation, other: Entity, reach: f64| {
            sim.world.track.get(other).is_some_and(|t| boxes_within(&mine, t, reach))
        };
        let mut found: Vec<(Entity, Entity)> = Vec::new();
        for &s in &self.armed {
            if s != e
                && self.world.owner.get(s) != Some(&owner)
                && near(self, s, farthest(self, s))
                && self.live_hull(s)
            {
                found.push((s, e));
            }
        }
        if self.armed.contains(&e) {
            let reach = farthest(self, e);
            for i in 0..self.world.hull_type.capacity() {
                let h = Entity(i as u64);
                if h != e
                    && self.world.owner.get(h).is_some_and(|&o| o != owner)
                    && near(self, h, reach)
                    && self.live_hull(h)
                {
                    found.push((e, h));
                }
            }
        }
        for (s, t) in found {
            self.seek(s, t, self.clock);
        }
    }

    /// Search for the moment `target` enters `shooter`'s reach at or after
    /// `from`, and schedule what the search found.
    fn seek(&mut self, shooter: Entity, target: Entity, from: f64) {
        let Some(reach) = self.fire_reach(shooter, target) else { return };
        let (gs, gt) = (self.gen_of(shooter), self.gen_of(target));
        match self.next_entry(shooter, target, reach + STATION_MARGIN, from) {
            Entry::At(t) => {
                self.schedule_at(t.max(self.clock), EventKind::EncounterBegin { shooter, target, gs, gt });
                // **Belief event A** (R-WAR30): the light of the shooter's
                // approach — from where its current trajectory began — may
                // reach the target before the first shot does.
                if let Some(seen) = self.light_reaches(target, shooter) {
                    if seen < t {
                        self.schedule_at(seen.max(self.clock), EventKind::ThreatSeen { target, shooter, gs, gt });
                    }
                }
            }
            Entry::Resume(t) => {
                self.schedule_at(t.max(self.clock), EventKind::EncounterSeek { shooter, target, gs, gt })
            }
            Entry::Never => {}
        }
    }

    /// **The first time two hulls are within `reach`** of each other on their
    /// current trajectories, at or after `from`.
    fn next_entry(&self, a: Entity, b: Entity, reach: f64, from: f64) -> Entry {
        let (Some(ma), Some(mb)) = (self.world.motion.get(a).copied(), self.world.motion.get(b).copied()) else {
            return Entry::Never;
        };
        match (ma.under_way(from), mb.under_way(from)) {
            (false, false) => {
                if ma.position_at(from).distance(mb.position_at(from)) <= reach {
                    Entry::At(from)
                } else {
                    Entry::Never
                }
            }
            (false, true) => {
                entry_against_point(&mb, ma.position_at(from), reach, from).map_or(Entry::Never, Entry::At)
            }
            (true, false) => {
                entry_against_point(&ma, mb.position_at(from), reach, from).map_or(Entry::Never, Entry::At)
            }
            (true, true) => {
                if closest_approach_bound(&ma, &mb) > reach {
                    return Entry::Never;
                }
                // **Conservative advancement**: two hulls can close at most at
                // the sum of their top speeds, so a step of `(gap − reach) /
                // that` cannot skip past the moment they meet. Exact to the
                // step it stops at; bounded per event.
                let end = ma.arrive.min(mb.arrive);
                let mut t = from;
                for _ in 0..SEEK_STEPS {
                    if t >= end {
                        break;
                    }
                    let gap = ma.position_at(t).distance(mb.position_at(t)) - reach;
                    if gap <= 1e-12 {
                        return Entry::At(t);
                    }
                    let closing = ma.top_speed_from(t) + mb.top_speed_from(t);
                    if closing <= 0.0 {
                        return Entry::Never;
                    }
                    t = (t + gap / closing).min(end);
                }
                if t < end {
                    return Entry::Resume(t);
                }
                // One of them has arrived and rests where its trajectory ends.
                let (moving, rest) = if ma.arrive > mb.arrive { (ma, mb.dest) } else { (mb, ma.dest) };
                if ma.arrive == mb.arrive {
                    return if ma.dest.distance(mb.dest) <= reach { Entry::At(end) } else { Entry::Never };
                }
                entry_against_point(&moving, rest, reach, end).map_or(Entry::Never, Entry::At)
            }
        }
    }

    /// **When the light that shows `source`'s current trajectory reaches
    /// `receiver`**: emitted where and when that trajectory began, received at
    /// the first `t` with `t − since ≥ |receiver(t) − at|`. The receiver is
    /// slower than light, so the gap only closes, and 48 halvings of an
    /// interval that contains the answer are exact to the last bits.
    fn light_reaches(&self, receiver: Entity, source: Entity) -> Option<f64> {
        let track = *self.world.track.get(source)?;
        let m = *self.world.motion.get(receiver)?;
        let now = self.clock;
        let ahead = |t: f64| (t - track.since) - m.position_at(t).distance(track.at);
        if ahead(now) >= 0.0 {
            return Some(now);
        }
        let mut hi = now.max(m.arrive) + m.dest.distance(track.at) + (now - track.since).abs() + 1.0;
        if ahead(hi) < 0.0 {
            return None;
        }
        let mut lo = now;
        for _ in 0..48 {
            let mid = 0.5 * (lo + hi);
            if ahead(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(hi)
    }

    pub(super) fn sys_encounter_begin(&mut self, shooter: Entity, target: Entity, gs: u32, gt: u32) {
        if self.gen_of(shooter) != gs
            || self.gen_of(target) != gt
            || !self.live_hull(shooter)
            || !self.live_hull(target)
        {
            return;
        }
        if self.fire_reach(shooter, target).is_none() {
            return;
        }
        if !self.in_reach.entry(shooter).or_default().insert(target) {
            return;
        }
        let seat = |e: Entity| self.world.owner.get(e).map_or(0, |o| o.0);
        let (shooter_seat, target_seat) = (seat(shooter), seat(target));
        self.log.push(self.clock, LogEvent::EncounterBegan { shooter_seat, target_seat, shooter, target });
        if self.firing.insert(shooter) {
            self.schedule(0.0, EventKind::Discharge { shooter });
        }
    }

    pub(super) fn sys_encounter_seek(&mut self, shooter: Entity, target: Entity, gs: u32, gt: u32) {
        if self.gen_of(shooter) != gs
            || self.gen_of(target) != gt
            || !self.live_hull(shooter)
            || !self.live_hull(target)
        {
            return;
        }
        self.seek(shooter, target, self.clock);
    }

    /// **A hull's station-keeping** — its own draw, the same at every moment
    /// of its life (combat §2.5), drawn on first use and kept.
    fn station(&mut self, e: Entity) -> StationKeeping {
        if let Some(st) = self.world.station.get(e) {
            return *st;
        }
        let mut rng = self.rng.fork(e.0.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ STATION_LABEL);
        let st = StationKeeping::draw(&mut rng, STATION_RADIUS, STATION_PERIOD);
        self.world.station.insert(e, st);
        st
    }

    /// Where a hull actually is: its trajectory plus its station-keeping.
    /// Where a hull actually is, without its velocity — what aiming reads.
    /// The same expression as [`Self::fix`]'s position, so the two agree to
    /// the last bit.
    fn fix_position(&self, e: Entity, station: &StationKeeping, t: f64) -> Option<Vec3> {
        let m = self.world.motion.get(e)?;
        Some(m.position_at(t).add(station.offset_at(t)))
    }

    fn fix(&self, e: Entity, station: &StationKeeping, t: f64) -> Option<(Vec3, Vec3)> {
        let m = self.world.motion.get(e)?;
        let pos = m.position_at(t).add(station.offset_at(t));
        let vel = m.velocity_at(t).add(station.offset_velocity_at(t));
        Some((pos, vel))
    }

    /// **One weapons discharge** (T-133 stage 6).
    ///
    /// Every mount aims at the nearest hull in reach that has not absorbed its
    /// structure, as many mounts as it takes and no more; mounts left over
    /// fire on the nearest, because past the structure more energy still moves
    /// a hull toward its wreck point, which the shooter cannot see. Fire
    /// control is the arena's test against the target's actual path: where it
    /// will be one light-crossing on against the straight-line prediction from
    /// its velocity now. A miss on a target wastes the rest of that discharge.
    pub(super) fn sys_discharge(&mut self, shooter: Entity) {
        let Some(loadout) = self.world.loadout.get(shooter).copied() else {
            self.stop_firing(shooter);
            return;
        };
        if !self.live_hull(shooter) || !loadout.is_armed() || loadout.discharge_years <= 0.0 {
            self.stop_firing(shooter);
            return;
        }
        let now = self.clock;
        let own_station = self.station(shooter);
        let Some((at, _)) = self.fix(shooter, &own_station, now) else { return };
        let reference_at = self.position_at(shooter, now).unwrap_or(at);
        let listed: Vec<Entity> = self.in_reach.get(&shooter).map(|v| v.iter().copied().collect()).unwrap_or_default();
        let mut aim: Vec<(f64, Entity)> = Vec::new();
        let mut lost: Vec<Entity> = Vec::new();
        for t in listed {
            let reach = match (self.live_hull(t), self.fire_reach(shooter, t)) {
                (true, Some(r)) => r,
                _ => {
                    lost.push(t);
                    continue;
                }
            };
            let reference = self.position_at(t, now).map_or(f64::INFINITY, |p| p.distance(reference_at));
            // **Hysteresis**: detection admits a hull at `reach + margin`, and
            // one is dropped only a further margin out. Dropping it on the
            // band detection uses left a hull on the boundary to be found
            // and dropped at one instant forever, one rounding apart.
            if reference > reach + 2.0 * STATION_MARGIN {
                lost.push(t);
                continue;
            }
            // Each end's station-keeping moves it at most one radius, so a
            // reference distance past `reach + STATION_MARGIN` cannot be in
            // reach at the actual positions either.
            if reference > reach + STATION_MARGIN * (1.0 + 1e-9) {
                continue;
            }
            let st = self.station(t);
            let Some(p) = self.fix_position(t, &st, now) else { continue };
            let d = p.distance(at);
            if d <= reach {
                aim.push((d, t));
            }
        }
        if let Some(set) = self.in_reach.get_mut(&shooter) {
            for t in &lost {
                set.remove(t);
            }
        }
        // A hull that left reach may come back later on its trajectory.
        for t in lost {
            if self.live_hull(t) {
                self.seek(shooter, t, now);
            }
        }
        if self.in_reach.get(&shooter).is_none_or(|v| v.is_empty()) {
            self.stop_firing(shooter);
            return;
        }
        // **Nearest first, extracted as needed.** Keys are unique (the entity
        // breaks ties), so the order is total. A discharge usually spends its
        // mounts on the first one or two, so the nearest few are selected in
        // place and the rest sorted only if the walk gets that far.
        let key = |a: &(f64, Entity), b: &(f64, Entity)| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1));
        const SELECT_FIRST: usize = 8;
        let per_mount = loadout.beam_power_kj_per_year * loadout.discharge_years;
        let mut mounts = loadout.beams;
        let mut missed = false;
        let mut deliveries: Vec<(Entity, f64)> = Vec::new();
        for i in 0..aim.len() {
            if mounts == 0 {
                break;
            }
            if i < SELECT_FIRST {
                let (j, _) = aim[i..].iter().enumerate().min_by(|a, b| key(a.1, b.1)).unwrap();
                aim.swap(i, i + j);
            } else if i == SELECT_FIRST {
                aim[i..].sort_unstable_by(key);
            }
            let t = aim[i].1;
            let st = *self.world.station.get(t).unwrap();
            let remaining = self.structure_of(t) - self.world.hull_damage.get(t).copied().unwrap_or(0.0);
            if remaining <= 0.0 {
                continue;
            }
            if !self.fire_control_holds(at, t, &st, now, loadout.fire_control_ly) {
                missed = true;
                break;
            }
            let mut needed = (remaining / per_mount).ceil();
            if needed * per_mount < remaining {
                needed += 1.0;
            }
            let used = (mounts as f64).min(needed) as u32;
            deliveries.push((t, used as f64 * per_mount));
            mounts -= used;
        }
        // Mounts left over fire on the nearest, which the walk has put first
        // (it ran at least one selection whenever there was a target).
        if mounts > 0 && !missed {
            if let Some(&(_, t)) = aim.first() {
                let st = *self.world.station.get(t).unwrap();
                if self.fire_control_holds(at, t, &st, now, loadout.fire_control_ly) {
                    deliveries.push((t, mounts as f64 * per_mount));
                }
            }
        }
        for (t, energy) in deliveries {
            self.deliver(shooter, t, energy);
        }
        self.schedule(loadout.discharge_years, EventKind::Discharge { shooter });
    }

    fn stop_firing(&mut self, shooter: Entity) {
        self.firing.remove(&shooter);
        self.in_reach.remove(&shooter);
    }

    /// The arena's fire-control test ([`crate::combat::laser_hit_check`]) on
    /// the simulation's own trajectories.
    fn fire_control_holds(&self, from: Vec3, target: Entity, st: &StationKeeping, now: f64, tolerance: f64) -> bool {
        let Some((p, v)) = self.fix(target, st, now) else { return false };
        let crossing = p.distance(from);
        let predicted = p.add(v.scale(crossing));
        let Some((actual, _)) = self.fix(target, st, now + crossing) else { return false };
        predicted.distance(actual) <= tolerance
    }

    /// A hull's structure, kJ — its volume times its Design class's `σ`.
    fn structure_of(&self, e: Entity) -> f64 {
        let hull = self.world.hull_type.get(e).copied().unwrap_or(HullType::LimitedSystems);
        let class = self.world.design_class.get(e).copied().unwrap_or(Class::Unnamed);
        crate::combat::hull_structure_kj(hull, class, &self.config, &self.combat)
    }

    /// **The damage at which this hull is wrecked** — its own draw, the same
    /// at every encounter (§8.19.5): the root generator is only ever forked,
    /// never advanced, so a label from the entity alone is one uniform for the
    /// hull's whole life.
    fn wreck_point(&self, e: Entity) -> f64 {
        let u = self.rng.fork(e.0.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ WRECK_DRAW_LABEL).unit();
        crate::combat::wreck_point_kj(self.structure_of(e), u, &self.combat)
    }

    /// **Energy lands on a hull** (T-133): the wreck check, then the fleet's
    /// decision if it survives.
    fn deliver(&mut self, shooter: Entity, target: Entity, energy: f64) {
        if !self.live_hull(target) {
            return;
        }
        let before = self.world.hull_damage.get(target).copied().unwrap_or(0.0);
        let after = before + energy;
        if after >= self.wreck_point(target) {
            self.wreck_hull(target, shooter, after);
            return;
        }
        self.world.hull_damage.insert(target, after);
        // **A hit says something about the destination only if the gun covers
        // it.** Fire taken leaving a world, or braking out of a course change,
        // comes from a shooter that will not be standing where the hull is
        // now going.
        let covers = match (self.world.motion.get(target), self.position_at(shooter, self.clock)) {
            (Some(m), Some(at)) => {
                self.fire_reach(shooter, target).is_some_and(|r| at.distance(m.dest) <= r + STATION_MARGIN)
            }
            _ => false,
        };
        if covers {
            self.fired_on_at_destination.insert(target);
        }
        let structure = self.structure_of(target);
        if before <= structure && after > structure {
            // **Ending 2** (R-WAR26): a hull past its structure has survived
            // its roll and is defeated — it leaves (§2.1).
            self.withdraw(target);
        } else {
            // **Belief event B**: an enemy fired on this hull's fleet.
            self.respond(target, shooter);
        }
    }

    /// **A hull reaches its wreck point** (T-133, §8.19.5), wherever it is:
    /// off every post, and **still on its course** (the author's ruling:
    /// "Wrecked hulls continue on their course at the moment of destruction").
    /// Its drive is dead, so it keeps the velocity it had and coasts, carrying
    /// its hull, its cargo and the people aboard as slag (design law #11).
    fn wreck_hull(&mut self, e: Entity, by: Entity, damage: f64) {
        let now = self.clock;
        let (from, velocity) =
            self.world.motion.get(e).map_or((self.position_at(e, now).unwrap_or(Vec3::ZERO), Vec3::ZERO), |m| {
                (m.position_at(now), m.velocity_at(now))
            });
        let owner = self.world.owner.get(e).map_or(0, |o| o.0);
        let role = self.world.role.get(e).copied().unwrap_or(Role::Reserve);
        let by_seat = self.world.owner.get(by).map_or(0, |o| o.0);
        self.leave_post(e);
        let mass = self.destroy_free_hulls(&[e]);
        self.world.wreck.insert(e, Wreck { from, since: now, velocity, mass });
        self.armed.remove(&e);
        self.stop_firing(e);
        self.fired_on_at_destination.remove(&e);
        if let Some(track) = self.world.track.get_mut(e) {
            track.gen = track.gen.wrapping_add(1);
        }
        self.log.push(
            now,
            LogEvent::HullWrecked { player: owner, vehicle: e, role, by: by_seat, damage, slag: mass.kilotons() },
        );
    }

    /// **Take a hull off whatever post it holds** — a mining crew, a picket
    /// stack, a blockade, the in-flight picket ledgers, a reserve pool — so a
    /// hull that is wrecked or leaves is not counted where it no longer is.
    fn leave_post(&mut self, e: Entity) {
        let Some(&owner) = self.world.owner.get(e) else { return };
        let seat = owner.0;
        let role = self.world.role.get(e).copied().unwrap_or(Role::Reserve);
        let target = self.world.voyage.get(e).map(|v| v.target);
        let under_way = self.world.motion.get(e).is_some_and(|m| m.under_way(self.clock));
        match role {
            Role::Miner => {
                if let Some(t) = target {
                    if let Some(crew) = self.mine_crew.get_mut(&(seat, t.0)) {
                        crew.retain(|&h| h != e);
                        if crew.is_empty() {
                            self.mine_crew.remove(&(seat, t.0));
                        }
                    }
                }
            }
            Role::Picket => {
                if let Some(t) = target {
                    if under_way {
                        if let Some(n) = self.picket_inbound.get_mut(&t.0) {
                            *n = n.saturating_sub(1);
                            if *n == 0 {
                                self.picket_inbound.remove(&t.0);
                            }
                        }
                        self.take_bound(seat, t.0);
                    } else {
                        self.detach_picket(t, e);
                        self.leave_blockade(t.0, seat, e);
                    }
                }
            }
            _ => {}
        }
        let p = seat as usize;
        if p < self.reserve_miners.len() {
            self.reserve_miners[p].retain(|&h| h != e);
            self.reserve_freighters[p].retain(|&h| h != e);
        }
    }

    /// **The fleet a hull decides with** (T-133, R-WAR32 interim): its crew at
    /// a rock, its stack at a world or port, or itself.
    fn fleet_of(&self, e: Entity) -> (FleetKey, Vec<Entity>) {
        let seat = self.world.owner.get(e).map_or(0, |o| o.0);
        let role = self.world.role.get(e).copied().unwrap_or(Role::Reserve);
        let target = self.world.voyage.get(e).map(|v| v.target);
        let parked = self.world.motion.get(e).is_some_and(|m| !m.under_way(self.clock));
        if let (true, Some(t)) = (parked, target) {
            match role {
                Role::Miner => {
                    if let Some(crew) = self.mine_crew.get(&(seat, t.0)) {
                        if crew.contains(&e) {
                            return (FleetKey::Crew(seat, t.0), crew.clone());
                        }
                    }
                }
                Role::Picket => {
                    if let Some((holder, stack, _)) = self.picket.get(&t.0) {
                        if *holder == seat && stack.contains(&e) {
                            return (FleetKey::Stack(seat, t.0), stack.clone());
                        }
                    }
                    if let Some(stack) = self.blockade.get(&(t.0, seat)) {
                        if stack.contains(&e) {
                            return (FleetKey::Stack(seat, t.0), stack.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        (FleetKey::Hull(e.0), vec![e])
    }

    /// **A fleet decides what to do about `shooter`** — once per shooter
    /// (T-133, R-WAR30), through the standing layer's resolver.
    fn respond(&mut self, target: Entity, shooter: Entity) {
        if !self.live_hull(target) {
            return;
        }
        let (key, members) = self.fleet_of(target);
        if !self.responded.insert((key, shooter)) {
            return;
        }
        let seat = self.world.owner.get(target).map_or(0, |o| o.0 as usize);
        let role = self.world.role.get(target).copied().unwrap_or(Role::Reserve);
        let returns_fire = self.fire_reach(target, shooter).is_some();
        let doctrine = self.doctrine_of(seat);
        let standing = Standing::of(&doctrine);
        let enemy = standing.regard() == crate::autopilot::Relation::Enemy;
        let under_way = self.world.motion.get(target).is_some_and(|m| m.under_way(self.clock));
        // **Believed kinematics at short range** (R-O41, warfare §5.4): the
        // shooter is within fire distance, a few light-days, so the
        // observation is current and belief equals truth.
        let own = self.laden_accel(target, self.config.civilian_accel_g);
        let theirs = self.laden_accel(shooter, self.config.civilian_accel_g);
        let may_disengage = crate::belief::can_disengage(own, theirs);
        // What the hull knows of the shooter's post is where it saw it standing
        // or heading, and a colony ship's own destination under fire.
        if role == Role::Colonizer {
            if let Some(w) = self.world.voyage.get(target).map(|v| v.target) {
                self.threatened[seat].insert(w.0);
            }
            if let Some(w) = self.world.voyage.get(shooter).map(|v| v.target) {
                self.threatened[seat].insert(w.0);
            }
        }
        match standing.under_fire(role, returns_fire, enemy, under_way, may_disengage) {
            UnderFire::Continue => {}
            UnderFire::Retarget => {
                for m in members {
                    if self.world.role.get(m).copied() == Some(Role::Colonizer) {
                        self.retarget_colonizer(m);
                    }
                }
            }
            UnderFire::Withdraw => {
                for m in members {
                    self.withdraw(m);
                }
            }
        }
    }

    /// **Belief event A reaches its target** (T-133, R-WAR30). A colony ship
    /// acts on it only if the fire would come before it could found.
    pub(super) fn sys_threat_seen(&mut self, target: Entity, shooter: Entity, gs: u32, gt: u32) {
        if self.gen_of(shooter) != gs
            || self.gen_of(target) != gt
            || !self.live_hull(shooter)
            || !self.live_hull(target)
        {
            return;
        }
        if self.world.role.get(target).copied() == Some(Role::Colonizer) {
            let Some(reach) = self.fire_reach(shooter, target) else { return };
            let arrive = self.world.motion.get(target).map_or(f64::INFINITY, |m| m.arrive);
            match self.next_entry(shooter, target, reach + STATION_MARGIN, self.clock) {
                Entry::At(t) if t < arrive => {}
                _ => return,
            }
        }
        self.respond(target, shooter);
    }

    /// **A course change from where the hull is and how it is moving**
    /// (T-133, R-WAR30): brake along the current velocity, then fly from rest
    /// to `dest`. Returns the arrival time.
    pub(super) fn course_change(&mut self, e: Entity, dest: Vec3) -> f64 {
        let now = self.clock;
        let current = self.world.motion.get(e).copied();
        let (stop, stop_at, brake) = current.map_or((dest, now, None), |m| m.come_to_rest(now));
        let accel = self.laden_accel(e, self.config.civilian_accel_g);
        let mut leg = Motion::leg(stop, dest, stop_at.max(now), accel);
        leg.brake = brake;
        self.world.motion.insert(e, leg);
        self.track_changed(e);
        leg.arrive
    }

    /// **A colony ship seeks a new destination** (T-133, the author's ruling:
    /// colonists are not suicidal). The nearest world its empire has scanned,
    /// does not own, has not already sent a ship to and does not believe an
    /// enemy holds, that its hold can found — measured from where it can come
    /// to rest. With none, it goes home.
    pub(super) fn retarget_colonizer(&mut self, e: Entity) {
        let Some(p) = self.world.owner.get(e).map(|o| o.0 as usize) else { return };
        let Some(m) = self.world.motion.get(e).copied() else { return };
        let hull = self.world.hull_type.get(e).copied().unwrap_or(HullType::MediumSystems);
        let Some(&home) = self.world.home_center.get(e) else {
            self.withdraw(e);
            return;
        };
        let (stop, _, _) = m.come_to_rest(self.clock);
        let knowledge = self.world.knowledge.get(self.player_entity[p]).unwrap();
        let mut candidates: Vec<(f64, Entity, PlanetId)> = knowledge
            .scanned
            .iter()
            .filter(|pid| !knowledge.targeted.contains(**pid))
            .filter_map(|&pid| {
                let w = *self.planet_entity.get(pid.0 as usize)?;
                if self.world.owner.contains(w) || self.threatened[p].contains(&w.0) {
                    return None;
                }
                Some((self.world.position.get(w)?.distance(stop), w, pid))
            })
            .collect();
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let choice = candidates
            .into_iter()
            .find(|&(_, w, _)| self.colony_seed_for(hull, home, w).is_some_and(|k| k > Kilotons::ZERO));
        let Some((_, w, pid)) = choice else {
            self.withdraw(e);
            return;
        };
        self.mark_targeted(p, pid);
        if let Some(v) = self.world.voyage.get_mut(e) {
            v.target = w;
        }
        let dest = *self.world.position.get(w).unwrap();
        let arrive = self.course_change(e, dest);
        self.schedule_at(arrive, EventKind::ColonyArrive { vehicle: e });
        self.log.push(
            self.clock,
            LogEvent::CourseChanged {
                player: p as u32,
                vehicle: e,
                role: Role::Colonizer,
                reason: CourseReason::Retarget,
                to: pid,
            },
        );
    }

    /// **A hull heads home** (T-133): past its structure (ending 2), off a
    /// post it cannot hold under fire, or breaking off on believed kinematics
    /// (ending 3). It stands down to Reserve on arrival, unloading whatever it
    /// carries (`sys_return_arrive`).
    fn withdraw(&mut self, e: Entity) {
        if !self.live_hull(e) {
            return;
        }
        let seat = self.world.owner.get(e).map_or(0, |o| o.0 as usize);
        let role = self.world.role.get(e).copied().unwrap_or(Role::Reserve);
        let here = self.position_at(e, self.clock).unwrap_or(Vec3::ZERO);
        let home = self.world.home_center.get(e).copied().or_else(|| self.nearest_owned_planet(seat, here));
        let Some(home) = home else { return };
        let Some(&home_pos) = self.world.position.get(home) else { return };
        self.leave_post(e);
        // Leaving the mission: a withdrawing hull holds fire on anyone it
        // does not regard as an enemy (`Standing::fire_distance`).
        self.world.role.insert(e, Role::Reserve);
        let arrive = self.course_change(e, home_pos);
        self.schedule_at(arrive, EventKind::ReturnArrive { vehicle: e });
        let pid = *self.world.planet_id.get(home).unwrap();
        self.log.push(
            self.clock,
            LogEvent::CourseChanged { player: seat as u32, vehicle: e, role, reason: CourseReason::Withdraw, to: pid },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Loadout;
    use crate::galaxy::GalaxyConfig;
    use crate::log::{LogCategory, LogFilter};

    /// The unit bed: two seats on a 200-planet field (`CLAUDE.md` §2's
    /// "reduce the galaxy before the horizon").
    fn bed(seed: u64) -> Simulation {
        let mut g = GalaxyConfig::new(2, seed);
        g.planet_count = 200;
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = 60.0;
        let mut sim = Simulation::with_baseline(Galaxy::generate(g).unwrap(), cfg);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Combat));
        sim
    }

    /// Seat `seat`'s homeworld.
    fn home(sim: &Simulation, seat: usize) -> Entity {
        sim.world.player_info.get(sim.player_entity[seat]).unwrap().home
    }

    /// A point in open space well off the disk, where nothing else flies.
    fn open_space(sim: &Simulation) -> Vec3 {
        sim.world.position.get(home(sim, 0)).unwrap().add(Vec3::new(0.0, 0.0, 40.0))
    }

    /// A picket of `seat`'s standing at `at`, placed through the engine's own
    /// [`Simulation::park`], so detection finds it the way it finds a hull
    /// that flew there.
    fn stand(sim: &mut Simulation, seat: u32, hull: HullType, class: Class, loadout: Loadout, at: Vec3) -> Entity {
        let e = sim.world.spawn();
        sim.world.owner.insert(e, PlayerId(seat));
        sim.world.role.insert(e, Role::Picket);
        sim.world.hull_type.insert(e, hull);
        sim.world.design_class.insert(e, class);
        sim.world.loadout.insert(e, loadout);
        let h = home(sim, seat as usize);
        sim.world.home_center.insert(e, h);
        sim.park(e, at);
        e
    }

    /// The Cairn picket's gun, the unit mount.
    fn cairn_gun(sim: &Simulation) -> Loadout {
        design_loadout(HullType::LimitedContactVehicle, Class::Cairn, &sim.config, &sim.combat)
    }

    /// A gun that fires but does not hurt: the hull returns fire, so it
    /// stands, and nothing it delivers moves a result.
    fn popgun(sim: &Simulation) -> Loadout {
        Loadout { beam_power_kj_per_year: 1e-9, ..cairn_gun(sim) }
    }

    fn enemies(sim: &mut Simulation) {
        for p in 0..2 {
            sim.world.doctrine.get_mut(sim.player_entity[p]).unwrap().engage_neutrals = true;
        }
    }

    fn damage(sim: &Simulation, e: Entity) -> f64 {
        sim.world.hull_damage.get(e).copied().unwrap_or(0.0)
    }

    fn wrecked(sim: &Simulation, e: Entity) -> Option<f64> {
        sim.log().iter().find_map(|r| match r.event {
            LogEvent::HullWrecked { vehicle, damage, .. } if vehicle == e => Some(damage),
            _ => None,
        })
    }

    fn run_to(sim: &mut Simulation, t: f64) {
        sim.config.horizon_years = sim.config.horizon_years.max(t + 1.0);
        while sim.clock < t && sim.step() {}
    }

    /// Two rival pickets a third of the engagement range apart in open space.
    fn duel(seed: u64, a: fn(&Simulation) -> Loadout, b: fn(&Simulation) -> Loadout) -> (Simulation, [Entity; 2]) {
        let mut sim = bed(seed);
        let at = open_space(&sim);
        let apart = Vec3::new(cairn_gun(&sim).fire_enemy_ly / 3.0, 0.0, 0.0);
        let (la, lb) = (a(&sim), b(&sim));
        let hull = HullType::LimitedContactVehicle;
        let e0 = stand(&mut sim, 0, hull, Class::Cairn, la, at);
        let e1 = stand(&mut sim, 1, hull, Class::Cairn, lb, at.add(apart));
        (sim, [e0, e1])
    }

    fn unarmed(_: &Simulation) -> Loadout {
        Loadout::UNARMED
    }

    /// **Each hull fires what its Design mounts** (T-125, T-133). Two unarmed
    /// hulls never meet; an armed hull hits an unarmed one on whichever seat
    /// it stands, and the unarmed hull leaves; two armed enemies stand and
    /// both take fire — neither shoots first by seat.
    #[test]
    fn a_hull_fires_what_its_design_mounts_and_nothing_else() {
        let days = |d: f64| d / crate::combat::DAYS_PER_YEAR;
        let (mut sim, [e0, e1]) = duel(7, unarmed, unarmed);
        run_to(&mut sim, days(30.0));
        assert!(!sim.log().iter().any(|r| matches!(r.event, LogEvent::EncounterBegan { .. })), "unarmed meets nobody");
        assert_eq!((damage(&sim, e0), damage(&sim, e1)), (0.0, 0.0));

        for armed_seat in [0usize, 1] {
            let (mut sim, hulls) =
                if armed_seat == 0 { duel(7, cairn_gun, unarmed) } else { duel(7, unarmed, cairn_gun) };
            run_to(&mut sim, days(30.0));
            let (gun, prey) = (hulls[armed_seat], hulls[1 - armed_seat]);
            assert!(damage(&sim, prey) > 0.0 || wrecked(&sim, prey).is_some(), "seat {armed_seat}'s gun hits");
            assert_eq!(damage(&sim, gun), 0.0, "an unarmed hull fires nothing back");
            let left = sim.log().iter().any(|r| {
                matches!(r.event, LogEvent::CourseChanged { vehicle, reason: CourseReason::Withdraw, .. } if vehicle == prey)
            });
            assert!(left, "a stationary hull that cannot return fire leaves");
        }

        let (mut sim, [e0, e1]) = duel(7, cairn_gun, cairn_gun);
        enemies(&mut sim);
        // Re-detect under the new regard: Doctrine is state, and a write to it
        // reaches detection on the next trajectory change.
        sim.track_changed(e0);
        sim.track_changed(e1);
        run_to(&mut sim, days(120.0));
        let took = |e| wrecked(&sim, e).unwrap_or_else(|| damage(&sim, e));
        assert!(took(e0) > 0.0 && took(e1) > 0.0, "fire is simultaneous: {} and {}", took(e0), took(e1));
        assert!(wrecked(&sim, e0).is_some() || wrecked(&sim, e1).is_some(), "a pitched battle ends in a wreck");
    }

    /// **Damage is a power, and the period only says how often it is checked**
    /// (T-132, T-133). One mount on a hull that stands: after any span the
    /// damage is power × span to within one discharge, at the Design's period
    /// and at half of it.
    #[test]
    fn damage_is_a_power_whatever_the_discharge_period() {
        for halve in [false, true] {
            let (mut sim, [e0, e1]) = duel(3, cairn_gun, popgun);
            enemies(&mut sim);
            if halve {
                let l = sim.world.loadout.get_mut(e0).unwrap();
                l.discharge_years *= 0.5;
            }
            sim.track_changed(e0);
            sim.track_changed(e1);
            let gun = *sim.world.loadout.get(e0).unwrap();
            let power = gun.beams as f64 * gun.beam_power_kj_per_year;
            let t0 = sim.clock;
            let kill = sim.structure_of(e1) / power;
            run_to(&mut sim, t0 + 0.5 * kill);
            let span = sim.clock - t0;
            let d = damage(&sim, e1);
            let one = power * gun.discharge_years;
            assert!((d - power * span).abs() <= one * (1.0 + 1e-9), "halved {halve}: {d} kJ after {span} yr");
        }
    }

    /// **A hull is wrecked at its wreck point, whatever the period and
    /// whatever it carried in** (T-133, §8.19.5): the damage it is wrecked
    /// with is at or past its own draw and short of it plus one discharge.
    #[test]
    fn a_hull_is_wrecked_at_its_wreck_point_whatever_the_period() {
        for (halve, worn) in [(false, false), (true, false), (false, true), (true, true)] {
            let (mut sim, [e0, e1]) = duel(5, cairn_gun, popgun);
            enemies(&mut sim);
            if halve {
                sim.world.loadout.get_mut(e0).unwrap().discharge_years *= 0.5;
            }
            if worn {
                let half = 0.5 * sim.structure_of(e1);
                sim.world.hull_damage.insert(e1, half);
            }
            sim.track_changed(e0);
            sim.track_changed(e1);
            let gun = *sim.world.loadout.get(e0).unwrap();
            let one = gun.beams as f64 * gun.beam_power_kj_per_year * gun.discharge_years;
            let at = sim.wreck_point(e1);
            assert!(at >= sim.structure_of(e1), "a wreck point is past the structure");
            run_to(&mut sim, 1.0);
            let d = wrecked(&sim, e1).unwrap_or_else(|| panic!("halved {halve} worn {worn}: never wrecked"));
            assert!(d >= at && d < at + one * (1.0 + 1e-9), "halved {halve} worn {worn}: wrecked at {d}, point {at}");
            assert_eq!(sim.world.role.get(e1).copied(), Some(Role::Scrapped));
        }
    }

    /// **A wrecked hull continues on its course** (T-133, the author's ruling).
    /// A hull flying past a Scarp's guns is wrecked in flight; the wreck starts
    /// where the hull was and keeps the velocity it had, coasting in a straight
    /// line with its drive dead, and carries its whole mass.
    #[test]
    fn a_wrecked_hull_continues_on_its_course() {
        let mut sim = bed(21);
        enemies(&mut sim);
        let at = open_space(&sim);
        let gun = design_loadout(HullType::GeneralContactVehicle, Class::Scarp, &sim.config, &sim.combat);
        stand(&mut sim, 0, HullType::GeneralContactVehicle, Class::Scarp, gun, at);
        // A seat-1 hull crossing the gun's reach at speed, off to one side.
        let prey = stand(&mut sim, 1, HullType::MediumSystems, Class::Delta, Loadout::UNARMED, at);
        let side = Vec3::new(0.0, 0.3 * gun.fire_enemy_ly, 0.0);
        let leg = Motion::leg(
            at.add(side).add(Vec3::new(-1.0, 0.0, 0.0)),
            at.add(side).add(Vec3::new(1.0, 0.0, 0.0)),
            0.0,
            2.0,
        );
        sim.world.motion.insert(prey, leg);
        sim.track_changed(prey);
        let before = sim.mass_ledger();
        run_to(&mut sim, leg.arrive);
        let w = *sim.world.wreck.get(prey).expect("the Scarp wrecks it in passing");
        assert!(w.since > leg.depart && w.since < leg.arrive, "wrecked in flight at {}", w.since);
        assert!(w.from.distance(leg.position_at(w.since)) < 1e-12, "where it was");
        assert!(w.velocity.distance(leg.velocity_at(w.since)) < 1e-12, "at the velocity it had");
        assert!(w.velocity.norm() > 0.0);
        let t = w.since + 3.0;
        assert!(sim.position_at(prey, t).unwrap().distance(w.from.add(w.velocity.scale(3.0))) < 1e-12);
        assert!(sim.position_at(prey, t).unwrap().distance(leg.dest) > 0.0, "and does not stop where it was going");
        assert!((w.mass - hull_dry_mass(HullType::MediumSystems, &sim.config)).kilotons().abs() < 1e-12);
        let after = sim.mass_ledger();
        assert!((after.wrecks - before.wrecks - w.mass.kilotons()).abs() < 1e-12, "the ledger carries the wreck");
    }

    /// **The entry into reach is exact on both stretches of a course
    /// change** (T-133): braking, and the leg flown from rest after it. At
    /// the time `entry_against_point` returns, the hull is `reach` from the
    /// point, and a moment earlier it is farther.
    #[test]
    fn the_entry_into_reach_is_exact_on_a_braking_and_a_flying_stretch() {
        let leg = Motion::leg(Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), 1.0, 0.241);
        // Mid-leg, before turnover: brake, then fly to a point off the line.
        let now = leg.depart + 0.3 * (leg.arrive - leg.depart);
        let (stop, stop_at, brake) = leg.come_to_rest(now);
        let b = brake.expect("under way, so it brakes");
        assert!(b.end > now && stop.distance(leg.position_at(now)) > 0.0);
        let dv = leg.velocity_at(now).distance(Motion { brake, ..leg }.velocity_at(now));
        assert!(dv < 1e-12, "the brake starts at the speed the hull had: {dv}");
        let mut turned = Motion::leg(stop, Vec3::new(0.0, 4.0, 0.0), stop_at, 0.241);
        turned.brake = brake;
        assert!(turned.velocity_at(b.end).norm() < 1e-9, "and ends at rest");
        let midway = stop.add(turned.dest.sub(stop).scale(0.5)).add(Vec3::new(0.0, 0.0, 0.001));
        for (p, reach) in [(stop.add(Vec3::new(0.01, 0.02, 0.0)), 0.03), (midway, 0.01)] {
            let t = entry_against_point(&turned, p, reach, now).expect("the course passes the point");
            let d = turned.position_at(t).distance(p);
            assert!((d - reach).abs() < 1e-9, "entered at {d} ly, not {reach}");
            assert!(turned.position_at(t - 1e-6).distance(p) > reach, "and was outside just before");
        }
        // Past turnover the hull stops where it was going.
        let late = leg.depart + 0.7 * (leg.arrive - leg.depart);
        let (stop, _, _) = leg.come_to_rest(late);
        assert!(stop.distance(leg.dest) < 1e-9, "past turnover it stops at the destination: {stop:?}");
    }

    /// **Two moving hulls are found where their separation reaches the
    /// reach** (T-133): conservative advancement never steps past the meeting,
    /// and the search resumes across events until it lands.
    #[test]
    fn two_moving_hulls_are_found_at_the_moment_they_come_in_reach() {
        let mut sim = bed(11);
        let at = open_space(&sim);
        let gun = cairn_gun(&sim);
        let a = stand(&mut sim, 0, HullType::LimitedContactVehicle, Class::Cairn, gun, at);
        let b = stand(
            &mut sim,
            1,
            HullType::MediumSystems,
            Class::Delta,
            Loadout::UNARMED,
            at.add(Vec3::new(2.0, 0.0, 0.0)),
        );
        // Mirror-image legs that cross at their midpoints at the same moment,
        // with one offset out of the plane so they pass just inside reach.
        let (accel, lift) = (0.5, Vec3::new(0.0, 0.0, 0.5 * cairn_gun(&sim).fire_neutral_ly));
        sim.world.motion.insert(a, Motion::leg(at, at.add(Vec3::new(2.0, 2.0, 0.0)), 0.0, accel));
        let b_from = at.add(Vec3::new(2.0, 0.0, 0.0)).add(lift);
        sim.world.motion.insert(b, Motion::leg(b_from, at.add(Vec3::new(0.0, 2.0, 0.0)).add(lift), 0.0, accel));
        let reach = gun.fire_neutral_ly;
        let mut from = 0.0;
        let t = loop {
            match sim.next_entry(a, b, reach, from) {
                Entry::At(t) => break t,
                Entry::Resume(t) => {
                    assert!(t > from, "a resumed search advances");
                    from = t;
                }
                Entry::Never => panic!("the courses cross"),
            }
        };
        let (ma, mb) = (*sim.world.motion.get(a).unwrap(), *sim.world.motion.get(b).unwrap());
        let d = ma.position_at(t).distance(mb.position_at(t));
        assert!((d - reach).abs() < 1e-9, "found at {d} ly, reach {reach}");
    }

    /// **The light of a new course overtakes a closing hull sooner than a
    /// standing one** (T-115, T-133): a picket takes station on a colony
    /// ship's world, and the ship, flying toward the wavefront, meets it
    /// before a standing observer at the ship's launch range would. The time
    /// returned is a root: the light has travelled exactly the ship's range.
    #[test]
    fn the_light_overtakes_a_closing_hull_sooner_than_a_standing_one() {
        let mut sim = bed(4242);
        let h = home(&sim, 1);
        let from = *sim.world.position.get(h).unwrap();
        let world = *sim
            .planet_entity
            .iter()
            .filter(|&&e| !sim.world.owner.contains(e) && sim.world.factors.contains(e))
            .max_by(|&&a, &&b| {
                let d = |e| sim.world.position.get(e).unwrap().distance(from);
                d(a).total_cmp(&d(b)).then(a.0.cmp(&b.0))
            })
            .unwrap();
        let ship = Entity(sim.world.next);
        sim.spawn_courier(1, Role::Colonizer, BuiltHull::unpaid(HullType::MediumSystems), h, world, 0.0);
        let world_pos = *sim.world.position.get(world).unwrap();
        let picket = stand(&mut sim, 0, HullType::LimitedContactVehicle, Class::Cairn, Loadout::UNARMED, world_pos);
        let d0 = sim.position_at(ship, sim.clock).unwrap().distance(world_pos);
        let seen = sim.light_reaches(ship, picket).expect("the light reaches the ship");
        let lag = seen - sim.clock;
        assert!(lag > 0.0 && lag < d0, "a closing hull is reached sooner: {lag} vs {d0}");
        let range = sim.position_at(ship, seen).unwrap().distance(world_pos);
        assert!((lag - range).abs() < 1e-9, "light travelled {lag}, the ship's range {range}");
    }
}
