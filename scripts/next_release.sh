#!/usr/bin/env bash
# Assigns the next release tag from test status alone (docs/Hyades_releases.md,
# R-TECH15). No input is a judgment: every field is a function of the files below.
#
#   next_release.sh PREV_TAG PREV_NOTES INVENTORY LAUNCH_RESULTS OUT_NOTES
#
# PREV_TAG        the last release tag, or "" when there is none
# PREV_NOTES      that tag's annotation (ignored when PREV_TAG is "")
# INVENTORY       every test name in the workspace, one per line
# LAUNCH_RESULTS  one line per launch-suite test: "<name> ok" or "<name> FAILED"
# OUT_NOTES       where to write the new tag's annotation
#
# Prints the new tag on stdout. Exits 1, printing the reason on stderr, when a
# launch test that passed at PREV_TAG fails now.
set -euo pipefail
prev_tag=$1 prev_notes=$2 inventory=$3 launch=$4 out=$5

section() { # section NAME FILE: the lines under "[NAME]" up to the next header
  awk -v s="[$1]" '/^\[/{on=($0==s); next} on' "$2"
}

inv_now=$(sort -u "$inventory")
pass_now=$(awk '$2=="ok"{print $1}' "$launch" | sort -u)
total=$(awk 'NF' "$launch" | wc -l)
passed=$(printf '%s' "$pass_now" | awk 'NF' | wc -l)
measured=0
if [ "$total" -gt 0 ]; then measured=$((100 * passed / total)); fi

if [ -z "$prev_tag" ]; then
  pM=0 pm=0 pb=0 inv_prev="" pass_prev=""
else
  IFS=. read -r pM pm pb <<<"${prev_tag#v}"
  pM=$((10#$pM)) pm=$((10#$pm)) pb=$((10#$pb))
  inv_prev=$(section inventory "$prev_notes" | sort -u)
  pass_prev=$(section launch-passing "$prev_notes" | sort -u)
fi

regressed=$(comm -23 <(printf '%s\n' "$pass_prev" | awk 'NF') <(printf '%s\n' "$pass_now" | awk 'NF'))
if [ -n "$regressed" ]; then
  echo "launch tests that passed at $prev_tag fail now; no release:" >&2
  echo "$regressed" >&2
  exit 1
fi

major=$(( measured > pM ? measured : pM ))
if [ "$major" -gt "$pM" ]; then
  minor=0 bugfix=0
elif [ -z "$prev_tag" ] || [ "$inv_now" != "$inv_prev" ]; then
  minor=$((pm + 1)) bugfix=0
else
  minor=$pm bugfix=$((pb + 1))
fi
tag=$(printf 'v%03d.%03d.%03d' "$major" "$minor" "$bugfix")

{
  echo "release $tag"
  echo
  echo "launch suite: $passed of $total pass ($measured%)"
  echo "[launch-passing]"
  [ -n "$pass_now" ] && echo "$pass_now"
  echo "[inventory]"
  echo "$inv_now"
} >"$out"
echo "$tag"
