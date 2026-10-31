#!/usr/bin/env bash
# Assigns the next release from the launch test alone (docs/Hyades_releases.md
# §2). No input is a judgment: the level is a function of two census files.
#
#   next_release.sh PREV_TAG PREV_NOTES CENSUS OUT_NOTES
#
# PREV_TAG    the last release tag, or "" when there is none
# PREV_NOTES  that tag's annotation, which carries its census (ignored when
#             PREV_TAG is "")
# CENSUS      this batch's `examples/launch_census` output
# OUT_NOTES   where to write the new tag's annotation
#
# Prints "<tag> <release branch> <level>" on stdout, level one of major, minor,
# bugfix. The reasons go to stderr.
#
# major   a card id in CENSUS that PREV lacks, a card PREV had that CENSUS lacks,
#         or a large swing: over the seeds both carry, the mean paired change in
#         a card's strength exceeds both SWING_LN and twice its standard error
# minor   otherwise, a kind line in CENSUS that PREV lacks
# bugfix  otherwise
set -euo pipefail
prev_tag=$1 prev_notes=$2 census=$3 out=$4
SWING_LN=0.223144 # ln(1.25): a card 25% stronger or weaker on its own tree's stock

prev=$(mktemp)
if [ -n "$prev_tag" ]; then grep -E '^(card|kind) ' "$prev_notes" >"$prev" || true; fi

level=$(awk -v swing="$SWING_LN" '
  { file = (FILENAME == ARGV[1]) ? 1 : 2 }
  $1 == "card" { if (file == 1) { p[$2] = 1; pv[$2, $3] = $4 } else { n[$2] = 1; nv[$2, $3] = $4 } }
  $1 == "kind" { if (file == 1) pk[$2 " " $3] = 1; else nk[$2 " " $3] = 1 }
  END {
    major = 0; minor = 0
    for (c in n) if (!(c in p)) { print "new card " c > "/dev/stderr"; major = 1 }
    for (c in p) if (!(c in n)) { print "card no longer defined " c > "/dev/stderr"; major = 1 }
    for (k in nv) {
      split(k, a, SUBSEP)
      if ((a[1], a[2]) in pv) { d = nv[k] - pv[k]; cnt[a[1]]++; sum[a[1]] += d; sq[a[1]] += d * d }
    }
    for (c in cnt) {
      m = sum[c] / cnt[c]
      var = cnt[c] > 1 ? (sq[c] - cnt[c] * m * m) / (cnt[c] - 1) : 0
      se = sqrt(var > 0 ? var : 0) / sqrt(cnt[c])
      am = m < 0 ? -m : m
      if (am > swing && am > 2 * se) {
        printf "card %s strength swing %+.4f (se %.4f, %d seeds)\n", c, m, se, cnt[c] > "/dev/stderr"; major = 1
      }
    }
    for (k in nk) if (!(k in pk)) { print "new kind " k > "/dev/stderr"; minor = 1 }
    print major ? "major" : (minor ? "minor" : "bugfix")
  }' "$prev" "$census")
rm -f "$prev"

if [ -z "$prev_tag" ]; then
  M=0 m=0 b=0
else
  IFS=. read -r M m b <<<"${prev_tag#v}"
  M=$((10#$M)) m=$((10#$m)) b=$((10#$b))
fi
case $level in
  major) M=$((M + 1)) m=0 b=0 ;;
  minor) m=$((m + 1)) b=0 ;;
  bugfix) b=$((b + 1)) ;;
esac
if [ "$M" -gt 999 ] || [ "$m" -gt 999 ] || [ "$b" -gt 999 ]; then
  echo "a field passed 999; the three-digit format cannot hold it" >&2
  exit 1
fi
tag=$(printf 'v%03d.%03d.%03d' "$M" "$m" "$b")
branch=$(printf 'release/v%03d' "$M")

{
  echo "release $tag ($level)"
  echo
  sort "$census"
} >"$out"
echo "$tag $branch $level"
