# §C. Cross-cutting — measurement artifacts, collected

*Part of the experiments record. Nothing here is normative. Table of contents: [`README.md`](README.md); how the record is organized: [`AGENTS.md`](AGENTS.md).*

*Seven shapes, all of them live in this project at some point. `AGENTS.md` §2
carries the working rules; this is the case list.*

| # | What was measured | What it actually was | Entry |
|---|---|---|---|
| 1 | `medium_fleet_size = 8` optimal, 12 a "cliff" | the capacity normalizer going to zero | — |
| 2 | `coverage_trace`: this knob "DID move it" | two of three sample points degenerate | — |
| 3 | `coverage_time`: cheaper colonizers at 6.0 | a General hull holding ~700× a Medium's | — |
| 4 | coverage "wants" a cheaper Medium hull | `cap_Medium` pinned by a live normalizer | — |
| 5 | `survey_reserve` is "significant" at −23.8 ± 10.1 | a ±10% probe on a plateau, clearing 2 SE by luck | A.3 |
| 6 | time-to-10% and coverage disagree on a knob's sign | two gradients taken at different operating points | A.2 |
| 7 | a dwell metric *rose* under the policy that founds colonies 399 yr earlier | the mix moved: hull-first share 55.0% → 97.4%, both components fell | — |

Shapes 1–4 share a cause: **the quantity that broke was in a denominator the
shipped autopilot never exercised**, so none of them was visible in the
objective. Shape 7 is the only one where **nothing was broken** — the
measurement was correct, the population it averaged over was not the same
population, and the sign it reported was the opposite of the mechanism.
