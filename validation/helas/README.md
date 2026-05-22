# HELAS Validation Harness

Validates the Rust HELAS helicity-amplitude implementation against the original
Fortran77 HELAS routines (KEK-91-11, bundled with MadGraph5\_aMC@NLO) by running
both on the same kinematic grid for **e⁺ e⁻ → μ⁺ μ⁻** (QED tree level, single
virtual photon exchange) and comparing |M|² point by point.

## What this harness does

1. **Compiles** five Fortran77 HELAS subroutines into a Python extension module
   (`helas_f`) via NumPy's `f2py`.
2. **Generates** a 20 × 20 reference grid in (√s, cos θ) and stores |M|² summed
   over all 16 helicity combinations.
3. **Cross-checks** the numerical result against the analytic QED formula
   `Σ|M|² = 4e⁴(1 + cos²θ)` and `σ_total = 4πα²/(3s)`.
4. (Future) **Drives** the Rust integration test in `tests/helas_validation.rs`.

## HELAS routines used

| Subroutine | Role |
|---|---|
| `ixxxxx` | Flowing-IN fermion wavefunction |
| `oxxxxx` | Flowing-OUT fermion wavefunction |
| `jioxxx` | Off-shell vector current from fermion pair |
| `iovxxx` | Fermion–fermion–vector amplitude |
| `vxxxxx` | External vector (photon) wavefunction (reference) |

## Amplitude recipe for e⁺ e⁻ → μ⁺ μ⁻

```
1. fi_em  = ixxxxx(p_e-,  me=0, nhel, nsf=+1)  # incoming e-
2. fo_ep  = oxxxxx(p_e+,  me=0, nhel, nsf=-1)  # incoming e+ (anti-particle)
3. jio    = jioxxx(fi_em, fo_ep, gc, m=0, Γ=0)  # off-shell photon current
4. fi_mup = ixxxxx(p_μ+,  mμ=0, nhel, nsf=-1)  # outgoing μ+ (anti-particle)
5. fo_mum = oxxxxx(p_μ-,  mμ=0, nhel, nsf=+1)  # outgoing μ-
6. M      = iovxxx(fi_mup, fo_mum, jio, gc)     # amplitude
```

Coupling: `gc = (-e, -e)` where `e = √(4πα)`, `α = 1/137`.

## Build

```bash
pixi run -e helas-validation build-helas
```

This runs `build.sh` which invokes `f2py` to compile the `.F` files into `helas_f.so`.

## Generate reference data

```bash
pixi run -e helas-validation gen-reference
```

Produces:

- `reference.npz` — NumPy archive with arrays `sqrt_s`, `cos_theta`, `M2`
  (all shape 20 × 20).  `M2[i,j]` is `Σ_{helicities} |M|²` at the
  corresponding kinematic point.
- `reference.csv` — Flat CSV for easy inspection:
  `sqrt_s_GeV, cos_theta, M2_summed`.

Also prints a sanity-check table comparing the HELAS result to the analytic
formula and the total cross section at √s = 91.2 GeV.

## Expected output (sanity check)

- `Σ|M|²` should match `4e⁴(1 + cos²θ)` to numerical precision (< 10⁻¹⁰
  relative difference).
- `σ_total(91.2 GeV)` ≈ 0.29 nb  (photon-only, no Z).

## Rust integration test

`tests/helas_validation.rs` contains a stub test (marked `#[ignore]`) that will
read `reference.csv` and compare each row against
`vibegraph::helas::compute_M2_ee_mumu(sqrt_s, cos_theta)` once the Rust HELAS
implementation is written.

To run the comparison once implemented:

```bash
cargo test --test helas_validation -- --include-ignored
```
