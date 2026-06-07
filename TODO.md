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

### `lorentz-runtime-eval` — Runtime Lorentz structure evaluator (🟡 TESTING)

**Status**: 567-line runtime evaluator staged in `helas/eval/run.rs` + integration test added.

**Completed in staged commit**:
- `AmplitudeEvaluator::compile()`: Resolves external particles, compiles AST, precomputes helicity states
- `eval_amplitude()`: Executes `EvalStep` instructions in topological order
  - Dispatch: `ExternalWf` → `OffShellCurrent` → `Propagate` → `ContractAmplitude`
  - Supported terms: FFV (ProjM/ProjP), FFS, VVV, VVS, SSS, SSSS
  - Propagators: Dirac (massive), massless/massive vectors, scalars
- `eval_m2()`: Helicity-summed driver; sums |amplitude|² over all valid helicity states
- Helper functions: slot extractors, complex/Lorentz arithmetic

**Test Status**:
- ✅ Integration test `test_eval_m2_ee_mumu_vs_hardcoded` runs without errors
- ✅ Generates correct helicity combinations (16 for e⁺e⁻→μ⁺μ⁻)
- ✅ Evaluator produces non-zero, finite amplitudes across angles
- 🔴 **AMPLITUDE SCALE MISMATCH**: Runtime evaluator gives ~15–30% of hardcoded reference
  - Suggests bug in wavefunction construction or vertex dispatch
  - Consistent across angles; not helicity counting issue
  - Likely in: fermion/antiparticle crossing logic, charge assignment, or vertex contraction sign

**Next steps**:
1. Debug amplitude scale factor (compare individual helicity terms vs hardcoded)
2. Commit staged changes with updated test notes once root cause identified
3. Fix underlying issue in evaluator or test setup
4. Extend `dispatch.rs` for additional vertex types if needed

_Depends on: `feyngraph-ufo-replace` (✅), `lorentz-parse` (✅)_
_Unblocks: `helas-generalize`_

---

## 🟡 Medium — wire up the generalized evaluator and CLI integration

### `helas-generalize` — Topology-driven HELAS evaluator (PENDING)
Replace the hardcoded `compute_m2_ee_mumu` with the generalized `AmplitudeEvaluator`.
Once the staged `eval_amplitude` tests pass, integrate it into the cross-section integration pipeline.

**Tasks**:
1. Replace calls to `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2`
2. Update phase-space loop to pass `&DiagramSet` and `&EvaluatedModel`
3. Extend `DispatchKind` coverage if needed for new processes
4. Validate against existing hardcoded reference

_Depends on: `lorentz-runtime-eval` (staged, testing)_
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

## 🎯 Updated Implementation Plan (2026-06-06)

### Immediate (next session)
1. **Test staged `run.rs` implementation** (15–30 min)
   - Run `cargo test -p vibegraph-lib --lib` to confirm no regressions
   - Add integration test: `eval_m2` against e⁺e⁻→μ⁺μ⁻ hardcoded reference
   - Commit if all tests pass

2. **Extend dispatch coverage if needed** (30 min–1 hour)
   - Review `dispatch.rs` for unhandled vertex types
   - Add new `DispatchKind` variants if required for broader processes
   - Update `eval_amplitude` helper functions for new terms

3. **Integration test: pp→bb with generalized evaluator** (1–2 hours)
   - Use existing feyngraph/diagram infrastructure
   - Compare sample diagram amplitudes against MadGraph
   - Confirm FFV + VVV dispatch works correctly

### Short-term (1–2 days)
4. **Replace hardcoded `compute_m2_ee_mumu` with `eval_m2`** (30 min–1 hour)
   - Update integration loop in `cross_section` module
   - Validate σ(e⁺e⁻→μ⁺μ⁻) unchanged vs hardcoded reference
   - Clean up old code

5. **Validate process generalization** (1–3 hours)
   - Test at least one new process (e.g., pp→bb, e⁺e⁻→tt̄)
   - Check σ against MadGraph reference (if available)
   - Log dispatch statistics (vertex types seen, dispatch hit rates)

### Medium-term (end of week)
6. **Implement GlobalConfig CLI glue** (2–4 hours)
   - Wire up proc card parsing → UFO loading
   - Add command-line flags for model search path, restrict card, parameter overrides
   - Integration test: full pipeline from proc card to σ

7. **Broader dispatch coverage** (ongoing)
   - Add color structure support if needed
   - Handle additional vertex types (Higgs couplings, etc.)

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (staged) ──→ helas-generalize ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┘
lips-nbody ─────────────────────────────────────────────────────────────────────────────┘
global-config ───────────────────────────────────────────────────────────────────────────┘
```

**Legend**: ✅ = complete, staged = ready for testing/commit, pending = blocked or not started
