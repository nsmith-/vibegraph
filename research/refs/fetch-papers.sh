#!/usr/bin/env bash
# Fetch paper content from ar5iv (LaTeX→HTML) into research/refs/papers/
# Papers are gitignored — run this script to populate locally.
#
# Usage:
#   ./research/refs/fetch-papers.sh          # fetch all
#   ./research/refs/fetch-papers.sh aloha    # fetch by key

set -euo pipefail

PAPERS_DIR="$(cd "$(dirname "$0")" && pwd)/../refs/papers"
mkdir -p "$PAPERS_DIR"

# Each entry: "key|url|description"
# HELAS (KEK-91-11, 1992) is pre-arXiv; InspireHEP is a JS SPA (not fetchable).
# VEGAS (Lepage 1978) is pre-arXiv; using Lepage's later pedagogical write-up.
PAPERS=(
  "aloha|https://ar5iv.org/html/1108.2041|ALOHA: Automatic Libraries Of Helicity Amplitudes (arXiv:1108.2041)"
  "ufo|https://ar5iv.org/html/1108.2040|UFO: Universal FeynRules Output (arXiv:1108.2040)"
  "madgraph5|https://ar5iv.org/html/1405.0301|MadGraph5_aMC@NLO (arXiv:1405.0301)"
  "madgraph_orig|https://ar5iv.org/html/hep-ph/9401258|Original MadGraph: diagram generation + HELAS calls (hep-ph/9401258)"
  "helas|https://lib-extopc.kek.jp/preprints/PDF/1991/9124/9124011.pdf|HELAS: HELicity Amplitude Subroutines, KEK-91-11 (PDF)"
  # VEGAS (Lepage 1978) is pre-arXiv. Use Lepage's 2020 VEGAS+ paper which
  # describes the original algorithm in full plus enhancements:
  "vegas|https://ar5iv.org/html/2009.05112|VEGAS+: Adaptive Multidimensional Integration, Lepage 2020 (arXiv:2009.05112)"
  "mcreview|https://ar5iv.org/html/1101.2599|General-purpose MC event generators review (arXiv:1101.2599)"
)

get_field() { echo "$1" | cut -d'|' -f"$2"; }

fetch_one() {
  local key="$1" url="$2" desc="$3"
  # Choose extension based on URL
  local ext="html"
  [[ "$url" == *.pdf ]] && ext="pdf"
  local outfile="$PAPERS_DIR/${key}.${ext}"
  echo "Fetching [$key]: $url"
  curl -sL \
    -H "Accept: text/html" \
    -A "Mozilla/5.0 (research fetch script)" \
    "$url" \
    -o "$outfile"
  local size
  size=$(wc -c < "$outfile")
  echo "  Saved: $outfile (${size} bytes)"
  # Write a small index stub alongside
  printf '# %s\n# Source: %s\n\n(%s saved as %s.%s)\n' \
    "$desc" "$url" "$ext" "$key" "$ext" > "$PAPERS_DIR/${key}.md"
}

if [[ $# -gt 0 ]]; then
  for requested in "$@"; do
    found=0
    for entry in "${PAPERS[@]}"; do
      key=$(get_field "$entry" 1)
      if [[ "$key" == "$requested" ]]; then
        fetch_one "$key" "$(get_field "$entry" 2)" "$(get_field "$entry" 3)"
        found=1
        break
      fi
    done
    if [[ $found -eq 0 ]]; then
      echo "Unknown key: $requested"
      echo "Available keys:"
      for entry in "${PAPERS[@]}"; do printf '  %s\n' "$(get_field "$entry" 1)"; done
      exit 1
    fi
  done
else
  for entry in "${PAPERS[@]}"; do
    fetch_one "$(get_field "$entry" 1)" "$(get_field "$entry" 2)" "$(get_field "$entry" 3)"
  done
fi

echo ""
echo "Done. Files in: $PAPERS_DIR"
echo "Note: HTML files are not committed (gitignored). Re-run this script to refresh."

