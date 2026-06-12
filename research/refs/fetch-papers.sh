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
  # Fixed-order / NLO infrastructure references:
  "feynrules|https://ar5iv.org/html/0806.4194|FeynRules: Feynman rules from Lagrangian (arXiv:0806.4194)"
  # NOTE: ar5iv fails to render 0808.3674 (Gleisberg & Höche, COMIX) — the fetched
  # HTML will be an 8KB error page. Use https://arxiv.org/pdf/0808.3674 instead if
  # a local copy is needed.
  "comix|https://ar5iv.org/html/0808.3674|COMIX/Sherpa: Berends-Giele recursive ME generation (arXiv:0808.3674)"
  "catani_seymour|https://ar5iv.org/html/hep-ph/9605323|Catani-Seymour: general IR subtraction at NLO (hep-ph/9605323)"
  # Polarization / helicity amplitude papers:
  "polarized_me|https://ar5iv.org/html/1912.01725|Polarized ME automation in MadGraph5_aMC@NLO: helicity definitions (arXiv:1912.01725)"
  "polarized_propagator|https://ar5iv.org/html/2512.10015|Truncated propagator paradigm for polarized amplitudes (arXiv:2512.10015)"
  # Phase-space integration:
  "loop_induced_ps|https://ar5iv.org/html/1507.00020|Loop-induced processes and phase-space optimisation in MadGraph5 (arXiv:1507.00020)"
  # UFO / model format extensions:
  "madwidth|https://ar5iv.org/html/1402.1178|MadWidth: automatic decay widths, extends UFO with decay tables (arXiv:1402.1178)"
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
  # Convert HTML to markdown using MarkItDown (Python tool installed in pixi)
  if [[ "$ext" == "html" ]]; then
    pixi run -e markitdown markitdown $outfile --output "$PAPERS_DIR/${key}.md"
    echo "  Converted to markdown: ${key}.md"
    rm "$outfile"  # Remove original HTML file
  fi
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

