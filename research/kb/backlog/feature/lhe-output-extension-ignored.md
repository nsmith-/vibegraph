---
type: Backlog Item
title: "generate -o ev.lhe.gz or ev.lhe.zst writes plain text under a compressed name"
description: "generate exits 0 and reports writing ev.lhe.gz, but the file is uncompressed LHE; the same for .zst. A silently mislabelled output, against the hard-error rule."
area: feature
state: open
priority: medium
closes_when: "generate either compresses by the output extension (.gz, .zst) or refuses a compression extension by name; plain LHE stays the default for any other name; a CLI test checks the first bytes for each extension."
blocked_by: []
opened: 2026-11-13
tags: [cli, generate, lhef, refusal, batch]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session: `generate -o ev.lhe.gz` exits 0
and prints "wrote ev.lhe.gz", but the file begins `<L`, which is plain text. The
same happens with `.zst`.

Compressing by extension is worth having. With the default `use_syst` (147
weights per event: the scale envelope plus the 325300 Hessian and αs set),
2,000 Drell–Yan events are 12.5 MB plain and 2.0 MB gzipped. Plain must stay
the default: CMS's `run_generic_tarball_cvmfs.sh` expects a plain
`cmsgrid_final.lhe`. If compression is not built, refusing the extension by
name meets the bar, because the defect is the silent mislabel.
