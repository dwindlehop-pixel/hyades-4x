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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MineralField {
    pub cyan: f64,
    pub magenta: f64,
    pub yellow: f64,
}

impl MineralField {
    #[inline]
    pub fn get(&self, b: Basic) -> f64 {
        match b {
            Basic::Cyan => self.cyan,
            Basic::Magenta => self.magenta,
            Basic::Yellow => self.yellow,
        }
    }

    #[inline]
    pub fn set(&mut self, b: Basic, v: f64) {
        match b {
            Basic::Cyan => self.cyan = v,
            Basic::Magenta => self.magenta = v,
            Basic::Yellow => self.yellow = v,
        }
    }

    /// Total tier-1 abundance — the planet's "metallicity" for the
    /// habitability↔metallicity anticorrelation (§4.4).
    #[inline]
    pub fn metallicity(&self) -> f64 {
        self.cyan + self.magenta + self.yellow
    }

    /// Extract up to `amount` total minerals, **depleting** the field (density
    /// falls as minerals are mined out). Draws from each colour in proportion to
    /// its remaining density and returns what was actually extracted as a cargo
    /// bank. A field mines out toward zero and then yields nothing.
    pub fn extract(&mut self, amount: f64) -> Minerals {
        let total = self.metallicity();
        let take = amount.min(total).max(0.0);
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
