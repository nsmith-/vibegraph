---
type: Backlog Item
title: "build.sh adds the C++ runtime for LHAPDF only on Darwin"
description: "build.sh appends -lc++ on Darwin and nothing elsewhere, so a pdlabel = lhapdf build on Linux needs LDFLAGS with -Wl,--as-needed stripped and -lstdc++ added by hand."
area: hygiene
state: open
priority: low
closes_when: "build.sh links a pdlabel = lhapdf process directory on Linux with no LDFLAGS set by hand (libstdc++ added, and --as-needed not dropping it), as madevent_seeds.sh and time_stages.py already do, checked by one Linux build."
blocked_by: []
opened: 2026-11-08
tags: [madgraph, build, lhapdf, linux, tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: build.sh has no Linux LHAPDF link path"}
  - {id: ha2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-A2-report.md", title: "H-A2 report, §3 and Found: the LDFLAGS workaround"}
  - {id: build, resource: "../../../../validation/madgraph/build.sh", title: "build.sh, the uname case near line 203"}
---
`validation/madgraph/build.sh` (about line 203) appends `-lc++` to `LDFLAGS`
on Darwin and passes `LDFLAGS` through unchanged elsewhere. The conda
activation's own `LDFLAGS` suppresses MadGraph's `STDLIB`, so a
`pdlabel = lhapdf` madevent link on Linux leaves LHAPDF's C++ symbols
unresolved. The CERN host's workaround is
`LDFLAGS="$(echo $LDFLAGS | sed s/-Wl,--as-needed//) -lstdc++"`.

The repository already has the Linux branch elsewhere:
- `madevent_seeds.sh:34` adds `-lstdc++`;
- `time_stages.py:63` picks `-lstdc++` off Darwin.

Whether `--as-needed` must also be stripped is the host's finding and has not
been re-checked here (inferred from the workaround).
