---
type: Sprint Record
title: "v0.2 diagnoses: one-point σ, VEGAS under-coverage, heap aborts"
description: "Sprint G part two (#89): card 048's unbounded τ draw fixed, a Breit–Wigner τ map for whole-state narrow poles, generate's no-weight panic refused, the small-width deviation registered, the VEGAS slice shown correct on main, and the heap aborts deferred after Linux tooling and an M3 soak found nothing."
closed: 2026-11-05
status: draft
tags: [diagnosis, vegas, heap-corruption, decay-chains, phase-space, sprint-g, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview, decisions G5, G8, G9"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/89", title: "nsmith-/vibegraph#89"}
---
Five sessions on #89. Every check the manager made is a dated line in
[the log](log.md).

## What landed

| Session | Result |
|---|---|
| [G-V1](sessions/G-V1-report.md) | Card 048's one-point σ: the hadronic τ draw had no ceiling at a whole-state decay window. `Cuts::shat_max` bounds both τ maps; card 048 agrees with MadGraph to 1.4 %, MadEvent's small-width convention predicting 1.55 %. |
| [G-V2](sessions/G-V2-report.md) | The cut-slice VEGAS deficit does not reproduce on main (30 seeds, −0.07 % and +0.07 %); its cause on the older tree was a heavy tail starved by unbounded adaptation, with the stop ending on calm iterations. No production code (G5). |
| [G-V3](sessions/G-V3-report.md) | No corrupting write found on Linux (unsafe audit, ASan with zstd-sys, valgrind, Miri, TSan). |
| [G-V4](sessions/G-V4-report.md) | A Breit–Wigner τ map for a narrow whole-state pole (G8): `p p > h z` through a 1 MeV Vz agrees with MadEvent to 3e-4. `generate`'s no-weight panic is `NoUsableWeight`. MadEvent's small-width treatment registered as a deviation (G9). |
| [G-HY2](sessions/G-HY2-report.md) | One τ range and one log draw for every map (bit-identical over 47000 dumped values); four new τ tests; `NoUsableWeight` pinned on a CLI route #88's refusals leave open. |

Closed: `vector-triplet-chain-one-nonzero-point`,
`vegas-under-covers-heavily-cut-slice`.

Deferred (user, 2026-11-05), claims released:
`validate-hadronic-sigabrt-unexplained`,
`malloc-abort-under-proton-sample-load`. The M3 soak of the crashing binary
gave 0 aborts in 200 runs, with and without the nano allocator.

Artifact format 21 (main's #99 took 20).

## Filed

`convergence-stop-biased-low-on-heavy-tails`, `tau-draw-ignores-shat-max-cut`,
`diagnoses-test-and-rule-gaps`; a site added to `tau-rule-reads-whole-process`;
site (f) on `madgraph-compat-sites-unconditional`.

## Process lessons

- **A main merge is gated before it is pushed.** One pushed unbuilt broke the
  build (#88 had made `from_model_card` fallible), and a second interaction
  (#88's σ = 0 refusal pre-empting a test fixture) surfaced only in the gate.
- **A refusal that another refusal seems to shadow may still be reachable.**
  The no-weight refusal looked dead under #88, but a σ > 0 grid scanned on one
  point per channel still reaches it.
