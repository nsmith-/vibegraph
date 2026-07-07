#!/usr/bin/env python3
"""Generic per-diagram MadGraph-vs-vibegraph amplitude comparison.

Reads vibegraph's [diagram x helicity] dump (written by the Rust probe
`helas::eval::run::tests::probe_process_diagrams` with VG_PROBE_NAME=<name>),
evaluates MadGraph's per-diagram AMP() (module mg_amp_probe_<name>, built by
build_amplitude.sh) at exactly the same phase-space point (CSV point 0) and
helicity combos, auto-matches diagrams by full-helicity-vector overlap, and
reports the per-diagram complex ratio r = <a_mg|a_vg>/<a_mg|a_mg>:
a correct diagram has |r| = 1 and phase 0 (up to one global sign/phase,
factored out and reported separately).

Usage:
  pixi run -e madgraph python validation/madgraph/compare_amps.py <name> [--cf CF]

Prerequisites (in order):
  pixi run -e madgraph build-amplitude          # builds mg_amp_probe_<name>
  pixi run -e madgraph generate-amplitude       # writes <name>_amplitude.csv
  VG_PROBE_NAME=<name> cargo test -p vibegraph-lib --release \
      --lib helas::eval::run::tests::probe_process_diagrams -- --ignored --nocapture
"""

import argparse
import importlib
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name", help="process name, e.g. ee_to_ee")
    ap.add_argument(
        "--cf", type=float, default=1.0, help="color factor CF(1,1) (1 leptons, 3, 9)"
    )
    args = ap.parse_args()
    name = args.name

    # vibegraph dump: '#hel' header + one row per diagram.
    vg_path = os.path.join(HERE, "output", f"vibegraph_amps_{name}.txt")
    with open(vg_path) as f:
        header = f.readline().rstrip("\n").split("\t")
        assert header[0] == "#hel", "run the Rust probe first (writes the #hel header)"
        combos = [[int(h) for h in c.split(",")] for c in header[1:]]
        sigs, rows = [], []
        for line in f:
            parts = line.rstrip("\n").split("\t")
            sigs.append(parts[1])
            vals = np.array(list(map(float, parts[2:])))
            rows.append(vals[0::2] + 1j * vals[1::2])
    vg = np.array(rows)  # [n_vg, n_hel]
    n_hel = len(combos)
    assert vg.shape[1] == n_hel

    # CSV point 0 momenta (same point the Rust probe used).
    csv_path = os.path.join(HERE, "output", f"{name}_amplitude.csv")
    with open(csv_path) as f:
        lines = [l for l in f if l.strip()]
    n_ext = next(int(l.split(":")[1]) for l in lines if l.startswith("# n_ext:"))
    data = [l for l in lines if not l.startswith("#")][1:]  # skip column header
    row0 = [float(x) for x in data[0].split(",")]
    m2_ref = row0[0]
    p = np.zeros((4, n_ext), dtype=np.float64, order="F")
    for leg in range(n_ext):
        p[:, leg] = row0[1 + 4 * leg : 5 + 4 * leg]

    card = os.path.join(HERE, "output", name, "Cards", "param_card.dat")
    module = importlib.import_module(f"mg_amp_probe_{name}")

    mg_rows = []
    for hel in combos:
        nhel = np.array(hel, dtype=np.int32)
        mg_rows.append(np.asarray(module.mg_eval_amp(p, nhel, card)))
    mg = np.array(mg_rows).T  # [n_mg, n_hel]

    print(f"[{name}] vibegraph diagrams: {vg.shape[0]}, MadGraph NGRAPHS: {mg.shape[0]}")
    m2_mg = args.cf * float(np.sum(np.abs(mg.sum(axis=0)) ** 2))
    print(f"CSV point-0 |M|² ref = {m2_ref:.10e}")
    print(
        f"MG probe Σ|ΣAMP|²·CF = {m2_mg:.10e}   (JAMP signs NOT applied; equality"
        " only when all JAMP coefficients are +1)"
    )

    # Auto-match: normalized overlap |<vg_i|mg_j>| / (||vg_i|| ||mg_j||), greedy.
    vgn = np.linalg.norm(vg, axis=1)
    mgn = np.linalg.norm(mg, axis=1)
    overlap = np.abs(vg.conj() @ mg.T) / np.outer(np.maximum(vgn, 1e-300), np.maximum(mgn, 1e-300))
    n = min(vg.shape[0], mg.shape[0])
    pairs = []
    used_vg, used_mg = set(), set()
    for _ in range(n):
        best = None
        for i in range(vg.shape[0]):
            if i in used_vg:
                continue
            for j in range(mg.shape[0]):
                if j in used_mg:
                    continue
                if best is None or overlap[i, j] > overlap[best[0], best[1]]:
                    best = (i, j)
        used_vg.add(best[0])
        used_mg.add(best[1])
        pairs.append(best)
    pairs.sort()

    # Per-diagram complex ratio, with the median phase factored out as "global".
    ratios = {}
    for i, j in pairs:
        r = (mg[j].conj() @ vg[i]) / max(mg[j].conj() @ mg[j], 1e-300).real
        ratios[(i, j)] = r
    phases = np.array([np.angle(r) for r in ratios.values()])
    mags = np.array([np.abs(r) for r in ratios.values()])
    global_phase = np.median(phases)
    print(f"global phase (median arg r) = {np.degrees(global_phase):+.2f} deg")
    print(f" vg   mg   overlap    |r|        arg(r)-global [deg]   sig")
    for i, j in pairs:
        r = ratios[(i, j)]
        rel_phase = np.degrees(np.angle(r) - global_phase)
        rel_phase = (rel_phase + 180.0) % 360.0 - 180.0
        flag = "" if abs(np.abs(r) - 1) < 1e-6 and abs(rel_phase) < 1e-4 else "  <-- MISMATCH"
        print(
            f" {i:3}  {j:3}  {overlap[i, j]:.6f}  {np.abs(r):.6f}  {rel_phase:+10.4f}"
            f"        [{sigs[i]}]{flag}"
        )
    _ = mags


if __name__ == "__main__":
    main()
