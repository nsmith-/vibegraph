# 10 — Lorentz Runtime Evaluator: Detailed Design and Implementation Plan

> **⚠ Implemented, and its future direction superseded (2026-07-08) by
> `13-typed-repr-conventions-design.md`.** This plan shipped as `helas-generalize` (the slot
> machine, `WaveformSlot`, `DiagramAst`/`EvalStep`). Its `DispatchKind` pattern-matching (§6) is
> refined by `13` into the general-IR + peephole/instruction-selection design; the untyped
> `WaveformSlot`/variance policy (§5.1, "meaningless for internal off-shell currents") is exactly
> what `13`'s typed two-level node replaces. Kept as implementation reference.

**Status:** Planning (2026-06-01)  
**Depends on:** note `09` (type matrix ✅), `feyngraph-ufo-replace` (spin_map ✅)  
**Implements:** `lorentz-runtime-eval` TODO item  
**Unblocks:** `helas-generalize`

---

## 1. Goal

Replace the hardcoded `compute_m2_ee_mumu` with a general amplitude evaluator
that works for any process whose diagrams are produced by `generate_from_proc_card`.

The design has two layers:

1. **Compile phase** (once per process): convert each `DiagramView` into a
   compact `DiagramAst` by walking the feyngraph topology and annotating each
   step with the appropriate HELAS routine and UFO coupling constants.

2. **Evaluation phase** (called millions of times in the VEGAS loop): given
   external momenta and a helicity configuration, walk the pre-built AST and
   accumulate the complex amplitude.

No code generation; all HELAS primitives are statically compiled into the
binary and dispatched via a Rust `enum`.

---

## 2. Data-Flow Recap

```
UFOModel
  ├─ particles: IndexMap<String, Particle>      (spin, mass, width, pdg_code)
  ├─ lorentz:   IndexMap<String, LorentzStructure>  (spins, expr: LorentzExpr)
  ├─ couplings: IndexMap<String, Coupling>      (symbolic definition)
  └─ vertices:  IndexMap<String, Vertex>        (particles, lorentz, couplings)

  ↑ name → Id lookups: model.particle_id(name), model.vertex_id(name),
    model.lorentz_id(name), model.coupling_id(name)

EvaluatedModel (model.evaluate(&param_card))
  └─ .coupling(CouplingId) → Complex64  (resolved numerical values)

DiagramSet
  ├─ particles_in:  Vec<String>
  ├─ particles_out: Vec<String>
  └─ diagrams: DiagramContainer
       └─ views() → DiagramView
            ├─ legs() → LegView
            │    └─ .particle().name()  → model.particle_id(name) → ParticleId
            ├─ vertices() → VertexView
            │    └─ .interaction().name()  → model.vertex_id(name) → VertexId
            └─ propagators() → PropagatorView
                 └─ .particle().name()  → model.particle_id(name) → ParticleId

compile_diagram_ast(view, &UFOModel) → DiagramAst
                                            │  (stores Id types throughout,
                                            │   no String keys in hot path)
                                            ▼
AmplitudeEvaluator.eval(momenta, helicities, &EvaluatedModel) → Complex<F>
```

The AST stores `Id` types (`VertexId`, `CouplingId`, `LorentzId`, `ParticleId`)
so all runtime model accesses are O(1) index operations.  Coupling values are
**not** baked in at compile time; they are resolved from `EvaluatedModel` at
eval time so the same AST works with any param card.

---

## 3. Current State

### 3.1 Implemented and validated

| Component | Location | Notes |
|-----------|----------|-------|
| `InDiracWf::new` (`ixxxxx`) | `helas/wavefn.rs` | ✅ validated vs Fortran |
| `OutDiracWf::new` (`oxxxxx`) | `helas/wavefn.rs` | ✅ validated vs Fortran |
| `jioxxx` — FFV off-shell vector current (fo+fi→V) | `helas/vertex.rs` | ✅ |
| `iovxxx` — FFV scalar amplitude (fo+fi+V→C) | `helas/vertex.rs` | ✅ |
| `j3xxxx` — γ+Z combined current | `helas/vertex.rs` | ✅ |
| `ScalarPropagator::propagate` | `repr/propagator.rs` | ✅ |
| `GammaL` / `GammaR` intertwiners | `repr/intertwiner.rs` | ✅ |
| UFO `LorentzExpr` parser | `ufo/lorentz.rs` | ✅ |
| `spin_map` computation | `ufo/lorentz.rs` | ✅ |

### 3.2 Stubs needing implementation

| Item | Location | Phase |
|------|----------|-------|
| `vxxxxx` — external vector wavefunction | `helas/wavefn.rs` (new fn) | 1 |
| `ScalarWf<F>` + `sxxxxx` | `helas/wavefn.rs` (new type) | 1 |
| `DiracPropagator::propagate` | `repr/propagator.rs:108` | 1 |
| `MasslessVectorPropagator::propagate` | `repr/propagator.rs:144` | 1 |
| `MassiveVectorPropagator::propagate` | `repr/propagator.rs:179` | 1 |
| `GammaV::apply` (γ^μ ε_μ on spinor) | `repr/intertwiner.rs:175` | 1 |
| `SigmaTensor::apply` | `repr/intertwiner.rs:203` | 3 |
| `Epsilon::apply` | `repr/intertwiner.rs:231` | 3 |

### 3.3 Missing vertex functions (new, needed for general dispatch)

| Function | Legs | Needed for |
|----------|------|------------|
| `fioxxx` — off-shell fermion from fi+V | fi + V → fo_offshell | QCD quark propagator |
| `foxxx`  — off-shell fermion from fo+V | fo + V → fi_offshell | QCD quark propagator |
| `jvvxxx` — off-shell vector from V+V | V + V → V_offshell | VVV vertex (ggg, WWZ) |
| `ggggxx` — quartic vector amplitude | V+V+V+V → C | VVVV contact (gggg) |
| `jgggxx` — quartic vector current | V+V+V → V_offshell | VVVV off-shell (jgggxx) |
| `jsixxx` — off-shell scalar from fo+fi | fo + fi → S_offshell | FFS (Yukawa) |
| `iosxxx` — FFS scalar amplitude | fo + fi + S → C | FFS amplitude |
| `jssxxx` — off-shell scalar from S+S | S + S → S_offshell | SSS vertex |
| `hvvsxx` — off-shell vector from V+S | V + S → V_offshell | VVS (HWW, HZZ) |
| `svsxxx` — off-shell scalar from V+S | V + S → S_offshell | VVS (H propagator) |

References: ALOHA `template_files/wavefunctions.py`, MadGraph
`madgraph/core/helas_objects.py`, and the existing Fortran77 HELAS library
in `research/refs/papers/helas.pdf`.

---

## 4. Missing Primitive Implementations

### 4.1 `vxxxxx` — external vector (spin-1) wavefunction

**Reference:** `research/refs/mg5amcnlo/aloha/template_files/vxxxxx.cc`

```rust
/// On-shell polarization vector for a spin-1 external particle.
///
/// `nsv = +1` for an incoming particle, `−1` for an outgoing one.
/// `nhel = +1` (right-handed), `−1` (left-handed), `0` (longitudinal),
/// `4` (BRST / longitudinal-check mode).
pub fn vxxxxx(p: LorentzVector<f64>, vmass: f64, nhel: i32, nsv: i32) -> VectorWf<f64>
```

ALOHA convention: `vc[0..1]` hold the momentum (re/im packed), `vc[2..5]`
hold the four polarization components (E, x, y, z split into re/im pairs
packed into 4 complex numbers in the ALOHA 6-element layout).

Vibegraph convention: `VectorWf` stores `eps: ComplexVector([ε_E, ε_x, ε_y,
ε_z])` and `momentum`. The first two elements of the ALOHA output encode
momentum; the last four encode the polarization vector. Map accordingly.

Key branches to implement:
- Massive (`vmass ≠ 0`), `pp = 0` (particle at rest)
- Massive, `pt = 0` (moving along z)
- Massive, general
- Massless, `pt = 0`
- Massless, general

### 4.2 `ScalarWf<F>` and `sxxxxx`

Add to `helas/wavefn.rs`:

```rust
/// External scalar wavefunction: amplitude = 1, momentum stored for routing.
///
/// ALOHA `sxxxxx`: sc[2] = 1+0i, sc[0] = (p0+ip3)*nss, sc[1] = (p1+ip2)*nss.
/// The scalar amplitude is always 1; the momentum is needed for off-shell routing.
pub struct ScalarWf<F: Real> {
    pub value: C<F>,       // always 1+0i for external scalars
    pub momentum: LorentzVector<F>,
}

impl<F: Real> ScalarWf<F> {
    pub fn sxxxxx(p: LorentzVector<F>, nss: i32) -> Self { … }
}
```

### 4.3 `DiracPropagator::propagate`

In the Weyl basis, `q̸ = q_μ γ^μ` has the block form:

```
q̸ = [[0,       q·σ̄],   where σ^μ = (1, σ^x, σ^y, σ^z)
      [q·σ,    0  ]]         σ̄^μ = (1, -σ^x, -σ^y, -σ^z)
```

```
q·σ  = [[q₀+q₃,    q₁-iq₂],    q·σ̄ = [[q₀-q₃,   -q₁+iq₂],
         [q₁+iq₂, q₀-q₃  ]]             [-q₁-iq₂,  q₀+q₃  ]]
```

The 4×4 matrix multiply `(q̸ + m) · ψ` then divides by `q²−m²+imΓ`.
Reference: HELAS paper eq. (2.17); cross-check at the on-shell pole:
`(q̸+m)|_{q²=m²} = Σ_s u_s ū_s`.

### 4.4 `MasslessVectorPropagator::propagate`

Feynman gauge: `Δ^μν = −g^μν/q²`.  Action on fiber: scale all 4 components
by `−1/q²`.  Guard against `q² = 0` (IR/collinear singularity — caller must
ensure q is off-shell).

### 4.5 `MassiveVectorPropagator::propagate`

Unitary gauge numerator: `Nμ = εμ − (q·ε)qμ/m²`  
Denominator: `q²−m²+imΓ` (Breit-Wigner with Fabio fixed-width prescription).

```
ε'_μ = −[ε_μ − (g^{αβ} q_α ε_β) q_μ / m²] / (q²−m²+imΓ)
```

where `g^{αβ}` is the `(+,−,−,−)` Minkowski metric.

### 4.6 `GammaV::apply` — `γ^μ ε_μ` contracted onto an off-shell spinor

This is the spinor-to-spinor map needed when a vector boson line is attached
to a fermion propagator:

```
(ε̸ ψ)_α = (ε_μ γ^μ)_{αβ} ψ^β = [[0,   ε·σ̄], [ε·σ, 0]] · ψ
```

where `ε·σ` and `ε·σ̄` are computed exactly as in the Dirac propagator but
with the complex polarization components instead of real momentum components.
No denominator — this is a pure numerator contraction.

---

## 5. DiagramAst Design

### 5.1 Overview

A `DiagramAst` is a compiled representation of one Feynman diagram.  It is
built once from a `DiagramView` + `UFOModel` and then evaluated rapidly at
each phase-space point.

The evaluation model is a **slot machine**: a fixed-length array of
`WaveformSlot` acts as a register file.  Each `EvalStep` reads from some
slots and writes to one slot.

```rust
/// A runtime wavefunction register (holds one particle's wavefunction).
///
/// The `Flow` phantom tag (`FlowIn`/`FlowOut`) on `DiracWf` enforces
/// correct pairing at function call boundaries but is meaningless for internal
/// off-shell currents.  Slots therefore hold `DiracWf<F>` (= `DiracWf<F, FlowIn>`
/// as a dummy default): correctness for off-shell lines is guaranteed by the AST
/// topology (the compile step knows which vertex leg is "in" vs "out"), not by
/// the type.  External legs are constructed as `InDiracWf`/`OutDiracWf` and then
/// coerced via `DiracWf::from_components` before storing into the slot.
enum WaveformSlot<F: Real> {
    Fermion(DiracWf<F>),    // 4-component Dirac spinor / off-shell fermion current
    Vector(VectorWf<F>),    // 4-component polarization / off-shell vector current
    Scalar(ScalarWf<F>),    // scalar amplitude + momentum
    // Tensor(TensorWf<F>)  future: spin-2
}
```

### 5.2 Compile-time descriptor types

```rust
/// Description of an external leg baked in at compile time.
struct ExtLegInfo {
    leg_idx:     usize,    // 0..n_in are incoming; n_in.. are outgoing
    spin:        i32,      // UFO spin code (1=scalar, 2=fermion, 3=vector)
    mass:        f64,
    is_incoming: bool,
    particle_name: String, // for debugging
}

/// Description of an internal propagator.
struct PropInfo {
    spin:    i32,      // determines which Propagator impl to use
    mass:    f64,
    width:   f64,
    /// momentum = Σ_i coeff[i] * p_ext[i]; i indexes external legs in order
    momentum_coeffs: Vec<i8>,
}

/// One (lorentz_structure, coupling_constant) pair at a vertex.
///
/// `lorentz_id` is stored rather than the resolved `LorentzExpr` so that the
/// AST remains independent of the UFO model reference.  At compile time the
/// `LorentzExpr` is pattern-matched into a `DispatchKind` (see §6) which is
/// what the hot-path dispatch loop actually uses.  `coupling_id` is resolved
/// to a `Complex<F>` at eval time via `EvaluatedModel.coupling(coupling_id)`.
struct VertexTerm {
    lorentz_id:    LorentzId,    // model.lorentz_struct(id) gives LorentzStructure
    dispatch_kind: DispatchKind, // pre-compiled from LorentzExpr at AST build time
    spins:         Vec<i32>,     // per-leg spin codes (from LorentzStructure.spins)
    coupling_id:   CouplingId,   // resolved via EvaluatedModel.coupling(id) at eval time
}

/// Which leg of this vertex is the "output" (receives the off-shell current).
/// Convention: the leg that connects to the rest of the tree (toward the root).
struct VertexInfo {
    terms:          Vec<VertexTerm>, // sum over lorentz × color terms
    result_leg_idx: usize,           // which vertex-local leg is the output slot
    n_legs:         usize,
}
```

### 5.3 Evaluation steps

```rust
enum EvalStep {
    /// Initialize an external wavefunction from momentum + helicity.
    ExternalWf {
        info:      ExtLegInfo,
        /// Index into the runtime helicity array (set at eval time).
        hel_index: usize,
        slot:      usize,
    },

    /// Apply a vertex to compute an off-shell current (all but one leg known).
    OffShellCurrent {
        info:         VertexInfo,
        input_slots:  Vec<usize>,   // slots for the known legs
        output_slot:  usize,        // slot receiving the off-shell wavefunction
    },

    /// Apply a propagator to an off-shell wavefunction.
    Propagate {
        info:     PropInfo,
        in_slot:  usize,
        out_slot: usize,
    },

    /// Final vertex: all legs known → produces a complex scalar amplitude.
    ContractAmplitude {
        info:        VertexInfo,
        input_slots: Vec<usize>,   // all legs (no output slot — writes to amplitude)
        result_slot: usize,        // holds a WaveformSlot::Scalar (Complex64)
    },
}

struct DiagramAst {
    n_ext:          usize,
    n_slots:        usize,
    steps:          Vec<EvalStep>,
    amplitude_slot: usize,  // which slot holds the final Complex64
    symmetry_factor: f64,   // 1 / (vertex_sym × prop_sym)
    fermi_sign:     i8,     // ±1 from the diagram's Fermi permutation
}
```

### 5.4 Topological ordering algorithm

For a tree-level diagram (no loops), the topology is a tree rooted at the
"final vertex" (the one with all legs as known inputs).

**Algorithm (`compile_diagram`):**

1. Mark all external legs as "available".
2. Repeat until no new vertices are found:
   - Find a vertex where exactly one attached propagator is **not yet
     available** (all other legs are either external or already-emitted
     off-shell currents).
   - That vertex emits an off-shell current into the unavailable propagator.
   - Mark that propagator slot as available.
3. The remaining vertex (with all legs available) is the "final vertex" and
   produces a scalar amplitude.

For a 2→2 s-channel diagram (e.g. `e⁺e⁻ → μ⁺μ⁻`):
```
ExternalWf(e⁻) → slot 0
ExternalWf(e⁺) → slot 1
OffShellCurrent(V_73, [slot 0, slot 1]) → slot 4   // jioxxx: fo+fi → V (γ/Z)
Propagate(Z/γ, slot 4) → slot 5
ExternalWf(μ⁻) → slot 2
ExternalWf(μ⁺) → slot 3
ContractAmplitude(V_73, [slot 2, slot 3, slot 5]) → amplitude_slot
```

### 5.5 Helicity iteration

The evaluator stores a list of valid helicity combinations for the process and
iterates over them. For a `1/2 1/2 → 1/2 1/2` process there are 16
combinations; many are zero (helicity conservation). The helicity array is
passed as a parameter to `eval_amplitude`.

For `|M|²` summed over helicities (the integrand), the evaluator loops over
all combinations, coherently sums amplitudes for diagrams of the same topology
(same initial/final states), and returns `Σ_hel |Σ_diag M_diag|²`.

---

## 6. Vertex Dispatch from LorentzExpr

Given a `LorentzExpr` and the spin codes of each leg, we dispatch to the
appropriate HELAS routine. The strategy is **pattern matching on the operator
set** (which `LorentzOp` variants appear) combined with the spin signature.

### 6.1 SM pattern table

| Spins (sorted) | Key operators | Routine(s) needed |
|---------------|--------------|-------------------|
| 2, 2, 3 (FFV) | `Gamma + ProjM` | `jioxxx` (V out), `fioxxx`/`foxxx` (F out), `iovxxx` (scalar) |
| 2, 2, 3 (FFV) | `Gamma + ProjP` | same family, different chiral coupling |
| 2, 2, 1 (FFS) | `Identity` or none | `jsixxx` (S out), `fioxxx` (F out via scalar vertex), `iosxxx` (scalar) |
| 1, 1, 1 (SSS) | none (trivial) | scalar vertex — just multiply coupling × values |
| 1, 1, 1, 1 (SSSS) | none | quartic scalar |
| 3, 3, 3 (VVV) | `Metric + P` | `jvvxxx` (V out) |
| 3, 3, 3, 3 (VVVV) | `Metric × Metric` | `ggggxx` (scalar), `jgggxx` (V out) |
| 2, 2, 3, 3 (FFVV via Sigma) | `Sigma` | future (SMEFT) |
| 3, 3, 1 (VVS) | `Metric` only | `hvvsxx` (V out), `svsxxx` (S out) |

### 6.2 Dispatch function (to implement in `helas/eval/dispatch.rs`)

`DispatchKind` is a pre-compiled enum that encodes the result of pattern-matching
the `LorentzExpr` at AST build time, eliminating any symbolic evaluation on the
hot path:

```rust
/// Pre-compiled dispatch tag, derived from LorentzExpr + spins at compile time.
enum DispatchKind {
    FfvProjM,   // FFV with left-chiral projector (ProjM)
    FfvProjP,   // FFV with right-chiral projector (ProjP)
    Ffs,        // FFS Yukawa (Identity in spinor space)
    Vvv,        // VVV triple gauge (Metric + P)
    Vvvv,       // VVVV quartic gauge (Metric × Metric)
    Vvs,        // VVS Higgs coupling (Metric only)
    Sss,        // SSS
    Ssss,       // SSSS
    // Future: FfvvSigma (SMEFT), ...
}

fn dispatch_vertex<F: Real>(
    step:          &VertexInfo,
    slots:         &mut [WaveformSlot<F>],
    evaluated:     &EvaluatedModel,
) {
    // For each term in step.terms:
    //   1. Look up coupling: evaluated.coupling(term.coupling_id)
    //   2. Match term.dispatch_kind to select the HELAS routine
    //   3. Extract input wavefunctions and momenta from input slots
    //      (momenta are embedded in the wf types — sum them for off-shell routing)
    //   4. Call the selected HELAS routine, scale result by coupling
    // Accumulate sum over terms into the output slot.
}
```

The `LorentzExpr` is only needed at compile time to produce `DispatchKind`;
it is not stored in the AST.  For BSM models with new Lorentz structures, a new
`DispatchKind` variant and corresponding HELAS routine would be added.

---

## 7. New Module Layout

```
vibegraph-lib/src/helas/
  mod.rs               (existing — add pub mod eval)
  wavefn.rs            (add vxxxxx, ScalarWf, sxxxxx)
  vertex.rs            (add fioxxx, foxxx, jvvxxx, jsixxx, iosxxx, etc.)
  repr/
    propagator.rs      (implement DiracPropagator, MasslessVectorPropagator,
                          MassiveVectorPropagator)
    intertwiner.rs     (implement GammaV; later SigmaTensor, Epsilon)
  eval/                (NEW)
    mod.rs             (AmplitudeEvaluator, pub API)
    ast.rs             (DiagramAst, EvalStep, WaveformSlot, descriptor types)
    compile.rs         (compile_diagram_ast: DiagramView × UFOModel → DiagramAst)
    dispatch.rs        (vertex dispatch: VertexInfo × slots → result)
    run.rs             (eval_amplitude: DiagramAst × momenta × helicities → C<f64>)
```

---

## 8. Public API

```rust
// helas/eval/mod.rs

/// Compiled amplitude evaluator for all diagrams of a process.
/// The AST is built once from `&UFOModel`; coupling values are resolved at
/// eval time from `&EvaluatedModel` so the same evaluator works with any
/// param card.
pub struct AmplitudeEvaluator {
    diagram_asts: Vec<DiagramAst>,
    n_ext:        usize,
    helicities:   Vec<Vec<i32>>,  // all helicity combinations (precomputed)
}

impl AmplitudeEvaluator {
    /// Compile from a DiagramSet + UFO model (symbolic, no param card needed).
    pub fn compile(set: &DiagramSet, model: &UFOModel) -> Result<Self, EvalError>;

    /// Evaluate |M|² summed over all helicities.
    ///
    /// `momenta[i]` is the 4-momentum of the i-th external particle
    /// in the order: incoming legs first, then outgoing legs.
    pub fn eval_m2<F: Real>(&self, momenta: &[LorentzVector<F>], evaluated: &EvaluatedModel) -> F;

    /// Evaluate the complex amplitude M for a single helicity configuration.
    pub fn eval_amplitude<F: Real>(&self, momenta: &[LorentzVector<F>], helicities: &[i32], evaluated: &EvaluatedModel) -> Complex<F>;
}
```

---

## 9. Implementation Phases

### Phase 1 — Missing primitives (enables `e⁺e⁻ → μ⁺μ⁻` via AST, cross-check)

1. Implement `vxxxxx` in `helas/wavefn.rs`  
2. Add `ScalarWf<F>` + `sxxxxx` to `helas/wavefn.rs`  
3. Implement `DiracPropagator::propagate` in `repr/propagator.rs`  
4. Implement `MasslessVectorPropagator::propagate`  
5. Implement `MassiveVectorPropagator::propagate`  
6. Implement `GammaV::apply` in `repr/intertwiner.rs`

### Phase 2 — Off-shell current routines

7. `fioxxx` / `foxxx` — fermion off-shell current (needed for QCD quark line)  
8. `jvvxxx` — vector off-shell current (needed for `u ū → g g`)  
9. `jsixxx` / `iosxxx` — FFS Yukawa routines  
10. `jvvxxx` scalar variants (`hvvsxx`, `svsxxx`) for Higgs processes

### Phase 3 — AST compiler

11. Define `DiagramAst`, `EvalStep`, and descriptor types in `eval/ast.rs`  
12. Implement `compile_diagram_ast` in `eval/compile.rs`:
    - Enumerate legs and look up UFO particle data for spin/mass/width
    - Topological sort of the vertex dependency graph
    - For each vertex: identify which leg is the "output" and which are inputs
    - Look up the UFO vertex by `interaction().name()` → `UFOModel.vertices`
    - For each lorentz index in the vertex, extract `LorentzExpr` and coupling
    - Emit `EvalStep::OffShellCurrent` or `EvalStep::ContractAmplitude`
    - Emit `EvalStep::Propagate` for each internal propagator

### Phase 4 — Dispatch and evaluation

13. Implement `dispatch_vertex` in `eval/dispatch.rs`:
    - Pattern-match spin signature + LorentzOp set
    - Call the appropriate vertex function from `helas/vertex.rs`  
14. Implement `eval_amplitude` in `eval/run.rs`:
    - Allocate a `Vec<WaveformSlot>` of size `ast.n_slots`
    - Execute each `EvalStep` in order
    - Return `slots[ast.amplitude_slot]` as `Complex<f64>`

### Phase 5 — Integration and validation

15. Implement `AmplitudeEvaluator::compile` and `eval_m2` in `eval/mod.rs`  
16. Test `e⁺e⁻ → μ⁺μ⁻`: `eval_m2` must agree with `compute_m2_ee_mumu`
    within numerical precision at multiple kinematic points  
17. Test `u ū → g g`: compare against MadGraph reference at fixed kinematics  
18. Benchmark: confirm `eval_m2` is fast enough for VEGAS (target: < 1 μs
    per phase-space point per diagram at 2→2)

---

## 10. Cross-reference to Existing Code

| Needed capability | Where it currently lives |
|------------------|--------------------------|
| LorentzId from UFO lorentz name | `model.lorentz_id(name)` → `LorentzId` |
| LorentzExpr (compile time only) | `model.lorentz_struct(id).expr` |
| CouplingId | `model.coupling_id(name)` → `CouplingId` |
| Coupling value (eval time) | `evaluated.coupling(CouplingId)` → `Complex64` |
| Particle mass/width (eval time) | `evaluated.mass(ParticleId)` / `evaluated.width(ParticleId)` |
| Particle spin | `model.particle(id).spin` (UFO 2s+1) |
| VertexId from feyngraph name | `model.vertex_id(interaction.name())` → `VertexId` |
| Vertex lorentz/coupling indices | `model.vertex_def(VertexId).lorentz` / `.couplings` |
| Momentum routing coefficients | `PropagatorView::momentum()` → `Vec<i8>` coeffs |
| Diagram sign (Fermi permutation) | `DiagramView::sign()` → `i8` |
| Symmetry factor | `DiagramView::symmetry_factor()` → `usize` |

---

## 11. Open Questions

1. **Color factors.** Tree-level color contractions for the SM are simple
   (trace of generators for QCD, delta for EW). Color is currently decoupled
   from the Lorentz amplitude evaluation. For the first implementation, color
   factors can be computed separately and multiplied into the coupling constant
   at compile time. A proper color-flow decomposition (note `08`) is a future
   refinement.

2. **Coherent vs. incoherent diagram sum.** For a multi-diagram process
   (e.g. `e⁺e⁻ → μ⁺μ⁻` with γ and Z), amplitudes must be summed
   **coherently** before squaring: `|M_γ + M_Z|²`. The `AmplitudeEvaluator`
   should sum all `DiagramAst` amplitudes first, then take the norm squared.

3. **Gauge choice for massless vector propagators.** Feynman gauge
   (`−g_μν/q²`) is simpler and sufficient for tree-level; unitary gauge is
   only needed if we want to expose the longitudinal mode (e.g. for checking
   Ward identities). Use Feynman gauge first.

4. **`SigmaTensor` and `Epsilon`.** These are needed for SMEFT operators and
   Majorana mass terms but not for the SM at tree level. Defer to a future phase
   (higher-spin/BSM extensions) unless a concrete SM process requires them.

5. **Spin-3/2 and spin-2 external wavefunctions.** The SM has no spin-3/2 or
   spin-2 particles. These (`txxxxx` and graviton wavefunctions) are deferred
   to a future phase of the project and are not needed for this task.
