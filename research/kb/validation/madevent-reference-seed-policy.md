---
type: Validation Methodology
title: Seed policy for MadEvent references
description: "At least five seeds (ten under a gate tighter than 0.3%), stored per seed, read as an inverse-variance mean with error max(quoted, spread/sqrt n); fresh directories per seed where shared-directory seeds are not independent enough."
status: draft
tags: [madgraph, references, seeds, statistics, sigma]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n34-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L145-L250", title: "Note 34 S2 — a five-seed misread, and MadGraph's last-3 combination"}
  - {id: n34-3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L338-L415", title: "Note 34 §3 — the converged llj value against MadGraph's combination"}
  - {id: n38-z1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1332-L1429", title: "Note 38 §8.1 — the seeded generators and the policy"}
  - {id: n38-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1474-L1527", title: "Note 38 §8.4 — MadEvent seeds bit-equal across hosts; Bhabha's quoted errors"}
  - {id: n38-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1528-L1686", title: "Note 38 §8.5 — the seeded σ gate; the e+e- > w+w- chi2 reading closed"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0 — MLM references, shared-directory seeds"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3 — D2: the reference sits low against fresh directories"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z — @2's reference from 21 independent directories"}
  - {id: code-manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml#L91-L110", title: "validation/manifest.toml — the seed policy header"}
  - {id: code-seeds, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/madevent_seeds.sh", title: "validation/madgraph/madevent_seeds.sh"}
---

# Seed policy for MadEvent references

One MadEvent run is one draw, and its quoted error is not its spread. Across
seeds, MadEvent's own cross sections have scattered at χ²/dof 3–14 against their
quoted errors (full-process runs included); Bhabha reaches 93 (`e+ e- > e+ e-`)
and 20 with `$$ z`[^n38-b1][^n38-z2]. A reference read as one run's value and quote
is therefore a confidently wrong number often enough to matter.

## The policy

Stated in `validation/manifest.toml`'s header ("MadEvent references: the seed
policy"). Every MadEvent cross section or width banked as a reference is:

- **at least five seeds**, and **at least ten** where the gate reading it is
  tighter than 0.3 %;
- **stored per seed** (`iseed`, value, quoted error), never as a combined mean, so
  the spread is on the record beside the quote;
- **read** as the seeds' inverse-variance mean with uncertainty
  `max(quoted, spread/√n)` — "quoted" the inverse-variance error of the seeds' own
  quotes, "spread" their sample standard deviation.

This crate's side of a seeded gate is read the same way: the gate asserts the
pull of the two means, each of this side's seeds against MadEvent's mean, and this
side's seed χ²/dof inside its 0.1–99.9 % band for its degrees of freedom
(0.13–3.1 for ten seeds). A seeded gate must also check that the thing it gates
is live: on restricted processes it asserts that the unrestricted control reads
far off the restricted reference[^n38-z2].

**Independent directories.** Seeds run in one MadEvent process directory are not
independent draws: later seeds inherit their predecessors' grids. On the MLM rows,
nine seeds in a shared directory scattered *less* than they quoted (χ²/dof
0.3–0.8 over the rows, 0.4–0.7 on `pp_to_ll_0j2j_mlm`), which was the sign;
against 21 independently generated directories the
shared-directory reference read `@1` 0.43 % high (−1.8σ combined) and `@2` 0.34 %
low[^n41-z]. The remedy is **one freshly generated directory per seed**, its error
the spread over those directories. It is not part of the written policy in the
manifest header: the user decision of 2026-09-29 (note 41 D2) applied it to the
`@2` reference only, and whether every MadEvent reference must follow it is
undecided. At `787070e` it holds only for
`pp_to_ll_0j2j_mlm` (`mlm_sigma_reference.json`, `independent_directories: true`:
the samples-grade `run_01` plus twenty fresh directories `run_s20261101`–`run_s20261120`,
each banked and its directory dropped); the other MLM rows still read nine
shared-directory seeds plus the samples run, where the policy's `max(quoted, …)`
takes the quote. A gate tighter than ~0.1 % on those would need independent
directories too. The shared-directory reading is one of the unseparated causes of
`pp_to_ttx_0j1j_mlm`'s +1.28 % `@1` disagreement
([ttx-mlm-at1-sigma-high](../backlog/validation/ttx-mlm-at1-sigma-high.md)).

**Which references are seeded.** `grammar_sigma_reference.json`, the decay, decay-chain
and on-shell-veto tables, and `mlm_sigma_reference.json` are seeded through
`madevent_seeds.sh`; `higgs_window_reference.json` (its unwindowed control) and
`pta_window_reference.json` also store per-seed runs, from their own generators'
seed loops. The cross
sections in `sigma_reference.json` and `hadronic_sigma_reference.json` predate the
rule and are single runs; a row moved onto a seeded reference says so in its
manifest cell.

## Generators

`validation/madgraph/madevent_seeds.sh` is the per-seed MadEvent loop the seeded
generators share: an existing process directory is not regenerated, a seed that
finished under a byte-identical run card is read back from
`Events/run_<tag>/vg_seed_result.txt`, and `VG_FORCE=1` re-runs. Work areas live
under `validation/madgraph/work/` (gitignored, outside the bundle); `ROWS=` and
`SEEDS=` run a subset and keep the committed rows they did not run.
`validation/generate_references.sh` has a `seeds` stage. MLM's fresh-directory runs use
`FRESH_ROWS`/`FRESH_SEEDS`. Every MadEvent seed reproduced bit-equal between the
Linux container and the macOS bank host (176 seeds over 40 rows)[^n38-b1]. Two
macOS traps in that loop are fixed but worth knowing: BSD `seq` prints `%g`, which
once collapsed every seed onto `2.02609e+07` and one cached run; and Darwin needs
`-lc++` appended for LHAPDF's C++ glue. Procedure:
[MadGraph reference generation](../tooling/madgraph-reference-generation.md),
[MadGraph reference runs](madgraph-reference-runs.md).

## Why: readings the policy overturned

- **`e+ e- > w+ w-`.** A −0.23 % offset came from one MadEvent run; against the
  seeded reference this crate reads +0.01 %. A five-seed χ²/dof of ≈ 2.4 on this
  side was a draw: twenty seeds give 1.45 over 19 dof (p ≈ 0.09), seeds 1–5 alone
  reproducing 2.38[^n38-z2].
- **Bhabha.** MadEvent's seeds split: unrestricted, three at 155.62–155.78 pb and
  two at 154.28 and 153.65. The +1.8σ pulls on those rows are the reference's
  spread, which is what the policy's `spread/√n` term exists to carry.
- **`pp_to_llj`'s "climbing ladder"** (this crate's side): a five-seed ladder read
  as a +0.04 % → +0.21 % drift was one draw 2.3σ low at its bottom rung; a
  40-seed-per-rung ensemble puts the expectation flat from 150k and kills the
  drift at 7.3σ. Five-seed scatter understated this row's per-seed spread by 2× at
  150k and 5× at 600k, where it read χ²/dof 0.03 — as loud a warning as 4.0
  (`probe_llj_seed_ensemble`)[^n34-s2]. See
  [seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md).
- **`pp_to_ll_0j2j_mlm` `@2`.** +1.49 % (5.2σ) against the old shared-directory
  reference decomposed into four parts, the largest single one (+0.65 pb) being
  the reference sitting low against fresh directories[^n41-m3]; see
  [the @2 excess decomposition](mlm-at2-excess-decomposition.md).

## MadGraph's own iteration combination

A single MadEvent run's σ is itself a combination of its iterations. MadGraph
combines the last three by `x²/σ²` weighting, which down-weights the iterations
that caught the weight tail; on `pp_to_llj` that pulls its own run 0.146 % below
the same iterations recombined by point count, and this crate's converged value
sits +0.09 % from the latter and +0.18–0.23 % from the banked number[^n34-s2][^n34-3].
This crate combines VEGAS iterations by unweighted mean after a warm-up for that
reason ([VEGAS iteration combination](../phase-space/vegas-iteration-combination.md)).
Unweighting and truncation on MadEvent's side are
[unweighting](../events/unweighting.md).

## What the policy cannot fix

- Seeds of one card share its systematic: grouped MadEvent references at
  `dynamical_scale_choice = −1` whose subprocess directories carry non-identity
  symmetry permutations are biased whether or not they are matched
  ([grouped MadEvent's permuted first call](madgraph-permuted-first-call.md)).
- A spread estimated from five seeds is itself uncertain; a rung-to-rung
  comparison needs the measured spread from 20+ seeds on the rungs that matter.

[^n34-s2]: Note 34 S2 close-out.
[^n34-3]: Note 34 §3.
[^n38-b1]: Note 38 §8.4.
[^n38-z2]: Note 38 §8.5.
[^n41-m3]: Note 41 M3, D2 diagnosis.
[^n41-z]: Note 41 Z, "`@2`'s reference from independent directories". M0's shared-directory values for `pp_to_ll_0j2j_mlm` are superseded and kept only in `shared_directory_runs`, which nothing reads.
