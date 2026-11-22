---
type: Procedure
title: Standing session roles and names
description: "The long-lived sessions around vibegraph, each named vg-<role>: planner, router, validator, batch, simshop and the vg-sprint-<letter> sessions, what each owns, and how messages flow between them."
status: draft
tags: [sessions, process, agents, messaging]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: sprint-l, resource: "../sprints/v023-orders-majorana-timeouts/sprint.md", title: "Sprint L plan, hosts and sessions"}
---
Every standing session carries a `vg-` name for its role. Plans, briefs and
reports name sessions by these names, not by host or by person.

| Name | Where | Owns |
|---|---|---|
| `vg-planner` | cloud | Sprint plans, the backlog, the roadmap, merge decisions with the user's approval, and instructions to the other sessions. |
| `vg-router` | Remote Control, the validator's machine | Carrying messages verbatim between the planner and the sprint sessions. Nothing else: no files, builds, GitHub posts or merges. |
| `vg-validator` | Remote Control, at CERN | Batch campaigns, MadGraph regenerations, local branch builds for long or heavy checks, the targeted check before a tag, and cutting the tag. |
| `vg-batch` | Remote Control, at CERN | Submitting pinned release binaries to the batch system for the validator. |
| `vg-simshop` | Remote Control | simshop: it consumes vibegraph releases and reports what it needs, as backlog requests through the planner. |
| `vg-sprint-<letter>[-<half>]` | cloud | One sprint, or one half of one. Sprint L runs as `vg-sprint-L-physics` and `vg-sprint-L-pipeline`. |

## Messaging

A cloud session put in Auto mode from a trusted device becomes elevated. An
elevated session accepts cross-session messages only from sessions on the
user's trusted devices, which means Remote Control sessions. Cloud sessions
never count as trusted.

This rule is inferred from which sends succeeded and failed, not from
documentation. On that reading:
- the planner and an elevated sprint session cannot message each other;
- the router can reach both, because it runs on a trusted device;
- both can reach the router, because it stays out of Auto mode and is not elevated.

So sprint sessions address reports to the router with a `To: vg-planner` line,
and the planner sends them instructions the same way. The validator, batch and
simshop sessions message the planner directly. They can also send through the
router, with a `To:` line naming one or more sessions, to reach a sprint session.
The router forwards verbatim and always copies the planner.

## Resources

The router runs on the validator's machine, so it uses messaging only: that
machine's CPU and disk belong to the validator's checks. Sprint sessions run in
4-core, 15 GB cloud containers, one heavy job at a time. Long or heavy runs go
to the validator.
