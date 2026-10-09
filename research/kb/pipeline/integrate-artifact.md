---
type: Design
title: The integrate artifact and the compiled-program cache
description: "What vibegraph integrate persists (per-channel grids, model identity, run card, map choices), how generate checks and replays it, and the compiled-program cache designed but not built."
status: draft
tags: [artifact, integrate, generate, model-identity, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: artifact-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/artifact.rs#L150-L346", title: "artifact.rs: ChannelKey, ChannelSampler, ChannelGrid, IntegrateArtifact"}
  - {id: integrate-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-cli/src/integrate.rs#L585-L710", title: "integrate.rs: building and writing the artifact"}
  - {id: generate-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-cli/src/generate.rs#L257-L433", title: "generate.rs: card_mismatches, pdf_mismatches and version guards"}
  - {id: generate-keys, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-cli/src/generate.rs#L1460-L1500", title: "generate.rs: check_channel_keys"}
  - {id: n18-h8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L871-L911", title: "Note 18 §5 H8: the first integrate CLI and artifact layout"}
  - {id: n23-identity, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L713-L804", title: "Note 23: model identity in the artifact"}
  - {id: n23-cache, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L805-L855", title: "Note 23: the compiled-program cache slot"}
  - {id: n24-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L1548-L1648", title: "Note 24: what P3 must know; channel keys"}
  - {id: n37-maps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L297-L324", title: "Note 37 §5: map choices banked in the artifact"}
  - {id: n37-defects, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/37-madevent-map-survey-and-soft-angle.md#L542-L551", title: "Note 37 §6.5: version-guard defects"}
---
`vibegraph integrate` writes one file, `<out>/grid.bin.zst` (`--out` defaults
to the current directory). It holds the trained VEGAS grids plus every input
that produced them, so that `vibegraph generate` can refuse a mismatched input
instead of silently sampling against the wrong grid. The type is
`IntegrateArtifact` in `vibegraph-lib/src/artifact.rs`, at `FORMAT_VERSION = 11`.
Its encoding and version rules are in
[artifact format versioning](artifact-format-versioning.md). Replay and event
writing are in [generate](../events/generate.md).

## Writing

- **No silent overwrite.** An existing file is refused unless `--force` is
  passed. The CLI checks this *before* integrating, so the refusal is instant.
- **No artifact from a run that measured nothing.** A run stopped before any
  iteration was kept writes nothing and says so.
- **Bit-exact floats.** bincode stores `f64` as raw bytes, so a grid
  round-trips bit for bit (pinned in `artifact::tests`). JSON is avoided:
  `serde_json`'s default float parser is not correctly rounded, and moved
  grid values by one ulp until its `float_roundtrip` feature was enabled.[^n18-h8]

## Contents

| Field | Meaning |
|---|---|
| `format_version` | The oldest version whose schema holds every channel key this file banks (`version_for`): 9, 10 or 11. |
| `process` | The canonical process string, every process line with its number. |
| `model` | `ModelIdentity { name, restrict, digest }`: the `import model` label and a SHA-256 of the restricted model. |
| `pdf_set`, `pdf_member` | The PDF set name (`"none"` on fixed-energy runs) and member (always 0). |
| `mu_f` | The fixed factorisation scale when both beams share one constant, else `0`. |
| `sqrt_s_had` | The collision energy. |
| `neval` | Evaluations per iteration as requested; each channel's actual share is in its own `neval`. |
| `niter` | Iterations the run actually made (a convergence stop can end early). |
| `seed`, `run_card` | The seed, and the whole resolved run card. |
| `channels` | One `ChannelGrid` per phase-space channel, in the combiner's order. |
| `sigma_pb`, `sigma_err_pb`, `chi2_per_dof` | `Σⱼ σⱼ`, the channel errors in quadrature, and χ²/dof. **On a 1→n decay run this field holds the partial width in GeV**, not a cross section in pb. |
| `maps` | `MapChoices` (split angle, τ map, rung order): every map choice the run settled. |

Each `ChannelGrid` holds:

- `key`, the channel space the grid belongs to;
- `alpha`, the selection weight `αⱼ` its term carries;
- `neval`, the grid itself, and the term's `sigma_pb`, `sigma_err_pb` and `chi2_per_dof`;
- `sampler: Option<ChannelSampler>`, a summary of what the rule-based
  composition chose. That is the map topology (`Timelike` or `Spine`), the
  resonance poles, the spacelike lines, and the spine's rung poles after the
  regulating floor. `None` means the writer predates the field, not "no
  composition".

**Why the key exists.** A grid's coordinate count does not identify its channel
space. A hadronic channel prepends `(τ, y)` to its own `3n − 4` coordinates, so
it can match a fixed-energy grid at another multiplicity. A bare index also
cannot say whether channels came from one subprocess or from several flavour
groups pooled together.[^n24-p3] The `ChannelKey` variants are:

- `Whole`, one grid over an undecomposed map;
- `Channel { channel }`, a fixed-beam multichannel;
- `GroupChannel { group, channel }`, a hadronic channel pooled across flavour groups;
- `MultiplicityChannel { final_state, group, channel }`, one term of a sum over
  final-state multiplicities;
- `MergedChannel { final_state, group, channel, pairs }`, one map serving several
  `(group, diagram)` pairs whose maps are the same function.

Channels count MadGraph's integration configurations, not diagrams
([channel set](../phase-space/channel-set.md)).

**Why `αⱼ` is banked.** `αⱼ` enters every channel's weight, so replay must
re-install the banked values. Re-running the α-adaptation reproduces them only
by accident.[^n24-p3]

**Why the maps are banked.** `generate` rebuilds its channels and its `τ` draw
from `maps`, whatever today's `auto` rule would choose
([map choices](../phase-space/map-choices.md)). An artifact older than the
field reads back as `MapChoices::LEGACY` (isotropic, log, derived), the maps
every such run used.[^n37-maps]

## What `generate` checks before replaying

1. **The version** (rules in [artifact format versioning](artifact-format-versioning.md)).
   Three guards follow it. A version below 7 on a clustering-scale card is
   refused, because its `sigma_pb` used another scale rule. A version below 10
   on a card summing several multiplicities is refused. A version below 11 is
   refused on a process whose channels merge (`refuse_unmerged_grids`).
2. **The model, process and run card** (`card_mismatches`), all listed together
   in one refusal.
   - The model label is compared first. If the labels agree, the digest is
     compared.
   - The process string must match exactly.
   - Every run-card parameter is compared, including ones no physics reads.
     Floats are compared for equality, because both sides come from the same
     parser. The card is a fingerprint of the run, and deciding case by case
     which differences are harmless is how a mismatched grid gets sampled anyway.
3. **The PDF set and member** (`pdf_mismatches`). The run card pins an LHAPDF
   id, but the set actually loaded is named by a flag, so the two can disagree.
4. **The channels** (`check_channel_keys`). They are compared position by
   position: the count, each key, and each grid's dimension. The same keys in
   a different order would install every grid on the wrong channel, and that
   samples a plausible wrong distribution. Comparing counts or key sets would
   not see it.[^generate-keys]

`generate` writes `XSECUP` from the artifact's `sigma_pb`, and also reports the
sample's own σ beside it.

## Model identity

The digest is SHA-256 (`sha2`) over the bincode encoding of the
**restricted** `ParsedModel`, not over the UFO source files. A reworded comment
in the Python must not refuse an artifact. A restrict card regenerated with
different contents under an unchanged name must refuse it, and only the digest
can see that case.[^n23-identity]

The digest is reproducible because every map reachable from `ParsedModel` is a
`BTreeMap` or `BTreeSet`. With `HashMap`s it varied per process, and `generate`
would have refused artifacts at random. Known-answer tests pin `digest_bytes` to
`shasum -a 256`, since a digest that changed silently would refuse every
existing artifact. When both comparisons were removed in a mutation check, a
wrong-model run was accepted and gave σ = 2061.99 pb against the banked
2022.62 pb (+1.95%). Details: [model identity](../model/model-identity-digest.md).

## Not self-contained

A clean worker needs more than the artifact: the binary, both cards, the PDF
set (unweighting reads densities and α_s per trial), and for a non-SM model the
UFO directory. It also recompiles the amplitude program. Closing that is
[generate-artifact-not-self-contained](../backlog/feature/generate-artifact-not-self-contained.md).

## The compiled-program cache (designed, not built)

`generate` recompiles the helicity program from `(model, process)`, which is
the whole input to compilation. A cache would key on `(model.digest, process,
compiler schema version)`. The first two are already in the artifact. The third
belongs to the cache entry, because only the entry knows which compiler wrote
it. So **no artifact field is reserved**, and adding the cache needs no artifact
version bump. The derivation is in the doc comment on `IntegrateArtifact`.[^n23-cache]

Building it was measured as not worth it yet. On 2026-07-28, compilation took
0.05–0.29 s for every process the CLI could then run, against about 13 s for a
20k-event `generate` (host not recorded). Larger processes may change that,
which [diagram-enumeration-cost-unmeasured](../backlog/feature/diagram-enumeration-cost-unmeasured.md)
tracks. Three obstacles come first, and none is a `derive`:

- **`helas::eval` has no serde.** `Op`, `Node<T>`, `Sym`, `Const`, `Folded` and
  the layout and constant-pool specs would all need it, under a schema version
  of their own. The arena encoding changes often for performance, so it cannot
  share the artifact's version.
- **The compiled program is not one object.** `folded_hel` is an `OnceLock`
  built lazily on the first `eval_m2`, and it is the large part. Whether it
  travels or is rebuilt on load decides what the cache is for.
- **A pruned evaluator carries a kinematic contract.** `prune_zero_helicities`
  is correct only for partonic-CM momenta with beams along ±z. Under other
  kinematics it silently drops helicities. The pruned flag must not travel
  unless the contract travels with it and is rechecked on load.

A future `vibegraph enumerate` diagram artifact
([diagram-inspection-command-missing](../backlog/feature/diagram-inspection-command-missing.md))
would supply the cache's input. It gets its own concept beside this one.

[^n18-h8]: Note 18 §5, H8 (`cli-integrate`) and H5's `float_roundtrip` warning.
[^n23-identity]: Note 23, "model identity in the artifact", with its test and mutation tables.
[^n23-cache]: Note 23, "The compiled-program cache slot"; the same text is the doc comment on `IntegrateArtifact`.
[^n24-p3]: Note 24, P2d "What P3 must know" items 2–3, and P3's schema section.
[^n37-maps]: Note 37 §5.
[^generate-keys]: `vibegraph-cli/src/generate.rs`, `check_channel_keys`.
