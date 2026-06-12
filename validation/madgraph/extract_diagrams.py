#!/usr/bin/env python3
"""
Extract diagram information from MadGraph output directories.

Dynamically processes all output directories under output/ and creates
a JSON file for each one with diagram counts and topology information.

Expected structure:
  output/
    ee_to_mumu_lo/
      SubProcesses/
        P*/
          configs.inc  ← primary source: diagram count + topology
    pp_to_ll_lo/
      ...

Output: Creates output/DIR.json for each directory with:
  {
    "process": "inferred from directory name",
    "total_diagrams": count,
    "diagrams_by_subprocess": {"P1_...": count, ...},
    "topologies_by_subprocess": {
      "P1_...": [
        {
          "diagram_id": 1,
          "clusters": [
            {"cluster": -1, "legs": [4, 3], "sprop": [22], "tprop": 0},
            ...
          ]
        },
        ...
      ]
    }
  }

The topology encoding mirrors MadGraph's IFOREST/SPROP/TPRID arrays:
  - legs:  the two sub-items that cluster together (positive = external leg
           number 1-indexed, negative = a previously formed cluster)
  - sprop: PDG codes of the s-channel particles that can propagate (when
           the cluster is an s-channel propagator, tprop == 0)
  - tprop: PDG code of the t-channel propagator (0 means this cluster is
           an s-channel propagator)
"""

import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


# ---------------------------------------------------------------------------
# configs.inc parser
# ---------------------------------------------------------------------------

def parse_configs_inc(path: str) -> Optional[Dict[str, Any]]:
    """
    Parse a MadGraph configs.inc file and return diagram topology data.

    Returns a dict with keys:
      "n_diagrams"  – total number of diagrams (from MAPCONFIG(0))
      "diagrams"    – list of diagram records, each with:
                        "diagram_id": int,
                        "clusters":   list of cluster records, each with:
                          "cluster": int  (negative index, e.g. -1)
                          "legs":    [int, int]  (external legs or cluster refs)
                          "sprop":   [int, ...]  (s-channel PDG codes, empty if t-chan)
                          "tprop":   int          (t-channel PDG, 0 = s-channel)

    Returns None if the file cannot be parsed.
    """
    try:
        with open(path) as f:
            text = f.read()
    except OSError:
        return None

    # Join Fortran 77 fixed-form continuation lines (non-blank char in column 6).
    # MadGraph generates DATA statements that span lines for high-multiplicity processes.
    joined_lines = []
    for line in text.split("\n"):
        if len(line) > 5 and line[5] not in (" ", "0") and line[0].upper() not in ("C", "!", "*"):
            if joined_lines:
                joined_lines[-1] = joined_lines[-1] + line[6:]
            else:
                joined_lines.append(line[6:])
        else:
            joined_lines.append(line)
    text = "\n".join(joined_lines)

    # DATA MAPCONFIG(D)/V/  ← diagram number or DATA MAPCONFIG(0)/N/ for total
    re_mapconfig = re.compile(
        r"DATA\s+MAPCONFIG\s*\(\s*(\d+)\s*\)\s*/\s*(\d+)\s*/", re.IGNORECASE
    )
    # DATA (IFOREST(I,-K,D),I=1,2)/A,B/
    re_iforest = re.compile(
        r"DATA\s+\(IFOREST\s*\(I\s*,\s*(-\d+)\s*,\s*(\d+)\s*\)\s*,I=1,2\)\s*/\s*(-?\d+)\s*,\s*(-?\d+)\s*/",
        re.IGNORECASE,
    )
    # DATA (SPROP(I,-K,D),I=1,N)/PDG1,...,PDGN/
    re_sprop = re.compile(
        r"DATA\s+\(SPROP\s*\(I\s*,\s*(-\d+)\s*,\s*(\d+)\s*\)\s*,I=1,\d+\)\s*/([^/]+)/",
        re.IGNORECASE,
    )
    # DATA TPRID(-K,D)/PDG/
    re_tprid = re.compile(
        r"DATA\s+TPRID\s*\(\s*(-\d+)\s*,\s*(\d+)\s*\)\s*/\s*(-?\d+)\s*/",
        re.IGNORECASE,
    )

    n_diagrams: Optional[int] = None
    # diagrams[d] = {"diagram_id": d, "clusters": {cluster_idx: {...}}}
    diagrams: Dict[int, Dict] = {}

    for m in re_mapconfig.finditer(text):
        d, v = int(m.group(1)), int(m.group(2))
        if d == 0:
            n_diagrams = v
        else:
            if d not in diagrams:
                diagrams[d] = {"diagram_id": d, "clusters": {}}

    for m in re_iforest.finditer(text):
        cluster, diag = int(m.group(1)), int(m.group(2))
        leg_a, leg_b = int(m.group(3)), int(m.group(4))
        if diag not in diagrams:
            diagrams[diag] = {"diagram_id": diag, "clusters": {}}
        if cluster not in diagrams[diag]["clusters"]:
            diagrams[diag]["clusters"][cluster] = {"cluster": cluster, "legs": [], "sprop": [], "tprop": 0}
        diagrams[diag]["clusters"][cluster]["legs"] = [leg_a, leg_b]

    for m in re_sprop.finditer(text):
        cluster, diag = int(m.group(1)), int(m.group(2))
        pdgs = [int(x.strip()) for x in m.group(3).split(",") if x.strip()]
        # Keep only the distinct non-zero PDG codes.
        unique_pdgs = sorted(set(p for p in pdgs if p != 0))
        if diag not in diagrams:
            diagrams[diag] = {"diagram_id": diag, "clusters": {}}
        if cluster not in diagrams[diag]["clusters"]:
            diagrams[diag]["clusters"][cluster] = {"cluster": cluster, "legs": [], "sprop": [], "tprop": 0}
        diagrams[diag]["clusters"][cluster]["sprop"] = unique_pdgs

    for m in re_tprid.finditer(text):
        cluster, diag = int(m.group(1)), int(m.group(2))
        pdg = int(m.group(3))
        if diag not in diagrams:
            diagrams[diag] = {"diagram_id": diag, "clusters": {}}
        if cluster not in diagrams[diag]["clusters"]:
            diagrams[diag]["clusters"][cluster] = {"cluster": cluster, "legs": [], "sprop": [], "tprop": 0}
        diagrams[diag]["clusters"][cluster]["tprop"] = pdg

    if n_diagrams is None and diagrams:
        n_diagrams = max(diagrams.keys())

    # Flatten clusters dict → sorted list
    result_diagrams = []
    for d in sorted(diagrams.keys()):
        clusters = sorted(diagrams[d]["clusters"].values(), key=lambda c: c["cluster"])
        result_diagrams.append({"diagram_id": d, "clusters": clusters})

    return {
        "n_diagrams": n_diagrams or 0,
        "diagrams": result_diagrams,
    }


def extract_from_output_dir(output_dir: str) -> Dict[str, Any]:
    """
    Extract diagram information from a MadGraph output directory.

    Expected structure:
      output_dir/
        SubProcesses/
          P*/  (subprocess directories)
            configs.inc  ← authoritative diagram count + topology
    """
    if not os.path.isdir(output_dir):
        raise NotADirectoryError(f"Output directory not found: {output_dir}")

    dir_name = os.path.basename(output_dir)
    process_name = infer_process_from_dirname(dir_name)

    subprocesses_dir = os.path.join(output_dir, "SubProcesses")
    diagrams_by_subprocess: Dict[str, int] = {}
    topologies_by_subprocess: Dict[str, List] = {}
    total_diagrams = 0

    if os.path.exists(subprocesses_dir):
        for subprocess_name in sorted(os.listdir(subprocesses_dir)):
            subprocess_path = os.path.join(subprocesses_dir, subprocess_name)
            if not (os.path.isdir(subprocess_path) and subprocess_name.startswith("P")):
                continue

            configs_path = os.path.join(subprocess_path, "configs.inc")
            parsed = parse_configs_inc(configs_path)
            if parsed is None:
                continue

            count = parsed["n_diagrams"]
            diagrams_by_subprocess[subprocess_name] = count
            topologies_by_subprocess[subprocess_name] = parsed["diagrams"]
            total_diagrams += count

    return {
        "process": process_name,
        "total_diagrams": total_diagrams,
        "diagrams_by_subprocess": diagrams_by_subprocess,
        "topologies_by_subprocess": topologies_by_subprocess,
    }


def infer_process_from_dirname(dir_name: str) -> str:
    """
    Infer the process string from the output directory name.

    Examples:
      ee_to_mumu -> e+ e- > mu+ mu-
      pp_to_ll -> p p > l+ l-
      pp_to_llj -> p p > l+ l- j
      pp_to_bb_qcd0 -> p p > b b~ QCD=0
    """
    parts = dir_name.split("_")

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

    result = ""
    for part in parts:
        if part in particle_map:
            result += particle_map[part]
            if part not in ["to"]:
                result += " "
        elif part in ["qcd0", "qcd2", "qed2"]:
            result += part.upper()
            if part != parts[-1]:
                result += " "
        else:
            result += part + " "

    return result.strip()


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

    for output_dir in sorted(output_base.glob("*/")):
        if not output_dir.is_dir():
            continue

        dir_name = output_dir.name

        try:
            print(f"Processing: {dir_name}...", file=sys.stderr)
            diagram_info = extract_from_output_dir(str(output_dir))

            json_path = output_base / f"{dir_name}.json"
            with open(json_path, "w") as f:
                json.dump(diagram_info, f, indent=2)

            n = diagram_info["total_diagrams"]
            print(f"  ✓ {n} diagrams found", file=sys.stderr)
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
