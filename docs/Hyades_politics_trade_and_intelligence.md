# Hyades — Politics: Trade, Exchange, and Shared Intelligence

*Draft Rev 1, proposed for ratification. Specifies the two systems the Politics
tree needs and neither of which exists yet: an **exchange for value** and
**granular shared intelligence**. Companion to `Hyades_matching.md` (which
already supplies the matching engine), `Hyades_standing_layer_and_observation.md`
(std §, the observation model this trades in), `Hyades_card_contract.md`
(card §), and `Hyades_netcode.md` (net §). New calls continue an **R-P n**
series.*

---

## 0. The thesis, and why it is the whole design

> **Eliminate the value of collusion by making the simulation-state effects of
> collusion available without a confederate.**

Out-of-band collusion is unpoliceable — net §2d says so flatly, and it is right:
two players on a voice call are outside any protocol. Every game that tries to
*forbid* table talk either fails or becomes an etiquette argument.

So do not forbid it. **Price it at zero.** Enumerate what a private agreement
actually buys, and sell each of those effects, individually, to a single player
acting alone:

| What collusion buys | Bought alone by | § |
|---|---|---|
| Resource complementarity — "you're Cyan-rich, I'm Magenta-rich, swap" | the Exchange: post a bid, any seller fills it | §2 |
| Shared vision — "their fleet is at Kepler" | Intelligence: buy the observation, or take it | §5 |
| Denial — "don't sell to them" | **Corner** and **Embargo**: outbid or tax, no partner needed | §4 |
| Coordinated timing — "we both strike in round 5" | **Solicit**: open a timed window and let the market coordinate | §3.4 |
| Non-aggression — "I won't hit you" | **imposed trade** (§6): interdependence you did not agree to | §6 |

The last row is the one that carries the design, and it is why **Politics cards
are not opt-in** (§6). If I can make you my supplier without asking, I have
bought the *effect* of a non-aggression pact unilaterally, in the open, at a
posted price. A back-room deal then adds nothing but coordination overhead — and
it is strictly worse than the card, because the card is enforced by the
simulation and the handshake is enforced by nothing.

**The autopilot premise is what makes this affordable.** Every participant is an
`Autopilot`, so the economy never has to be legible to a human in real time. It
can be a genuine continuous double auction with escrow, risk premia, and
distance-discounted clearing, running at simulation speed, because nobody is
clicking. Games that must present a market to a human hand simplify it into a
fixed price list; this one does not have to.

---

## 1. What already exists, and what this spec adds

**`src/matching.rs` is the substrate and it is already built** — deterministic
order-book matching, bids and asks carrying one scalar (`pressure`), highest
first, nearest-within-that, partial fills, reserved quantity, unmatched
remainder queued. It is dependency-free and has no `HashMap` anywhere, so it is
already network-safe under net §6.

This reclassifies a chore. **T-01 ("wire `matching.rs` into `lib.rs`") is not
cleanup — it is the first step of the Politics tree**, and it should be
re-read that way in `hyades_todo.md`.

What matching.rs does *not* have, and this spec adds:

| Missing | § |
|---|---|
| A means of exchange (`$`) distinct from `pressure` | §2 |
| Cross-empire books — today it is one book per (owner, commodity) | §3.1 |
| Escrow, settlement on delivery, and default | §3.3 |
| A travel-time discount | §3.2 |
| Timed bid windows on events | §3.4 |
| Intelligence as a tradeable commodity | §5 |
| Non-consensual initiation | §6 |

---

## 2. `$` — the means of exchange

### 2.1 `$` is not mass, and that is load-bearing

Design law #11 conserves mass **with no exclusions**. If `$` were a commodity it
would be a mass in kilotons and would have to be conserved, which would make a
faucet illegal and the whole economy a closed barter system.

So: **`$` is a claim, not a substance.** A ledger entry, an obligation, a
reputation-weighted promise to deliver. It has no mass, occupies no hold, cannot
be mined, cannot be shot down, and does not appear in the mass ledger at all.
Conservation is untouched because `$` was never in it.

This is not a dodge. It is the actual economics: what crosses interstellar
distance in a trade is *the mineral*, and the mineral is conserved. The `$` is
the accounting that decided which direction it went.

**R-P1 — ratified: `$` has zero mass.** It sits outside the mass ledger
entirely, so design law #11 is untouched and a faucet is legal. The state digest
(net §8.1) gains a `$`-ledger leaf of its own rather than folding into
`players`, since a claim is not a property of a planet or a hull.

### 2.2 Why a numeraire at all, rather than barter

Two reasons, and the second is the one that matters.

1. **Barter needs a double coincidence of wants.** A matching engine over
   heterogeneous goods without a common scalar is a constraint-satisfaction
   problem, not an auction.
2. **You cannot buy what you do not want, and denial requires exactly that.**
   The brief calls for a deep-Politics player outbidding an enemy for *crucial
   minerals*. Under barter you can only acquire what you can pay for in goods
   the seller wants; under `$` you can buy a mineral you have no use for,
   purely so your enemy does not get it. **Denial is the Politics tree's
   attack, and it is impossible without a numeraire.**

### 2.3 Faucet and sink — four models, and the recommendation

The sink is the hard half. A pure transfer economy plus any faucet inflates
until `$` stops discriminating between bids.

| | Faucet | Sink | Verdict |
|---|---|---|---|
| **M1 Closed** | none; fixed endowment at genesis | none | Elegant and inflation-proof, but a player who never trades is illiquid forever and early trade becomes compulsory rather than chosen. **Rejected** — it removes the decision. |
| **M2 Volume-minted** | minted on trade completion | none | Rewards churn, inflates without bound, and wash-trading with a confederate becomes the dominant strategy. **Rejected outright** — it *pays* for collusion, inverting §0. |
| **M3 Pop-backed** | accrues per pop per cycle | card costs | Ties liquidity to empire size, which double-counts the snowball: the biggest empire also gets the deepest purse. **Rejected as sole model**, kept as a component. |
| **M4 Transit-burn** ✅ | pop × Politics depth | **the travel-time discount itself** | Recommended. See below. |

**The recommendation, M4, and the reason it is more than a compromise: the sink
and the travel-time discount are the same mechanism.**

The brief asks for "a discount for time to travel to complete the transaction."
Make that discount a *burn* rather than a rebate:

```
escrowed at match:   E                       (buyer's $, locked)
seller receives:     E · exp(−λ · t_transit)
burned:              E · (1 − exp(−λ · t_transit))
```

One mechanism, four jobs:

- **The travel-time discount**, as asked, with `λ` its single tunable.
- **A real sink**, scaling with trade volume *and* distance, so the economy
  self-regulates: more trading burns more `$`.
- **Geography enters the economy.** A near partner is strictly better than a
  far one at the same price, which means trade has a *map*, and the map is the
  one the rest of the game already plays on.
- **Denial is expensive.** A cornering bid pays full escrow and burns the
  transit share, so buying purely to deny costs real purchasing power. Denial
  should be available, not cheap.

**Faucet: production, not population (R-P3 — ratified against the earlier
recommendation, and the author's reasoning is better than mine).**

```
$_income = base · production · politics_multiplier(depth)
```

I had recommended population on the grounds that production rewards the
mobilized. The correction: **production is the sum of both halves of an
economy** — population growth *and* infrastructure deepening both feed it,
where population alone counts only one. An empire that has invested in infra
rather than bodies is not poorer, and a pop-only faucet would say it was.

It also removes the double-count I was worried about from the other side: the
snowball compounds through *colonies*, and production is what those colonies
actually do, so income tracks the thing being built rather than the headcount
riding on it.

**Politics depth buys purchasing power, not merely access** — that is what makes
the tree's late nodes able to outbid a warring rival directly (§4.3).

**R-P2 — the `λ` decay sink is ratified, conditionally.** The condition is
that it must also be *the* solution to freighter routing, not merely compatible
with it.

That is a real claim about the engine, and a strong one. Today a laden freighter
picks the highest-pressure owned center with **no distance term at all**
(`most_needed_center`), so it will cross the galaxy for a marginally needier
destination. If internal haulage is a trade you clear with yourself, the same
discount should price it:

```
route to argmax over owned centers of   mineral_pressure(center) · exp(−λ · t_transit)
```

**`λ = 0` reduces exactly to `most_needed_center`**, which is both the shipped
default and — pleasingly — the same oracle design law #5 already keeps for
single-supply matching. One function now checks two independent degeneracies.

**Measured, and the condition is met by a wide margin.**
`examples/lambda_routing.rs`, 3 seats / 3 seeds / 4,000 yr:

| λ | half-life | mean coverage |
|---|---|---|
| 0 (`most_needed_center`) | ∞ | 14.35% |
| 0.002 | 347 yr | 27.71% |
| 0.005 | 139 yr | 35.20% |
| **0.010** | **69 yr** | **39.04%** |
| 0.020 | 35 yr | 36.88% |
| 0.050 | 14 yr | 36.74% |

A genuine interior optimum and **2.7× the shipped baseline** — a larger effect
than the entire five-parameter doctrine search produced. Ratified as
`SimConfig::trade_decay_lambda = 0.01`.

The scale is physically sensible rather than merely fitted: a laden hop of
10–30 ly at 1 g takes 20–45 years, so a 69-year half-life discriminates exactly
at the range real hauls happen. Below that the discount is too sharp and
freighters stop serving genuinely needy distant centers; above it, need swamps
distance again and the rule degenerates toward `most_needed_center`.

**What this says about the design, beyond the number.** The Exchange discount
was proposed as a `$` sink that happened to give the travel-time behaviour the
brief asked for. It turns out to be the *correct routing rule for the existing
economy*, independently of trade — which is the strongest kind of evidence for a
mechanism: it pays for itself before the system it was designed for exists.

Three seeds is thin for a ratified constant. The value is confirmed in direction
and order of magnitude; the precise optimum wants the ten-seed bed (T-44). The
base income rate and the Politics depth multiplier remain open MC surfaces.

---

## 3. The Exchange

### 3.1 Books go cross-empire

Today `matching.rs` runs one `Book` per `(owner, commodity)` — an intra-empire
logistics matcher. Politics needs **one book per commodity, spanning empires**,
which the module's own header already anticipates ("or per commodity globally
once cross-empire bidding lands").

Determinism is unaffected: the book is already ordered by pressure then entity
id, and entity ids are globally unique.

### 3.2 Price, and willingness to pay

`pressure` becomes a `$` price, and the bid a player posts is derived, not
chosen by a human:

```
wtp(mineral) = base_value(mineral)
             × doctrine_demand(mineral)      // Doctrine that wants it, wants it more
             × shortfall_pressure(center)    // mineral_pressure_of, already in the engine
             × risk_discount(counterparty)   // §3.5
```

`Simulation::mineral_pressure_of` already computes the third term. The first two
are new and both belong on `Doctrine` — which is exactly the diplomatic-fields
slot **T-11/R-O27** has been holding open with no field list. **This spec is
that field list** (§7).

### 3.3 Escrow, settlement, default, theft

A cleared match is **not** a completed trade. It is a contract with a physical
leg:

1. **Match.** Buyer's `$` moves to escrow. Seller's cargo is reserved.
2. **Ship.** A laden hull flies the goods. It is an ordinary vehicle — it has an
   acceleration signature (std §6.2), it can be observed, and **it can be shot**.
3. **Settle on delivery.** Escrow releases per §2.3, discounted and burned by
   transit time.

Three ways it goes wrong, all of them intended:

- **First-party default** — the buyer takes delivery and repudiates, or the
  seller takes escrow and never ships. Available as the **Default** verb.
- **Third-party interdiction** — someone else takes the cargo in flight. This
  needs no new mechanism: it is a laden freighter in open space.
- **Non-delivery** — the hull is destroyed en route. Escrow returns to the
  buyer minus the burn. Loss is shared, which is what makes escorting worth
  paying for.

**Reputation is mechanical, not social.** A defaulter's counterparties raise
their escrow requirement and discount their bids — `risk_discount` in §3.2. No
human judgement, no table talk, no appeal: the autopilot prices you. This is the
only workable design when every participant is a program, and it is *better*
than a social norm because it is legible and exactly as forgiving as its decay
constant says.

**R-P4 — resolved: reputation is public by default, and a card switches *you*
to per-observer.** Not a phase-in of one model toward the other; both ship, and
which one you use is a purchase.

- **Public (default).** One consensus number per player. Cheap — O(N) storage —
  coarse, and **gameable by anyone who can manufacture visible defaults**. It is
  a shared signal, which means it is also a shared attack surface.
- **Per-observer (bought).** You price every counterparty from *what you
  personally saw*. Correct under the observation model, and strictly more
  information: you can deal with someone the public record has blacklisted but
  who has never actually burned *you*, and you can refuse someone the table
  still trusts.

Three things fall out, and the third is the reason to prefer this over either
model alone:

1. **It makes the observation model itself a purchasable upgrade**, which is
   exactly what a Politics tree should sell — not a bigger number, but a better
   epistemic position.
2. **It is a real counter to reputation attacks.** Manufacturing public defaults
   to poison a rival's credit stops working against anyone who has bought out of
   the consensus. That gives the tree an internal counter rather than needing
   one from outside.
3. **It bounds the storage cost the engine was worried about.** Per-pair
   reputation is the same expensive representation T-33/T-26 flag at fleet
   scale — and here **only the players who bought it pay for it.** The cost
   scales with adoption instead of with seat count, and a table where nobody
   buys the card costs nothing at all.

Still open: the **decay rate** on both models, and whether a per-observer player
still *contributes* to the public number (recommend yes — you can leave the
consensus without leaving the record, and a defector who could also go silent
would be too strong).

### 3.4 Solicitation — timed bid windows on events

> *"The system must solicit bids on events with a fixed time budget."*

An **event** is any simulation occurrence with contestable value: a world
scanned, an outpost exhausted, a fleet detected, a colony founded, a hull
completed. Soliciting opens a book on it that closes after `window_years`.

The window is a **round-layer** object, not a wall-clock one (net §1.1 forbids
host clocks, and a bid window driven by a local timer is a desync). It opens on
a sim event and closes at a sim time, so every client computes the same
clearing.

This is also the answer to collusion's "coordinated timing" row in §0: a timed
window *is* a coordination mechanism, open to everyone, requiring no confederate.

**R-P5** — the event taxonomy that admits solicitation, and `window_years`.
Recommend the window be a fraction of `years_per_round` so it composes with the
round layer rather than fighting it.

### 3.5 Risk premium — how mobilization blunts trade

See §8. The short form: an armed fleet is *visible* (std §6.2), and visibly
armed players get worse prices.

---

## 4. The Politics verbs

The tree's vocabulary. Every one is **unilateral** — that is the §0 test, and a
verb that fails it does not belong here.

### 4.1 Market verbs

| Verb | Effect | Tier |
|---|---|---|
| **Offer** | Post an ask: commit stock at a reserve price | 0 |
| **Bid** | Post a bid at your derived WTP | 0 |
| **Consign** | Ship before a buyer exists — pay transit early, clear on arrival. Trades price for speed | 1 |
| **Underwrite** | Lower a named counterparty's escrow requirement. A favour with a price, and the seed of a bloc | 1 |
| **Broker** | Clear through a third party, so a trade completes between players who cannot deal directly (§4.3) | 2 |
| **Embargo** | Raise the effective `λ` on a named counterparty's trades — tax their distance, not their price | 2 |
| **Corner** | Bid across an entire mineral *class* rather than a lot. Denial at scale | 3 |
| **Default** | Take without paying. Reputation cost, no legal cost | any |
| **Interdict** | Seize goods in transit. A Warfare action with a Politics payoff — the natural combo edge | — |

### 4.2 Intelligence verbs

| Verb | Effect | Tier |
|---|---|---|
| **Disclose (own)** | Publish your own observations. A gift, or a bid for a bloc | 0 |
| **Disclose (other)** | Publish **someone else's** Design or Doctrine. Transparency as an attack (§5.3) | 2 |
| **Solicit** | Open a timed window on an event (§3.4) | 1 |
| **Recognize / Denounce** | Move a counterparty's risk premium directly, up or down | 2 |
| **Audit** | Switch this empire from the public reputation consensus to **per-observer** pricing — judge counterparties by what you saw, not by what the table says (§3.3, R-P4) | 2 |

### 4.3 Trading with someone you are at war with

The brief's specific requirement, and the tree's clearest depth gradient:

- **Tier 0–1:** you cannot. Books exclude counterparties you are at war with.
- **Tier 2 — Broker.** A third party clears the trade. The minerals reach you;
  the broker takes a cut; your enemy sold into a market and cannot tell to whom.
  **This is the collusion effect bought without a colluder** — the third party is
  a market participant maximizing its own return, not an ally.
- **Tier 3 — direct.** Deep Politics buys the ability to outbid an enemy for
  crucial minerals *in the open*. They can see you doing it and cannot stop it
  except by outbidding you, which costs them the same `$` they wanted the
  minerals for.

The mid-tier/endgame split the brief asks for falls straight out: brokerage is
indirection, and depth removes the need for it.

---

## 5. Shared intelligence

### 5.1 What is tradeable, and the ordering principle

std §5 establishes the **asymmetric leak**: *Design never goes stale; Doctrine
dies on retasking.* That ordering is the price ladder — the longer an
observation stays true, the more it is worth.

| Tier | Commodity | Half-life | Why it sits here |
|---|---|---|---|
| **0** | **Planetary scan data** | decays with the game phase | Worth a great deal while the map is dark and **nothing at all once everyone has scanned everything**. A tier-0 card whose value is game-phase dependent by construction, which is exactly what a mouth card should be |
| 1 | Movement / trajectories | hours-to-years; stale on arrival | Perishable, high tactical value, and it is what makes a reachability cone (R-O31) worth buying |
| 2 | Doctrine | until retasking | A distribution over intent, not a fact — buying it buys SPRT samples (std §6.5), not certainty |
| 3 | **Design** | **permanent** | A roster entry never goes stale, so it is the most valuable thing on the board and the most damaging to have published |

### 5.2 Scan data is the right tier-0 card

Its value curve is the argument. Early, the galaxy is dark and a scan is
decisive; late, coverage is near-total and the same card is waste paper. A
tier-0 card that is strong in round 1 and dead by round 8 needs no balance
scaffolding to stop it dominating — **the game phase does the balancing**, and
the player's judgement about *when* it stops being worth an action is the skill.

It also composes with the coverage objective the engine is currently tuned
against, which makes it the one Politics card whose value can be measured today.

### 5.3 Disclosure is the attack

The non-obvious move, and the one that most directly serves §0.

Publishing your *own* intelligence is a gift. Publishing **someone else's** is an
attack — and it is the Politics tree's answer to a Design advantage. A player who
has quietly unlocked a superior hull has bought concealment (std §2: Design
conceals *spatially*, you must come close to read the fit). **Disclose (other)**
takes what one player paid to learn and hands it to everybody.

Two properties make this the right shape:

- **It is not theft of a thing.** The victim loses no mass and no `$`. They lose
  an *information asymmetry*, which is the only thing the Politics tree should
  be able to take.
- **It kills the collusion channel dead.** "My ally told me their Design" is a
  private, unpriced, unpoliceable transfer. If the same effect is a card any
  single player can buy, the private version is worth nothing.

### 5.4 Who receives a disclosure (R-P6 — resolved: it depends on the card)

Not one answer. The recipient rule **bifurcates with depth**, and the direction
it bifurcates in is the interesting part:

| | Recipient | Where it sits |
|---|---|---|
| **Tier 0** | **everyone** | the mouth card; no targeting, no choice to make |
| **Deep / win-condition branch** | **everyone**, still | broadcast is the win path |
| **Shallower branch** | **a chosen recipient** | targeted, and worth *less* |

The counterintuitive part is which one is stronger, and it falls straight out of
§0. **Targeted disclosure is closer to actual collusion** — you pick a
confederate and hand them something nobody else gets — so it must be worth less,
or the tree would be paying players to do the thing the design is trying to make
worthless. **Broadcast is the thesis-aligned act**, so broadcast is where the
win-object lives.

It is also self-limiting in a way targeting is not: you cannot weaponize a
broadcast against one rival without arming the whole table, which means a
disclosure war escalates against its own initiator. Targeting has no such brake,
which is the other reason to price it as the cash-out rather than the payoff.

**This makes a player argument part of card selection**, not just card
resolution: a targeted disclose is a different play from an untargeted one and
must be chosen as such. The engine already carries this —
`cards::Target::Player` and the `needs_subject` flag — but note the shape it
implies once both indices exist: a *targeted disclose about a third party* takes
**two** player arguments, a recipient and a subject. `Target` is currently a
closed set with room for exactly one referent, and the wire protocol's
`target_kind`/`target_ref` pair (net §4.2) is sized for one. **R-P15** — does
the target set need a two-player variant, and does that push `target_ref` wider
than R-NET4 currently assumes?

---

## 6. Politics cards are not opt-in

> *"An opponent may initiate trade or shared intelligence without your consent
> as a player. There exist counters in other cards."*

This is the rule that makes §0 work, so it is worth being precise about what it
does and does not mean.

**What can be imposed on you:**

- A trade relationship. Someone can become your supplier, or your customer,
  without your agreeing.
- Intelligence about you can be published.
- Intelligence can be pushed *to* you — and you cannot un-know it. (Consider
  this a real cost: your autopilot will act on it.)
- Your goods can be bid for, cornered, or embargoed.

**What cannot:**

- Your minerals cannot be taken without either payment or a physical seizure
  (Interdict — a Warfare act, resolved by combat).
- Your Doctrine and Design cannot be *written* by another player. Only read,
  and only published.

**Why imposed trade is not merely an annoyance.** Because of §8's supply-chain
effect, becoming dependent on a supplier is a real strategic state — and the
supplier chose it for you. A player who imposes trade on a rival is buying
future restraint from that rival, unilaterally. That is a non-aggression pact
with no counterparty, which is the single most valuable thing collusion offers
and the hardest to price. Here it has a price.

**Counters live in other trees**, which is design law #7's combo requirement
made concrete: Warfare's Interdict takes the goods; Growth's autarky reduces the
`doctrine_demand` that makes you biddable; Industry's substitution (design law
#1's counter-graph) routes around the cornered mineral.

**R-P7** — the counter list, per tree, with costs. This is the balance surface
of the entire tree and it cannot be set analytically.

---

## 7. Relationships — the thing another player *can* write

### 7.1 The line: Doctrine is yours, your view of others is not

The rule that makes §6 precise, and it is sharper than "cards are not opt-in":

> **Your Doctrine cannot be written by another player. Your empire's view of
> another player's empire absolutely can.**

Doctrine is your policy — how hard you grow, how far you survey, how much you
reinvest. Nobody else writes it. But *who you take that policy to be about* —
whether this empire over here is a friend, a foe, a mercenary, a trade partner —
is a separate piece of state, and it is exactly the surface a Politics card
operates on.

That split is what lets §6's "not opt-in" have teeth without letting an opponent
pilot your empire. They cannot make you expand faster. They can make you treat
them as a supplier, or make you treat a third party as a threat.

### 7.2 Stance is granular, and it is per-ordered-pair

**Not a binary flag.** A boolean `at_war` collapses every distinction the tree
is about — a mercenary is not a friend, a trade partner is not an ally, a rival
you still sell to is not a foe. The stance set is a small closed enum, and
**every Doctrine must specify conduct for every stance**, not for "hostile vs
not":

| Stance | What it means operationally |
|---|---|
| `Unknown` | never contacted; no basis for conduct at all |
| `Neutral` | contacted, no relationship — the default after first contact |
| `TradePartner` | clears on the Exchange, low escrow, no denial bidding |
| `Mercenary` | transactional; will deal, will also take a better offer |
| `Client` / `Patron` | asymmetric: one side underwrites the other (§4.1) |
| `Rival` | competes for the same worlds; still trades, at a premium |
| `Foe` | no direct clearing (brokerage only, §4.3); interdiction legal |

Stance is stored **per ordered pair** — `stance[me][them]` — and is *not*
symmetric. You may see them as a trade partner while they see you as a mark.
That asymmetry is not an edge case to be normalised away; it is the entire
content of a successful deception, and it is what a disclosure attack (§5.3)
collapses when it publishes the truth.

### 7.3 Conduct is a table, not a branch

The consequence for `Doctrine`: it carries a **conduct row per stance**, so
every interaction resolves by lookup rather than by an `if hostile` branch.

```rust
pub struct Conduct {
    /// Multiplier on willingness to pay when this counterparty is the seller.
    pub trade_appetite: f64,
    /// Escrow demanded of them, as a multiple of contract value (§3.3).
    pub escrow_ratio: f64,
    /// Willingness to pay above derived WTP purely to deny them (§4.1).
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
    /// Per-mineral demand multiplier — the WTP term of §3.2.
    pub demand: [f64; N_BASIC],
    /// Fraction of income committed to the book per round.
    pub trade_budget: f64,
    /// **One row per stance.** Not a list of exceptions.
    pub conduct: [Conduct; N_STANCE],
}
```

Three things this buys, beyond legibility:

1. **`excluded` disappears, and R-P8 dissolves with it.** The earlier draft
   carried a `Vec<PlayerId>` of counterparties to refuse, which was a per-player
   categorical and had to be checked against design law #13. There is no list
   now — refusal is `conduct[Foe].clears_directly == false`, a policy about a
   *kind* of relationship rather than about named players.
2. **It gives the belief layer its missing input.** `belief::decide_engagement`
   takes an `Unobserved` policy for contacts never seen, and the honest default
   was `PeerOf`. Stance supplies a better one: what you assume about a fleet you
   cannot measure should depend on whose it is.
3. **Imposing a stance is a real attack with a bounded blast radius.** Writing
   `stance[victim][me] = TradePartner` changes how the victim's autopilot prices
   and treats you, using *the victim's own* conduct table. You have not taken
   their policy — you have moved yourself within it. That is precisely the
   non-aggression effect §0 wants purchasable alone.

### 7.4 The strongest form: writing a stance you are not party to

`stance[me][them]` has two indices, and the Politics attack is available on
both:

- **`stance[victim][me]`** — make them see *you* differently. Imposed trade,
  imposed truce, imposed dependence.
- **`stance[victim][third_party]`** — make them see *someone else* differently.

The second has no counterpart in a two-player negotiation, and it is the
sharpest expression of §0 in the whole design. *"Get two other players to fight
each other"* is among the highest-value things a real alliance buys, and it
normally requires a confederate to arrange. Here it is a card, played alone, in
the open, at a posted price — which is exactly the trade this spec exists to
make.

**R-P13** — which stances may be written, by which tier, and on which index.
Recommend: the near index (`stance[victim][me]`) unlocks first and only toward
*less* hostile — you can make yourself a trade partner, not make yourself
trusted — and the far index (`stance[victim][third_party]`) is a deep node,
since manufacturing a war between two other empires should cost most of a tree.

**R-P14** — does an imposed stance decay? A permanent write is a permanent
non-aggression pact for one card, which is too strong. Recommend decay on the
same clock as reputation, so maintaining an imposed relationship costs actions
rather than being bought once.

---

## 8. The counter-graph: trade and mobilization blunt each other

> *"Initiating trade early should blunt the impact of later mobilization, and
> early mobilization should blunt the effects of later trade. This must be
> legible to both trees. The tradeoff should come from the effect on the
> simulation state."*

The requirement is that neither direction is a *modifier*. No "Politics gets
−20% vs Warfare" table. Both effects must be consequences of state that the
simulation is already tracking, and both must be readable from outside.

### 8.1 Early trade blunts later mobilization — the supply-chain effect

Mobilizing consumes minerals. If a share of your mineral inflow arrives through
the Exchange, then **mobilizing against your supplier cuts the supply that the
mobilization is made of.** Your fleet comes out smaller than the plan, and it
comes out smaller *because* of a choice you made two hundred years earlier.

No modifier is applied anywhere. The state is: minerals arrived by trade, they
are in the stockpile, the flow stops when the relationship does.

**Legible from outside** as freighter traffic — a trade-dependent empire has a
visible pattern of laden hulls arriving from a foreign origin, and under the
shell model a laden hull is *conspicuously* slow (std §9.2). You can see who
depends on whom by watching who is sending whom slow ships.

### 8.2 Early mobilization blunts later trade — the risk-premium effect

Arming is loud. `a = thrust/(dry + cargo)` and a warship's signature is high and
tightly clustered (std §3); a visibly mobilized empire is a *credit risk*. Its
counterparties raise `escrow_ratio` and apply `risk_discount`, so its `$` buys
less and its goods clear worse.

Again no modifier: the state is an observed acceleration distribution, and the
risk term is a function of it.

**Legible from outside** because it *is* the observation — the same burn
signature the yomi layer already runs on, priced.

### 8.3 Why this makes the edge time-dependent

Both effects **accumulate**. Trade dependence builds with volume over time;
reputation as an armed power builds with observation count over time (std §6.5's
SPRT — detection needs samples, and samples take rounds). So the counter-graph
edge between Politics and Warfare is not a constant: **it is a function of when
each player started**, which is exactly the "time-dependent counter graph" the
brief asks for.

Concretely, the strategic reading a player can make:

| | opponent trades early | opponent arms early |
|---|---|---|
| **you trade early** | mutual dependence; both mobilizations are weak; the game goes long | you supply someone who will hit you — profitable and dangerous |
| **you arm early** | you pay a premium for everything, but they cannot hit back hard | conventional arms race; Politics is irrelevant to both |

The bottom-left and top-right cells are where the yomi lives, and neither is
dominated.

**R-P9** — the strength of both effects. The supply-chain effect is bounded by
the trade share of mineral inflow, which is self-limiting; the risk-premium
effect is not obviously bounded and could make early arming unplayable. Needs MC.

---

## 9. Open ratification points

| Code | Question | Blocked on |
|---|---|---|
| **R-P1** | `$` sits outside the mass ledger; state digest needs a `$` leaf | net R-NET5 |
| **R-P2** | `λ` (transit burn), base income rate, Politics depth multiplier | MC |
| **R-P3** | `$` income on population or on production? Recommend population | MC |
| **R-P4** | ~~Reputation decay; public vs per-observer~~ **resolved**: public by default, a card switches the *buyer* to per-observer (§3.3). Decay rate still open | decay: MC |
| **R-P5** | The event taxonomy admitting solicitation, and `window_years` | round layer |
| **R-P6** | ~~Does Disclose (other) publish to everyone?~~ **resolved**: depends on the card — tier 0 and the win-condition branch broadcast, the shallower branch targets and is worth less (§5.4) | — |
| **R-P15** | A targeted disclose *about a third party* needs two player arguments. Does `Target` need a two-referent variant, and does it widen `target_ref` past what R-NET4 assumes? | R-NET4, R-C1 |
| **R-P7** | The per-tree counter list with costs — the tree's whole balance surface | MC, R-P2 |
| **R-P8** | ~~Is `Diplomacy::excluded` an exception to design law #13?~~ **dissolved** — there is no `excluded` list. Refusal is `conduct[Foe].clears_directly`, a policy about a kind of relationship rather than about named players (§7.3) | — |
| **R-P13** | Which stances may be written, by which tier, and on which index. Recommend near index first and only toward less hostile; far index (making two other empires enemies) a deep node | — |
| **R-P14** | Does an imposed stance decay? Recommend yes, on the reputation clock — a permanent write is a permanent pact for one card | MC |
| **R-P9** | Strength of both counter-graph effects; is the risk premium bounded? | MC |
| **R-P10** | Does the Exchange clear once per round, or continuously on the event queue? Per-round is simpler and matches the barrier; continuous is truer to a discrete-event engine | round layer |

---

## References

**Internal**
- `Hyades_matching.md` — the matching engine this builds on; `src/matching.rs`
- `Hyades_standing_layer_and_observation.md` §2 (concealment by vector), §3 (σ),
  §5 (the asymmetric leak), §6.2 (acceleration as the observable), §6.5 (SPRT),
  §9.2 (laden hulls are conspicuous)
- `Hyades_card_contract.md` §1 (a card is `(costs, target)`), §6 (deterministic value)
- `Hyades_netcode.md` §1.1 (no wall clocks), §2d (collusion is out of scope for
  any protocol — the premise of §0), §8.1 (the state digest)
- CLAUDE.md design laws #1 (counter-graph), #7 (combo cards), #11 (mass
  conservation), #13 (no colour-coextensive classification), #15 (the seam)

**External**
- Double coincidence of wants — the barter problem a numeraire solves:
  https://en.wikipedia.org/wiki/Coincidence_of_wants
- Continuous double auction:
  https://en.wikipedia.org/wiki/Double_auction
- Bertsekas, *auction algorithms* (prices as the matching scalar) — already the
  prior art behind `matching.rs`:
  https://web.mit.edu/dimitrib/www/Auction_Encycl.pdf
- Escrow and the hold-up problem — why settlement-on-delivery is the right shape:
  https://en.wikipedia.org/wiki/Hold-up_problem
- Akerlof, "The Market for Lemons" (1970) — quality uncertainty and the risk
  premium of §3.5: https://doi.org/10.2307/1879431
- Faucet/sink as a virtual-economy primitive:
  https://en.wikipedia.org/wiki/Virtual_economy
