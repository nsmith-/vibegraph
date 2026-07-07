#!/usr/bin/env python3
"""Full per-diagram x per-helicity VG/MG ratio table for ee->mumutata.

The decisive oracle for the continuum chirality bug: prints VG/MG for all 25
diagrams at every helicity config where MG is nonzero. A correct eval gives
magnitude 1.000 in every cell with the SAME phase everywhere (the global
convention, -i since the chain-phase normalization; cells render 1.000* with
the phase flagged). Chiral-coupling errors show up as signed powers of
gL/gR = 1.25 / gR/gL = 0.80 keyed to the helicity config of the lepton pair
each Z vertex attaches to; chain-phase errors show up as per-diagram-class
phase splits (cf. compare_uux_amps.py).

IMPORTANT - matched parameters: the MG reference (mg_amps_full.npy, regenerate
with probe_amp.py) has MTA = ymtau = 1.777 hardcoded at process-generation time
(Source/MODEL/param_read.inc is a static INCLUDE of ../param_card.inc; SETPARA
ignores the card path passed at runtime). The VG dump therefore must be
generated with the default massive-tau card:

  VG_PARAM_CARD=param_card_default.dat cargo test -p vibegraph-lib \
      --features extended-validation \
      --lib helas::eval::run::tests::probe_eemumutata_diagrams -- --ignored

A '*' marks ratios with a significant phase (mass-mixed tau entries).
"""
import os
import itertools
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
mg = np.load(os.path.join(HERE, "output", "mg_amps_full.npy"))

vg_list, sigs = {}, {}
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

# Hand-verified VG->MG diagram map (see compare_hel38_42.py)
vg_to_mg = {0: 9, 1: 11, 2: 13, 3: 15, 4: 10, 5: 12, 6: 14, 7: 16, 8: 0, 9: 2,
            10: 4, 11: 6, 12: 1, 13: 3, 14: 5, 15: 7, 16: 8, 17: 17, 18: 19,
            19: 21, 20: 23, 21: 18, 22: 22, 23: 20, 24: 24}

hels = list(itertools.product((-1, 1), repeat=6))
mg_m2 = np.abs(mg.sum(axis=0)) ** 2
nonzero = [h for h in range(64) if mg_m2[h] > 1e-20 * mg_m2.max()]

print("legend cols: (mu+,mu-,ta+,ta-) helicities; e pair (-1,1) for h<32 else (1,-1)")
print(f"{'VG':>3} {'sig':<10} " + "  ".join(f"h{h:<7}" for h in nonzero))
print(f"{'':>3} {'':<10} " + "  ".join(f"{str(hels[h][2:]):8.8}" for h in nonzero))
for i in range(n_vg):
    j = vg_to_mg[i]
    cells = []
    for h in nonzero:
        a, b = vg[i, h], mg[j, h]
        if abs(b) < 1e-30:
            cells.append("   --   " if abs(a) < 1e-30 else " VGonly ")
        else:
            r = a / b
            if abs(r.imag) < 0.02 * max(abs(r.real), 1e-12):
                cells.append(f"{r.real:+8.3f}")
            else:
                cells.append(f"{abs(r):7.3f}*")
    print(f"{i:>3} {sigs[i]:<10} " + "  ".join(cells))

vg_m2 = np.abs(vg.sum(axis=0)) ** 2
print(f"\ntotal |M|^2 VG/MG = {vg_m2.sum() / mg_m2.sum():.5f}   "
      f"nonzero hels: VG={np.sum(vg_m2 > 1e-20 * vg_m2.max())} MG={len(nonzero)}")
