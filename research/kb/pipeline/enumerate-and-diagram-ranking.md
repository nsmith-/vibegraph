---
type: Design
title: "vibegraph enumerate, the diagram artifact, and the diagram-importance ranking"
description: "Design for vibegraph enumerate (diagram artifact, summary JSON, per-diagram SVG via an upstream feyngraph drawing API), integrate starting from a proc card or an enumerate run, a self-contained integrate artifact (model, param card, diagrams; no compiled program), and a ranking JSON pairing AMP2-share and density-share channel cross sections with the drawings."
status: draft
tags: [cli, enumerate, diagrams, feyngraph, svg, artifact, ranking, multichannel, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: fg-diagram, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/diagram/mod.rs#L85-L106", title: "feyngraph Diagram: every field pub(crate), no public constructor"}
  - {id: fg-draw, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/drawing/mod.rs#L239-L245", title: "feyngraph drawing: DiagramView::draw and the DiagramContainer SVG entry points"}
  - {id: mg-results, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/matrix_madevent_v4.inc#L174-L189", title: "MadEvent AMP2 multi-channel factor"}
measured:
  commit: 3cc7f9c
  host: "cloud container, 4 vCPUs"
  command: "scratch #[ignore] test in vibegraph-lib/src/helas/eval (not committed): per process, generate_from_proc_card, then AmplitudeEvaluator::compile, then folded_hel().program(), timed with Instant; cargo test -p vibegraph-lib --lib (dev profile, opt-level 2, debug assertions on)"
---
# `vibegraph enumerate`, the diagram artifact, and the diagram ranking

This design closes
[diagram-inspection-command-missing](../backlog/feature/diagram-inspection-command-missing.md)
and takes the model, param-card and diagram parts of
[generate-artifact-not-self-contained](../backlog/feature/generate-artifact-not-self-contained.md).
It adds three things:

1. `vibegraph enumerate`, which turns a proc card into a **diagram artifact**,
   a summary JSON and one SVG per diagram.
2. `vibegraph integrate`, which accepts that artifact in place of a proc card.
   Its own artifact then carries everything `generate` needs except the PDF
   files.
3. A **ranking JSON**, read from the integrate artifact, that orders
   configurations and channels by their share of σ. Each entry names diagram
   ids that match the SVG file names. The status page that renders it is
   designed separately.

## Decisions taken (human:nsmith-, 2026-11-09)

| Question | Decision |
|---|---|
| How drawings are produced | An **upstream feyngraph API** that draws a diagram vibegraph owns. Until it is released, a fork revision is pinned. |
| What the ranking ranks by | **Both, side by side.** The AMP2 share is the importance measure; the density share (σⱼ, αⱼ) is reported as sampler diagnostics. |
| How enumerate and integrate outputs are joined | **Integrate banks the membership**: each channel's full list of diagrams, and each configuration's share. |
| Diagram artifact | **In scope now.** Integrate starts from a proc card or from an enumerate run. |
| Param card | **Locked in by the integration.** It defines the grid, so it is banked with the result and `generate` never takes one. |

## 1. Drawing: why feyngraph needs an upstream change

feyngraph can draw only a `DiagramView`. Every field of its `Diagram` is
`pub(crate)` and there is no public constructor.[^fg-diagram] Its drawing module
is private, and its entry points are methods on `DiagramView`,
`DiagramContainer` and `TopologyContainer`.[^fg-draw] vibegraph converts each
view into its own `diagrams::Diagram` and drops the container at the module
boundary (`diagrams/mod.rs`, "feyngraph views never escape `diagrams/`").
Three classes of diagram therefore never have a view to draw:

- **stitched decay-chain diagrams** (`diagrams/chain.rs`, `generate_chain`);
- **leg-reordered sets** (`DiagramSet::with_final_order`, used by the
  hadronic flavour grouping in `proton.rs`);
- **diagrams loaded from an artifact**, which is the whole point of the
  artifact.

**The upstream change.** A public way to build a drawable diagram from parts:
external legs (particle name, incoming or outgoing), internal propagators
(particle, two vertex endpoints) and vertices, all against a feyngraph
`Model`. One form is a `Diagram::from_parts(…) -> Result<Diagram, …>` that
checks connectivity, plus a `DiagramView`-free `draw_svg_string(&Model)`.
The layout code (`drawing/layout.rs`, `DiagramLayout::from(&DiagramView)`)
reads only the connectivity and the particles, so the change exposes a
constructor and does not touch the drawing.

**On the vibegraph side**, a `diagrams::draw` module converts a
`diagrams::Diagram` back to feyngraph's parts, reusing `UFOModel::topo`. The
conversion stays inside `diagrams/`, as the module boundary requires. Leg
labels follow the vibegraph leg index, so an SVG drawn from a reordered or
stitched diagram labels its legs as that diagram does.

**Until upstream merges**, `Cargo.toml` pins a fork revision carrying the
same patch. The pin is replaced by the upstream revision when it is released.

## 2. The diagram artifact (`diagrams.bin.zst`)

The same encoding rules as every artifact
([artifact format versioning](artifact-format-versioning.md)): bincode, then
zstd, with a `format_version` header read first. It has its own version
sequence, starting at 1.

| Field | Meaning |
|---|---|
| `format_version` | Starts at 1. |
| `proc_card` | The proc card's text, verbatim. |
| `process` | The canonical process string. |
| `model` | `ModelIdentity` (name, restriction, digest) **and the restricted `ParsedModel` itself.** |
| `sets` | Every `DiagramSet` in enumeration order: particles, polarizations, `mirrored`, and the diagrams. |
| `diagram_set_digest` | SHA-256 over the canonical bincode of `sets`. |

- **The model is embedded.** The digest already hashes the restricted
  `ParsedModel`'s bincode (`ufo/identity.rs`), so the embedded bytes check
  themselves on load. A restricted SMEFTsim model is 46 KB compressed (§5), so
  embedding it costs little. Integrate started from this artifact reads no UFO
  directory, and its restriction is fixed.
- **`Diagram` needs serde.** That includes `Leg`, `Prop`, `Vertex`, `Ray`,
  `Provenance`, and the `sign` and `symmetry_factor` fields that are
  `pub(crate)` today. Everything that crosses the encoding uses ordered
  collections, as `ParsedModel` does, so the digest is reproducible.
- **Determinism is a hypothesis to pin.** The digest is only useful if
  enumeration is deterministic in order under feyngraph's parallel search
  (`EnumerationPool`). A test enumerates one card at `-j 1` and at `-j 4` and
  requires the two digests to be equal; it fails if the order varies.
- **Diagram ids** are `<set>.<diagram>`, zero-based in enumeration order, for
  example `s3.d17`. Each set's label (`u u~ > e+ e- g`) is carried alongside.
  Stitched and reordered diagrams take ids from the set they are stored in.

## 3. `vibegraph enumerate`

```
vibegraph enumerate <proc_card> [--out DIR] [--force] [--ufo-dir DIR]
                    [--svg all|none|first=N] [--svg-per-set N]
```

It writes into `<out>/`, `.` by default, with no silent overwrite, as
integrate does:

- `diagrams.bin.zst`, the artifact of §2;
- `diagrams.json`, the summary (schema below);
- `svg/<set-id>/<diagram-id>.svg`, one file per diagram. A large card such as
  `p p > e+ e- j j j` has 6,400 diagrams, so `--svg first=N` caps the count
  per set and the summary records which diagrams were drawn.

Under `--output-ldjson`, stdout carries the same summary as one result object.

**`diagrams.json`** carries `schema_version`, `process`, `model`
(label and digest), `diagram_set_digest`, and per set:

- the set's id and label, its particles and polarizations, and `mirrored`;
- the diagram count;
- coupling orders as a histogram (`{QCD: 2, QED: 2}` → count);
- the integration configurations as `config_groups` forms them
  (`helas/eval/compile.rs`), each with its member diagram ids, plus the
  contact diagrams that carry no configuration. Grouping needs the rooted
  amplitudes, so `enumerate` runs the compile up to `Configurations::group`.
  That costs at most 0.65 s on every card measured (§5);
- the flavour group the set belongs to. The hadronic grouping must be checked
  first: if it depends on the run card, `enumerate` reports it only when given
  `--run-card`.

Per diagram: id, set, index, propagators (particle, on-shell forced or not),
vertices (interaction names), coupling orders, configuration id or `null`,
and the SVG path or `null`.

## 4. `integrate` from either source, and what its artifact banks

The positional input of `integrate` is either a proc card or a
`diagrams.bin.zst`. The two are told apart by content, not by extension:
an artifact starts with the zstd magic `28 B5 2F FD`, and a proc card is
text. From an artifact, `integrate` skips parsing and enumeration and uses the
embedded model, so passing `--ufo-dir` is refused. `--param-card` binds as
today and must respect the restriction, which the artifact fixes.

The integrate artifact (its next version) adds:

| Field | Why |
|---|---|
| `model: ParsedModel` (restricted) | `generate` needs no UFO directory, and the parameter diff below becomes complete. Today's `inputs.params` records only parameters that differ from the model's defaults, so it means something only next to the exact model. |
| `param_card: Option<String>` | The card text verbatim, for the record and the status page. Binding still replays from `inputs.params` and `inputs.decays` (`ufo::bind::bind_recorded`), which are already banked. |
| `sets` + `diagram_set_digest` | `generate` reads the diagrams instead of enumerating again. Enumeration is the slow step: 5 s for `p p > e+ e- j j j`, and over 900 s on some 4-jet CMS cards ([diagram-enumeration-cost-unmeasured](../backlog/feature/diagram-enumeration-cost-unmeasured.md)). An integration started from an enumerate run records the same digest, so a ranking can be checked against the SVG directory it is paired with. |
| `ChannelGrid::members` | The full `(set, configuration, [diagram ids], final-order permutation)` list each channel serves. `MergedChannel` records only its first pair and a count today. A member of a hadronic group may be a reordered copy of an enumerated set, so the permutation maps it back to the drawn diagram. |
| `configurations` | Per `(set, configuration)`: `sigma_amp2`, `sigma_amp2_err`, and the member and contact diagram ids (§6). |
| `amplitude_probe` | Per set, \|M\|² at a few deterministic phase-space points under the bound card (§5, last row). |

**No compiled program is banked.** §5 gives the measurements behind this.

## 5. What to store: model, folded AST or lowered program

The question was what a self-contained artifact should carry: the UFO model,
the amplitude program's initial (folded) AST, or the fully lowered program.

The scratch harness took each card through three steps: enumeration
(`generate_from_proc_card`), compilation to the card-independent folded
`Ast<Const>` (`AmplitudeEvaluator::compile`), and helicity expansion plus
lowering to the instruction stream (`folded_hel().program()`).

**How the sizes were taken.** They are in-memory bytes, not serialized
encodings:
- Diagrams: an estimated compact encoding (4 B per leg, 8 B plus `n_ext` per
  propagator, 4 B plus 4 B per ray per vertex).
- Folded and expanded ASTs: node array plus CSR tables.
- Lowered program: `instrs`, `dest`, `operands`, `mom_operands` and
  `amp_locs` only, so it is a lower bound.

Compression would shrink every column, but would not change the 40–80× ratio
between the AST and the program.

| Process | Sets | Diagrams | Enumerate | Diagrams (est.) | Compile → AST | Folded AST | Expand + lower | Lowered program |
|---|---|---|---|---|---|---|---|---|
| `u u~ > e+ e-` | 1 | 2 | 0.001 s | 120 B | <0.001 s | 0.5 KB | <0.001 s | 3.1 KB |
| `g g > t t~ g` | 1 | 16 | 0.001 s | 1.5 KB | 0.002 s | 5.2 KB | 0.001 s | 91 KB |
| `g g > t t~ g g` | 1 | 123 | 0.008 s | 16 KB | 0.025 s | 46 KB | 0.046 s | 2.1 MB |
| `g g > g g g g` | 1 | 220 | 0.009 s | 25 KB | 0.65 s | 230 KB | 0.32 s | 11 MB |
| `p p > e+ e- j j` | 64 | 736 | 0.15 s | 96 KB | 0.05 s | 136 KB | 0.06 s | 3.8 MB |
| `p p > t t~ j j` | 65 | 1,119 | 0.23 s | 145 KB | 0.12 s | 315 KB | 0.18 s | 12 MB |
| `p p > e+ e- j j j` | 96 | 6,400 | **4.96 s** | 1.07 MB | 0.49 s | 1.07 MB | 1.25 s | 82 MB |
| SMEFTsim `g g > t t~ g NP<=1` | 1 | 172 | 0.006 s | 15 KB | 0.02 s | 54 KB | 0.03 s | 1.2 MB |
| SMEFTsim `g g > t t~ g g NP<=1` | 1 | 2,387 | 0.05 s | 294 KB | 0.64 s | 1.06 MB | 2.16 s | **48 MB** |

In the `p p` rows, `p` and `j` are `g u c d s u~ c~ d~ s~`. SMEFTsim is
`SMEFTsim_topU3l_MwScheme_UFO` under `restrict_massless`. SMEFTsim
`u u~ > t t~ b b~ NP<=1` could not be measured: the compiler panics on it
(Found, below).

Model sizes, as the bincode of the `ParsedModel` that would be embedded:

| Model | Raw | zstd -19 |
|---|---|---|
| SM, interned | 62 KB | 5.1 KB |
| SMEFTsim, unrestricted | 1.24 MB | 55 KB |
| SMEFTsim, `restrict_massless` (913 vertices) | 1.01 MB | 46 KB |
| SMEFTsim, `restrict_SMlimit_massless` (62 vertices) | 0.82 MB | 38 KB |

**Conclusions.**

- **The model must be embedded whatever else is.** A compiled program is not
  self-sufficient: its constant pools request couplings, masses and widths by
  id (`fold.rs`, `ComplexReq`/`RealReq`), and `BoundAmplitude::bind` resolves
  them from the model evaluated at the param card. At 5–46 KB compressed, the
  model is cheap.
- **Embed the diagrams.** Enumeration is the step that grows fastest: 4.96 s
  against 1.7 s for the whole compile on `p p > e+ e- j j j`, and the CMS
  survey has cards still enumerating after 900 s. The diagrams are also the
  smallest representation that carries all the information.
- **Do not store the lowered program.**
  - *Size:* it is 40–80× the folded AST, 48 MB for SMEFTsim
    `g g > t t~ g g` and 82 MB for `p p > e+ e- j j j`, before compression.
  - *Rebuild time:* at most 2.8 s on any card measured, against integrations
    of minutes or more.
  - *Compatibility:* the instruction stream, slot recycling and arena layout
    are what performance work changes, so almost every performance PR would
    need a format bump.
  - *Contract:* a pruned program carries the kinematic contract recorded in
    [the integrate artifact](integrate-artifact.md), and `folded_hel` is built
    lazily.
- **Do not store the folded AST either.**
  - *Benefit:* it saves only the compile step, at most 0.65 s; expansion and
    lowering, the larger cost at the top of the table, still run on load.
  - *Size:* it is 1–4× the diagrams.
  - *Compatibility:* its node language (`op.rs`) changes whenever a kernel is
    fused or an op is added.
  - *Bugs:* a banked AST also keeps the compiler's bugs. Recompiling from the
    diagrams picks up every compiler fix.
- **What recompiling costs, and the guard.** If a compiler change moves |M|²,
  an old artifact's banked σ no longer describes the matrix element the events
  are drawn from. That is already true today, since `generate` recompiles. The
  guard is `amplitude_probe`: |M|² at K = 4 fixed points per set, under the
  banked card, compared on load at a relative tolerance of 1e-10. It is a
  value check, so it ignores arena and ordering changes. Like
  `channel_map_digest`, it refuses a stale artifact without a hand-written
  version bump. Its tolerance follows the
  [ULP rule](../../../AGENTS.md): far above last-ulp noise, far below any
  convention or coupling error.

## 6. Importance: the AMP2 share and the density share

Integrate's channel σⱼ is a **density share**. A point counts towards channel j
in proportion to αⱼgⱼ(x)/g(x). It says which sampler did the work, not which
diagram matters, and channels whose maps coincide split the integral
uniformly ([multichannel](../phase-space/multichannel.md), "the channel
label").

The importance measure is MadEvent's partition:[^mg-results]

  σ_c = ∫ dΦ |M|² · AMP2_c / Σᵢ AMP2ᵢ,

summed over helicities, for each configuration c of each set. It is accumulated
from the same points and weights as σ during the iterations that enter the
result.

- **Cost.** `AMP2_c` is already computed on every point: the configuration
  amplitudes ride under the program's root (`amp_locs`). The cost is one ratio
  and one accumulator per configuration per point, which the first session
  measures.
- **Two built-in checks.** The shares sum to σ (Σ_c σ_c = σ to rounding,
  because the weights partition unity), and their errors are evaluated from
  the same points.
- **A per-configuration oracle against MadGraph.** MadEvent integrates each
  configuration in its own `G<n>` directory, and its `results.dat` records
  that channel's σ, which is exactly σ_c. That is a finer check than the total
  σ, as AGENTS.md asks.
- **Contact diagrams** have no configuration. They are listed under their set
  as unranked, and they contribute to every σ_c through |M|².

## 7. The ranking JSON

The ranking JSON is written by `vibegraph inspect --ranking <grid.bin.zst>`,
a read of the artifact with nothing run again. The schema is versioned and
documented here; the page that renders it is designed separately.

```json
{
  "schema_version": 1,
  "process": "...", "model": {"label": "...", "digest": "..."},
  "observable": "sigma", "unit": "pb", "sigma": 0.0, "sigma_err": 0.0,
  "diagram_set_digest": "...",
  "sets": [{"id": "s0", "label": "u u~ > e+ e- g", "mirrored": true, "diagrams": 16}],
  "configurations": [
    {"rank": 1, "id": "s0.c3", "set": "s0", "diagrams": ["s0.d5", "s0.d9"],
     "sigma": 0.0, "sigma_err": 0.0, "share": 0.0}
  ],
  "unranked": [{"set": "s0", "diagrams": ["s0.d15"], "reason": "contact"}],
  "channels": [
    {"index": 0, "key": {"MergedChannel": {}}, "alpha": 0.0, "neval": 0,
     "sigma": 0.0, "sigma_err": 0.0, "chi2_per_dof": 0.0,
     "configurations": ["s0.c3", "s4.c3"]}
  ]
}
```

- `configurations` is sorted by `sigma` descending; `share` is `sigma / σ`.
- Diagram ids are the SVG file stems, so a page loads
  `svg/<set>/<id>.svg` directly.
- A page given `diagrams.json` and a ranking with different
  `diagram_set_digest` values must refuse to pair them.

## 8. Sessions

Each session is one PR. They land in this order.

1. **Upstream feyngraph drawing API.** The PR to Jens-Braun/FeynGraph, and a
   fork pin until it merges. Gate: a round-trip test that draws a view
   directly and through `from_parts`, and requires byte-identical SVG.
2. **Diagram artifact and serde.** Serde on `Diagram` and its parts,
   `DiagramArtifact` v1 with the embedded model, `diagram_set_digest`, and the
   `-j 1`/`-j 4` determinism test. The refusal test writes a real old shape,
   per the versioning rules.
3. **`vibegraph enumerate`.** Artifact, `diagrams.json`, SVGs, and the
   configuration grouping.
4. **`integrate` from either source; self-contained replay.** Input sniffing,
   the integrate artifact's new fields (model, param card text, sets, digest,
   `amplitude_probe`), and `generate` reading them with no UFO directory and
   no enumeration. Gate: the `extended-validation` replay gates, plus an
   integration from a proc card and one from the same card's enumerate run
   that give bit-identical artifacts at a pinned seed.
5. **AMP2-share accounting and channel membership.** `configurations` and
   `ChannelGrid::members`. Gates: the sum rule, a ≥5-seed sweep, and the
   per-configuration comparison against MadEvent `results.dat` on a few
   multi-configuration rows.
6. **Ranking JSON.** `inspect --ranking`, its schema test, and the
   digest-pairing check.

## Open questions

- Does the hadronic flavour grouping depend on the run card? If it does,
  `diagrams.json` reports flavour groups only under `--run-card` (§3).
- Where should the feyngraph fork live, and under which account, until
  upstream merges?
- Should the ranking come from `inspect --ranking`, as above, or from a
  separate `vibegraph rank` command?
- `amplitude_probe`: is K = 4 points per set at a relative tolerance of 1e-10
  the right default, or should the probe be opt-in?

## Found while designing

- **A compiler panic.** `AmplitudeEvaluator::compile` panics on SMEFTsim
  (`restrict_massless`) `u u~ > t t~ b b~ NP<=1`: index out of bounds at
  `helas/eval/analysis.rs:603`, where `momentum_into` meets an `Op::Add` with
  no operands. It is reached from `Folded::build` (`fold.rs:220`), so any
  `integrate` run on that process fails the same way.
- **Stale concepts.** [The integrate artifact](integrate-artifact.md) and
  [artifact format versioning](artifact-format-versioning.md) describe format
  version 15, but `artifact.rs` is at 21, and neither covers the
  `inputs.params`/`inputs.decays` param-card record.

[^fg-diagram]: feyngraph `src/diagram/mod.rs`, `pub struct Diagram`, at the pinned revision.
[^fg-draw]: feyngraph `src/drawing/mod.rs`, at the pinned revision.
[^mg-results]: MadEvent `matrix_madevent_v4.inc`, the `MULTI_CHANNEL` factor.
