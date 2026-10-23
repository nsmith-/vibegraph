---
type: Design
title: Integrate and generate split by data dependency
description: "generate takes the artifact plus generate-time inputs only; the run card splits into integrate settings and generate settings, each run-card field classified; artifact version 13; the CLI and its migration; PR split."
status: draft
tags: [artifact, generate, integrate, run-card, cli, gridpack, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
# Integrate and generate split by data dependency

**Principle (user, 2026-10-21, `sprints/pdf-from-run-card/log.md`).**
Information a workflow step necessarily introduces is recorded in that step's
artifact and is never requested again by a later step: a later step that
re-took it could only compare it, so it does not take it.

So `generate` works like a gridpack. It takes the integrate artifact plus what
only generation introduces: the event count, the generation seed, output and
weight-strategy options, and the reweight card. The proc card, run card, param
card and PDF choice are integrate inputs. `integrate` records them, and
`generate` reads them back from the artifact. Model and PDF *files* are still
read at generate as assets, checked against the digests the artifact records.
`card_mismatches`, `refuse_on_mismatch` and their "the card is a fingerprint"
rationale (in [the integrate artifact](integrate-artifact.md)) go, replaced by
those digest refusals. What the artifact records of the PDF and the param
card, and the PDF lookup, are in
[run inputs in the artifact](run-inputs-in-the-artifact.md).

## 1. Classifying the run card

Each `FIELD_CLASSES` row (`runcard/classes.rs`) gains a second column,
`Stage::Integrate` or `Stage::Generate`, and a test asserts that every
`PARAM_DEFAULTS` name has one. **Generate** is the 14 fields that reach only
the event budget or the written record. **Integrate** is every other field.
That includes the benign job and integrator bookkeeping (`run_tag`,
`gridpack`, `gseed`, `hel_*`, `survey_*`, …), which nothing reads and which
rides with the card that trained the grid. The refusal of an
`IgnoredPhysics` field off its default stays where the card is parsed, in
`integrate`, whatever its stage.

| field | class today | stage | note |
|---|---|---|---|
| `nevents` | Consumed | Generate | `--nevents` overrides it |
| `iseed` | IgnoredBenign → Consumed | Integrate | the integration seed when `--seed` is absent and `iseed` is non-zero (N4, decided) |
| `event_norm` | IgnoredPhysics (Consumed after #66) | Generate | `XWGTUP` and `<init>` normalisation only |
| `use_syst` | Consumed | **both** | see below |
| `systematics_program`, `systematics_arguments` | IgnoredBenign | Generate | carried verbatim |
| `sys_scalefact`, `sys_alpsfact`, `sys_matchscale`, `sys_pdf`, `sys_scalecorrelation` | IgnoredBenign | Generate | carried verbatim |
| `clusinfo` | IgnoredBenign | Generate | adds a `<clustering>` block to an event; not written today |
| `time_of_flight`, `boost_event`, `lhe_version` | IgnoredPhysics | Generate | event writing only; still refused off default |
| everything else (beams, PDF, scales, cuts, matching, `SDE_strategy`, `me_frame`, `nhel`, `bias_*`, bookkeeping) | – | Integrate | – |

**`use_syst` is the one field read at both stages.** At integrate,
`runcard::matching::resolve` forces `alpsfact = 1` under it
(`setrun.f:151-159`). The integrate settings record the card after that
resolution, so they hold the resolved `alpsfact` and need `use_syst` no
further. `use_syst` itself goes into the generate settings, where it selects
the systematics block. A non-MadGraph front end that flips `use_syst` between
generations therefore does not move `alpsfact`. That is correct: the grid and
σ were formed at the recorded `alpsfact`.

**Ambiguous rows, for the user to confirm:**
- `use_syst`, split as above.
- `iseed`: Integrate, decided (N4).
- `clusinfo`: the record is generate-time, but it is an MLM input. Today it is
  benign because nothing writes the block.
- `bias_module` and `bias_parameters`: Integrate, because a bias reshapes the
  sampling the grid was trained on as well as the event weights. They are
  refused today anyway.
- The bookkeeping rows: Integrate by the rule above. Nothing compares them any
  more, so their stage only decides where they are stored.

## 2. Structures and version

`RunCard::split(self) -> (RunCard, GenerateSettings)` is pure and runs at the
end of the card's resolution in `integrate`. The `RunCard` it returns holds
the integrate fields only; `nevents` leaves the struct.

```rust
pub struct GenerateSettings {          // vibegraph-lib, MadGraph-neutral
    pub nevents: u64,
    pub event_norm: EventNorm,         // Average until #66 admits Sum, Unity
    pub systematics: Systematics,      // use_syst + program/arguments/sys_* strings, verbatim
    pub clustering_info: bool,         // clusinfo
}
```

The library's generation entry point takes `(&IntegrateArtifact, assets,
&GenerateSettings, &GenerateOptions)`. `GenerateOptions` holds the seed, the
strategy, the scan budget and the truncation. The entry point never takes a
`RunCard`. It reads the integrate settings from the artifact, the same way the
integrand reads them at integrate. A front end that refines a grid in passes
can then call it between passes with new settings, without re-integrating.

**Integrate artifact version 13** (H1):

- adds `proc_card: String`, the proc card's text verbatim (N1);
- narrows `run_card` to the integrate settings;
- adds `generate: GenerateSettings`, the generate part of the card, recorded
  for replay (N2).

The MadGraph-style `generate` replays `generate` with its flags applied. The
library does not read it.

- **Version 14** (H2) replaces `pdf_set`/`pdf_member` with `inputs:
  RunInputs`. That struct holds `params` as well; H2 writes `[]` and refuses
  a non-empty list.
- The param-card PR then fills `params` and lifts that refusal, with no shape
  change and so no version bump.

**Older files** (C3, settled: refused; N7 is how): none carries a proc card,
so `generate` cannot replay one. Every artifact is written at the current
version (C11, settled). H1 raises `OLDEST_READABLE_VERSION` to 13 and deletes the `v3`, `v4`,
`v5`, `v7` and `v9` readers. It also deletes everything that existed only for
them:

- the version-threshold guards: `SCALE_DRAW_VERSION`, `MULTIPLICITY_VERSION`,
  `MERGED_CHANNEL_VERSION`, `refuse_unmerged_grids` and the
  `refuse_stale_artifact_on_*` functions;
- the `version_for` write-down.

H2 raises the floor to 14, because a version-13 file records no PDF checksum.
The refusal is the existing `UnsupportedVersion`, which already says
"regenerate it with `vibegraph integrate`". [Artifact format
versioning](artifact-format-versioning.md) keeps its rules; this artifact
simply has no live instance of rules 6 and 7 after H1.

## 3. The command line afterwards

```
vibegraph integrate <PROC_CARD> [--run-card F] [--param-card F] [--out D] [--force]
    [--ufo-dir D] [--seed S] [budget, map, grid-rule and parallel options]
vibegraph generate <ARTIFACT> --seed S [--nevents N] [-o FILE] [--force]
    [--strategy …] [--scan-points …] [--max-truncation …] [--event-norm …]
    [--reweight-card F] [--reweight-exact] [--reweight-couplings …]
    [--ufo-dir D] [parallel options]
```

The global `--no-network` and `--yes` are unchanged. The environment
variables are `LHAPDF_DATA_PATH` (or, when it is unset, `LHAPATH`),
`VIBEGRAPH_UFO_DIR`, `VIBEGRAPH_HOME` and `VIBEGRAPH_NO_NETWORK`;
`VIBEGRAPH_PDF_DIR` is retired (C7, settled; sites in
[run inputs](run-inputs-in-the-artifact.md) §7).

- **Removed from `generate`:** the positional proc card, `--run-card`,
  `--pdf-set` and `--pdf-dir`. `--param-card` never exists there.
- **Removed from `integrate`:** `--pdf-set` and `--pdf-dir`.
- **`--ufo-dir` stays on `generate`** (N8). It locates an asset; it does not
  choose one.
- **`--event-norm` is N5.**

`generate`, in order:

1. Read the artifact. A version below the floor is refused.
2. Parse `proc_card` and check that `process_string` equals the recorded
   `process`. A difference means this build parses the card differently, and
   it is refused with "re-integrate".
3. Resolve the model by the card's `import` through the UFO search path, load
   it, and compare its digest with `model.digest`. The refusal reads:
   `model {label} at {dir} has digest {got}; the grid was integrated against
   {want}: its files changed after vibegraph integrate ran; re-integrate, or
   restore them`. The interned SM goes through the same check.
4. Bind the parameters and load the PDF ([run inputs](run-inputs-in-the-artifact.md)
   §3).
5. Run the channel-key check, unchanged.
6. Resolve the reweight card against the model at the recorded parameter
   point. The forbidden s-channels come from the recorded proc card.
7. Build the settings: the recorded `generate`, with `--nevents`, `--seed` and
   `--event-norm` applied.

The `<MGRunCard>` banner is written from the integrate settings plus the
generate settings as used (N9).

## 4. Migration

Every `generate` call drops the proc card and `--run-card`. Where a test set
`nevents` through the run card, it passes `--nevents`, or it relies on the
recorded value.

**`generate` invocations** (in `vibegraph-cli/tests/`, with the number of
calls):

- `cli_generate.rs` (4)
- `cli_generate_proton.rs` (6)
- `cli_decay_chain_events.rs` (2)
- `cli_first_run.rs` (1)
- `cli_decay.rs` (1)
- `cli_reweight_proton.rs` (1)
- `cli_ufo_model.rs` (1)
- `cli_hard_errors.rs` (1, at line 231; its run-card refusal now happens at
  integrate only, so the `generate` leg goes)
- `validate_samples_proton.rs` (1)
- `cli_reference_docs.rs` follows the regenerated help.

**Tests that go or are reframed:**

- `cli_generate.rs:578` `a_card_that_did_not_train_the_grid_is_refused`
  becomes "a proc card or `--run-card` passed to `generate` is an argument
  error".
- `cli_generate.rs:674` `a_pre_draw_artifact_is_refused_on_a_clustering_card`
  becomes "a version-12 artifact is refused".
- The `refuse` helper at `cli_generate.rs:194` goes.
- `cli_generate_proton.rs:639` `a_different_pdf_set_is_refused` becomes the
  checksum refusal.
- `cli_ufo_model.rs:125` `generate_refuses_another_restrict_variant_of_the_same_model`
  becomes "a restrict card edited after integrate is refused by digest".
- The `generate.rs` unit tests at lines 2185–2242 go with `card_mismatches`.
  The changed-restrict-card case becomes a digest test.
- PR #59's test that `generate` reads the run card first goes, since
  `generate` reads none.

**Scripts:**

- `scripts/acceptance.sh:234`
- `scripts/demo_ee.tape:44`
- `scripts/demo_pp.tape:40`
- `validation/pythia/generate_samples.sh:51`
- `validation/pythia/generate_mlm_samples.sh:50`
- `validation/madgraph/gen_reweight_oracle.py:208`
- `validation/madgraph/bench_reweight.py:73`

**Docs:** `docs/src/cli/overview.md:12` and `:62`, and
`docs/src/cli/reference.md` from line 239.

**Concepts:**

- `pipeline/integrate-artifact.md`: its "What generate checks" section and
  the version text, which is already stale at 11.
- `events/generate.md`
- `pipeline/artifact-format-versioning.md`: the floor, and the worked
  examples that become history.
- `tooling/release-acceptance.md`

### The `nn23lo1` migration (C4, settled)

Every built-in `pdlabel` is refused where a density is read, and the default
card becomes `pdlabel = lhapdf`, `lhaid = 247000`
([run inputs](run-inputs-in-the-artifact.md) §4). Sites, all in H2:

- **Code.**
  - `vibegraph-lib/src/runcard.rs:615-618`: the `pdlabel`, `pdlabel1`,
    `pdlabel2` defaults become `lhapdf`, and `lhaid` becomes 247000. The
    comment above them records the departure from MadGraph's template.
  - `runcard.rs:289`: the `pdfsup` doc names the new default. The
    `get_pdf_id` table stays, because fixed-energy cards keep spelling
    `nn23lo1`.
  - `vibegraph-lib/src/coupling/alphas.rs`: `pdf_label_alpha_s` (L279-293)
    and its doc at L241 go. Its tests change as follows:
    - L564 and L649 become refusal tests where the label is refused;
    - L570 and L580 stay (fixed-energy beams ignore the label).
  - `vibegraph-lib/src/runcard/matching.rs:264`: the fixed-energy default
    resolves to `lhapdf`. L258 stays (asymmetric labels are still refused
    first).
  - `vibegraph-lib/src/cache/pinned.rs:32`: doc comment.
- **Tests and the αs source.** In these files, proton cards written inline
  without a `pdlabel` line move from the `nn23lo1` αs table (0.130, two-loop)
  to the 247000 grid's αs:
  - `vibegraph-cli/tests/`: `cli_generate_proton`, `cli_integrate`,
    `cli_reweight_proton`, `cli_decay_chain_events`, `cli_hard_errors`,
    `validate_samples_proton`;
  - `vibegraph-lib/tests/`: `validate_hadronic`, `decay_chain_ladder`,
    `diagram_channel`, `validate_alphas`.

  The densities do not move, and Drell-Yan carries no αs. Any number pinned
  from a default-card proton QCD run in these files moves. H2 lists each
  moved pin with its old and new value rather than retuning it silently.
  This list was not run here.
  - `cli_decay.rs:475`: only a comment changes, since `PDFSUP` stays 247000.
- **Validation cards.**
  - `validation/madgraph/onshell_uu_run_card.dat` is the only committed
    proton `nn23lo1` card, but no run reads it on proton beams: every row
    that uses it (`uu_tt`, `uu_tt_mg371` and `uu_full` in
    `gen_onshell_veto.sh`, `cli_onshell_veto` and `bank_seeded_sigma.sh:416`)
    overrides `lpp1 = lpp2 = 0` at √s = 500, so the label is inert, nothing
    is refused, and neither the card nor those MadGraph rows need re-carding.
  - Two banked runs are proton `nn23lo1`/230000, and gated tests read them:
    `pp_to_ll` and `pp_to_ll_qcd0` (their own `Events/run_01`, not the
    `dy13_*` cards their σ and sample cells use). `validate_alphas` reproduces
    their `AQCDUP` and `validate_scales` replays their `AQCDUP` from the
    computed scale, both through the `nn23lo1` αs table that C4 removes.
    Decided (user, 2026-10-23): re-card both to `lhapdf`/247000 and rebank
    them. Until then both gates report them as awaiting that rebank
    (`AWAITING_REBANK_RUNS` in `vibegraph-lib/tests/common/mod.rs`); their
    scales are still replayed, since no density or αs enters that.
  - These cards are fixed-energy or decays, read no density, and are not
    refused, so they stay as they are: `onshell_ee_run_card.dat` (`lpp = 0`),
    and `onshell_t_bev_run_card.dat`, `decay_h4l_cuts_run_card.dat`,
    `decay_tbev_cuts_run_card.dat` (decays, `for_decay` sets `lpp = 0`).
  - The `research/upstream/madgraph-grouped-first-call-unpermuted/` cards are
    MadGraph-only repro material and stay as they are.
  - `validation/manifest.toml:629`: the `pp_to_jj` rationale ("departs from
    them in pdlabel and lhaid alone") now describes vibegraph's own default.
- **Docs.**
  - `docs/src/cli/overview.md:55`: the departure paragraph.
  - `docs/src/guide/10-hadronic.md:37`: the default set sentence.
  - `docs/src/guide/11-unweighting-events.md:145`: the PDFSUP example.
- **Concepts.**
  - `scales-pdf/pinned-pdf-set.md`: "What vibegraph does with a `nn23lo1`
    card".
  - `scales-pdf/alpha-s-sources.md`: the table.
  - `run-card/field-classification.md:148`.
  - `run-card/run-card-parser-and-defaults.md`: the defaults list.
  - `events/lhef-record-conventions.md:36`.

**No σ or sample comparison changes; two αs comparisons are held back.** The
log says the banked references used MadGraph's internal `nn23lo1` grid. For
the σ and sample rows they did not. Every banked proton σ and sample row was carded at `pdlabel = lhapdf`,
`lhaid = 247000` on the MadGraph side ([MadGraph oracle
pinning](../validation/madgraph-oracle-pinning.md); `manifest.toml:151`: the
four `nn23lo1` rows were re-carded at cut 5), and vibegraph reads those same
cards. So both sides were already LHAPDF 247000 with the grid's αs, and no
σ or sample comparison changes:

- the default-card change touches only cards that omit `pdlabel`, and no
  gated MadGraph card does;
- the only proton `nn23lo1` comparisons are the `AQCDUP` cells of
  `pp_to_ll` and `pp_to_ll_qcd0` above, held back until the rebank (the
  `uu_tt` row runs on fixed-energy beams);
- fixed-energy and decay rows are untouched, though a MadGraph-written
  fixed-energy card (`none` per beam, `lhaid = 230000`, no `pdlabel` line)
  must keep its `PDFSUP` at MadGraph's 247000 under the new default
  (`RunCard::pdfsup`).

## 5. Choices (decided 2026-10-21)

All twelve C-choices are settled (sprint log, 2026-10-21; summary in [run
inputs](run-inputs-in-the-artifact.md)). Open, with the recommendation taken
as the working assumption:

| | choice | recommended | alternative |
|---|---|---|---|
| N1 | proc card in the artifact | verbatim text, re-parsed and checked against `process` | a serialized parsed card |
| N2 | generate settings | inside the artifact | a sidecar file beside `grid.bin.zst` |
| N3 | classification | §1's table, the ambiguous rows as listed | move `clusinfo` or the bookkeeping rows |
| N4 | seed | **decided otherwise:** `iseed` feeds integrate; `generate` requires `--seed` | – |
| N5 | `event_norm` at generate | a `--event-norm` flag, default the recorded value | recorded value only |
| N6 | systematics | verbatim strings | typed fields now |
| N7 | older artifacts (C3's how) | raise the floor at each step, delete old readers and guards | keep readers; refuse by named thresholds |
| N8 | `--ufo-dir` on `generate` | kept | environment variable only |
| N9 | `<MGRunCard>` banner | integrate ∪ generate settings as used, `iseed` = generation seed when it fits an `i32` | recorded settings verbatim |
| N10 | library boundary | H1 moves the sampling driver into `vibegraph-lib` | boundary in the CLI crate first |
| N11 | where `generate` records generate-read PDF files (new, from C1) | the `<vibegraphInputs>` header block only | also a sidecar record file |
| N12 | a set `$VIBEGRAPH_PDF_DIR` (new, from C7) | one warning naming `LHAPDF_DATA_PATH` | ignored silently; or refused |

**Decided by the user, 2026-10-21.** N1, N2, N3, N5–N12 as recommended. N4
differs from the recommendation:
- **`integrate`:** the seed is `--seed` when given, else `iseed` when it is
  non-zero, else the fixed default. `iseed` is therefore an Integrate field and
  Consumed. The seed actually used is recorded in the artifact.
- **`generate`:** `--seed` is required and has no default. A missing
  `--seed` is refused, with a message saying each generation job needs its own
  seed. A default would let a batch of grid jobs silently draw identical
  samples.

N3's "bookkeeping rows" are the job and integrator fields that are
`IgnoredBenign` today and that nothing reads. They ride with the integrate
settings:
- job fields: `run_tag`, `gridpack`, `gseed`, `bypass_check`, `python_seed`,
  `keep_log`, `issgridfile`, and the compiler flags;
- integrator fields: `gridrun`, `job_strategy`, the `survey_*` and `refine_*`
  splitting, `hel_*`, and the vectorisation sizes.

## 6. PR split

The work no longer fits one PR. In order:

1. **#59** (cut refusal) and **#66** (`event_norm`) merge first, as they are.
   - #59's "generate reads the run card first" becomes moot in H1, which
     deletes generate's run-card load and that order test.
   - #66 makes `event_norm` Consumed in `generate.rs`. H1 moves that read
     from `rc` to `GenerateSettings::event_norm` and admits `Sum` and `Unity`
     there.
2. **H1, gridpack `generate`.**
   - Covers: the stage column, `RunCard::split`, `GenerateSettings`, artifact
     version 13 (`proc_card`, the narrowed `run_card`, `generate`), the floor
     raised to 13, and the new `generate` CLI and library entry point.
   - Also takes C7 and C8, because it rewrites every `generate` call anyway:
     `LHAPDF_DATA_PATH` (with `LHAPATH`) replaces `VIBEGRAPH_PDF_DIR` as the
     environment tier, the `validation/pdf` fallback goes, and tests set
     `LHAPDF_DATA_PATH` instead of passing `--pdf-dir`. `generate` locates the
     recorded `pdf_set` by name through that tier and today's cache. Moving
     the tests once avoids a second migration in H2.
   - Closes no current item. The manager files one (see Found in the D1b
     report).
3. **H2, the PDF from the run card.**
   - Covers: [run inputs](run-inputs-in-the-artifact.md) §1 PDF, §3, §3a's
     central entry and grid digest, §4 (every built-in label refused, the
     default card, the αs table removed, the `nn23lo1` migration above), §5's
     CVMFS, index and download tiers, `RunInputs` at version 14 with the floor
     raised, and `--pdf-*` removed from `integrate`.
   - §3a's systematics entries land with whatever PR first evaluates PDF
     systematics, not here.
   - Closes `pdf-set-not-taken-from-run-card`.
4. **The param-card PR.**
   - Covers: `integrate --param-card`, filling `inputs.params`, and the rebind
     in `generate`.
   - Closes `cli-has-no-param-card-option`, whose scope line about
     `generate --param-card` the manager amends to the 2026-10-21 decision.
     It can run in parallel with H2 once H1 has merged; whichever merges
     second rebases onto `RunInputs`.
