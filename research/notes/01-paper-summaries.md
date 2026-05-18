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
*(InspireHEP renders as a JS SPA — not directly fetchable. Get the KEK
preprint from https://lib-extopc.kek.jp/preprints/PDF/1991/9124/9124011.pdf)*

### What it does
The original Fortran library of helicity amplitude subroutines. Defines
the calling convention that ALOHA and MadGraph are built on.

### Key routine families
| Prefix | Type | Description |
|---|---|---|
| `ixxxxx` | wavefunction | Incoming fermion (u-spinor) |
| `oxxxxx` | wavefunction | Outgoing fermion (v-spinor) |
| `vxxxxx` | wavefunction | External vector boson (polarization ε^μ) |
| `sxxxxx` | wavefunction | External scalar |
| `FFVx_y` | vertex+current | Fermion-fermion-vector → off-shell current |
| `VVVx_y` | vertex+current | Triple gauge vertex |
| `FFVx_0` | amplitude | Fermion-fermion-vector → complex scalar M |

### Wavefunction convention
Each wavefunction is a fixed-size complex array. For spin-1/2: 6 elements
(4 spinor components + 4-momentum packed in last 2). For spin-1: 6 elements
(4 polarization components + 4-momentum).

### Relevance to vibegraph
Defines the data layout for wavefunctions and the function signatures we
need to implement in Rust. The "array of complex numbers" representation
maps naturally to `[Complex<f64>; N]`.

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
