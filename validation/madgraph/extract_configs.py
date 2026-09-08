#!/usr/bin/env python3
"""
Extract MadGraph's integration-configuration and colour-flow counts.

Each single-subprocess run directory carries ``SubProcesses/P*/coloramps.inc``,
whose declaration ``LOGICAL ICOLAMP(maxflows, nconfigs, nsubproc)`` is written
by MadGraph's own exporter: ``maxflows`` is NCOLOR and ``nconfigs`` the number
of integration channels MadEvent runs the subprocess with. They come from the
exporter rather than from any dump this repository asked MadGraph for, which
is what makes them an independent pin on the channel set the multichannel
sampler is built on.

The committed output is ``configs.json`` beside this script:

  {
    "<manifest row key>": {"flows": NCOLOR, "configs": nconfigs},
    ...
  }

keyed by every manifest row whose work-area directory holds exactly one
subprocess directory declaring one subprocess. A directory grouping several
flavour assignments (``nsubproc > 1``, or several ``P*`` directories) writes
one ``nconfigs`` over the whole group, which no single subprocess's diagrams
determine, so those rows are skipped rather than recorded. A manifest row with no work-area directory
is skipped too: the file is a function of the manifest and the work area
present on this machine, and the gate that reads it says how many rows it
requires.
"""

import json
import re
import sys
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUTPUT = HERE / "output"
MANIFEST = HERE.parent / "manifest.toml"
TARGET = HERE / "configs.json"

DECL = re.compile(r"^\s*LOGICAL\s+ICOLAMP\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", re.I)


def icolamp_dims(inc: Path):
    for line in inc.read_text().splitlines():
        m = DECL.match(line)
        if m:
            return tuple(int(g) for g in m.groups())
    sys.exit(f"{inc}: no ICOLAMP declaration")


def main():
    with MANIFEST.open("rb") as f:
        manifest = tomllib.load(f)
    result = {}
    for row in manifest["process"]:
        key = row["key"]
        subprocesses = OUTPUT / key / "SubProcesses"
        if not subprocesses.is_dir():
            continue
        dirs = sorted(p for p in subprocesses.iterdir() if p.is_dir() and p.name.startswith("P"))
        if len(dirs) != 1:
            continue
        inc = dirs[0] / "coloramps.inc"
        if not inc.is_file():
            sys.exit(f"{key}: {inc} missing")
        flows, configs, subprocs = icolamp_dims(inc)
        # One directory, several flavour assignments: MadGraph writes the group's
        # configuration count, which no single subprocess's diagrams determine.
        if subprocs != 1:
            continue
        result[key] = {"flows": flows, "configs": configs}
    TARGET.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(f"wrote {TARGET} ({len(result)} rows)")


if __name__ == "__main__":
    main()
