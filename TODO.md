# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status |
|------|-----------|--------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done |
| 2 | Feynman diagram enumeration | 🔲 Pending |
| 3 | HELAS helicity amplitudes (e⁺e⁻→μ⁺μ⁻, hardcoded) | ✅ Done |
| 3′ | HELAS generalized (topology-driven, arbitrary process) | 🔲 Pending |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done |
| 6 | Unweighted event output (LHEF) | 🔲 Pending |

---

## 🔴 ~~Immediate — complete the toy e⁺e⁻→μ⁺μ⁻ pipeline~~ ✅ Done

The end-to-end cross-section pipeline is complete:
- `src/phasespace.rs` — massless 2-body LIPS Jacobian, unit-hypercube mapping, `cos_range` support
- `src/vegas.rs` — classic Lepage VEGAS adaptive Monte Carlo integrator (1D + nD)
- `helas::sigma_ee_mumu` — wires HELAS + LIPS + VEGAS; validated against σ = 4πα²/(3s)
- Validated against MadGraph5 reference (σ ≈ 2025 pb at √s = 91.2 GeV) to < 0.1% closure
  once MadGraph's default acceptance cuts (`ptl > 10 GeV`, `|η| < 2.5`) are applied

## 🟡 Medium — generalize beyond the hardcoded process

### `process-grammar` — Process specification parser
PEG grammar for MadGraph-style process strings (`"e+ e- > mu+ mu-"`).
Map particle names to PDG codes via the UFO model, then invoke the diagram
enumerator.  Grammar documented in `research/notes/06-process-grammar.md`.

### `diagram-enum` — Feynman diagram enumeration via feyngraph
Call feyngraph's diagram generator with the loaded UFO model for a parsed
process.  Map topology + propagator list into a form the HELAS evaluator
can consume.
_Depends on: `process-grammar`_

### `lorentz-parse` — Parse `lorentz.py` into a symbolic tensor AST
Extend the UFO loader to preserve and parse the `structure` field of each
Lorentz object (e.g. `"Gamma(1,2,3)*ProjM(4,5)"`).  Currently feyngraph
silently drops all operator type information.  Prerequisite for automatic
HELAS routine generation.
_Depends on: `ufo-full-ownership` (or at minimum a parallel parser for lorentz.py)_

### `aloha-codegen` — Code-generate HELAS routines from Lorentz structures
Walk the symbolic Lorentz tensor AST and emit Rust HELAS-style vertex /
current functions.  Eliminates hand-coded routines and enables arbitrary
SM/BSM vertices.
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
_Depends on: `xsec-ee-mumu`_

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
