#!/usr/bin/env python3
"""Generate MadGraph amplitude reference CSVs for helas_mg_validation.

For each registered process, evaluates MadGraph's compiled MATRIX1 Fortran
subroutine over a fixed kinematic grid and writes the results to
  output/PROCESSNAME_amplitude.csv

CSV format (3 columns, matching validation/helas/reference.csv convention):
  # process: PROCESS_STRING
  sqrt_s_GeV,cos_theta,M2_summed
  10.0,...,...

M2_summed = sum_hel |M|^2 (NOT divided by IDEN / spin-averaging factor).
This matches the output of vibegraph's AmplitudeEvaluator.eval_m2().

Usage:
  python validation/madgraph/gen_amplitude.py
  pixi run -e madgraph generate-amplitude

Prerequisites:
  pixi run -e madgraph build-amplitude
"""

import sys
import os
import math
import numpy as np

# Allow importing compiled .so modules from this directory
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

OUTPUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "output")

# SM parameters matching MadGraph's default param_card.dat.
# Must agree with validation/helas/gen_reference.py and the UFO SM model defaults
# used by vibegraph (ParamCard::from_str("") → model.evaluate()).
AEWM1 = 132.507
MDL_GF = 1.16639e-5
MDL_MZ = 91.188
MDL_WZ = 2.441404

# Kinematic grid: 20×20, same as gen_reference.py
SQRT_S_VALUES = np.linspace(10.0, 200.0, 20)
COS_THETA_VALUES = np.linspace(-0.9, 0.9, 20)


def make_momenta_ee_mumu(sqrt_s: float, cos_theta: float) -> np.ndarray:
    """Physical 4-momenta for e+ e- > mu+ mu- in the CM frame (massless limit).

    Ordering matches AmplitudeEvaluator's external leg order and
    helas_validation.rs (lines 145-149):
      col 0: e+  → (E,  0,       0,  -E)
      col 1: e-  → (E,  0,       0,  +E)
      col 2: mu+ → (E, -E*sin_t, 0,  -E*cos_t)
      col 3: mu- → (E, +E*sin_t, 0,  +E*cos_t)

    Returns a Fortran-contiguous (4, 4) float64 array as required by f2py
    for REAL*8 P(0:3, NEXTERNAL) with NEXTERNAL=4.
    """
    E = sqrt_s / 2.0
    sin_t = math.sqrt(max(0.0, 1.0 - cos_theta**2))
    p = np.zeros((4, 4), dtype=np.float64, order="F")
    p[:, 0] = [E, 0.0, 0.0, -E]                   # e+
    p[:, 1] = [E, 0.0, 0.0, +E]                   # e-
    p[:, 2] = [E, -E * sin_t, 0.0, -E * cos_theta]  # mu+
    p[:, 3] = [E, +E * sin_t, 0.0, +E * cos_theta]  # mu-
    return p


def gen_ee_to_mumu() -> list[tuple[float, float, float]]:
    """Evaluate MadGraph ee->mumu amplitude on the kinematic grid."""
    import mg_ee_to_mumu  # compiled by build_amplitude.sh

    rows = []
    for sqrt_s in SQRT_S_VALUES:
        for cos_theta in COS_THETA_VALUES:
            p = make_momenta_ee_mumu(float(sqrt_s), float(cos_theta))
            m2 = mg_ee_to_mumu.mg_eval_m2(p, AEWM1, MDL_GF, MDL_MZ, MDL_WZ)
            rows.append((float(sqrt_s), float(cos_theta), float(m2)))
    return rows


def write_csv(path: str, process_str: str, rows: list[tuple[float, float, float]]):
    with open(path, "w") as f:
        f.write(f"# process: {process_str}\n")
        f.write("sqrt_s_GeV,cos_theta,M2_summed\n")
        for sqrt_s, cos_theta, m2 in rows:
            f.write(f"{repr(sqrt_s)},{repr(cos_theta)},{repr(m2)}\n")
    print(f"Wrote {len(rows)} rows to {path}")


def sanity_check_ee_mumu(rows: list[tuple[float, float, float]]):
    """Cross-check against pure-QED formula at low energy (off Z-pole)."""
    aew = 1.0 / AEWM1
    ee = math.sqrt(4.0 * math.pi * aew)
    # Off-pole QED: sum_hel |M|^2 = 4*e^4*(1 + cos^2_theta) for massless limit
    # This is NOT divided by IDEN=4, matching what MATRIX1 returns.
    qed_check_sqrt_s = [10.0, 20.0, 30.0]
    for target_sqrt_s in qed_check_sqrt_s:
        # Find a row close to cos_theta = 0 at this sqrt_s
        candidates = [(r, abs(r[1])) for r in rows if abs(r[0] - target_sqrt_s) < 1.0]
        if not candidates:
            continue
        row, _ = min(candidates, key=lambda x: x[1])
        sqrt_s, cos_theta, m2_mg = row
        m2_qed = 4.0 * ee**4 * (1.0 + cos_theta**2)
        rel = abs(m2_mg - m2_qed) / m2_qed
        # Allow 5% for Z-interference and mass effects at these energies
        assert rel < 0.05, (
            f"QED sanity check failed at sqrt_s={sqrt_s:.1f}, cos={cos_theta:.2f}: "
            f"MG={m2_mg:.4e}, QED={m2_qed:.4e}, rel={rel:.3f}"
        )
    print("Sanity check passed: ee->mumu agrees with QED formula off Z-pole.")


if __name__ == "__main__":
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    print("Generating ee->mumu amplitude reference...")
    rows = gen_ee_to_mumu()
    sanity_check_ee_mumu(rows)
    write_csv(
        os.path.join(OUTPUT_DIR, "ee_to_mumu_amplitude.csv"),
        "e+ e- > mu+ mu-",
        rows,
    )

    print("Done.")
