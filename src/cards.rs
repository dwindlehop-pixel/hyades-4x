//! **The card layer — tier-0 placeholders and the one inbound channel.**
//!
//! `Hyades_standing_layer_and_observation.md` §0 makes every card a *tree
//! card*, and makes Doctrine and Design **state written only by tree cards**.
//! That is the shape here: a card's whole job is to write standing-layer state,
//! and the three things it can write are exactly the three the engine already
//! stores — [`Doctrine`], the [`Roster`](crate::sim::Roster), and per-player
//! knowledge.
//!
//! ## What is real and what is a placeholder
//!
//! **Real:** card identity, the six trees, the three slants, mineral cost,
//! targets, legality, the coercion rule, and three effect families that write
//! live engine state ([`CardEffect::DiscloseScans`],
//! [`CardEffect::WriteDoctrine`], [`CardEffect::UnlockDesign`]).
//!
//! **Placeholder:** the *flavour names, costs, and the specific effect assigned
//! to each of the 18 tier-0 slots.* Card text is the author's own and none of
//! it is written yet; `TIER0` is scaffolding that makes the layer testable, not
//! a proposal about what the cards should be. Effects that need systems the
//! engine does not have — combat, the Exchange, `$` — are
//! [`CardEffect::NotYetImplemented`] rather than silently doing nothing, so a
//! run can *count* how many plays were inert.
//!
//! ## The coercion rule (net §5.1)
//!
//! An illegal order is **replaced by the default order, never rejected**.
//! Legality is a pure function of state every client already has, so every
//! client coerces identically with no message exchanged. Rejecting instead of
//! coercing is how a lockstep system desyncs. [`Order::coerce`] is that rule,
//! and it is the only place it lives.
//!
//! ## Politics cards are not opt-in
//!
//! `Hyades_politics_trade_and_intelligence.md` §6: an opponent may initiate
//! trade or shared intelligence **without your consent**. [`Target::Player`]
//! therefore names a *subject*, not a partner, and no handshake exists anywhere
//! in this module. That is deliberate — a consent channel would be a second
//! inbound path across the presentation seam (design law #15), and the whole
//! anti-collusion thesis (§0 of that spec) depends on these effects being
//! purchasable alone.

use crate::autopilot::Doctrine;
use crate::galaxy::PlayerId;
use crate::sim::{Class, HullType};

/// The six trees (`Hyades_command_cards.md` §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tree {
    Warfare,
    Production,
    Politics,
    Growth,
    Expansion,
    Technology,
}

impl Tree {
    pub const ALL: [Tree; 6] =
        [Tree::Warfare, Tree::Production, Tree::Politics, Tree::Growth, Tree::Expansion, Tree::Technology];
}

/// Where a tier-0 card sits on the slant axis (std §2).
///
/// Slant is **σ read from the other side of the table** (design law #9), so
/// this is not a power level — it is how much the play tells a watcher about
/// whether you meant it. The σ→value curve must stay **convex** or everyone
/// opens inscrutable and the yomi channel carries nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slant {
    /// Useful whether or not you descend this tree. Low σ.
    Inscrutable,
    /// Partly conditional. Mid σ.
    Balanced,
    /// Strongly conditional — playing it and not continuing is a real loss.
    LessGuarded,
}

/// A card's identity: its index into [`TIER0`]. Stable, and the value the
/// netcode's 144-byte frame carries as `card_id` (net §4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u16);

/// What a card does. Every variant either writes standing-layer state the
/// engine actually has, or says out loud that it cannot yet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardEffect {
    /// **Politics tier 0 — publish planetary scan data.**
    ///
    /// The one tier-0 card whose value is *game-phase dependent by
    /// construction* (politics §5.2): decisive while the galaxy is dark, waste
    /// paper once coverage is near-total. No balance scaffolding is needed to
    /// stop it dominating, because the phase does the balancing.
    ///
    /// `subject` is whose knowledge is published, and it need not be the
    /// player playing the card — publishing *someone else's* holdings is the
    /// attack (politics §5.3), and it is not opt-in.
    DiscloseScans,
    /// Write a field of the player's own [`Doctrine`] — the standing layer's
    /// policy half. Doctrine dies on retasking, so this leaks temporally
    /// (std §2).
    WriteDoctrine(DoctrineWrite),
    /// Unlock a `(hull, class)` design — the standing layer's Design half.
    /// Design is permanent and strictly earlier-is-better, so this leaks
    /// spatially and never goes stale (std §5).
    UnlockDesign(HullType, Class),
    /// The effect needs a system the engine does not have yet — combat, the
    /// Exchange, `$`, the counter-graph. Counted rather than hidden, so a run
    /// can report how much of the card layer is still scaffolding.
    NotYetImplemented,
}

/// The Doctrine fields a tier-0 card may move. Deliberately a closed set: a
/// card that could write *any* field would make Doctrine a free-form inbound
/// channel rather than standing-layer state with a card-shaped write surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoctrineWrite {
    /// Scale `growth_rate` — Growth's mouth.
    GrowthRate(f64),
    /// Scale `survey_vehicles` — Expansion's mouth.
    SurveyVehicles(i32),
    /// Scale `biosphere_regen_bonus` — the Greening/Growth ecology lever (L6).
    BiosphereRegen(f64),
    /// Set `reinvest_bias` — Production's deepen-vs-expand dial.
    ReinvestBias(f64),
}

/// A card's target. The closed set card §1 requires, and the reason the wire
/// protocol can be fixed-width (net §0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Self or global — no referent needed.
    None,
    /// Another empire. **Not a partner — a subject.** No consent is sought.
    Player(PlayerId),
}

/// One tier-0 card.
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub id: CardId,
    pub tree: Tree,
    pub slant: Slant,
    /// Mineral cost, in the same unit as everything else (design law #11).
    /// **Placeholder magnitudes** — the slant triad's cost *ratios* (std §2:
    /// Floor 5:4:3, Default 3:2:1, Peak 4:2:1) are the ratified shape, and
    /// these numbers do not yet implement them. R-P11.
    pub cost: f64,
    pub effect: CardEffect,
    /// Whether the card needs a `Target::Player`. Checked in coercion.
    pub needs_subject: bool,
}

/// **The 18 tier-0 cards — 3 slants × 6 trees (std §1).**
///
/// **Scaffolding, not a proposal.** The count and the tree/slant grid are
/// ratified; the flavour names, the costs, and which effect sits in which slot
/// are placeholders. Flavour text is the author's own (CLAUDE.md §6) and none
/// has been written — these carry no names at all rather than inventing them.
pub const TIER0: [Card; 18] = {
    const fn c(i: u16, tree: Tree, slant: Slant, cost: f64, effect: CardEffect, needs_subject: bool) -> Card {
        Card { id: CardId(i), tree, slant, cost, effect, needs_subject }
    }
    use CardEffect::*;
    use Slant::*;
    use Tree::*;
    [
        // Politics — the only tree with all three slots implementable today,
        // because scan data is state the engine already keeps per player.
        c(0, Politics, Inscrutable, 0.5, DiscloseScans, false),
        c(1, Politics, Balanced, 0.8, DiscloseScans, true),
        c(2, Politics, LessGuarded, 1.2, DiscloseScans, true),
        // Growth — writes the population and ecology levers.
        c(3, Growth, Inscrutable, 0.5, WriteDoctrine(DoctrineWrite::GrowthRate(1.15)), false),
        c(4, Growth, Balanced, 0.8, WriteDoctrine(DoctrineWrite::GrowthRate(1.35)), false),
        c(5, Growth, LessGuarded, 1.2, WriteDoctrine(DoctrineWrite::BiosphereRegen(1.5)), false),
        // Expansion — writes the survey levers.
        c(6, Expansion, Inscrutable, 0.5, WriteDoctrine(DoctrineWrite::SurveyVehicles(2)), false),
        c(7, Expansion, Balanced, 0.8, WriteDoctrine(DoctrineWrite::SurveyVehicles(4)), false),
        c(8, Expansion, LessGuarded, 1.2, WriteDoctrine(DoctrineWrite::SurveyVehicles(8)), false),
        // Production — the deepen/expand dial.
        c(9, Production, Inscrutable, 0.5, WriteDoctrine(DoctrineWrite::ReinvestBias(0.5)), false),
        c(10, Production, Balanced, 0.8, WriteDoctrine(DoctrineWrite::ReinvestBias(0.9)), false),
        c(11, Production, LessGuarded, 1.2, WriteDoctrine(DoctrineWrite::ReinvestBias(0.97)), false),
        // Technology — Design writes. These are the ones that unblock roster
        // enforcement (T-25): the Medium hull has no unlock path without them.
        c(12, Technology, Inscrutable, 0.5, UnlockDesign(HullType::MediumSystems, Class::Unnamed), false),
        c(13, Technology, Balanced, 0.8, UnlockDesign(HullType::GeneralSystems, Class::Unnamed), false),
        c(14, Technology, LessGuarded, 1.2, UnlockDesign(HullType::GeneralContactVehicle, Class::Unnamed), false),
        // Warfare — nothing implementable: sim has no combat path at all
        // (`combat::resolve_engagement` is never called from `sim`).
        c(15, Warfare, Inscrutable, 0.5, NotYetImplemented, false),
        c(16, Warfare, Balanced, 0.8, NotYetImplemented, true),
        c(17, Warfare, LessGuarded, 1.2, NotYetImplemented, true),
    ]
};

/// Look a card up by id. `None` for an id outside the published list — which is
/// a legality failure, not a panic (net §4.3 drops before it interprets).
pub fn card(id: CardId) -> Option<&'static Card> {
    TIER0.get(id.0 as usize)
}

/// One seat's play for one round. The **entire** network→sim game surface
/// (net §11): a seat, a card, a target, and nothing else.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Order {
    pub seat: PlayerId,
    pub card: Option<CardId>,
    pub target: Target,
}

impl Order {
    /// The deterministic default order: pass, target none (net §5.1).
    pub fn pass(seat: PlayerId) -> Self {
        Order { seat, card: None, target: Target::None }
    }

    /// **Coerce to legality — never reject.**
    ///
    /// Returns the order unchanged if legal, or [`Order::pass`] if not. Every
    /// client runs this over the same state and reaches the same answer with no
    /// message exchanged, which is what keeps the barrier from desyncing.
    ///
    /// `affordable` is passed in rather than read here because legality is a
    /// question about *sim* state and this module deliberately cannot reach it.
    pub fn coerce(self, affordable: bool) -> Self {
        let Some(id) = self.card else { return Order::pass(self.seat) };
        let Some(c) = card(id) else { return Order::pass(self.seat) };
        if !affordable {
            return Order::pass(self.seat);
        }
        match (c.needs_subject, self.target) {
            // A card needing a subject must have one, and it may not be you:
            // "not opt-in" is about the *subject's* consent, not about being
            // able to name yourself as a victim.
            (true, Target::Player(p)) if p != self.seat => self,
            (true, _) => Order::pass(self.seat),
            (false, _) => self,
        }
    }
}

/// Apply a [`DoctrineWrite`] to a doctrine. Separated from the sim so the write
/// surface is one small, testable function rather than a match arm buried in an
/// event handler.
pub fn apply_doctrine_write(d: &mut Doctrine, w: DoctrineWrite) {
    match w {
        DoctrineWrite::GrowthRate(f) => d.growth_rate *= f,
        DoctrineWrite::SurveyVehicles(n) => {
            d.survey_vehicles = (d.survey_vehicles as i64 + n as i64).clamp(0, 64) as usize;
        }
        DoctrineWrite::BiosphereRegen(f) => d.biosphere_regen_bonus *= f,
        DoctrineWrite::ReinvestBias(v) => d.reinvest_bias = v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tier0_grid_is_three_slants_by_six_trees() {
        // std §1: 18 tier-0 cards, 3 slants × 6 trees, and nothing else.
        assert_eq!(TIER0.len(), 18);
        for tree in Tree::ALL {
            for slant in [Slant::Inscrutable, Slant::Balanced, Slant::LessGuarded] {
                let n = TIER0.iter().filter(|c| c.tree == tree && c.slant == slant).count();
                assert_eq!(n, 1, "{tree:?}/{slant:?} should have exactly one card, found {n}");
            }
        }
    }

    #[test]
    fn card_ids_are_their_own_index() {
        // The netcode frame carries `card_id` as a bare u16 into a pinned list
        // (net §4.2), so the id must *be* the index or lookup is a scan.
        for (i, c) in TIER0.iter().enumerate() {
            assert_eq!(c.id, CardId(i as u16));
            assert_eq!(card(CardId(i as u16)).unwrap().id, c.id);
        }
        assert!(card(CardId(18)).is_none());
    }

    #[test]
    fn slant_cost_is_monotone_within_a_tree() {
        // Placeholder magnitudes, but the *ordering* is the ratified part: a
        // less-guarded card is more conditional and must cost more than the
        // inscrutable one, or the convexity design law #9 needs cannot hold.
        for tree in Tree::ALL {
            let get = |s: Slant| TIER0.iter().find(|c| c.tree == tree && c.slant == s).unwrap().cost;
            assert!(get(Slant::Inscrutable) < get(Slant::Balanced), "{tree:?}");
            assert!(get(Slant::Balanced) < get(Slant::LessGuarded), "{tree:?}");
        }
    }

    #[test]
    fn illegal_orders_coerce_to_pass_and_are_never_rejected() {
        let me = PlayerId(0);
        let you = PlayerId(1);

        // Unknown card id.
        assert_eq!(Order { seat: me, card: Some(CardId(999)), target: Target::None }.coerce(true), Order::pass(me));
        // Unaffordable.
        assert_eq!(Order { seat: me, card: Some(CardId(0)), target: Target::None }.coerce(false), Order::pass(me));
        // Needs a subject, none given.
        assert_eq!(Order { seat: me, card: Some(CardId(1)), target: Target::None }.coerce(true), Order::pass(me));
        // Needs a subject, named self.
        assert_eq!(Order { seat: me, card: Some(CardId(1)), target: Target::Player(me) }.coerce(true), Order::pass(me));

        // Legal cases survive unchanged.
        let legal_global = Order { seat: me, card: Some(CardId(0)), target: Target::None };
        assert_eq!(legal_global.coerce(true), legal_global);
        let legal_subject = Order { seat: me, card: Some(CardId(1)), target: Target::Player(you) };
        assert_eq!(legal_subject.coerce(true), legal_subject);
    }

    #[test]
    fn a_politics_card_needs_no_consent_from_its_subject() {
        // politics §6. There is no handshake anywhere in this module, and the
        // coercion rule asks only that the subject not be the player.
        let attacker = PlayerId(2);
        let victim = PlayerId(5);
        let o = Order { seat: attacker, card: Some(CardId(2)), target: Target::Player(victim) };
        assert_eq!(o.coerce(true), o, "the victim is never asked");
    }

    #[test]
    fn doctrine_writes_move_the_field_they_name_and_nothing_else() {
        let base = Doctrine::default();

        let mut d = base;
        apply_doctrine_write(&mut d, DoctrineWrite::GrowthRate(2.0));
        assert_eq!(d.growth_rate, base.growth_rate * 2.0);
        assert_eq!(d.survey_vehicles, base.survey_vehicles);
        assert_eq!(d.reinvest_bias, base.reinvest_bias);

        let mut d = base;
        apply_doctrine_write(&mut d, DoctrineWrite::SurveyVehicles(3));
        assert_eq!(d.survey_vehicles, base.survey_vehicles + 3);

        // Clamped, so a stack of cards cannot drive it negative or unbounded.
        let mut d = base;
        for _ in 0..100 {
            apply_doctrine_write(&mut d, DoctrineWrite::SurveyVehicles(-5));
        }
        assert_eq!(d.survey_vehicles, 0);
    }

    #[test]
    fn every_unimplemented_effect_says_so_rather_than_doing_nothing_quietly() {
        // The point of the variant: a run can count inert plays. Today that is
        // Warfare's three, because `sim` never calls `combat::resolve_engagement`.
        let inert: Vec<_> = TIER0.iter().filter(|c| c.effect == CardEffect::NotYetImplemented).collect();
        assert_eq!(inert.len(), 3);
        assert!(inert.iter().all(|c| c.tree == Tree::Warfare));
    }
}
