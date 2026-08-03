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

Two outputs. The committed one is ``diagrams.json`` beside this script: the
per-process counts alone, which is what the diagram gate asserts against and
what lets it run on a checkout with no work area. The per-directory
``output/DIR.json`` files additionally carry the configs.inc topologies (large
for the 2 -> 6 processes, and printed by the gate for debugging when present),
so they stay in the work area.

The committed file's keys are exactly the rows ``validation/manifest.toml``
declares ``diagrams`` hermetic -- a function of the manifest and the work
area, not of which runs a machine happens to have. A declared row with no
matching work-area directory is an error naming the row; a work-area
directory with no declaration is skipped.

Per-directory ``output/DIR.json`` shape:
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

Diagram counts (``total_diagrams`` / ``diagrams_by_subprocess``) come from the
``NGRAPHS`` parameter in the representative subprocess's matrix file
(``matrix1_orig.f``): the true number of Feynman diagrams for subprocess index 1
of each P-class.  This is deliberately *not* ``MAPCONFIG(0)`` from configs.inc —
MAPCONFIG counts phase-space integration channels for the whole P-class (the
union over every flavour variant in the class), which can exceed the
per-subprocess diagram count (e.g. q q~ > q q~ l+ l- l+ l-: NGRAPHS=2316 for
u u~ > u u~ ... vs MAPCONFIG(0)=2672 for the union).  We compare against the
per-subprocess diagram count because that is what vibegraph enumerates for a
single concrete subprocess.  ``topologies_by_subprocess`` still mirrors the
configs.inc structure, so its entry count reflects the MAPCONFIG union.
"""

import json
import os
import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


# ---------------------------------------------------------------------------
# matrix<N>.f NGRAPHS parser
# ---------------------------------------------------------------------------

# PARAMETER (NGRAPHS=2316)
_RE_NGRAPHS = re.compile(r"PARAMETER\s*\(\s*NGRAPHS\s*=\s*(\d+)\s*\)", re.IGNORECASE)


def parse_ngraphs(path: str) -> Optional[int]:
    """Return the NGRAPHS value (number of Feynman diagrams) from a matrix .f file."""
    try:
        with open(path) as f:
            text = f.read()
    except OSError:
        return None
    m = _RE_NGRAPHS.search(text)
    return int(m.group(1)) if m else None


def representative_ngraphs(subprocess_path: str) -> Optional[int]:
    """
    Number of Feynman diagrams for the representative subprocess (index 1) of a
    P-class directory, read from its matrix file.

    MadGraph writes one matrix file per flavour variant in the class; index 1 is
    the representative (the same one leshouche.inc lists first).  We prefer
    ``matrix1_orig.f`` and fall back to ``template_matrix1.f``.
    """
    for fname in ("matrix1_orig.f", "template_matrix1.f"):
        ng = parse_ngraphs(os.path.join(subprocess_path, fname))
        if ng is not None:
            return ng
    return None


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

            # Diagram count: true NGRAPHS of the representative subprocess (index 1),
            # not MAPCONFIG(0) (which counts the integration-channel union over the
            # whole P-class — see module docstring).
            count = representative_ngraphs(subprocess_path)

            configs_path = os.path.join(subprocess_path, "configs.inc")
            parsed = parse_configs_inc(configs_path)

            # Fall back to the configs.inc count only if no matrix file was found.
            if count is None:
                if parsed is None:
                    continue
                print(
                    f"  ! {subprocess_name}: no matrix1 file, "
                    f"falling back to MAPCONFIG(0)={parsed['n_diagrams']}",
                    file=sys.stderr,
                )
                count = parsed["n_diagrams"]

            diagrams_by_subprocess[subprocess_name] = count
            topologies_by_subprocess[subprocess_name] = (
                parsed["diagrams"] if parsed is not None else []
            )
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


COMMITTED_HEADER = (
    "Per-process Feynman-diagram counts from MadGraph's own enumeration: the "
    "NGRAPHS of each P-class's representative subprocess, summed. Written by "
    "validation/madgraph/extract_diagrams.py from the local MadGraph work area "
    "and committed, so the diagram gate runs against a checkout that has never "
    "run MadGraph. The per-process files beside the work-area process "
    "directories carry the same counts plus the configs.inc topologies, which "
    "the gate prints for debugging when they are present. Keys are exactly the "
    "rows validation/manifest.toml declares diagrams hermetic; the process "
    "string each one generates is in the manifest and in its .mg5 script."
)


def hermetic_diagram_rows(repo_root: Path) -> set:
    """Every ``[[process]]`` key of ``validation/manifest.toml`` whose
    ``categories.diagrams.tier`` is ``"hermetic"``. This is the selector the
    committed ``diagrams.json`` covers: a superset (the oracle layer's bespoke
    runs) or a subset (a row not yet extracted) would make the committed file
    a function of which runs a machine happens to have, rather than of the
    manifest."""
    manifest_path = repo_root / "validation" / "manifest.toml"
    with open(manifest_path, "rb") as f:
        manifest = tomllib.load(f)
    return {
        process["key"]
        for process in manifest.get("process", [])
        if process.get("categories", {}).get("diagrams", {}).get("tier") == "hermetic"
    }


def main():
    """Write the per-directory topology files and the committed count reference."""
    script_dir = Path(__file__).parent
    output_base = script_dir / "output"

    if not output_base.exists():
        print(f"Output directory does not exist: {output_base}")
        print("Run the build script first: bash build.sh")
        sys.exit(1)

    extracted_count = 0
    failed_count = 0
    counts: Dict[str, Any] = {}

    # The committed reference covers exactly the rows the manifest declares
    # `diagrams` hermetic. A work area is a superset — the oracle layer also
    # drives bespoke runs (windowed cross sections, run-card variants) whose
    # directories carry a SubProcesses tree too — and folding those in would
    # make the committed file depend on which of them a machine happens to
    # have run.
    validated = hermetic_diagram_rows(script_dir.parent.parent)
    seen = set()

    for output_dir in sorted(output_base.glob("*/")):
        if not (output_dir / "SubProcesses").is_dir():
            continue

        dir_name = output_dir.name
        if dir_name not in validated:
            print(f"⊘ {dir_name}: not a manifest row with diagrams = hermetic", file=sys.stderr)
            continue
        seen.add(dir_name)

        try:
            print(f"Processing: {dir_name}...", file=sys.stderr)
            diagram_info = extract_from_output_dir(str(output_dir))

            json_path = output_base / f"{dir_name}.json"
            with open(json_path, "w") as f:
                json.dump(diagram_info, f, indent=2)

            counts[dir_name] = {
                "total_diagrams": diagram_info["total_diagrams"],
                "diagrams_by_subprocess": diagram_info["diagrams_by_subprocess"],
            }

            n = diagram_info["total_diagrams"]
            print(f"  ✓ {n} diagrams found", file=sys.stderr)
            print(f"    Wrote: {dir_name}.json", file=sys.stderr)
            extracted_count += 1

        except (FileNotFoundError, NotADirectoryError) as e:
            print(f"  ✗ {e}", file=sys.stderr)
            failed_count += 1

    missing = validated - seen
    if missing:
        print(
            f"✗ manifest declares diagrams hermetic for {sorted(missing)}, "
            "but no matching work-area directory exists; the committed file "
            "may not silently lose a gated row",
            file=sys.stderr,
        )
        sys.exit(1)

    committed = script_dir / "diagrams.json"
    with open(committed, "w") as f:
        json.dump(
            {"_comment": COMMITTED_HEADER, "schema": 1, "processes": counts},
            f,
            indent=2,
            sort_keys=True,
        )
        f.write("\n")

    print(
        f"\n✓ Extracted {extracted_count} process(es) ({failed_count} failed)",
        file=sys.stderr,
    )
    print(f"  Wrote: {committed.relative_to(script_dir.parent.parent)}", file=sys.stderr)


if __name__ == "__main__":
    main()
