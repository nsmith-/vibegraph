#!/usr/bin/env python3
"""Generate the parton oracle JSON for the pinned LHAPDF6 set.

Runs the `parton` package (github.com/DavidMStraub/parton) against the
NNPDF23_lo_as_0130_qed member-0 grid fetched by fetch.sh, and dumps
(pdg, x, Q^2, x*f) tuples covering:

  - "knot"           exact grid values, read directly from each subgrid's raw
                      xfgrid array (no spline evaluation) -- the ground truth
                      for the Rust parser's on-disk parsing.
  - "off_knot"        interior points strictly between x/Q knots, evaluated
                      through parton's RectBivariateSpline interpolator.
  - "seam"            points at each subgrid's Q boundary. NNPDF23_lo_as_0130_qed
                      ships as a *single* subgrid (see the "single_subgrid"
                      note in the output), so these reduce to the grid's global
                      QMin/QMax rather than an internal flavor-threshold seam;
                      still useful as an edge-of-validity check for H2.
  - "x_to_one_tail"   x close to the x=1 boundary.
  - "corner"          the four (XMin/XMax, QMin/QMax) grid corners.

Only "knot" is consumed by H1's Rust gate; the rest are banked for H2's
interpolation oracle.

Usage:
  python validation/pdf/gen_oracle.py
  pixi run -e pdf-validation generate-pdf-oracle

Prerequisites:
  pixi run -e pdf-validation fetch-pdf
"""

import json
import os
import sys

import numpy as np
from parton.pdf import PDF

SET_NAME = "NNPDF23_lo_as_0130_qed"
MEMBER = 0
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_PATH = os.path.join(SCRIPT_DIR, "oracle.json")


def knot_points(pdf):
    """Exact on-knot tuples read directly from each subgrid's raw array."""
    points = []
    for grid in pdf.pdfgrids:
        nx, nq = len(grid.x), len(grid.Q)
        # A bounded subsample of knots (corners + a few interior indices),
        # not the full nx*nq*nf product.
        ix_samples = sorted(set([0, nx // 3, 2 * nx // 3, nx - 1]))
        iq_samples = sorted(set([0, nq // 2, nq - 1]))
        for ix in ix_samples:
            for iq in iq_samples:
                row = ix * nq + iq
                for ifl, pdg in enumerate(grid.flavors):
                    points.append(
                        {
                            "category": "knot",
                            "pdg": int(pdg),
                            "x": float(grid.x[ix]),
                            "q2": float(grid.Q[iq] ** 2),
                            "xf": float(grid.xfgrid[row, ifl]),
                        }
                    )
    return points


def eval_xfxq2(pdf, pdg, x, q2):
    """Evaluate x*f(x, Q^2) for one point, walking subgrids in order and
    taking the first in-range (non-NaN) hit -- the same policy as
    `PDF.xfxQ2`, called directly on each `PDFGrid` to avoid two version-drift
    issues in the top-level wrapper under modern numpy/scipy: `grid=False`
    mismatches shapes for scalar input (see the `MyRectBivariateSpline`
    broadcast in `pdf.py`), and its size-1-array-to-float collapse
    (`float(res)`) is rejected by numpy >= 2.0 for non-0-d arrays.
    """
    for grid in pdf.pdfgrids:
        arr = grid.xfxQ2(pdg, np.array([x]), np.array([q2]), grid=True)
        val = np.asarray(arr).reshape(-1)[0]
        if not np.isnan(val):
            return float(val)
    return float("nan")


def spline_points(pdf, category, xq2_pairs, pdgs):
    """(pdg, x, Q^2, x*f) tuples evaluated through parton's interpolator."""
    points = []
    for x, q2 in xq2_pairs:
        for pdg in pdgs:
            xf = eval_xfxq2(pdf, pdg, x, q2)
            points.append(
                {
                    "category": category,
                    "pdg": int(pdg),
                    "x": float(x),
                    "q2": float(q2),
                    "xf": xf,
                }
            )
    return points


def main():
    pdf = PDF(SET_NAME, member=MEMBER, pdfdir=SCRIPT_DIR)
    info = pdf.pdfset.info
    all_pdgs = sorted(set(int(p) for grid in pdf.pdfgrids for p in grid.flavors))
    # Also exercise the 0<->21 gluon alias explicitly.
    probe_pdgs = all_pdgs + [0]

    # PyYAML's default resolver only auto-types exponent-form floats that
    # contain a decimal point (e.g. "1.0e-09", not "1e-09"), so these must be
    # coerced explicitly rather than trusted as already-parsed floats.
    x_min, x_max = float(info["XMin"]), float(info["XMax"])
    q_min, q_max = np.sqrt(float(info["QMin"])), np.sqrt(float(info["QMax"]))

    points = []
    points += knot_points(pdf)

    # Off-knot interior samples: log-spaced in x, log-spaced in Q, avoiding
    # exact knot coincidence.
    off_knot_x = np.geomspace(x_min * 10, 0.5, 4)
    off_knot_q = np.geomspace(q_min * 1.3, q_max * 0.7, 4)
    off_knot_pairs = [(x, q * q) for x in off_knot_x for q in off_knot_q]
    points += spline_points(pdf, "off_knot", off_knot_pairs, all_pdgs)

    # Subgrid Q^2 seams: the shared Q boundary between consecutive subgrids,
    # sampled at a representative interior x. With a single subgrid (this
    # pinned set) the only "seams" are the global QMin/QMax edges.
    seam_qs = sorted(
        {pdf.pdfgrids[0].Q[0]}
        | {pdf.pdfgrids[-1].Q[-1]}
        | {g.Q[0] for g in pdf.pdfgrids[1:]}
    )
    seam_pairs = [(0.05, q * q) for q in seam_qs]
    points += spline_points(pdf, "seam", seam_pairs, all_pdgs)

    # x -> 1 tail.
    tail_pairs = [(x, (0.5 * (q_min + q_max)) ** 2) for x in (0.9, 0.99, 0.999, 1.0 - 1e-6)]
    points += spline_points(pdf, "x_to_one_tail", tail_pairs, all_pdgs)

    # Grid corners.
    corner_pairs = [
        (x_min, q_min**2),
        (x_min, q_max**2),
        (x_max, q_min**2),
        (x_max, q_max**2),
    ]
    # probe_pdgs includes 0 (the gluon alias) alongside 21 itself, so the two
    # entries at matching (x, Q^2) pin the 0<->21 aliasing convention.
    points += spline_points(pdf, "corner", corner_pairs, probe_pdgs)

    oracle = {
        "set": SET_NAME,
        "member": MEMBER,
        "num_subgrids": len(pdf.pdfgrids),
        "single_subgrid": len(pdf.pdfgrids) == 1,
        "subgrids": [
            {
                "nx": len(g.x),
                "nq": len(g.Q),
                "flavors": [int(f) for f in g.flavors],
            }
            for g in pdf.pdfgrids
        ],
        "points": points,
    }

    with open(OUTPUT_PATH, "w") as f:
        json.dump(oracle, f, indent=2)

    n_knot = sum(1 for p in points if p["category"] == "knot")
    print(
        f"Wrote {OUTPUT_PATH}: {len(points)} points "
        f"({n_knot} on-knot, {len(pdf.pdfgrids)} subgrid(s))",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
