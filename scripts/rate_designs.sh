#!/usr/bin/env bash
# Writes data/design_ratings.tsv and data/design_rating_matches.tsv from one
# run of `examples/design_rating all <seeds>`, stamped with a release
# (R-TECH15, docs/Hyades_releases.md §3). Run by .github/workflows/ratings.yml;
# runnable by hand.
#
# usage: scripts/rate_designs.sh RELEASE RAW
#   RELEASE  the release the table is measured at, vMMM.mmm.bbb
#   RAW      design_rating's stdout (STAMP, MATCH, RATING and INTRANSITIVITY lines)
set -euo pipefail
release=$1
raw=$2
[[ $release =~ ^v[0-9]{3}\.[0-9]{3}\.[0-9]{3}$ ]] || { echo "not a release: $release" >&2; exit 1; }
stamp=$(awk -F'\t' '$1 == "STAMP" { print $2; exit }' "$raw")
# A release from before design_rating printed its STAMP line: name the seeds
# from the MATCH lines and leave the spend out rather than guess it.
if [ -z "$stamp" ]; then
  seeds=$(awk -F'\t' '$1 == "MATCH" && !seen[$3]++ { printf "%s%s", (n++ ? "," : ""), $3 }' "$raw")
  stamp="seeds $seeds, both seatings"
fi
grep -q '^RATING' "$raw" || { echo "no RATING lines in $raw" >&2; exit 1; }
{
  echo "# design_rating, release $release, $stamp"
  echo "# columns: bed, design, gamma on the ratio scale (anchor 1; R-TECH8), 5th and 95th percentile over 1000 seed bootstraps"
  printf 'bed\tdesign\tgamma\tp05\tp95\n'
  awk -F'\t' -v OFS='\t' '$1 == "RATING" { print $2, $3, $4, $5, $6 }' "$raw"
  echo "# intransitivity (R-TECH7), per bed:"
  grep '^INTRANSITIVITY' "$raw" | sed 's/^/# /'
} > data/design_ratings.tsv
{
  printf 'bed\tseed\tseat0\tseat1\tx0\tx1\tshare0\n'
  awk -F'\t' -v OFS='\t' '$1 == "MATCH" { print $2, $3, $4, $5, $6, $7, $8 }' "$raw"
} > data/design_rating_matches.tsv
