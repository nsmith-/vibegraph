---
type: Backlog Item
title: "vibegraph has no command that writes a CMS runcmsgrid.sh gridpack"
description: "CMSSW's ExternalLHEProducer runs a tarball's runcmsgrid.sh <nevents> <seed> <ncpu> and reads cmsgrid_final.lhe; today a downstream tool assembles that tarball from the binary and the artifact by hand."
area: feature
state: open
priority: low
closes_when: "`vibegraph export-gridpack <artifact> -o <name>.tar.xz` writes a tarball whose runcmsgrid.sh, run the way run_generic_tarball runs it, writes cmsgrid_final.lhe from the artifact alone, refusing a missing PDF set by name; a test runs it that way."
blocked_by: [generate-artifact-not-self-contained]
opened: 2026-11-23
tags: [cms, gridpack, generate, artifact, v0.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: self-contained, resource: "generate-artifact-not-self-contained.md", title: "Backlog: a generate run needs the PDF set beside the binary and the artifact"}
---
CMSSW's `ExternalLHEProducer` fetches a gridpack tarball and runs
`run_generic_tarball` on it. That script untars it and calls
`./runcmsgrid.sh <nevents> <seed> <ncpu>`, then reads `cmsgrid_final.lhe` from
the working directory.

A v0.2.2 tarball needs three files:

- the static musl x86_64 binary;
- the integration artifact;
- a `runcmsgrid.sh` that calls
  `vibegraph generate <artifact> --seed $2 --nevents $1 -j $3 -o cmsgrid_final.lhe`.

The PDF sets come from `LHAPDF_DATA_PATH`.

The user ruled on 2026-11-23 that downstream tooling assembles this tarball
for now, and that vibegraph owns the command in v0.3. It is paired with
bundling the PDF data into the artifact, piece 2 of
[generate-artifact-not-self-contained](generate-artifact-not-self-contained.md),
so that the tarball does not depend on the worker's `LHAPDF_DATA_PATH`.

Out of scope here (each has its own item):

- whether the LHE headers carry what CMSSW's Pythia8 JetMatching reads:
  [pythia-gate-header-semantics-unchecked](../validation/pythia-gate-header-semantics-unchecked.md);
- the merging cut's PDG codes:
  [mgruncard-pdgs-for-merging-cut-unverified](../validation/mgruncard-pdgs-for-merging-cut-unverified.md);
- the banner's `iseed = 0`:
  [lhef-banner-records-no-seed](../validation/lhef-banner-records-no-seed.md).
