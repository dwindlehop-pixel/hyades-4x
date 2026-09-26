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

use crate::galaxy::PlayerId;
use crate::resources::Basic;

pub type Entity = u64;

/// What is being exchanged. One `Book` per (owner, commodity) for the
/// intra-empire haulage books; **one book per commodity globally** for the
/// cross-empire Exchange (`Hyades_politics_trade_and_intelligence.md` §2.10).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Commodity {
    /// Basic-mineral haulage: bids are production centers (price =
    /// `mineral_pressure_of`), asks are laden/loading freighters.
    ///
    /// **Color-blind, and that is correct for haulage** — an intra-empire
    /// freighter moves whatever its origin has toward whoever needs most, and
    /// R-IND17 already scores that leg by *bill completion* rather than by
    /// color. The color axis below is for the Exchange, where the two sides
    /// are different empires and the whole point is which color moves.
    Minerals,
    /// **One basic color, priced in `$`** (T-83). This is the Exchange's
    /// commodity and the reason `Commodity` needed an axis at all.
    ///
    /// §5.1 made works bills color-payable and T-73 measured what that costs
    /// on a log-normal per-color field: **1,494 of 1,515 banks are
    /// single-colored**, mean dominant share 0.789, and every works ratio
    /// except `1:0:0` demands all three. A color-blind market cannot fix
    /// that — moving "minerals" from a Yellow-rich empire to a Yellow-poor one
    /// is not a trade anyone can express without naming the color.
    Basic(Basic),
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
    /// **Who is offering** (T-83).
    ///
    /// The intra-empire books never needed this — one book per owner meant the
    /// owner was the book. A cross-empire book has both sides in it, and a fill
    /// has to know **who owes whom**: escrow is debited from one purse and
    /// credited to another, and §3.3's default case has to know whose cargo was
    /// lost. Without it a `Fill` names two entities and no counterparties.
    ///
    /// It is also what makes a self-trade detectable, which §4's Corner needs:
    /// an empire outbidding for a mineral it has no use for is a legitimate
    /// play, but an empire filling its *own* ask is a no-op that would mint
    /// reputation and burn `$` for nothing.
    pub owner: PlayerId,
}

/// One executed pairing. The caller turns fills into scheduled events
/// (freighter leg, build order) — the Exchange itself never mutates world
/// state; it is a Resource like the event queue, not behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fill {
    pub bid: Entity,
    pub ask: Entity,
    pub qty: f64,
    /// The counterparties (T-83). Equal for an intra-empire haulage fill;
    /// different for an Exchange trade, which is what makes it one.
    pub buyer: PlayerId,
    pub seller: PlayerId,
    /// **The price this cleared at** (T-77).
    ///
    /// Carried on the fill because it cannot be looked up afterwards: a wave
    /// *drops exhausted offers*, so the bids that matched in full — exactly the
    /// ones a caller most wants to price — are gone from the book by the time it
    /// reads the fills. The first implementation of escrow looked the bid up
    /// after the fact and priced **307 of 942 fills at zero**, which is every
    /// cross-empire fill that matched completely.
    pub price: f64,
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

    /// The standing bids, in posting order.
    pub fn bids(&self) -> &[Offer] {
        &self.bids
    }

    /// The standing asks, in posting order.
    pub fn asks(&self) -> &[Offer] {
        &self.asks
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
            b.price.partial_cmp(&a.price).unwrap_or(core::cmp::Ordering::Equal).then(a.entity.cmp(&b.entity))
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
                        let (da, db) = (dist2(a.pos, self.bids[bi].pos), dist2(b.pos, self.bids[bi].pos));
                        da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal).then(a.entity.cmp(&b.entity))
                    })
                    .map(|(ai, _)| ai);
                let Some(ai) = best else { break };
                let q = self.bids[bi].qty.min(self.asks[ai].qty);
                fills.push(Fill {
                    bid: self.bids[bi].entity,
                    ask: self.asks[ai].entity,
                    qty: q,
                    buyer: self.bids[bi].owner,
                    seller: self.asks[ai].owner,
                    price: self.bids[bi].price,
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
// The Exchange's clearing: a spatial price equilibrium (R-MX7, T-134).

/// **One way an ask can reach one buyer empire** (R-MX7).
///
/// `decay` is `λ·t`: the settlement burn's exponent for this leg, so the seller
/// keeps `exp(−decay)` of what the buyer escrows (politics §3.3). The caller
/// prices `t` off the seller's standing Freighter Design flying to the venue
/// both empires share; the matcher never sees geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Route {
    pub buyer: PlayerId,
    pub decay: f64,
}

/// **Kilotonnes of one ask sold to one buyer empire, at that empire's price.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flow {
    /// Index into the `asks` slice the clearing was given.
    pub ask: usize,
    /// Index into that ask's route list — the leg the caller already priced.
    pub route: usize,
    pub seller: PlayerId,
    pub buyer: PlayerId,
    pub qty: f64,
    /// What the buyer escrows per kilotonne: its empire's uniform price.
    pub price: f64,
}

/// **The quoted price never goes below this fraction of the book's top offer.**
///
/// A numerical device, not a tunable: the clearing runs on log prices, a
/// reservation of zero has no logarithm, and 74% of posted asks carry one (a
/// center with no bill to pay values its spare ore at nothing). Any floor far
/// below every positive price gives the same allocation; this one only decides
/// how close to zero an uncontested price is quoted.
pub const PRICE_FLOOR_FRACTION: f64 = 1e-9;

/// A total order on `f64` keys for the heaps below (no NaN reaches here: every
/// key is a sum of finite logs and finite decays).
#[derive(Clone, Copy, PartialEq)]
struct Key(f64);

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// **Clear one color's cross-empire book at spatial-equilibrium prices**
/// (R-MX7, T-134).
///
/// Every buyer empire pays one price per kilotonne; a seller ships to the
/// buyer whose price, net of the settlement burn on the leg, pays it most;
/// a bid buys only if its value is at least its empire's price; an ask sells
/// only if what it keeps is at least its reservation. That is Samuelson's
/// (1952) spatial price equilibrium, as Takayama and Judge (1971) set it out
/// as a programming problem, with the burn `exp(−λt)` as the transport term.
///
/// **Taking logs makes it a transportation problem.** The seller keeps
/// `P_B·exp(−λt)`, so in `ln P` the burn is an additive cost `λt` and every
/// equilibrium condition is a comparison of sums. The allocation that meets
/// them all maximizes
///
/// ```text
/// Σ_bids ln(b)·x  −  Σ_(ask, buyer) (ln(a) + λt)·y      subject to
/// Σ_(bids of B) x = Σ_asks y_(ask, B)   (mass is conserved on every leg)
/// ```
///
/// and the empire prices are the constraints' duals. It is solved exactly by
/// successive shortest paths (Ahuja, Magnanti and Orlin 1993, §9.7) on a graph
/// whose only interior nodes are the buyer empires — an ask is an arc from the
/// source into an empire, and rerouting an ask already flowing to one empire
/// is an arc between two — so each augmentation is a Bellman–Ford over at most
/// one node per seat.
///
/// **The prices are the least equilibrium prices** — each empire pays what its
/// strongest excluded competitor would have paid, and no more (the ascending
/// auction's outcome; Demange, Gale and Sotomayor 1986). A buyer with no
/// competitor and ample supply pays the sellers' reservation, which is zero for
/// a center with nothing to build: then the price is the floor above.
///
/// Deterministic: offers are read in slice order, every tie breaks on an
/// index, and no hash map is used.
pub fn clear_spatial(bids: &[Offer], asks: &[Offer], routes: &[Vec<Route>]) -> Vec<Flow> {
    use crate::transcendental::{exp, ln};
    use core::cmp::Reverse;
    type Heap<T> = std::collections::BinaryHeap<Reverse<T>>;
    debug_assert_eq!(asks.len(), routes.len());
    let top = bids.iter().chain(asks.iter()).fold(0.0f64, |m, o| m.max(o.price));
    if top <= 0.0 {
        return Vec::new();
    }
    let floor = top * PRICE_FLOOR_FRACTION;
    let ln_floor = ln(floor);

    // Buyer empires, in id order; a bid below the floor can never trade.
    let mut buyers: Vec<PlayerId> = bids.iter().filter(|b| b.price > floor).map(|b| b.owner).collect();
    buyers.sort();
    buyers.dedup();
    let nb = buyers.len();
    if nb == 0 {
        return Vec::new();
    }
    let node = |p: PlayerId| buyers.binary_search(&p).ok();

    // Each empire's demand curve: (ln b, remaining qty), highest first.
    let mut demand: Vec<Vec<(f64, f64)>> = vec![Vec::new(); nb];
    let mut order: Vec<usize> = (0..bids.len()).filter(|&i| bids[i].price > floor && bids[i].qty > 0.0).collect();
    order.sort_by(|&i, &j| bids[j].price.total_cmp(&bids[i].price).then(bids[i].entity.cmp(&bids[j].entity)));
    for i in order {
        let b = node(bids[i].owner).unwrap();
        demand[b].push((ln(bids[i].price), bids[i].qty));
    }
    let mut next = vec![0usize; nb];

    let ln_ask: Vec<f64> = asks.iter().map(|a| if a.price > floor { ln(a.price) } else { ln_floor }).collect();
    // Route arcs resolved to nodes; a route to an empire with no bid is dropped.
    let arcs: Vec<Vec<(usize, f64)>> = routes
        .iter()
        .enumerate()
        .map(|(j, rs)| {
            rs.iter()
                .map(|r| match node(r.buyer) {
                    Some(b) if r.buyer != asks[j].owner => (b, r.decay),
                    _ => (usize::MAX, 0.0),
                })
                .collect()
        })
        .collect();
    let mut free: Vec<f64> = asks.iter().map(|a| a.qty.max(0.0)).collect();
    let mut flow: Vec<Vec<f64>> = routes.iter().map(|rs| vec![0.0; rs.len()]).collect();

    // Source arcs: per empire, the cheapest ask with supply left.
    let mut source: Vec<Heap<(Key, usize, usize)>> = (0..nb).map(|_| Heap::new()).collect();
    for (j, rs) in arcs.iter().enumerate() {
        for (r, &(b, decay)) in rs.iter().enumerate() {
            if b != usize::MAX && free[j] > 0.0 {
                source[b].push(Reverse((Key(ln_ask[j] + decay), j, r)));
            }
        }
    }
    // Reroute arcs: per ordered pair (from, to), the cheapest ask flowing at
    // `from` that could go to `to` instead. Pushed when a flow opens.
    let mut reroute: Vec<Heap<(Key, usize, usize, usize)>> = (0..nb * nb).map(|_| Heap::new()).collect();
    let open = |reroute: &mut Vec<Heap<(Key, usize, usize, usize)>>, j: usize, r: usize| {
        let (from, c) = arcs[j][r];
        for (r2, &(to, c2)) in arcs[j].iter().enumerate() {
            if r2 != r && to != usize::MAX {
                reroute[from * nb + to].push(Reverse((Key(c2 - c), j, r, r2)));
            }
        }
    };

    const TOL: f64 = 1e-12;
    let cap = 4 * (bids.len() + asks.len() + arcs.iter().map(Vec::len).sum::<usize>()) + 16;
    let mut dist = vec![0.0f64; nb];
    let mut pred: Vec<Option<(usize, usize, usize, usize)>> = vec![None; nb];
    let mut via: Vec<Option<(usize, usize)>> = vec![None; nb];
    let mut edge: Vec<Option<(f64, usize, usize, usize)>> = vec![None; nb * nb];
    for _ in 0..cap {
        // Arc weights, dropping exhausted heap tops.
        for b in 0..nb {
            let h = &mut source[b];
            while h.peek().is_some_and(|Reverse((_, j, _))| free[*j] <= 0.0) {
                h.pop();
            }
            match h.peek() {
                Some(Reverse((Key(w), j, r))) => {
                    dist[b] = *w;
                    via[b] = Some((*j, *r));
                }
                None => {
                    dist[b] = f64::INFINITY;
                    via[b] = None;
                }
            }
            pred[b] = None;
            while next[b] < demand[b].len() && demand[b][next[b]].1 <= 0.0 {
                next[b] += 1;
            }
        }
        for (k, h) in reroute.iter_mut().enumerate() {
            while h.peek().is_some_and(|Reverse((_, j, r, _))| flow[*j][*r] <= 0.0) {
                h.pop();
            }
            edge[k] = h.peek().map(|Reverse((Key(w), j, r, r2))| (*w, *j, *r, *r2));
        }
        // Shortest paths from the source over the empire nodes.
        for _ in 0..nb {
            let mut changed = false;
            for from in 0..nb {
                if !dist[from].is_finite() {
                    continue;
                }
                for to in 0..nb {
                    if let Some((w, j, r, r2)) = edge[from * nb + to] {
                        if dist[from] + w < dist[to] - TOL {
                            dist[to] = dist[from] + w;
                            pred[to] = Some((from, j, r, r2));
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        // The cheapest path into a bid; stop when no path gains anything.
        let mut best: Option<(f64, usize)> = None;
        for b in 0..nb {
            if next[b] < demand[b].len() && dist[b].is_finite() {
                let cost = dist[b] - demand[b][next[b]].0;
                if best.is_none_or(|(c, _)| cost < c) {
                    best = Some((cost, b));
                }
            }
        }
        let Some((cost, sink)) = best else { break };
        if cost >= -TOL {
            break;
        }
        // Walk the path back to its source arc, taking the bottleneck.
        let mut path: Vec<(usize, usize, usize)> = Vec::new(); // (ask, from-route, to-route)
        let mut b = sink;
        let mut amount = demand[sink][next[sink]].1;
        while let Some((from, j, r, r2)) = pred[b] {
            amount = amount.min(flow[j][r]);
            path.push((j, r, r2));
            b = from;
            if path.len() > nb {
                debug_assert!(false, "a negative cycle cannot arise on a shortest-path augmentation");
                return Vec::new();
            }
        }
        let (j0, r0) = via[b].expect("a finite distance has a source arc");
        amount = amount.min(free[j0]);
        free[j0] -= amount;
        if flow[j0][r0] <= 0.0 {
            flow[j0][r0] += amount;
            open(&mut reroute, j0, r0);
        } else {
            flow[j0][r0] += amount;
        }
        for &(j, r, r2) in &path {
            flow[j][r] -= amount;
            let opened = flow[j][r2] <= 0.0;
            flow[j][r2] += amount;
            if opened {
                open(&mut reroute, j, r2);
            }
        }
        demand[sink][next[sink]].1 -= amount;
    }

    // Least equilibrium log-prices: each empire's price is at least its best
    // unfilled bid and at least what each of its sellers is paid elsewhere, and
    // the least vector meeting those is found by relaxation.
    let mut pi = vec![f64::NEG_INFINITY; nb];
    for b in 0..nb {
        if let Some(&(lb, _)) = demand[b][next[b].min(demand[b].len())..].iter().find(|(_, q)| *q > 0.0) {
            pi[b] = lb;
        }
    }
    for (j, rs) in arcs.iter().enumerate() {
        for (r, &(b, c)) in rs.iter().enumerate() {
            if flow[j][r] > 0.0 {
                pi[b] = pi[b].max(ln_ask[j] + c);
            }
        }
    }
    for _ in 0..=nb {
        let mut changed = false;
        for (j, rs) in arcs.iter().enumerate() {
            for (r, &(b, c)) in rs.iter().enumerate() {
                if flow[j][r] <= 0.0 {
                    continue;
                }
                for (r2, &(b2, c2)) in rs.iter().enumerate() {
                    if r2 != r && b2 != usize::MAX && pi[b2] - c2 + c > pi[b] + TOL {
                        pi[b] = pi[b2] - c2 + c;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    let price: Vec<f64> = pi.iter().map(|&p| if p.is_finite() { exp(p) } else { 0.0 }).collect();

    let mut out = Vec::new();
    for (j, rs) in arcs.iter().enumerate() {
        for (r, &(b, _)) in rs.iter().enumerate() {
            if flow[j][r] > TOL * asks[j].qty.max(1.0) {
                out.push(Flow {
                    ask: j,
                    route: r,
                    seller: asks[j].owner,
                    buyer: buyers[b],
                    qty: flow[j][r],
                    price: price[b],
                });
            }
        }
    }
    out
}

// ------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// The pre-T-83 helper: one owner, because these tests are about the
    /// *matching*, not about who the counterparties are. `owned` below is the
    /// cross-empire form.
    fn o(e: Entity, price: f64, qty: f64, x: f64) -> Offer {
        Offer { entity: e, price, qty, pos: [x, 0.0, 0.0], owner: PlayerId(0) }
    }

    fn owned(e: Entity, price: f64, qty: f64, x: f64, owner: u32) -> Offer {
        Offer { entity: e, price, qty, pos: [x, 0.0, 0.0], owner: PlayerId(owner) }
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
        assert!(f.contains(&Fill { bid: 1, ask: 100, qty: 1.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.9 }));
        assert!(f.contains(&Fill { bid: 2, ask: 101, qty: 1.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.6 }));
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
        assert_eq!(f, vec![Fill { bid: 2, ask: 100, qty: 1.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.8 }]);
    }

    /// Unmatched supply/need stays queued — some producers and consumers
    /// may legitimately go unmatched this wave.
    #[test]
    fn unmatched_offers_queue() {
        let mut b = Book::new();
        b.post_bid(o(1, 0.5, 1.0, 0.0));
        b.post_ask(o(100, 0.0, 3.0, 0.0));
        let f = b.match_wave();
        assert_eq!(f, vec![Fill { bid: 1, ask: 100, qty: 1.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.5 }]);
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
                Fill { bid: 1, ask: 100, qty: 2.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.9 },
                Fill { bid: 1, ask: 101, qty: 1.0, buyer: PlayerId(0), seller: PlayerId(0), price: 0.9 },
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
    /// **T-83: a fill names who owes whom.**
    ///
    /// The intra-empire books never needed this — one book per owner meant the
    /// owner *was* the book. A cross-empire book has both sides in it, and
    /// escrow has to be debited from one purse and credited to another
    /// (`Hyades_politics_trade_and_intelligence.md` §2.6). A `Fill` that names
    /// two entities and no counterparties cannot settle.
    #[test]
    fn a_fill_carries_its_counterparties() {
        let mut b = Book::new();
        b.post_bid(owned(1, 0.9, 2.0, 0.0, 3)); // empire 3 wants
        b.post_ask(owned(100, 0.0, 2.0, 1.0, 7)); // empire 7 has
        let f = b.match_wave();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].buyer, PlayerId(3));
        assert_eq!(f[0].seller, PlayerId(7));
        assert_ne!(f[0].buyer, f[0].seller, "this is what makes it a trade rather than haulage");
    }

    /// **The color axis exists and orders canonically** (T-83).
    ///
    /// `Commodity` keys the Exchange's books and §10.5 clears per round, which
    /// requires the book set to have a *canonical order* — that is the whole
    /// argument for per-round clearing over continuous matching, because a
    /// continuous book makes price a function of event ordering and two clients
    /// that tie-break differently clear at different prices, which is a desync.
    /// So `Ord` here is load-bearing, not a convenience.
    #[test]
    fn the_color_axis_orders_canonically() {
        use crate::resources::Basic;
        let mut v = vec![
            Commodity::BuildTarget,
            Commodity::Basic(Basic::Yellow),
            Commodity::Minerals,
            Commodity::Basic(Basic::Cyan),
            Commodity::Basic(Basic::Magenta),
        ];
        v.sort();
        let mut w = v.clone();
        w.reverse();
        w.sort();
        assert_eq!(v, w, "the ordering must not depend on the order it was built in");

        // Every color is a distinct commodity — the point of the axis. A
        // color-blind market cannot express "move Yellow to the Yellow-poor",
        // which is what T-73 measured the need for: 1,494 of 1,515 banks are
        // single-colored and every works ratio but 1:0:0 wants all three.
        let all: Vec<Commodity> = Basic::ALL.iter().map(|&c| Commodity::Basic(c)).collect();
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(i == j, a == b, "colors must be distinct commodities");
            }
        }
    }
    fn route(buyer: u32, decay: f64) -> Route {
        Route { buyer: PlayerId(buyer), decay }
    }

    /// **The winner pays its strongest excluded rival's value, not its own**
    /// (R-MX7). Two empires want one kilotonne from a third; the higher bid
    /// takes it at the lower bid's price — the ascending auction's outcome.
    #[test]
    fn a_contested_ask_goes_to_the_higher_bid_at_the_rivals_price() {
        let bids = [owned(1, 10.0, 1.0, 0.0, 1), owned(2, 8.0, 1.0, 0.0, 2)];
        let asks = [owned(100, 0.0, 1.0, 0.0, 0)];
        let flows = clear_spatial(&bids, &asks, &[vec![route(1, 0.0), route(2, 0.0)]]);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].buyer, PlayerId(1));
        assert!((flows[0].qty - 1.0).abs() < 1e-12);
        assert!((flows[0].price - 8.0).abs() < 1e-9, "price {}", flows[0].price);
    }

    /// **The burn on the leg is the transport cost.** One buyer, two sellers
    /// with free ore, one leg costing more burn: the cheaper leg ships.
    #[test]
    fn supply_ships_on_the_leg_that_burns_least() {
        let bids = [owned(1, 3.0, 1.0, 0.0, 1)];
        let asks = [owned(100, 0.0, 1.0, 0.0, 0), owned(101, 0.0, 1.0, 0.0, 2)];
        let flows = clear_spatial(&bids, &asks, &[vec![route(1, 0.5)], vec![route(1, 0.1)]]);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].ask, 1);
    }

    /// **A seller never sells below what the ore is worth to it**, and the
    /// price it is paid covers the burn: `P·exp(−λt) ≥ a`.
    #[test]
    fn a_reservation_and_the_burn_set_the_floor_of_the_price() {
        let asks = [owned(100, 5.0, 1.0, 0.0, 0)];
        let low = clear_spatial(&[owned(1, 4.0, 1.0, 0.0, 1)], &asks, &[vec![route(1, 0.0)]]);
        assert!(low.is_empty(), "a bid below the reservation must not trade");
        let d = 0.2;
        let high = clear_spatial(&[owned(1, 9.0, 1.0, 0.0, 1)], &asks, &[vec![route(1, d)]]);
        assert_eq!(high.len(), 1);
        let keep = crate::transcendental::exp(-d);
        assert!((high[0].price * keep - 5.0).abs() < 1e-9, "seller keeps {}", high[0].price * keep);
    }

    /// **An empire never buys its own ask** — a route to the ask's owner is
    /// dropped even if a caller supplies one.
    #[test]
    fn an_empire_never_clears_against_itself() {
        let bids = [owned(1, 9.0, 1.0, 0.0, 3)];
        let asks = [owned(100, 0.0, 1.0, 0.0, 3)];
        assert!(clear_spatial(&bids, &asks, &[vec![route(3, 0.0)]]).is_empty());
    }

    /// A deterministic pseudo-random book: `sellers` asks and `bids` bids over
    /// `seats` empires, a fraction of asks at zero reservation as on the bed.
    fn random_book(seed: u64, seats: u32, n_asks: usize, n_bids: usize) -> (Vec<Offer>, Vec<Offer>, Vec<Vec<Route>>) {
        let mut rng = crate::rng::Rng::new(seed);
        let mut asks = Vec::new();
        let mut routes = Vec::new();
        for j in 0..n_asks {
            let owner = rng.below(seats as usize) as u32;
            let price = if rng.unit() < 0.7 { 0.0 } else { rng.unit() * 2.0 };
            asks.push(owned(1000 + j as u64, price, 0.1 + rng.unit() * 5.0, 0.0, owner));
            let mut rs = Vec::new();
            for b in 0..seats {
                if b != owner && rng.unit() < 0.7 {
                    rs.push(route(b, rng.unit() * 0.6));
                }
            }
            routes.push(rs);
        }
        let bids = (0..n_bids)
            .map(|i| {
                owned(i as u64, 0.05 + rng.unit() * 3.0, 0.1 + rng.unit() * 3.0, 0.0, rng.below(seats as usize) as u32)
            })
            .collect();
        (bids, asks, routes)
    }

    /// **Every competitive-equilibrium condition holds on random books** —
    /// which, by LP duality, is what makes the allocation the optimum of the
    /// log-price transportation problem `clear_spatial` documents. Checked:
    /// no ask oversold; each empire buys at least every bid above its price
    /// and at most every bid at or above it; every flow is the seller's best
    /// net price and at least its reservation; no unsold ore would sell.
    #[test]
    fn random_books_clear_at_a_competitive_equilibrium() {
        use crate::transcendental::exp;
        for seed in 0..40u64 {
            let seats = 2 + (seed % 5) as u32;
            let (bids, asks, routes) = random_book(seed, seats, 30, 60);
            let flows = clear_spatial(&bids, &asks, &routes);
            let top = bids.iter().chain(asks.iter()).fold(0.0f64, |m, o| m.max(o.price));
            let floor = top * PRICE_FLOOR_FRACTION;
            let tol = 1e-7;
            let mut price = vec![None::<f64>; seats as usize];
            let mut sold = vec![0.0; asks.len()];
            let mut bought = vec![0.0; seats as usize];
            for f in &flows {
                let p = price[f.buyer.0 as usize].get_or_insert(f.price);
                assert_eq!(*p, f.price, "one price per empire");
                sold[f.ask] += f.qty;
                bought[f.buyer.0 as usize] += f.qty;
                assert_ne!(f.buyer, f.seller);
            }
            for (j, a) in asks.iter().enumerate() {
                assert!(sold[j] <= a.qty * (1.0 + 1e-12), "seed {seed}: ask {j} oversold");
            }
            for b in 0..seats as usize {
                let Some(p) = price[b] else { continue };
                let above: f64 =
                    bids.iter().filter(|x| x.owner.0 as usize == b && x.price > p * (1.0 + tol)).map(|x| x.qty).sum();
                let at: f64 =
                    bids.iter().filter(|x| x.owner.0 as usize == b && x.price >= p * (1.0 - tol)).map(|x| x.qty).sum();
                assert!(
                    bought[b] >= above * (1.0 - 1e-9) && bought[b] <= at * (1.0 + 1e-9),
                    "seed {seed}: empire {b} demand"
                );
            }
            // An empire that bought nothing has no quoted price; its bids are
            // what a seller could have had from it.
            let offer = |b: usize| -> f64 {
                price[b].unwrap_or_else(|| {
                    bids.iter().filter(|x| x.owner.0 as usize == b).fold(0.0f64, |m, x| m.max(x.price))
                })
            };
            for f in &flows {
                let net = f.price * exp(-routes[f.ask][f.route].decay);
                assert!(net >= asks[f.ask].price.max(floor) * (1.0 - tol), "seed {seed}: sold below reservation");
                for r in &routes[f.ask] {
                    if r.buyer != asks[f.ask].owner {
                        let rival = offer(r.buyer.0 as usize) * exp(-r.decay);
                        assert!(net >= rival * (1.0 - tol), "seed {seed}: a seller had a better buyer");
                    }
                }
            }
            for (j, a) in asks.iter().enumerate() {
                if a.qty - sold[j] <= 1e-9 * a.qty.max(1.0) {
                    continue;
                }
                for r in &routes[j] {
                    let b = r.buyer.0 as usize;
                    if r.buyer == a.owner {
                        continue;
                    }
                    if let Some(p) = price[b] {
                        assert!(
                            p * exp(-r.decay) <= a.price.max(floor) * (1.0 + tol),
                            "seed {seed}: unsold ore {j} would sell to {b}"
                        );
                    }
                }
            }
        }
    }

    /// Identical books clear identically — the determinism contract.
    #[test]
    fn spatial_clearing_is_deterministic() {
        let (bids, asks, routes) = random_book(7, 5, 40, 80);
        assert_eq!(clear_spatial(&bids, &asks, &routes), clear_spatial(&bids, &asks, &routes));
    }
}
