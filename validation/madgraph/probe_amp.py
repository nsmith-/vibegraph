#!/usr/bin/env python3
"""Dump MadGraph per-diagram amplitudes for e+e->mumu tata at CSV point 0.

Computes the basis-independent helicity-summed quantities used to localize the
uux/eemumutata continuum relative-phase bug:
  diag[i]  = sum_hel |AMP_i|^2            (per-diagram magnitude; basis-indep)
  Rrow[i]  = sum_hel conj(AMP_i)*A_total  (diagram i's contribution to |M|^2;
                                           sum_i Re(Rrow[i]) = |M|^2; basis-indep)
A_total = sum_i AMP_i.  Writes results to output/mg_amp_probe.txt and prints
the total |M|^2 (must equal the CSV point-0 reference).
"""
import os
import sys
import itertools
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import mg_ee_amp_probe as M

CARD = os.path.join(
    HERE, "output", "ee_to_mumu_tata_qcd0", "Cards", "param_card_masslesstau.dat"
)
NGRAPHS = 25
NEXT = 6
# Single helicity for the complex per-diagram dump (massless: e+/e- opposite, etc.)
HEL0 = (-1, 1, -1, 1, -1, 1)  # [e+,e-,mu+,mu-,ta+,ta-]

# CSV point 0 momenta (E,px,py,pz) per leg, order [e+,e-,mu+,mu-,ta+,ta-].
MOM = [
    (250.0, 0.0, 0.0, 250.0),
    (250.0, 0.0, 0.0, -250.0),
    (130.98844490914234, -106.66561232781022, -0.9379201403415187, -76.02328690775641),
    (167.2530959714149, 134.2336665209957, -62.607066356179416, -77.68703963098595),
    (94.5533499515044, -18.39281604525598, -22.219961151047247, 90.04617499607066),
    (107.2051091679384, -9.175238147929512, 85.76494764756818, 63.66415154267164),
]

P = np.zeros((4, NEXT), dtype=np.float64, order="F")
for leg, m in enumerate(MOM):
    P[:, leg] = m

diag = np.zeros(NGRAPHS)              # sum_hel |AMP_i|^2
Rrow = np.zeros(NGRAPHS, dtype=complex)  # sum_hel conj(AMP_i)*A_total
m2_total = 0.0
amp_hel0 = None                       # per-diagram complex AMP at HEL0
hels = list(itertools.product((-1, 1), repeat=NEXT))
full = np.zeros((NGRAPHS, len(hels)), dtype=complex)  # [diagram, helicity]

for hi, hel in enumerate(hels):
    nhel = np.array(hel, dtype=np.int32)
    amp = np.asarray(M.mg_eval_amp(P, nhel, CARD))  # COMPLEX*16 (25,)
    full[:, hi] = amp
    a_tot = amp.sum()
    m2_total += abs(a_tot) ** 2
    diag += np.abs(amp) ** 2
    Rrow += np.conj(amp) * a_tot
    if tuple(hel) == HEL0:
        amp_hel0 = amp.copy()

np.save(os.path.join(HERE, "output", "mg_amps_full.npy"), full)
print(f"saved MG full amp array {full.shape} to output/mg_amps_full.npy")

print(f"MG total |M|^2 (sum_hel |sum_i AMP_i|^2) = {m2_total:.10e}")
print("(CSV point-0 reference m2_summed         = 1.1519918572120465e-10)")
print(f"check sum_i Re(Rrow) = {Rrow.real.sum():.10e}")

print(f"Specific helicity {HEL0} per-diagram complex amplitudes:")
for i, amp in enumerate(amp_hel0):
    print(f"Diagram {i:02}  {amp.real:+.8e} {amp.imag:+.8e} (diag[{i}] = {diag[i]:.8e})")

with open(os.path.join(HERE, "output", "mg_amp_probe.txt"), "w") as f:
    f.write(f"# MG per-diagram, point0. total_m2 = {m2_total:.10e}\n")
    f.write("# idx  diag_mag(sum|a|^2)   Re(Rrow)            Im(Rrow)\n")
    for i in range(NGRAPHS):
        f.write(f"{i+1:3d}  {diag[i]:.8e}  {Rrow[i].real:+.8e}  {Rrow[i].imag:+.8e}\n")
print("wrote output/mg_amp_probe.txt")

# Sorted-by-magnitude view for matching against vibegraph.
order = np.argsort(-diag)
print("\n  MG diagrams sorted by magnitude:")
print("  rank  mgidx  diag_mag        Re(Rrow)")
for rank, i in enumerate(order):
    print(f"  {rank:3d}   {i+1:3d}   {diag[i]:.6e}  {Rrow[i].real:+.6e}")
