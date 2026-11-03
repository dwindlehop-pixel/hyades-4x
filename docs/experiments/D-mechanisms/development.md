# §D. Mechanisms — development as a profitability test

*Part of the experiments record. Nothing here is normative. Table of contents: [`../README.md`](../README.md); how the record is organized: [`../AGENTS.md`](../AGENTS.md).*

---

## D.66 The development test: a tolerance sweep, a forecast sweep, and a census

*Supports `Hyades_industry.md` §1.8 and the resolution of R-IND22
([#199](https://github.com/dwindlehop-pixel/hyades-4x/issues/199)), T-107 and
T-108. Bed: `examples/work_years` (3 seats, standard galaxy, 1,500-yr screen,
sampled every 50 yr), CRN seeds 1, 7, 42, 31337, replication seeds 2, 3, 5, 11.
Every figure is a paired difference against the test switched off, as a mean
over seeds ± one standard error, with the count of seeds positive. The
4,000-yr objective was not run.*

**Symbols.** `I` is a center's infrastructure stock (kt). `I'` is the stock one
whole Band up. `P` is its population (kt). `K` is its carrying capacity (kt).
`u = min(1, P / P_req(I))` is the staffing factor. `W(I, P) = I · u` is the
worked stock (kt). The **step return** is `(W(I', P_f) − W(I, P_f)) / (I' − I)`,
where `P_f` is the population the step is scored at. **Worked-years** is
`∫ Σ W dt` over owned worlds. **Standing work-years** is `∫ Σ I dt`, Growth's
objective as `Hyades_trees_and_card_value.md` §2.3.3 defines it.

### D.66.1 A tolerance `k` on today's population

Arms develop when the step return at `P_f = P` is at least `1/k`.

| k | worked-years | standing work-years | colony-years |
|---|---|---|---|
| ∞ | +5.19 ± 2.92 (3/4) | −26.69 ± 2.90 | +0.20 ± 0.19 |
| 4 | −9.34 ± 2.32 (0/4) | −73.12 ± 1.31 | +0.46 ± 0.21 |
| 2 | −28.93 ± 1.31 (0/4) | −83.43 ± 0.59 | +0.27 ± 0.16 |
| 1 | −78.98 ± 1.97 (0/4) | −95.71 ± 0.32 | +0.28 ± 0.15 |

`k = ∞` on the replication seeds: worked-years +3.75 ± 2.54 (3/4). Colony count
is identical in every arm on every seed.

### D.66.2 Census: when a step's people arrive

A scratch build printed, at each production decision where deepening is
possible, the time until the logistic forecast reaches `P_req(I')`. Seeds 1
and 7, 1,500 yr; 579,032 and 703,776 decisions.

- 19.1% / 17.2% are staffed already.
- 58.2% / 60.3% can never be fully staffed, because `P_req(I') > K`. Every one
  of these is a Band III to IV step.
- Band II to III steps have a median time-to-staffed of 95.4 / 102.1 yr.
- 47.1% / 44.6% have a negative step return at today's population. Within a
  Band segment `P_req ∝ I^1.5`, so `W ∝ P / √I` once understaffed.

### D.66.3 A forecast on the logistic

`P_f` is the closed-form logistic forecast `H` years ahead (`H = ∞` reads `K`).
The `H = 0` arm reproduces D.66.1 bit for bit (checked at 400 yr, seed 1).

| arm | worked-years | standing work-years | colony-years |
|---|---|---|---|
| H = 50 yr, k = ∞ | +5.24 ± 2.81 (3/4) | −23.95 ± 1.04 | +0.25 ± 0.13 |
| H = ∞, k = ∞ | +1.21 ± 1.32 (3/4) | −18.62 ± 1.27 | −0.00 ± 0.02 |
| H = 50 yr, k = 2 | −23.46 ± 1.51 (0/4) | −81.06 ± 0.53 | +0.41 ± 0.23 |
| H = ∞, k = 2 | −27.67 ± 1.81 (0/4) | −78.30 ± 0.36 | +0.01 ± 0.02 |

Replication seeds: `H = 50, k = ∞` worked-years +2.87 ± 1.24 (3/4), pooled over
8 seeds +4.05 ± 1.49 (6/8). `H = ∞, k = ∞` +1.03 ± 0.87 (3/4), pooled
**+1.12 ± 0.73 (6/8)**, inside 2 SE. Standing work-years −19.06 ± 0.64 (0/8),
colony-years +0.01 ± 0.01.

**What it refuted.** The inference after D.66.1 was that a finite `k` loses
output because it reads today's population and refuses stock people will
arrive to staff. Forecasting population did not recover the loss at `k = 2`,
so that inference is refuted. The remaining inference, not proven, is that
minerals are slack on this bed, so any step that adds worked stock is worth
its price and any threshold above zero forgoes some.

**What was ratified** (the author, 2026-11-03): `H = ∞` and `k = ∞`. The test
refuses only a step that the world could never work. Its measured effect on
worked-years is inside noise, and it removes 19% of the standing stock, which
is idle stock.
