#!/usr/bin/env python3
"""Summarise MadEvent's decay-chain and `@N` event samples for the committed
reference decay_chain_events_reference.json.

Input: the rows.txt gen_decay_chain_events.sh writes, one MadEvent run per line
(`name|lines|card|seed|sigma|err|wall|lhe`). Output: per row, the seeds' cross
sections, the first run's <init> block, and — pooled over every seed's events —
what vibegraph_cli's `cli_decay_chain_events` compares its own sample against:

* `structures`: every event's resonance tree, order- and position-free:
  each top-level status-2 record as `pdg[daughters]`, the daughters sorted,
  nested records spelled the same way, the trees sorted and joined by spaces
  (`-6[-24[-14,13],-5] 6[24[-11,12],5]`). Outgoing legs no record claims are
  listed as `.`-prefixed codes at the end.
* `color_mismatches`: status-2 records whose ICOLUP is not what their direct
  daughters leave open once every line one daughter carries as colour and
  another as anticolour is contracted (addmothers.f's elim_indices), and
  `momentum_mismatches`: records whose momentum is not their daughters' sum at
  the file's printed precision. Both are counts over every record.
* `histograms`: fixed edges per observable, counts pooled over seeds; events
  outside every bin are counted in `outside`.
* `spins`: counts of SPINUP per (status, pdg).
* `processes`: counts of IDPRUP.
* `replay`: the first REPLAY_EVENTS events of the first run, as the incoming
  and outgoing momenta, PDG codes, SCALUP and AQCDUP a per-event scale replay
  reads.
"""
import gzip
import json
import math
import re
import sys

REPLAY_EVENTS = 300

MT, WT = 173.0, 1.4915
MW, WW = 80.419002445756, 2.0476
MZ, WZ = 91.188, 2.441404
BWCUTOFF = 15.0


def window(mass, width, bins=20):
    lo, hi = mass - BWCUTOFF * width, mass + BWCUTOFF * width
    return [lo + (hi - lo) * i / bins for i in range(bins + 1)]


def linear(lo, hi, bins):
    return [lo + (hi - lo) * i / bins for i in range(bins + 1)]


def geometric(lo, hi, bins):
    return [lo * (hi / lo) ** (i / bins) for i in range(bins + 1)]


# Observables per row: name -> (edges, extractor(event) -> list of values).
def status2(event, pdg):
    return [p for p in event["particles"] if p["status"] == 2 and p["pdg"] == pdg]


def mass_of(event, pdg):
    return [p["mass"] for p in status2(event, pdg)]


def pt_of(event, pdg):
    return [math.hypot(p["p"][1], p["p"][2]) for p in status2(event, pdg)]


def rapidity_of(event, pdg):
    out = []
    for p in status2(event, pdg):
        e, pz = p["p"][0], p["p"][3]
        out.append(0.5 * math.log((e + pz) / (e - pz)))
    return out


def outgoing_pair_mass(event, a, b):
    ps = [p for p in event["particles"] if p["status"] == 1 and p["pdg"] in (a, b)]
    if len(ps) != 2:
        return []
    s = [ps[0]["p"][i] + ps[1]["p"][i] for i in range(4)]
    return [math.sqrt(max(0.0, s[0] ** 2 - s[1] ** 2 - s[2] ** 2 - s[3] ** 2))]


OBSERVABLES = {
    "pp_ttx_lep_dyn": {
        "m_t": (window(MT, WT), lambda e: mass_of(e, 6)),
        "m_tbar": (window(MT, WT), lambda e: mass_of(e, -6)),
        "m_wp": (window(MW, WW), lambda e: mass_of(e, 24)),
        "m_wm": (window(MW, WW), lambda e: mass_of(e, -24)),
        "pt_t": (linear(0.0, 500.0, 20), lambda e: pt_of(e, 6)),
        "y_t": (linear(-4.0, 4.0, 20), lambda e: rapidity_of(e, 6)),
        "scalup": (geometric(160.0, 900.0, 20), lambda e: [e["scalup"]]),
        "aqcdup": (linear(0.096, 0.121, 20), lambda e: [e["aqcdup"]]),
    },
    "ttx_nested": {
        "m_t": (window(MT, WT), lambda e: mass_of(e, 6)),
        "m_tbar": (window(MT, WT), lambda e: mass_of(e, -6)),
        "m_wp": (window(MW, WW), lambda e: mass_of(e, 24)),
        "pt_t": (linear(0.0, 250.0, 20), lambda e: pt_of(e, 6)),
    },
    "zz_ee": {
        "m_z": (window(MZ, WZ), lambda e: mass_of(e, 23)),
    },
    "dy_two_procs": {
        "m_ee": (linear(60.0, 120.0, 20), lambda e: outgoing_pair_mass(e, -11, 11)),
        "m_mumu": (linear(60.0, 120.0, 20), lambda e: outgoing_pair_mass(e, -13, 13)),
    },
}


def read_lhe(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt") as f:
        text = f.read()
    init = re.search(r"<init>\n(.*?)</init>", text, re.S).group(1).strip().split("\n")
    init = [l for l in init if not l.lstrip().startswith("<")]
    events = []
    for block in re.finditer(r"<event>\n(.*?)</event>", text, re.S):
        lines = block.group(1).strip().split("\n")
        head = lines[0].split()
        n = int(head[0])
        particles = []
        for line in lines[1 : 1 + n]:
            f = line.split()
            particles.append(
                {
                    "pdg": int(f[0]),
                    "status": int(f[1]),
                    "mothers": [int(f[2]), int(f[3])],
                    "color": [int(f[4]), int(f[5])],
                    "p": [float(f[9]), float(f[6]), float(f[7]), float(f[8])],
                    "mass": float(f[10]),
                    "spin": float(f[12]),
                }
            )
        events.append(
            {
                "idprup": int(head[1]),
                "scalup": float(head[3]),
                "aqcdup": float(head[5]),
                "particles": particles,
            }
        )
    return init, events


def daughters(event, index):
    """1-based positions of the records naming `index` as their first mother."""
    return [
        i + 1
        for i, p in enumerate(event["particles"])
        if p["status"] != -1 and p["mothers"][0] == index
    ]


def tree(event, index):
    p = event["particles"][index - 1]
    if p["status"] != 2:
        return str(p["pdg"])
    return "%d[%s]" % (p["pdg"], ",".join(sorted(tree(event, d) for d in daughters(event, index))))


def structure(event):
    tops = []
    loose = []
    for i, p in enumerate(event["particles"]):
        if p["status"] == 2 and p["mothers"][0] <= 2 and event["particles"][p["mothers"][0] - 1]["status"] == -1:
            tops.append(tree(event, i + 1))
        elif p["status"] == 1:
            m = p["mothers"][0]
            if m == 0 or event["particles"][m - 1]["status"] == -1:
                loose.append("." + str(p["pdg"]))
    return " ".join(sorted(tops) + sorted(loose))


def color_ok(event, index):
    p = event["particles"][index - 1]
    open_c, open_a = [], []
    for d in daughters(event, index):
        c, a = event["particles"][d - 1]["color"]
        if c:
            open_c.append(c)
        if a:
            open_a.append(a)
    free_c = [c for c in open_c if c not in open_a]
    free_a = [a for a in open_a if a not in open_c]
    want = [free_c[0] if free_c else 0, free_a[0] if free_a else 0]
    return p["color"] == want and len(free_c) <= 1 and len(free_a) <= 1


def momentum_ok(event, index):
    p = event["particles"][index - 1]
    s = [0.0] * 4
    for d in daughters(event, index):
        for k in range(4):
            s[k] += event["particles"][d - 1]["p"][k]
    return all(abs(s[k] - p["p"][k]) <= 1e-7 * max(1.0, abs(p["p"][0])) for k in range(4))


def summarise(name, runs):
    row = {"lines": runs[0]["lines"], "run_card": runs[0]["card"], "runs": []}
    structures, spins, processes = {}, {}, {}
    hists = {k: {"edges": v[0], "counts": [0] * (len(v[0]) - 1), "outside": 0} for k, v in OBSERVABLES.get(name, {}).items()}
    color_bad = momentum_bad = records = 0
    n_events = 0
    for k, run in enumerate(runs):
        init, events = read_lhe(run["lhe"])
        row["runs"].append({"iseed": run["seed"], "sigma_pb": run["sigma"], "err_pb": run["err"], "events": len(events)})
        if k == 0:
            row["init"] = init
            row["replay"] = [
                {
                    "incoming": [[p["pdg"]] + p["p"] for p in e["particles"] if p["status"] == -1],
                    "outgoing": [[p["pdg"]] + p["p"] for p in e["particles"] if p["status"] == 1],
                    "scalup": e["scalup"],
                    "aqcdup": e["aqcdup"],
                }
                for e in events[:REPLAY_EVENTS]
            ]
        for e in events:
            n_events += 1
            s = structure(e)
            structures[s] = structures.get(s, 0) + 1
            processes[str(e["idprup"])] = processes.get(str(e["idprup"]), 0) + 1
            for i, p in enumerate(e["particles"]):
                key = "%d %d %g" % (p["status"], p["pdg"], p["spin"])
                spins[key] = spins.get(key, 0) + 1
                if p["status"] == 2:
                    records += 1
                    color_bad += not color_ok(e, i + 1)
                    momentum_bad += not momentum_ok(e, i + 1)
            for obs, (edges, fn) in OBSERVABLES.get(name, {}).items():
                for v in fn(e):
                    b = next((j for j in range(len(edges) - 1) if edges[j] <= v < edges[j + 1]), None)
                    if b is None:
                        hists[obs]["outside"] += 1
                    else:
                        hists[obs]["counts"][b] += 1
    row.update(
        {
            "events": n_events,
            "structures": structures,
            "spins": spins,
            "processes": processes,
            "histograms": hists,
            "status2_records": records,
            "color_mismatches": color_bad,
            "momentum_mismatches": momentum_bad,
        }
    )
    if name not in ("pp_ttx_lep_dyn",):
        row.pop("replay", None)
    return row


def main():
    rows_path, out_path, version_path = sys.argv[1:4]
    runs = {}
    for line in open(rows_path):
        name, lines, card, seed, sigma, err, wall, lhe = line.strip().split("|")
        runs.setdefault(name, []).append(
            {"lines": lines, "card": card, "seed": int(seed), "sigma": float(sigma), "err": float(err), "lhe": lhe}
        )
    version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
    out = {
        "_comment": "MadEvent decay-chain and @N event samples, one run per iseed at 10k "
        "unweighted events, summarised by validation/madgraph/summarise_decay_chain_events.py "
        "(see its docstring for every field). Generated by gen_decay_chain_events.sh.",
        "mg_version": version,
        "rows": {name: summarise(name, rs) for name, rs in runs.items()},
    }
    with open(out_path, "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    print("wrote", out_path)


if __name__ == "__main__":
    main()
