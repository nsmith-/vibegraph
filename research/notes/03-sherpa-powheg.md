# Reference Implementations: Sherpa and POWHEG-BOX-V2

**Status:** Reference material — no action items. Detailed architectural survey of reference implementations.

Surveyed revisions:
- **Sherpa** (`research/refs/sherpa`): `e12c72f` (GitLab `sherpa-team/sherpa`, `master`)
- **POWHEG-BOX-V2** (`research/refs/powheg-box-v2`): `e26982d` (GitLab
  `POWHEG-BOX/V2/POWHEG-BOX-V2`, `master`)

## Purpose

Both codes extend beyond our LO scope:

- **Sherpa/COMIX** — a full MC framework whose COMIX component implements the
  Berends-Giele recursive current algorithm for LO amplitudes (described in
  the COMIX paper, arXiv:0808.3674). It is the main alternative to the
  diagram-enumeration approach in MadGraph.
- **POWHEG-BOX-V2** — a Fortran NLO+PS framework. It is an infrastructure
  layer: users supply Born/virtual/real matrix elements; the BOX provides
  subtraction, integration, Sudakov resummation, and LHE output. Neither
  code generates amplitudes itself — it delegates to whatever ME code the
  user provides.

---

## Part 1 — Sherpa / COMIX

### 1.1 Top-level directory layout

```
COMIX/         Matrix element generator (Berends-Giele)
METOOLS/       Shared ME tools: wavefunctions, Lorentz/color calculators
MODEL/         Particle/model loading, including a UFO loader
PHASIC++/      Phase space and cross-section integration
ATOOLS/        General utilities (4-vectors, Flavour class, spinors)
AMEGIC++/      Legacy diagram-level ME generator (kept for comparison)
CSSHOWER++/    CSS parton shower
DIRE/          DIRE shower
MCATNLO/       MC@NLO interface
```

### 1.2 COMIX — Berends-Giele recursion

The key insight in COMIX vs MadGraph is the **off-shell current recursion**:
instead of enumerating O(n!) Feynman diagrams, build up a tree of currents
bottom-up.  Each leaf is an external wavefunction; each internal node is a
vertex connecting two lower currents.  Complexity is O(3^n) for an n-particle
process.

#### Off-shell Current

| File | Lines | Class | Description |
|---|---|---|---|
| `METOOLS/Explicit/Current.H` | 43–150 | `Current` (abstract) | Base class for all off-shell currents; carries `m_p` (4-momentum), `m_j` (wavefunction objects indexed by helicity), `m_cid` (bitmask of contributing external legs) |
| `METOOLS/Currents/S_C.C` | 10–62 | `CS<SType>` | Scalar current |
| `METOOLS/Currents/F_C.C` | — | `CF<SType>` | Fermion/spinor current |
| `METOOLS/Currents/V_C.C` | 10–85 | `CV<SType>` | Vector (gluon/photon) current |

The `m_cid` bitmask is the compact ID: bit i is set if external particle i
contributes.  This is used to detect when a current can be combined with
another to form a higher-level current.

#### Vertex evaluation (the recursion step)

| File | Lines | Method | Description |
|---|---|---|---|
| `METOOLS/Explicit/Vertex.H` | 17–128 | `class Vertex` | Connects two child currents via a vertex factor |
| `METOOLS/Explicit/Vertex.C` | 102–179 | `Vertex::Evaluate()` | **Core recursion**: J = sum_{colors,helicities} V × J₁ × J₂; loops over (h₀,h₁,c₀,c₁) combinations, calls `Color_Calculator` then `Lorentz_Calculator`, accumulates into parent current |

#### Lorentz and color calculators

| File | Lines | Class | Description |
|---|---|---|---|
| `METOOLS/Explicit/Lorentz_Calculator.H` | 10–41 | `Lorentz_Calculator` (virtual) | Performs Lorentz tensor contractions (V·J₁·J₂ → J_out); one subclass per Lorentz structure (FFV, VVV, SSV, …) |
| `METOOLS/Explicit/Color_Calculator.H` | 14–74 | `Color_Calculator` (virtual) | SU(3) color algebra; carries color representation info in `CInfo{m_cr, m_ca}` |
| `METOOLS/Explicit/Color_Calculator.C` | — | — | Implementations for S-F, S-T, F-A, A-A color contractions |

**Color treatment**: currents carry explicit color indices throughout the
recursion.  Color is summed only at the end in `Amplitude::EvaluateAll()`.
This is "color-dressed Berends-Giele", in contrast to MadGraph which sums
color at the end of each diagram.

#### Amplitude orchestration

| File | Lines | Class/Method | Description |
|---|---|---|---|
| `COMIX/Amplitude/Amplitude.H` | 74–293 | `class Amplitude` | Owns `Current_Matrix m_cur[n]` (currents organized by number of external legs), `Spin_Structure<DComplex> m_ress` (result for each helicity combination) |
| `COMIX/Amplitude/Amplitude.C` | ~1375 | `Amplitude::EvaluateAll()` | Entry point: calls `CalcJL()` to build all currents; accumulates |M|² over color flows at lines 1435–1456 |
| `COMIX/Amplitude/Amplitude.H` | ~173 | `CalcJL()` | Recursive current evaluator: builds `m_cur[2]`, `m_cur[3]`, … from `m_cur[1]` (external wavefunctions) |

#### Helicity storage

| File | Lines | Class | Description |
|---|---|---|---|
| `METOOLS/Main/Spin_Structure.H` | 18–80 | `Spin_Structure<Value>` | Flat vector indexed by helicity ID; constructed from `Flavour_Vector`, allocates (2s+1) slots per particle |
| `METOOLS/Main/Polarization_Index.H` | — | `Polarization_Index` | Maps helicity tuples ↔ linear index |

#### Process entry points

| File | Lines | Class/Method | Description |
|---|---|---|---|
| `COMIX/Main/Single_Process.H` | 17–118 | `Single_Process` | Single subprocess; `Partonic(Vec4D_Vector,…)` → calls amplitude; `GetAmplitude()` returns `Amplitude*` |
| `COMIX/Main/Process_Group.H` | 11–44 | `Process_Group` | Manages multiple `Single_Process` instances (initial-state crossings, color flows) |

### 1.3 MODEL — model and UFO loading

Sherpa has a native UFO loader, unlike MadGraph which generates Python code
from UFO and then imports it.

| File | Lines | Class/Method | Description |
|---|---|---|---|
| `MODEL/UFO/UFO_Model.H` | 11–44 | `UFO_Model : Model_Base` | Reads UFO files; `ModelInit()` populates particles and vertices; `FillLorentzMap()` maps UFO Lorentz structures to internal METOOLS format |
| `MODEL/UFO/UFO_Param_Reader.H` | — | `UFO_Param_Reader` | Reads UFO `parameters.py`-style parameter cards |
| `MODEL/UFO/UFO_Color_Functions.H` | — | — | UFO color structure representations |
| `MODEL/Main/Model_Base.H` | — | `Model_Base` | Owns vertex map, coupling map, particle database |
| `MODEL/Main/Single_Vertex.H` | — | `Single_Vertex` | Internal vertex: Lorentz structure + color structure + coupling |
| `MODEL/Main/Coupling_Data.H` | — | `Coupling_Data` | alpha_s, alpha_em coupling values |
| `ATOOLS/Phys/Flavour.H` | 79–150 | `Flavour` | Particle type; `StrongCharge()`, `IntCharge()`, `IntSpin()` (×2), `Mass()`, `Width()` |

### 1.4 METOOLS — wavefunction objects

| File | Lines | Class | Description |
|---|---|---|---|
| `METOOLS/Explicit/C_Object.H` | — | `CObject` | Base for all wavefunction objects; carries 4-momentum, color indices, helicity index, complex value |
| `METOOLS/Currents/C_Vector.H` | 12–196 | `CVec4<Scalar>` | Template complex 4-vector; gluon/photon wavefunctions; Lorentz product `operator*` at lines 184–186 |
| `ATOOLS/Phys/Spinor.H` | 12–100 | `Spinor<Scalar>` | Two-component spinor (`m_u1`, `m_u2`); chirality `m_r`; `Construct(Vec4)` builds from 4-momentum; `operator*` gives u·v |

### 1.5 PHASIC++ — phase space and integration

| File | Lines | Class/Method | Description |
|---|---|---|---|
| `PHASIC++/Main/Phase_Space_Integrator.H` | 10–74 | `Phase_Space_Integrator` | Multi-channel adaptive (VEGAS-like) integrator; `Calculate(eps,…)` drives the loop |
| `PHASIC++/Main/Color_Integrator.H` | 64–200 | `Color_Integrator` | Color flow sampling or summing; `GenerateColours()` samples; mode `cls::sum` or `cls::sample` |
| `PHASIC++/Main/Helicity_Integrator.H` | 26–76 | `Helicity_Integrator` | Helicity sampling or summing; `GeneratePoint()` samples; `Optimize()` adapts weights; mode `hls::sum` or `hls::sample` |
| `PHASIC++/Main/Phase_Space_Handler.H` | — | `Phase_Space_Handler` | Manages PS channels, links integrator to process |

### 1.6 Comparison with MadGraph/HELAS

| Aspect | COMIX/Sherpa | HELAS/MadGraph |
|---|---|---|
| Algorithm | O(3^n) Berends-Giele current recursion | O(n!) Feynman diagram enumeration |
| Unit of computation | Off-shell current J^μ | Individual Feynman diagram |
| Color treatment | Color-dressed currents; color summed at end in `EvaluateAll()` | Color-stripped JAMP arrays; summed post-diagram |
| Helicity | `Spin_Structure` carries all combinations; adaptive sampling possible | HELAS routines hardcode spin sums |
| Wavefunction objects | `CObject` hierarchy with embedded color | HELAS spinors/vectors (no color attached) |
| Vertex evaluation | `Vertex::Evaluate()` — one call per vertex node with color+helicity loops | Direct multiplication per diagram node |
| Code structure | Single recursive engine, model-independent | Per-diagram generated Fortran subroutines |
| UFO loading | Native C++ (`UFO_Model`) | Import generated Python |
| Integration | Adaptive multi-channel, separate `Color_Integrator`/`Helicity_Integrator` | Direct phase space with color always summed |
| Practical advantage | Scales better for ≥6 external particles | Cleaner per-diagram book-keeping, easier to audit |

---

## Part 2 — POWHEG-BOX-V2

### 2.1 Architecture: framework vs. process

POWHEG-BOX is not a matrix element generator.  It is an **NLO+PS
infrastructure layer** written in Fortran 77/90.  Users supply process-specific
code; the BOX provides subtraction, integration, Sudakov resummation, and LHE
output.

**What the BOX provides:**
- `btilde.f` — master integrand combining B + V + R - CT
- `sigborn.f` / `sigvirtual.f` / `sigreal.f` — infrastructure around the
  user-supplied amplitude calls
- `integrator.f` — MINT adaptive integration
- `find_regions.f` — singular region identification
- `gen_Born_phsp.f` / `gen_real_phsp.f` — phase space generation scaffolding
- `lhefwrite.f` — LHEF 3.0 output

**What users supply** (one file per subroutine, in a process subdirectory):

| Subroutine | Called from | Purpose |
|---|---|---|
| `setborn(p,bflav,born,bornjk,bmunu)` | `sigborn.f:222` | Born |M|², color-correlated Born `bornjk(j,k)`, spin-correlated Born `bmunu(μ,ν,j)` |
| `setvirtual(p,bflav,virt_arr)` | `sigvirtual.f:42` | One-loop virtual corrections |
| `sigreal_btl(rr)` | `sigreal.f:107` | Real-emission |M|² for each ALR |
| `born_phsp(xborn)` | `gen_Born_phsp.f:6` | Map unit cube → Born phase space momenta |
| `bbinit` | `pwhg_init.f:203` | Process initialization |
| `init_processes` | init phase | Set up `flst_born` / `flst_real` flavour tables |

### 2.2 Main program flow

**File:** `pwhg_main.f` (line 1)

```
pwhginit()                     ! Initialize physics, flags, random seeds
  → bbinit()                   ! User Born initialization
  → mint(..., imode=0)         ! Build integration grid
  → mint(..., imode=1)         ! Integrate with grid

do j = 1, nev
  pwhgevent()                  ! Generate one event (rejection sampling from btilde)
  lhefwritev(iunout)           ! Write to LHE file
enddo
```

**File:** `pwhg_init.f` (line 1), subroutine `pwhginit`
- Line 16: `call init_flsttag` — sets Born/real flavour structure tables
- Line 150: `call init_phys` — alpha_s, masses, PDFs
- Lines 41–114: flags (`flg_bornonly`, `flg_minlo`, `flg_bornzerodamp`, …)

### 2.3 Common block definitions (include files)

All key data is communicated between the BOX and user code via Fortran common
blocks declared in `include/`.

| File | Key variables |
|---|---|
| `include/pwhg_flst.h` | `flst_nborn`, `flst_born(nlegborn,maxprocborn)`, `flst_nreal`, `flst_nalr` |
| `include/pwhg_kn.h` | `kn_pborn(0:3,nlegborn)`, `kn_cmpborn`, `kn_sborn`, `kn_jacborn` |
| `include/pwhg_rad.h` | `rad_tot`, `rad_etot`, `rad_btilde_arr()` |
| `include/pwhg_st.h` | `st_mufact2`, `st_muren2` |
| `include/pwhg_br.h` | `br_born(maxprocborn)`, `br_bornjk`, `br_bmunu` |
| `include/LesHouches.h` | `nup`, `idup(maxnup)`, `pup(5,maxnup)`, `istup`, `icolup` |

### 2.4 The B-tilde integrand

**File:** `btilde.f`, function `btilde(xx,www0,ifirst,imode,retval,retval0)` (line 1)

This is the heart of POWHEG.  The NLO integrand is:

```
d sigma_POWHEG = B(Φ_n)
               + V(Φ_n) - C_integrated(Φ_n)      [ virtual + collinear remnant ]
               + [ R(Φ_{n+1}) - C(Φ_{n+1}) ]      [ real minus local counterterm ]
```

In code (lines 63–101):
- Line 63: `call btildeborn(resborn)` — computes **B**
- Line 68: `call btildevirt(resvirt)` — computes **V** (virtual loop corrections)
- Line 76: `call btildecoll(xrad,rescoll)` — computes **C** collinear remnant
- Line 81: `call btildereal(xrad,resreal)` — computes **R - C** (real minus counterterms)
- Lines 89–101: Sum all contributions: `retval += sum(resborn+resvirt+rescoll+resreal)`

The `ifirst` parameter controls a **folding** mechanism that correlates Monte
Carlo samples from symmetric regions to reduce variance.

### 2.5 Born infrastructure

**File:** `sigborn.f`

| Lines | Subroutine | Description |
|---|---|---|
| 69 | `allborn()` | Loops over all Born configs; calls `setborn0` for each; stores in `br_born`, `br_bornjk`, `br_bmunu` |
| 222 | `setborn0(p,bflav,born,bornjk,bmunu)` | Calls user's `setborn`; divides by `2*kn_sborn` flux; checks for NaN |
| 25–26 | `btildeborn(res)` | `res(j) = br_born(j) * pdf1 * pdf2 * kn_jacborn` — applies PDFs and jacobian |

**File:** `gen_Born_phsp.f`
- Line 1: `gen_born_phsp(xborn)` — calls user's `born_phsp(xborn)`; then computes `kn_csimax` for each FSR emitter

### 2.6 Real-emission and subtraction

**File:** `sigreal.f`, `sigcollremn.f`, `sigcollsoft.f`

Key subroutines:

| File | Lines | Subroutine | Description |
|---|---|---|---|
| `sigreal.f` | 1 | `btildereal(xrad,resreal,www)` | Loops over emitters; calls `gen_real_phsp_fsr`; calls user's `sigreal_btl`; subtracts `rrrc` (collinear CT) + `rrrs` (soft CT) |
| `sigcollremn.f` | 1 | `btildecoll(xrad,rescoll,www)` | Integrated collinear remnant: `∼ log((1-x)/ε) log(s/μ²)` |
| `sigcollsoft.f` | 49 | `collfsr(rc)` | Computes (1-y) ξ² R_collinear for FSR; calls `collfsrnopdf` then multiplies by PDFs |
| `sigvirtual.f` | 1 | `sigvirtual(virt_arr)` | Calls user's `setvirtual` for each Born config; divides by flux; NaN check |

The local counterterm structure is FKS-like (not CS dipoles):
```fortran
! sigreal.f lines 77-78 (FSR):
resreal(iuborn) = resreal(iuborn) + rrr - rrrc - rrrs + rrrcs + remnant
```

### 2.7 Singular region identification

**File:** `find_regions.f`

| Lines | Subroutine | Description |
|---|---|---|
| 21 | `find_regions(a,ares,atags,indexreal,nregions,iregions)` | Identifies all (emitter,radiated) pairs in real graph; returns `iregions(2,nregions)` |
| 52–83 | FSR regions | Loops all final-state pairs (i,j); calls `same_splitting(…)` |
| 85–131 | ISR regions | Loops final-state j vs. initial-state emitter 1 or 2 |
| 135 | `ubornflav(alr)` | For a given ALR, determine underlying Born flavour structure |

### 2.8 MINT integration

**File:** `integrator.f`, subroutine `mint` (line 12)

MINT is a **VEGAS-like adaptive integration** algorithm with a folding
extension.

| Mode | Behavior |
|---|---|
| `imode=0` | Build grid: integrate \|f\| to find equal-contribution intervals; updates `xgrid(0:50,ndim)` |
| `imode=1` | Integration + upper-bound setup; builds `ymax` envelope for event unweighting |

The folding mechanism (`ifold(k)>1`): correlated samples from symmetric
regions are summed before being passed to the integrand, reducing variance for
structured functions.

**File:** `mint_upb.f`
- `startstoremintupb(filetag)` (line 30): opens file for upper-bound storage
- `storemintupb(ndim,ncell,imode,f,f0)` (line 60): writes cell index + function value; used in rejection sampling

### 2.9 LHE output

**File:** `lhefwrite.f`

| Lines | Subroutine | Description |
|---|---|---|
| 3 | `lhefwritehdr(nlf)` | Writes LHEF 3.0 XML header; `<init>` block with beam PDGs, energies, PDF sets, process cross sections |
| 75 | `lhefwritev(nlf)` | Writes one event: `<event>` tag, header line (nup, weight, scales), particle table (PDG ID, status, color, 4-momentum) |
| 124 | `lhefwritetrailer(nlf)` | Closes `</LesHouchesEvents>`; saves RNG state for resumption |

**Particle data format** (line 218):
```fortran
write(buffer,220) idup(i), istup(i), mothup(1,i), mothup(2,i),
                  icolup(1,i), icolup(2,i), (pup(j,i),j=1,5),
                  vtimup(i), spinup(i)
```

### 2.10 Example process layout

A typical user process directory (e.g., `hvq/` for heavy-quark production):
```
hvq/
  nlegborn.h        — defines integer parameter nlegborn
  nlegreal.h        — nlegreal = nlegborn + 1
  maxprocborn.h     — max number of Born subprocesses
  maxprocreal.h     — max number of real subprocesses
  maxalr.h          — max attachment-to-region count
  born.f            — subroutine setborn(...)
  virtual.f         — subroutine setvirtual(...)
  real.f            — subroutine sigreal_btl(...)
  born_phsp.f       — subroutine born_phsp(...)
  init_processes.f  — subroutine init_processes(...)
  bbinit.f          — subroutine bbinit
  Makefile
```

The `hvq/` (heavy quark, q qbar → t tbar) directory is the canonical example
and the most thoroughly documented in the POWHEG-BOX papers.

### 2.11 Relevance to vibegraph

POWHEG-BOX is **not needed for our LO implementation**.  It is included as
the definitive reference for how NLO+PS would be organized if vibegraph were
later extended.  Key lessons:

1. The **user interface** (setborn/setvirtual/sigreal_btl) cleanly separates
   matrix elements from infrastructure — a good design principle for vibegraph.
2. **MINT** is an alternative to VEGAS worth understanding: adaptive grid
   building with folding reduces variance without requiring the user to choose
   importance-sampling variables manually.
3. **B-tilde** is the correct formula to implement if adding POWHEG-style
   NLO+PS (not MC@NLO subtraction).
4. **Color/spin-correlated Born** (`bornjk`, `bmunu`) is needed for NLO
   subtraction but not for LO.

---

## Part 3 — Comparison of All Surveyed Generators

| Aspect | MadGraph5/HELAS | Sherpa/COMIX | POWHEG-BOX |
|---|---|---|---|
| Algorithm | O(n!) diagram enumeration | O(3^n) Berends-Giele recursion | NLO framework; delegates ME to user |
| LO amplitude | HELAS Fortran subroutines, one per diagram node | Recursive `Current` tree with `Vertex::Evaluate()` | User-supplied `setborn` |
| NLO amplitude | MadGraph5\_aMC@NLO (FKS subtraction) | COMIX/CS subtraction in Sherpa framework | User-supplied `setvirtual` + `sigreal_btl` |
| Color treatment | JAMP arrays, summed at diagram level | Color-dressed currents, summed at end | `bornjk` color-correlated Born for subtractions |
| Model loading | Import generated Python from UFO | Native C++ `UFO_Model` | User writes flavour tables; no model loading |
| Integration | MadEvent multi-channel VEGAS | PHASIC++ adaptive multi-channel | MINT adaptive with folding |
| Output format | LHE or internal | HEPMC/LHE | LHEF 3.0 |
| Language | Python + generated Fortran | C++ | Fortran 77/90 |
| Scales well to | Low multiplicity (≤6 legs) | High multiplicity (Berends-Giele advantage) | NLO processes with explicit loop code |
| Primary reference | arXiv:1405.0301 | arXiv:0808.3674 | arXiv:hep-ph/0409146, arXiv:1002.2581 |
