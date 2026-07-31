#!/usr/bin/env python3
"""Bank MadGraph's per-diagram `AMP()` values as a committed reference for the
Rust per-diagram amplitude oracle (`vibegraph-lib/tests/amp_diagram_oracle.rs`).

Why a dedicated reference and not the |M|² CSV: `validate_helas_mg` compares one
real number per phase-space point, the colour-contracted helicity sum. That
number is blind to a relative sign or phase between diagrams whenever the
interference it moves is small, blind to a permutation of the helicity
assignment that leaves the sum invariant, and blind to a per-diagram error
compensated by a colour coefficient. The finest linear object MadGraph exposes
for a single-flow process is `AMP(1:NGRAPHS)` per helicity, so that is what gets
banked, complex and element-wise, alongside the coherent `JAMP(1)` the colour
coefficients build out of it.

Only NCOLOR = 1 processes are banked. For a multi-flow process the coherent
object per colour flow is `JAMP(1:NCOLOR)`, already banked by
`gen_jamp_reference.py`, and a diagram root there is not a scalar.

Output: `amp_reference.json`, committed. Momenta come from each process's
already-generated `output/<name>_amplitude.csv` so the reference sits on the same
phase-space points `validate_helas_mg` uses.

Usage:
  pixi run -e madgraph generate-amp-reference

Prerequisites:
  pixi run -e madgraph build-amplitude     # builds mg_amp_probe_<name>
  pixi run -e madgraph generate-amplitude  # writes <name>_amplitude.csv
"""

import importlib
import json
import os
import re
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(HERE, "output")
# The compiled mg_*.so matrix elements are build products in the work area.
sys.path.insert(0, os.path.join(HERE, "output", "f2py"))

# NCOLOR = 1 processes with a compiled `mg_amp_probe_<name>` module and a diagram
# count small enough that the two sides' diagram orderings can be compared one
# for one. The first three are already-enforced controls, one per convention
# channel the rooting signs distinguish (crossed-line spine, Yang-Mills VVV,
# scalar bilinear); the rest are the `p p > l+ l- j` subprocess representatives.
PROCESSES = [
    "ee_to_ee",
    "ee_to_wpwm",
    "ee_to_tatah",
    "uux_to_epemg",
    "ddx_to_epemg",
    "gu_to_epemu",
    "gux_to_epemux",
]

N_POINTS = 5


def read_csv_points(name, n_points):
    """(process_str, n_ext, [momenta]) from the amplitude CSV's first rows."""
    with open(os.path.join(OUTPUT_DIR, f"{name}_amplitude.csv")) as f:
        lines = [l for l in f if l.strip()]
    process_str = next(
        l.split(":", 1)[1].strip() for l in lines if l.startswith("# process:")
    )
    n_ext = next(int(l.split(":")[1]) for l in lines if l.startswith("# n_ext:"))
    data = [l for l in lines if not l.startswith("#")][1:]  # skip column header
    points = []
    for row in data[:n_points]:
        vals = [float(x) for x in row.split(",")]
        p = np.zeros((4, n_ext), dtype=np.float64, order="F")
        for leg in range(n_ext):
            p[:, leg] = vals[1 + 4 * leg : 5 + 4 * leg]
        points.append(p)
    return process_str, n_ext, points


def matrix_file(name):
    """The process's single SubProcesses/P*/matrix1_orig.f."""
    sub = os.path.join(OUTPUT_DIR, name, "SubProcesses")
    dirs = [d for d in sorted(os.listdir(sub)) if d.startswith("P")]
    assert len(dirs) == 1, f"{name}: expected one P* subprocess dir, got {dirs}"
    return os.path.join(sub, dirs[0], "matrix1_orig.f")


def read_dimensions(name):
    """(NGRAPHS, NCOLOR, [flow structure labels]) from matrix1_orig.f."""
    with open(matrix_file(name)) as f:
        src = f.read()
    ngraphs = int(re.search(r"NGRAPHS=(\d+)", src).group(1))
    ncolor = int(re.search(r"NCOLOR=(\d+)", src).group(1))
    structures = re.findall(r"^C\s+\d+ (T\(.*|Tr\(.*|1)\s*$", src, re.MULTILINE)
    return ngraphs, ncolor, structures[:ncolor]


def join_continuations(src):
    """Fold Fortran fixed-form continuation lines into their statement."""
    return re.sub(r"\n     [^ ]", "", src)


def read_jamp_coefficients(name, ngraphs):
    """The colour coefficients c_i of `JAMP(1,1) = Σ_i c_i AMP(i)`, as complex pairs.

    These are the weights that turn the per-diagram amplitudes into the coherent
    single-flow amplitude, and they are where MadGraph puts the relative sign
    between an annihilation and an exchange diagram — vibegraph puts that sign in
    the diagram root instead, so the comparable object is the product c_i·AMP(i)
    rather than AMP(i) alone. Verified numerically against the probe's own
    JAMP(1) at every banked point, so a mis-parse cannot pass.
    """
    with open(matrix_file(name)) as f:
        src = join_continuations(f.read())
    stmt = re.search(r"JAMP\(1,1\)\s*=(.*)", src).group(1).replace(" ", "")
    coeffs = [0j] * ngraphs
    seen = set()
    for sign, coef, idx in re.findall(r"([+-]?)(?:\(([^)]*)\)\*)?AMP\((\d+)\)", stmt):
        i = int(idx) - 1
        assert i not in seen, f"{name}: AMP({idx}) appears twice in JAMP(1,1)"
        seen.add(i)
        if coef:
            parts = [float(p.replace("D", "E")) for p in coef.split(",")]
            value = complex(parts[0], parts[1] if len(parts) > 1 else 0.0)
        else:
            value = 1.0 + 0j
        coeffs[i] = -value if sign == "-" else value
    assert len(seen) == ngraphs, f"{name}: JAMP(1,1) covers {len(seen)}/{ngraphs} graphs"
    return coeffs


def helicity_combos(name, n_ext):
    """MadGraph's own NHEL table, in its own row order.

    Read from matrix1_orig.f's `DATA (NHEL(I,N),I=1,n_ext) / ... /` block rather
    than enumerated as a product, so the banked helicity set is MadGraph's and a
    disagreement with vibegraph's enumeration is a finding rather than something
    the reference quietly conforms to.
    """
    with open(matrix_file(name)) as f:
        src = join_continuations(f.read())
    combos = []
    for row in re.findall(r"DATA \(NHEL\(I,\s*\d+\),I=1,\s*\d+\) /([^/]*)/", src):
        vals = [int(v) for v in row.replace(" ", "").split(",")]
        assert len(vals) == n_ext, f"{name}: NHEL row {vals} is not {n_ext} long"
        combos.append(vals)
    assert combos, f"{name}: no NHEL rows found in {matrix_file(name)}"
    return combos


def main():
    entries = {}
    for name in PROCESSES:
        process_str, n_ext, points = read_csv_points(name, N_POINTS)
        ngraphs, ncolor, structures = read_dimensions(name)
        if ncolor != 1:
            sys.exit(f"{name}: NCOLOR={ncolor}; this reference banks single-flow only")
        combos = helicity_combos(name, n_ext)
        coeffs = read_jamp_coefficients(name, ngraphs)
        card = os.path.join(OUTPUT_DIR, name, "Cards", "param_card.dat")
        module = importlib.import_module(f"mg_amp_probe_{name}")

        banked = []
        for p in points:
            amps, jamps = [], []
            for hel in combos:
                amp, jamp = module.mg_eval_amp(p, np.array(hel, dtype=np.int32), card)
                amp, jamp = np.asarray(amp), np.asarray(jamp)
                rebuilt = sum(c * a for c, a in zip(coeffs, amp))
                assert abs(rebuilt - jamp[0]) <= 1e-10 * max(abs(jamp[0]), 1e-30), (
                    f"{name}: parsed JAMP coefficients {coeffs} do not rebuild "
                    f"JAMP(1)={jamp[0]} from AMP() at helicity {hel}"
                )
                amps.append([[float(z.real), float(z.imag)] for z in amp])
                jamps.append([[float(z.real), float(z.imag)] for z in jamp])
            banked.append(
                {
                    "momenta": [[float(x) for x in p[:, leg]] for leg in range(n_ext)],
                    "amps": amps,
                    "jamps": jamps,
                }
            )

        entries[name] = {
            "process": process_str,
            "n_graphs": ngraphs,
            "n_flows": ncolor,
            "flow_structures": structures,
            "jamp_coefficients": [[c.real, c.imag] for c in coeffs],
            "helicities": combos,
            "points": banked,
        }
        nonzero = sum(
            1
            for pt in banked
            for row in pt["amps"]
            for z in row
            if z[0] != 0.0 or z[1] != 0.0
        )
        print(
            f"[{name}] {process_str}: {ngraphs} graphs, {len(combos)} helicity "
            f"combos, {len(points)} points, {nonzero} non-zero AMP entries"
        )

    out = os.path.join(HERE, "amp_reference.json")
    with open(out, "w") as f:
        json.dump(
            {
                "_comment": "MadGraph per-diagram AMP() and coherent JAMP(1) for the "
                "single-flow amplitude-gate processes; generated by "
                "gen_amp_reference.py, consumed by tests/amp_diagram_oracle.rs",
                "processes": entries,
            },
            f,
            indent=1,
        )
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
