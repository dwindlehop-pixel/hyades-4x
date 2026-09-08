//! **Where the throughput goes — hauling, not mining (T-62).**
//!
//! T-62 made the mineral field a Gaussian over *Bands*, so it is log-normal in
//! kilotons. Colony-years barely moved (−0.9%) but wall time **doubled**, and
//! the obvious story — "there are more vehicles now" — is wrong by a factor of
//! five. This counts the engine's events by kind so the cost is attributed
//! rather than guessed at, which is `CLAUDE.md` §2's rule about mechanisms
//! applied to performance.
//!
//! Seed 1, 3 seats, shipped defaults, measured across the T-62 landing:
//!
//! | | before | after | ratio |
//! |---|---|---|---|
//! | events | 540,787 | 870,083 | 1.61× |
//! | vehicles | 19,406 | 23,227 | 1.20× |
//! | extraction ticks | 210,620 | 213,823 | **1.02×** |
//! | **freighter transfers** | **20,968** | **142,729** | **6.81×** |
//! | ore hauled (kt) | 8,200 | 18,502,131 | **2,256×** |
//! | wall | 51.6 s | 112.2 s | 2.17× |
//!
//! The rock is worked the same number of times; each working now yields orders
//! of magnitude more ore, and a freighter's hold is a fixed size, so the trips
//! multiply. **The cost is proportional to ore hauled, not to worlds mined** —
//! and none of that ore bought a colony, which is the part worth fixing.
//!
//! Run: `cargo run --release --example haul_census`
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::prelude::*;

const SEED: u64 = 1;
const PLAYERS: usize = 3;

fn main() {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, SEED)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(Doctrine::default())) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, SimConfig::new(SEED), autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Mining).with(LogCategory::Vehicles));

    let t = std::time::Instant::now();
    sim.run();
    let secs = t.elapsed().as_secs_f64();

    let (mut transfers, mut extracts, mut exhausted, mut spawned) = (0usize, 0usize, 0usize, 0usize);
    let mut ore = 0.0;
    for r in sim.log().iter() {
        match r.event {
            LogEvent::FreighterTransfer { .. } => transfers += 1,
            LogEvent::MineralsExtracted { amount, .. } => {
                extracts += 1;
                ore += amount;
            }
            LogEvent::MiningExhausted { .. } => exhausted += 1,
            LogEvent::VehicleSpawned { .. } => spawned += 1,
            _ => {}
        }
    }
    let snap = sim.snapshot();

    println!("haul census — seed {SEED}, {PLAYERS} seats, shipped defaults");
    println!("  events processed     {:>12}", sim.events_processed());
    println!("  vehicles alive       {:>12}", snap.vehicles.len());
    println!("  vehicles spawned     {:>12}", spawned);
    println!("  extraction ticks     {:>12}", extracts);
    println!("  outposts exhausted   {:>12}", exhausted);
    println!("  freighter transfers  {:>12}", transfers);
    println!("  ore hauled (kt)      {:>12.0}", ore);
    println!("  wall                 {secs:>12.1}s  ({:.0} yr/s)", 4000.0 / secs);
}
