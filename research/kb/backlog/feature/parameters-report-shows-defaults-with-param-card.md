---
type: Backlog Item
title: "`vibegraph parameters --param-card` reports each parameter's default, not the bound value"
description: "With --param-card, parameters binds the card but keeps only its Auto widths; parameters.json shows model defaults (DarkPhoton mZDinput 20 on every card), so a reader cannot confirm a bound value."
area: feature
state: open
priority: low
closes_when: "With --param-card, parameters.json records each external parameter's bound value (and the card's ignored entries) beside its default, a test pins a card that moves a parameter off its default, and the --param-card help text says what the report shows."
blocked_by: []
opened: 2026-11-24
tags: [param-card, cli, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
`vibegraph parameters --param-card <card>` binds the card the way `integrate`
binds it, but `run` keeps only the bound card's `auto_widths`
(`vibegraph-cli/src/parameters.rs`, the `bind_param_card(...).auto_widths` line).
The report is then built from the model alone (`describe_parameters_at`), so
every parameter shows its model default.

Found by the sprint L validator on PR 11's DarkPhoton cards 78, 79, 418 and 419. The
gridpack cards set `mZDinput` to 40, 86, 20 and 70, and `integrate` used each of
those values: the Zp leg mass in the channel lines matches. But
`parameters.json` showed 20 on all four. The bound value could only be read from
the integrate log.

The help text says a card `integrate` refuses is refused here too. That is true,
but a reader takes the flag to mean the report describes the card.

The companion `param_card.dat` is also always the restricted model's default
card (`default_param_card(model)` in `parameters.rs`), even with `--param-card`:
a run with MT moved to 172.5 writes a byte-identical `parameters.json` and
`param_card.dat`. simshop's Parameters wizard step builds on this command, so it
holds the edited values itself until the report carries them.
