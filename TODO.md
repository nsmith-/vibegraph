# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser replacing PEG (as of 316598b) |
| 2 | Feynman diagram enumeration | 🔲 Pending | Using feyngraph crate; awaits process grammar |
| 3 | HELAS helicity amplitudes (e⁺e⁻→μ⁺μ⁻, hardcoded) | ✅ Done | Validated against MadGraph to <0.1% (Z-pole) |
| 3′ | HELAS generalized (topology-driven, arbitrary process) | 🔲 Pending | Awaits diagram enum + ALOHA codegen |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | σ ≈ 2025 pb at √s = 91.2 GeV vs MadGraph ref |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

---

## 🔴 ~~Immediate — complete the toy e⁺e⁻→μ⁺μ⁻ pipeline~~ ✅ Done

The end-to-end cross-section pipeline is complete (as of 2d8418a):
- `vibegraph-lib::phasespace` — massless 2-body LIPS Jacobian, unit-hypercube mapping, `cos_range` support
- `vibegraph-lib::vegas` — classic Lepage VEGAS adaptive Monte Carlo integrator (1D + nD)
- `vibegraph-lib::helas::sigma_ee_mumu` — wires HELAS + LIPS + VEGAS; validated against σ = 4πα²/(3s)
- **Validated against MadGraph5 reference**: σ ≈ 2025 pb at √s = 91.2 GeV to < 0.1% closure
  (measured 2026-05-22 via `pixi run -e madgraph generate-ee`)
- Project refactored into `vibegraph-lib` (library) and `vibegraph` (CLI) as of 7f6e82a

## 🟡 Medium — generalize beyond the hardcoded process

### `madgraph-diagram-validation` — Validate diagram enumeration
Comprehensive test suite comparing vibegraph diagram counts against MadGraph5_aMC@NLO
reference output. Covers 7 processes (default + constrained orders) at tree level.
**Status:** ✅ All 7 processes pass (2026-05-27).
- MadGraph scripts for e⁺e⁻→μ⁺μ⁻, p p→l⁺l⁻, p p→l⁺l⁻j, p p→bb with order constraints
- Python `extract_diagrams.py` reads `configs.inc` (IFOREST/SPROP/TPRID) for both diagram
  counts and per-diagram topology (cluster structure + propagator PDG codes).
  **Previously** counted `.ps` files (wrong — one file per process, not per diagram).
- Rust test `validate_madgraph_diagrams.rs` prints both MadGraph and vibegraph topologies
  side-by-side for each process, then compares total counts.
- pixi tasks: `build-diagrams`, `extract-diagrams`

**Known count discrepancies (root causes) — resolved and remaining:**
1. **Light fermion Yukawa = 0** ✅ resolved: restrict_default.dat loaded; zero-coupling vertex
   filter removes H/G0 diagrams for e/μ/c via `DiagramSelector::add_custom_function`.
2. **WEIGHTED coupling order** ✅ resolved: `coupling_orders.py` parsed; `generate_from_process_spec`
   now discovers the minimum WEIGHTED order iteratively (MadGraph algorithm) and filters diagrams
   above that threshold.  For QCD-dominant processes (e.g. `p p > b b~`), this removes photon/Z
   s-channel diagrams from quark initial states (QED=2 → WEIGHTED=4 > QCD=2 → WEIGHTED=2).
3. **Subprocess expansion / flavor deduplication** ✅ resolved: validation test now uses
   MadGraph-style subprocess class grouping: each set is keyed by (sorted initial particle-type
   classes, sorted final particle-type classes) where all quarks/antiquarks → "quark", all
   leptons/antileptons → "lepton", gluon → "gluon".  One representative diagram count per class.
   Implemented in `count_mg_style_topologies` in `validate_madgraph_diagrams.rs`.
4. **`gq` initial states missing** ✅ resolved: root cause was two bugs: (a) `parse_particles`
   skipped `.anti()` definitions → anti-quark python-names absent → CKM-zero vertices mis-resolved
   to single-particle entries like `["u"]`, over-broadly rejecting all u/c quark diagrams; fixed by
   adding `.anti()` handling to `parse_particles` (now exposed as `Particle::make_anti`; forward
   refs in `.anti()` now return `ParticleError` instead of silently dropping the entry).
   (b) `generate_sets_inner` deduped on `sorted_initial` only → first final-state combo for
   `["d","g"]` (i.e. `g d > e⁺ e⁻ g`, 0 diagrams) blocked the correct `g d > e⁺ e⁻ d`; fixed
   by deduplicating on `(sorted_initial, final_state)` pair.

### `process-grammar` — Process specification parser
PEG grammar for MadGraph-style process strings (`"e+ e- > mu+ mu-"`).
Map particle names to PDG codes via the UFO model, then invoke the diagram
enumerator.  Grammar documented in `research/notes/06-process-grammar.md`.
**Status:** ✅ Complete; now validated against MadGraph via diagram-count tests.

### `diagram-enum` — Feynman diagram enumeration via feyngraph
Call feyngraph's diagram generator with the loaded UFO model for a parsed
process.  Map topology + propagator list into a form the HELAS evaluator
can consume.  **Status:** ✅ Complete; `diagrams` module integrated and validated.
_Depends on: `process-grammar` (✅), UFO parsing (✅)_

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
3. Pass `&UFOModel` to diagram / ALOHA / reweighting layers.

Restrict-card resolution: `restrict_path_override` takes precedence; otherwise
look for `<model_dir>/restrict_<variant>.dat` (variant from `ModelImport`), then
fall back to auto-discovery of `restrict_default.dat`.

### `feyngraph-ufo-replace` — Replace `TopoModel::from_ufo()` with vibegraph-built model

Currently feyngraph's `TopoModel::from_ufo(path)` re-parses the UFO directory independently.
Replace it by building the feyngraph `Model` directly from vibegraph's already-parsed data
using feyngraph's mutation API (`Model::empty()` + `add_particle` + `add_vertex`).

The missing piece is `spin_map`: feyngraph needs a `Vec<isize>` of length `n_legs` mapping
each leg to its spin-contracted partner. Add `compute_spin_map(structure: &LorentzExpr, n_legs: usize) -> Vec<isize>`
to `vibegraph-lib/src/ufo/lorentz/` — walk the AST and extract spinor-index contractions from
`Gamma`, `ProjP`, `ProjM`, `C`, and spinor-index `Metric` operators.

**Benefit**: eliminates feyngraph's UFO parser entirely; vibegraph owns the full parsing pipeline,
enabling support for non-standard UFOs (`loop_sm`, etc.) and removing the `TopoModel::from_ufo` path.
The `spin_map` is also required by `aloha-codegen` — feyngraph's internal spin_map is not public,
so vibegraph must compute and carry it alongside each `LorentzStructure` regardless.

_Depends on: `lorentz-parse`, `ufo-full-ownership` (both ✅)_
_Unblocks: `aloha-codegen`_

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

### `ufo-full-ownership` — Replace feyngraph's UFO parser (COMPLETED ✅)
Full Rust ownership of `particles.py` / `vertices.py` / `lorentz.py` / `couplings.py` 
parsing using Python AST walker instead of PEG (as of 316598b). 
Eliminates external tool dependency and enables full ALOHA support.

### `lorentz-parse` — Parse `lorentz.py` into a symbolic tensor AST  
Extract and preserve the `structure` field of each Lorentz object 
(e.g. `"Gamma(1,2,3)*ProjM(4,5)"`) for automatic HELAS code generation.
**Status:** PEG parser implemented with proper arithmetic precedence and named-operator
dispatch (`build_lorentz_op`). Unknown operators produce `LorentzError::UnknownOperator(name)`.
UFOModel now uses `IndexMap` for ordered O(1) name→index lookup; `EvaluatedModel` uses
index-based `Vec<Complex64>` for coupling values (no string detour).
Next: walk the AST for ALOHA codegen.
_Depends on: `ufo-full-ownership` (now ✅)_

### `aloha-codegen` — Code-generate HELAS routines from Lorentz structures
Walk the symbolic Lorentz tensor AST and emit Rust HELAS-style vertex/current 
functions. See `research/notes/09-ufo-aloha-type-matrix.md` for type mappings.
_Depends on: `lorentz-parse`_

### `helas-generalize` — Topology-driven HELAS evaluator
Replace the hardcoded `compute_m2_ee_mumu` with a generic evaluator that
accepts a diagram topology (propagator chain + vertex list) and dispatches
to the appropriate generated HELAS routines.
_Depends on: `diagram-enum`, `aloha-codegen`_

---

## 🟢 Later — polish and extensibility

### `lips-nbody` — n-body LIPS phase-space generator
Generalize phase-space sampling to 3+ final-state particles using a
recursive 2-body decomposition (RAMBO-style) or Sudakov parametrization.
**Research first:** survey available Rust (and portable Fortran/Python) n-body
phase-space implementations before committing to an approach. Options include
porting MadGraph's phase-space routines or adapting an existing crate.
_Depends on: `xsec-ee-mumu`_

### `typed-units` — Typed physical units throughout the codebase
Research Rust typed-units crates (`uom`, `dimensioned`, `units`) for applicability
to HEP quantities (GeV, mb, etc.). Goal: typed four-momenta and cross sections
throughout to catch dimension errors at compile time. Evaluate ergonomics against
the natural-units convention (ℏ = c = 1) before committing.

### `event-output-lhef` — Unweighted events in LHEF format
Accept/reject sampling with `w(p) = |M(p)|²/w_max` to produce an
unweighted event sample.  Serialize to Les Houches Event File (LHEF) format
for downstream tools (Pythia, Herwig, etc.).
_Depends on: `xsec-ee-mumu`, `helas-generalize`_

### `ufo-full-ownership` — Replace feyngraph's UFO parser
Take full ownership of `particles.py` / `vertices.py` / `lorentz.py`
parsing without delegating topology to feyngraph.  Required for loop-level
UFOs (`loop_sm`) and for full ALOHA support.  See
`research/notes/04-ufo-parsing-future.md` for approach options (PEG vs.
Python AST walker).

---

## Dependency graph

```
vegas-survey ──→ vegas-integrator ─────────────────────────────┐
lips-2body ─────────────────────────── xsec-ee-mumu ──→ lips-nbody
                                             │
                                             └──────────────────────┐
process-grammar ──→ diagram-enum ──────────────────────────┐       │
ufo-full-ownership ──→ lorentz-parse ──→ aloha-codegen ──→ helas-generalize ──→ event-output-lhef
```
