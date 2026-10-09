---
type: Procedure
title: "Measuring a no-change claim: report diffs and binary byte identity"
description: "How 'nothing else moved' is measured: per-cell report JSON and the rendered report diffed in both work-area states, and two binaries in separate targets compared on grids and LHE."
status: draft
tags: [validation, byte-identity, regression, procedure, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n28-k5b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3237-L3272", title: "Note 28 K5b.6, the report differing on five rows"}
  - {id: n28-c23, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3888-L3917", title: "Note 28 C2.3, blast radius measured"}
  - {id: n28-z5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L4259-L4287", title: "Note 28 Z.5, the report diffed"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1, byte identity at ickkw = 0"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L758-L1013", title: "Note 41 M2, the shared-target trap"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3, byte identity with binary hashes"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, byte identity"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2701", title: "Note 41 F-B and P12, when byte identity cannot hold"}
  - {id: n29-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5102-L5167", title: "Note 29 B.6-B.7, the before/after comparison"}
---
# Measuring a no-change claim

"Nothing else moved" is a measurement, not a reading of the diff. Two
instruments make it: a diff of the validation report taken before and after the
change, and a byte comparison of two binaries' outputs. Both are read against a
set of cells or cases written down beforehand
([pre-registered-verdicts](pre-registered-verdicts.md)).

## 1. The report diff

`pixi run --skip-deps validate` writes one JSON per measured cell under
`target/validation-report/{integrals,samples}/` (`sigma_vg_pb`, `sigma_vg_err_pb`,
`rel`, `pull`, `chi2_dof`, `per_seed`) and renders `report.md`;
`validation/validate.sh` deletes the directory first, so every cell is this
invocation's measurement ([validation-report](validation-report.md)). Always
`--skip-deps`: a bare `pixi run validate` takes the MadGraph dependency live and
starts a multi-hour regeneration.

Steps[^n29-b6]:
1. Before touching a production line, on a clean tree: run it, copy
   `target/validation-report` aside (`cp -Rc`, instant on APFS), and record the
   census line.
2. Make the change, re-run, and `diff -r` the two `integrals` and `samples` trees.
   A digest over the tree (`find … -type f | sort | xargs shasum -a 256 | shasum
   -a 256`) is a one-line record of a baseline.
3. Diff the rendered `report.md` too, with footnote indices normalised away:
   footnotes renumber after any row that moves, so a raw diff of a one-row change
   moves tens of lines.
4. Read every difference against the pre-registered set. A cell that may not
   move must be byte-identical JSON, not "within tolerance"; any other movement is
   stop-and-report, never retune, re-seed or widen.

**Run both work-area states.** A change that touches bundle membership is
measured with the unbundled runs present, and from the bundle alone (a clean
`git archive` export given only the pinned submodule, the fetched PDF sets and
the bundle, or the same tree with the unbundled runs held out). With nothing
standing on files only one machine has, the two rendered reports are
byte-identical, or differ exactly where the held-out runs' cells read ⏳
"awaiting the bundle"[^n28-z5].

Worked examples:
- A gate fix reaching one process: the rendered tables differ on one row
  (`pp_to_jj`), two appendix lines and the census; the held-out report is
  character-identical across all 78 measurement lines to the earlier held-out
  one[^n28-c23].
- A close-out pruning one row and re-wording eight blocker strings: the tables
  differ in exactly four kinds of place, and no ✅ or ⚠️ value changed.
- A feature reaching five rows: the two rendered tables differ on exactly those
  five rows, a stronger statement than "unmoved to the printed digit"[^n28-k5b].

## 2. Binary byte identity

For a change that must leave existing output untouched (a code path only a new
card reaches), build the base and the new binary and compare their outputs on
fixed cases[^n41-m1].

- **Separate targets.** Build each in its own worktree **and** its own
  `CARGO_TARGET_DIR`. A shared target does not separate two worktrees of one
  workspace: cargo hashes path packages workspace-relative, so the second build
  silently reuses the first's binary until `cargo clean -p` forces it, and a
  concurrent session building into the same target can overwrite a test binary
  mid-run. Record each binary's sha256 and confirm they differ before trusting
  any comparison[^n41-m2]. See
  [tooling/cargo-configuration](../tooling/cargo-configuration.md).
- **The six cases.** `p p > e+ e- j` on `pp_to_llj_fixed`'s and `pp_to_llj`'s
  banked cards, `p p > j j`, `p p > e+ e-`, and the fixed-beam
  `g u > e+ e- u` and `e+ e- > mu+ mu-`; `integrate --fixed-budget` 20k × 4 at
  seed 7 and `generate` 500 events, `RAYON_NUM_THREADS=2`.
- **Compare** every `grid.bin.zst` by sha256 and every LHE file byte for byte.
  The header line naming the artifact path always differs; nothing else may.
- **Recorded digests are per binary epoch.** Grid sha256 prefixes reproduce only
  between binaries whose sampling did not change; re-establish the base's digests
  from the base binary, never from a note.

## 3. When byte identity cannot hold

Some changes alter what a case draws while leaving every value at a point
unchanged. Say which class the change is, and measure accordingly[^n41-fb]:

- **Statistical.** Merging channels that share a map changes channels, grids
  and streams wherever a map is shared, and every hadronic case of the six
  merges. Show σ instead with a five-seed sweep at a fixed budget, base and new
  interleaved, and report the shift in seed-spread units; only the fixed-beam
  cases stay byte-identical.
- **Confined to named fields.** A writer policy that rescales weights leaves
  grids identical; compare the LHE files column by column: events differing
  outside `XWGTUP` (zero), the `XWGTUP` ratio per part (uniform), `<init>`
  process lines and the header.
- **Inert plumbing first.** Land new plumbing with its output discarded and
  require byte identity, then switch it on: if any cell moves at the inert stage,
  the stream discipline is wrong and nothing should be built on it[^n29-b6].

## Caveats

- Byte identity on six cases shows those six are untouched, not the code path in
  general. Pin the claim with a hermetic test where one exists
  (`without_matching_the_record_scale_is_the_density_scale`,
  `scalup_reads_the_record_scale`).
- An artifact writer that records the oldest format version whose schema holds
  its keys is what keeps unaffected artifacts byte-identical across a format bump.
- `CARGO_PROFILE_RELEASE_DEBUG_DEBUG` cannot set the `release-debug` profile's
  debug level (cargo reads it as a key under `profile.release` and fails); use
  `--config 'profile.release-debug.debug=0'` on a cargo line, or an untracked
  `.cargo/config.toml` for `validate.sh` and pixi tasks.

[^n29-b6]: Note 29 B.6 (the procedure) and B.7 (stage B-1a, the inert-plumbing control).
[^n28-z5]: Note 28 Z.3 and Z.5.
[^n28-c23]: Note 28 C2.3.
[^n28-k5b]: Note 28 K5b.6.
[^n41-m1]: Note 41 M1, "Byte identity at `ickkw = 0`"; repeated in M2–M4.
[^n41-m2]: Note 41 M2, implementation and dump-gate records.
[^n41-fb]: Note 41 F-B ("Byte identity: M1's six cases") and P12.
