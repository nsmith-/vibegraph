#!/usr/bin/env python3
"""MadGraph's own |M|^2 for 1 -> n decays at fixed rest-frame points.

For each decay below, writes MadGraph standalone output, compiles its
`SMATRIX` against a small driver, and evaluates it at points drawn here: the
mother at rest, the products built by nested two-body splits whose invariant
masses are drawn over their whole range and, half the time, from a
Breit-Wigner around the resonance the diagram carries, so the table samples
both the peak and the tails. `SMATRIX` returns the helicity- and colour-summed
|M|^2 divided by `IDEN`, the mother's spin and colour states times the final
state's identical-particle factor.

Writes `decay_amplitudes.json`, read by the hermetic
`vibegraph-lib/tests/decay_widths.rs`.

Usage: pixi run -e madgraph python validation/madgraph/gen_decay_amplitudes.py
"""

import json
import math
import os
import random
import re
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))
MG5_ROOT = os.path.join(REPO_ROOT, "research", "refs", "mg5amcnlo")
OUT = os.path.join(HERE, "decay_amplitudes.json")
POINTS = 24

DRIVER = """      PROGRAM DRV
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      REAL*8 P(0:3,NEXTERNAL), ANS
      INTEGER I,J,N
      CALL SETPARA('../../Cards/param_card.dat')
      READ(*,*) N
      DO J=1,N
        DO I=1,NEXTERNAL
          READ(*,*) P(0,I),P(1,I),P(2,I),P(3,I)
        ENDDO
        CALL SMATRIX(P,ANS)
        WRITE(*,'(E26.17)') ANS
      ENDDO
      END
"""

# (process, mother mass, daughter masses, split tree, resonance (mass, width) of
# the tree's first composite node). A tree is a nested pair of leg indices
# (0-based over the products).
MT, MB, MW, WW, MZ, WZ, MH = 173.0, 4.7, 80.419002445756163, 2.047600, 91.188, 2.441404, 125.0
CASES = [
    ("t > b e+ ve", MT, [MB, 0.0, 0.0], (0, (1, 2)), (MW, WW)),
    ("h > e+ e- mu+ mu-", MH, [0.0] * 4, ((0, 1), (2, 3)), (MZ, WZ)),
    ("z > e+ e- mu+ mu-", MZ, [0.0] * 4, ((0, 1), (2, 3)), (MZ, WZ)),
    ("t > b e+ ve a", MT, [MB, 0.0, 0.0, 0.0], ((0, 3), (1, 2)), (MW, WW)),
]


def boost(p, b):
    bx, by, bz = b
    b2 = bx * bx + by * by + bz * bz
    if b2 == 0.0:
        return p
    g = 1.0 / math.sqrt(1.0 - b2)
    bp = bx * p[1] + by * p[2] + bz * p[3]
    g2 = (g - 1.0) / b2
    return [
        g * (p[0] + bp),
        p[1] + g2 * bp * bx + g * bx * p[0],
        p[2] + g2 * bp * by + g * by * p[0],
        p[3] + g2 * bp * bz + g * bz * p[0],
    ]


def leaves(tree):
    return [tree] if isinstance(tree, int) else leaves(tree[0]) + leaves(tree[1])


def mass_floor(tree, masses):
    return sum(masses[i] for i in leaves(tree))


def split(p, m, tree, masses, rng, resonance, out, first=True):
    """Decay `p` (mass `m`) into the two branches of `tree`."""
    if isinstance(tree, int):
        out[tree] = p
        return
    left, right = tree
    floors = [mass_floor(left, masses), mass_floor(right, masses)]
    ms = []
    for k, branch in enumerate((left, right)):
        if isinstance(branch, int):
            ms.append(masses[branch])
            continue
        hi = m - (ms[0] if k == 1 else floors[1])
        lo = floors[k]
        mr, wr = resonance
        if rng.random() < 0.5 and lo < mr < hi:
            while True:
                x = mr + wr * math.tan(math.pi * (rng.random() - 0.5)) / 2.0
                if lo < x < hi:
                    break
        else:
            x = lo + (hi - lo) * rng.random()
        ms.append(x)
    m1, m2 = ms
    lam = (m * m - (m1 + m2) ** 2) * (m * m - (m1 - m2) ** 2)
    q = math.sqrt(max(lam, 0.0)) / (2.0 * m)
    c = 2.0 * rng.random() - 1.0
    s = math.sqrt(1.0 - c * c)
    phi = 2.0 * math.pi * rng.random()
    d = [s * math.cos(phi), s * math.sin(phi), c]
    b = [p[i + 1] / p[0] for i in range(3)]
    for mass, sign, branch in ((m1, 1.0, left), (m2, -1.0, right)):
        k = [math.sqrt(q * q + mass * mass)] + [sign * q * x for x in d]
        split(boost(k, b), mass, branch, masses, rng, resonance, out, False)


def main():
    rng = random.Random(20260925)
    rows = []
    work = tempfile.mkdtemp(prefix="vg-decay-amp-")
    for process, mother, masses, tree, resonance in CASES:
        outdir = os.path.join(work, re.sub(r"[^a-z0-9]+", "_", process))
        script = os.path.join(work, "gen.mg5")
        with open(script, "w") as fh:
            fh.write(f"import model sm\ngenerate {process}\noutput standalone {outdir}\n")
        subprocess.run(["bash", os.path.join(HERE, "mg5_pinned.sh"), script], check=True,
                       stdout=subprocess.DEVNULL)
        sub = [d for d in os.listdir(os.path.join(outdir, "SubProcesses")) if d.startswith("P1_")]
        pdir = os.path.join(outdir, "SubProcesses", sub[0])
        with open(os.path.join(pdir, "drv.f"), "w") as fh:
            fh.write(DRIVER)
        subprocess.run(["make", "matrix.o"], cwd=pdir, check=True, stdout=subprocess.DEVNULL)
        for lib in ("DHELAS", "MODEL"):
            subprocess.run(["make", "-C", os.path.join(outdir, "Source", lib)], check=True,
                           stdout=subprocess.DEVNULL)
        subprocess.run(["gfortran", "-O", "-o", "drv", "drv.f", "matrix.o", "-L../../lib",
                        "-ldhelas", "-lmodel"], cwd=pdir, check=True)
        points = []
        for _ in range(POINTS):
            out = [None] * len(masses)
            split([mother, 0.0, 0.0, 0.0], mother, tree, masses, rng, resonance, out)
            points.append([[mother, 0.0, 0.0, 0.0]] + out)
        stdin = f"{len(points)}\n" + "".join(
            "".join("%.17e %.17e %.17e %.17e\n" % tuple(p) for p in point) for point in points
        )
        res = subprocess.run(["./drv"], cwd=pdir, input=stdin, capture_output=True, text=True,
                             check=True)
        values = [float(x.replace("D", "E")) for x in res.stdout.split()]
        assert len(values) == len(points)
        rows.append({"process": process, "points": points, "smatrix": values})
        print(f"{process}: {len(points)} points")

    version = re.search(r"version\s*=\s*(\S+)", open(os.path.join(MG5_ROOT, "VERSION")).read()).group(1)
    with open(OUT, "w") as fh:
        json.dump(
            {
                "_comment": "MadGraph standalone SMATRIX (|M|^2 summed over helicities and "
                "colours, divided by IDEN) for 1 -> n decays on `import model sm`, at rest-frame "
                "points, externals in process order, momenta [E, px, py, pz]. Generated by "
                "validation/madgraph/gen_decay_amplitudes.py.",
                "mg_version": version,
                "cases": rows,
            },
            fh,
            indent=1,
        )
    print("wrote", OUT)


if __name__ == "__main__":
    main()
