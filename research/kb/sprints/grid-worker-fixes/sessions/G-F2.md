---
type: Session Brief
title: "G-F2: generate on p p > t t~ j with a quark-only jet"
description: "Reproduce generate's refusal on the quark-jet ttx card, fix the member-resonance lookup in mixed-flavour groups, pin it with a test that fails before the fix, and check the events against integrate's per-subprocess split."
status: draft
agent: feature-dev (Opus)
depends_on: [G-F1]
closes: [generate-refuses-quark-only-jet-ttx]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker` (draft PR #87). Start
at the commit that adds this brief; G-F1's commit is below it. Read the item
(`python3 scripts/kb.py backlog --item generate-refuses-quark-only-jet-ttx`),
the [sprint](../sprint.md), and Found 2 of
`research/kb/sprints/sigma-ttx-mlm-at1/sessions/V2-report.md`. The refusal
is raised in `vibegraph-lib/src/generate.rs` (about line 1336). This brief may
be wrong: if the code says otherwise, say so in the report.

**Environment.**
- Linux container, 4 cores. Pixi is not installed.
- Build with
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
- A sibling performance session times things in `/home/user/wt/M`. Keep
  CPU-heavy runs as short as the evidence needs.
- Reference data is hard-linked in; never write into it. That includes
  refdata-13 under `validation/madgraph/output`, `pp_to_ttx_0j1j_mlm` among
  it, and the SM UFO.

**The card.** V2's `gq.dat` was a scratch file and is gone. Rebuild it:
- `p p > t t~ j`, with `j` holding the light quarks and antiquarks only (no
  gluon), `p` as on the `pp_to_ttx_0j1j_mlm` row;
- that row's `Cards/run_card.dat`.

Record the card you use in the report. The refusal reads "the resonance 2 of
g u > t t~ u has no flavour in the subprocess with outgoing [6, -6, 1] that
shares its matrix element". Reproduce it first.

**Task.**
- Find why the resonance record of a mixed-flavour group (u and d members
  that share a matrix element) looks up the wrong member, and fix the
  mapping, not the symptom.
- Say whether any other process reaches the same path:
  - which groups are mixed-flavour with resonance records;
  - whether events written before the fix could have carried a wrong
    resonance record without being refused.
- Pin it with a test that fails before the fix (stash, run, record the
  output). Use the smallest process that reaches the path; a cheap
  integrate-and-generate in the default suite if it is cheap enough, else
  the narrowest unit.
- **Check:**
  - generate on the full card writes events;
  - the per-subprocess event counts agree with integrate's σ split within
    their statistical error, with the numbers in the report;
  - the gates the `extended-validation` skill names for generate, resonance
    records and phase space, run with the cargo commands that pixi tasks wrap;
  - `cargo test`, fmt and clippy.

**Report.** `research/kb/sprints/grid-worker-fixes/sessions/G-F2-report.md`
(type Session Report), with commands, outputs and a Found section. Commit as
described in your agent definition. Do not push.
