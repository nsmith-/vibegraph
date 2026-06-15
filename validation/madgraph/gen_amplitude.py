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


def rambo_massless(n: int, sqrt_s: float, rng: np.random.Generator) -> np.ndarray:
    """RAMBO: n massless 4-momenta distributed flat in phase space, summing to
    (sqrt_s, 0, 0, 0).  Returns an (n, 4) array with [E, px, py, pz] rows.

    Standard Kleiss-Stirling-van der Bij construction: isotropic massless q_i,
    then a single boost+scale maps Σq_i onto the CM frame at total energy sqrt_s.
    """
    q = np.zeros((n, 4))
    for i in range(n):
        cth = 2.0 * rng.random() - 1.0
        phi = 2.0 * math.pi * rng.random()
        sth = math.sqrt(max(0.0, 1.0 - cth * cth))
        e = -math.log(max(1e-300, rng.random() * rng.random()))
        q[i] = [e, e * sth * math.cos(phi), e * sth * math.sin(phi), e * cth]

    big_q = q.sum(axis=0)
    mass = math.sqrt(big_q[0] ** 2 - big_q[1:] @ big_q[1:])
    b = -big_q[1:] / mass
    gamma = big_q[0] / mass
    a = 1.0 / (1.0 + gamma)
    x = sqrt_s / mass

    p = np.zeros((n, 4))
    for i in range(n):
        bq = b @ q[i, 1:]
        p[i, 0] = x * (gamma * q[i, 0] + bq)
        p[i, 1:] = x * (q[i, 1:] + b * q[i, 0] + a * bq * b)
    return p


def gen_nbody(
    module, process_str: str, n_final: int, sqrt_s: float, npoints: int, seed: int
) -> tuple[int, list[list[float]]]:
    """Evaluate an n-body MadGraph amplitude over RAMBO phase-space points.

    Incoming legs 1,2 are massless beams along ±z at energy sqrt_s/2 (matching
    the generic wrapper / launch run_card with lpp=0).  Returns (n_ext, rows)
    where each row is [m2, E0,px0,py0,pz0, E1,...] over all n_ext legs.
    """
    import os as _os

    n_ext = 2 + n_final
    card = _os.path.join(
        OUTPUT_DIR, "..", "output", process_str_to_dir(process_str), "Cards", "param_card.dat"
    )
    card = _os.path.abspath(card)
    rng = np.random.default_rng(seed)
    e_beam = sqrt_s / 2.0

    rows: list[list[float]] = []
    for _ in range(npoints):
        out = rambo_massless(n_final, sqrt_s, rng)
        p = np.zeros((4, n_ext), dtype=np.float64, order="F")
        p[:, 0] = [e_beam, 0.0, 0.0, e_beam]   # leg 1 (incoming)
        p[:, 1] = [e_beam, 0.0, 0.0, -e_beam]  # leg 2 (incoming)
        for j in range(n_final):
            p[:, 2 + j] = out[j]
        m2 = float(module.mg_eval_m2(p, card))
        row = [m2]
        for leg in range(n_ext):
            row.extend(float(x) for x in p[:, leg])
        rows.append(row)

    # Quick profiling run
    inputs_array = np.array(rows, dtype=np.float64)[:, 1:].T.reshape(4, n_ext, npoints, order="F")
    # make it bigger if needed
    inputs_array = np.tile(inputs_array, (1, 1, max(1, PROFILE_N // npoints)))
    # actually make values different (how can it be so fast with repeated inputs??) by changing sqrt_s slightly
    inputs_array *= rng.uniform(0.9, 1.1, size=(1, 1, inputs_array.shape[2]))
    nbench = inputs_array.shape[2]

    # warm-up
    module.mg_eval_m2_batch(inputs_array, card)

    t0 = time.perf_counter()
    module.mg_eval_m2_batch(inputs_array, card)
    t1 = time.perf_counter()

    elapsed_ms = (t1 - t0) * 1e3
    print(
        f"MadGraph {module.__name__} MATRIX1 (n={n_ext} legs): {nbench} evals in {elapsed_ms:.2f} ms"
        f"  ({elapsed_ms / nbench:.2f} ms/eval)"
    )

    return n_ext, rows


def process_str_to_dir(process_str: str) -> str:
    """Map a process string to its output directory name (registry-driven)."""
    return NBODY_DIR_BY_PROCESS[process_str]


# Registry of n-body processes: process string -> output dir name.
NBODY_DIR_BY_PROCESS = {
    "u u~ > c c~ e+ e- mu+ mu-": "uux_to_ccx_emmm_qcd0",
}


def write_csv_nbody(path: str, process_str: str, n_ext: int, rows: list[list[float]]):
    """Write a momenta-based CSV: header carries n_ext, each row is m2 + all
    leg 4-momenta.  Distinct from the 2->2 (sqrt_s, cos_theta) schema."""
    comps = ",".join(
        f"{c}{leg}" for leg in range(n_ext) for c in ("E", "px", "py", "pz")
    )
    with open(path, "w") as f:
        f.write(f"# process: {process_str}\n")
        f.write(f"# n_ext: {n_ext}\n")
        f.write(f"m2_summed,{comps}\n")
        for row in rows:
            f.write(",".join(repr(v) for v in row) + "\n")
    print(f"Wrote {len(rows)} rows ({n_ext} legs) to {path}")


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

    print("\nGenerating u u~ > c c~ e+ e- mu+ mu- (QCD=0) 2->6 amplitude reference...")
    import mg_uux_to_ccx_emmm_qcd0  # compiled by build_amplitude.sh (generic wrapper)

    # The bare process string is used to locate the output directory; the CSV
    # header records the full spec including coupling orders so the harness can
    # reproduce the same diagram set (QCD=0 → pure-EW, 579 diagrams).
    proc = "u u~ > c c~ e+ e- mu+ mu-"
    proc_with_orders = "u u~ > c c~ e+ e- mu+ mu- QCD=0"
    n_ext, rows_6 = gen_nbody(
        mg_uux_to_ccx_emmm_qcd0, proc, n_final=6, sqrt_s=500.0, npoints=50, seed=7
    )
    write_csv_nbody(
        os.path.join(OUTPUT_DIR, "uux_to_ccx_emmm_qcd0_amplitude.csv"),
        proc_with_orders,
        n_ext,
        rows_6,
    )

    print("\nDone.")
