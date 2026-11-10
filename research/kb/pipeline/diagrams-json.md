---
type: Design
title: "diagrams.json: the summary vibegraph enumerate writes"
description: "Schema version 1 of diagrams.json: per subprocess its label, coupling-order histogram, integration configurations and contact diagrams, the optional flavour-group preview, and per diagram its propagators, vertices, orders, configuration and SVG drawing; how ids join it to the diagram artifact and the drawings."
status: draft
tags: [cli, enumerate, diagrams, json, schema, svg, configurations, flavour-groups]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: enumerate-rs, resource: "vibegraph-cli/src/enumerate.rs", title: "enumerate.rs: the command, the Summary types and SCHEMA_VERSION"}
  - {id: draw-rs, resource: "vibegraph-lib/src/diagrams/draw.rs", title: "diagrams/draw.rs: drawing a vibegraph Diagram through feyngraph"}
  - {id: artifact-rs, resource: "vibegraph-lib/src/diagrams/artifact.rs", title: "diagrams/artifact.rs: the diagram artifact and diagram_set_digest"}
  - {id: compile-rs, resource: "vibegraph-lib/src/helas/eval/compile.rs", title: "helas/eval/compile.rs: config_groups"}
  - {id: multiplicity-rs, resource: "vibegraph-lib/src/multiplicity.rs", title: "multiplicity.rs: multiplicity_groups, integrate's flavour decomposition"}
  - {id: design-114, resource: "https://github.com/nsmith-/vibegraph/pull/114", title: "nsmith-/vibegraph#114: the vibegraph enumerate and diagram-ranking design"}
---
`vibegraph enumerate <proc_card>` writes three outputs into `--out`:[^enumerate-rs]

| Path | What |
|---|---|
| `diagrams.bin.zst` | The diagram artifact: the card, the restricted model, every subprocess's diagrams in enumeration order, and `diagram_set_digest`.[^artifact-rs] |
| `diagrams.json` | The summary this concept describes. |
| `svg/<set>/<diagram>.svg` | One drawing per diagram, for the diagrams `--svg` selects. |

The command and the ranking that pairs with it are designed in
nsmith-/vibegraph#114.[^design-114] Without `--force` it refuses to write when
any of the three paths exists, and it checks that before reading the card. With
`--force` it first removes the drawings an earlier run wrote (every
`svg/s<k>/s<k>.d<j>.svg`, then each directory left empty), so no drawing
outlives the summary that named it, and nothing else under `svg/` is touched.

## Ids

- A **set** is one subprocess, `s<k>`, numbered from zero in enumeration
  order: the order of `DiagramArtifact::sets`.
- A **diagram** is `s<k>.d<j>`, numbered from zero within its set in the
  set's diagram order. Stitched decay-chain diagrams take the id of the set
  they are stored in.
- A **configuration** is `c<j>`, local to its set, numbered in the order
  `config_groups` returns them: the order of their opening diagram, MadGraph's
  `AMP2` numbering.[^compile-rs]
- A **flavour group** is `g<j>`, numbered over the whole card in
  `integrate`'s order: phase space by phase space, group by group.

Drawing paths are relative to the output directory and are the diagram id
with `.svg`, so a page that holds a diagram id loads `svg/<set>/<id>.svg`
without consulting the summary.

## Top level

| Key | Type | Meaning |
|---|---|---|
| `schema_version` | integer | `1`. Adding a key keeps it; removing, renaming or re-meaning one bumps it. |
| `process` | string | The card's canonical process string. |
| `model` | object | `label` (`<name>-<restrict>`) and `digest`, the model digest the artifact records. |
| `diagram_set_digest` | string | The artifact's digest of its sets. A file paired with another output (a ranking) is paired only when the two digests are equal. |
| `flavour_groups` | object | Present only when `--run-card` or `--param-card` was given (below). |
| `sets` | array | One object per set, in id order. |

## Per set

| Key | Type | Meaning |
|---|---|---|
| `id` | string | `s<k>`. |
| `label` | string | `u u~ > e+ e- g`, with each polarized leg's helicities (`z{0}`). |
| `particles_in`, `particles_out` | string arrays | The legs in the set's own order; leg `i` of every diagram is the `i`-th of the two lists joined. |
| `polarizations` | array | Per leg, incoming first: the helicities a polarized leg is restricted to, or `null`. |
| `mirrored` | bool | The set stands for both beam orderings. |
| `diagrams` | integer | The number of diagrams. |
| `coupling_orders` | array | The histogram: one `{orders, diagrams}` per distinct order map, `orders` mapping each order name to its power (an order no vertex carries is absent). |
| `configurations` | array | `{id, diagrams}`: the diagram ids sharing one integration configuration, as `config_groups` partitions them. |
| `contact` | string array | The diagrams in no configuration (the ones with a vertex wider than the narrowest diagram's widest, such as the four-gluon contact). |
| `flavour_group` | string or null | Present only when `flavour_groups` is: the group the set joins, or `null` when it joins none. |
| `diagram_list` | array | One object per diagram, in id order. |

## Per diagram

| Key | Type | Meaning |
|---|---|---|
| `id` | string | `s<k>.d<j>`. |
| `propagators` | array | Per internal line, `{particle, forced_onshell}`: the particle flowing with the line's momentum, and whether a decay chain forces it on shell. |
| `vertices` | string array | The UFO interaction of each vertex (`V_12`), without the fermion-flow group. |
| `coupling_orders` | object | Order name → power, summed over the vertices. |
| `configuration` | string or null | The configuration id, or `null` for a contact diagram. |
| `svg` | string or null | The drawing's path, or `null` when `--svg` did not select it. |

`--svg` is `all`, `none` or `first=N` per set, `first=500` by default. Drawing
costs about a millisecond and 5 KB per diagram: `p p > e+ e- j j j` (6,400
diagrams) enumerates in 8.0 s with no drawing and 12.3 s with all of them, for
31 MB of SVG. The default draws every diagram of the cards up to that size and
caps the rare set larger than 500 diagrams.

## Drawings

Each diagram is rebuilt as a feyngraph diagram from its parts and drawn by
feyngraph's layout and SVG renderer.[^draw-rs] An external leg is labelled with
its physical particle and its index in the set's leg order, so a stitched or
reordered diagram is labelled as its set reads. A diagram whose vertices do not
carry their interactions' particles, or that feyngraph refuses as a tree, fails
the run before anything is written.

The conversion is pinned by drawing every diagram feyngraph enumerates for a
set of SM and SMEFTsim processes both ways and requiring byte-identical SVG and
the same rebuilt graph and sign.

## The flavour-group preview

The grouping `integrate` banks depends on both cards: the param card moves
masses and couplings, and the run card moves the cut classes. So `enumerate`
reports none unless a card is given, and then reports a preview computed by
`multiplicity_groups`, the function `integrate` calls, with `integrate`'s
default for the card not given.[^multiplicity-rs]

| Key | Type | Meaning |
|---|---|---|
| `run_card`, `param_card` | string or null | The card paths given; `null` for `integrate`'s default. |
| `beams` | string | `proton`, `fixed_energy` or `decay`. Only proton beams group flavours; for the other two, `groups` is empty and every set's `flavour_group` is `null`. |
| `groups` | array | `{id, phase_space, representative, members}`: `phase_space` is the index of the final-state multiplicity and outgoing-mass part the group is integrated in, and `members` the set ids, the representative first. |

A set is matched to a group member by its card line, its beams in order, its
outgoing content with polarizations, its mirroring and its diagram count. Two
sets that agree on all of these (decay chains reaching one subprocess through
different decay assignments) are matched to members in enumeration order.

## Under `--output-ldjson`

The `result` object carries `artifact`, `summary_path`, `svg_dir` (`null` when
nothing was drawn), `drawings`, and the whole summary under `summary`. It is
nested rather than flattened so that its `schema_version` stays apart from the
stream's own.

[^enumerate-rs]: `vibegraph-cli/src/enumerate.rs`.
[^draw-rs]: `vibegraph-lib/src/diagrams/draw.rs`, `DiagramDrawer`.
[^artifact-rs]: `vibegraph-lib/src/diagrams/artifact.rs`, `DiagramArtifact`.
[^compile-rs]: `vibegraph-lib/src/helas/eval/compile.rs`, `config_groups`.
[^multiplicity-rs]: `vibegraph-lib/src/multiplicity.rs`, `multiplicity_groups`.
[^design-114]: The design is not on `main`; it is the concept added by nsmith-/vibegraph#114.
