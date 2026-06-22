#!/usr/bin/env python3
"""Dump MadGraph intermediate wavefunctions for e+e->mumu tata at a specific helicity.

Calls MG_EVAL_WFUNCS (in ee_amp_probe.f) to retrieve:
  WF slot 1: W7  = gamma current from e-spine  (FFV1P0_3)
  WF slot 2: W8  = gamma current from mu-spine (FFV1P0_3)
  WF slot 3: W10 = Z current from e-spine      (FFV2_4_3)
  WF slot 4: W9  = off-shell ta- after Z abs.  (FFV2_4_2)
  WF slot 5: W11 = Z current from mu-spine     (FFV2_4_3)

Chosen helicity: hel 42 (0-indexed from product((-1,1),repeat=6) with e+ slowest).
hel 42 = 0b101010 → (e+:+1, e-:-1, mu+:+1, mu-:-1, ta+:+1, ta-:-1)
This is one of the "wrong" helicities where VG/MG ratio ≈ 0.64.

Also prints MG's per-diagram AMP array for the same helicity.
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

# Momenta from CSV point 0 (E,px,py,pz) for [e+, e-, mu+, mu-, ta+, ta-]
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

# Helicity order: product((-1,+1), repeat=6) with leg0 (e+) varying slowest
hels = list(itertools.product((-1, 1), repeat=NEXT))
WF_NAMES = [
    "W7  = gamma[e-spine]  (FFV1P0_3(e-, e+))",
    "W8  = gamma[mu-spine] (FFV1P0_3(mu-, mu+))",
    "W10 = Z[e-spine]      (FFV2_4_3(e-, e+))",
    "W9  = ta-_offshell_Z  (FFV2_4_2(ta-, Z_e))",
    "W11 = Z[mu-spine]     (FFV2_4_3(mu-, mu+))",
    "e*  = off-shell e+ <- gamma[mu] (FFV1_1)   -> AMP(18) CORRECT",
    "e*  = off-shell e+ <- Z[mu]     (FFV2_4_1) -> AMP(22) 0.64@hel42",
    "W12 = gamma[ta] sink  (FFV1P0_3(ta-, ta+))",
]

def print_wf(name, wf):
    print(f"  {name}")
    for i, c in enumerate(wf):
        print(f"    [{i}] {c.real:+.8e} + {c.imag:+.8e}i")

# --- hel 38 (a correct helicity) ---
HEL_IDX_CORRECT = 38
HEL_IDX_WRONG   = 42

for hel_idx, label in [(HEL_IDX_CORRECT, "CORRECT"), (HEL_IDX_WRONG, "WRONG")]:
    hel = hels[hel_idx]
    nhel = np.array(hel, dtype=np.int32)
    print(f"\n{'='*70}")
    print(f"Helicity {hel_idx} ({label}): {hel}  [e+, e-, mu+, mu-, ta+, ta-]")
    print('='*70)

    wf_out, ext_out, amp_out = M.mg_eval_wfuncs(P, nhel, CARD)
    # wf_out shape: (6, 5) — column-major (Fortran), so wf_out[:, slot] is one wf
    # ext_out shape: (6, 6)

    EXT_NAMES = [
        "W1 = e+  (OXXXXX nsf=-1, incoming antiparticle)",
        "W2 = e-  (IXXXXX nsf=+1, incoming particle)",
        "W3 = mu- (IXXXXX nsf=-1, outgoing particle)",
        "W4 = mu+ (OXXXXX nsf=+1, outgoing antiparticle)",
        "W5 = ta- (IXXXXX nsf=-1, outgoing particle, massive)",
        "W6 = ta+ (OXXXXX nsf=+1, outgoing antiparticle, massive)",
    ]

    print("\nExternal wavefunctions:")
    for slot, name in enumerate(EXT_NAMES):
        wf = ext_out[:, slot]
        print_wf(name, wf)

    print("\nIntermediate wavefunctions:")
    for slot, name in enumerate(WF_NAMES):
        wf = wf_out[:, slot]
        print_wf(name, wf)

    print("\nPer-diagram MG amplitudes:")
    for i, a in enumerate(amp_out):
        if abs(a) > 1e-15:
            print(f"  AMP({i+1:2d}) = {a.real:+.8e} + {a.imag:+.8e}i  |a|={abs(a):.6e}")

    total = amp_out.sum()
    print(f"\n  Sum AMP = {total.real:+.8e} + {total.imag:+.8e}i")
    print(f"  |Sum|^2 = {abs(total)**2:.8e}")

    # --- Controlled e-spine contrast: same topology, mu-side boson gamma vs Z ---
    # AMP(18): e+ -gamma[mu]- e* -gamma[ta]- e-   (all photon, matches VG)
    # AMP(22): e+ -  Z[mu] - e* -gamma[ta]- e-    (one Z[mu], 0.64 vs VG @ hel42)
    e_gam = wf_out[:, 5]   # off-shell e+ after gamma[mu]
    e_z   = wf_out[:, 6]   # off-shell e+ after Z[mu]
    a18, a22 = amp_out[17], amp_out[21]
    print("\n  e-spine gamma-vs-Z contrast (only the mu-side boson differs):")
    print("    off-shell e+ components [2..6] (physical spinor):")
    for k in range(2, 6):
        print(f"      [{k}] e*<-gamma {e_gam[k].real:+.6e}{e_gam[k].imag:+.6e}i"
              f"   e*<-Z {e_z[k].real:+.6e}{e_z[k].imag:+.6e}i")
    print(f"    AMP(18) e*<-gamma sink: {a18.real:+.6e}{a18.imag:+.6e}i")
    print(f"    AMP(22) e*<-Z     sink: {a22.real:+.6e}{a22.imag:+.6e}i")
    if abs(a18) > 1e-30:
        print(f"    AMP22/AMP18 = {(a22/a18).real:+.5f}{(a22/a18).imag:+.5f}i  "
              f"|.|={abs(a22/a18):.5f}")

# Save the wavefunctions to a numpy file for Rust comparison
print("\n--- Saving wavefunction data for Rust comparison ---")
data = {}
for hel_idx, label in [(HEL_IDX_CORRECT, "correct"), (HEL_IDX_WRONG, "wrong")]:
    hel = hels[hel_idx]
    nhel = np.array(hel, dtype=np.int32)
    wf_out, ext_out, amp_out = M.mg_eval_wfuncs(P, nhel, CARD)
    data[f"hel{hel_idx}_{label}_wfuncs"] = wf_out.copy()
    data[f"hel{hel_idx}_{label}_ext"]    = ext_out.copy()
    data[f"hel{hel_idx}_{label}_amps"]   = amp_out.copy()
    data[f"hel{hel_idx}_{label}_nhel"]   = nhel.copy()

data["momenta"] = P.copy()
out_path = os.path.join(HERE, "output", "mg_wfuncs_debug.npz")
np.savez(out_path, **data)
print(f"Saved to {out_path}")
print("Keys:", list(data.keys()))
