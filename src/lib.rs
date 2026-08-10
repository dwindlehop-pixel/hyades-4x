//! # Hyades engine
//!
//! The simulation core for *Hyades*. This crate is the **single engine** shared
//! by two consumers that must never see each other:
//!
//! * the **Monte-Carlo balancer** (`Hyades_card_contract.md` §7) — runs many
//!   seeds × matchups headlessly and reads outcome distributions; and
//! * the **production game** — same rules, wrapped by a presentation/command
//!   layer that renders the theater and draws the hex command view.
//!
//! ## The decoupling contract (load-bearing)
//!
//! The caller's requirement — *"the simulation state is a standalone module; no
//! coupling between presentation and simulation; the two are swappable"* — is
//! enforced structurally:
//!
//! * **Nothing in this crate renders, reads input, touches the clock, the
//!   filesystem, the network, threads, or the OS RNG.** All randomness flows
//!   from a seeded [`rng::Rng`]; all time is the in-sim event clock in years.
//!   That is also exactly what makes it WASM-portable and bit-reproducible.
//! * **No hexes live here.** Per `Hyades_autopilot_colonization_growth.md` §1,
//!   *"the simulation has no hexes and no boundaries; it runs in continuous 3D
//!   space with each star system abstracted to a single point."* Hexes are a
//!   **command-view** concept and belong to the presentation layer, which binds
//!   the engine's continuous planet points into the flat hex tiling at render
//!   time. The engine exposes read-only [`snapshot`] views for that binding.
//! * **The autopilot is an interface, not a hard-coded loop.** [`autopilot`]
//!   defines the [`autopilot::Autopilot`] trait (a swappable per-seat policy)
//!   and the [`autopilot::Doctrine`] (the standing-behavior knobs a *card* edits
//!   — `Hyades_card_contract.md` §5). The baseline colonization/growth policy is
//!   one impl; the greedy-`V` MC policy will be another.
//!
//! ## Module map
//!
//! | module | spec home | role |
//! |---|---|---|
//! | [`math`] | sim §1a | 3-vectors + relativistic 1 g flight & light-lag |
//! | [`rng`] | — | deterministic seeded PRNG (reproducible MC) |
//! | [`resources`] | world-model §4 | CMY basics, RGB supers, apex, archetypes |
//! | [`galaxy`] | world-model §2–5 | galaxy generation → continuous planet field |
//! | [`autopilot`] | world-model §6, autopilot-doc | the policy interface + baseline |
//! | [`sim`] | sim §3–4, card-contract §2 | the light-lagged discrete-event engine |
//! | [`log`] | — | optional diagnostic event log (the *interrogation* seam) |

#![forbid(unsafe_code)]
// Portability discipline: deny accidental reliance on the wider std surface that
// is unavailable or nondeterministic under wasm32. (We still link std for
// alloc/collections, which wasm32 supports.)
#![deny(clippy::disallowed_methods)]

pub mod arena;
pub mod autopilot;
pub mod belief;
pub mod cards;
pub mod combat;
pub mod galaxy;
pub mod log;
pub mod math;
pub mod resources;
pub mod rng;
pub mod sim;
pub mod snapshot;

/// Common imports for engine consumers.
pub mod prelude {
    pub use crate::autopilot::{Autopilot, BaselineAutopilot, Doctrine, ExpandBias, RankWeights, SurveyStrategy};
    pub use crate::belief::{BeliefAMax, Engagement, Observation, Unobserved};
    pub use crate::cards::{Card, CardEffect, CardId, Order, Slant, Target, Tree};
    pub use crate::combat::{CombatConfig, Combatant, EngagementOutcome, FleetTrajectory, Winner};
    pub use crate::galaxy::{Galaxy, GalaxyConfig, Planet, PlanetClass, PlanetId, PlayerId};
    pub use crate::log::{LogCategory, LogEvent, LogFilter, LogRecord, SimLog};
    pub use crate::math::Vec3;
    pub use crate::resources::{Archetype, Basic, MineralField, Minerals, Super};
    pub use crate::rng::Rng;
    pub use crate::sim::{Entity, FleetSummary, HullType, Role, SimConfig, SimReport, Simulation};
    pub use crate::snapshot::Snapshot;
}
