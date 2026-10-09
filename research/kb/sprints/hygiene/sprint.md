---
type: Sprint
title: "Hygiene sprint"
description: "One pass over the existing codebase for maintainability, test non-vacuity, minimal visibility and reusable abstractions, closing the localised hygiene items, whose lessons design the hygiene agent."
status: draft
active: true
tags: [hygiene, sprint, visibility, non-vacuity, maintainability]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: item, resource: "../../backlog/hygiene/hygiene-sprint.md", title: "Backlog item: the codebase has had no dedicated hygiene pass"}
  - {id: decision, resource: "../../decisions/pr-per-backlog-item.md", title: "Decision: one PR per backlog item, four session types"}
---
The first sprint run in the [sprint lifecycle](../../workflow/sprint-lifecycle.md)'s
folder shape, and its test on live work. It reviews the existing codebase from
the four perspectives of the planned hygiene session type, fixes what is small,
files what is large, and closes the hygiene items that already name their sites.
How the reviews went is recorded as the input to
[hygiene-agent-type](../../backlog/hygiene/hygiene-agent-type.md).

## Goal and exit criteria

The sprint is done when:

1. **Every cluster has been reviewed** (R-sessions below) on all four points,
   and each report's findings are triaged in `log.md` as *fixed here*, *filed*,
   or *rejected, with the reason*.
2. **Every claimed item is closed** (its file deleted) or released, with the
   reason recorded in `closeout.md`.
3. **Visibility:** every `pub` item in `vibegraph-lib` that nothing outside its
   crate uses is demoted, and a proposed supported library surface is drafted
   for the user's decision on
   [lib-pub-api-surface-unaudited](../../backlog/feature/lib-pub-api-surface-unaudited.md).
   That item stays open ([D2](decisions/D2-visibility-mechanical-demotion.md)).
4. **Gates are unmoved:** `cargo fmt --all --check`,
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`, the
   hermetic `cargo test --workspace`, and `pixi run --skip-deps validate` on the
   final commit, with every previously enforced cell unchanged.
5. **Lessons written:** a draft methodology concept on running a hygiene
   review, covering what found real defects, what produced false positives,
   and how much each point cost. It is the design input for the hygiene agent.

## Decisions

Decided by the user on 2026-10-09, and the decision concepts reviewed and signed off the same day (`log.md`, Approval):

- [D1](decisions/D1-scope-localised-items.md): claim filed items from any area
  that are local, need no MadGraph or Pythia run or seed re-measurement, and
  settle no open physics question. `needs-user` items stay out.
- [D2](decisions/D2-visibility-mechanical-demotion.md): visibility is mechanical
  demotion only. Anything another crate or a test target uses stays `pub`, and
  the supported surface is proposed, not decided.
- [D3](decisions/D3-fix-small-file-large.md): reviews expose; fix sessions fix
  what is local; multi-file refactors and new abstractions are filed.
- [D4](decisions/D4-module-clusters-one-pr.md): one sprint PR. Each review
  session covers one module cluster on all four points.

## Clusters

| Cluster | Code (under `vibegraph-lib/src/` unless stated) | ≈ lines |
|---|---|---|
| A | `helas/eval/` | 26k |
| B | `helas/repr/`, `helas/color/`, `helas/{vertex,wavefn,mod}.rs` | 9k |
| C | `ufo/`, `diagrams/`, `onshell.rs`, `reweight/` | 16k |
| D | `phasespace/`, `vegas.rs`, `budget.rs`, `cuts.rs`, `multiplicity.rs`, `unweight.rs`, `stats.rs`, `select.rs` | 17k |
| E | `proton.rs`, `hadronic.rs`, `pdf/`, `coupling/` (couplings, αs, scales, kT clustering) | 21k |
| F | `runcard*`, `artifact.rs`, `lhef/`, `cache/`, `config.rs`, `progress.rs`, `validation*`, `bin/`; `vibegraph-cli/` (src and tests); `validation-report/` | 30k |
| G1 | `vibegraph-lib/tests/` on amplitudes, models and diagrams (list in [R-G1](sessions/R-G1.md)); `benches/` | 15k |
| G2 | `vibegraph-lib/tests/` on sampling, scales, PDFs and events (list in [R-G2](sessions/R-G2.md)); `validation/manifest.toml` | 25k |

In-module `#[cfg(test)]` tests are reviewed with their cluster.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR), with
the session that closes each:

| Item | Closed in |
|---|---|
| hygiene-sprint | Z |
| dead-types-with-stale-docs | V1 |
| evaluator-doc-comments-stale | F-A |
| ufo-asin-acos-evaluate-as-acsc-asec | F-C |
| make-anti-negates-singlet-octet-colour | F-C |
| process-model-and-artifact-doc-comments-stale | F-C |
| reweight-forbidden-onshell-guard-is-dead | F-C |
| feyngraph-submodule-pin-differs-from-build | F-C |
| sampler-and-phase-space-doc-comments-stale | F-D |
| madgraph-line-citations-predate-pin | F-D |
| configuration-weights-wrong-at-sde1-with-tmin | F-E |
| artifact-reader-arm-names-format-version | F-F |
| runcard-opaque-defaults-unverified | F-F |
| no-network-variable-read-two-ways | F-F |
| validation-test-comments-stale | F-G |
| validate-scales-module-doc-stale | F-G |
| validate-hadronic-calibration-comments-superseded | F-G |
| manifest-notes-describe-superseded-state | F-G |
| jj-banked-orderings-eta-uses-wrong-components (validation) | F-G |
| config-amp-phase-and-sign-unpinned (validation) | F-G |
| smeftsim-vendored-checksum-not-hermetic (validation) | F-G |
| host-info-null-cpu-block-on-linux | T1 |
| profile-script-forwards-one-filter | T1 |
| madgraph-generators-hardcode-lcxx-and-bypass-pin | T1 |
| ci-and-agent-skill-docs-stale | T1 |
| acceptance-yml-fails-on-refdata-releases | T1 (the file is deleted only once a `refdata-*` release has run without a red acceptance run; otherwise the claim is released) |

Not claimed, by [D1](decisions/D1-scope-localised-items.md): acceptance-yml-never-passed,
acceptance-yml-weekly-schedule, co-authored-by-trailer-from-cloud-agent-pr,
llvm-preserve-none-musttail-bug-unfiled, manifest-blocked-tier-unused (all
`needs-user` or blocked on one); kt-dump-tables-lack-directory-key,
pp-to-jj-tie-break-no-cluster-dump (MadGraph re-extraction);
llj-gate-comments-quote-pre-floor-ladders, sigma-calibration-comments-stale
(long re-measurement); jioxxx-reference-port-comparison-has-no-teeth (settles
a HELAS-against-ALOHA question); pythia-gate-momenta-unchecked (needs the Pythia
environment); nnpdf23-redistribution-terms-unverified (licence research,
not code); lorentz-coefficients-still-f64 (R-A recommends a disposition; no
code change in this sprint); hygiene-agent-type (blocked on this sprint; its
own PR uses the lessons); lib-pub-api-surface-unaudited (`needs-user`; this
sprint supplies its proposal).

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| [V1](sessions/V1.md) visibility demotion | `feature-dev` (Opus) | — | lint + hermetic, inert | dead-types-with-stale-docs |
| [V1b](sessions/V1b.md) allows and doc links | `feature-dev` (Opus), fresh | V1 | lint + hermetic, inert | — |
| [T1](sessions/T1.md) tooling and CI | `validation-dev` (Sonnet) | — | scripts' own checks | five tooling items |
| [R-A](sessions/R-A.md) … [R-G2](sessions/R-G2.md) reviews (8) | `claude` (Opus), read-only | V1b | none: report only | — |
| Triage (manager) | — | all R | — | — |
| [F-A](sessions/F-A.md), [F-B](sessions/F-B.md), [F-C](sessions/F-C.md), [F-D](sessions/F-D.md), [F-E](sessions/F-E.md), [F-F](sessions/F-F.md), [F-CLI](sessions/F-CLI.md), [F-G1](sessions/F-G1.md), [F-G2](sessions/F-G2.md) fixes (9, [protocol](sessions/fix-protocol.md)) | per brief | [triage](triage.md); run serially, in that order | lint + hermetic + touched banked targets | the cluster's claimed items |
| L lessons | `claude` (Opus) | all F | `kb-lint` | — |
| Z close-out | manager | L | the exit criteria | hygiene-sprint |

**Order and parallelism.**
- **V1 first.** It touches files in every cluster, and reviewers should see the
  demoted surface, where dead code shows up as `dead_code` warnings.
- **T1** touches only scripts and workflows, so it runs beside V1 or the reviews.
- **The eight reviews run in parallel.** They are read-only and build nothing,
  so the one-heavy-suite-per-host rule does not bind them.
- **Fix sessions run one at a time** on the sprint branch, F-G last, because
  source fixes can move the comments and tests it touches.
- **Fix briefs are written at triage.** A fix session's scope is the cluster's
  claimed items plus the triaged *fix here* findings, and neither is known
  before the reviews report. Triage may merge small clusters into one fix
  session.

## Risks

- **Review volume.** Eight reports on ~150k lines can overrun triage. Each
  report caps itself at its 30 strongest findings and ranks them; the rest go
  in one line each under "Also seen".
- **Demotion that breaks a feature-gated target.** V1 builds with
  `--all-targets --all-features`; banked-layer tests compile only under
  `extended-validation` and would otherwise hide an external use.
- **Hot module.** No other PR is open (checked 2026-10-09). If an evaluator PR
  opens mid-sprint, F-A rebases onto it or lands after it
  ([sprint lifecycle](../../workflow/sprint-lifecycle.md), "Parallel PRs on a hot module").
