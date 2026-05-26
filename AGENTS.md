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

## Repository Layout

```
vibegraph/
├── AGENTS.md           # this file — project context for AI agents
├── Cargo.toml
├── src/                # Rust implementation
└── research/
    ├── README.md       # overview of research materials
    ├── notes/          # working notes, derivations, algorithm sketches
    ├── refs/           # git submodules for reference implementations
    └── ufo/            # sample UFO model files for testing
```

## Key Concepts & References

### UFO (Universal FeynRules Output)
- Python package format describing a BSM/SM model: particles, parameters, vertices, Lorentz structures, couplings
- Reference: https://arxiv.org/abs/1108.2040
- `research/refs/ufo` — canonical UFO repository (if added)

### Feynman Diagram Enumeration
- MadGraph5_aMC@NLO: generates amplitudes from Feynman diagrams
- Rust crate `feyngraph`: https://github.com/Jens-Braun/FeynGraph
- Key algorithm: recursive diagram generation from vertices + external particles

### Helicity Amplitudes (HELAS / ALOHA)
- HELAS: Helas Amplitude Subroutines — Fortran routines for wavefunction/vertex computations
- ALOHA: Automatic Libraries Of Helicity Amplitudes — generates HELAS-like code from UFO Lorentz structures
- Reference: https://arxiv.org/abs/1108.2041 (ALOHA), https://inspirehep.net/literature/336604 (HELAS)

### Phase-Space Sampling
- VEGAS: adaptive Monte Carlo integration (Lepage 1978)
- Maps unit hypercube → physical phase space (LIPS measure)
- **TODO (research phase):** survey available Rust LIPS/VEGAS implementations and choose one
  (vegas-rs is unrelated; options may include porting MadGraph's phase-space routines directly)

### Cross Section
- σ = ∫ dΦ_n |M|² / flux  (integrated over n-body Lorentz-invariant phase space)
- Events are sampled with weight = |M|² / max(|M|²)

## Implementation Patterns & Conventions

### Rust Type System

The codebase leverages Rust's trait system and higher-kinded types extensively:

- **Basis-independence via trait bounds**: Lorentz/spinor/color representations are generic over
  the scalar field `F` to keep physics-layer code independent of representation details.
  For example, `LorentzRepr<F>` works over any `F: Real`.

- **Phantom types for zero-cost abstraction**: Types like `DiracWf` use `PhantomData` to distinguish
  physical meaning (flowing-in vs. flowing-out) at compile time with zero runtime cost.

### Module Organization

The `helas/repr/` submodules are organized by **geometric/physical meaning**, not mathematical type:

- `lorentz.rs` — Lorentz covariance layer (spinors, vectors, scalars, metric)
- `color.rs` — Gauge/color structure (SU(3) fund. and adj. reps, singlet)
- `coupling.rs` — Vertex structures coupling Lorentz and color (e.g., quark-gluon coupling)
- `intertwiner.rs` — Intertwiners (γ^μ, σ^μν, ε^μνρσ) and their leg-count specializations
- `propagator.rs` — Propagator types (Dirac, vector, scalar, with mass terms)

**Import style after recent refactoring**: Direct submodule imports (e.g., `repr::lorentz::Bispinor`)
are preferred over re-exports to avoid unused-import warnings. The `repr/mod.rs` re-exports only the
scalar primitives `Real` and `C<F>` which are used universally.

### Code Style & Conventions

- **Natural units**: ℏ = c = 1 (GeV is the fundamental energy scale)
- **Metric signature**: (+, −, −, −)
- **Comment guidelines**: Avoid narrative comments; add notes only for non-obvious constraints or physics assumptions
- **Constants**: Physical constants (α_QED, m_Z, coupling strengths) are defined at the top level in `helas/mod.rs`
- **Four-momentum layout**: `[E, px, py, pz]` (energy first, spatial components follow)

## Build & Test

```bash
cargo build          # Compile the library and binary
cargo test           # Run all tests (includes helas_validation.rs)
```

## Agent Tooling Guidelines

**Prefer Unix CLI tools over Python scripts for search and extraction tasks.**

Use `grep`, `sed`, `awk`, `find`, etc. instead of writing ad-hoc Python scripts. This enables
auto-approval of these commands in agentic workflows, whereas arbitrary Python scripts require
manual review. Key flags to remember:

- `grep -n` — include line numbers in output
- `grep -r` — recursive search through directories
- `grep -C N` — show N lines of context around each match
- `grep -l` — list only matching filenames
- `sed -n 'N,Mp'` — print lines N through M of a file
- `find . -name "*.rs"` — locate files by pattern

Only write a Python (or other) script when the task genuinely requires logic that these tools
cannot express (e.g., structured parsing of binary formats, complex data transformations).

## Working Notes

See `research/notes/` for step-by-step derivations and implementation notes.

## Open Research Questions

- **Units library:** Research Rust typed-units crates (e.g., `uom`, `dimensioned`, `units`) for
  applicability to HEP quantities (GeV, mb, etc.) — goal is typed four-momenta and cross sections
  throughout to catch dimension errors at compile time.
- **LIPS sampling:** Survey Rust (and portable Fortran/Python) implementations of n-body
  phase-space generators before committing to an approach.
