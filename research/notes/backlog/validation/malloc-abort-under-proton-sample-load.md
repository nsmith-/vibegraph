---
type: Backlog Item
title: Nondeterministic heap-corruption abort under the proton-sample suite
description: A vibegraph integrate child aborted once in seven runs with libmalloc's "pointer being freed was not allocated" during VEGAS warm-up, unreproduced since.
area: validation
state: open
priority: medium
closes_when: The abort is reproduced and its cause fixed, or attributed outside this crate (allocator, OS or a dependency) with the evidence recorded.
blocked_by: []
opened: 2026-09-07
tags: [flaky, memory-safety, rayon, kt-clustering, proton, macos]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L362-L374", title: "TODO.md entry T024"}
---
On 2026-09-07, one `validate_samples_proton` run in seven aborted. The aborting
process was a `vibegraph integrate` child on `pp_to_llj_dyn`'s card, in VEGAS
warm-up, and macOS libmalloc reported "pointer being freed was not allocated".
The crashing rayon worker was dropping a `Vec<f64>` inside `kt::Clustering`
while another worker was in `cluster::kt` / `cluster::graph`.

Ruled out so far:
- `unsafe` in the workspace (there is none).
- A non-`System` allocator.
- Any local difference against `main` in `coupling/cluster/**`, `hadronic.rs`,
  `proton.rs` or `vegas`.

It has not reproduced in any targeted rerun since, including about 12 full
`validate` runs at host loads up to 84, or in the three
`validate_samples_proton` invocations of the seed-headroom census
([note 36a §9](../../36a-seed-headroom-census.md)). The crash report is on the
user's macOS host at
`~/Library/Logs/DiagnosticReports/vibegraph-2026-09-07-085712.ips`.

Standing protocol until this closes: when a gate under this suite aborts this
way, rerun it once and report both outcomes. Do not treat either run alone as
the measurement ([note 36 §2](../../36-banked-open-ends-plan.md)).
