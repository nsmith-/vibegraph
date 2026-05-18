# Overview: LO Monte Carlo Event Generation

## Goal

Build a minimal but correct LO event generator from scratch in Rust,
following the same pipeline used by MadGraph5_aMC@NLO.

## Steps

### 1. Load UFO Model
- Parse Python-format UFO files (particles.py, vertices.py, parameters.py, lorentz.py, couplings.py)
- Build internal representation: particle table, vertex rules, coupling constants

### 2. Enumerate Feynman Diagrams
- Given: initial state (IS) and final state (FS) particle lists
- Output: set of tree-level Feynman diagrams (graphs)
- Algorithm: recursive graph traversal matching vertex rules
- See feyngraph crate for a Rust starting point

### 3. Construct Helicity Amplitude M(h₁, h₂, ..., hₙ)
- For each diagram: contract wavefunctions with vertex factors and propagators
- Use HELAS-style routines: external wavefunctions → internal currents → amplitude
- Sum over all diagrams: M_total = Σ M_diagram
- |M|² = Σ_{helicities} |M_total|²

### 4. Sample Phase Space
- n-body Lorentz-invariant phase space (LIPS):
  dΦ_n = (2π)⁴ δ⁴(p_in - Σp_f) Π_i d³pᵢ/(2Eᵢ(2π)³)
- Use VEGAS for importance-sampled integration over a unit hypercube
- Map: unit cube → physical momenta via Sudakov/recursive decomposition

### 5. Compute Cross Section
- σ = (1/flux) ∫ dΦ_n |M(p)|² 
- Flux = 2 * λ^(1/2)(s, m1², m2²) for 2→n (≈ 2s in massless limit)
- VEGAS gives σ ± δσ and a set of weighted phase-space points

### 6. Generate (Unweighted) Events
- Accept/reject on weight w(p) = |M(p)|² / w_max
- Output: list of four-momenta with flavor labels

## Toy Process to Start

**e⁺e⁻ → μ⁺μ⁻** via a single photon or Z propagator.
- 2→2, tree-level, QED only
- Analytic result known: σ = 4πα²/3s (massless limit)
- Good integration test before tackling more complex processes

## References

- MadGraph paper: https://arxiv.org/abs/1405.0301
- HELAS manual: https://inspirehep.net/literature/336604
- ALOHA paper: https://arxiv.org/abs/1108.2041
- UFO format: https://arxiv.org/abs/1108.2040
- FeynRules: https://arxiv.org/abs/0806.4194
- VEGAS: Lepage, J.Comput.Phys. 27 (1978) 192 — https://inspirehep.net/literature/119196
- feyngraph: https://github.com/Jens-Braun/FeynGraph
- General-purpose event generators for LHC physics: https://arxiv.org/abs/1101.2599
- COMIX/Sherpa (Berends-Giele ME): https://arxiv.org/abs/0808.3674
- Catani-Seymour IR subtraction: https://arxiv.org/abs/hep-ph/9605323

## Beyond LO: NLO and Fixed-Order Calculations

This package targets **LO (tree-level) matrix element generation only**. For context,
here is how the workflow differs at higher orders, and what would need to be added if
this is later extended toward NLO.

### What parton shower programs can do at ME level

Some parton shower frameworks include their own LO ME generators:

- **Sherpa**: Has two built-in LO generators — **AMEGIC** (Feynman diagram based,
  similar architecture to MadGraph) and **COMIX** (Berends-Giele off-shell recursive
  currents, scales better to high multiplicity). Also supports NLO via OpenLoops/BlackHat.
- **Pythia 8**: Primarily interfaces to external ME codes via Les Houches Event Files
  (LHEF); its internal hard-process library covers only specific processes.
- **Herwig 7**: Uses **MATCHBOX** for automated NLO; primarily interfaces external
  one-loop providers (OpenLoops, GoSam).

The Berends-Giele recursion used in COMIX is an interesting alternative to
HELAS diagram-by-diagram evaluation: amplitude computation scales polynomially (not
factorially) with the number of external legs, making it better suited to high-multiplicity
final states.

### NLO structure

At NLO, three new components are required beyond the LO pipeline:

| Component | Description | Tools |
|---|---|---|
| **One-loop amplitudes** | Virtual corrections; loop integrals evaluated with dimensional regularization, producing poles in ε | OpenLoops, GoSam, MadLoop, BlackHat |
| **UV renormalization** | Counterterms to absorb UV divergences; model-dependent analytic work | FeynArts/FeynCalc, FORM |
| **IR subtraction** | Construct local counterterms so real+virtual singularities cancel numerically (Catani-Seymour dipoles, FKS scheme) | Automated in MadGraph5_aMC@NLO, Sherpa |

Unlike LO, NLO requires analytic manipulation of divergent expressions *before*
any numerical integration is possible.

### NNLO and beyond

- **IBP reduction**: All loop integrals expressed in terms of a small basis of
  "master integrals" via Integration By Parts identities (Laporta algorithm).
  Tools: FIRE6, Kira, LiteRed, Reduze.
- **Differential equations**: Master integrals solved analytically/semi-analytically.
- **Sector decomposition**: Numerical evaluation of master integrals with
  overlapping IR divergences. Tools: pySecDec, FIESTA.

### Computer algebra

Fixed-order calculations rely heavily on computer algebra, but the dominant tool is
not Mathematica:

- **FORM** (Vermaseren): Purpose-built for polynomial manipulation of enormous HEP
  expressions — millions of terms, Dirac traces, color algebra. The de facto standard
  for serious amplitude calculations. Faster and more memory-efficient than Mathematica
  for this class of problem.
- **Mathematica + FeynArts + FeynCalc**: Common for pedagogical use and simpler NLO
  calculations. FeynRules (which generates UFO output) is also Mathematica-based.
- **FormCalc**: Uses FeynArts for diagram generation, pipes to FORM/LoopTools for
  tensor reduction and numerical evaluation.

### Summary: LO vs NLO pipeline

```
LO:   UFO → diagrams → HELAS numerical code → VEGAS     (mostly numerical)

NLO:  UFO → diagrams → CAS analytic reduction
                     → UV renormalization
                     → IR subtraction construction
                     → numerical one-loop evaluation
                     → VEGAS over (real + virtual − subtraction)
```