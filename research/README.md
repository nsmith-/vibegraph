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
| `refs/sherpa` | https://gitlab.com/sherpa-team/sherpa | Sherpa MC: COMIX Berends-Giele ME generator + UFO loader |
| `refs/powheg-box-v2` | https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2 | POWHEG-BOX-V2: NLO+PS Fortran framework (MINT, B-tilde, LHE output) |

### Key paths in mg5amcnlo

| Path | Contents |
|---|---|
| `HELAS/` | Fortran77 HELAS subroutines (the originals described in KEK-91-11) |
| `aloha/` | ALOHA Python code — generates HELAS-style routines from UFO Lorentz structures |
| `models/sm/` | Standard Model UFO — the canonical input for testing |
| `madgraph/core/` | Diagram generation algorithm (Python) |

### Key paths in sherpa

| Path | Contents |
|---|---|
| `COMIX/Amplitude/` | `Amplitude.{H,C}` — main ME calculator; `CalcJL()` recursion |
| `METOOLS/Explicit/` | `Vertex.{H,C}`, `Current.{H,C}`, `Lorentz_Calculator.H`, `Color_Calculator.H` |
| `METOOLS/Currents/` | `C_Vector.H`, `F_C.C`, `V_C.C`, `S_C.C` — concrete wavefunction types |
| `MODEL/UFO/` | `UFO_Model.{H,C}` — native C++ UFO loader |
| `PHASIC++/Main/` | `Phase_Space_Integrator.H`, `Color_Integrator.H`, `Helicity_Integrator.H` |

### Key paths in powheg-box-v2

| Path | Contents |
|---|---|
| `pwhg_main.f` | Main program; event generation loop |
| `btilde.f` | B-tilde NLO integrand (core POWHEG formula) |
| `sigborn.f` | Born infrastructure (`allborn`, `setborn0`) |
| `sigreal.f` | Real-emission + local counterterm subtraction |
| `integrator.f` | MINT adaptive integration |
| `lhefwrite.f` | LHEF 3.0 output |
| `include/` | Fortran common block headers (pwhg_flst.h, pwhg_kn.h, LesHouches.h, …) |
| `hvq/` | Heavy-quark production — canonical complete example process |
