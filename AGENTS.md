# vibegraph — Agent Context

## Project Goal

Implement a toy LO (leading-order, tree-level) Monte Carlo event generator in Rust,
studying the standard HEP event simulation toolchain step by step.

## Toolchain Pipeline

```
UFO model data
     │
     ▼
Feynman diagram enumeration  (cf. MadGraph, feyngraph crate)
     │
     ▼
Helicity amplitude construction  (cf. HELAS/ALOHA)
     │
     ▼
Phase-space sampling  (cf. VEGAS adaptive Monte Carlo)
     │
     ▼
Cross-section integral + weighted event sample
```

## Planning & Progress

**Before starting any new feature or task**, read:
- `TODO.md` — prioritized task list
- `research/PROGRESS.md` — research status and decisions already made

**After completing any planned change**, update both files to reflect current status.

## Codebase Exploration

Do not rely on a static layout description. Instead:
- Read `Cargo.toml` (and workspace member `Cargo.toml` files) to understand crate structure and dependencies.
- Use `ls`, `find`, and `grep` to explore the source tree as needed.

## Key Concepts

The pipeline uses standard HEP concepts: **UFO** (Python-based model description format), **Feynman diagram enumeration** (recursive generation from vertices + external legs), **HELAS/ALOHA** (helicity amplitude routines and their automatic generation from UFO Lorentz structures), **VEGAS** (adaptive Monte Carlo integrator), **LIPS** (Lorentz-invariant phase space), and **cross section** (σ = ∫ dΦ_n |M|²/flux, events sampled with weight |M|²/max|M|²).

For paper references, submodule locations and key paths, and instructions for fetching papers and populating submodules, see `research/refs/README.md`. Keep that file up to date when references change.

## Implementation Conventions

### Rust Type System

- **Basis-independence via trait bounds**: Lorentz/spinor/color representations are generic over
  the scalar field `F` to keep physics-layer code independent of representation details.
- **Phantom types for zero-cost abstraction**: Types like `DiracWf` use `PhantomData` to distinguish
  physical meaning (flowing-in vs. flowing-out) at compile time with zero runtime cost.
- **Import style**: Direct submodule imports are preferred over re-exports to avoid unused-import warnings.

### Code Style & Conventions

- **Natural units**: ℏ = c = 1 (GeV is the fundamental energy scale)
- **Metric signature**: (+, −, −, −)
- **Comment guidelines**: Avoid narrative comments; add notes only for non-obvious constraints or physics assumptions
- **Four-momentum layout**: `[E, px, py, pz]` (energy first, spatial components follow)

## Build & Test

```bash
cargo build          # Compile the library and binary
cargo test           # Run all tests
```

### Extended Validation (Fortran HELAS Cross-check)

The `helas_validation` test compares Rust HELAS amplitudes against Fortran77 reference data.
This is a comprehensive but slow integration test, **not run by default**.

**When to run:** after modifying `helas/` representation layer or amplitude computations.

```bash
# Generate reference data (one-time per environment)
pixi run -e helas-validation build-helas
pixi run -e helas-validation gen-reference

# Run the validation test
pixi run -e helas-validation validate-helas
```

## Agent Tooling Guidelines

**Prefer Unix CLI tools over Python scripts for search and extraction tasks.**

Use `grep`, `sed`, `awk`, `find`, etc. instead of writing ad-hoc Python scripts. Only write a
script when the task genuinely requires logic these tools cannot express.

Key flags: `grep -n` (line numbers), `grep -r` (recursive), `grep -C N` (context), `grep -l`
(filenames only), `sed -n 'N,Mp'` (line range), `find . -name "*.rs"`.

## Working Notes

See `research/notes/` for step-by-step derivations and implementation notes.
