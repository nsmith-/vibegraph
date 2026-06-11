# lorentz-runtime-eval — COMPLETE ✅

The topology-driven Lorentz structure evaluator is fully implemented and validated.
All 135 tests pass; amplitude agrees with Fortran HELAS to <1e-7 for massive e⁺e⁻→μ⁺μ⁻.

## What was built

### Core primitives
- External wavefunctions: `vxxxxx`, `sxxxxx`, `ixxxxx`/`oxxxxx`
- Propagators: `DiracPropagator`, massive/massless vector (inline), scalar
- Projectors: `GammaL::apply`, `GammaR::apply`; `SpinorRepr::slash`, `project_left/right`, `scalar_bilinear`

### LorentzEvalTree compiler (`dispatch.rs`)
Converts undirected UFO tensor network into a directed eval tree rooted at the output leg.
All SM node types implemented:

| Node | Physics |
|------|---------|
| `Leg(i)` | External wavefunction / off-shell input |
| `GammaVout { i, j }` | γ^μ bilinear → vector (ffV vertex) |
| `GammaIout { mu, j }` | ε̸ψ → fermion out (fioxxx) |
| `GammaJout { mu, i }` | ε̸ψ̄ → fermion out (foxxx) |
| `ProjM/P { i }` | Left/right chiral projector on fermion |
| `ProjMAmp/PAmp { i, j }` | Chiral scalar bilinear (FFS Yukawa) |
| `Metric { mu, nu }` | g^{μν} contraction → scalar |
| `ScalarProduct { children }` | Implicit product of disconnected factors |
| `P { leg }` | Momentum 4-vector of leg (VSS1, VVV1, UUV1) |
| `IdentityAmp { i, j }` | Full bilinear ψ̄_i δ ψ_j (BSM Identity) |

Unsupported (raise `CompileError::UnsupportedVertex`): Sigma, Epsilon, C.

### Runtime evaluator (`run.rs`)
- `evaluate_lorentz_node`: recursive tree walker, implements all node types above
- `WaveformSlot<F>`: register file holding Fermion/Vector/Scalar/Empty; supports `+`, `C<F>*`, `momentum()`
- `evaluate_off_shell_current` / `evaluate_contract_amplitude`: per-vertex entry points
- `evaluate_propagation`: Dirac, massive vector (unitary gauge, Fabio fixed-width), massless vector, scalar

### AST compiler (`topo_sort.rs`, `compile.rs`)
- Recursive depth-first walk from root vertex → topologically ordered `DiagramAst`
- `VertexTerm::from_ufo`: calls `root_term` for each LorentzTerm in the expression
- `ExtLegInfo`: spin + charge populated at compile time; eliminates redundant metadata

### Key design decisions
- **Fermion flow by charge**: `GammaVout` selects `(fo, fi)` by `Charge::Particle/Antiparticle`, not positional order
- **Propagator momentum convention**: all propagated slots carry −q (outgoing)
- **Off-shell scalar output-leg fix**: trivial `Leg(root)` leaf is dropped from build_at_leg; prevents reading the output slot as an input
- **`involves_vector` for P**: only checks `*mu`, not `*leg` (particle index ≠ Lorentz index)

## Next step

`helas-generalize`: replace `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2`
and validate σ(e⁺e⁻→μ⁺μ⁻) + a second process vs MadGraph. See `TODO.md`.
