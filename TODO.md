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

### `process-grammar` — Process specification parser
PEG grammar for MadGraph-style process strings (`"e+ e- > mu+ mu-"`).
Map particle names to PDG codes via the UFO model, then invoke the diagram
enumerator.  Grammar documented in `research/notes/06-process-grammar.md`.
**Status:** Design phase; awaits `diagram-enum` and architecture finalization.

### `diagram-enum` — Feynman diagram enumeration via feyngraph
Call feyngraph's diagram generator with the loaded UFO model for a parsed
process.  Map topology + propagator list into a form the HELAS evaluator
can consume.  **Status:** feyngraph integration started; blocked on process grammar.
_Depends on: `process-grammar`_

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
