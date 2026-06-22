#!/usr/bin/env python3
"""Match vibegraph diagrams to MadGraph diagrams by full-helicity-vector overlap,
then report the per-diagram complex ratio (magnitude + phase) to localize the
continuum relative-phase bug.

Each diagram is a 64-dim complex vector over helicities (massless τ → exact, no
basis ambiguity).  Two diagrams are "the same" if |<a_vg|a_mg>|/(||a_vg|| ||a_mg||)
≈ 1.  The matched complex ratio r = <a_mg|a_vg>/<a_mg|a_mg> gives the relative
magnitude (|r|) and phase (arg r); a correct diagram has r = +1.
"""
import os
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
mg = np.load(os.path.join(HERE, "output", "mg_amps_full.npy"))  # [25, 64]

# Load vibegraph: lines "idx\tsig\tre\tim\tre\tim..."
vg = np.zeros_like(mg)
sigs = [""] * mg.shape[0]
with open(os.path.join(HERE, "output", "vibegraph_amps_full.txt")) as f:
    for line in f:
        parts = line.rstrip("\n").split("\t")
        i = int(parts[0])
        sigs[i] = parts[1]
        vals = list(map(float, parts[2:]))
        vg[i] = np.array(vals[0::2]) + 1j * np.array(vals[1::2])

NG = mg.shape[0]

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
mgnorm = np.linalg.norm(mg, axis=1)
vgnorm = np.linalg.norm(vg, axis=1)

print(f"{'vg':>3} {'sig':<10} {'->mg':>4}  {'overlap':>8}  {'|ratio|':>9}  {'phase(deg)':>10}  "
      f"{'|a_vg|':>10} {'|a_mg|':>10}")
used = set()
for i in range(NG):
    if vgnorm[i] == 0:
        print(f"{i:3} {sigs[i]:<10} {'--':>4}   (zero amplitude)")
        continue
    # overlaps against all MG diagrams
    ov = np.abs(vg[i].conj() @ mg.T) / (vgnorm[i] * np.maximum(mgnorm, 1e-300))
    # j = int(np.argmax(ov))
    j = vg_to_mg.get(i)
    if j is None:
        print(f"{i:3} {sigs[i]:<10} {'--':>4}   (no mapping)")
        continue
    ratio = (mg[j].conj() @ vg[i]) / (mg[j].conj() @ mg[j])
    phase = np.degrees(np.angle(ratio))
    mark = "" if j not in used else "  <DUP>"
    used.add(j)
    print(f"{i:3} {sigs[i]:<10} {j:4d}  {ov[j]:8.5f}  {abs(ratio):9.5f}  {phase:+10.3f}  "
          f"{vgnorm[i]:.4e} {mgnorm[j]:.4e}{mark}")
