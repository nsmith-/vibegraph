# lorentz-runtime-eval Implementation Progress

## Completed 

### Phase 1: Core Primitives (6/6)
-   external vector wavefunction (massless, massive, all 5 cases)vxxxxx() 
-  sxxxxx() +  external scalar wavefunctionScalarWf 
- g MassiveVectorPropagator:: unitary gauge with Fabio fixed-widthpropagate() _{
-  GammaV:: apply() 
**Status**: 110 unit tests passing

### Phase 2: Off-Shell Vertex Routines (5/5)
 fo_offshell  
  - Composition: GammaV::apply() + DiracPropagator::propagate()
  - Momentum: q = fi.p + V.p
  - Output: OutDiracWf
  
 fi_offshell  
  - Similar structure to fioxxx
  - Momentum: q = fo.p + V.p
  - Output: InDiracWf
  
 V_offshell (triple-gauge coupling)
  - Translated from ALOHA VVV1P0_1.f Fortran
  - Computes 5 Minkowski contractions (TMP5)TMP1
  - Supports massless case (mass=0)
  
 S_offshell  
  - Scalar current from fermion pair
  - Left/right helicity contractions
  - Uses ScalarPropagator
  
 amplitude)  
  - Direct spinor contraction with scalar
  - Supports left/right couplings

**Status**: 5 new unit tests, 118 total tests passing (115 active, 3 ignored)

## Immediate Next Steps

### Phase 3a: AST Compiler (✅ COMPLETE)
- **topo_sort.rs**: Recursive tree-walk from arbitrary root vertex
  - Converts undirected diagram graph into directed DAG by rooting at first vertex
  - External legs → ExternalWf steps (slots 0..n_ext)
  - Internal vertices → OffShellCurrent + Propagate (non-root) or ContractAmplitude (root)
  - Depth-first traversal ensures all inputs available before evaluation
- **compile.rs**: Wired compile_diagram_ast to call compile_single_diagram for each diagram
- **Status**: Tests passing for e⁺e⁻→μ⁺μ⁻ (photon and Z diagrams)

### Phase 3b: Vertex Dispatch (pending)
- **dispatch.rs**: Runtime evaluation of DispatchKind variants
  - Map vertex terms (FFV, VVV, FFS, etc.) to HELAS routines
  - Resolve couplings and helicity indices at eval time

### Phase 3c: AmplitudeEvaluator (pending)
- **run.rs**: Runtime evaluation loop
  - Walk DiagramAst, execute EvalSteps in order
  - Read from input slots, write to output slots
  - Final amplitude from amplitude_slot
  - Validation: Compare against hardcoded `compute_m2_ee_mumu`

## Implementation Notes

### Weyl Basis Spinor Indexing
- **InDiracWf (fi)**: indices [0,1]=LEFT-chiral, [2,3]=RIGHT-chiral
- **OutDiracWf (fo)**: indices [0,1]=RIGHT-chiral, [2,3]=LEFT-chiral (after sfomeg swap)
- **Critical**: jsixxx and iosxxx contract indices carefully to match HELAS/ALOHA spinor layout

### Momentum Convention
- HELAS FFV1_2: `P = -(fi.p + V.p)` for propagator denominator
- vibegraph: DiracPropagator expects `q` directly; sign handled internally
- Off-shell currents store accumulated momentum with outflow convention

### Phase 2 Public API Additions
- **wavefn.rs**: Added `InDiracWf::from_spinor()` and `OutDiracWf::from_spinor()`
  - Allow constructing off-shell currents from arbitrary spinor+momentum
  - Needed by fioxxx/foxxx implementations

## References
- FFV1_2.f (ALOHA-generated): fioxxx reference at validation/madgraph/output/pp_to_bb/Source/DHELAS/
- FFV1_1.f (ALOHA-generated): foxxx reference
- VVV1P0_1.f (ALOHA-generated): jvvxxx reference
- iosxxx.F (HELAS reference): powheg-box-v2/MadGraphStuff/MadGraph_POWHEG/HELAS/
