# lorentz-runtime-eval Implementation Progress

## Completed 

### Phase 1: Core Primitives (6/6)
- External vector wavefunction (massless, massive, all 5 cases): `vxxxxx()`
- External scalar wavefunction: `sxxxxx()` + `ScalarWf`
- Dirac propagator: `DiracPropagator::propagate()` (massive case)
- Massive vector propagator: `MassiveVectorPropagator::propagate()` (unitary gauge)
- Left/right projectors: `GammaL::apply()` / `GammaR::apply()`
**Status**: 110+ unit tests passing

### Phase 2: Off-Shell Vertex Routines (5/5)
- **fioxxx**: FFV1_2 vertex (fermion in, vector in → fermion off-shell)
  - Composition: `GammaV::apply()` + `DiracPropagator::propagate()`
  - Momentum: `q = fi.p + V.p`
  - Output: `OutDiracWf`

- **foxxx**: FFV1_1 vertex (fermion out, vector in → fermion off-shell)
  - Similar structure to fioxxx
  - Momentum: `q = fo.p + V.p`
  - Output: `InDiracWf`

- **jvvxxx**: VVV1P0_1 triple-gauge coupling
  - Translated from ALOHA VVV1P0_1.f Fortran
  - Computes 5 Minkowski contractions

- **jsixxx**: FFS Yukawa vertex (fermion pair + scalar)
  - Scalar current from fermion pair
  - Left/right helicity contractions

- **iosxxx**: Final contraction (fermion pair with vector)
  - Direct spinor contraction
  - Supports left/right couplings

**Status**: 118+ unit tests passing; Phase 2 complete and validated

### Phase 3: AST Compiler & Runtime Evaluator (✅ LARGELY COMPLETE)

#### Phase 3a: AST Compiler (✅ COMPLETE)
- **topo_sort.rs**: Recursive tree-walk from arbitrary root vertex
  - Converts undirected diagram graph into directed DAG
  - External legs → `ExternalWf` steps (slots 0..n_ext)
  - Internal vertices → `OffShellCurrent` + `Propagate` (non-root) or `ContractAmplitude` (root)
  - Depth-first traversal ensures all inputs available before evaluation
- **compile.rs**: Wired `compile_diagram_ast` to call `compile_single_diagram` for each diagram
- **Status**: Tests passing for e⁺e⁻→μ⁺μ⁻ (photon and Z diagrams)

#### Phase 3b: Vertex Dispatch (✅ COMPLETE)
- **dispatch.rs**: Pattern-match `LorentzExpr` → `DispatchKind`
  - Maps Lorentz structures to HELAS routine types (FFV, VVV, FFS, VVS, SSS, SSSS, etc.)
  - 10 dispatch tests passing

#### Phase 3c: AmplitudeEvaluator Runtime (🟡 STAGED, INTEGRATION TESTING)
- **run.rs**: Complete runtime evaluation loop (567 lines, staged changes)
  - `AmplitudeEvaluator::compile`:
    - Resolves external particle IDs from `DiagramSet` against `UFOModel`
    - Compiles diagrams via `compile_diagram_ast`
    - Precomputes valid helicity combinations from external spin codes
    - Stores incoming-leg count and external-particle ordering metadata
  - `eval_amplitude`: Walks `DiagramAst`, executes `EvalStep`s in order
    - Reads from input slots, writes to output slots
    - Dispatches `ExternalWf`, `OffShellCurrent`, `Propagate`, `ContractAmplitude` steps
    - Supported dispatch kinds: FFV (ProjM/ProjP), FFS, VVV, VVS, SSS, SSSS
    - Propagators: Dirac (massive), massless/massive vectors, scalars
  - `eval_m2`: Helicity-summed driver (sums |amplitude|² over valid helicity states)
  - Input-shape guards for `momenta`/`helicities` dimensions
  - **Status**: Staged; integration test added but reveals amplitude scale mismatch

**Test Status**: 
- ✅ `cargo test -p vibegraph-lib --lib helas::eval` passes all dispatch/compilation tests
- ✅ `test_eval_m2_ee_mumu_vs_hardcoded` runs without panics; produces valid amplitudes
- 🔴 Amplitude values systematically too low (~15–30% of reference hardcoded implementation)
  - e.g., at √s=100 GeV, cos_θ=0: evaluator=3.75e-2 vs hardcoded=2.05e-1 (5.5× discrepancy)
  - Non-resonant region; suggests systematic issue in evaluator, not kinematics
  - Likely source: fermion wavefunction sign/crossing, charge convention, or contraction order

## Implementation Notes

### Weyl Basis Spinor Indexing
- **InDiracWf (fi)**: indices [0,1]=LEFT-chiral, [2,3]=RIGHT-chiral
- **OutDiracWf (fo)**: indices [0,1]=RIGHT-chiral, [2,3]=LEFT-chiral (after sfomeg swap)
- **Critical**: jsixxx and iosxxx contract indices carefully to match HELAS/ALOHA spinor layout

### Momentum Convention
- HELAS FFV1_2: `P = -(fi.p + V.p)` for propagator denominator
- vibegraph: DiracPropagator expects `q` directly; sign handled internally
- Off-shell currents store accumulated momentum with outflow convention

### Fermion Flow Removal
- Recent commit (6fa66e6): Completely removed fermion flow from AST
  - Flow is implicit in topology (compile-time knowledge of in/out edges)
  - `DiracWf` default phantom type (no flow tag) for off-shell currents
  - `InDiracWf::to_outgoing()` converts when needed for vertex input
  - Simplifies AST representation and eliminates redundancy

### Phase 2 Public API Additions
- **wavefn.rs**: `InDiracWf::from_spinor()` and `OutDiracWf::from_spinor()`
  - Construct off-shell currents from arbitrary spinor+momentum
  - Needed by fioxxx/foxxx implementations

## References
- FFV1_2.f (ALOHA-generated): fioxxx reference at validation/madgraph/output/pp_to_bb/Source/DHELAS/
- FFV1_1.f (ALOHA-generated): foxxx reference
- VVV1P0_1.f (ALOHA-generated): jvvxxx reference
- iosxxx.F (HELAS reference): powheg-box-v2/MadGraphStuff/MadGraph_POWHEG/HELAS/
