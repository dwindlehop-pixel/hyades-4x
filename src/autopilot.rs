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
use crate::combat::Loadout;
use crate::galaxy::{PlanetClass, PlanetId, PlayerId};
use crate::math::Vec3;
use crate::sim::{Class, HullType, Role};
use crate::units::{Band, BandTier, Kilotons, Price};

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

/// **How a center picks the hull for a colonizer (R-IND11).**
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
    /// of its colonizers, pays 10x each, and lands back at baseline (-0.08%).
    ///
    /// Which makes it an exploit rather than a strategy: **R-O74** — the
    /// settlers are conjured, with nothing debited from the founding center —
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
    /// Which hull a center lays down for a colonizer (R-IND11, open).
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
    /// **Is an empire you have no quarrel with a target?** (R-O27/T-11, T-111.)
    ///
    /// The first — and for now the only — diplomatic field on `Doctrine`. R-O27
    /// has stood open since the standing layer shipped because nobody specified
    /// the *list*; this adds the one field the design has a concrete consumer
    /// for and leaves the rest of the list open, rather than inventing a
    /// diplomatic model to hold it.
    ///
    /// **Default `false`, and that is the design's own default, not a safety
    /// catch.** `Hyades_warfare_tree.md` §7.2 specifies the first Warfare card
    /// as writing exactly this field — *"a Neutral empire is an Enemy empire"* —
    /// which only means anything if neutrality is where everyone starts. So the
    /// shipped galaxy is at peace and a card is what starts the shooting.
    ///
    /// **Hostility is not symmetric and is not negotiated.** If you set this and
    /// your neighbor does not, you attack and they are attacked; declining is
    /// not theirs to do. What they *can* do is outrun you, which is
    /// [`crate::belief::can_disengage`] and is decided on kinematics rather than
    /// on consent.
    pub engage_neutrals: bool,
    /// **Do colonizers hold the ground they did not take?** (T-112.)
    ///
    /// The second half of the first Warfare card (`Hyades_warfare_tree.md` §8),
    /// and the half that carries the card's actual intent. With this set, a
    /// colonizer is **not consumed when it founds** — it flies on to a nearby
    /// unclaimed world and sits there, and nobody else founds there while it
    /// does.
    ///
    /// **The product of a picket is a colony that does not happen**, which is
    /// the only thing in the engine whose output is a negative. That is what
    /// makes it a Warfare card rather than an Expansion one: the tree's
    /// objective is colony-years *relative to the table* (§0), so denying a
    /// neighbor a world scores exactly what founding one does.
    ///
    /// **What it costs the player who plays it** is the recycled hull: roles
    /// §4.2 turns a spent colonizer into the new colony's `Band I`
    /// infrastructure, and a picket keeps its minerals in the hull instead. So
    /// every colony this empire founds starts thinner, and that is the price of
    /// the denial rather than a side effect.
    ///
    /// Separate from [`Self::engage_neutrals`] on purpose, so the two arms can
    /// be ablated apart (`CLAUDE.md` §2's 2×2 rule). Denial and shooting are
    /// different mechanisms and the card carries both.
    pub picket_after_founding: bool,
    /// **How many worlds this empire tries to keep held** (T-113).
    ///
    /// Pickets built as a *purpose* rather than as a colonizer's afterlife, on
    /// the cheapest armed hull there is (`Role::Picket` → `LimitedOffensive`).
    /// This is the fix §8.6's arithmetic pointed at: denial bought with a
    /// colonizer loses under Warfare's objective by `ΔW = −k(1 − w_0A)`
    /// whatever the placement, because `Σ_j w_0j = 1`. Denial bought with a
    /// hull that costs a fraction of a colony does not.
    ///
    /// **A fallback, never a pre-emption**, for the reason
    /// [`Self::survey_reserve`] records: an earlier survey rule that *did*
    /// pre-empt made its own knob non-monotonic, because centers scouted every
    /// cycle and never colonized. Raising this converts idle cycles into
    /// pickets and can never starve expansion.
    ///
    /// The cost of that ordering is that the branch almost never runs — see
    /// [`Self::picket_claims_target`], which is the one state where a picket
    /// may go ahead of survey, and the measurement that says why it has to.
    ///
    /// **Placeholder magnitude, default 0** (R-WAR6) — the card sets it.
    pub picket_reserve: usize,
    /// **Claim the world this center is saving for** (T-113).
    ///
    /// [`Self::picket_reserve`] alone is a fallback behind survey, and measurement
    /// says that is a fallback that almost never fires: `wants_survey` is
    /// `survey_frontier > 0 && candidate_count < survey_reserve`, and R-O86
    /// measured the second term a constant `true` (median `candidate_count` 0,
    /// maximum 164, against a ratified reserve of 1024). So the picket branch is
    /// reachable only once the frontier is exhausted, which on the 800-year bed
    /// produced **100 pickets across eight seeds** — a supply too small for any
    /// placement or preference rule to select over.
    ///
    /// This opens the one other state where a cheap hull is the best thing a
    /// center can do with a cycle: it has **named an outward target and cannot
    /// pay for it yet**. Those cycles are already spent saving, the world is
    /// already the one this empire intends to settle, and a picket standing on
    /// it is what makes the voyage safe to have committed to.
    ///
    /// Self-limiting per target rather than by a global reserve: the branch is
    /// gated on the target *not already being held*, so a center lays down at
    /// most one picket per world it is saving for, and stops the moment that
    /// hull arrives. [`Self::picket_reserve`] still bounds the empire-wide
    /// stock on top of that.
    ///
    /// **Placeholder, default false** (R-WAR6) — measured as its own ablation
    /// arm in `examples/denial_census`, because it and [`Self::picket_reserve`]
    /// address one diagnosis and landing them together would make either
    /// unattributable (`CLAUDE.md` §2's 2×2 rule).
    pub picket_claims_target: bool,
    /// **The Limited Offensive hull scouts, in place of the Limited Contact
    /// Vehicle** (T-115).
    ///
    /// Survey is what finds worlds to hold — [`Self::picket_reserve`] records
    /// that a picket branch placed *ahead* of survey cost its own player 24.5%
    /// of its colonies — so an empire that wants ground held is already
    /// building the unarmed light hull in the slot where it could be building
    /// an armed one. This write says: build the armed one. A scout that arrives
    /// somewhere worth holding can hold it, and the survey cadence is untouched
    /// because the *branch* has not moved, only the hull it names.
    ///
    /// **The price was expected to be paid at the yard, and there is none**:
    /// under R-O57 cost *is* dry mass and `hull_dry_mass` reads the cost
    /// *tier*, which groups every Limited hull — so an LOU and an LCV are the
    /// same 0.020 kt object, and the write is inert in **price** until hull
    /// types carry differentiated cost (R-O64, R-L0). It is live in **speed**:
    /// the survey leg flies the hull's own drive (a flat `survey_accel_g` was
    /// removed on the author's ruling that the sim does not overwrite a
    /// Design), and an LCV's empty drive is 0.911 g against an LSV's 1.00.
    /// `Hyades_warfare_tree.md` §8.9.7.
    ///
    /// **A scout built this way carries `Class::Tor`**, which is how
    /// `assign_role` tells it from a picket built on the same hull. The class
    /// *is* the design (R-O28/R-O42b), and a class names one hull, so the
    /// armed survey Design (Tor, on the Contact hull) and the unarmed one
    /// (Spur, on the Systems hull) are two Designs.
    ///
    /// **Placeholder, default false** (R-WAR8) — the card sets it.
    pub scout_hull_offensive: bool,
    /// **A picket leaves station to head off a colony ship it can beat there**
    /// (T-115).
    ///
    /// §8.6 measured the standing picket's problem exactly: 493–793 hulls on
    /// station produced **3–12 diversions per run**, because both placement
    /// rules put them on ground nobody was racing for. A picket that waits is
    /// betting that a rival picks the world it happens to be standing on; a
    /// picket that moves is answering a launch it has actually seen.
    ///
    /// **It is a race under light-lag and it is allowed to lose.** The launch
    /// is observed at `depart + distance(origin, station)`, the flight from
    /// station to the contested world takes what it takes, and the interception
    /// is attempted only when the sum lands before the colony ship does. Every
    /// term is an existing quantity — no reaction time, no interception radius,
    /// and nothing the picket knows that light has not delivered.
    ///
    /// The ground it leaves is ground nobody was taking, which is what makes
    /// this cheap in a way the standing rule was not.
    ///
    /// **Placeholder, default false** (R-WAR8) — the card sets it.
    pub picket_intercepts: bool,
    /// **A picket stands at a rival's port and strikes the colony ships it
    /// launches** (T-123, R-WAR16, `Hyades_warfare_tree.md` §8.16).
    ///
    /// T-122 closed every way the first Warfare card reached `W` and found
    /// that the one fight it can win never happens at a *destination*: a colony
    /// ship picks one of thousands of worlds, a picket guesses from a bearing,
    /// and handing it the true destination still produced no fights. The
    /// *origin* is different in kind. Every colony ship launches from one of a
    /// rival's production centers, so a hull standing there meets it at zero
    /// range, with nothing to guess and no light-lag to lose to.
    ///
    /// **Placement reads only what light has delivered.** Each launch is seen
    /// by this empire's capital at `depart + distance(port, capital)`, and a
    /// blockader goes to the rival port this empire has *seen* launch the most
    /// that it does not already hold. `examples/launch_census` priced the
    /// reach: a seat launches from a mean of 213.6 origins, and its eight
    /// busiest carry 25.5% of launches in hindsight — an upper bound on what
    /// eight blockaders per rival could meet.
    ///
    /// Supply is [`Self::picket_reserve`]'s, and the build branch is unchanged:
    /// this write moves where a picket goes, not whether one is built.
    ///
    /// **Placeholder, default false** (R-WAR16) — the card sets it.
    pub picket_blockades: bool,
    /// **Build the blockade before expanding** (T-125). With this on, a center
    /// whose empire holds fewer than [`Self::picket_reserve`] pickets —
    /// counting hulls already flying to a port — and has a seen rival port to
    /// send one to builds a picket ahead of every other order.
    ///
    /// The fallback rule [`Self::picket_reserve`] records exists because a
    /// picket that pre-empted survey once cost its player 24.5% of its
    /// colonies while denying nothing. A blockader strikes, and the census in
    /// appendix §D.4 measured the fallback as the blockade's binding
    /// constraint: **no hull on station for the first century after the card**,
    /// through the rivals' fastest expansion. The reserve bounds what this can
    /// cost.
    ///
    /// **Placeholder, default false** (R-WAR18) — the card sets it.
    pub picket_first: bool,
    /// **The General colonizer is a Contact hull, not a Systems hull** (T-116).
    ///
    /// This is §8.2's Design write — *"switch the colony ship role from Systems
    /// vehicles to Contact vehicles"* — and it is a **substitution inside the
    /// option set, not a replacement of it**: the Medium Systems hull stays on
    /// the menu, so a center that cannot afford a General hull colonizes
    /// exactly as it did.
    ///
    /// Read off the ladder rather than asserted (`examples/hull_compare`), the
    /// trade is what §8 says it is:
    ///
    /// | | GSV | GCV |
    /// |---|---|---|
    /// | cost = dry mass | 1.3154 kt | **1.0995 kt** |
    /// | colony seed hold | 31.62 kt | **12.04 kt** |
    /// | settlers per kt of cost | 24.04 | **10.95** |
    /// | empty acceleration | 5.22 | 2.56 |
    ///
    /// So it is **cheaper and much worse at carrying people** — a Contact hull
    /// reserves 15% of its volume for what makes it armed, and carries a
    /// thicker shell to survive using it. Its settlers-per-mineral (10.95) is
    /// still above a Medium hull's (9.16), so it is not dominated; it is a
    /// middle rung the ladder did not have.
    ///
    /// **A colonizer built this way carries `Class::Unnamed`**, which is how
    /// `assign_role` tells it from a scout on the same hull — the same
    /// class-as-design rule [`Self::scout_hull_offensive`] uses, and for the
    /// same reason.
    ///
    /// **Placeholder, default false** (R-WAR4) — the card sets it.
    pub colonizer_general_contact: bool,
    /// **Share of a colonizer's mineral endowment erected as infrastructure on
    /// arrival**, rather than banked (T-113).
    ///
    /// A colonizer's hold is already one kiloton budget carrying settlers *and*
    /// minerals sized to the destination's build-out (R-O74,
    /// `Hyades_industry.md` §1.7). Those minerals land in the new colony's
    /// **stockpile** — and a colony whose hull did not recycle has no
    /// infrastructure to spend them with, because `employment_rate` returns
    /// exactly `0.0` at a stock of zero. The bank is full and the colony is
    /// stillborn (§8.7).
    ///
    /// This erects part of the hold *as* the stock instead. It is not a grant:
    /// the minerals were debited from the founding center at launch and
    /// infrastructure **is** the minerals standing in it (T-70, R-O57), so the
    /// conversion is an identity rather than a rate. What it buys is that a
    /// colonizer which keeps its hull can still found something that works.
    ///
    /// **Placeholder magnitude, default 0.0** (R-WAR6) — inert until a card
    /// sets it, so the shipped galaxy is untouched.
    pub founding_infra_share: f64,
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
    /// center can consume and cannot currently get — §4.3's extraction law
    /// inverted. See `Hyades_industry.md` §4.5. There is nothing here to tune.

    /// **Floor price per basic color, `$`/kt** (politics §3.2, §10.4, T-84).
    ///
    /// The first term of `wtp`. Colors start equal — at first a kilotonne of
    /// Cyan is worth a kilotonne of Yellow — and diverge as the game develops,
    /// because **value is set by demand and demand is Doctrine**
    /// (`Hyades_industry.md` §7.1). This is the floor they diverge *from*.
    ///
    /// **Placeholder magnitudes** (R-P2). Nothing clears yet, so nothing can
    /// price them.
    pub base_value: [f64; 3],

    /// **How much this empire's policy wants each color** (politics §3.2,
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
    /// **Defaults to the works mix**, which is not arbitrary: the colors an
    /// empire wants to buy are the colors its bills are denominated in, and
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
    /// The crossover is state-dependent — a center facing a cheap next rung and
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
    /// - founding bills the colonizer's price and the new colony's stock is
    ///   `founding_infra = hull_cost` — the recycled hull's minerals *are* the
    ///   stock (T-70) because a hull's mass is its cost (R-O57, design law #11) —
    ///   so works per mineral is **1**.
    ///
    /// At the card-free `eta_works = 1` those are identical to the last bit
    /// (`a_mineral_buys_the_same_works_whether_it_deepens_or_founds`).
    ///
    /// **That identity is about stock and it is only half the argument.** On
    /// *flow* deepening wins: rung I → II is **+29.0% hull/yr for nine
    /// colonizers**, paid back in 62 years. It still does not reach the
    /// objective, for three reasons that are the engine's rather than the
    /// knob's — homeworlds start at rung II already, `slips` is pinned at 2 at
    /// every rung, and **build rate governs only ~19% of a center's timeline**
    /// because a declined build waits out `cycle_years = 50` (T-88). See
    /// `Hyades_industry.md` §6.19a and `examples/founding_tree`.
    ///
    /// Measured accordingly (`examples/work_years`, 4,000 yr, 3 seats):
    /// `b = 0.972` — the best point a 1,500-year screen could find — scores
    /// **+2.33% ± 0.96 work-years on the standard four-seed bed, 4/4 seeds
    /// positive**, and **−1.70% ± 2.42 on four seeds it was not chosen
    /// against**. Pooled over all eight: **+0.32% ± 1.42, 0.22 SE, 5/8
    /// positive.** Flat, as the identity says it must be. Neighboring values
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
            // behavior, not a ratified answer. R-IND11 is the open question and
            // `examples/colonizer_policy` is the harness.
            colonizer_policy: ColonizerPolicy::CheapestViable,
            productivity_step: 0.20,
            // 0.873 — re-ratified at the operating point the previous step
            // produced; see the field doc for the attribution table and the
            // starvation cliff above ~1.4 that sets the safe margin.
            growth_rate: 0.873,
            biosphere_regen_bonus: 1.0,
            survey_vehicles: 6,
            // 1024 — ratified with k_high above; survey must scale with the
            // empire or expansion outruns its own map. Monotone by construction
            // (survey is a fallback, never a pre-emption), so raising it is safe.
            survey_reserve: 1024,
            // Off until a card or board state turns it on — see the field doc.
            survey_avoids_inhabited: false,
            // Peace is the default (see the field doc): a Warfare card is what
            // turns a neutral into an enemy.
            engage_neutrals: false,
            picket_after_founding: false,
            picket_reserve: 0,
            picket_claims_target: false,
            scout_hull_offensive: false,
            picket_intercepts: false,
            picket_blockades: false,
            picket_first: false,
            colonizer_general_contact: false,
            founding_infra_share: 0.0,
            survey_strategy: SurveyStrategy::OpeningSectors,
            base_value: [1.0; 3],
            // The works mix, normalized — an empire wants to buy the colors its
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
    /// **The three per-color Band readings, precomputed** (T-100). `rank`
    /// wants the *readings*, not the masses — `CLAUDE.md` §4's "hand a decision
    /// only the fields it reads" — and computing them here lets the engine
    /// memoize a conversion that was 136 M logarithms per run.
    ///
    /// **This replaces the `MineralField` the view used to carry.** Nothing in
    /// the seam read the masses; `rank` converted them and threw them away. A
    /// view is built 45.4 M times a run, so the copy was a memory-traffic tax
    /// on the hottest path in the engine for a field with no reader.
    pub mineral_bands: [f64; 3],
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
    /// Pop-4 is the only occupancy signal modeled today, because it is the one
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
    /// **This empire already has a picket standing on this world** (T-113).
    ///
    /// Held ground is ground a rival's colonizer turns back from, so settling it
    /// is the cheapest colony available: the race for it is already won. This
    /// is what turns a picket from *denial* — which §8.6 measured as a losing
    /// trade under Warfare's own objective — into **claiming**, where the hull
    /// reserves a place in the queue rather than spending a colony to deny one.
    ///
    /// Read off the engine's own picket index, so it is ground truth about
    /// *this* empire's own ships, not about a rival's — no light-lag question
    /// arises (design law #15).
    pub held_by_me: bool,
    /// **This empire has a picket already on its way here** (T-113).
    ///
    /// Distinct from [`Self::held_by_me`] because the two answer different
    /// questions. *Settling* held ground wants a hull that has **arrived** —
    /// an inbound picket denies nobody yet, and a colonizer sent on the
    /// strength of one is racing an empty world. *Claiming* wants to know
    /// whether a hull is already committed, so a center saving for a world
    /// lays down one picket for it rather than one per decision for the whole
    /// voyage.
    pub claim_inbound: bool,
    /// **What a colonizer would actually land here, per hull** — Medium first,
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
    /// founding center can consume and cannot currently get
    /// (`Simulation::mining_crew_for`). It depends on the *pair* — which rock,
    /// and which center is buying — so it cannot be a doctrine field and cannot
    /// be recomputed from the view alone. It is precomputed here for the same reason
    /// `settlers_by_hull` is: the crew sets the pair's *price*, and a price the
    /// decision reads that differs from the price the build charges is how a
    /// center ends up sitting Idle next to hulls it can afford.
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
    /// **total**, kept for magnitude comparisons; affordability is per color.
    pub infra_cost: Price,
    /// **The works bill for the next rung, split by color** — Cyan, Magenta,
    /// Yellow (T-73, `Hyades_industry.md` §5.1).
    ///
    /// A work is payable *in named colors*, not out of a total, which is how
    /// the galaxy's mineral distribution finally bites on development rather
    /// than only on card costs. So the deepen branch cannot ask
    /// `stockpile_total >= infra_cost` any more: a center with plenty of ore and
    /// none of the color the bill names cannot buy the rung.
    pub infra_bill: [Price; 3],
    /// This center's bank, by color, in the same order — the other half of that
    /// comparison.
    pub stockpile_by_color: [Price; 3],
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
    /// Mineral cost of one Scout — an **LSV** by default, and an LCV once the
    /// Warfare card arms the frontier (T-121). Bootstrap hands each seat
    /// `survey_vehicles` of these free (autopilot-doc §2); every later one is
    /// paid for out of a center's stockpile like any other build.
    pub light_vehicle_cost: Price,
    /// Mineral cost of one picket — a **Limited Contact Vehicle** (T-113,
    /// rebased at T-121).
    ///
    /// Numerically equal to [`Self::light_vehicle_cost`] whenever the scout is
    /// also a Limited hull, because `cost_fraction` gives the whole Limited
    /// tier one price. It is a separate field anyway: the equality is a
    /// property of today's cost ladder, not of the two roles, and R-O64/R-L0
    /// are open on giving hull types differentiated cost.
    pub picket_cost: Price,
    /// How many worlds this empire is already holding, against
    /// [`Doctrine::picket_reserve`].
    pub pickets_held: usize,
    /// Whether this empire has a seen rival port it does not yet cover — the
    /// only state in which a blockading picket has somewhere to go (T-125).
    pub blockade_ready: bool,
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
    /// happens constantly in a colonized galaxy and says nothing about whether
    /// exploring would help. This is *unexplored*, and it is the only honest
    /// precondition for building a survey craft — at zero, `launch_survey` has
    /// nothing to pick and the hull flies nowhere.
    ///
    /// Counted off a running total rather than a walk, because a production
    /// decision reads it (`CLAUDE.md` §4: per-decision work must be `O(what the
    /// decision reads)`).
    pub survey_frontier: usize,
}

impl ProductionContext {
    /// **What this center would pay for `hull`, today.**
    ///
    /// The context carries prices for the hulls the policy can name, and until
    /// T-117 each caller picked the right field by branching on the same
    /// Doctrine write the hull came from — so a write that moved a role to a
    /// different hull had to be remembered in two places. This is the lookup
    /// instead: name the hull, get its price.
    ///
    /// Unknown hulls fall back to the General colonizer's price, which is the
    /// dearest thing the context carries — a decision made on a price that is
    /// too high declines, where one made on a price that is too low commits to
    /// a build the yard then cannot pay for.
    pub fn price_of(&self, hull: HullType) -> Price {
        match hull {
            // The survey hull under either standing layer — LSV unarmed, LCV
            // armed (T-121) — plus the two Limited hulls nothing mounts.
            HullType::LimitedSystems | HullType::LimitedContactVehicle => self.light_vehicle_cost,
            HullType::LimitedContactUnit | HullType::LimitedOffensive => self.picket_cost,
            HullType::MediumSystems => self.colonizer_cost,
            _ => self.general_colonizer_cost,
        }
    }
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
    /// autopilot's behavior must not change just because the round layer
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
        // A scarcity-weighted **score** over the three Band readings, not a sum
        // of quantities — the readings are taken explicitly (`.bands()`) for
        // the same reason `hub_value` does: weights carry the units. Summing
        // the *masses* would be a different question (how much ore is here),
        // and `MineralField::total_mass` answers that one.
        // Reads the precomputed Band readings rather than converting three
        // masses here (T-100). Same values, same summation order, so the score
        // is bit-identical — the conversion simply moved to where it can be
        // memoized across the 45.4 M times this runs.
        let base_mineral = view.mineral_bands.iter().zip(ctx.scarcity.iter()).map(|(b, w)| w * b).sum::<f64>();
        let mineral_value = base_mineral * (1.0 + w.mineral_pressure_gain * ctx.mineral_pressure);

        // hub_value: high-K worlds near the empire's center of mass are hubs.
        let dist = view.position.distance(ctx.holdings_centroid);
        // **Four multiplies** (T-130): `exp_fast`, relative error 7.5e-5. This
        // is 45.4 M calls a run and the result is a classification *weight*.
        // It replaced T-102's degree-7 `math::exp_decay` (5.4e-7, eight
        // multiplies in Estrin form) on the author's four-multiply budget; a
        // single degree-4 fit over the measured `[−2, 0]` reaches only 5.0e-4.
        let centrality = crate::transcendental::exp_fast(-dist / w.centrality_scale);
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
        class: Class,
        candidates: &[Candidate],
    ) -> Option<Tasking> {
        // **Ask the standing layer what this design is for; do not switch on
        // the hull** (T-117). `Standing::role_of` inverts `design_for`, so a
        // Doctrine write that moves a role onto a different hull is answered
        // here with no edit — which is what makes the layer a layer, and what
        // T-115 and T-116 each added three branches instead of.
        //
        // Eligibility is permissive with varying competence (R-O44): any hull
        // may take any role, and `role_of` falls through to a competence table
        // for a hull no write has claimed. What is *capability* rather than
        // competence — a Limited hull has no cargo hold, so a Limited
        // colonizer founds nothing — shows up as the target search returning
        // `None`, which is a decline rather than a refusal by hull type.
        let standing = Standing::of(doctrine);
        let best = |want: PlanetClass| candidates.iter().filter(|c| c.ranked.class == want).max_by(score_then_id);
        match standing.role_of(hull, class)? {
            // A scout needs no target here — `launch_survey` picks the nearest
            // unvisited world from the survey frontier.
            Role::Scout => Some(Tasking { role: Role::Scout, target: None }),

            Role::Colonizer => colonize(doctrine, candidates),

            // A miner takes the best rock; the freighter that hauls for it is
            // produced alongside (roles §5 — the center produces both).
            Role::Miner => {
                best(PlanetClass::MiningOutpost).map(|c| Tasking { role: Role::Miner, target: Some(c.ranked.id) })
            }

            // **A picket holds ground** (T-113, design law #8: not force
            // projection, but harass-and-hold). It takes the best world on the
            // candidate list — the same ranking a *rival's* colonizer would be
            // reading, which is the point: denial is only worth anything on
            // ground somebody else wants.
            //
            // That is a third placement rule, and the first two were both
            // refuted (§8.6): nearest-to-own-founding threatens nobody, and
            // nearest-to-rival finds nothing unclaimed to stand on. Ranking by
            // *value* rather than by geometry is the remaining axis.
            Role::Picket => {
                let (a, b) = match doctrine.expand_bias {
                    ExpandBias::ProductionCentersFirst => (PlanetClass::ProductionCenter, PlanetClass::Colony),
                    ExpandBias::ColoniesFirst => (PlanetClass::Colony, PlanetClass::ProductionCenter),
                };
                let ground = best(a).or_else(|| best(b)).map(|c| c.ranked.id);
                // **A blockader's target is a rival port, which is not on this
                // list** (T-125): the engine picks the port, so the hull is worth
                // building with no unclaimed world in view.
                if doctrine.picket_blockades {
                    Some(Tasking { role: Role::Picket, target: ground })
                } else {
                    ground.map(|t| Tasking { role: Role::Picket, target: Some(t) })
                }
            }

            // Nothing else is tasked from a finished hull; hold rather than
            // invent a mission.
            Role::Freighter | Role::Reserve | Role::Scrapped => None,
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
        // **Every color, not the total** (T-73). `works_bill` in `sim` produces
        // both sides of this and the build spends against the same function, so
        // the decision and the purchase cannot drift — the failure
        // `mining_pair_cost` carries a comment about and `settlers_by_hull` was
        // written to end.
        let can_afford_infra = (0..3).all(|i| ctx.stockpile_by_color[i] + eps >= ctx.infra_bill[i]);

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
        // **Price the hull this doctrine will actually lay down** (T-115).
        // The standing layer answers which hull that is — LSV unarmed, LCV once
        // the Warfare card is played — and `price_of` turns it into a price, so
        // the affordability test needs no new context field and no branch on
        // the write.
        let scout_cost = ctx.price_of(Standing::of(doctrine).design_for(Role::Scout).0);
        let can_afford_light = ctx.stockpile_total + Price::new(1e-9) >= scout_cost;

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
                Standing::of(doctrine).scout_order()
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
        // center with an empty frontier, a full bank and three Bands of headroom
        // built a scout, every time, forever.
        //
        // And `candidates.is_empty()` is not the exploration question. It goes to
        // zero the moment everything *scanned* is owned or targeted, which in a
        // colonized galaxy is the common case — median `candidate_count` is **0**
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
            return Standing::of(doctrine).scout_order();
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
        // Whether a picket is already standing on the world this center is
        // aiming at. Read here rather than in the claim branch below because
        // `colony_target` is the only place the *candidate* is in scope —
        // `outward` reduces it to an order, a score and a price.
        let colony_target_claimed = colony_target.is_some_and(|c| c.held_by_me || c.claim_inbound);

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
                // **R-IND11 (open):** whether a General colonizer is ever worth
                // it now that the hold is the only thing distinguishing the
                // hulls. It cannot be answered yet — it turns on the industrial
                // ramp (`Hyades_industry.md` §3), where a General hull is a
                // twelve-year yard commitment against a Medium's three, and on
                // whether a deep seed reaches a rate threshold sooner. Both are
                // stages that have not landed.
                // **What a hull delivers is the least of three things**, since
                // R-O74 was closed: its hold, the target world's ceiling, and
                // what this center can actually spare. The third is new — a
                // hold used to be a promise because the settlers were conjured
                // — and leaving it out would price a General hull for people
                // the center does not have.
                // **What a hull delivers is decided per destination, not per
                // hull** (R-IND12). `settlers_by_hull` is `sim::settler_target`
                // evaluated for this candidate — the world's own ceiling, this
                // center's population, and the transit discount, already folded
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
                    (general_colonizer_hull(doctrine), ctx.general_colonizer_cost, col.settlers_by_hull[1]),
                ];
                // **Only hulls this center can pay for today.** The score picks
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
                    // all, so `can_expand` refuses it and the center saves
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
        // of ceiling, deepening **realizes** a Band of them here. The `min(1, ·)`
        // is what a rung actually delivers — `apply_build` steps to the next
        // whole rung whatever the headroom, so a last partial step pays a full
        // price for less than a Band.
        //
        // The comparison is then an odds ratio — depth wins when
        // `b/(1 − b) >= expand/deepen` — and that crossover is **state-
        // dependent**, which is the graded region the old form had nowhere: a
        // center facing a cheap next rung and a mediocre candidate deepens where
        // one facing an expensive rung and a hub does not.
        //
        // **What it does not do is revive the branch at the shipped defaults,
        // and that is the finding rather than a shortfall.** The infra ladder
        // charges 0.9 kt for the rung above the founding one where a Medium
        // colonizer costs 0.1 kt, so expansion buys tens of times the score per
        // kilotonne and *ought* to win: the dead branch was the right answer
        // reached for a wrong reason.
        //
        // Measured (`examples/deepen_census`, 600 planets / 1,500 yr, seeds 1
        // and 7): the run is **bit-identical to the old form** everywhere below
        // `b = 0.96` — same build mix, same colony count, same colony-years to
        // the decimal — so this is a units fix and not a behavior change. What
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
        //
        // **The picket sits in front of survey in the same fallback** (T-113).
        // Same self-limiting shape and the same reason: an idle cycle is the
        // only thing either is allowed to spend, so neither can starve
        // expansion however high its reserve is set. Ahead of survey because a
        // held world is a standing asset and a scan is a one-off — and because
        // `picket_reserve` is a *stock* target that converges, where
        // `survey_reserve` is a frontier that refills.
        // **Behind survey, not in front of it.** Survey is ratified and
        // load-bearing — R-O86 showed a scout with nowhere to scout was 99% of
        // all production, and `survey_reserve = 1024` is what keeps the map
        // ahead of the expansion loop. A picket that pre-empts it starves the
        // thing that finds worlds to picket. The first version of this branch
        // sat *ahead* of survey and cost its own player 24.5% of its colonies.
        let wants_picket = ctx.pickets_held < doctrine.picket_reserve;
        let can_afford_picket = ctx.stockpile_total + Price::new(1e-9) >= ctx.picket_cost;
        // **The blockade first, bounded by the reserve** (T-125,
        // [`Doctrine::picket_first`]).
        if doctrine.picket_first && doctrine.picket_blockades && ctx.blockade_ready && wants_picket && can_afford_picket
        {
            return Standing::of(doctrine).order_for(Role::Picket);
        }
        let survey_fallback = if wants_survey && can_afford_light {
            Standing::of(doctrine).scout_order()
        } else if wants_picket && can_afford_picket {
            Standing::of(doctrine).order_for(Role::Picket)
        } else {
            BuildOrder::Idle
        };
        // **The saving-for-a-world fallback** (T-113,
        // [`Doctrine::picket_claims_target`]). This is the one state where the
        // picket goes *ahead* of survey, and the reason it may is that the
        // center is not choosing between scouting and holding in the abstract:
        // it has already named the world, and the cycle is already committed to
        // waiting for it.
        //
        // It cannot run away, because it is gated on the target not yet being
        // held — one hull per world, not one per cycle — as well as on the
        // empire-wide reserve. The branch that *did* run away sat in front of
        // survey in every arm and cost its own player 24.5% of its colonies.
        let claim_fallback = if doctrine.picket_claims_target
            && wants_picket
            && can_afford_picket
            && !colony_target_claimed
            && colony_target.is_some()
        {
            Standing::of(doctrine).order_for(Role::Picket)
        } else {
            survey_fallback
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
                claim_fallback
            }
        } else if deepen_possible && can_afford_infra {
            BuildOrder::UpgradeInfrastructure
        } else {
            survey_fallback
        }
    }
}

/// **The hull the armed frontier writes survey onto** — the one branch
/// [`Standing::design_for`] takes for [`Role::Scout`], and the only place the
/// `scout_hull_offensive` write is read.
///
/// Private on purpose (T-121). Everything outside this module asks
/// [`Standing::design_for`] or [`Standing::scout_order`], which are derived
/// from this one answer rather than repeating it — the build branch,
/// `assign_role` and `launch_survey` used to read the write separately, and
/// three readings of one write is how they come to disagree (T-116's
/// `launch_survey` paid for a hull and discarded it).
fn scout_hull(doctrine: &Doctrine) -> HullType {
    if doctrine.scout_hull_offensive {
        HullType::LimitedContactVehicle
    } else {
        HullType::LimitedSystems
    }
}

/// **The standing layer, resolved** — Design and Doctrine composed into one
/// answer per question (`Hyades_standing_layer_and_observation.md` §1).
///
/// **Ask it; do not branch on the writes.** Every Doctrine write that moves a
/// role onto a different hull used to add a condition in three places — the
/// build order, the price the context carries, and `assign_role`'s match — and
/// three readings of one write is how they come to disagree. T-115 and T-116
/// each added a write and each added those three branches; this is what they
/// should have added instead.
///
/// The load-bearing property is that **[`Self::role_of`] is the inverse of
/// [`Self::design_for`]**, derived rather than written out. A card that mounts
/// the colony role on a Contact hull changes `design_for`, and `role_of`
/// follows with no edit — which is the difference between a standing layer and
/// a switch statement. `role_of_inverts_design_for_every_role` pins it.
///
/// Borrowed rather than owned because it is constructed per decision, on the
/// engine's hottest path, and holds no state of its own: it is a *reading* of
/// the two writes, the way a `Band` is a reading of a mass (§4).
#[derive(Clone, Copy, Debug)]
pub struct Standing<'a> {
    /// The Doctrine half. The Design half is `Roster`, which does not yet gate
    /// anything (`SimConfig::enforce_roster` defaults off, T-25), so it is not
    /// a field until it is one.
    pub doctrine: &'a Doctrine,
}

/// The roles `assign_role` can hand a finished hull. `Freighter` is absent on
/// purpose — it is produced alongside a `Miner` rather than tasked (roles §5) —
/// and so are `Reserve` and `Scrapped`, which are terminal states.
///
/// **The order is load-bearing since T-121.** [`Standing::role_of`]'s second
/// pass takes the first role that *mounts* the hull, and the unarmed default
/// puts Scout and Miner on the same Limited Systems hull — so a `LimitedSystems`
/// carrying a class neither has named resolves to whichever comes first here.
/// Miner precedes Scout because that is what [`competent_role`] says a bare LSV
/// is for, and the two answers should not disagree. Nothing production-built
/// reaches that pass — every build stamps a class — but
/// `every_hull_has_a_role_under_every_doctrine` does.
const ASSIGNABLE: [Role; 4] = [Role::Colonizer, Role::Miner, Role::Scout, Role::Picket];

impl<'a> Standing<'a> {
    /// Read the standing layer for one player.
    pub fn of(doctrine: &'a Doctrine) -> Self {
        Self { doctrine }
    }

    /// **The design this layer lays down for `role`** — the hull *and* the
    /// class, because the class is what distinguishes two roles sharing a hull.
    /// Two pairs share one today: a scouting Limited Contact Vehicle (`Tor`)
    /// against a picketing one (`Cairn`) once the Warfare card is played, and
    /// a scouting Limited Systems hull (`Spur`) against a mining one (`Meadow`)
    /// before it (T-115, T-121).
    pub fn design_for(&self, role: Role) -> (HullType, Class) {
        match role {
            Role::Scout => match scout_hull(self.doctrine) {
                HullType::LimitedSystems => (HullType::LimitedSystems, Class::Spur),
                hull => (hull, Class::Tor),
            },
            Role::Colonizer => (HullType::MediumSystems, Class::Delta),
            Role::Miner => (HullType::LimitedSystems, Class::Meadow),
            Role::Picket => (HullType::LimitedContactVehicle, Class::Cairn),
            // **Not assignable** — a freighter is produced beside a miner, not
            // tasked (roles §5), so `ASSIGNABLE` excludes it. It shares the
            // Medium Systems *hull* with the colonizer and not the Design:
            // since T-133 the two classes differ (Ford, Delta), so a freighter
            // can no longer read back as a colony ship. The two terminal
            // states answer with the freighter's Design because they must
            // answer something; nothing builds to them.
            Role::Freighter | Role::Reserve | Role::Scrapped => (HullType::MediumSystems, Class::Ford),
        }
    }

    /// **The build order that produces a hull this layer will task as
    /// `role`** — `design_for` turned into a `BuildOrder`, so the hull the
    /// yard is charged for and the role `role_of` reads back are the same
    /// answer to the same question.
    ///
    /// **This is not `hull_order(some_hull)` and the difference bites** (T-121).
    /// A free function mapping a hull to a class cannot know which *role* the
    /// yard meant: a picket and an armed scout share the Limited Contact hull,
    /// and `hull_order` stamps that hull's one class — so ordering a picket by
    /// naming its hull produced a scout. That is T-116's defect exactly, a yard
    /// paying for one thing and receiving another, and the fix is the same:
    /// ask for the role, not the shell.
    pub fn order_for(&self, role: Role) -> BuildOrder {
        let (hull_type, class) = self.design_for(role);
        BuildOrder::Hull { hull_type, class }
    }

    /// The survey craft's build order. The class is always `Tor` — the survey
    /// design — whatever shell it is mounted on, and that is what
    /// distinguishes a scouting hull from the mining or picketing one it
    /// shares a shell with.
    pub fn scout_order(&self) -> BuildOrder {
        self.order_for(Role::Scout)
    }

    /// **The colonizer ladder, cheapest rung first.**
    ///
    /// Two rungs because the Contact ladder has no Medium tier, so a card that
    /// arms the colony ship is forced to the General one (T-116). Substituting
    /// happens *inside* this array — the Medium rung is not the card's to take.
    pub fn colonizer_ladder(&self) -> [HullType; 2] {
        [HullType::MediumSystems, general_colonizer_hull(self.doctrine)]
    }

    /// **Does this layer mount `role` on `hull`?**
    ///
    /// Eligibility is permissive with varying competence (R-O44): any hull may
    /// take any role, and this asks the narrower question of whether *this*
    /// standing layer would put that role on that hull.
    pub fn mounts(&self, role: Role, hull: HullType) -> bool {
        match role {
            Role::Colonizer => self.colonizer_ladder().contains(&hull),
            _ => self.design_for(role).0 == hull,
        }
    }

    /// **What a finished design is for** — the inverse of [`Self::design_for`].
    ///
    /// Resolved in three passes, narrowest first: an exact design match, then
    /// the hull alone for a class this layer has not named, then competence
    /// (R-O44) for a hull it mounts nothing on at all. `None` means the hull
    /// has no mission this layer can give it, which is "hold" rather than an
    /// error.
    ///
    /// **Two configurations resolve differently than the switch it replaced,
    /// and both are unreachable today.** A `Tor` on a Limited Offensive hull
    /// with [`Doctrine::scout_hull_offensive`] *off* now reads as a picket
    /// rather than a scout — nothing builds that pairing, because only
    /// `Standing::scout_order` stamps `Tor` and it names the armed hull only when the
    /// write is on. And a Rapid or General Offensive hull now reads as a
    /// picket where the switch returned `None`; nothing builds those either.
    /// Both moves make the resolver *total*, which
    /// `every_hull_has_a_role_under_every_doctrine` requires and the switch
    /// did not satisfy.
    pub fn role_of(&self, hull: HullType, class: Class) -> Option<Role> {
        ASSIGNABLE
            .iter()
            .copied()
            .find(|&r| self.design_for(r) == (hull, class))
            .or_else(|| ASSIGNABLE.iter().copied().find(|&r| self.mounts(r, hull)))
            .or_else(|| competent_role(hull))
    }

    /// **Is a colonizer consumed by the colony it founds?**
    ///
    /// The one question the Warfare card's Doctrine half turns on, and T-116
    /// measured it as the card's entire cost: a hull that is not consumed
    /// leaves no recycled stock, and that compounds
    /// (`Hyades_warfare_tree.md` §8.10).
    pub fn recycles_on_founding(&self) -> bool {
        !self.doctrine.picket_after_founding
    }

    /// **How this layer regards another empire** (T-133). There is no
    /// diplomacy yet (T-11), so every other empire is neutral unless the
    /// hostility write makes neutrals enemies — §8.2's *"a Neutral empire is an
    /// Enemy empire"*.
    pub fn regard(&self) -> Relation {
        if self.doctrine.engage_neutrals {
            Relation::Enemy
        } else {
            Relation::Neutral
        }
    }

    /// **How far a hull in `role` carrying `loadout` fires on a hull it
    /// regards as `toward`**, or `None` where it holds fire
    /// (`Hyades_warfare_tree.md` §8.19, T-133).
    ///
    /// The distances are the Design's (`Loadout::fire_enemy_ly`,
    /// `fire_neutral_ly`), both the engagement range its fire control supports
    /// (`combat::engagement_range_ly`); Doctrine decides which it ignores, and
    /// ignoring one holds fire at any range (the author's ruling). The author
    /// also ruled that engagement range *sometimes* depends on role; this is
    /// where a role would narrow it, and no role does yet (R-WAR33). The default layer fires
    /// on enemies and holds fire on neutrals — **except in the picket role**,
    /// whose whole mission is denying a neutral's colony ships, which is what
    /// the Warfare card's writes put hulls in that role to do.
    /// **What a fleet does when an enemy threatens or fires on it** (T-133,
    /// warfare §8.19.2 and §8.19.7, the author's rulings).
    ///
    /// - A **colony ship** seeks a new destination: colonists who believe an
    ///   enemy will kill them before they can found are not suicidal.
    /// - A hull that **returns fire on an enemy** stands: that is a pitched
    ///   battle, which needs both sides' Doctrine to kill. One that returns
    ///   fire but regards the shooter only as a neutral breaks off when it
    ///   believes it can outrun it (R-WAR26's third ending); otherwise it is
    ///   committed and stands.
    /// - A hull **holding a post** it cannot defend — a mining crew, a
    ///   reserve — leaves: an unarmed ship runs or completes its mission, and
    ///   its mission here is over while it is under fire.
    /// - A hull **under way** completes its mission.
    ///
    /// A hull past its structure withdraws whatever this says (§2.1, R-WAR26's
    /// second ending); that is decided per hull, not per fleet.
    pub fn under_fire(
        &self,
        role: Role,
        returns_fire: bool,
        regards_enemy: bool,
        under_way: bool,
        may_disengage: bool,
    ) -> UnderFire {
        if role == Role::Colonizer {
            return UnderFire::Retarget;
        }
        if returns_fire {
            return if !regards_enemy && may_disengage { UnderFire::Withdraw } else { UnderFire::Continue };
        }
        if !under_way {
            return UnderFire::Withdraw;
        }
        UnderFire::Continue
    }

    pub fn fire_distance(&self, role: Role, loadout: &Loadout, toward: Relation) -> Option<f64> {
        if !loadout.is_armed() {
            return None;
        }
        match toward {
            Relation::Enemy => Some(loadout.fire_enemy_ly),
            Relation::Neutral if role == Role::Picket => Some(loadout.fire_neutral_ly),
            Relation::Neutral => None,
        }
    }
}

/// **What a fleet does about a threat or a hit** (T-133, warfare §8.19.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnderFire {
    /// Keep flying the mission — or stand and fight.
    Continue,
    /// A colony ship seeks a new destination.
    Retarget,
    /// Head home.
    Withdraw,
}

/// How one empire regards another (T-133). Only the two the fire distances
/// distinguish; the rest of the diplomatic list is T-11.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Enemy,
    Neutral,
}

/// **What a hull is good for when the standing layer mounts nothing on it.**
///
/// The permissive rule (R-O44, roles §4): any hull may take any role, and what
/// differs is how well it does the job. This is the competence table — not an
/// eligibility restriction — and it is the last resort in
/// [`Standing::role_of`], reached only for a hull no Doctrine write has claimed.
fn competent_role(hull: HullType) -> Option<Role> {
    match hull {
        // A Systems hull above the Limited tier has a hold, so it can found.
        HullType::MediumSystems | HullType::GeneralSystems => Some(Role::Colonizer),
        // A Contact hull scouts; that is what its sensors are.
        HullType::LimitedContactVehicle
        | HullType::LimitedContactUnit
        | HullType::GeneralContactVehicle
        | HullType::GeneralContactUnit => Some(Role::Scout),
        // A Limited Systems hull mines.
        HullType::LimitedSystems => Some(Role::Miner),
        // An Offensive hull holds ground (design law #8: harass-and-hold).
        HullType::LimitedOffensive | HullType::RapidOffensive | HullType::GeneralOffensive => Some(Role::Picket),
    }
}

/// **Pick a colonization target**, shared by every hull that can found.
///
/// Extracted at T-116 because the card mounts the colony role on a Contact hull
/// as well as on the two Systems hulls, and three copies of a preference order
/// is how one of them comes to be a different preference order.
fn colonize(doctrine: &Doctrine, candidates: &[Candidate]) -> Option<Tasking> {
    let best = |want: PlanetClass| candidates.iter().filter(|c| c.ranked.class == want).max_by(score_then_id);
    let (a, b) = match doctrine.expand_bias {
        ExpandBias::ProductionCentersFirst => (PlanetClass::ProductionCenter, PlanetClass::Colony),
        ExpandBias::ColoniesFirst => (PlanetClass::Colony, PlanetClass::ProductionCenter),
    };
    // **Settle the ground the pickets are holding, first** (T-113).
    //
    // A world this empire holds is one a rival's colonizer turns back from, so
    // the race for it is already won and the voyage cannot be wasted on a claim
    // someone else got to first. That is what makes a picket worth its hull:
    // not the colony a rival does not found, which §8.6 measured as a losing
    // trade, but the one *this* empire does.
    //
    // Ahead of the class preference rather than folded into `rank`: holding is
    // a fact about the board, not a score, and mixing it into the weighted sum
    // would make it tradeable against mineral richness at some exchange rate
    // nobody has ratified.
    let held =
        |want: PlanetClass| candidates.iter().filter(|c| c.held_by_me && c.ranked.class == want).max_by(score_then_id);
    held(a)
        .or_else(|| held(b))
        .or_else(|| best(a))
        .or_else(|| best(b))
        .map(|c| Tasking { role: Role::Colonizer, target: Some(c.ranked.id) })
}

/// **The General-tier hull this doctrine colonizes with** (T-116).
///
/// One function, for the same reason `Standing::scout_order` is one: the option set, the
/// price the context carries and `assign_role` all have to name the same hull,
/// and three readings of one doctrine write is how they come to disagree.
///
/// The Medium hull is untouched — this substitutes *within* the option set.
pub fn general_colonizer_hull(doctrine: &Doctrine) -> HullType {
    if doctrine.colonizer_general_contact {
        HullType::GeneralContactVehicle
    } else {
        HullType::GeneralSystems
    }
}

/// A build order for `hull`, taking whichever class this policy names for it.
///
/// **Use [`Standing::order_for`] whenever the caller knows the role**, which is
/// almost always. This maps a *shell* to a class and therefore cannot express
/// two roles sharing one — ordering a picket through here returned an armed
/// scout (T-121), which is the T-116 defect again. It survives for the mining
/// pair and the colonizer ladder, where the caller is choosing a hull by price
/// and the role is not in question.
fn hull_order(hull: HullType) -> BuildOrder {
    let class = match hull {
        HullType::LimitedSystems => Class::Meadow,
        HullType::LimitedContactVehicle => Class::Tor,
        HullType::MediumSystems => Class::Delta,
        HullType::GeneralSystems => Class::Range,
        HullType::GeneralContactVehicle => Class::Scarp,
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
    use crate::resources::{Basic, MineralField};
    use crate::units::Measure;

    fn view(id: u32, pos: Vec3, hab: f64, bio: f64, minerals: MineralField) -> PlanetView {
        PlanetView {
            id: PlanetId(id),
            position: pos,
            habitability: Band::new(hab),
            biosphere: Band::new(bio),
            mineral_bands: [
                minerals.get(Basic::Cyan).in_bands().bands(),
                minerals.get(Basic::Magenta).in_bands().bands(),
                minerals.get(Basic::Yellow).in_bands().bands(),
            ],
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

    /// **The blockade first — only while it has a port to cover and the
    /// reserve is short** (T-125). Each precondition is removed in turn and
    /// the branch must fall through; with all of them the order is a picket,
    /// ahead of the colonizer the same context would otherwise buy.
    #[test]
    fn the_blockade_is_built_first_only_while_a_port_is_uncovered() {
        let ap = BaselineAutopilot::default();
        let mut d = Doctrine::default();
        crate::cards::apply_doctrine_write(&mut d, crate::cards::DoctrineWrite::ArmedFrontier);
        let mut ctx = prod_ctx(BandTier::III, 2.0, 50.0);
        // Nothing left to survey, so the empty-list scout return (which comes
        // first, and should — a seat that knows no worlds needs to find some)
        // stays out of the way.
        ctx.survey_frontier = 0;
        ctx.blockade_ready = true;
        ctx.pickets_held = 0;
        let picket = Standing::of(&d).order_for(Role::Picket);
        assert_eq!(ap.production_choice(&d, &ctx, &[]), picket, "all preconditions hold");
        let off = Doctrine { picket_first: false, ..d };
        assert_ne!(ap.production_choice(&off, &ctx, &[]), picket, "without the write it is a fallback again");
        let covered = ProductionContext { blockade_ready: false, ..ctx };
        assert_ne!(ap.production_choice(&d, &covered, &[]), picket, "nowhere to send it");
        let full = ProductionContext { pickets_held: d.picket_reserve, ..ctx };
        assert_ne!(ap.production_choice(&d, &full, &[]), picket, "the reserve is met");
        assert!(!Doctrine::default().picket_first, "the write must ship unplayed");
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
            // Pickets are off in these fixtures: the reserve is 0 by default, so
            // the branch never fires and these tests stay about deepen/expand.
            picket_cost: Price::new(0.02),
            pickets_held: 0,
            blockade_ready: false,
            infra_cost: Price::new(infra + 1.0),
            // Even thirds against a bank of even thirds: these cases are about
            // the deepen/expand branch, not about color scarcity, and
            // `a_color_poor_center_cannot_buy_the_rung` covers that
            // deliberately.
            infra_bill: [Price::new((infra + 1.0) / 3.0); 3],
            stockpile_by_color: [Price::new(stockpile / 3.0); 3],
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
            held_by_me: false,
            claim_inbound: false,
        }];
        let order = ap.production_choice(&doctrine, &ctx, &cands);
        assert!(matches!(order, BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }));
    }

    /// **A colonizer settles held ground over a better world nobody holds**
    /// (T-113). This is the preference the six-slot reduction in
    /// `sim::commit_one_build` exists to feed: with a three-slot reduction the
    /// held world reaches the policy only when it already wins its class, so
    /// the filter below has nothing to select and the arm is a no-op by
    /// construction — which is exactly how it measured before the reduction was
    /// widened (`examples/denial_census`: every printed digit unchanged).
    ///
    /// The ranking is asserted rather than assumed, so a future change to
    /// `rank` cannot make this test pass by accident.
    #[test]
    fn a_colonizer_prefers_held_ground_to_a_better_unheld_world() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        let rctx = RankContext { scarcity: [1.0, 1.0, 1.0], holdings_centroid: Vec3::ZERO, mineral_pressure: 0.0 };
        let cand = |id: u32, hab: f64, held: bool| {
            let v = view(id, Vec3::new(10.0, 0.0, 0.0), hab, hab, MineralField::default());
            let ranked = ap.rank(&doctrine, &v, &rctx);
            Candidate {
                view: v,
                ranked,
                settlers_by_hull: [Kilotons::at_tier(BandTier::I), Kilotons::at_tier(BandTier::II)],
                mining_crew: 1,
                held_by_me: held,
                claim_inbound: false,
            }
        };
        let better = cand(5, 3.5, false);
        let held = cand(6, 3.3, true);
        assert_eq!(better.ranked.class, held.ranked.class, "the two must be the same class for this to be a choice");
        assert!(better.ranked.score > held.ranked.score, "the unheld world must be the one rank actually prefers");

        let tasking = ap
            .assign_role(&doctrine, HullType::MediumSystems, Class::Unnamed, &[better, held])
            .expect("a Medium Systems hull should find a colonization mission");
        assert_eq!(tasking.role, Role::Colonizer);
        assert_eq!(tasking.target, Some(held.ranked.id), "held ground wins: the race for it is already over");

        // And with nothing held, the ranking is untouched — the preference is a
        // tie-break on a subset, not a new term in the score.
        let neither =
            ap.assign_role(&doctrine, HullType::MediumSystems, Class::Unnamed, &[better, cand(6, 3.3, false)]).unwrap();
        assert_eq!(neither.target, Some(better.ranked.id), "without a picket the best world still wins");
    }

    /// **The class is what tells a scouting LOU from a picketing one** (T-115).
    ///
    /// `Doctrine::scout_hull_offensive` puts the armed hull in the survey
    /// branch, so both the survey branch and the picket branch lay down a
    /// `LimitedContactVehicle` and the hull alone can no longer say which
    /// errand it was built for. `Standing::scout_order` stamps the survey design on it
    /// and `assign_role` reads that back — one write, read in two places,
    /// which is the property worth pinning.
    ///
    /// **T-121 rebased the armed hull from `LimitedOffensive` to
    /// `LimitedContactVehicle`** and made the *unarmed* scout an LSV, so the
    /// class now disambiguates on both sides of the write rather than only the
    /// armed one: unarmed it separates a scouting LSV (`Tor`) from a mining one
    /// (`Meadow`), armed it separates a scouting LCV from a picketing one.
    /// **Every build order round-trips to the role that asked for it** (T-121).
    ///
    /// `Standing::order_for` and `Standing::role_of` are the two halves of one
    /// question, and they only agree by construction if the order carries the
    /// *class* the design names. Ordering by shell does not: a picket and an
    /// armed scout share the Limited Contact hull, so `hull_order` stamped
    /// `Tor` on a picket order and the yard received a scout — T-116's defect,
    /// a build paid for and tasked as something else.
    ///
    /// Asserted under both standing layers, because the collision only exists
    /// under one of them and a test run on the default would not see it.
    #[test]
    fn a_build_order_round_trips_to_the_role_that_asked_for_it() {
        for doctrine in [Doctrine::default(), Doctrine { scout_hull_offensive: true, ..Doctrine::default() }] {
            let st = Standing::of(&doctrine);
            for role in [Role::Scout, Role::Colonizer, Role::Miner, Role::Picket] {
                let BuildOrder::Hull { hull_type, class } = st.order_for(role) else {
                    panic!("{role:?} must order a hull");
                };
                assert_eq!(
                    st.role_of(hull_type, class),
                    Some(role),
                    "ordering a {role:?} laid down {hull_type:?}/{class:?}, which reads back as something else"
                );
            }
        }

        // And the defect itself, pinned rather than described: ordering a
        // picket by naming its *shell* does not round-trip once the armed write
        // puts the scout on the same shell. If this ever starts round-tripping,
        // the collision is gone and the rule above has lost its reason.
        let armed = Doctrine { scout_hull_offensive: true, ..Doctrine::default() };
        let st = Standing::of(&armed);
        let by_shell = hull_order(st.design_for(Role::Picket).0);
        let BuildOrder::Hull { hull_type, class } = by_shell else { panic!() };
        assert_eq!(
            st.role_of(hull_type, class),
            Some(Role::Scout),
            "a picket ordered by shell must still read back as the armed scout it collides with"
        );
    }

    /// **Fire distances, and Doctrine holding fire** (T-133, warfare §8.19):
    /// an unarmed hull never fires; by default a picket fires on neutrals and
    /// every other role holds fire on them; everyone fires on an enemy, and
    /// the hostility write is what makes a neutral one.
    #[test]
    fn doctrine_holds_fire_on_neutrals_except_in_the_picket_role() {
        let cfg = crate::sim::SimConfig::new(1);
        let combat = crate::combat::CombatConfig::default();
        let gun = crate::sim::design_loadout(HullType::LimitedContactVehicle, Class::Cairn, &cfg, &combat);
        let none = crate::combat::Loadout::UNARMED;
        let peace = Doctrine::default();
        let st = Standing::of(&peace);
        assert_eq!(st.regard(), Relation::Neutral, "every other empire is neutral by default");
        assert_eq!(st.fire_distance(Role::Picket, &none, Relation::Enemy), None, "unarmed never fires");
        assert_eq!(st.fire_distance(Role::Picket, &gun, Relation::Neutral), Some(gun.fire_neutral_ly));
        for role in [Role::Scout, Role::Colonizer, Role::Miner, Role::Freighter, Role::Reserve] {
            assert_eq!(st.fire_distance(role, &gun, Relation::Neutral), None, "{role:?} holds fire on a neutral");
            assert_eq!(st.fire_distance(role, &gun, Relation::Enemy), Some(gun.fire_enemy_ly));
        }
        let war = Doctrine { engage_neutrals: true, ..Doctrine::default() };
        assert_eq!(Standing::of(&war).regard(), Relation::Enemy, "the hostility write makes a neutral an enemy");
    }

    #[test]
    fn a_contact_hull_scouts_as_a_tor_and_pickets_otherwise() {
        let ap = BaselineAutopilot::default();
        let plain = Doctrine::default();
        let armed = Doctrine { scout_hull_offensive: true, ..Doctrine::default() };
        assert!(!plain.scout_hull_offensive, "the write must ship unplayed");

        assert_eq!(
            Standing::of(&armed).scout_order(),
            BuildOrder::Hull { hull_type: HullType::LimitedContactVehicle, class: Class::Tor },
            "the armed survey hull carries the survey design"
        );
        assert_eq!(
            Standing::of(&plain).scout_order(),
            BuildOrder::Hull { hull_type: HullType::LimitedSystems, class: Class::Spur },
            "the unarmed one carries its own Design: a class names one hull"
        );

        let cands = one_colony_candidate(&ap, &armed);
        let hull = HullType::LimitedContactVehicle;
        let as_scout = ap.assign_role(&armed, hull, Class::Tor, &cands).unwrap();
        assert_eq!(as_scout.role, Role::Scout);
        assert_eq!(as_scout.target, None, "a scout picks its own world from the frontier");

        let as_picket = ap.assign_role(&armed, hull, Class::Cairn, &cands).unwrap();
        assert_eq!(as_picket.role, Role::Picket, "the same hull without the survey design still holds ground");

        // And the same disambiguation holds on the unarmed side, where the
        // scout shares its hull with the miner instead. Asserted on `role_of`
        // rather than `assign_role`, because the latter also requires a viable
        // target and this fixture carries no mining candidate — a decline
        // there would say nothing about which role the design resolves to.
        let st = Standing::of(&plain);
        let lsv = HullType::LimitedSystems;
        assert_eq!(st.role_of(lsv, Class::Spur), Some(Role::Scout));
        assert_eq!(st.role_of(lsv, Class::Meadow), Some(Role::Miner));
    }

    /// **The General colonizer is unreachable under `CheapestViable`, and that
    /// is algebra rather than a magnitude** (T-116).
    ///
    /// `production_choice` filters the hull options to those the center can pay
    /// for *and* that would deliver settlers, then takes the max by policy key.
    /// Under `CheapestViable` the key is the negated price, and a Medium hull
    /// costs **0.1092 kt against any General hull's ~1.1–1.3** — so the General
    /// option wins only if the Medium is filtered out, i.e. delivers zero.
    ///
    /// It cannot. `sim::settler_target` takes
    /// `hi = hold.min(k_target).min(pop − floor)` and returns zero exactly when
    /// `hi ≤ 0`, `k_origin ≤ 0` or `k_target ≤ floor` — **none of which
    /// mentions the hold beyond it being positive.** So the two options are
    /// admitted together or not at all, and the cheaper one always wins.
    ///
    /// This is why `Doctrine::colonizer_general_contact` is inert on its own:
    /// the card's Design write substitutes a hull into a slot the shipped
    /// policy never selects (`Hyades_warfare_tree.md` §8.10).
    #[test]
    fn the_cheapest_viable_policy_never_names_a_general_colonizer() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine::default();
        assert_eq!(doctrine.colonizer_policy, ColonizerPolicy::CheapestViable, "the shipped policy");
        let cands = one_colony_candidate(&ap, &doctrine);
        // Rich enough for either hull, so affordability is not what decides.
        let mut ctx = prod_ctx(BandTier::III, 3.0, 500.0);
        ctx.colonizer_cost = Price::new(0.1092);
        ctx.general_colonizer_cost = Price::new(1.0995);
        let order = ap.production_choice(&doctrine, &ctx, &cands);
        assert!(
            matches!(order, BuildOrder::Hull { hull_type: HullType::MediumSystems, .. }),
            "the cheap hull wins whatever sits in the General slot, got {order:?}"
        );

        // And substituting the Contact hull into that slot changes nothing —
        // the *same* order comes back, which is the inert verdict.
        let gcv = Doctrine { colonizer_general_contact: true, ..Doctrine::default() };
        assert_eq!(
            general_colonizer_hull(&gcv),
            HullType::GeneralContactVehicle,
            "the write must actually name the Contact hull"
        );
        assert_eq!(ap.production_choice(&gcv, &ctx, &cands), order, "substituting into a dead slot is a no-op");
    }

    /// **Under `SettlersPerMineral` the substitution bites, and it is a
    /// downgrade** (T-116) — which is the card's price, paid where §8.2 says.
    ///
    /// Settlers per kilotonne of cost, off the ladder
    /// (`examples/hull_compare`): MSV **9.16**, GCV **10.95**, GSV **24.04**.
    /// So a General Systems hull is far and away the best deal, a Contact hull
    /// on the same rung is less than half of it, and the Contact hull still
    /// beats the Medium — it is a middle rung, not a dominated one.
    #[test]
    fn under_settlers_per_mineral_the_contact_hull_is_chosen_and_is_worse() {
        let ap = BaselineAutopilot::default();
        let base = Doctrine { colonizer_policy: ColonizerPolicy::SettlersPerMineral, ..Doctrine::default() };
        let gcv = Doctrine { colonizer_general_contact: true, ..base };
        let cands = one_colony_candidate(&ap, &base);
        let mut ctx = prod_ctx(BandTier::III, 3.0, 500.0);
        ctx.colonizer_cost = Price::new(0.1092);
        ctx.general_colonizer_cost = Price::new(1.0995);
        // The candidate's per-hull settler figures stand in for `settler_target`;
        // the point is the *ranking*, so they carry the ladder's proportions.
        let mut rich = cands.clone();
        rich[0].settlers_by_hull = [Kilotons::new(1.0), Kilotons::new(12.0358)];

        assert!(
            matches!(
                ap.production_choice(&base, &ctx, &rich),
                BuildOrder::Hull { hull_type: HullType::GeneralSystems, .. }
            ),
            "per-mineral picks the General hull when it carries more per kilotonne"
        );
        assert!(
            matches!(
                ap.production_choice(&gcv, &ctx, &rich),
                BuildOrder::Hull { hull_type: HullType::GeneralContactVehicle, .. }
            ),
            "and the write substitutes the Contact hull into that same slot"
        );
    }

    /// **`role_of` inverts `design_for`, for every role and every doctrine**
    /// (T-117) — the property that makes `Standing` a layer rather than a
    /// second switch statement.
    ///
    /// If it holds, a card that moves a role onto a different hull is answered
    /// by `assign_role` with no edit. If it stops holding, some hull is being
    /// built for one errand and tasked to another, which is how the engine
    /// spent three landings paying for hulls it then discarded
    /// (`launch_survey`, T-116).
    ///
    /// Checked across the writes that move a design, in every combination —
    /// which is the point: they compose, and a resolver that is only correct
    /// one write at a time is not one.
    #[test]
    fn role_of_inverts_design_for_every_role() {
        for scout_armed in [false, true] {
            for colonizer_contact in [false, true] {
                let d = Doctrine {
                    scout_hull_offensive: scout_armed,
                    colonizer_general_contact: colonizer_contact,
                    ..Doctrine::default()
                };
                let st = Standing::of(&d);
                for role in [Role::Scout, Role::Colonizer, Role::Miner, Role::Picket] {
                    let (hull, class) = st.design_for(role);
                    assert_eq!(
                        st.role_of(hull, class),
                        Some(role),
                        "design_for({role:?}) = ({hull:?}, {class:?}) must resolve back \
                         (scout_armed={scout_armed}, colonizer_contact={colonizer_contact})"
                    );
                    assert!(st.mounts(role, hull), "and the layer must agree it mounts it there");
                }
                // Every rung of the colonizer ladder resolves to a colonizer,
                // not only the one `design_for` names — otherwise the General
                // rung would be built and then tasked as something else.
                for hull in st.colonizer_ladder() {
                    assert_eq!(st.role_of(hull, Class::Unnamed), Some(Role::Colonizer), "{hull:?} is a colonizer rung");
                }
            }
        }
    }

    /// **Every hull resolves to something, under every doctrine** (T-117).
    ///
    /// `assign_role` returning `None` means "hold rather than invent a
    /// mission", which is legitimate — but a hull the engine *builds* and then
    /// cannot task is a hull the yard was charged for and threw away, and that
    /// defect has now happened twice. The permissive rule (R-O44) says any hull
    /// may take any role, so the competence table must be total.
    #[test]
    fn every_hull_has_a_role_under_every_doctrine() {
        use HullType::*;
        let all = [
            LimitedSystems,
            MediumSystems,
            GeneralSystems,
            LimitedContactVehicle,
            LimitedContactUnit,
            GeneralContactVehicle,
            GeneralContactUnit,
            LimitedOffensive,
            RapidOffensive,
            GeneralOffensive,
        ];
        for scout_armed in [false, true] {
            for colonizer_contact in [false, true] {
                let d = Doctrine {
                    scout_hull_offensive: scout_armed,
                    colonizer_general_contact: colonizer_contact,
                    ..Doctrine::default()
                };
                let st = Standing::of(&d);
                for hull in all {
                    for class in [Class::Unnamed, Class::Spur, Class::Tor, Class::Meadow] {
                        assert!(st.role_of(hull, class).is_some(), "{hull:?}/{class:?} has no mission");
                    }
                }
            }
        }
    }

    /// **A class names one hull** (the author's ruling). Every Design the
    /// standing layer lays down, under every combination of the writes, every
    /// Design a card unlocks, and every class `hull_order` stamps sits on the
    /// hull its class names — and the armed survey Design is armed.
    #[test]
    fn a_named_class_is_on_one_hull() {
        let on_its_hull = |hull: HullType, class: Class, site: &str| {
            if let Some(h) = class.hull() {
                assert_eq!(hull, h, "{site}: {class:?} on {hull:?}, but it names {h:?}");
            }
        };
        for scout_armed in [false, true] {
            for colonizer_contact in [false, true] {
                let d = Doctrine {
                    scout_hull_offensive: scout_armed,
                    colonizer_general_contact: colonizer_contact,
                    ..Doctrine::default()
                };
                let st = Standing::of(&d);
                for role in [Role::Scout, Role::Colonizer, Role::Miner, Role::Picket] {
                    let (hull, class) = st.design_for(role);
                    on_its_hull(hull, class, "design_for");
                }
            }
        }
        for hull in HullType::ALL {
            if let BuildOrder::Hull { hull_type, class } = hull_order(hull) {
                on_its_hull(hull_type, class, "hull_order");
            }
        }
        for hull in [HullType::MediumSystems, HullType::GeneralSystems] {
            on_its_hull(hull, Class::freighter_for(hull), "a freighter");
        }
        for card in crate::cards::TIER0.iter() {
            for effect in card.effects {
                if let crate::cards::CardEffect::UnlockDesign(hull, class) = *effect {
                    on_its_hull(hull, class, "a card's unlock");
                }
            }
        }
        let cfg = crate::sim::SimConfig::new(1);
        let tor = crate::sim::design_loadout(HullType::LimitedContactVehicle, Class::Tor, &cfg, &Default::default());
        assert!(tor.is_armed(), "a Tor is on a Contact hull, so it is armed");
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
            held_by_me: false,
            claim_inbound: false,
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
    /// **not** change: an infra rung costs several colonizers, so expansion wins
    /// at `b = 0.5` and the branch stays cold. That is now a statement about
    /// prices (R-O85) rather than about units.
    #[test]
    fn reinvest_bias_is_an_odds_ratio_on_two_returns_per_kiloton() {
        let ap = BaselineAutopilot::default();
        let mut doctrine = Doctrine::default();
        // A mature center with the most deepening headroom the ladder allows
        // (infra 1 against k_potential 4) and one ordinary colony candidate —
        // i.e. the case most favorable to depth that can actually occur.
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
        //    colonizer and buys less. Prices, not units.
        doctrine.reinvest_bias = 0.5;
        assert!(
            crossover(&ctx) > 0.5,
            "the shipped ladder must still favor expansion at b = 0.5 — if this has crossed, \
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
        //    center deepens and the dear-rung one does not. That band is the
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
    /// `candidates.is_empty()` used to send a center straight to a survey hull,
    /// justified by "no candidates means every other branch below returns Idle."
    /// That is false: the branch it pre-empts is the `outward == None` deepen
    /// fallback, and that fallback is the only deepening path above
    /// `medium_min_level` that actually runs at the shipped `reinvest_bias`.
    ///
    /// So the two halves are pinned here. With frontier left, an empty candidate
    /// list still buys a scout — that is the mechanic restarting expansion, and
    /// it must not regress. With the galaxy explored, the same center deepens.
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
                BuildOrder::Hull { hull_type, .. } if hull_type == Standing::of(&doctrine).design_for(Role::Scout).0
            ),
            "with galaxy left to explore, an empty candidate list must still buy a scout"
        );

        ctx.survey_frontier = 0;
        assert_eq!(
            ap.production_choice(&doctrine, &ctx, &[]),
            BuildOrder::UpgradeInfrastructure,
            "with nothing left to survey, the same center must spend on depth rather than on a hull \
             that would be tasked Scout and fly nowhere"
        );

        // And it must not deepen past the ceiling just because survey is shut:
        // a capped center with nothing to explore and nothing to take idles.
        ctx.infra = ctx.k_potential;
        assert_eq!(
            ap.production_choice(&doctrine, &ctx, &[]),
            BuildOrder::Idle,
            "a capped center with no frontier and no candidates has nothing to buy"
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
            BuildOrder::Hull { hull_type, .. } if hull_type == Standing::of(&doctrine).design_for(Role::Scout).0
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
            BuildOrder::Hull { hull_type, .. } if hull_type == Standing::of(&doctrine).design_for(Role::Scout).0
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
    fn survey_reserve_zero_restores_the_old_never_scout_behavior() {
        let ap = BaselineAutopilot::default();
        let doctrine = Doctrine { survey_reserve: 0, ..Doctrine::default() };
        let mut ctx = prod_ctx_frontier(BandTier::III, 3.0, 0.3, 0);
        ctx.k_potential = 3.0;
        let cands = one_colony_candidate(&ap, &doctrine);
        assert_eq!(ap.production_choice(&doctrine, &ctx, &cands), BuildOrder::Idle);
    }
}
