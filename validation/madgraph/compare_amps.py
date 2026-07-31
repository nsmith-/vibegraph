#!/usr/bin/env python3
"""Generic per-diagram / per-flow MadGraph-vs-vibegraph amplitude comparison.

Reads vibegraph's [diagram x helicity] AMP dump (written by the Rust probe
`helas::eval::run::tests::probe_process_diagrams` with VG_PROBE_NAME=<name>),
evaluates MadGraph's per-diagram AMP() and per-flow JAMP() (module
mg_amp_probe_<name>, built by build_amplitude.sh) at exactly the same
phase-space point (CSV point 0) and helicity combos, auto-matches rows by
full-helicity-vector overlap, and reports the per-row complex ratio
r = <a_mg|a_vg>/<a_mg|a_mg>: a correct row has |r| = 1 and phase 0 (up to one
global sign/phase, factored out and reported separately).

For a multi-flow process the same matcher is run over the per-flow JAMPs
(vibegraph_jamps_<name>.txt, written by the probe for NCOLOR > 1) against
MadGraph's JAMP(1:NCOLOR). Because both sides order flows by the color basis's
sorted keys, the (vg_flow -> mg_flow) pairing should be the identity, and the
matcher takes the identity whenever it fits at least as well as any other
pairing — see greedy_match for why overlap alone cannot decide.

This script is a diagnosis tool, not a gate. The enforcing per-flow JAMP oracle
is vibegraph-lib/tests/color_jamp_oracle.rs, which compares against banked
MadGraph JAMP values element-wise under the identity pairing at a fixed
tolerance; use this script when that oracle fails and you need the per-diagram
breakdown behind it.

Usage:
  pixi run -e madgraph python validation/madgraph/compare_amps.py <name>

Prerequisites (in order):
  pixi run -e madgraph build-amplitude          # builds mg_amp_probe_<name>
  pixi run -e madgraph generate-amplitude       # writes <name>_amplitude.csv
  VG_PROBE_NAME=<name> cargo test -p vibegraph-lib --features extended-validation \
      --lib helas::eval::run::tests::probe_process_diagrams -- --ignored --nocapture
"""

import argparse
import importlib
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
# The compiled mg_*.so matrix elements are build products in the work area.
sys.path.insert(0, os.path.join(HERE, "output", "f2py"))


def read_vg_dump(path):
    """Load a vibegraph [row x helicity] complex dump. Returns (rows, sigs, combos)
    or None if the file is absent."""
    if not os.path.exists(path):
        return None
    with open(path) as f:
        header = f.readline().rstrip("\n").split("\t")
        assert header[0] == "#hel", "run the Rust probe first (writes the #hel header)"
        combos = [[int(h) for h in c.split(",")] for c in header[1:]]
        sigs, rows = [], []
        for line in f:
            parts = line.rstrip("\n").split("\t")
            sigs.append(parts[1])
            vals = np.array(list(map(float, parts[2:])))
            rows.append(vals[0::2] + 1j * vals[1::2])
    return np.array(rows), sigs, combos


def greedy_match(vg, mg):
    """Greedy 1-1 matching of vg rows to mg rows by normalized overlap. Returns
    (pairs sorted by vg index, overlap matrix).

    Overlap alone does not determine a pairing when the rows are linearly
    dependent, and for colour flows they routinely are: the tree-level four-gluon
    amplitude is MHV, so every colour-ordered partial carries the same helicity
    dependence (Parke-Taylor) and `g g > g g`'s [flow x helicity] JAMP matrix is
    rank 1. Every pair of rows then has overlap exactly 1 and greedy max-overlap
    picks a pairing arbitrarily, reporting a spurious permutation and spurious
    |r| values. Since the expected answer is the identity (both sides order flows
    by the colour basis's sorted keys), the identity pairing wins any tie; a
    reported permutation then means the identity genuinely fits worse."""
    vgn = np.linalg.norm(vg, axis=1)
    mgn = np.linalg.norm(mg, axis=1)
    overlap = np.abs(vg.conj() @ mg.T) / np.outer(
        np.maximum(vgn, 1e-300), np.maximum(mgn, 1e-300)
    )
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
    if vg.shape[0] == mg.shape[0]:
        identity = [(i, i) for i in range(vg.shape[0])]
        score = lambda ps: sum(overlap[i, j] for i, j in ps)
        if score(identity) >= score(pairs) - 1e-9:
            pairs = identity
    return pairs, overlap


def report_match(label, vg, mg, sigs, name, strict_phase):
    """Auto-match vg rows to mg rows and print per-row |r| / phase(r), with the
    median phase factored out as one global phase.

    A unit |r| means the row's magnitude is reproduced. `strict_phase` controls
    whether a per-row phase left over *after* the global phase counts as a
    failure: JAMPs are physical (a per-flow relative phase changes off-diagonal
    interference in |M|²) so they run strict; raw per-diagram AMPs are not (MG's
    AMP() excludes the fermion-permutation / i-placement signs vibegraph bakes
    into its diagram roots, and those are reabsorbed into the color coefficients
    at the JAMP level), so their per-diagram phase is reported but not failed."""
    print(f"\n[{name}] {label}: vibegraph rows {vg.shape[0]}, MadGraph rows {mg.shape[0]}")
    pairs, overlap = greedy_match(vg, mg)

    ratios = {}
    for i, j in pairs:
        r = (mg[j].conj() @ vg[i]) / max((mg[j].conj() @ mg[j]).real, 1e-300)
        ratios[(i, j)] = r
    phases = np.array([np.angle(r) for r in ratios.values()])
    global_phase = np.median(phases)
    identity = all(i == j for i, j in pairs)
    # Row counts differ only for the per-diagram AMP comparison, where MadGraph
    # splits a multi-structure vertex across several AMP() slots vibegraph keeps
    # in one diagram root (the 4-gluon vertex's three colour structures). The
    # numbering cannot line up there, so a permutation is expected, not a finding.
    same_shape = vg.shape[0] == mg.shape[0]
    print(f" global phase (median arg r) = {np.degrees(global_phase):+.2f} deg")
    print(f" flow/diagram map is identity: {identity}"
          + ("" if identity or not same_shape else "   <-- PERMUTATION")
          + ("" if same_shape else "   (row counts differ; numbering cannot align)"))
    print(f" vg   mg   overlap    |r|        arg(r)-global [deg]   label")
    mag_ok = True
    phase_ok = True
    for i, j in pairs:
        r = ratios[(i, j)]
        rel_phase = np.degrees(np.angle(r) - global_phase)
        rel_phase = (rel_phase + 180.0) % 360.0 - 180.0
        this_mag = abs(np.abs(r) - 1) < 1e-6
        this_phase = abs(rel_phase) < 1e-4
        mag_ok = mag_ok and this_mag
        phase_ok = phase_ok and this_phase
        if not this_mag:
            flag = "  <-- MAGNITUDE MISMATCH"
        elif not this_phase:
            flag = "  <-- phase diff" + ("  <-- MISMATCH" if strict_phase else " (convention)")
        else:
            flag = ""
        print(
            f" {i:3}  {j:3}  {overlap[i, j]:.6f}  {np.abs(r):.6f}  {rel_phase:+10.4f}"
            f"        [{sigs[i]}]{flag}"
        )
    ok = (identity or not same_shape) and mag_ok and (phase_ok or not strict_phase)
    note = ""
    if ok and not phase_ok:
        note = " (per-diagram phase conventions differ; absorbed at JAMP level)"
    print(f" {label}: {'MATCH' + note if ok else 'DISCREPANCY (see above)'}")
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name", help="process name, e.g. ee_to_ee")
    args = ap.parse_args()
    name = args.name

    vg_amp = read_vg_dump(os.path.join(HERE, "output", f"vibegraph_amps_{name}.txt"))
    if vg_amp is None:
        sys.exit(f"missing vibegraph_amps_{name}.txt — run the Rust probe first")
    vg, sigs, combos = vg_amp
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

    # mg_eval_amp returns (AMP(NGRAPHS), JAMP(NCOLOR)) per helicity combo.
    mg_amp_rows, mg_jamp_rows = [], []
    for hel in combos:
        nhel = np.array(hel, dtype=np.int32)
        out = module.mg_eval_amp(p, nhel, card)
        amp, jamp = out if isinstance(out, tuple) else (out, None)
        mg_amp_rows.append(np.asarray(amp))
        if jamp is not None:
            mg_jamp_rows.append(np.asarray(jamp))
    mg = np.array(mg_amp_rows).T  # [n_mg, n_hel]

    m2_mg = float(np.sum(np.abs(mg.sum(axis=0)) ** 2))
    print(f"CSV point-0 |M|² ref = {m2_ref:.10e}")
    print(
        f"MG Σ|ΣAMP|² (no CF) = {m2_mg:.10e}   (JAMP coefficients / CF NOT applied)"
    )

    amp_ok = report_match("per-diagram AMP", vg, mg, sigs, name, strict_phase=False)

    # Per-flow JAMPs are compared only for genuinely multi-flow processes: the
    # Rust probe writes vibegraph_jamps_<name>.txt only when NCOLOR > 1 (a
    # single-flow amplitude has no Op::Flows root — its lone JAMP is just the
    # coherent diagram sum already covered above). MadGraph's probe always
    # returns a JAMP(1:NCOLOR), so a missing vibegraph dump is expected for
    # NCOLOR = 1 and is not a warning.
    jamp_ok = True
    vg_jamp = read_vg_dump(os.path.join(HERE, "output", f"vibegraph_jamps_{name}.txt"))
    if vg_jamp is not None and mg_jamp_rows:
        vgj, jsigs, _ = vg_jamp
        mgj = np.array(mg_jamp_rows).T  # [n_color, n_hel]
        jamp_ok = report_match("per-flow JAMP", vgj, mgj, jsigs, name, strict_phase=True)
    elif vg_jamp is not None and not mg_jamp_rows:
        print("\nWARNING: vibegraph dumped JAMPs but the MadGraph probe returned "
              "none — rebuild the probe (build_amplitude.sh) so its JAMP_OUT is current")
        jamp_ok = False

    sys.exit(0 if amp_ok and jamp_ok else 1)


if __name__ == "__main__":
    main()
