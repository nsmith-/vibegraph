---
type: Session Report
title: "L-Z2 report: hygiene of Majorana fermion flow, the chirality exchange and the Goldstone filter"
description: "Findings, mutations, fixes and identity oracles of the hygiene pass over the Majorana/fermion-number-violation change (#169) and the Goldstone-spelling and list-propagator parser fix (#178)."
status: draft
tags: [hygiene, majorana, fermion-flow, goldstone, mutation-testing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-fermion` off main 8b3af5ab (brief bbffe26c). Scope:
`git show a5bf8fca` (#169) and `git show c18042ed` (#178), their callers and
tests. This section was committed before any code edit.

## Findings

Ranked; *fix* marks what this pass fixes. Final dispositions: F1–F9, F11, F15 fixed (f02dedf1); F10 not reproduced; F12 Found; F13 leave (M13 survived, an equivalent mutant); F14 leave. Line numbers are at bbffe26c.
Mutation ids (M*n*) refer to the mutation table below; a finding marked
*suspected* is settled by its mutation.

| id | pt | site | class | what is wrong (evidence) | disposition |
|---|---|---|---|---|---|
| F1 | 2 | `diagrams/diagram.rs:690-716` | refusal no test reaches | The four-fermion refusals `ConvertError::MajoranaFermion { vertex }` and `FermionNumberViolation` have no test: #169 deleted the CLI refusal tests, and `grep -rn "MajoranaFermion\|FermionNumberViolation\|four-fermion vertex" --include=*.rs` finds only the definitions and the two `return Err` sites. Mutation M6 (`fermions <= 2` → `<= 99`, never refuse) is suspected to survive. | fix: a test per variant in `tests/majorana_flow.rs`, on the toy models with a four-fermion vertex appended |
| F2 | 2 | `diagrams/chain.rs:509-518` | refusal no test reaches | The decay-chain Majorana refusal has no test: `grep -rn "decay-chain diagram carries" --include=*.rs` returns only its own line. M7 (`if false && …`) suspected to survive. | fix: a test in `tests/majorana_flow.rs` |
| F3 | 2 | `ufo/lorentz.rs:743-783` | test blind to a branch | `charge_conjugation_parity_of_two_fermion_structures` has no `Sigma` structure (M4: `σ` conjugation `(-l,-r)` → `(l,r)`), and every chirality it places left of a gamma is in a combination symmetric under `c_L ↔ c_R`, so the left-of-gamma swap is unobserved (M5: drop the swap). Both suspected to survive. | fix: add `σ^{μν}` (odd) and a mixed left/right-of-gamma chirality case |
| F4 | 2 | `ufo/particles.rs:621-625` | test blind to a branch | The Goldstone-spelling test sets only `= True`; M2 (`== Some(true)` → `.is_some()`, a `= False` flag marks a Goldstone) suspected to survive. | fix: assert a `= False` flag in every spelling leaves the particle physical |
| F5 | 2 | `ufo/particles.rs:628-643` | test blind to a branch | The dict propagator case `{0: Prop.U, 1: Prop.F}` puts key 0 first, so taking the first value regardless of key (M3) is suspected to survive. | fix: add `{1: Prop.F, 0: Prop.U}` |
| F6 | 1 | `tests/toy_models.rs:850-853` | comment breaks AGENTS.md | `/// The parser at a0a6eb4 (vibegraph-lib/src/ufo/particles.rs lines 233-235) reads only goldstoneboson and goldstone, so the GoldstoneBoson case fails there` describes a past commit's code. | fix: drop the history paragraph |
| F7 | 1 | `diagrams/diagram.rs:626-629` | doc does not match body | `fermion_flow_summary` says each line prints "its propagator count"; the `format!("{}{}~{}{} [{}]", …)` at 641-648 prints ends and vertices only. | fix: doc to the body |
| F8 | 3 | `helas/eval/diagram_eval.rs:203-233` | production contract on a test-only item | `VertexTerm::from_ufo` is `#[cfg(test)]` and carries the production doc ("Generate a VertexTerm from a UFO vertex definition …"); the production `from_ufo_read` is documented as "[`from_ufo`](Self::from_ufo), with …", a link to an item absent from a non-test build. | fix: the contract on `from_ufo_read`, the test wrapper's doc pointing to it |
| F9 | 1/4 | `diagrams/diagram.rs:666-669, 699-702` | duplicate within the module | The Majorana predicate `p.spin.abs() == 2 && p.is_self_conjugate` is written twice in `diagram.rs` (`violates_fermion_flow`, `check_fermion_flow`); the same expression is in `ufo/bind.rs:252` and `ufo/report.rs:281`. | fix: `Particle::is_majorana` used at the two `diagram.rs` sites; bind/report to Found |
| F10 | 2 | `tests/standalone_jamps.rs:1667` | `catch_unwind` discards its payload | `probe_graph_ratios` prints "the probe panicked" without the panic message. | not reproduced: `catch_unwind` does not suppress the panic hook, which prints the message to stderr, and the probe runs under `--nocapture`; nothing is lost |
| F11 | 1 | `ufo/lorentz.rs:122-246` | function past 100 lines | `charge_conjugation_parity` is 125 lines; the chain walk (bra → ket over the spinor operators' `i`/`j`) is a separable unit and matches the six spinor variants twice. | fix: extract the chain walk; identity oracle is the unit test plus the enumerate/σ oracles |
| F12 | 2 | `helas/eval/diagram_eval.rs:421-422` | hermetic blind spot | `conjugate_standalone_projectors` (the chirality exchange) is pinned only by the MSSM rows, which are `extended-validation`: the toy Majorana and diquark models carry `Identity(2,1)` scalar vertices, whose exchange is the identity. M9 (`swap = false`) is suspected to survive the hermetic suite. A hermetic chiral row needs a new MadGraph table. | Found (validation-dev) |
| F13 | 2 | `diagrams/diagram.rs:620-621` | no mutation reaches | `orient_fermion_lines` picks which end to flip; by Denner–Eck–Hahn–Küblbeck the amplitude does not depend on the choice, so M13 (`flip = b` always) is expected to be an equivalent mutant: no amplitude test can see it. | leave, reported as unreachable if M13 survives |
| F14 | 1 | `tests/standalone_jamps.rs:1675-1781` | function past 100 lines | `probe_one` is ~110 lines. It is an `#[ignore]`d diagnosis, read top to bottom; splitting it buys nothing. | leave |
| F15 | 3 | `helas/eval/diagram_eval.rs:233` | `pub(crate)` with no user outside its module | `grep -rn from_ufo_read vibegraph-lib/src` finds it only in `diagram_eval.rs`. Added after the findings commit, while fixing F8. | fix: private |

Backlog check: `grep -rln "particles.rs\|/lorentz.rs\|/diagram.rs\|diagram_eval.rs\|majorana_flow\|toy_models.rs\|chain.rs" research/kb/backlog` lists 13 items. None is a finding of this pass: `hygiene/diagram-count-test-helper-duplicated`
names `toy_models.rs::diagram_count` (unchanged here), `hygiene/comment-leftovers-after-hygiene-sprint`
a history comment in `ufo/lorentz.rs` outside `charge_conjugation_parity`, and the others name
other functions of the same files.

## What changed

Commit f02dedf1, behaviour unchanged (identity oracle below).

- **F1, F2** `vibegraph-lib/tests/majorana_flow.rs`: three tests on toy models
  loaded from a scratch copy with lines appended (`toy_with`).
  `a_majorana_fermion_on_a_four_fermion_vertex_is_refused` appends a contact
  `FFFF1 = Identity(2,1)*Identity(4,3)` vertex `nt nt lt~ lt` to the Majorana
  model and matches the `MajoranaFermion` message naming `nt` and the vertex;
  `a_four_fermion_vertex_pairing_two_particles_is_refused` appends
  `at bt at~ bt~` with the same structure to the diquark model (total fermion
  number kept, the pairing `(at, bt)` at fault) and matches the
  `FermionNumberViolation` message; `a_decay_chain_through_a_majorana_fermion_is_refused`
  generates `lt~ lt > sc sc~` and matches `DiagramError::DecayChain` for
  `lt~ lt > sc sc~, sc > lt nt`.
- **F3** `ufo/lorentz.rs` test: `σ^{μν} p_ν` → `Some(-1)`;
  `γ^μ P_L + P_R γ^μ` → `None` (one chirality); `γ^μ P_L + P_L γ^μ` → `Some(-1)`
  (the vector current). Physics identities, no computed number moves.
- **F4, F5** `ufo/particles.rs` tests: `= False` in every spelling leaves the
  particle physical; `{1: Prop.F, 0: Prop.U}` reads `U`.
- **F6** `tests/toy_models.rs`: the past-commit paragraph dropped; "removes
  exactly those diagrams" (the test asserts `<`, not equality) now reads
  "removes diagrams and keeps the vector's".
- **F7** `fermion_flow_summary`'s doc to its body.
- **F8, F15** `diagram_eval.rs`: `from_ufo_read` carries the contract and is
  private; the `cfg(test)` `from_ufo` points to it.
- **F9** `Particle::is_majorana` (`pub(crate)`), used at the three predicate
  sites of `diagram.rs` (the third, `orient_fermion_lines`, read only
  `is_self_conjugate` on a line end, always a fermion).
- **F11** `LorentzOp::spinor_pair` (moved from a free function in
  `helas/eval/root_lorentz.rs`, five call sites there) and
  `LorentzTerm::spinor_chain`; `charge_conjugation_parity` goes from 125 to 85
  lines and loses its `unreachable!`.

Tests added: 3 (`majorana_flow.rs`); none deleted. Assertions added inside
three existing unit tests.

## Mutations

Driver: `scratchpad/lz2_mut.py` (apply one string replacement, run each command
under `flock /home/user/wt/heavy.lock`, revert, log), in a separate worktree
`/home/user/wt/lz2-mut` with its own target dir; "old" at 40c43387 (= main's
code), "new" at f02dedf1. Logs `/home/user/wt/logs/lz2-mut-{old,new}*.log`.

| id | site | mutation | guarding test(s) | old tests (40c43387) | new tests (f02dedf1) |
|---|---|---|---|---|---|
| M1 | `particles.rs:236` | drop `"GoldstoneBoson"` from the spellings | particles unit test; `toy_models` goldstone pair | caught (all three) | — |
| M2 | `particles.rs:238` | `== Some(true)` → `.is_some()` | particles unit test | **survived** | caught |
| M3 | `particles.rs:377` | dict: take any key, not 0 | propagator unit test | **survived** | caught |
| M4 | `lorentz.rs` | `σ` conjugation `(-l,-r)` → `(l,r)` | `charge_conjugation_parity_of_two_fermion_structures` | **survived** | caught |
| M5 | `lorentz.rs` | drop the left-of-gamma chirality swap | same | **survived** | caught |
| M6 | `diagram.rs:696` | `fermions <= 2` → `<= 99` (no four-fermion refusal) | `majorana_flow` | not run (no test) | caught (both new tests) |
| M6b | `diagram.rs:706` | `FermionNumberViolation` never returned | `majorana_flow` | not run (no test) | caught |
| M7 | `chain.rs:509` | `if false && glued.violates_fermion_flow(model)` | `majorana_flow` | not run (no test) | caught |
| M8 | `diagram.rs:847` | drop `reoriented` from the pairing sign | hermetic `standalone_jamps` | caught (`ab_to_ab_toy_fnv`, `ntl_to_ntl_toy_majorana`) | — |
| M9 | `diagram_eval.rs:422` | `swap = false` (no chirality exchange) | `standalone_jamps` | **survived** hermetic; caught by `--features extended-validation … ttx_to_hphm_mssm` | — (Found 2) |
| M10 | `diagram_eval.rs:255` | skip `split_chained_gamma5` | hermetic `standalone_jamps` | caught (4 Majorana rows) | — |
| M11 | `diagram_eval.rs:388` | `crossed = !at_named_slot` | hermetic `standalone_jamps` | caught (many rows) | — |
| M12 | `diagram.rs:728` | refuse only `None` parity, not `-1` | `a_majorana_pair_vertex_odd_under_conjugation_is_refused` | caught | — |
| M13 | `diagram.rs:621` | always flip the higher-index end | `standalone_jamps`, `majorana_flow` | **survived**: equivalent mutant (below) | — |
| M14 | `diagram.rs:696` | `fermions <= 2` → `< 2` (refuse 3-point Majorana vertices) | `cli_fermion_flow` | caught (3 tests) | — |
| M15 | `particles.rs` | list propagator: `last()` for `first()` | propagator unit test | caught | — |

Each new-test row also passes unmutated (Gates). Under M1–M5 the unit tests
ran with `--lib ufo::particles` / `--lib ufo::lorentz`; under M6–M7 the
target `--test majorana_flow` with no filter.

M6 and M7 were not run against the old tests: no test in the workspace names
either message (`grep` in F1, F2), so nothing could fail.

## Identity oracle

Main 8b3af5ab built in `/home/user/wt/lz2-main-target` (release), branch at the
fixed code (`v0.2.2-29-g40c43387-dirty`, the tree of f02dedf1) in its own
target; driver `/home/user/wt/lz2-oracle/oracle.sh`, run from the branch
worktree, `LHAPDF_DATA_PATH=validation/pdf`, `integrate <card> --run-card <rc>
--fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`:

| row | proc / run card | σ main | σ branch | grid.bin.zst sha256 (main = branch, `cmp` identical) |
|---|---|---|---|---|
| `dy13_default` | `dy13_proc_card.dat` / `dy13_default_run_card.dat` | 935 ± 11 pb | 935 ± 11 pb | 0642e8166775ce67… |
| `pp_to_llj_mlm` | `generate p p > e+ e- j` / `pp_to_llj_mlm_run_card.dat` | 267.0 ± 2.5 pb | 267.0 ± 2.5 pb | b6f5d6a22443490… |
| `ttx_to_hphm_mssm` | `import model MSSM_SLHA2; generate t t~ > h+ h-`, `--ufo-dir research/refs/mg5amcnlo/models` / `config_share_ee_mumu_run_card.dat` with `ebeam1 = ebeam2 = 750` (the row's √s = 1500, `lpp = 0`) | 1.1543 ± 0.0020 fb | 1.1543 ± 0.0020 fb | cf3e1b05188b3bb… |

`enumerate <card> --ufo-dir <copies> -j 1`: `diagrams.bin.zst`,
`diagrams.json`, `param_card.dat` and every `svg/` file `cmp`-identical on
`lt~ lt > nt nt` (the `majorana_flow.rs` process, 3 diagrams), `lt~ lt > nt nt vt`
(12), `at bt > at bt` (diquark, 2) and `lt nt > lt nt` in the toy model with
`toy_models.rs`'s charged Goldstone appended under `GoldstoneBoson=True` (1
diagram: the vector's; the filter is live). `parameters.json` differs only in
its `version` line. The MSSM row exercises the chirality exchange through the
CLI; the refactored `charge_conjugation_parity` is exercised by the enumerations
of the Majorana processes (each `nt nt vt` vertex goes through it).

Main's target dir `/home/user/wt/lz2-main-target`, its source worktree and the mutation worktree were deleted after the runs.

## What each test cannot detect

- **`conjugate_standalone_projectors`** (the chirality exchange): hermetically,
  nothing. M9 survives the hermetic `standalone_jamps` rows because the toy
  scalar vertices are `Identity(2,1)`, whose exchange is the identity; only the
  `extended-validation` MSSM rows see it (`ttx_to_hphm_mssm` caught M9). Per
  diagram it is pinned against MadGraph's `AMP()`, so a global phase common to
  every diagram of a row is invisible, as is a chirality error on a structure no
  row carries (a bare `Gamma5` on a non-self-conjugate scalar vertex is one).
- **`orient_fermion_lines`' choice of end** (M13, survived): equivalent mutant.
  The amplitude does not depend on which end is flipped (Denner–Eck–Hahn–Küblbeck),
  and `fermion_pairing_sign`'s `reoriented` count compensates, so no amplitude
  test can see the choice; M8 shows the compensation itself is pinned.
- **`charge_conjugation_parity`**: the unit test pins the table it encodes
  (`γ`, `σ` odd; `γγ⁵`, `1`, `γ⁵`, `P_χ` even; the left-of-gamma chirality
  swap). It cannot see a wrong entry shared by the doc and the table (the
  physics is the textbook `CΓᵀC⁻¹` list), nor structures with two gammas, which
  it returns `None` for and the generator therefore refuses on a Majorana pair.
- **The four-fermion refusals**: each new test matches its own variant's text.
  They cannot see a four-fermion Majorana vertex that ought to be refused but
  whose pairing groups differently (one structure, one pairing tested).
- **The decay-chain refusal**: one chain through a Majorana propagator in the
  core; a fermion-number-violating vertex inside a decay is the same branch
  (`violates_fermion_flow`) and is not separately tested.
- **Goldstone filter**: `<` against the unflagged count, plus `flagged > 0`.
  Every filter is all-or-nothing on the particle (`ufo/topo.rs:134`), so `<`
  sees any spelling the parser misses; it would not see a filter that also
  dropped an unrelated particle's diagrams while keeping one.
- **Propagator parsing**: a dict whose key 0 is absent returns `None` (no
  propagator); untested, and MadGraph's `value[0]` would raise there.

## Gates

`CARGO_TARGET_DIR=/home/user/wt/sprintl-hyg-fermion/target` with the lean cargo
variables, each under the heavy lock (driver `/home/user/wt/lz2-oracle/gates.sh`,
logs `/home/user/wt/logs/lz2-gate-*.log`):

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished`, rc 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished`, rc 0.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: rc 0, 13
  warnings, all in files this pass does not touch (`coupling/alphas.rs`,
  `coupling/cluster/configs.rs`, `coupling/scales.rs`, `diagrams/squared.rs`,
  `helas/eval/egraph.rs`, `helas/eval/lower.rs`, `lhef/mod.rs`,
  `phasespace/channel.rs`); none new. `vibegraph` (CLI) is untouched.
- `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test standalone_jamps`
  (the Majorana, diquark and MSSM rows): 94 passed, 1 ignored.
- `… --test majorana_flow --test toy_models`: 4 and 19 passed.
- `… --lib ufo::` (lorentz, particles): 209 passed.
- Hermetic `cargo test --workspace` with `research/refs/mg5amcnlo`,
  `validation/madgraph/output` and `validation/pdf` moved to
  `/home/user/wt/lz2-hidden` (an `ls` of each failed during the run) and
  restored afterwards: 79 test binaries, 2355 passed, 0 failed, 28 ignored.
  Main's count was not run; the change adds 3 tests and deletes none.
- `python3 scripts/kb.py lint`: 0 errors, 7 warnings (pre-existing backlog items).

Not run: `pixi run validate` and any MadGraph run (CI's banked job, as the
brief says); seed sweeps.

## Found

1. **The Majorana predicate outside `diagram.rs`.** `spin.abs() == 2 &&
   is_self_conjugate` is written out at `ufo/bind.rs:252` and
   `ufo/report.rs:281` (both in the widths cluster's files). Closing it:
   `Particle::is_majorana` at both.
2. **No hermetic oracle for the chirality exchange** (validation-dev). Mutation
   M9 (`swap = false`) survives the hermetic suite; only the MSSM rows under
   `extended-validation` catch it. Closing it: a chiral scalar vertex
   (`ProjM`/`ProjP` with different couplings, the `t~ b H+` shape) in a
   committed toy model and a banked `AMP()` table for it, so CI's `test` job
   sees the exchange.
3. **A dict `propagator` without key 0** is read as no propagator, silently;
   MadGraph's `value[0]` raises there. Closing it: a named refusal in
   `propagator_ref`, with a test.

## Brief corrections

- The brief lists `ufo/lorentz.rs` and `diagram_eval.rs` but not
  `helas/eval/root_lorentz.rs`; F11 edits it (the copy of `spinor_pair` moved
  onto `LorentzOp`). No sibling cluster's PR touched that file.
- `ttx_to_hphm_mssm` is a standalone `AMP()` row, not an integrate row: it has
  no run card. The σ oracle for it used a fixed-beam card at the row's √s
  (above).
