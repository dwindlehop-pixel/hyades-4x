//! **Missiles, point defense and the supply line** (T-139,
//! `Hyades_technology_tree.md` §9, `Hyades_warfare_tree.md` §8.22).
//!
//! A missile Design (`Class::Butte`, `Class::Mesa`) mounts tubes and a magazine
//! where a beam Design mounts beams. Its rounds are
//! [`Material::Ordnance`] in the hull's hold — carried mass, so a full magazine
//! is a slower hull — and fire is a sequence of events on the main loop like a
//! beam's:
//!
//! | event | when | what it does |
//! |---|---|---|
//! | [`EventKind::Discharge`] | every reload while anything it fires on is in reach | launches a burst per tube, nearest target first, no more than a target still needs |
//! | [`EventKind::MissileEnters`] | the round enters point-defense range of its target | the allied beam hull that can finish it before impact commits its mounts, which fire at nothing else meanwhile (R-WAR45) |
//! | [`EventKind::MissileArrive`] | the round's flight time later | stopped if its defender is still there; otherwise the hit or the miss |
//! | [`EventKind::SentryArrive`] | a sentry leaves its yard | takes station at its center and fills its magazine there |
//! | [`EventKind::RearmArrive`] | a hull out of rounds reaches a center | rearms there and flies back to its post |
//!
//! **Every round ends as debris** (design law #11): its mass leaves the
//! launcher's hold at launch, flies, and lands in [`Simulation::ordnance_debris`]
//! whether it hit, was shot down or missed.
//!
//! **Resupply** (the author's choice: return and ammo runs) is the Standing
//! question [`Standing::resupply`]: a hull at a center rearms there; one at a
//! post waits for a hauler carrying rounds while the empire has one idle, and
//! otherwise flies to the nearest center itself.
use super::*;
use crate::autopilot::Resupply;
use crate::combat::{Loadout, STATION_RADIUS};
use crate::log::MissileOutcome;

/// **One round in flight** (T-139).
#[derive(Clone, Copy, Debug)]
pub(super) struct Missile {
    shooter: Entity,
    owner: u32,
    target: Entity,
    /// Where it was launched from — its powered reach is measured from here.
    from: Vec3,
    /// How far it reaches under power, ly.
    reach: f64,
    /// Its speed at the target, c — how long point defense has to engage it.
    speed: f64,
    /// When it reaches the target.
    impact: f64,
    /// The point-defense hull that committed its mounts to it, if one could
    /// finish it before impact (decided as it enters range).
    stopped_by: Option<Entity>,
    mass_kt: f64,
    warhead_kj: f64,
}

/// **Missile and supply counts** since the run began (T-139).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MissileStats {
    pub launched: u64,
    pub hits: u64,
    pub intercepted: u64,
    pub missed: u64,
    /// Magazines filled or topped up at a center the hull stood at.
    pub rearmed_at_center: u64,
    /// Ammo runs dispatched.
    pub ammo_runs: u64,
    /// Ammo runs that reached their post with rounds.
    pub ammo_delivered: u64,
    /// Hulls that flew to a center to rearm.
    pub returns: u64,
    /// Kilotonnes of rounds fabricated from basics.
    pub fabricated_kt: f64,
    /// Sentries ordered (R-WAR47's census).
    pub sentries_ordered: u64,
    /// Sentries that left their post under fire — wrecked or withdrawn.
    pub sentries_lost: u64,
}

impl Simulation {
    /// Missile and supply counts since the run began (T-139).
    pub fn missile_stats(&self) -> MissileStats {
        self.missile_stats
    }

    /// **What one sentry costs a center** — the hull and its magazine's
    /// basics.
    pub(super) fn sentry_price(&self) -> Price {
        let (hull, class) = (HullType::LimitedOffensive, Class::Butte);
        let l = design_loadout(hull, class, &self.config, &self.combat);
        hull_cost(hull, &self.config) + Price::new(l.magazine as f64 * self.combat.missile_round_kt)
    }

    /// Rounds in a hull's magazine.
    pub(super) fn rounds_aboard(&self, e: Entity) -> u32 {
        let held = self.world.cargo.get(e).map_or(0.0, |c| c.ordnance);
        (held / self.combat.missile_round_kt + 1e-9).floor().max(0.0) as u32
    }

    /// Put `kt` of rounds in a hull's hold.
    fn add_rounds(&mut self, e: Entity, kt: f64) {
        let mut c = self.world.cargo.get(e).copied().unwrap_or_default();
        c.ordnance += kt;
        self.world.cargo.insert(e, c);
    }

    /// **Fill a missile hull's magazine** without drawing on any bank — for
    /// a fleet generated with the galaxy (`seed_fleets`), whose rounds come
    /// into being with it. A hull with no tubes is left as it is.
    pub(super) fn fill_magazine(&mut self, e: Entity) {
        if self.world.loadout.get(e).is_some_and(|l| l.fires_missiles()) {
            let room = self.magazine_room_kt(e);
            if room > 0.0 {
                self.add_rounds(e, room);
            }
        }
    }

    /// Kilotonnes of rounds a hull's magazine lacks.
    fn magazine_room_kt(&self, e: Entity) -> f64 {
        let Some(l) = self.world.loadout.get(e) else { return 0.0 };
        let held = self.world.cargo.get(e).map_or(0.0, |c| c.ordnance);
        (l.magazine as f64 * self.combat.missile_round_kt - held).max(0.0)
    }

    /// **A missile Design's discharge** (T-139): a burst from every tube,
    /// nearest target first.
    ///
    /// A target is fired on only if the rounds this hull's post can launch
    /// now, with the rounds already flying at it, exceed what its empire's
    /// point defense around it is believed to stop
    /// ([`Standing::launches_into`]) — a salvo point defense absorbs whole is
    /// rounds the supply line paid for and nothing else. It then takes no more
    /// rounds than that capacity plus what its structure still needs. A hull
    /// that holds fire on every target stops firing until its situation
    /// changes; one with an empty magazine is resupplied.
    pub(super) fn missile_salvo(&mut self, shooter: Entity, loadout: Loadout, at: Vec3, mut aim: Vec<(f64, Entity)>) {
        if self.rounds_aboard(shooter) == 0 && !self.resupply(shooter) {
            return;
        }
        aim.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let burst = self.burst_of(&loadout);
        let mut tubes = burst.min(self.rounds_aboard(shooter));
        let ready = self.post_ready_rounds(shooter);
        let warhead = self.combat.missile_warhead_kj;
        let seat = self.world.owner.get(shooter).map_or(0, |o| o.0);
        let doctrine = self.doctrine_of(seat as usize);
        let (mut held, mut launched) = (false, 0u32);
        for (d, t) in aim {
            if tubes == 0 {
                break;
            }
            let owed = self.missile_inbound.get(&t).copied().unwrap_or(0.0);
            let left = self.structure_of(t) - self.world.hull_damage.get(t).copied().unwrap_or(0.0);
            let Some((years, speed)) = crate::combat::missile_flight(d, loadout.missile_accel, &self.combat) else {
                continue;
            };
            let Some(p) = self.position_at(t, self.clock) else { continue };
            let capacity = self.point_defense_capacity(t, p, speed);
            let flying = (owed / warhead).round() as u32;
            if !Standing::of(&doctrine).launches_into(capacity, ready + flying) {
                held = true;
                continue;
            }
            let wanted = capacity + ((left / warhead).ceil().max(1.0) as u32);
            let n = tubes.min(wanted.saturating_sub(flying));
            if n == 0 {
                continue;
            }
            launched += n;
            for _ in 0..n {
                self.launch(shooter, t, at, loadout.fire_enemy_ly, years, speed);
            }
            tubes -= n;
        }
        // Holding fire on everything in reach: nothing changes until a hull
        // there moves, and detection finds that.
        if held && launched == 0 {
            self.stop_firing(shooter);
            return;
        }
        self.schedule(loadout.discharge_years, EventKind::Discharge { shooter });
    }

    /// Rounds one salvo launches: a burst from every tube — the arena's burst
    /// size (R-WAR44).
    fn burst_of(&self, loadout: &Loadout) -> u32 {
        loadout.tubes * self.combat.burst_count as u32
    }

    /// **Rounds the missile hulls at a shooter's post can launch in one
    /// salvo** — the shooter's own and its fellows'. Co-located launchers
    /// discharge at one instant, so this is the salvo a target's point defense
    /// actually faces, and every launcher there reads the same number.
    fn post_ready_rounds(&self, shooter: Entity) -> u32 {
        let seat = self.world.owner.get(shooter).map_or(0, |o| o.0);
        let mut hulls = self.world.voyage.get(shooter).map_or_else(Vec::new, |v| self.missile_hulls_at(seat, v.target));
        if !hulls.contains(&shooter) {
            hulls.push(shooter);
        }
        hulls
            .iter()
            .map(|&h| {
                let b = self.world.loadout.get(h).map_or(0, |l| self.burst_of(l));
                b.min(self.rounds_aboard(h))
            })
            .sum()
    }

    /// One round leaves `shooter` for `target`.
    fn launch(&mut self, shooter: Entity, target: Entity, from: Vec3, reach: f64, years: f64, speed: f64) {
        let mass_kt = self.combat.missile_round_kt;
        let warhead_kj = self.combat.missile_warhead_kj;
        if let Some(c) = self.world.cargo.get_mut(shooter) {
            c.ordnance = (c.ordnance - mass_kt).max(0.0);
        }
        let owner = self.world.owner.get(shooter).map_or(0, |o| o.0);
        let id = self.next_missile;
        self.next_missile += 1;
        let impact = self.clock + years;
        let missile =
            Missile { shooter, owner, target, from, reach, speed, impact, stopped_by: None, mass_kt, warhead_kj };
        self.missiles.insert(id, missile);
        *self.missile_inbound.entry(target).or_default() += warhead_kj;
        self.missile_stats.launched += 1;
        // Point defense engages from the moment the round enters the widest
        // point-defense range any Design has.
        let widest = crate::combat::point_defense_range_ly(
            self.combat.laser_hit_tolerance * self.combat.fire_control_by_class.max(),
            &self.combat,
        );
        let enters = (impact - widest / speed.max(1e-9)).max(self.clock);
        self.schedule_at(enters, EventKind::MissileEnters { missile: id });
        self.schedule_at(impact, EventKind::MissileArrive { missile: id });
    }

    /// **A round enters point-defense range** (T-139; R-WAR45, the author's
    /// ruling: point defense comes out of a beam's ordinary firing cycle, and
    /// its rate is what volume of fire overwhelms). Of the target's defenders
    /// ([`Simulation::point_defenders`]), the one that can finish the round
    /// soonest commits its mounts from when it is free, or from when the round
    /// enters its own range, until it is done — if that is before impact. Its
    /// discharges in that interval deliver nothing at hulls.
    pub(super) fn sys_missile_enters(&mut self, id: u64) {
        let Some(&m) = self.missiles.get(&id) else { return };
        if !self.live_hull(m.target) {
            return;
        }
        let Some(at) = self.position_at(m.target, self.clock) else { return };
        let now = self.clock;
        let mut best: Option<(f64, f64, Entity)> = None;
        for (d, window, busy) in self.point_defenders(m.target, at, m.speed) {
            let free = self.pd_busy.get(&d).and_then(|v| v.last()).map_or(f64::NEG_INFINITY, |&(_, end)| end);
            let start = free.max(m.impact - window).max(now);
            let done = start + busy;
            if done <= m.impact && best.is_none_or(|(t, _, e)| done < t || (done == t && d < e)) {
                best = Some((done, start, d));
            }
        }
        let Some((done, start, d)) = best else { return };
        let slots = self.pd_busy.entry(d).or_default();
        slots.retain(|&(_, end)| end > now);
        slots.push((start, done));
        if let Some(mm) = self.missiles.get_mut(&id) {
            mm.stopped_by = Some(d);
        }
    }

    /// **Are this beam hull's mounts committed to point defense now?**
    pub(super) fn defending_now(&self, e: Entity) -> bool {
        let now = self.clock;
        self.pd_busy.get(&e).is_some_and(|v| v.iter().any(|&(start, end)| start <= now && now < end))
    }

    /// **A round reaches its target's position** (T-139): it misses if the
    /// target is gone or has left the round's powered reach; otherwise point
    /// defense engages it, and a round it does not stop delivers its warhead
    /// through the beam's own damage and wreck path.
    pub(super) fn sys_missile_arrive(&mut self, id: u64) {
        let Some(m) = self.missiles.remove(&id) else { return };
        if let Some(owed) = self.missile_inbound.get_mut(&m.target) {
            *owed -= m.warhead_kj;
            if *owed <= m.warhead_kj * 1e-9 {
                self.missile_inbound.remove(&m.target);
            }
        }
        self.ordnance_debris += m.mass_kt;
        let now = self.clock;
        let target_seat = self.world.owner.get(m.target).map_or(0, |o| o.0);
        let at = self.position_at(m.target, now).filter(|_| self.live_hull(m.target));
        let outcome = match at {
            Some(p) if p.distance(m.from) <= m.reach + STATION_RADIUS.1 => {
                // Stopped only if the defender that took it is still there to
                // finish it.
                if m.stopped_by.is_some_and(|d| self.live_hull(d)) {
                    MissileOutcome::Intercepted
                } else {
                    // The beam's own damage, wreck roll and fleet decision.
                    self.deliver(m.shooter, m.target, m.warhead_kj);
                    MissileOutcome::Hit
                }
            }
            _ => MissileOutcome::Missed,
        };
        match outcome {
            MissileOutcome::Hit => self.missile_stats.hits += 1,
            MissileOutcome::Intercepted => self.missile_stats.intercepted += 1,
            MissileOutcome::Missed => self.missile_stats.missed += 1,
        }
        self.log.push(
            now,
            LogEvent::MissileResolved { player: m.owner, vehicle: m.shooter, target: m.target, target_seat, outcome },
        );
    }

    /// **The point-defense hulls around a target** (T-139, the author's
    /// ruling: intercept by nearby allies): every live hull of the target's
    /// empire that mounts beams, whose Doctrine defends
    /// ([`Standing::point_defense`]), and that stands within its point-defense
    /// range ([`crate::combat::point_defense_range_ly`]) of the target — the
    /// target itself included. Each with the years a round arriving at `speed`
    /// spends inside its range, and the years its mounts take to destroy one
    /// round: `⌈structure / (power · period)⌉` discharges shared over its mounts.
    fn point_defenders(&self, target: Entity, at: Vec3, speed: f64) -> Vec<(Entity, f64, f64)> {
        let Some(&owner) = self.world.owner.get(target) else { return Vec::new() };
        let doctrine = self.doctrine_of(owner.0 as usize);
        let standing = Standing::of(&doctrine);
        let now = self.clock;
        let mut out = Vec::new();
        for &d in &self.armed {
            if self.world.owner.get(d) != Some(&owner) || !self.live_hull(d) {
                continue;
            }
            let Some(l) = self.world.loadout.get(d) else { continue };
            let role = self.world.role.get(d).copied().unwrap_or(Role::Reserve);
            if !l.has_beams() || l.discharge_years <= 0.0 || !standing.point_defense(role) {
                continue;
            }
            let range = crate::combat::point_defense_range_ly(l.fire_control_ly, &self.combat);
            let Some(p) = self.position_at(d, now) else { continue };
            if p.distance(at) > range {
                continue;
            }
            let per = l.beam_power_kj_per_year * l.discharge_years;
            let shots = (self.combat.missile_structure_kj / per).ceil().max(1.0);
            out.push((d, range / speed.max(1e-9), shots * l.discharge_years / l.beams as f64));
        }
        out
    }

    /// **Rounds a target's point defense is believed to stop in one
    /// window** — the launcher's belief, at a range where light takes days
    /// (R-O41: belief equals truth this close). Each defender stops
    /// `⌊window / time per round⌋`.
    fn point_defense_capacity(&self, target: Entity, at: Vec3, speed: f64) -> u32 {
        self.point_defenders(target, at, speed).iter().map(|&(_, window, busy)| (window / busy).floor() as u32).sum()
    }

    // --- Sentries --------------------------------------------------------

    /// **Order a sentry for `center`** (T-139): it leaves the berth when the
    /// build clears and takes station at the center.
    pub(super) fn spawn_sentry(&mut self, p: usize, built: BuiltHull, center: Entity, launch_delay: f64) {
        let hull = built.hull;
        let e = self.world.spawn();
        self.world.owner.insert(e, PlayerId(p as u32));
        self.world.role.insert(e, Role::Sentry);
        self.world.hull_type.insert(e, hull);
        self.stamp_composition(e, hull, &built.mix);
        self.stamp_loadout(e, built);
        self.world.voyage.insert(e, Voyage { target: center, heading_bias: None, hops: 0 });
        self.world.home_center.insert(e, center);
        self.world.cargo.insert(e, Minerals::default());
        self.world.pop_cargo.insert(e, Kilotons::ZERO);
        self.sentries.entry(center.0).or_default().push(e);
        self.missile_stats.sentries_ordered += 1;
        let at = *self.world.position.get(center).unwrap();
        let accel = self.laden_accel(e);
        let arrive = self.set_leg(e, at, at, accel, launch_delay);
        self.schedule_at(arrive, EventKind::SentryArrive { vehicle: e });
        let pid = *self.world.planet_id.get(center).unwrap();
        self.log.push(
            self.clock,
            LogEvent::VehicleSpawned {
                player: p as u32,
                vehicle: e,
                role: Role::Sentry,
                hull,
                from: at,
                to: pid,
                settlers: 0.0,
                endowment: 0.0,
            },
        );
    }

    /// **A sentry takes station** at its center and fills its magazine there.
    pub(super) fn sys_sentry_arrive(&mut self, e: Entity) {
        let Some(center) = self.world.voyage.get(e).map(|v| v.target) else { return };
        let Some(&at) = self.world.position.get(center) else { return };
        self.arm_at(e, center);
        self.park(e, at);
        let (Some(&o), Some(&pid)) = (self.world.owner.get(e), self.world.planet_id.get(center)) else { return };
        self.log.push(self.clock, LogEvent::VehicleParked { player: o.0, vehicle: e, role: Role::Sentry, at: pid });
    }

    // --- Resupply --------------------------------------------------------

    /// **Fill a hull's magazine at `center`** from its empire's holding there:
    /// rounds held first, then rounds fabricated from the basics the center's
    /// standing order leaves ([`Simulation::available_at`]), one kilotonne of
    /// rounds from one of basics. Returns the kilotonnes loaded. What the
    /// center could not supply is recorded as its ordnance shortfall.
    pub(super) fn arm_at(&mut self, e: Entity, center: Entity) -> f64 {
        let room = self.magazine_room_kt(e);
        let Some(&owner) = self.world.owner.get(e) else { return 0.0 };
        if room <= 0.0 {
            return 0.0;
        }
        self.stock_rounds(owner.0, center, room);
        let held = self.holding(owner.0, center).map_or(0.0, |h| h.ordnance);
        let loaded = room.min(held);
        if loaded <= 0.0 {
            return 0.0;
        }
        self.holding_mut(owner.0, center).ordnance -= loaded;
        if let Some(c) = self.world.cargo.get_mut(e) {
            c.ordnance += loaded;
        }
        self.missile_stats.rearmed_at_center += 1;
        loaded
    }

    /// **Fabricate up to `kt` of rounds at `seat`'s own `center`** from the
    /// basics its standing order leaves, one kilotonne of rounds from one of
    /// basics in proportion to the bank's mix (design law #11). Returns kt
    /// made.
    pub(super) fn fabricate_rounds(&mut self, seat: u32, center: Entity, kt: f64) -> f64 {
        if kt <= 0.0 || self.world.owner.get(center).is_none_or(|o| o.0 != seat) {
            return 0.0;
        }
        let take = kt.min(self.available_at(center).basic_total().kilotons());
        if take <= 1e-12 {
            return 0.0;
        }
        let Some(basics) = self.held_at_mut(center).unwrap().try_take_total(Price::new(take)) else { return 0.0 };
        let made = basics.basic_total().kilotons();
        self.held_at_mut(center).unwrap().ordnance += made;
        self.missile_stats.fabricated_kt += made;
        made
    }

    /// **Make sure `center` holds `want` kt of rounds for `seat`**, fabricating
    /// the difference when the center is the seat's own. Records any
    /// shortfall as the center's ordnance need. Returns kt fabricated.
    fn stock_rounds(&mut self, seat: u32, center: Entity, want: f64) -> f64 {
        let held = self.holding(seat, center).map_or(0.0, |h| h.ordnance);
        let lack = want - held;
        if lack <= 0.0 {
            self.ordnance_short.remove(&center.0);
            return 0.0;
        }
        let made = self.fabricate_rounds(seat, center, lack);
        let still = lack - made;
        if still > 1e-12 {
            *self.ordnance_short.entry(center.0).or_default() = still;
        } else {
            self.ordnance_short.remove(&center.0);
        }
        made
    }

    /// **The center of `seat` a hull is standing at**, if any — within two
    /// station-keeping radii of it, where a hull holding station there is.
    fn center_at(&self, seat: u32, e: Entity) -> Option<Entity> {
        let here = self.position_at(e, self.clock)?;
        let c = self.nearest_center(seat, here)?;
        let at = *self.world.position.get(c)?;
        (at.distance(here) <= 2.0 * STATION_RADIUS.1).then_some(c)
    }

    /// **A missile hull is out of rounds** (T-139). Returns whether it holds
    /// rounds again now — only possible at a center.
    fn resupply(&mut self, e: Entity) -> bool {
        // Already flying to rearm: nothing to fire and nothing more to ask.
        if self.rearm_trips.contains_key(&e.0) {
            self.stop_firing(e);
            return false;
        }
        let Some(seat) = self.world.owner.get(e).map(|o| o.0) else { return false };
        let doctrine = self.doctrine_of(seat as usize);
        let center = self.center_at(seat, e);
        let hauler_free = !self.reserve_freighters[seat as usize].is_empty();
        let Some(post) = center.or_else(|| self.world.voyage.get(e).map(|v| v.target)) else { return false };
        match Standing::of(&doctrine).resupply(center.is_some(), hauler_free) {
            Resupply::AtCenter => {
                if self.arm_at(e, post) > 0.0 && self.rounds_aboard(e) > 0 {
                    return true;
                }
                // The center cannot make rounds now: rounds held elsewhere —
                // another center, or a lot the Exchange delivered — come by
                // ammo run while a hauler is free.
                if self.ammo_runs.contains_key(&(seat, post.0)) || (hauler_free && self.send_ammo_run(seat, post)) {
                    self.stop_firing(e);
                    return false;
                }
                // Nothing to send; look again a cycle on.
                self.schedule(self.config.cycle_years, EventKind::Discharge { shooter: e });
                false
            }
            Resupply::AmmoRun => {
                if !self.ammo_runs.contains_key(&(seat, post.0)) && !self.send_ammo_run(seat, post) {
                    self.return_to_rearm(e, post);
                } else {
                    self.stop_firing(e);
                }
                false
            }
            Resupply::Return => {
                self.return_to_rearm(e, post);
                false
            }
        }
    }

    /// **Kilotonnes of rounds the missile hulls of `seat` at `post` lack.**
    fn rounds_wanted_at(&self, seat: u32, post: Entity) -> f64 {
        self.missile_hulls_at(seat, post).iter().map(|&h| self.magazine_room_kt(h)).sum()
    }

    /// The missile hulls of `seat` standing at `post` — its picket or
    /// blockade stack there, or its sentries.
    fn missile_hulls_at(&self, seat: u32, post: Entity) -> Vec<Entity> {
        let mut out: Vec<Entity> = Vec::new();
        if let Some((holder, stack, _)) = self.picket.get(&post.0) {
            if *holder == seat {
                out.extend(stack.iter().copied());
            }
        }
        if let Some(stack) = self.blockade.get(&(post.0, seat)) {
            out.extend(stack.iter().copied());
        }
        if let Some(s) = self.sentries.get(&post.0) {
            out.extend(s.iter().copied());
        }
        out.retain(|&h| self.live_hull(h) && self.world.loadout.get(h).is_some_and(|l| l.fires_missiles()));
        out.sort();
        out.dedup();
        out
    }

    /// The center of `seat` nearest `from`, off the per-seat index.
    fn nearest_center(&self, seat: u32, from: Vec3) -> Option<Entity> {
        let mut best: Option<(f64, Entity)> = None;
        for &c in &self.owned_planets[seat as usize] {
            let d = self.world.position.get(c).map_or(f64::INFINITY, |p| p.distance(from));
            if best.is_none_or(|(bd, be)| d < bd || (d == bd && c < be)) {
                best = Some((d, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// **Where an ammo run for `post` loads** — the nearest place `seat` can
    /// fill it from: a holding with the rounds, or one of its own centers
    /// with the basics to make them. `None` when no place can.
    fn ammo_source(&self, seat: u32, post: Entity, want: f64) -> Option<Entity> {
        let post_at = *self.world.position.get(post)?;
        let mut best: Option<(f64, Entity)> = None;
        let mut consider = |sim: &Simulation, c: Entity| {
            let held = sim.holding(seat, c).map_or(0.0, |h| h.ordnance);
            let own = sim.world.owner.get(c).is_some_and(|o| o.0 == seat);
            let makes = if own { sim.available_at(c).basic_total().kilotons() } else { 0.0 };
            if held + makes + 1e-12 < want {
                return;
            }
            let d = sim.world.position.get(c).map_or(f64::INFINITY, |p| p.distance(post_at));
            if best.is_none_or(|(bd, be)| d < bd || (d == bd && c < be)) {
                best = Some((d, c));
            }
        };
        for &c in &self.owned_planets[seat as usize] {
            consider(self, c);
        }
        for (&(p, at), h) in &self.holdings.elsewhere {
            if p == seat && h.ordnance > 0.0 {
                consider(self, Entity(at));
            }
        }
        best.map(|(_, c)| c)
    }

    /// **Send an ammo run to `post`** (T-139): the idle hauler nearest the
    /// [`Self::ammo_source`] flies there, loads rounds, and carries them out.
    /// Returns `false` when no place can fill it or no hauler is idle.
    fn send_ammo_run(&mut self, seat: u32, post: Entity) -> bool {
        let want = self.rounds_wanted_at(seat, post);
        if want <= 0.0 {
            return false;
        }
        let Some(source) = self.ammo_source(seat, post, want) else { return false };
        let source_at = *self.world.position.get(source).unwrap();
        let Some(h) = self.take_nearest_reserve(seat as usize, false, source_at) else { return false };
        self.world.role.insert(h, Role::Freighter);
        self.ammo_runs.insert((seat, post.0), h);
        self.side_runs.insert(h.0, SideRun::AmmoPickup { source, post });
        self.missile_stats.ammo_runs += 1;
        let from = self.position_at(h, self.clock).unwrap_or(source_at);
        let accel = self.laden_accel(h);
        let arrive = self.set_leg(h, from, source_at, accel, 0.0);
        self.schedule_at(arrive, EventKind::DutyArrive { vehicle: h });
        true
    }

    /// **One leg of an ammo run ends** (called from `sys_duty_arrive`).
    pub(super) fn ammo_leg(&mut self, h: Entity, run: SideRun) {
        let Some(seat) = self.world.owner.get(h).map(|o| o.0) else { return };
        match run {
            SideRun::AmmoPickup { source, post } => {
                let hull = self.world.hull_type.get(h).copied().unwrap_or(HullType::MediumSystems);
                let hold = hull.cargo_capacity(&self.config).kilotons();
                let want = self.rounds_wanted_at(seat, post).min(hold);
                self.stock_rounds(seat, source, want);
                let have = self.holding(seat, source).map_or(0.0, |m| m.ordnance).min(want);
                if have > 0.0 {
                    self.holding_mut(seat, source).ordnance -= have;
                    self.add_rounds(h, have);
                }
                let from = *self.world.position.get(source).unwrap();
                let to = *self.world.position.get(post).unwrap();
                self.side_runs.insert(h.0, SideRun::AmmoDeliver { source, post });
                let accel = self.laden_accel(h);
                let arrive = self.set_leg(h, from, to, accel, 0.0);
                self.schedule_at(arrive, EventKind::DutyArrive { vehicle: h });
            }
            SideRun::AmmoDeliver { source, post } => {
                self.ammo_runs.remove(&(seat, post.0));
                let mut aboard = self.world.cargo.get(h).map_or(0.0, |c| c.ordnance);
                if aboard > 0.0 {
                    self.missile_stats.ammo_delivered += 1;
                }
                for m in self.missile_hulls_at(seat, post) {
                    let give = self.magazine_room_kt(m).min(aboard);
                    if give <= 0.0 {
                        continue;
                    }
                    aboard -= give;
                    if let Some(c) = self.world.cargo.get_mut(h) {
                        c.ordnance -= give;
                    }
                    self.add_rounds(m, give);
                    // Back in the fight: detection finds what is in reach.
                    self.track_changed(m);
                }
                let from = *self.world.position.get(post).unwrap();
                let home =
                    if self.owns_planet(source) { source } else { self.nearest_center(seat, from).unwrap_or(source) };
                let to = *self.world.position.get(home).unwrap();
                self.side_runs.insert(h.0, SideRun::AmmoHome { center: home });
                let accel = self.laden_accel(h);
                let arrive = self.set_leg(h, from, to, accel, 0.0);
                self.schedule_at(arrive, EventKind::DutyArrive { vehicle: h });
            }
            SideRun::AmmoHome { center } => {
                self.deliver_side_cargo(h, PlayerId(seat), center);
                let at = *self.world.position.get(center).unwrap();
                let pid = *self.world.planet_id.get(center).unwrap();
                self.park(h, at);
                self.release_to_reserve(h, Role::Freighter, pid);
            }
            _ => {}
        }
    }

    /// **A hull flies to the nearest center to rearm** (T-139), off its post's
    /// books like a picket on a sortie, and back on them when it returns.
    fn return_to_rearm(&mut self, e: Entity, post: Entity) {
        // Every path out of here leaves the hull not firing: a shooter left in
        // `firing` with no discharge scheduled could never start another.
        self.stop_firing(e);
        let Some(seat) = self.world.owner.get(e).map(|o| o.0) else { return };
        let here = self.position_at(e, self.clock).unwrap_or(Vec3::ZERO);
        let Some(center) = self.nearest_center(seat, here) else { return };
        if self.world.role.get(e).copied() == Some(Role::Picket) {
            if self.blockade.get(&(post.0, seat)).is_some_and(|st| st.contains(&e)) {
                self.leave_blockade(post.0, seat, e);
                self.bind(seat, post.0);
            } else if self.detach_picket(post, e).is_none() {
                return;
            }
            *self.picket_inbound.entry(post.0).or_default() += 1;
        }
        self.rearm_trips.insert(e.0, (post, center));
        self.missile_stats.returns += 1;
        let dest = *self.world.position.get(center).unwrap();
        let arrive = self.course_change(e, dest);
        self.schedule_at(arrive, EventKind::RearmArrive { vehicle: e });
    }

    /// **A hull reaches the center it flew to rearm at**: it rearms and flies
    /// back to its post, where it takes station as a picket arriving.
    pub(super) fn sys_rearm_arrive(&mut self, e: Entity) {
        let Some((post, center)) = self.rearm_trips.remove(&e.0) else { return };
        if !self.live_hull(e) {
            return;
        }
        self.arm_at(e, center);
        let dest = *self.world.position.get(post).unwrap();
        let arrive = self.course_change(e, dest);
        let ev = match self.world.role.get(e).copied() {
            Some(Role::Sentry) => EventKind::SentryArrive { vehicle: e },
            _ => EventKind::PicketArrive { vehicle: e },
        };
        self.schedule_at(arrive, ev);
    }

    /// **Take a hull off whatever missile books it is on** — a sentry's
    /// center, a rearm trip — when it is wrecked or leaves (called from
    /// `leave_post`).
    ///
    /// **A sentry that leaves is a loss to its center** (R-WAR47), whether it
    /// is wrecked at its post or withdraws from it: both callers of
    /// `leave_post` are answers to fire, and a withdrawn sentry stands down to
    /// Reserve and holds fire on neutrals, so the post is gone either way.
    /// Measured on the card bed, withdrawals were all of the losses: 1,922 in
    /// the last 50 of 800 years on seed 1, against no sentry wrecked as a
    /// sentry (appendix §D.24).
    pub(super) fn leave_missile_post(&mut self, e: Entity) {
        self.rearm_trips.remove(&e.0);
        if self.world.role.get(e).copied() == Some(Role::Sentry) {
            if let Some(center) = self.world.voyage.get(e).map(|v| v.target) {
                *self.sentries_lost.entry(center.0).or_default() += 1;
                self.missile_stats.sentries_lost += 1;
                if let Some(s) = self.sentries.get_mut(&center.0) {
                    s.retain(|&h| h != e);
                    if s.is_empty() {
                        self.sentries.remove(&center.0);
                    }
                }
            }
        }
        self.pd_busy.remove(&e);
    }

    /// **A center's offers in the ordnance book** (T-139, matching §10.5) —
    /// only while its empire's Doctrine opens the book.
    ///
    /// - A **bid** for the rounds its missile hulls wait on and it could not
    ///   make, at the mean of its three basic willingnesses to pay: a round is
    ///   a kilotonne of basics.
    /// - An **ask** for the rounds it holds and the rounds it could make from
    ///   basics above its next works bill, at the same price; sold capacity is
    ///   fabricated when the contract comes due.
    pub(super) fn post_ordnance_offers(
        &mut self,
        e: Entity,
        owner: PlayerId,
        doctrine: &Doctrine,
        deficit: &[Price; 3],
        bank: &Minerals,
    ) {
        if !Standing::of(doctrine).trades_ordnance() {
            return;
        }
        let book = Material::Ordnance as usize;
        let price = Basic::ALL.iter().map(|&c| self.willingness_to_pay(e, c, doctrine)).sum::<f64>() / 3.0;
        let pos = *self.world.position.get(e).unwrap();
        let at = [pos.x, pos.y, pos.z];
        let short = self.ordnance_short.get(&e.0).copied().unwrap_or(0.0).min(self.rounds_wanted_at(owner.0, e));
        if short > 1e-9 {
            if price > 0.0 {
                self.exchange.posted[book].0 += 1;
                self.exchange.markets[book].bids.push(matching::Offer {
                    entity: e.0,
                    price,
                    qty: short,
                    pos: at,
                    owner,
                });
            }
            return;
        }
        let bill = self.next_bill(e, owner).unwrap_or([Price::ZERO; 3]);
        let spare_basics: f64 = Basic::ALL
            .iter()
            .map(|&c| {
                let i = c as usize;
                if deficit[i] > Price::ZERO {
                    0.0
                } else {
                    (bank.get_basic(c) - bill[i].kilotons()).max(0.0)
                }
            })
            .sum();
        let qty = bank.ordnance.max(0.0) + spare_basics;
        if qty > 1e-9 {
            self.exchange.posted[book].1 += 1;
            self.exchange.markets[book].asks.push(matching::Offer { entity: e.0, price, qty, pos: at, owner });
        }
    }

    /// **Re-file a seat's sentries after a Doctrine write** — a write that
    /// makes neutrals enemies lets a sentry fire on an unarmed hull.
    pub(super) fn classify_sentries(&mut self, seat: usize) {
        let mine: Vec<Entity> = self
            .sentries
            .values()
            .flatten()
            .copied()
            .filter(|&e| self.world.owner.get(e).is_some_and(|o| o.0 as usize == seat) && self.armed.contains(&e))
            .collect();
        for e in mine {
            self.classify_fire(e);
        }
    }

    /// Kilotonnes of rounds in flight and spent — the ledger's ordnance
    /// outside any hold or holding.
    pub(super) fn ordnance_outside(&self) -> f64 {
        self.ordnance_debris + self.missiles.values().map(|m| m.mass_kt).sum::<f64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::GalaxyConfig;

    /// Two seats on a 200-planet field, regrowth off so the mass ledger is a
    /// closed sum.
    fn bed(seed: u64) -> Simulation {
        let mut g = GalaxyConfig::new(2, seed);
        g.planet_count = 200;
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = 60.0;
        cfg.biosphere_regen_rate = 0.0;
        Simulation::with_baseline(Galaxy::generate(g).unwrap(), cfg)
    }

    fn home(sim: &Simulation, seat: usize) -> Entity {
        sim.world.player_info.get(sim.player_entity[seat]).unwrap().home
    }

    fn pos(sim: &Simulation, e: Entity) -> Vec3 {
        *sim.world.position.get(e).unwrap()
    }

    /// A hull of `seat`'s Design `(hull, class)` in `role`, standing at `at`
    /// with `post` as its voyage target — placed through the engine's own
    /// [`Simulation::park`], so detection finds it the way it finds a hull
    /// that flew there.
    fn hull(sim: &mut Simulation, seat: u32, role: Role, design: (HullType, Class), at: Vec3, post: Entity) -> Entity {
        let e = sim.world.spawn();
        sim.world.owner.insert(e, PlayerId(seat));
        sim.world.role.insert(e, role);
        sim.world.hull_type.insert(e, design.0);
        sim.world.design_class.insert(e, design.1);
        let l = design_loadout(design.0, design.1, &sim.config, &sim.combat);
        if l.is_armed() {
            sim.world.loadout.insert(e, l);
        }
        sim.world.cargo.insert(e, Minerals::default());
        sim.world.voyage.insert(e, Voyage { target: post, heading_bias: None, hops: 0 });
        let h = home(sim, seat as usize);
        sim.world.home_center.insert(e, h);
        sim.park(e, at);
        e
    }

    /// A sentry of `seat` at its homeworld, magazine full.
    fn sentry(sim: &mut Simulation, seat: u32) -> Entity {
        let h = home(sim, seat as usize);
        let at = pos(sim, h);
        let e = hull(sim, seat, Role::Sentry, (HullType::LimitedOffensive, Class::Butte), at, h);
        sim.sentries.entry(h.0).or_default().push(e);
        assert!(sim.arm_at(e, h) > 0.0, "the homeworld can make the first magazine");
        sim.track_changed(e);
        e
    }

    fn run_for(sim: &mut Simulation, years: f64) {
        let t = sim.clock + years;
        sim.config.horizon_years = sim.config.horizon_years.max(t + 1.0);
        while sim.clock < t && sim.step() {}
    }

    fn resolved(sim: &Simulation, target: Entity) -> (usize, usize, usize) {
        let (mut hit, mut stopped, mut missed) = (0, 0, 0);
        for r in sim.log().iter() {
            if let LogEvent::MissileResolved { target: t, outcome, .. } = r.event {
                if t == target {
                    match outcome {
                        MissileOutcome::Hit => hit += 1,
                        MissileOutcome::Intercepted => stopped += 1,
                        MissileOutcome::Missed => missed += 1,
                    }
                }
            }
        }
        (hit, stopped, missed)
    }

    fn logged(seed: u64) -> Simulation {
        let mut sim = bed(seed);
        sim.set_log_filter(crate::log::LogFilter::none().with(crate::log::LogCategory::Combat));
        sim
    }

    /// **The missile Designs mount tubes and a magazine, and outrange a beam
    /// from the same hull three times over** (T-139).
    #[test]
    fn the_missile_designs_mount_tubes_and_outrange_a_beam() {
        let sim = bed(1);
        let (cfg, combat) = (&sim.config, &sim.combat);
        let cairn = design_loadout(HullType::LimitedContactVehicle, Class::Cairn, cfg, combat);
        let warship = design_loadout(HullType::LimitedOffensive, Class::Unnamed, cfg, combat);
        assert!(warship.has_beams() && !warship.fires_missiles(), "the unnamed warship still mounts a beam");
        for class in [Class::Butte, Class::Mesa] {
            let l = design_loadout(HullType::LimitedOffensive, class, cfg, combat);
            assert!(l.fires_missiles() && !l.has_beams(), "{class:?} mounts tubes in place of beams");
            assert_eq!(l.magazine, l.tubes * combat.missile_rounds_per_tube);
            assert!(l.fire_enemy_ly > 2.9 * cairn.fire_enemy_ly, "{class:?} reaches {}", l.fire_enemy_ly);
            assert!(l.is_armed());
        }
    }

    /// **A round flies under power only, below light, and arrives later the
    /// farther it goes** (T-139).
    #[test]
    fn a_round_reaches_only_under_power_and_never_at_light_speed() {
        let combat = CombatConfig::default();
        let a = combat.missile_accel_multiplier * G * 0.91;
        let reach = crate::combat::missile_reach_ly(a, &combat);
        let mut last = 0.0;
        for k in 1..=10 {
            let (t, v) = crate::combat::missile_flight(reach * k as f64 / 10.0, a, &combat).unwrap();
            assert!(t > last && t <= combat.missile_fuel_years + 1e-12, "flight time rises within the burn");
            assert!(v > 0.0 && v < 1.0, "below light: {v}");
            last = t;
        }
        assert!(crate::combat::missile_flight(reach * 1.001, a, &combat).is_none(), "nothing past the reach");
    }

    /// **The sentry Design is locked behind the missile card** — no default
    /// role is on a missile Design, nothing orders a sentry, and the card's
    /// two writes open both (T-139; `AGENTS.md`'s *a lock needs both halves*).
    #[test]
    fn the_sentry_design_is_locked_behind_the_missile_card() {
        let d = Doctrine::default();
        let st = Standing::of(&d);
        let rich = Kilotons::new(1e12);
        assert_eq!(st.sentries_wanted(rich, Price::new(0.03), 0), 0, "no center orders a sentry by default");
        for role in [Role::Scout, Role::Colonizer, Role::Miner, Role::Picket] {
            assert!(!st.design_for(role).1.is_missile(), "{role:?} is not on a missile Design by default");
        }
        let card = crate::cards::card(crate::cards::CardId(13)).unwrap();
        assert!(card.writes(CardEffect::UnlockDesign(HullType::LimitedOffensive, Class::Butte)));
        let mut played = d;
        for &w in card.effects {
            if let CardEffect::WriteDoctrine(w) = w {
                crate::cards::apply_doctrine_write(&mut played, w);
            }
        }
        let st = Standing::of(&played);
        // In proportion to what there is to defend (the author's ruling).
        let one = Price::new(0.03);
        let per = one.kilotons() / crate::cards::MISSILE_SENTRY_RATIO;
        assert_eq!(st.sentries_wanted(Kilotons::new(per * 0.99), one, 0), 0);
        assert_eq!(st.sentries_wanted(Kilotons::new(per * 1.01), one, 0), 1);
        assert_eq!(st.sentries_wanted(Kilotons::new(per * 10.01), one, 0), 10, "ten times the value, ten sentries");
        assert_eq!(st.design_for(Role::Sentry), (HullType::LimitedOffensive, Class::Butte));
        assert!(!played.missile_pickets, "the card arms no picket: its supply line is not hardened");
        for role in Class::MISSILE_ROLES {
            assert!(matches!(role, Role::Sentry | Role::Picket), "a missile Design serves near a center or in a stack");
        }
    }

    /// **A sentry fires on an armed rival in its reach and lets unarmed
    /// traffic pass**, and every round's mass is accounted for (T-139).
    #[test]
    fn a_sentry_fires_on_an_armed_rival_and_lets_unarmed_traffic_pass() {
        let mut sim = logged(1);
        let s = sentry(&mut sim, 0);
        let h0 = home(&sim, 0);
        let at = pos(&sim, h0);
        // Outside a beam's reach and inside a round's, on either side.
        let gun = hull(
            &mut sim,
            1,
            Role::Picket,
            (HullType::LimitedContactVehicle, Class::Cairn),
            at.add(Vec3::new(0.015, 0.0, 0.0)),
            h0,
        );
        let ford = hull(
            &mut sim,
            1,
            Role::Reserve,
            (HullType::MediumSystems, Class::Ford),
            at.add(Vec3::new(-0.015, 0.0, 0.0)),
            h0,
        );
        let before = sim.mass_ledger().total();
        run_for(&mut sim, 0.3);
        let (hit, _, _) = resolved(&sim, gun);
        assert!(hit > 0, "the armed rival takes a round: {:?}", sim.missile_stats());
        assert_eq!(resolved(&sim, ford), (0, 0, 0), "no round at an unarmed hull");
        assert!(sim.rounds_aboard(s) < 8 || sim.missile_stats().rearmed_at_center > 1, "the sentry spent rounds");
        let drift = sim.mass_ledger().total() - before;
        assert!(drift.abs() < 1e-9 * before, "rounds conserve mass: {drift:+e}");
        assert!(sim.ordnance_outside() > 0.0, "spent rounds are on the ledger as debris");
    }

    /// **Point defense on a hull that is not the target stops rounds aimed
    /// at its ally** (the author's ruling), and only while its Doctrine says
    /// so.
    #[test]
    fn an_ally_s_point_defense_stops_rounds_aimed_at_a_hull_that_cannot_shoot() {
        let run = |defends: bool| {
            let mut sim = logged(1);
            sim.world.doctrine.get_mut(sim.player_entity[1]).unwrap().point_defense = defends;
            sentry(&mut sim, 0);
            let h0 = home(&sim, 0);
            let at = pos(&sim, h0);
            // Armed (tubes) but no beams, and it fires on no neutral: it can
            // neither shoot back nor defend itself.
            let target = hull(
                &mut sim,
                1,
                Role::Reserve,
                (HullType::LimitedOffensive, Class::Mesa),
                at.add(Vec3::new(0.015, 0.0, 0.0)),
                h0,
            );
            hull(
                &mut sim,
                1,
                Role::Picket,
                (HullType::LimitedContactVehicle, Class::Cairn),
                at.add(Vec3::new(0.015, 0.002, 0.0)),
                h0,
            );
            run_for(&mut sim, 0.3);
            resolved(&sim, target)
        };
        let (_, stopped, _) = run(true);
        assert!(stopped > 0, "the ally shot rounds down");
        let (hit, stopped, _) = run(false);
        assert_eq!(stopped, 0, "no defense when the Doctrine holds it");
        assert!(hit > 0, "and the rounds land");
    }

    /// **A sentry holds fire on a stack whose point defense it cannot
    /// saturate** (T-139, `Standing::launches_into`), and fires on a lone hull
    /// in the same place.
    #[test]
    fn a_sentry_holds_fire_on_point_defense_it_cannot_saturate() {
        for (stack, fires) in [(1usize, true), (6, false)] {
            let mut sim = logged(1);
            sentry(&mut sim, 0);
            let h0 = home(&sim, 0);
            let at = pos(&sim, h0);
            for k in 0..stack {
                let off = Vec3::new(0.015, 0.0005 * k as f64, 0.0);
                hull(&mut sim, 1, Role::Picket, (HullType::LimitedContactVehicle, Class::Cairn), at.add(off), h0);
            }
            run_for(&mut sim, 0.3);
            assert_eq!(sim.missile_stats().launched > 0, fires, "stack of {stack}: {:?}", sim.missile_stats());
        }
    }

    /// The unowned world nearest seat 0's home.
    fn nearest_free_world(sim: &Simulation) -> Entity {
        let at = pos(sim, home(sim, 0));
        sim.planet_entity
            .iter()
            .copied()
            .filter(|&w| !sim.world.owner.contains(w))
            .min_by(|&a, &b| pos(sim, a).distance(at).total_cmp(&pos(sim, b).distance(at)))
            .unwrap()
    }

    /// A missile picket of seat 0 with an empty magazine, posted at the
    /// nearest free world, and an armed seat-1 picket in its reach there.
    fn dry_picket(sim: &mut Simulation) -> (Entity, Entity, Entity) {
        let w = nearest_free_world(sim);
        let at = pos(sim, w);
        sim.world.doctrine.get_mut(sim.player_entity[0]).unwrap().missile_pickets = true;
        let m = hull(sim, 0, Role::Picket, (HullType::LimitedOffensive, Class::Mesa), at, w);
        let clock = sim.clock;
        sim.picket.entry(w.0).or_insert((0, Vec::new(), clock)).1.push(m);
        sim.picket_count[0] += 1;
        // A rival picket that regards seat 0 as an enemy stands and returns
        // fire rather than leaving, so it is still there when the rounds
        // come back; it is out of its own beams' reach.
        sim.world.doctrine.get_mut(sim.player_entity[1]).unwrap().engage_neutrals = true;
        let gun = (HullType::LimitedContactVehicle, Class::Cairn);
        let target = hull(sim, 1, Role::Picket, gun, at.add(Vec3::new(0.02, 0.0, 0.0)), w);
        sim.track_changed(m);
        (m, w, target)
    }

    /// **A missile picket that runs dry is resupplied: by an ammo run while
    /// a hauler is idle, by flying to a center to rearm while none is** — and
    /// either way it ends back on its post with rounds, mass conserved (the
    /// author's choice: return and ammo runs).
    #[test]
    fn a_dry_missile_picket_is_resupplied_by_ammo_run_or_by_return() {
        for hauler in [true, false] {
            let mut sim = bed(1);
            let h0 = home(&sim, 0);
            sim.reserve_freighters[0].clear();
            if hauler {
                let at = pos(&sim, h0);
                let f = hull(&mut sim, 0, Role::Reserve, (HullType::MediumSystems, Class::Ford), at, h0);
                sim.reserve_freighters[0].push(f);
            }
            let (m, w, target) = dry_picket(&mut sim);
            let before = sim.mass_ledger().total();
            run_for(&mut sim, 0.05);
            let st = sim.missile_stats();
            if hauler {
                assert_eq!((st.ammo_runs, st.returns), (1, 0), "{st:?}");
            } else {
                assert_eq!((st.ammo_runs, st.returns), (0, 1), "{st:?}");
            }
            // Until the picket stands at its post again with rounds — the
            // world may be settled later in the run, which ends any picket.
            let w_at = pos(&sim, w);
            let back = |sim: &Simulation| {
                sim.rounds_aboard(m) > 0
                    && sim
                        .world
                        .motion
                        .get(m)
                        .is_some_and(|mo| !mo.under_way(sim.clock) && mo.dest.distance(w_at) < 1e-9)
            };
            sim.config.horizon_years = 60.0;
            while !back(&sim) && sim.clock < 40.0 && sim.step() {}
            assert!(back(&sim), "resupplied by t = 40: {:?}", sim.missile_stats());
            let st = sim.missile_stats();
            if hauler {
                assert_eq!(st.ammo_delivered, 1, "the run reached the post: {st:?}");
            }
            assert!(sim.picket.get(&w.0).is_some_and(|(_, s, _)| s.contains(&m)), "back on its post");
            assert!(!sim.picket_inbound.contains_key(&w.0), "nothing left on the in-flight book");
            assert!(sim.rearm_trips.is_empty(), "no trip left open");
            run_for(&mut sim, 0.1);
            assert!(
                sim.missile_stats().launched > 0,
                "rounds left the post once resupplied: hauler={hauler} t={} {:?} firing={} in_reach={:?} target_live={}",
                sim.clock,
                sim.missile_stats(),
                sim.firing.contains(&m),
                sim.in_reach.get(&m),
                sim.live_hull(target)
            );
            let drift = sim.mass_ledger().total() - before;
            assert!(drift.abs() < 1e-9 * before, "resupply conserves mass: {drift:+e}");
        }
    }

    /// **Point defense comes out of the beam's firing cycle** (R-WAR45, the
    /// author's ruling). A seat-1 Cairn dueling a seat-0 hull is fired on by a
    /// seat-0 sentry it cannot reach. While its mounts are committed to a
    /// round, its discharges put nothing on the hull it is dueling; once the
    /// commitment ends, they do again.
    #[test]
    fn point_defense_takes_a_beam_off_its_target_while_it_defends() {
        let mut sim = logged(1);
        // Seat 1 regards seat 0 as an enemy, so its picket stands under the
        // rounds rather than breaking off (R-WAR26's third ending).
        sim.world.doctrine.get_mut(sim.player_entity[1]).unwrap().engage_neutrals = true;
        sentry(&mut sim, 0);
        let h0 = home(&sim, 0);
        let at = pos(&sim, h0);
        let cairn = (HullType::LimitedContactVehicle, Class::Cairn);
        let gun = hull(&mut sim, 1, Role::Picket, cairn, at.add(Vec3::new(0.015, 0.0, 0.0)), h0);
        // A large hull, so the duel outlasts the rounds' flight, whose fire
        // does not hurt: it stands and returns fire without deciding the duel.
        let big = (HullType::GeneralContactVehicle, Class::Scarp);
        let mine = hull(&mut sim, 0, Role::Picket, big, at.add(Vec3::new(0.019, 0.0, 0.0)), h0);
        let popgun = Loadout { beam_power_kj_per_year: 1e-9, ..*sim.world.loadout.get(mine).unwrap() };
        sim.world.loadout.insert(mine, popgun);
        sim.track_changed(mine);
        let damage = |sim: &Simulation| sim.world.hull_damage.get(mine).copied().unwrap_or(0.0);
        // Until the rival Cairn commits its mounts to a round.
        let mut slot = None;
        while slot.is_none() && sim.clock < 0.3 {
            assert!(sim.step());
            slot = sim.pd_busy.get(&gun).and_then(|v| v.iter().copied().find(|&(_, end)| end > sim.clock));
        }
        let (start, end) = slot.expect("the rival Cairn defended against a round");
        let wait = (start - sim.clock).max(0.0) + 1e-6;
        run_for(&mut sim, wait);
        let before = damage(&sim);
        assert!(before > 0.0, "the duel was on before the round came");
        let inside = end - sim.clock - 1e-6;
        run_for(&mut sim, inside);
        assert_eq!(damage(&sim), before, "no offensive output while defending");
        // A burst chains its rounds' commitments back to back: wait out the
        // last one, then one discharge period.
        while sim.pd_busy.get(&gun).is_some_and(|v| v.iter().any(|&(_, end)| end > sim.clock)) && sim.clock < 0.5 {
            assert!(sim.step());
        }
        run_for(&mut sim, 1.0 / crate::combat::DAYS_PER_YEAR);
        assert!(
            damage(&sim) > before,
            "and the duel resumes after: t={} live={} role={:?} slots={:?} stats={:?}",
            sim.clock,
            sim.live_hull(gun),
            sim.world.role.get(gun),
            sim.pd_busy.get(&gun),
            sim.missile_stats()
        );
    }

    /// **A lost sentry leaves its center's count** (R-WAR46), so the center
    /// may buy another at its price.
    #[test]
    fn a_wrecked_sentry_leaves_its_centers_count() {
        let mut sim = bed(1);
        let s = sentry(&mut sim, 0);
        let h0 = home(&sim, 0);
        assert_eq!(sim.sentries.get(&h0.0).map(|v| v.len()), Some(1));
        let at = pos(&sim, h0);
        let gun = hull(&mut sim, 1, Role::Picket, (HullType::LimitedContactVehicle, Class::Cairn), at, h0);
        sim.deliver(gun, s, 1e15);
        assert!(!sim.live_hull(s), "wrecked");
        assert!(sim.sentries.get(&h0.0).is_none_or(|v| v.is_empty()), "and off its center's count");
        assert_eq!(sim.sentries_lost.get(&h0.0), Some(&1), "and on its center's losses (R-WAR47)");
        assert_eq!(sim.missile_stats().sentries_lost, 1);
    }

    /// **A sentry that withdraws is lost to its center too** (R-WAR47): it
    /// stands down to Reserve and leaves the count, and the loss is priced.
    #[test]
    fn a_withdrawn_sentry_is_a_loss_to_its_center() {
        let mut sim = bed(1);
        let s = sentry(&mut sim, 0);
        let h0 = home(&sim, 0);
        sim.withdraw(s);
        assert_eq!(sim.world.role.get(s), Some(&Role::Reserve), "stood down");
        assert!(sim.sentries.get(&h0.0).is_none_or(|v| v.is_empty()), "off its center's count");
        assert_eq!(sim.sentries_lost.get(&h0.0), Some(&1), "and on its losses");
    }

    /// **A loss raises the price of the next sentry by `κ` of its own**
    /// (R-WAR47): a center that wanted ten at `κ = 1` wants five after one
    /// loss and two after four, and `κ = 0` replaces every loss.
    #[test]
    fn each_lost_sentry_adds_kappa_to_the_price_of_the_next() {
        let one = Price::new(0.03);
        let ten = Kilotons::new(10.01 * one.kilotons() / crate::cards::MISSILE_SENTRY_RATIO);
        let at = |kappa: f64| Doctrine {
            sentry_ratio: crate::cards::MISSILE_SENTRY_RATIO,
            sentry_loss_price: kappa,
            ..Doctrine::default()
        };
        let (free, priced) = (at(0.0), at(1.0));
        for lost in [0, 1, 4, 1000] {
            assert_eq!(Standing::of(&free).sentries_wanted(ten, one, lost), 10, "κ = 0 replaces every loss");
        }
        let wanted: Vec<u32> =
            [0, 1, 4, 9].iter().map(|&l| Standing::of(&priced).sentries_wanted(ten, one, l)).collect();
        assert_eq!(wanted, vec![10, 5, 2, 1]);
    }

    /// **The ordnance book is closed until a Doctrine opens it** (T-139,
    /// matching §10.5): a center whose sentry waits on rounds it cannot make
    /// bids only then, and a center with basics to spare asks only then.
    #[test]
    fn the_ordnance_book_is_closed_until_a_doctrine_opens_it() {
        for open in [false, true] {
            let mut sim = bed(1);
            for p in 0..2 {
                sim.world.doctrine.get_mut(sim.player_entity[p]).unwrap().ordnance_market = open;
            }
            let s = sentry(&mut sim, 0);
            // Spend the magazine and the bank: the center cannot refill it.
            sim.world.cargo.insert(s, Minerals::default());
            let h0 = home(&sim, 0);
            *sim.held_at_mut(h0).unwrap() = Minerals::default();
            // A rival with basics well past its next works bill.
            let h1 = home(&sim, 1);
            sim.held_at_mut(h1).unwrap().add_all(&Minerals {
                cyan: 1e3,
                magenta: 1e3,
                yellow: 1e3,
                ..Minerals::default()
            });
            assert_eq!(sim.arm_at(s, h0), 0.0);
            assert!(sim.ordnance_short.get(&h0.0).is_some_and(|&k| k > 0.0), "the shortfall is recorded");
            sim.post_exchange_offers();
            let book = &sim.exchange.markets[Material::Ordnance as usize];
            let bids = book.bids.iter().filter(|b| b.owner == PlayerId(0)).count();
            let asks = book.asks.iter().filter(|a| a.owner == PlayerId(1)).count();
            assert_eq!((bids > 0, asks > 0), (open, open), "open = {open}");
        }
    }
}
