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

use crate::cards::Order;
use crate::galaxy::{PlanetClass, PlanetId, PlayerId};
use crate::math::Vec3;
use crate::resources::{Basic, MineralField};
use crate::sim::{Class, HullType, Role};
use crate::units::{Band, BandTier, Kilotons, Measure, Price};

/// Which of the two cheap classes the colony pipeline reaches for first
/// (autopilot-doc §4; R-AC1 / R-A1). Default is production-centers-first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpandBias {
    ProductionCentersFirst,
    ColoniesFirst,
}

/// Whether survey craft favor a heading, and for how long (autopilot-doc §2,
/// R-AC3). All three variants share the same fallback rule when a heading
/// is biased: prefer the hemisphere, fall back to global nearest-unscanned
/// once it's exhausted (never strands a vehicle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurveyStrategy {
    /// No heading bias, ever — every survey craft, opening fan-out included,
    /// always picks the globally nearest unscanned world.
    GlobalPool,
    /// The opening fan-out (one craft per cube face) keeps its heading bias
    /// for its whole hop chain; every later, paid-for Scout pools globally.
    /// **Default — this is the shipped behavior predating this enum**, kept
    /// as the baseline so adding the knob changes nothing by itself.
    OpeningSectors,
    /// Like `OpeningSectors`, but every later Scout *also* gets a heading
    /// bias: outward from home through the production center that built it
    /// (`(center_pos - home_pos).normalized()`), so sector discipline
    /// persists as the empire scales rather than existing only at bootstrap.
    /// A center at home itself (bias `ZERO`) degrades to global pool for
    /// that one craft, which is the correct degenerate case.
    PersistentSectors,
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
    pub k_high: Band,
    /// mineral_value at/above which a low-K world is a mining outpost.
    pub mineral_high: f64,
    /// hub_value at/above which a high-K world is a production center.
    pub hub_high: Band,
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
            // Validated 4/4 test-bed seeds to 100% of colonizable worlds —
            // where "colonizable" is *this threshold's own* set, 51-53% of the
            // galaxy. That is the binding constraint on the T-20 coverage
            // objective, which counts every world with min(hab,bio) > 0.01:
            // the bed already colonizes 90-100% of what this admits
            // (`examples/reach_limit.rs`). R-AC18 asks whether the Colony
            // class should have a K floor at all.
            k_high: Band::new(3.2),
            mineral_high: 2.0,
            hub_high: Band::new(0.8),
            centrality_scale: 150.0,
            mineral_pressure_gain: 1.0,
        }
    }
}

/// **How a centre picks the hull for a coloniser (R-IND11).**
///
/// This was `Doctrine::colonizer_hull` until T-56 stage 4c derived the hull
/// instead, and T-67 reopened the question by taking infrastructure out of `K`:
/// both viable hulls now seed to the *world's* own ceiling, so the hold is the
/// only thing separating them and R-O76's measured answer no longer describes a
/// mechanism the engine has.
///
/// It is Doctrine rather than a constant because it is exactly what Doctrine is
/// for — policy over the roster — and because the answer is a Monte-Carlo
/// question. `examples/colonizer_policy` measures it — and has: see
/// [`ColonizerPolicy::SettlersPerMineral`] for why the answer it returns is not
/// yet a ratification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColonizerPolicy {
    /// **The cheapest hull that can found at all.** A deeper seed does not found
    /// another world: ten Mediums make ten colonies, each with its own `K` and
    /// its own growth curve, where one General makes one colony that starts
    /// further up a curve it would have climbed anyway.
    CheapestViable,
    /// **The best settlers-delivered per mineral.** A General hull costs 10x a
    /// Medium and its hold is 31.6x, so where the world can absorb the load it
    /// lands three times the people per mineral spent.
    ///
    /// **Measured, and do not ship it (`Hyades_industry.md` §1.6).** It scores
    /// **+13.97% colony-years on a bit-identical colony count** — the objective
    /// itself does not move — and the case made for it here was *transit*: a
    /// deep seed becomes a forward base sooner and shortens later voyages. That
    /// is refuted. Mean flight time is 107.6-116.7 yr in every measured arm and
    /// moves 4.0 yr, against a 398.8 yr shift in mean founding.
    ///
    /// Two ablations put the effect on **the seed mass alone**: force every
    /// seed to the General's hold and `CheapestViable` reproduces the entire
    /// gain **while building zero General hulls** (+14.35%); force every seed
    /// to the Medium's hold and this policy still buys General hulls for 91.9%
    /// of its colonisers, pays 10x each, and lands back at baseline (-0.08%).
    ///
    /// Which makes it an exploit rather than a strategy: **R-O74** — the
    /// settlers are conjured, with nothing debited from the founding centre —
    /// so what this measures is how much free mass a policy can extract from an
    /// open design-law-#11 violation. R-IND11 is blocked on that, not on the
    /// industrial ramp.
    SettlersPerMineral,
}

/// The standing behavior the sim executes "unasked" — every field a default a
/// card can override (sim §0a). Defaults are the round-1 baseline values
/// (six-vehicle survey, 1 g, +20% productivity step).
#[derive(Clone, Copy, Debug)]
pub struct Doctrine {
    /// Which hull a centre lays down for a coloniser (R-IND11, open).
    pub colonizer_policy: ColonizerPolicy,

    // --- Exploit / Growth (the build cycle, autopilot-doc §6) ---
    /// Base productivity step per build cycle. `0.20` is the doctrine value The
    /// Compass retunes (autopilot-doc §6, R-AC11).
    pub productivity_step: f64,
    /// Logistic growth rate `r` for population toward `K` per cycle.
    ///
    /// **MC-ratified at 0.873**, re-derived at the operating point the previous
    /// ratification produced (it had been 0.546, itself one leg of the joint
    /// step that took coverage 38.26% → 49.25%).
    ///
    /// *How this one was measured, in order:* `gradient_probe --screen`
    /// re-measured every knob **at the current defaults** — necessary because a
    /// gradient is local and the old direction had been consumed by the step
    /// that produced it. `growth_rate` came back the largest surviving lever at
    /// **+2.27 ± 0.50** (4.5 SE) and the only one clearly outside the noise
    /// besides `biosphere_regen_rate`. `gradient_step` then line-searched that
    /// direction **on the full objective**, and `--attribution 0.5` split the
    /// winning step into its parts on the same CRN seeds:
    ///
    /// | config | coverage | paired gain |
    /// |---|---|---|
    /// | baseline (0.546) | 49.11% | — |
    /// | **`growth_rate` alone → 0.873** | **51.42%** | **+2.31 ± 1.05** |
    /// | `biosphere_regen_rate` alone | 50.57% | +1.46 ± 0.68 |
    /// | both | 51.62% | +2.51 ± 1.13 |
    ///
    /// So this moved **alone**: it earns 92% of the joint gain, and the
    /// biosphere dial's marginal contribution on top of it is +0.20 points,
    /// far inside the noise. The two are substitutes rather than additive —
    /// they relieve the same bottleneck from opposite sides (one raises the
    /// `K` ceiling, the other the rate of approach to it) — so collecting both
    /// was never on offer. `SimConfig::biosphere_regen_rate` therefore stays at
    /// its R-O63 placeholder, which is a *design* dial (whether a razed ecology
    /// is a durable wound or a rounding error) and not the search's to spend.
    ///
    /// **⚠ There is a cliff above this value, and it is close.** The same line
    /// search: `r = 1.395` still works (50.99%), `r = 2.229` collapses to
    /// **28.46%** and `r = 3.563` to 19.82%. Population growth consumes
    /// biosphere 1:1 (L6), so a high enough `r` eats the ecology that caps `K`
    /// and the economy starves itself. 0.873 is deliberately ~2.5× below the
    /// cliff rather than at the noisy argmax — the measured optima at α = 0.25 /
    /// 0.50 / 1.00 (+2.32 / +2.51 / +1.88) are within noise of each other, so
    /// margin to the cliff is the tiebreaker, not the best mean.
    ///
    /// Guarded by `growth_rate_stays_clear_of_the_starvation_cliff`.
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
    /// pop-Band-IV world is almost certainly already held, so flying to it spends a
    /// hop to learn something the spectrometry already said.
    ///
    /// **Default `false`, deliberately** (R-SIM3). Early game there is no
    /// filtering at all: an empire that has not developed the instruments or the
    /// doctrine to act on remote signatures simply flies out and finds out, and
    /// those wasted hops are part of what early expansion costs. This is exactly
    /// the shape of a card — one field, standing behavior, flipped when the
    /// right tech or board state is reached (`Hyades_card_contract.md` §5).
    pub survey_avoids_inhabited: bool,
    /// Whether heading-bias discipline is global, opening-only, or persistent
    /// (R-AC3). See [`SurveyStrategy`].
    pub survey_strategy: SurveyStrategy,

    // --- Expand (autopilot-doc §4) ---
    pub expand_bias: ExpandBias,

    /// ~~**How many miner hulls open one outpost**~~ (`miners_per_outpost`,
    /// T-57) and ~~**what share of a deposit's veins to crew**~~
    /// (`miner_vein_fraction`, T-72) — **both retired at T-87. Crew size is not
    /// a policy any more; it falls out of mineral demand.**
    ///
    /// The two retired knobs were the same mistake twice: a number someone had
    /// to choose, against an objective that could not price it. `3` was
    /// ratified (+2.74% colony-years) under a law with no deposit term at all;
    /// `0.3` replaced it and measured monotone in the wrong direction — every
    /// crew size worse than the one below, on both seeds and both objectives.
    ///
    /// The mechanism behind that verdict is now in the model instead of in a
    /// footnote: **a deposit is a finite stock, so a crew can only bring its
    /// yield forward, and ore nobody can spend is a pure cost.** Sizing a crew
    /// without reference to demand buys hulls, freight and entity count to
    /// accelerate a resource the empire is not short of.
    ///
    /// `Simulation::mining_crew_for` derives the crew from what the founding
    /// centre can consume and cannot currently get — §4.3's extraction law
    /// inverted. See `Hyades_industry.md` §4.5. There is nothing here to tune.

    /// **Floor price per basic colour, `$`/kt** (politics §3.2, §10.4, T-84).
    ///
    /// The first term of `wtp`. Colours start equal — at first a kilotonne of
    /// Cyan is worth a kilotonne of Yellow — and diverge as the game develops,
    /// because **value is set by demand and demand is Doctrine**
    /// (`Hyades_industry.md` §7.1). This is the floor they diverge *from*.
    ///
    /// **Placeholder magnitudes** (R-P2). Nothing clears yet, so nothing can
    /// price them.
    pub base_value: [f64; 3],

    /// **How much this empire's policy wants each colour** (politics §3.2,
    /// §10.4, T-84) — the term that turns the map's mineral geography into a
    /// market.
    ///
    /// This is where the works mix enters the Exchange. An empire deep in
    /// Production is bidding on a Yellow-heavy works bill
    /// (`Hyades_industry.md` §6.10 — the default mix is already `3:2:1` Y:C:M),
    /// so its demand for Yellow is a **standing bid that moves the price of
    /// Yellow for everyone**, including empires that never touch the Production
    /// tree. §3.0's "value diverges from kilotons because demand is Doctrine",
    /// made mechanical.
    ///
    /// **Defaults to the works mix**, which is not arbitrary: the colours an
    /// empire wants to buy are the colours its bills are denominated in, and
    /// having two independent statements of that would be two copies of a rule
    /// that must agree with nothing checking that they do.
    pub doctrine_demand: [f64; 3],

    /// **Discount applied to a counterparty by reputation** (politics §3.5,
    /// §10.4). The fourth term of `wtp`. Inert until T-86 ships reputation.
    ///
    /// **Placeholder magnitude** (R-P2).
    pub risk_aversion: f64,

    /// **Expansion rate knob**: how strongly the production queue favors
    /// *upgrading own infrastructure* (deepening) over *spending minerals to
    /// reach outward* (expanding). `0.0` = always expand when able, `1.0` =
    /// always deepen.
    ///
    /// Since R-O68 the two sides are `rank` score per kilotonne committed, so
    /// this is an **odds ratio**: depth wins when `b/(1 − b) ≥ expand/deepen`.
    /// The crossover is state-dependent — a centre facing a cheap next rung and
    /// a mediocre candidate deepens where one facing an expensive rung and a hub
    /// does not — and on the shipped ladder it sits between **0.96 and 0.98**.
    ///
    /// **Stays at 0.5, and R-O87 is why it is not worth sweeping again.** The
    /// obvious objective for this knob is Growth's own — work-years,
    /// `∫ Σ_p infra_p dt` — and it does not move it, because the two things the
    /// knob chooses between are worth **exactly the same** to that metric:
    ///
    /// - deepening bills `infra_step_price / eta_works` and raises works by
    ///   `infra_step_price`, so works per mineral is `eta_works`;
    /// - founding bills the coloniser's price and the new colony's stock is
    ///   `founding_infra = hull_cost` — the recycled hull's minerals *are* the
    ///   stock (T-70) because a hull's mass is its cost (R-O57, design law #11) —
    ///   so works per mineral is **1**.
    ///
    /// At the card-free `eta_works = 1` those are identical to the last bit
    /// (`a_mineral_buys_the_same_works_whether_it_deepens_or_founds`).
    ///
    /// **That identity is about stock and it is only half the argument.** On
    /// *flow* deepening wins: rung I → II is **+29.0% hull/yr for nine
    /// colonisers**, paid back in 62 years. It still does not reach the
    /// objective, for three reasons that are the engine's rather than the
    /// knob's — homeworlds start at rung II already, `slips` is pinned at 2 at
    /// every rung, and **build rate governs only ~19% of a centre's timeline**
    /// because a declined build waits out `cycle_years = 50` (T-88). See
    /// `Hyades_industry.md` §6.19a and `examples/founding_tree`.
    ///
    /// Measured accordingly (`examples/work_years`, 4,000 yr, 3 seats):
    /// `b = 0.972` — the best point a 1,500-year screen could find — scores
    /// **+2.33% ± 0.96 work-years on the standard four-seed bed, 4/4 seeds
    /// positive**, and **−1.70% ± 2.42 on four seeds it was not chosen
    /// against**. Pooled over all eight: **+0.32% ± 1.42, 0.22 SE, 5/8
    /// positive.** Flat, as the identity says it must be. Neighbouring values
    /// swing the same magnitude in both directions (0.968 is −0.20%, 0.975 is
    /// +2.24%), which is the signature of a chaotic reordering rather than a
    /// gradient.
    ///
    /// **`eta_works` is the lever this knob is not.** It divides the deepening
    /// bill and nothing else, so a Production card genuinely does make a mineral
    /// buy more works — which is the tie-break the baseline has no access to.
    /// Re-sweep this only once a card or a ladder change has broken the
    /// identity; the test above is what will say so.
    pub reinvest_bias: f64,

    // --- Ranking (autopilot-doc §3) ---
    pub rank: RankWeights,
}

impl Default for Doctrine {
    fn default() -> Self {
        Doctrine {
            // Unmeasured as of T-67; `CheapestViable` is the *incumbent*
            // behaviour, not a ratified answer. R-IND11 is the open question and
            // `examples/colonizer_policy` is the harness.
            colonizer_policy: ColonizerPolicy::CheapestViable,
            productivity_step: 0.20,
            // 0.873 — re-ratified at the operating point the previous step
            // produced; see the field doc for the attribution table and the
            // starvation cliff above ~1.4 that sets the safe margin.
            growth_rate: 0.873,
            biosphere_regen_bonus: 1.0,
            survey_vehicles: 6,
            survey_accel_g: 1.0,
            // 1024 — ratified with k_high above; survey must scale with the
            // empire or expansion outruns its own map. Monotone by construction
            // (survey is a fallback, never a pre-emption), so raising it is safe.
            survey_reserve: 1024,
            // Off until a card or board state turns it on — see the field doc.
            survey_avoids_inhabited: false,
            survey_strategy: SurveyStrategy::OpeningSectors,
            base_value: [1.0; 3],
            // The works mix, normalised — an empire wants to buy the colours its
            // bills are denominated in. `WORKS_MIX_DEFAULT` is in `Basic` order
            // and is `3:2:1` Yellow : Cyan : Magenta (`Hyades_industry.md` §6.10).
            doctrine_demand: crate::cards::WORKS_MIX_DEFAULT,
            risk_aversion: 0.0,
            expand_bias: ExpandBias::ProductionCentersFirst,
            // 0.5 — **held, not defaulted** (R-O87). Work-years is flat in this
            // knob wherever it does anything at all, and falls off a cliff above
            // ~0.98; see the field doc for the identity that makes it so.
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
    pub habitability: Band,
    /// The **pristine** biosphere ceiling, on the Band ladder — what this world
    /// can support, not what is standing on it today. The colonization
    /// decision is about the ceiling; the standing stock only sets how fast a
    /// colony fills toward it, and is not remotely legible anyway.
    pub biosphere: Band,
    pub minerals: MineralField,
    pub owner: Option<PlayerId>,
    pub pop_level: BandTier,
}

impl PlanetView {
    /// Ceiling infra can be built to (autopilot-doc §3).
    #[inline]
    pub fn k_potential(&self) -> Band {
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
    pub habitability: Band,
    /// The **pristine** biosphere ceiling, on the Band ladder (see
    /// [`PlanetView::biosphere`]).
    pub biosphere: Band,
    /// **Inferential tier** (R-SIM3): this world carries the waste-heat and
    /// atmospheric signature of a pop-Band-IV civilization, legible at interstellar
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
    pub fn k_potential(&self) -> Band {
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
    /// **What a coloniser would actually land here, per hull** — Medium first,
    /// then General, in kilotons of settlers (R-IND12).
    ///
    /// Since R-O74 was closed a hold is not a promise: settlers come out of the
    /// origin, and how many is a *policy* question answered per destination — it
    /// depends on that world's carrying capacity and on how long the voyage is,
    /// not on the hull alone. `sim::settler_target` is the one implementation of
    /// that rule, and this field is how its answer reaches the hull choice.
    ///
    /// It is precomputed per candidate rather than recomputed here on purpose.
    /// Deriving it a second time in the autopilot is how `mining_pair_cost` came
    /// to need a comment explaining that it must agree with what
    /// `apply_build_with` will spend — two copies of a rule that must not
    /// disagree, with nothing checking that they don't.
    pub settlers_by_hull: [Kilotons; 2],
    /// **How many miners this body would be crewed with** (T-87).
    ///
    /// Derived, not chosen: §4.3's extraction law inverted against what the
    /// founding centre can consume and cannot currently get
    /// (`Simulation::mining_crew_for`). It depends on the *pair* — which rock,
    /// and which centre is buying — so it cannot be a doctrine field and cannot
    /// be recomputed from the view alone. It is precomputed here for the same reason
    /// `settlers_by_hull` is: the crew sets the pair's *price*, and a price the
    /// decision reads that differs from the price the build charges is how a
    /// centre ends up sitting Idle next to hulls it can afford.
    pub mining_crew: usize,
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
    /// Current development rung of the center.
    pub level: BandTier,
    /// Current infrastructure value.
    pub infra: f64,
    /// `min(hab, bio)` — the ceiling infrastructure can be built to.
    pub k_potential: f64,
    /// Minerals on hand at this center (the spendable pool).
    pub stockpile_total: Price,
    /// Minimum level required to build "medium" vehicles (colony/mining). Per the
    /// production schedule this is **3** (2 = limited, 3 = medium/rapid, 4 = all).
    pub medium_min_level: BandTier,
    /// Minimum level required to build "limited" vehicles — the Scout/LCV. The
    /// same schedule puts this at **2**, one tier below expansion.
    pub limited_min_level: BandTier,
    /// Mineral cost to raise infra by one level (= the target level). The
    /// **total**, kept for magnitude comparisons; affordability is per colour.
    pub infra_cost: Price,
    /// **The works bill for the next rung, split by colour** — Cyan, Magenta,
    /// Yellow (T-73, `Hyades_industry.md` §5.1).
    ///
    /// A work is payable *in named colours*, not out of a total, which is how
    /// the galaxy's mineral distribution finally bites on development rather
    /// than only on card costs. So the deepen branch cannot ask
    /// `stockpile_total >= infra_cost` any more: a centre with plenty of ore and
    /// none of the colour the bill names cannot buy the rung.
    pub infra_bill: [Price; 3],
    /// This centre's bank, by colour, in the same order — the other half of that
    /// comparison.
    pub stockpile_by_colour: [Price; 3],
    /// Mineral cost of a Colonizer on the **Medium** hull —
    /// `Hyades_vehicle_roles.md` §6's 1 CMY = 1 fleet model, not a flat
    /// placeholder anymore.
    pub colonizer_cost: Price,
    /// Mineral cost of a Colonizer on the **General** hull, for the errands a
    /// Medium hull's hold cannot cover (T-56 stage 4).
    pub general_colonizer_cost: Price,
    /// The founding population a **Medium** hull can deliver — `Band I` at the
    /// ratified ladder. A colony errand takes the Medium hull unless the target
    /// needs more than this.
    pub medium_seed_capacity: Kilotons,
    /// The founding population a **General** hull can deliver — `Band II`.
    pub general_seed_capacity: Kilotons,
    /// The infrastructure a **Medium**-hulled colony is founded at — the
    /// recycled hull, converted at the infra ladder's rate (R-O76). `Band I`.
    pub medium_founding_infra: Band,
    /// The infrastructure a **General**-hulled colony is founded at. `Band IV`
    /// at the ratified ladder: a General hull costs ten Medium hulls and the
    /// infra ladder charges `1+2+3+4 = 10` to reach the top playable rung.
    pub general_founding_infra: Band,
    /// Mineral cost of a Miner + its paired Freighter (an LSV + an MSV),
    /// bundled since they're built together (§4.4).
    pub mining_pair_cost: Price,
    /// Mineral cost of one Scout (an LCV) — the survey craft the limited tier
    /// unlocks. Bootstrap hands each seat `survey_vehicles` of these free
    /// (autopilot-doc §2); every later one is paid for out of a center's
    /// stockpile like any other build.
    pub light_vehicle_cost: Price,
    /// Known, unclaimed, non-Barren worlds this empire could still expand to.
    /// The autopilot builds survey craft to keep this above
    /// [`Doctrine::survey_reserve`] — expansion consumes candidates, so without
    /// replenishment the empire runs out of places to go long before it runs
    /// out of galaxy.
    pub candidate_count: usize,
    /// **Worlds no survey craft has been dispatched to yet** — the frontier a
    /// new scout could actually be *pointed at*.
    ///
    /// Distinct from [`Self::candidate_count`], and the distinction is
    /// load-bearing. `candidate_count` is *known and still available*: it falls
    /// to zero when everything scanned is owned or already targeted, which
    /// happens constantly in a colonised galaxy and says nothing about whether
    /// exploring would help. This is *unexplored*, and it is the only honest
    /// precondition for building a survey craft — at zero, `launch_survey` has
    /// nothing to pick and the hull flies nowhere.
    ///
    /// Counted off a running total rather than a walk, because a production
    /// decision reads it (`CLAUDE.md` §4: per-decision work must be `O(what the
    /// decision reads)`).
    pub survey_frontier: usize,
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

    /// **What card to play at this round barrier**, or `None` to pass.
    ///
    /// The card layer's policy seam (`Hyades_netcode.md` §5, cards §). Default
    /// is `None` — **passing every round** — which is deliberate: the baseline
    /// autopilot's behaviour must not change just because the round layer
    /// exists, or every coverage number in the tree moves at once and the
    /// offline search is invalidated. A policy that plays cards is a new
    /// `Autopilot`, not an edit to this one.
    ///
    /// Receives no world state, only doctrine and the round index: card choice
    /// is subject to the same one-directional seam as every other decision
    /// (design law #15), so a policy that wants board state must be handed a
    /// view, never `&Simulation`.
    fn choose_card(&self, _doctrine: &Doctrine, _seat: PlayerId, _round: u32) -> Option<Order> {
        None
    }

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
        // A scarcity-weighted **score** over the three Band readings, not a sum
        // of quantities — the readings are taken explicitly (`.bands()`) for
        // the same reason `hub_value` does: weights carry the units. Summing
        // the *masses* would be a different question (how much ore is here),
        // and `MineralField::total_mass` answers that one.
        let base_mineral =
            Basic::ALL.iter().zip(ctx.scarcity.iter()).map(|(&b, w)| w * m.get(b).in_bands().bands()).sum::<f64>();
        let mineral_value = base_mineral * (1.0 + w.mineral_pressure_gain * ctx.mineral_pressure);

        // hub_value: high-K worlds near the empire's centre of mass are hubs.
        let dist = view.position.distance(ctx.holdings_centroid);
        let centrality = (-dist / w.centrality_scale).exp();
        // `Band` has no `Mul` since the units fix, and rightly: scaling a
        // position on a log ladder is not scaling a quantity. This is not a
        // quantity — it is a **classification score**, a `k_potential` reading
        // discounted by distance and compared against another reading
        // (`hub_high`). Constructed explicitly so the discount is visibly on
        // the log-scale *reading*, not on the stuff it stands for. Same number
        // as before the type change.
        let hub_value = Band::new(k_potential.bands() * centrality);

        // The score is a weighted comparison across incommensurate things —
        // a Band, a mineral density, a hub figure — so the Band readings are
        // taken explicitly here rather than the weights pretending to be
        // dimensionless. Weights carry the units; that is what they are for.
        let score = w.w_k * k_potential.bands() + w.w_mineral * mineral_value + w.w_hub * hub_value.bands();

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

            // A Systems hull above the Limited tier settles, preferring
            // whichever class doctrine leads with — the same order
            // `production_choice` weighed. The General hull joins the Medium
            // here because T-56 stage 4 lets doctrine order one; without this
            // arm a General colonizer would be built and then find no mission,
            // and `apply_build` would refund nothing.
            HullType::MediumSystems | HullType::GeneralSystems => {
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
        // The epsilon is a price too — the whole comparison is on one ladder.
        let eps = Price::new(1e-9);
        // **Every colour, not the total** (T-73). `works_bill` in `sim` produces
        // both sides of this and the build spends against the same function, so
        // the decision and the purchase cannot drift — the failure
        // `mining_pair_cost` carries a comment about and `settlers_by_hull` was
        // written to end.
        let can_afford_infra = (0..3).all(|i| ctx.stockpile_by_colour[i] + eps >= ctx.infra_bill[i]);

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
        // **Never build a survey craft when there is nothing left to survey.**
        // `survey_frontier` is the set `launch_survey` picks from, so at zero the
        // hull is built, tasked `Scout`, and flies nowhere — a pure cost, and on
        // a fully-explored bed it is *most* of what the policy builds.
        //
        // The second term is the actual reserve test, and measurement says it is
        // inert at the shipped value (R-O86). `candidate_count` is the count of
        // known, unclaimed, non-Barren worlds; measured on seed 1
        // (`examples/survey_timing`, 600 planets / 1,500 yr) its **median is 0**
        // and its maximum over the whole run is **164**, against a ratified
        // `survey_reserve` of **1024**. So the comparison is a constant `true`
        // and every value above ~200 is bit-identical. That also explains the
        // plateau `CLAUDE.md` §2 records as a measurement artifact — 2048 reads
        // as noise, 512 / 256 / 64 fall off a cliff — as a threshold sitting
        // above the whole range of the thing it thresholds.
        let wants_survey = ctx.survey_frontier > 0 && ctx.candidate_count < doctrine.survey_reserve;
        let can_afford_light = ctx.stockpile_total + Price::new(1e-9) >= ctx.light_vehicle_cost;

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
        // ever restart expansion — **if there is anything left to survey.**
        //
        // ~~"no candidates means every other branch below returns Idle."~~ That
        // justification was false, and it was load-bearing: the branch it
        // pre-empts is the `outward == None` deepen fallback, which is the only
        // deepening path above `medium_min_level` that actually runs. So a
        // centre with an empty frontier, a full bank and three Bands of headroom
        // built a scout, every time, forever.
        //
        // And `candidates.is_empty()` is not the exploration question. It goes to
        // zero the moment everything *scanned* is owned or targeted, which in a
        // colonised galaxy is the common case — median `candidate_count` is **0**
        // on the standard bed. `survey_frontier` is the honest precondition:
        // worlds no craft has been dispatched to.
        //
        // Measured (`examples/deepen_census`, 600 planets / 1,500 yr), with the
        // gate ablated so deepening pre-empts the scout instead: infrastructure
        // builds **88 → 550** on seed 1 and **83 → 502** on seed 7, mean infra
        // Band 1.028 → 1.167 and 1.026 → 1.151, with colonies and colony-years
        // **up on both seeds** (3,309 → 3,314 / 2,540,752.7 → 2,544,150.4;
        // 3,334 → 3,336 / 2,608,344.6 → 2,609,993.2). Strictly better, which is
        // what a wasted build should look like when it stops.
        if candidates.is_empty() && can_afford_light && ctx.survey_frontier > 0 {
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
            (Some(col), _) => {
                // **Carry up to the target's carrying capacity, and no more —
                // then take the smallest hull that earns its price.**
                //
                // Population above `K` does not settle back to it, it crashes
                // below it: a `Band II` seed on a `Band I` colony ends its first
                // tick at 0.25 Bands, and forcing that cost 5.9% of colony-years
                // with the colony *count* unchanged on every seed. So the load a
                // ship flies is `min(hull capacity, K)`.
                //
                // **Since T-67 the founding `K` does not depend on the hull at
                // all.** Infrastructure left the carrying-capacity minimum
                // (`Hyades_industry.md` §1.1), so a colony seeds to the
                // *world's* ceiling whoever founded it. R-O76's criterion —
                // "founding `K` per mineral" — was reading
                // `k_pot.min(founding_infra)`, and that term no longer
                // describes anything the engine does. Left in place it would be
                // the exact defect this project keeps recording: a score whose
                // inputs stopped meaning what the score says they mean.
                //
                // **The replacement is the cheapest hull that can found at
                // all**, and the reason is the objective rather than the
                // physics. "Settlers delivered per mineral" is the tempting
                // reading — a General hull costs 10x a Medium and its hold is
                // 31.6x, so per mineral it lands three times the people — and
                // it is the wrong question: **a deeper seed does not found
                // another world.** Ten Mediums make ten colonies, each with its
                // own `K` and its own growth curve; one General makes one
                // colony that starts further up a curve it would have climbed
                // anyway. Against a colony-count objective the cheap hull wins,
                // which is R-O76's *finding* surviving even though R-O76's
                // mechanism did not.
                //
                // **R-IND11 (open):** whether a General coloniser is ever worth
                // it now that the hold is the only thing distinguishing the
                // hulls. It cannot be answered yet — it turns on the industrial
                // ramp (`Hyades_industry.md` §3), where a General hull is a
                // twelve-year yard commitment against a Medium's three, and on
                // whether a deep seed reaches a rate threshold sooner. Both are
                // stages that have not landed.
                // **What a hull delivers is the least of three things**, since
                // R-O74 was closed: its hold, the target world's ceiling, and
                // what this centre can actually spare. The third is new — a
                // hold used to be a promise because the settlers were conjured
                // — and leaving it out would price a General hull for people
                // the centre does not have.
                // **What a hull delivers is decided per destination, not per
                // hull** (R-IND12). `settlers_by_hull` is `sim::settler_target`
                // evaluated for this candidate — the world's own ceiling, this
                // centre's population, and the transit discount, already folded
                // in. There is nothing left to `min` against here, and doing so
                // would be the second copy of a rule this repo has been bitten
                // by before.
                let per_mineral = |delivered: Kilotons, cost: Price| {
                    if cost > Price::ZERO {
                        delivered.kilotons() / cost.kilotons()
                    } else {
                        f64::INFINITY
                    }
                };
                let options = [
                    (HullType::MediumSystems, ctx.colonizer_cost, col.settlers_by_hull[0]),
                    (HullType::GeneralSystems, ctx.general_colonizer_cost, col.settlers_by_hull[1]),
                ];
                // **Only hulls this centre can pay for today.** The score picks
                // between real options; it does not pick an option and then
                // discover it is unaffordable.
                //
                // That was the bug: the best `K` per mineral was chosen against
                // the whole ladder, and `can_expand` below then refused it — so
                // a young colony that could afford a Medium colonizer *now*
                // picked a General it could not afford and built **nothing**,
                // banking indefinitely. `ColonizerHull::GeneralWhenAffordable`
                // used to carry this rule explicitly; deriving the hull dropped
                // it, and nothing noticed because the shipped ladder happened
                // to make Medium the answer anyway.
                let affordable = |c: Price| ctx.stockpile_total + Price::new(1e-9) >= c;
                let best = options
                    .iter()
                    .filter(|(_, cost, delivered)| affordable(*cost) && *delivered > Kilotons::ZERO)
                    .max_by(|a, b| {
                        // Cheapest-wins is the negated price, so both policies
                        // are a `max` over one key and the comparator stays one
                        // expression.
                        let key = |o: &&(HullType, Price, Kilotons)| match doctrine.colonizer_policy {
                            ColonizerPolicy::CheapestViable => -o.1.kilotons(),
                            ColonizerPolicy::SettlersPerMineral => per_mineral(o.2, o.1),
                        };
                        key(a).partial_cmp(&key(b)).unwrap_or(core::cmp::Ordering::Equal).then(a.0.cmp(&b.0))
                    })
                    // Nothing affordable: name the cheapest that could found at
                    // all, so `can_expand` refuses it and the centre saves
                    // toward something real rather than toward nothing.
                    .or_else(|| options.iter().find(|(_, _, delivered)| *delivered > Kilotons::ZERO));
                best.map(|&(hull, cost, _)| (hull_order(hull), col.ranked.score, cost))
            }
            (None, Some(mine)) => Some((hull_order(HullType::LimitedSystems), mine.ranked.score, ctx.mining_pair_cost)),
            (None, None) => None,
        };
        let outward_cost = outward.map(|(_, _, c)| c).unwrap_or(Price::ZERO);
        let can_expand = ctx.stockpile_total + Price::new(1e-9) >= outward_cost;

        // **Deepen-vs-expand as a return on the minerals it costs (R-O68, T-51).**
        //
        // ~~`b · deepen_headroom >= (1 − b) · score`.~~ Those two sides were not
        // in the same unit. The left was a *Band* difference, `k_potential −
        // infra`, bounded by 4. The right was `rank`'s weighted sum over a Band,
        // a mineral density and a hub figure — unbounded, and with no price in
        // it at all. So `reinvest_bias` was not a convex trade but a unit
        // conversion with a preference hidden inside it: measured on seed 1
        // (`examples/score_scale`), headroom ran to a mean 2.55 against colony
        // scores of p05 4.40 / median 6.17 / max 12.16 — and the branch compares
        // against the **max**, because `outward` takes the best candidate. It
        // could not fire at the shipped `0.5` while any candidate existed.
        //
        // **Both sides are now `rank` score per kilotonne committed**, and
        // neither half introduces a constant:
        //
        // ```text
        // expand = score / outward_cost                 // this candidate, at its price
        // deepen = w_k · min(1, headroom) / infra_cost  // one rung, at its price
        // ```
        //
        // `w_k` is the weight `rank` already puts on one Band of `k_potential`
        // (autopilot-doc §3), and it is the right converter because the two
        // moves trade in one commodity: expansion **acquires** a world's Bands
        // of ceiling, deepening **realises** a Band of them here. The `min(1, ·)`
        // is what a rung actually delivers — `apply_build` steps to the next
        // whole rung whatever the headroom, so a last partial step pays a full
        // price for less than a Band.
        //
        // The comparison is then an odds ratio — depth wins when
        // `b/(1 − b) >= expand/deepen` — and that crossover is **state-
        // dependent**, which is the graded region the old form had nowhere: a
        // centre facing a cheap next rung and a mediocre candidate deepens where
        // one facing an expensive rung and a hub does not.
        //
        // **What it does not do is revive the branch at the shipped defaults,
        // and that is the finding rather than a shortfall.** The infra ladder
        // charges 0.9 kt for the rung above the founding one where a Medium
        // coloniser costs 0.1 kt, so expansion buys tens of times the score per
        // kilotonne and *ought* to win: the dead branch was the right answer
        // reached for a wrong reason.
        //
        // Measured (`examples/deepen_census`, 600 planets / 1,500 yr, seeds 1
        // and 7): the run is **bit-identical to the old form** everywhere below
        // `b = 0.96` — same build mix, same colony count, same colony-years to
        // the decimal — so this is a units fix and not a behaviour change. What
        // moved is the far end. The old form's cliff sat between 0.5 and 0.9
        // with **nothing working beyond it** (seed 1 `b = 0.9`: 70 colonies;
        // seed 7 `b = 0.95`: 3, i.e. the homeworlds alone). The new crossover is
        // between **0.96 and 0.98**, and 0.97 and 0.98 are working empires that
        // deepen — 326 infra builds against 88, mean infra Band 1.028 → 1.099,
        // colony-years −0.24%. So the odds ratio at the crossover is 24–49,
        // which is the price ladder's own ratio measured from the other side.
        //
        // **R-O85** carries what that exposes, and it is not a tuning question:
        // infrastructure is priced as if it were scarce on a bed where minerals
        // are the thing piling up unspent, and `fabrication_rate` saturates by
        // the second rung, so the rungs above it cost 19 kt and 780 kt to buy
        // almost no throughput.
        let b = doctrine.reinvest_bias;
        let deepen_headroom = (ctx.k_potential - ctx.infra).max(0.0);
        // Floored rather than branched on zero: `0.0 * f64::INFINITY` is `NaN`,
        // and a NaN reaching replicated state is fatal (design law #16). Every
        // price in the engine is positive, so the floor is unreachable in play
        // and exists only to keep `b = 0` arithmetic.
        let per_kt = |value: f64, cost: Price| value / cost.kilotons().max(1e-12);
        let w_deepen = if deepen_possible {
            b * per_kt(doctrine.rank.w_k * deepen_headroom.min(1.0), ctx.infra_cost)
        } else {
            f64::NEG_INFINITY
        };
        let w_expand = match outward {
            Some((_, score, cost)) => (1.0 - b) * per_kt(score, cost),
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
    Ranked::score_then_id(&a.ranked, &b.ranked)
}

impl Ranked {
    /// **The one ordering on candidates**, exposed because the engine reduces
    /// the candidate list to its per-class maxima before the policy ever sees
    /// it (R-O70) — and a reduction that used a different comparator than the
    /// policy would silently pick a different winner.
    ///
    /// Higher score wins, ties broken by planet id. Ids are unique, so this is
    /// a total order and the maximum is unique: taking the max of the per-class
    /// maxima is exactly taking the max of the whole list, whatever order the
    /// scan visited them in.
    #[inline]
    pub fn score_then_id(a: &Ranked, b: &Ranked) -> core::cmp::Ordering {
        a.score.partial_cmp(&b.score).unwrap_or(core::cmp::Ordering::Equal).then(a.id.0.cmp(&b.id.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::MineralField;

    fn view(id: u32, pos: Vec3, hab: f64, bio: f64, minerals: MineralField) -> PlanetView {
        PlanetView {
            id: PlanetId(id),
            position: pos,
            habitability: Band::new(hab),
            biosphere: Band::new(bio),
            minerals,
            owner: None,
            pop_level: BandTier::Empty,
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
            MineralField {
                cyan: Band::new(3.0).in_kilotons().kilotons(),
                magenta: Band::new(0.5).in_kilotons().kilotons(),
                yellow: Band::new(0.2).in_kilotons().kilotons(),
            },
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
        SurveyView {
            id: PlanetId(id),
            position: pos,
            habitability: Band::new(1.0),
            biosphere: Band::new(1.0),
            industrial_signature: false,
        }
    }

    /// The same, but radiating the waste heat of a pop-Band-IV civilization.
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
    fn prod_ctx(level: BandTier, infra: f64, stockpile: f64) -> ProductionContext {
        prod_ctx_frontier(level, infra, stockpile, usize::MAX)
    }

    fn prod_ctx_frontier(level: BandTier, infra: f64, stockpile: f64, candidate_count: usize) -> ProductionContext {
        ProductionContext {
            center_pos: Vec3::ZERO,
            level,
            infra,
            k_potential: 4.0,
            stockpile_total: Price::new(stockpile),
            medium_min_level: BandTier::III,
            limited_min_level: BandTier::II,
            infra_cost: Price::new(infra + 1.0),
            // Even thirds against a bank of even thirds: these cases are about
            // the deepen/expand branch, not about colour scarcity, and
            // `a_colour_poor_centre_cannot_buy_the_rung` covers that
            // deliberately.
            infra_bill: [Price::new((infra + 1.0) / 3.0); 3],
            stockpile_by_colour: [Price::new(stockpile / 3.0); 3],
            colonizer_cost: Price::new(1.0),
            general_colonizer_cost: Price::new(10.0),
            medium_seed_capacity: Kilotons::at_tier(BandTier::I),
            general_seed_capacity: Kilotons::at_tier(BandTier::II),
            medium_founding_infra: BandTier::I.band(),
            general_founding_infra: BandTier::IV.band(),
            mining_pair_cost: Price::new(1.0),
            light_vehicle_cost: Price::new(0.25),
            candidate_count,
            // An unexplored galaxy, so the survey gate is open and these cases
            // exercise the branch they are about. `a_fully_explored_empire_deepens_instead_of_scouting`
            // is the one that closes it.
            survey_frontier: 1,
        }
    }

    #[test]
    fn below_medium_gate_deepens_infrastructure() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // level 2, can afford the 3-mineral upgrade, no candidates yet.
        let order = ap.production_choice(&doctrine, &prod_ctx(BandTier::II, 2.0, 5.0), &[]);
        assert_eq!(order, BuildOrder::UpgradeInfrastructure);
    }

    #[test]
    fn below_gate_with_no_minerals_idles() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let order = ap.production_choice(&doctrine, &prod_ctx(BandTier::II, 2.0, 0.0), &[]);
        assert_eq!(order, BuildOrder::Idle);
    }

    #[test]
    fn mature_center_expands_to_a_colony_when_affordable() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let ctx = prod_ctx(BandTier::III, 3.0, 2.0);
        let rctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(5, Vec3::new(10.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        let ranked = ap.rank(&doctrine, &v, &rctx);
        let cands = vec![Candidate {
            view: v,
            ranked,
            settlers_by_hull: [Kilotons::at_tier(BandTier::I), Kilotons::at_tier(BandTier::II)],
            mining_crew: 1,
        }];
        let order = ap.production_choice(&doctrine, &ctx, &cands);
        assert!(matches!(order, BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }));
    }

    /// One `Candidate` a mature center would happily colonize.
    fn one_colony_candidate(ap: &BaselineAutopilot, doctrine: &Doctrine) -> Vec<Candidate> {
        let rctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let v = view(5, Vec3::new(10.0, 0.0, 0.0), 3.5, 3.5, MineralField::default());
        let ranked = ap.rank(doctrine, &v, &rctx);
        vec![Candidate {
            view: v,
            ranked,
            settlers_by_hull: [Kilotons::at_tier(BandTier::I), Kilotons::at_tier(BandTier::II)],
            mining_crew: 1,
        }]
    }

    /// **`reinvest_bias` is an odds ratio on two returns per kilotonne
    /// (R-O68, resolved).**
    ///
    /// `production_choice` picks depth when `b · deepen >= (1 − b) · expand`
    /// with both sides in `rank` score per kilotonne committed. So the crossover
    /// is a **price ratio**, `b* = expand / (expand + deepen)`, and it moves
    /// with the state rather than sitting at one global step: halve the rung's
    /// price and the crossover falls. That graded region is the thing the old
    /// form did not have anywhere, and it is what this pins.
    ///
    /// It also pins the sign of the shipped configuration, which the fix did
    /// **not** change: an infra rung costs several colonisers, so expansion wins
    /// at `b = 0.5` and the branch stays cold. That is now a statement about
    /// prices (R-O85) rather than about units.
    #[test]
    fn reinvest_bias_is_an_odds_ratio_on_two_returns_per_kiloton() {
        let ap = BaselineAutopilot::default();
        let mut doctrine = Doctrine::default();
        // A mature center with the most deepening headroom the ladder allows
        // (infra 1 against k_potential 4) and one ordinary colony candidate —
        // i.e. the case most favourable to depth that can actually occur.
        let mut ctx = prod_ctx(BandTier::III, 1.0, 100.0);
        ctx.k_potential = 4.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        let score = cands[0].ranked.score;

        // The crossover, written from the inequality itself.
        let crossover = |ctx: &ProductionContext| {
            let headroom = (ctx.k_potential - ctx.infra).clamp(0.0, 1.0);
            let deepen = doctrine.rank.w_k * headroom / ctx.infra_cost.kilotons();
            let expand = score / ctx.colonizer_cost.kilotons();
            expand / (expand + deepen)
        };

        // 1. It is a *dial over state*: a cheaper rung is a lower crossover, and
        //    the movement is continuous rather than a single global step.
        let dear = crossover(&ctx);
        let mut cheap_ctx = ctx;
        cheap_ctx.infra_cost = ctx.infra_cost / 8.0;
        cheap_ctx.infra_bill = [cheap_ctx.infra_cost / 3.0; 3];
        let cheap = crossover(&cheap_ctx);
        assert!(
            cheap < dear - 0.05,
            "crossover must track the rung price: {cheap:.3} at 1/8 the price against {dear:.3}"
        );

        // 2. The shipped bias still expands, because a rung costs more than a
        //    coloniser and buys less. Prices, not units.
        doctrine.reinvest_bias = 0.5;
        assert!(
            crossover(&ctx) > 0.5,
            "the shipped ladder must still favour expansion at b = 0.5 — if this has crossed, \
             re-read R-O85 before ratifying"
        );
        assert!(
            matches!(
                ap.production_choice(&doctrine, &ctx, &cands),
                BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }
            ),
            "at the shipped bias, a center with maximal headroom still expands"
        );

        // 3. Above the crossover the same state flips to depth — and the two
        //    crossovers differ, so there is a band of `b` where the cheap-rung
        //    centre deepens and the dear-rung one does not. That band is the
        //    graded region.
        doctrine.reinvest_bias = (dear + 1.0) / 2.0;
        assert!(
            matches!(ap.production_choice(&doctrine, &ctx, &cands), BuildOrder::UpgradeInfrastructure),
            "above the crossover the same state must flip to depth"
        );
        doctrine.reinvest_bias = (cheap + dear) / 2.0;
        assert!(
            matches!(ap.production_choice(&doctrine, &cheap_ctx, &cands), BuildOrder::UpgradeInfrastructure)
                && matches!(
                    ap.production_choice(&doctrine, &ctx, &cands),
                    BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }
                ),
            "between the two crossovers the decision must depend on the rung price, not only on b"
        );
    }

    /// **A fully-explored empire deepens instead of scouting (R-O86).**
    ///
    /// `candidates.is_empty()` used to send a centre straight to a survey hull,
    /// justified by "no candidates means every other branch below returns Idle."
    /// That is false: the branch it pre-empts is the `outward == None` deepen
    /// fallback, and that fallback is the only deepening path above
    /// `medium_min_level` that actually runs at the shipped `reinvest_bias`.
    ///
    /// So the two halves are pinned here. With frontier left, an empty candidate
    /// list still buys a scout — that is the mechanic restarting expansion, and
    /// it must not regress. With the galaxy explored, the same centre deepens.
    #[test]
    fn a_fully_explored_empire_deepens_instead_of_scouting() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        // Mature, funded, nothing known left to take.
        let mut ctx = prod_ctx(BandTier::III, 1.0, 100.0);
        ctx.candidate_count = 0;

        ctx.survey_frontier = 1;
        assert!(
            matches!(
                ap.production_choice(&doctrine, &ctx, &[]),
                BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, .. }
            ),
            "with galaxy left to explore, an empty candidate list must still buy a scout"
        );

        ctx.survey_frontier = 0;
        assert_eq!(
            ap.production_choice(&doctrine, &ctx, &[]),
            BuildOrder::UpgradeInfrastructure,
            "with nothing left to survey, the same centre must spend on depth rather than on a hull \
             that would be tasked Scout and fly nowhere"
        );

        // And it must not deepen past the ceiling just because survey is shut:
        // a capped centre with nothing to explore and nothing to take idles.
        ctx.infra = ctx.k_potential;
        assert_eq!(
            ap.production_choice(&doctrine, &ctx, &[]),
            BuildOrder::Idle,
            "a capped centre with no frontier and no candidates has nothing to buy"
        );
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
        let mut ctx = prod_ctx(BandTier::II, 2.0, 5.0);
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
        let mut ctx = prod_ctx_frontier(BandTier::II, 3.0, 5.0, 0);
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
        let mut ctx = prod_ctx_frontier(BandTier::I, 3.0, 5.0, 0);
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
        let ctx = prod_ctx_frontier(BandTier::III, 3.0, 2.0, 0);
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
        let mut ctx = prod_ctx_frontier(BandTier::III, 3.0, 0.3, 0);
        ctx.k_potential = 3.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        assert!(matches!(
            ap.production_choice(&doctrine, &ctx, &cands),
            BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, .. }
        ));
    }

    /// The ratified `growth_rate` must keep real margin below the starvation
    /// cliff the line search found. Population growth consumes biosphere 1:1
    /// (L6), so a high enough `r` eats the ecology that caps `K` and the
    /// economy starves: measured on the full objective, `r = 1.395` still
    /// works (50.99% coverage) but `r = 2.229` collapses to 28.46%.
    ///
    /// This is a **guard rail, not a tuning assertion** — it does not claim
    /// 0.873 is optimal (the measured optima at alpha 0.25/0.50/1.00 are within
    /// noise of each other). It claims only that a future retune cannot drift
    /// the default up against the cliff without someone deciding to.
    #[test]
    fn growth_rate_stays_clear_of_the_starvation_cliff() {
        let r = Doctrine::default().growth_rate;
        assert!(r > 0.0, "growth rate must be positive, got {r}");
        assert!(
            r <= 1.395,
            "growth_rate {r} is at or past the last value measured to still work (1.395); \
             beyond it coverage collapses (2.229 -> 28.46%). Re-run examples/gradient_step.rs \
             and re-ratify deliberately rather than drifting into the cliff."
        );
    }

    /// **Two ceilings on `growth_rate`, and since T-64 one of them is
    /// arithmetic rather than a measurement.**
    ///
    /// The step is now `x' = x + r·x·(1 − x/K)` on *people*. Substituting
    /// `v = (r/(1+r))·(x/K)` turns it into the logistic map `v' = μ·v·(1 − v)`
    /// with `μ = 1 + r` exactly, so the standard bifurcation picture applies:
    /// the fixed point at `K` is stable only for `μ < 3`, i.e. **`r < 2`**, and
    /// period-doubles above it (May, *Nature* 261:459–467, 1976).
    ///
    /// **The engine's `clamp` hides that from above, and that is the trap.** A
    /// population climbing from below overshoots `K`, gets clamped exactly to
    /// it, and the growth term is then zero — so a too-large `r` does not
    /// oscillate visibly, it *jumps*: the whole logistic collapses into a step
    /// function that reaches `K` in one cycle. A sweep would score that
    /// beautifully. It is still wrong, because growth is supposed to be a rate,
    /// and because a population arriving from *above* `K` — which is what a
    /// colony ship seeding over capacity does — gets the unclamped dynamics
    /// and the full crash (`a_colony_seeded_above_its_capacity_crashes_below_it`).
    ///
    /// So this asserts the thing the clamp cannot fake: the approach to `K`
    /// takes several cycles.
    #[test]
    fn the_population_logistic_is_a_rate_and_not_a_step() {
        // The engine's expression, verbatim.
        let steps_to_capacity = |r: f64| {
            let k = 100.0f64;
            let mut x = 1.0f64;
            for n in 1..1000 {
                x = (x + r * x * (1.0 - x / k)).clamp(0.0, k);
                if x >= 0.99 * k {
                    return n;
                }
            }
            1000
        };
        let r = Doctrine::default().growth_rate;
        let n = steps_to_capacity(r);
        assert!(n >= 4, "growth_rate {r} fills a world in {n} cycles — that is a step, not a rate");
        assert!(n <= 60, "growth_rate {r} takes {n} cycles to fill a world; the economy would never start");

        // And the bare map, which is where the ceiling actually lives. Started
        // just below `K` so the trajectory stays in the basin: a hard crash to
        // zero is also a fixed point, and "settled at extinction" is not the
        // question being asked.
        let settles_near_capacity = |r: f64| {
            let k = 100.0f64;
            let mut x = 0.9 * k;
            for _ in 0..500 {
                x += r * x * (1.0 - x / k);
            }
            let a = x + r * x * (1.0 - x / k);
            (a - x).abs() < 1e-6
        };
        assert!(settles_near_capacity(r), "the shipped rate must settle at K, not orbit it");
        assert!(settles_near_capacity(1.9), "just under the bifurcation, still settles");
        assert!(!settles_near_capacity(2.1), "past r = 2 it must be seen to period-double");
        assert!(r < 2.0, "growth_rate must stay under the period-doubling bifurcation at r = 2");
    }

    #[test]
    fn survey_reserve_zero_restores_the_old_never_scout_behaviour() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine { survey_reserve: 0, ..Doctrine::default() };
        let mut ctx = prod_ctx_frontier(BandTier::III, 3.0, 0.3, 0);
        ctx.k_potential = 3.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        assert_eq!(ap.production_choice(&doctrine, &ctx, &cands), BuildOrder::Idle);
    }
}
