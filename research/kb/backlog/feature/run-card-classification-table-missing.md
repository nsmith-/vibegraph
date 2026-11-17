---
type: Backlog Item
title: "No machine-readable table says how each run-card parameter is treated"
description: "Consumers cannot tell which run-card parameters vibegraph honours, ignores or refuses without reading code names like R_UNIMPL; ship a table per release with a one-line note each."
area: feature
state: open
priority: medium
closes_when: "Each release ships a machine-readable table (JSON, generated from the code's own parameter lists, not hand-kept) giving every known run-card parameter as honoured, ignored or refused (with the refused values when only some are), plus a one-line human note; a test fails when a parameter is added to the code's lists without a table entry."
blocked_by: []
opened: 2026-11-17
tags: [run-card, cli, json, release, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Requested by simshop, whose sectioned run-card form wants to grey out, warn on
or block each field. Its notes today are code names such as `R_UNIMPL`. The
table could be a `vibegraph run-card --describe --json` command or a release
asset. Either way it must come from the lists the run-card parser and the cut
compiler already hold (`UNIMPLEMENTED_CUTS`, `NONPOSITIVE_IS_OFF` and the like),
so it cannot drift from the code.
