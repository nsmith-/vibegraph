---
type: Design
title: The fetched reference-data bundle
description: "Banked MadGraph runs ship as a sha256-pinned release asset with plain-text events; the member list is the contract, and a gate reading an unlisted file passes only locally."
status: draft
tags: [refdata, bundle, madgraph, banked-layer, fetch]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n25-census, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L198-L266", title: "Note 25 §4, inventory and reference-data census"}
  - {id: n25-52, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L283-L325", title: "Note 25 §5.2, one entry point and the bundle"}
  - {id: n25-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L561-L621", title: "Note 25 §9-10, decisions and close-out"}
  - {id: n25-rd2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L740-L765", title: "Note 25 §10, refdata-2"}
  - {id: n26, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/26-refdata-compact-representation.md#L16-L218", title: "Note 26, a compact banked-event representation: measurement and verdict"}
  - {id: n27-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L716-L911", title: "Note 27 B5, the 3.7.1 re-bank"}
  - {id: n27-b7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L1039-L1124", title: "Note 27 B7, the source-preserving round trip and the export proof"}
  - {id: n28-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L4136-L4326", title: "Note 28 Z.2-Z.7, refdata-4"}
  - {id: n28-zplan, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L239-L253", title: "Note 28 §5 Z, the close-out plan"}
  - {id: n29-g, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5832-L6009", title: "Note 29 G.5-G.12, the re-bank bundle and pin flip"}
  - {id: n36-ci, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L750-L768", title: "Note 36 7.2, the CI failure"}
  - {id: n38-71, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1226-L1246", title: "Note 38 7.1, what a bundle carries"}
  - {id: assemble, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/assemble_bundle.sh", title: "validation/madgraph/assemble_bundle.sh"}
---
# The fetched reference-data bundle

The banked validation layer reads frozen MadGraph runs without running MadGraph.
They are too large to commit, so they ship as one archive,
`vibegraph-refdata-<n>.tar.zst`, published as the release asset of tag
`refdata-<n>` and pinned by sha256 in `validation/manifest.toml`'s `[refdata]`
table[^n25-52]. It unpacks into `validation/madgraph/output/`, so a fetching
checkout and a machine that generated the runs present the gates with the same
paths. The [layers](validation-layers.md) are defined by what each may assume;
the banked layer may assume this bundle, the pinned submodule and the two fetched
PDF sets, and nothing else.

## What is in it

`assemble_bundle.sh` selects, per process directory, the files the banked gates
read: `Cards/` and `Events/` (event files, banners, seed records), the combined
`results.dat`, each subprocess's `leshouche.inc` and every `matrix[0-9]*_orig.f`
(one per diagram-group case; the glob once matched only `matrix1_orig.f` and a cut
silently lacked the others), the per-channel `run_*_log.txt` the `αs` source rule
is read from, and `build.log`; plus the fixed-grid amplitude tables[^assemble].

Left out on purpose: build products; generated Fortran beyond those two files;
`output/models/` (restaged by `build.sh` from the committed `validation/ufo/` and
cards, so carrying it would add a drifting second copy); the kT and MLM per-event
dumps and full process directories (oracle layer); shared-seed work areas, whose
scalars are committed JSON; and committed tables, which a generator rewrites and
the reproduction check compares[^n38-71].

**Event files travel decompressed** and the unpack gzips them back. Tarring
already-gzipped files under zstd compressed nothing further; carrying the
`.lhe` text instead made the archive about a third smaller, keeps a gzip encoder's
bytes out of the archive, and means a work area unpacked from a bundle
re-assembles to the same bundle. The byte round-trip gate keeps its meaning: it
compares Les Houches text, and the archive holds exactly the text it
asserts on[^n25-rd2].

## The pin is the contract

- **Byte-reproducible assembly**: members sorted in the C locale, one timestamp,
  mode and blank ownership, single-threaded zstd at a fixed level. Two assemblies
  of one work area hash the same, which makes the sha256 a pin rather than a
  snapshot. `assemble_bundle.sh --check` verifies an archive against the pin;
  the archive's bytes do not encode its own file name.
- **Fetch**: `pixi run fetch-refdata` (`fetch_refdata.sh` over
  `validation/fetch_common.sh`, the only place allowed to download). It asks for
  consent unless `VIBEGRAPH_FETCH_CONSENT=1` (what CI sets); while the repository
  is private the plain `url` 404s for unauthenticated clients and the fetch falls
  back to an authenticated `gh release download`; `$VIBEGRAPH_REFDATA_SOURCE`
  overrides both with a local file. The checksum is enforced on every route, and
  the check refuses a wrong cut by digest as well as accepting the right
  one[^n29-g]. The pattern is shared with the CLI's asset fetch
  ([tooling/asset-resolution](../tooling/asset-resolution.md)).
- **A work area that already holds process directories is left alone**: it is
  the oracle layer's cache and a superset of the bundle, and unpacking over it
  would replace locally generated files.
- **Changing the reference runs means a new `n`**: a new archive, a new pin and a
  comment line for the cut in `[refdata]`. Each re-cut costs a publish, so a
  close-out cuts once. How to cut is
  [refdata-banking-procedure](refdata-banking-procedure.md).

## The member list is the contract

A gate that reads a work-area file the bundle does not list passes on the
machine that generated the runs and fails on every fetching checkout, CI
included. That happened when a channel-count gate read each run's
`coloramps.inc`: green on the work area, red on CI's banked job. The fix was the
`diagrams.json` pattern: `extract_configs.py` writes the counts to a committed
`configs.json` in the `refs` stage and the gate reads that[^n36-ci]. A new banked
input either joins the bundle (a re-cut) or is extracted into a committed
reference. The same rule makes some gates oracle-layer for good:
`validate_mlm_dumps` reads `configs.inc`, `config_nqcd.inc` and
`config_subproc_map.inc` beside the dumps, so bundling the dumps alone would not
make it banked.

Rows in transit are marked in the manifest: `bundled = false` for a run banked
locally but not yet in the pinned cut (a fetching checkout lacks it, and its cells
read ⏳ "awaiting the bundle"), `status = "planned"` for a run that does not exist
yet. Both disappear when a cut carries the row, which restores the hard `require()`
on its cells ([process-manifest](process-manifest.md)).

## Why not commit a compact representation

The bundle was the fallback; the preferred endpoint was an event projection
small enough to commit. It was built and measured, and rejected[^n26]:
- **Size: fail.** A Parquet projection holding only the fields the gates read
  (per leg `IDUP`, `ISTUP`, `MOTHUP`, `ICOLUP`, momenta, mass, lifetime, `SPINUP`;
  per event `IDPRUP`, `XWGTUP`, `SCALUP`, `AQEDUP`, `AQCDUP` and the `<mgrwt>`
  replay payload) shrank the events 3.4×, to about 27.5 MB against a 5–10 MB
  target. Most of the gain was dropping `<rwgt>`: 61.8% of all the banked event
  text, all of it on the hadronic runs, and read by no gate.
- **The floor is information, not format.** The momentum columns were 81% of
  what remained. A component printed to eleven significant digits carries about
  36.5 bits, and re-encoding the printed decimals instead of doubles saved only 9%.
  Reaching 5–10 MB means keeping fewer events, which weakens the per-event and
  sample gates: a coverage decision, not a storage one.
- **Fidelity: pass.** Every retained value round-tripped exactly, and Rust's
  `str::parse::<f64>` on the printed tokens and Python on the Parquet columns
  hashed identically over 6.6 million values: both conversions are correctly
  rounded.
- **What the projection could not detect**: a field dropped from it entirely.
- The non-event components (banners, cards, `leshouche.inc`, `matrix*_orig.f`,
  logs, amplitude tables) compress to about half a megabyte and were always
  committable; they never made the bundle large.

The generator (`compact_events.py`) and its `lhe-compact` environment were
deleted afterwards, so these numbers are not reproducible from the current tree;
the verdict stands and note 26 keeps the measurements. The byte round-trip gate
needs raw text by construction, which no projection can serve; had the projection
won, two or three short raw runs reaching every layout this crate writes would
have been kept for it, rather than moving the gate to the oracle layer.

## Reading numbers across cuts

A σ banked in one cut is not always comparable with the same row in another: the
3.5.7 → 3.7.1 re-bank changed the `αs` of fixed-energy runs, and the `nn23lo1` →
LHAPDF re-card changed the beams ([refdata-sigma-comparability](refdata-sigma-comparability.md)).
The manifest's per-cut comments say what each cut changed; superseded runs are kept
in a local retired area off the bundle, recoverable by hand. Licensing of banked
MadGraph outputs is [tooling/licensing](../tooling/licensing.md).

## Proving a cut

Every cut is accepted on the same evidence: two assemblies byte-identical; each
run's decompressed event text sha256-stable through pack and unpack; and a clean
`git archive` export, given the pinned submodule content, the two PDF sets and the
bundle alone, running the banked layer green with a report byte-identical to the
work area's[^n28-z][^n27-b7]. Byte-identical reports are the strong form: a
fetching checkout does not merely pass, it produces the same table.

[^n25-52]: Note 25 §5.2 and decision 3.
[^assemble]: `validation/madgraph/assemble_bundle.sh`, header comment and member selection; manifest cut 6 for the `matrix*_orig.f` glob.
[^n38-71]: Note 38 §7.1 and note 41 Z.1.
[^n25-rd2]: Note 25 §10 "`refdata-2`", taking note 26's incidental finding.
[^n29-g]: Note 29 G.9.
[^n36-ci]: Note 36 §7.2.
[^n26]: Note 26.
[^n28-z]: Note 28 Z.2–Z.3.
[^n27-b7]: Note 27 B7, the `refdata-3` export proof.
