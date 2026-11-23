---
type: Session Report
title: "L-Z5 report: hygiene pass on run-card and UFO refusals and the parameter and provenance reports"
description: "Tracked-tree audit (two root scratch cards untracked), then a findings-first review of PR 7 (#172) and PR 8 (#171): a single-sourced subset text, a typed run-card problem, tightened placeholder, writer-version and diagram-artifact reason tests, and the report's identity shown byte for byte against main."
status: draft
tags: [hygiene, run-card, ufo, parameters, provenance, audit]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-ufo-reports` off main 8b3af5ab. Reviewed: #172 `7ca8d8bd`
(run-card placeholders and UFO parameter refusals) and #171 `d5e7c71f`
(model-parameter and artifact-provenance reports), with their direct callers and
tests.

## Tracked-tree audit

Commit 8819800e, on its own. Commands, from the worktree root:

```
$ git ls-files -s | awk '$1!="160000"{print $4}' > files.txt   # regular files only: the brief's
                                                               # `git ls-files | xargs du` also du's
                                                               # the four submodule gitlinks as directories
$ xargs -d '\n' du -k < files.txt | sort -n | tail -50          # largest: validation/madgraph/standalone/*.json
                                                               # (≤2.4 MB, oracle tables), assets/demo_pp.gif 2.4 MB,
                                                               # reweight_mg_reference.json 764 kB, assets/badger.png 760 kB
$ git ls-files | grep -i -E '\.(log|tmp|bak|orig|rej|swp|o|a|so|rlib|rmeta|d)$|/(out|tmp|scratch)/|\.DS_Store'
                                                               # (no output)
$ git ls-files -ci --exclude-standard                           # (no output, before and after)
binary check (grep -Iq): assets/badger.png, assets/demo_ee.gif, assets/demo_pp.gif (README.md),
  validation/helas/reference.npz (validation/helas/gen_reference.py, pixi.toml),
  vibegraph-lib/src/ufo/sm_assets/sm_parsed.bin.zst (the embedded parsed SM): each has a reader.
by extension outside research/kb and validation: .dat, .txt, .json, .lock, .js, .html each checked by name.
```

Untracked:

| path | size | added by | why it is junk |
|---|---|---|---|
| `proc_card.dat` | 25 B | 5331e45d "release: dispatch acceptance, which the release event cannot start" (2026-08-11; d0097b9c in the rewritten history) | the README quickstart's card (`echo 'generate e+ e- > mu+ mu-' > proc_card.dat`), left at the root; no test or script reads a root card (every test writes its own in a temp dir) |
| `run_card.dat` | 46 B | same | the README quickstart's run card (`0 = lpp1 … 45.6 = ebeam2`), same |

`.gitignore` gains `/proc_card.dat` and `/run_card.dat`, so following the
quickstart at the root does not add them again. Nothing else in the tracked tree
is build output, scratch, log or editor output: the `research/kb` non-Markdown
files (`pr13-speedup/*.dat`, `v022-1009-never-retried.tsv`) are the inputs and
data of recorded measurements, and the `validation/` tree holds scripts,
patches, Fortran probes and reference tables.

## Identity oracle

Chosen before the first edit. Main is 8b3af5ab, built in its own detached
worktree `/home/user/wt/lz5-main` with its own target. Every byte comparison
runs both binaries on the same inputs, in the same relative layout, and diffs
the outputs; the only masked field is the `version` string of `parameters.json`
and the ldjson `vibegraph_version`, which carry the git description and so
differ between any two commits.

1. `vibegraph parameters CARD --out out --output-ldjson`, then `parameters.json`,
   `param_card.dat` and the ldjson stream byte-identical, for: `import model sm`
   + `generate e+ e- > mu+ mu-`; `import model sm-no_b_mass` + `generate p p > e+ e- j`;
   the hermetic SMEFTsim (`SMEFTsim_topU3l_MwScheme_UFO-massless`, `e+ e- > t t~ NP<=1`);
   and the two conditional store models L-U1 unpacked,
   `SMEFTsim_topU3l_MwScheme_UFO_ctGpatched-massless` (a conditional `value`) and
   `dim6top_LO_UFO` (`if configuration.` blocks), model only.
2. `inspect ARTIFACT --output-ldjson` byte-identical (paths are relative) on one
   artifact from `integrate` of `e+ e- > mu+ mu-` (45.6 GeV beams,
   `--fixed-budget --neval 4000 --niter 3 --seed 5 -j 1`), each binary reading
   the same file; and the two binaries' `grid.bin.zst` sha256 equal.
3. `run-card CARD --output-ldjson` byte-identical on
   `validation/madgraph/pp_to_llj_mlm_run_card.dat` and on the placeholder card
   of `cli_run_card.rs` (`fixed()` plus `$DEFAULT_PDF_SETS = lhaid`).
4. `enumerate` on `e+ e- > mu+ mu-` and `p p > e+ e- j`: `diagrams.bin.zst`
   sha256 equal and `diagrams.json`, `parameters.json`, `param_card.dat`
   byte-identical (`version` masked).

## Findings

Committed before any code edit. Classes: *dup* dead or duplicated code; *vac* a
vacuous or tautological test; *vis* a public item that should be private;
*list* a hand-kept list that should be derived; *abs* a missing reusable
abstraction; *cmt* a comment that breaks AGENTS.md's rules; *panic* a panic or
unwrap on a fallible path. Ranked by consequence. Confidence is *checked* (the
quoted evidence settles it) or *suspected* (settled by the mutation run below).

| id | rank | file:line | class | what is wrong | evidence | proposed fix | fix here |
|---|---|---|---|---|---|---|---|
| Z5-1 | 1 | `ufo/parameters.rs:26-39`, `ufo/couplings.rs:159-161`, `ufo/config_eval.rs:9-13` | list | The subset of Python the loader evaluates is written by hand four times, and the copies disagree with the evaluator and each other: both `ParameterError` messages omit unary `-`/`+` and `A if T else B`; the coupling-order message omits unary `-`/`+` and the boolean literals; the module doc omits unary `+`. `eval_int` (`config_eval.rs:52-113`) accepts all of them. *checked* | quoted lines; `eval_int` arms `UnaryOp::USub`, `UnaryOp::UAdd`, `IfExp` | one `SUBSET` text in `config_eval`, read by the three messages and pointed to by the doc | yes |
| Z5-2 | 2 | `runcard/scan.rs:16-20,48-58`, `runcard/report.rs:427-438` | list, dup | `UnknownKey::problem` is a `&'static str` that `report.rs` compares with the literal `"placeholder"` written separately in `scan.rs`; a typo in either silently drops the key's refusal. The `UnfilledPlaceholder` error is also built twice from the same three fields (scan for `RunCard::parse`, report for the key). *checked* | `grep -rn '"placeholder"' vibegraph-lib/src` → `report.rs:428`, `scan.rs:57` (and a test) | a `Problem` enum serialised snake_case (the JSON is unchanged) and one `UnknownKey::refusal()` that both sites call | yes |
| Z5-3 | 3 | `vibegraph-cli/tests/cli_run_card.rs:315-374` | vac | The placeholder test checks the refusal on `lhaid`'s key only; nothing checks that no other key is refused. Mutation: `canonical_name(&key.name) == Some(name)` → `true` in `report.rs:428`. *suspected* to survive | the asserts quoted in the test | assert the set of keys carrying a refusal is exactly the placeholder's key | yes |
| Z5-4 | 4 | `artifact.rs:718-731`, test `write_refuses_a_version_this_build_cannot_read` (`artifact.rs:1437`) | vac, cmt | The write guard was narrowed to `!= FORMAT_VERSION` because a version-24 file has no writer field, but the test loops over `[OLDEST_READABLE_VERSION - 1, FORMAT_VERSION + 1]` only, so version 24 (readable, unwritable) is untested. Mutation: restore `!(OLDEST_READABLE_VERSION..=FORMAT_VERSION).contains(..)`. The doc still says it refuses a version "`read_from_path` would refuse", and `UnwritableVersion` says "this build reads versions 24..=25" when it refuses 24. *suspected* to survive | quoted lines | add `WRITERLESS_VERSION` to the loop; doc and message say only the current version is written | yes |
| Z5-5 | 5 | `vibegraph-cli/src/integrate.rs:739`, `error.rs` `From<&DiagramArtifactError>` | vac | No test reaches a diagram-artifact refusal's `reason`: every provenance test feeds `inspect`/`generate` a grid artifact. Mutation: delete `.with_reason(&e)` at `integrate.rs:739` (reason becomes `other`). *suspected* to survive | `grep -n with_reason vibegraph-cli/src` → integrate.rs:739 is the only `DiagramArtifactError` site; `grep -n 'diagrams.bin.zst' vibegraph-cli/tests/cli_artifact_provenance.rs` → none | a test: enumerate, stamp enumeration version 2 into `diagrams.bin.zst`, `integrate` it, expect `reason.kind = enumeration_version` with `found` 2 | yes |
| Z5-6 | 6 | `vibegraph-cli/src/error.rs` (both `From` impls, `_ => ArtifactReason::Other`) | list | A wildcard arm maps every variant not listed to `other`, so a new `ArtifactError` or `DiagramArtifactError` variant (a new kind of corrupt file) gets `other` with no compiler or test notice. *checked* | `grep -n '_ => ArtifactReason::Other' vibegraph-cli/src/error.rs` → 3 lines | name every remaining variant in each arm so the match is exhaustive | yes |
| Z5-7 | 7 | `vibegraph-cli/src/parameters.rs:100-105` | dup | `io_error` is a byte-identical copy of `enumerate.rs:266-271`. *checked* | `grep -rn 'fn io_error' -A6 vibegraph-cli/src` | one `io_error` in `error.rs` | yes |
| Z5-8 | 8 | `ufo/report.rs:176-181,449-454` | dup | The physical particles in PDG order (`pdg_code > 0 && !is_unphysical()`, sorted) are built twice in the module. *checked* | quoted lines | one helper | yes |
| Z5-9 | 9 | `ufo/report.rs:382` | vis | `describe_parameters` is `pub` with no user outside the module (the CLI calls `describe_parameters_at`). *checked* | `grep -rn 'describe_parameters(' --include=*.rs` → report.rs only | private | yes |
| Z5-10 | 10 | `ufo/report.rs` test `the_entering_list_follows_the_couplings` | cmt | A stray `println!("{entering:?}")`. *checked* | quoted | delete | yes |
| Z5-11 | 11 | `artifact.rs:124` | cmt | One 104-character line (the file's only one over 100) in the `FORMAT_VERSION` doc. *checked* | `awk 'length > 100' ` on the file | rewrap | yes |
| Z5-12 | — | `ufo/parameters.rs:403` | defect | `ConditionOutside.names` lists every `Parameter` under *any* `if` of the file, not the one whose condition failed, and is empty when that `if` holds no `Parameter`: the refusal then reads "parameter(s)  are defined under an `if`…". A non-`Parameter` statement under an out-of-subset `if` was accepted before #172 and is refused now. *suspected*; settled by a scratch test below | `conditional_definitions(&parsed, "Parameter").unwrap_or_default()` | the failing `if` should be named by itself (its line, its own definitions); the choice between refusing and skipping a definition-free `if` is a design call | no: Found |
| Z5-13 | — | `ufo/report.rs:312-342` vs `reweight/engine.rs:705-727`, `reweight/mod.rs` `check_merged` | dup | `verdict_of` re-derives the external-mass rule (moved = dependents + name; any external's mass parameter in it) and the merged rule rather than calling the reweight code, while the module doc says the report "holds no copy of those rules". *checked* | quoted lines | one shared predicate in `reweight` | no: Found (cross-module) |
| Z5-14 | — | `ufo/report.rs:281`, `ufo/bind.rs:252`, `diagrams/diagram.rs:668,701` | abs | The Majorana test `spin.abs() == 2 && is_self_conjugate` is written four times. *checked* | `grep -rn 'spin.abs() == 2 && .*is_self_conjugate'` | `Particle::is_majorana_fermion()` | no: Found |
| Z5-15 | — | `ufo/bind.rs:251`, `lhef/write.rs:484` (and Z5-8's two) | abs | the same physical-particles filter in two more modules. *checked* | `grep -rn 'pdg_code > 0 && !p.is_unphysical()'` | a `UFOModel` method | no: Found |
| Z5-16 | — | `vibegraph-cli/src/{integrate.rs:971,inspect.rs:76,generate.rs:421,enumerate.rs:519,parameters.rs:143}` | abs | Five hand-written "already exists (pass --force to overwrite)" refusals. *checked* | grep, 5 lines | one helper in `error.rs`; generate.rs is another cluster's | no: Found |
| Z5-17 | — | `artifact.rs` `ArtifactError`, `diagrams/artifact.rs` `DiagramArtifactError` | abs | Two error enums with the same ten variants, so `error.rs` maps each twice (two 40-line `From` impls). *checked* | the variant lists | one shared header/digest error | no: Found |
| Z5-18 | — | `ufo/couplings.rs:88-90` | other | A `configuration.py` syntax error is reported as "Failed to parse couplings.py", because `parse_configuration` maps to `CouplingError::Parse`. Predates #172, which made every load read the file first. *checked* | the `#[error]` text, `couplings.rs:19` | its own error naming the file | no: Found |
| Z5-19 | — | brief | — | "A second evaluator of UFO parameter expressions between `config_eval.rs` and `parameters.rs` or `couplings.rs`": not reproduced. #172 deleted `couplings.rs`'s `order_value`/`literal_int` and both readers call `eval_int`; `expr.rs`'s `eval` evaluates UFO value *strings* (complex arithmetic), a different language. | `git show 7ca8d8bd -- vibegraph-lib/src/ufo/couplings.rs` | — | not reproduced |
| Z5-20 | — | `ufo/report.rs` tests, `cli_parameters.rs` `a_restriction_that_ties_and_fixes_parameters` | dup | The tied-restriction fixture is built twice, once per crate. | quoted | — | leave: the library test pins the types, the CLI test the wire shape |

Found by the mutation run (step 2), after the findings commit. Each is a test
that passed its one-line mutation on the pre-fix tree (evidence in
[Mutations](#mutations)):

| id | file | class | what the test could not see | fix |
|---|---|---|---|---|
| Z5-21 | `ufo/config_eval.rs` `the_subset_matches_python` | vac | every `and`/`or` case decides on its last operand, so an evaluator that never short-circuits passes (M01) | cases `configuration.f and 4` → 0, `configuration.t or 9` → 1 |
| Z5-22 | `ufo/config_eval.rs` `branches_flatten_to_the_ones_that_run` | vac | no false `if` without an `else`, so a flatten that runs the body of one passes (M04); no other test in the workspace has one either | a leading false `if` with no `else`, whose binding must not appear |
| Z5-23 | `ufo/report.rs` tests, `cli_parameters.rs` | vac | no model in either has a Majorana mass, so giving every mass a `minimum` passes (M16) | `a_majorana_mass_has_no_minimum` on the hermetic `vibegraph_toy_majorana_UFO` (`MNT` none, `MLT` one) |
| Z5-24 | `cli_parameters.rs` `the_entering_list_of_a_smeft_process` | vac | `enters` is checked for `couplings` only and `max_degree` nowhere, so never reporting `masses_widths` (M22) or reporting degree 0 (M21) passes | `MT` enters `["masses_widths"]`, `MZ` both; every entering Wilson coefficient is `{"status": "polynomial", "max_degree": 1}` |
| Z5-25 | `vibegraph-cli/src/parameters.rs:137` | vac | nothing runs `parameters` twice, so ignoring `--force` passes (M25) | `an_earlier_report_is_not_overwritten_without_force` |

## What changed

| commit | findings | change |
|---|---|---|
| 8819800e | audit | untrack `proc_card.dat`, `run_card.dat` at the root; ignore them there |
| 57a44622 | — | this report's findings and oracle, before any code edit |
| a044f73d | Z5-1, Z5-21, Z5-22 | `config_subset!` in `config_eval.rs`, read by both `ParameterError` messages and the coupling-order `EXPECTED` (`concat!`); module doc points at it; two subset cases and a false-`if` case |
| 998a804b | Z5-2, Z5-3 | `runcard::scan::Problem` (snake_case, JSON unchanged) replaces the `&'static str`; `UnknownKey::placeholder_refusal` builds the one refusal for the parser and the key report; the CLI placeholder test asserts which keys carry it |
| b38aa809 | Z5-4, Z5-11 | `write_to_path` doc and `UnwritableVersion` text say only `FORMAT_VERSION` is written; the test (renamed `write_refuses_every_version_but_the_current_one`) adds `WRITERLESS_VERSION`; doc line rewrapped |
| ce8952f2 | Z5-5, Z5-6, Z5-7 | every `ArtifactError`/`DiagramArtifactError` variant named in its reason map; one `io_error` in `error.rs`; `a_diagram_artifact_refusal_carries_its_reason` |
| ac7f2f9f | Z5-8, Z5-9, Z5-10, Z5-23, Z5-24, Z5-25 | `physical_particles` helper; `describe_parameters` private, its doc moved to `describe_parameters_at`; `println!` removed; three tests tightened or added |

Tests added: `ufo::report::tests::a_majorana_mass_has_no_minimum`,
`cli_artifact_provenance::a_diagram_artifact_refusal_carries_its_reason`,
`cli_parameters::an_earlier_report_is_not_overwritten_without_force` (3; none
deleted; one renamed). Refusal *texts* changed on purpose, and only these:
`ParameterError::ConditionOutside`/`ValueConditionOutside`, the coupling-order
`order` expectation, and `ArtifactError::UnwritableVersion` (a library-only path).

Final dispositions of the step-1 table: Z5-1 to Z5-11 *fixed*; Z5-12 to Z5-18
*Found*; Z5-19 *not reproduced*; Z5-20 *leave*. Z5-12 is now *checked*: on main
and on this branch, a model whose `parameters.py` ends
`import configuration` / `if configuration.x > 1:` / `    plain = 1.0`
(`configuration.py`: `x = 3`) is refused with
`parameter(s)  are defined under an `if` of parameters.py whose condition is outside the subset …`
(two spaces: no name).

## Mutations

Gated mutations: every edit is applied at once, each active only when
`LZ5_MUT=<id>` is set at run time, so one build serves the whole batch
(`scratchpad/lz5/gated.py`, driver `gdrive.py`); an unmutated control run
passes first. The pre-fix tree is 57a44622, the fixed tree ac7f2f9f, each in a
throwaway worktree with its own target. *Pre* is the PR's own test, *post* the
test after this pass. Logs: `/home/user/wt/logs/lz5-gmut-pre.log`,
`lz5-gmut-post.log` (both `CONTROL_OK`).

| id | site | mutation | test | pre | post |
|---|---|---|---|---|---|
| M01 | config_eval `BoolOp` | no short-circuit | `the_subset_matches_python` | **survived** | killed |
| M02 | config_eval `USub` | drop the negation | same | killed | killed |
| M03 | `read_configuration` | `bare: false` | `configuration_assignments_read_earlier_ones` | killed | killed |
| M04 | `flatten_branches` | a false `if` with no `else` runs its body | `branches_flatten_to_the_ones_that_run` | **survived** | killed |
| M05 | `select_branch` | swap the arms | `definitions_under_configuration_flags_follow_the_branch_that_runs` | killed | killed |
| M06 | parameters | value refusal → definition refusal | `a_condition_outside_the_subset_is_refused_by_name` | killed | killed |
| M07 | parameters | accept a number where a string is required | `a_non_string_value_that_is_not_a_conditional_is_still_refused` | killed | killed |
| M08 | `flatten_branches` | refuse every `if` | `conditional_non_parameter_statements_are_not_refused` | killed | killed |
| M09 | `ufo/mod.rs` | parameters read with an empty configuration | `definitions_and_values_switched_on_a_configuration_flag_follow_the_flag` | killed | killed |
| M10 | `select_branch` | an unevaluable test reads as true | `a_parameter_condition_outside_the_subset_is_refused_by_name` | killed | killed |
| M11 | config_eval `Add` | subtract | `a_coupling_order_may_switch_on_a_configuration_flag` | killed | killed |
| M12 | config_eval other operators | `/` evaluates | `a_coupling_order_outside_the_read_forms_is_refused` | killed | killed |
| M13 | runcard key report | refuse every key | `an_unfilled_placeholder_is_refused_by_name` | **survived** | killed |
| M14 | runcard scan | the parser never refuses a placeholder | same | killed | killed |
| M15 | `class_of` | never `one` | `one_and_merged_classes_have_their_records` | killed | killed |
| M16 | `constraints_of` | every mass gets a minimum | `ufo::report::tests` | **survived** | killed |
| M17 | internal records | none listed | `internal_masses_are_listed_with_their_computed_value` | killed | killed |
| M18 | `verdict_of` | strong coupling → movable | `verdicts_agree_with_the_reweight_resolver` | killed | killed |
| M19 | `reaches` | ignore dependents | `the_entering_list_matches_a_numeric_perturbation` | killed | killed |
| M20 | `default_param_card` | drop the `DECAY` lines | `default_card_matches_the_restriction_card` | killed | killed |
| M21 | `polynomial_of` | degree 0 | `ufo::report::tests` + `cli_parameters` | **survived** | killed |
| M22 | `describe_parameters_at` | never `masses_widths` | same | **survived** | killed |
| M23 | `verdict_of` | external mass → movable | `the_entering_list_of_a_smeft_process` | killed | killed |
| M24 | `particles_of` / `physical_particles` | reverse the PDG order | `ufo::report::tests` + `cli_parameters` | survived | **survived** |
| M25 | CLI `parameters` | ignore `--force` | `cli_parameters` | **survived** | killed |
| M26 | `enumerate` | report without the sets | `enumerate_writes_the_report_and_card` | killed | killed |
| M27 | CLI `parameters` | schema version 2 | `a_model_only_report_pins_each_class` | killed | killed |
| M28 | `integrate.rs:739` | no reason on a diagram-artifact refusal | `cli_artifact_provenance` | **survived** | killed |
| M29 | `inspect.rs` | no reason | `an_enumeration_version_2_artifact_is_refused_by_reason` | killed | killed |
| M30 | `generate.rs` read | no reason | `an_edited_digest_is_refused_by_reason` | killed | killed |
| M31 | `generate.rs` `cli_error` | no reason | `generate_errors_about_the_artifact_carry_their_reason` | killed | killed |
| M32 | `read_from_path` | no absent-writer tag for 24 | `a_format_24_artifact_loads_unchanged_with_no_writer` | killed | killed |
| M33 | `write_to_path` | the readable range is writable | `artifact::tests` | **survived** | killed |
| M34 | reason map | unreadable → other | `other_refusals_name_unreadable_or_carry_no_reason` | killed | killed |
| M35 | `Refusal` | serialize an absent reason as `null` | same | killed | killed |
| M36 | integrate result | enumeration version 0 | `results_carry_the_enumeration_version_and_the_writer` | killed | killed |
| M37 | integrate result | no writer | `a_new_artifact_records_its_writer` | killed | killed |
| M38 | `err` | every kind carries a reason | `other_refusals…` + `output::tests` | killed | killed |

Pre: 28 killed, 10 survived. Post: 37 killed, 1 survived (M24). The tightened
tests were each shown failing on their mutation (post column); none was
weakened.

**No mutation reaches M24's site:** the order of a record's `particles` matters
only for a parameter that is the mass or width of two physical particles. No
record in the oracle's five reports (SM, SM-no_b_mass, SMEFTsim, ctGpatched,
dim6top) lists more than one particle, so no test can pin it.

## Oracles

**Identity, main 8b3af5ab against head ac7f2f9f** (binaries copied to the
scratchpad as `vibegraph-main`, `vibegraph-head`):

```
$ oracle.sh vibegraph-main o-main2 o-main/ig/own.bin.zst
$ oracle.sh vibegraph-head o-head  o-main/ig/own.bin.zst
$ python3 compare.py o-main2 o-head
compared 57 files, 0 differ; 36/36 unmasked files byte-identical
$ cat o-main2/ig/grid.sha256 o-head/ig/grid.sha256                 # integrate e+ e- > mu+ mu-, seed 5
20577c674b4b5eb471aef53afd9ea7e769b197e009a296bfea9a1e4513b63239
20577c674b4b5eb471aef53afd9ea7e769b197e009a296bfea9a1e4513b63239
$ cat o-main2/e_*/diagrams.sha256 o-head/e_*/diagrams.sha256       # e+ e- > mu+ mu-, p p > e+ e- j
fa143aa62622df43462720386590a60a1aafe6a3653dabd0fae76d75f10bc180
5182d2d7368f716f4f3d90257d522d357cc4c1e08a5f12ae52c26e5e4b57d243
fa143aa62622df43462720386590a60a1aafe6a3653dabd0fae76d75f10bc180
5182d2d7368f716f4f3d90257d522d357cc4c1e08a5f12ae52c26e5e4b57d243
$ vibegraph-head --version
vibegraph v0.2.2-35-gac7f2f9f
```

The 57 files are, per binary: for `parameters` on `sm` (27 records, 3 entering),
`sm-no_b_mass` with `p p > e+ e- j` (27, 4), SMEFTsim-massless with
`e+ e- > t t~ NP<=1` (219, 20), ctGpatched-massless (219) and dim6top (473):
`parameters.json`, `param_card.dat`, the ldjson stream and stderr; for
`enumerate` on the two processes: `diagrams.json`, `parameters.json`,
`param_card.dat`, the stream and stderr; the two `run-card` reports
(`pp_to_llj_mlm_run_card.dat`; the placeholder card); `integrate`'s result and
the `inspect` of its own artifact and of one shared artifact written by main.
Masked: `version`/`vibegraph_version` (the git description) and `timestamp`
values; dropped: ldjson `log`, `phase` and `progress` lines (wall-clock
timings). The field-by-field diff of `integrate`'s result object differs in
`vibegraph_version` only. 36 of the files need no mask at all and are
byte-identical as written.

An intermediate binary built before the test-only edits (`vibegraph-br1`)
gave the same `0 differ`.

**Z5-12** (no fix, the evidence for Found): see the end of
[What changed](#what-changed).

## What each test cannot detect

- `the_subset_matches_python` and the flatten test share my reading of Python
  precedence and short-circuit rules; nothing runs Python.
- The placeholder test pins *which* key carries the refusal; it cannot see a
  placeholder for a key that is not a run-card parameter being attached to a
  look-alike key under another canonical spelling.
- `a_majorana_mass_has_no_minimum` uses a toy model; it is blind to a model
  that marks a Majorana fermion other than by `spin = 2` and self-conjugacy.
- The degree-one assertion holds because SMEFTsim's couplings are linear in the
  Wilson coefficients at `NP<=1`; it cannot see a wrong degree for a parameter
  that is not a Wilson coefficient (`Gf`, `MZ` read `not_polynomial`, which no
  test pins).
- `a_diagram_artifact_refusal_carries_its_reason` covers the enumeration-version
  reason of a diagram artifact only; the other reasons of that path are pinned
  through the grid artifact's identical mapping, not end to end.
- M24: the order of a record's `particles` (above).
- The identity oracle covers loadable models and their reports; refusal texts
  are out of it, and three changed on purpose (What changed).

## Gate

Run at ac7f2f9f.

```
$ flock /home/user/wt/heavy.lock /tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-hyg-ufo-reports lz5
head ac7f2f9f
FMT_OK
    Finished `dev` profile [optimized] target(s) in 44.70s
CLIPPY_RC=0
    Finished `dev` profile [optimized] target(s) in 24.26s
CLIPPY_EV_RC=0                       # --features vibegraph/extended-validation,vibegraph-lib/extended-validation
TEST_RC=0
lines=79 passed=2355 failed=0        # cargo test --workspace --no-fail-fast, the three data paths hidden
$ cargo doc -p vibegraph-lib --no-deps --document-private-items
warning: `vibegraph-lib` (lib doc) generated 13 warnings      # main 8b3af5ab: 13, the same sites
$ cargo doc -p vibegraph --no-deps --document-private-items   # no warning, as on main
$ cargo test -p vibegraph --features extended-validation --test cli_check_only --test cli_integrate
test result: ok. 5 passed; 0 failed      # cli_check_only: integrate --check on run cards (the placeholder refusal's path)
test result: ok. 6 passed; 0 failed      # cli_integrate: artifact writing (the writer-version guard's path)
$ pixi run --skip-deps kb-lint
1394 concepts, 480 backlog items; 0 errors, 7 warnings  # none on this report
```

## Long-run requests

None.

## Method

- Diff reading and grepping each new item's name and body line across the
  workspace (about 35%): Z5-2, Z5-7, Z5-8, Z5-9, Z5-13 to Z5-17.
- Comparing hand-written texts with the code they describe (about 15%): Z5-1,
  Z5-4's doc, Z5-18.
- Mutations (about 35%): 38, one per added test or test group; 10 survived,
  including Z5-3, Z5-4 and Z5-5 that the review had predicted and Z5-21 to
  Z5-25 that it had not. Running them one rebuild each cost 8 minutes a
  mutation under the shared lock; gating every mutation on an environment
  variable and building once cut the batch from about five hours to two builds.
- Tracked-tree audit (about 10%).
- Leads that did not hold: the brief's second evaluator (Z5-19); the
  `ConditionOutside` empty-name case is real (Z5-12) but its fix is a design
  call.

## Found

1. **Out-of-subset `if` refusals name the wrong parameters, or none**
   (Z5-12). `parameters.rs:403` lists every `Parameter` under any `if` of the
   file; a failing `if` that defines no `Parameter` gives
   "parameter(s)  are defined…", and such an `if` was accepted before #172.
   Close: name the failing `if` itself (its line and its own definitions), and
   decide whether a definition-free `if` is refused or skipped.
2. **The report re-derives two reweight rules** (Z5-13): `report.rs`
   `verdict_of` copies `reweight/engine.rs` `check_external_masses` and the
   merged-group test, against its module doc. Close: one predicate in
   `reweight`, called by both.
3. **The Majorana-fermion test is written four times** (Z5-14):
   `ufo/report.rs:281`, `ufo/bind.rs:252`, `diagrams/diagram.rs:668,701`.
   Close: `Particle::is_majorana_fermion`.
4. **The physical-particle filter is written in three modules** (Z5-15):
   `ufo/report.rs` (now one helper), `ufo/bind.rs:251`, `lhef/write.rs:484`.
   Close: a `UFOModel` method.
5. **Five hand-written overwrite refusals** (Z5-16): `integrate.rs:971`,
   `inspect.rs:76`, `generate.rs:421`, `enumerate.rs:519`, `parameters.rs:143`.
   Close: one helper in `error.rs` (generate.rs belongs to the multicore
   generate cluster).
6. **Two parallel artifact error enums** (Z5-17): `ArtifactError` and
   `DiagramArtifactError` share ten variants, so `error.rs` maps each twice.
   Close: one shared header-and-digest error.
7. **A `configuration.py` syntax error is reported as a couplings.py one**
   (Z5-18): `couplings.rs:88-90` maps it to `CouplingError::Parse`, whose text is
   "Failed to parse couplings.py". Close: its own variant naming the file.
8. **Kb sites:** none made stale by this pass (no kb page quotes the changed
   refusal texts; `L-U1-report.md` paraphrases them, as a record).

## Brief corrections

- The audit command `git ls-files | xargs -d '\n' du -k` also sizes the four
  submodule gitlinks as directories (`research/refs/mg5amcnlo` heads the list at
  143 MB); the audit used regular files only.
- "`vibegraph-lib/src/runcard/`" is `runcard.rs` plus `runcard/`; #172 touched
  both.
- The likeliest finding, a second evaluator of parameter expressions, did not
  reproduce (Z5-19); the duplicated refusal text did (Z5-1, Z5-2).
- The brief's `ufo_value_forms`/`cli_parameters` "hermetic store" holds no
  conditional BSM model: those tests write their own. The oracle used the
  ctGpatched and dim6top models L-U1 unpacked in the scratchpad.
- `verify-hidden.sh` runs several cargo commands and moves the data paths
  aside, so it was run under one `flock` for its whole length rather than one
  per cargo command.
- Five fix commits were made after the targeted tests and before the full
  gate, which then ran on their head and passed; nothing was pushed before it.
