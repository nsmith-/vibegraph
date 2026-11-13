---
type: Backlog Item
title: "Adding a subcommand moves the global options within every command's --help"
description: "clap orders global args by declaration position across subcommands, so registering enumerate reshuffled --no-network, -y, -v and others in every command's help and the whole generated CLI reference."
area: hygiene
state: open
priority: low
closes_when: "The global args carry an explicit display_order, and adding a subcommand changes only that subcommand's section of docs/src/cli/reference.md."
blocked_by: []
opened: 2026-11-10
tags: [cli, docs, clap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr120, resource: "https://github.com/nsmith-/vibegraph/pull/120", title: "vibegraph enumerate (found there)"}
---
Seen when `vibegraph enumerate` was added (nsmith-/vibegraph#120): the
regenerated `docs/src/cli/reference.md` diff touched every command, because
the global options moved. Pinning `display_order` on the global args in
`Cli` and `logging::LogArgs` keeps the reference diff to the new command.
