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

## Conventions

- Natural units: ℏ = c = 1
- Metric signature: (+, -, -, -)
- Spinor conventions: Weyl/van der Waerden unless noted otherwise

## Open Research Questions

- **Units library:** Research Rust typed-units crates (e.g., `uom`, `dimensioned`, `units`) for
  applicability to HEP quantities (GeV, mb, etc.) — goal is typed four-momenta and cross sections
  throughout to catch dimension errors at compile time.
- **LIPS sampling:** Survey Rust (and portable Fortran/Python) implementations of n-body
  phase-space generators before committing to an approach.
