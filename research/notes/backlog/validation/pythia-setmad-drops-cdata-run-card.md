---
type: Backlog Item
title: Pythia's setMad reads nothing from MadGraph's CDATA-wrapped run card
description: Pythia 8.312 drops the CDATA section MadGraph wraps <MGRunCard> in, so setMad = on on MadEvent's own file reads a 4-byte card and does not match.
area: validation
state: needs-user
priority: low
closes_when: The interplay is reported upstream (to MadGraph's LHE writer or Pythia's reader) and the report's link is recorded in note 07.
blocked_by: []
opened: 2026-09-29
tags: [mlm, pythia, lhef, upstream-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L242-L248", title: "TODO.md entry T006"}
---
On MadEvent's file as written, Pythia 8.312 with `JetMatching:setMad = on`
reads a 4-byte card. Without an explicit `JetMatching:merge = on` it does not
match at all (10000/10000 events accepted). Once the CDATA markers are removed
(a 14084-byte card), `setMad` reads `xqcut 20`, `ickkw 1`,
`maxjetflavor 4`, `alpsfact 1` and the acceptances agree with vibegraph's.

Nothing in this crate depends on it. MadGraph drives Pythia with
`setMad = off`, which is unaffected. This crate writes the card as escaped
element text, which Pythia reads (`vibegraph-lib/src/lhef/write.rs:270`). What
remains is an upstream report, which the user files.

Detail: [note 41 M4, M5](../../41-mlm-feature-sprint-plan.md); the `pp_to_ll_0j2j_mlm` samples note in
`validation/manifest.toml`.
