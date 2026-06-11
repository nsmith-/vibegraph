# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser replacing PEG (as of 316598b) |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph (2026-05-27) |
| 3 | HELAS helicity amplitudes (e⁺e⁻→μ⁺μ⁻, hardcoded) | ✅ Done | Validated against MadGraph to <0.1% (Z-pole) |
| 3′ | HELAS generalized (topology-driven, arbitrary process) | 🟡 In Progress | Runtime evaluator agrees with HELAS for massive e⁺e⁻→μ⁺μ⁻ to 1e-7 |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | σ ≈ 2025 pb at √s = 91.2 GeV vs MadGraph ref |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

---

## ✅ Completed

- **e⁺e⁻→μ⁺μ⁻ pipeline** (2d8418a): LIPS + VEGAS + HELAS; σ ≈ 2025 pb at Z-pole vs MadGraph
- **UFO full ownership** (316598b): Python AST parser for particles/vertices/lorentz/couplings
- **Lorentz parse**: PEG parser for Lorentz structure strings → `LorentzExpr` AST
- **Process grammar**: MadGraph-style process string and proc_card parser
- **Diagram enumeration**: feyngraph + UFO model + alias expansion + WEIGHTED coupling-order discovery
- **MadGraph diagram validation**: all 7 processes pass vs MadGraph5_aMC@NLO reference (2026-05-27)

---

## 🔴 High — unblock amplitude generalization

### `lorentz-runtime-eval` — Runtime Lorentz structure evaluator (🟡 IN PROGRESS)

**Status**: Runtime evaluator agrees with Fortran HELAS for massive e⁺e⁻→μ⁺μ⁻ to ~1e-7 relative.
Remaining work: extend node coverage for fermion-out currents, FFS/VVS scalars.

**Completed (2026-06-07/08/10 sessions)**:
1. ✅ `SpinorRepr::{project_left,project_right,scalar_bilinear}` + refactored `iosxxx`/`jsixxx`.
2. ✅ `LorentzEvalTree` + `LorentzEvalNode` DAG in `dispatch.rs`; recursive `build_child()`
   turns undirected UFO tensor network into a directed tree rooted at the output leg.
   Handles Gamma (Vout/Iout/Jout), ProjM/P, ProjMAmp/PAmp, Metric, ScalarProduct.
   Sigma/Epsilon/C raise `CompileError::UnsupportedVertex`.
3. ✅ `VertexTerm.terms: Vec<RootedTerm>` (multi-term support); `WaveformSlot::Add` + `C<F> * WaveformSlot`.
4. ✅ `evaluate_lorentz_node()` tree walker in `run.rs`: implements Leg, GammaVout, ProjM, ProjP, Metric.
   Both `evaluate_off_shell_current` and `evaluate_contract_amplitude` now iterate over rooted trees.
5. ✅ `test_eval_m2_ee_mumu_vs_hardcoded` **passes** (125/125 tests green).
6. ✅ `VectorSpace<F>` trait + `impl_add/mul_for_array!` macros; `Scalar<F>` removed; `GammaV` de-genericized.
7. ✅ **Massive fermion kinematics**: `MDL_ME`/`MDL_MMU` enabled; momenta built as `(E, 0, 0, ±|p|)` with `|p| = sqrt(E²−m²)`.
8. ✅ **Fermion-flow fix**: `GammaVout` node selects `(fo, fi)` by charge rather than fixed order; matches jioxxx/iovxxx convention.
9. ✅ **Propagator momentum flip**: all propagated waveform slots now carry `−q` (outgoing convention), fixing coherent sum cancellation.
10. ✅ **Massive vector propagator**: inline unitary-gauge formula with Fabio fixed-width prescription (replaces `MassiveVectorPropagator`).
11. ✅ **Massless vector propagator**: simplified to `−i/q²` inline, removed `MasslessVectorPropagator` from runtime path.
12. ✅ **`iovxxx` signature**: coupling `[F; 2]` instead of `[C<F>; 2]`; callers updated.
13. ✅ **`Bispinor::dirac_conjugate` → `dirac_adjoint`** rename for clarity.
14. ✅ **`helas_validation` extended test** updated to use `compute_m2_ee_mumu_dynamic`; agrees with Fortran HELAS to <1e-4.
15. ✅ **`spin`/`charge` fields added to `ExtLegInfo`**; propagated from `topo_sort.rs`; removes redundant `ext_spins`/`ext_is_antiparticle` from `AmplitudeEvaluator`.

**Completed (2026-06-11 session)**:
16. ✅ **`SpinorRepr::slash`**: single γ-slash `v̸ = γ^μ v_μ` method; `DiracPropagator`,
    `GammaV`, `fioxxx`/`foxxx` all share it (removed the duplicated `q·σ`/`gamma_v_apply` block algebra).
17. ✅ **`GammaIout`/`GammaJout`** in `evaluate_lorentz_node`: off-shell fermion currents
    (`fioxxx`/`foxxx` analogues). `Iout` slashes the column leg directly; `Jout` adjoints the row
    leg first. Cross-checked vs `fioxxx`/`foxxx` to 1e-10 (`test_eval_off_shell_fermion_vs_fioxxx`).

**Pending**:
- `evaluate_lorentz_node` is `todo!()` for: `ProjMAmp`, `ProjPAmp` (FFS scalar bilinears),
  `ScalarProduct` (multi-factor products)
- `dispatch.rs` `build_child` is `todo!()` for: `P` (momentum insertion) and `Identity` operators
- Add determinism test (compile/eval ~20× → bit-identical results)

_Depends on: `feyngraph-ufo-replace` (✅), `lorentz-parse` (✅)_
_Unblocks: `helas-generalize`_

---

## 🟡 Medium — wire up the generalized evaluator and CLI integration

### `helas-generalize` — Topology-driven HELAS evaluator (PENDING)
Replace the hardcoded `compute_m2_ee_mumu` with the generalized `AmplitudeEvaluator`.
Once the `lorentz-runtime-eval` redesign lands and its tests pass, integrate it into the
cross-section integration pipeline.

**Tasks**:
1. Replace calls to `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2`
2. Update phase-space loop to pass `&DiagramSet` and `&EvaluatedModel`
3. Extend `RootedNode` coverage if needed for new processes
4. Validate against existing hardcoded reference

_Depends on: `lorentz-runtime-eval` (🔴 redesign planned)_
_Unblocks: Process generalization beyond e⁺e⁻→μ⁺μ⁻_

### `global-config` — Implement `vibegraph_lib::config::GlobalConfig` (PENDING)

A thin coordinator that wires `ParsedProcCard` → `UFOModel` loading for the CLI
and future Python/WASM bindings. The parsing side is already done (`ModelImport`
lives in `ParsedProcCard`); this task implements the runtime config module.

```rust
pub struct GlobalConfig {
    pub ufo_search_path: PathBuf,
    pub restrict_path_override: Option<PathBuf>,
}

impl GlobalConfig {
    pub fn model_dir(&self, spec: &ModelImport) -> PathBuf { ... }
    pub fn restrict_card_path(&self, spec: &ModelImport) -> Option<PathBuf> { ... }
    pub fn load_ufo(&self, spec: &Option<ModelImport>) -> Result<UFOModel, UfoError> { ... }
}

#[derive(Debug, Clone, Default)]
pub struct RunConfig {} // placeholder for VEGAS / phase-space tuning
```

Caller flow:
1. `parse_proc_card(content, &ParsingOptions::default())` → `ParsedProcCard`
2. `global_cfg.load_ufo(&card.model)` → `UFOModel`
3. Pass `&UFOModel` to diagram / amplitude / reweighting layers.

Restrict-card resolution: `restrict_path_override` takes precedence; otherwise
look for `<model_dir>/restrict_<variant>.dat` (variant from `ModelImport`), then
fall back to auto-discovery of `restrict_default.dat`.

_Depends on: `helas-generalize` (for pipeline integration)_
_Unblocks: Full CLI with process cards_

---

## 🟢 Later — polish and extensibility

### `feyngraph-perf` — Investigate feyngraph allocation overhead
Profiling (`pixi run profile-diagrams`) shows the hot loop is feyngraph's
recursive topology workspace (`connect_node` / `connect_leg` / `connect_next_class`),
with malloc/free dominating CPU time. Two contributing factors identified:
- feyngraph uses rayon unconditionally; when trials run sequentially the thread-pool
  scheduling adds lock overhead → mitigated with `rayon::ThreadPoolBuilder::num_threads(1)`
  in the test harness, but the per-call allocation pressure remains.
- The recursive backtracking clones partial topology state on every branch, producing
  many small short-lived heap allocations.

**Possible directions (cheapest first):**
1. Try an alternate global allocator (`mimalloc` or `tikv-jemallocator`) — two-line change,
   often 20–40% win on allocation-heavy workloads with no upstream changes required.
2. Upstream contribution to feyngraph: arena-allocate the workspace, or add a cargo feature
   to gate rayon use so the call site can opt into sequential iteration.

### `lips-nbody` — n-body LIPS phase-space generator
Generalize phase-space sampling to 3+ final-state particles using a
recursive 2-body decomposition (RAMBO-style) or Sudakov parametrization.
**Research first:** survey available Rust (and portable Fortran/Python) n-body
phase-space implementations before committing to an approach. Options include
porting MadGraph's phase-space routines or adapting an existing crate.
_Depends on: `xsec-ee-mumu` (✅)_

### `event-output-lhef` — Unweighted events in LHEF format
Accept/reject sampling with `w(p) = |M(p)|²/w_max` to produce an
unweighted event sample.  Serialize to Les Houches Event File (LHEF) format
for downstream tools (Pythia, Herwig, etc.).
_Depends on: `helas-generalize`_

### `typed-units` — Typed physical units throughout the codebase
Research Rust typed-units crates (`uom`, `dimensioned`, `units`) for applicability
to HEP quantities (GeV, mb, etc.). Goal: typed four-momenta and cross sections
throughout to catch dimension errors at compile time. Evaluate ergonomics against
the natural-units convention (ℏ = c = 1) before committing.

---

---

## 🎯 Updated Implementation Plan (2026-06-08)

Active work: completing `lorentz-runtime-eval` — tree-walk evaluator is working for
e⁺e⁻→μ⁺μ⁻; remaining node variants needed for fermion-out currents, FFS scalars, and
general scalar products.

### Immediate (next session) — finish `lorentz-runtime-eval`
1. Implement `GammaIout` / `GammaJout` in `evaluate_lorentz_node` (off-shell fermion currents:
   vector + fermion → fermion; needed for `fioxxx`/`foxxx` analogues).
2. Implement `ProjMAmp` / `ProjPAmp` (chiral scalar bilinears; needed for FFS Yukawa amplitude).
3. Implement `ScalarProduct` (multiply scalar children; needed for SSS/VVS amplitude).
4. Implement `P` and `Identity` in `dispatch.rs` `build_child` (momentum insertion; deferred but
   needed for processes with off-shell scalars carrying momentum).
5. Add determinism test (compile/eval ~20× → bit-identical).

### Short-term (1–2 days) — `helas-generalize`
7. Replace hardcoded `compute_m2_ee_mumu` with `eval_m2` in the cross-section loop;
   validate σ(e⁺e⁻→μ⁺μ⁻) unchanged.
8. Validate a second process (e.g. uū→dd̄) vs MadGraph.

### Medium-term — `global-config`
9. Wire proc-card parsing → UFO loading; CLI flags for model path / restrict card;
   full-pipeline integration test.

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (🔴 redesign) ──→ helas-generalize ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┘
lips-nbody ─────────────────────────────────────────────────────────────────────────────┘
global-config ───────────────────────────────────────────────────────────────────────────┘
```

**Legend**: ✅ = complete, 🔴 = redesign planned, pending = blocked or not started
