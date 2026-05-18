#!/usr/bin/env bash
# Fetch paper content from ar5iv (LaTeX→HTML→Markdown) into research/refs/papers/
# Papers are gitignored — run this script to populate locally.
#
# Usage:
#   ./research/refs/fetch-papers.sh          # fetch all
#   ./research/refs/fetch-papers.sh aloha    # fetch by key

set -euo pipefail

PAPERS_DIR="$(cd "$(dirname "$0")" && pwd)/../refs/papers"
mkdir -p "$PAPERS_DIR"

# Format: KEY  ARXIV_OR_URL  FILENAME  DESCRIPTION
# For non-arXiv papers, use the best available HTML URL.
declare -A URLS DESCS
URLS=(
  [aloha]="https://ar5iv.org/html/1108.2041"
  [ufo]="https://ar5iv.org/html/1108.2040"
  [madgraph5]="https://ar5iv.org/html/1405.0301"
  [helas]="https://inspirehep.net/literature/336604"
  [vegas]="https://inspirehep.net/literature/119196"
  [mcreview]="https://ar5iv.org/html/1101.2599"
)
DESCS=(
  [aloha]="ALOHA: Automatic Libraries Of Helicity Amplitudes (arXiv:1108.2041)"
  [ufo]="UFO: Universal FeynRules Output (arXiv:1108.2040)"
  [madgraph5]="MadGraph5_aMC@NLO (arXiv:1405.0301)"
  [helas]="HELAS: HELicity Amplitude Subroutines (InspireHEP:336604)"
  [vegas]="VEGAS: Monte Carlo integration, Lepage 1978 (InspireHEP:119196)"
  [mcreview]="General-purpose MC event generators review (arXiv:1101.2599)"
)

fetch_one() {
  local key="$1"
  local url="${URLS[$key]}"
  local outfile="$PAPERS_DIR/${key}.md"
  echo "Fetching [$key]: $url"
  curl -sL \
    -H "Accept: text/html" \
    -A "Mozilla/5.0 (research fetch script)" \
    "$url" \
    -o "$PAPERS_DIR/${key}.html"
  echo "# ${DESCS[$key]}" > "$outfile"
  echo "# Source: $url" >> "$outfile"
  echo "" >> "$outfile"
  echo "  (raw HTML saved as ${key}.html — use a markdown converter or read HTML directly)" >> "$outfile"
  echo "Saved: $outfile (+ ${key}.html)"
}

if [[ $# -gt 0 ]]; then
  for key in "$@"; do
    if [[ -v URLS[$key] ]]; then
      fetch_one "$key"
    else
      echo "Unknown key: $key. Available: ${!URLS[*]}"
      exit 1
    fi
  done
else
  for key in "${!URLS[@]}"; do
    fetch_one "$key"
  done
fi

echo ""
echo "Done. Files in: $PAPERS_DIR"
echo "Note: HTML files are not committed (gitignored). Re-run this script to refresh."
