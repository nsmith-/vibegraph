#!/usr/bin/env python3
"""The MLM rows' two censuses, written to mlm_census.json.

1. Jet-ness within one IPROC (static, from each banked process directory's
   leshouche.inc). rewgt overwrites a final-state line's PDG with the chosen
   flavour combination's only when that flavour is a jet (reweight.f, "Set jet
   identities according to chosen subprocess"); a line that is a jet in one
   combination of an IPROC and not in another keeps whatever PDG an earlier
   event left there. Any leg of any IPROC whose isjet() differs across its
   flavour combinations is listed; an empty list means the defect cannot reach
   the row's weights. The dump's RWLEG records measure the same thing per event
   (final_leg_ipdgcl_differs_from_idup).

2. The jet memo (dynamic, from the instrumented replay's dump head, when it
   exists): how often setclscales took the store-and-recluster and the
   restricted-recluster branches at any point of the run, which njetstore values
   the written events' first calls found per directory and channel, and the
   per-event coverage counters that bear on it.

isjet(id) is |id| <= maxjetflavor or id = 21, with maxjetflavor from the row's
committed run card (Template/LO/SubProcesses/reweight.f, isjet).

Usage: python validation/madgraph/dump_mlm_census.py [row ...]
"""

from __future__ import annotations

import gzip
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROWS = [
    "pp_to_llj_mlm",
    "pp_to_llj_xqcut_only",
    "pp_to_llj_mlm_alps2",
    "pp_to_ll_0j2j_mlm",
    "pp_to_ttx_0j1j_mlm",
]
OUT = HERE / "mlm_census.json"
IDUP = re.compile(r"DATA \(IDUP\(I,(\d+),(\d+)\),I=1,\d+\)/([^/]+)/")


def maxjetflavor(row: str) -> int:
    card = (HERE / f"{row}_run_card.dat").read_text()
    m = re.search(r"^\s*(\d+)\s*=\s*maxjetflavor\b", card, re.M)
    return int(m.group(1)) if m else 4


def iproc_census(row: str) -> dict | None:
    procdir = HERE / "output" / row / "SubProcesses"
    if not procdir.is_dir():
        return None
    mjf = maxjetflavor(row)

    def isjet(pdg: int) -> bool:
        return abs(pdg) <= mjf or pdg == 21

    mixed = []
    n_iproc = 0
    n_combinations = 0
    for pdir in sorted(procdir.glob("P*")):
        les = pdir / "leshouche.inc"
        if not les.is_file():
            continue
        by_iproc: dict[int, list[list[int]]] = defaultdict(list)
        for ipsel, iproc, ids in IDUP.findall(les.read_text()):
            by_iproc[int(iproc)].append([int(x) for x in ids.split(",")])
        for iproc, combos in sorted(by_iproc.items()):
            n_iproc += 1
            n_combinations += len(combos)
            for leg in range(len(combos[0])):
                flags = {isjet(c[leg]) for c in combos}
                if len(flags) > 1:
                    mixed.append({
                        "directory": pdir.name,
                        "iproc": iproc,
                        "leg": leg + 1,
                        "final_state": leg >= 2,
                        "flavours": sorted({c[leg] for c in combos}),
                    })
    return {
        "maxjetflavor": mjf,
        "n_iproc": n_iproc,
        "n_flavour_combinations": n_combinations,
        "mixed_jetness": mixed,
        "n_mixed_final_state": sum(1 for m in mixed if m["final_state"]),
    }


def memo_census(row: str) -> dict | None:
    dump = HERE / "output" / "ktdump" / "dumps" / f"{row}.jsonl.gz"
    if not dump.is_file():
        return None
    with gzip.open(dump, "rt") as f:
        head = json.loads(f.readline())
    cov = head.get("coverage", {})
    census = head.get("census", {})
    keep = [
        "memo",
        "cluster_calls_per_event",
        "setclscales_calls_per_event",
        "final_leg_ipdgcl_differs_from_idup",
        "vec_igraph_zero",
        "vec_igraph_is_igraphs1",
        "vec_igraph_is_iconfig",
        "rwgt_kill",
    ]
    return {
        "n_events": head.get("n_events"),
        "coverage": {k: cov[k] for k in keep if k in cov},
        "memo_branches_all_points": census.get("memo_branches_all_points", {}),
        "setclscales_calls_through_last_written_event": census.get(
            "setclscales_calls_through_last_written_event", {}),
        "njetstore_on_entry_by_channel": census.get("njetstore_on_entry_by_channel", {}),
    }


def main() -> int:
    rows = sys.argv[1:] or ROWS
    doc = json.loads(OUT.read_text()) if OUT.is_file() else {"rows": {}}
    for row in rows:
        entry = doc["rows"].get(row, {})
        static = iproc_census(row)
        if static is not None:
            entry["iproc_jetness"] = static
        dynamic = memo_census(row)
        if dynamic is not None:
            entry["jet_memo"] = dynamic
        if entry:
            doc["rows"][row] = entry
            s = entry.get("iproc_jetness", {})
            m = entry.get("jet_memo", {})
            print(f"{row}: {s.get('n_iproc')} IPROCs, "
                  f"{s.get('n_mixed_final_state')} final-state legs of mixed jet-ness; "
                  f"memo branches {m.get('memo_branches_all_points')}")
        else:
            print(f"{row}: no banked run and no dump yet", file=sys.stderr)
    doc["_comment"] = (
        "MLM-row censuses: jet-ness mixed within an IPROC (the stale-ipdgcl defect in "
        "rewgt) from leshouche.inc, and the per-channel jet memo's re-cluster branches "
        "from the instrumented replay. Generated by validation/madgraph/dump_mlm_census.py."
    )
    doc["rows"] = dict(sorted(doc["rows"].items()))
    OUT.write_text(json.dumps(doc, indent=2, sort_keys=True) + "\n")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
