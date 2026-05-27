# vibegraph — Research & Implementation Progress

**Last updated:** 2026-05-27 (diagram validation complete)

This document cross-references `TODO.md` with research notes to track which design decisions are finalized vs. in-flight vs. archived.

---

## Quick Reference: Research Note Status

| Note | Title | Type | Status | Action |
|------|-------|------|--------|--------|
| `00` | LO Event Generation Overview | Reference | ✅ Finalized | None — foundational architecture fixed |
| `01` | Paper Summaries | Reference | ✅ Reference | Consult for theory background |
| `02` | FeynGraph Analysis | Reference | ✅ Reference | Alternative diagram-enum approaches |
| `03` | Sherpa & POWHEG-BOX | Reference | ✅ Reference | COMIX recursion vs MadGraph diagram enumeration |
| `04` | UFO Parsing Future | 🔄 Archive → Done | ✅ Completed | **Python AST parser now live (316598b)** — no longer a blocker |
| `05` | MadGraph Setup | ✅ Reference | ✅ Reference | Validation pipeline: `pixi run -e madgraph generate-ee` |
| `06` | Process Grammar (PEG) | 🔄 In-flight | 🔲 Design phase | Awaits `diagram-enum` integration plan |
| `07` | MadGraph Code Quality | Reference | ✅ Reference | Design patterns and architecture lessons |
| `08` | Repr Geometry | 🔄 Completed | ✅ Done | Implemented in `src/repr.rs` (cdb41de) |
| `09` | UFO-ALOHA Type Matrix | 🔄 In-flight | 🔲 Design phase | Input for `aloha-codegen` implementation |

---

## Implementation Pipeline vs. Research Ownership

### Phase 1: LO Event Generation (✅ DONE)
- **Code:** `vibegraph-lib` crate (7f6e82a)
- **Papers:** `00`, `01`, `05`, `07`
- **Pipeline:** UFO loader → HELAS (hardcoded) → LIPS → VEGAS → σ
- **Validation:** σ(e⁺e⁻→μ⁺μ⁻, Z-pole) = 2025 ± 1 pb vs MadGraph 2026-05-22

### Phase 2: Process Generalization (🔲 IN PROGRESS)
- **Blocker:** Process grammar (PEG) — `06` awaits architecture finalization
- **Next:** `06` → `process-grammar` → `diagram-enum` (depends on feyngraph + UFO model)
- **Research:** `02`, `03`, `06`, `07`

### Phase 3: ALOHA Codegen (🔲 PENDING)
- **Design:** `09` maps UFO/ALOHA types to Rust HELAS routines
- **Progress:** Lorentz PEG parser now produces `LorentzExpr = Vec<LorentzTerm>` from structure strings.
  Grammar uses proper arithmetic precedence and captures operator names via `build_lorentz_op`.
  UFOModel uses `IndexMap` for ordered storage; `EvaluatedModel` coupling values are index-based `Vec`.
- **Blocker:** `aloha-codegen` — walk the `LorentzExpr` AST to emit Rust HELAS routines
- **Research:** `04` (now ✅ completed — Python AST owns UFO parsing), `09`

### Phase 4: Generic HELAS + LIPS-nbody (🔲 PENDING)
- **Design:** `08` (repr geometry) finalized; intertwiners in place
- **Blocker:** `diagram-enum` → `helas-generalize` → `lips-nbody` (recursive RAMBO)
- **Research:** `00`, `08`

### Phase 5: Event Output (🔲 PENDING)
- **Design:** Accept/reject sampling + LHEF serialization
- **Blocker:** `helas-generalize` must be stable
- **Research:** `05` (MadGraph LHEF reference)

---

## Archived / No-Longer-Active Research

**Note 04: UFO Parsing Future**
- **Why retired:** As of commit 316598b, vibegraph now uses a **Python AST parser** (not PEG) 
  to own all UFO parsing (particles, vertices, lorentz, couplings).
- **Outcome:** This eliminates external tool dependency and enables full ALOHA support.
- **Action:** None — this is complete. The note documents the decision history.

---

## Next Steps (Priority Order)

1. **Finalize process grammar** (`06`)
   - Lock down PEG or switch to hand-written parser
   - Coordinate with diagram-enum architecture
   - Estimated: 2–3 days exploratory design

2. **Integrate diagram-enum** 
   - Wire feyngraph + UFO model
   - Map topology output to HELAS form
   - Estimated: 1 week integration + testing

3. **ALOHA type matrix → codegen** (`09`)
   - Parse `lorentz.py` structures (extend UFO loader)
   - Code-generate Rust HELAS vertices
   - Estimated: 1–2 weeks with testing

4. **Generic HELAS evaluator** (`08` + implementation)
   - Replace hardcoded `compute_m2_ee_mumu`
   - Dispatch on topology + routing
   - Estimated: 1 week

5. **LIPS-nbody + event output**
   - Recursive RAMBO sampling
   - Accept/reject + LHEF
   - Estimated: 2 weeks
