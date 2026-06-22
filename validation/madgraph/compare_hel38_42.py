#!/usr/bin/env python3
"""Compare VG vs MG per-diagram amplitudes for helicities 38 (correct) and 42 (wrong).

Identifies which specific diagrams have the helicity-dependent ratio error.
VG/MG ratio should be +i (or -i) for correct diagrams; deviations pinpoint the bug.
"""
import os
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))

# Load MG: [25, 64]
mg = np.load(os.path.join(HERE, "output", "mg_amps_full.npy"))
NGRAPHS_MG = mg.shape[0]  # 25

# Load VG: [n_vg, 64]
vg_list = {}
sigs = {}
with open(os.path.join(HERE, "output", "vibegraph_amps_full.txt")) as f:
    for line in f:
        parts = line.rstrip("\n").split("\t")
        i = int(parts[0])
        sigs[i] = parts[1]
        vals = list(map(float, parts[2:]))
        vg_list[i] = np.array(vals[0::2]) + 1j * np.array(vals[1::2])

n_vg = max(vg_list) + 1
vg = np.zeros((n_vg, 64), dtype=complex)
for i, v in vg_list.items():
    vg[i] = v

# Hand-built vibegraph to madgraph diagram mapping using madgraph matrix1.ps diagram visualization
vg_to_mg = {
    # VG names read from e- leg's vertex
     0: 9,  # [a+mu-+a]
     1: 11,  # [Z+mu-+a]
     2: 13,  # [a+mu-+Z]
     3: 15,  # [Z+mu-+Z]
     4: 10,  # [a+mu++a]
     5: 12,  # [Z+mu++a]
     6: 14,  # [a+mu++Z]
     7: 16,  # [Z+mu++Z]
     8: 0,  # [a+ta-+a]
     9: 2,  # [Z+ta-+a]
    10: 4,  # [a+ta-+Z]
    11: 6,  # [Z+ta-+Z]
    12: 1,  # [a+ta++a]
    13: 3,  # [Z+ta++a]
    14: 5,  # [a+ta++Z]
    15: 7,  # [Z+ta++Z]
    16: 8,  # [Z+H+Z]
    17: 17,  # [a+e++a]
    18: 19,  # [Z+e++a]
    19: 21,  # [a+e++Z]
    20: 23,  # [Z+e++Z]
    21: 18,  # [a+e++a]
    22: 22,  # [Z+e++a]
    23: 20,  # [a+e++Z]
    24: 24,  # [Z+e++Z]
}

# Helicities of interest
HEL_38 = 38  # (1,-1,-1,1,1,-1): CORRECT
HEL_42 = 42  # (1,-1,1,-1,1,-1): WRONG

import itertools
hels = list(itertools.product((-1, 1), repeat=6))
print(f"Hel {HEL_38} = {hels[HEL_38]}  [CORRECT: VG/MG ratio ≈ 1.000]")
print(f"Hel {HEL_42} = {hels[HEL_42]}  [WRONG:   VG/MG ratio ≈ 0.640]")
print()

# Per-diagram ratios at hel 38 and 42
print(f"{'VG':>3}  {'sig':<10}  {'->MG':>4}  "
      f"{'VG38':>20}  {'MG38':>20}  {'ratio38':>10}  "
      f"{'VG42':>20}  {'MG42':>20}  {'ratio42':>10}  {'r42/r38':>10}")

for i in range(n_vg):
    j = vg_to_mg.get(i)
    if j is None:
        print(f"{i:3}  {sigs[i]:<10}  {'--':>4}  (zero)")
        continue

    vg38 = vg[i, HEL_38]
    vg42 = vg[i, HEL_42]
    mg38 = mg[j, HEL_38]
    mg42 = mg[j, HEL_42]

    def ratio(a, b):
        if abs(b) < 1e-30:
            return float('nan')
        return a / b

    r38 = ratio(vg38, mg38)
    r42 = ratio(vg42, mg42)

    def fmt(c):
        if c != c:  # nan
            return "         nan"
        return f"{c.real:+.4f}{c.imag:+.4f}i"

    def fmtc(c):
        return f"({c.real:+.4e},{c.imag:+.4e})"

    rrel = ratio(r42, r38) if r38 == r38 and abs(r38) > 1e-10 else float('nan')
    print(f"{i:3}  {sigs[i]:<10}  {j:4d}  "
          f"{fmtc(vg38):>25}  {fmtc(mg38):>25}  {fmt(r38):>15}  "
          f"{fmtc(vg42):>25}  {fmtc(mg42):>25}  {fmt(r42):>15}  {fmt(rrel):>15}")

# Summary: check total amplitude ratios
tot_vg38 = vg[:, HEL_38].sum()
tot_mg38 = mg[:, HEL_38].sum()
tot_vg42 = vg[:, HEL_42].sum()
tot_mg42 = mg[:, HEL_42].sum()
print()
print(f"VG total hel38:  {tot_vg38:.6e}  |.| = {abs(tot_vg38):.6e}")
print(f"MG total hel38:  {tot_mg38:.6e}  |.| = {abs(tot_mg38):.6e}")
print(f"VG/MG hel38 ratio: {tot_vg38/tot_mg38:.6f}")
print(f"VG |M|²38 / MG |M|²38 = {abs(tot_vg38)**2 / abs(tot_mg38)**2:.6f}")
print()
print(f"VG total hel42:  {tot_vg42:.6e}  |.| = {abs(tot_vg42):.6e}")
print(f"MG total hel42:  {tot_mg42:.6e}  |.| = {abs(tot_mg42):.6e}")
print(f"VG/MG hel42 ratio: {tot_vg42/tot_mg42:.6f}")
print(f"VG |M|²42 / MG |M|²42 = {abs(tot_vg42)**2 / abs(tot_mg42)**2:.6f}")
