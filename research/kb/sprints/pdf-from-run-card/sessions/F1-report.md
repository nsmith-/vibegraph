---
type: Session Report
title: "F1 report: the PDF from the run card (H2, library half)"
description: "RunInputs at artifact version 14, lhaid → (set, member) through the index with fall-through, every built-in pdlabel refused, lhapdf/247000 as the default card, per-file SHA-256 of what the loader parsed, the four-tier lookup behind an injected fetch factory, and generate's checksum refusal."
status: draft
tags: [pdf, lhapdf, run-card, artifact, cvmfs, network-consent, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/H`, branch `sprintd-pdf-from-run-card`, from e5e00be.
All cargo commands ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`.

## What changed

**Artifact, version 14** (`vibegraph-lib/src/artifact.rs`). `pdf_set` and
`pdf_member` are replaced by `inputs: RunInputs { pdf: PdfInput, params }`,
with `PdfInput::{None, Lhapdf { lhaid, set, member, files }}`, `FileDigest`
and `MovedParam` as in the design §1. `FORMAT_VERSION` and
`OLDEST_READABLE_VERSION` are 14; a version-13 file is refused with the
existing `UnsupportedVersion`. A non-empty `params` is refused on writing and
on reading (`ArtifactError::MovedParams`).

**Read once, hash what was parsed** (`pdf/mod.rs`, `pdf/grid.rs`). `PdfSet::load`
and `PdfSet::member` read each file once (`read_hashed`), SHA-256 the bytes
through `ufo::identity::digest_bytes`, and parse from the same buffer.
`PdfSet::info_file()` and `PdfMember::file()` expose the digests. The `.dat`
parser takes `&str` only; the path-reading wrappers are gone. `.info`
`SetIndex` is parsed (trimmed, optional).

**Choosing the set and member** (`pdf/select.rs`, new; `cache/index.rs`, new).
`load_for_run_card` → `RunCard::pdf_choice()` → `load_for_lhaid`. An id in a
pinned set's range (`PinnedPdfSet` gains `set_index`, `num_members`; C5)
needs no index. Otherwise each local `pdfsets.index` is tried in tier order,
the candidate is the largest `SetIndex ≤ lhaid`, the set is located and its
`.info` must give that same `SetIndex` (absent → refused, different →
refused) and `lhaid − SetIndex < NumMembers`; an index that does not cover the
id falls through to the next, and last the current index is fetched into the
cache with consent, replacing a stale copy (C6). Refusals carry the design's
two messages; when the index cannot be fetched the refusal names the lhaid
and the indices read.

**Built-in pdlabels** (`runcard.rs`, `coupling/alphas.rs`). `RunCard::pdf_choice`
returns `PdfChoice::None` on fixed-energy beams (decays included) and refuses
every label but `lhapdf` on proton beams with the design's message, the
`247000` hint for `nn23lo1` only. `pdf_label_alpha_s` and its table are gone;
`RunningAlphaS::from_run_card` routes through `pdf_choice`.
`PARAM_DEFAULTS` gives `lhapdf` on all three label rows and `lhaid = 247000`,
with the departure documented above the rows; the `banner.py` oracle test
records the four departures and still checks MadGraph's own values. `PDFSUP`
of the default card stays 247000. The default-card writer is the
`<MGRunCard>` writer, which reads these values; it needed no code change.

**Lookup** (`cache/lhapdf.rs`, new). `Tiers { lhapdf_data_path,
data_path_tier, cvmfs, cache_root }` (`data_path_tier` says whether the list
came from `LHAPDF_DATA_PATH` or `LHAPATH`, so the tier can be named),
`Tiers::from_env`, `locate_set`, `fetch_index`, `DownloadRequest` and
`FetchFactory`. CVMFS skips `@…` entries and `.tar.gz` twins. In the cache a
pinned set must match its pin and an unpinned one must carry its sidecar;
`pdf_set_is_cached` says the same. Pinned downloads go through `VerifiedFetch`;
unpinned ones are published with the archive SHA-256 in the sidecar. Tests use
an in-memory `MapFetch` and a counting factory: no socket, no
`VIBEGRAPH_FETCH_CONSENT`. `resolve::locate_pdf_set` and `Source::DataPath`
are removed.

**Consent stays structural** (`vibegraph-cli/src/assets.rs`, `network.rs`).
`assets::pdf_fetch(policy)` is the factory: the only place that calls
`network::confirm` for a PDF and constructs `HttpFetch`, after `Granted`.
`network::Download` has `bytes: Option<u64>` and `sha256: Option<&str>`; an
unpinned prompt prints "unknown (not pinned by this build)" (C10).

**generate** (`pdf/select.rs::load_recorded`, called from
`vibegraph-cli/src/generate.rs`). Finds the recorded set by name, compares
the `.info` digest, checks `SetIndex + member == lhaid`, loads the member and
compares the `.dat` digest; a mismatch is the design §3 message naming the
file and both digests (C9).

## Deviations and CLI changes forced by the library

- **`--pdf-set` and `--pdf-dir` stay on `integrate`** (F2 removes them), with
  the meaning the card now forces: `--pdf-set NAME` names the set the card's
  `lhaid` lies in (its first id read from the set's `.info`, the member range
  checked, no index read; `pdf::select::load_named`), and `--pdf-dir DIR` is
  put ahead of the LHAPDF data path. Without `--pdf-set` the card decides.
  Both help texts changed, so `docs/src/cli/reference.md` was regenerated
  with `scripts/gen-cli-docs.sh` (the `cli_reference_docs` gate).
- **Order of the generate checks**: the `.info` digest is compared before
  the `SetIndex + member == lhaid` check, so a `.info` whose `SetIndex` moved
  is reported as a changed file (C9's message, naming it) rather than as an
  id mismatch. The design lists the id check first.
- **Error type**: `PdfSelectError` boxes its reason (`kind()` returns
  `SelectFailure`) to satisfy `clippy::result_large_err`.
- **Tests touched to compile or because the behaviour is this session's**:
  `cli_integrate.rs` now asserts the recorded `PdfInput` equals lhaid 247000,
  member 0 and the SHA-256 of the two files on disk; `cli_generate_proton.rs`,
  `cli_fixed_energy.rs` read `inputs.pdf` and `FORMAT_VERSION`;
  `cli_pdf_cache.rs`'s unpinned-set test now expects the consent refusal with
  unknown size and checksum (an unpinned set is downloadable), plus two new
  CLI tests (an unindexed lhaid, a built-in pdlabel at integrate).

## Tests and mutation evidence

New tests (library unless marked): `cache::index` (2), `cache::lhapdf` (8),
`pdf::select` (10), `pdf::grid::set_index_is_trimmed_optional_and_typed`,
`pdf::tests::a_loaded_set_records_the_digest_of_the_bytes_it_parsed`,
`artifact::{a_version_13_artifact_is_refused, an_artifact_that_read_no_density_records_none, moved_parameters_are_refused_on_writing_and_reading}`,
`runcard::{the_default_card_reads_lhapdf_247000, every_label_but_lhapdf_is_refused_on_proton_beams}`,
`cache::pinned::cached_means_at_the_pin_or_completely_extracted`,
`coupling::alphas::{a_built_in_label_on_proton_beams_is_refused, a_built_in_label_is_refused_even_with_a_set_at_hand}`,
`network::an_unpinned_download_says_its_size_and_checksum_are_unknown` (CLI
unit), `cli_pdf_cache::{an_unindexed_lhaid_is_refused_naming_the_id, a_built_in_pdlabel_is_refused_at_integrate}` (CLI).

Mutation sweep: a script applied one mutation, ran
`cargo test -p vibegraph-lib --lib -- <tests>` (M20:
`cargo test -p vibegraph --bin vibegraph -- an_unpinned_download_says`),
and restored the file byte for byte; `git diff --stat` after the sweep was
unchanged.

| id | mutation | result |
|---|---|---|
| M1 | `pdf_choice` accepts every label | caught (exit 101) |
| M2 | 247000 hint on every label | caught |
| M3 | default `lhaid` back to 230000 | caught |
| M4 | index candidate `SetIndex < lhaid` | caught |
| M5 | `.info` `SetIndex` not compared with the index | caught |
| M6 | missing `SetIndex` taken as the index's | caught |
| M7 | member range `≤ NumMembers` | caught |
| M8 | no index fall-through | caught |
| M9 | pinned range ignored | caught |
| M10 | digest of the path, not the bytes | caught |
| M11 | recorded `.dat` digest not compared | caught |
| M12 | recorded `.info` digest not compared early | survived first; the test then gained a `.info` with a moved `SetIndex`, and the rerun caught it (exit 101) |
| M13 | recorded lhaid not checked | caught |
| M14 | CVMFS non-set names accepted | caught |
| M15 | unpinned cache entry current without sidecar | caught |
| M16 | pinned cache entry current at any checksum | caught |
| M17 | unpinned request claims a size | caught |
| M18 | moved params accepted on read | caught |
| M19 | `FORMAT_VERSION` back to 13 | caught |
| M20 | unpinned prompt prints a size | caught |
| M21 | `SetIndex` not parsed | caught |

## Gates

```
$ cargo fmt --all --check                                   exit 0
$ cargo clippy --workspace --all-targets -- -D warnings     Finished … exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
                                                            Finished … exit 0
$ cargo build --workspace                                   Finished … exit 0
$ cargo test --workspace --no-fail-fast                     exit 0
  (every target "test result: ok"; lib unit tests 1083 passed, 12 ignored)
$ python3 scripts/kb.py lint                                exit 0 (4 pre-existing
  backlog warnings: `state: blocked` without `blocked_by`)
```

Extended proton CLI suites (`cargo test -p vibegraph --features
extended-validation --test <t>`):

| suite | result |
|---|---|
| `cli_integrate` | ok, 4 passed |
| `cli_pdf_cache_fetched` | ok, 3 passed |
| `cli_decay_chain` | ok, 1 passed |
| `cli_generate_proton` | 1 passed, 4 failed: all `validation::require` (no MadGraph reference bundle in this worktree) |
| `cli_reweight_proton` | 1 passed, 1 failed: `validation::require` |
| `cli_decay_chain_events` | 6 passed, 1 failed: `validation::require` |
| `validate_samples_proton` | 11 failed: `validation::require` or reading `validation/madgraph/output/<run>/Events` (absent) |

All 17 panics in those runs are at `vibegraph-lib/src/validation.rs:32`
(`require`) or `validate_samples_proton.rs:110` (the banked LHE read): this
worktree has `validation/pdf` but no `validation/madgraph/output`. The suites
that need only the PDF set pass. Every proton suite that compares against a
banked MadGraph run is therefore unmeasured here, including the expectation
in the design §4 that no gated σ moves.

## Found

- **The banked proton gates were not run.** `validation/madgraph/output` is
  absent from `/home/user/wt/H`, so `cli_generate_proton`,
  `validate_samples_proton`, `cli_reweight_proton` and one
  `cli_decay_chain_events` test stop at `validation::require`. The default
  card change moves every inline proton card without a `pdlabel` line from
  the `nn23lo1` αs table to the 247000 grid's αs; whether any pinned number
  moves is unmeasured. Closing it: run those suites (and `pixi run --skip-deps
  validate`) with the reference bundle present, and list each moved pin.
- **A stale local index can trigger a wrong set download.** When a local
  `pdfsets.index` lacks a newer set, its candidate for the id is an older set
  whose range cannot be checked without that set's `.info`; if the older set
  is not local, the lookup asks to download it before falling through to the
  next index. Evidence: `try_table` in `pdf/select.rs` locates the candidate
  through every tier. Closing it: try every local index whose candidate set
  is local first, and download a candidate only from the freshest index.
- **The index prompt says "unpacks to".** `network::Download::terms` is
  shared by set archives and `pdfsets.index`, a plain file. Closing it: a
  wording field on `DownloadRequest`, or "lands at".
- **`--pdf-set`/`--pdf-dir` interim semantics** (above) are F2's to remove
  with the test and script migration.
