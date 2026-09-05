//! **The broken-tradeoff loop: audit → isolate → candidate fix.**
//!
//! CLAUDE.md §2 records six measurement artifacts with one shape between them —
//! *a plausible number produced by a quantity the shipped configuration never
//! exercises*. Every one was found by hand, and every one took a different
//! accident to notice. This driver looks for the rest of them on purpose.
//!
//! ## The mechanism it looks for
//!
//! A tradeoff in this engine is a clamp (`a.min(b)`) or a threshold (`x >= k`).
//! Either one stops being a tradeoff when one side wins every time, and the
//! consequence is exact rather than approximate: **a saturated site has zero
//! derivative with respect to the losing side.** Any knob whose only path to the
//! objective runs through that side is therefore inert — not weakly coupled,
//! not noisy, *invisible*. That is why `cargo_unit_size` reads `-1.2 ± 35.3`
//! and why five values of it were once bit-identical.
//!
//! So the search for broken tradeoffs and the search for lost gradient
//! sensitivity are the same search, and it can be run without any sweep at all:
//! count which side binds (`crate::census`), and rank sites by how one-sided
//! they are.
//!
//! ## The loop
//!
//! ```text
//! cargo run --release --example tradeoff_audit -- baseline   # operating point, both horizons
//! cargo run --release --example tradeoff_audit -- audit      # census -> ranked suspects
//! cargo run --release --example tradeoff_audit -- isolate    # sweep each suspect's knob to find where it un-saturates
//! cargo run --release --example tradeoff_audit               # all three, in order
//! ```
//!
//! **Audit** is one instrumented run: it costs a single simulation and names
//! every site that is one-sided, with the share.
//!
//! **Isolate** is the step that separates a *bug* from a *design*. For each
//! suspect it sweeps the knob feeding the losing side over a wide logarithmic
//! range and reports the value — if any — at which the site starts binding,
//! together with what the objective does there. Three outcomes, and they call
//! for different things:
//!
//! - **Un-saturates inside the legal range, and the objective moves.** The knob
//!   is live but the *default* sits outside its working band. That is a tuning
//!   finding, and the sweep hands you the band.
//! - **Un-saturates, and the objective does not move.** The site binds but leads
//!   nowhere; the quantity is real and unimportant. Leave it.
//! - **Never un-saturates at any value.** The knob cannot reach the objective at
//!   all through this path. That is a modelling finding — a missing term, a
//!   dead branch, or a constant wearing a knob's clothes.
//!
//! Screening runs at the calibrated `colonies@2000` proxy (CLAUDE.md §2,
//! ρ = 0.923); the baseline is reported at both horizons so the proxy's scale
//! is visible rather than assumed. **Nothing here ratifies a default** — it
//! nominates candidates, and the objective confirms them.

use std::io::Write;

use hyades_engine::autopilot::{Autopilot, BaselineAutopilot, Doctrine};
use hyades_engine::census::Site;
use hyades_engine::prelude::*;

const PLAYERS: usize = 3;
/// The standard CRN bed, shared with `gradient_probe` and `gradient_step`.
const SEEDS: &[u64] = &[1, 7, 42, 31337];
/// Calibrated screening horizon (CLAUDE.md §2): ρ = 0.923 against the objective.
const SCREEN_HORIZON: f64 = 2000.0;
/// A side taking at least this share owns the site outright.
const SATURATION: f64 = 0.98;

/// Colonies held at the horizon — the objective, an absolute count.
fn trial(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> f64 {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.run();
    sim.snapshot().planets.iter().filter(|p| p.owner.is_some()).count() as f64
}

fn profile(cfg: SimConfig, doctrine: Doctrine) -> Vec<f64> {
    SEEDS.iter().map(|&s| trial(s, cfg, doctrine)).collect()
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

/// Run one seed with the census on, returning the counters.
fn census_run(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> hyades_engine::census::BindingCensus {
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.enable_binding_census();
    sim.run();
    sim.binding_census().clone()
}

fn baseline() {
    println!("\n=== BASELINE — merged defaults, {} seats, {} seeds (CRN) ===", PLAYERS, SEEDS.len());
    let full = profile(SimConfig::new(0), Doctrine::default());
    let mut screen_cfg = SimConfig::new(0);
    screen_cfg.horizon_years = SCREEN_HORIZON;
    let screen = profile(screen_cfg, Doctrine::default());
    println!(
        "  objective  (4000 yr): {:>8.1} ± {:.1} colonies   per-seed {:?}",
        mean(&full),
        stderr(&full),
        full.iter().map(|x| format!("{x:.0}")).collect::<Vec<_>>()
    );
    println!(
        "  screen     (2000 yr): {:>8.1} ± {:.1} colonies   per-seed {:?}",
        mean(&screen),
        stderr(&screen),
        screen.iter().map(|x| format!("{x:.0}")).collect::<Vec<_>>()
    );
    println!("  screen / objective ratio: {:.3}", mean(&screen) / mean(&full).max(1.0));
    std::io::stdout().flush().ok();
}

/// A knob the sweep can drive: its name, its default, and how to set it.
type Knob = (&'static str, f64, fn(&mut SimConfig, &mut Doctrine, f64));

/// A site the audit nominates, with the knob that feeds its losing side.
struct Suspect {
    site: Site,
    dominant: usize,
    share: f64,
    /// The knob whose only path to the objective runs through the losing side,
    /// and the range worth sweeping it over. `None` where no single knob owns it.
    knob: Option<Knob>,
}

fn knob_for(site: Site) -> Option<Knob> {
    let c = SimConfig::new(0);
    let d = Doctrine::default();
    match site {
        Site::FreighterLoad => Some(("cargo_unit_size", c.cargo_unit_size, |c, _, v| c.cargo_unit_size = v)),
        Site::GrowthBiomass => {
            Some(("biosphere_regen_rate", c.biosphere_regen_rate, |c, _, v| c.biosphere_regen_rate = v))
        }
        Site::SurveyReserve => {
            Some(("survey_reserve", d.survey_reserve as f64, |_, d, v| d.survey_reserve = v.max(0.0) as usize))
        }
        Site::RankClass => Some(("rank.k_high", d.rank.k_high, |_, d, v| d.rank.k_high = v)),
        Site::MineralPressure => Some(("rank.mineral_pressure_gain", d.rank.mineral_pressure_gain, |_, d, v| {
            d.rank.mineral_pressure_gain = v
        })),
        // KLiebig / KPotential / ProductionTier / DeepenHeadroom are structural:
        // no single knob owns their losing side, so `isolate` cannot sweep them
        // and says so rather than picking one arbitrarily.
        _ => None,
    }
}

fn audit() -> Vec<Suspect> {
    println!("\n=== AUDIT — which tradeoffs actually bind? (seed 1, full horizon) ===");
    let census = census_run(SEEDS[0], SimConfig::new(0), Doctrine::default());
    let mut suspects = Vec::new();

    for site in Site::ALL {
        let total = census.total(site);
        if total == 0 {
            println!("\n  {:<34} NEVER FIRED — the branch is dead, not one-sided", site.name());
            suspects.push(Suspect { site, dominant: 0, share: 1.0, knob: knob_for(site) });
            continue;
        }
        let parts: Vec<String> = site
            .sides()
            .iter()
            .enumerate()
            .map(|(i, name)| format!("{name} {:.1}%", census.share(site, i).unwrap_or(0.0) * 100.0))
            .collect();
        let verdict = match census.saturated(site, SATURATION) {
            Some((i, share)) => {
                suspects.push(Suspect { site, dominant: i, share, knob: knob_for(site) });
                format!("SATURATED on `{}` ({:.2}%)", site.sides()[i], share * 100.0)
            }
            None => "binds both ways — a live tradeoff".to_string(),
        };
        println!("\n  {:<34} n = {total}", site.name());
        println!("    {}", parts.join("   "));
        println!("    -> {verdict}");
        std::io::stdout().flush().ok();
    }
    suspects
}

/// Sweep a suspect's knob and report where — if anywhere — the site starts
/// binding, and what the objective does there.
fn isolate(suspects: &[Suspect]) {
    println!("\n=== ISOLATE — does the losing side ever bind? (screen horizon {SCREEN_HORIZON:.0} yr) ===");
    // A wide logarithmic ladder: if a knob has a working band at all, a factor
    // of 10^3 either side of the default will find its edge.
    const FACTORS: [f64; 7] = [0.01, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0];

    for s in suspects {
        let Some((name, base, set)) = s.knob else {
            println!("\n  {:<34} no single knob owns the losing side — structural, not tunable", s.site.name());
            continue;
        };
        println!("\n  {} — saturated on `{}` ({:.1}%)", s.site.name(), s.site.sides()[s.dominant], s.share * 100.0);
        println!("  sweeping `{name}` (default {base})");
        println!("    {:>12}  {:>10}  {:>26}  {:>10}", "value", "dominant%", "sides", "colonies");
        let mut unsaturated_at: Option<f64> = None;
        let mut objective: Vec<f64> = Vec::new();
        for f in FACTORS {
            let v = base * f;
            let mut cfg = SimConfig::new(0);
            cfg.horizon_years = SCREEN_HORIZON;
            let mut doc = Doctrine::default();
            set(&mut cfg, &mut doc, v);
            if cfg.hull_ladder_fault().is_some() {
                println!("    {v:>12.4}  (skipped — degenerate hull ladder)");
                continue;
            }
            let census = census_run(SEEDS[0], cfg, doc);
            let total = census.total(s.site);
            if total == 0 {
                println!("    {v:>12.4}  {:>10}  {:>26}  {:>10}", "—", "site never fired", "—");
                continue;
            }
            let dom = census.share(s.site, s.dominant).unwrap_or(0.0);
            let sides: Vec<String> = (0..s.site.sides().len())
                .map(|i| format!("{:.0}%", census.share(s.site, i).unwrap_or(0.0) * 100.0))
                .collect();
            let colonies = trial(SEEDS[0], cfg, doc);
            objective.push(colonies);
            if dom < SATURATION && unsaturated_at.is_none() {
                unsaturated_at = Some(v);
            }
            println!("    {v:>12.4}  {:>9.1}%  {:>26}  {colonies:>10.0}", dom * 100.0, sides.join("/"));
            std::io::stdout().flush().ok();
        }
        // **A saturated site condemns the path, not the knob.** A knob can reach
        // the objective by another route entirely — `biosphere_regen_rate` does,
        // through `K = min(hab, bio, infra)` — and reporting it as inert because
        // *one* of its paths is dead would be the same error in a new place. So
        // the verdict reads the objective's spread across the sweep as well.
        let spread = match (
            objective.iter().cloned().fold(f64::NAN, f64::max),
            objective.iter().cloned().fold(f64::NAN, f64::min),
        ) {
            (hi, lo) if hi.is_finite() && lo.is_finite() => hi - lo,
            _ => 0.0,
        };
        let moves = spread > 0.05 * objective.first().copied().unwrap_or(1.0).max(1.0);
        match (unsaturated_at, moves) {
            (Some(v), _) => println!(
                "    -> un-saturates at `{name}` = {v} (objective spread {spread:.0} colonies)\n\
                 \x20      the knob is live and the default sits outside this site's working band"
            ),
            (None, true) => println!(
                "    -> stays saturated across 4 orders of magnitude, but the objective moves\n\
                 \x20      ({spread:.0} colonies): the knob reaches the objective by ANOTHER path.\n\
                 \x20      This clamp is redundant with that path, not load-bearing. Not a bug."
            ),
            (None, false) => println!(
                "    -> stays saturated AND the objective does not move ({spread:.0} colonies):\n\
                 \x20      this knob cannot reach the objective at all. A real dead path."
            ),
        }
    }
}

/// **Confirm on the objective.** `isolate` runs one seed at the screen horizon,
/// which is enough to see a site un-saturate and nowhere near enough to move a
/// default. This is the paired CRN comparison at the real horizon that a
/// candidate has to clear — the step CLAUDE.md §2 exists to insist on.
fn confirm(name: &str, base: f64, candidate: f64, set: fn(&mut SimConfig, &mut Doctrine, f64)) {
    println!("\n=== CONFIRM — `{name}` {base} vs {candidate}, objective, {} seeds paired ===", SEEDS.len());
    let mut a_cfg = SimConfig::new(0);
    let mut a_doc = Doctrine::default();
    set(&mut a_cfg, &mut a_doc, base);
    let mut b_cfg = SimConfig::new(0);
    let mut b_doc = Doctrine::default();
    set(&mut b_cfg, &mut b_doc, candidate);
    let a = profile(a_cfg, a_doc);
    let b = profile(b_cfg, b_doc);
    let diffs: Vec<f64> = b.iter().zip(a.iter()).map(|(x, y)| x - y).collect();
    println!(
        "  {base:>10} -> {:.1} colonies\n  {candidate:>10} -> {:.1} colonies\n  paired {:+.1} ± {:.1}   per-seed {:?}",
        mean(&a),
        mean(&b),
        mean(&diffs),
        stderr(&diffs),
        diffs.iter().map(|d| format!("{d:+.0}")).collect::<Vec<_>>()
    );
    let se = stderr(&diffs);
    println!(
        "  verdict: {}",
        if mean(&diffs).abs() > 2.0 * se { "clears 2 SE" } else { "inside 2 SE — not a finding" }
    );
    std::io::stdout().flush().ok();
}

/// **The candidate fix for gradient sensitivity: score the ramp, not only the
/// end state.**
///
/// The confirm step above found `cargo_unit_size` worth +34 colonies on the
/// screen and nothing at all (−13.5 ± 14.9) on the objective. That is not the
/// proxy failing at random — it is the two metrics measuring different things.
/// `reach_limit` established that a run takes 95–100% of what `k_high` admits
/// well before the horizon, so **the end state is saturated**: a knob that makes
/// expansion *faster* arrives at the same place and scores zero. Its derivative
/// is not small, it is structurally absent.
///
/// Colony-years recovers it. `AUC = ∫ N(t) dt = Σ (horizon − t_founded)`, exact
/// from the founding log because colonies are never lost, and free — the same
/// run yields both numbers. It is still an absolute count with no denominator,
/// so it keeps the invariance property the objective was corrected for: there is
/// nothing in it a card could move without moving the world.
///
/// This does **not** propose replacing the objective. It proposes carrying AUC
/// alongside it as the *search* signal, because a search cannot follow a
/// gradient the objective does not have.
fn auc_and_end(seed: u64, cfg: SimConfig, doctrine: Doctrine) -> (f64, f64) {
    use hyades_engine::log::{LogCategory, LogEvent, LogFilter};
    let galaxy = Galaxy::generate(GalaxyConfig::new(PLAYERS, seed)).unwrap();
    let autopilots: Vec<Box<dyn Autopilot>> =
        (0..PLAYERS).map(|_| Box::new(BaselineAutopilot::new(doctrine)) as Box<_>).collect();
    let mut sim = Simulation::new(galaxy, cfg, autopilots);
    sim.set_log_filter(LogFilter::none().with(LogCategory::Vehicles));
    sim.run();
    let horizon = cfg.horizon_years;
    let mut auc = 0.0;
    for rec in sim.log().iter() {
        if matches!(rec.event, LogEvent::ColonyFounded { .. }) {
            auc += horizon - rec.time;
        }
    }
    let end = sim.snapshot().planets.iter().filter(|p| p.owner.is_some()).count() as f64;
    (end, auc)
}

fn compare_metrics(name: &str, base: f64, candidate: f64, set: fn(&mut SimConfig, &mut Doctrine, f64)) {
    println!("\n=== GRADIENT SENSITIVITY — end state vs colony-years, `{name}` {base} vs {candidate} ===");
    let run_all = |v: f64| -> (Vec<f64>, Vec<f64>) {
        let mut ends = Vec::new();
        let mut aucs = Vec::new();
        for &seed in SEEDS {
            let mut cfg = SimConfig::new(0);
            let mut doc = Doctrine::default();
            set(&mut cfg, &mut doc, v);
            let (e, a) = auc_and_end(seed, cfg, doc);
            ends.push(e);
            aucs.push(a);
        }
        (ends, aucs)
    };
    let (e_a, a_a) = run_all(base);
    let (e_b, a_b) = run_all(candidate);
    let de: Vec<f64> = e_b.iter().zip(e_a.iter()).map(|(x, y)| x - y).collect();
    let da: Vec<f64> = a_b.iter().zip(a_a.iter()).map(|(x, y)| x - y).collect();
    // Report each in units of its own SE — the only way to compare a count
    // against an integral of a count.
    let t = |d: &[f64]| {
        let se = stderr(d);
        if se > 0.0 {
            mean(d) / se
        } else {
            0.0
        }
    };
    println!(
        "  end-state colonies : {:>12.1} -> {:>12.1}   paired {:+.1} ± {:.1}   t = {:+.2}",
        mean(&e_a),
        mean(&e_b),
        mean(&de),
        stderr(&de),
        t(&de)
    );
    println!(
        "  colony-years (AUC) : {:>12.0} -> {:>12.0}   paired {:+.0} ± {:.0}   t = {:+.2}",
        mean(&a_a),
        mean(&a_b),
        mean(&da),
        stderr(&da),
        t(&da)
    );
    println!(
        "  -> {}",
        if t(&da).abs() > 2.0 && t(&de).abs() <= 2.0 {
            "AUC resolves a difference the end state cannot see: sensitivity restored"
        } else if t(&da).abs() > t(&de).abs() {
            "AUC is the sharper signal, but neither clears 2 SE here"
        } else {
            "AUC adds nothing on this knob"
        }
    );
    std::io::stdout().flush().ok();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let all = args.is_empty();
    println!("Broken-tradeoff audit — a saturated clamp has zero derivative on its losing side.");

    if all || args.iter().any(|a| a == "baseline") {
        baseline();
    }
    if all || args.iter().any(|a| a == "audit") || args.iter().any(|a| a == "isolate") {
        let suspects = audit();
        println!(
            "\n  {} of {} sites saturated at the {:.0}% bar.",
            suspects.len(),
            Site::ALL.len(),
            SATURATION * 100.0
        );
        if all || args.iter().any(|a| a == "isolate") {
            isolate(&suspects);
        }
    }
    if let Some(i) = args.iter().position(|a| a == "auc") {
        let cand: f64 = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(25.0);
        compare_metrics("cargo_unit_size", SimConfig::new(0).cargo_unit_size, cand, |c, _, v| c.cargo_unit_size = v);
    }
    if let Some(i) = args.iter().position(|a| a == "confirm") {
        let cand: f64 = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(25.0);
        confirm("cargo_unit_size", SimConfig::new(0).cargo_unit_size, cand, |c, _, v| c.cargo_unit_size = v);
    }
}
