---
type: Session Brief
title: "T-H: hygiene review of the triple-gluon derivation, and the concepts it corrects"
description: "Review PR #39's triple_gluon tests and doc for maintainability, test non-vacuity and visibility, fix the local findings, and rewrite the four kb concepts that still call the triple-gluon −1 colour-based or underived."
status: draft
agent: hygiene-dev (Opus)
depends_on: [T1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
PR #39 derived the triple-gluon −1 ([T1 report](T1-report.md)). It is the slot
exchange the all-incoming crossing gives every triple-vector vertex with a
conjugate pair of legs: a charged vertex carries it in its kernel, and the
self-conjugate gluon vertex gets it from `triple_gluon_sign`. The constant and
every number are unchanged. The change is `triple_gluon_sign`'s doc, a
`pub(super)`, and a new test module `helas/eval/triple_gluon.rs` (531 lines,
four tests).

1. **Review the change.** Write the findings first, in your report, before any
   edit:
   - `triple_gluon.rs`: does it duplicate helpers that already exist
     (`tests/common/mg_graphs.rs`, the standalone table readers, the
     `renumbering` / `probe_bridge` probes)? Is each test's failure message
     enough to diagnose a failure? Is the JAMP-coefficient transcription checked
     against the banked `JAMP()` as the report says?
   - `triple_gluon_sign`'s doc: accurate against the report, self-contained, and
     free of plan or session references (AGENTS.md comment rules).
   - Each test's stated blind spot: confirm it with a mutation where cheap.
2. **Fix the local findings**, with mutation evidence for any test you change.
   Nothing may change a computed value.
3. **Rewrite the stale concept passages** (T1 Found 1) from the T1 report's
   derivation, each claim as the code and the report have it:
   - `research/kb/amplitudes/vector-vertex-signs.md`: its description, the
     table row, the "triple-gluon −1" section and the "has no derivation" blind
     spot. Its premise that a colourless vector vertex carries no convention
     sign needs restating: a charged triple-vector vertex carries the −1 in its
     kernel, not in `convention_sign`.
   - `research/kb/amplitudes/convention-sign-inventory.md`: the triple-gluon
     paragraph and the "no derivation" blind spot.
   - `research/kb/validation/convention-channel-coverage.md`: the
     "calibrated, not derived" blind spot. Add T1's M2 finding, that both
     MadGraph gates are blind to which factor carries the sign.
   - `research/kb/validation/bit-exact-amplitude-debugging.md`: the "calibrated
     −1" mentions.

   Leave the concepts at `status: draft`.

Gates: fmt, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `pixi run -e kb kb-lint` (0 errors) and
`python3 scripts/kb.py index --check`. Report: `sessions/T-H-report.md`, with a
Found section. Correct this brief if it is wrong.
