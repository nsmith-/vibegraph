# vibegraph — Research & Implementation Progress

**Last updated:** 2026-06-01 (research note 10 written; lorentz-runtime-eval implementation plan finalized)

This document cross-references `TODO.md` with research notes to track which design decisions are finalized vs. in-flight vs. archived.

---

## Quick Reference: Research Note Status

| Note | Title | Type | Status | Action |
|------|-------|------|--------|--------|
| `00` | LO Event Generation Overview | Reference | ✅ Finalized | None — foundational architecture fixed |
| `01` | Paper Summaries | Reference | ✅ Reference | Consult for theory background |
| `02` | FeynGraph Analysis | Reference | ✅ Reference | Alternative diagram-enum approaches |
| `03` | Sherpa & POWHEG-BOX | Reference | ✅ Reference | COMIX recursion vs MadGraph diagram enumeration |
| `04` | UFO Parsing Future | ✅ Archived | ✅ Completed | Python AST parser live (316598b) — no longer a blocker |
| `05` | MadGraph Setup | ✅ Reference | ✅ Reference | Validation pipeline: `pixi run -e madgraph generate-ee` |
| `06` | Process Grammar (PEG) | ✅ Completed | ✅ Done | Process grammar implemented and validated (diagram-count tests) |
| `07` | MadGraph Code Quality | Reference | ✅ Reference | Design patterns and architecture lessons |
| `08` | Repr Geometry | ✅ Completed | ✅ Done | Implemented in `src/repr.rs` (cdb41de) |
| `09` | UFO-ALOHA Type Matrix | 🔄 In-flight | 🔲 Design phase | Input for `lorentz-runtime-eval` primitive dispatch table |
| `10` | Lorentz Runtime Eval Plan | 🔄 In-flight | 🔲 Implementation phase | Detailed plan for AST compiler, missing primitives, vertex dispatch |

---

## Implementation Pipeline vs. Research Ownership

### Phase 1: LO Event Generation (✅ DONE)
- **Code:** `vibegraph-lib` crate (7f6e82a)
- **Papers:** `00`, `01`, `05`, `07`
- **Pipeline:** UFO loader → HELAS (hardcoded) → LIPS → VEGAS → σ
- **Validation:** σ(e⁺e⁻→μ⁺μ⁻, Z-pole) = 2025 ± 1 pb vs MadGraph 2026-05-22

### Phase 2: Process Generalization (✅ DONE)
- **Code:** `diagrams` module (process grammar + feyngraph integration + alias expansion)
- **Validation:** All 7 MadGraph reference processes match diagram counts (2026-05-27)
- **Research:** `02`, `03`, `06`, `07`

### Phase 3: Lorentz Runtime Evaluator (🔲 IN PROGRESS)
- **Approach:** Statically-compiled runtime dispatch — all Lorentz primitives pre-compiled into
  the binary; the `LorentzExpr` AST (from `ufo/lorentz.rs`) is walked at runtime to dispatch to them.
  No code generation; no compiler shipped with the binary.
- **Progress:** Lorentz PEG parser produces `LorentzExpr = Vec<LorentzTerm>` from structure strings.
  Core primitives `GammaL`, `GammaR`, `ScalarPropagator` are implemented in `helas/repr/`.
  Missing: `GammaV`, `SigmaTensor`, `Epsilon`, `DiracPropagator`, `MasslessVectorPropagator`,
  `MassiveVectorPropagator`, `GaugeVertex::apply`, and the AST-walker dispatch layer.
- **Design finalized:** Note `10` documents the complete phased plan: slot-based `DiagramAst`,
  topological ordering algorithm, vertex dispatch pattern table, all missing primitives with
  ALOHA references, and a new `helas/eval/` module structure.
- **Next:** Phase 1 primitives (vxxxxx, sxxxxx, propagators, GammaV) → Phase 2 vertex
  routines (fioxxx, jvvxxx, jsixxx) → Phase 3 AST compiler
- **Research:** `09` (type matrix for primitive dispatch), `10` (detailed plan)

### Phase 4: Generic HELAS + LIPS-nbody (🔲 PENDING)
- **Design:** `08` (repr geometry) finalized; intertwiners partially in place
- **Blocker:** `lorentz-runtime-eval` → `helas-generalize` → `lips-nbody` (recursive RAMBO)
- **Research:** `00`, `08`

### Phase 5: Event Output (🔲 PENDING)
- **Design:** Accept/reject sampling + LHEF serialization
- **Blocker:** `helas-generalize` must be stable
- **Research:** `05` (MadGraph LHEF reference)

---

## Archived / No-Longer-Active Research

**Note 04: UFO Parsing Future**
- **Why retired:** As of commit 316598b, vibegraph uses a Python AST parser to own all UFO
  parsing (particles, vertices, lorentz, couplings). Eliminates external tool dependency.
- **Action:** None — complete.

**Note 06: Process Grammar**
- **Why retired:** Process grammar is fully implemented and validated. The PEG-vs-hand-written
  question was resolved in favor of PEG (already in use).
- **Action:** None — complete.

**ALOHA codegen approach (rejected)**
- Code-generating Rust source at runtime (as MadGraph does) would require shipping a compiler
  alongside the vibegraph CLI binary. This is not acceptable for a static binary distribution.
  Replaced by `lorentz-runtime-eval`: statically-compiled primitives with runtime AST dispatch.

---

## Next Steps (Priority Order)

1. **`lorentz-runtime-eval`** — Phase 1: implement missing primitives (vxxxxx, sxxxxx, propagators, GammaV)
2. **`lorentz-runtime-eval`** — Phase 2: off-shell vertex routines (fioxxx, jvvxxx, jsixxx, iosxxx)
3. **`lorentz-runtime-eval`** — Phase 3-5: AST compiler (`helas/eval/`), dispatch, validation vs. `compute_m2_ee_mumu`
4. **`helas-generalize`** — topology-driven evaluator replacing hardcoded `compute_m2_ee_mumu`
5. **`global-config`** — thin coordinator wiring proc_card → UFO model
6. **`lips-nbody` + `event-output-lhef`** — n-body phase space and unweighted event output
