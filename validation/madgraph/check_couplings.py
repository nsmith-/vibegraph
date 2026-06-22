#!/usr/bin/env python3
"""Check SM coupling values for the Z-muon vertex."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

CARD = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "output", "ee_to_mumu_tata_qcd0", "Cards", "param_card_masslesstau.dat"
)

import mg_ee_amp_probe as M
import numpy as np
import itertools

# Use dummy momenta to trigger SETPARA
MOM = [
    (250.0, 0.0, 0.0, 250.0),
    (250.0, 0.0, 0.0, -250.0),
    (130.99, -106.67, -0.938, -76.02),
    (167.25, 134.23, -62.61, -77.69),
    (94.55, -18.39, -22.22, 90.05),
    (107.21, -9.18, 85.76, 63.66),
]
P = np.zeros((4, 6), dtype=np.float64, order="F")
for leg, m in enumerate(MOM):
    P[:, leg] = m
hels = list(itertools.product((-1, 1), repeat=6))
nhel = np.array(hels[38], dtype=np.int32)
wf_out, ext_out, amp_out = M.mg_eval_wfuncs(P, nhel, CARD)

# Now access the parameter block (coupl.inc values) from the compiled module
# The matrix1_func.f links against libmodel which contains SETPARA
# SETPARA sets GC_3, GC_50, GC_59 etc. from param_card

# Let's read the coupling values from the probe directly by examining the
# Z current ratio between helicities.
# The Z current FFV2_4_3(fi=mu+, fo=mu-) at hel 38 vs 42 should differ by gL/gR.

# W11 = Z current from mu-spine at hel 38 and 42:
nhel38 = np.array(hels[38], dtype=np.int32)
nhel42 = np.array(hels[42], dtype=np.int32)

wf38, _, _ = M.mg_eval_wfuncs(P, nhel38, CARD)
wf42, _, _ = M.mg_eval_wfuncs(P, nhel42, CARD)

W11_38 = wf38[:, 4]  # Z current from mu-spine, hel 38
W11_42 = wf42[:, 4]  # Z current from mu-spine, hel 42
W8_38 = wf38[:, 1]   # gamma current from mu-spine, hel 38
W8_42 = wf42[:, 1]   # gamma current from mu-spine, hel 42

print("W11 (Z from mu-spine) at hel 38:")
for i, c in enumerate(W11_38):
    print(f"  [{i}] {c.real:+.6e} + {c.imag:+.6e}i")

print("W11 (Z from mu-spine) at hel 42:")
for i, c in enumerate(W11_42):
    print(f"  [{i}] {c.real:+.6e} + {c.imag:+.6e}i")

print("\nW8 (photon from mu-spine) at hel 38:")
for i, c in enumerate(W8_38):
    print(f"  [{i}] {c.real:+.6e} + {c.imag:+.6e}i")

print("\nW8 (photon from mu-spine) at hel 42:")
for i, c in enumerate(W8_42):
    print(f"  [{i}] {c.real:+.6e} + {c.imag:+.6e}i")

# The transverse components (indices 2-5 in W) carry the helicity-dependent information.
# For the Z current: at hel 38, dominated by gR; at hel 42, dominated by gL.
# Ratio of transverse Z components:
print("\nTransverse ratio W11_42[2] / W11_38[2]:", W11_42[2] / W11_38[2])
print("Transverse ratio W11_42[3] / W11_38[3]:", W11_42[3] / W11_38[3])
print("Transverse ratio W8_42[2] / W8_38[2]:", W8_42[2] / W8_38[2])
print("Transverse ratio W8_42[3] / W8_38[3]:", W8_42[3] / W8_38[3])

# The ratio W11_42/W11_38 for transverse components should be gL/gR (or gR/gL)
# while W8_42/W8_38 should be -1 (pure helicity flip).
print("\n|W11_42[2]/W11_38[2]| =", abs(W11_42[2] / W11_38[2]))
print("|W8_42[2]/W8_38[2]| =", abs(W8_42[2] / W8_38[2]))

# Transverse part magnitude ratio:
Z_trans_38 = np.linalg.norm(W11_38[2:])
Z_trans_42 = np.linalg.norm(W11_42[2:])
g_trans_38 = np.linalg.norm(W8_38[2:])
g_trans_42 = np.linalg.norm(W8_42[2:])
print(f"\n|Z_trans_38| = {Z_trans_38:.6e},  |Z_trans_42| = {Z_trans_42:.6e}")
print(f"|Z_trans_42| / |Z_trans_38| = {Z_trans_42/Z_trans_38:.6f}  <- gL/gR?")
print(f"|g_trans_38| = {g_trans_38:.6e},  |g_trans_42| = {g_trans_42:.6e}")
print(f"|g_trans_42| / |g_trans_38| = {g_trans_42/g_trans_38:.6f}  <- should be ~1")

# What is gL/gR numerically?
# gL = |GC_50 + GC_59|, gR = |2*GC_59|
# From the Z current: W11 = FFV2_4_3(fi=W3, fo=W4, GC_50, GC_59, MZ, WZ)
# The coupling GC_50 gives gL contribution, GC_59 gives additional right-handed.
# The actual formulas from ALOHA:
#   ffv2_4_3 = GC_50 * (bilinear_L / propagator) + GC_59 * (bilinear_L + 2*bilinear_R) / propagator
# where bilinear_L = ū gamma P_L v, bilinear_R = ū gamma P_R v

# So W11 ∝ (GC_50 + GC_59) * bilinear_L + 2*GC_59 * bilinear_R

# At hel 38: bilinear_L = 0, so W11 ∝ 2*GC_59 * bilinear_R → proportional to gR
# At hel 42: bilinear_R = 0, so W11 ∝ (GC_50 + GC_59) * bilinear_L → proportional to gL
# And |bilinear_L(hel42)| = |bilinear_R(hel38)| by parity.
# Therefore: |W11_42| / |W11_38| = |gL/gR| = |(GC_50+GC_59)| / |2*GC_59|

# And W8 ∝ bilinear (same L and R, no chirality weighting) → |W8_42|/|W8_38| = 1
print(f"\nExpected |gL/gR| = |Z_trans_42|/|Z_trans_38| = {Z_trans_42/Z_trans_38:.6f}")
print(f"Observed r42/r38 error factor = 0.6402 (from compare_hel38_42.py)")

# If gL/gR is the ratio of the Z currents, then gR/gL is the inverse.
print(f"\n|gR/gL| = {Z_trans_38/Z_trans_42:.6f}")
