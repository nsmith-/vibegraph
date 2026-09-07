#!/usr/bin/env python3
"""Bank the interaction count MadGraph arrives at for each restricted model.

MadGraph does not carry one interaction per UFO vertex. `import_ufo.add_interaction`
splits a vertex into one interaction per distinct coupling-order tuple, each
carrying only that tuple's `(color, lorentz)` couplings, and the restriction then
drops the interactions whose couplings all vanish under the card. The count that
survives both steps is a property of the (model, restrict card) pair alone -- not
of any process -- and it is the number a loader claiming to reproduce MadGraph's
model topology has to reproduce first, before any diagram is enumerated.

MadGraph exposes it through `display interactions`, whose first line is

    Current model contains 913 interactions

followed by one line per interaction giving its particles and its order tuple.
Every `.mg5` script that imports a non-default model runs that command right
after the import, so the count lands in the process directory's `build.log`, and
this reads it back out.

Two rows that name the same (model, restrict) must report the same count; that
they do is asserted here rather than assumed, since the count is banked once per
card and read by rows that never see each other.

Usage:
  python validation/madgraph/extract_interactions.py
"""

import json
import os
import re
import sys
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
OUTPUT_BASE = HERE / "output"
COMMITTED = HERE / "interactions.json"

RE_COUNT = re.compile(r"Current model contains\s+(\d+)\s+interactions")

COMMITTED_HEADER = (
    "How many interactions MadGraph's own model topology holds for each "
    "(UFO model, restrict card) pair the validation manifest names: the count "
    "after import_ufo.add_interaction has split every vertex into one "
    "interaction per coupling-order tuple and the restriction has dropped the "
    "ones whose couplings all vanish. Read out of `display interactions` in each "
    "row's build log by validation/madgraph/extract_interactions.py and "
    "committed, so a checkout that has never run MadGraph can compare its own "
    "model topology against it. `rows` lists the manifest rows generated under "
    "that pair."
)


def model_rows():
    """Manifest rows that name a non-default UFO model, keyed by (model, restrict)."""
    with open(REPO_ROOT / "validation" / "manifest.toml", "rb") as f:
        manifest = tomllib.load(f)
    pairs = {}
    for entry in manifest.get("process", []):
        model = entry.get("model")
        if model is None:
            continue
        key = (model, entry.get("restrict"))
        pairs.setdefault(key, []).append(entry["key"])
    return pairs


def count_from_build_log(row: str):
    path = OUTPUT_BASE / row / "build.log"
    if not path.exists():
        return None
    counts = RE_COUNT.findall(path.read_text(errors="replace"))
    if not counts:
        return None
    return int(counts[-1])


def main():
    pairs = model_rows()
    if not pairs:
        print("no manifest row names a model; nothing to extract", file=sys.stderr)
        return

    banked = {}
    missing = []
    for (model, restrict), rows in sorted(pairs.items()):
        observed = {}
        for row in sorted(rows):
            n = count_from_build_log(row)
            if n is None:
                missing.append(row)
                continue
            observed[row] = n
        if not observed:
            continue
        distinct = sorted(set(observed.values()))
        if len(distinct) != 1:
            print(
                f"✗ {model}-{restrict}: rows disagree on the interaction count: {observed}",
                file=sys.stderr,
            )
            sys.exit(1)
        name = f"{os.path.basename(model)}-{restrict}"
        banked[name] = {
            "model": model,
            "restrict": restrict,
            "interactions": distinct[0],
            "rows": sorted(observed),
        }
        print(f"  ✓ {name}: {distinct[0]} interactions", file=sys.stderr)

    if missing:
        print(
            f"✗ no `display interactions` line in the build log of {sorted(missing)}; "
            "the committed file may not silently lose a row's card",
            file=sys.stderr,
        )
        sys.exit(1)

    with open(COMMITTED, "w") as f:
        json.dump(
            {"_comment": COMMITTED_HEADER, "schema": 1, "models": banked},
            f,
            indent=2,
            sort_keys=True,
        )
        f.write("\n")
    print(f"  Wrote: {COMMITTED.relative_to(REPO_ROOT)}", file=sys.stderr)


if __name__ == "__main__":
    main()
