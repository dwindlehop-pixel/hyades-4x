//! The **autopilot interface** — the seam where player agency meets the sim.
//!
//! Two distinct things live here, matching the design's two-layer split:
//!
//! 1. [`Doctrine`] — the *standing-behavior knobs*. "Player agency is the power
//!    to change the defaults" (`Hyades_simulation_model.md` §0a); a card "edits
//!    the standing behavior" by mutating **exactly one** doctrine field
//!    (`Hyades_card_contract.md` §5). The sim itself never reads cards — it reads
//!    the Doctrine the cards have already edited.
//! 2. [`Autopilot`] — the *policy trait*. The simulation calls it for every
//!    decision (rank a planet, pick a survey target, choose a build). The
//!    [`BaselineAutopilot`] is the colonization/growth policy of
//!    `Hyades_autopilot_colonization_growth.md`; the Monte-Carlo greedy-`V`
//!    policy (`Hyades_card_contract.md` §7) will be a second impl. Swapping the
//!    policy per seat is how the balancer pits doctrines against each other.
//!
//! Crucially the autopilot reads only a player's **scanned** view (fog of war,
//! autopilot-doc §1) — never ground truth — so the sim hands it small `Copy`
//! view structs rather than its internals.

use crate::galaxy::{PlanetClass, PlanetId, PlayerId};
use crate::math::Vec3;
use crate::resources::MineralField;

/// Which of the two cheap classes the colony pipeline reaches for first
/// (autopilot-doc §4; R-AC1 / R-A1). Default is production-centers-first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpandBias {
    ProductionCentersFirst,
    ColoniesFirst,
}

/// Weights & thresholds for the numeric planet rank (autopilot-doc §3, R-AC5).
/// `score = w_k·K_potential + w_mineral·mineral_value + w_hub·hub_value`; the
/// thresholds then assign the [`PlanetClass`].
#[derive(Clone, Copy, Debug)]
pub struct RankWeights {
    pub w_k: f64,
    pub w_mineral: f64,
    pub w_hub: f64,
    /// K-potential at/above which a world is "high-K" (center- or colony-class).
    pub k_high: f64,
    /// mineral_value at/above which a low-K world is a mining outpost.
    pub mineral_high: f64,
    /// hub_value at/above which a high-K world is a production center.
    pub hub_high: f64,
    /// ly scale over which centrality-to-holdings decays (hub value falloff).
    pub centrality_scale: f64,
    /// How strongly live mineral *scarcity* (the empire running short for its
    /// builds) inflates a world's mineral_value — i.e. how much the ranking
    /// pivots toward mining when starved. The optimal value is an MC question
    /// (R-AC5); this is the tunable baseline.
    pub mineral_pressure_gain: f64,
}

impl Default for RankWeights {
    fn default() -> Self {
        RankWeights {
            w_k: 1.0,
            w_mineral: 0.8,
            w_hub: 1.2,
            k_high: 1.5,
            mineral_high: 2.0,
            hub_high: 0.8,
            centrality_scale: 150.0,
            mineral_pressure_gain: 1.0,
        }
    }
}

/// The standing behavior the sim executes "unasked" — every field a default a
/// card can override (sim §0a). Defaults are the round-1 baseline values
/// (six-vehicle survey, 1 g, +20% productivity step).
#[derive(Clone, Copy, Debug)]
pub struct Doctrine {
    // --- Exploit / Growth (the build cycle, autopilot-doc §6) ---
    /// Base productivity step per build cycle. `0.20` is the doctrine value The
    /// Compass retunes (autopilot-doc §6, R-AC11).
    pub productivity_step: f64,
    /// Logistic growth rate `r` for population toward `K` per cycle.
    pub growth_rate: f64,

    // --- Explore / Survey (autopilot-doc §2) ---
    /// Number of survey vehicles in the opening fan-out. Base `6` (cube faces).
    pub survey_vehicles: usize,
    /// Survey acceleration in g (base `1.0`).
    pub survey_accel_g: f64,

    // --- Expand (autopilot-doc §4) ---
    pub expand_bias: ExpandBias,

    /// **Expansion rate knob** (MC experiment): how strongly the production
    /// queue favors *upgrading own infrastructure* (deepening) over *spending
    /// minerals to reach outward* (expanding). `0.0` = always expand when able,
    /// `1.0` = always deepen toward `K` first. The optimal value is state-
    /// dependent (current pop, K-potential, neighbors) and is exactly what the
    /// expansion-rate Monte-Carlo experiment optimizes (R-AC11/R-AC12).
    pub reinvest_bias: f64,

    // --- Ranking (autopilot-doc §3) ---
    pub rank: RankWeights,
}

impl Default for Doctrine {
    fn default() -> Self {
        Doctrine {
            productivity_step: 0.20,
            growth_rate: 0.5,
            survey_vehicles: 6,
            survey_accel_g: 1.0,
            expand_bias: ExpandBias::ProductionCentersFirst,
            reinvest_bias: 0.5,
            rank: RankWeights::default(),
        }
    }
}

/// A planet as the autopilot sees it after a close scan (fog-of-war view).
#[derive(Clone, Copy, Debug)]
pub struct PlanetView {
    pub id: PlanetId,
    pub position: Vec3,
    pub habitability: f64,
    pub biosphere: f64,
    pub minerals: MineralField,
    pub owner: Option<PlayerId>,
    pub pop_level: u8,
}

impl PlanetView {
    /// Ceiling infra can be built to (autopilot-doc §3).
    #[inline]
    pub fn k_potential(&self) -> f64 {
        self.habitability.min(self.biosphere)
    }
}

/// Context the rank reads about *this* empire's standing state.
#[derive(Clone, Copy, Debug)]
pub struct RankContext {
    /// Per-basic scarcity weight `[C, M, Y]` (higher = scarcer = more valued).
    pub scarcity: [f64; 3],
    /// Centroid of the empire's current holdings (for centrality / hub value).
    pub holdings_centroid: Vec3,
    /// Live mineral pressure `∈ [0, 1]`: 0 when the deciding center can comfortably
    /// fund its builds, →1 when it is starved. Raises the value of mining targets
    /// so the ranking reflects the *current* need for minerals (optimal growth).
    pub mineral_pressure: f64,
}

/// A planet's numeric rank result (autopilot-doc §3).
#[derive(Clone, Copy, Debug)]
pub struct Ranked {
    pub id: PlanetId,
    pub score: f64,
    pub class: PlanetClass,
}

/// A scanned, unexploited candidate the production cycle may target.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub view: PlanetView,
    pub ranked: Ranked,
}

/// What a production center decides to build this cycle (autopilot-doc §§4–6).
/// All builds draw minerals from the center's stockpile (3 CMY ≈ 3 ships is the
/// costing anchor); infrastructure upgrades cost minerals equal to the target
/// level. The relative value of the three growth moves — deepen (infra),
/// colonize, or mine — is an **optimal-growth question settled by Monte-Carlo
/// experiment** (R-AC5/R-AC11), so the baseline here is a tunable heuristic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildOrder {
    /// Nothing affordable/worth building this cycle.
    Idle,
    /// A survey craft launched along `heading` (the opening fan-out, §2).
    LightVehicle { heading: Vec3 },
    /// Spend minerals to raise this center's own infrastructure by one level
    /// (deepening toward `K`). Cost = target level (1→2 costs 2, …).
    UpgradeInfrastructure,
    /// A colony vehicle bound for a center- or colony-class world (§4). Founds a
    /// colony and is recycled into its level-1 infrastructure on arrival.
    ColonyVehicle { target: PlanetId },
    /// A mining vehicle bound for a mining outpost (§5); it shuttles cargo home.
    MiningPair { target: PlanetId },
}

/// Everything a production center needs to weigh deepen-vs-expand this cycle.
#[derive(Clone, Copy, Debug)]
pub struct ProductionContext {
    pub center_pos: Vec3,
    /// Current development level 0–4 of the center.
    pub level: u8,
    /// Current infrastructure value.
    pub infra: f64,
    /// `min(hab, bio)` — the ceiling infrastructure can be built to.
    pub k_potential: f64,
    /// Minerals on hand at this center (the spendable pool).
    pub stockpile_total: f64,
    /// Minimum level required to build "medium" vehicles (colony/mining). Per the
    /// production schedule this is **3** (2 = limited, 3 = medium/rapid, 4 = all).
    pub medium_min_level: u8,
    /// Mineral cost to raise infra by one level (= the target level).
    pub infra_cost: f64,
    /// Mineral cost of a Colonizer (an MSV) — `Hyades_vehicle_roles.md` §6's
    /// 1 CMY = 1 fleet model, not a flat placeholder anymore.
    pub colonizer_cost: f64,
    /// Mineral cost of a Miner + its paired Freighter (an LSV + an MSV),
    /// bundled since they're built together (§4.4).
    pub mining_pair_cost: f64,
}

/// The swappable per-seat decision **algorithm** (`Hyades_vehicle_roles.md`
/// §9 — confirmed this conversation: "autopilot isn't a resource because
/// these are just components and systems"). An implementor is stateless
/// dispatch code, the System half of the pair; the tunable [`Doctrine`] it
/// reads is data, the Component half, living on the player entity in
/// `sim::World` from [`Autopilot::default_doctrine`] onward — never owned by
/// the object implementing this trait.
pub trait Autopilot {
    /// The starting [`Doctrine`] this policy seeds a seat with. Called once,
    /// at bootstrap, to populate the player's Doctrine component; never read
    /// again afterward — from that point the live, mutable value is the
    /// component, not this object.
    fn default_doctrine(&self) -> Doctrine;

    /// Numeric rank + class for one close-scanned planet (autopilot-doc §3).
    fn rank(&self, doctrine: &Doctrine, view: &PlanetView, ctx: &RankContext) -> Ranked;

    /// Pick the next survey target: nearest unscanned, optionally biased to a
    /// heading hemisphere for the opening fan-out (autopilot-doc §2, R-AC3).
    fn choose_survey_target(
        &self,
        doctrine: &Doctrine,
        from: Vec3,
        heading_bias: Option<Vec3>,
        unscanned: &[PlanetView],
    ) -> Option<PlanetId>;

    /// Decide this cycle's build for a production center (autopilot-doc §§4–6),
    /// weighing infrastructure-deepening against colonizing or mining under the
    /// center's mineral budget and level gates.
    fn production_choice(&self, doctrine: &Doctrine, ctx: &ProductionContext, candidates: &[Candidate]) -> BuildOrder;
}

/// The baseline colonization/growth policy (`Hyades_autopilot_colonization_growth.md`).
/// Holds a [`Doctrine`] only as the seed value [`Autopilot::default_doctrine`]
/// hands to bootstrap — not live state; the struct itself is otherwise a
/// stateless dispatcher (every method takes `doctrine` as a parameter).
#[derive(Clone, Debug, Default)]
pub struct BaselineAutopilot {
    pub doctrine: Doctrine,
}

impl BaselineAutopilot {
    pub fn new(doctrine: Doctrine) -> Self {
        BaselineAutopilot { doctrine }
    }
}

impl Autopilot for BaselineAutopilot {
    fn default_doctrine(&self) -> Doctrine {
        self.doctrine
    }

    fn rank(&self, doctrine: &Doctrine, view: &PlanetView, ctx: &RankContext) -> Ranked {
        let w = &doctrine.rank;

        let k_potential = view.k_potential();

        // mineral_value: scarcity-weighted tier-1 density (§3), inflated by the
        // empire's *live* mineral pressure so mining is valued when we're short.
        let m = &view.minerals;
        let base_mineral = ctx.scarcity[0] * m.cyan + ctx.scarcity[1] * m.magenta + ctx.scarcity[2] * m.yellow;
        let mineral_value = base_mineral * (1.0 + w.mineral_pressure_gain * ctx.mineral_pressure);

        // hub_value: high-K worlds near the empire's centre of mass are hubs.
        let dist = view.position.distance(ctx.holdings_centroid);
        let centrality = (-dist / w.centrality_scale).exp();
        let hub_value = k_potential * centrality;

        let score = w.w_k * k_potential + w.w_mineral * mineral_value + w.w_hub * hub_value;

        // classification (§3): thresholds on the components.
        let class = if k_potential >= w.k_high {
            if hub_value >= w.hub_high {
                PlanetClass::ProductionCenter
            } else {
                PlanetClass::Colony
            }
        } else if mineral_value >= w.mineral_high {
            PlanetClass::MiningOutpost
        } else {
            PlanetClass::Barren
        };

        Ranked { id: view.id, score, class }
    }

    fn choose_survey_target(
        &self,
        _doctrine: &Doctrine,
        from: Vec3,
        heading_bias: Option<Vec3>,
        unscanned: &[PlanetView],
    ) -> Option<PlanetId> {
        // Prefer the heading hemisphere (dot > 0); fall back to global nearest.
        let pick = |restrict: bool| -> Option<(PlanetId, f64)> {
            unscanned
                .iter()
                .filter(|p| {
                    !restrict
                        || match heading_bias {
                            Some(h) => p.position.sub(from).dot(h) > 0.0,
                            None => true,
                        }
                })
                .map(|p| (p.id, p.position.distance(from)))
                // deterministic argmin: distance, tie-broken by id
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0 .0.cmp(&b.0 .0)))
        };
        pick(heading_bias.is_some()).or_else(|| pick(false)).map(|(id, _)| id)
    }

    fn production_choice(&self, doctrine: &Doctrine, ctx: &ProductionContext, candidates: &[Candidate]) -> BuildOrder {
        let deepen_possible = ctx.infra + 1.0 <= ctx.k_potential + 1e-9;
        let can_afford_infra = ctx.stockpile_total + 1e-9 >= ctx.infra_cost;

        // Below the medium gate the only growth move is to deepen toward the
        // expansion level; save (Idle) if the upgrade isn't yet affordable.
        if ctx.level < ctx.medium_min_level {
            return if deepen_possible && can_afford_infra {
                BuildOrder::UpgradeInfrastructure
            } else {
                BuildOrder::Idle
            };
        }

        // Mature enough to expand. Find the best colony target and outpost.
        let best_center =
            candidates.iter().filter(|c| c.ranked.class == PlanetClass::ProductionCenter).max_by(score_then_id);
        let best_colony = candidates.iter().filter(|c| c.ranked.class == PlanetClass::Colony).max_by(score_then_id);
        let best_mining =
            candidates.iter().filter(|c| c.ranked.class == PlanetClass::MiningOutpost).max_by(score_then_id);

        let colony_target = match doctrine.expand_bias {
            ExpandBias::ProductionCentersFirst => best_center.or(best_colony),
            ExpandBias::ColoniesFirst => best_colony.or(best_center),
        };

        // The best outward move (colony vs mining, by score).
        let outward = match (colony_target, best_mining) {
            (Some(col), Some(mine)) if mine.ranked.score > col.ranked.score => {
                Some((BuildOrder::MiningPair { target: mine.ranked.id }, mine.ranked.score))
            }
            (Some(col), _) => Some((BuildOrder::ColonyVehicle { target: col.ranked.id }, col.ranked.score)),
            (None, Some(mine)) => Some((BuildOrder::MiningPair { target: mine.ranked.id }, mine.ranked.score)),
            (None, None) => None,
        };
        let outward_cost = match &outward {
            Some((BuildOrder::ColonyVehicle { .. }, _)) => ctx.colonizer_cost,
            Some((BuildOrder::MiningPair { .. }, _)) => ctx.mining_pair_cost,
            _ => 0.0,
        };
        let can_expand = ctx.stockpile_total + 1e-9 >= outward_cost;

        // Deepen-vs-expand as a genuine convex dial. `reinvest_bias` shifts weight
        // between deepening this center's own K and reaching outward. Crucially the
        // *preference* is computed independent of what is affordable this cycle:
        // when deepening wins but the (pricier) upgrade isn't funded yet, the
        // center **saves** (Idle) rather than frittering minerals on cheap
        // vehicles. That is what lets the bias actually trade expansion for depth.
        // The optimal bias — possibly state-dependent on pop / K / neighbors — is
        // the expansion-rate MC experiment; this is the tunable baseline.
        let b = doctrine.reinvest_bias;
        let deepen_headroom = (ctx.k_potential - ctx.infra).max(0.0);
        let w_deepen = if deepen_possible { b * deepen_headroom } else { f64::NEG_INFINITY };
        let w_expand = match outward {
            Some((_, score)) => (1.0 - b) * score,
            None => f64::NEG_INFINITY,
        };

        if w_deepen >= w_expand && deepen_possible {
            // Prefer depth: upgrade if funded, else save toward it.
            if can_afford_infra {
                BuildOrder::UpgradeInfrastructure
            } else {
                BuildOrder::Idle
            }
        } else if let Some((order, _)) = outward {
            // Prefer expansion: build if funded, else save toward the vehicle.
            if can_expand {
                order
            } else {
                BuildOrder::Idle
            }
        } else if deepen_possible && can_afford_infra {
            BuildOrder::UpgradeInfrastructure
        } else {
            BuildOrder::Idle
        }
    }
}

/// Deterministic comparison for `max_by`: higher score wins, ties broken by id.
fn score_then_id(a: &&Candidate, b: &&Candidate) -> core::cmp::Ordering {
    a.ranked
        .score
        .partial_cmp(&b.ranked.score)
        .unwrap_or(core::cmp::Ordering::Equal)
        .then(a.ranked.id.0.cmp(&b.ranked.id.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::MineralField;

    fn view(id: u32, pos: Vec3, hab: f64, bio: f64, minerals: MineralField) -> PlanetView {
        PlanetView {
            id: PlanetId(id),
            position: pos,
            habitability: hab,
            biosphere: bio,
            minerals,
            owner: None,
            pop_level: 0,
        }
    }

    #[test]
    fn rich_low_hab_world_is_a_mining_outpost() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let ctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(
            1,
            Vec3::new(10.0, 0.0, 0.0),
            0.4, // low habitability
            0.4,
            MineralField { cyan: 3.0, magenta: 0.5, yellow: 0.2 },
        );
        assert_eq!(ap.rank(&doctrine, &v, &ctx).class, PlanetClass::MiningOutpost);
    }

    #[test]
    fn habitable_central_world_is_a_production_center() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let ctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(2, Vec3::new(5.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        assert_eq!(ap.rank(&doctrine, &v, &ctx).class, PlanetClass::ProductionCenter);
    }

    #[test]
    fn distant_habitable_world_is_a_colony() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let ctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        // far from holdings ⇒ low hub value ⇒ colony, not center
        let v = view(3, Vec3::new(900.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        assert_eq!(ap.rank(&doctrine, &v, &ctx).class, PlanetClass::Colony);
    }

    #[test]
    fn survey_prefers_nearest() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let cands = vec![
            view(1, Vec3::new(100.0, 0.0, 0.0), 1.0, 1.0, MineralField::default()),
            view(2, Vec3::new(10.0, 0.0, 0.0), 1.0, 1.0, MineralField::default()),
        ];
        assert_eq!(ap.choose_survey_target(&doctrine, Vec3::ZERO, None, &cands), Some(PlanetId(2)));
    }

    fn prod_ctx(level: u8, infra: f64, stockpile: f64) -> ProductionContext {
        ProductionContext {
            center_pos: Vec3::ZERO,
            level,
            infra,
            k_potential: 4.0,
            stockpile_total: stockpile,
            medium_min_level: 3,
            infra_cost: infra + 1.0,
            colonizer_cost: 1.0,
            mining_pair_cost: 1.0,
        }
    }

    #[test]
    fn below_medium_gate_deepens_infrastructure() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // level 2, can afford the 3-mineral upgrade, no candidates yet.
        let order = ap.production_choice(&doctrine, &prod_ctx(2, 2.0, 5.0), &[]);
        assert_eq!(order, BuildOrder::UpgradeInfrastructure);
    }

    #[test]
    fn below_gate_with_no_minerals_idles() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let order = ap.production_choice(&doctrine, &prod_ctx(2, 2.0, 0.0), &[]);
        assert_eq!(order, BuildOrder::Idle);
    }

    #[test]
    fn mature_center_expands_to_a_colony_when_affordable() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let ctx = prod_ctx(3, 3.0, 2.0);
        let rctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(5, Vec3::new(10.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        let ranked = ap.rank(&doctrine, &v, &rctx);
        let cands = vec![Candidate { view: v, ranked }];
        let order = ap.production_choice(&doctrine, &ctx, &cands);
        assert!(matches!(order, BuildOrder::ColonyVehicle { .. }));
    }
}
