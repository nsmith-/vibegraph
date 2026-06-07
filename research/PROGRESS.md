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

#### Phase 3d: Diagnosis + redesign (🔴 PLANNED — `.claude/plans/starry-discovering-pascal.md`)

**Root cause**: `dispatch.rs` collapses a whole `LorentzExpr` to a single chiral tag
(`DispatchKind`) instead of evaluating the full structure. Three compounding bugs:
1. `dispatch_ffv` loses chirality/structure — FFV1 (photon, full γ^μ) defaults to ProjP
   (right-handed only); FFV4 projector tie drops the ProjM term + its coefficient.
2. `LorentzTerm.coeff` is never stored on `VertexTerm` → FFV4's `+2·ProjP` loses the 2.
3. Early `return` in the per-term loop keeps only the *first* term of a multi-term vertex;
   `VertexInfo.terms` comes from a `HashMap`, so the surviving term is hash-order
   nondeterministic — **the primary source of run-to-run non-determinism**, since
   `evaluate_contract_amplitude` *does* sum its terms (asymmetry with the off-shell path).

**Planned fix**: resolve each `LorentzTerm` into a **compile-time rooted contraction
tree**. `topo_sort.rs` already knows `result_leg_idx: Option<usize>` when it builds each
`VertexInfo`; thread it into `from_ufo` and translate the tensor network implicit in each
term into a rooted tree of resolved primitives (`RootedNode`) with the output fiber fixed
at compile time. Eval becomes a double-sum over terms (coupling × `coeff`) with **no early
returns**.
- **Adds** two missing primitives as `SpinorRepr` methods: `project_left`/`project_right`
  (chiral projection, zero the opposite Weyl 2-block) and `scalar_bilinear` (the FFS
  `f̄Γf` contraction); `iosxxx`/`jsixxx` refactored onto them.
- **In scope**: FFV (all orientations incl. fermion-out via project+`GammaV`), FFS, VVS,
  SSS/SSSS — i.e. spinor chains + single Metric/P boson factors.
- **Deferred to future work** (loud `UnsupportedVertex`): `Sigma`/`Epsilon`/`C` and
  genuine higher-rank tensor contractions (VVV, VVVV — terms with ≥2 free non-root vector
  indices). This drops the previous draft's "generic Metric/P contractor."

**Verification target**: `test_eval_m2_ee_mumu_vs_hardcoded` matches to <1e-6 relative
across the 5 angles + a Z-pole point; a determinism test (compile/eval ~20×, bit-identical);
parser + new-trait-method unit tests; generic-vs-reference equivalence tests.

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
