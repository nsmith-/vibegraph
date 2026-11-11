---
type: Design
title: "The ranking JSON: vibegraph inspect --ranking"
description: "Schema version 1 of the document vibegraph inspect --ranking writes from an integrate artifact: configurations ranked by their AMP2 share σ_c, the unattributed weight and contact diagrams, the sampler's per-channel density shares kept apart, every diagram with its coupling orders and drawing, the pairing rule with diagrams.json, and how the schema is versioned."
status: draft
tags: [cli, inspect, ranking, json, schema, configurations, multichannel, svg]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: ranking-rs, resource: "vibegraph-cli/src/ranking.rs", title: "ranking.rs: the document's types, build and SCHEMA_VERSION"}
  - {id: inspect-rs, resource: "vibegraph-cli/src/inspect.rs", title: "inspect.rs: the --ranking, --diagrams and --force flags"}
  - {id: enumerate-rs, resource: "vibegraph-cli/src/enumerate.rs", title: "enumerate.rs: set_id, diagram_id, configuration_id, svg_path"}
  - {id: tests, resource: "vibegraph-cli/tests/cli_ranking.rs", title: "cli_ranking.rs: the end-to-end, pairing, ldjson and proton-beam checks"}
  - {id: design-114, resource: "https://github.com/nsmith-/vibegraph/pull/114", title: "nsmith-/vibegraph#114: the vibegraph enumerate and diagram-ranking design, §6 and §7"}
---
`vibegraph inspect <grid.bin.zst> --ranking <FILE>` writes one JSON document
that orders an integration's configurations by their share of the observable,
under the ids [`diagrams.json`](diagrams-json.md) gives the same sets,
diagrams and drawings.[^ranking-rs] It reads the
[integrate artifact](integrate-artifact.md) alone: nothing is integrated or
enumerated again. The page that renders it is designed separately; the
design is nsmith-/vibegraph#114, §6 and §7.[^design-114]

## The command

```
vibegraph inspect <grid.bin.zst> --ranking <FILE|-> [--diagrams <DIR|diagrams.json>] [--force]
```

- `--ranking -` writes the document to stdout and nothing else.[^inspect-rs] It is refused
  under `--output-ldjson`, whose stream stdout already carries.
- An existing `--ranking` file is refused without `--force`, before the
  artifact is read, as `integrate` and `enumerate` refuse theirs.
- `--diagrams` names an `enumerate` output directory or its `diagrams.json`.
  It is refused unless its `diagram_set_digest` equals the artifact's; the
  refusal names both digests and says to pass the `enumerate` run the
  integration started from, or to enumerate the artifact's card again. It is
  also refused if a configuration it lists does not hold the diagrams this
  build groups under that id: the digest covers the diagrams, not the code
  that groups them, so two builds can agree on one and not the other. Every
  refusal comes before anything is written.
- In the text modes the command prints one line (the path, the number ranked,
  the first configuration and its share). Under `--output-ldjson` the
  `result` object carries `ranking_path` and the document under `ranking`,
  nested so its `schema_version` stays apart from the stream's.

## σ_c against σⱼ: what is ranked

Two partitions of σ exist, and the document keeps them apart:

- **σ_c, the AMP2 share** (`configurations`). Every point's weight is divided
  over the evaluator's configurations by `w_c / Σ w`, MadEvent's
  single-diagram-enhancement factor (`AMP2_c` under `sde_strategy = 1`). It
  says how much of σ each configuration's diagrams carry, and it is what
  MadEvent's `G` directory for the configuration integrates. It is the
  importance measure, and the only thing ranked.
- **σⱼ, the density share** (`channels`). A point counts towards channel j in
  proportion to `αⱼgⱼ(x)/g(x)`. It says which sampler did the work, not which
  diagrams matter: channels whose maps coincide split the integral evenly
  whatever the diagrams' weights, and on the
  [configuration-share oracle](../validation/configuration-share-oracle.md)'s
  rows it reads 3 to 2027 standard deviations off MadEvent's per-channel σ,
  where σ_c agrees within 1.9.

So a page ranks by `configurations[].sigma` and shows `channels` as sampler
diagnostics (α, `neval`, χ²/dof). Ranking by σⱼ would rank sampling
efficiency.

## Ids and drawings

The ids are `diagrams.json`'s, formed by the same functions:[^enumerate-rs]

- a set is `s<k>`; a diagram `s<k>.d<j>`;
- a configuration is `s<k>.c<j>`: `diagrams.json`'s `c<j>` of set `s<k>`,
  qualified so it is unique across the card. The index is that of
  `config_groups`, found by the configuration's diagrams, not the integrated
  evaluator's `AMP2` index: the two differ where a whole configuration has no
  amplitude the colour basis references;
- a flavour group is `g<j>`, in the artifact's evaluator order, which is the
  order `diagrams.json`'s preview numbers them in.

`svg` paths are relative to the `enumerate` output directory, as in
`diagrams.json`. With `--diagrams`, each is the path `diagrams.json`
records, `null` where `enumerate` drew none (`--svg`). Without it, every
diagram carries the conventional `svg/<set>/<diagram>.svg`, which exists only
where it was drawn; `drawings.paired` is `false`. Paths relative to the
`enumerate` directory, not to the ranking file, keep the two documents'
strings identical, and a ranking on stdout has no location to be relative to.

**The pairing rule.** A reader given a `diagrams.json` and a ranking must
refuse to pair them unless their `diagram_set_digest` values are equal.

## Top level

| Key | Type | Meaning |
|---|---|---|
| `schema_version` | integer | `1` (versioning below). |
| `process` | string | The canonical process string. |
| `model` | object | `label` (`<name>-<restrict>`) and `digest`, as in `diagrams.json`. |
| `observable` | string | `cross_section` or `partial_width`. |
| `unit` | string | `pb` or `GeV`: the unit of every `sigma` and `sigma_err` in the document. |
| `sigma`, `sigma_err` | number | The run's total and its error. |
| `chi2_per_dof` | number or null | The run's iteration χ²/dof; `null` where fewer than two iterations entered. |
| `seed` | integer | The integration seed. |
| `diagram_set_digest` | string | The artifact's digest of its sets: the pairing key. |
| `artifact_format_version` | integer | The integrate artifact's format version. |
| `beams` | string | `proton`, `fixed_energy` or `decay`. |
| `weight` | string or null | The enhancement weight σ_c was split by: `amp2` (`sde_strategy = 1`) or `channel_cut` (`sde_strategy = 2`). |
| `drawings` | object | `paired` (a `diagrams.json` was read) and `summary` (its path as given, or `null`). |
| `flavour_groups` | array | `{id, representative, members}`, `members` each `{set, final_order}`: the groups the integration banked. Empty except at proton beams. |
| `sets` | array | Every set of the artifact, in id order. |
| `configurations` | array | The ranked configurations. |
| `unranked` | array | What carries no rank. |
| `channels` | array | The sampler's channels, in the artifact's order. |
| `diagrams` | array | Every diagram, in id order. |

`final_order[i]` is the set's outgoing leg that is outgoing leg `i` of the
integrated term; reordering keeps diagram indices.

### `sets[]`

| Key | Type | Meaning |
|---|---|---|
| `id`, `label`, `mirrored`, `diagrams` | | As in `diagrams.json` (`diagrams` is the count). |
| `integrated` | bool | The set's matrix element was integrated: a fixed-energy subprocess, or a flavour group's representative. Ranked configurations name only these. |
| `flavour_group` | string or null | The group the set is a member of; `null` off proton beams. |

### `configurations[]`

Sorted by `sigma` descending. Equal values keep the artifact's order
(evaluator by evaluator, each in `AMP2` order), so the order is a function of
the artifact.

| Key | Type | Meaning |
|---|---|---|
| `rank` | integer | From 1. |
| `id` | string | `s<k>.c<j>`. |
| `set` | string | The integrated set. |
| `diagrams` | string array | The configuration's diagrams that carry an amplitude the colour basis references. |
| `sigma`, `sigma_err` | number | σ_c and its error. |
| `share` | number | `sigma / σ`. |
| `cumulative_share` | number | The shares of this rank and every rank above it. |
| `members` | array | `{set, configuration, diagrams}`: the same configuration in each set the share includes, as the channels' membership records it. At proton beams a share is the whole group's (every member's luminosity, both beam orderings), not split by member. |
| `channels` | integer array | The channels whose maps serve the configuration. |

### `unranked[]`

`{set, reason, diagrams, sigma, sigma_err, share}`, per integrated set:

| `reason` | What |
|---|---|
| `no_configuration` | The weight at points where every enhancement weight was zero. `sigma`, `sigma_err` and `share` are set (often `0`); `diagrams` is empty. One per integrated set. |
| `contact` | Diagrams in no configuration (`diagrams.json`'s `contact`, such as the four-gluon vertex). `sigma` is `null`: they enter every σ_c through \|M\|². |
| `colour_null` | Diagrams of a configuration that carry no amplitude the colour basis references. `sigma` is `null`. |

The ranked shares and the `no_configuration` shares sum to 1 to rounding.

### `channels[]`

`index`, `key` (tagged by `type`, as `inspect --output-ldjson` writes it),
`alpha`, `neval`, `sigma` and `sigma_err` (σⱼ), `chi2_per_dof`,
`configurations` (the ranked ids its members are counted in, each once) and
`members`: per `(set, configuration)` the channel serves, `{set,
final_order, configuration, diagrams, counted_in}`, `configuration` being the
set's own and `counted_in` the ranked configuration (`null` where the
channel's diagrams carry none).

### `diagrams[]`

`{id, set, configuration, coupling_orders, svg}`: the configuration id or
`null` for a contact diagram, the coupling orders summed over the vertices
(order name → power), and the drawing path.

## Versioning

`schema_version` follows `diagrams.json`'s rule: adding a key keeps it;
removing, renaming or re-meaning one bumps it. Re-meaning includes changing
what `sigma` partitions by, the sort key, or what an id refers to. A reader
refuses a version it does not know. Numbers are JSON numbers written with
full round-trip precision (`serde_json`'s `float_roundtrip`), so a value read
back is the artifact's bit for bit.

## Checks

`vibegraph-cli/tests/cli_ranking.rs`:[^tests]

- `g g > g g g` at fixed energy, enumerated with `--svg first=20` and
  integrated from the diagram artifact: the ranking is descending; the shares
  and the unattributed weight sum to 1 within 1e-12; every configuration and
  diagram id is one `diagrams.json` names, with the same membership; every
  `svg` equals `diagrams.json`'s and exists on disk; the contact diagrams are
  `diagrams.json`'s.
- The numbers are those `inspect --output-ldjson` reports, value for value.
- A `diagrams.json` of another card is refused, naming both digests, with no
  file written and an existing one untouched even under `--force`.
- `p p > e+ e- j` over `g u c d s` and antiquarks with the committed PDF set:
  the flavour groups equal `enumerate`'s preview under the same run card.

Mutation-checked: with the digest comparison removed, the refusal test fails
(the grouping check still refuses, but names no digest); with the sort
reversed, both end-to-end tests fail. **Blind spots:** whether σ_c is right
(the configuration-share oracle covers that), and a configuration whose
`AMP2` index differs from its `config_groups` index, which none of these cards
has.

[^ranking-rs]: `vibegraph-cli/src/ranking.rs`, `build`.
[^inspect-rs]: `vibegraph-cli/src/inspect.rs`, `InspectArgs`.
[^enumerate-rs]: `vibegraph-cli/src/enumerate.rs`, `set_id`, `diagram_id`, `configuration_id`, `svg_path`.
[^tests]: `vibegraph-cli/tests/cli_ranking.rs`.
[^design-114]: The design is not on `main`; it is the concept added by nsmith-/vibegraph#114.
