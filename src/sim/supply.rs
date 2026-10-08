//! **Supers to the yards that want them** (T-146, the author's rulings).
//!
//! A yard whose standing order lacks a super ([`Simulation::refined_short`])
//! is sent one by an idle hauler from Reserve. Where the hauler loads is
//! Doctrine, so a card can change it:
//!
//! | Doctrine | source | terms |
//! |---|---|---|
//! | `forge_supply_runs` | the nearest forge of the yard's own empire with any of what it lacks to spare | free: one empire's holding moves between its planets |
//! | `buy_from_rival_forges` | where no forge of its own can, the nearest forge of another empire holding supers that empire does not need | bought at the supers' floor price, buyer's purse to seller's |
//!
//! A run is two legs, [`SideRun::SuperPickup`] and [`SideRun::SuperDeliver`];
//! the hauler stands down in Reserve at the yard. A run is started when a yard
//! records a want and when a forge of its empire forges — the two events that
//! can make one possible — and a yard has at most one run in flight.
//!
//! What a forge has **to spare** is what its standing order leaves
//! ([`Simulation::available_at`]); what a rival forge **sells** is that, less
//! what its own empire waits on and what it has sold on the Exchange and not
//! delivered ([`Simulation::supers_kept`]) — the same rule as its refined asks.
//! **Read without light-lag**, as the Exchange's books are (T-145, OPEN).
use super::*;

/// What the supply runs have done since the run began.
#[derive(Clone, Copy, Debug, Default)]
pub struct SupplyStats {
    /// Runs started from a forge of the yard's own empire, and from a rival's.
    pub runs_own: u64,
    pub runs_rival: u64,
    /// Kilotonnes of supers and apex loaded at own forges, and bought at rivals'.
    pub loaded_own: f64,
    pub bought_rival: f64,
    /// `$` paid to rival empires for what was bought.
    pub paid: f64,
}

impl Simulation {
    /// What the supply runs have done since the run began.
    pub fn supply_stats(&self) -> SupplyStats {
        self.supply_stats
    }

    /// **Keep the forge index current** — called at every center's economy
    /// tick, where whether it is a forge can have changed.
    pub(super) fn note_forge(&mut self, p: usize, center: Entity) {
        let key = (p as u32, center.0);
        if self.is_forge(center) {
            self.forges.insert(key);
        } else {
            self.forges.remove(&key);
        }
    }

    /// The forges of `seat` that it still owns and that are still forges.
    fn forges_of(&self, seat: u32) -> impl Iterator<Item = Entity> + '_ {
        self.forges
            .range((seat, 0)..=(seat, u64::MAX))
            .map(|&(_, c)| Entity(c))
            .filter(move |&c| self.world.owner.get(c).is_some_and(|o| o.0 == seat) && self.is_forge(c))
    }

    /// What `forge` can give its own empire's yards: its holding of each
    /// refined material that no standing order of its own reserves.
    fn forge_spare(&self, forge: Entity) -> [f64; 4] {
        let avail = self.available_at(forge);
        Material::REFINED.map(|m| avail.get(m).max(0.0))
    }

    /// What `forge`, of empire `seller`, sells to another empire: its spare,
    /// less what its own empire waits on and what it has sold and not yet
    /// delivered.
    fn forge_sells(&self, seller: u32, forge: Entity) -> [f64; 4] {
        let spare = self.forge_spare(forge);
        let kept = self.supers_kept(seller as usize, forge);
        core::array::from_fn(|i| if i < 3 { (spare[i] - kept[i]).max(0.0) } else { 0.0 })
    }

    /// Does `offer` hold any of what `want` lacks?
    fn covers_some(offer: &[f64; 4], want: &[Price; 4]) -> bool {
        (0..4).any(|i| want[i] > Price::ZERO && offer[i] > 1e-9)
    }

    /// **Start a run to `yard` if it wants supers and none is in flight.**
    pub(super) fn seek_supers(&mut self, seat: u32, yard: Entity) {
        if self.standing.is_empty() {
            return;
        }
        let want = self.refined_short(yard);
        if want.iter().all(|w| *w <= Price::ZERO) {
            return;
        }
        if let Some(&h) = self.super_runs.get(&yard.0) {
            if self.live_hull(h) && self.side_runs.contains_key(&h.0) {
                return;
            }
            self.super_runs.remove(&yard.0);
        }
        if self.reserve_freighters[seat as usize].is_empty() {
            return;
        }
        let doctrine = self.doctrine_of(seat as usize);
        let Some(&at) = self.world.position.get(yard) else { return };
        let nearest = |sim: &Simulation, list: Vec<Entity>| {
            list.into_iter()
                .filter(|&c| c != yard)
                .filter_map(|c| sim.world.position.get(c).map(|q| (q.distance(at), c)))
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                .map(|(_, c)| c)
        };
        let mut source = None;
        if doctrine.forge_supply_runs {
            let own: Vec<Entity> =
                self.forges_of(seat).filter(|&f| Self::covers_some(&self.forge_spare(f), &want)).collect();
            source = nearest(self, own).map(|f| (f, false));
        }
        if source.is_none() && doctrine.buy_from_rival_forges {
            let mut rivals: Vec<Entity> = Vec::new();
            for s in (0..self.player_entity.len() as u32).filter(|&s| s != seat) {
                rivals.extend(self.forges_of(s).filter(|&f| Self::covers_some(&self.forge_sells(s, f), &want)));
            }
            source = nearest(self, rivals).map(|f| (f, true));
        }
        let Some((source, rival)) = source else { return };
        let source_at = *self.world.position.get(source).unwrap();
        let Some(h) = self.take_nearest_reserve(seat as usize, false, source_at) else { return };
        self.world.role.insert(h, Role::Freighter);
        self.super_runs.insert(yard.0, h);
        self.side_runs.insert(h.0, SideRun::SuperPickup { source, yard });
        if rival {
            self.supply_stats.runs_rival += 1;
        } else {
            self.supply_stats.runs_own += 1;
        }
        let from = self.position_at(h, self.clock).unwrap_or(source_at);
        let accel = self.laden_accel(h);
        let arrive = self.set_leg(h, from, source_at, accel, 0.0);
        self.schedule_at(arrive, EventKind::DutyArrive { vehicle: h });
    }

    /// **A forge has forged: start runs to its empire's yards that want
    /// supers**, nearest first, while it has any to spare and the empire has
    /// idle haulers.
    pub(super) fn supply_from_forge(&mut self, p: usize, forge: Entity) {
        if self.standing.is_empty() || !self.doctrine_of(p).forge_supply_runs {
            return;
        }
        let Some(&at) = self.world.position.get(forge) else { return };
        let mut yards: Vec<(f64, Entity)> = self.owned_planets[p]
            .iter()
            .filter(|&&c| c != forge && self.standing.contains_key(&c.0))
            .filter_map(|&c| self.world.position.get(c).map(|q| (q.distance(at), c)))
            .collect();
        yards.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (_, yard) in yards {
            if self.reserve_freighters[p].is_empty() || !self.forge_spare(forge).iter().any(|&s| s > 1e-9) {
                return;
            }
            self.seek_supers(p as u32, yard);
        }
    }

    /// **One leg of a supply run ends** (called from `sys_duty_arrive`).
    pub(super) fn super_leg(&mut self, h: Entity, run: SideRun) {
        let Some(seat) = self.world.owner.get(h).map(|o| o.0) else { return };
        match run {
            SideRun::SuperPickup { source, yard } => {
                let hull = self.world.hull_type.get(h).copied().unwrap_or(HullType::MediumSystems);
                let mut room = hull.cargo_capacity(&self.config).kilotons();
                let want = self.refined_short(yard);
                let seller = self.world.owner.get(source).map(|o| o.0);
                let offer = match seller {
                    Some(s) if s == seat => self.forge_spare(source),
                    Some(s) => self.forge_sells(s, source),
                    None => [0.0; 4],
                };
                let mut moved = Minerals::default();
                if let Some(s) = seller {
                    let seller_doctrine = self.doctrine_of(s as usize);
                    let mut purse = if s == seat { f64::INFINITY } else { self.purse_of(PlayerId(seat)) };
                    for (i, &m) in Material::REFINED.iter().enumerate() {
                        let price = if s == seat { 0.0 } else { self.refined_floor(&seller_doctrine, i) };
                        let afford = if price > 0.0 { (purse / price).max(0.0) } else { f64::INFINITY };
                        let q = want[i].kilotons().min(offer[i]).min(room).min(afford);
                        if q <= 1e-12 {
                            continue;
                        }
                        room -= q;
                        moved.add(m, q);
                        self.holding_mut(s, source).add(m, -q);
                        if price > 0.0 {
                            let cost = q * price;
                            purse -= cost;
                            self.credit(self.player_entity[seat as usize], -cost);
                            self.credit(self.player_entity[s as usize], cost);
                            self.supply_stats.paid += cost;
                        }
                    }
                    let total = moved.refined_total().kilotons();
                    if s == seat {
                        self.supply_stats.loaded_own += total;
                    } else {
                        self.supply_stats.bought_rival += total;
                    }
                }
                self.world.cargo.insert(h, moved);
                let pid = *self.world.planet_id.get(source).unwrap();
                self.log.push(
                    self.clock,
                    LogEvent::FreighterTransfer {
                        player: seat,
                        vehicle: h,
                        leg: FreighterLeg::Loaded,
                        amount: moved.total().kilotons(),
                        refined: moved.refined_total().kilotons(),
                        at: pid,
                    },
                );
                let from = *self.world.position.get(source).unwrap();
                let to = *self.world.position.get(yard).unwrap();
                self.side_runs.insert(h.0, SideRun::SuperDeliver { yard });
                let accel = self.laden_accel(h);
                let arrive = self.set_leg(h, from, to, accel, 0.0);
                self.schedule_at(arrive, EventKind::DutyArrive { vehicle: h });
            }
            SideRun::SuperDeliver { yard } => {
                self.super_runs.remove(&yard.0);
                self.deliver_side_cargo(h, PlayerId(seat), yard);
                let at = *self.world.position.get(yard).unwrap();
                let pid = *self.world.planet_id.get(yard).unwrap();
                self.park(h, at);
                self.release_to_reserve(h, Role::Freighter, pid);
            }
            _ => {}
        }
    }
}
