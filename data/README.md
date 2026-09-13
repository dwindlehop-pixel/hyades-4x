# `data/` — raw measurement records

**T-50: the elasticities are being spent and thrown away.** Every gradient this
project has run ended up as a sentence in a doc and was then invalidated by the
next operating point, leaving nothing behind. The *raw per-evaluation* record is
the artifact worth keeping, because elasticities, standard errors and rankings
are all recoverable from it and the reverse is not true — and because this
project has changed its objective twice already, so a later reader must be able
to re-analyse without re-running.

Everything here is plain TSV: zero dependencies, diffable, appended rather than
overwritten.

## `tree_gradient.tsv`

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

**An elasticity without its operating point is a rumour**, so the file is only
meaningful against the commit that produced it. Each run appends; the base rows
repeat per chunk and should be identical across chunks, which is a free
determinism check.

Definitions: `Hyades_trees_and_card_value.md` §2.3.
