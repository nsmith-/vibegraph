---
type: Design Decision
title: "Hygiene sprint findings: reviews expose, fix sessions fix what is local, the rest is filed"
description: "Review sessions only report; fix sessions take local changes (comments, demotions, tightened asserts, small dedupes); multi-file refactors and new abstractions are filed as items."
decided: 2026-10-09
decided_by: human:nsmith-
status: draft
tags: [hygiene, process, expose-dont-fix]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "../log.md", title: "Hygiene sprint log, 2026-10-09: the user's answers in the planning session"}
---
Chosen over "expose only" and "fix everything in-sprint".

- **Review sessions change no code.** This is the
  [expose, don't fix](../../../workflow/expose-dont-fix.md) split. It also
  keeps the reviewer's hit rate measurable, which the lessons need.
- **The manager triages each finding:**
  - *fix here*: confined to one module, needs no new public type, and leaves
    every gate's meaning unchanged. Examples: a stale or wrong comment, a
    demotion, an assertion tightened to a real tolerance, a helper repeated
    within one module merged into one, a dead branch removed.
  - *file*: crosses modules, adds an abstraction, changes an API, or needs a
    re-measurement. It becomes a backlog item at close-out.
  - *rejected*: with the reason. Rejections are lesson data, not noise.
- **A finding that is a real bug** follows the
  [stop-rule](../../../workflow/session-scoping-rules.md): fix it only if the
  fix is unambiguous and pinned by a test that fails before it. Otherwise file it.
- **Tightening a test is not inert.** Fix sessions confirm that a tightened
  assertion still passes, and show that it would fail on the defect it now
  guards, by a temporary mutation that is reverted before the commit and
  reported with its command.
