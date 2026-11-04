---
type: Sprint Record
title: "Grid-worker fixes: PDF lookup without a home directory, and quark-jet ttx at generate"
description: "Two items closed in PR #87: the cache root is required only at the cache tier; mixed-flavour groups take a quark line's member flavour from the leg that carries it, so the ttx MLM row's own process generates and is gated against MadEvent."
closed: 2026-11-04
status: draft
tags: [pdf, cache, grid-worker, generate, resonance, flavour-groups, mlm, sprint, sprint-record, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: pr87, resource: "https://github.com/nsmith-/vibegraph/pull/87", title: "PR #87 (merged)"}
---
Part of sprint F (CMS production readiness), alongside
[cms-pdf-memory](../cms-pdf-memory/closeout.md).

## What was banked

### No home directory

The PDF lookup reads `$LHAPDF_DATA_PATH` and CVMFS first. The cache root is
needed only when the lookup reaches the cache or download tier, and the
refusal there names that tier ([G-F1](sessions/G-F1-report.md)).

`CACHE_ROOT_VAR` now lives in the library, and `pdf_tiers` cannot fail
([G-HY1](sessions/G-HY1-report.md)).

On Unix, unsetting `$HOME` is not "no home directory": `dirs` falls back to
the passwd database. The CLI tests therefore run the binary as a uid with no
passwd entry, and on non-root CI they skip.

### Quark-jet ttx

- **Root cause.** `member_line_pdg` assumed that a member differs from its
  representative only by conjugation. A QCD-only group mixes u and d.
- **The fix.** `carried_flavour` takes the member's flavour from the leg
  that carries the line. If no leg carries it, or two legs disagree, the
  charge rule still decides ([G-F2](sessions/G-F2-report.md)).
- **Scope.** The row's own full-`j` process was refused too.
- **Records.** No wrong record was ever written: the affected lines have
  zero width, and records list only lines with a width.

### The ttx MLM row against MadEvent

The row's own sample now has a banked gate against MadEvent's `run_01`
([G-V1](sessions/G-V1-report.md)):
- 18 population shares;
- the largest pull is 1.89;
- a u↔d jet swap fails at about 9–10σ.

## Lessons

- **The briefs were wrong in ways the sessions caught:**
  - the package name;
  - "no `$HOME`" as a way to reach the no-home path;
  - integrate's nonexistent per-subprocess σ split;
  - a clustering-frame bias that #23 had already fixed.

  Each session reported its correction rather than following the brief.
- **Both hygiene sessions returned their reports as messages**, not files,
  and one cited its agent definition, which does not say that. The manager
  transcribed them. A brief should say which of the two it wants, and the
  hygiene-dev definition could settle it.

## Filed from the sessions

- `no-home-cli-tests-skip-on-ci`
- `llj-fraction-test-ignored-though-banked`
- `ttx-mlm-sample-untested-in-distributions` (needs-user: a
  `hard_survey = 2` event run of the ttx row on the M3)
