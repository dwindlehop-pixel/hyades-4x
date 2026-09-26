# `data/` — raw measurement records

**T-50: the elasticities are being spent and thrown away.** Every gradient this
project has run ended up as a sentence in a doc and was then invalidated by the
next operating point, leaving nothing behind. The *raw per-evaluation* record is
the artifact worth keeping, because elasticities, standard errors and rankings
are all recoverable from it and the reverse is not true — and because this
project has changed its objective twice already, so a later reader must be able
to re-analyze without re-running.

Everything here is plain TSV: zero dependencies, diffable, appended rather than
overwritten.

## `tree_gradient.tsv`

> **⚠ The Production column and the composite geomean in the committed rows are
> denominated in *mass*, and the objective is now *volume* (R-PROD5,
> `Hyades_trees_and_card_value.md` §2.3.4).** Expansion, Growth and the per-seed
> pairing are unaffected. The rows are kept as measured rather than
> re-denominated — this file exists precisely so a later reader can re-analyze
> without re-running, and rewriting a recorded measurement to match a later
> definition destroys that. **R-TREE10** carries the re-run.

Written by `examples/tree_gradient` when `TG_RECORD` is set. **One row per
`(knob, arm, seed)`** — the CRN pairing is the whole point of the measurement and
a mean discards it.

| column | meaning |
|---|---|
| `knob` | the parameter perturbed; `(default)` for the unperturbed base runs |
| `arm` | `base`, `hi` (`x·1.10`) or `lo` (`x·0.90`) |
| `value` | the value actually used on that arm; `NaN` on `base` |
| `seed` | galaxy and sim seed — the CRN key |
| `horizon` | simulated years |
| `players` | seat count |
| `expansion` | `∫ Σ_i C_i dt` — colony-years |
| `growth` | `∫ Σ_i V_i dt` — work-years, in kt |
| `production` | `∫ Σ_i F_i dt` — fleet-years, in kt of **dry mass** |
| `warfare` | `∫ Σ_i [C_i − Σ_j w_ij C_j] dt` — an algebraic zero at 3 seats, see R-TREE8 |
| `politics` | `∫ Σ_i [C_i + κ Σ_j φ_ij C_j] dt` — identical to `expansion` while `φ ≡ 0` |

**An elasticity without its operating point is a rumor**, so the file is only
meaningful against the commit that produced it. Each run appends; the base rows
repeat per chunk and should be identical across chunks, which is a free
determinism check.

**Read it by available gain, not by elasticity.** A central difference averages
the two arms, so a knob that is pure downside scores like one that is pure
upside — `cargo_unit_size` is third of 32 by |elasticity| and twenty-second by
benefit. Compute `max(S(+δ), S(−δ)) − 1` per knob and keep the other arm beside
it; `AGENTS.md` §2's "Six traps in reading a gradient" has the rest.

The file carries **two seed sets**: 1 / 7 / 42 / 31337 is the standard CRN bed,
and 2 / 3 / 5 / 11 is the independent replication set for the shortlist the first
one produced. They are distinguished by the `seed` column alone, and mixing them
in one estimate defeats the point of having them — a candidate's error bar on the
bed it was selected on is not evidence about the candidate.

Definitions: `Hyades_trees_and_card_value.md` §2.3.

## `design_ratings.tsv` and `design_rating_matches.tsv`

The Technology tree's static Design rating (`Hyades_technology_tree.md` §4,
T-131), written from `examples/design_rating all <seeds>`. The first line of
`design_ratings.tsv` stamps the engine commit and the bed; **a rating is a
measurement of that engine and no other** (R-TECH15), so read the stamp before
the numbers.

`design_rating_matches.tsv` is the raw record — **one row per match**, every
pair on every seed with the seats swapped — from which every rating and interval
is recoverable:

| column | meaning |
|---|---|
| `bed` | the role bed (§4.4) |
| `seed` | galaxy and sim seed |
| `seat0`, `seat1` | the Design on each seat, named as the pool deduplicates it (R-TECH17) |
| `x0`, `x1` | each side's task score, in the bed's own unit (appendix §D.18) |
| `share0` | `x0 / (x0 + x1)`, ½ when both are zero |

`design_ratings.tsv`: `gamma` is the Bradley–Terry maximum-likelihood strength
with one virtual draw per pair, on the **ratio scale** with the bed's anchor
Design at 1 (R-TECH8): A takes `γ_A / (γ_A + γ_B)` of a match against B, and
every rating is positive. `p05`/`p95` are the 5th and 95th percentiles over
1,000 bootstrap resamples of the seeds. An interval that is a point means every
pair was separated completely, and the ratio is the virtual draw's. The comment
lines at the end carry each bed's intransitivity report (R-TECH7), which a
regeneration is not complete without reading.
