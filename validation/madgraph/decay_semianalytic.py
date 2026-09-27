#!/usr/bin/env python3
"""The h > e+ e- mu+ mu- partial width by one-dimensional integrals.

With the leptons massless and no photon coupling to the Higgs, the decay's one
diagram factorises exactly into the two off-shell Z lines:

    Γ = ∫ dm1² dm2² ρ(m1²) ρ(m2²) Γ(H → V(m1) V(m2)),

    ρ(m²) = (1/π) m Γ_ll(m) / ((m² − M_Z²)² + M_Z² Γ_Z²),

where Γ_ll(m) is the width of a vector of mass m into a massless lepton pair
(the SM UFO's `decays.py` Z > e- e+ expression at mass m) and Γ(H → V V) the
two-body width into vectors of masses m1 and m2 through the HZZ vertex, with the
off-shell polarization sum −g + qq/q² that a conserved lepton current leaves.
Both pieces are checked against `decays.py` at points where they are on shell.
The double integral is done by Gauss–Legendre quadrature in the Breit–Wigner
angle of each line, the outer one split around m1 = M_H − M_Z, where the inner
line's peak leaves its range; it is converged to 1e-9.

Parameters are MadGraph's model_reader on restrict_default.dat. Writes
`decay_width_exact.json`, which the width gate compares both MadEvent and this
crate against.

Usage: pixi run -e madgraph python validation/madgraph/decay_semianalytic.py
"""

import cmath
import io
import json
import math
import os
import re
import sys
from contextlib import redirect_stdout

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))
MG5_ROOT = os.path.join(REPO_ROOT, "research", "refs", "mg5amcnlo")
OUT = os.path.join(HERE, "decay_width_exact.json")
sys.path.insert(0, MG5_ROOT)

import models.import_ufo as import_ufo  # noqa: E402
import models.model_reader as model_reader  # noqa: E402

H_ZZ = (
    "(((9*ee**4*vev**2)/2. + (3*ee**4*MH**4*vev**2)/(8.*MZ**4) - (3*ee**4*MH**2*vev**2)/(2.*MZ**2)"
    " + (3*cw**4*ee**4*vev**2)/(4.*sw**4) + (cw**4*ee**4*MH**4*vev**2)/(16.*MZ**4*sw**4)"
    " - (cw**4*ee**4*MH**2*vev**2)/(4.*MZ**2*sw**4) + (3*cw**2*ee**4*vev**2)/sw**2"
    " + (cw**2*ee**4*MH**4*vev**2)/(4.*MZ**4*sw**2) - (cw**2*ee**4*MH**2*vev**2)/(MZ**2*sw**2)"
    " + (3*ee**4*sw**2*vev**2)/cw**2 + (ee**4*MH**4*sw**2*vev**2)/(4.*cw**2*MZ**4)"
    " - (ee**4*MH**2*sw**2*vev**2)/(cw**2*MZ**2) + (3*ee**4*sw**4*vev**2)/(4.*cw**4)"
    " + (ee**4*MH**4*sw**4*vev**2)/(16.*cw**4*MZ**4) - (ee**4*MH**2*sw**4*vev**2)/(4.*cw**4*MZ**2))"
    "*cmath.sqrt(MH**4 - 4*MH**2*MZ**2))/(32.*cmath.pi*abs(MH)**3)"
)
Z_EE = (
    "((-5*ee**2*Me**2 - ee**2*MZ**2 - (cw**2*ee**2*Me**2)/(2.*sw**2) + (cw**2*ee**2*MZ**2)/(2.*sw**2)"
    " + (7*ee**2*Me**2*sw**2)/(2.*cw**2) + (5*ee**2*MZ**2*sw**2)/(2.*cw**2))"
    "*cmath.sqrt(-4*Me**2*MZ**2 + MZ**4))/(48.*cmath.pi*abs(MZ)**3)"
)


def main():
    card = os.path.join(MG5_ROOT, "models", "sm", "restrict_default.dat")
    with redirect_stdout(io.StringIO()):
        model = import_ufo.import_model("sm", restrict=False)
        reader = model_reader.ModelReader(model)
        reader.set_parameters_and_couplings(card)
    P = {
        (k[4:] if k.startswith("mdl_") else k): complex(v).real
        for k, v in reader.get("parameter_dict").items()
    }
    ee, sw, cw, vev, MZ, WZ, MH = (P[k] for k in ("ee", "sw", "cw", "vev", "MZ", "WZ", "MH"))
    env = {"cmath": cmath, "abs": abs}

    g = ee**2 * vev / (2 * sw**2 * cw**2)
    c_ll = (-ee**2 + cw**2 * ee**2 / (2 * sw**2) + 5 * ee**2 * sw**2 / (2 * cw**2)) / (48 * math.pi)

    def gamma_hvv(mh, m1, m2):
        lam = (mh * mh - (m1 + m2) ** 2) * (mh * mh - (m1 - m2) ** 2)
        if lam <= 0:
            return 0.0
        p = math.sqrt(lam) / (2 * mh)
        p1p2 = (mh * mh - m1 * m1 - m2 * m2) / 2
        return g * g * p / (8 * math.pi * mh * mh) * (2 + p1p2**2 / (m1 * m1 * m2 * m2))

    # The two pieces against decays.py where it applies: H > Z Z on shell at a
    # Higgs mass above threshold (carrying its identical-particle 1/2), and
    # Z > e- e+ with the electron massless.
    check_hzz = eval(H_ZZ, env, dict(P, MH=250.0)).real
    assert abs(0.5 * gamma_hvv(250.0, MZ, MZ) / check_hzz - 1) < 1e-12
    check_zee = eval(Z_EE, env, dict(P, Me=0.0)).real
    assert abs(c_ll * MZ / check_zee - 1) < 1e-12

    a = MZ * WZ

    def theta(m2):
        return math.atan((m2 - MZ * MZ) / a)

    def mass(t):
        m2 = MZ * MZ + a * math.tan(t)
        return math.sqrt(m2) if m2 > 0 else None

    def inner(m1, nodes):
        x, w = nodes
        lo, hi = theta(0.0), theta((MH - m1) ** 2)
        total = 0.0
        for xj, wj in zip(x, w):
            m2 = mass(0.5 * (hi - lo) * xj + 0.5 * (hi + lo))
            if m2:
                total += wj * c_ll * m2 * m2 / (math.pi * a) * gamma_hvv(MH, m1, m2)
        return total * 0.5 * (hi - lo)

    def width(n_outer, n_inner):
        outer = np.polynomial.legendre.leggauss(n_outer)
        nodes = np.polynomial.legendre.leggauss(n_inner)
        edge = MH - MZ
        cuts = sorted(
            {0.0, (MH / 2) ** 2, MH * MH}
            | {max(edge + k * WZ, 0.0) ** 2 for k in (-20, -5, -1, 0, 1, 5, 20)}
        )
        total = 0.0
        for lo, hi in zip(cuts[:-1], cuts[1:]):
            tlo, thi = theta(lo), theta(hi)
            for xi, wi in zip(*outer):
                m1 = mass(0.5 * (thi - tlo) * xi + 0.5 * (thi + tlo))
                if m1:
                    rho = c_ll * m1 * m1 / (math.pi * a)
                    total += 0.5 * (thi - tlo) * wi * rho * inner(m1, nodes)
        return total

    coarse, fine = width(100, 400), width(200, 800)
    assert abs(coarse / fine - 1) < 1e-8, (coarse, fine)
    print(f"h > e+ e- mu+ mu-: {fine:.10e} GeV (coarse grid {coarse:.10e})")

    version = re.search(r"version\s*=\s*(\S+)", open(os.path.join(MG5_ROOT, "VERSION")).read()).group(1)
    with open(OUT, "w") as fh:
        json.dump(
            {
                "_comment": "Partial widths (GeV) known to quadrature accuracy on `import model "
                "sm`: h > e+ e- mu+ mu- factorised into its two off-shell Z lines. Generated by "
                "validation/madgraph/decay_semianalytic.py.",
                "mg_version": version,
                "widths": {"h > e+ e- mu+ mu-": fine},
                "quadrature_rel": abs(coarse / fine - 1),
            },
            fh,
            indent=2,
        )
    print("wrote", OUT)


if __name__ == "__main__":
    main()
