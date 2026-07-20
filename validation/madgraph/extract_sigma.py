#!/usr/bin/env python3
"""
Extract the banked leading-order cross section (sigma +- error) from each
fixed-energy MadGraph run under output/ into a single committed JSON reference.

Only fixed-energy partonic runs (lpp1 == lpp2 == 0) are banked: their result is
a partonic sigma-hat directly comparable to a vibegraph integral with no PDF
convolution. Proton-beam (lpp = 1) runs are skipped — their sigma is PDF-folded
and is covered separately by the hadronic reference.

Source of sigma +- err: SubProcesses/results.dat, whose first line's first two
whitespace-separated columns are the cross section and its Monte-Carlo error, in
picobarns (the same numbers MadGraph prints as "Cross-section : X +- Y pb").

The process string ("generate ...") and beam configuration come from the run's
own cards, so the banked entry records exactly what was integrated.

Output: validation/madgraph/sigma_reference.json
  {
    "<dirname>": {
      "process":      "e+ e- > t t~",
      "sigma_pb":     0.54855,
      "sigma_err_pb": 0.00032917,
      "ebeam1":       250.0,
      "ebeam2":       250.0
    },
    ...
  }
"""

import json
import re
import sys
from pathlib import Path
from typing import Dict, Optional, Tuple


def read_results_dat(path: Path) -> Optional[Tuple[float, float]]:
    """First line's (sigma, err) in pb, or None if the file is absent/empty."""
    try:
        with open(path) as f:
            first = f.readline()
    except OSError:
        return None
    cols = first.split()
    if len(cols) < 2:
        return None
    try:
        return float(cols[0]), float(cols[1])
    except ValueError:
        return None


def read_generate(proc_card: Path) -> Optional[str]:
    """The process string from the first `generate` line of a proc_card_mg5.dat."""
    try:
        text = proc_card.read_text()
    except OSError:
        return None
    for line in text.splitlines():
        s = line.strip()
        if s.lower().startswith("generate "):
            return s[len("generate ") :].strip()
    return None


_RE_PARAM = re.compile(r"^\s*([^#!].*?)\s*=\s*([A-Za-z_]\w*)\s*(?:!.*)?$")


def read_run_card_scalars(run_card: Path) -> Dict[str, str]:
    """Map of scalar `name -> raw value` from a run_card.dat (last wins)."""
    out: Dict[str, str] = {}
    try:
        text = run_card.read_text()
    except OSError:
        return out
    for line in text.splitlines():
        m = _RE_PARAM.match(line)
        if m:
            out[m.group(2)] = m.group(1).strip()
    return out


def extract(output_base: Path) -> Dict[str, Dict]:
    banked: Dict[str, Dict] = {}
    for run_dir in sorted(output_base.glob("*/")):
        if not run_dir.is_dir():
            continue
        params = read_run_card_scalars(run_dir / "Cards" / "run_card.dat")
        if params.get("lpp1") != "0" or params.get("lpp2") != "0":
            continue  # only fixed-energy partonic runs are directly comparable
        sigma = read_results_dat(run_dir / "SubProcesses" / "results.dat")
        if sigma is None:
            print(f"  ! {run_dir.name}: no usable results.dat", file=sys.stderr)
            continue
        process = read_generate(run_dir / "Cards" / "proc_card_mg5.dat")
        if process is None:
            print(f"  ! {run_dir.name}: no generate line", file=sys.stderr)
            continue
        banked[run_dir.name] = {
            "process": process,
            "sigma_pb": sigma[0],
            "sigma_err_pb": sigma[1],
            "ebeam1": float(params.get("ebeam1", "0")),
            "ebeam2": float(params.get("ebeam2", "0")),
        }
        print(
            f"  + {run_dir.name}: {process}  "
            f"sigma = {sigma[0]:.5g} +- {sigma[1]:.5g} pb",
            file=sys.stderr,
        )
    return banked


def main() -> None:
    script_dir = Path(__file__).parent
    output_base = script_dir / "output"
    if not output_base.exists():
        print(f"Output directory does not exist: {output_base}", file=sys.stderr)
        print("Run the MadGraph build first (bash build.sh)", file=sys.stderr)
        sys.exit(1)

    banked = extract(output_base)
    if not banked:
        print("No fixed-energy runs found to bank", file=sys.stderr)
        sys.exit(1)

    out_path = script_dir / "sigma_reference.json"
    with open(out_path, "w") as f:
        json.dump(banked, f, indent=2, sort_keys=True)
        f.write("\n")
    print(f"\nBanked {len(banked)} process(es) -> {out_path.name}", file=sys.stderr)


if __name__ == "__main__":
    main()
