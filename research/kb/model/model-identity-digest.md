---
type: Design Decision
title: "Model identity: SHA-256 over the serialized restricted model"
description: "The integrate artifact banks the import label and a SHA-256 of the restricted ParsedModel's bincode bytes, so a different model under the same process string is refused."
status: draft
tags: [artifact, model, identity, digest, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-identity, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L713-L804", title: "Note 23, model identity in the artifact"}
  - {id: code-identity, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/ufo/identity.rs", title: "vibegraph-lib/src/ufo/identity.rs"}
  - {id: code-generate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/generate.rs#L274-L330", title: "vibegraph-cli/src/generate.rs card_mismatches"}
---

`vibegraph generate` replays the grids an `integrate` run trained, so it must refuse a
card that would describe a different integrand. A process string does not name a model:
`import model sm-no_b_mass` with a byte-identical `generate` line and run card is a
different model from `import model sm`. The integrate artifact therefore banks a
**model identity** beside the process and run card. [^n23-identity]

```rust
pub struct ModelIdentity { name: String, restrict: String, digest: String }
```

- `name` and `restrict` are the `import model <name>[-<restrict>]` directive
  (`"default"` for a bare `import model sm`); `label()` prints `<name>-<restrict>` for
  error messages.
- `digest` is **SHA-256** (`sha2`) over the **bincode encoding of the restricted
  `ParsedModel`** (`ufo::identity::model_digest`), the same encoding the interned SM
  blob holds. [^code-identity]

The schema the identity lives in, and which artifact versions are still read, are in
[artifact format versioning](../pipeline/artifact-format-versioning.md) and
[the integrate artifact](../pipeline/integrate-artifact.md). The artifacts written
before the identity existed are older than the oldest version the reader decodes.

## Why the digest is over the parsed model

The digest hashes the model's own serialized form, not the UFO source files:

- Two source trees that parse to the same particles, couplings, vertices and parameters
  are the same model. A reworded comment, reordered imports or reformatted whitespace
  must not refuse an artifact.
- The restriction is baked in before hashing (`ParsedModel::apply_restriction`), so a
  restrict card contributes through its **effect** and not its text. Each interned SM
  variant gets its own digest (`ufo::sm::sm_digest`: the blob with that variant's card
  applied).
- Loading and identifying share one path, `UFOModel::load_with_digest`, reached from
  `GlobalConfig::load_ufo_with_identity` (`vibegraph-lib/src/config.rs`), so a banked
  digest cannot describe a different model from the one loaded.

The hash function is a dependency, not an in-tree one. What the digest needs is
stability across builds and platforms, which rules out `std`'s `DefaultHasher`
(documented as unstable between releases); `digest_bytes` is pinned by a known answer
(`digest_bytes_matches_sha256`). This is the `AGENTS.md` rule on standard primitives
applied.

**Determinism is a standing constraint on `ParsedModel`.** Serializing must be
reproducible, so nothing reachable from `ParsedModel` may be a `HashMap` or `HashSet`:
their iteration order varies per instance and per process, and a digest over them would
make `generate` refuse `integrate`'s artifacts at random. Name-keyed collections are
`IndexMap`s (their order is semantic: `ParticleId`/`CouplingId` index by it); everything
else is a `BTreeMap`/`BTreeSet`. `the_digest_survives_a_serialization_round_trip` guards
this, and `cli_generate` covers the cross-process case, banking a digest in one process
and comparing it in another.

## How a mismatch is reported

`card_mismatches` (`vibegraph-cli/src/generate.rs`) compares the banked identity with the
loaded one [^code-generate]:

- a differing **label** is reported as `model`, by name;
- with equal labels, a differing **digest** is reported as ``model `<label>` contents``.
  This is the case a name provably cannot see: a restrict card regenerated with
  different contents under an unchanged name.

The process string and every run-card parameter are compared in the same pass.

## Tests and their blind spots

| Test | Pins | Blind to |
|---|---|---|
| `generate::tests::a_different_restrict_variant_is_refused` | `sm` vs `sm-no_b_mass` under an identical process and run card is refused, as `model` | a digest wrong the same way on both sides |
| `generate::tests::a_restrict_card_that_changed_under_its_name_is_refused` | same label, different digest is refused | whether the digest's inputs are the right bytes (it plants the digest) |
| `ufo::identity::every_interned_variant_has_its_own_digest` | the restrict card reaches the SM digest | a stale blob (`interned_blob_matches_submodule_exactly` covers that) |
| `ufo::identity::a_changed_model_digests_differently`, `interned_identity_is_reproducible` | a model change moves the digest; the same model gives the same digest | collisions in general |
| `config::identity_follows_the_resolved_variant` | the identity returned is the variant resolved | a digest wrong the same way on both sides |
| `cli_generate::a_card_that_did_not_train_the_grid_is_refused` | end to end: a card importing another restrict variant is refused by the binary | — |

With both comparisons deleted, the end-to-end test fails, and the accepted wrong-model run
wrote events at σ = 2061.99 pb against the banked 2022.62 pb (+1.95%): that is the size
of error the check stands in front of. [^n23-identity]

## What the identity does not cover

- **The param card.** The digest identifies the restricted model, not parameter values
  a run binds later. The CLI has no param-card option today and every run computes at
  the restriction's defaults
  ([backlog](../backlog/feature/cli-has-no-param-card-option.md)); when one is added, the
  artifact has to record the card beside the identity.
- **A self-contained replay.** A worker still needs the cards, the PDF set and, for a
  non-SM model, the UFO directory
  ([backlog](../backlog/feature/generate-artifact-not-self-contained.md)).

Related: [restrict-card semantics](restriction-semantics.md), which defines what the
restriction bakes in.

[^n23-identity]: Note 23, "model identity in the artifact": the gap, the digest design, the determinism defect it exposed, the test and mutation tables.
[^code-identity]: `vibegraph-lib/src/ufo/identity.rs` module documentation and `model_digest`.
[^code-generate]: `vibegraph-cli/src/generate.rs` `card_mismatches`.
