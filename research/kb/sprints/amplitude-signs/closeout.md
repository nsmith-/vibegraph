---
type: Sprint Record
title: "Amplitude signs and conventions"
description: "Ten items closed across seven PRs: every pp → t t~ + jet @1 matrix element gated per flow, the anchor-read role signs replaced by one root-free convention sign that agrees with MadGraph per graph on every table, two crossed-line and derivative-scalar signs fixed, and three oracles given teeth."
closed: 2026-10-16
status: draft
tags: [amplitudes, sign-convention, sprint, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log, with the user's decisions"}
  - {id: pr29, resource: "https://github.com/nsmith-/vibegraph/pull/29", title: "PR #29: t t~ + jet per-flow gate (merged)"}
  - {id: pr30, resource: "https://github.com/nsmith-/vibegraph/pull/30", title: "PR #30: the root-free convention sign"}
  - {id: pr31, resource: "https://github.com/nsmith-/vibegraph/pull/31", title: "PR #31: probe-production bridge (merged)"}
  - {id: pr32, resource: "https://github.com/nsmith-/vibegraph/pull/32", title: "PR #32: Identity crossed-line and derivative-scalar signs"}
  - {id: pr33, resource: "https://github.com/nsmith-/vibegraph/pull/33", title: "PR #33: wpwm_to_wpwmz_cw gated per graph"}
  - {id: pr34, resource: "https://github.com/nsmith-/vibegraph/pull/34", title: "PR #34: jioxxx chirality and longitudinal oracle"}
  - {id: pr35, resource: "https://github.com/nsmith-/vibegraph/pull/35", title: "PR #35: same-flavour four-fermion contact gate"}
---
The sprint looked for places where |M|² itself might be wrong. It ran as seven
PRs, one per item or small bundle. Every diagnosis was made on MadGraph 3.7.1
standalone per-diagram × per-helicity (and per-flow) dumps, and every fix was
verified on those dumps before production code changed.

## What was banked

- **The convention sign is root-free** (PR #30). The anchor-read role signs
  (`Diagram::anchor`, `yang_mills_vvv_sign`, `vector_contact_sign`,
  `gluon_scalar_current_sign`) are replaced by `convention_sign`, a product of
  local constants read off the graph:
  - the propagator phases (−1 per internal scalar line and per crossed fermion
    line);
  - each vertex's own reversed-bilinear parity;
  - a −1 per triple-gluon vertex. At close-out it was calibrated, not derived;
    the follow-up session [T1](sessions/T1-report.md) (PR #39) derived it as the
    slot exchange a self-conjugate vertex does not get from the crossing.

  A hermetic relabelling-invariance test pins the result.
- **Gates.** Every MadGraph per-graph table agrees: 49 `standalone_jamps` rows,
  with `KNOWN_DISAGREEMENT` empty, and 49 `amplitude_oracle` rows. The defect
  behind `w+ w- > e+ e-` was wider than its item said: it appeared whenever
  external leg 0 was a colourless vector (22 processes). `wpwm_to_wpwmz_cw` went
  from |M|² 2.79e1 to gated.
- **The `pp_to_ttx_0j1j_mlm` @1 matrix elements** are all gated per flow
  (PR #29). The result was handed to the parallel σ sprint: its +1.28 % `@1`
  excess is not in |M|², widths included.
- **Two derived sign fixes** (PR #32): the Identity bilinear on a crossed line,
  and the derivative all-scalar vertex. A mixed-structure vertex now gets a typed
  refusal instead of a panic. PR #30 later subsumed both signs into the
  convention.
- **Oracles given teeth:**
  - the per-diagram probes are bridged to the production `Program` (PR #31);
  - `test_eval_jioxxx` covers massive, mixed-chirality currents and the
    longitudinal denominator (PR #34);
  - same-flavour four-fermion contacts are gated per MadGraph graph (PR #35),
    with the 22-against-19 diagram count registered as a counting convention.
- **No banked σ, sample or diagram cell moved** in any PR. Only
  `wpwm_to_wpwmz_cw`'s amplitudes cell changed, from info to gate.

## Decisions (user, recorded in the sprint log)

- Sign fixes that align with MadGraph per diagram land without a separate design
  approval.
- W2 and F2 ran as one derivation session (B′). A derived convention passed, so
  no fitted rule landed.
- The per-pairing four-fermion diagram count is a registered convention. Exact
  diagram-count parity with MadGraph is not achievable in general:
  `diagram-count-oracle-is-raw-ngraphs` and
  `amplitude-partial-sums-attributed-to-diagrams` carry what follows from that.
- MadGraph's seven-digit `GC_303`/`GC_299` literals are a registered deviation of
  the approximation kind. Only the affected graphs' modulus is exempted, by name.

## Backlog

**Closed (10):**
- `gg-ttxg-no-amplitude-gate`
- `wpwm-to-epem-neutrino-exchange-sign`
- `per-diagram-probes-bypass-production-runtime`
- `standalone-identity-crossed-line-sign-unpinned`
- `derivative-scalar-contact-sign-unoracled`
- `wpwmz-cw-ow-five-vector-residual`
- `wpwmz-cw-config-partition-uncompared`
- `jioxxx-relative-chiral-sign-unpinned`
- `jioxxx-reference-port-comparison-has-no-teeth`
- `same-flavour-four-fermion-diagram-count-ungated`

**Filed (27)** from the session reports' Found sections; each item cites its
report. Six concern oracle coverage of the new convention:
- `triple-gluon-sign-calibrated-not-derived`
- `convention-sign-unreached-vertex-classes`
- `live-parity-cancellation-thinly-pinned`
- `mixed-structure-vertex-unoracled`
- `derivative-scalar-asymmetric-unoracled`
- `bilinear-amplitude-sink-no-independent-oracle`

One is tooling every diagnosis rebuilt: `per-diagram-dump-tools-uncommitted`.
The rest are gate, hygiene and usability gaps.

## Concepts

Twelve concepts were rewritten around `convention_sign` (B-H, B-D) or updated to
the new gates, and are left at `status: draft` for the user's review. The
convention ones are
[the convention-sign inventory](../../amplitudes/convention-sign-inventory.md),
[rooting invariance and the canonical form](../../amplitudes/rooting-invariance-and-canonical-form.md),
[vector-vertex signs](../../amplitudes/vector-vertex-signs.md),
[the fermion-line sign](../../amplitudes/fermion-line-sign.md) and
[convention-channel coverage](../../validation/convention-channel-coverage.md).
The agent definitions now carry the cloud disk budget (PR #29, at the owner's
review).

## Process lessons

- **One heavy suite per host held, with exceptions.** Two agents fit the 4-core,
  30 GB container only with the lean cargo settings. A 15 GB debug tree filled
  the disk once, and the rule is now in every agent definition.
- **Merges dropped lines silently.** Two generator merges lost a shared trailing
  `zero_widths=True),` that git factored out of conflicting rows. A Python AST
  parse and a row-set union check caught both. Check structured files by parse
  after every conflicted merge.
- **Agents self-stamped `verified`** once. Later briefs forbade it.
- **Diagnosis-only sessions paid off.** W1's leg-0 finding and F1's two signs
  showed one mechanism, and that turned a fitted fix into the derived
  convention.

## Gates on the final commits

Each PR's CI ran `build`, `test`, `fmt`, `kb` and `banked` (`pixi run validate`)
on its final head. The manager re-ran the PR's own gates locally after each
session; the PR descriptions record the commands.
