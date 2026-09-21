# Hyades — Politics: Trade, Exchange, and Shared Intelligence

*The two systems the Politics tree needs: an **exchange for value** and
**granular shared intelligence**. Companion to `Hyades_matching.md` (the
matching engine this builds on), `Hyades_standing_layer_and_observation.md`
(std §, the observation model this trades in), `Hyades_industry.md` §8.1
(refined mass traverses real space), `Hyades_card_contract.md` (card §) and
`Hyades_netcode.md` (net §). Calls flagged **R-Pn**.*

**Rev 2.** Rewritten to carry **ratified decisions and open decisions only**
(`CLAUDE.md` §6). The implementation history that used to be inlined — the λ
routing sweep, the four faucet/sink models, T-77's settlement census, the
400-planet screen that was wrong twice, the stage plan that predicted the wrong
risky stage, and the guard that turned out inverted — is in
**`Hyades_experiments_appendix.md` §B**, linked per decision.

**Rev 1 was labeled "proposed for ratification" and much of it still is.** Rev 2
marks each item's status explicitly rather than leaving it to the prose, because
a proposal written in the present indicative for long enough gets read as
settled.

---

## 0. How to read this file

- **`RATIFIED`** — decided and, where it exists in the engine, built.
- **`OPEN`** — not settled, with what would settle it. A recommendation with no
  ratification behind it lives here, not in a third category.

**The thesis is ratified and everything else answers to it:**

> **Eliminate the value of collusion by making the simulation-state effects of
> collusion available without a confederate.**

Out-of-band collusion is unpoliceable — net §2d says so and is right. So do not
forbid it: **price it at zero.** Enumerate what a private agreement buys and sell
each effect, individually, to a single player acting alone.

| What collusion buys | Bought alone by | § |
|---|---|---|
| Resource complementarity | the Exchange: post a bid, any seller fills it | §2 |
| Shared vision | Intelligence: buy the observation, or take it | §5 |
| Denial | **Corner** and **Embargo**: outbid or tax, no partner needed | §4 |
| Coordinated timing | **Solicit**: open a timed window, let the market coordinate | §2.13 |
| **Non-aggression** | **imposed trade**: interdependence you did not agree to | §6 |

The last row carries the design, and it is why **Politics cards are not opt-in**
(§6). If I can make you my supplier without asking, I have bought the *effect* of
a non-aggression pact unilaterally, in the open, at a posted price. A back-room
deal then adds nothing but coordination overhead — and it is strictly worse,
because the card is enforced by the simulation and the handshake is enforced by
nothing.

**The autopilot premise is what makes this affordable.** Every participant is an
`Autopilot`, so the economy never has to be legible to a human in real time. It
can be a genuine continuous double auction with escrow, risk premia and
distance-discounted clearing, running at simulation speed, because nobody is
clicking.

---

## 1. `$` — the means of exchange

**1.1 `RATIFIED` — `$` has zero mass and sits outside the mass ledger.**
Design law #11 conserves mass with **no exclusions**, so a `$` that were a
commodity would have to be conserved and a faucet would be illegal. `$` is a
**claim, not a substance**: a ledger entry, an obligation, a reputation-weighted
promise to deliver. No mass, no hold, cannot be mined, cannot be shot down.

This is the actual economics rather than a dodge — what crosses interstellar
distance is *the mineral*, and the mineral is conserved. The `$` is the
accounting that decided which direction it went. **R-P1.** The state digest gains
a `$`-ledger leaf of its own, since a claim is not a property of a planet or a
hull.

**1.2 `RATIFIED` — a numeraire rather than barter, and denial is why.** Barter
needs a double coincidence of wants, and a matcher over heterogeneous goods
without a common scalar is a constraint-satisfaction problem, not an auction.
More importantly: **you cannot buy what you do not want, and denial requires
exactly that.** Under `$` you can buy a mineral you have no use for, purely so
your enemy does not get it. **Denial is the Politics tree's attack and it is
impossible without a numeraire.**

**1.3 `RATIFIED` — the sink and the travel-time discount are one mechanism.**

```text
escrowed at match:   E                      (buyer's $, locked)
seller receives:     E · exp(−λ · t_transit)
burned:              E · (1 − exp(−λ · t_transit))
```

One tunable, four jobs: the travel-time discount the brief asked for; a real sink
scaling with volume **and** distance; **geography entering the economy** (a near
partner is strictly better than a far one at the same price, so trade has a map);
and **denial that costs something** (a cornering bid pays full escrow and burns
the transit share).

Three rejected models and why are in appendix §B.2 — the shortest reason to keep
them on record is that **volume-minting *pays* for collusion**, inverting §0.

**1.4 `RATIFIED` — `trade_decay_lambda = 0.01`** (half-life 69 yr). Ratified
first as the **internal freighter routing rule**, where `λ = 0` reduces exactly to
`most_needed_center` — design law #5's single-supply oracle — so one function
checks two independent degeneracies. **Confirmed on 3 seeds, which is thin**;
direction and order of magnitude hold, the precise optimum wants a ten-seed bed.
Appendix §B.1.

**The condition of ratification was that the discount must be *the* solution to
freighter routing, not merely compatible with it, and it is:** the mechanism paid
for itself before the system it was designed for existed.

**1.5 `RATIFIED` — the faucet is production, not population.**

```text
$_income = base · production · politics_multiplier(depth)
```

Production is the sum of **both** halves of an economy — population growth and
infrastructure deepening both feed it, where population alone counts one. An
empire that invested in infra rather than bodies is not poorer. It also removes
the double-count a pop faucet has: the snowball compounds through *colonies*, and
production is what colonies do. **R-P3.**

**Politics depth buys purchasing power, not merely access** — that is what lets
the tree's late nodes outbid a warring rival directly (§4.4).

**1.6 `RATIFIED as a placeholder` — `dollar_per_fabrication = 1.0` against
infrastructure stock.** `production` in 1.5 is the works fabrication rate, which
is `Hyades_industry.md` T-74. Until that lands the faucet reads `Factors::infra`
— kilotons of works, which is what production is *bought with*, a leading
indicator of the real quantity rather than a different one. **R-P16 — revisit at
T-74 rather than quietly keeping it.**

**1.7 `OPEN` — R-P2: the base income rate and the Politics depth multiplier.**
Both are unset. MC.

**1.8 `OPEN` — λ is a cross-tree conflict and was ratified on one metric.**
`trade_decay_lambda` measures **+0.002 on Expansion and −0.348 on Growth**
(`examples/tree_gradient`). It is the largest ratification in this project's
history and coverage was the only objective it was taken against. Re-ratify on
the composite.

---

## 2. The Exchange

**2.1 `RATIFIED` — the market is free; everything that bends it is a card.**

| Transaction | Needs | Why |
|---|---|---|
| **Balanced exchange** — even trade balance *in value* | nothing | the default, always available |
| **Deficit** — giving more value than you receive | a card | a gift is a commitment, and commitments are what the standing layer prices |
| **Forging a pact** | a card | |
| **Breaking a pact** | a card | |
| **Smuggling** — trade evading an embargo or a pact | a card | supported, deliberately |

**R-IND5.** Two consequences are load-bearing. **Balance is in *value*, not in
kilotons** — they begin equal and diverge as the game develops, because value is
set by demand and demand is Doctrine. And **a free par market is what makes the
deficit cards mean something**: if ordinary trade needed a pact, a pact would be a
license to trade and the interesting play would be gated behind bookkeeping.

**2.2 `RATIFIED` — settlement is a voyage, not a transfer.** Minerals, supers and
apex traverse real space (`Hyades_industry.md` §8.1). A matched contract is
delivered on a leg, under light-lag, and that hull can be attacked, diverted,
stolen and blockaded. This is why §2.5 needs escrow and default at all: **the gap
between agreeing a trade and receiving it is a physical interval, and it is where
piracy lives.** Blockade is not a special rule — it is a fleet sitting on a route
that has to be flown.

It makes scarcity **positional rather than only geological**: an empire can sit on
all six colors and be unable to use them because the routes run through somebody
else's reach, while a poor empire astride a corridor has something to sell that is
not ore.

**2.3 `RATIFIED` — settlement happens at a shared outpost, not at the buyer's
world.** Caravan trade: goods change hands at a waypoint. *An outsider's freighter
near your colony is a step left to cards.*

**The engine already has the venue.** An outpost is unowned — no `owner` component
is ever set on a worked rock — while `outpost_stock` is keyed `(player, rock)`, so
each empire holds its own pile at the same body. **2,226 of 2,494 worked sites are
cross-player** on the standard bed.

Four consequences:

- **No cross-empire vehicle is constructed.** A hull whose home is one empire and
  whose destination is another's colony is not needed at all.
- **The buyer's own freighter carries it home**, on the need-based route it was
  already flying — a leg that exists, is laden, is light-lagged and is
  interdictable.
- **§8.1 is satisfied and not weakened.** A trade is still not a ledger entry:
  the goods sit on a rock in the dark until somebody comes for them.
- **Geography becomes the trade constraint, which is the point.** An empire can
  only settle with a counterparty it **shares an outpost with**, so the map
  decides who can trade with whom and a contested rock is valuable for a second
  reason.

**2.4 `RATIFIED` — a contract has two drops, one per shipper.** The seller leaves
Yellow at a shared outpost near *itself*; the buyer leaves Magenta or Cyan at a
shared outpost near *itself*. Each drop is the shared rock nearest **the party
shipping to it**, because a shipper pays for its own leg — one compromise venue
would make each side pay for the other's geography, and §2.1's default
transaction is balanced in value.

**R-P17 resolved, and the question was wrong**: the earlier formulation minimized
the two parties' *summed* transit, which is right only if there is a single venue.
Appendix §B.6.

**2.5 `RATIFIED` — the obligation is instant; only the goods are light-lagged.**
Not an exception to design law #15, but R-P1 taken seriously: `$` and the claim it
denominates are not substances, so there is nothing for light-lag to bind. A
contract is struck at the **round barrier**, which is the protocol clock's
synchronisation point and the same moment cards resolve — not an in-world
observation by an in-world agent, which is what design law #15 constrains.

**The asymmetry is the design.** A deal can be agreed across the theatre in a
single round while the ore it commits takes decades to arrive, and everything that
can happen to that ore on the way lives in the gap. **A contract is a promise that
outruns its cargo**, which is what makes defaulting, escorting and interdiction
worth anything.

**2.6 `RATIFIED` — escrow, settlement, default, non-delivery.**

1. **Match.** Buyer's `$` moves to escrow; seller's cargo is reserved.
2. **Ship.** A laden hull flies it — an ordinary vehicle with an acceleration
   signature that can be observed and **can be shot**.
3. **Settle on delivery.** Escrow releases per 1.3, discounted and burned by
   transit time.

Three ways it goes wrong, all intended: **first-party default** (the buyer
repudiates, or the seller takes escrow and never ships); **third-party
interdiction** (someone takes the cargo in flight — no new mechanism, it is a
laden freighter in open space); **non-delivery** (the hull is destroyed; **escrow
returns to the buyer minus the burn**, so the buyer loses the burn, the seller
loses the cargo, and the loss is shared — which is what makes escorting worth
paying for).

**R-IND10 resolved by this clause**, and it sat open in the register for a
revision because nothing was sweeping the register against the spec body.
Appendix §B.7.

**2.7 `RATIFIED` — reputation is mechanical, not social.** A defaulter's
counterparties raise their escrow requirement and discount their bids. No human
judgment, no table talk, no appeal: the autopilot prices you. This is the only
workable design when every participant is a program, and it is *better* than a
social norm because it is legible and exactly as forgiving as its decay constant
says.

**2.8 `RATIFIED` — reputation is public by default and a card switches *you* to
per-observer.** Both ship; which you use is a purchase. **R-P4.**

- **Public (default).** One consensus number per player. Cheap, coarse, and
  **gameable by anyone who can manufacture visible defaults** — a shared signal
  is also a shared attack surface.
- **Per-observer (bought).** You price every counterparty from what *you*
  personally saw. Correct under the observation model and strictly more
  information: you can deal with someone the public record has blacklisted but who
  has never burned *you*.

Three things fall out, and the third is why this beats either model alone: it
makes **the observation model itself a purchasable upgrade** (not a bigger number,
a better epistemic position); it is a **real counter to reputation attacks**, so
the tree has an internal counter rather than needing one from outside; and it
**bounds the storage cost** — per-pair reputation is the expensive representation
T-33/T-26 flag at fleet scale, and here only the players who bought it pay for it.

**2.9 `RATIFIED` — clearing is per round, at the barrier.** Not continuously on
the event queue. **R-P10.** Three reasons and the third settles it: cards resolve
at barriers and the `Works` fold is recomputed there; it is what the tabletop
lineage does; and **a continuous book makes price a function of event ordering**,
so two clients processing a tie in a different order clear at different prices —
a desync, and by design law #16 an unreproducible one. Per-round clearing makes
the book's contents a **set**, and a set has a canonical order.

**2.10 `RATIFIED` — books are one per commodity, spanning empires**, with the
commodity axis being **per color**. Determinism is unaffected: the book is
already ordered by pressure then entity id, and entity ids are globally unique.

**2.11 `RATIFIED` — a bid is derived, never chosen by a human.**

```text
wtp(mineral) = base_value(mineral)
             × doctrine_demand(mineral)      // Doctrine that wants it, wants it more
             × shortfall_pressure(center)    // Simulation::mineral_pressure_of
             × risk_discount(counterparty)   // §3
```

**`doctrine_demand` is where the works mix enters the market**, and it is
measured: an empire deep in Production bids a Yellow-heavy bill, and trade flow
reproduces the `3:2:1` Y:C:M works mix on both seeds without anything in the
market being told about it. Appendix §B.3.

**2.12 `RATIFIED` — the Doctrine field list.** §2.11's first two terms and the
risk term live on `Doctrine`, which is the diplomatic-fields slot **T-11/R-O27**
was holding open with no field list. **This spec is that field list:**

```rust
/// Floor price per basic color, `$`/kt. Placeholder magnitudes.
pub base_value: [f64; 3],
/// How much this empire's policy wants each color (§2.1).
pub doctrine_demand: [f64; 3],
/// Discount applied to a counterparty by reputation (§3).
pub risk_aversion: f64,
```

Shipped defaults: `base_value = [1.0; 3]`, `doctrine_demand = WORKS_MIX_DEFAULT`
(`3:2:1` Yellow : Cyan : Magenta), `risk_aversion = 0.0`. **All placeholder.**

> **`risk_aversion = 0.0` cannot be moved by a multiplicative gradient probe** —
> `x(1 ± δ)` is `0.0` twice, and the probe reports a confident flat. An absolute
> step is the fix.

**2.13 `OPEN` — R-P5: the event taxonomy that admits solicitation, and
`window_years`.** An **event** is any occurrence with contestable value — a world
scanned, an outpost exhausted, a fleet detected, a colony founded, a hull
completed. Soliciting opens a book on it that closes after `window_years`. The
window is a **round-layer** object, not a wall-clock one (net §1.1 forbids host
clocks, and a bid window driven by a local timer is a desync). *Recommend* the
window be a fraction of `years_per_round` so it composes with the round layer.

**2.14 `OPEN` — R-P18: why trade costs ~4.4% of work-years on both seeds.**
Settlement works and moves the right color in the right direction, and the empire
is measurably poorer for it. **Leading hypothesis, recorded as unproven:** the two
legs are asymmetric — the seller's ore leaves a spendable bank at settlement while
the buyer's lands in an outpost pile until its own freighter calls. Checkable by
comparing banked against piled holdings over time and measuring settlement-to-bank
dwell. **Ablate before tuning anything.** Appendix §B.3.

Carries two structural limiters on volume too: **ten clearings per game**
(`years_per_round = 400`, set by the card layer rather than the market) and a bid
sized to one infrastructure rung rather than to consumption.

**2.15 `OPEN` — the guard for Exchange work is the build-mix census, not
colony-years.** Colony-years is **monotone inverse** for anything that changes how
minerals are spent — it rose as development collapsed and fell as it recovered, on
both seeds, across four industry landings. `examples/bank_mix` is what actually
caught every defect on that branch. The right guard is Growth's **work-years** and
it is not fully measurable until T-74. Appendix §B.5.

---

## 3. Risk premium

**3.1 `RATIFIED` — an armed fleet is visible, and visibly armed players get worse
prices.** `a = thrust/(dry + cargo)` and a warship's signature is high and tightly
clustered (std §3), so a mobilised empire is a **credit risk**: counterparties
raise `escrow_ratio` and apply `risk_discount`. **No modifier is applied
anywhere** — the state is an observed acceleration distribution and the risk term
is a function of it.

---

## 4. The Politics verbs

**4.1 `RATIFIED` — every verb is unilateral.** That is the §0 test, and a verb
that fails it does not belong in this tree.

**4.2 `RATIFIED` — the market verbs and their tiers.** *Tier assignment is the
ratified part; costs are R-P7.*

| Verb | Effect | Tier |
|---|---|---|
| **Offer** | post an ask: commit stock at a reserve price | 0 |
| **Bid** | post a bid at your derived WTP | 0 |
| **Consign** | ship before a buyer exists — pay transit early, clear on arrival | 1 |
| **Underwrite** | lower a named counterparty's escrow requirement — a favor with a price, and the seed of a bloc | 1 |
| **Broker** | clear through a third party, so a trade completes between players who cannot deal directly | 2 |
| **Embargo** | raise the effective `λ` on a counterparty's trades — tax their distance, not their price | 2 |
| **Corner** | bid across an entire mineral *class* rather than a lot — denial at scale | 3 |
| **Default** | take without paying; reputation cost, no legal cost | any |
| **Interdict** | seize goods in transit — a Warfare action with a Politics payoff | — |

**4.3 `RATIFIED` — the intelligence verbs.**

| Verb | Effect | Tier |
|---|---|---|
| **Disclose (own)** | publish your own observations — a gift, or a bid for a bloc | 0 |
| **Disclose (other)** | publish **someone else's** Design or Doctrine — transparency as an attack (§5.3) | 2 |
| **Solicit** | open a timed window on an event (§2.13) | 1 |
| **Recognize / Denounce** | move a counterparty's risk premium directly | 2 |
| **Audit** | switch this empire to per-observer reputation pricing (§2.8) | 2 |

**4.4 `RATIFIED` — trading with someone you are at war with is the tree's
clearest depth gradient.**

- **Tier 0–1:** you cannot. Books exclude counterparties you are at war with.
- **Tier 2 — Broker.** A third party clears it. The minerals reach you, the
  broker takes a cut, and your enemy sold into a market and cannot tell to whom.
  **This is the collusion effect bought without a colluder** — the third party is
  maximizing its own return, not acting as an ally.
- **Tier 3 — direct.** Deep Politics buys the ability to outbid an enemy *in the
  open*. They can see you doing it and cannot stop it except by outbidding you,
  which costs them the same `$` they wanted the minerals for.

**4.5 `OPEN` — R-P7: the counter list, per tree, with costs.** This is the balance
surface of the entire tree and it cannot be set analytically. Counters live in
other trees, which is design law #7's combo requirement made concrete: Warfare's
Interdict takes the goods; Growth's autarky reduces the `doctrine_demand` that
makes you biddable; Production's substitution routes around the cornered mineral.

---

## 5. Shared intelligence

**5.1 `RATIFIED` — the price ladder is the observation half-life.** std §5's
asymmetric leak: *Design never goes stale; Doctrine dies on retasking.* The longer
an observation stays true, the more it is worth.

| Tier | Commodity | Half-life | Why it sits here |
|---|---|---|---|
| **0** | **planetary scan data** | decays with game phase | decisive while the map is dark, waste paper once everyone has scanned everything |
| 1 | movement / trajectories | hours-to-years; stale on arrival | perishable, high tactical value; what makes a reachability cone worth buying |
| 2 | Doctrine | until retasking | a distribution over intent, not a fact — buying it buys SPRT samples |
| 3 | **Design** | **permanent** | a roster entry never goes stale, so it is the most valuable thing on the board and the most damaging to have published |

**5.2 `RATIFIED` — scan data is the tier-0 card, and the game phase does its
balancing.** A card strong in round 1 and dead by round 8 needs no balance
scaffolding; the player's judgment about *when* it stops being worth an action is
the skill. It is also the one Politics card whose value can be measured against
the objective the engine is already tuned on.

**5.3 `RATIFIED` — publishing someone else's intelligence is the attack.**
Publishing your own is a gift. Publishing another player's Design is the Politics
answer to a Design advantage: a player who quietly unlocked a superior hull bought
**spatial** concealment (std §2 — you must come close to read the fit), and
**Disclose (other)** takes what one player paid to learn and hands it to everybody.

Two properties make this the right shape. **It is not theft of a thing** — the
victim loses no mass and no `$`, only an information asymmetry, which is the only
thing this tree should be able to take. And **it kills the collusion channel
dead**: "my ally told me their Design" is a private, unpriced, unpoliceable
transfer, and if the same effect is a card any single player can buy, the private
version is worth nothing.

**5.4 `RATIFIED` — the recipient rule bifurcates with depth, and broadcast is the
stronger branch.** **R-P6.**

| | Recipient | Where it sits |
|---|---|---|
| **Tier 0** | **everyone** | the mouth card; no targeting, no choice to make |
| **Deep / win-condition branch** | **everyone**, still | broadcast is the win path |
| **Shallower branch** | **a chosen recipient** | targeted, and worth *less* |

The counterintuitive part falls straight out of §0: **targeted disclosure is
closer to actual collusion** — you pick a confederate and hand them something
nobody else gets — so it must be worth less, or the tree would be paying players
to do the thing the design is trying to make worthless. It is also self-limiting
in a way targeting is not: **you cannot weaponise a broadcast against one rival
without arming the whole table**, so a disclosure war escalates against its own
initiator.

**5.5 `OPEN` — R-P15: a targeted disclose *about a third party* takes two player
arguments** — a recipient and a subject. `cards::Target` is a closed set with room
for exactly one referent, and the wire protocol's `target_kind`/`target_ref` pair
is sized for one. Does `Target` need a two-referent variant, and does that push
`target_ref` wider than R-NET4 assumes? Blocked on R-NET4 and R-C1.

---

## 6. Politics cards are not opt-in

**6.1 `RATIFIED` — what can be imposed on you.** A trade relationship (someone can
become your supplier, or your customer, without your agreeing); intelligence about
you can be published; intelligence can be pushed *to* you and **you cannot
un-know it** (a real cost — your autopilot will act on it); your goods can be bid
for, cornered or embargoed.

**6.2 `RATIFIED` — what cannot.** Your minerals cannot be taken without either
payment or a physical seizure (Interdict — a Warfare act, resolved by combat).
**Your Doctrine and Design cannot be *written* by another player.** Only read, and
only published.

**6.3 `RATIFIED` — the line, and it is sharper than "not opt-in".**

> **Your Doctrine cannot be written by another player. Your empire's view of
> another player's empire absolutely can.**

Doctrine is your policy — how hard you grow, how far you survey, how much you
reinvest. Nobody else writes it. But *who you take that policy to be about* is a
separate piece of state, and it is exactly the surface a Politics card operates
on. That split lets §6.1 have teeth without letting an opponent pilot your empire:
they cannot make you expand faster; they can make you treat them as a supplier, or
a third party as a threat.

**6.4 `RATIFIED` — imposed trade is not merely an annoyance.** Becoming dependent
on a supplier is a real strategic state (§8.1) and the supplier chose it for you.
A player who imposes trade on a rival is buying future restraint from that rival,
unilaterally — **a non-aggression pact with no counterparty**, which is the single
most valuable thing collusion offers and the hardest to price. Here it has a
price.

**6.5 `RATIFIED` — no consent channel exists anywhere in `cards.rs`, and that is
deliberate.** A consent channel would be a second inbound path across the
presentation seam (design law #15), and the anti-collusion thesis depends on these
effects being purchasable alone. `Target::Player` names a *subject*, not a partner.

---

## 7. Relationships — stance and conduct

**7.1 `RATIFIED` — stance is a small closed enum, per *ordered* pair, and not
symmetric.** A boolean `at_war` collapses every distinction the tree is about: a
mercenary is not a friend, a trade partner is not an ally, a rival you still sell
to is not a foe.

| Stance | What it means operationally |
|---|---|
| `Unknown` | never contacted; no basis for conduct at all |
| `Neutral` | contacted, no relationship — the default after first contact |
| `TradePartner` | clears on the Exchange, low escrow, no denial bidding |
| `Mercenary` | transactional; will deal, will also take a better offer |
| `Client` / `Patron` | asymmetric: one side underwrites the other |
| `Rival` | competes for the same worlds; still trades, at a premium |
| `Foe` | no direct clearing (brokerage only, §4.4); interdiction legal |

`stance[me][them]` is **not** symmetric — you may see them as a trade partner
while they see you as a mark. **That asymmetry is the entire content of a
successful deception**, and it is what a disclosure attack (§5.3) collapses when
it publishes the truth.

**7.2 `RATIFIED` — conduct is a table, not a branch.** `Doctrine` carries **one
conduct row per stance**, so every interaction resolves by lookup rather than by
an `if hostile` branch.

```rust
pub struct Conduct {
    /// Multiplier on willingness to pay when this counterparty is the seller.
    pub trade_appetite: f64,
    /// Escrow demanded of them, as a multiple of contract value (§2.6).
    pub escrow_ratio: f64,
    /// Willingness to pay above derived WTP purely to deny them (§4.2).
    pub denial_premium: f64,
    /// Will this empire clear with them directly at all, or only via a broker?
    pub clears_directly: bool,
    /// Willingness to publish intelligence *to* them, and *about* them.
    pub disclose_to: f64,
    pub disclose_about: f64,
    /// Kinematic posture when their fleet is in the theater — feeds
    /// `belief::decide_engagement`'s `Unobserved` policy (`src/belief.rs`).
    pub engage: EngagePolicy,
}

pub struct Diplomacy {
    /// Per-mineral demand multiplier — the WTP term of §2.11.
    pub demand: [f64; N_BASIC],
    /// Fraction of income committed to the book per round.
    pub trade_budget: f64,
    /// **One row per stance.** Not a list of exceptions.
    pub conduct: [Conduct; N_STANCE],
}
```

Three things this buys beyond legibility:

1. **`excluded` disappears and R-P8 dissolves with it.** Refusal is
   `conduct[Foe].clears_directly == false` — a policy about a *kind* of
   relationship rather than about named players, so design law #13 was never in
   danger. Appendix §B.8.
2. **It gives the belief layer its missing input.** `belief::decide_engagement`
   takes an `Unobserved` policy for contacts never seen, and the honest default
   was `PeerOf`. What you assume about a fleet you cannot measure should depend on
   whose it is.
3. **Imposing a stance is a real attack with a bounded blast radius.** Writing
   `stance[victim][me] = TradePartner` changes how the victim's autopilot prices
   and treats you, **using the victim's own conduct table**. You have not taken
   their policy — you have moved yourself within it.

**7.3 `RATIFIED` — both indices are attackable.** `stance[victim][me]` makes them
see *you* differently; `stance[victim][third_party]` makes them see *someone else*
differently. The second has no counterpart in a two-player negotiation and is the
sharpest expression of §0 in the design: *"get two other players to fight each
other"* is among the highest-value things a real alliance buys, and it normally
requires a confederate.

**7.4 `OPEN` — R-P13: which stances may be written, by which tier, on which
index.** *Recommend* the near index unlocks first and only toward *less* hostile
— you can make yourself a trade partner, not make yourself trusted — and the far
index is a deep node, since manufacturing a war between two other empires should
cost most of a tree.

**7.5 `OPEN` — R-P14: does an imposed stance decay?** A permanent write is a
permanent non-aggression pact for one card, which is too strong. *Recommend* decay
on the reputation clock, so maintaining an imposed relationship costs actions
rather than being bought once. MC.

**7.6 `OPEN` — the reputation decay rate**, on both the public and per-observer
models, and whether a per-observer player still *contributes* to the public
number. *Recommend* yes: you can leave the consensus without leaving the record,
and a defector who could also go silent would be too strong.

---

## 8. The counter-graph: trade and mobilization blunt each other

**8.1 `RATIFIED` — early trade blunts later mobilization, by supply chain and not
by modifier.** Mobilizing consumes minerals. If a share of your mineral inflow
arrives through the Exchange, **mobilizing against your supplier cuts the supply
the mobilization is made of.** Your fleet comes out smaller, and it comes out
smaller *because* of a choice you made two hundred years earlier.

**Legible from outside** as freighter traffic: a trade-dependent empire has a
visible pattern of laden hulls arriving from a foreign origin, and under the shell
model a laden hull is conspicuously slow (std §9.2). **You can see who depends on
whom by watching who is sending whom slow ships.**

**8.2 `RATIFIED` — early mobilization blunts later trade, by risk premium and not
by modifier.** §3.1. **Legible from outside** because it *is* the observation —
the same burn signature the yomi layer already runs on, priced.

**8.3 `RATIFIED` — both effects accumulate, so the counter-graph edge is
time-dependent.** Trade dependence builds with volume; reputation as an armed
power builds with observation count (std §6.5's SPRT — detection needs samples,
and samples take rounds). **The edge is a function of when each player started.**

| | opponent trades early | opponent arms early |
|---|---|---|
| **you trade early** | mutual dependence; both mobilizations are weak; the game goes long | you supply someone who will hit you — profitable and dangerous |
| **you arm early** | you pay a premium for everything, but they cannot hit back hard | conventional arms race; Politics is irrelevant to both |

The bottom-left and top-right cells are where the yomi lives, and neither is
dominated.

**8.4 `OPEN` — R-P9: the strength of both effects.** The supply-chain effect is
bounded by the trade share of mineral inflow, which is self-limiting. **The
risk-premium effect is not obviously bounded and could make early arming
unplayable.** MC.

---

## 9. The implementation contract

**9.1 `RATIFIED` — `src/matching.rs` is the substrate and its core is reusable.**
Highest price first, nearest within price, partial fills, matched quantity
*reserved* (the anti-herding fix), unmatched remainder queued, ties by entity id,
**no `HashMap` anywhere** — already network-safe under net §6. What changes is
what it is a book *of*.

> It was never in the module list, so it did not compile as part of the crate and
> its tests never ran in CI. **"Built and audited" and "compiled" are different
> claims**, and this file was only the first place that mattered. Wired at T-01.

**9.2 `RATIFIED` — terms.**

| symbol | meaning | unit | where it comes from |
|---|---|---|---|
| `$` | the means of exchange | `$` — **not mass** (§1.1) | per-player ledger, §9.3 |
| `E` | escrow locked at match | `$` | §1.3 |
| `λ` | transit discount and burn rate | 1/yr | `SimConfig::trade_decay_lambda = 0.01` |
| `t` | one-way transit of the settling leg | yr | `math::ship_travel_years` |
| `wtp` | a center's willingness to pay | `$`/kt | §2.11 |
| `base_value[c]` | a color's floor price | `$`/kt | `Doctrine`, §2.12 |
| `doctrine_demand[c]` | how much this empire's policy wants color `c` | dimensionless | `Doctrine`, §2.12 |
| `c` | a basic color — Cyan, Magenta, Yellow | — | `resources::Basic` |

**9.3 `RATIFIED` — state, and where it lives.** Per player, alongside `Doctrine`,
`Roster` and `Works`: `purse: $` (**replicated state**, so it is in the digest and
design law #16 applies — no NaN, no infinity) and `reputation`. Global, one per
commodity: the cross-empire `Book`. Per contract in flight: buyer, seller, color,
quantity, escrow `E`, and the two drops. **That last is the first thing in the
engine that is *owed* rather than owned.**

**9.4 `RATIFIED` — the build order, and where it stands.**

| # | Stage | State | T-code |
|---|---|---|---|
| 1 | `$` ledger + faucet; nothing spends it | **done**, bit-identical | ~~T-82~~ |
| 2 | `Commodity` gains the color axis; `Offer` gains an owner | **done**, bit-identical | ~~T-83~~, with ~~T-01~~ |
| 3 | Cross-empire book; centers post `wtp` bids | **done**, bit-identical | ~~T-84~~ |
| 4 | Clearing at the round barrier → contracts + escrow | **done**, bit-identical | ~~T-85~~ |
| 5 | The freight leg; escrow settles on arrival | **done; costs ~4.4% work-years** (§2.14) | ~~T-77~~ |
| 6 | Default, interdiction, reputation | **open** | **T-86** |

Stages 1–4 were deliberately inert, for the reason `Hyades_industry.md` §6.7's
stages were: **a system that lands neutral can be verified against a bit-identical
bed before anything switches on.**

> **The plan predicted the wrong risky stage.** Stage 4 was expected to be it;
> once §2.3 moved settlement to a shared rock, the risk moved to stage 5 with the
> goods. **A stage plan is a claim about code, and an amendment to the design
> invalidates the plan's predictions along with everything else it touches.**
> Appendix §B.4.

---

## 10. Register

### Ratified

| Code | Decision |
|---|---|
| R-P1 | `$` has zero mass and sits outside the mass ledger; the digest gains a `$` leaf |
| R-P2 (part) | `trade_decay_lambda = 0.01` — **3 seeds, thin**, and measured on coverage alone |
| R-P3 | the faucet is production, not population |
| R-P4 | reputation public by default; a card switches the buyer to per-observer |
| R-P6 | disclosure recipient bifurcates with depth; broadcast is the win path |
| R-P8 | **dissolved** — refusal is `conduct[Foe].clears_directly`, not a player list |
| R-P10 | clearing is per round, at the barrier |
| R-P16 | the faucet ships against infrastructure stock **as an explicit placeholder** |
| R-P17 | a contract has **two** drops, one per shipper |
| R-IND5 | the market is free; everything that bends it is a card |
| R-IND10 | non-delivery returns escrow minus the burn; the loss is shared |
| — | settlement is at a shared outpost (§2.3); the obligation is instant, the goods are not (§2.5) |

### Open

| Code | Question | Blocked on |
|---|---|---|
| R-P2 (rest) | base income rate, Politics depth multiplier; and λ on a ten-seed bed | MC |
| R-P5 | the event taxonomy admitting solicitation, and `window_years` | the round layer |
| R-P7 | the per-tree counter list with costs — the tree's whole balance surface | MC, R-P2 |
| R-P9 | strength of both counter-graph effects; is the risk premium bounded? | MC |
| R-P13 | which stances may be written, by which tier, on which index | a design pass |
| R-P14 | does an imposed stance decay? | MC |
| R-P15 | does a two-referent `Target` widen `target_ref` past R-NET4? | R-NET4, R-C1 |
| R-P18 | **why trade costs ~4.4% of work-years on both seeds** — ablate before tuning | §2.14 |
| — | reputation decay rate on both models | MC |
| — | λ is +0.002 on Expansion and −0.348 on Growth | the composite objective |
| T-86 | default, interdiction, reputation (stage 6) | — |

---

## References

**Internal**
- `Hyades_experiments_appendix.md` §B — the measurement record behind every
  ratified item above
- `Hyades_matching.md` — the matching engine this builds on; `src/matching.rs`
- `Hyades_industry.md` §8.1 (refined mass traverses real space), §6.10 (the works
  mix), §6.20 (freight by color)
- `Hyades_standing_layer_and_observation.md` §2 (concealment by vector), §3 (σ),
  §5 (the asymmetric leak), §6.2 (acceleration as the observable), §6.5 (SPRT),
  §9.2 (laden hulls are conspicuous)
- `Hyades_trees_and_card_value.md` §2.3.6 — the Politics objective, and why it is
  Expansion exactly until `φ_ij` is instrumented
- `Hyades_card_contract.md` §1, §6 · `Hyades_netcode.md` §1.1, §2d, §8.1
- CLAUDE.md design laws #1, #7, #11, #13, #15, #16

**External**
- Double coincidence of wants: https://en.wikipedia.org/wiki/Coincidence_of_wants
- Continuous double auction: https://en.wikipedia.org/wiki/Double_auction
- Bertsekas, *auction algorithms* — the prior art behind `matching.rs`:
  https://web.mit.edu/dimitrib/www/Auction_Encycl.pdf
- Hold-up problem — why settlement-on-delivery is the right shape:
  https://en.wikipedia.org/wiki/Hold-up_problem
- Akerlof, "The Market for Lemons" (1970) — quality uncertainty and §3's risk
  premium: https://doi.org/10.2307/1879431
- Faucet/sink as a virtual-economy primitive:
  https://en.wikipedia.org/wiki/Virtual_economy
