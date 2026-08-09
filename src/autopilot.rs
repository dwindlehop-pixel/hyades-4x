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
use crate::sim::{Class, HullType, Role};

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
    /// K-potential at/above which a world is "high-K" (center- or colony-class);
    /// below it, a mineral-rich world is a *Mining outpost* instead. **This
    /// threshold gates the entire hauling economy** — set it under the galaxy's
    /// K distribution and no outpost is ever classified, so no freighter ever
    /// flies and colonies cannot fund their way to the expansion tier (R-AC17).
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
            // 3.2 — ratified: "the snowball is the design". The old 1.5 was
            // tuned to a ~25 ly galaxy; against the current one, where 99% of
            // planets have min(hab,bio) >= 1.76, it made the Mining-outpost
            // class unreachable and stalled expansion at a few dozen colonies.
            // 3.2 sits just above the median K (~3.22), so the low-K half of the
            // galaxy becomes mining and the high-K half becomes colonies.
            // Validated 4/4 test-bed seeds to 100% of colonizable worlds.
            k_high: 3.2,
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
    /// Multiplier on `SimConfig::biosphere_regen_rate` for this empire's worlds.
    /// The card lever on ecology: Growth-tree cards raise it to make a
    /// biosphere recover faster, and hostile cards lower it — or attack
    /// `bio_max` directly — to make the wound durable. `1.0` is untouched
    /// baseline; `0.0` means a razed world never comes back on its own.
    pub biosphere_regen_bonus: f64,

    // --- Explore / Survey (autopilot-doc §2) ---
    /// Number of survey vehicles in the opening fan-out. Base `6` (cube faces).
    pub survey_vehicles: usize,
    /// Survey acceleration in g (base `1.0`).
    pub survey_accel_g: f64,
    /// How many known, unclaimed candidate worlds the empire wants on hand.
    /// When [`ProductionContext::candidate_count`] falls below this, a center
    /// at the limited tier builds a Scout instead of idling — this is what
    /// makes survey scale with the empire rather than being fixed at the
    /// bootstrap fan-out. `0` restores the old behavior (never build survey
    /// craft after bootstrap). **1024, ratified** (R-AC16) as part of the
    /// snowball defaults; the offline search may refine it, but the stalled
    /// low-reserve configuration is no longer the reference.
    pub survey_reserve: usize,
    /// Skip survey targets showing [`SurveyView::industrial_signature`] — a
    /// pop-4 world is almost certainly already held, so flying to it spends a
    /// hop to learn something the spectrometry already said.
    ///
    /// **Default `false`, deliberately** (R-SIM3). Early game there is no
    /// filtering at all: an empire that has not developed the instruments or the
    /// doctrine to act on remote signatures simply flies out and finds out, and
    /// those wasted hops are part of what early expansion costs. This is exactly
    /// the shape of a card — one field, standing behavior, flipped when the
    /// right tech or board state is reached (`Hyades_card_contract.md` §5).
    pub survey_avoids_inhabited: bool,

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
            biosphere_regen_bonus: 1.0,
            survey_vehicles: 6,
            survey_accel_g: 1.0,
            // 1024 — ratified with k_high above; survey must scale with the
            // empire or expansion outruns its own map. Monotone by construction
            // (survey is a fallback, never a pre-emption), so raising it is safe.
            survey_reserve: 1024,
            // Off until a card or board state turns it on — see the field doc.
            survey_avoids_inhabited: false,
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

/// What a survey craft may know about a world it has **not yet visited** — the
/// *remote* tier of `Hyades_autopilot_colonization_growth.md` §1.
///
/// The spec draws the fog line precisely: *"Biosphere and Habitability are known
/// from interstellar distance (remote spectroscopy). Ownership, infrastructure,
/// and mineral density require a close-range scan."* So a survey target may be
/// picked on position and the K-ceiling factors, and on nothing else — this type
/// exists so that boundary is enforced by the type system rather than by the
/// policy's good manners.
///
/// It replaced [`PlanetView`] in [`Autopilot::choose_survey_target`], which had
/// been handing out `minerals`, `owner` and `pop_level` for unscanned worlds
/// (all close-scan-only facts) and filtering the candidate list on ground-truth
/// ownership. Sizing the view to the query also removed the engine's single
/// hottest cost — see `Hyades_simulation_model.md` §2b.
#[derive(Clone, Copy, Debug)]
pub struct SurveyView {
    pub id: PlanetId,
    pub position: Vec3,
    pub habitability: f64,
    pub biosphere: f64,
    /// **Inferential tier** (R-SIM3): this world carries the waste-heat and
    /// atmospheric signature of a pop-4 civilization, legible at interstellar
    /// range. It does *not* say who owns it — the industry of billions is simply
    /// not concealable from spectrometry, so an empire may reasonably conclude
    /// the world is taken without ever having gone there.
    ///
    /// Pop-4 is the only occupancy signal modelled today, because it is the one
    /// that is exact: a threshold on realized population. The richer signal the
    /// design calls for — departure traffic, where repeated sightings of ships
    /// leaving raise confidence — needs accumulated light-lagged observations
    /// and is R-SIM4.
    pub industrial_signature: bool,
}

impl SurveyView {
    /// Ceiling infra could be built to, from remote spectroscopy alone.
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
    /// Spend minerals to raise this center's own infrastructure by one level
    /// (deepening toward `K`). Cost = target level (1→2 costs 2, …).
    UpgradeInfrastructure,
    /// **Production makes an object, not a mission** (R-O29,
    /// `Hyades_standing_layer_and_observation.md` §7).
    ///
    /// The old variants named the job — `ColonyVehicle { target }`,
    /// `MiningPair { target }` — which leaked doctrine for free: anyone reading
    /// a shipyard learned not just that a hull was laid down but what it was
    /// *for*, with no scan and no lag. Hull and class are what production
    /// decides; [`Tasking`] is a separate decision made afterwards by
    /// [`Autopilot::assign_role`], and role is reassignable thereafter.
    Hull { hull_type: HullType, class: Class },
}

/// What a freshly-produced hull is *for* — assigned after production, never as
/// part of the build order (R-O29). Reassignable: roles §4 already keys
/// eligibility to hull plus loadout, so this is the existing model; the build
/// order was the leak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tasking {
    pub role: Role,
    /// Where it is going. `None` for a hull with nothing to do yet, which holds
    /// station rather than launching.
    pub target: Option<PlanetId>,
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
    /// Minimum level required to build "limited" vehicles — the Scout/LCV. The
    /// same schedule puts this at **2**, one tier below expansion.
    pub limited_min_level: u8,
    /// Mineral cost to raise infra by one level (= the target level).
    pub infra_cost: f64,
    /// Mineral cost of a Colonizer (an MSV) — `Hyades_vehicle_roles.md` §6's
    /// 1 CMY = 1 fleet model, not a flat placeholder anymore.
    pub colonizer_cost: f64,
    /// Mineral cost of a Miner + its paired Freighter (an LSV + an MSV),
    /// bundled since they're built together (§4.4).
    pub mining_pair_cost: f64,
    /// Mineral cost of one Scout (an LCV) — the survey craft the limited tier
    /// unlocks. Bootstrap hands each seat `survey_vehicles` of these free
    /// (autopilot-doc §2); every later one is paid for out of a center's
    /// stockpile like any other build.
    pub light_vehicle_cost: f64,
    /// Known, unclaimed, non-Barren worlds this empire could still expand to.
    /// The autopilot builds survey craft to keep this above
    /// [`Doctrine::survey_reserve`] — expansion consumes candidates, so without
    /// replenishment the empire runs out of places to go long before it runs
    /// out of galaxy.
    pub candidate_count: usize,
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
        unscanned: &[SurveyView],
    ) -> Option<PlanetId>;

    /// Decide this cycle's build for a production center (autopilot-doc §§4–6),
    /// weighing infrastructure-deepening against colonizing or mining under the
    /// center's mineral budget and level gates.
    fn production_choice(&self, doctrine: &Doctrine, ctx: &ProductionContext, candidates: &[Candidate]) -> BuildOrder;

    /// Task a hull that production has just finished (R-O29). Called *after*
    /// the object exists, with the empire's current candidate list — so the
    /// job is chosen from the situation at completion, not baked into the
    /// build order where a rival could read it off the shipyard.
    ///
    /// Returning `None` means the hull has nothing to do and holds station.
    fn assign_role(
        &self,
        doctrine: &Doctrine,
        hull: HullType,
        class: Class,
        candidates: &[Candidate],
    ) -> Option<Tasking>;
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
        doctrine: &Doctrine,
        from: Vec3,
        heading_bias: Option<Vec3>,
        unscanned: &[SurveyView],
    ) -> Option<PlanetId> {
        // Occupancy inferred at range, not ownership known by visiting. Gated on
        // doctrine so the early game does no filtering at all (R-SIM3) — the
        // engine always reports the signature; whether to act on it is a
        // standing behavior a card edits.
        let legible_as_taken = |p: &SurveyView| doctrine.survey_avoids_inhabited && p.industrial_signature;

        // Prefer the heading hemisphere (dot > 0); fall back to global nearest.
        let pick = |restrict: bool| -> Option<(PlanetId, f64)> {
            unscanned
                .iter()
                .filter(|p| !legible_as_taken(p))
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

    fn assign_role(
        &self,
        doctrine: &Doctrine,
        hull: HullType,
        _class: Class,
        candidates: &[Candidate],
    ) -> Option<Tasking> {
        // Eligibility is **permissive with varying competence** (R-O44): any
        // hull may take any role. What differs is how well it does the job, so
        // this picks the competent assignment for each hull rather than the
        // only legal one.
        let best = |want: PlanetClass| candidates.iter().filter(|c| c.ranked.class == want).max_by(score_then_id);
        match hull {
            // A Contact hull scouts. It needs no target here — `launch_survey`
            // picks the nearest unvisited world from the survey frontier.
            HullType::LimitedContactVehicle
            | HullType::LimitedContactUnit
            | HullType::GeneralContactVehicle
            | HullType::GeneralContactUnit => Some(Tasking { role: Role::Scout, target: None }),

            // A Medium hull settles, preferring whichever class doctrine leads
            // with — the same order `production_choice` weighed.
            HullType::MediumSystems => {
                let (a, b) = match doctrine.expand_bias {
                    ExpandBias::ProductionCentersFirst => (PlanetClass::ProductionCenter, PlanetClass::Colony),
                    ExpandBias::ColoniesFirst => (PlanetClass::Colony, PlanetClass::ProductionCenter),
                };
                best(a).or_else(|| best(b)).map(|c| Tasking { role: Role::Colonizer, target: Some(c.ranked.id) })
            }

            // A Limited Systems hull mines; the freighter that hauls for it is
            // produced alongside (roles §5 — the center produces both).
            HullType::LimitedSystems => {
                best(PlanetClass::MiningOutpost).map(|c| Tasking { role: Role::Miner, target: Some(c.ranked.id) })
            }

            // Nothing else is produced yet; hold rather than invent a mission.
            _ => None,
        }
    }

    fn production_choice(&self, doctrine: &Doctrine, ctx: &ProductionContext, candidates: &[Candidate]) -> BuildOrder {
        // Deepen while any headroom remains below the ceiling, rather than only
        // when a whole level fits under it. `K = min(hab, bio, infra)`, so infra
        // overshooting `k_potential` buys nothing — but *blocking* the last
        // partial step strands a center below the level bands for good. A world
        // with `k_potential = 2.86` sits at infra 2 under the old
        // `infra + 1 <= k_potential` test, which caps `K` at 2, which caps
        // population at 2, which never crosses the level-3 band edge (~2.675).
        // It then hoards minerals it can never spend. Measured on seed 1: 1050
        // of 2435 Idle decisions were centers in exactly that state, several
        // holding 3.5–4.7 minerals against a 3-mineral upgrade.
        let deepen_possible = ctx.infra < ctx.k_potential - 1e-9;
        let can_afford_infra = ctx.stockpile_total + 1e-9 >= ctx.infra_cost;

        // Below even the limited tier there is nothing to build; deepen or save.
        if ctx.level < ctx.limited_min_level {
            return if deepen_possible && can_afford_infra {
                BuildOrder::UpgradeInfrastructure
            } else {
                BuildOrder::Idle
            };
        }

        // The limited tier unlocks survey craft. Replenishing the scout fleet is
        // what lets expansion compound: colonies are drawn from *known* worlds,
        // so an empire that never scouts again exhausts its candidate list and
        // stops, however rich it gets.
        let wants_survey = ctx.candidate_count < doctrine.survey_reserve;
        let can_afford_light = ctx.stockpile_total + 1e-9 >= ctx.light_vehicle_cost;

        // Between the limited and medium tiers, survey is the only outward move.
        if ctx.level < ctx.medium_min_level {
            // Deepening toward the medium gate stays the priority — that is what
            // turns this center into a colonizer — but a center that cannot
            // deepen (capped, or saving) still contributes survey rather than
            // idling with a full stockpile.
            if deepen_possible && can_afford_infra {
                return BuildOrder::UpgradeInfrastructure;
            }
            return if wants_survey && can_afford_light && !deepen_possible {
                hull_order(HullType::LimitedContactVehicle)
            } else {
                BuildOrder::Idle
            };
        }

        // With nothing known left to expand to, survey is the only move that can
        // ever restart expansion. This is the one case where it outranks
        // everything: no candidates means every other branch below returns Idle.
        if candidates.is_empty() && can_afford_light {
            return hull_order(HullType::LimitedContactVehicle);
        }

        // Find the best colony target and outpost.
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
        // What the center *wants* to reach still drives which hull it lays down —
        // but the order records the hull, not the errand. `assign_role` re-derives
        // the job from the same candidate list once the object exists.
        let outward = match (colony_target, best_mining) {
            (Some(col), Some(mine)) if mine.ranked.score > col.ranked.score => {
                Some((hull_order(HullType::LimitedSystems), mine.ranked.score, ctx.mining_pair_cost))
            }
            (Some(col), _) => Some((hull_order(HullType::MediumSystems), col.ranked.score, ctx.colonizer_cost)),
            (None, Some(mine)) => Some((hull_order(HullType::LimitedSystems), mine.ranked.score, ctx.mining_pair_cost)),
            (None, None) => None,
        };
        let outward_cost = outward.map(|(_, _, c)| c).unwrap_or(0.0);
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
            Some((_, score, _)) => (1.0 - b) * score,
            None => f64::NEG_INFINITY,
        };

        // Survey is the fallback for a cycle that would otherwise be spent idle,
        // never a pre-emption of an affordable expansion. That ordering matters:
        // an earlier revision gave survey outright priority whenever the frontier
        // was below `survey_reserve`, which made the knob non-monotonic —
        // reserve=256 reached 1047 colonies but reserve=4096 collapsed to 3,
        // because centers scouted every cycle and never colonized at all. As a
        // fallback it is self-limiting: raising the reserve converts idle cycles
        // into survey and can never starve expansion.
        let survey_fallback = if wants_survey && can_afford_light {
            hull_order(HullType::LimitedContactVehicle)
        } else {
            BuildOrder::Idle
        };

        if w_deepen >= w_expand && deepen_possible {
            // Prefer depth: upgrade if funded, else save toward it.
            if can_afford_infra {
                BuildOrder::UpgradeInfrastructure
            } else {
                survey_fallback
            }
        } else if let Some((order, _, _)) = outward {
            // Prefer expansion: build if funded, else save toward the vehicle.
            if can_expand {
                order
            } else {
                survey_fallback
            }
        } else if deepen_possible && can_afford_infra {
            BuildOrder::UpgradeInfrastructure
        } else {
            survey_fallback
        }
    }
}

/// A build order for `hull`, taking whichever class this policy names for it.
/// Classes are seeded by the roster (R-O42); until Design cards author more,
/// the two starting designs are the only named ones.
fn hull_order(hull: HullType) -> BuildOrder {
    let class = match hull {
        HullType::LimitedSystems => Class::Meadow,
        HullType::LimitedContactVehicle => Class::Tor,
        _ => Class::Unnamed,
    };
    BuildOrder::Hull { hull_type: hull, class }
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

    /// A remote-tier sighting: position plus K factors, nothing close-scan-only.
    fn survey_view(id: u32, pos: Vec3) -> SurveyView {
        SurveyView { id: PlanetId(id), position: pos, habitability: 1.0, biosphere: 1.0, industrial_signature: false }
    }

    /// The same, but radiating the waste heat of a pop-4 civilization.
    fn inhabited_survey_view(id: u32, pos: Vec3) -> SurveyView {
        SurveyView { industrial_signature: true, ..survey_view(id, pos) }
    }

    #[test]
    fn survey_prefers_nearest() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let cands = vec![survey_view(1, Vec3::new(100.0, 0.0, 0.0)), survey_view(2, Vec3::new(10.0, 0.0, 0.0))];
        assert_eq!(ap.choose_survey_target(&doctrine, Vec3::ZERO, None, &cands), Some(PlanetId(2)));
    }

    /// A center with a comfortably stocked frontier, so the survey branch stays
    /// out of the way of the deepen/expand cases these tests are about. Use
    /// [`prod_ctx_frontier`] to exercise survey itself.
    fn prod_ctx(level: u8, infra: f64, stockpile: f64) -> ProductionContext {
        prod_ctx_frontier(level, infra, stockpile, usize::MAX)
    }

    fn prod_ctx_frontier(level: u8, infra: f64, stockpile: f64, candidate_count: usize) -> ProductionContext {
        ProductionContext {
            center_pos: Vec3::ZERO,
            level,
            infra,
            k_potential: 4.0,
            stockpile_total: stockpile,
            medium_min_level: 3,
            limited_min_level: 2,
            infra_cost: infra + 1.0,
            colonizer_cost: 1.0,
            mining_pair_cost: 1.0,
            light_vehicle_cost: 0.25,
            candidate_count,
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
        assert!(matches!(order, BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }));
    }

    /// One `Candidate` a mature center would happily colonize.
    fn one_colony_candidate(ap: &BaselineAutopilot, doctrine: &Doctrine) -> Vec<Candidate> {
        let rctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(5, Vec3::new(10.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        let ranked = ap.rank(doctrine, &v, &rctx);
        vec![Candidate { view: v, ranked }]
    }

    #[test]
    fn survey_ignores_the_industrial_signature_by_default() {
        // R-SIM3: early game does no filtering. The nearest world is visibly
        // inhabited and the scout goes anyway — finding out costs a hop, and
        // that cost is the early game's to pay.
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        assert!(!doctrine.survey_avoids_inhabited, "filtering must be off until a card enables it");
        let cands =
            vec![inhabited_survey_view(1, Vec3::new(10.0, 0.0, 0.0)), survey_view(2, Vec3::new(100.0, 0.0, 0.0))];
        assert_eq!(ap.choose_survey_target(&doctrine, Vec3::ZERO, None, &cands), Some(PlanetId(1)));
    }

    #[test]
    fn survey_avoids_the_industrial_signature_once_doctrine_enables_it() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine { survey_avoids_inhabited: true, ..Doctrine::default() };
        let cands =
            vec![inhabited_survey_view(1, Vec3::new(10.0, 0.0, 0.0)), survey_view(2, Vec3::new(100.0, 0.0, 0.0))];
        assert_eq!(ap.choose_survey_target(&doctrine, Vec3::ZERO, None, &cands), Some(PlanetId(2)));
    }

    #[test]
    fn survey_still_flies_when_every_candidate_looks_inhabited() {
        // Filtering must never strand a scout with nothing to do; an inhabited
        // world is a worse target than an empty one, not worse than no target.
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine { survey_avoids_inhabited: true, ..Doctrine::default() };
        let cands = vec![inhabited_survey_view(1, Vec3::new(10.0, 0.0, 0.0))];
        assert_eq!(ap.choose_survey_target(&doctrine, Vec3::ZERO, None, &cands), None);
    }

    #[test]
    fn partial_headroom_below_the_ceiling_still_deepens() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // k_potential 2.86: a whole level does NOT fit above infra 2, but there
        // is real headroom. The old `infra + 1 <= k_potential` guard stranded
        // this center at K=2 forever, below the level-3 band edge (~2.675), so
        // it could never build anything and hoarded minerals it could not spend.
        let mut ctx = prod_ctx(2, 2.0, 5.0);
        ctx.k_potential = 2.86;
        assert_eq!(ap.production_choice(&doctrine, &ctx, &[]), BuildOrder::UpgradeInfrastructure);
    }

    #[test]
    fn limited_tier_builds_survey_when_capped_and_frontier_is_thin() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // Level 2 (limited tier), infra already at the ceiling so deepening is
        // impossible, minerals on hand, and a thin frontier. Before the limited
        // tier existed this returned Idle and the stockpile sat dead forever.
        let mut ctx = prod_ctx_frontier(2, 3.0, 5.0, 0);
        ctx.k_potential = 3.0;
        assert!(matches!(
            ap.production_choice(&doctrine, &ctx, &[]),
            BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, .. }
        ));
    }

    #[test]
    fn below_the_limited_tier_never_builds_survey() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let mut ctx = prod_ctx_frontier(1, 3.0, 5.0, 0);
        ctx.k_potential = 3.0; // capped, so deepening is off the table too
        assert_eq!(ap.production_choice(&doctrine, &ctx, &[]), BuildOrder::Idle);
    }

    #[test]
    fn survey_never_preempts_an_affordable_expansion() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // Thin frontier (0 < survey_reserve) AND an affordable colony target.
        // Survey must stay a fallback: an earlier revision gave it priority here,
        // which made `survey_reserve` non-monotonic — a large reserve had every
        // center scouting every cycle and colonies collapsed from 1047 to 3.
        let ctx = prod_ctx_frontier(3, 3.0, 2.0, 0);
        let cands = one_colony_candidate(&ap, &doctrine);
        assert!(matches!(
            ap.production_choice(&doctrine, &ctx, &cands),
            BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }
        ));
    }

    #[test]
    fn mature_center_surveys_when_it_cannot_afford_to_expand() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // Same thin frontier, but too poor for the colony ship and capped so it
        // cannot deepen either — the cycle would otherwise be pure Idle. A scout
        // is cheap enough to afford, so the idle capacity goes to survey.
        let mut ctx = prod_ctx_frontier(3, 3.0, 0.3, 0);
        ctx.k_potential = 3.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        assert!(matches!(
            ap.production_choice(&doctrine, &ctx, &cands),
            BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, .. }
        ));
    }

    #[test]
    fn survey_reserve_zero_restores_the_old_never_scout_behaviour() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine { survey_reserve: 0, ..Doctrine::default() };
        let mut ctx = prod_ctx_frontier(3, 3.0, 0.3, 0);
        ctx.k_potential = 3.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        assert_eq!(ap.production_choice(&doctrine, &ctx, &cands), BuildOrder::Idle);
    }
}
