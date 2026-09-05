#!/bin/bash
# Build MadGraph validation output directories for all processes
# This runs mg5_aMC batch scripts to generate diagrams and process information
#
# Dynamically maps script names to output directories:
#   script.mg5 -> script (removes .mg5 extension)
#
# The generator is the pinned submodule by way of mg5_pinned.sh, not whatever
# mg5_aMC sits on PATH — see that script for why the version matters to a run
# whose events and cross section become a reference. It runs from a temporary
# directory, so each script's relative `output <name>` is rewritten to the
# absolute work-area path before it is handed over.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SCRIPTS_DIR="$SCRIPT_DIR/scripts"
OUTPUT_BASE="$SCRIPT_DIR/output"
MODELS_DIR="$OUTPUT_BASE/models"
SUMMARY_FILE="$OUTPUT_BASE/build_summary.txt"

# Ensure output directory exists
mkdir -p "$OUTPUT_BASE"

# Stage the vendored UFO models into the work area.
#
# Two reasons the generators read a copy rather than validation/ufo/ itself:
# MadGraph caches a restricted model as a pickle *inside* the model directory it
# imported, and the vendored directories are committed byte for byte against a
# SHA256SUMS manifest. The committed per-class restrict cards are added to the
# staged copy, which is what lets a script name one with `import model DIR-NAME`
# while the card itself stays in the repository beside the manifest row that
# selects it.
#
# Restaging on every run is deliberate: the copy's file times move, so MadGraph
# rebuilds its pickle rather than serving a model built from an older card.
stage_models() {
  local src name cards
  [ -d "$REPO_ROOT/validation/ufo" ] || return 0
  mkdir -p "$MODELS_DIR"
  for src in "$REPO_ROOT/validation/ufo"/*/; do
    [ -f "${src}particles.py" ] || continue
    name="$(basename "$src")"
    rm -rf "${MODELS_DIR:?}/$name"
    cp -R "$src" "$MODELS_DIR/$name"
    # Per-class restrict cards this repository authors for the model. They are
    # not part of the upstream directory, so they are committed separately and
    # placed here, where `import model <dir>-<name>` resolves restrict_<name>.dat.
    cards="$SCRIPT_DIR/cards/$(model_card_dir "$name")"
    if [ -d "$cards" ]; then
      cp "$cards"/restrict_*.dat "$MODELS_DIR/$name/"
    fi
    echo "  staged model: $name"
  done
}

# strip_ansi_escapes DIR — remove terminal colour codes from a run's text files.
#
# MadGraph reads its own version out of its git checkout. Where the checkout has
# no `.git` — a copied submodule, which is what a fresh worktree gets — it falls
# back to a "development version" warning it writes in red, and those escape
# bytes go into the banner it embeds in every card and event file it writes. An
# LHE banner is XML, so the escapes make the event file unparseable; nothing in
# them is physics, and a checkout whose submodule kept its `.git` never produces
# them, so removing them makes the two agree rather than diverge.
strip_ansi_escapes() {
  local dir="$1" f
  while IFS= read -r f; do
    LC_ALL=C sed -i $'s/\x1b\[[0-9;]*[a-zA-Z]//g' "$f"
  done < <(grep -rlI $'\x1b\[' "$dir/Cards" "$dir/Events" 2>/dev/null || true)
  while IFS= read -r f; do
    if zgrep -qa $'\x1b\[' "$f" 2>/dev/null; then
      gzip -dc "$f" | LC_ALL=C sed $'s/\x1b\[[0-9;]*[a-zA-Z]//g' | gzip -n > "$f.stripped"
      mv "$f.stripped" "$f"
    fi
  done < <(find "$dir/Events" -name '*.lhe.gz' 2>/dev/null || true)
}

# The cards/ subdirectory holding a model's authored restrict cards.
model_card_dir() {
  case "$1" in
    SMEFTsim_*) echo "smeft" ;;
    *) echo "$1" ;;
  esac
}

echo "Building MadGraph validation output directories..."
echo "Output base: $OUTPUT_BASE"
stage_models
echo ""

SKIPPED=0
GENERATED=0
TOTAL_ELAPSED=0

BUILD_START=$(date +%s)

# Dynamically process all .mg5 scripts in scripts/ directory
for script_path in "$SCRIPTS_DIR"/*.mg5; do
  [ -f "$script_path" ] || continue

  script=$(basename "$script_path")

  # Derive output directory name from script name by removing .mg5 extension
  # ee_to_mumu.mg5 -> ee_to_mumu
  # pp_to_ll_qcd0.mg5 -> pp_to_ll_qcd0
  output_dir="${script%.mg5}"
  output_path="$OUTPUT_BASE/$output_dir"

  if [ -d "$output_path" ]; then
    echo "⊘ Skipping: $script (output already exists at $output_dir/)"
    SKIPPED=$((SKIPPED + 1))
    continue
  fi

  echo "Processing: $script -> $output_dir/"

  t0=$(date +%s)

  # Both paths a script names are repository-relative and both are rewritten to
  # the work area, because MadGraph runs from a temporary directory: `output`
  # to this process's directory, and `import model validation/ufo/NAME[-CARD]`
  # to the staged copy of that model.
  driver="$(mktemp -t vg_build_XXXXXX).mg5"
  sed -E -e "s|^([[:space:]]*output[[:space:]]+)[^[:space:]]+|\1$output_path|" \
    -e "s|^([[:space:]]*import[[:space:]]+model[[:space:]]+)validation/ufo/|\1$MODELS_DIR/|" \
    "$script_path" > "$driver"
  grep -qE "^[[:space:]]*output[[:space:]]+$output_path( |$)" "$driver" || {
    echo "!!! $script has no 'output' line to redirect" >&2
    exit 1
  }
  if grep -qE "^[[:space:]]*import[[:space:]]+model[[:space:]]+validation/" "$driver"; then
    echo "!!! $script imports a model path that was not rewritten into the work area" >&2
    exit 1
  fi

  # The conda activation exports its own LDFLAGS, which suppresses MadGraph's
  # make_opts `STDLIB=-lc++` (its `ifeq($(origin LDFLAGS),undefined)` guard sees
  # LDFLAGS as already set), so a `pdlabel = lhapdf` run leaves the LHAPDF C++
  # runtime symbols unresolved when madevent links libpdf.a. Appending -lc++ is
  # what gen_hadronic_sigma.sh already does for the same reason; it is inert for a
  # script that links no C++. `-lc++` names libc++, which is the system C++
  # runtime on macOS and generally absent on Linux, where the same symbols come
  # from libstdc++ and the flag would fail the link instead of fixing it.
  log="$(mktemp -t vg_build_log_XXXXXX)"
  case "$(uname -s)" in
    Darwin) build_ldflags="${LDFLAGS:-} -lc++" ;;
    *) build_ldflags="${LDFLAGS:-}" ;;
  esac
  if ! LDFLAGS="$build_ldflags" bash "$SCRIPT_DIR/mg5_pinned.sh" "$driver" > "$log" 2>&1; then
    echo "!!! $script failed; last 40 lines of its log:" >&2
    tail -40 "$log" >&2
    if [ -d "$output_path" ]; then mv "$log" "$output_path/build.log"; fi
    exit 1
  fi
  rm -f "$driver"
  mv "$log" "$output_path/build.log"

  strip_ansi_escapes "$output_path"

  t1=$(date +%s)
  elapsed=$((t1 - t0))
  echo "Total build time: (${elapsed}s)" >> "$output_path/build.log"
  TOTAL_ELAPSED=$((TOTAL_ELAPSED + elapsed))

  echo "✓ Completed: $script (${elapsed}s)"
  GENERATED=$((GENERATED + 1))
  echo ""
done

BUILD_END=$(date +%s)
WALL_TIME=$((BUILD_END - BUILD_START))

# Count total files across all output directories (excluding JSON/CSV summaries)
TOTAL_FILES=$(find "$OUTPUT_BASE" -maxdepth 4 -type f | wc -l | tr -d ' ')

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ MadGraph build complete"
echo "  Generated: $GENERATED new directories"
echo "  Skipped:   $SKIPPED existing directories"
echo "  Wall time: ${WALL_TIME}s  (MG5 time for new processes: ${TOTAL_ELAPSED}s)"
echo "  Total files in output/: $TOTAL_FILES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Output directories available in: $OUTPUT_BASE"

# Write summary file
{
  echo "MadGraph build summary"
  echo "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
  echo "Generated: $GENERATED new directories"
  echo "Skipped:   $SKIPPED existing directories"
  echo "Wall time: ${WALL_TIME}s"
  echo "MG5 time (new processes): ${TOTAL_ELAPSED}s"
  echo "Total files in output/: $TOTAL_FILES"
} > "$SUMMARY_FILE"

echo "Summary written to: $SUMMARY_FILE"
