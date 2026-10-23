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
[network consent](../tooling/network-consent.md). The twelve choices of the
first draft are **settled** (user, 2026-10-21, sprint log): C1 per-file
SHA-256 of what was read, generate-read files recorded by `generate`; C2 moved
externals only; C3 older artifacts refused; C4 every built-in `pdlabel`
refused; C5 pins carry `set_index`/`num_members`; C6 index fall-through;
C7 `LHAPDF_DATA_PATH` replaces `VIBEGRAPH_PDF_DIR`; C8 no `validation/pdf`
fallback; C9 a checksum mismatch is refused; C10 the unpinned prompt says size
and SHA-256 are unknown; C11 always the new version; C12 the download tier
tested at library level, the consent prompt separately. The `(Cn)` tags below
mark where each applies. What is still open is listed at the end of
[the split concept](integrate-generate-split.md).

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

- **`lhaid`** is the id actually loaded, `SetIndex + member`, equal to the
  card's `lhaid`: every built-in `pdlabel` is refused where a density is read
  (§4), so `pdlabel = lhapdf` is the only way a proton run names its PDF.
- **`files`** (C1): SHA-256 (`sha2`, through `ufo::identity::digest_bytes`)
  of exactly the bytes the loader parsed, in load order: `<set>.info`, then
  `<set>_<mmmm>.dat`. The loader reads each file once and parses from that
  buffer (`parse_info` already takes a `&str`; the `.dat` parser gains the same
  form), so the hash and the grid cannot come from two different reads. The
  digest is per file, not per archive, because CVMFS and `$LHAPDF_DATA_PATH`
  have no archive in use, and a set reached from a different tier on a worker
  must compare equal when its files are equal. Members the run never read are
  not hashed. Measured on CVMFS (sprint log, 2026-10-20): the default set's
  `.info` and `_0000.dat` hash identically to the pinned download's. This is
  the record of what *integrate* read; files that only `generate` reads (PDF
  systematics members and sets) are recorded by `generate` in what it writes,
  §3a (user, C1).
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
   re-integrate, or point LHAPDF_DATA_PATH at the original files.`
   A mismatch is refused, never searched past (C9, user). `generate` never
   reads `pdfsets.index`: the set name is recorded.

## 3a. What `generate` records of the PDF files it reads (C1)

Every PDF file `generate` reads is hashed as it is parsed, by the same
read-once loader as at integrate, and listed in the event file's header in a
block of its own, beside `<MGRunCard>`:

```xml
<vibegraphInputs>
  <pdf use="central" set="NNPDF23_lo_as_0130_qed" lhaid="247000" member="0" tier="cvmfs">
    <file name="NNPDF23_lo_as_0130_qed.info" sha256="f961a1…"/>
    <file name="NNPDF23_lo_as_0130_qed_0000.dat" sha256="5ac949…"/>
  </pdf>
  <pdf use="systematics" set="…" lhaid="…" member="…" tier="cache" archive_sha256="…">
    <file name="…" sha256="…"/>
  </pdf>
  <grid sha256="…"/>  <!-- the integrate artifact file as read -->
</vibegraphInputs>
```

- The **central** entry repeats the digests `generate` has just checked
  against the artifact, so the event file alone says which densities its
  weights came from.
- A **systematics** member or set is introduced at generate (`sys_pdf`,
  `use_syst`, generate settings in the split concept), so it has nothing to be
  checked against. It is recorded, not compared. When it sits in the same set
  directory as the verified central member it carries the same `tier`; when
  `generate` had to fetch it, the entry carries the archive's SHA-256 from the
  cache sidecar (`archive_sha256`), and for a pinned set that archive was
  verified against the pin before anything was extracted, so its members are
  verified transitively. A consent refusal for a systematics set refuses the
  run before any event is written: a sample silently missing a requested
  variation is worse than none.
- `tier` names the tier (`lhapdf_data_path`, `lhapath`, `cvmfs`, `cache`,
  `download`), never a path: the header is copied between machines, and a home
  directory is not provenance.
- No separate generate record file: the event file is what `generate` writes,
  and a sidecar would be a second copy to keep consistent (open as N11 in the
  split concept). No systematics are evaluated today, so until they are the
  block holds the central entry and the grid digest only; the schema above
  fixes where the rest goes.

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
- **Built-in `pdlabel`s are refused wherever a density is read** (C4, user):
  proton beams on a scattering process. The message:
  `run card sets pdlabel = {label}, MadGraph's built-in parameterisation; vibegraph
  reads parton densities only from LHAPDF6 grids. Set pdlabel = lhapdf and lhaid
  to the set's LHAPDF id (247000 is NNPDF23_lo_as_0130_qed, the set nn23lo1 is
  fitted to).` The `247000` hint is printed only for `nn23lo1`; for other
  labels the message ends after "LHAPDF id". On fixed-energy beams and decays
  (`for_decay` sets `lpp = 0`) the label is inert, as `setrun.f` makes it, and
  is not refused; `RunCard::pdfsup` still writes `get_pdf_id(pdlabel)` there.
  With every proton label refused, `pdf_label_alpha_s` and its table are
  unreachable and go: on proton beams αs is always the set's own grid
  (`GridAlphaS`). The table below records why no label could have been mapped:

| pdlabel | MadGraph loads[^mg-pdfwrap] | `get_pdf_id`[^mg-pdfid] | index today[^index] | verdict |
|---|---|---|---|---|
| `nn23lo1` | `NNPDF23_lo_as_0130_qed_mem0.grid` | 247000 | 247000 `NNPDF23_lo_as_0130_qed` | refuse: same fit, not the same numbers; MadGraph's internal grid and LHAPDF's moved a `b b̄` σ by 9.8% ([pinned PDF set](../scales-pdf/pinned-pdf-set.md)) |
| `nn23lo` | `NNPDF23_lo_as_0119_qed_mem0.grid` | 246800 | 246800 `NNPDF23_lo_as_0119_qed` | refuse: names agree, no gate has compared the densities |
| `nn23nlo` | `NNPDF23nlo_as_0119_qed_mem0.grid` (its comment says `nlo_as_0130_qed_mc`) | 244800 | 244800 `NNPDF23_nlo_as_0119_qed` | refuse: MadGraph's own comment and file disagree |
| `cteq6l1` | `cteq6l1.tbl` through `Ctq6Pdf` | 10042 | 10042 `cteq6l1` | refuse: CTEQ's own table and interpolation, not an lhagrid1 file |
| `cteq6_m` | `cteq6m.tbl` | 10000 | 10000 is **`CoSMAO22Proton`** | refuse: the id has been reassigned |
| `cteq6_l` | `cteq6l.tbl` | 10041 | absent | refuse |
| `cteq6_d` | `cteq6d.tbl` | none | – | refuse |
| `none`, `iww`, `eva`, `edff`, `chff` | lepton or no PDF | 0 | – | refuse on proton beams |

- **The default card changes** so a proton run with no run card is not
  refused: `PARAM_DEFAULTS` gets `pdlabel = pdlabel1 = pdlabel2 = lhapdf` and
  `lhaid = 247000` (MadGraph's template: `nn23lo1`, `230000`). A card that
  omits the lines gets the same. This is a documented departure from
  MadGraph's default card, written in three places: the `PARAM_DEFAULTS` rows'
  comment in `runcard.rs`, the run-card section of `docs/src/cli/overview.md`
  (the "With no run card at all" paragraph, line 55), and the defaults list of
  [run-card parser and defaults](../run-card/run-card-parser-and-defaults.md).
  `PDFSUP` is unchanged by it: `get_pdf_id` gives 247000 for both spellings.
  On proton beams the αs source moves from the `nn23lo1` table (0.130, two-loop
  β-function) to the set's own `AlphaS_*` grid, which is what MadGraph does
  under `pdlabel = lhapdf` and so what every banked proton reference used.

## 5. Lookup and consent

Tiers, for both a set directory and `pdfsets.index`, first hit wins:

| tier | set | index |
|---|---|---|
| 1. `$LHAPDF_DATA_PATH`, split on `:`; if it is unset, `$LHAPATH` (C7)[^lhapdf-paths][^mg-datadir] | each `<entry>/<set>/` | each `<entry>/pdfsets.index` |
| 2. CVMFS `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current` | `<root>/<set>/`; `@DEPRECATED`, `@UNVALIDATED` and the `.tar.gz` twins are not sets | `<root>/pdfsets.index` (layout measured on a CERN host, sprint log 2026-10-20) |
| 3. cache `<root>/pdf/` | `<set>/` with sidecar | `pdfsets.index`, fetched copy |
| 4. HTTPS, consent | `lhapdfsets.web.cern.ch/current/<set>.tar.gz` | `…/current/pdfsets.index` (47.6 kB) |

- `--pdf-set`, `--pdf-dir` and `$VIBEGRAPH_PDF_DIR` are retired, and so is
  the cwd-relative `validation/pdf` dev fallback (C7, C8, user). Tiers 1–2 are
  used as found, no checksum.
- **The variable is parsed as LHAPDF 6.5.3 parses it** (`Paths.cc`,
  `paths()`): `LHAPDF_DATA_PATH` if it is set (even to the empty string),
  otherwise `LHAPATH`, split on `:`. vibegraph honours `LHAPATH` for the same
  reason it reads `LHAPDF_DATA_PATH` at all: a user whose LHAPDF installation
  works resolves the same sets here as their MadGraph does. Two LHAPDF details
  are not copied: the install-prefix entry LHAPDF appends (vibegraph has no
  LHAPDF install; CVMFS, the cache and the download are its later tiers), and
  the trailing `::` that blocks it (empty entries are skipped). A set
  `$VIBEGRAPH_PDF_DIR` draws one warning naming `LHAPDF_DATA_PATH` (N12), so an
  old environment does not silently fall through to a download.
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
  function over a `Tiers { lhapdf_data_path: Vec<PathBuf>,
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
- Tests and scripts that passed `--pdf-dir` set `LHAPDF_DATA_PATH` and
  remove `LHAPATH` and `VIBEGRAPH_PDF_DIR`. A host with a real `/cvmfs` is still
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
- *built-in pdlabel*: every label of §4, `nn23lo1` included, refused on proton
  beams with the `pdlabel = lhapdf` message (the `247000` hint only for
  `nn23lo1`); the same labels accepted on fixed-energy beams and decays, with
  `PDFSUP` unchanged.
- *environment*: `LHAPDF_DATA_PATH` as a `:` list with an empty entry;
  `LHAPATH` used only when `LHAPDF_DATA_PATH` is unset, and not when it is set
  to the empty string; `$VIBEGRAPH_PDF_DIR` alone does not resolve a set and
  draws the warning; from a checkout holding `validation/pdf/` and no variable
  set, `--no-network` refuses (no dev fallback).
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
- *the default card*: CLI, no `--run-card` on proton beams resolves to
  `pdlabel = lhapdf`, loads `NNPDF23_lo_as_0130_qed` member 0, records lhaid
  247000, takes αs from the set's grid, and writes `lhapdf = pdlabel` and
  `247000 = lhaid` in `<MGRunCard>`.
- *what generate records*: the `<vibegraphInputs>` block lists the central
  member's two files with the artifact's digests, the tier it was found in,
  and the grid file's SHA-256.

Informational, while the PDF PR is open: the survey's `p p > e+ e-` runs gave
one σ, 690.095053 pb, for the default card and for `pdlabel = lhapdf` at lhaid
325300 and 230000; afterwards the last two must differ from the first. Every
proton card a banked σ or sample gate reads is `lhapdf`/247000, so every
banked σ gate is expected bit-for-bit unchanged; the PR confirms that rather
than assumes it. Two banked proton runs are `nn23lo1`/230000, `pp_to_ll` and
`pp_to_ll_qcd0`, and the αs gates (`validate_alphas`, `validate_scales`) read
them; they are reported as awaiting a re-card to `lhapdf`/247000 and a rebank
(user, 2026-10-23) rather than compared.

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
  `cli_pdf_cache_fetched`. `env_remove("VIBEGRAPH_PDF_DIR")` becomes
  `env_remove("LHAPDF_DATA_PATH")` and `env_remove("LHAPATH")` in
  `cli_first_run.rs:47`, `cli_hard_errors.rs:16`, `cli_pdf_cache.rs:47,132`,
  `cli_pdf_cache_fetched.rs:96`.
- **`VIBEGRAPH_PDF_DIR` sites** (C7): `vibegraph-lib/src/cache/mod.rs:64,90`
  (`AssetKind::Pdf.env_var`), `cache/resolve.rs:43,101`,
  `vibegraph-cli/src/assets.rs:76`, the `integrate.rs:352` and `generate.rs:150`
  help text, `scripts/acceptance.sh:183`, `scripts/demo_pp.tape:29` (becomes
  `export LHAPDF_DATA_PATH=…`), `docs/src/cli/overview.md:96`,
  `docs/src/cli/reference.md:89,313`, `tooling/asset-resolution.md`,
  `tooling/release-acceptance.md:52`. CI (`.github/workflows/ci.yml`) does not
  set it; it caches `validation/pdf`, which tests now reach through
  `LHAPDF_DATA_PATH`. The MadGraph-side scripts already export
  `LHAPDF_DATA_PATH=validation/pdf:…` (`validation/madgraph/mg5_pinned.sh:55`
  and the `gen_*.sh` scripts) and need no change.
- **`nn23lo1` sites**: the migration list is in the split concept, §4.
- **Scripts**: `--pdf-dir` in `validation/pythia/generate_samples.sh`,
  `validation/pythia/generate_mlm_samples.sh`,
  `validation/madgraph/gen_reweight_oracle.py`,
  `validation/madgraph/bench_reweight.py`; `scripts/demo_pp.tape`.
  `scripts/acceptance.sh` passes no PDF flag: its `unset VIBEGRAPH_PDF_DIR`
  (line 183) becomes `unset LHAPDF_DATA_PATH LHAPATH`, and with C5 its
  `lhapdf`/247000 card still downloads only the set. Without C5, `validation/pdf/fetch.sh` must also
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
[^lhapdf-paths]: LHAPDF 6.5.3 `src/Paths.cc`, `paths()`, lines 20–28: `LHAPDF_DATA_PATH`, falling back to `LHAPATH` when unset, split on `:`, then the install prefix unless the value ends in `::` (https://gitlab.com/hepcedar/lhapdf/-/blob/lhapdf-6.5.3/src/Paths.cc).
