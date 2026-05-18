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
- Rust crate `feyngraph`: https://crates.io/crates/feyngraph
- Key algorithm: recursive diagram generation from vertices + external particles

### Helicity Amplitudes (HELAS / ALOHA)
- HELAS: Helas Amplitude Subroutines — Fortran routines for wavefunction/vertex computations
- ALOHA: Automatic Libraries Of Helicity Amplitudes — generates HELAS-like code from UFO Lorentz structures
- Reference: https://arxiv.org/abs/1108.2040 (ALOHA), https://arxiv.org/abs/hep-ph/9401258 (HELAS)

### Phase-Space Sampling
- VEGAS: adaptive Monte Carlo integration (Lepage 1978)
- Maps unit hypercube → physical phase space (LIPS measure)
- Rust crate `vegas-rs` or implement directly

### Cross Section
- σ = ∫ dΦ_n |M|² / flux  (integrated over n-body Lorentz-invariant phase space)
- Events are sampled with weight = |M|² / max(|M|²)

## Working Notes

See `research/notes/` for step-by-step derivations and implementation notes.

## Conventions

- Natural units: ℏ = c = 1
- Metric signature: (+, -, -, -)
- Spinor conventions: Weyl/van der Waerden unless noted otherwise
