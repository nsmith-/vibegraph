---
type: Backlog Item
title: Unweighting progress and the line layer are rough in the TUI
description: "progress::unweighting has no trials field, and the line layer and terminal probe have cosmetic faults."
area: feature
state: open
priority: low
closes_when: "Unweighting progress shows trials, and the line format follows a runtime level change."
blocked_by: []
opened: 2026-08-06
tags: [cli, tui, logging-tui]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: note33, resource: "../../33-logging-tui-plan.md", title: "Note 33 §9.5, filed to the backlog (never carried into TODO.md)"}
  - {id: network, resource: "../../../../vibegraph-cli/src/network.rs", title: "vibegraph-cli/src/network.rs confirm (~:248-256), consent through the pane"}
---
Leftovers from the logging/TUI work ([note 33 §9.5](../../33-logging-tui-plan.md)):

- **Trials.** `progress::unweighting` cannot show trials because
  `EventSource` does not expose its trial count.
- **Cosmetic.** The line layer's format is fixed at init from the starting
  level, so a runtime climb to DEBUG keeps the compact form. crossterm's
  cursor-position probe costs ~2 s before falling back to plain lines where
  nothing answers DSR (bare `script(1)`).

The third leftover, the network-consent prompt garbling the inline pane, is
resolved. `network::confirm` (`vibegraph-cli/src/network.rs` ~:248-256) asks
through the display's own question row via `tui::ask_to_download` when the pane
is live, and prompts on the stream path otherwise (checked 2026-10-09; Phase 2
drafter D15).
