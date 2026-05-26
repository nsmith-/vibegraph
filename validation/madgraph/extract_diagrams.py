#!/usr/bin/env python3
"""
Extract diagram information from MadGraph output directories.

Dynamically processes all output directories under output/ and creates
a JSON file for each one with diagram counts.

Expected structure:
  output/
    ee_to_mumu_lo/
      SubProcesses/
        P*/
          *.ps (diagram files)
    pp_to_ll_lo/
      ...

Output: Creates output/DIR.json for each directory with:
  {
    "process": "inferred from directory name",
    "total_diagrams": count,
    "diagrams_by_subprocess": {...}
  }
"""

import json
import os
import re
import sys
from pathlib import Path
from typing import Dict, Any


def count_diagrams_in_subprocess(subprocess_dir: str) -> int:
    """
    Count diagrams in a MadGraph subprocess directory.

    Diagrams are stored as .ps (PostScript) files in the subprocess directory.
    Each .ps file represents one diagram.
    """
    if not os.path.exists(subprocess_dir):
        return 0

    # Count *.ps files (each represents a diagram)
    ps_files = list(Path(subprocess_dir).glob("*.ps"))
    return len(ps_files)


def infer_process_from_dirname(dir_name: str) -> str:
    """
    Infer the process string from the output directory name.

    Examples:
      ee_to_mumu -> e+ e- > mu+ mu-
      pp_to_ll -> p p > l+ l-
      pp_to_llj -> p p > l+ l- j
      pp_to_bb_qcd0 -> p p > b b~ QCD=0
    """
    # Build process string from underscore-separated parts
    parts = dir_name.split("_")

    # Map particle names and special suffixes
    particle_map = {
        "ee": "e+ e-",
        "pp": "p p",
        "to": ">",
        "mumu": "mu+ mu-",
        "ll": "l+ l-",
        "llj": "l+ l- j",
        "bb": "b b~",
        "bbx": "b b~",
    }

    # Reconstruct with proper spacing
    result = ""
    for part in parts:
        if part in particle_map:
            result += particle_map[part]
            if part not in ["to"]:
                result += " "
        elif part in ["qcd0", "qcd2", "qed2"]:
            # Append coupling orders
            result += part.upper()
            if part != parts[-1]:
                result += " "
        else:
            # Pass through unknown parts
            result += part + " "

    return result.strip()


def extract_from_output_dir(output_dir: str) -> Dict[str, Any]:
    """
    Extract diagram information from a MadGraph output directory.

    Expected structure:
      output_dir/
        SubProcesses/
          P*/  (subprocess directories)
            *.ps (diagram files)
    """
    if not os.path.isdir(output_dir):
        raise NotADirectoryError(f"Output directory not found: {output_dir}")

    dir_name = os.path.basename(output_dir)
    process_name = infer_process_from_dirname(dir_name)

    # Count diagrams in SubProcesses
    subprocesses_dir = os.path.join(output_dir, "SubProcesses")
    diagrams_by_subprocess = {}
    total_diagrams = 0

    if os.path.exists(subprocesses_dir):
        for subprocess_name in sorted(os.listdir(subprocesses_dir)):
            subprocess_path = os.path.join(subprocesses_dir, subprocess_name)
            if os.path.isdir(subprocess_path) and subprocess_name.startswith("P"):
                count = count_diagrams_in_subprocess(subprocess_path)
                diagrams_by_subprocess[subprocess_name] = count
                total_diagrams += count

    return {
        "process": process_name,
        "total_diagrams": total_diagrams,
        "diagrams_by_subprocess": diagrams_by_subprocess,
    }


def main():
    """Extract diagrams from all output directories and write per-directory JSON files."""
    script_dir = Path(__file__).parent
    output_base = script_dir / "output"

    if not output_base.exists():
        print(f"Output directory does not exist: {output_base}")
        print("Run the build script first: bash build.sh")
        sys.exit(1)

    extracted_count = 0
    failed_count = 0

    # Process each output directory and create a JSON file for it
    for output_dir in sorted(output_base.glob("*/")):
        if not output_dir.is_dir():
            continue

        dir_name = output_dir.name

        try:
            print(f"Processing: {dir_name}...", file=sys.stderr)
            diagram_info = extract_from_output_dir(str(output_dir))

            # Write individual JSON file for this directory
            json_path = output_base / f"{dir_name}.json"
            with open(json_path, "w") as f:
                json.dump(diagram_info, f, indent=2)

            print(
                f"  ✓ {diagram_info['total_diagrams']} diagrams found",
                file=sys.stderr,
            )
            print(f"    Wrote: {dir_name}.json", file=sys.stderr)
            extracted_count += 1

        except (FileNotFoundError, NotADirectoryError) as e:
            print(f"  ✗ {e}", file=sys.stderr)
            failed_count += 1

    print(
        f"\n✓ Extracted {extracted_count} process(es) ({failed_count} failed)",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
