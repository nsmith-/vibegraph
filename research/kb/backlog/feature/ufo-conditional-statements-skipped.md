---
type: Backlog Item
title: Non-constructor statements under 'if' in UFO files are still skipped silently
description: "Conditional UFO object definitions are refused, but other statements under 'if' (a conditional x.anti(), an attribute set) are still skipped without a word."
area: feature
state: open
priority: low
closes_when: "Any statement under an 'if' (or other compound statement) at module level of a UFO file the loader reads is refused naming the file and line, unless it is shown to be one the loader ignores anyway, by a test."
blocked_by: []
opened: 2026-10-20
tags: [ufo, parser, silent, non-sm-ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f2, resource: "../../sprints/ufo-conditional-params/sessions/F2-report.md", title: "ufo-conditional-params F2 report, Found 2"}
  - {id: hy1, resource: "../../sprints/ufo-conditional-params/sessions/HY1-report.md", title: "ufo-conditional-params HY1 report, H6"}
---
After the [ufo-conditional-params sprint](../../sprints/ufo-conditional-params/sprint.md),
`refuse_conditional_definitions` refuses conditional constructor calls in six
files. A conditional `x.anti()` in `particles.py`, or any other statement
under `if`, is still dropped silently. The (file, constructor) pairs it checks
are written separately from the six readers' own constructor names (HY1 H6);
a shared table would keep them in step, and the refusal test exercises all six
pairs meanwhile.
