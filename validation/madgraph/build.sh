#!/bin/bash
# Build MadGraph validation output directories for all processes
# This runs mg5_aMC batch scripts to generate diagrams and process information
#
# Dynamically maps script names to output directories:
#   script.mg5 -> script (removes .mg5 extension)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="$SCRIPT_DIR/scripts"
OUTPUT_BASE="$SCRIPT_DIR/output"
SUMMARY_FILE="$OUTPUT_BASE/build_summary.txt"

# Ensure output directory exists
mkdir -p "$OUTPUT_BASE"

echo "Building MadGraph validation output directories..."
echo "Output base: $OUTPUT_BASE"
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

  # Run mg5_aMC in the output directory to keep things organized
  cd "$OUTPUT_BASE"
  # The conda activation exports its own LDFLAGS, which suppresses MadGraph's
  # make_opts `STDLIB=-lc++` (its `ifeq($(origin LDFLAGS),undefined)` guard sees
  # LDFLAGS as already set), so a `pdlabel = lhapdf` run leaves the LHAPDF C++
  # runtime symbols unresolved when madevent links libpdf.a. Appending -lc++ is
  # what gen_hadronic_sigma.sh already does for the same reason; it is inert for a
  # script that links no C++.
  LDFLAGS="${LDFLAGS:-} -lc++" mg5_aMC "$script_path" > "tmp.log" 2>&1
  mv tmp.log "$output_dir/build.log"
  cd - > /dev/null

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
