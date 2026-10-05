#!/usr/bin/env python3
"""Time `generate --reweight-card` against MadGraph's reweight module on the
reweight-oracle rows.

Reads the work area `gen_reweight_oracle.py` leaves behind (each row's integration
grid, proc card, reweight card and event file), so run that first.

vibegraph: the wall time of `generate` with the row's reweight card minus the same
run without it (same grid, seed and event count), median over `--reps` runs, on the
exact path, the default plan and, where the row names couplings, the joint
polynomial path. Measured at the row's own event count and at `--nevents`, where
the difference is resolved above the noise of the generation itself.

MadGraph: its reweight module on the row's own events with the row's first
hypothesis, under cProfile, split into the one-off setup (process generation and
the Fortran build, `create_standalone_directory` and `compile`) and the event loop
(`launch_actual_reweighting`) it runs once per launch. Every hypothesis of the row
is then estimated as setup + H x loop: MadGraph 3.7.1 writes one mislabelled weight
for a multi-launch card through `ReweightInterface.import_command_file`, the same
defect that makes `gen_reweight_oracle.py` run one hypothesis per work area.

The two loops do not do the same work: MadGraph reads and rewrites the event file
and runs its per-event bookkeeping in Python, while vibegraph reweights the momenta
it holds inside `generate`.

  python validation/madgraph/bench_reweight.py                 # every row
  python validation/madgraph/bench_reweight.py ee_tth_ymt --reps 7 --no-madgraph
"""

import argparse
import importlib.util
import json
import os
import pstats
import resource
import shutil
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
_spec = importlib.util.spec_from_file_location("gen_reweight_oracle", os.path.join(HERE, "gen_reweight_oracle.py"))
oracle = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(oracle)

SEED = "20261005"
DEFAULT_OUT = os.path.join(HERE, "output", "reweight_bench")


def timed(cmd, log=None):
    """Wall and child CPU seconds of one command, and its stderr."""
    r0 = resource.getrusage(resource.RUSAGE_CHILDREN)
    t0 = time.perf_counter()
    p = subprocess.run(cmd, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    wall = time.perf_counter() - t0
    r1 = resource.getrusage(resource.RUSAGE_CHILDREN)
    if log:
        with open(log, "w") as f:
            f.write(p.stdout + p.stderr)
    if p.returncode:
        sys.stderr.write(p.stdout[-3000:] + p.stderr[-3000:])
        raise SystemExit(f"failed: {' '.join(cmd[:3])} ...")
    cpu = (r1.ru_utime - r0.ru_utime) + (r1.ru_stime - r0.ru_stime)
    return wall, cpu, p.stderr


def vibegraph(binary, row, work, out, nevents, reps):
    """Per variant: median wall and CPU of `generate`, and the plan it logged."""
    model, ufo = oracle.vibegraph_model(row)
    card = os.path.join(work, "reweight_card.dat")
    base = [binary, "generate", os.path.join(work, "integrate", "grid.bin.zst"), os.path.join(work, "proc_card.dat"),
            *ufo, "--run-card", os.path.join(ROOT, row["run_card"]), "--pdf-dir", os.path.join(ROOT, "validation", "pdf"),
            "--seed", SEED, "--nevents", str(nevents), "--force", "-o", os.path.join(out, "events.lhe")]
    variants = {
        "none": [],
        "exact": ["--reweight-card", card, "--reweight-exact"],
        "default": ["--reweight-card", card],
    }
    if row["couplings"]:
        variants["joint"] = ["--reweight-card", card, "--reweight-couplings", row["couplings"]]
    runs = {name: [] for name in variants}
    plans = {}
    # Interleaved, so a drift in the host's speed lands on every variant alike.
    for _ in range(reps):
        for name, extra in variants.items():
            wall, cpu, err = timed(base + extra)
            runs[name].append((wall, cpu))
            plans[name] = [l.split("INFO ", 1)[-1].strip() for l in err.splitlines() if "evaluations per event" in l]
    return {
        name: dict(wall=statistics.median(w for w, _ in r), cpu=statistics.median(c for _, c in r), plan=plans[name])
        for name, r in runs.items()
    }


def madgraph(row, work, out):
    """One hypothesis under cProfile: setup, event loop and matrix-element calls."""
    here = os.path.join(out, "madgraph")
    shutil.rmtree(here, ignore_errors=True)
    os.makedirs(here)
    param_card, proc_card = oracle.madgraph_cards(row, work)
    events = os.path.join(here, "events.lhe")
    oracle.madgraph_input(os.path.join(work, "events.lhe"), proc_card, os.path.join(ROOT, row["run_card"]),
                          param_card, events)
    card = os.path.join(here, "reweight_card.dat")
    hid, lines = row["hypotheses"][0]
    with open(card, "w") as f:
        f.write("change helicity False\n" f"launch --rwgt_name={hid}\n" + "".join(f" {l}\n" for l in lines))
    prof = os.path.join(here, "profile.out")
    code = (
        "import sys, os, cProfile\n"
        f"sys.path.insert(0, {oracle.MG_ROOT!r})\n"
        f"os.chdir({here!r})\n"
        "import madgraph.interface.reweight_interface as rwi\n"
        "def go():\n"
        f"    cmd = rwi.ReweightInterface({events!r}, allow_madspin=True)\n"
        f"    cmd.import_command_file({card!r})\n"
        "    cmd.do_quit('')\n"
        f"cProfile.run('go()', {prof!r})\n"
    )
    wall, cpu, _ = timed([sys.executable, "-u", "-c", code], log=os.path.join(here, "log.txt"))
    stats = pstats.Stats(prof).stats

    def own(name, column):
        return sum(v[column] for k, v in stats.items() if k[2] == name and "reweight_interface" in k[0])

    return dict(
        wall=wall,
        cpu=cpu,
        setup=own("create_standalone_directory", 3) + own("compile", 3),
        loop=own("launch_actual_reweighting", 3),
        calculate_weight=own("calculate_weight", 3),
        me_calls=own("calculate_matrix_element", 1),
    )


def report(key, r):
    """The row's summary, in microseconds per event and seconds per sample."""
    n, h = r["nevents"], r["hypotheses"]
    print(f"== {key}: {n} events, {h} hypotheses")
    for label, block, count in (("sample", r["vibegraph"], n), ("large", r["vibegraph_large"], r["nevents_large"])):
        none = block["none"]["wall"]
        for name in ("exact", "default", "joint"):
            if name in block:
                cost = block[name]["wall"] - none
                plan = "; ".join(block[name]["plan"])
                print(f"   vibegraph {label:6} {name:7} {cost * 1e3:8.1f} ms  {cost / count * 1e6:7.1f} us/event  ({plan})")
    m = r.get("madgraph")
    if m:
        per = m["loop"] / n * 1e6
        print(f"   madgraph  setup {m['setup']:.1f} s, loop {m['loop']:.2f} s per hypothesis = {per:.0f} us/event; "
              f"all {h} hypotheses ~ {m['setup'] + h * m['loop']:.1f} s")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rows", nargs="*", default=sorted(oracle.ROWS))
    ap.add_argument("--work", default=oracle.DEFAULT_WORK, help="gen_reweight_oracle.py's work area")
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--reps", type=int, default=5)
    ap.add_argument("--nevents", type=int, default=20000, help="the larger sample vibegraph is timed at")
    ap.add_argument("--no-madgraph", action="store_true")
    args = ap.parse_args()
    binary = os.environ.get("VIBEGRAPH_BIN", os.path.join(ROOT, "target", "release-debug", "vibegraph"))

    results = {}
    for key in args.rows:
        row, work = oracle.ROWS[key], os.path.join(args.work, key)
        if not os.path.isfile(os.path.join(work, "integrate", "grid.bin.zst")):
            raise SystemExit(f"{key}: no grid under {work}; run gen_reweight_oracle.py {key} first")
        out = os.path.join(args.out, key)
        os.makedirs(out, exist_ok=True)
        r = dict(nevents=row["nevents"], nevents_large=args.nevents, hypotheses=len(row["hypotheses"]),
                 vibegraph=vibegraph(binary, row, work, out, row["nevents"], args.reps),
                 vibegraph_large=vibegraph(binary, row, work, out, args.nevents, args.reps))
        if not args.no_madgraph:
            r["madgraph"] = madgraph(row, work, out)
        results[key] = r
        report(key, r)
    with open(os.path.join(args.out, "bench.json"), "w") as f:
        json.dump(results, f, indent=1)


if __name__ == "__main__":
    main()
