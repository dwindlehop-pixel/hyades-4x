//! Optional, structured, in-memory event logging — the **diagnostic seam**.
//!
//! Companion to [`crate::snapshot`] (the read-only *presentation* seam): where a
//! [`crate::snapshot::Snapshot`] answers "what does the board look like right
//! now," a [`SimLog`] answers "what happened, and why." It exists to let a
//! human (or the Monte-Carlo harness) **interrogate** a run after the fact —
//! e.g. *"why did this center sit idle for 600 years at `reinvest_bias =
//! 1.0`?"* — without reaching into engine internals.
//!
//! Design constraints carried over from the rest of the crate: **zero external
//! dependencies** (no `log`/`tracing` crates — this is a small bespoke
//! facility) and **WASM-portable** (no wall-clock timestamps; every record is
//! stamped with the deterministic sim-years clock, and records are plain owned
//! `Copy` data the host pulls and prints/renders however it likes — there is no
//! assumption of a stdout).
//!
//! **Off by default, and free when off.** Every category in [`LogFilter`]
//! starts disabled, so a fresh [`SimLog`] records nothing. [`SimLog::push`]
//! checks the filter and drops the record (no allocation — every [`LogEvent`]
//! variant is `Copy`) when its category isn't enabled, so a Monte-Carlo sweep
//! across thousands of seeds pays only a branch per event for a facility it
//! isn't using. Logging is pure side-channel bookkeeping: it cannot perturb the
//! deterministic float arithmetic the rest of the engine depends on (see
//! `sim::tests::logging_does_not_affect_outcomes`).

use core::fmt;

use crate::autopilot::BuildOrder;
use crate::galaxy::PlanetId;
use crate::math::Vec3;
use crate::sim::{Entity, Role};

/// Which subsystem a record came from. Independently toggleable in a
/// [`LogFilter`] so an interrogation can focus on just the part in question
/// (e.g. `Production` alone, to chase a starved-economy question) without
/// paying for — or wading through — the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogCategory {
    /// Deepen/colonize/mine decisions and the funding behind them
    /// (autopilot-doc §§4–6) — `ProductionDecision`, `BuildApplied`.
    Production,
    /// In-ground density extraction, depletion, and freighter cargo transfer
    /// (sim mineral economy) — `MineralsExtracted`, `MiningExhausted`,
    /// `FreighterTransfer`.
    Mining,
    /// Vehicle lifecycle: spawn, arrival, parking, colony-recycling — the
    /// "vehicles persist; post-arrival behavior varies by role" contract.
    Vehicles,
    /// Per-cycle population growth toward `K`.
    Population,
    /// Survey scan reports reaching an empire's knowledge (light-lagged).
    Scanning,
}

impl LogCategory {
    pub const ALL: [LogCategory; 5] = [
        LogCategory::Production,
        LogCategory::Mining,
        LogCategory::Vehicles,
        LogCategory::Population,
        LogCategory::Scanning,
    ];
}

/// Which categories [`SimLog`] is collecting. Defaults to **nothing enabled**
/// — the engine stays silent unless asked to talk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LogFilter {
    production: bool,
    mining: bool,
    vehicles: bool,
    population: bool,
    scanning: bool,
}

impl LogFilter {
    /// Nothing enabled. Equivalent to `LogFilter::default()`; spelled out for
    /// readability at call sites (`sim.set_log_filter(LogFilter::none())`).
    pub fn none() -> Self {
        Self::default()
    }

    /// Every category enabled — full interrogation, highest overhead. Good for
    /// a single re-run of one seed; not meant for a Monte-Carlo sweep.
    pub fn all() -> Self {
        LogFilter { production: true, mining: true, vehicles: true, population: true, scanning: true }
    }

    /// Builder-style: `LogFilter::none().with(LogCategory::Production)`.
    pub fn with(mut self, cat: LogCategory) -> Self {
        self.set(cat, true);
        self
    }

    pub fn set(&mut self, cat: LogCategory, on: bool) {
        match cat {
            LogCategory::Production => self.production = on,
            LogCategory::Mining => self.mining = on,
            LogCategory::Vehicles => self.vehicles = on,
            LogCategory::Population => self.population = on,
            LogCategory::Scanning => self.scanning = on,
        }
    }

    #[inline]
    pub fn enabled(&self, cat: LogCategory) -> bool {
        match cat {
            LogCategory::Production => self.production,
            LogCategory::Mining => self.mining,
            LogCategory::Vehicles => self.vehicles,
            LogCategory::Population => self.population,
            LogCategory::Scanning => self.scanning,
        }
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.production || self.mining || self.vehicles || self.population || self.scanning
    }
}

/// One leg of a freighter's center↔outpost shuttle (sim §economy).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreighterLeg {
    /// Loaded cargo from an outpost's stockpile.
    Loaded,
    /// Deposited cargo into a center's stockpile.
    Deposited,
}

/// A structured fact about one instant of the simulation. Every field is
/// `Copy`; the boundary deliberately mirrors the inputs/outputs of the
/// [`crate::autopilot::Autopilot`] trait rather than reaching into its private
/// scoring — logging observes decisions, it doesn't second-guess them.
#[derive(Clone, Copy, Debug)]
pub enum LogEvent {
    /// A production center weighed deepen/colonize/mine this cycle. Carries the
    /// full [`crate::autopilot::ProductionContext`]-shaped state so "why did it
    /// choose `Idle`" is answerable directly from the record (e.g.
    /// `stockpile < infra_cost` ⇒ saving toward an upgrade it can't afford yet).
    ProductionDecision {
        player: u32,
        center: PlanetId,
        pop_level: u8,
        infra: f64,
        k_potential: f64,
        stockpile: f64,
        infra_cost: f64,
        colonizer_cost: f64,
        mining_pair_cost: f64,
        mineral_pressure: f64,
        candidates_seen: u32,
        chosen: BuildOrder,
    },
    /// A chosen build was funded and applied this cycle.
    BuildApplied { player: u32, center: PlanetId, order: BuildOrder, cost: f64, stockpile_after: f64 },

    /// In-ground density was mined into a stockpile (a center's local take, or
    /// an outpost's periodic [`crate::sim`] mining tick).
    MineralsExtracted { planet: PlanetId, amount: f64, density_after: f64 },
    /// A body's density crossed the floor; mining there has stopped for good.
    MiningExhausted { planet: PlanetId },
    /// A freighter loaded at an outpost or deposited at a center.
    FreighterTransfer { player: u32, vehicle: Entity, leg: FreighterLeg, amount: f64, at: PlanetId },

    /// A vehicle was built and launched toward a target.
    VehicleSpawned { player: u32, vehicle: Entity, role: Role, from: Vec3, to: PlanetId },
    /// A vehicle reached its destination and is holding station / idle there —
    /// the resting state for every non-contact role (autopilot-doc post-arrival
    /// behavior: systems vehicles return, offensive units hold station).
    VehicleParked { player: u32, vehicle: Entity, role: Role, at: PlanetId },
    /// A contact (survey) unit reached a target and either continues scouting
    /// (`next = Some`) or has run out of reachable, unscanned worlds (`next =
    /// None`) and holds. Contact units never despawn; this is the same entity
    /// re-tasked, not a respawn.
    ContactArrived { player: u32, vehicle: Entity, planet: PlanetId, next: Option<PlanetId> },
    /// A colony vehicle founded a colony and was **recycled into its level-1
    /// infrastructure** — it does not return, scrap, or persist as a ship.
    ColonyFounded { player: u32, vehicle: Entity, planet: PlanetId },
    /// A colony vehicle arrived at a target someone else had already claimed
    /// (race under light-lag); it turns back rather than founding.
    ColonyContested { player: u32, vehicle: Entity, planet: PlanetId },
    /// An exhausted Scout reached a friendly colony and scrapped — confirmed
    /// only for this case (`Hyades_vehicle_roles.md` §4.1/§4.6); recovers
    /// some mineral value into the colony it scrapped at.
    VehicleScrapped { player: u32, vehicle: Entity, at: PlanetId, recovered: f64 },

    /// A planet's population advanced one production cycle's logistic step.
    PopulationStep { planet: PlanetId, population: f64, k: f64 },

    /// A scan result reached an empire's knowledge base (light-delayed from the
    /// contact unit's actual arrival at the world).
    ScanReceived { player: u32, planet: PlanetId },
}

impl LogEvent {
    /// The category this variant belongs to (used by `SimLog::push`).
    pub fn category(&self) -> LogCategory {
        use LogEvent::*;
        match self {
            ProductionDecision { .. } | BuildApplied { .. } => LogCategory::Production,
            MineralsExtracted { .. } | MiningExhausted { .. } | FreighterTransfer { .. } => LogCategory::Mining,
            VehicleSpawned { .. }
            | VehicleParked { .. }
            | ContactArrived { .. }
            | ColonyFounded { .. }
            | ColonyContested { .. }
            | VehicleScrapped { .. } => LogCategory::Vehicles,
            PopulationStep { .. } => LogCategory::Population,
            ScanReceived { .. } => LogCategory::Scanning,
        }
    }

    /// The player this record is about, if any (players are 0-indexed seats).
    pub fn player(&self) -> Option<u32> {
        use LogEvent::*;
        match *self {
            ProductionDecision { player, .. }
            | BuildApplied { player, .. }
            | FreighterTransfer { player, .. }
            | VehicleSpawned { player, .. }
            | VehicleParked { player, .. }
            | ContactArrived { player, .. }
            | ColonyFounded { player, .. }
            | ColonyContested { player, .. }
            | VehicleScrapped { player, .. }
            | ScanReceived { player, .. } => Some(player),
            MineralsExtracted { .. } | MiningExhausted { .. } | PopulationStep { .. } => None,
        }
    }

    /// The planet this record is principally about, if any.
    pub fn planet(&self) -> Option<PlanetId> {
        use LogEvent::*;
        match *self {
            ProductionDecision { center, .. } | BuildApplied { center, .. } => Some(center),
            MineralsExtracted { planet, .. }
            | MiningExhausted { planet, .. }
            | PopulationStep { planet, .. }
            | ScanReceived { planet, .. } => Some(planet),
            FreighterTransfer { at, .. } | VehicleParked { at, .. } | VehicleScrapped { at, .. } => Some(at),
            VehicleSpawned { to, .. } => Some(to),
            ContactArrived { planet, .. } => Some(planet),
            ColonyFounded { planet, .. } | ColonyContested { planet, .. } => Some(planet),
        }
    }

    /// The vehicle entity this record is about, if any. Combine with
    /// [`crate::sim::Simulation::position_at`] to recover its exact trajectory.
    pub fn vehicle(&self) -> Option<Entity> {
        use LogEvent::*;
        match *self {
            FreighterTransfer { vehicle, .. }
            | VehicleSpawned { vehicle, .. }
            | VehicleParked { vehicle, .. }
            | ContactArrived { vehicle, .. }
            | ColonyFounded { vehicle, .. }
            | ColonyContested { vehicle, .. }
            | VehicleScrapped { vehicle, .. } => Some(vehicle),
            _ => None,
        }
    }
}

impl fmt::Display for LogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LogEvent::*;
        match self {
            ProductionDecision {
                player,
                center,
                pop_level,
                infra,
                k_potential,
                stockpile,
                infra_cost,
                colonizer_cost,
                mining_pair_cost,
                mineral_pressure,
                candidates_seen,
                chosen,
            } => write!(
                f,
                "P{player} planet#{} production: pop_lvl={pop_level} infra={infra:.2}/{k_potential:.2} \
                 stock={stockpile:.2} (infra_cost={infra_cost:.2} colonizer={colonizer_cost:.2} \
                 mining_pair={mining_pair_cost:.2}) \
                 pressure={mineral_pressure:.2} candidates={candidates_seen} -> {chosen:?}",
                center.0
            ),
            BuildApplied { player, center, order, cost, stockpile_after } => write!(
                f,
                "P{player} planet#{} built {order:?} (cost={cost:.2}, stockpile now {stockpile_after:.2})",
                center.0
            ),
            MineralsExtracted { planet, amount, density_after } => {
                write!(f, "planet#{} mined {amount:.3} (density now {density_after:.3})", planet.0)
            }
            MiningExhausted { planet } => write!(f, "planet#{} mined out", planet.0),
            FreighterTransfer { player, leg, amount, at, .. } => {
                let verb = match leg {
                    FreighterLeg::Loaded => "loaded",
                    FreighterLeg::Deposited => "deposited",
                };
                write!(f, "P{player} freighter {verb} {amount:.2} at planet#{}", at.0)
            }
            VehicleSpawned { player, role, to, .. } => {
                write!(f, "P{player} launched {role:?} -> planet#{}", to.0)
            }
            VehicleParked { player, role, at, .. } => {
                write!(f, "P{player} {role:?} holding station at planet#{}", at.0)
            }
            ContactArrived { player, planet, next, .. } => match next {
                Some(n) => write!(f, "P{player} scout reached planet#{} -> on to planet#{}", planet.0, n.0),
                None => write!(f, "P{player} scout reached planet#{} -> no targets left, holding", planet.0),
            },
            ColonyFounded { player, planet, .. } => {
                write!(f, "P{player} founded colony at planet#{} (vehicle recycled into infra-1)", planet.0)
            }
            ColonyContested { player, planet, .. } => {
                write!(f, "P{player} colony ship found planet#{} already claimed, turning back", planet.0)
            }
            VehicleScrapped { player, at, recovered, .. } => {
                write!(f, "P{player} scrapped a Scout at planet#{} (+{recovered:.2} minerals)", at.0)
            }
            PopulationStep { planet, population, k } => {
                write!(f, "planet#{} population {population:.3} (K={k:.2})", planet.0)
            }
            ScanReceived { player, planet } => write!(f, "P{player} scan of planet#{} received", planet.0),
        }
    }
}

/// One timestamped, categorized fact.
#[derive(Clone, Copy, Debug)]
pub struct LogRecord {
    /// Simulation clock at the moment this was recorded, in years.
    pub time: f64,
    pub category: LogCategory,
    pub event: LogEvent,
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[t={:>8.1}] {}", self.time, self.event)
    }
}

/// The in-memory log: a filter plus the records it has collected. Owned by
/// [`crate::sim::Simulation`]; query it after (or during, by stepping and
/// reading between steps) a run.
#[derive(Clone, Debug, Default)]
pub struct SimLog {
    filter: LogFilter,
    records: Vec<LogRecord>,
}

impl SimLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_filter(filter: LogFilter) -> Self {
        SimLog { filter, records: Vec::new() }
    }

    pub fn filter(&self) -> LogFilter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: LogFilter) {
        self.filter = filter;
    }

    /// Record `event` at `time`, if its category is enabled. No-op (no
    /// allocation; `LogEvent` is `Copy`) otherwise — this is the only thing a
    /// `sys_*` call site needs to call, unconditionally.
    #[inline]
    pub(crate) fn push(&mut self, time: f64, event: LogEvent) {
        let category = event.category();
        if self.filter.enabled(category) {
            self.records.push(LogRecord { time, category, event });
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Drop all collected records (keeps the filter). Useful when reusing one
    /// `Simulation` across repeated interrogation windows.
    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &LogRecord> {
        self.records.iter()
    }

    pub fn by_category(&self, cat: LogCategory) -> impl Iterator<Item = &LogRecord> {
        self.records.iter().filter(move |r| r.category == cat)
    }

    pub fn by_player(&self, player: u32) -> impl Iterator<Item = &LogRecord> {
        self.records.iter().filter(move |r| r.event.player() == Some(player))
    }

    pub fn by_planet(&self, planet: PlanetId) -> impl Iterator<Item = &LogRecord> {
        self.records.iter().filter(move |r| r.event.planet() == Some(planet))
    }

    /// Records with `start <= time < end`.
    pub fn in_range(&self, start: f64, end: f64) -> impl Iterator<Item = &LogRecord> {
        self.records.iter().filter(move |r| r.time >= start && r.time < end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(player: u32) -> LogEvent {
        LogEvent::ScanReceived { player, planet: PlanetId(3) }
    }

    #[test]
    fn disabled_filter_drops_everything() {
        let mut log = SimLog::new(); // LogFilter::none()
        log.push(10.0, sample(0));
        assert!(log.is_empty(), "default filter should record nothing");
    }

    #[test]
    fn enabling_a_category_captures_only_that_category() {
        let mut log = SimLog::with_filter(LogFilter::none().with(LogCategory::Scanning));
        log.push(1.0, sample(0));
        log.push(2.0, LogEvent::PopulationStep { planet: PlanetId(0), population: 1.0, k: 2.0 });
        assert_eq!(log.len(), 1, "Population category was not enabled, should be dropped");
        assert_eq!(log.iter().next().unwrap().category, LogCategory::Scanning);
    }

    #[test]
    fn query_helpers_filter_correctly() {
        let mut log = SimLog::with_filter(LogFilter::all());
        log.push(1.0, sample(0));
        log.push(2.0, sample(1));
        log.push(3.0, LogEvent::MineralsExtracted { planet: PlanetId(5), amount: 1.0, density_after: 0.5 });

        assert_eq!(log.by_player(0).count(), 1);
        assert_eq!(log.by_player(1).count(), 1);
        assert_eq!(log.by_planet(PlanetId(3)).count(), 2); // both scans target planet 3
        assert_eq!(log.by_category(LogCategory::Mining).count(), 1);
        assert_eq!(log.in_range(0.0, 2.5).count(), 2);
    }

    #[test]
    fn all_enables_every_category() {
        let f = LogFilter::all();
        for cat in LogCategory::ALL {
            assert!(f.enabled(cat), "{cat:?} should be enabled by LogFilter::all()");
        }
    }

    #[test]
    fn display_is_human_readable() {
        let rec = LogRecord { time: 123.4, category: LogCategory::Scanning, event: sample(2) };
        let s = rec.to_string();
        assert!(s.contains("123.4"));
        assert!(s.contains("P2"));
        assert!(s.contains("planet#3"));
    }
}
