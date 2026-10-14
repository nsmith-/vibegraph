---
type: Session Report
title: "V1 report: mechanical visibility demotion"
description: "pub lines cut from 2601 to 1359 in vibegraph-lib and to 0 in the two binary crates; dead types deleted; 144 dead_code allows and ~120 de-linked doc links left for V1b."
status: draft
tags: [hygiene, visibility, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: 827e098, resource: "https://github.com/nsmith-/vibegraph/commit/827e098", title: "refactor: demote pub items of the binary crates to pub(crate)"}
  - {id: bddeb92, resource: "https://github.com/nsmith-/vibegraph/commit/bddeb92", title: "refactor(lib): demote pub items that no other target names"}
  - {id: cd51d8d, resource: "https://github.com/nsmith-/vibegraph/commit/cd51d8d", title: "refactor(lib): narrow visibility the first demotion pass over-granted"}
  - {id: 34f92ea, resource: "https://github.com/nsmith-/vibegraph/commit/34f92ea", title: "style(lib): drop blank lines left in the re-export blocks"}
---
The dev agent's report, condensed by the manager. Everything except the
manager check is the agent's own claim.

## Result

Four commits on `hygiene-v1` (base `4bdd921`): `827e098`, `bddeb92`, `cd51d8d`
and `34f92ea`. The diff is 114 files, +1767 / −2234.

`pub` lines, counted by `grep -rhE '^\s*pub ' <crate>/src --include='*.rs' | wc -l`:

| crate | before | after |
|---|---|---|
| vibegraph-lib | 2601 | 1359 |
| vibegraph-cli | 101 | 0 |
| validation-report | 95 | 0 |

Both binary crates have only bin targets, so their items became
`pub(crate)` wholesale, with no dead code.

**dead-types-with-stale-docs is closed.** Deleted:
- the whole of `helas/repr/coupling.rs`: `Vertex3`, `GaugeVertex` (whose
  `apply` was `todo!()`), `ColorStructure` and `LorentzStructure`;
- `Intertwiner2Leg`/`3Leg`/`4Leg`. `intertwiner.rs` remains as a doc-only
  private module that keeps the vertex-factor table;
- `runcard::classes::Applicability` and the `when:` field of
  `FieldClass::IgnoredPhysics`, with the `lpp1`/`lpp2` parameters of
  `refuse_ignored_physics` and the test's `proton_only` bookkeeping.

**Other deletions** (unused after demotion, and named by no code, test, doc
or kb):
- `ScaleChoice::pdfwgt`, `DiagramSet::is_polarized`, `Diagram::ray_momentum_in`
- `FixedBeams::masses`, `FixedBeamIntegrand::{initial_state, subprocess_count}`
- `ColorFactor::conj`, `ColorFlowTags::select`
- `AmplitudeEvaluator::coupling_particle_ids`, `Folded::{coupling_ids, particle_ids}`
- `LorentzVector` `raise`/`lower`/`from_p_theta_phi_mass`, `ComplexVector::raise`,
  `AsymRank2Tensor::components`, `Chirality::flip`, `VectorWf::{raise, lower}`
- `SubprocessRecord::masses`, `MultiplicitySum::set_budget_shares`
- `PdfMember::force_positive` (the getter), `GridAlphaS::declared_mz_value`
- `RamboChannel::masses`, `DiagramChannel::n_out`, `SubStream::stream`
- `Support::{arity, len, is_empty}`, `UFOModel::load_auto`,
  `EvaluatedModel::vertex_couplings`, `ParamCard::has_block`
- the `Comparison::worst_*` family and `worst_distribution`
- `AliasTable::labels`, `Unweighter::scans`, `Ast::is_empty`

23 `pub use` re-exports nothing outside used were narrowed or removed.

## Remaining surface (the proposal for lib-pub-api-surface-unaudited)

The agent's full grouping, per `pub mod`, into *used by vibegraph-cli*,
*test/bench-only* and *named by no target* (pub only through a pub
signature), exists only in its hand-back in the manager's session. It is not
copied here, because V1b changes the surface. The proposal is re-derived from
the final tree at close-out ([Z](Z.md), step 2).

The grouping is by identifier grep: common names (`new`, `len`, field names)
put some members under "used by vibegraph-cli" wrongly. Two structural facts
from it hold:
- **Used by validation-report is empty:** that crate does not depend on
  `vibegraph-lib`. The brief assumed it did.
- **About 110 types are `pub` only because a pub signature exposes them,**
  mostly error enums. They belong to the surface decision.

## Method (lesson data)

1. **Binary crates:** a `sed` from `pub` to `pub(crate)`. Clippy was clean
   at once.
2. **Library, pass 1:** a script demoted every non-module `pub`, then a loop
   ran `cargo check --keep-going --workspace --all-targets --all-features
   --message-format=json` and re-promoted each definition an error pointed
   to. The error classes were E0603, E0624, E0616/E0451, E0532/E0423,
   E0364/E0365, "type X is private" and private associated types. About 50
   iterations, each under a minute.
   - **Pitfall:** E0364/E0365 inside the library block every dependent
     target, so error counts swing wildly (4, then 2603).
   - **Over-promotion the compiler cannot reject:** whole `pub use {…}`
     groups promoted for one name; field-name matches against same-named
     structs in the `artifact::v3`…`v7` inline modules; window searches.
3. **Pass 2 (strict):** demote everything again, and re-promote only the
   exact span or the definition resolved by module path. Three hand fixes.
4. **Not seen by the error loop:**
   - `private_interfaces`/`private_bounds` forced 24 types back to `pub`.
   - clippy lints that skip the public API now fired: `len_without_is_empty`
     (two `is_empty` back to `pub`) and `wrong_self_convention` (one
     commented allow).
5. **`cargo fix` for unused imports** checks the non-test lib only. It
   removed re-exports that unit tests use; these were restored under
   `#[allow(unused_imports)]`.
6. **Dead-code classification** compared three builds: non-test, test
   profile, and default features. Grepping names was too noisy (`masses`,
   `stream`, `lower`).
7. **Doc warnings:** rustdoc JSON spans fed a scripted de-linker. Its first
   version mangled multi-link lines, and a regex pass repaired them.
8. **Near miss:** a stray `git checkout -q -- .` in a script command was
   blocked by the permission classifier before it discarded the pass-1 work.

## Manager check (2026-10-09)

Re-run in `/home/user/wt/hygiene-v1` at `34f92ea`:

- **Commits and trailers:** four commits, `Assisted-by` only.
- **`pub` counts:** reproduced exactly (2601 → 1359, 101 → 0, 95 → 0).
- **Deleted names:** `WeylBasis`, `MinkowskiRep` and `ProtonBeams` are absent
  from `vibegraph-lib/src`.
- **Tests:** the diff removes and adds no `#[test]`.
- **Lints:** `cargo fmt --all --check` passes, and so does
  `cargo clippy --workspace --all-targets [--all-features] -- -D warnings`
  in both configurations, exit 0.
- **Hermetic suite:** `cargo test --workspace` exits 0 with 35 suites, 1348
  passed, 0 failed, 17 ignored.
- **Allows:** `git diff 4bdd921 HEAD -- vibegraph-lib/src | grep -c '^+.*allow(dead_code)'`
  gives 144.

## Found

1. **The 144 `#[allow(dead_code)]` sites:**
   - most are items only unit tests use (`vegas` runners, `helas::vertex`
     kernels, `repr::lorentz` constructors, `schannel` matching, lips2
     helpers, `proton`/`hadronic`/`lhef::build` helpers, `ColorRepr`);
   - some are items only feature-gated code uses (the `stats` KS helpers,
     `lhef::observables`, `root_diagram::propagator_particles`,
     `MG_VALIDATED_PROCESSES`, `assert_op_coverage`);
   - some have no code user and are named only in docs or the kb
     (`LHAPDF_INDEX_URL`, `enumerate_decay`, `expect_fermion_in/out`,
     `ColorRepr::DIM/REP`, `ColorSinglet`, `FlavorGroup::external_legs/member_luminosity`,
     `DiracWf::charge`, `VegasGrid::combination`);
   - some are write-only fields (`Clustering::lines`,
     `ClusterScales::njets/mt2`, `EmitSummary::weight_sum/max_source_weight`,
     `UFOModel::propagators`, `ChannelScan::draws/mean`).

   **Taken up by V1b.**
2. **Stale kb after the deletions:**
   - `amplitudes/repr-layer-geometry-and-axes.md` (~:65-78 and its
     footnotes), on the intertwiner traits and `Vertex3`/`GaugeVertex`;
   - `run-card/field-classification.md` (~:68 and ~:132), on
     `Applicability`/`ProtonBeams`.

   Close-out.
3. **`intertwiner.rs` is a doc-only private module:** fold its table into
   `repr/mod.rs` or the kb, then delete the module.
4. **The API site lost every demoted item, and about 120 intra-doc links
   became plain code spans.** Taken up by V1b, which restores the links and
   documents private items in every build.
5. **The test-only re-exports carry `#[allow(unused_imports)]`.** Taken up
   by V1b.
6. **About 110 types are `pub` only through pub signatures.** This belongs
   to the surface decision.
7. **The `FIELD_CLASSES` column alignment** is stale now that `when:` is gone.
8. **The surface grouping is grep-based.** A precise answer needs a per-item
   demote-and-compile run, or rustdoc JSON.

## Brief corrections

- **validation-report does not depend on vibegraph-lib.**
- **The doc gate conflicted with "no doc rewrites beyond the item".** It is
  resolved by de-linking, which the user then reversed (log, 2026-10-09).
- **"Allow with no comment" produced 144 sites.** The rule was amended in
  D2.
- **The gate omitted CI's default-feature clippy.** V1 ran it as well.
- **Deleting `Applicability`** reached `refuse_ignored_physics`'s signature
  and its caller.
- **One comment outside scope was rewritten:** `lorentz.rs`'s "WeylBasis
  numerics" became "Weyl-basis numerics".
