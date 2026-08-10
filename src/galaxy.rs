//! Galaxy generation → the **continuous 3-D planet field** the simulation runs on.
//!
//! Per `Hyades_autopilot_colonization_growth.md` §1 the simulation has *no hexes*:
//! each star system is one point ("a planet"). This module produces exactly that
//! field plus the seeded homeworlds. The command-view hex tiling
//! (`Hyades_galaxy_and_autopilot.md` §1–2) is a *presentation* concern and is
//! deliberately **not** generated here — but its **scale** is authoritative for
//! sizing the continuous field. `Hyades_galaxy_and_autopilot.md` §1 originally
//! stated `s ∈ [50,250] ly` per side (R-G1 explicitly left "final `s` and
//! depth" open); this conversation revised the *side* down to **10 ly**,
//! measured against real simulation throughput — see
//! [`GalaxyConfig::hex_side_ly`] — while keeping the stated prism depth
//! (`1×–5× s`, "start `3×`") as-is. The playable galaxy still spans a
//! **hex-grid radius** built from the fair-count starting cluster (§2's
//! tri-hex clique / ring / radius-`r` ring — "three hexes for a minimum
//! player count start") plus **2–4 hex-steps outward in each direction**.
//! [`GalaxyConfig::hex_grid_radius`] turns that into one number;
//! [`GalaxyConfig::xy_scale`]/[`GalaxyConfig::z_scale`] turn *that* into the
//! two physical scale lengths below. Star **count** is not an independent
//! knob — it's *derived* from those scales plus the target local spacing
//! (kept exactly, per this conversation: *"lots of empty space between
//! planets does not create drama and tension... keep the 7 ly mean spacing
//! and reduce the size of a hex"* — not the earlier turn's planet-count
//! cap, which diluted density instead and was reversed), so more hexes
//! (more players) means more stars at the same density, not the same stars
//! spread thinner.
//!
//! What the generator encodes from the world model:
//! * **Star positions — XY radially Poisson (exponential-disk profile), Z
//!   exponential**, independently scaled from the hex grid (previous turns
//!   tied both to one shared length; that was this module's own invention,
//!   not what the hex spec actually says — corrected here). Not a hard-edged
//!   uniform disc (no real galaxy has a wall) and not isotropic (an isotropic
//!   field gives every empire room to expand "vertically" where no one else
//!   is looking, letting conflict be avoided — deliberately rejected). XY's
//!   radial coordinate follows the standard **exponential-disk surface-
//!   density profile** real spiral/disk galaxies actually have,
//!   `Σ(r) ∝ exp(−r/L_xy)` — the radial marginal of that is `Gamma(shape=2,
//!   scale=L_xy)` (area grows as `r`, so density-times-area peaks at
//!   `r=L_xy`, not at the center). Z is a plain two-sided exponential at its
//!   own scale `L_z`, set from the hex-prism depth. Mean *near-typical-
//!   radius* nearest-neighbor spacing is targeted at
//!   [`GalaxyConfig::star_spacing_ly`] (default 7 ly, matched to real
//!   interstellar spacing near a Sun-like star —
//!   [~5 ly](https://www.astronomy.com/science/how-close-can-stars-get-to-each-other-in-galaxy-cores/),
//!   [~0.004/ly³ ⇒ ~3.5–7 ly by method](https://en.wikipedia.org/wiki/Stellar_density))
//!   — see [`GalaxyConfig::derived_planet_count`] for the derivation, an
//!   approximation validated empirically in `tests`, not a closed form for
//!   the true inhomogeneous process.
//! * **§4.3 tier-1 field** — Gaussian in XY around each hue's hotspot ×
//!   exponential decay in Z, matching the star field's own shape.
//! * **§4.4 anticorrelation** — metal-rich planets trend low-habitability; the
//!   colony-vs-mine tension falls out of this.
//! * **§3 homeworlds** — identical `4/4/2` shape (`K = min = 2`), super-aligned
//!   (rich in two basics, poor in the third), placed on a **vertex-transitive
//!   ring** so 2/3/6/12 are the fair counts (§2), one archetype per seat.
//! * **§5 population** — integer level 0–4 read off **Weibull-quantile bands**,
//!   Gibrat-spaced (each level a fixed multiplicative jump).
//!
//! All magnitudes here are placeholders (R-G/R-M/R-P): every knob lives in
//! [`GalaxyConfig`] so the Monte-Carlo balancer can sweep them.

use crate::math::Vec3;
use crate::resources::{Archetype, Basic, MineralField};
use crate::rng::Rng;

/// `Γ(4/3)`, the mean-scaling constant for a Weibull(k=3) distribution — see
/// [`GalaxyConfig::derived_planet_count`]. `Γ(4/3) = (1/3)Γ(1/3)`.
const GAMMA_4_3: f64 = 0.892_979_511_569_249;

/// Sample a point from the flattened field: XY radius via `Gamma(shape=2,
/// scale=xy_scale)` (sum of two `Exponential` draws — the standard, simplest
/// exact sampler for that shape) with a uniform angle, giving the
/// exponential-disk radial profile; Z via a plain two-sided `Exponential`
/// at its own, independently-set `z_scale`.
fn sample_flattened_field(rng: &mut Rng, xy_scale: f64, z_scale: f64) -> Vec3 {
    let r = -xy_scale * (rng.unit().max(1e-12).ln() + rng.unit().max(1e-12).ln());
    let theta = rng.range(0.0, core::f64::consts::TAU);
    let z = {
        let mag = -z_scale * rng.unit().max(1e-12).ln();
        if rng.unit() < 0.5 {
            -mag
        } else {
            mag
        }
    };
    Vec3::new(r * theta.cos(), r * theta.sin(), z)
}

/// Index of a planet within a [`Galaxy`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlanetId(pub u32);

/// Index of a player / seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u32);

/// The four target classes a ranked planet falls into
/// (`Hyades_autopilot_colonization_growth.md` §3). The *thresholds* that assign
/// a class are an autopilot/doctrine concern; the enum is pure data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanetClass {
    /// High K-potential **and** hub value — like the homeworld. Colony vehicle.
    ProductionCenter,
    /// High K-potential, weak hub value. Colony vehicle.
    Colony,
    /// High mineral density, low K-potential — *can out-rank a colony*.
    /// Mining vehicle + freighter.
    MiningOutpost,
    /// Low on all. Ignored.
    Barren,
}

/// A single star system, abstracted to one point in continuous 3-D space.
///
/// Carrying-capacity factors (`habitability`, `biosphere`, `infrastructure`) and
/// `population` are all on the **same level-unit scale** (`Hyades_simulation_model.md`
/// §2a): `0` ≈ empty, `4` ≈ many-billions. `K = min` of the three (Liebig).
#[derive(Clone, Debug)]
pub struct Planet {
    pub id: PlanetId,
    pub position: Vec3,

    // --- carrying-capacity factors (level-units) ---
    /// Hardest to change. The fundamental ceiling.
    pub habitability: f64,
    /// Easy to destroy, slow to improve.
    pub biosphere: f64,
    /// Built capacity. `0` on a wild world; raised by the build cycle. Soft
    /// factor and the early binding constraint (homeworlds start at 1).
    pub infrastructure: f64,

    /// Tier-1 mineral density (ground truth; a close scan reveals it).
    pub minerals: MineralField,

    pub is_homeworld: bool,
    /// `Some` only for homeworlds — fixes the seat's native super & alignment.
    pub archetype: Option<Archetype>,

    // --- mutable sim state ---
    pub owner: Option<PlayerId>,
    /// Continuous population (level-units), grows logistically toward `K`.
    pub population: f64,
}

impl Planet {
    /// Liebig carrying capacity `K = min(hab, bio, infra)`
    /// (`Hyades_simulation_model.md` §2a). A wild world (`infra = 0`) has `K = 0`.
    #[inline]
    pub fn k(&self) -> f64 {
        self.habitability.min(self.biosphere).min(self.infrastructure)
    }

    /// The ceiling infrastructure (and thus population) can be *built* to:
    /// `min(hab, bio)` (autopilot-doc §3).
    #[inline]
    pub fn k_potential(&self) -> f64 {
        self.habitability.min(self.biosphere)
    }
}

/// The three hue hotspots (§4.3), placed on a ring in the reference (z=0)
/// plane. Density of each basic peaks at its hotspot and falls off as an
/// isotropic 3-D Gaussian — no privileged disc plane, matching the isotropic
/// Poisson star field.
#[derive(Clone, Copy, Debug)]
pub struct Hotspots {
    pub cyan: Vec3,
    pub magenta: Vec3,
    pub yellow: Vec3,
}

impl Hotspots {
    fn get(&self, b: Basic) -> Vec3 {
        match b {
            Basic::Cyan => self.cyan,
            Basic::Magenta => self.magenta,
            Basic::Yellow => self.yellow,
        }
    }
}

/// **Weibull-quantile population bands** (`Hyades_galaxy_and_autopilot.md` §5.1).
/// The four internal edges split the continuous population value into levels 0–4.
/// Choosing the shape near log-normal makes the bands Gibrat-spaced (each level a
/// fixed multiplicative jump). R-P1 owns the final `k` and edges.
#[derive(Clone, Copy, Debug)]
pub struct PopBands {
    pub edges: [f64; 4],
}

impl PopBands {
    /// Build bands from a Weibull shape `k`, scaling so the top edge (the 0.8
    /// quantile → the level-3/4 boundary) lands on `top_edge`.
    pub fn from_weibull(k: f64, top_edge: f64) -> Self {
        let q = [0.2, 0.4, 0.6, 0.8];
        // weibull quantile: λ · (−ln(1−p))^(1/k); solve λ so q(0.8) == top_edge.
        let shape = |p: f64| (-(1.0 - p).ln()).powf(1.0 / k);
        let lambda = top_edge / shape(0.8);
        let mut edges = [0.0; 4];
        for (i, &p) in q.iter().enumerate() {
            edges[i] = lambda * shape(p);
        }
        PopBands { edges }
    }

    /// Integer level 0–4 = how many band edges the population value has crossed.
    #[inline]
    pub fn level(&self, population: f64) -> u8 {
        self.edges.iter().filter(|&&e| population >= e).count() as u8
    }
}

impl Default for PopBands {
    fn default() -> Self {
        PopBands::from_weibull(1.4, 4.0)
    }
}

/// Every tunable knob of galaxy generation. Defaults are placeholders pending
/// R-G/R-M/R-P; the MC balancer sweeps them.
#[derive(Clone, Copy, Debug)]
pub struct GalaxyConfig {
    /// Seat count. Must be a *fair* count (2, 3, 6, 12) — vertex-transitive (§2).
    pub players: usize,
    /// Number of wild (un-seeded) planets scattered in the field. Defaulted
    /// by [`Self::new`] via [`Self::derived_planet_count`] — the hex grid
    /// and target spacing are what actually size the galaxy now (confirmed
    /// this conversation), star count follows from them — but left as a
    /// plain mutable field, like everything else here, for direct override.
    pub planet_count: usize,

    /// Hex side length (ly) — the command-view hex-prism footprint
    /// (`Hyades_galaxy_and_autopilot.md` §1 originally stated `s ∈
    /// [50,250] ly`; **revised down** this conversation on measured
    /// simulation-speed grounds — R-G1 was always explicitly open on the
    /// final value, and *"lots of empty space between planets does not
    /// create drama and tension"* independently favors the smaller end
    /// anyway). **10 ly**, chosen from `examples/bench_hex_size.rs`'s
    /// measured throughput: at the 12-player worst case this clears the
    /// confirmed 2.5-simulated-years/real-second target by an **847×**
    /// margin (2,116 yr/s measured), leaving headroom for combat/loadout
    /// costs that don't exist in the engine yet. The empirically-
    /// extrapolated crossover (where throughput would actually drop to
    /// 2.5 yr/s) is ≈47 ly, from the measured local scaling trend, not a
    /// guess — so this isn't a photo-finish choice, it's the smaller,
    /// higher-tension end of a wide comfortably-safe range. Hexes are
    /// never represented in the engine itself ("no hexes in the sim" —
    /// `Hyades_autopilot_colonization_growth.md` §1); this exists purely
    /// to size the continuous star field to the *right scale*.
    pub hex_side_ly: f64,
    /// Hex-prism depth as a multiple of `hex_side_ly` (§1: `1×–5×`, *"start
    /// 3×"* — not a placeholder, the spec's own stated default).
    pub hex_depth_multiple: f64,
    /// How many hex-steps the playable galaxy extends beyond the starting
    /// cluster, in each direction (confirmed this conversation: *"at least
    /// two and maybe as many as four hexes... outward... in each
    /// direction"* — 3 here is the middle of that stated range).
    pub hex_rings_beyond_start: f64,

    /// Target **mean near-typical-radius nearest-neighbor spacing** (ly) of
    /// the star field. Default 7 ly ([real interstellar spacing runs roughly
    /// 4–7 ly by method](https://www.astronomy.com/science/how-close-can-stars-get-to-each-other-in-galaxy-cores/)).
    /// No longer what sizes the galaxy (the hex grid does, above) — this now
    /// sizes [`Self::derived_planet_count`] instead, given the hex-derived
    /// scale, so more hexes at the same target spacing means more stars, not
    /// the same stars spread thinner.
    pub star_spacing_ly: f64,

    /// Radius of the hue-hotspot ring, as a **fraction of the mean XY
    /// radius** (`2 · `[`Self::xy_scale`]`()`).
    pub hotspot_ring_frac: f64,
    /// Gaussian width of each hue hotspot, as a fraction of the mean XY radius.
    pub hotspot_sigma_frac: f64,
    /// Peak tier-1 density at a hotspot center.
    pub mineral_peak: f64,

    /// Strength `∈ [0,1]` of the habitability↔metallicity anticorrelation
    /// (§4.4, R-M4). `0` = independent, `1` = metal-rich worlds are dead.
    pub anticorrelation: f64,

    /// Radius of the homeworld ring, as a fraction of the mean XY radius (§2).
    pub homeworld_ring_frac: f64,
    /// Homeworld density in each of its two *rich* basics (modest — "enough for
    /// one modest super, not super-rich", R-G4).
    pub homeworld_rich_density: f64,
    /// Homeworld density in its one *poor* basic.
    pub homeworld_poor_density: f64,

    /// Weibull shape for the pop bands (§5.1, R-P1).
    pub weibull_k: f64,

    pub seed: u64,
}

impl GalaxyConfig {
    /// A reasonable starting configuration for `players` seats.
    pub fn new(players: usize, seed: u64) -> Self {
        let mut cfg = GalaxyConfig {
            players,
            planet_count: 0, // set below, once the rest of self exists
            hex_side_ly: 10.0,
            hex_depth_multiple: 3.0,
            hex_rings_beyond_start: 3.0,
            star_spacing_ly: 7.0,
            hotspot_ring_frac: 0.55,
            hotspot_sigma_frac: 0.42,
            mineral_peak: 4.0,
            anticorrelation: 0.7,
            homeworld_ring_frac: 0.5,
            homeworld_rich_density: 1.4,
            homeworld_poor_density: 0.25,
            weibull_k: 1.4,
            seed,
        };
        cfg.planet_count = cfg.derived_planet_count();
        cfg
    }

    pub fn pop_bands(&self) -> PopBands {
        PopBands::from_weibull(self.weibull_k, 4.0)
    }

    /// Hex-grid radius (in hex-steps) of the **starting cluster** for
    /// `players` seats — a *scale* reference, not the exact vertex-
    /// transitive topology from `Hyades_galaxy_and_autopilot.md` §2 (that's
    /// a command-view rendering concern, out of scope for sizing the
    /// continuous sim). *"Three hexes for a minimum player count start"*
    /// (this conversation) sets the floor; larger fair counts (6/12/18, the
    /// ring / radius-`r` ring configurations) get proportionally more.
    fn starting_hex_radius(players: usize) -> f64 {
        match players {
            0..=3 => 1.5, // ~3 hexes' worth of starting radius (tri-hex clique)
            // The `6r` ring family, as one closed form instead of three magic
            // numbers: a radius-`r` ring holds `6r` hexes, so `r = N/6`. This
            // reproduces the previous table exactly — 6 → 2.5, 12 → 3.5,
            // 18 → 4.5 — and keeps extending correctly if a larger ring is ever
            // admitted to FAIR_COUNTS.
            n if n % 6 == 0 => (n / 6) as f64 + 1.5,
            // Unreachable for a generated galaxy: `Galaxy::generate` rejects
            // non-fair counts before this runs. It survives only for callers
            // poking `hex_grid_radius` on an unvalidated config, and is
            // deliberately *not* the ring formula — at 18 it would say 3.95
            // against the ring's 4.5, so letting it serve the family would
            // silently mis-size the galaxy.
            n => 1.5 + ((n as f64) / 3.0).sqrt(),
        }
    }

    /// Total hex-grid radius: starting cluster + the outward extension.
    /// *"A game with more players will have more hexes"* (this
    /// conversation) — a galaxy generation parameter, not a fixed constant.
    pub fn hex_grid_radius(&self) -> f64 {
        Self::starting_hex_radius(self.players) + self.hex_rings_beyond_start
    }

    /// XY scale length `L_xy` (ly) for the `Gamma(2, L_xy)` radial profile —
    /// derived from the hex grid, **not** from star count (confirmed this
    /// conversation, replacing the earlier count-derived approach): the
    /// hex-grid radius converted straight to ly. (An earlier pass here
    /// divided by 2, reasoning `L_xy` as "half the mean reach" — that made
    /// `z_scale` rival or exceed this at small player counts, undermining
    /// the flattening `Hyades_vehicle_roles.md`-era "don't let empires find
    /// room vertically" goal; dropped in favor of the direct conversion,
    /// which keeps XY meaningfully ahead of Z at every fair player count —
    /// see `tests`.)
    pub fn xy_scale(&self) -> f64 {
        self.hex_grid_radius() * self.hex_side_ly
    }

    /// Z scale length `L_z` (ly) for the two-sided `Exponential(L_z)`
    /// vertical profile — the hex-prism depth directly (`hex_side_ly ×
    /// hex_depth_multiple`), independent of the XY scale (confirmed this
    /// conversation: earlier tying both to one shared length was this
    /// module's own invention, not the actual hex spec, which defines depth
    /// on its own terms).
    pub fn z_scale(&self) -> f64 {
        self.hex_side_ly * self.hex_depth_multiple
    }

    /// Mean XY radius (`2·L_xy`) — the natural "typical extent" reference
    /// for the hotspot/homeworld ring fractions.
    pub fn mean_xy_radius(&self) -> f64 {
        2.0 * self.xy_scale()
    }

    /// Star count that gives [`Self::star_spacing_ly`] average near-typical-
    /// radius nearest-neighbor spacing, **given** the hex-derived
    /// [`Self::xy_scale`]/[`Self::z_scale`] (an inversion of the derivation
    /// used before the hex grid became authoritative for scale — solving for
    /// `N` given fixed `L_xy, L_z`, instead of solving for `L` given `N`) —
    /// an approximation, not a closed form for the true inhomogeneous
    /// process:
    ///
    /// The exponential-disk areal density is `Σ(r) = N·e^{-r/L_xy} /
    /// (2πL_xy²)`; at `r = L_xy`: `Σ(L_xy) = N·e^{-1}/(2πL_xy²)`. `Z`'s peak
    /// (midplane) density is `1/(2L_z)`. Treating their product as the
    /// local 3-D density near the typical star's location, `λ ≈
    /// N·e^{-1}/(4πL_xy²L_z)`, as locally homogeneous, and reusing the
    /// homogeneous-process nearest-neighbor mean `E[R_nn] = Γ(4/3) /
    /// (λ·(4/3)π)^{1/3}` (Weibull(k=3) mean), solving `E[R_nn] =
    /// star_spacing_ly` for `N` gives a first-pass closed form, corrected by
    /// the same empirically-measured factor as before (~1.87× on `L`,
    /// applied here as `1.87³` on `N` since `N ∝ L³` at fixed spacing — `r =
    /// L_xy` is the radial marginal's peak, but a star actually there
    /// doesn't also sit at the Z-peak `z=0`, so the naive product overstates
    /// true local density) — see `tests` for the empirical check.
    /// Star count that gives [`Self::star_spacing_ly`] average near-typical-
    /// radius nearest-neighbor spacing, **given** the hex-derived
    /// [`Self::xy_scale`]/[`Self::z_scale`] (an inversion of the derivation
    /// used before the hex grid became authoritative for scale — solving for
    /// `N` given fixed `L_xy, L_z`, instead of solving for `L` given `N`) —
    /// an approximation, not a closed form for the true inhomogeneous
    /// process:
    ///
    /// The exponential-disk areal density is `Σ(r) = N·e^{-r/L_xy} /
    /// (2πL_xy²)`; at `r = L_xy`: `Σ(L_xy) = N·e^{-1}/(2πL_xy²)`. `Z`'s peak
    /// (midplane) density is `1/(2L_z)`. Treating their product as the
    /// local 3-D density near the typical star's location, `λ ≈
    /// N·e^{-1}/(4πL_xy²L_z)`, as locally homogeneous, and reusing the
    /// homogeneous-process nearest-neighbor mean `E[R_nn] = Γ(4/3) /
    /// (λ·(4/3)π)^{1/3}` (Weibull(k=3) mean), solving `E[R_nn] =
    /// star_spacing_ly` for `N` gives a first-pass closed form, corrected by
    /// the same empirically-measured factor as before (~1.87× on `L`,
    /// applied here as `1.87³` on `N` since `N ∝ L³` at fixed spacing — `r =
    /// L_xy` is the radial marginal's peak, but a star actually there
    /// doesn't also sit at the Z-peak `z=0`, so the naive product overstates
    /// true local density) — see `tests` for the empirical check.
    ///
    /// **No longer capped** (confirmed this conversation, reversing the
    /// previous turn's `MAX_PLANET_COUNT`): *"lots of empty space between
    /// planets does not create drama and tension... keep the 7 ly mean
    /// spacing and reduce the size of a hex"* instead — the right
    /// `hex_side_ly` for a tractable, genuinely-7-ly-dense galaxy is a
    /// question for `examples/bench_hex_size.rs`'s measured throughput, not
    /// a cap on this method.
    pub fn derived_planet_count(&self) -> usize {
        /// Same empirical correction as before (`Hyades_habitability.md`-style
        /// honesty: measured, not derived), cubed since this solves for `N`
        /// rather than `L`.
        const CALIBRATION_CUBED: f64 = 1.87 * 1.87 * 1.87;
        let l_xy = self.xy_scale();
        let l_z = self.z_scale();
        let d = self.star_spacing_ly;
        let n = 3.0 * l_xy * l_xy * l_z * core::f64::consts::E * (GAMMA_4_3 / d).powi(3) * CALIBRATION_CUBED;
        (n.round() as usize).saturating_sub(self.players).max(10)
    }
}

/// Why generation refused a configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenError {
    /// `players` is not a vertex-transitive (fair) count.
    UnfairPlayerCount(usize),
}

impl core::fmt::Display for GenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GenError::UnfairPlayerCount(n) => {
                write!(f, "player count {n} has no vertex-transitive arrangement; fair counts are 2, 3, 6, 12")
            }
        }
    }
}

/// The whole generated galaxy: the planet field, the homeworld ids per seat, the
/// hotspots, and the pop bands. Pure data — no rendering, no hexes.
#[derive(Clone, Debug)]
pub struct Galaxy {
    pub planets: Vec<Planet>,
    /// `homeworlds[p]` is the [`PlanetId`] of player `p`'s homeworld.
    pub homeworlds: Vec<PlanetId>,
    pub hotspots: Hotspots,
    pub bands: PopBands,
    pub config: GalaxyConfig,
}

impl Galaxy {
    /// The fair (vertex-transitive) seat counts (§2).
    ///
    /// A hex ring at radius `r` holds exactly `6r` cells, so the ring family is
    /// **6, 12, 18, 24, …** — which is why 9 and 15 are *not* here despite being
    /// multiples of 3: neither forms a ring. Below the rings sit two special
    /// cases: the **tri-hex clique** at 3, and the **domino** at 2.
    ///
    /// **18 was missing (R-O12, resolved).** `starting_hex_radius` already
    /// carried an `18 => 4.5` branch, and all three of its ring radii are
    /// exactly `N/6 + 1.5`, so the branch was the third term of the family, not
    /// a stray — the list was simply truncated one term early.
    ///
    /// **Balance targets the 2-neighbour configurations** — 3, 6, 12, 18 — where
    /// every seat borders exactly two others. **N=2 is supported but is not a
    /// balance target**: the domino gives each player *one* neighbour, and the
    /// `p % 3` archetype cycle leaves it with Blue and Red and no Green (R-O9).
    /// Both are accepted consequences of a configuration nothing is tuned
    /// around, not defects to fix.
    pub const FAIR_COUNTS: [usize; 5] = [2, 3, 6, 12, 18];

    #[inline]
    pub fn planet(&self, id: PlanetId) -> &Planet {
        &self.planets[id.0 as usize]
    }

    #[inline]
    pub fn planet_mut(&mut self, id: PlanetId) -> &mut Planet {
        &mut self.planets[id.0 as usize]
    }

    /// Integer pop level 0–4 of a planet (§5.1).
    #[inline]
    pub fn pop_level(&self, id: PlanetId) -> u8 {
        self.bands.level(self.planet(id).population)
    }

    /// Generate a galaxy from a configuration.
    pub fn generate(config: GalaxyConfig) -> Result<Galaxy, GenError> {
        if !Galaxy::FAIR_COUNTS.contains(&config.players) {
            return Err(GenError::UnfairPlayerCount(config.players));
        }
        let mut rng = Rng::new(config.seed);
        let xy_scale = config.xy_scale();
        let z_scale = config.z_scale();
        let mean_xy = config.mean_xy_radius();

        // --- hue hotspots: 120° apart on a ring, random phase (§4.3) ---
        let hotspot_ring = mean_xy * config.hotspot_ring_frac;
        let hotspot_sigma = mean_xy * config.hotspot_sigma_frac;
        let phase = rng.range(0.0, core::f64::consts::TAU);
        let hotspot = |k: f64| {
            let a = phase + k * core::f64::consts::TAU / 3.0;
            Vec3::new(hotspot_ring * a.cos(), hotspot_ring * a.sin(), 0.0)
        };
        let hotspots = Hotspots { cyan: hotspot(0.0), magenta: hotspot(1.0), yellow: hotspot(2.0) };

        let mut planets: Vec<Planet> = Vec::with_capacity(config.planet_count + config.players);

        // --- wild field: XY radially Poisson, Z exponential (module doc) ---
        for i in 0..config.planet_count {
            let mut prng = rng.fork(0x5EED_0000 ^ i as u64);

            let position = sample_flattened_field(&mut prng, xy_scale, z_scale);

            // tier-1 density: Gaussian(XY to hue hotspot) × exp(−|z|/H), §4.3 —
            // same flattened shape as the star field itself.
            let z_decay = (-(position.z.abs()) / z_scale).exp();
            let mut minerals = MineralField::default();
            for b in Basic::ALL {
                let h = hotspots.get(b);
                let dx = position.x - h.x;
                let dy = position.y - h.y;
                let r2 = dx * dx + dy * dy;
                let g = (-r2 / (2.0 * hotspot_sigma * hotspot_sigma)).exp();
                // light multiplicative noise so the field isn't perfectly smooth
                let noise = (1.0 + 0.25 * prng.gaussian()).max(0.0);
                minerals.set(b, config.mineral_peak * g * z_decay * noise);
            }

            // §4.4 anticorrelation: normalize metallicity, depress habitability.
            let norm_met = (minerals.metallicity() / (3.0 * config.mineral_peak)).clamp(0.0, 1.0);
            let habitability =
                (4.0 * (1.0 - config.anticorrelation * norm_met) + 0.4 * prng.gaussian()).clamp(0.0, 4.0);
            // biosphere tracks habitability with its own spread.
            let biosphere = (habitability * prng.range(0.7, 1.1) + 0.3 * prng.gaussian()).clamp(0.0, 4.0);

            planets.push(Planet {
                id: PlanetId(i as u32),
                position,
                habitability,
                biosphere,
                infrastructure: 0.0, // wild
                minerals,
                is_homeworld: false,
                archetype: None,
                owner: None,
                population: 0.0,
            });
        }

        // --- homeworlds on a vertex-transitive ring (§2, §3) ---
        let homeworld_ring = mean_xy * config.homeworld_ring_frac;
        let mut homeworlds = Vec::with_capacity(config.players);
        for p in 0..config.players {
            let a = (p as f64) * core::f64::consts::TAU / (config.players as f64);
            let position = Vec3::new(homeworld_ring * a.cos(), homeworld_ring * a.sin(), 0.0);

            // rotational archetype assignment: B-R-G cycling (§3).
            let archetype = Archetype::ALL[p % 3];
            let (rich_a, rich_b, poor) = archetype.alignment();

            // super-aligned, bounded exception to anticorrelation: habitable AND
            // modestly mineralized in two colors (R-G4).
            let mut minerals = MineralField::default();
            minerals.set(rich_a, config.homeworld_rich_density);
            minerals.set(rich_b, config.homeworld_rich_density);
            minerals.set(poor, config.homeworld_poor_density);

            let id = PlanetId(planets.len() as u32);
            planets.push(Planet {
                id,
                position,
                habitability: 4.0, // identical 4 / 4 / 2 shape (§3, rev: infra 2)
                biosphere: 4.0,
                infrastructure: 2.0, // K = min = 2: the new starting development gate
                minerals,
                is_homeworld: true,
                archetype: Some(archetype),
                owner: Some(PlayerId(p as u32)),
                population: 2.0, // filled to its starting K
            });
            homeworlds.push(id);
        }

        Ok(Galaxy { planets, homeworlds, hotspots, bands: config.pop_bands(), config })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unfair_counts() {
        for n in [4usize, 5, 7, 8, 11] {
            assert_eq!(Galaxy::generate(GalaxyConfig::new(n, 1)).err(), Some(GenError::UnfairPlayerCount(n)));
        }
    }

    #[test]
    fn fair_counts_are_the_ring_family_plus_the_two_special_cases() {
        // A hex ring at radius r holds 6r cells, so the family is 6, 12, 18…
        // 9 and 15 are multiples of 3 but form no ring, which is why an
        // archetype-style `% 3` rule would be the wrong predicate here.
        for &n in &Galaxy::FAIR_COUNTS {
            assert!(n == 2 || n == 3 || n % 6 == 0, "{n} is neither a special case nor a 6r ring");
        }
        for n in [6usize, 12, 18] {
            assert!(Galaxy::FAIR_COUNTS.contains(&n), "ring count {n} must be fair");
        }
        for n in [4usize, 5, 7, 9, 15] {
            assert!(!Galaxy::FAIR_COUNTS.contains(&n), "{n} forms no vertex-transitive cluster");
        }
    }

    #[test]
    fn ring_radius_is_the_closed_form_that_replaced_the_magic_numbers() {
        // r + 1.5 where r = N/6, which is exactly what the old hand-written
        // table said for 6/12/18. Pinned so the family cannot drift back apart.
        for n in [6usize, 12, 18, 24] {
            let expected = (n / 6) as f64 + 1.5;
            let cfg = GalaxyConfig { players: n, ..GalaxyConfig::new(6, 1) };
            assert!(
                (cfg.hex_grid_radius() - (expected + cfg.hex_rings_beyond_start)).abs() < 1e-12,
                "N={n} radius drifted off the 6r closed form"
            );
        }
    }

    #[test]
    fn eighteen_seats_generate_a_symmetric_ring() {
        // R-O12: 18 is a 2-neighbour configuration and must place like one.
        let g = Galaxy::generate(GalaxyConfig::new(18, 1)).expect("18 is a fair count");
        assert_eq!(g.homeworlds.len(), 18);
        let hw: Vec<_> = g.homeworlds.iter().map(|&id| g.planet(id).position).collect();
        let d0 = hw[0].distance(hw[1]);
        for i in 0..hw.len() {
            let d = hw[i].distance(hw[(i + 1) % hw.len()]);
            assert!((d - d0).abs() < 1e-9, "ring spacing not uniform at seat {i}: {d} vs {d0}");
        }
    }

    #[test]
    fn accepts_fair_counts() {
        for n in Galaxy::FAIR_COUNTS {
            assert!(Galaxy::generate(GalaxyConfig::new(n, 1)).is_ok());
        }
    }

    #[test]
    fn homeworlds_are_identical_in_shape() {
        let g = Galaxy::generate(GalaxyConfig::new(6, 123)).unwrap();
        for &hw in &g.homeworlds {
            let p = g.planet(hw);
            assert_eq!(p.habitability, 4.0);
            assert_eq!(p.biosphere, 4.0);
            assert_eq!(p.infrastructure, 2.0);
            assert_eq!(p.k(), 2.0); // K = min = 2
        }
    }

    #[test]
    fn homeworld_starts_at_pop_level_two() {
        // K = 2, pop ~2 ⇒ the level-2 "limited vehicles" production gate (§5.1).
        let g = Galaxy::generate(GalaxyConfig::new(3, 7)).unwrap();
        for &hw in &g.homeworlds {
            assert_eq!(g.pop_level(hw), 2, "homeworld should start at level 2");
        }
    }

    #[test]
    fn pop_bands_span_zero_to_four() {
        let b = PopBands::default();
        assert_eq!(b.level(0.0), 0);
        assert_eq!(b.level(4.0), 4);
        // monotone non-decreasing
        let (mut prev, mut x) = (0u8, 0.0);
        while x <= 5.0 {
            let l = b.level(x);
            assert!(l >= prev);
            prev = l;
            x += 0.1;
        }
    }

    #[test]
    fn three_seats_get_one_of_each_archetype() {
        let g = Galaxy::generate(GalaxyConfig::new(3, 9)).unwrap();
        let mut kinds: Vec<Archetype> = g.homeworlds.iter().map(|&h| g.planet(h).archetype.unwrap()).collect();
        kinds.sort_by_key(|a| format!("{a:?}"));
        assert_eq!(kinds, vec![Archetype::BlueType, Archetype::GreenType, Archetype::RedType]);
    }

    #[test]
    fn metallicity_and_habitability_anticorrelate() {
        // Over the wild field, planets above median metallicity should have
        // lower mean habitability than those below (the §4.4 rule).
        let g = Galaxy::generate(GalaxyConfig::new(6, 2024)).unwrap();
        let wild: Vec<&Planet> = g.planets.iter().filter(|p| !p.is_homeworld).collect();
        let mut mets: Vec<f64> = wild.iter().map(|p| p.minerals.metallicity()).collect();
        mets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = mets[mets.len() / 2];
        let (mut hi_sum, mut hi_n, mut lo_sum, mut lo_n) = (0.0, 0, 0.0, 0);
        for p in &wild {
            if p.minerals.metallicity() >= median {
                hi_sum += p.habitability;
                hi_n += 1;
            } else {
                lo_sum += p.habitability;
                lo_n += 1;
            }
        }
        let hi_mean = hi_sum / hi_n as f64;
        let lo_mean = lo_sum / lo_n as f64;
        assert!(hi_mean < lo_mean, "metal-rich {hi_mean} !< metal-poor {lo_mean}");
    }

    /// Nearest-neighbor distance from every planet to its closest other planet
    /// (brute force — fine at N ~ 200 for a test).
    fn nearest_neighbor_distances(planets: &[Planet]) -> Vec<f64> {
        planets
            .iter()
            .map(|p| {
                planets
                    .iter()
                    .filter(|q| q.id != p.id)
                    .map(|q| p.position.distance(q.position))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect()
    }

    #[test]
    fn near_midplane_star_spacing_is_roughly_the_configured_mean() {
        // "Roughly 7 ly" per the spec, genuinely — the planet-count cap that
        // diluted this to "exceeds target" is gone (this conversation:
        // shrink the hex instead of capping density). Checked near the
        // midplane, since that's what the derivation actually targets (the
        // formula treats local density near the typical star's location as
        // locally homogeneous). Planets far from the plane are deliberately
        // sparser — that's the z-flattening working as intended.
        let g = Galaxy::generate(GalaxyConfig::new(6, 55)).unwrap();
        let target = g.config.star_spacing_ly;
        let scale = g.config.z_scale();
        let near_plane: Vec<Planet> = g.planets.iter().filter(|p| p.position.z.abs() < scale).cloned().collect();
        assert!(near_plane.len() > 20, "need enough near-plane planets for a meaningful mean");
        let dists = nearest_neighbor_distances(&near_plane);
        let mean = dists.iter().sum::<f64>() / dists.len() as f64;
        assert!(
            mean > target * 0.5 && mean < target * 2.0,
            "near-midplane mean nearest-neighbor {mean:.2} ly, target {target} ly"
        );
    }

    #[test]
    fn star_spacing_has_significant_variation_not_a_lattice() {
        // A genuine Poisson-derived process gives real spread; a lattice
        // would have ~zero variance. Just check the spread is clearly
        // nonzero, without over-fitting an exact distribution shape (the
        // field is no longer the clean homogeneous case that would predict
        // one, now that it's deliberately flattened).
        let g = Galaxy::generate(GalaxyConfig::new(6, 55)).unwrap();
        let dists = nearest_neighbor_distances(&g.planets);
        let mean = dists.iter().sum::<f64>() / dists.len() as f64;
        let var = dists.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / dists.len() as f64;
        let stddev = var.sqrt();
        assert!(stddev > mean * 0.15, "suspiciously little spread: mean={mean:.2} sd={stddev:.2}");
        let min = dists.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = dists.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(max > min * 2.0, "min {min:.2} / max {max:.2} too uniform for this field");
    }

    #[test]
    fn the_field_is_flattened_not_a_sphere() {
        // The actual design intent: don't let the autopilot find room
        // "vertically" — Z spread must be meaningfully tighter than XY
        // spread. The ratio is no longer a fixed 2:1 (it grows with the
        // hex-grid radius, hence with player count — more hexes spanning
        // XY, Z pinned to one hex-prism's depth regardless) — just check it
        // lands clearly on the flattened side, with margin for sampling
        // noise. Median/RMS, not max: Z's exponential tail is technically
        // unbounded, so a single rare outlier isn't a fair way to judge the
        // bulk of the distribution.
        let g = Galaxy::generate(GalaxyConfig::new(6, 55)).unwrap();
        let wild: Vec<&Planet> = g.planets.iter().filter(|p| !p.is_homeworld).collect();

        let mut xy: Vec<f64> = wild.iter().map(|p| (p.position.x.powi(2) + p.position.y.powi(2)).sqrt()).collect();
        let mut z_abs: Vec<f64> = wild.iter().map(|p| p.position.z.abs()).collect();
        xy.sort_by(|a, b| a.partial_cmp(b).unwrap());
        z_abs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_xy = xy[xy.len() / 2];
        let median_z = z_abs[z_abs.len() / 2];
        assert!(
            median_z < median_xy * 0.85,
            "field isn't flattened: median |z|={median_z:.1} vs median xy-radius={median_xy:.1}"
        );

        let xy_rms =
            (wild.iter().map(|p| p.position.x.powi(2) + p.position.y.powi(2)).sum::<f64>() / wild.len() as f64).sqrt();
        let z_rms = (wild.iter().map(|p| p.position.z.powi(2)).sum::<f64>() / wild.len() as f64).sqrt();
        assert!(z_rms < xy_rms, "RMS z-spread {z_rms:.1} should be less than RMS xy-spread {xy_rms:.1}");
    }

    #[test]
    fn hex_grid_and_scale_grow_with_player_count() {
        // "A game with more players will have more hexes" (this
        // conversation) — the hex grid radius, and therefore both physical
        // scales, must grow monotonically with player count across the fair
        // counts, not stay fixed or shrink.
        let mut prev_radius = 0.0;
        let mut prev_xy = 0.0;
        let mut prev_z = 0.0;
        for &n in &Galaxy::FAIR_COUNTS {
            let cfg = GalaxyConfig::new(n, 1);
            let radius = cfg.hex_grid_radius();
            // 2 and 3 players share the same "minimum start" cluster size
            // (both read as "3 hexes" per this conversation), so this is
            // non-decreasing, not strictly increasing, across every step.
            assert!(radius >= prev_radius, "hex_grid_radius should not shrink as player count grows (n={n})");
            assert!(cfg.xy_scale() >= prev_xy, "xy_scale should not shrink as player count grows (n={n})");
            prev_radius = radius;
            prev_xy = cfg.xy_scale();
            prev_z = cfg.z_scale();
        }
        // and it must grow at least once across the full span of fair counts.
        assert!(GalaxyConfig::new(18, 1).hex_grid_radius() > GalaxyConfig::new(2, 1).hex_grid_radius());
        // z_scale is pinned to one hex-prism's depth, independent of player
        // count — confirm it's the same across every fair count.
        let z0 = GalaxyConfig::new(2, 1).z_scale();
        for &n in &Galaxy::FAIR_COUNTS {
            assert!((GalaxyConfig::new(n, 1).z_scale() - z0).abs() < 1e-9);
        }
        let _ = prev_z;
    }

    #[test]
    fn xy_scale_stays_meaningfully_ahead_of_z_scale_at_every_fair_count() {
        // The actual regression this test guards against: an earlier
        // formula let z_scale rival or exceed xy_scale at small player
        // counts, undermining "don't let the autopilot find room
        // vertically." Confirmed fixed: XY should lead Z by a healthy
        // margin at every fair count, growing (not shrinking) as player
        // count rises.
        for &n in &Galaxy::FAIR_COUNTS {
            let cfg = GalaxyConfig::new(n, 1);
            let ratio = cfg.mean_xy_radius() / cfg.z_scale();
            assert!(ratio > 2.5, "n={n}: mean_xy_radius/z_scale = {ratio:.2}, should be well over 1");
        }
    }

    #[test]
    fn derived_planet_count_grows_uncapped_with_player_count() {
        // Corrected this conversation: no more artificial cap diluting
        // density ("lots of empty space... does not create drama and
        // tension") — hex_side_ly was shrunk instead (10 ly, measured
        // against real throughput in examples/bench_hex_size.rs). Star
        // count should now genuinely grow with the hex grid, uncapped.
        let small = GalaxyConfig::new(2, 1).derived_planet_count();
        let large = GalaxyConfig::new(18, 1).derived_planet_count();
        assert!(large > small, "should grow, not sit at a shared cap");
        // sanity: at the 10 ly default this should land in the thousands,
        // not the millions from the old (rejected) 100 ly default.
        assert!(small > 100 && small < 100_000);
        assert!(large > 100 && large < 200_000);
    }
}
