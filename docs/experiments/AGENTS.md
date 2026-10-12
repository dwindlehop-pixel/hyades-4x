# AGENTS.md — how the experiments record is organized

This directory replaced the single file `docs/Hyades_experiments_appendix.md`
(T-168). Read this before adding an entry, moving one, or looking for one.

## Why the record exists

`AGENTS.md` §6 splits a spec into **decisions the engine must honor** and
**decisions still open**, and sends everything else here. The reason is that the
two kinds of statement decay differently:

- **A decision is true until something contradicts it**, and is meant to be read
  on every visit.
- **A measurement is a record of a run on a bed that no longer exists**, and is
  meant to be read once — when somebody re-opens the question.

`biosphere_regen_rate = +141.2 ± 18.1 colonies per ln` is the largest lever this
project ever measured and it is **bit-identically inert today**. The number was
never wrong; the engine it described stopped existing. A spec that inlines it
invites a reader to act on it; an appendix that records it lets a reader
*check* whether it still applies.

**Three rules for entries here:**

1. **Every entry names its bed.** Seeds, seat count, horizon, harness. A result
   without a bed cannot be compared against anything and is not worth keeping.
2. **Refuted results stay, marked refuted, with the refutation.** Deleting a
   retracted claim takes its correction with it, and the next reader re-derives
   the same wrong idea. This project has done that at least twice.
3. **Entries are append-only in spirit.** Correct one by adding the correction
   beneath it, not by editing the original into agreement with the present.

**How to cite:** a spec links a decision to an entry by identifier
(`see appendix §A.4`); the entry links back to the decision it supports. If an
entry supports nothing, it is a curiosity and should say so.

## Layout

```
docs/experiments/
  README.md                  table of contents: every entry identifier → its file
  AGENTS.md                  this file
  A-expansion-growth.md      §A — Expansion and Growth
  B-politics-trade.md        §B — Politics, Trade and Intelligence
  C-measurement-artifacts.md §C — the seven artifact shapes, collected
  D-mechanisms/              §D — mechanisms from T-122 onward, one file per subject
    cards.md  combat.md  exchange-and-freight.md  galaxy-and-color.md
    interface.md  performance-and-determinism.md  sessions.md
    spread-between-empires.md  supers-and-forging.md
```

**Two levels: a section, then a subject.** A section (§A–§D) is a letter and
never changes. Within §D, a file holds the entries on one subject in identifier
order, so its entries need not be consecutive: `combat.md` holds §D.10–§D.19,
§D.24 and §D.29.

## Identifiers are permanent; files are not

- **An identifier names an entry, not a place.** Specs, `AGENTS.md` and source
  comments cite `appendix §D.24` without a path, so an entry can move to another
  file without editing a single citation. `README.md` is the only place that maps
  an identifier to a file, and it must be updated in the same commit as any move.
- **Identifiers are never reused or renumbered.** A new §D entry takes the next
  free number (the highest in `README.md` plus one), whatever file it lands in.
- **Subsections keep their parent's number** (§D.58.1 lives with §D.58).

## Adding an entry

1. Take the next free identifier from `README.md`.
2. Put the entry in the file for its subject. If no file fits, create one in the
   section's directory with a lowercase, hyphenated subject name, and give it the
   same two-line header the other files carry.
3. Add a line to `README.md` under that file: `- **§D.nn** — title`.
4. Link the entry from the spec decision it supports, and link the decision from
   the entry.

## Size

**Each Markdown file stays at or under about 50,000 tokens** (the author's
ruling, T-168) — roughly 175 KB of this record's prose, at ~3.5 bytes per token
(an estimate for English with numbers and code spans; the largest file at the
split, `D-mechanisms/combat.md`, was 58 KB). When a file passes that, split it by
subject into two files in the same directory, move the entries whole, and update
`README.md`. Never split one entry across files.

```bash
wc -c docs/experiments/*.md docs/experiments/*/*.md   # bytes; divide by 3.5 for tokens
```

## Data

**Data files may be as large as needed** (T-168); the size limit is for prose.

- **`data/` at the repository root** holds the raw records the harnesses write
  (`TG_RECORD=data/tree_gradient.tsv`, `data/design_ratings.tsv`,
  `data/design_rating_matches.tsv`), documented column by column in
  `data/README.md`. Harness code writes there, so those files stay where they are.
- **A data file that belongs to one entry and no harness** goes beside the
  entry, in `docs/experiments/data/<identifier>-<name>.<ext>` (for example
  `data/D.60-relay-limits.tsv`), and the entry links it. Plain TSV or CSV by
  preference: diffable, and readable without a dependency.
- **To find the data behind an entry**, read the entry: it names its harness,
  bed and output file. `grep -rn "<harness name>" data/README.md` gives the
  columns.

## Finding things

- **By identifier:** `README.md`, or `grep -rln "^## D.24 " docs/experiments`.
- **By subject or code:** `grep -rn "T-139\|R-MX8" docs/experiments`.
- **By date:** entries carry no date. The commit that added an entry has it:
  `git log --format='%ad %h %s' --date=short -S "## D.24 " -- docs/` (the
  `-- docs/` path also finds entries written while the record was one file).
