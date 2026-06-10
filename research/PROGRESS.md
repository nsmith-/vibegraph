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

#### Phase 3c: AmplitudeEvaluator Runtime (✅ COMMITTED, ❌ INCORRECT — redesign planned)
- **run.rs**: Runtime evaluation loop committed (07896fa)
  - `AmplitudeEvaluator::compile`: resolves external particle IDs, compiles diagrams,
    precomputes helicity combinations, stores leg-count/ordering metadata
  - `eval_amplitude`: walks `DiagramAst`, executes `EvalStep`s in topological order
  - `eval_m2`: helicity-summed driver
- **Status**: Disagrees with the hardcoded reference for e⁺e⁻→μ⁺μ⁻ **and is
  non-deterministic across runs**. Root cause diagnosed (see Phase 3d).

#### Phase 3d: Diagnosis + redesign (🟡 IN PROGRESS — runtime agrees with HELAS for massive e⁺e⁻→μ⁺μ⁻)

**Completed (2026-06-07/08/10 sessions)**:
- ✅ `SpinorRepr::{project_left, project_right, scalar_bilinear}` added to `lorentz.rs`
- ✅ `LorentzEvalTree` + `LorentzEvalNode` DAG in `dispatch.rs`
  - Recursive `build_child()` turns undirected UFO tensor network into directed tree
  - Node types: Leg, GammaVout/Iout/Jout, ProjM/P, ProjMAmp/PAmp, Metric, ScalarProduct
  - Sigma/Epsilon/C raise `CompileError::UnsupportedVertex`; P/Identity are `todo!()`
  - 7 unit tests (FFV1/FFV2/FFS/Yukawa/VVS/SSS/Sigma); all passing
- ✅ `VertexTerm.terms: Vec<RootedTerm>` — each `LorentzTerm` rooted independently
- ✅ `WaveformSlot::Add` + `C<F> * WaveformSlot`; `result_leg_idx` removed from `EvalStep`
- ✅ `evaluate_lorentz_node()` tree walker in `run.rs`
  - Implemented: Leg, GammaVout, ProjM, ProjP, Metric
  - Remaining (`todo!()`): GammaIout, GammaJout, ProjMAmp, ProjPAmp, ScalarProduct
- ✅ `repr/vectorspace.rs`: `VectorSpace<F>` trait + macros; `Scalar<F>` removed; `GammaV` de-genericized
- ✅ `test_eval_m2_ee_mumu_vs_hardcoded` passes — 125/125 tests green
- ✅ **Massive fermion kinematics (2026-06-10)**: enabled `MDL_ME`/`MDL_MMU`; momenta use `|p| = sqrt(E²−m²)`
  in both the hardcoded reference and the runtime evaluator + validation script
- ✅ **Fermion-flow fix**: `GammaVout` selects `(fo, fi)` by charge (particle → outgoing) rather than
  positional order; eliminates sign error in the off-shell current
- ✅ **Propagator momentum convention**: all propagated slots carry `−q` (outgoing); fixes cancellation
  in the coherent amplitude sum
- ✅ **Massive vector propagator**: inline unitary-gauge formula with Fabio fixed-width prescription
  (`m² − imΓ` denominator for longitudinal subtraction), replacing `MassiveVectorPropagator` in runtime
- ✅ **Massless vector propagator**: simplified to `eps * (−i/q²)` inline; removes `MasslessVectorPropagator` from runtime
- ✅ **`iovxxx` signature**: coupling `[F; 2]` instead of `[C<F>; 2]`; callers updated
- ✅ **`Bispinor::dirac_adjoint`** rename (was `dirac_conjugate`) for clarity
- ✅ **`spin`/`charge` fields in `ExtLegInfo`**: populated by `topo_sort.rs`; removes redundant
  `ext_spins`/`ext_is_antiparticle` from `AmplitudeEvaluator`; charge assertion added vs feyngraph
- ✅ **`helas_validation` extended test** updated to call `compute_m2_ee_mumu_dynamic`; passes <1e-4 vs Fortran
- ✅ **`compute_m2_ee_mumu_dynamic`** added to `helas/mod.rs` (feature-gated) as the public runtime entry point

**Remaining**:
- ⏳ Implement `GammaIout`/`GammaJout` in `evaluate_lorentz_node` (off-shell fermion-out currents)
- ⏳ Implement `ProjMAmp`/`ProjPAmp` (FFS chiral scalar bilinears)
- ⏳ Implement `ScalarProduct` (multi-factor product for SSS/VVS/etc.)
- ⏳ Implement `P` and `Identity` in `build_child` (momentum insertion; needed for scalars)
- ⏳ Add determinism test (compile/eval ~20× → bit-identical)

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
