---
type: Backlog Item
title: "No banked test runs the cut guards on every banked card's production path"
description: "Sprint I's 'every banked card accepted' came from a scratch sweep; most σ gates call Cuts::compile without diagrams, so they never exercise the uncut-limit guards."
area: validation
state: open
priority: medium
closes_when: "A banked test compiles every manifest row's cuts through ForcedResonances::of with its real diagrams and asserts the expected accept/refuse outcome per row."
blocked_by: []
opened: 2026-11-09
tags: [guards, cuts, banked-layer, tests]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: ic2, resource: "../../sprints/v02-last-refusals/sessions/I-C2-report.md", title: "I-C2 report and addenda"}
---
I-C2's sweep (`scratchpad/ic2/sweep.sh` with `extract_cards.py`) ran all 62
seeded rows through `vibegraph integrate`. It is not committed, so "every
banked card is accepted" is not kept true automatically.

Also from I-C2: the coloured-only `refuse_ir_unsafe_exemption` is broader than
the limit check needs. No case was found where it alone fires. Restate it over
`limit_regulated` once that is proven.
