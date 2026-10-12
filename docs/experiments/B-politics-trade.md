# §B. Politics, Trade and Intelligence

*Part of the experiments record. Nothing here is normative. Table of contents: [`README.md`](README.md); how the record is organized: [`AGENTS.md`](AGENTS.md).*

*Moved out of `docs/Hyades_politics_trade_and_intelligence.md` at Rev 2. That
spec had reached 1,125 lines, roughly half of it implementation history.*

---

## B.1 R-P2 — λ, and a trade mechanism that paid for itself before trade existed

The transit burn was proposed as a `$` sink that happened to give the
travel-time behavior the brief asked for. The ratification condition was
stronger: it had to also be *the* solution to freighter routing.

Before it, a laden freighter picked the highest-pressure owned center with **no
distance term at all** (`most_needed_center`), so it would cross the galaxy for a
marginally needier destination. `λ = 0` reduces exactly to `most_needed_center`,
which is also the oracle design law #5 keeps for single-supply matching — one
function checking two independent degeneracies.

**Harness:** `examples/lambda_routing.rs`, 3 seats / 3 seeds / 4,000 yr.

| λ | half-life | mean coverage |
|---|---|---|
| 0 (`most_needed_center`) | ∞ | 14.35% |
| 0.002 | 347 yr | 27.71% |
| 0.005 | 139 yr | 35.20% |
| **0.010** | **69 yr** | **39.04%** |
| 0.020 | 35 yr | 36.88% |
| 0.050 | 14 yr | 36.74% |

A genuine interior optimum and **2.7× the shipped baseline** — a larger effect
than the entire five-parameter doctrine search produced.

**The scale is physically sensible rather than merely fitted:** a laden hop of
10–30 ly at 1 g takes 20–45 years, so a 69-year half-life discriminates exactly at
the range real hauls happen. Below that the discount is too sharp and freighters
stop serving genuinely needy distant centers; above it, need swamps distance again
and the rule degenerates toward `most_needed_center`.

**Three seeds is thin for a ratified constant.** Direction and order of magnitude
are confirmed; the precise optimum wants the ten-seed bed.

> **Cross-tree conflict found later** (`examples/tree_gradient`):
> `trade_decay_lambda` is **+0.002 on Expansion and −0.348 on Growth.** λ is the
> largest ratification in this project's history and it was measured on coverage
> alone. A single-metric probe cannot see this.

---

## B.2 The faucet and sink — four models, and why three were rejected

| | Faucet | Sink | Verdict |
|---|---|---|---|
| **M1 Closed** | fixed endowment at genesis | none | Elegant and inflation-proof, but a player who never trades is illiquid forever and early trade becomes compulsory rather than chosen. **Rejected — it removes the decision.** |
| **M2 Volume-minted** | minted on trade completion | none | Rewards churn, inflates without bound, and wash-trading with a confederate becomes dominant. **Rejected outright — it *pays* for collusion, inverting §0.** |
| **M3 Pop-backed** | accrues per pop per cycle | card costs | Ties liquidity to empire size, double-counting the snowball: the biggest empire also gets the deepest purse. **Rejected as sole model, kept as a component.** |
| **M4 Transit-burn** ✅ | production × Politics depth | **the travel-time discount itself** | Shipped. |

**M4's merit is that the sink and the travel-time discount are the same
mechanism** — one tunable doing four jobs: the discount the brief asked for, a
real sink scaling with volume *and* distance, geography entering the economy (a
near partner is strictly better than a far one at the same price), and denial
being expensive (a cornering bid pays full escrow and burns the transit share).

**The faucet is production, not population, and the author's reasoning overrode
the original recommendation:** production is the sum of *both* halves of an
economy — population growth and infrastructure deepening both feed it, where
population alone counts only one. An empire that invested in infra rather than
bodies is not poorer, and a pop-only faucet would say it was.

---

## B.3 T-77 — settlement landed, and the screen that preceded it was wrong twice

**Bed:** 3 seats, 4,000 yr.

Contracts clear, price, escrow, deliver into the buyer's pile at the shared rock,
default when the seller's bank is short the color it owes, and conserve mass.
**64,642 contracts settled and 3,484 defaulted** (seed 1); **64,553 and 3,905**
(seed 7). Only *geography* rejected anything — 47 and 31 fills out of ~68,000
found no shared rock.

**Yellow dominates the flow, which is the design goal arriving:**

| color | seed 1 delivered | seed 7 delivered | share |
|---|---|---|---|
| **Yellow** | **15,416.6 kt** | **14,380.5 kt** | **52%** |
| Cyan | 9,188.0 kt | 9,204.6 kt | 31% |
| Magenta | 4,855.0 kt | 5,012.6 kt | 17% |

That is the `3:2:1` Y:C:M works mix reproduced as *trade flow*, on both seeds,
without anything in the market being told about it. **Demand is Doctrine, and
Doctrine is the works bill** — measured, not asserted.

### The 400-planet screen was wrong twice

A 400-planet / 1,200-year probe reported **776 contracts and 327 kt** — 0.002% of
extracted mass — and the conclusion written from it was *"the market clears and
moves nothing that matters"*. The standard bed reports **64,642 contracts and
29,460 kt**, ~0.2%: **83× the contracts and 90× the volume.** The screen was not
merely imprecise, it was *qualitatively* wrong, because trade volume is
superlinear in galaxy size — more empires' worth of outposts overlap, so more
pairs can reach each other at all.

**A conclusion of the form "X does not matter" cannot be drawn from a screen at
all.** It is an absolute statement, and a screen only ever supports a relative
one.

### What it costs — and the mechanism is unproven

**Trade is measurably negative on Growth's objective**, on both seeds
(`examples/work_years`, against T-87's bed):

| | work-years | colony-years |
|---|---|---|
| seed 1, no Exchange | 1,495,212.5 | 10,888,100 |
| seed 1, with trade | 1,425,905.0 (**−4.64%**) | 10,889,400 (+0.01%) |
| seed 7, no Exchange | 1,540,712.5 | 10,983,925 |
| seed 7, with trade | 1,477,482.5 (**−4.11%**) | 10,982,875 (−0.01%) |

Colony-years is flat to a hundredth on both seeds — which is why it is not the
guard (B.5) — while work-years falls ~4.4% consistently.

**The leading hypothesis, recorded as unproven:** the two legs are not
symmetric. The seller's ore leaves its bank *immediately* at settlement, where it
was spendable; the buyer's ore lands in an **outpost pile** and stays there until
the buyer's own freighter happens to call. If collection lags delivery, trade is
a machine for moving minerals out of banks and into piles — strictly worse than
not trading, regardless of which color moves where.

That is checkable and must be checked before anything is tuned: compare banked
against piled holdings over time, and measure the dwell between a contract
settling and its ore reaching a bank. **A plausible mechanism attached to a real
number is the shape of every measurement artifact in this project.** Carried as
**R-P18**.

---

## B.4 The stage plan predicted the wrong risky stage

| # | Stage | Predicted | Actual |
|---|---|---|---|
| 1 | `$` ledger + faucet | neutral | neutral, bit-identical |
| 2 | `Commodity` gains color; `Offer` gains an owner | neutral | neutral, bit-identical |
| 3 | Cross-empire book; centers post `wtp` | neutral | neutral, bit-identical |
| 4 | Clearing at the barrier → contracts + escrow | **the risky one** | **inert** |
| 5 | The freight leg; escrow settles on arrival | — | **the risky one: −4.4% work-years** |

Stages 1–3 were deliberately inert, for the reason `Hyades_industry.md` §6.7's
stages 3–5 were: **a system that lands neutral can be verified against a
bit-identical bed before anything switches on.**

**Stage 4 turned out inert too, and the §10.6 amendment is why.** The table was
written when a cleared match was a cross-empire *delivery*, so clearing and
moving goods were one step. Once settlement moved to a shared rock, stage 4
strikes contracts and locks `$` while every kilotonne stays where it was.

**So the stage expected to be risky was not, and the risk moved with the goods.**
A stage plan is a claim about code, and an amendment to the design invalidates
the plan's predictions along with everything else it touches.

---

## B.5 Colony-years is inverted as a guard for anything that changes how minerals are spent

Seed 1 and 7, across four industry landings:

| | infra builds | colony-years, seed 1 | colony-years, seed 7 |
|---|---|---|---|
| pre-works | 1,032 | 10,558,680 | 10,474,865 |
| T-73 | 57 | 10,606,309 | 10,583,150 |
| T-81 | 31 | **10,633,441** | **10,599,130** |
| R-IND17 | 66 | 10,582,211 | 10,546,759 |

**Monotone inverse on both seeds:** colony-years *rises* as development collapses
and *falls* as it recovers, because minerals denied to infrastructure buy hulls
and a `k_high`-bound bed takes worlds earlier. **Four industry changes were
guarded on it and it rewarded the breakage every time.**

The interim guard for Exchange work is the build-mix census
(`examples/bank_mix`), which is what actually caught every defect on the works
branch. Colony-years stays as a *side-effect* read, never as the verdict.

---

## B.6 R-P17 — the venue question was wrong, not merely unanswered

**The question as framed:** which outpost do buyer and seller settle at, when
they share more than one? The answer written first minimized the two parties'
*summed* transit.

**The author's ruling made the question disappear.** A contract has **two drops,
not one venue**: the seller leaves Yellow at a shared outpost near *itself*, the
buyer leaves Magenta or Cyan at a shared outpost near *itself*. Summed transit is
the right objective only if there is a single venue for both legs, and there is
not — each shipper pays for its own leg, and one compromise venue would make each
side pay for the other's geography, which contradicts the default transaction
being *balanced in value*.

---

## B.7 R-IND10 — a register entry that was already answered

"Who bears the loss when a matched contract's carrier is destroyed" was carried
open while §3.3 already said it: on non-delivery **escrow returns to the buyer
minus the burn**, so the buyer loses the burn, the seller loses the cargo, and the
loss is shared. That is what makes escorting worth paying for.

**Kept as an entry because the failure is instructive**: an open register is only
as good as the sweep that reconciles it against the spec body, and nothing was
performing that sweep.

---

## B.8 R-P8 — a question dissolved by a better representation

"Is `Diplomacy::excluded` an exception to design law #13 (no categorical
classification co-extensive with a color domain)?" The earlier draft carried a
`Vec<PlayerId>` of counterparties to refuse, which was a per-player categorical.

**There is no list now.** Refusal is `conduct[Foe].clears_directly == false` — a
policy about a *kind* of relationship rather than about named players. The law
was never in danger; the representation was.
