---
type: Design
title: The integrate artifact and the compiled-program cache
description: "What vibegraph integrate persists (per-channel grids and their members, configuration shares, model identity, run card, map choices), how generate checks and replays it, and the compiled-program cache designed but not built."
status: draft
tags: [artifact, integrate, generate, model-identity, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: artifact-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/artifact.rs#L150-L346", title: "artifact.rs: ChannelKey, ChannelSampler, ChannelGrid, IntegrateArtifact"}
  - {id: integrate-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/integrate.rs#L585-L710", title: "integrate.rs: building and writing the artifact"}
  - {id: generate-keys, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/generate.rs#L1460-L1500", title: "generate.rs: check_channel_keys"}
  - {id: n18-h8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L871-L911", title: "Note 18 §5 H8: the first integrate CLI and artifact layout"}
  - {id: n23-identity, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L713-L804", title: "Note 23: model identity in the artifact"}
  - {id: n23-cache, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L805-L855", title: "Note 23: the compiled-program cache slot"}
  - {id: n24-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1548-L1648", title: "Note 24: what P3 must know; channel keys"}
  - {id: n37-maps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L297-L324", title: "Note 37 §5: map choices banked in the artifact"}
  - {id: n37-defects, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L542-L551", title: "Note 37 §6.5: version-guard defects"}
---
`vibegraph integrate` writes one file, `<out>/grid.bin.zst` (`--out` defaults
to the current directory). It holds the trained VEGAS grids plus every input
that produced them: the cards, the restricted model itself, the diagram sets
and the matrix element at fixed points. So `vibegraph generate` needs no card,
no UFO directory and no enumeration of its own, and can refuse a build or an
asset that changed instead of silently sampling against the wrong grid. The
type is `IntegrateArtifact` in `vibegraph-lib/src/artifact.rs`, at
`FORMAT_VERSION = 23`, which is also the oldest version read.

`integrate`'s positional input is a proc card or a diagram artifact
(`diagrams.bin.zst`, written by `vibegraph enumerate`), told apart by content:
a file starting with the zstd magic `28 B5 2F FD` is an artifact, anything
else, and stdin (`-`), a proc card. From an artifact nothing is enumerated:
the sets, the proc card and the restricted model are the artifact's, the card
must re-parse to the process the artifact records, and `--ufo-dir` is refused
because no UFO directory is read. `--param-card` binds as from a proc card,
under the restriction the embedded model fixes. Either way the card is
enumerated at most once: the multiplicity and flavour grouping takes the sets
already enumerated (`multiplicity_groups_of`). Building the default run card
(no `--run-card`, a scattering card) reads MadGraph's process facts
(`config::process_facts`): from a diagram artifact the facts it banks, so
nothing is enumerated; from a proc card the card's own enumeration, split by
process line (`diagrams::enumerate_lines_in`), plus each decay-chain line's
core enumerated again without its decays, because MadGraph reads a decay
chain's legs off the undecayed core. `GlobalConfig::load_run_card` asks for
the facts only when it builds that card, so a `--run-card` run or a decay
computes none.
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
| `format_version` | `FORMAT_VERSION` (23); every artifact is written at the current version. |
| `process` | The canonical process string, every process line with its number. |
| `proc_card` | The proc card's text, verbatim, which `generate` re-parses. From a diagram artifact, the card that artifact records. |
| `model` | `ModelIdentity { name, restrict, digest }`: the `import model` label and a SHA-256 of the restricted model. |
| `inputs` | `RunInputs`: the PDF set and member with the SHA-256 of every PDF file read, the parameters the param card moved off the restricted defaults, and its decay tables. `generate` binds from these (`bind_recorded`). |
| `pdf_set`, `pdf_member` | The PDF set name (`"none"` on fixed-energy runs) and member (always 0). |
| `mu_f` | The fixed factorisation scale when both beams share one constant, else `0`. |
| `sqrt_s_had` | The collision energy. |
| `neval` | Evaluations per iteration as requested; each channel's actual share is in its own `neval`. |
| `niter` | Iterations the run actually made (a convergence stop can end early). |
| `seed`, `run_card` | The integration seed actually used, and the resolved run card narrowed to its integrate fields (`RunCard::split`). |
| `generate` | `GenerateSettings`: the run card's generate fields (`nevents`, `event_norm`, the systematics request, `clusinfo`), recorded for `generate` to replay. |
| `channels` | One `ChannelGrid` per phase-space channel, in the combiner's order. |
| `sigma_pb`, `sigma_err_pb`, `chi2_per_dof` | `Σⱼ σⱼ`, the channel errors in quadrature, and χ²/dof. **On a 1→n decay run this field holds the partial width in GeV**, not a cross section in pb. |
| `maps` | `MapChoices` (split angle, τ map, rung order): every map choice the run settled. |
| `channel_map_digest` | SHA-256 over every channel's `DiagramChannel::map_key`, in channel order, each key newline-terminated (`artifact::channel_map_digest`). Empty key list on a flat-RAMBO run. Since version 15. |
| `embedded` | `EmbeddedModel { parsed, naming }`: the restricted `ParsedModel` and its particle naming, which `generate` rebuilds the model from (`IntegrateArtifact::loaded_model`). `inputs.params` is relative to its defaults. Since version 22. |
| `param_card` | The param card's text, verbatim, or `None` for a run that read none. For the record: binding replays from `inputs`. Since version 22. |
| `sets` | Every `DiagramSet` in enumeration order, which `generate` reads instead of enumerating. Since version 22. |
| `diagram_set_digest` | SHA-256 over the bincode of `sets` (`diagrams::artifact::diagram_set_digest`), the function the diagram artifact uses, so an integration started from an enumerate run records that run's digest and one from a proc card records what `enumerate` would. Since version 22. |
| `amplitude_probe` | Per set, in set order, four phase-space points (momenta banked) and the colour- and helicity-summed \|M\|² at each under the bound param card (`amplitude_probe`, below). Since version 22. |
| `configurations` | `ConfigShare` per integration configuration of every integrated evaluator: its share of σ (below). Since version 23. |

Each `ChannelGrid` holds:

- `key`, the channel space the grid belongs to;
- `alpha`, the selection weight `αⱼ` its term carries;
- `neval`, the grid itself, and the term's `sigma_pb`, `sigma_err_pb` and `chi2_per_dof`;
- `sampler: Option<ChannelSampler>`, a summary of what the rule-based
  composition chose. That is the map topology (`Timelike` or `Spine`), the
  resonance poles, the spacelike lines, and the spine's rung poles after the
  regulating floor. `None` means the writer predates the field, not "no
  composition";
- `members: Vec<ChannelMember>` (since version 23), every `(diagram set,
  configuration)` the channel's map serves: per pair of a merged channel and
  per member set of the pair's flavour group, the set as a `SetRef { set,
  final_order }` (outgoing leg `i` of the integrated term is outgoing leg
  `final_order[i]` of `sets[set]`; reordering keeps diagram indices), the
  set's own diagrams that make up the configuration, and `share`, the index
  of the `configurations` entry the configuration is counted in. A member's
  diagrams are matched to the representative's configuration by
  `config_groups` partition, else by `IdentifyConfigTag`; empty where neither
  pairs them. A flat-RAMBO run's one channel has none.

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

## Configuration shares

`configurations` holds, per integrated evaluator (each fixed-energy
subprocess; each hadronic flavour group, part by part), one `ConfigShare` per
integration configuration in `AMP2` order and one for weight no configuration
carried (`configuration: None`). Each records the evaluator's `set` (a flavour
group's representative), its `diagrams`, the set's `contact_diagrams` (in no
configuration: contact diagrams and diagrams the colour basis never
references; they enter every share through \|M\|²), `members` (every set the
share includes: at proton beams every member of the group, so a share is the
group's, both beam orderings and all members' luminosity, not split by
member), `sigma` and `sigma_err` in the observable's unit.

**The partition.** At every integration point each evaluator's term is
divided over its configurations by `w_c / Σ w`, MadEvent's single-diagram
enhancement factor (`matrix_madevent_group_v4.inc:214-228`,
[MadEvent's single-diagram enhancement](../phase-space/madevent-single-diagram-enhancement.md)):

| run card | `w_c` |
|---|---|
| `sde_strategy = 1` (`tmin_for_channel = −1`, the only value accepted) | `AMP2_c`, read off the evaluation the point's \|M\|² just ran (`BoundAmplitude::filled_amp2`), at the coupling it ran at |
| `sde_strategy = 2` | `get_channel_cut`'s product (`ChannelSet::channel_cuts`) |

`AMP2` at the evaluation's coupling is what MadEvent forms beside `ANS` from
the same amplitudes; the per-event configuration *draw* uses the bound
coupling instead, to stay a function of the momenta. The two differ only for a
matrix element mixing orders in αs at a running scale. Where marked lines are
zeroed by an on-shell veto (`$`), the whole amplitude's `AMP2` is masked as the
draw masks it. A fixed-energy run of several subprocesses under
`sde_strategy = 2` records every subprocess past the first as unattributed:
the channel forests are derived for the first only. The split costs nothing
off the integration (`ShareSplit` is inactive until an integration asks).

**Accumulation.** The parts are integrated from the same weighted points as
σ, per channel (`ChannelIntegrand::value_in_channel_shared`,
`adapt_blocks_iteration`'s `n_shares`), combined over the kept iterations
with the weights the channel's own term uses (equal under the default
unweighted rule) and summed over channels with errors in quadrature, as σ is.
Warm-up iterations, whose shares would be dropped, accumulate none, and a
point of value zero adds nothing to any share; what this costs is in
[configuration-share cost](../performance/configuration-share-cost.md).
They are summed in tiles of 256 points of a block's iteration and then tile by
tile, which every chunk size respects (chunks are whole tiles), so the shares
are as independent of the thread count as the grids. **Sum rule:** the shares
of a run sum to `sigma_pb` to rounding; `integrate` warns otherwise.

**Checks.** `hadronic::tests::configuration_shares_split_each_point_by_amp2_and_sum_to_sigma`
(`g g → g g`: three configurations and the contact; each point's shares
against `AMP2` recomputed independently, the sum rule per channel and over
the run), `proton::tests::hadronic_shares_follow_each_groups_amp2_and_sum_to_sigma`,
`enhancement_weights_are_read_at_the_evaluated_coupling` (`u u~ → u u~ QED<=2`,
whose fractions move with αs), and the MadEvent oracle `cli_config_share`
([configuration-share oracle](../validation/configuration-share-oracle.md)).
The sum rule is blind to a share moved between two configurations; the oracle
is what sees that.

## What `generate` checks before replaying

`generate` takes no card, so nothing it is given can mismatch the record;
what it checks are the assets it reads again, the artifact's own integrity,
and the build it runs on ([events/generate](../events/generate.md) lists the
steps). `--ufo-dir` is refused: no UFO directory is read.

1. **The version.** Below 23 is refused by `UnsupportedVersion`, which says
   to regenerate with `vibegraph integrate` (rules in
   [artifact format versioning](artifact-format-versioning.md)).
2. **The embedded bytes** (on read, `IntegrateArtifact::read_hashed`). The
   embedded model must digest to `model.digest` and the sets to
   `diagram_set_digest`; a file that was edited or corrupted is refused by
   name. This replaces the check of a model re-read from a UFO directory,
   which no longer exists.
3. **The process.** The recorded `proc_card` is re-parsed and its process
   string must equal `process` (`vibegraph::generate::check_process`).
4. **The PDF set.** The recorded PDF set is located by name and member, and
   the files read are recorded in the event file's header.
5. **The channels** (`check_channel_keys`). They are compared position by
   position: the count, each key, and each grid's dimension. The same keys in
   a different order would install every grid on the wrong channel, and that
   samples a plausible wrong distribution. Comparing counts or key sets would
   not see it.[^generate-keys]
6. **The channel maps** (`check_channel_maps`, both the hadronic and the
   fixed-energy path). The digest is recomputed from the channels `generate`
   rebuilt and must equal `channel_map_digest`. Equal keys and grid
   dimensions say the channels are the same diagrams, not that each diagram
   got the same map: the no-cut spine change (version 15) kept every key and
   dimension while moving `2 → 2` poles and turning timelike trees into
   spines. A mismatch is refused naming both digests, with the remedy to
   re-integrate. **Blind spot:** `map_key` encodes a map's parameters
   (topology, poles, floors, transfer bounds, beam masses, `√s`), not the code
   that draws from them. A change to how a map with an unchanged key samples
   or weighs a point passes this check and still needs a version bump.
7. **The amplitude probe** (`check_amplitude_probe`, before any scan). Every
   set is compiled by the running build and \|M\|² recomputed at each banked
   point under the bound card; a relative deviation above 1e-10 is refused,
   naming the set, the point and the deviation, with the remedy to integrate
   again. It covers what no format version sees: a compiler fix that moves
   \|M\|², a change to parameter evaluation or binding, and replay on another
   binary, none of which changes a field of the artifact. It does not cover a
   change to enumeration or to the conversion of feyngraph's diagrams, whose
   output the artifact banks (blind spots below).

### The amplitude probe

`vibegraph-lib/src/amplitude_probe.rs`. Per set, four points in the partonic
centre of mass with the beams on ±z (a decay: the parent at rest at its mass).
A scattering set is probed at `√ŝ = max(3.3 × threshold, 1234.5 GeV)`, the
threshold being the larger of the incoming and outgoing mass sums, so every
point is well above threshold. The outgoing momenta are massive RAMBO from a
fixed-seed ChaCha8 stream, keeping only draws with `2 pᵢ·pⱼ ≥ 10⁻³ ŝ` for
every pair of external legs, so no point is soft, collinear or symmetric. The
momenta are banked with the values, so the check depends on neither the RNG
nor this construction staying the same. The tolerance is five orders above
libm last-ulp differences and far below any convention or coupling error.
**Blind spots:** a change to \|M\|² away from the four points, the running
of α_s with the event scale (the probe evaluates at the card's α_s), cuts and
maps. A one-body final state records no point; it is never integrated.
Tests: `amplitude_probe::tests` refuse a banked value moved by 1e-8, accept
one moved by 1e-13, and refuse a recomputation scaled by 1 + 1e-9.

`generate` writes `XSECUP` from the artifact's `sigma_pb`, and also reports the
sample's own σ beside it.

## Model identity

The digest is SHA-256 (`sha2`) over the bincode encoding of the
**restricted** `ParsedModel`, not over the UFO source files. A reworded comment
in the Python must not refuse an artifact. A restrict card regenerated with
different contents under an unchanged name must refuse it, and only the digest
can see that case.[^n23-identity]

The digest is reproducible because nothing reachable from `ParsedModel` is a
`HashMap` or `HashSet`: the name-keyed collections are `IndexMap`s, whose order
is semantic, and everything else is a `BTreeMap` or `BTreeSet`
(`ufo/identity.rs`). With `HashMap`s it varied per process, and `generate`
would have refused artifacts at random. Known-answer tests pin `digest_bytes` to
`shasum -a 256`, since a digest that changed silently would refuse every
existing artifact. When both comparisons were removed in a mutation check, a
wrong-model run was accepted and gave σ = 2061.99 pb against the banked
2022.62 pb (+1.95%). Details: [model identity](../model/model-identity-digest.md).

## Not self-contained

A clean worker needs the binary, the artifact and, at proton beams, the PDF
files (unweighting reads densities and α_s per trial). Nothing else: no card,
no UFO directory, no enumeration. It recompiles the amplitude program from
the banked diagrams. The lowered program is not banked: it is 40–80× the
folded AST, rebuilds in under 3 s on every card measured, and its encoding
moves with almost every performance change. The amplitude probe guards the
recompilation.

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
[^generate-keys]: `vibegraph-lib/src/generate.rs`, `check_channel_keys`.
