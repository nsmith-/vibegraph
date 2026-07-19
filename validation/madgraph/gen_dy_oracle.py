#!/usr/bin/env python3
"""Pointwise integrand oracle for the hadronic Drell-Yan cross section (H7).

Independently recomputes vibegraph's `DrellYanIntegrand` factors at a handful of
pinned kinematic points, from:
  * LHAPDF `xfxQ2` for the PDF luminosity (NNPDF23_lo_as_0130_qed, member 0),
  * MadGraph's standalone matrix elements for |M|^2 (mg_dy_probe: MATRIX1 = u u~,
    MATRIX2 = d d~, from the built p p > e+ e- subprocess),
  * plain arithmetic for the flux prefactor, the (tau, y) Jacobian, and the
    lab-frame lepton cuts,
and writes dy_integrand_oracle.json for validate_hadronic's Rust oracle test to
pin vibegraph against at <= 1e-9 on each factor.

The points are addressed in VEGAS coordinates u in [0,1]^3 (so the Rust side can
call `debug_factors(u)` directly); they are chosen from physical
(sqrt_shat, y_parton, cos_theta) triples, including two points straddling the
pt_l = 10 GeV cut boundary.

Prerequisites:
  pixi run -e madgraph fetch-pdf
  pixi run -e madgraph generate-hadronic-sigma   # builds output/dy13_default
  # build the |M|^2 probe module (once):
  (cd output/dy13_default/SubProcesses/P1_qq_ll && \
   python -m numpy.f2py -c --f77flags='-fallow-argument-mismatch -ffixed-line-length-132 -I.' \
     matrix1_optim.f matrix2_optim.f ../../../../wrappers/dy_probe.f \
     -L../../lib -lmodel -ldhelas -m mg_dy_probe && mv mg_dy_probe*.so ../../../../)

Usage: pixi run -e madgraph generate-dy-oracle
"""

import json
import math
import os
import shutil
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import lhapdf  # noqa: E402
import mg_dy_probe  # noqa: E402

# Conventions shared verbatim with vibegraph's hadronic module and the run card.
MU_F = 91.1880
SQRT_S_HAD = 13000.0
S_HAD = SQRT_S_HAD**2
PDF_SET = "NNPDF23_lo_as_0130_qed"
GEV2_TO_PB = 3.893793721e8
UP_FLAVORS = (2, 4)
DOWN_FLAVORS = (1, 3)
# Default-cut lower shat: max((2*ptl)^2, ...) = (2*10)^2 = 400 GeV^2.
PTL = 10.0
ETAL = 2.5
SHAT_MIN = (2.0 * PTL) ** 2
TAU_MIN = SHAT_MIN / S_HAD

PARAM_CARD = os.path.join(HERE, "output", "dy13_default", "Cards", "param_card.dat")
# Committed copy so the Rust oracle test binds the identical param card.
PARAM_CARD_COMMIT = os.path.join(HERE, "dy13_param_card.dat")


def u_from_physical(sqrt_shat, y_parton, cos_theta):
    """VEGAS coordinates u for a physical (sqrt_shat, y_parton, cos_theta)."""
    tau = sqrt_shat**2 / S_HAD
    u1 = 1.0 - math.log(tau) / math.log(TAU_MIN)
    y_max = -0.5 * math.log(tau)
    u2 = 0.5 * (y_parton / y_max + 1.0)
    u3 = 0.5 * (cos_theta + 1.0)
    return [u1, u2, u3]


def map_point(u):
    """Mirror DrellYanIntegrand::map_point exactly."""
    tau = TAU_MIN ** (1.0 - u[0])
    sqrt_tau = math.sqrt(tau)
    y_max = -0.5 * math.log(tau)
    y = (2.0 * u[1] - 1.0) * y_max
    jac = math.log(1.0 / TAU_MIN) * 2.0 * y_max
    return {
        "x1": sqrt_tau * math.exp(y),
        "x2": sqrt_tau * math.exp(-y),
        "sqrt_shat": math.sqrt(tau * S_HAD),
        "cos_theta": 2.0 * u[2] - 1.0,
        "jac": jac,
    }


def cm_and_lab(sqrt_shat, cos_theta, x1, x2):
    """CM momenta [q, qbar, e+, e-] and the z-boosted lab momenta (mirror of
    hadronic::build_kinematics)."""
    half = sqrt_shat / 2.0
    sin_t = math.sqrt(max(0.0, 1.0 - cos_theta**2))
    q = [half, 0.0, 0.0, half]
    qbar = [half, 0.0, 0.0, -half]
    ep = [half, half * sin_t, 0.0, half * cos_theta]
    em = [half, -half * sin_t, 0.0, -half * cos_theta]

    beta = (x1 - x2) / (x1 + x2)
    gamma = 1.0 / math.sqrt(1.0 - beta * beta)

    def boost_z(p):
        return [gamma * (p[0] + beta * p[3]), p[1], p[2], gamma * (p[3] + beta * p[0])]

    lab = [
        [x1 * SQRT_S_HAD / 2.0, 0.0, 0.0, x1 * SQRT_S_HAD / 2.0],
        [x2 * SQRT_S_HAD / 2.0, 0.0, 0.0, -x2 * SQRT_S_HAD / 2.0],
        boost_z(ep),
        boost_z(em),
    ]
    return [q, qbar, ep, em], lab


def rapidity(p):
    e, pz = p[0], p[3]
    if e <= abs(pz):
        return -1e99
    return 0.5 * math.log((e + pz) / (e - pz))


def pt(p):
    return math.hypot(p[1], p[2])


def passes_cuts(lab):
    """Default DY lepton cuts (pt_l >= 10, |y_l| <= 2.5) on the two lab-frame
    leptons; drll never fires for the back-to-back pair (delta-phi = pi)."""
    for p in lab[2:]:
        if pt(p) < PTL:  # strict: pt == PTL passes
            return False
        if abs(rapidity(p)) > ETAL:
            return False
    return True


def mg_m2(cm):
    p = np.zeros((4, 4), order="F")
    for leg in range(4):
        p[:, leg] = cm[leg]
    up, down = mg_dy_probe.dy_m2(p, PARAM_CARD)
    return float(up), float(down)


def luminosity(pdf, flavors, x1, x2):
    acc = 0.0
    q2 = MU_F**2
    for q in flavors:
        fq1 = pdf.xfxQ2(q, x1, q2)
        fq2 = pdf.xfxQ2(q, x2, q2)
        fqb1 = pdf.xfxQ2(-q, x1, q2)
        fqb2 = pdf.xfxQ2(-q, x2, q2)
        acc += fq1 * fqb2 + fqb1 * fq2
    return acc


def prefactor2(sqrt_shat):
    return 1.0 / (64.0 * math.pi * sqrt_shat**2)


def main():
    pdf = lhapdf.mkPDF(PDF_SET, 0)
    shutil.copyfile(PARAM_CARD, PARAM_CARD_COMMIT)

    # (sqrt_shat, y_parton, cos_theta) probe points. The pt-boundary pair sits at
    # sqrt_shat = 91.19, cos_theta = +-(pt_l = 10) i.e. sin_theta = 20/91.19; one
    # just inside, one just outside.
    ss_z = 91.19
    sin_edge = 2.0 * PTL / ss_z
    cos_edge = math.sqrt(1.0 - sin_edge**2)  # pt_l = 10 exactly
    physical = [
        (91.19, 0.0, 0.3),       # Z peak, central
        (91.19, 0.8, -0.4),      # Z peak, boosted, backward
        (60.0, 0.5, 0.6),        # low edge of the Z window
        (150.0, -1.2, 0.1),      # above the Z peak
        (250.0, 1.5, -0.7),      # high mass, forward parton
        (40.0, 0.0, 0.0),        # Drell-Yan continuum, central
        (500.0, -0.3, 0.5),      # far tail
        (91.19, 2.0, 0.2),       # large parton rapidity
        (ss_z, 0.0, cos_edge - 0.002),  # pt_l just above 10 -> passes
        (ss_z, 0.0, cos_edge + 0.002),  # pt_l just below 10 -> fails
    ]

    points = []
    for sqrt_shat, y_parton, cos_theta in physical:
        u = u_from_physical(sqrt_shat, y_parton, cos_theta)
        m = map_point(u)
        cm, lab = cm_and_lab(m["sqrt_shat"], m["cos_theta"], m["x1"], m["x2"])
        m2_up, m2_down = mg_m2(cm)
        lum_up = luminosity(pdf, UP_FLAVORS, m["x1"], m["x2"])
        lum_down = luminosity(pdf, DOWN_FLAVORS, m["x1"], m["x2"])
        phat = prefactor2(m["sqrt_shat"]) / 9.0
        passed = passes_cuts(lab)
        value = m["jac"] * phat * (m2_up * lum_up + m2_down * lum_down) if passed else 0.0
        points.append(
            {
                "u": u,
                "x1": m["x1"],
                "x2": m["x2"],
                "cos_theta": m["cos_theta"],
                "sqrt_shat": m["sqrt_shat"],
                "lum_up": lum_up,
                "lum_down": lum_down,
                "m2_up": m2_up,
                "m2_down": m2_down,
                "phat": phat,
                "jac": m["jac"],
                "pass": passed,
                "value": value,
            }
        )

    doc = {
        "_comment": "Pointwise DY integrand oracle for validate_hadronic. "
        "LHAPDF xfxQ2 (NNPDF23_lo_as_0130_qed member 0) for luminosity, MadGraph "
        "standalone MATRIX1/MATRIX2 for |M|^2, arithmetic for flux/Jacobian/cuts. "
        "value is in natural units (GeV^-2). Bind vibegraph with dy13_param_card.dat.",
        "mu_f": MU_F,
        "sqrt_s_had": SQRT_S_HAD,
        "shat_min": SHAT_MIN,
        "tau_min": TAU_MIN,
        "pdf_set": PDF_SET,
        "param_card": "dy13_param_card.dat",
        "points": points,
    }
    out = os.path.join(HERE, "dy_integrand_oracle.json")
    with open(out, "w") as f:
        json.dump(doc, f, indent=2)
        f.write("\n")
    print(f"wrote {len(points)} oracle points -> {out}")
    for p in points:
        print(
            f"  sqrt_shat={p['sqrt_shat']:7.2f} cos={p['cos_theta']:+.4f} "
            f"pass={p['pass']!s:5} m2_up={p['m2_up']:.4e} m2_down={p['m2_down']:.4e} "
            f"lum_up={p['lum_up']:.4e} value={p['value']:.4e}"
        )


if __name__ == "__main__":
    main()
