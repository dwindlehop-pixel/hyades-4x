# Hyades — releases and the staleness of measurements

**Status: `RATIFIED`** — the author's ruling on R-TECH15
([#312](https://github.com/dwindlehop-pixel/hyades-4x/issues/312)):

> I want to add releases as git tags. We cannot afford to redo the calculation
> of the capability for every commit. We'll record staleness as release number.
> Use the common major.minor.bugfix triplet. Use three digits with leading
> zeroes. Major releases measure progress to launch: major version 100 indicates
> the game is launch ready. Minor releases indicate changes in systems but no
> significant change in player strategy. Bugfix just more closely align spec
> with implementation, or resolve performance issues.

## 1. The tag

A release is an **annotated git tag on a commit of `main`**, named

```
vMMM.mmm.bbb
```

| field | name | width | meaning |
|---|---|---|---|
| `MMM` | major | 3 digits, zero-padded | progress toward launch; **`100` is launch-ready** |
| `mmm` | minor | 3 digits, zero-padded | a change to a system that does not significantly change player strategy |
| `bbb` | bugfix | 3 digits, zero-padded | the implementation brought closer to the spec, or a performance fix |

The `v` prefix is the common git convention and keeps release tags apart from
any other tag; the repository had no tags before this ruling, so no convention
competed with it.

## 2. The version is assigned by test status

The author's follow-up ruling: *"Version number should be assigned mechanically
based on test status, not produced by manual intervention or agent inference."*
No person or agent chooses a release number, and no step takes one as input.

**When.** `.github/workflows/release.yml` runs after every CI run on a push to
`main`. If CI passed and the commit has no release tag, it cuts one. A pull
request is never a release. CI's `cancel-in-progress` means a commit superseded
mid-run is not released; the next one is.

**What it reads.** Two test results, both stored in the tag's annotation so the
next release can compare against them:

| input | how it is read |
|---|---|
| the **launch suite**, `tests/launch.rs` | run with `--include-ignored`; each test is one launch criterion, ignored by the CI gate because it fails until met. `L` = the pass percentage, `floor(100 × passed / total)`, `0` for an empty suite |
| the **test inventory** | the name of every test in the workspace, `cargo test --workspace --all-targets -- --list` |

**The rule** (`scripts/next_release.sh`), against the previous release
`P.p.b`:

| condition, checked in order | next release |
|---|---|
| a launch test that passed at the previous release fails now | **none** — the workflow fails |
| `L > P` | `L.000.000` |
| the inventory differs from the previous release's (a test added, removed or renamed) | `P.(p+1).000` |
| otherwise | `P.p.(b+1)` |

The first release has no predecessor and comes out `v000.001.000`.

**Consequences, so nobody re-derives them:**

- **Major never falls.** Adding a criterion lowers the pass fraction; major holds
  until the fraction passes it again. A criterion that regresses blocks every
  release until it passes again.
- **Major is only as meaningful as the suite.** The suite starts empty, so major
  is `000` until the author writes criteria into it.
- **The minor/bugfix split is a proxy.** Under this rule a new test marks a
  system change; a merge that only fixes or speeds up existing behavior under
  existing tests is a bugfix. *`OPEN`:* the author is choosing between this rule,
  a change to a pinned golden, and a change under `src/`.
- **The crates' Cargo versions do not follow the release.** A tag is cut after
  the commit lands, so no commit can carry its own release number; the tag is the
  only record.

## 3. Staleness is a release number

A measurement too expensive to repeat on every commit — the first is the
Technology tree's Design rating table, `data/design_ratings.tsv`
(`Hyades_technology_tree.md` §4.10) — is **stamped with the release it was
measured at**, not with a commit. Its stamp line carries `release vMMM.mmm.bbb`.

A reader compares the stamp against the current release:

| stamp against current | reader |
|---|---|
| same release | accepts |
| bugfix differs | accepts, and prints both releases |
| minor or major differs | **refuses** |
| no release stamp (a commit stamp from before the first release) | **refuses** |

A bugfix release by definition changes no system, so a measurement survives
one; a minor release changes a system, and a measurement of the old one is
stale. Regenerating the table is what clears a refusal.

The engine never reads a stamped table: capability is a measurement, and no
autopilot decision may consult it (`Hyades_technology_tree.md` §4.10).
