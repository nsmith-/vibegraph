# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser replacing PEG (as of 316598b) |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph (2026-05-27) |
| 3 | HELAS helicity amplitudes (e⁺e⁻→μ⁺μ⁻, hardcoded) | ✅ Done | Validated against MadGraph to <0.1% (Z-pole) |
| 3′ | HELAS generalized (topology-driven, arbitrary process) | 🔲 Pending | Awaits Lorentz runtime evaluator |
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

### `lorentz-runtime-eval` — Runtime Lorentz structure evaluator (🔴 REDESIGN PLANNED)

**Status**: Runtime evaluator committed (07896fa) but disagrees with the hardcoded
reference for e⁺e⁻→μ⁺μ⁻ **and is non-deterministic across runs**. Root cause diagnosed;
redesign planned (see `.claude/plans/starry-discovering-pascal.md`).

**Root cause** — `dispatch.rs` collapses a whole `LorentzExpr` to a single chiral tag
(`DispatchKind`) instead of evaluating the full structure. Three compounding bugs:
1. `dispatch_ffv` loses chirality/structure: FFV1 (photon, full γ^μ) defaults to ProjP →
   right-handed only; FFV4 projector tie drops the ProjM term + coefficient.
2. `LorentzTerm.coeff` is never stored on `VertexTerm`, so FFV4's `+2·ProjP` loses its 2.
3. Early `return` inside the per-term loop keeps only the *first* term of a multi-term
   vertex; `VertexInfo.terms` comes from a `HashMap`, so which term survives is
   hash-order nondeterministic — **the primary source of run-to-run non-determinism.**

**Redesign** (decided with user): resolve each `LorentzTerm` into a **compile-time rooted
contraction tree**. `topo_sort.rs` already knows `result_leg_idx: Option<usize>` when it
builds each `VertexInfo`; thread that into `from_ufo` and translate the tensor network
implicit in each term into a rooted tree of resolved primitives (`RootedNode`), output
fiber fixed at compile time. Eval becomes a double-sum over terms with **no early
returns**, carrying `coeff` and complex couplings end-to-end.

**Scope**: SM spinor-chain model — Gamma/Proj/Identity spinor chains + single Metric/P
boson factors (FFV all orientations incl. fermion-out, FFS, VVS, SSS/SSSS). Adds two
missing primitives as `SpinorRepr` methods: `project_left`/`project_right` (chiral
projection) and `scalar_bilinear` (FFS contraction); refactors `iosxxx`/`jsixxx` onto
them. **Deferred to future work** (loud `UnsupportedVertex`): `Sigma`/`Epsilon`/`C` and
genuine higher-rank tensors (VVV, VVVV).

**Next steps** (per plan):
1. Add `SpinorRepr::{project_left,project_right,scalar_bilinear}`; refactor `iosxxx`/`jsixxx`.
2. Resolved `RootedNode` descriptors + `root_term` parser; thread `result_leg_idx` through
   `VertexTerm`/`VertexInfo::from_ufo` and `topo_sort.rs`; delete `DispatchKind`.
3. Rewrite the two eval fns (double-sum, no early return): SpinorCurrent, SpinorAmplitude,
   BosonScalar, ScalarProduct — covers e⁺e⁻→μμ + VVS + scalars.
4. SpinorOut (fioxxx/foxxx fermion-out split; FFS fermion-out).
5. Confirm VVV/VVVV/Sigma/Epsilon/C raise `UnsupportedVertex`.

**Verification**: tighten `test_eval_m2_ee_mumu_vs_hardcoded` to <1e-6 relative across 5
angles + a Z-pole point; add a determinism test (compile/eval ~20×, assert bit-identical);
parser + new-trait-method unit tests; generic-vs-reference equivalence tests.

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

## 🎯 Updated Implementation Plan (2026-06-07)

Active work: **`lorentz-runtime-eval` redesign** into a compile-time rooted contraction
tree (full plan in `.claude/plans/starry-discovering-pascal.md`).

### Immediate (next session) — the redesign
1. **Add `SpinorRepr::{project_left,project_right,scalar_bilinear}`** and refactor
   `iosxxx`/`jsixxx` onto them; unit-test against the old inline formulas.
2. **Resolved descriptors + `root_term` parser** in `dispatch.rs`; thread
   `result_leg_idx: Option<usize>` through `VertexTerm`/`VertexInfo::from_ufo` and
   `topo_sort.rs`; delete `DispatchKind`/`dispatch_ffv`; port parser unit tests.
3. **Rewrite the two eval fns** (double-sum, no early return): SpinorCurrent,
   SpinorAmplitude, BosonScalar, ScalarProduct — covers e⁺e⁻→μμ + VVS + scalars.
4. **SpinorOut** (fioxxx/foxxx fermion-out split; FFS fermion-out).
5. **Confirm** VVV/VVVV/Sigma/Epsilon/C raise `UnsupportedVertex` (deferred).
6. **Verify**: tighten `test_eval_m2_ee_mumu_vs_hardcoded` to <1e-6 across 5 angles +
   Z-pole; determinism test; equivalence tests vs reference routines; full suite green.

### Short-term (1–2 days) — `helas-generalize`
7. Replace hardcoded `compute_m2_ee_mumu` with `eval_m2` in the cross-section loop;
   validate σ(e⁺e⁻→μ⁺μ⁻) unchanged.
8. Validate a second process where in scope (e.g. pp→bb) vs MadGraph.

### Medium-term (end of week) — `global-config`
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
