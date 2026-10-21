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
`PARAM_DEFAULTS` name has one. **Generate** is the 15 fields that reach only
the event budget or the written record. **Integrate** is every other field.
That includes the benign job and integrator bookkeeping (`run_tag`,
`gridpack`, `gseed`, `hel_*`, `survey_*`, …), which nothing reads and which
rides with the card that trained the grid. The refusal of an
`IgnoredPhysics` field off its default stays where the card is parsed, in
`integrate`, whatever its stage.

| field | class today | stage | note |
|---|---|---|---|
| `nevents` | Consumed | Generate | `--nevents` overrides it |
| `iseed` | IgnoredBenign | Generate | recorded but not read; `--seed` is the seed (N4) |
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
- `iseed`, recorded but unused (N4).
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

**Older files** (N7): none carries a proc card, so `generate` cannot replay
one. H1 raises `OLDEST_READABLE_VERSION` to 13 and deletes the `v3`, `v4`,
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
vibegraph generate <ARTIFACT> [--nevents N] [--seed S] [-o FILE] [--force]
    [--strategy …] [--scan-points …] [--max-truncation …] [--event-norm …]
    [--reweight-card F] [--reweight-exact] [--reweight-couplings …]
    [--ufo-dir D] [parallel options]
```

The global `--no-network`, `--yes` and the environment variables
(`VIBEGRAPH_PDF_DIR`, `LHAPDF_DATA_PATH`, `VIBEGRAPH_UFO_DIR`,
`VIBEGRAPH_HOME`, `VIBEGRAPH_NO_NETWORK`) are unchanged.

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

## 5. Open choices for the user, in one round

N-numbers are new. C-numbers keep the first draft's numbering. C3 and C11 are
gone: no file older than the floor is replayed or written.

| | choice | recommended | alternative |
|---|---|---|---|
| N1 | proc card in the artifact | verbatim text, re-parsed and checked against `process` | a serialized parsed card (needs serde on `SupportedCard`, and pins this build's parse) |
| N2 | generate settings | inside the artifact, one file | a sidecar file beside `grid.bin.zst`, editable without a tool |
| N3 | classification | §1's table, with the ambiguous rows as listed | move `clusinfo` or the bookkeeping rows |
| N4 | seed | `--seed` only, default constant; `iseed` recorded, unread | a non-zero `iseed` is the default seed |
| N5 | `event_norm` at generate | a `--event-norm` flag, defaulting to the recorded value | recorded value only; changing it means re-integrating |
| N6 | systematics | verbatim strings; nothing evaluates them yet | typed fields now |
| N7 | older artifacts | raise the floor at each step, delete the old readers and guards | keep the readers; `generate` refuses by named thresholds |
| N8 | `--ufo-dir` on `generate` | kept (asset location) | environment variable only |
| N9 | `<MGRunCard>` banner | integrate settings ∪ generate settings as used, `nevents` as requested, `iseed` = the generation seed when it fits an `i32` | integrate settings ∪ recorded generate settings, verbatim |
| N10 | library boundary | H1 moves the sampling driver from `vibegraph-cli/src/generate.rs` into `vibegraph-lib` behind the new signature | the boundary in the CLI crate first; move later |
| C1 | PDF checksum | per-file SHA-256 of `.info` + the member `.dat` read | whole-directory digest; archive digest |
| C2 | param record | moved externals, which are also what `generate` rebinds from | every external's value |
| C4 | built-in labels | `nn23lo1` only | also `nn23lo` on name evidence |
| C5 | pinned shortcut | pin carries `set_index`, `num_members` | every lhaid through an index |
| C6 | index across tiers | first found, fall through on "no known set" | first found only |
| C7 | `$VIBEGRAPH_PDF_DIR` | kept as tier 0; tests use it | retired; tests set `$LHAPDF_DATA_PATH` |
| C8 | `validation/pdf` dev fallback | removed | kept as a last local tier |
| C9 | checksum mismatch in `generate` | refuse at the first tier holding the set | try later tiers for a matching copy |
| C10 | unpinned prompt | states size and SHA-256 unknown | HEAD request for the size first |
| C12 | download-tier test | library level with `MapFetch` only | also a CLI test through a base-URL override and a local server |

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
   - Takes no PDF lookup change: `generate` locates the recorded `pdf_set` by
     name through today's resolution.
   - Closes no current item. The manager files one (see Found in the D1b
     report).
3. **H2, the PDF from the run card.**
   - Covers: [run inputs](run-inputs-in-the-artifact.md) §1 PDF, §3–§5,
     `RunInputs` at version 14 with the floor raised, and `--pdf-*` removed
     from `integrate`.
   - Closes `pdf-set-not-taken-from-run-card`.
4. **The param-card PR.**
   - Covers: `integrate --param-card`, filling `inputs.params`, and the rebind
     in `generate`.
   - Closes `cli-has-no-param-card-option`, whose scope line about
     `generate --param-card` the manager amends to the 2026-10-21 decision.
     It can run in parallel with H2 once H1 has merged; whichever merges
     second rebases onto `RunInputs`.
