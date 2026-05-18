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
- VEGAS: Lepage, J.Comput.Phys. 27 (1978) 192 — https://inspirehep.net/literature/119196
- feyngraph: https://github.com/Jens-Braun/FeynGraph
- General-purpose event generators for LHC physics: https://arxiv.org/abs/1101.2599