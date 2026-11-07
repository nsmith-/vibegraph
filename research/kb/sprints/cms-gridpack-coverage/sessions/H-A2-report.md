---
type: Session Report
title: "H-A2 report: every all-boson vertex of self-conjugate legs takes (−1)^{n_P} against MadGraph"
description: "The momentum sign MadGraph's all-outgoing reading puts on a vertex against vibegraph's all-incoming one was applied only to coloured three-vector vertices. It now applies to every all-boson vertex whose legs are all self-conjugate: −1 iff its Lorentz terms carry an odd number of momenta; a vertex whose terms disagree on the parity is refused. Four new standalone JAMP rows (O_HG's and O_HGtil's Higgs-three-gluon contacts, O_HG's Higgs-four-gluon contact, O_G's five-gluon contact) agree with MadGraph to ≤ 1.2e-15 and all fail with the three-leg restriction back. σ, grids and events of three cards without such a vertex are bit for bit. TCP_m50/m55 over five seeds: σ/MG 1.007 ± 0.010 and 1.025 ± 0.008, identical to the last bit to the diagnostic trial; the m55 2.7σ residual is not explained here."
status: draft
tags: [amplitudes, signs, vector-vertex, smeft, alp, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-07}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: aloha-p, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_writers.py#L124", title: "ALOHA aloha_writers.py get_momentum_conservation_sign, the routines' momentum convention"}
---
Session H-A2 of sprint H: the production fix for issue A of the TCP diagnosis
(the missing MadGraph↔vibegraph sign on all-boson contact vertices). Worktree
`/tmp/claude-1000/-home-ncsmith-vibegraph/e1ba4216-b5e6-4fbe-a71f-c293f1c32330/scratchpad/wt-signfix`,
branch `sprinth-contact-vertex-sign` off `9d1e9e6f`, verified before any action:

```
$ cd …/wt-signfix && git rev-parse --show-toplevel && git branch --show-current && git log -1 --format=%h
/tmp/claude-1000/-home-ncsmith-vibegraph/e1ba4216-b5e6-4fbe-a71f-c293f1c32330/scratchpad/wt-signfix
sprinth-contact-vertex-sign
9d1e9e6f
```

`$S` below is `/tmp/claude-1000/-home-ncsmith-vibegraph/e1ba4216-b5e6-4fbe-a71f-c293f1c32330/scratchpad/signfix`,
where every log and scratch artefact of this session lives (`signfix-*.log`).

## 1. The rule and the change

**Derivation.** HELAS stores every external momentum outgoing: `vxxxxx`
writes `nsv·p` with `nsv = −1` on an initial leg
(`aloha/template_files/aloha_functions.f:798-873` at `b7687064`; the generated
`matrix.f` calls `VXXXXX(P(0,1),…,-1*IC(1),…)`), so ALOHA reads every momentum
at a vertex outgoing. The kernels
here read every momentum incoming: an off-shell leg's `P` is minus the sum of
the inputs (`LorentzEvalNode::POut`). So each `P` of a Lorentz term is minus
MadGraph's, and a term with `n_P` momenta is `(−1)^{n_P}` times MadGraph's. The
incoming reading also puts each leg in its antiparticle's slot of the UFO
vertex, which multiplies the structure by its parity `ε_slot` under that
exchange. Per vertex, vibegraph reads `(−1)^{n_P} · ε_slot` times MadGraph's
`COUP · L`.

- All legs self-conjugate: nothing is exchanged, `ε_slot = +1`, the factor is
  `(−1)^{n_P}`.
- A conjugate pair (`γW⁺W⁻`, `ZW⁺W⁻`, SMEFTsim's W-pair structures): the kernel
  already evaluates the exchanged structure, and `ε_slot = (−1)^{n_P}` on every
  such structure the gated rows reach, so no factor.
- In a Lorentz-scalar term of an all-boson vertex, every vector leg and every
  `P` carries one free index, and `Metric`/`Epsilon` contract an even number.
  So `n_P` has the parity of the number of vector legs and is the same in every
  term. A vertex whose terms disagree is not such a scalar; it is refused.

**Colour.** The rule involves no colour. The old code required a coloured
vertex; its own docstring already said a colourless three-vector vertex of
self-conjugate legs "would take it too". The new predicate drops the colour
test. No gated model has a colourless all-self-conjugate odd-`n_P` vertex in
unitary gauge (SM, SMEFTsim; an `A Z h`-type vertex of a 2HDM would be one), so
this choice is derived, not measured. See Found.

**Code** (`vibegraph-lib/src/helas/eval/root_diagram.rs`):

- `is_yang_mills_vvv` + `is_coloured` → `momentum_parity_sign(model, vertex) ->
  Result<i8, RootLorentzError>`: `+1` unless every leg is a boson
  (odd spin code) and self-conjugate; then `−1` iff every term's `P` count is
  odd. Mixed parity → `RootLorentzError::UnsupportedVertex` naming the vertex.
  The existing refusal path is a typed error, so no panic was added.
- `triple_gluon_sign(diagram, model) -> i8` → `boson_momentum_sign(diagram,
  model) -> Result<i8, CompileError>`, the product over vertices. It feeds
  `convention_sign` with `?`.
- The derivation is in `boson_momentum_sign`'s docstring. The refusal is
  documented on `momentum_parity_sign`.
- `triple_gluon.rs` (the per-factor pins against `gg_to_ttx`/`ee_to_wpwm`)
  calls the renamed function.
- Fermion vertices are untouched.

**Brief correction (`n_P` per term).** The brief described hggg as "one P" and
hgggg as "even P, f·f". SMEFTsim's Higgs-three-gluon vertex mixes terms with 1
and 3 momenta, and its Higgs-four-gluon vertex mixes terms with 0 and 2. The
loop-induced `gHgg2…4` structures sit on the same vertex as O_HG's. The parity
is uniform, the count is not. The first unit tests asserted exact counts and
failed on this (`$S/signfix` log of the first `root_diagram::tests` run). They
now assert parity.

## 2. Unit tests (hermetic, `root_diagram::tests`)

| test | vertex class | expected |
|---|---|---|
| `the_triple_gluon_vertex_takes_minus_one` | SM `ggg` | −1, odd |
| `a_charged_triple_vector_vertex_takes_none` | SM `γW⁺W⁻`, `ZW⁺W⁻` | +1, odd, not self-conjugate |
| `an_odd_momentum_contact_takes_minus_one` | SMEFTsim `Hggg` under `vg_cHG` and `vg_cHGtil`; `ggggg` under `vg_cG` | −1, odd |
| `an_even_momentum_vertex_takes_none` | `Hgggg` and `Hgg` (`vg_cHG`), `gggg` (SM and `vg_cG`, the latter asserted momentum-bearing) | +1, even |
| `a_vertex_of_mixed_momentum_parity_is_refused` | SM `ggg` with a `Metric(1,2)` term appended | `UnsupportedVertex` naming the vertex, and `compile_single_diagram` on `g g > t t~` returns `CompileError::RootVertex` |

Every check panics if the model has no vertex with the named legs, so none can
pass vacuously. The old `a_momentum_bearing_vector_contact_is_not_a_triple_vector_vertex`
asserted the reverse of the rule (that a momentum-bearing contact of four or
five vectors takes no sign). It is replaced by the two contact tests.

**Non-vacuity.** Two mutations, each run then reverted (the file was restored
from a copy and checked with `diff`):

1. The three-leg restriction reinstated (`|| def.particles.len() != 3` on the
   early return). `$S/signfix-mutation.log`:

```
$ cargo test -j 2 -p vibegraph-lib --lib -- root_diagram::tests triple_gluon
test helas::eval::root_diagram::tests::an_odd_momentum_contact_takes_minus_one ... FAILED
test result: FAILED. 14 passed; 1 failed; 1 ignored; …
$ cargo test -j 2 -p vibegraph-lib --test standalone_jamps
test gg_to_hg_chg ... FAILED
test gg_to_hgg_chg ... FAILED
test gg_to_hg_chgtil ... FAILED
test gg_to_ggg_cg ... FAILED
test result: FAILED. 50 passed; 4 failed; …
```

2. The self-conjugate requirement dropped. `$S/signfix-mutation2.log`:

```
test helas::eval::root_diagram::tests::a_charged_triple_vector_vertex_takes_none ... FAILED
test helas::eval::triple_gluon::a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel ... FAILED
test result: FAILED. 13 passed; 2 failed; …
test aem_to_wmve / ee_to_wpwmz / wmwp_to_emep / wpem_to_zve / wpwm_to_epem / wpwm_to_epemz /
     wpwm_to_uux / wpwm_to_uuxg / ee_to_wpwmwpwm_cw ... FAILED
test result: FAILED. 45 passed; 9 failed; …
```

**Blind spot.** Reinstating the colour test would fail nothing. Every
odd-`n_P` all-self-conjugate vertex in the gated models is coloured.

## 3. Banked oracle rows (standalone JAMPs)

Generated with `gen_standalone_jamps.py` on the pinned MadGraph from the pixi
`madgraph` environment. `LDFLAGS` had `-Wl,--as-needed` stripped and
`-lstdc++` added. The SMEFTsim model was staged the way `build.sh`'s
`stage_models` stages it (copy, the repository's restrict cards, `convert
model`) into `$S/models`, with `VG_MODELS_DIR` pointing there (`$S/mg_rows.sh`).

| row | process (model card) | what it pins | flow entries | worst flow | worst \|M\|² | G |
|---|---|---|--:|--:|--:|---|
| `gg_to_hg_chg` | `g g > h g NP<=1` (`vg_cHG`) | O_HG `Hggg` contact (odd, −1) beside `Hgg`+`ggg` exchanges | 48 | 1.164e-15 | 7.814e-16 | −i |
| `gg_to_hg_chgtil` | same (`vg_cHGtil`) | O_HGtil's `Epsilon` `Hggg` contact (odd, −1): the a·GG̃ structure of an ALP | 48 | 5.751e-16 | 5.840e-16 | −i |
| `gg_to_hgg_chg` | `g g > h g g NP<=1` (`vg_cHG`), zero widths | O_HG `Hgggg` contact (even, +1) beside `Hggg` and the exchanges | 432 | 7.332e-16 | 1.135e-15 | +i |
| `gg_to_ggg_cg` | `g g > g g g NP<=1` (`vg_cG`), zero widths | O_G `ggggg` contact (odd, −1) beside its `gggg` (even) and `ggg` (odd) | 4224 | 7.377e-16 | 4.134e-16 | −i |

(`$S/signfix-standalone2.log`.) They are enforced at `REL_TOL = 1e-12` as
hermetic tests in `tests/standalone_jamps.rs`. The tables carry JAMPs, not
AMP(): `bank_amps` is single-flow only and these rows have 2, 9 and 44
flows. The per-diagram AMP() were dumped for diagnosis (`$S/amps_*.json`).
A sign on one diagram class is O(1) at the JAMP level, so the flow comparison
sees exactly the error class at issue. It cannot see a sign shared by every
diagram of the process (the fitted `G`).

**Brief correction (rows).** The brief asked for "per-diagram AMP() rows".
The banked rows are per-flow JAMP rows, because the generator banks AMP() only
for single-flow processes. The O_G five-gluon row was not in the brief. It was
added because the new rule flips that contact's sign, and an untested flip is
a claim.

**Zero widths, and why.** On the first generation `gg_to_hgg_chg` and
`gg_to_ggg_cg` agreed to 1.3e-9 and 2.4e-11, not to 1e-15. Probes located the
residual:

- `NP<=1 SMHLOOP=0` gives 1.1e-15 and 7.4e-16.
- `NP=0` keeps the residual: hgg 1.3e-10, ggg 2.4e-11.
- Raising MT tenfold leaves the hgg residual in place (1.9e-10), so it is not
  in the `1/MT²` structures.
- The vibegraph diagram census shows 15 and 50 diagrams with an internal Higgs
  (two loop-induced Higgs–gluon vertices joined by a Higgs line). The 2 → 2 rows,
  which passed at 1e-15, have none.

Standalone output keeps the Higgs width on a spacelike line, while vibegraph
zeroes it as MadEvent does. `zero_widths=True` exists for exactly this. With it,
both rows agree to ≤ 1.2e-15. The probe rows were removed before the commit. The residual is
`Γ_H/M_H`-sized (~3e-5 relative on the Higgs-exchange diagrams), not a sign.

**ALP UFO.** Not vendored. `SM_alp_UFO` (FeynRules 2.3.26 output, 2017) ships
no licence file and no licence text in any file, only `__author__` lines of
the generic UFO library files. Unlicensed code cannot be redistributed, so it
cannot go under `validation/ufo/`. `gg_to_hg_chgtil` covers the same structure
in a model that can be vendored. The ALP's a·GG̃ contact and O_HGtil's h·GG̃
contact are one Lorentz-and-colour structure (a singlet scalar times
`Epsilon`·`f` with one momentum). The diagnosis session's ALP tables
(`…/diag/alp/*.json`) agreed under the trial predicate, which equals this
rule on every SM_alp vertex: the TCP σ below is bit-identical between them.

## 4. Bit for bit on cards without such a vertex

The baseline binary is `…/diag/alp/target/release/vibegraph`, `9d1e9e6f` plus
the env-gated `ALP_TRIAL` branch. `ALP_TRIAL` was unset (`run.sh` does `unset
ALP_TRIAL`), so it is `9d1e9e6f`'s logic; this was not rebuilt by stashing. The
branch binary is `cargo build --release` of this branch. Each card ran `integrate
--neval 4000 --niter 3 --fixed-budget --seed 7 -j 2`, then `generate --seed 11
--nevents 500` (`$S/bitexact/run.sh`):

| card | σ base (pb) | σ branch (pb) | grid SHA-256 | events (`<init>` onward) |
|---|---|---|---|---|
| `p p > e+ e-` (sm, pp_to_ll run card at lhaid 247000) | 875.6963021431723 ± 25.237982057603123 | identical | `36bbad03…` both | identical (`e8ec9902…`) |
| `p p > j j j` (sm, pp_to_jj run card; `ggg` and `gggg`) | 67996958.5272029 ± 4657255.94350087 | identical | `fc3a40ee…` both | identical (`78ba935b…`) |
| `p p > e+ e- j j QCD=0 QED=4 $ h` (sm-no_b_mass, pp_to_epemjj_qcd0 run card; `WWγ`, `WWZ`) | 3.51138445554066 ± 0.15352591442423544 | identical | `c3d280f9…` both | identical (`88b30870…`) |

The only line in which the LHE files differ is line 5, which records the
artefact path (`base/…` against `branch/…`).

## 5. TCP over five seeds

The campaign's invocation (`campaign202/launchtrial.py` → `gpdb run`, read
from `vgrun.py`) was reproduced directly, outside the gridpack database:
`integrate proc_card.dat --run-card run_card.dat --neval 10000 --niter 3
--fixed-budget --seed N -j 4 --parallel-diagrams --output-ldjson --out …
--log-file … --no-network --ufo-dir models --param-card param_card.dat` with
`LHAPDF_DATA_PATH=/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current`. The
cards and staged model are copies of the campaign's adapted `attempt1`
directories (identical across seeds). As a check, the trial binary with
`ALP_TRIAL=1`, seed 1, m50, reproduces the campaign's 4.1272181155572504e-05
to the last digit (`$S/signfix-chain.log`).

Final code, no environment variable (`$S/tcp/branch/*/output.txt`):

| seed | m50 σ (pb) | σ/MG | m55 σ (pb) | σ/MG |
|---|---|---|---|---|
| 1 | 4.1272e-05 ± 9.20e-07 | 1.012 | 4.5634e-05 ± 1.25e-06 | 1.046 |
| 2 | 4.0377e-05 ± 1.07e-06 | 0.990 | 4.4223e-05 ± 1.33e-06 | 1.013 |
| 3 | 4.1630e-05 ± 1.08e-06 | 1.021 | 4.5506e-05 ± 1.28e-06 | 1.043 |
| 4 | 3.9945e-05 ± 8.78e-07 | 0.979 | 4.4021e-05 ± 1.09e-06 | 1.009 |
| 5 | 4.2201e-05 ± 1.12e-06 | 1.035 | 4.4201e-05 ± 9.89e-07 | 1.013 |
| mean ± sd/√5 | 4.1085e-05 ± 4.1e-07 | **1.007 ± 0.010** | 4.4717e-05 ± 3.5e-07 | **1.025 ± 0.008** |
| MG (README) | 4.079e-05 ± 1.216e-07 | | 4.364e-05 ± 1.933e-07 | |
| pull of the mean | +0.69 | | +2.69 | |
| χ²/dof of the seeds about their mean | 0.85 | | 0.43 | |

Every seed is bit-identical to the trial campaign `main-202a540-alptrial-s1…s5`
(`campaign202/runtrial.log`). The brief's trial numbers, 1.007 ± 0.010 and
1.025 ± 0.008, are reproduced exactly; before the fix σ/MG was 3.89 and 4.21.

**What this does not show.** m55 sits 2.5% high at 2.7σ of the five-seed
spread. Five seeds detect a missed region; they cannot calibrate a 2.5% offset.
Its seed χ²/dof of 0.43 is low enough to be a warning in itself. This session
did not diagnose it. See Found.

## 6. The fermion dipole: an open question

The naive form of the rule, `(−1)^{n_P}` per vertex, would also predict a
factor on a fermion vertex with one momentum, such as the `σ^{μν}q_ν` dipole
(SMEFTsim's `P(-1,3)·γ^{-1}γ^3` FFV structures under `vg_ctdipole`, the toy
`Sigma(3,-1,2,-2)*P(-1,3)` dipole). The dipole rows (`ee_to_ttx_dipole`,
`ll_to_qqx_toy_dipole`, and the `ctZ`/`ctA` chains of `ee_to_ttx_smeft`) agree with
MadGraph today with no such factor, and so the naive rule is contradicted
there. A candidate reading is that the fermion slots' `ε_slot` (the bilinear
reversal `CΓᵀC⁻¹`, which `vertex_root_signs` applies) supplies the
compensating sign, the way the W-pair slot exchange does for `γW⁺W⁻`. This
session did not establish it. Fermion vertices are left alone, and the
docstring claims only that "their slots' reading is the bilinear reversal of
`vertex_root_signs`".

## 7. Gates

```
$ cargo fmt --all --check
(exit 0)
$ cargo clippy -j 2 --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 39s            (exit 0)
$ cargo test -j 2 --workspace --no-fail-fast                           ($S/signfix-test.log)
suites 55 passed 1967 failed 10 ignored 25
  cli_first_run: an_unattended_run_refuses_the_download_and_names_the_consent_flag,
                 the_kill_switch_outranks_explicit_consent, the_refusal_names_the_directory_to_install_into
  cli_output_ldjson: refusals_carry_the_kind_of_what_failed
  cli_pdf_cache: an_unindexed_lhaid_is_refused_naming_the_id, a_checkouts_validation_pdf_is_not_a_fallback,
                 lhapath_is_read_only_when_lhapdf_data_path_is_unset, vibegraph_pdf_dir_resolves_nothing_and_draws_a_warning,
                 a_missing_pinned_set_reports_what_it_would_download,
                 the_no_network_variable_refuses_at_any_value_in_the_binary_and_the_fetch_scripts
```

All ten failures are environmental. They assert that a PDF set is missing,
and this host has `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current`
(`CVMFS_LHAPDF_SETS`), where the CLI finds it. With this branch's changes
stashed (baseline `9d1e9e6f` sources), the same three test binaries fail the
same ten tests (`$S/signfix-baseline-cli.log`: 3, 1 and 6 failures). Every
other suite passes, `standalone_jamps` (54) and `amplitude_oracle` among them.

**Banked layer.** `validation/validate.sh` was run directly, which is what
`pixi run --skip-deps validate` runs. It used main's pixi `default` environment
(shell-hook), `CARGO_BUILD_JOBS=2`, and an untracked `.cargo/config.toml` with
`[profile.release-debug] debug = 0`, deleted afterwards
(`$S/signfix-validate.log`):

```
$ bash validation/validate.sh
suites 85 passed 2359 failed 34 ignored 110
VALIDATE EXIT 101
!!! the report does not match validation/manifest.toml: 7 banked integrals/samples cells missing
    (pp_to_ll_beams_6500_4000 ×2, pp_to_ll_mz95, pp_to_ll_etau, pp_to_jj_ht, pp_to_bb_iht, ee_to_tth_ymt250)
amplitude_oracle: test result: ok. 51 passed; 0 failed
standalone_jamps: test result: ok. 54 passed; 0 failed
```

**The banked layer is red, and none of the failures involve amplitudes.**
Each of the 34 failures is one of these:

1. The ten CVMFS-LHAPDF CLI tests above, plus `generate_locates_the_recorded_pdf_set_by_name`
   ("a set the data path lacks was found"), same cause.
2. Reference data missing from this worktree's copied bundle: `seeded_sigma/*/seed_record.json`
   (pp_to_epemjj_qcd0, dy13_mmll_60_120_mz95, ee_to_tth_ymt250, dy13_etau_default),
   the banked runs pp_to_bb_iht, pp_to_jj_ht, bbx_to_tapta_ckm_no_b_mass,
   `dy13_beams_6500_4000/Events/…` and `pp_to_ll/Events/run_01_cms/…`. Each
   message is the `validation::require` "incomplete environment" refusal or a
   missing file. These account for the seven missing manifest cells.
3. `banked_run_logs_pin_the_alpha_s_source_rule` and
   `banked_events_reproduce_aqcdup` refuse `pp_to_ll`'s banked run card
   (`pdlabel = nn23lo1`). That is a run card from the copied bundle, which this
   change does not read.

None is in a gate this change can reach. The bundle in this worktree is
incomplete; a full `pixi run validate` on a fetched bundle is still owed (see
Found). The amplitude cells: 48 ✅, 2 ⚠️ (`ee_to_wpwm_cw`, `ee_to_zh_smeft`,
the registered informational deviations, not listed as mismatches), 11 —.

**Existing amplitude rows unchanged, measured.** `amplitude_oracle` and
`standalone_jamps` ran with `--nocapture --test-threads 1` on this branch and
on `9d1e9e6f`'s `vibegraph-lib` and generator (checked out over the tree, then
restored with `git checkout HEAD`; `$S/ampcmp.sh`). After dropping the four new
rows' lines, the printed output (every row's measured deviations, `G`, entry
counts) is identical line for line (`diff a.txt b.txt` shows only the test count,
50 against 54, and the four new `ok` lines).

## Commits

- `9e1e96f3` helas: sign every all-boson vertex of self-conjugate legs by its
  momentum parity (code, unit tests, four standalone tables and their generator rows)
- this report and the regenerated `sessions/index.md` (the branch tip)

## Found

- **m55 sits 2.5% high (2.7σ) after the fix.** TCP_m55 reads σ/MG 1.025 ± 0.008
  over five seeds, against m50's 1.007 ± 0.010, and the seed χ²/dof is 0.43.
  Evidence: §5. What would close it: a budget ladder with 20+ seeds on the two
  rungs that matter, and per-multiplicity σ against MadGraph's per-P-directory
  outputs (`…/diag/tcp`) to localise the excess to 1j or 2j.
- **Colourless odd-momentum vertices of self-conjugate legs are untested.** The
  rule now gives them −1 by derivation (e.g. a 2HDM `A Z h` vertex, or Feynman-
  gauge `G⁰ Z h`), but no gated model reaches one, so dropping the colour
  condition is unmeasured (§2, blind spot). What would close it: a toy-UFO
  vertex (authored, so no licence issue) with three self-conjugate colourless
  bosons and one momentum, and a standalone row.
- **`ε_slot = (−1)^{n_P}` for conjugate-pair vertices holds only where measured.**
  Every momentum-bearing W-pair structure the SM and SMEFTsim put on a gated row
  agrees, but no general argument guarantees that a structure is odd under the
  W-slot exchange exactly when `n_P` is odd. A BSM vertex with a conjugate pair
  and an even-under-exchange odd-`n_P` structure would be silently wrong. What
  would close it: compute `ε_slot` from the structure (apply the slot
  permutation to the parsed Lorentz expression and compare) and assert the
  identity at compile time, refusing otherwise.
- **The fermion-dipole open question (§6).** What would close it: derive
  `ε_slot` for `σ^{μν}`/`γ^μ` bilinears under the slot reversal, and show that
  `(−1)^{n_P} · ε_slot = +1` for every momentum-bearing fermion vertex the
  dipole rows reach (or find the counterexample).
- **Backlog items the rule touches** (for the manager; dev sessions do not edit
  items). `odd-gluon-vertex-sign-unmeasured`: its `closes_when` (a gated
  standalone row such as `g g > g g g NP<=1` under `vg_cG`, or `g g > g h` under
  `vg_cHG`) is met by `gg_to_ggg_cg` and `gg_to_hg_chg`.
  `colourless-self-conjugate-vvv-sign-unoracled`: still open, but its text
  ("gets none", `is_yang_mills_vvv && is_coloured`) describes the old code; the
  code now gives `−1` by derivation. `w-pair-vvv-slot-exchange-oddness-unpinned`
  and `code-docs-stale-after-root-free-convention` name `triple_gluon_sign`.
  The kb concept pages were brought in line in the follow-up commit on this
  branch.
- **A full banked run on a complete bundle is owed.** This worktree's copied
  `validation/madgraph/output` lacks several seed records and banked runs (§7),
  so `validate.sh` could not go green here. What would close it: `pixi run
  validate` (which fetches the bundle) on this branch in a fresh work area.
- **Eleven CLI tests fail on any host with the CVMFS LHAPDF tree.** `cli_first_run`,
  `cli_output_ldjson`, `cli_pdf_cache` and `cli_generate_proton`
  (`generate_locates_the_recorded_pdf_set_by_name`) assume a missing set stays missing,
  but `CVMFS_LHAPDF_SETS` is searched unconditionally. The first ten fail the
  same way on `9d1e9e6f` (measured); the eleventh, a banked test, was not rerun
  at the baseline. What would close it: let the tests point the CVMFS root at an
  empty directory (an environment override), or skip it under the hermetic
  layer.
- **The brief's "outcome.txt with MG σ" is vibegraph's σ.** The
  sprinth-diagnosis-cards `outcome.txt` files carry vibegraph's pre-fix σ
  (1.587e-4 / 1.837e-4 pb). MadGraph's σ is in that directory's `README.md`.
- **`build.sh` on Linux.** It was not needed: the rows were banked through
  `gen_standalone_jamps.py` directly with the brief's `LDFLAGS`. The brief's note
  that `build.sh` adds `-lc++` only on Darwin was not exercised.
