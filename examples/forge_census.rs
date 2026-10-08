//! **What forges make, and what crosses between empires** — the census for the
//! author's ruling that forging is a high-priced activity and a forge's primary
//! purpose (galaxy §4.5).
//!
//! Card-free, standard galaxy, 3 seats. Per seat: when its first synthesis
//! ran, the kilotonnes of each super and of apex it made, the share of its
//! super mass that is its archetype's native super, and its colony-years and
//! work-years (`∫ colonies dt`, `∫ infra dt`, sampled every economy tick).
//! Per run: kilotonnes the Exchange delivered between empires, per material
//! (the clearing never pairs an empire with itself, so every delivered
//! kilotonne crossed).
//!
//! Per run, also the worlds whose ceiling `min(hab, bio)` reaches `k_high`
//! (`Band 3.2`, the colonization classifier), which the color field moves
//! through §4.4's anticorrelation.
//!
//! Run: `cargo run --release --example forge_census -- [horizon]`;
//! `FC_SEEDS=2,3,5,11` for another seed set; `FC_SITE_SPACING` and
//! `FC_SITE_SIGMA` (ly) set the color sites' spacing and width (§4.3) — the
//! galaxy is the only thing a bed varies.
use hyades_engine::autopilot::RankWeights;
use hyades_engine::galaxy::{Galaxy, GalaxyConfig};
use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
use hyades_engine::resources::{Material, Super};
use hyades_engine::sim::{SimConfig, Simulation};
use std::io::Write;

const SEEDS: [u64; 4] = [1, 7, 42, 31337];
const SEATS: usize = 3;
const DEFAULT_HORIZON: f64 = 1500.0;
/// One sample per economy tick (`cycle_years = 5`).
const SAMPLE_YEARS: f64 = 5.0;

fn seeds() -> Vec<u64> {
    match std::env::var("FC_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => SEEDS.to_vec(),
    }
}

fn env_f64(name: &str) -> Option<f64> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn galaxy_config(seed: u64) -> GalaxyConfig {
    let mut cfg = GalaxyConfig::new(SEATS, seed);
    if let Some(v) = env_f64("FC_SITE_SPACING") {
        cfg.color_site_spacing_ly = v;
    }
    if let Some(v) = env_f64("FC_SITE_SIGMA") {
        cfg.color_site_sigma_ly = v;
    }
    cfg
}

fn main() {
    let horizon: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_HORIZON);
    let probe = galaxy_config(1);
    println!(
        "forge_census: {SEATS} seats, horizon {horizon} yr, seeds {:?}, color sites {} ly apart, width {} ly",
        seeds(),
        probe.color_site_spacing_ly,
        probe.color_site_sigma_ly
    );
    println!("seed seat native first_yr     red   green    blue    apex native% colony_yr work_yr");
    std::io::stdout().flush().ok();
    for seed in seeds() {
        let galaxy = Galaxy::generate(galaxy_config(seed)).unwrap();
        let k_high = RankWeights::default().k_high;
        let admitted = galaxy.planets.iter().filter(|p| !p.is_homeworld && p.k_potential() >= k_high).count();
        let native: Vec<Super> = galaxy
            .homeworlds
            .iter()
            .map(|h| galaxy.planets[h.0 as usize].archetype.expect("a homeworld has an archetype").native_super())
            .collect();
        let mut cfg = SimConfig::new(seed);
        cfg.horizon_years = horizon;
        let mut sim = Simulation::with_baseline(galaxy, cfg);
        sim.set_log_filter(LogFilter::none().with(LogCategory::Production));
        let t0 = std::time::Instant::now();
        let (mut cy, mut wy) = ([0.0f64; SEATS], [0.0f64; SEATS]);
        let mut next = SAMPLE_YEARS;
        while sim.step() {
            while sim.clock() >= next && next <= horizon {
                for p in 0..SEATS {
                    let (colonies, infra) = sim.tree_stock(p);
                    cy[p] += colonies as f64 * SAMPLE_YEARS;
                    wy[p] += infra * SAMPLE_YEARS;
                }
                next += SAMPLE_YEARS;
            }
        }
        let secs = t0.elapsed().as_secs_f64();
        // Per seat: Red, Green, Blue, apex made, and the first synthesis.
        let mut made = [[0.0f64; 4]; SEATS];
        let mut first = [f64::NAN; SEATS];
        for r in sim.log().iter() {
            if let LogEvent::Synthesized { player, material, made: m, .. } = r.event {
                let p = player as usize;
                if first[p].is_nan() {
                    first[p] = r.time;
                }
                if let Some(i) = Material::REFINED.iter().position(|&x| x == material) {
                    made[p][i] += m;
                }
            }
        }
        for p in 0..SEATS {
            let supers: f64 = made[p][..3].iter().sum();
            let ni = Super::ALL.iter().position(|&s| s == native[p]).unwrap();
            let share = if supers > 0.0 { 100.0 * made[p][ni] / supers } else { f64::NAN };
            println!(
                "{seed:>4} {p:>4} {:>6} {:>8.0} {:>7.2} {:>7.2} {:>7.2} {:>7.2} {:>7.1} {:>9.0} {:>7.0}",
                format!("{:?}", native[p]),
                first[p],
                made[p][0],
                made[p][1],
                made[p][2],
                made[p][3],
                share,
                cy[p],
                wy[p],
            );
        }
        let traded = sim.exchange_traded();
        let names: Vec<String> =
            Material::ALL.iter().zip(traded.iter()).map(|(m, t)| format!("{m:?} {t:.2}")).collect();
        println!(
            "{seed:>4} traded between empires (kt): {} | admitted {admitted} | {} events, {:.1} s",
            names.join(", "),
            sim.events_processed(),
            secs
        );
        std::io::stdout().flush().ok();
    }
}
