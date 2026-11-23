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
