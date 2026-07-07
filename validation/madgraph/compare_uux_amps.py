#!/usr/bin/env python3
"""Match vibegraph ↔ MadGraph per-diagram amplitudes for u u~ > c c~ e+ e- mu+ mu-
(QCD=0, 579 diagrams, 256 helicity combos) at CSV point 0 and characterize the
per-diagram VG/MG ratios.

Inputs (regen recipes):
  output/mg_uux_amps_full.npy        — pixi run -e madgraph python validation/madgraph/probe_uux_amp.py
  output/vibegraph_uux_amps_full.txt — cargo test -p vibegraph-lib --release --lib \
        helas::eval::run::tests::probe_uux_diagram_classes -- --ignored --nocapture

Diagrams are matched by nearest log-magnitude helicity fingerprint (the two
sides enumerate diagrams in different orders).  For a correct diagram the
ratio vg/mg is a helicity-independent constant (the global convention phase);
helicity-dependent ratios mark broken diagrams.
"""

import os
from collections import defaultdict

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
MG_NPY = os.path.join(HERE, "output", "mg_uux_amps_full.npy")
VG_TXT = os.path.join(HERE, "output", "vibegraph_uux_amps_full.txt")

mg = np.load(MG_NPY)  # (579, 256), raw AMP(i) (JAMP coeff -1 NOT folded in)
vg_rows = []
sigs = []
with open(VG_TXT) as f:
    for line in f:
        parts = line.rstrip("\n").split("\t")
        sigs.append(parts[1])
        vals = np.array([float(x) for x in parts[2:]])
        vg_rows.append(vals[0::2] + 1j * vals[1::2])
vg = np.array(vg_rows)  # (579, 256)
assert vg.shape == mg.shape, (vg.shape, mg.shape)
n, nhel = vg.shape

# --- match diagrams by log-|amp| fingerprint --------------------------------
FLOOR = 1e-300


def fp(a):
    return np.log10(np.abs(a) + FLOOR)


fvg = fp(vg)
fmg = fp(mg)
# cost[i,j] = mean squared log-magnitude distance
cost = ((fvg[:, None, :] - fmg[None, :, :]) ** 2).mean(axis=2)
try:
    from scipy.optimize import linear_sum_assignment

    ri, ci = linear_sum_assignment(cost)
    match = dict(zip(ri, ci))
except ImportError:
    match = {}
    used = set()
    for i in np.argsort(cost.min(axis=1)):
        j = min((j for j in range(n) if j not in used), key=lambda j: cost[i, j])
        match[i] = j
        used.add(j)

match_cost = np.array([cost[i, match[i]] for i in range(n)])
print(
    f"matched {n} diagrams; fingerprint cost: median {np.median(match_cost):.3e}, "
    f"max {match_cost.max():.3e}"
)

# --- per-diagram ratio characterization --------------------------------------
print(
    "\nper-diagram vg/mg ratio (over hels with |mg| > 1e-8·max_hel|mg|):\n"
    "  ratio0 = |mg|²-weighted mean ratio; spread = max |ratio-ratio0|/|ratio0|"
)
clusters = defaultdict(list)  # rounded ratio -> [diagram indices]
bad = []
for i in range(n):
    j = match[i]
    a, b = vg[i], mg[j]
    mask = np.abs(b) > 1e-8 * np.abs(b).max()
    r = a[mask] / b[mask]
    w = np.abs(b[mask]) ** 2
    r0 = np.sum(r * w) / np.sum(w)
    spread = np.max(np.abs(r - r0) / abs(r0)) if abs(r0) > 0 else np.inf
    key = (round(r0.real, 3), round(r0.imag, 3), spread < 1e-6)
    clusters[key].append(i)
    if spread >= 1e-6:
        bad.append((i, j, r0, spread))

print(f"\nratio clusters (ratio0.re, ratio0.im, hel-independent?):")
for key in sorted(clusters, key=lambda k: -len(clusters[k])):
    idxs = clusters[key]
    sig_counts = defaultdict(int)
    for i in idxs:
        sig_counts[sigs[i]] += 1
    top = sorted(sig_counts.items(), key=lambda kv: -kv[1])[:4]
    print(f"  {key}: {len(idxs)} diagrams   e.g. {top}")

print(f"\n{len(bad)} diagrams with helicity-DEPENDENT ratio; worst 20 by spread:")
for i, j, r0, spread in sorted(bad, key=lambda t: -t[3])[:20]:
    print(
        f"  vg {i:3d} <-> mg {j:3d}  ratio0 = {r0.real:+.4f}{r0.imag:+.4f}i  "
        f"spread = {spread:.3e}  [{sigs[i]}]"
    )

# --- coherent totals ----------------------------------------------------------
vg_tot = vg.sum(axis=0)
mg_tot = mg.sum(axis=0)
m2_vg = 9.0 * float(np.sum(np.abs(vg_tot) ** 2))
m2_mg = 9.0 * float(np.sum(np.abs(mg_tot) ** 2))
print(f"\ncoherent |M|²: vg = {m2_vg:.6e}  mg = {m2_mg:.6e}  vg/mg = {m2_vg/m2_mg:.4f}")
