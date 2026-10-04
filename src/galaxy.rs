//! Galaxy generation → the **continuous 3-D planet field** the simulation runs on.
//!
//! Per `Hyades_autopilot_colonization_growth.md` §1 the simulation has *no hexes*:
//! each star system is one point ("a planet"). This module produces exactly that
//! field plus the seeded homeworlds. The command-view hex tiling
//! (`Hyades_galaxy_and_autopilot.md` §1–2) is a *presentation* concern, and
//! nothing here reads it: [`GalaxyConfig::hex_side_ly`] (70 ly a side) is
//! carried for the command view only (R-G1). Ore color varies at the scale
//! of the **color sites** (§4.3) — randomly placed, with their own spacing and
//! width.
//!
//! The star field's **extent** is sized separately, by a **ring step** of
//! [`GalaxyConfig::ring_step_ly`] = 10 ly — the hex side before the hex was
//! resized, kept under its own name so the planet field (every position, and
//! the count) did not move when the hex did. The playable galaxy spans a
//! **ring radius** built from the fair-count starting cluster (§2's tri-hex
//! clique / ring / radius-`r` ring) plus **2–4 ring steps outward in each
//! direction**. [`GalaxyConfig::ring_radius`] turns that into one number;
//! [`GalaxyConfig::xy_scale`]/[`GalaxyConfig::z_scale`] turn *that* into the
//! two physical scale lengths below. Star **count** is not an independent
//! knob — it's *derived* from those scales plus the target local spacing
//! (kept exactly: *"lots of empty space between planets does not create drama
//! and tension... keep the 7 ly mean spacing"*), so more ring steps (more
//! players) means more stars at the same density, not the same stars spread
//! thinner.
//!
//! What the generator encodes from the world model:
//! * **Star positions — XY radially Poisson (exponential-disk profile), Z
//!   exponential**, independently scaled from the ring radius (previous turns
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
//!   own scale `L_z`, a multiple of the ring step. Mean *near-typical-
//!   radius* nearest-neighbor spacing is targeted at
//!   [`GalaxyConfig::star_spacing_ly`] (default 7 ly, matched to real
//!   interstellar spacing near a Sun-like star —
//!   [~5 ly](https://www.astronomy.com/science/how-close-can-stars-get-to-each-other-in-galaxy-cores/),
//!   [~0.004/ly³ ⇒ ~3.5–7 ly by method](https://en.wikipedia.org/wiki/Stellar_density))
//!   — see [`GalaxyConfig::derived_planet_count`] for the derivation, an
//!   approximation validated empirically in `tests`, not a closed form for
//!   the true inhomogeneous process.
//! * **§4.3 tier-1 field** — randomly placed color sites, each one hue weighted
//!   by the three hue hotspots, × exponential decay in Z, matching the star
//!   field's own shape.
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
use crate::transcendental;
use crate::units::{Band, BandTier, Kilotons, Measure};

/// `Γ(4/3)`, the mean-scaling constant for a Weibull(k=3) distribution — see
/// [`GalaxyConfig::derived_planet_count`]. `Γ(4/3) = (1/3)Γ(1/3)`.
const GAMMA_4_3: f64 = 0.892_979_511_569_249;

/// Sample a point from the flattened field: XY radius via `Gamma(shape=2,
/// scale=xy_scale)` (sum of two `Exponential` draws — the standard, simplest
/// exact sampler for that shape) with a uniform angle, giving the
/// exponential-disk radial profile; Z via a plain two-sided `Exponential`
/// at its own, independently-set `z_scale`.
fn sample_flattened_field(rng: &mut Rng, xy_scale: f64, z_scale: f64, arc: f64) -> Vec3 {
    let r = -xy_scale * (transcendental::ln(rng.unit().max(1e-12)) + transcendental::ln(rng.unit().max(1e-12)));
    let theta = rng.range(0.0, arc);
    let z = {
        let mag = -z_scale * transcendental::ln(rng.unit().max(1e-12));
        if rng.unit() < 0.5 {
            -mag
        } else {
            mag
        }
    };
    let (sin, cos) = transcendental::sin_cos(theta);
    Vec3::new(r * cos, r * sin, z)
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
/// `population` are all on the **same ladder** (`Hyades_simulation_model.md`
/// §2a): [`Band`] `0` ≈ empty, `4` ≈ many-billions. They are typed rather than
/// bare `f64` because the simulation also holds the biosphere as a *mass* —
/// see [`crate::units`] for why that distinction was load-bearing and silently
/// wrong.
///
/// `biosphere` is the **pristine** ceiling as generated. Once a galaxy is
/// loaded into a [`Simulation`](crate::sim::Simulation) the standing stock is
/// held in kilotons and can be drawn down; this field is the Band it started
/// at.
#[derive(Clone, Debug)]
pub struct Planet {
    pub id: PlanetId,
    pub position: Vec3,

    // --- carrying-capacity factors, on the Band ladder ---
    /// Hardest to change. The fundamental ceiling.
    pub habitability: Band,
    /// Easy to destroy, slow to improve.
    pub biosphere: Band,
    /// Built capacity. `0` on a wild world; raised by the build cycle. Soft
    /// factor and the early binding constraint (homeworlds start at 1).
    pub infrastructure: Band,

    /// Tier-1 mineral density (ground truth; a close scan reveals it).
    pub minerals: MineralField,

    pub is_homeworld: bool,
    /// `Some` only for homeworlds — fixes the seat's native super & alignment.
    pub archetype: Option<Archetype>,

    // --- mutable sim state ---
    pub owner: Option<PlayerId>,
    /// **The people living here, as a mass in kilotons.**
    ///
    /// Growth is logistic *in this number* — the ladder is a reading of it, for
    /// design and for print, never a second counting system the simulation
    /// steps in (T-64). `PopBands::level` is that reading.
    pub population: Kilotons,
}

impl Planet {
    /// Liebig carrying capacity `K = min(hab, bio, infra)`
    /// (`Hyades_simulation_model.md` §2a). A wild world (`infra = 0`) has `K = 0`.
    ///
    /// All three terms are Bands, which is the whole point of the type: the
    /// engine's copy of this minimum used to include the biosphere's *mass*.
    #[inline]
    pub fn k(&self) -> Band {
        self.habitability.min(self.biosphere).min(self.infrastructure)
    }

    /// The ceiling infrastructure (and thus population) can be *built* to:
    /// `min(hab, bio)` (autopilot-doc §3).
    #[inline]
    pub fn k_potential(&self) -> Band {
        self.habitability.min(self.biosphere)
    }
}

/// `√3`, written out: `f64::sqrt` is exact, but a constant needs no call.
const SQRT_3: f64 = 1.732_050_807_568_877_2;

/// A color site reaches a world only within this many of its widths: past
/// `4σ` its Gaussian is under 3.4e-4 of its peak, under 0.0014 Band.
const COLOR_SITE_REACH_SIGMAS: f64 = 4.0;

/// **Color sites** (§4.3; the author: color varies at the scale of an
/// empire). Placed at random, uniformly over the square the star field fits
/// in, one per [`GalaxyConfig::color_site_spacing_ly`]² of area. Each site is
/// one hue, drawn in proportion to the three hotspots' large-scale factors
/// there, with a peak of `mineral_peak · (floor + (1 − floor) · w / w_max)`:
/// `w` the hue's factor at the site and `w_max` the largest factor among the
/// sites of that hue, so each hue's strongest site reaches `mineral_peak` —
/// `Band IV` (R-O82) — wherever the draw falls relative to its hotspot. A
/// world's deposit in a hue is the Gaussian, of width
/// [`GalaxyConfig::color_site_sigma_ly`], of its strongest site of that hue.
/// The command-view hex plays no part. Generation only; nothing is stored.
struct ColorSites {
    sigma: f64,
    /// Side of a lookup bucket, ly — at least the reach, so a world's sites
    /// all lie in the 3×3 buckets around it.
    bucket: f64,
    /// Buckets per side is `2·half + 1`, centered on the origin.
    half: i64,
    /// Per bucket, the indices of the sites in it, in draw order.
    buckets: Vec<Vec<u32>>,
    /// Per site: its hue, position and peak Band.
    sites: Vec<(Basic, f64, f64, f64)>,
}

impl ColorSites {
    fn generate(
        config: &GalaxyConfig,
        hotspots: &Hotspots,
        hotspot_sigma: f64,
        mut rng: Rng,
        planted: &[(Basic, f64, f64, f64)],
    ) -> ColorSites {
        // The radial profile is Gamma(2, L_xy): ten scale lengths hold all
        // but ~5e-4 of the stars, and a world past the square reads trace.
        let extent = 10.0 * config.xy_scale();
        let spacing = config.color_site_spacing_ly.max(1e-6);
        let count = ((2.0 * extent) * (2.0 * extent) / (spacing * spacing)).round().max(3.0) as usize;
        let floor = config.color_site_floor.clamp(0.0, 1.0);
        let mut sites = Vec::with_capacity(count);
        for _ in 0..count {
            let x = rng.range(-extent, extent);
            let y = rng.range(-extent, extent);
            let w = Basic::ALL.map(|b| hotspots.factor(b, x, y, hotspot_sigma));
            let total: f64 = w.iter().sum();
            let mut pick = rng.unit() * total;
            let mut hue = Basic::ALL[2];
            for (k, &b) in Basic::ALL.iter().enumerate() {
                if pick < w[k] {
                    hue = b;
                    break;
                }
                pick -= w[k];
            }
            sites.push((hue, x, y, w[hue as usize]));
        }
        // Normalize each hue to its strongest site. With sites far apart the
        // nearest one can sit far from its hotspot, and the peak is a Band:
        // the one site that happened to land closest would hold most of the
        // galaxy's ore in one hue.
        let mut w_max = [0.0f64; 3];
        for &(hue, _, _, w) in &sites {
            w_max[hue as usize] = w_max[hue as usize].max(w);
        }
        for site in &mut sites {
            let top = w_max[site.0 as usize];
            let share = if top > 0.0 { site.3 / top } else { 0.0 };
            site.3 = config.mineral_peak * (floor + (1.0 - floor) * share);
        }
        // Planted sites (`Homeworlds::ColorCentered`) keep the peak they were
        // given: they are what guarantees a homeworld its threshold.
        sites.extend_from_slice(planted);
        let sigma = config.color_site_sigma_ly.max(1e-6);
        let bucket = COLOR_SITE_REACH_SIGMAS * sigma;
        let half = (extent / bucket).ceil() as i64;
        let width = 2 * half + 1;
        let mut buckets = vec![Vec::new(); (width * width) as usize];
        for (i, &(_, x, y, _)) in sites.iter().enumerate() {
            let (bx, by) = ((x / bucket).floor() as i64, (y / bucket).floor() as i64);
            if bx.abs() <= half && by.abs() <= half {
                buckets[((by + half) * width + (bx + half)) as usize].push(i as u32);
            }
        }
        ColorSites { sigma, bucket, half, buckets, sites }
    }

    /// Each hue's Band at `(x, y)` before the vertical decay and noise: its
    /// strongest site within reach.
    fn bands_at(&self, x: f64, y: f64) -> [f64; 3] {
        let width = 2 * self.half + 1;
        let (bx, by) = ((x / self.bucket).floor() as i64, (y / self.bucket).floor() as i64);
        let reach = COLOR_SITE_REACH_SIGMAS * self.sigma;
        let mut out = [0.0f64; 3];
        for cy in (by - 1)..=(by + 1) {
            for cx in (bx - 1)..=(bx + 1) {
                if cx.abs() > self.half || cy.abs() > self.half {
                    continue;
                }
                for &i in &self.buckets[((cy + self.half) * width + (cx + self.half)) as usize] {
                    let (hue, sx, sy, peak) = self.sites[i as usize];
                    let (dx, dy) = (x - sx, y - sy);
                    let d2 = dx * dx + dy * dy;
                    if d2 > reach * reach {
                        continue;
                    }
                    let band = peak * transcendental::exp(-d2 / (2.0 * self.sigma * self.sigma));
                    let k = hue as usize;
                    if band > out[k] {
                        out[k] = band;
                    }
                }
            }
        }
        out
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
    /// Hue `b`'s large-scale Gaussian factor at `(x, y)`.
    fn factor(&self, b: Basic, x: f64, y: f64, sigma: f64) -> f64 {
        let h = self.get(b);
        let (dx, dy) = (x - h.x, y - h.y);
        transcendental::exp(-(dx * dx + dy * dy) / (2.0 * sigma * sigma))
    }

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
    /// The four internal edges, **as masses**. They are *generated* as ladder
    /// positions — the Weibull quantiles are Gibrat-spaced, which is a
    /// statement about whole Bands — and stored as the population each edge stands
    /// for, so the comparison in [`PopBands::level`] is a comparison of people
    /// against people (T-64).
    pub edges: [Kilotons; 4],
}

impl PopBands {
    /// Build bands from a Weibull shape `k`, scaling so the top edge (the 0.8
    /// quantile → the level-3/4 boundary) lands on `top_edge`.
    pub fn from_weibull(k: f64, top_edge: f64) -> Self {
        let q = [0.2, 0.4, 0.6, 0.8];
        // weibull quantile: λ · (−ln(1−p))^(1/k); solve λ so q(0.8) == top_edge.
        let shape = |p: f64| transcendental::pow(-transcendental::ln(1.0 - p), 1.0 / k);
        let lambda = top_edge / shape(0.8);
        let mut edges = [Kilotons::ZERO; 4];
        for (i, &p) in q.iter().enumerate() {
            edges[i] = Kilotons::at_band(Band::new(lambda * shape(p)));
        }
        PopBands { edges }
    }

    /// Integer level 0–4 = how many band edges the population has crossed.
    ///
    /// Takes the **people**, because that is what the edges are now. The Band
    /// ladder is what generated them; it is not a second quantity to compare
    /// against.
    #[inline]
    pub fn level(&self, population: Kilotons) -> BandTier {
        // The edges are the *reached* thresholds, so the count of crossings is
        // the whole Band index. `BandTier::PLAYABLE` is indexed rather than matched so a
        // sixth whole Band cannot silently fall off the end.
        BandTier::PLAYABLE[self.edges.iter().filter(|&&e| population >= e).count()]
    }
}

impl Default for PopBands {
    fn default() -> Self {
        PopBands::from_weibull(1.4, 4.0)
    }
}

/// **The ground each seat starts on** (galaxy §2, T-147). On both identical
/// kinds one wedge of `1/N` of the disk is generated and turned to each of the
/// `N` seats, and the planet count is rounded down to a multiple of `N`;
/// homeworlds and their companions are the same on every kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// One field generated over the whole disk (the shipped default).
    Random,
    /// The wedge turned to every seat with the same colors, so each archetype
    /// starts beside the same deposits as every other. Any seat count above 1.
    Identical,
    /// The wedge turned to every seat with its colors stepped Cyan → Magenta →
    /// Yellow → Cyan once per seat, as the archetypes step (§3), so each seat's
    /// neighborhood is the next one's with its colors cycled. Seat counts that
    /// are multiples of 3.
    ColorRotated,
}

/// **How each seat's homeworld is made** (galaxy §3, T-147).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Homeworlds {
    /// The habitable world holds a trace of every basic, and beside it one
    /// companion world is rich in each of the archetype's two rich basics
    /// (the shipped default, `RATIFIED`).
    Trio,
    /// The habitable world alone, holding under `Band I` of every basic (each
    /// reading drawn uniformly in `[0, 1)`), with one color site of each hue
    /// planted at [`GalaxyConfig::homeworld_site_distance_ly`], 120° apart, at
    /// peak [`GalaxyConfig::homeworld_site_band`], and one outpost world of
    /// each hue on the bearing to its site at
    /// [`GalaxyConfig::homeworld_outpost_distance_ly`], holding
    /// [`GalaxyConfig::homeworld_outpost_band`] of that color alone (`OPEN`,
    /// T-147).
    ColorCentered,
}

/// Every tunable knob of galaxy generation. Defaults are placeholders pending
/// R-G/R-M/R-P; the MC balancer sweeps them.
#[derive(Clone, Copy, Debug)]
pub struct GalaxyConfig {
    /// Seat count. Must be a *fair* count (2, 3, 6, 12) — vertex-transitive (§2).
    pub players: usize,
    /// Number of wild (un-seeded) planets scattered in the field. Defaulted
    /// by [`Self::new`] via [`Self::derived_planet_count`] — the ring radius
    /// and target spacing are what size the galaxy, star count follows from
    /// them — but left as a
    /// plain mutable field, like everything else here, for direct override.
    pub planet_count: usize,

    /// **Hex side, ly — 70: the command view's hex, read by no engine code**
    /// (R-G1; the author: "only intended to be a human legible interface").
    /// Flat-top, one centered on the galactic center, `√3 · 70` = 121.2 ly
    /// across flats ([`Self::hex_across_flats_ly`]). From the author's target
    /// of hexes per player — 3–6 at 3 seats, 6–12 at 6 and 12, 3–6 at 18 —
    /// and a human-scale number; no width meets all four (`examples/hex_census`,
    /// appendix §D.32; R-G5, open). Nothing generated or simulated depends on
    /// it, so changing it changes no run.
    pub hex_side_ly: f64,
    /// **Ring step, ly — 10.** The length that sizes the star field's extent:
    /// [`Self::xy_scale`] is [`Self::ring_radius`] ring steps and
    /// [`Self::z_scale`] is [`Self::depth_multiple`] of them. It was the hex
    /// side until the hex was resized to the scale of an empire (R-G1); it
    /// keeps the value so the planet field — every position, and the count —
    /// is unchanged. 10 ly was chosen against measured throughput (at the time
    /// 2,116 yr/s at 12 seats, a figure `AGENTS.md` §7 records as stale).
    pub ring_step_ly: f64,
    /// Vertical scale length as a multiple of [`Self::ring_step_ly`] — `3`,
    /// carried from the prism depth `1×–5×` ("start `3×`") of the old 10-ly hex.
    pub depth_multiple: f64,
    /// How many ring steps the playable galaxy extends beyond the starting
    /// cluster, in each direction (*"at least two and maybe as many as four
    /// ... outward ... in each direction"* — 3 is the middle of that range).
    pub rings_beyond_start: f64,

    /// Target **mean near-typical-radius nearest-neighbor spacing** (ly) of
    /// the star field. Default 7 ly ([real interstellar spacing runs roughly
    /// 4–7 ly by method](https://www.astronomy.com/science/how-close-can-stars-get-to-each-other-in-galaxy-cores/)).
    /// No longer what sizes the galaxy (the ring radius does, above) — this
    /// now sizes [`Self::derived_planet_count`] instead, given the ring-derived
    /// scale, so more ring steps at the same target spacing means more stars,
    /// not the same stars spread thinner.
    pub star_spacing_ly: f64,

    /// Radius of the hue-hotspot ring, as a **fraction of the mean XY
    /// radius** (`2 · `[`Self::xy_scale`]`()`).
    pub hotspot_ring_frac: f64,
    /// Gaussian width of each hue hotspot, as a fraction of the mean XY radius.
    pub hotspot_sigma_frac: f64,
    /// **Peak tier-1 richness at a hotspot center, as a Band — `Band IV`,
    /// ratified** (R-O82).
    ///
    /// Since T-62 the §4.3 Gaussian is over *Bands*, so this is the top of the
    /// ladder rather than a linear density, and the field is log-normal in
    /// kilotons: the richest seams hold **~715,000×** what a `Band I` world
    /// does, where the old linear reading made it 4×. That is the design
    /// requirement — *"the game design requires very very high value planets
    /// located near each other"* — and it is deliberate, not a scale slip.
    ///
    /// **What it costs, ratified with eyes open.** One peak world holds ~715,500
    /// kt against a General hull costing 1.0 kt, the standard bed hauls **2,256×**
    /// the ore it did before, and none of that surplus bought a colony. The
    /// hauling is the engine's largest single cost (`examples/haul_census`:
    /// freighter transfers ×6.81 where vehicles rose ×1.20), and it is what puts
    /// the 12-seat × 8-kyr corner under T-24's throughput floor. **That is now an
    /// optimization problem, not a tuning one** (T-66) — `AGENTS.md` §7 is
    /// explicit that approaching the floor is the trigger to optimize rather
    /// than to shrink the scenario, and the scale is no longer available to
    /// shrink.
    pub mineral_peak: f64,

    /// **Mean spacing of the color sites, ly**: one site per spacing² of area,
    /// placed at random (§4.3). With the width below, it is the scale at which
    /// ore color varies — the author's ruling is that it varies at the scale
    /// of an empire. **Placeholder** (appendix §D.33).
    pub color_site_spacing_ly: f64,
    /// **Width of a color site's Gaussian, ly** (its σ, on the Band). At half
    /// the spacing, a world midway between two sites carries 0.61 of each
    /// peak and a world beside one carries one. **Placeholder** (§D.33).
    pub color_site_sigma_ly: f64,
    /// **The floor on a color site's peak**, as a fraction of `mineral_peak`.
    /// The hotspots' envelope falls to nothing across most of the disk, and
    /// with it every color, so a region far from the hue centers had no hue at
    /// all; with a floor every site carries its hue, and every region leans to
    /// one or two. A site's peak is `mineral_peak · (floor + (1 − floor) · w)`,
    /// `w` its hue's large-scale factor. **Placeholder.**
    pub color_site_floor: f64,

    /// Strength `∈ [0,1]` of the habitability↔metallicity anticorrelation
    /// (§4.4, R-M4). `0` = independent, `1` = metal-rich worlds are dead.
    pub anticorrelation: f64,

    /// Radius of the homeworld ring, as a fraction of the mean XY radius (§2).
    pub homeworld_ring_frac: f64,
    /// **A homeworld is a trio** (the author's ruling): the habitable world
    /// where the seat's population grows — and where its forge will stand —
    /// holding only a trace of ore, and two companion worlds beside it, one
    /// rich in each of the archetype's two rich basics. Every forge's
    /// precursors therefore arrive by freight (§4.5). This is each companion's
    /// Band in its one color; the other two are trace. **Placeholder.**
    pub homeworld_companion_density: f64,
    /// Distance from a homeworld to each companion, ly, tangential to the
    /// homeworld ring on either side. **Placeholder.**
    pub homeworld_companion_ly: f64,
    /// A companion's habitability and pristine biosphere, as a Band position —
    /// low, as the anticorrelation of §4.4 makes a mineral-rich world.
    /// **Placeholder.**
    pub homeworld_companion_habitability: f64,
    /// **A homeworld's habitability and pristine biosphere, as a Band
    /// position** — its carrying capacity `K` (industry §1.1).
    ///
    /// **`4.2`, the author's ruling**: a little past `Band IV`, so that true
    /// population `Band IV` — the synthesis gate — is reachable. At `4.0` the
    /// ceiling *was* the top population edge and the logistic approaches its
    /// ceiling without reaching it, so no world could ever forge. The target:
    /// a growth-dedicated build reaches `Band IV` before round two's card
    /// selection, most builds by round three. Measured on seeds 1, 7, 42 and
    /// 31337: card-free homeworlds cross at 632–685 yr, a Growth card at the
    /// first barrier brings seat 0 across at 473–482 yr (appendix §D.23).
    pub homeworld_ceiling: f64,

    /// Weibull shape for the pop bands (§5.1, R-P1).
    pub weibull_k: f64,

    /// **What ground each seat starts on** (the author's target: card-free, a
    /// standard deviation of about 20 colonies between empires). See
    /// [`Ground`]. **`OPEN`** (T-147).
    pub ground: Ground,

    /// **How each seat's homeworld is made.** See [`Homeworlds`]. **`OPEN`**
    /// (T-147).
    pub homeworlds: Homeworlds,
    /// Under [`Homeworlds::ColorCentered`], the distance from a homeworld to
    /// each of its three planted color sites, ly. **Placeholder.**
    pub homeworld_site_distance_ly: f64,
    /// Under [`Homeworlds::ColorCentered`], the peak (a Band reading) of each
    /// planted site — the least each color reaches beside a homeworld.
    /// **Placeholder** (`Band 3.0`, the trio companions' density).
    pub homeworld_site_band: f64,
    /// Under [`Homeworlds::ColorCentered`], the distance from a homeworld to
    /// each of its three planted outposts, ly — one per hue, on the bearing to
    /// that hue's planted site. **Placeholder.**
    pub homeworld_outpost_distance_ly: f64,
    /// Under [`Homeworlds::ColorCentered`], the deposit (a Band reading) of
    /// each planted outpost in its one color. **Placeholder** (`Band I`).
    pub homeworld_outpost_band: f64,
    /// **A homeworld's starting population**, a whole Band and a fraction of
    /// the way to the next: `Band II .785` (`RATIFIED`, the author's ruling,
    /// T-147), 1,076 kt. It sets the first forge's date — a forge stands at
    /// population `Band IV`, and growth is the logistic toward the
    /// homeworld's ceiling — at about 400 yr (appendix §D.42).
    pub homeworld_start_population: (BandTier, f64),

    pub seed: u64,
}

impl GalaxyConfig {
    /// A reasonable starting configuration for `players` seats.
    pub fn new(players: usize, seed: u64) -> Self {
        let mut cfg = GalaxyConfig {
            players,
            planet_count: 0, // set below, once the rest of self exists
            hex_side_ly: 70.0,
            ring_step_ly: 10.0,
            depth_multiple: 3.0,
            rings_beyond_start: 3.0,
            star_spacing_ly: 7.0,
            hotspot_ring_frac: 0.55,
            hotspot_sigma_frac: 0.42,
            mineral_peak: 4.0,
            color_site_spacing_ly: 10.0,
            color_site_sigma_ly: 5.0,
            color_site_floor: 0.5,
            anticorrelation: 0.7,
            homeworld_ring_frac: 0.5,
            homeworld_companion_density: 3.0,
            homeworld_companion_ly: 2.0,
            homeworld_companion_habitability: 0.5,
            homeworld_ceiling: 4.2,
            weibull_k: 1.4,
            ground: Ground::Random,
            homeworlds: Homeworlds::Trio,
            homeworld_site_distance_ly: 10.0,
            homeworld_site_band: 3.0,
            homeworld_outpost_distance_ly: 5.0,
            homeworld_outpost_band: 1.0,
            homeworld_start_population: (BandTier::II, 0.785),
            seed,
        };
        cfg.planet_count = cfg.derived_planet_count();
        cfg
    }

    /// How many turns of the wedge make the disk: `N` on identical ground (at
    /// a seat count that is a multiple of 3 when the colors step, since the
    /// step closes only after three seats), `1` otherwise.
    pub fn symmetry_turns(&self) -> usize {
        let closes = match self.ground {
            Ground::Random => false,
            Ground::Identical => self.players > 1,
            Ground::ColorRotated => self.players.is_multiple_of(3) && self.players > 0,
        };
        if closes {
            self.players
        } else {
            1
        }
    }

    pub fn pop_bands(&self) -> PopBands {
        PopBands::from_weibull(self.weibull_k, 4.0)
    }

    /// Radius (in ring steps) of the **starting cluster** for
    /// `players` seats — a *scale* reference, not the exact vertex-
    /// transitive topology from `Hyades_galaxy_and_autopilot.md` §2 (that's
    /// a command-view rendering concern, out of scope for sizing the
    /// continuous sim). *"Three hexes for a minimum player count start"*
    /// (this conversation) sets the floor; larger fair counts (6/12/18, the
    /// ring / radius-`r` ring configurations) get proportionally more.
    fn starting_ring_radius(players: usize) -> f64 {
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
            // poking `ring_radius` on an unvalidated config, and is
            // deliberately *not* the ring formula — at 18 it would say 3.95
            // against the ring's 4.5, so letting it serve the family would
            // silently mis-size the galaxy.
            n => 1.5 + ((n as f64) / 3.0).sqrt(),
        }
    }

    /// Total ring radius, in ring steps: starting cluster + the outward
    /// extension. *"A game with more players will have more hexes"* — a galaxy
    /// generation parameter, not a fixed constant.
    pub fn ring_radius(&self) -> f64 {
        Self::starting_ring_radius(self.players) + self.rings_beyond_start
    }

    /// Hex width across flats, ly: `√3 ·` [`Self::hex_side_ly`] — the spacing
    /// between neighboring hex centers.
    pub fn hex_across_flats_ly(&self) -> f64 {
        SQRT_3 * self.hex_side_ly
    }

    /// XY scale length `L_xy` (ly) for the `Gamma(2, L_xy)` radial profile —
    /// derived from the ring radius, **not** from star count (replacing the
    /// earlier count-derived approach): the ring radius converted straight to
    /// ly. (An earlier pass here
    /// divided by 2, reasoning `L_xy` as "half the mean reach" — that made
    /// `z_scale` rival or exceed this at small player counts, undermining
    /// the flattening `Hyades_vehicle_roles.md`-era "don't let empires find
    /// room vertically" goal; dropped in favor of the direct conversion,
    /// which keeps XY meaningfully ahead of Z at every fair player count —
    /// see `tests`.)
    pub fn xy_scale(&self) -> f64 {
        self.ring_radius() * self.ring_step_ly
    }

    /// Z scale length `L_z` (ly) for the two-sided `Exponential(L_z)`
    /// vertical profile — `ring_step_ly × depth_multiple`, independent of the
    /// XY scale (earlier tying both to one shared length was this module's own
    /// invention, not the hex spec, which defines depth on its own terms).
    pub fn z_scale(&self) -> f64 {
        self.ring_step_ly * self.depth_multiple
    }

    /// Mean XY radius (`2·L_xy`) — the natural "typical extent" reference
    /// for the hotspot/homeworld ring fractions.
    pub fn mean_xy_radius(&self) -> f64 {
        2.0 * self.xy_scale()
    }

    /// Star count that gives [`Self::star_spacing_ly`] average near-typical-
    /// radius nearest-neighbor spacing, **given** the ring-derived
    /// [`Self::xy_scale`]/[`Self::z_scale`] (an inversion of the derivation
    /// used before the ring radius became authoritative for scale — solving for
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
    /// spacing"* instead — the extent is set by [`Self::ring_step_ly`], chosen
    /// against measured throughput, not by a cap on this method.
    pub fn derived_planet_count(&self) -> usize {
        /// Same empirical correction as before (`Hyades_habitability.md`-style
        /// honesty: measured, not derived), cubed since this solves for `N`
        /// rather than `L`.
        const CALIBRATION_CUBED: f64 = 1.87 * 1.87 * 1.87;
        let l_xy = self.xy_scale();
        let l_z = self.z_scale();
        let d = self.star_spacing_ly;
        let n = 3.0
            * l_xy
            * l_xy
            * l_z
            * core::f64::consts::E
            * (GAMMA_4_3 / d)
            * (GAMMA_4_3 / d)
            * (GAMMA_4_3 / d)
            * CALIBRATION_CUBED;
        (n.round() as usize).saturating_sub(self.players).max(10)
    }
}

/// Why generation refused a configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenError {
    /// `players` is not a vertex-transitive (fair) count.
    UnfairPlayerCount(usize),
    /// A seeded fleet names a seat the galaxy does not have, moves at or past
    /// `c`, or is given no spend to build it with.
    BadFleet(usize),
}

impl core::fmt::Display for GenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GenError::UnfairPlayerCount(n) => {
                write!(f, "player count {n} has no vertex-transitive arrangement; fair counts are 2, 3, 6, 12")
            }
            GenError::BadFleet(i) => {
                write!(f, "seeded fleet {i} names no seat, moves at or past c, or has no spend")
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
    /// Fleets generated with the galaxy — none unless a bed asks for them.
    pub fleets: FleetSeeding,
    /// Under [`Homeworlds::ColorCentered`], `homeworld_sites[p]` is the
    /// position of the planted site of each color, in [`Basic`] order, around
    /// seat `p`'s homeworld; empty under [`Homeworlds::Trio`].
    pub homeworld_sites: Vec<[Vec3; 3]>,
}

/// **A fleet generated with the galaxy** (the author's ruling, T-133 follow-up:
/// "fleets can be optionally generated at Galaxy generation, with a position
/// and velocity"). How a test bed puts two fleets face to face without any
/// code the game does not run: the only thing a bed varies is the galaxy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeedFleet {
    /// The seat that owns it.
    pub seat: usize,
    /// Its Design: every hull of the fleet is one `(hull, class)`.
    pub hull: crate::sim::HullType,
    pub class: crate::sim::Class,
    /// The role its hulls are tasked with.
    pub role: crate::sim::Role,
    /// Where every hull of it starts, ly — its station-keeping spreads them.
    pub position: Vec3,
    /// Its coordinate velocity at the start, ly/yr (`c = 1`). A fleet under way
    /// sheds it at its own acceleration, the way any course change from a
    /// moving start does.
    pub velocity: Vec3,
}

/// **The fleets a galaxy is generated with, and what each may spend.** One
/// spend for every fleet (the author's ruling: equal mineral spend per
/// fleet), so a fleet's hull count is `round(spend_kt / dry mass)` — derived
/// by the engine, which is what knows a hull's mass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FleetSeeding {
    /// Minerals each fleet is built from, kt.
    pub spend_kt: f64,
    /// **A surveyed start**: each seat begins having scanned every world within
    /// this distance of its homeworld, ly. `0` — the default — is an ordinary
    /// start, where only the homeworld is known.
    pub known_radius_ly: f64,
    pub fleets: Vec<SeedFleet>,
    /// **Twin Designs paid in supers**: every seat starts with a twin of every
    /// Design — the same hull, class, mass and stats — billed `twin_bill`, and
    /// its yards build the twin wherever they can pay it (`Roster::twin`).
    /// `None` — the default — seeds none. A bed's way to give supers a final
    /// demand without a card (appendix §D.35).
    pub twin_bill: Option<crate::sim::DesignBill>,
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
    /// **Balance targets the 2-neighbor configurations** — 3, 6, 12, 18 — where
    /// every seat borders exactly two others. **N=2 is supported but is not a
    /// balance target**: the domino gives each player *one* neighbor, and the
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
    pub fn pop_level(&self, id: PlanetId) -> BandTier {
        self.bands.level(self.planet(id).population)
    }

    /// Generate a galaxy from a configuration.
    pub fn generate(config: GalaxyConfig) -> Result<Galaxy, GenError> {
        Self::generate_with(config, FleetSeeding::default())
    }

    /// [`Self::generate`], with fleets placed in the generated galaxy.
    pub fn generate_with(config: GalaxyConfig, fleets: FleetSeeding) -> Result<Galaxy, GenError> {
        for (i, f) in fleets.fleets.iter().enumerate() {
            // Written as the acceptable set so that a NaN anywhere is refused.
            let good = f.seat < config.players
                && f.velocity.norm() < 1.0
                && fleets.spend_kt > 0.0
                && f.position.norm().is_finite();
            let bad = !good;
            if bad {
                return Err(GenError::BadFleet(i));
            }
        }
        let mut galaxy = Self::generate_field(config)?;
        galaxy.fleets = fleets;
        Ok(galaxy)
    }

    fn generate_field(config: GalaxyConfig) -> Result<Galaxy, GenError> {
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
            let (sin, cos) = transcendental::sin_cos(a);
            Vec3::new(hotspot_ring * cos, hotspot_ring * sin, 0.0)
        };
        let hotspots = Hotspots { cyan: hotspot(0.0), magenta: hotspot(1.0), yellow: hotspot(2.0) };
        // **Color-centered homeworlds** (`Homeworlds::ColorCentered`): each
        // seat's homeworld stands on the ring between its neighbors' angles,
        // and one site of each hue is planted around it, equidistant and 120°
        // apart. On identical ground seat 0's are planted inside its wedge and
        // the wedge carries them to every seat — turned, and color-stepped on
        // `Ground::ColorRotated`.
        let turns = config.symmetry_turns();
        let homeworld_ring = mean_xy * config.homeworld_ring_frac;
        let centered: Vec<(Vec3, [Vec3; 3])> = if config.homeworlds == Homeworlds::ColorCentered {
            let sector = core::f64::consts::TAU / config.players as f64;
            let place = |p: usize| {
                let theta = (p as f64 + 0.5) * sector;
                let (sin, cos) = transcendental::sin_cos(theta);
                let home = Vec3::new(homeworld_ring * cos, homeworld_ring * sin, 0.0);
                let three: [Vec3; 3] = core::array::from_fn(|j| {
                    let (s3, c3) = transcendental::sin_cos(theta + j as f64 * core::f64::consts::TAU / 3.0);
                    let d = config.homeworld_site_distance_ly;
                    Vec3::new(home.x + d * c3, home.y + d * s3, 0.0)
                });
                (home, three)
            };
            if turns > 1 {
                let arc = core::f64::consts::TAU / turns as f64;
                let (home, three) = place(0);
                (0..config.players)
                    .map(|k| {
                        let (sin, cos) = transcendental::sin_cos(k as f64 * arc);
                        let turn = |v: Vec3| Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, v.z);
                        let shift = if config.ground == Ground::ColorRotated { k } else { 0 };
                        let mut turned = [Vec3::ZERO; 3];
                        for j in 0..3 {
                            turned[(j + shift) % 3] = turn(three[j]);
                        }
                        (turn(home), turned)
                    })
                    .collect()
            } else {
                (0..config.players).map(place).collect()
            }
        } else {
            Vec::new()
        };
        // Planted where generation reads them: every seat's on a random
        // ground, seat 0's alone where the wedge is turned.
        let planted: Vec<(Basic, f64, f64, f64)> = centered
            .iter()
            .take(if turns > 1 { 1 } else { centered.len() })
            .flat_map(|(_, three)| {
                Basic::ALL.iter().zip(three.iter()).map(|(&b, v)| (b, v.x, v.y, config.homeworld_site_band))
            })
            .collect();
        let sites = ColorSites::generate(&config, &hotspots, hotspot_sigma, rng.fork(0xC010_5173), &planted);

        let mut planets: Vec<Planet> = Vec::with_capacity(config.planet_count + 3 * config.players);

        // --- wild field: XY radially Poisson, Z exponential (module doc) ---
        // **Identical ground** (`GalaxyConfig::ground`): one wedge of `1/N` of
        // the disk is generated and turned to every seat, on
        // `Ground::ColorRotated` with its colors stepped once per seat the way
        // the archetypes step.
        let steps = config.ground == Ground::ColorRotated;
        let wedge = config.planet_count / turns;
        for i in 0..wedge {
            let mut prng = rng.fork(0x5EED_0000 ^ i as u64);

            let position = sample_flattened_field(&mut prng, xy_scale, z_scale, core::f64::consts::TAU / turns as f64);

            // tier-1 density: Gaussian(XY to hue hotspot) × exp(−|z|/H), §4.3 —
            // same flattened shape as the star field itself.
            let z_decay = transcendental::exp(-(position.z.abs()) / z_scale);
            let mut minerals = MineralField::default();
            let mut band_sum = 0.0;
            let peaks = sites.bands_at(position.x, position.y);
            // **One noise draw per world, added to every color** (a world's
            // richness wobbles as a whole): independent per-color noise gave
            // ore-poor worlds three similar traces and their regions no slant.
            // Three draws are still taken, so the stream after them is
            // unchanged.
            let draws = [prng.gaussian(), prng.gaussian(), prng.gaussian()];
            let common = 0.25 * draws[0];
            for b in Basic::ALL {
                // The hue's site field: its nearest site of that hue, whose
                // peak the large-scale hotspots set (§4.3).
                let g = peaks[b as usize] / config.mineral_peak.max(1e-12);
                // **The Gaussian is over Bands (T-62).** Density is a position
                // on the ladder, so the field is log-normal in mass: a
                // `Band IV` seam holds ~715,000× a `Band I` one, where the old
                // linear reading made it 4×. That concentration — a handful of
                // extraordinary worlds sitting next to each other — is the
                // design requirement the smooth 0..4 spread could not express.
                //
                // Noise is **additive on the Band**, which is the natural
                // wobble for a log-normal field: it is multiplicative in mass.
                // Multiplying the Band instead would put the noise in the
                // exponent. The amplitude is now a *Band* amplitude and is
                // therefore wider than the old `1 + 0.25·N` on density —
                // 0.25 Bands is a factor of ~2.4 in mass on the I→II segment —
                // which is the whole of T-62's residual effect on habitability
                // (§4.4 reads the mean Band, so the representation change
                // itself is neutral there).
                let band = (config.mineral_peak * g * z_decay + common).clamp(0.0, config.mineral_peak);
                minerals.set(b, Band::new(band).in_kilotons());
            }
            // **A world's total ore is its richest color, shared in the rolled
            // proportions** (the author's ruling): the three colors are
            // rescaled so they sum to the largest of them, keeping their
            // ratios. A one-color world barely changes; a balanced one keeps a
            // third, so fewer worlds clear `Band I` in any color.
            let kt = Basic::ALL.map(|b| minerals.get(b).kilotons());
            let total: f64 = kt.iter().sum();
            let cap = kt.iter().cloned().fold(0.0, f64::max);
            if total > 0.0 {
                for (i, &b) in Basic::ALL.iter().enumerate() {
                    minerals.set(b, Kilotons::new(cap * kt[i] / total));
                }
            }
            // §4.4 reads the deposit the world actually holds.
            for b in Basic::ALL {
                band_sum += minerals.get(b).band().bands().max(0.0);
            }

            // §4.4 anticorrelation: normalize richness, depress habitability.
            //
            // **The reading is the mean Band, not the Band of the total mass**
            // — i.e. the *geometric* mean of the three colors rather than the
            // arithmetic one. Both are legitimate classifications and they are
            // wildly different on a log ladder: the total-mass reading is
            // dominated by whichever color is richest, so a world at
            // `(II, I, Empty)` reads ~`II` instead of ~`I`, and at
            // `anticorrelation = 0.6` that is a whole extra Band of
            // habitability burned off every such world. Measured: routing
            // §4.4 through the total cost **−52% colony-years** on seed 1
            // (10,105,286 → 4,845,144), all of it habitability the galaxy
            // never had.
            //
            // The mean Band is also the reading that *survives* T-62 — it is
            // the same expression the old linear one computed, over the same
            // per-color numbers — so what remains of the habitability shift
            // is the noise model (below), not the distribution.
            let norm_met = (band_sum / (3.0 * config.mineral_peak)).clamp(0.0, 1.0);
            let habitability =
                (4.0 * (1.0 - config.anticorrelation * norm_met) + 0.4 * prng.gaussian()).clamp(0.0, 4.0);
            // biosphere tracks habitability with its own spread.
            let biosphere = (habitability * prng.range(0.7, 1.1) + 0.3 * prng.gaussian()).clamp(0.0, 4.0);
            let (habitability, biosphere) = (Band::new(habitability), Band::new(biosphere));

            for k in 0..turns {
                let (sin, cos) = transcendental::sin_cos(k as f64 * core::f64::consts::TAU / turns as f64);
                let at =
                    Vec3::new(position.x * cos - position.y * sin, position.x * sin + position.y * cos, position.z);
                // Seat `p + 1`'s archetype is seat `p`'s with every color
                // stepped Cyan → Magenta → Yellow → Cyan (§3), so on
                // `Ground::ColorRotated` the wedge turned `k` seats over
                // carries its colors stepped `k` times.
                let shift = if steps { k } else { 0 };
                let mut stepped = MineralField::default();
                for (j, &b) in Basic::ALL.iter().enumerate() {
                    stepped.set(Basic::ALL[(j + shift) % 3], minerals.get(b));
                }
                planets.push(Planet {
                    id: PlanetId(planets.len() as u32),
                    position: if turns == 1 { position } else { at },
                    habitability,
                    biosphere,
                    infrastructure: Band::ZERO, // wild
                    minerals: if turns == 1 { minerals } else { stepped },
                    is_homeworld: false,
                    archetype: None,
                    owner: None,
                    population: Kilotons::ZERO,
                });
            }
        }

        // --- homeworlds on a vertex-transitive ring (§2, §3) ---
        let mut homeworlds = Vec::with_capacity(config.players);
        // A color-centered homeworld's own deposit: each basic under `Band I`,
        // drawn once and turned with the wedge on identical ground.
        let deposit = |p: usize| {
            let mut hrng = rng.fork(0x40E3_0000 ^ p as u64);
            let mut m = MineralField::default();
            for b in Basic::ALL {
                m.set(b, Band::new(hrng.unit()).in_kilotons());
            }
            m
        };
        let home_deposits: Vec<MineralField> = (0..centered.len())
            .map(|k| {
                if turns == 1 {
                    return deposit(k);
                }
                let base = deposit(0);
                let shift = if config.ground == Ground::ColorRotated { k } else { 0 };
                let mut m = MineralField::default();
                for (j, &b) in Basic::ALL.iter().enumerate() {
                    m.set(Basic::ALL[(j + shift) % 3], base.get(b));
                }
                m
            })
            .collect();
        let homeworld_sites: Vec<[Vec3; 3]> = centered.iter().map(|c| c.1).collect();
        for p in 0..config.players {
            let a = (p as f64) * core::f64::consts::TAU / (config.players as f64);
            let (sin, cos) = transcendental::sin_cos(a);
            let ring_position = Vec3::new(homeworld_ring * cos, homeworld_ring * sin, 0.0);
            let position = centered.get(p).map_or(ring_position, |c| c.0);

            // rotational archetype assignment: B-R-G cycling (§3).
            let archetype = Archetype::ALL[p % 3];
            let (rich_a, rich_b, _) = archetype.alignment();

            // The habitable world of the trio holds a trace of every color: its
            // forge's precursors come from its companions, by freight. A
            // color-centered homeworld holds its own draw under `Band I`.
            let mut minerals = MineralField::default();
            for b in Basic::ALL {
                minerals.set(b, Band::ZERO.in_kilotons());
            }
            if let Some(m) = home_deposits.get(p) {
                minerals = *m;
            }

            let id = PlanetId(planets.len() as u32);
            planets.push(Planet {
                id,
                position,
                habitability: Band::new(config.homeworld_ceiling),
                biosphere: Band::new(config.homeworld_ceiling),
                infrastructure: Band::new(2.0), // K = min = 2: the new starting development gate
                minerals,
                is_homeworld: true,
                archetype: Some(archetype),
                owner: Some(PlayerId(p as u32)),
                population: Kilotons::at(config.homeworld_start_population.0, config.homeworld_start_population.1),
            });
            homeworlds.push(id);

            // The two companions, one per rich basic, either side of the
            // homeworld along the ring.
            let tangent = Vec3::new(-sin, cos, 0.0);
            // The worlds beside the homeworld, each rich in one color: the
            // trio's two companions along the ring, or a color-centered
            // homeworld's three planted outposts — one per hue, on the bearing
            // to that hue's planted site, at `homeworld_outpost_distance_ly`.
            let beside: Vec<(Vec3, Basic, f64)> = match centered.get(p) {
                None => [(1.0, rich_a), (-1.0, rich_b)]
                    .map(|(side, rich)| {
                        let at = Vec3::new(
                            position.x + side * config.homeworld_companion_ly * tangent.x,
                            position.y + side * config.homeworld_companion_ly * tangent.y,
                            0.0,
                        );
                        (at, rich, config.homeworld_companion_density)
                    })
                    .to_vec(),
                Some((home, three)) => Basic::ALL
                    .iter()
                    .zip(three.iter())
                    .map(|(&hue, site)| {
                        let (dx, dy) = (site.x - home.x, site.y - home.y);
                        let d = (dx * dx + dy * dy).sqrt().max(1e-12);
                        let r = config.homeworld_outpost_distance_ly / d;
                        (Vec3::new(home.x + dx * r, home.y + dy * r, 0.0), hue, config.homeworld_outpost_band)
                    })
                    .collect(),
            };
            for (at, rich, density) in beside {
                let mut minerals = MineralField::default();
                for b in Basic::ALL {
                    let band = if b == rich { density } else { 0.0 };
                    minerals.set(b, Band::new(band).in_kilotons());
                }
                planets.push(Planet {
                    id: PlanetId(planets.len() as u32),
                    position: at,
                    habitability: Band::new(config.homeworld_companion_habitability),
                    biosphere: Band::new(config.homeworld_companion_habitability),
                    infrastructure: Band::ZERO,
                    minerals,
                    is_homeworld: false,
                    archetype: None,
                    owner: None,
                    population: Kilotons::ZERO,
                });
            }
        }

        Ok(Galaxy {
            planets,
            homeworlds,
            hotspots,
            bands: config.pop_bands(),
            config,
            fleets: FleetSeeding::default(),
            homeworld_sites,
        })
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
                (cfg.ring_radius() - (expected + cfg.rings_beyond_start)).abs() < 1e-12,
                "N={n} radius drifted off the 6r closed form"
            );
        }
    }

    #[test]
    fn resizing_the_hex_left_the_planet_field_alone() {
        // R-G1: the hex grew to the scale of an empire and the star field kept
        // its extent and its count — the ring step is the old 10-ly hex side.
        for (n, count) in [(2usize, 6723usize), (3, 6722), (6, 10041), (12, 14020), (18, 18664)] {
            let cfg = GalaxyConfig::new(n, 1);
            assert_eq!(cfg.planet_count, count, "{n} seats");
            assert_eq!(cfg.z_scale(), 30.0);
        }
        assert_eq!(GalaxyConfig::new(3, 1).xy_scale(), 45.0);
        assert_eq!(GalaxyConfig::new(18, 1).xy_scale(), 75.0);
    }

    #[test]
    fn identical_ground_turns_one_wedge_to_every_seat_and_steps_colors_only_when_asked() {
        for (ground, seats) in [(Ground::ColorRotated, 3), (Ground::Identical, 3), (Ground::Identical, 2)] {
            let cfg = GalaxyConfig { ground, ..GalaxyConfig::new(seats, 9) };
            let g = Galaxy::generate(cfg).unwrap();
            let wild: Vec<&Planet> = g.planets.iter().filter(|p| !p.is_homeworld && p.archetype.is_none()).collect();
            assert_eq!(cfg.symmetry_turns(), seats);
            let wedge = cfg.planet_count / seats;
            for i in 0..wedge {
                let base = wild[seats * i];
                for k in 1..seats {
                    let p = wild[seats * i + k];
                    let (sin, cos) = transcendental::sin_cos(k as f64 * core::f64::consts::TAU / seats as f64);
                    let x = base.position.x * cos - base.position.y * sin;
                    let y = base.position.x * sin + base.position.y * cos;
                    assert!((p.position.x - x).abs() < 1e-9 && (p.position.y - y).abs() < 1e-9);
                    assert_eq!(p.position.z, base.position.z);
                    assert_eq!(p.habitability.bands(), base.habitability.bands());
                    let shift = if ground == Ground::ColorRotated { k } else { 0 };
                    for (j, &b) in Basic::ALL.iter().enumerate() {
                        let stepped = Basic::ALL[(j + shift) % 3];
                        assert_eq!(p.minerals.get(stepped).kilotons(), base.minerals.get(b).kilotons());
                    }
                }
            }
        }
        // Random by default, and the colors cannot step round two seats.
        assert_eq!(GalaxyConfig::new(3, 9).symmetry_turns(), 1);
        assert_eq!(GalaxyConfig { ground: Ground::ColorRotated, ..GalaxyConfig::new(2, 9) }.symmetry_turns(), 1);
    }

    #[test]
    fn a_color_centered_homeworld_stands_alone_among_one_site_of_each_color() {
        for ground in [Ground::Random, Ground::Identical, Ground::ColorRotated] {
            let cfg = GalaxyConfig { ground, homeworlds: Homeworlds::ColorCentered, ..GalaxyConfig::new(3, 9) };
            let g = Galaxy::generate(cfg).unwrap();
            // No companions: the wild ground, the homeworlds, and three planted
            // outposts per seat.
            let wild = cfg.planet_count / cfg.symmetry_turns() * cfg.symmetry_turns();
            assert_eq!(g.planets.len(), wild + 3 * 4, "{ground:?}");
            assert_eq!(g.homeworld_sites.len(), 3);
            let home = |p: usize| &g.planets[g.homeworlds[p].0 as usize];
            for p in 0..3 {
                for b in Basic::ALL {
                    assert!(home(p).minerals.get(b).kilotons() < 1.0, "{ground:?} seat {p} holds Band I of {b:?}");
                }
                for site in g.homeworld_sites[p] {
                    let d = site.distance(home(p).position);
                    assert!((d - cfg.homeworld_site_distance_ly).abs() < 1e-9, "{ground:?} seat {p}: {d}");
                }
                // One outpost of each hue, on the bearing to that hue's site,
                // holding `Band I` of its color and the floor of the others.
                let id = g.homeworlds[p].0 as usize;
                for (j, &b) in Basic::ALL.iter().enumerate() {
                    let o = &g.planets[id + 1 + j];
                    assert!((o.position.distance(home(p).position) - cfg.homeworld_outpost_distance_ly).abs() < 1e-9);
                    let toward = o.position.distance(g.homeworld_sites[p][j]);
                    let d = cfg.homeworld_site_distance_ly - cfg.homeworld_outpost_distance_ly;
                    assert!((toward - d).abs() < 1e-9, "{ground:?} seat {p} outpost {j} off its bearing");
                    assert!((o.minerals.get(b).band().bands() - cfg.homeworld_outpost_band).abs() < 1e-6);
                    for &other in Basic::ALL.iter().filter(|&&c| c != b) {
                        assert!(o.minerals.get(other).kilotons() < 0.01);
                    }
                }
            }
            if ground == Ground::Random {
                continue;
            }
            // Identical ground turns seat 0's homeworld and sites to every
            // seat, colors stepped on `ColorRotated`.
            for k in 1..3 {
                let (sin, cos) = transcendental::sin_cos(k as f64 * core::f64::consts::TAU / 3.0);
                let turn = |v: Vec3| Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, v.z);
                assert!(turn(home(0).position).distance(home(k).position) < 1e-9);
                let shift = if ground == Ground::ColorRotated { k } else { 0 };
                for (j, &b) in Basic::ALL.iter().enumerate() {
                    let stepped = Basic::ALL[(j + shift) % 3];
                    assert!(turn(g.homeworld_sites[0][j]).distance(g.homeworld_sites[k][(j + shift) % 3]) < 1e-9);
                    assert_eq!(home(k).minerals.get(stepped).kilotons(), home(0).minerals.get(b).kilotons());
                }
            }
        }
        // The default is the trio.
        assert_eq!(GalaxyConfig::new(3, 9).homeworlds, Homeworlds::Trio);
        assert!(Galaxy::generate(GalaxyConfig::new(3, 9)).unwrap().homeworld_sites.is_empty());
    }

    #[test]
    fn the_hex_is_read_by_no_generation() {
        // R-G1: the hex is a human-legible interface. Changing it changes no
        // world.
        let cfg = GalaxyConfig::new(3, 7);
        assert_eq!(cfg.hex_side_ly, 70.0);
        assert!((cfg.hex_across_flats_ly() - 121.243_556_529_821_4).abs() < 1e-9);
        let a = Galaxy::generate(cfg).unwrap();
        let b = Galaxy::generate(GalaxyConfig { hex_side_ly: 13.0, ..cfg }).unwrap();
        for (p, q) in a.planets.iter().zip(&b.planets) {
            assert_eq!(p.position, q.position);
            for m in Basic::ALL {
                assert_eq!(p.minerals.get(m).kilotons().to_bits(), q.minerals.get(m).kilotons().to_bits());
            }
            assert_eq!(p.habitability.bands().to_bits(), q.habitability.bands().to_bits());
        }
    }

    #[test]
    fn a_world_reads_its_strongest_site_of_each_hue_within_reach() {
        let mut cfg = GalaxyConfig::new(3, 3);
        cfg.color_site_spacing_ly = 40.0;
        cfg.color_site_sigma_ly = 25.0;
        let g = Galaxy::generate(cfg).unwrap();
        let sigma = cfg.mean_xy_radius() * cfg.hotspot_sigma_frac;
        let sites = ColorSites::generate(&cfg, &g.hotspots, sigma, Rng::new(3), &[]);
        let mut rng = Rng::new(11);
        for _ in 0..300 {
            let (x, y) = (rng.range(-200.0, 200.0), rng.range(-200.0, 200.0));
            // Brute force over every site.
            let mut want = [0.0f64; 3];
            for &(hue, sx, sy, peak) in &sites.sites {
                let d2 = (x - sx) * (x - sx) + (y - sy) * (y - sy);
                if d2 <= (4.0 * 25.0) * (4.0 * 25.0) {
                    let band = peak * transcendental::exp(-d2 / (2.0 * 25.0 * 25.0));
                    want[hue as usize] = want[hue as usize].max(band);
                }
            }
            assert_eq!(sites.bands_at(x, y), want, "at ({x}, {y})");
        }
    }

    #[test]
    fn every_hue_reaches_the_peak_at_its_strongest_site() {
        // R-O82 at any spacing: each hue's strongest site is `mineral_peak`
        // wherever the draw falls relative to its hotspot.
        for seed in [1u64, 7, 42] {
            let g = Galaxy::generate(GalaxyConfig::new(3, seed)).unwrap();
            let cfg = g.config;
            let sigma = cfg.mean_xy_radius() * cfg.hotspot_sigma_frac;
            // Any stream: the property holds for every draw.
            let sites = ColorSites::generate(&cfg, &g.hotspots, sigma, Rng::new(seed), &[]);
            let mut top = [0.0f64; 3];
            for &(hue, _, _, peak) in &sites.sites {
                top[hue as usize] = top[hue as usize].max(peak);
            }
            for (k, t) in top.iter().enumerate() {
                assert!((t - cfg.mineral_peak).abs() < 1e-12, "seed {seed} hue {k} peaks at {t}");
            }
        }
    }

    #[test]
    fn eighteen_seats_generate_a_symmetric_ring() {
        // R-O12: 18 is a 2-neighbor configuration and must place like one.
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
        let ceiling = GalaxyConfig::new(6, 123).homeworld_ceiling;
        assert!(ceiling > 4.0, "population Band IV must be reachable below the ceiling");
        for &hw in &g.homeworlds {
            let p = g.planet(hw);
            assert_eq!(p.habitability, Band::new(ceiling));
            assert_eq!(p.biosphere, Band::new(ceiling));
            assert_eq!(p.infrastructure, Band::new(2.0));
            assert_eq!(p.k(), Band::new(2.0)); // K = min = 2
        }
    }

    #[test]
    fn a_homeworld_starts_at_band_ii_785() {
        // The ratified start (T-147): `Band II .785`, 1,076 kt, which the
        // Weibull population bands read as level III — so a homeworld clears
        // the level-III build gate from the start.
        let g = Galaxy::generate(GalaxyConfig::new(3, 7)).unwrap();
        for &hw in &g.homeworlds {
            assert_eq!(g.planet(hw).population, Kilotons::at(BandTier::II, 0.785));
            assert!((g.planet(hw).population.kilotons() - 1076.373).abs() < 1e-3);
            assert_eq!(g.pop_level(hw), BandTier::III);
        }
    }

    #[test]
    fn pop_bands_span_zero_to_four() {
        let b = PopBands::default();
        assert_eq!(b.level(Kilotons::ZERO), BandTier::Empty);
        assert_eq!(b.level(Kilotons::at_band(Band::new(4.0))), BandTier::IV);
        // monotone non-decreasing
        let (mut prev, mut x) = (BandTier::Empty, 0.0);
        while x <= 5.0 {
            let l = b.level(Kilotons::at_band(Band::new(x)));
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
        let mut mets: Vec<f64> = wild.iter().map(|p| p.minerals.abundance().bands()).collect();
        mets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = mets[mets.len() / 2];
        let (mut hi_sum, mut hi_n, mut lo_sum, mut lo_n) = (0.0, 0, 0.0, 0);
        for p in &wild {
            if p.minerals.abundance().bands() >= median {
                hi_sum += p.habitability.bands();
                hi_n += 1;
            } else {
                lo_sum += p.habitability.bands();
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
        let var = dists.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / dists.len() as f64;
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
        // ring radius, hence with player count — more ring steps spanning
        // XY, Z pinned to a multiple of the ring step regardless) — just check it
        // lands clearly on the flattened side, with margin for sampling
        // noise. Median/RMS, not max: Z's exponential tail is technically
        // unbounded, so a single rare outlier isn't a fair way to judge the
        // bulk of the distribution.
        let g = Galaxy::generate(GalaxyConfig::new(6, 55)).unwrap();
        let wild: Vec<&Planet> = g.planets.iter().filter(|p| !p.is_homeworld).collect();

        let mut xy: Vec<f64> =
            wild.iter().map(|p| (p.position.x * p.position.x + p.position.y * p.position.y).sqrt()).collect();
        let mut z_abs: Vec<f64> = wild.iter().map(|p| p.position.z.abs()).collect();
        xy.sort_by(|a, b| a.partial_cmp(b).unwrap());
        z_abs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_xy = xy[xy.len() / 2];
        let median_z = z_abs[z_abs.len() / 2];
        assert!(
            median_z < median_xy * 0.85,
            "field isn't flattened: median |z|={median_z:.1} vs median xy-radius={median_xy:.1}"
        );

        let xy_rms = (wild.iter().map(|p| p.position.x * p.position.x + p.position.y * p.position.y).sum::<f64>()
            / wild.len() as f64)
            .sqrt();
        let z_rms = (wild.iter().map(|p| p.position.z * p.position.z).sum::<f64>() / wild.len() as f64).sqrt();
        assert!(z_rms < xy_rms, "RMS z-spread {z_rms:.1} should be less than RMS xy-spread {xy_rms:.1}");
    }

    #[test]
    fn ring_radius_and_scale_grow_with_player_count() {
        // "A game with more players will have more hexes" — the ring radius,
        // and therefore both physical
        // scales, must grow monotonically with player count across the fair
        // counts, not stay fixed or shrink.
        let mut prev_radius = 0.0;
        let mut prev_xy = 0.0;
        let mut prev_z = 0.0;
        for &n in &Galaxy::FAIR_COUNTS {
            let cfg = GalaxyConfig::new(n, 1);
            let radius = cfg.ring_radius();
            // 2 and 3 players share the same "minimum start" cluster size
            // (both read as "3 hexes" per this conversation), so this is
            // non-decreasing, not strictly increasing, across every step.
            assert!(radius >= prev_radius, "ring_radius should not shrink as player count grows (n={n})");
            assert!(cfg.xy_scale() >= prev_xy, "xy_scale should not shrink as player count grows (n={n})");
            prev_radius = radius;
            prev_xy = cfg.xy_scale();
            prev_z = cfg.z_scale();
        }
        // and it must grow at least once across the full span of fair counts.
        assert!(GalaxyConfig::new(18, 1).ring_radius() > GalaxyConfig::new(2, 1).ring_radius());
        // z_scale is pinned to a multiple of the ring step, independent of player
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
        // tension") — the sizing length was shrunk instead (10 ly, measured
        // against throughput; now `ring_step_ly`). Star count should
        // genuinely grow with the ring radius, uncapped.
        let small = GalaxyConfig::new(2, 1).derived_planet_count();
        let large = GalaxyConfig::new(18, 1).derived_planet_count();
        assert!(large > small, "should grow, not sit at a shared cap");
        // sanity: at the 10 ly default this should land in the thousands,
        // not the millions from the old (rejected) 100 ly default.
        assert!(small > 100 && small < 100_000);
        assert!(large > 100 && large < 200_000);
    }
}
