#!/usr/bin/env python3
"""Dump MadGraph per-diagram amplitudes for u u~ > c c~ e+ e- mu+ mu- (QCD=0)
at CSV point 0 of uux_to_ccx_emmm_qcd0_amplitude.csv.

Writes output/mg_uux_amps_full.npy: complex array (NGRAPHS, 256) of AMP(i) per
helicity combo (itertools.product((-1,1), repeat=8), leg 0 slowest — same order
as the vibegraph probe helas::eval::run::tests::probe_uux_diagram_classes).

Also prints:
  * total |M|² = CF(1,1)=9 × Σ_hel |Σ_i (−1)·AMP_i|² (JAMP coeffs are uniformly
    −1), which must reproduce the CSV point-0 reference, and
  * the H-diagram class decomposition (AMP 113–115, 1-based: the three
    ZZH-ZZH diagrams — the only diagrams with a scalar propagator).

Run: pixi run -e madgraph python validation/madgraph/probe_uux_amp.py
"""

import itertools
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import mg_uux_amp_probe as M  # noqa: E402

CARD = os.path.join(HERE, "output", "uux_to_ccx_emmm_qcd0", "Cards", "param_card.dat")
CSV = os.path.join(HERE, "output", "uux_to_ccx_emmm_qcd0_amplitude.csv")
NGRAPHS = 579
NEXT = 8
CF_COLOR = 9.0
H_DIAGRAMS_1BASED = (113, 114, 115)

# CSV point 0: m2_ref + momenta, order [u,u~,c,c~,e+,e-,mu+,mu-].
with open(CSV) as f:
    rows = [
        line for line in f if line.strip() and not line.lstrip().startswith("#")
    ]
row0 = [float(x) for x in rows[1].split(",")]  # rows[0] is the column header
m2_ref = row0[0]
P = np.zeros((4, NEXT), dtype=np.float64, order="F")
for leg in range(NEXT):
    P[:, leg] = row0[1 + 4 * leg : 5 + 4 * leg]

hels = list(itertools.product((-1, 1), repeat=NEXT))
full = np.zeros((NGRAPHS, len(hels)), dtype=complex)

for hi, hel in enumerate(hels):
    nhel = np.array(hel, dtype=np.int32)
    full[:, hi] = np.asarray(M.mg_eval_amp(P, nhel, CARD))

out = os.path.join(HERE, "output", "mg_uux_amps_full.npy")
np.save(out, full)
print(f"wrote {out}  shape={full.shape}")

# JAMP(1,1) = -sum(AMP); |M|² = CF(1,1)·|JAMP|².
jamp = -full.sum(axis=0)
m2_total = CF_COLOR * float(np.sum(np.abs(jamp) ** 2))
print(f"MG total |M|²  = {m2_total:.10e}")
print(f"CSV point-0 ref = {m2_ref:.10e}   ratio = {m2_total / m2_ref:.6f}")

h_idx = [i - 1 for i in H_DIAGRAMS_1BASED]
r_idx = [i for i in range(NGRAPHS) if i not in h_idx]
th = full[h_idx].sum(axis=0)
tr = full[r_idx].sum(axis=0)
m2_h = CF_COLOR * float(np.sum(np.abs(th) ** 2))
m2_rest = CF_COLOR * float(np.sum(np.abs(tr) ** 2))
interf = CF_COLOR * float(np.sum(2.0 * (np.conj(th) * tr).real))
print("class decomposition (×9 color factor):")
print(f"  |H class|²        = {m2_h:.10e}")
print(f"  |rest|²           = {m2_rest:.10e}")
print(f"  2·Re interference = {interf:.10e}")
