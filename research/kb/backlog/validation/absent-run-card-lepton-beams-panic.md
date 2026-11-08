---
type: Backlog Item
title: "With no run card, lepton-beam processes get proton beams and panic"
description: "MadGraph writes lpp = 0, ebeam = 500 for e+ e- > mu+ mu- with no run card; vibegraph keeps lpp = 1 and integrate panics at proton.rs:945 on parton -11."
area: validation
state: open
priority: high
closes_when: "The absent-run-card default follows create_default_for_process's beam rules (lepton, e p, neutrino and EVA beams), a test pins lpp and ebeam for e+ e- > mu+ mu- against MadGraph's written card, and no card reaches a panic on an untabulated parton."
blocked_by: []
opened: 2026-11-05
tags: [run-card, defaults, beams, panic, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: hj1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-J1-report.md", title: "H-J1 report, Found 1"}
---
Found by sprint H's H-J1 (`sprinth-maxjetflavor`). For `e+ e- > mu+ mu-` with no run
card, MadGraph's `create_default_for_process` (`banner.py`) writes `lpp = 0`,
`ebeam = 500` and `use_syst = F`. vibegraph's absent-card default keeps
`lpp = 1`, and `integrate` panics at `proton.rs:945` ("initial-state parton -11
is not a tabulated parton density", exit 101). An explicit empty card does the
same, so this predates the `maxjetflavor` fix.

The fix belongs in `RunCard::scattering_default`, alongside the `maxjetflavor`
rule. Any parton the PDF does not tabulate should be refused, not panic.
