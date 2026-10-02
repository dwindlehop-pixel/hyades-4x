//! **Is there demand for what the Holdings hold?** (T-134.) `holding_demand
//! [horizon] [seed,seed,...]` — at every round barrier, per color and summed
//! over empires: what centers bid for, what yards and away holdings ask, what
//! is held away from a yard, the room the empire's own haulers have left, and
//! what centers bank. Then, per empire, the part of its own away asks that its
//! own bids could absorb (the most a self-trade could move) and the part its
//! own haulers' room could carry (the most the self-route could deliver).
use hyades_engine::prelude::*;
use std::io::Write;

fn main() {
    let h: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(2000.0);
    let seeds: Vec<u64> = match std::env::args().nth(2) {
        Some(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        None => vec![1, 7],
    };
    println!(
        "seed  year  color  bid_kt  ask_yard  ask_away  held_away  haul_room  banked  self_demand  self_room  filled  filled_self"
    );
    for seed in seeds {
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = h;
        let every = cfg.years_per_round;
        let mut next = cfg.years_to_first_round + 1e-6;
        let mut sim = Simulation::with_baseline(Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap(), cfg);
        while sim.step() {
            if sim.clock() < next {
                continue;
            }
            next += every;
            let c = sim.book_census();
            for (i, name) in ["C", "M", "Y"].iter().enumerate() {
                let sum = |v: &Vec<[f64; hyades_engine::sim::MATERIALS]>| v.iter().map(|r| r[i]).sum::<f64>();
                // Per empire: its own away asks against its own bids, and against its haulers' room.
                let own_demand: f64 = (0..c.bid.len()).map(|p| c.ask_away[p][i].min(c.bid[p][i])).sum();
                let own_room: f64 =
                    (0..c.bid.len()).map(|p| c.ask_away[p][i].min(c.bid[p][i]).min(c.haul_room[p][i])).sum();
                println!(
                    "{seed:>4} {:>5.0}  {name}  {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1} {:>10.1}",
                    sim.clock(),
                    sum(&c.bid),
                    sum(&c.ask_yard),
                    sum(&c.ask_away),
                    sum(&c.held_away),
                    sum(&c.haul_room),
                    sum(&c.banked),
                    own_demand,
                    own_room,
                    sum(&c.filled),
                    sum(&c.filled_self)
                );
            }
            let _ = std::io::stdout().flush();
        }
    }
}
