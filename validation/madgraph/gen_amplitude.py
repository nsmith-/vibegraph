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
import time
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

# Number of random points used for the batch timing benchmark.
PROFILE_N = 10_000


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


def make_momenta_batch_ee_mumu(
    sqrt_s_arr: np.ndarray, cos_theta_arr: np.ndarray
) -> np.ndarray:
    """Build a Fortran-contiguous (4, 4, N) batch array for MG_EVAL_M2_BATCH.

    Axis layout: [4-momentum component, particle index, event index].
    The first two axes match the (0:3, 4) P array expected by MATRIX1.
    """
    e = sqrt_s_arr / 2.0
    sin_t = np.sqrt(np.maximum(0.0, 1.0 - cos_theta_arr**2))
    n = len(sqrt_s_arr)
    p = np.zeros((4, 4, n), dtype=np.float64, order="F")
    # e+: (E, 0, 0, -E)
    p[0, 0, :] = e
    p[3, 0, :] = -e
    # e-: (E, 0, 0, +E)
    p[0, 1, :] = e
    p[3, 1, :] = e
    # mu+: (E, -E*sin_t, 0, -E*cos_t)
    p[0, 2, :] = e
    p[1, 2, :] = -e * sin_t
    p[3, 2, :] = -e * cos_theta_arr
    # mu-: (E, +E*sin_t, 0, +E*cos_t)
    p[0, 3, :] = e
    p[1, 3, :] = e * sin_t
    p[3, 3, :] = e * cos_theta_arr
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


def profile_ee_to_mumu(n: int = PROFILE_N) -> None:
    """Time MadGraph's MATRIX1 on a batch of N random kinematic points.

    Uses the MG_EVAL_M2_BATCH entry point so common-block setup overhead is
    paid once per batch rather than once per event.  Prints ns/eval to stdout.
    """
    import mg_ee_to_mumu

    rng = np.random.default_rng(42)
    sqrt_s_arr = rng.uniform(10.0, 200.0, n)
    cos_theta_arr = rng.uniform(-0.9, 0.9, n)
    p_batch = make_momenta_batch_ee_mumu(sqrt_s_arr, cos_theta_arr)

    # Warm-up: triggers MATRIX1's first-call SAVE initialisation
    p_warm = make_momenta_ee_mumu(50.0, 0.0)
    mg_ee_to_mumu.mg_eval_m2(p_warm, AEWM1, MDL_GF, MDL_MZ, MDL_WZ)

    t0 = time.perf_counter()
    m2 = mg_ee_to_mumu.mg_eval_m2_batch(p_batch, AEWM1, MDL_GF, MDL_MZ, MDL_WZ)
    t1 = time.perf_counter()

    elapsed_ms = (t1 - t0) * 1e3
    ns_per_eval = (t1 - t0) / n * 1e9
    print(
        f"MadGraph MATRIX1 (ee->mumu): {n} evals in {elapsed_ms:.2f} ms"
        f"  ({ns_per_eval:.0f} ns/eval)"
    )
    _ = m2  # result available if caller needs it


def gen_pp_to_ll_qcd0() -> list[tuple[float, float, float]]:
    """Evaluate MadGraph u u~ > mu+ mu- amplitude on the kinematic grid.

    The subprocess P1_qq_ll covers u u~, c c~ > e+ e-, mu+ mu- (same coupling
    structure: up-type quarks, massless limit).  MATRIX1 returns CF*sum_hel|M|^2
    where CF=3 is the quark color factor; this is NOT divided by IDEN=36.

    Momentum layout (columns match Rust make_momenta_2to2 and Fortran MATRIX1):
      col 0: u   → (E,  0,       0,  -E)
      col 1: u~  → (E,  0,       0,  +E)
      col 2: mu+ → (E, -E*sin_t, 0,  -E*cos_t)
      col 3: mu- → (E, +E*sin_t, 0,  +E*cos_t)
    """
    import mg_pp_to_ll_qcd0  # compiled by build_amplitude.sh

    rows = []
    for sqrt_s in SQRT_S_VALUES:
        for cos_theta in COS_THETA_VALUES:
            p = make_momenta_ee_mumu(float(sqrt_s), float(cos_theta))
            m2 = mg_pp_to_ll_qcd0.mg_eval_m2(p, AEWM1, MDL_GF, MDL_MZ, MDL_WZ)
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

    print(f"\nProfiling batch evaluator ({PROFILE_N} random points)...")
    profile_ee_to_mumu(PROFILE_N)

    print("\nGenerating pp->ll QCD=0 amplitude reference (u u~ > mu+ mu-)...")
    rows_qq = gen_pp_to_ll_qcd0()
    write_csv(
        os.path.join(OUTPUT_DIR, "pp_to_ll_qcd0_amplitude.csv"),
        "u u~ > mu+ mu-",
        rows_qq,
    )

    print("\nDone.")
