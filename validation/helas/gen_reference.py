#!/usr/bin/env python3
"""
gen_reference.py — Generate HELAS reference data for e+ e- → μ+ μ- (QED, tree level).

Calls the Fortran77 HELAS routines (compiled via f2py as `helas_f`) on a 20×20
kinematic grid in (√s, cos θ), computes |M|² summed over all helicities for the
single-photon diagram, and writes:

    reference.npz  — NumPy archive with keys: sqrt_s, cos_theta, M2
    reference.csv  — Human-readable table for inspection

Run with:
    pixi run -e helas-validation gen-reference
or equivalently:
    cd validation/helas && python gen_reference.py
"""

import math
import sys

import numpy as np

# ---------------------------------------------------------------------------
# Physical constants
# ---------------------------------------------------------------------------
ALPHA_QED = 1.0 / 137.035999084   # fine-structure constant
E_QED = math.sqrt(4 * math.pi * ALPHA_QED)  # |e| in natural units


# ---------------------------------------------------------------------------
# Kinematics
# ---------------------------------------------------------------------------

def make_momenta(sqrt_s: float, cos_theta: float) -> tuple:
    """
    Return (p_em, p_ep, p_mum, p_mup) as numpy float64 arrays of shape (4,)
    indexed as p[0..3] = (E, px, py, pz), i.e. p[0] = E (metric: +---).

    CM frame:
        e-  along +z  (p1)
        e+  along -z  (p2)
        μ-  at polar angle θ in the x-z plane  (p3)
        μ+  opposite  (p4)

    Massless approximation: me = mμ = 0.
    HELAS uses the ordering p(0:3) = (E, px, py, pz).
    """
    E = sqrt_s / 2.0
    sin_theta = math.sqrt(max(0.0, 1.0 - cos_theta**2))

    p_em  = np.array([E,          0.0,       0.0,  E          ], dtype=np.float64)
    p_ep  = np.array([E,          0.0,       0.0, -E          ], dtype=np.float64)
    p_mum = np.array([E,  E*sin_theta,       0.0,  E*cos_theta], dtype=np.float64)
    p_mup = np.array([E, -E*sin_theta,       0.0, -E*cos_theta], dtype=np.float64)
    return p_em, p_ep, p_mum, p_mup


# ---------------------------------------------------------------------------
# HELAS amplitude calculation
# ---------------------------------------------------------------------------

def compute_M2_helas(helas_f, sqrt_s: float, cos_theta: float) -> float:
    """
    Compute |M|² summed over all 16 helicity combinations for e+ e- → μ+ μ-
    via a single virtual photon exchange at tree level.

    Diagram:
        e-(p1) + e+(p2) → γ*(q=p1+p2) → μ-(p3) + μ+(p4)

    HELAS calls:
        Electron line (flowing-in fermion number e-→photon vertex):
            ixxxxx(p1, me=0, nhel1, nsf=+1) → fi_em   [e-, incoming particle]
            oxxxxx(p2, me=0, nhel2, nsf=-1) → fo_ep   [e+, incoming anti-particle]
            jioxxx(fi_em, fo_ep, gc, vmass=0, vwidth=0) → jio  [off-shell photon current]

        Muon line (flowing-in fermion number μ+→photon vertex):
            ixxxxx(p4, mmu=0, nhel4, nsf=-1) → fi_mup  [μ+, outgoing anti-particle]
            oxxxxx(p3, mmu=0, nhel3, nsf=+1) → fo_mum  [μ-, outgoing particle]
            iovxxx(fi_mup, fo_mum, jio, gc) → vertex  [amplitude]

    Coupling: gc = [−e, −e] (pure vector QED, Q_e = −1).

    Returns:
        |M|² summed over all helicities (no averaging).
        Divide by 4 to get the spin-averaged |M̄|².
    """
    p_em, p_ep, p_mum, p_mup = make_momenta(sqrt_s, cos_theta)

    # QED photon-fermion coupling: gc(1)=gc(2)=-e (pure vector, massless)
    gc = np.array([-E_QED, -E_QED], dtype=np.complex128)

    M2_total = 0.0

    for nhel_em in (-1, 1):
        for nhel_ep in (-1, 1):
            # Electron line → off-shell photon current
            fi_em  = helas_f.ixxxxx(p_em,  0.0, nhel_em,  1)
            fo_ep  = helas_f.oxxxxx(p_ep,  0.0, nhel_ep, -1)
            jio    = helas_f.jioxxx(fi_em, fo_ep, gc, 0.0, 0.0)

            for nhel_mum in (-1, 1):
                for nhel_mup in (-1, 1):
                    # Muon line → amplitude
                    fi_mup = helas_f.ixxxxx(p_mup, 0.0, nhel_mup, -1)
                    fo_mum = helas_f.oxxxxx(p_mum, 0.0, nhel_mum,  1)
                    vertex = helas_f.iovxxx(fi_mup, fo_mum, jio, gc)
                    M2_total += abs(vertex)**2

    return M2_total


# ---------------------------------------------------------------------------
# Analytic cross-check
# ---------------------------------------------------------------------------

def analytic_M2_summed(sqrt_s: float, cos_theta: float) -> float:
    """
    Analytic result for Σ_{all helicities} |M|² at tree level, QED only,
    massless fermions, single photon exchange:

        Σ |M|² = 4 e⁴ (1 + cos²θ)

    Reference: Peskin & Schroeder section 5.5 (or any QFT textbook).
    The 1/s² from the propagator is already included.
    """
    e4 = E_QED**4
    return 4.0 * e4 * (1.0 + cos_theta**2)


def analytic_sigma_total(sqrt_s: float) -> float:
    """
    Total cross section: σ = 4πα² / (3s)  [in natural units, GeV^-2]
    Convert to nb: 1 GeV^-2 = 0.3894 mb = 3.894e5 nb.
    """
    s = sqrt_s**2
    return 4.0 * math.pi * ALPHA_QED**2 / (3.0 * s)


# ---------------------------------------------------------------------------
# Main: generate grid and save reference files
# ---------------------------------------------------------------------------

def main():
    try:
        import helas_f
    except ImportError:
        print(
            "ERROR: helas_f module not found.\n"
            "Run 'pixi run -e helas-validation build-helas' first.",
            file=sys.stderr,
        )
        sys.exit(1)

    # 20×20 grid
    sqrt_s_values    = np.linspace(10.0, 200.0, 20)
    cos_theta_values = np.linspace(-0.9,   0.9, 20)

    grid_s, grid_c = np.meshgrid(sqrt_s_values, cos_theta_values, indexing="ij")
    M2_grid = np.zeros_like(grid_s)

    print("Computing HELAS reference grid (20×20)...")
    for i, sqrt_s in enumerate(sqrt_s_values):
        for j, cos_theta in enumerate(cos_theta_values):
            M2_grid[i, j] = compute_M2_helas(helas_f, sqrt_s, cos_theta)

    # Save .npz
    np.savez(
        "reference.npz",
        sqrt_s=grid_s,
        cos_theta=grid_c,
        M2=M2_grid,
    )
    print("Saved reference.npz")

    # Save .csv (flat table)
    with open("reference.csv", "w") as f:
        f.write("sqrt_s_GeV,cos_theta,M2_summed\n")
        for i, sqrt_s in enumerate(sqrt_s_values):
            for j, cos_theta in enumerate(cos_theta_values):
                f.write(f"{sqrt_s:.6f},{cos_theta:.6f},{M2_grid[i,j]:.15e}\n")
    print("Saved reference.csv")

    # ------------------------------------------------------------------
    # Sanity check: compare HELAS result to analytic formula
    # ------------------------------------------------------------------
    print("\n--- Sanity check vs analytic formula ---")
    print(f"{'sqrt_s':>10}  {'cos_θ':>8}  {'HELAS M²':>18}  {'Analytic M²':>18}  {'ratio':>8}")
    max_reldiff = 0.0
    for sqrt_s in [10.0, 50.0, 91.2, 200.0]:
        for cos_theta in [0.0, 0.5, -0.5]:
            helas_val    = compute_M2_helas(helas_f, sqrt_s, cos_theta)
            analytic_val = analytic_M2_summed(sqrt_s, cos_theta)
            ratio = helas_val / analytic_val if analytic_val != 0 else float("nan")
            reldiff = abs(ratio - 1.0)
            max_reldiff = max(max_reldiff, reldiff)
            print(f"{sqrt_s:>10.1f}  {cos_theta:>8.2f}  {helas_val:>18.8e}  {analytic_val:>18.8e}  {ratio:>8.5f}")

    print(f"\nMax relative deviation from analytic: {max_reldiff:.2e}")
    tol = 1e-10
    if max_reldiff > tol:
        print(f"WARNING: deviation {max_reldiff:.2e} exceeds tolerance {tol:.2e}")
    else:
        print(f"PASS: deviation < {tol:.2e}")

    # Total cross section check at sqrt_s = 91.2 GeV
    print("\n--- σ_total at sqrt_s = 91.2 GeV ---")
    sqrt_s_check = 91.2
    # Numerical integration: σ = (1/64π²s) * ∫ Σ|M|² dΩ * (1/4) [initial avg]
    # = (1/64π²s) * (1/4) * 2π * ∫_{-1}^{1} Σ|M|²(c) dc
    s_check = sqrt_s_check**2
    cos_vals = np.linspace(-1.0, 1.0, 1000)
    M2_vals = np.array([compute_M2_helas(helas_f, sqrt_s_check, c) for c in cos_vals])
    integ = np.trapezoid(M2_vals, cos_vals)  # ∫_{-1}^{1} Σ|M|²(c) dc
    # dσ/dΩ = |M̄|²/(64π²s), |M̄|² = Σ|M|²/4
    # σ = (1/4) * (1/(64π²s)) * 2π * ∫ Σ|M|² dc
    GeVm2_to_nb = 0.3894e6  # 1 GeV^-2 = 0.3894 mb = 0.3894e6 nb
    sigma_helas    = (2 * math.pi / (4 * 64 * math.pi**2 * s_check)) * integ * GeVm2_to_nb
    sigma_analytic = analytic_sigma_total(sqrt_s_check) * GeVm2_to_nb
    print(f"  HELAS (numerical):  σ = {sigma_helas:.4f} nb")
    print(f"  Analytic 4πα²/3s:   σ = {sigma_analytic:.4f} nb")
    print(f"  Relative difference: {abs(sigma_helas/sigma_analytic - 1):.2e}")


if __name__ == "__main__":
    main()
