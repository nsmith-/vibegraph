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

# Ensure output directory exists
mkdir -p "$OUTPUT_BASE"

echo "Building MadGraph validation output directories..."
echo "Output base: $OUTPUT_BASE"
echo ""

SKIPPED=0
GENERATED=0

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

  # Run mg5_aMC in the output directory to keep things organized
  cd "$OUTPUT_BASE"
  mg5_aMC "$script_path"
  cd - > /dev/null

  echo "✓ Completed: $script"
  GENERATED=$((GENERATED + 1))
  echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ MadGraph build complete"
echo "  Generated: $GENERATED new directories"
echo "  Skipped:   $SKIPPED existing directories"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Output directories available in: $OUTPUT_BASE"
