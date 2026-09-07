# Reference Implementations & Papers

Git submodules for upstream code we study or adapt from. Fetched papers live in `papers/` (gitignored).

**Agents: keep this file up to date whenever submodules or paper references are added, removed, or changed.**

## Current Submodules

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

## Fetching submodules

After cloning, populate submodules with:

```bash
git submodule update --init --depth=1
```

To add a new submodule:

```bash
git submodule add --depth=1 <url> research/refs/<name>
git commit -m "research: add <name> reference"
```

## Papers

Fetched papers live in `papers/` (gitignored). Run the fetch script to download all reference PDFs and markdown snapshots:

```bash
bash research/refs/fetch-papers.sh
```

| Key | Description | Format |
|---|---|---|
| `aloha` | ALOHA helicity amplitude generator (arXiv:1108.2041) | markdown |
| `ufo` | Universal FeynRules Output format (arXiv:1108.2040) | markdown |
| `madgraph5` | MadGraph5_aMC@NLO | markdown |
| `madgraph_orig` | Original MadGraph (Stelzer & Long) | markdown |
| `helas` | HELAS manual (KEK-91-11, scanned PDF) | PDF |
| `vegas` | VEGAS+ adaptive importance sampling | markdown |
| `mcreview` | Monte Carlo methods review | markdown |
| `egglog` | egglog: Better Together, Unifying Datalog and Equality Saturation (arXiv:2304.04332) | markdown |
| `egg` | egg: Fast and Extensible Equality Saturation (arXiv:2004.03082) | markdown |
| `madgraph5_beyond` | MadGraph 5: Going Beyond (arXiv:1106.0522) | markdown |
| `madevent` | MadEvent: single-diagram-enhanced multichannel integration (hep-ph/0208156) | markdown |
| `kleiss_pittau` | Weight optimisation in multichannel Monte Carlo (hep-ph/9405257) | markdown |
| `helicity_recycling` | Speeding up MadGraph5_aMC@NLO (arXiv:2102.00773) | markdown |
| `color_flow` | Color-flow decomposition of QCD amplitudes (hep-ph/0209271) | markdown |
| `lhapdf6` | LHAPDF6 (arXiv:1412.7420) | markdown |
| `ckkw` | CKKW: QCD matrix elements + parton showers (hep-ph/0109231) | markdown |
| `lha` | Les Houches Accord, generic user process interface (hep-ph/0109068) | markdown |
| `lhef` | Les Houches Event File format (hep-ph/0609017) | markdown |
| `feynrules2` | FeynRules 2.0 (arXiv:1310.1921) | markdown |
| `slha` | SUSY Les Houches Accord (hep-ph/0311123) | markdown |

The documentation site's bibliography (`docs/src/bibliography.md`) cites a few
pre-arXiv or paywalled papers the script cannot fetch — RAMBO, Lepage's 1978
VEGAS paper, QGRAF, Byckling & Kajantie, Neyman, Schwartz, Zippel — by journal
reference only. The comment at the end of the script's table lists them.

## OCR for scanned PDFs (HELAS)

The HELAS manual (`papers/helas.pdf`) is a scanned document and requires OCR.
We use [Nougat](https://github.com/facebookresearch/nougat) (Meta), which outputs markdown+LaTeX.

The `nougat` pixi environment is configured in `pixi.toml` with the required dependency pins:

| Package | Pin | Reason |
|---|---|---|
| `albumentations` | `<1.4` | `ImageCompression` API changed (int → string for `compression_type`) |
| `pypdfium2` | `<5` | `PdfDocument.render()` removed in v5 |
| `transformers` | `<4.36` | `cache_position` kwarg added to `generate()`, not handled by nougat |

To run OCR:

```bash
pixi run -e nougat ocr
```

Output is written to `papers/helas.mmd` (markdown+LaTeX). This takes several minutes on CPU;
runs faster with an MPS/CUDA GPU available.
