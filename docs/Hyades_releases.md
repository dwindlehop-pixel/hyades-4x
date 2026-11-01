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

Followed by two rulings that §2 implements: the number is assigned
mechanically from test status, and the launch test decides it by sampling
games for new cards, card-strength swings and new systems.

## 1. The tag

A release is an **annotated git tag on the head of a release branch**
(`release/vMMM`, §2), named

```
vMMM.mmm.bbb
```

| field | name | width | meaning |
|---|---|---|---|
| `MMM` | major | 3 digits, zero-padded | progress toward launch: a new card, or a large swing in a defined card's behavior |
| `mmm` | minor | 3 digits, zero-padded | a new game system (Design, Doctrine, galaxy generation, the Exchange, …) seen in sampled games |
| `bbb` | bugfix | 3 digits, zero-padded | everything else: the implementation brought closer to the spec, or a performance fix |

The `v` prefix is the common git convention and keeps release tags apart from
any other tag; the repository had no tags before this ruling, so no convention
competed with it.

## 2. The version is assigned by the launch test

The author's rulings: *"Version number should be assigned mechanically based on
test status, not produced by manual intervention or agent inference"*, and
*"All changes are bugfix unless launch test detects changes."* No person or
agent chooses a release number, and no step takes one as input.

**When.** `.github/workflows/release.yml` runs once a day (06:00 UTC) and on
demand, never per commit. It takes the newest commit of `main` whose CI run
passed, and releases the batch of commits since the last release tag. With no
new commits it releases nothing and says so on the run page ("Nothing to release: …"), so a green run is not mistaken for a release.

**The launch test**, `examples/launch_census`, samples games and prints a
census (its doc comment defines every line):

| census line | what it samples |
|---|---|
| `card <id> <seed> <strength>` | each **defined card** (no `NotYetImplemented` write, landed on every even seat), played at the round-0 barrier against a pass arm on the same galaxy; `strength` is the mean log-ratio of the seat's **capability-years** (below) |
| `kind <category> <name>` | each **kind** with at least one instance in a sampled game: a `LogEvent` variant raised (`system`), a `VehicleKind` standing (`entity`), a `CardEffect` variant that landed (`effect`) |

It also prints a `table` line (the rating table's stamp) and `unrated` lines
(hull-years of Designs the table does not rate); the rule below reads neither.

Eight seeds, six seats, 300 worlds, 500 years, sampled yearly; about 70 s in
release on one core, bit-identical across runs.

**Strength is capability, for every card** (the author's ruling: *"Strength is
a measure of fleet (hull, role, number) capability. … It is defined over the
current active fleet, and changes as roles change. All cards will alter the
measure of strength."*). It is Technology's stock (`Hyades_technology_tree.md`
§4.2, §4.7), read from the rating table, whatever tree the card is in:

| symbol | name | unit |
|---|---|---|
| `T_i` | capability-years, `∫ Q_i dt` from the round-0 barrier to the horizon, sampled yearly | kt·yr |
| `Q_i` | `Σ_h m_h · γ(d_h, bed(role_h))` over seat `i`'s hulls at the sample | kt of anchor-equivalent |
| `m_h` | the hull's dry mass, its mineral cost (R-O57) | kt |
| `γ(d, b)` | Design `d`'s strength in bed `b`, ratio scale, anchor 1 (`data/design_ratings.tsv`) | ratio |
| `bed(role)` | the bed rating the hull's active role; Reserve has none and counts zero (R-TECH10) | — |

A card's strength on a seed is the mean over the seats that played it of
`ln(T_i(card) / T_i(pass))`. A warship tasked as a freighter carries the
freighter bed's `γ` for its Design, which is low; more hulls raise `Q_i`.

*`OPEN`:* the Sentry role has no bed and reads the long-range bed, a
placeholder; the missile Designs (Butte, Mesa) are not in the table yet and
count zero, so the missile card's strength omits its sentries (its census
shows them on the `unrated` line). The census is stored in the release tag's
annotation, so the next batch compares against it.

**The rule** (`scripts/next_release.sh`), against the previous release
`P.p.b`:

| condition, checked in order | level | next release |
|---|---|---|
| a card id is defined that was not, or one that was is no longer | major | `(P+1).000.000` |
| a **large swing**: over the seeds both censuses carry, a card's mean paired change in strength exceeds `ln 1.25` *and* twice its standard error | major | `(P+1).000.000` |
| a kind appears that the previous census lacks | minor | `P.(p+1).000` |
| otherwise | bugfix | `P.p.(b+1)` |

Adding code that no sampled game reaches changes nothing: a system counts once
an instance of it shows up in the census. The first release has no
predecessor, so every card is new and it comes out **`v001.000.000`**.

The large-swing threshold, `ln 1.25` (a card 25% more or less capability) at 2
standard errors, is **ratified** (the author: *"Threshold is fine."*).

**Release branches.** Each major release is its own branch, `release/vMMM`,
started at the batch's last commit of `main`. Pull requests merge to `main`
as before; the release workflow then fast-forwards the current major's branch
to the batch for a minor or bugfix release, or starts the next major's branch
for a major one. A release branch only ever receives `main`, so the forward is
always fast, and an older major's branch stays where its last release left it.
The tag sits on the branch's head.

**Consequences, so nobody re-derives them:**

- **Major counts card changes, not a pass fraction.** The original ruling's
  "major 100 is launch-ready" has no mechanical reading under this rule yet.
- **A sampling bed is a bed** (`AGENTS.md` §6): the census plays cards through
  `apply_orders` at the protocol's barrier and varies nothing but the galaxy.
- **Changing the launch test changes the census.** A new census column or a new
  seed reads as new kinds or new strengths, so a landing that edits
  `launch_census` should expect its batch to be a minor or major release.
- **The crates' Cargo versions do not follow the release.** A tag is cut after
  its commits land, so the tag is the only record.

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

**The launch census reads the table and does not refuse it**, because the
census is what assigns the release a stamp would be checked against; it
prints the stamp on its `table` line instead. **Regeneration is `RATIFIED`** (the author: *"if a minor release is detected
then the Design rating must be regenerated. This is a separate optional
workflow that creates a commit."*). `.github/workflows/ratings.yml` runs after
every Release run. When the newest release is minor or major (a stamped table
survives a bugfix release and no other, so a major release regenerates too), it
runs `examples/design_rating all` at the release's own commit, writes both
tables through `scripts/rate_designs.sh` stamped `release vMMM.mmm.bbb`,
commits them on `ratings/vMMM.mmm.bbb`, and opens a pull request carrying the
intransitivity report, which someone reads before merging (R-TECH7). It can also
be run by hand for a named release. The commit reaches `main` only through that
pull request, so the release after it measures card strength against the new
table.
