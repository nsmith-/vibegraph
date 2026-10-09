---
type: Design Decision
title: Release scope restricted to fixed-order Standard Model processes
description: "The release goal covers arbitrary fixed-order SM processes over unpolarized pp or fixed-energy partonic beams; every descoped surface a card can reach is a hard error."
decided: 2026-08-02
decided_by: human:nsmith-
superseded_by: release-scope-mg-lo-parity
status: deprecated
verified: [{by: "human:nsmith-", at: 2026-10-09}]
tags: [scope]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L27-L33", title: "TODO.md scope decision (user, 2026-08-02)"}
---

The release goal is **arbitrary fixed-order Standard Model processes** over
unpolarized proton–proton or fixed-energy partonic beams. Every extension
beyond that — BSM UFO support, other beam configurations, polarization — is
a feature backlog item tagged `descoped-v1`.

The rule that outlives this decision: every descoped surface a card can still
reach is a **hard error**, never a silent acceptance.

Widened on 2026-09-25 by [release-scope-mg-lo-parity](release-scope-mg-lo-parity.md),
which brought decay chains and 1→n decays into scope.
