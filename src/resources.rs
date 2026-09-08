//! The material ladder (`Hyades_galaxy_and_autopilot.md` §4).
//!
//! Three tiers:
//! * **Tier 1 — basics: Cyan, Magenta, Yellow** — *mined* from the 3-D field.
//! * **Tier 2 — supers: Red, Green, Blue** — *synthesized*, never mined, only at
//!   pop-Band-IV, via fixed two-basic recipes (Blue←C+M, Red←M+Y, Green←Y+C).
//! * **Apex** — synthesized from supers; metallic silver-white. *"Platinum"* is a
//!   placeholder name (R-M1).
//!
//! Only **Tier-1 densities** matter for galaxy generation and the
//! colonization/growth autopilot; supers and apex are carried here so the same
//! types serve the later production/synthesis autopilots without a rewrite.

use crate::units::{Band, Kilotons, Measure};

/// Tier-1 basic minerals (the CMY primaries). Mined.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Basic {
    Cyan,
    Magenta,
    Yellow,
}

impl Basic {
    pub const ALL: [Basic; 3] = [Basic::Cyan, Basic::Magenta, Basic::Yellow];
}

/// Tier-2 super minerals (the RGB primaries). Synthesized at pop-Band-IV only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Super {
    Red,
    Green,
    Blue,
}

impl Super {
    pub const ALL: [Super; 3] = [Super::Red, Super::Green, Super::Blue];

    /// The fixed two-basic recipe (`Hyades_galaxy_and_autopilot.md` §4.2):
    /// `Blue ← C+M`, `Red ← M+Y`, `Green ← Y+C`.
    pub fn recipe(self) -> (Basic, Basic) {
        match self {
            Super::Blue => (Basic::Cyan, Basic::Magenta),
            Super::Red => (Basic::Magenta, Basic::Yellow),
            Super::Green => (Basic::Yellow, Basic::Cyan),
        }
    }
}

/// The three rotationally-symmetric homeworld archetypes
/// (`Hyades_galaxy_and_autopilot.md` §3). Each is rich in two basics, poor in
/// the third — the two precursors of its single native super.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archetype {
    /// Rich Cyan+Magenta, poor Yellow → native **Blue**. Militarist-expander-
    /// technologist (weak economy/diplomacy).
    BlueType,
    /// Rich Magenta+Yellow, poor Cyan → native **Red**. Tall industrial-military-
    /// political power (weak at spreading).
    RedType,
    /// Rich Yellow+Cyan, poor Magenta → native **Green**. Economic-expansionist
    /// (weak on the whole war-tech axis).
    GreenType,
}

impl Archetype {
    pub const ALL: [Archetype; 3] = [Archetype::BlueType, Archetype::RedType, Archetype::GreenType];

    /// The native super this archetype self-synthesizes at pop-Band-IV (exactly one).
    pub fn native_super(self) -> Super {
        match self {
            Archetype::BlueType => Super::Blue,
            Archetype::RedType => Super::Red,
            Archetype::GreenType => Super::Green,
        }
    }

    /// `(rich_a, rich_b, poor)` basics for this archetype.
    pub fn alignment(self) -> (Basic, Basic, Basic) {
        match self {
            Archetype::BlueType => (Basic::Cyan, Basic::Magenta, Basic::Yellow),
            Archetype::RedType => (Basic::Magenta, Basic::Yellow, Basic::Cyan),
            Archetype::GreenType => (Basic::Yellow, Basic::Cyan, Basic::Magenta),
        }
    }
}

/// Per-planet **density** of each tier-1 basic (the mineable field, §4.3). Not a
/// stockpile — a rate-determining ground truth a close scan reveals.
///
/// **One number per colour: the ore in the ground, in kilotons.** A Band is a
/// *reading* of that number — a log shorthand for talking about it — never a
/// second thing to store, and this field stores no Bands. That matters here
/// because the field **depletes**: hold a Band and write it back after each
/// extraction and ore vanishes at the bottom of the ladder, since the reading
/// floors at [`BAND_FLOOR`](crate::units::BAND_FLOOR). Measured on the first
/// attempt: three miners took 4.20× what one took instead of 3.00×. Mass is
/// conserved (L6), so the stored quantity is the mass.
///
/// **What T-62 changed is the distribution, not the storage.** The concentric
/// 2-D Gaussian of §4.3 is over the *Band* — that is the design statement, and
/// it makes the field log-normal in kilotons. A `Band IV` seam holds ~715,000×
/// a `Band I` one where the old linear reading made it 4×, which is the
/// concentration the design asks for: a handful of extraordinary worlds sitting
/// next to each other, not a gentle spread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MineralField {
    pub cyan: f64,
    pub magenta: f64,
    pub yellow: f64,
}

impl MineralField {
    #[inline]
    pub fn get(&self, b: Basic) -> Kilotons {
        Kilotons::new(match b {
            Basic::Cyan => self.cyan,
            Basic::Magenta => self.magenta,
            Basic::Yellow => self.yellow,
        })
    }

    #[inline]
    pub fn set(&mut self, b: Basic, v: Kilotons) {
        let v = v.kilotons();
        match b {
            Basic::Cyan => self.cyan = v,
            Basic::Magenta => self.magenta = v,
            Basic::Yellow => self.yellow = v,
        }
    }

    /// Total tier-1 ore in the ground, as a **mass**. The sum is meaningful
    /// precisely because these are kilotons; the same sum over Band positions
    /// would not be a quantity at all.
    #[inline]
    pub fn total_mass(&self) -> Kilotons {
        Kilotons::new(self.cyan + self.magenta + self.yellow)
    }

    /// The field's richness **read back onto the ladder** — a classification of
    /// how good this world is, on the same `0..4` scale as `habitability` and
    /// `biosphere`, and therefore the right thing for a threshold to compare
    /// against.
    ///
    /// Every consumer that asks "is this world rich?" wants this; every
    /// consumer that asks "how much ore comes out?" wants
    /// [`total_mass`](Self::total_mass). Same number, two readings — but since
    /// T-62 the readings differ by five orders of magnitude across the field,
    /// so which one a call site means is no longer a matter of taste.
    #[inline]
    pub fn abundance(&self) -> Band {
        self.total_mass().in_bands()
    }

    /// Extract up to `amount` of ore, **depleting** the field (density falls as
    /// minerals are mined out). Draws from each colour in proportion to its
    /// remaining mass and returns what was actually extracted as a cargo bank.
    /// A field mines out toward zero and then yields nothing.
    pub fn extract(&mut self, amount: Kilotons) -> Minerals {
        let total = self.total_mass().kilotons();
        let take = amount.kilotons().min(total).max(0.0);
        let mut out = Minerals::default();
        if total <= 0.0 || take <= 0.0 {
            return out;
        }
        let f = take / total;
        let (dc, dm, dy) = (self.cyan * f, self.magenta * f, self.yellow * f);
        self.cyan -= dc;
        self.magenta -= dm;
        self.yellow -= dy;
        out.cyan = dc;
        out.magenta = dm;
        out.yellow = dy;
        out
    }
}

/// A stockpile of every tier (basics, supers, apex). Carried by an empire.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Minerals {
    pub cyan: f64,
    pub magenta: f64,
    pub yellow: f64,
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    /// Apex ultra-resource (placeholder name *Platinum*), R-M1.
    pub apex: f64,
}

impl Minerals {
    #[inline]
    pub fn add_basic(&mut self, b: Basic, v: f64) {
        match b {
            Basic::Cyan => self.cyan += v,
            Basic::Magenta => self.magenta += v,
            Basic::Yellow => self.yellow += v,
        }
    }

    #[inline]
    pub fn get_basic(&self, b: Basic) -> f64 {
        match b {
            Basic::Cyan => self.cyan,
            Basic::Magenta => self.magenta,
            Basic::Yellow => self.yellow,
        }
    }

    /// Total tier-1 (basic) minerals on hand — the spendable pool for builds.
    #[inline]
    pub fn basic_total(&self) -> f64 {
        self.cyan + self.magenta + self.yellow
    }

    /// Fold another bank's basics into this one (cargo deposited at a center).
    #[inline]
    pub fn add_basics(&mut self, o: &Minerals) {
        self.cyan += o.cyan;
        self.magenta += o.magenta;
        self.yellow += o.yellow;
    }

    /// Try to spend `amount` total basic minerals, drawing from each color in
    /// proportion to how much is held. Returns `false` (and spends nothing) if
    /// the pool is short. Vehicle/infra costs flow through here.
    pub fn try_spend_total(&mut self, amount: f64) -> bool {
        if amount <= 0.0 {
            return true;
        }
        let total = self.basic_total();
        if total + 1e-9 < amount {
            return false;
        }
        let f = amount / total;
        self.cyan -= self.cyan * f;
        self.magenta -= self.magenta * f;
        self.yellow -= self.yellow * f;
        true
    }
}
