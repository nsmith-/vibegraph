#!/usr/bin/env python3
"""
gen_reference.py — Generate HELAS reference data for e+ e- → μ+ μ- (SM, tree level).

Calls the Fortran77 HELAS routines (compiled via f2py as `helas_f`) on a 20×20
kinematic grid in (√s, cos θ), computes |M|² summed over all helicities for the
full Standard Model (γ + Z s-channel diagrams), and writes:

    reference.npz  — NumPy archive with keys: sqrt_s, cos_theta, M2
    reference.csv  — Human-readable table for inspection

SM parameters are taken from MadGraph's default param_card.dat:
    aEWM1 = 132.507,  Gf = 1.16639e-5,  MZ = 91.188 GeV,  WZ = 2.441404 GeV

Run with:
    pixi run -e helas-validation gen-reference
or equivalently:
    cd validation/helas && python gen_reference.py
"""

import math
import sys

import numpy as np

# ---------------------------------------------------------------------------
# SM physical constants (matching MadGraph default param_card.dat)
# ---------------------------------------------------------------------------
AEWM1    = 132.507          # α⁻¹ at MZ scale
ALPHA_QED_MZ = 1.0 / AEWM1
GF       = 1.16639e-5       # Fermi constant (GeV⁻²)
MDL_MZ   = 91.188           # Z mass (GeV)
MDL_WZ   = 2.441404         # Z total width (GeV)

# Derived EW parameters
E_SM  = math.sqrt(4 * math.pi * ALPHA_QED_MZ)   # e = sqrt(4π α(MZ))
SW2   = 0.5 - math.sqrt(0.25 - math.pi * ALPHA_QED_MZ / (GF * math.sqrt(2) * MDL_MZ**2))
SW    = math.sqrt(SW2)
CW    = math.sqrt(1.0 - SW2)

# Thompson-limit values (kept for backward-compatible tests at very low energy)
ALPHA_QED = 1.0 / 137.035999084
E_QED     = math.sqrt(4 * math.pi * ALPHA_QED)


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
# HELAS amplitude calculation (full SM: γ + Z)
# ---------------------------------------------------------------------------

def compute_M2_helas(helas_f, sqrt_s: float, cos_theta: float) -> float:
    """
    Compute |M|² summed over all 16 helicity combinations for e+ e- → μ+ μ-
    via the full SM tree level (s-channel γ and Z diagrams).

    HELAS calls:
        Electron line:
            ixxxxx(p1, me=0, nhel1, nsf=+1) → fi_em
            oxxxxx(p2, me=0, nhel2, nsf=-1) → fo_ep
            jioxxx(fi_em, fo_ep, gc_gamma, vmass=0, vwidth=0)  → jio_gamma
            jioxxx(fi_em, fo_ep, gc_z,     MZ,      WZ)        → jio_z

        Muon line:
            ixxxxx(p4, mmu=0, nhel4, nsf=-1) → fi_mup
            oxxxxx(p3, mmu=0, nhel3, nsf=+1) → fo_mum
            iovxxx(fi_mup, fo_mum, jio_gamma, gc_gamma) → amp_gamma
            iovxxx(fi_mup, fo_mum, jio_z,     gc_z)     → amp_z

        Coherent sum (matching MadGraph JAMP convention):
            amp_total = amp_gamma + amp_z

    Couplings:
        gc_gamma = [−e, −e]                              (vector, Q_e=−1)
        gc_z     = [gL_Z, gR_Z]
            gL_Z = e (−½ + sin²θW) / (sin θW cos θW)    (matches MadGraph GC_59 / i)
            gR_Z = e sin θW / cos θW                     (matches MadGraph GC_50 / i)

    Returns:
        |M|² summed over all 16 helicities (no averaging).
        Divide by 4 to get the spin-averaged |M̄|².
    """
    p_em, p_ep, p_mum, p_mup = make_momenta(sqrt_s, cos_theta)

    # Photon coupling: gc_γ = [−e, −e]  (pure vector, Q_e = −1)
    gc_gamma = np.array([-E_SM, -E_SM], dtype=np.complex128)

    # Z couplings
    gL_z = E_SM * (-0.5 + SW2) / (SW * CW)
    gR_z = E_SM * SW / CW
    gc_z = np.array([gL_z, gR_z], dtype=np.complex128)

    M2_total = 0.0

    for nhel_em in (-1, 1):
        for nhel_ep in (-1, 1):
            # Electron line — one call each for γ and Z off-shell current
            fi_em = helas_f.ixxxxx(p_em, 0.0, nhel_em,  1)
            fo_ep = helas_f.oxxxxx(p_ep, 0.0, nhel_ep, -1)
            jio_gamma = helas_f.jioxxx(fi_em, fo_ep, gc_gamma, 0.0,    0.0   )
            jio_z     = helas_f.jioxxx(fi_em, fo_ep, gc_z,     MDL_MZ, MDL_WZ)

            for nhel_mum in (-1, 1):
                for nhel_mup in (-1, 1):
                    # Muon line
                    fi_mup = helas_f.ixxxxx(p_mup, 0.0, nhel_mup, -1)
                    fo_mum = helas_f.oxxxxx(p_mum, 0.0, nhel_mum,  1)
                    amp_gamma = helas_f.iovxxx(fi_mup, fo_mum, jio_gamma, gc_gamma)
                    amp_z     = helas_f.iovxxx(fi_mup, fo_mum, jio_z,     gc_z    )
                    amp_total = amp_gamma + amp_z
                    M2_total += abs(amp_total)**2

    return M2_total


# ---------------------------------------------------------------------------
# Analytic cross-checks
# ---------------------------------------------------------------------------

def analytic_M2_qed_only(sqrt_s: float, cos_theta: float) -> float:
    """
    Analytic Σ|M|² for pure QED (photon only, using α(MZ)):

        Σ |M|² = 4 e⁴ (1 + cos²θ)

    Valid when √s ≪ MZ so that Z exchange is negligible.
    """
    e4 = E_SM**4
    return 4.0 * e4 * (1.0 + cos_theta**2)


def analytic_sigma_total_qed(sqrt_s: float) -> float:
    """
    Total QED cross section: σ = 4πα²(MZ) / (3s)  [GeV⁻²]
    Convert to nb: 1 GeV⁻² = 0.3894 mb = 3.894×10⁵ nb.
    """
    s = sqrt_s**2
    return 4.0 * math.pi * ALPHA_QED_MZ**2 / (3.0 * s)


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

    print("Computing HELAS reference grid (20×20, SM: γ+Z)...")
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
    # Sanity check 1: at low √s, compare HELAS to pure-QED formula
    # (Z contribution is small at √s << MZ, so ratio should be ~1)
    # ------------------------------------------------------------------
    print("\n--- Sanity check: off-Z-pole QED approximation (√s ≪ MZ) ---")
    print(f"{'sqrt_s':>10}  {'cos_θ':>8}  {'HELAS M²':>18}  {'QED M²':>18}  {'ratio':>8}")
    max_reldiff_qed = 0.0
    for sqrt_s in [10.0, 20.0, 30.0]:
        for cos_theta in [0.0, 0.5, -0.5]:
            helas_val    = compute_M2_helas(helas_f, sqrt_s, cos_theta)
            analytic_val = analytic_M2_qed_only(sqrt_s, cos_theta)
            ratio = helas_val / analytic_val if analytic_val != 0 else float("nan")
            reldiff = abs(ratio - 1.0)
            max_reldiff_qed = max(max_reldiff_qed, reldiff)
            print(f"{sqrt_s:>10.1f}  {cos_theta:>8.2f}  {helas_val:>18.8e}  "
                  f"{analytic_val:>18.8e}  {ratio:>8.5f}")

    # At √s=10 GeV, Z correction is ~0.1 %; at √s=30 GeV it's ~1 %.
    print(f"\nMax QED ratio deviation at off-Z-pole: {max_reldiff_qed:.2e}")
    tol_qed = 0.05   # 5 % — accounts for Z interference at √s=30 GeV
    if max_reldiff_qed > tol_qed:
        print(f"WARNING: deviation {max_reldiff_qed:.2e} exceeds tolerance {tol_qed:.2e}")
    else:
        print(f"PASS: deviation < {tol_qed:.2e}")

    # ------------------------------------------------------------------
    # Sanity check 2: Z-pole enhancement
    # (SM result must be >> pure-QED at √s ≈ MZ)
    # ------------------------------------------------------------------
    print("\n--- Sanity check: Z-pole enhancement ---")
    for sqrt_s, cos_theta in [(MDL_MZ, 0.0), (MDL_MZ, 0.5), (MDL_MZ, -0.5)]:
        helas_val    = compute_M2_helas(helas_f, sqrt_s, cos_theta)
        analytic_val = analytic_M2_qed_only(sqrt_s, cos_theta)
        ratio = helas_val / analytic_val
        status = "PASS" if ratio > 50.0 else "FAIL"
        print(f"  √s={sqrt_s:.1f} GeV, cos_θ={cos_theta:+.1f}: "
              f"SM/QED = {ratio:.1f}  [{status}]")

    # ------------------------------------------------------------------
    # Total cross section at √s = 91.2 GeV (Z pole)
    # ------------------------------------------------------------------
    print("\n--- σ_total at √s = 91.2 GeV (SM, numerical integration) ---")
    sqrt_s_check = MDL_MZ
    s_check = sqrt_s_check**2
    cos_vals = np.linspace(-1.0, 1.0, 1000)
    M2_vals  = np.array([compute_M2_helas(helas_f, sqrt_s_check, c) for c in cos_vals])
    integ    = np.trapezoid(M2_vals, cos_vals)
    # σ = (1/4) * (1/(64π²s)) * 2π * ∫ Σ|M|² dc
    GeVm2_to_pb = 0.3894e9   # 1 GeV⁻² = 0.3894 mb = 3.894×10⁸ pb
    sigma_sm_pb  = (2 * math.pi / (4 * 64 * math.pi**2 * s_check)) * integ * GeVm2_to_pb
    sigma_qed_pb = analytic_sigma_total_qed(sqrt_s_check) * GeVm2_to_pb
    print(f"  SM (numerical):    σ ≈ {sigma_sm_pb:.1f} pb")
    print(f"  Pure QED:          σ ≈ {sigma_qed_pb:.2f} pb")
    print(f"  MadGraph reference: σ ≈ 2025 pb  (from ee_to_mumu run)")


if __name__ == "__main__":
    main()

