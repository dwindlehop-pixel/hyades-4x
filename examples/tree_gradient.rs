//! **Which knobs move the *game*, not just Expansion** — a gradient probe whose
//! objective is the geometric mean of every tree objective that is defined and
//! measurable today (`Hyades_trees_and_card_value.md` §2.3).
//!
//! ## Why a geomean, and why it is the right shape
//!
//! `examples/gradient_probe` ranks knobs against colony count. §2.1 of the trees
//! doc says that is **correct for Expansion and actively misleading for the
//! other five trees**, and `examples/work_years` has already caught it scoring a
//! development regression as an improvement four times running. But the six
//! objectives are in six different units — colony-years cannot be added to
//! kilotons of fleet-years — so a composite has to be built out of something
//! dimensionless.
//!
//! **Ratio to the shipped default, geometrically averaged.** Each tree's stock
//! integral is divided by its value at the default configuration, so the default
//! scores exactly `1.0` on every tree and therefore `1.0` composite, by
//! construction. Then:
//!
//! ```text
//! S(x) = ( Π_m  M_m(x) / M_m(default) ) ^ (1/|m|)
//! ```
//!
//! Three properties make this the right choice rather than merely a convenient
//! one:
//!
//! - **Units cancel before the average**, so no tree's scale sets its weight.
//!   An arithmetic mean of ratios would still be unit-free but would reward a
//!   +100% on one tree over −50% on another; the geomean treats a halving and a
//!   doubling as equal and opposite, which is what "balanced across trees"
//!   means.
//! - **`ln S` is the *arithmetic* mean of the per-tree log-ratios**, so the
//!   composite elasticity `∂ln S/∂ln x` is exactly the mean of the per-tree
//!   elasticities. The decomposition is free — this harness prints the parts
//!   beside the whole and they reconcile by construction, which is `CLAUDE.md`
//!   §2's mix rule satisfied rather than merely obeyed.
//! - **Everything reported is dimensionless** (% per %), so knobs are comparable
//!   to each other *and* trees are comparable to each other. That is a second
//!   route to §2.4's commensurability requirement, independent of the
//!   doubling-time numeraire and much cheaper to read.
//!
//! ## What is actually measurable, and what is not
//!
//! Four of the six are not honestly available on this bed and saying which is
//! more useful than printing six numbers:
//!
//! | tree | §2.3 | status here |
//! |---|---|---|
//! | Expansion | `∫ C dt` | **measured** |
//! | Growth | `∫ V dt` | **measured** — works exist since T-73/T-75 |
//! | Production | `∫ F dt`, in **mass** | **measured** — needs `VehicleSnapshot::dry_mass` |
//! | Warfare | `∫ [C_i − Σ w_ij C_j] dt` | **computed and excluded** — see below |
//! | Politics | `∫ [C_i + κ Σ φ_ij C_j] dt` | **degenerate** — `φ_ij ≡ 0` |
//! | Technology | `∫ Q dt` | **undefined** — R-TREE4 |
//!
//! **Warfare sums to exactly zero on the 3-seat bed, and the reason is the
//! galaxy generator, not the autopilot.** The obvious explanation — every seat
//! runs the same policy, so a knob helps all three equally and relative standing
//! does not move — predicts a small *noisy* number. What it reports is
//! `-0.0000 ± 0.0000` on all four seeds, which is a different claim, so it was
//! measured: at 3 seats the homeworlds sit on a hex ring at pairwise distance
//! **77.942 on every seed, spread 0.000%** — an equilateral triangle. So every
//! `w_ij = 1/2`, the weight matrix is doubly stochastic, and
//!
//! ```text
//! Σ_i W_i = Σ_i C_i − Σ_j C_j · (Σ_{i≠j} w_ij) = Σ_i C_i − Σ_j C_j = 0
//! ```
//!
//! **identically** — for any colony distribution, at any horizon, under any
//! configuration. It is not degenerate *on this bed*; it is an algebraic zero
//! that no amount of seeds or horizon can rescue.
//!
//! Two consequences worth carrying. **Wider tables are not symmetric**: the same
//! measurement gives spread 100.0% at 6 seats and 286.4% at 12, so `w_ij` is
//! non-uniform there and the sum is no longer identically zero — though it is
//! still a contrast between near-identical players and will be dominated by
//! noise. And **per-seat Warfare is what has content**, which needs an
//! *asymmetric* bed — one seat perturbed, the rest at the default. That is a
//! different experiment and is filed rather than smuggled in here.
//!
//! **Politics is exactly Expansion here.** `φ_ij` is defined (§2.3.6) as the
//! share of `j`'s exported output that physically reached `i`, and the engine
//! has no inter-player freight accounting, so `φ ≡ 0` and `Pol_i ≡ E_i` to the
//! last bit. Including it would silently double Expansion's weight in the
//! geomean, so it is printed and excluded.
//!
//! **Technology has no definition to implement.** §2.3.5's power mean needs `ρ`,
//! per-axis weights and reference values, all flagged placeholders (R-TREE4),
//! and two of its three axes need combat reach the engine does not model.
//!
//! ## Method
//!
//! Unchanged from `gradient_probe` and stated so the two are comparable: common
//! random numbers, paired central differences at ±10%, a standard error on every
//! number, and nothing inside 2 SE treated as a finding. The one correction is
//! that the log-space step is `ln((1+δ)/(1−δ))` rather than `2δ` — the same
//! thing to `O(δ²)`, but there is no reason to carry a 0.35% bias when the exact
//! value is one call.
//!
//! **The horizon is a screen, not the bed.** §3.1 ratifies 3,000 years; this
//! defaults to 1,500 because cost is violently superlinear in duration and a
//! broad probe is `2 × knobs × seeds` runs. Colony count is 97.9% saturated at
//! 1,500 yr while works and fleet mass are still compounding, so **Expansion is
//! the tree most compressed by the truncation** and the composite is, if
//! anything, biased against expansion-side knobs. Trust the ranking; ratify a
//! magnitude on the full bed (`CLAUDE.md` §2).
//!
//! ## It records the raw evaluations, not the summary (T-50)
//!
//! Every gradient this project has run has ended up as a sentence in a doc and
//! then been invalidated by the next step, leaving nothing behind. `TG_RECORD`
//! appends one TSV row **per (knob, arm, seed)** carrying all five tree
//! integrals and the operating point that produced them. Elasticities, standard
//! errors and rankings are all recoverable from that; the reverse is not true,
//! and it lets a later reader re-analyse under a *different* objective without
//! re-running — which matters, because this project has changed its objective
//! twice already.
//!
//! Run: `cargo run --release --example tree_gradient`
//! Env: `TG_SEEDS=1,7,42` · `TG_HORIZON=1500` · `TG_FROM=0` `TG_TO=12`
//!      · `TG_RECORD=data/tree_gradient.tsv`
//!      (`TG_FROM`/`TG_TO` slice the knob list so a long probe can be run in
//!      chunks and a killed container costs one chunk, not the whole sweep.)
use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::prelude::*;
use std::io::Write;

/// **Raw per-evaluation record** (T-50). One row per `(knob, arm, seed)`, with
/// the operating point beside the numbers — *an elasticity without its operating
/// point is not a measurement, it is a rumour.*
///
/// Appended, never overwritten, so chunked runs accumulate into one dataset and
/// a killed container loses nothing already written. Zero dependencies and
/// diffable, by design: this is input to card costing, not a report.
struct Recorder {
    out: Option<std::fs::File>,
}

impl Recorder {
    fn open() -> Self {
        let Ok(path) = std::env::var("TG_RECORD") else { return Self { out: None } };
        if let Some(dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(dir).ok();
        }
        let fresh = !std::path::Path::new(&path).exists();
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok();
        if fresh {
            if let Some(f) = f.as_mut() {
                writeln!(
                    f,
                    "knob\tarm\tvalue\tseed\thorizon\tplayers\texpansion\tgrowth\tproduction\twarfare\tpolitics"
                )
                .ok();
            }
        }
        Self { out: f }
    }

    fn row(&mut self, knob: &str, arm: &str, value: f64, seed: u64, horizon: f64, t: &Trees) {
        if let Some(f) = self.out.as_mut() {
            writeln!(
                f,
                "{knob}\t{arm}\t{value:.10}\t{seed}\t{horizon:.0}\t{PLAYERS}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}",
                t[0], t[1], t[2], t[3], t[4]
            )
            .ok();
        }
    }
}

const PLAYERS: usize = 3;
const DEFAULT_SEEDS: [u64; 4] = [1, 7, 42, 31337];
const DEFAULT_HORIZON: f64 = 1500.0;

/// Relative step for the central difference — the same ±10% `gradient_probe`
/// uses, so the two tables can be read against each other.
const DELTA: f64 = 0.10;

/// One sample per production cycle. Finer buys nothing: works and hulls both
/// change on build events and `cycle_years = 50` is the coarsest thing driving
/// them (`examples/work_years` uses the same grid).
const SAMPLE_YEARS: f64 = 50.0;

/// Politics' coupling constant (§2.3.6). **Placeholder, R-TREE5** — and inert
/// here, because the term it multiplies is identically zero.
const KAPPA: f64 = 0.5;

/// The tree objectives this harness computes, in the order they are printed.
/// `MEASURED` is the subset the composite is built from; the rest are reported
/// for their own sake and excluded for the reasons in the module docs.
const TREES: [&str; 5] = ["Expansion", "Growth", "Production", "Warfare", "Politics"];
const MEASURED: [usize; 3] = [0, 1, 2];

/// One run's five tree integrals, summed over seats.
///
/// **Summed, not averaged over seats** — the bed is symmetric so the sum is the
/// natural summary and it is what every prior measurement in this project used.
/// It also makes Warfare's degeneracy explicit rather than hiding it: at 3 seats
/// `Σ_i W_i` is an algebraic zero (see the module docs), and printing it is how
/// that was found rather than assumed.
type Trees = [f64; 5];

fn run(seed: u64, horizon: f64, cfg: SimConfig, doctrine: Doctrine) -> Trees {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let mut cfg = cfg;
    cfg.horizon_years = horizon;
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);

    // **`w_ij` is frozen at setup, from homeworld positions** (§2.3.2). A live
    // distance would be a term the Warfare player can move without fighting —
    // expand away from a strong rival and their weight falls, scoring the same
    // as having beaten them. This reads the snapshot once, before the clock
    // moves.
    let homes: Vec<Vec3> = {
        let snap = sim.snapshot();
        (0..PLAYERS)
            .map(|p| {
                snap.planets
                    .iter()
                    .find(|pl| pl.is_homeworld && pl.owner == Some(p as u32))
                    .map(|pl| pl.position)
                    .unwrap_or_default()
            })
            .collect()
    };
    let w = neighbour_weights(&homes);

    let mut acc = [0.0f64; 5];
    let mut prev = [0.0f64; 5];
    let mut prev_t = 0.0f64;
    let mut next = SAMPLE_YEARS;
    let mut done = false;
    while !done && next <= horizon {
        while sim.clock() < next {
            if !sim.step() {
                done = true;
                break;
            }
        }
        let snap = sim.snapshot();
        let mut colonies = [0.0f64; PLAYERS];
        let mut works = [0.0f64; PLAYERS];
        let mut fleet = [0.0f64; PLAYERS];
        for pl in snap.planets.iter() {
            if let Some(o) = pl.owner {
                colonies[o as usize] += 1.0;
                works[o as usize] += pl.works.kilotons();
            }
        }
        // **Mass, never hull count** (§2.3.4). Counting hulls rewards
        // fragmentation and contradicts design law #3 outright.
        for v in snap.vehicles.iter() {
            fleet[v.owner as usize] += v.dry_mass.kilotons();
        }

        let mut now = [0.0f64; 5];
        for i in 0..PLAYERS {
            let rivals: f64 = (0..PLAYERS).filter(|&j| j != i).map(|j| w[i][j] * colonies[j]).sum();
            // `phi_ij` is delivered inter-player freight and the engine does not
            // account for it, so this term is identically zero and Politics
            // reduces to Expansion. Written out rather than elided so the
            // degeneracy is legible at the call site.
            let allies: f64 = 0.0;
            now[0] += colonies[i];
            now[1] += works[i];
            now[2] += fleet[i];
            now[3] += colonies[i] - rivals;
            now[4] += colonies[i] + KAPPA * allies;
        }

        // Trapezoid, matching `examples/work_years` so the numbers compose.
        let t = sim.clock().min(next);
        let dt = t - prev_t;
        for m in 0..5 {
            acc[m] += 0.5 * (now[m] + prev[m]) * dt;
        }
        prev = now;
        prev_t = t;
        next += SAMPLE_YEARS;
    }
    acc
}

/// `w_ij ∝ 1 / (1 + d_ij / λ_w)`, normalised so `Σ_{j≠i} w_ij = 1` (§2.3.2).
///
/// **`λ_w` is R-TREE2 and unset**, so this uses the bed's own median pairwise
/// homeworld distance as the length scale. That is a harness choice, not a
/// ratification, and it is deliberately scale-free: it cannot import a galaxy
/// size as a hidden constant, and it makes the weights comparable across seeds
/// whose galaxies differ in extent.
fn neighbour_weights(homes: &[Vec3]) -> Vec<Vec<f64>> {
    let n = homes.len();
    let mut pair: Vec<f64> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            pair.push(homes[i].distance(homes[j]));
        }
    }
    pair.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lambda_w = pair.get(pair.len() / 2).copied().unwrap_or(1.0).max(1e-9);

    (0..n)
        .map(|i| {
            let raw: Vec<f64> = (0..n)
                .map(|j| if i == j { 0.0 } else { 1.0 / (1.0 + homes[i].distance(homes[j]) / lambda_w) })
                .collect();
            let total: f64 = raw.iter().sum::<f64>().max(1e-12);
            raw.into_iter().map(|x| x / total).collect()
        })
        .collect()
}

struct Knob {
    name: &'static str,
    value: f64,
    set: fn(&mut SimConfig, &mut Doctrine, f64),
}

struct Finding {
    name: &'static str,
    value: f64,
    /// `∂ln S / ∂ln x` — the composite, dimensionless.
    elasticity: f64,
    se: f64,
    /// Per-tree `∂ln M_m / ∂ln x`, same units, and the composite is the mean of
    /// the entries `MEASURED` selects.
    per_tree: [f64; 5],
    /// Composite score of the two probe points, default = 1.0.
    s_hi: f64,
    s_lo: f64,
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn stderr(v: &[f64]) -> f64 {
    let n = v.len() as f64;
    if n < 2.0 {
        return f64::INFINITY;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

fn env_f64(k: &str, d: f64) -> f64 {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn knobs(c: &SimConfig, d: &Doctrine) -> Vec<Knob> {
    vec![
        // --- the economy's rates -------------------------------------------
        Knob { name: "growth_rate", value: d.growth_rate, set: |_, d, v| d.growth_rate = v },
        Knob { name: "biosphere_regen_rate", value: c.biosphere_regen_rate, set: |c, _, v| c.biosphere_regen_rate = v },
        Knob {
            name: "outpost_mining_fraction",
            value: c.outpost_mining_fraction,
            set: |c, _, v| c.outpost_mining_fraction = v,
        },
        Knob {
            name: "center_mining_fraction",
            value: c.center_mining_fraction,
            set: |c, _, v| c.center_mining_fraction = v,
        },
        Knob { name: "veins_per_band", value: c.veins_per_band, set: |c, _, v| c.veins_per_band = v },
        Knob { name: "crowding_beta", value: c.crowding_beta, set: |c, _, v| c.crowding_beta = v },
        Knob { name: "density_floor", value: c.density_floor, set: |c, _, v| c.density_floor = v },
        // --- industry ------------------------------------------------------
        Knob { name: "fab_cap", value: c.fab_cap, set: |c, _, v| c.fab_cap = v },
        Knob { name: "productivity_step", value: d.productivity_step, set: |_, d, v| d.productivity_step = v },
        Knob { name: "build_lead_years", value: c.build_lead_years, set: |c, _, v| c.build_lead_years = v },
        Knob { name: "cycle_years", value: c.cycle_years, set: |c, _, v| c.cycle_years = v },
        Knob { name: "mining_tick_years", value: c.mining_tick_years, set: |c, _, v| c.mining_tick_years = v },
        // --- hulls and logistics -------------------------------------------
        Knob { name: "cargo_unit_size", value: c.cargo_unit_size, set: |c, _, v| c.cargo_unit_size = v },
        Knob { name: "medium_fleet_size", value: c.medium_fleet_size, set: |c, _, v| c.medium_fleet_size = v },
        Knob { name: "limited_fleet_size", value: c.limited_fleet_size, set: |c, _, v| c.limited_fleet_size = v },
        Knob { name: "general_vehicle_cost", value: c.general_vehicle_cost, set: |c, _, v| c.general_vehicle_cost = v },
        Knob { name: "civilian_accel_g", value: c.civilian_accel_g, set: |c, _, v| c.civilian_accel_g = v },
        Knob { name: "trade_decay_lambda", value: c.trade_decay_lambda, set: |c, _, v| c.trade_decay_lambda = v },
        Knob {
            name: "scrap_recovery_fraction",
            value: c.scrap_recovery_fraction,
            set: |c, _, v| c.scrap_recovery_fraction = v,
        },
        Knob {
            name: "homeworld_start_minerals",
            value: c.homeworld_start_minerals,
            set: |c, _, v| c.homeworld_start_minerals = v,
        },
        // --- policy ----------------------------------------------------------
        Knob { name: "reinvest_bias", value: d.reinvest_bias, set: |_, d, v| d.reinvest_bias = v },
        Knob { name: "risk_aversion", value: d.risk_aversion, set: |_, d, v| d.risk_aversion = v },
        Knob { name: "rank.w_k", value: d.rank.w_k, set: |_, d, v| d.rank.w_k = v },
        Knob { name: "rank.w_mineral", value: d.rank.w_mineral, set: |_, d, v| d.rank.w_mineral = v },
        Knob { name: "rank.w_hub", value: d.rank.w_hub, set: |_, d, v| d.rank.w_hub = v },
        Knob { name: "rank.mineral_high", value: d.rank.mineral_high, set: |_, d, v| d.rank.mineral_high = v },
        Knob {
            name: "rank.centrality_scale",
            value: d.rank.centrality_scale,
            set: |_, d, v| d.rank.centrality_scale = v,
        },
        Knob {
            name: "rank.mineral_pressure_gain",
            value: d.rank.mineral_pressure_gain,
            set: |_, d, v| d.rank.mineral_pressure_gain = v,
        },
        // --- survey ------------------------------------------------------------
        Knob { name: "survey_accel_g", value: d.survey_accel_g, set: |_, d, v| d.survey_accel_g = v },
        Knob {
            name: "survey_vehicles",
            value: d.survey_vehicles as f64,
            set: |_, d, v| d.survey_vehicles = v.round().max(1.0) as usize,
        },
        Knob {
            name: "survey_reserve",
            value: d.survey_reserve as f64,
            set: |_, d, v| d.survey_reserve = v.round().max(0.0) as usize,
        },
        Knob {
            name: "max_survey_hops",
            value: c.max_survey_hops as f64,
            set: |c, _, v| c.max_survey_hops = v.round().max(1.0) as usize,
        },
    ]
}

fn main() {
    let horizon = env_f64("TG_HORIZON", DEFAULT_HORIZON);
    let seeds: Vec<u64> = match std::env::var("TG_SEEDS") {
        Ok(v) => v.split(',').filter_map(|s| s.trim().parse().ok()).collect(),
        Err(_) => DEFAULT_SEEDS.to_vec(),
    };
    let base_cfg = SimConfig::new(0);
    let base_doc = Doctrine::default();
    let all = knobs(&base_cfg, &base_doc);
    let from = env_f64("TG_FROM", 0.0) as usize;
    let to = (env_f64("TG_TO", all.len() as f64) as usize).min(all.len());

    println!(
        "tree gradient — geomean over {} measurable tree objectives, {PLAYERS} seats, {:.0} yr",
        MEASURED.len(),
        horizon
    );
    println!("seeds {seeds:?} (common random numbers), central difference ±{:.0}%", DELTA * 100.0);
    println!("knobs [{from}..{to}) of {}; budget {} runs\n", all.len(), 2 * (to - from) * seeds.len());
    std::io::stdout().flush().ok();

    // The base profile: one run per seed, and the denominator every ratio is
    // taken against. It also fixes the default at exactly 1.000 composite.
    let mut rec = Recorder::open();
    let base: Vec<Trees> = seeds.iter().map(|&s| run(s, horizon, base_cfg, base_doc)).collect();
    for (i, &s) in seeds.iter().enumerate() {
        rec.row("(default)", "base", f64::NAN, s, horizon, &base[i]);
    }
    println!("{:>26} {:>16} {:>16}", "default (per seed sum)", "mean", "SE");
    for (m, name) in TREES.iter().enumerate() {
        let v: Vec<f64> = base.iter().map(|b| b[m]).collect();
        let tag = if MEASURED.contains(&m) { "" } else { "   (excluded)" };
        println!("{name:>26} {:>16.4} {:>16.4}{tag}", mean(&v), stderr(&v));
    }
    println!("\n  Warfare: at 3 seats the homeworlds are equilateral (77.942 ly, spread 0.000%),");
    println!("  so every w_ij = 1/2, the weight matrix is doubly stochastic, and the seat sum is");
    println!("  an ALGEBRAIC zero — not a noisy one. Politics is Expansion exactly, since");
    println!("  inter-player delivered freight is not accounted.");
    println!(
        "  Politics/Expansion = {:.9}\n",
        mean(&base.iter().map(|b| b[4]).collect::<Vec<_>>()) / mean(&base.iter().map(|b| b[0]).collect::<Vec<_>>())
    );
    std::io::stdout().flush().ok();

    // Exact log-space separation between x(1+δ) and x(1−δ). `2δ` is the same to
    // O(δ²) and there is no reason to carry the bias.
    let log_step = ((1.0 + DELTA) / (1.0 - DELTA)).ln();

    println!(
        "{:<26} {:>10}  {:>9} {:>7}   {:>7} {:>7} {:>7}   {:>7} {:>7}",
        "knob", "value", "d lnS/d lnx", "SE", "Expan", "Growth", "Produc", "S(+10%)", "S(-10%)"
    );
    let mut findings: Vec<Finding> = Vec::new();
    for k in all.iter().take(to).skip(from) {
        let mut hi_cfg = base_cfg;
        let mut hi_doc = base_doc;
        (k.set)(&mut hi_cfg, &mut hi_doc, k.value * (1.0 + DELTA));
        let mut lo_cfg = base_cfg;
        let mut lo_doc = base_doc;
        (k.set)(&mut lo_cfg, &mut lo_doc, k.value * (1.0 - DELTA));
        if hi_cfg.hull_ladder_fault().is_some() || lo_cfg.hull_ladder_fault().is_some() {
            println!("{:<26} skipped — step leaves the legal hull ladder", k.name);
            std::io::stdout().flush().ok();
            continue;
        }

        let hi: Vec<Trees> = seeds.iter().map(|&s| run(s, horizon, hi_cfg, hi_doc)).collect();
        let lo: Vec<Trees> = seeds.iter().map(|&s| run(s, horizon, lo_cfg, lo_doc)).collect();
        for (i, &s) in seeds.iter().enumerate() {
            rec.row(k.name, "hi", k.value * (1.0 + DELTA), s, horizon, &hi[i]);
            rec.row(k.name, "lo", k.value * (1.0 - DELTA), s, horizon, &lo[i]);
        }

        // Per-tree log-ratio against the *same seed's* base — CRN, and the
        // reason the composite's error bar is far tighter than either level's.
        let logr = |arm: &Vec<Trees>, m: usize| -> Vec<f64> {
            arm.iter()
                .zip(base.iter())
                .map(|(a, b)| if a[m] > 0.0 && b[m] > 0.0 { (a[m] / b[m]).ln() } else { 0.0 })
                .collect()
        };

        // `ln S` is the arithmetic mean of the measured trees' log-ratios, so
        // the composite is assembled from exactly the numbers printed beside it.
        let composite = |arm: &Vec<Trees>| -> Vec<f64> {
            let parts: Vec<Vec<f64>> = MEASURED.iter().map(|&m| logr(arm, m)).collect();
            (0..seeds.len()).map(|s| parts.iter().map(|p| p[s]).sum::<f64>() / MEASURED.len() as f64).collect()
        };
        let ln_hi = composite(&hi);
        let ln_lo = composite(&lo);
        let diffs: Vec<f64> = ln_hi.iter().zip(ln_lo.iter()).map(|(a, b)| (a - b) / log_step).collect();

        let mut per_tree = [0.0f64; 5];
        for (m, slot) in per_tree.iter_mut().enumerate() {
            let (a, b) = (logr(&hi, m), logr(&lo, m));
            *slot = mean(&a.iter().zip(b.iter()).map(|(x, y)| (x - y) / log_step).collect::<Vec<_>>());
        }

        let f = Finding {
            name: k.name,
            value: k.value,
            elasticity: mean(&diffs),
            se: stderr(&diffs),
            per_tree,
            s_hi: mean(&ln_hi).exp(),
            s_lo: mean(&ln_lo).exp(),
        };
        println!(
            "{:<26} {:>10.4}  {:>+9.4} {:>7.4}   {:>+7.3} {:>+7.3} {:>+7.3}   {:>7.4} {:>7.4}",
            f.name, f.value, f.elasticity, f.se, f.per_tree[0], f.per_tree[1], f.per_tree[2], f.s_hi, f.s_lo
        );
        std::io::stdout().flush().ok();
        findings.push(f);
    }

    findings.sort_by(|a, b| b.elasticity.abs().partial_cmp(&a.elasticity.abs()).unwrap());
    println!("\n=== ranked by |composite elasticity| ===");
    println!("{:<26} {:>12} {:>9}  {:<10}  which trees disagree", "knob", "d lnS/d lnx", "SE", "verdict");
    for f in &findings {
        let verdict = if f.se.is_finite() && f.elasticity.abs() > 2.0 * f.se {
            "FINDING"
        } else if f.elasticity.abs() < 1e-4 {
            "flat"
        } else {
            "~noise"
        };
        // A knob that helps one tree and hurts another is the interesting case
        // and the whole reason for the composite; a single mean would hide it.
        let signs: Vec<f64> = MEASURED.iter().map(|&m| f.per_tree[m]).collect();
        let split = signs.iter().any(|&x| x > 1e-3) && signs.iter().any(|&x| x < -1e-3);
        let note = if split {
            let names: Vec<String> = MEASURED
                .iter()
                .map(|&m| format!("{}{:+.2}", TREES[m].chars().next().unwrap(), f.per_tree[m]))
                .collect();
            format!("SPLIT  {}", names.join(" "))
        } else {
            String::new()
        };
        println!("{:<26} {:>+12.4} {:>9.4}  {:<10}  {}", f.name, f.elasticity, f.se, verdict, note);
    }
}
