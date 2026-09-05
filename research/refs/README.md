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
| `refs/smeftsim` | https://github.com/SMEFTsim/SMEFTsim | SMEFTsim 3.0 (pinned at tag `v3.0.2`): dimension-6 SMEFT UFO models — the non-SM Lorentz-structure test bed (MIT licence) |

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

### Key paths in smeftsim

| Path | Contents |
|---|---|
| `UFO_models/SMEFTsim_topU3l_MwScheme_UFO/` | The model vibegraph gates against: `topU3l` flavour assumption (U(2)³ quarks, U(3)² leptons), `{m_W, m_Z, G_F}` inputs. 21 particles (incl. the four `NPprop` auxiliary fields `Z1`/`W1±`/`t1`/`H1`, which carry `propagators.py`), 260 Lorentz structures, 904 vertices |
| `UFO_models/SMEFTsim_topU3l_MwScheme_UFO/lorentz.py` | The structure census that sized the `ufo-lorentz` sprint (note 35 §1): `P` 3189, `Metric` 2885, `Epsilon` 846, `Gamma` 137, `ProjM`/`ProjP` 40 each, `Gamma5` 13, `Identity` 3, **no `Sigma`, no `C`**; `**2` powers of momenta; vertices up to six legs |
| `UFO_models/SMEFTsim_topU3l_MwScheme_UFO/restrict_SMlimit_massless.dat` | Every Wilson coefficient zero: the SM-limit card, the loader/parameter oracle needing no new physics |
| `UFO_models/SMEFTsim_topU3l_MwScheme_UFO/restrict_massless.dat` | Every real Wilson coefficient at a fixed non-zero value: one card that turns every structure class on at once |
| `UFO_models/*_alphaScheme_UFO`, `*_U35_*`, `*_MFV_*`, `*_general_*`, `*_top_*` | The other nine flavour/input-scheme variants; not read by any gate |
| `FeynRules_source/`, `Mathematica_notebooks/` | The FeynRules model files the UFOs were exported from (most of the 100 MB checkout); reference only |

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
