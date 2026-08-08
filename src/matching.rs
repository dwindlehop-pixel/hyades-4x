//! matching.rs — the Exchange: deterministic order-book matching for
//! hyades-engine. Drop-in module: dependency-free, no_std-compatible except
//! for `Vec`, `Entity = u64` exactly as `sim.rs` defines it.
//!
//! Design (see `Hyades_matching.md`):
//! - Producers post **Asks** (supply: a laden freighter, a center with free
//!   build output). Consumers post **Bids** (need: a center's mineral
//!   pressure, an unexploited planet's rank). Both carry one scalar,
//!   **pressure** — the abstract value axis ("money") that lets unlike
//!   offers be compared. It is not player-facing currency.
//! - Offers are posted/updated **only when underlying state changes**
//!   (event-driven, dirty-flag). Nothing scans per cycle.
//! - `match_wave` pairs them: highest pressure first, nearest supply within
//!   that, partial fills allowed, **matched quantity is reserved** (the
//!   anti-herding fix over per-agent argmax), and unmatched remainder
//!   **stays queued** in the book for the next wave.
//! - Fully deterministic: ties broken by entity id; no HashMap anywhere;
//!   identical books yield identical fills bit-for-bit.
//!
//! Prior art: the Cities: Skylines `TransferManager` (incoming/outgoing
//! offers matched by priority block, then distance) and Bertsekas's auction
//! algorithm (prices as the matching scalar). References in the spec doc.

pub type Entity = u64;

/// What is being exchanged. One `Book` per (owner, commodity) — or per
/// commodity globally once cross-empire bidding (todo §1) lands.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Commodity {
    /// Basic-mineral haulage: bids are production centers (price =
    /// `mineral_pressure_of`), asks are laden/loading freighters.
    Minerals,
    /// Exploitation targets: bids are unexploited planets (price = rank,
    /// posted on scan / card re-rank events), asks are production centers
    /// with free output. Colonization fills are exclusive (qty 1).
    BuildTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offer {
    pub entity: Entity,
    /// The value scalar ("pressure"). For Minerals: mineral_pressure_of ∈
    /// [0,1]. For BuildTarget: planet rank. Any monotone "how much this
    /// matters" number works; that is the whole point of the abstraction.
    pub price: f64,
    /// How much is wanted (bid) or available (ask). Never negative.
    pub qty: f64,
    /// Position in sim space (ly). Used only as the within-price tiebreak;
    /// a light-lag-aware caller can pre-adjust by substituting effective
    /// distance for geometric distance before posting.
    pub pos: [f64; 3],
}

/// One executed pairing. The caller turns fills into scheduled events
/// (freighter leg, build order) — the Exchange itself never mutates world
/// state; it is a Resource like the event queue, not behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fill {
    pub bid: Entity,
    pub ask: Entity,
    pub qty: f64,
}

/// A two-sided order book. At most one live bid and one live ask per
/// entity (re-posting replaces — the dirty-flag update path).
#[derive(Default, Clone, Debug)]
pub struct Book {
    bids: Vec<Offer>,
    asks: Vec<Offer>,
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn upsert(side: &mut Vec<Offer>, o: Offer) {
    debug_assert!(o.qty >= 0.0 && o.price.is_finite());
    match side.iter_mut().find(|x| x.entity == o.entity) {
        Some(slot) => *slot = o,
        None => side.push(o),
    }
    // Keep zero-qty offers out so book size tracks live need/supply.
    side.retain(|x| x.qty > 0.0);
}

impl Book {
    pub fn new() -> Self {
        Self::default()
    }

    /// Post or update need. O(n) upsert; n = live offers, small and
    /// event-driven — this is called on state *change*, not per cycle.
    pub fn post_bid(&mut self, o: Offer) {
        upsert(&mut self.bids, o);
    }

    /// Post or update supply.
    pub fn post_ask(&mut self, o: Offer) {
        upsert(&mut self.asks, o);
    }

    pub fn cancel_bid(&mut self, e: Entity) {
        self.bids.retain(|x| x.entity != e);
    }

    pub fn cancel_ask(&mut self, e: Entity) {
        self.asks.retain(|x| x.entity != e);
    }

    pub fn bid_of(&self, e: Entity) -> Option<&Offer> {
        self.bids.iter().find(|x| x.entity == e)
    }

    pub fn ask_of(&self, e: Entity) -> Option<&Offer> {
        self.asks.iter().find(|x| x.entity == e)
    }

    pub fn len(&self) -> (usize, usize) {
        (self.bids.len(), self.asks.len())
    }

    /// One matching wave — run as a scheduled discrete event (per empire,
    /// per commodity), NOT per agent. Policy: bids in descending price
    /// (highest pressure served first; ties → lower entity id first);
    /// each bid consumes the *nearest* remaining ask (distance ties →
    /// lower entity id) until the bid is filled or supply is exhausted.
    /// Partial fills reserve quantity on both sides; whatever remains on
    /// either side stays queued in the book.
    ///
    /// Complexity: O(B log B) sort + O(B·A) nearest scans worst case —
    /// fine at empire scale (hundreds); bucket by theater before posting
    /// if a book ever grows past that (spec §5).
    ///
    /// Determinism: total order on every comparison; no float NaN can
    /// enter (debug-asserted on post); identical books ⇒ identical fills.
    pub fn match_wave(&mut self) -> Vec<Fill> {
        // Deterministic bid order: price desc, entity asc.
        let mut bid_idx: Vec<usize> = (0..self.bids.len()).collect();
        bid_idx.sort_by(|&i, &j| {
            let (a, b) = (&self.bids[i], &self.bids[j]);
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then(a.entity.cmp(&b.entity))
        });

        let mut fills = Vec::new();
        for bi in bid_idx {
            while self.bids[bi].qty > 0.0 {
                // Nearest live ask: dist asc, entity asc.
                let best = self
                    .asks
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.qty > 0.0)
                    .min_by(|(_, a), (_, b)| {
                        let (da, db) = (
                            dist2(a.pos, self.bids[bi].pos),
                            dist2(b.pos, self.bids[bi].pos),
                        );
                        da.partial_cmp(&db)
                            .unwrap_or(core::cmp::Ordering::Equal)
                            .then(a.entity.cmp(&b.entity))
                    })
                    .map(|(ai, _)| ai);
                let Some(ai) = best else { break };
                let q = self.bids[bi].qty.min(self.asks[ai].qty);
                fills.push(Fill {
                    bid: self.bids[bi].entity,
                    ask: self.asks[ai].entity,
                    qty: q,
                });
                self.bids[bi].qty -= q; // reservation — the anti-herding fix
                self.asks[ai].qty -= q;
            }
        }
        // Drop exhausted offers; the rest stay queued for the next wave.
        self.bids.retain(|x| x.qty > 0.0);
        self.asks.retain(|x| x.qty > 0.0);
        fills
    }
}

// ------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn o(e: Entity, price: f64, qty: f64, x: f64) -> Offer {
        Offer { entity: e, price, qty, pos: [x, 0.0, 0.0] }
    }

    /// Identical books produce identical fills — the determinism contract.
    #[test]
    fn deterministic() {
        let build = || {
            let mut b = Book::new();
            b.post_bid(o(10, 0.9, 2.0, 0.0));
            b.post_bid(o(11, 0.9, 1.0, 5.0));
            b.post_ask(o(20, 0.0, 1.5, 1.0));
            b.post_ask(o(21, 0.0, 2.0, 4.0));
            b
        };
        assert_eq!(build().match_wave(), build().match_wave());
    }

    /// Reservation prevents herding: two freighters, two needy centers →
    /// one each, not both dogpiling the max-pressure center (which is what
    /// per-freighter `most_needed_center` argmax does today).
    #[test]
    fn reservation_spreads_supply() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.9, 1.0, 0.0)); // most-pressured center
        b.post_bid(o(2, 0.6, 1.0, 10.0)); // second center
        b.post_ask(o(100, 0.0, 1.0, 0.0)); // freighter near center 1
        b.post_ask(o(101, 0.0, 1.0, 10.0)); // freighter near center 2
        let f = b.match_wave();
        assert_eq!(f.len(), 2);
        assert!(f.contains(&Fill { bid: 1, ask: 100, qty: 1.0 }));
        assert!(f.contains(&Fill { bid: 2, ask: 101, qty: 1.0 }));
    }

    /// With exactly one unit of supply, the wave reduces to
    /// `most_needed_center`: the single highest-pressure bid wins. The old
    /// function is the degenerate case, kept as the test oracle.
    #[test]
    fn degenerate_case_equals_most_needed_center() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.3, 1.0, 0.0));
        b.post_bid(o(2, 0.8, 1.0, 100.0)); // farther but needier
        b.post_bid(o(3, 0.8, 1.0, 200.0)); // tie → lower id wins
        b.post_ask(o(100, 0.0, 1.0, 0.0));
        let f = b.match_wave();
        assert_eq!(f, vec![Fill { bid: 2, ask: 100, qty: 1.0 }]);
    }

    /// Unmatched supply/need stays queued — some producers and consumers
    /// may legitimately go unmatched this wave.
    #[test]
    fn unmatched_offers_queue() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.5, 1.0, 0.0));
        b.post_ask(o(100, 0.0, 3.0, 0.0));
        let f = b.match_wave();
        assert_eq!(f, vec![Fill { bid: 1, ask: 100, qty: 1.0 }]);
        assert_eq!(b.len(), (0, 1)); // 2.0 units of supply still queued
        assert_eq!(b.ask_of(100).unwrap().qty, 2.0);
    }

    /// Partial fills split across asks, nearest first.
    #[test]
    fn partial_fill_nearest_first() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.9, 3.0, 0.0));
        b.post_ask(o(100, 0.0, 2.0, 1.0)); // nearer
        b.post_ask(o(101, 0.0, 2.0, 9.0)); // farther
        let f = b.match_wave();
        assert_eq!(
            f,
            vec![
                Fill { bid: 1, ask: 100, qty: 2.0 },
                Fill { bid: 1, ask: 101, qty: 1.0 },
            ]
        );
        assert_eq!(b.ask_of(101).unwrap().qty, 1.0);
    }

    /// Re-posting replaces (the dirty-flag update path); zero-qty removes.
    #[test]
    fn upsert_and_zero_qty() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.5, 1.0, 0.0));
        b.post_bid(o(1, 0.7, 2.0, 0.0));
        assert_eq!(b.len(), (1, 0));
        assert_eq!(b.bid_of(1).unwrap().price, 0.7);
        b.post_bid(o(1, 0.7, 0.0, 0.0));
        assert_eq!(b.len(), (0, 0));
    }
}
