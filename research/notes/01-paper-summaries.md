# Paper Summaries: Key References for vibegraph

Quick-reference summaries of the core papers, focused on what is
directly relevant to implementing LO event generation in Rust.

---

## ALOHA — Automatic Libraries Of Helicity Amplitudes
**arXiv:** 1108.2041  **fetch:** `https://ar5iv.org/html/1108.2041`

### What it does
Takes a UFO model as input and automatically generates helicity amplitude
routines (HELAS-style) for every vertex in the model. Output can be
Fortran, C++, or Python.

### Algorithm (relevant to our implementation)
1. For each UFO vertex, extract the Lorentz structure tensor.
2. Multiply by each possible combination of external wavefunctions and
   propagators to produce an analytic expression for the internal current
   or amplitude.
3. Write the resulting expression as a callable subroutine.

### Supported field types
Spin 0, 1/2, 1, and 2. Propagators are canonical for each type.

### Key terminology
- **Wavefunction routine:** initialises an external particle (e.g. `ixxxxx`
  for incoming fermion, `oxxxxx` for outgoing fermion, `vxxxxx` for vector).
- **Vertex routine:** contracts two wavefunctions/currents at a vertex to
  produce an off-shell current (e.g. `FFV1_3` = fermion+fermion→vector).
- **Amplitude routine:** final contraction; returns a complex scalar M.

### Relevance to vibegraph
ALOHA defines the calling convention we must implement per-vertex.
When we parse a UFO model we need to auto-generate the equivalent Rust
functions following this pattern: wavefunction → current → amplitude.

---

## UFO — Universal FeynRules Output
**arXiv:** 1108.2040  **fetch:** `https://ar5iv.org/html/1108.2040`

### What it does
Defines a model format as a Python module (not text files). Allows
arbitrary Lorentz and color structures with no a priori assumptions.
Used by MadGraph 5, GoSam, and others.

### Module structure (files in a UFO model directory)
| File | Contents |
|---|---|
| `particles.py` | Particle objects: PDG code, spin, color, mass, width, antiparticle |
| `parameters.py` | External (input) and internal (derived) parameters, e.g. α, mZ |
| `vertices.py` | Vertex objects: list of particles + Lorentz structure refs + coupling refs |
| `lorentz.py` | Lorentz tensor structures as symbolic expressions (e.g. `Gamma(mu,1,2)`) |
| `couplings.py` | Numerical coupling coefficients, potentially color-structured |
| `coupling_orders.py` | QCD/QED order bookkeeping |
| `object_library.py` | Base classes for all objects |

### Key data model
```python
# Example vertex
V_1 = Vertex(name='V_1',
             particles=[P.e__minus__, P.e__plus__, P.a],  # e- e+ γ
             color=['1'],
             lorentz=['FFV1'],
             couplings={(0,0): C.GC_3})                  # -i e γ^μ
```

### Relevance to vibegraph
UFO is the primary input format. We need a Rust parser/loader for the
Python-format files (or a converter to JSON/TOML). The Python files can
be executed with PyO3, or we can write a dedicated parser.
Key insight: UFO is already object-oriented Python — straightforward to
represent as Rust structs.

---

## Original MadGraph (Stelzer & Long)
**arXiv:** hep-ph/9401258  **fetch:** `https://ar5iv.org/html/hep-ph/9401258`

### What it does
First MadGraph: automatically generates Feynman diagrams and HELAS
Fortran code for any tree-level process in the SM.

### Diagram enumeration algorithm (directly relevant)
1. **Topology generation:** start from the unique 3-particle topology.
   Recursively add one external leg — to each existing leg and to each
   vertex — to generate all topologies for n particles.
   - 3 particles: 1 topology
   - 4 particles: 4 topologies
   - 5 particles: 25 topologies
2. **Particle insertion:** for each topology, insert particle flavors
   consistent with vertex rules and quantum number conservation.
3. **Color/symmetry factors:** computed per diagram.
4. **HELAS code generation:** traverse each diagram tree, emit wavefunction
   and vertex calls in the correct order.

### Amplitude evaluation order
- External particles → wavefunctions (`ixxxxx`, `oxxxxx`, `vxxxxx`)
- Combine at each vertex working inward → off-shell currents
- Final vertex → amplitude M (complex scalar)

### Relevance to vibegraph
This paper contains the clearest description of the topology-first
diagram generation algorithm. The FeynGraph crate
(https://github.com/Jens-Braun/FeynGraph) implements this in Rust and
is a primary reference for step 2 of our pipeline.

---

## MadGraph5_aMC@NLO
**arXiv:** 1405.0301  **fetch:** `https://ar5iv.org/html/1405.0301`

### What it does
Complete automation of tree-level and NLO QCD cross section computation,
parton shower matching, and multi-leg merging. The LO (tree-level) subset
is directly relevant to vibegraph.

### LO pipeline (our scope)
1. UFO model loaded → particle/vertex table
2. ALOHA generates helicity routines for all vertices
3. Diagram generation (recursive topologies + particle insertion)
4. Per-phase-space-point: evaluate all diagrams → sum amplitudes → |M|²
5. VEGAS integration over phase space → σ ± δσ
6. Unweighted event generation via accept/reject

### Key design feature
Diagrams are factored: sub-diagrams appearing in multiple diagrams are
cached and evaluated only once. This makes evaluation O(diagrams) rather
than O(diagrams²).

### Relevance to vibegraph
Provides the authoritative description of the full automated pipeline.
Sections 2–4 cover the LO case in detail. Read alongside the older
MadGraph paper for the diagram enumeration algorithm.

---

## HELAS — HELicity Amplitude Subroutines
**Ref:** Murayama, Watanabe, Hagiwara, KEK-91-11 (1992)
**InspireHEP:** https://inspirehep.net/literature/336604
*(Superseded below by full summary from OCR'd document — see second HELAS entry)*

---

## VEGAS — Adaptive Monte Carlo Integration
**Ref:** Lepage, J.Comput.Phys. 27 (1978) 192 — pre-arXiv, no arXiv ID
**VEGAS+ (updated):** arXiv:2009.05112  **fetch:** `https://ar5iv.org/html/2009.05112`

### Classic VEGAS algorithm (importance sampling only)
1. Divide each integration dimension into Ng bins (typical Ng=1000) of
   initially equal width Δxi.
2. Map the integration variable x ∈ [a,b] to y ∈ [0,1] via x(y), where
   equal Δy intervals in y-space map to variable-width Δxi intervals in x-space.
   The Jacobian is J(y) = Ng·Δx_{i(y)}.
3. Evaluate the integrand at Nev random points uniform in y-space. Because
   J ∝ 1/|f(x)| at the optimum, this concentrates samples near peaks in x-space
   (importance sampling — **not** stratification).
4. After each iteration, refine the grid: shrink Δxi where |f| is large
   (so J is small and more y-points map there), widen where |f| is small.
   Optimal condition: J²/Δxi · ∫f²dx = constant across all bins.
5. Repeat for several iterations; combine estimates weighted by 1/σ²_i.

### VEGAS+ additions (arXiv:2009.05112)
- Adds **adaptive stratified sampling** on top of importance sampling.
- Subdivides the unit hypercube into Ns stratification cells per dimension;
  allocates more integrand evaluations to cells with higher variance.
- Much more effective for integrands with multiple peaks or diagonal
  structures that importance sampling alone cannot handle.
- 2–19× improvement over classic VEGAS on relevant problems.
- **For our use case** (single s-channel or t-channel peak in 2→2),
  classic VEGAS importance sampling is likely sufficient.

### Phase space integration
For an n-body final state, LIPS has 3n−4 independent degrees of freedom
(3n momenta, minus 4 from the on-shell δ⁴(p_in−Σp_f), minus nothing for
massless case). VEGAS adapts to peaks from propagators (e.g. 1/(p²−m²)²
divergence near resonance).

### Relevance to vibegraph
The integration driver. Our |M|² × LIPS Jacobian is the integrand; VEGAS
provides σ ± δσ and a set of weighted phase-space points for event generation.
**Open question:** survey available Rust implementations vs. porting
the algorithm directly. See AGENTS.md for this research task.

---

## General-Purpose MC Event Generators Review
**arXiv:** 1101.2599  **fetch:** `https://ar5iv.org/html/1101.2599`

### Scope
Review of Ariadne, Herwig++, Pythia 8, Sherpa for LHC proton-proton
collisions. Covers parton showers, hadronization, underlying event — all
beyond our current scope.

### Relevant sections for vibegraph
- **§3.1** Factorization formula for QCD cross sections (defines what σ_hard is)
- **§3.2** LO matrix-element generators: survey of Alpgen, MadGraph, Sherpa/Comix
  approaches to tree-level diagram generation
- **§3.3–3.4** Scale choices, PDFs (relevant when we extend beyond e+e-)

### Relevance to vibegraph
Good big-picture context for where LO matrix elements fit in the full
simulation chain. Not needed for the initial toy implementation.

---

## FeynRules - Feynman Rules Made Easy
**arXiv:** 0806.4194  **fetch:** `https://ar5iv.org/html/0806.4194`
**Authors:** Christensen & Duhr (2009). Note: FeynRules 2.0 (Alloul et al., arXiv:1310.1921) adds full UFO output support.

### What it does
A Mathematica package that takes a model Lagrangian as input and automatically
derives Feynman rules (interaction vertices), then exports them in the format
required by various ME generators. It is the upstream tool that produces UFO
files consumed by MadGraph5 and ALOHA.

### Workflow
1. User writes a model file declaring: particle content, parameters, and
   the Lagrangian in Mathematica notation.
2. FeynRules applies canonical quantization to extract all interaction vertices.
3. Vertices are stored in a generic internal representation.
4. A translation interface exports to the desired format (UFO, CalcHEP,
   FeynArts, Sherpa, etc.).

### UFO connection
The UFO format (`particles.py`, `vertices.py`, etc.) is precisely the
serialization of FeynRules' internal vertex representation. When we load a
UFO model in vibegraph, we are consuming FeynRules output.

### Key data structures
- **Particle**: `PDG code`, mass symbol, spin, color representation, charge
- **Vertex**: list of participating particles + Lorentz structure + color factor
  + coupling symbol
- **Parameter**: internal (derived) or external (input) with numerical value

### Relevance to vibegraph
Essential context for understanding the UFO file format. We do not need to
run FeynRules ourselves — we consume pre-generated UFO models (e.g. the
Standard Model UFO from the MadGraph model library).

---

## COMIX - A New Matrix Element Generator (Sherpa)
**arXiv:** 0808.3674  **fetch:** `https://ar5iv.org/html/0808.3674` (note: ar5iv fails to render this paper; see PDF at https://arxiv.org/pdf/0808.3674)
**Authors:** Gleisberg & Höche (2008), published in JHEP 0812 (2008) 039.

### What it does
COMIX is the high-multiplicity ME generator inside Sherpa. It uses
**Berends-Giele off-shell recursive currents** rather than summing individual
Feynman diagrams, enabling polynomial (rather than factorial) scaling with
the number of external legs.

### Berends-Giele recursion (the core idea)
Instead of enumerating all diagrams and computing each independently, define
an **off-shell current** J(1,...,n) as the sum over all sub-diagrams
connecting external legs {1,...,n} to a single off-shell leg. The recursion is:

    J(1,...,n) = sum over partitions: V * J(subset_A) * J(subset_B)

where V is the vertex factor. Physical amplitudes are obtained by contracting
the current for all-but-one legs with the remaining external wavefunction.

**Scaling:** O(3^n) operations vs O(n!) diagrams — dramatically better
for 6+ external particles.

### Color/helicity decomposition
COMIX uses color-dressed Berends-Giele currents: currents carry explicit color
indices, summed at the end. Helicity states are handled by computing currents
for each helicity configuration.

### Comparison to HELAS (our approach)
| | HELAS/MadGraph | COMIX/Berends-Giele |
|---|---|---|
| Unit | Individual Feynman diagram | Recursive current |
| Scaling | O(n!) diagrams | O(3^n) recursion steps |
| Code structure | Per-diagram FORTRAN routines | Single recursive function |
| Good for | Low multiplicity (2->2, 2->4) | High multiplicity (2->6+) |

### Relevance to vibegraph
Not used in our initial LO implementation (we follow HELAS/MadGraph). Worth
understanding as an alternative approach if we later target processes with many
final-state particles. The scaling advantage becomes significant above ~6 legs.

---

## Catani-Seymour - General IR Subtraction at NLO
**arXiv:** hep-ph/9605323  **fetch:** `https://ar5iv.org/html/hep-ph/9605323`
**Authors:** Catani & Seymour (1996), published in Nucl.Phys. B485 (1997) 291-419.

### Context
This paper is **not needed for our LO implementation**, but is included as a
reference for future NLO extension. It defines the canonical method for
handling infrared (soft and collinear) divergences in NLO QCD calculations.

### The problem it solves
At NLO, two contributions must be added:
- **Real emission**: m+1 partons in final state — integrand is singular when
  one parton is soft or two partons are collinear.
- **Virtual correction**: m partons + one loop — produces explicit 1/epsilon poles
  in dimensional regularization.

Each piece is separately divergent; their sum is finite for infrared-safe
observables. The challenge is to make both pieces numerically integrable.

### The subtraction method
Introduce a local counterterm dσ^A that:
1. Has the same pointwise singular behaviour as the real emission dσ^R
2. Can be integrated analytically over the one-parton subspace

Then:

    σ^NLO = ∫_{m+1} [dσ^R - dσ^A]_{ε=0}   (finite, integrate numerically)
           + ∫_m    [dσ^V + ∫_1 dσ^A]_{ε=0}  (poles cancel analytically)

### Dipole factorization formulae
The key innovation: the counterterms d^A are built from **dipole terms**,
which factorize the singular limits in a Lorentz-covariant way that smoothly
interpolates between soft and collinear limits. Each dipole involves:
- An **emitter** parton *i*
- An **emitted/spectator** parton *j* (or *k*)
- A reduced m-parton kinematics with a momentum mapping (i,j) -> ĩ

### Appendix C
The most practically useful section: collects all explicit dipole formulae
needed to implement the method. The paper covers all combinations of
final-state and initial-state emitters/spectators, including massive quarks.

### Relevance to vibegraph
**Not needed for the current LO scope.** Included as the essential reference
for "what would have to be added" if we extend to NLO. The C-S dipole method
is used directly inside MadGraph5_aMC@NLO's NLO infrastructure (MadFKS).

---

## HELAS — HELicity Amplitude Subroutines
**Source:** KEK Report 91-11 (1992), not on arXiv; fetched from KEK preprint server
**fetch:** `research/refs/papers/helas.pdf` (scanned); OCR output: `research/refs/papers/helas.mmd`
**Authors:** Murayama, Watanabe & Hagiwara

### What it does
A library of FORTRAN77 subroutines for computing helicity amplitudes of
arbitrary tree-level Feynman diagrams. Given external four-momenta and
helicities, HELAS computes wavefunctions, propagates them through vertices,
and returns the complex amplitude. MadGraph generates calls to HELAS routines
automatically from diagram topology.

### Core architecture
Two building blocks:
1. **Wavefunction subroutines** — compute external particle states from
   four-momenta and helicity quantum numbers
2. **Vertex subroutines** — take wavefunction arrays as input, return either
   an off-shell internal current (for propagation to the next vertex) or a
   scalar amplitude (at the final vertex of a diagram)

### Wavefunction subroutines
All wavefunction outputs are `complex(6)` arrays: 4 Lorentz/spinor components
+ 4-momentum packed as 2 complex numbers.

| Subroutine | Computes | Output symbol |
|---|---|---|
| `IXXXXX(P, FMASS, NHEL, NSF, FI)` | Flowing-in fermion u(p) or v(p) | `\|f>` |
| `OXXXXX(P, FMASS, NHEL, NSF, FO)` | Flowing-out fermion u-bar(p) or v-bar(p) | `<f\|` |
| `VXXXXX(P, VMASS, NHEL, NSV, VC)` | Vector boson polarization ε(p) or ε*(p) | `V^μ` |
| `SXXXXX(P, NSS, SC)` | Scalar boson wavefunction (unity + momentum) | `S` |

**Key parameter conventions:**
- `NHEL`: helicity (+1/-1 for spin-1/2; +1/0/-1 for spin-1)
- `NSF`/`NSV`/`NSS`: +1 = final state (outgoing), -1 = initial state (incoming);
  for fermions, NSF also selects particle (u-spinor) vs antiparticle (v-spinor)
- `P(0:3)`: 4-momentum with P(0) = energy, always positive
- Momentum packed into last 2 complex slots: `(P(0)+iP(3), P(1)+iP(2))`

### Vertex subroutines
For each vertex type, HELAS provides variants that return either a complex
scalar amplitude (all external legs provided) or an off-shell current array
for one of the legs (with propagator factor included).

All vertex types in renormalizable theories:

| Vertex | Amplitude routine | Current routine(s) |
|---|---|---|
| FFV (fermion-fermion-vector) | `IOVXXX` | `FVIXXX`, `FVOXXX` (fermion), `JIOXXX`, `J3XXXX` (vector) |
| FFS (Yukawa) | `IOSXXX` | `FSIXXX`, `FSOXXX`, `HIOXXX` |
| VVV (3-gauge-boson) | `VVVXXX` | `JVVXXX` |
| VVS | `VVSXXX` | `JVSXXX`, `HVVXXX` |
| VSS | `VSSXXX` | `JSSXXX`, `HVSXXX` |
| SSS | `SSSXXX` | `HSSXXX` |
| VVVV (4-gauge-boson) | `WWWWXX`, `W3W3XX` | `JWWWWX`, `JW3WXX` |
| VVSS (seagull) | `VVSSXX` | `JVSSXX`, `HVVSXX` |
| SSSS | `SSSSXX` | `HSSSX` |
| EEA (collinear e-γ) | — | `EAIXX`, `EAOXX`, `JEEXX` |

**Coupling convention for FFV:** `G(2)` array:
- `G(1)` = left-chiral coupling (coefficient of `(1-γ₅)/2`)
- `G(2)` = right-chiral coupling (coefficient of `(1+γ₅)/2`)

### Naming scheme
Subroutine names encode vertex type + input/output roles, padded to 8 chars with `X`:
- Input codes: `I` = flowing-in fermion, `O` = flowing-out fermion, `V` = vector, `S` = scalar
- Output codes: `J` = off-shell vector current, `H` = off-shell scalar current,
  `F` = off-shell fermion current; amplitude routines use all-input naming (e.g. `IOV`)
- Special codes: `E` = collinear electron, `A` = photon (EEA vertex), `W`/`3` = W/Z bosons

### Example: W+W− → tt̄ (4 diagrams, 11 CALL lines)
```fortran
! External wavefunctions
CALL VXXXXX(PWM, WMASS, NHWM, -1, WM)   ! W- incoming
CALL VXXXXX(PWP, WMASS, NHWP, -1, WP)   ! W+ incoming
CALL OXXXXX(PT,  TMASS, NHT,  +1, FO)   ! t outgoing
CALL IXXXXX(PTB, TMASS, NHTB, -1, FI)   ! tbar outgoing (flowing-in)
! Diagrams
CALL J3XXXX(FI, FO, GAU, GZU, ZMASS, ZWIDTH, J3)  ! Z/γ current (s-channel)
CALL VVVXXX(WP, WM, J3, GW, AMPS)                  ! s-channel diagram
CALL FVIXXX(FI, WM, GWF, 0., 0., FVI)              ! off-shell b quark
CALL IOVXXX(FVI, FO, WP, GWF, AMPT)                ! t-channel diagram
CALL HIOXXX(FI, FO, GCHT, HMASS, HWIDTH, HTT)      ! Higgs current
CALL VVSXXX(WM, WP, HTT, GWHH, AMPH)               ! s-channel Higgs diagram
AMP = AMPS + AMPT + AMPH
```

### Utility subroutines
- `MOMNTX(E, M, COSTH, PHI, P)`: construct 4-momentum from energy, mass, angles
- `MOM2CX(SQRT_S, M1, M2, COSTH, P1, P2)`: two-body CM frame momenta
- `BOOSTX(P, Q, R)`: Lorentz boost of momentum P along direction Q
- `ROTXXX(P, COSTH, PHI, R)`: rotate momentum
- `COUP1X`–`COUP4X`: Standard Model coupling constants (VVV, FFV, VVS/Higgs, FFS)

### Conventions (Appendix A)
- Dirac matrices in Weyl (chiral) representation
- Massless spinors defined via reference momentum for longitudinal gauge
- Unitary gauge for weak boson propagators (minimises diagram count)
- Single precision throughout; double precision `DHELAS` available separately

### Relevance to vibegraph
**Central reference for the helicity amplitude module.** The Rust implementation
will mirror this structure:
- Wavefunction functions return `[Complex<f64>; 6]`
- Vertex functions take wavefunction arrays, return current array or amplitude
- FFV coupling maps to `g: [f64; 2]` for chiral left/right components
- ALOHA (see that summary) describes how to auto-generate these routines from
  UFO vertices, extending HELAS to arbitrary BSM models
