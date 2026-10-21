---
type: Design
title: Run inputs the integrate artifact records (PDF set and param card)
description: "What the integrate artifact records of the PDF (lhaid, set, member, per-file SHA-256) and the param card (moved parameters); how generate re-reads them as checked assets; lhaid lookup across five tiers."
status: draft
tags: [artifact, pdf, lhapdf, param-card, run-card, cvmfs, network-consent, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
sources:
  - {id: mg-pdfid, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L3839-L3853", title: "banner.py get_pdf_id: built-in pdlabel to LHAPDF id"}
  - {id: mg-pdfwrap, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/PDF/pdfwrap.f#L224-L267", title: "pdfwrap.f: the data file and alpha_s each built-in pdlabel loads"}
  - {id: mg-copy, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/common_run_interface.py#L4554-L4572", title: "common_run_interface.py copy_lhapdf_set: lhaid must be an index key"}
  - {id: mg-lo-lhapdf, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madevent_interface.py#L6111-L6116", title: "madevent_interface.py: the LO path calls copy_lhapdf_set on lhaid"}
  - {id: mg-datadir, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/common_run_interface.py#L4870-L4893", title: "common_run_interface.py: LHAPDF_DATA_PATH is colon-separated; first entry with pdfsets.index wins"}
  - {id: index, resource: "https://lhapdfsets.web.cern.ch/current/pdfsets.index", title: "LHAPDF pdfsets.index (read 2026-10-20: 1598 lines, 47,618 bytes)"}
measured:
  command: "curl https://lhapdfsets.web.cern.ch/current/pdfsets.index on 2026-10-20; grep for the ids below"
---
# Run inputs the integrate artifact records

Two PRs add run inputs to `IntegrateArtifact`:
[pdf-set-not-taken-from-run-card](../backlog/feature/pdf-set-not-taken-from-run-card.md)
(the run card picks the PDF) and
[cli-has-no-param-card-option](../backlog/feature/cli-has-no-param-card-option.md)
(`integrate --param-card`). This concept fixes what the artifact records of
each, how `generate` re-reads them, and the PDF lookup. The artifact as a
whole, the run-card split and `generate`'s new command line are
[integrate and generate split by data dependency](integrate-generate-split.md),
which this concept assumes: `generate` takes no proc card, run card, param card
or PDF option, and binds everything from the artifact (user decision,
2026-10-21, `sprints/pdf-from-run-card/log.md`). The lookup half amends
[asset resolution](../tooling/asset-resolution.md) and
[network consent](../tooling/network-consent.md). Open choices are collected
in the split concept's last section; the ones decided here keep their
numbers (C1, C2, C4–C10, C12).

## 1. The fields

`pdf_set: String` and `pdf_member: u32` are replaced by one field,
`inputs: RunInputs`:

```rust
pub struct RunInputs { pub pdf: PdfInput, pub params: Vec<MovedParam> }
pub enum PdfInput {
    None,                                   // fixed-energy beams or a decay
    Lhapdf { lhaid: i64, set: String, member: u32, files: Vec<FileDigest> },
}
pub struct FileDigest { pub name: String, pub sha256: String }
pub struct MovedParam { pub name: String, pub block: String, pub code: Vec<i32>, pub value: f64 }
```

- **`lhaid`** is the id actually loaded, `SetIndex + member`. For a built-in
  `pdlabel` it is the label's equivalent (247000 for `nn23lo1`), not the card's
  ignored `lhaid`; the card's own `pdlabel` and `lhaid` stay in the recorded
  integrate settings.
- **`files`** (C1): SHA-256 (`sha2`, through `ufo::identity::digest_bytes`)
  of exactly the bytes the loader parsed, in load order: `<set>.info`, then
  `<set>_<mmmm>.dat`. The loader reads each file once and parses from that
  buffer (`parse_info` already takes a `&str`; the `.dat` parser gains the same
  form), so the hash and the grid cannot come from two different reads. The
  digest is per file, not per archive, because CVMFS and `$LHAPDF_DATA_PATH`
  have no archive in use, and a set reached from a different tier on a worker
  must compare equal when its files are equal. Members the run never read are
  not hashed. Measured on CVMFS (sprint log, 2026-10-20): the default set's
  `.info` and `_0000.dat` hash identically to the pinned download's.
- **`params`** (C2): every external parameter whose bound value differs from
  the restricted model's default, in the model's external order, after
  `integrate --param-card` is bound through `EvaluatedModel::from_model_card`.
  It is the **only** record of the card: `generate` rebuilds a `ParamCard`
  from these entries and binds the model with it. Because `evaluate` takes the
  card value or the default, that rebind gives the same `f64` for every
  external as integrate had. No card, an empty card and a card restating the
  defaults all record `[]` and give byte-identical artifacts. The defaults are
  pinned by `model.digest`, which hashes the restricted model.

## 2. Version

These fields land in the version step of
[the split concept](integrate-generate-split.md) (§2 there). No older file
carries a proc card, so `generate` cannot replay one; the old `Unrecorded`
upgrade and the write-down rule of the first draft are dropped with it.

## 3. What `generate` checks of these inputs

`card_mismatches`, `pdf_mismatches`, `refuse_on_mismatch`, `NO_PDF` and
`PDF_MEMBER` go: nothing is passed that could mismatch. What remains are asset
checks, after the model's digest check (split concept, §3):

1. **Params**: bind the model from `inputs.params`. A recorded name the model
   lacks, or one the restriction zeroed, cannot happen unless the model files
   changed, which the digest check already refused.
2. **PDF**: locate the recorded `set` by name through the tiers of §5
   (download with consent if absent), check `.info` `SetIndex + member ==
   lhaid` (the parser trims: CVMFS's 325300 `.info` writes `SetIndex: 325300 `
   with a trailing space), hash the two files and compare each with `files`
   (C9 for a mismatch):
   `PDF set {set} member {m} at {dir} ({tier}) is not the one the grid was
   trained on: {file} has sha256 {got}, the artifact recorded {want}. The set
   changed after vibegraph integrate ran (a CVMFS or LHAPDF update does this);
   re-integrate, or point $VIBEGRAPH_PDF_DIR at the original files.`
   `generate` never reads `pdfsets.index`: the set name is recorded.

## 4. Choosing the set and member

- **`pdlabel = lhapdf`**: the candidate is the index entry with the largest
  `SetIndex ≤ lhaid`; `member = lhaid − SetIndex`. The set is then located and
  its `.info` read (`SetIndex` is parsed for the first time; absent → refused).
  Accepted only if `.info SetIndex` equals the index entry and
  `member < NumMembers`. LHAPDF id ranges are disjoint, so a stale index that
  lacks a newer set can only fail this check, never pick a wrong set silently.
  Refusals: `lhaid {id} is below every set in {index paths}`;
  `lhaid {id} is in no known set: the nearest, {set} (SetIndex {s}), has
  {n} members (ids {s}..{s+n-1}); indices read: {paths}`.
- **MadGraph accepts only member 0.** LO `madevent` passes `lhaid` to
  `copy_lhapdf_set`, which raises unless `lhaid` is itself a key of
  `pdfsets.index`.[^mg-copy][^mg-lo-lhapdf] So a non-zero member is a
  vibegraph extension with no MadGraph reference; its test is synthetic (§6).
- **Pinned sets skip the index** (C5): `PinnedPdfSet` gains `set_index` and
  `num_members` (247000, 101), so the default lhaid needs no index fetch.
- **Built-in `pdlabel`s** (C4). αs stays the `pdfwrap.f` table value; only the
  densities come from LHAPDF:

| pdlabel | MadGraph loads[^mg-pdfwrap] | `get_pdf_id`[^mg-pdfid] | index today[^index] | verdict |
|---|---|---|---|---|
| `nn23lo1` | `NNPDF23_lo_as_0130_qed_mem0.grid` | 247000 | 247000 `NNPDF23_lo_as_0130_qed` | **admit** (user, 2026-10-20), member 0, with a logged note: same fit, not the same numbers; MadGraph's internal grid and LHAPDF's moved a `b b̄` σ by 9.8% ([pinned PDF set](../scales-pdf/pinned-pdf-set.md)) |
| `nn23lo` | `NNPDF23_lo_as_0119_qed_mem0.grid` | 246800 | 246800 `NNPDF23_lo_as_0119_qed` | refuse: names agree, no gate has compared the densities |
| `nn23nlo` | `NNPDF23nlo_as_0119_qed_mem0.grid` (its comment says `nlo_as_0130_qed_mc`) | 244800 | 244800 `NNPDF23_nlo_as_0119_qed` | refuse: MadGraph's own comment and file disagree |
| `cteq6l1` | `cteq6l1.tbl` through `Ctq6Pdf` | 10042 | 10042 `cteq6l1` | refuse: CTEQ's own table and interpolation, not an lhagrid1 file |
| `cteq6_m` | `cteq6m.tbl` | 10000 | 10000 is **`CoSMAO22Proton`** | refuse: the id has been reassigned |
| `cteq6_l` | `cteq6l.tbl` | 10041 | absent | refuse |
| `cteq6_d` | `cteq6d.tbl` | none | – | refuse |
| `none`, `iww`, `eva`, `edff`, `chff` | lepton or no PDF | 0 | – | refuse on proton beams |

  No built-in label has an *exact* LHAPDF equivalent: each reads MadGraph's
  own tabulation. `nn23lo1` is admitted because it is the default card's label
  and the user named its set; refusing it would refuse every proton run
  without a run card. Refusals name the label and suggest `pdlabel = lhapdf`
  with an explicit `lhaid`. The `cteq6_m` row is why `get_pdf_id` cannot be
  adopted wholesale: its id now names another set.

## 5. Lookup and consent

Tiers, for both a set directory and `pdfsets.index`, first hit wins:

| tier | set | index |
|---|---|---|
| 0. `$VIBEGRAPH_PDF_DIR` (C7) | `<dir>/<set>/` | `<dir>/pdfsets.index` |
| 1. `$LHAPDF_DATA_PATH`, split on `:`[^mg-datadir] | each `<entry>/<set>/` | each `<entry>/pdfsets.index` |
| 2. CVMFS `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current` | `<root>/<set>/`; `@DEPRECATED`, `@UNVALIDATED` and the `.tar.gz` twins are not sets | `<root>/pdfsets.index` (layout measured on a CERN host, sprint log 2026-10-20) |
| 3. cache `<root>/pdf/` | `<set>/` with sidecar | `pdfsets.index`, fetched copy |
| 4. HTTPS, consent | `lhapdfsets.web.cern.ch/current/<set>.tar.gz` | `…/current/pdfsets.index` (47.6 kB) |

- `--pdf-set` and `--pdf-dir` are removed; so is the cwd-relative
  `validation/pdf` dev fallback (C8). Tiers 0–2 are used as found, no checksum.
- **Index fall-through** (C6): the first index found is used; if the lhaid is
  "in no known set" there, the next tier's index is tried, and last the
  current index is fetched (consent) into the cache, replacing a stale copy.
- **Pinned sets keep their digest.** In tier 3 a pinned set is current only
  if its sidecar equals the pin (`pinned::is_current`), and a tier-4 fetch of
  it goes through `VerifiedFetch`. An unpinned set is current if its entry
  exists (the sidecar marks a complete publish, holding the archive's
  SHA-256 as fetched); `pdf_set_is_cached` changes to say so.
- **Prompts.** The index and the set are separate prompts (the set is not
  known until the index is read). An unpinned set's prompt cannot state a size
  or SHA-256 in advance and says so (C10); `network::Download` makes both
  optional.
- **Injection.** Resolution moves into `vibegraph-lib::cache` as a pure
  function over a `Tiers { vibegraph_pdf_dir, lhapdf_data_path: Vec<PathBuf>,
  cvmfs: Option<PathBuf>, cache_root }` value and a fetch-factory callback
  `&dyn Fn(&DownloadRequest) -> Result<Box<dyn Fetch>, String>`, where
  `DownloadRequest` is a library type carrying what `network::Download` shows
  (what, URL, optional size and SHA-256, destination). The CLI's callback
  is the only place that calls `network::confirm` and constructs `HttpFetch`,
  after `Granted`, so the structural invariant of network consent survives.
  Tests pass a callback that counts calls and returns an in-memory
  `MapFetch` (url → bytes) or a refusal: every tier, every refusal, with no
  socket and no `VIBEGRAPH_FETCH_CONSENT` (C12). The CLI reads the variables
  once (`Tiers::from_env`, untested like `locate_from_env`).
- Tests and scripts that passed `--pdf-dir` set `$VIBEGRAPH_PDF_DIR` (C7) and
  `env_remove("LHAPDF_DATA_PATH")`. A host with a real `/cvmfs` is still
  reachable from tier 2; refusal tests therefore use an lhaid and set name
  that exist in no index.

## 6. Tests

For `pdf-set-not-taken-from-run-card`, one line per `closes_when` clause:

- *no PDF option*: `integrate --pdf-set x` and `generate --pdf-dir x` fail as
  unknown arguments (CLI).
- *lhaid picks set and member*: unit, synthetic index + set `SetIndex 900000,
  NumMembers 3`: 900000 → member 0, 900002 → member 2; a `SetIndex` line with
  a trailing space. CLI, non-zero member: member 1 is member 0 with every knot
  ×2 (exact in binary), fixed budget, same seed; σ(member 1) / σ(member 0) = 4
  within 1e-12 for a qq̄ process.
- *built-in pdlabel*: unit, `nn23lo1` → `NNPDF23_lo_as_0130_qed` member 0;
  each refused label of §4 refused with its reason.
- *lhaid in no known set*: unit, below every set; past `NumMembers`; `.info`
  `SetIndex` disagreeing with the index; `.info` without `SetIndex`. CLI: the
  refusal names the lhaid and the index paths read.
- *each lookup tier*: unit on the library resolver over temp dirs, each tier
  alone and each shadowing the next; the `:` list; a CVMFS-shaped root with
  `@DEPRECATED/` and `<set>.tar.gz` entries; index fall-through; cache
  pinned-current, pinned-stale (refetched), unpinned-present; download
  granted, refused, and a pinned checksum mismatch that publishes nothing; the
  callback is never called when a local tier hits.
- *artifact records lhaid, set, member, checksum*: round trip; CLI integrate
  then read back: `files` equal the SHA-256 of the two files on disk.
- *generate reads them and refuses a changed checksum*: CLI, integrate on a
  synthetic set, append a comment line to the member `.dat` (parses the same,
  hashes differently), `generate` refuses naming the file; `generate` with the
  set reachable only from another tier with equal files succeeds.
- *the default card*: CLI, no `--run-card` on proton beams loads
  `NNPDF23_lo_as_0130_qed` member 0 and records lhaid 247000.

Informational, while the PDF PR is open: the survey's `p p > e+ e-` runs gave
one σ, 690.095053 pb, for the default card and for `pdlabel = lhapdf` at lhaid
325300 and 230000; afterwards the last two must differ from the first. Every
gated proton card is `nn23lo1` or `lhapdf`/247000, so every banked σ gate is
expected bit-for-bit unchanged; the PR confirms that rather than assumes it.

For `cli-has-no-param-card-option`: `params == []` for no card, an empty card
and a defaults card, and the artifacts compare byte for byte; a moved `MZ`
records one `MovedParam`; `generate` on that artifact writes events whose
replayed weights equal integrate's at the same points (the rebind is exact);
`generate --reweight-card` uses the recorded point as its denominator; a card
reviving a zeroed parameter is refused by `integrate`. The item's scope line
"`generate` uses the same card or refuses" is superseded by the 2026-10-21
decision.

## 7. What else moves

The command-line migration (every test and script that passes cards or PDF
options to `generate`) is in the split concept, §4. Specific to this concept:

- **Code**: `vibegraph-cli/src/{integrate,assets,network}.rs`;
  `vibegraph-lib/src/{artifact.rs,cache/*,pdf/mod.rs,pdf/grid.rs}`.
- **Tests passing `--pdf-set`/`--pdf-dir` to `integrate`**
  (`vibegraph-cli/tests/`): `cli_integrate`, `cli_generate_proton`,
  `cli_reweight_proton`, `cli_onshell_veto`, `cli_decay_chain`,
  `cli_decay_chain_events`, `validate_samples_proton`, `cli_pdf_cache`,
  `cli_pdf_cache_fetched`. `cli_first_run` and `cli_hard_errors` also remove
  `LHAPDF_DATA_PATH`.
- **Scripts**: `--pdf-dir` in `validation/pythia/generate_samples.sh`,
  `validation/pythia/generate_mlm_samples.sh`,
  `validation/madgraph/gen_reweight_oracle.py`,
  `validation/madgraph/bench_reweight.py`; `scripts/demo_pp.tape`.
  `scripts/acceptance.sh` passes no PDF flag: it unsets `VIBEGRAPH_PDF_DIR`,
  must also unset `LHAPDF_DATA_PATH`, and with C5 its `lhapdf`/247000 card
  still downloads only the set. Without C5, `validation/pdf/fetch.sh` must also
  fetch `pdfsets.index`.
- **Gates**: the `cli-pdf-cache-fetched` rationale in `validation/manifest.toml`.
- **Docs**: `docs/src/cli/reference.md`, `docs/src/cli/overview.md:96`.
- **Concepts**: `tooling/{asset-resolution,network-consent,pdf-set-distribution,release-acceptance}.md`,
  `scales-pdf/{pinned-pdf-set,alpha-s-sources}.md`.

[^mg-copy]: `common_run_interface.py`, `copy_lhapdf_set`, L4566–4572.
[^mg-lo-lhapdf]: `madevent_interface.py`, L6111–6116.
[^mg-pdfwrap]: `pdfwrap.f`, L224–267.
[^mg-pdfid]: `banner.py`, `get_pdf_id`, L3839–3853.
[^mg-datadir]: `common_run_interface.py`, `get_lhapdf_pdfsetsdir_static`, L4870–4893.
[^index]: `pdfsets.index` as served on 2026-10-20.
