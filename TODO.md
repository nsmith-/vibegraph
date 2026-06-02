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

### `feyngraph-ufo-replace` — Replace `TopoModel::from_ufo()` with vibegraph-built model

Status: **✅ COMPLETE** (as of current implementation)

**Completed**: `compute_spin_map()` function (commit d19a4e7)
- Added `compute_spin_map(expr: &LorentzExpr, n_legs: usize) -> Vec<isize>` to `ufo/lorentz.rs`
- Traces spinor index chains through `Gamma`, `Sigma`, `Identity`, `ProjM`, `ProjP`, `C` operators
- Returns 1-indexed mapping: positive values map external legs, 0 means unmapped
- Updated `LorentzStructure` struct with `spin_map` field
- Integrated into `parse_lorentz()` so all structures have spin_map computed automatically
- Added comprehensive unit tests (FFV1, no-spinor, projector-chain cases)

**Completed**: Enhanced `build_feyngraph_model()` function with proper coupling order handling
- Refactored into `ufo/topo.rs` module for better organization
- Now properly extracts coupling order information from UFO model's coupling definitions
- Ensures feyngraph vertices include accurate coupling orders for proper diagram generation
- Maintains compatibility with existing code while improving accuracy

**Current approach**: Using the enhanced `build_feyngraph_model()` function that properly constructs feyngraph models with accurate coupling orders, while still leveraging vibegraph's computed `spin_map` for the lorentz-runtime-eval task.

**Benefit**: The `spin_map` is now computed and available — it's needed by the Lorentz runtime evaluator since feyngraph's internal spin_map is not public. Full UFO ownership is now fully functional with proper coupling orders.

_Depends on: `lorentz-parse` (✅), `ufo-full-ownership` (✅)_
_Unblocks: `lorentz-runtime-eval` (spin_map is now ready)_

### `lorentz-runtime-eval` — Runtime Lorentz structure evaluator

Walk the `LorentzExpr` AST (parsed by `ufo/lorentz.rs`) and dispatch to pre-compiled primitives
in `helas/repr/`. No code generation — all primitives are statically compiled into the binary
and the AST is interpreted at runtime. This is the generalization of the hardcoded `compute_m2_ee_mumu`.

**Current primitive state** (in `helas/repr/`):
- Working: `GammaL`, `GammaR` (`intertwiner.rs`); `ScalarPropagator` (`propagator.rs`);
  `weyl_ixxxxx`/`oxxxxx` (`lorentz.rs`); `j3xxxx` (`vertex.rs`)
- Need implementation:
  - `GammaV::apply` — `γ^μ` on off-shell spinor current (`intertwiner.rs:176`)
  - `SigmaTensor::apply` — `σ^μν` bilinear (`intertwiner.rs:204`)
  - `Epsilon::apply` — spinor metric `ε_{αβ}` (`intertwiner.rs:232`)
  - `DiracPropagator::propagate` — `(q̸ + m)/(q²−m²+imΓ)` (`propagator.rs:109`)
  - `MasslessVectorPropagator::propagate` — `−g_{μν}/q²` (`propagator.rs:146`)
  - `MassiveVectorPropagator::propagate` — unitary gauge (`propagator.rs:180`)
  - `GaugeVertex::apply` — color intertwiner dispatch (`coupling.rs:281`)
- Need design: bridge from `LorentzOp` variants (`Gamma`, `Sigma`, `ProjM`, `ProjP`, `Metric`,
  `P`, `Epsilon`, `C`) to runtime dispatch using `spin_map` to route spinor indices

Revised plan: see research/notes/10-lorentz-runtime-eval-plan.md for detailed design and current status.

_Depends on: `feyngraph-ufo-replace` (for spin_map), `lorentz-parse` (✅)_
_Unblocks: `helas-generalize`_

---

## 🟡 Medium — wire up the generalized evaluator

### `helas-generalize` — Topology-driven HELAS evaluator
Replace the hardcoded `compute_m2_ee_mumu` with a generic evaluator that
accepts a diagram topology (propagator chain + vertex list) and dispatches
to the appropriate Lorentz runtime primitives.
_Depends on: `diagram-enum` (✅), `lorentz-runtime-eval`_

### `global-config` — Implement `vibegraph_lib::config::GlobalConfig`

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

## Dependency graph

```
feyngraph-ufo-replace ──→ lorentz-runtime-eval ──→ helas-generalize ──→ event-output-lhef
lorentz-parse (✅) ────────────────────────────────┘                        │
diagram-enum (✅) ──────────────────────────────────────────────────────────┘
lips-nbody ─────────────────────────────────────────────────────────────────────────────┘
```
