---
type: Backlog Item
title: Unweighting progress and the consent prompt are rough in the TUI
description: "progress::unweighting has no trials field, the network-consent prompt breaks the inline pane, and the line layer and terminal probe have cosmetic faults."
area: feature
state: open
priority: low
closes_when: "Unweighting progress shows trials, network::confirm suspends and resumes the pane around its prompt, and the line format follows a runtime level change."
blocked_by: []
opened: 2026-08-06
tags: [cli, tui, logging-tui]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: note33, resource: "../../33-logging-tui-plan.md", title: "Note 33 §9.5, filed to the backlog (never carried into TODO.md)"}
---
Three leftovers from the logging/TUI work ([note 33 §9.5](../../33-logging-tui-plan.md)):

- **Trials.** `progress::unweighting` cannot show trials because
  `EventSource` does not expose its trial count.
- **Consent prompt.** `network::confirm` prompts in raw mode under the
  viewport and garbles it; it needs a suspend/resume around the prompt. It only
  fires on an uncached PDF fetch, so no gate path reaches it today.
- **Cosmetic.** The line layer's format is fixed at init from the starting
  level, so a runtime climb to DEBUG keeps the compact form; crossterm's
  cursor-position probe costs ~2 s before falling back to plain lines where
  nothing answers DSR (bare `script(1)`).
