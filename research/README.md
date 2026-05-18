# Research Materials

This directory contains notes, reference implementations, and sample data
supporting the vibegraph LO Monte Carlo event generator.

## Structure

- `notes/` — working notes, algorithm derivations, implementation sketches
- `refs/` — git submodules for reference implementations (MadGraph, ALOHA, UFO, etc.)
- `ufo/` — sample UFO model files for testing (e.g., SM, scalar toy models)

## Adding a Reference Implementation

```bash
git submodule add --depth=1 <url> research/refs/<name>
```

After cloning, populate submodules with:

```bash
git submodule update --init --depth=1
```

## Current References

| Submodule | URL | Purpose |
|---|---|---|
| `refs/feyngraph` | https://github.com/Jens-Braun/FeynGraph | Rust Feynman diagram generator |
| `refs/mg5amcnlo` | https://github.com/mg5amcnlo/mg5amcnlo | MadGraph5: HELAS routines, ALOHA code generation, SM UFO model |

### Key paths in mg5amcnlo

| Path | Contents |
|---|---|
| `HELAS/` | Fortran77 HELAS subroutines (the originals described in KEK-91-11) |
| `aloha/` | ALOHA Python code — generates HELAS-style routines from UFO Lorentz structures |
| `models/sm/` | Standard Model UFO — the canonical input for testing |
| `madgraph/core/` | Diagram generation algorithm (Python) |
