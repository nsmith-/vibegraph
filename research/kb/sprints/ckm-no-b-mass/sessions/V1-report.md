---
type: Session Report
title: "V1 report: ckm_no_b_mass blind spots closed from the card, and the banking plumbing"
description: "Three hermetic tests close MadGraph's zero threshold, coupling merging and the probe-completeness gap at this card; a fourth reads MadGraph's generated param cards; model = \"sm\" names an interned-SM restriction on both sides and gen_couplings honours VG_MG5_ROOT; the M3 banking request in final form."
status: draft
tags: [model, restriction, ckm, sm, cms, sprint-d, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: mg-restrict, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L2365", title: "MadGraph import_ufo.py RestrictModel (restrict_model, detect_identical_couplings, remove_interactions)"}
---
## Blind spots of F1's six tests

Read against MadGraph's `RestrictModel` at b7687064 (`import_ufo.py`
L2365–2900): `restrict_model` evaluates every coupling at the restrict card;
`detect_identical_couplings` treats `|g| < 1e-13` as zero unless some coupling
lies in `[1e-13, 1e-10)`, in which case it restarts treating only exact zeros as
zero; couplings equal up to sign after `limit_to_6_digit` rounding (at least ten
significant digits) and of equal orders are merged; `remove_interactions` drops
a vertex only when all its couplings vanish; `detect_special_parameters` locks
exact zeros (and exact ones); `detect_identical_parameters` merges externals of
equal value within one block (none here: MT/ymt and MTA/ymtau are in different
blocks). The SM has no conditional expressions, so
`detect_conditional_statements_simplifications` is inert. This crate prunes
`|g| < 1e-20` and merges nothing (`ufo/mod.rs` `is_zero_coupling`).

| F1 test | cannot detect | closed now? |
|---|---|---|
| `vendored_card_is_the_cms_genproductions_bytes` | whether CMS ran these bytes (88-gridpack survey is the evidence); whether the MadGraph copy used for banking carries the same bytes | the banking request checks the copy's sha256 |
| `card_is_restrict_ckm_with_b_massless_and_a_tau_width` | card entries no SM external names | no (none expected; MadGraph ignores them too) |
| `tau_width_survives_and_b_mass_locks` | MadGraph's own lock set: exact-zero rule read from source, never run | **after M3**: `interned_sm_defaults_are_madgraphs_generated_param_card` reads `DECAY 15` (2.27e-12, not 0) and `MASS 5` (0) from MadGraph's generated card |
| `couplings_compose_ckm_and_no_b_mass` | (a) a b- or CKM-dependent coupling the value probe misses; (b) anything this crate and its own `ckm` share wrongly | (a) **yes**, `value_probes_find_every_coupling_their_parameters_enter`; (b) after M3, `coupling_oracle` at this card |
| `vertices_compose_ckm_and_no_b_mass` | MadGraph's 1e-13 zero threshold; MadGraph's coupling merging; MadGraph's interaction split/count at this card | threshold **yes** (`no_coupling_sits_between_the_zero_thresholds`); merging **yes** (`couplings_madgraph_could_merge_are_identities`); count after M3 (`smeftsim::restricted_interaction_counts_match_madgraph`) |
| `madgraph_default_couplings_hold_outside_the_b_and_ckm_sectors` | the b and CKM sectors (no MadGraph reference at all); its sector exclusion inherits the probes' completeness | sectors after M3 (`coupling_oracle`); completeness **yes** |

What the three new composition-side tests found:

- **The value probe is incomplete, harmlessly.** Poisoning the Wolfenstein
  parameters with NaN reaches 30 couplings; the value probe finds 24. The six
  it misses (`GC_16 = -I2x12 = -yc·conj(CKM2x1)`, GC_18, GC_20, GC_22, GC_23,
  GC_24) carry `yc`, which every card the composition reads locks at zero.
  They are zero at any parameter point of this restriction and on no vertex,
  so F1's assertions about them are vacuous but not wrong. The b-sector probe
  is complete (8 = 8). The test asserts exactly this: probe ⊆ poisoned, and
  every coupling only poisoning reaches is zero with every free external
  moved and carried by no vertex of `ckm` or `ckm_no_b_mass`.
- **No coupling sits between the thresholds.** At the CMS card, 108
  couplings: 24 exactly zero, smallest non-zero 1.561e-3. So MadGraph's
  1e-13 rule (in either mode) and this crate's 1e-20 prune the same set.
- **Merging is value-preserving.** 35 coupling pairs agree to 1e-9 up to sign
  with equal orders at the card (a superset of MadGraph's merge pairs); every
  one holds to 1e-13 at a card moving every non-zero external off its value.
  So MadGraph's merged model and this crate's unmerged one agree at any param
  card, not just at the restrict card.

## What landed

1. `vibegraph-lib/tests/sm_ckm_no_b_mass.rs`: three tests above, a
   `moved_card` helper, a `poisoned` helper, and a corrected module doc (which
   two MadGraph behaviours are now closed from the card, and what is not).
2. **The manifest spelling.** `model = "sm"`, `restrict = "<suffix>"` means
   `import model sm-<suffix>` on both sides.
   - Rust: `tests/common/manifest.rs` gains `INTERNED_SM` and
     `RowModel::is_interned_sm`; `tests/common/mod.rs` splits
     `load_model_for_row` into a public `load_row_model(&RowModel)`, which
     resolves an interned-SM row through `SMRestrict::from_suffix` to the
     cached `sm_model(variant)` and refuses an unknown suffix by name. Every
     gate that loads a row model (`model_for_row`: amplitude, coupling,
     diagram, colour, samples, sigma, interaction-count) picks it up.
     `validate_madgraph_diagrams`'s script/manifest import comparison already
     formats `{dir}-{restrict}` = `sm-ckm_no_b_mass`, which is the script's
     `import model` line verbatim; `coupling_oracle`'s model check compares
     `("sm", "ckm_no_b_mass")` with what `gen_couplings.py` writes;
     `validate_scale_couplings` skips every row naming a model (correct: it
     runs the default SM). `validation-report` needs no change (it renders
     `sm-ckm_no_b_mass` beside the process); its doc says what `"sm"` means.
   - Python: `gen_couplings.model_import_path` returns `sm-<restrict>` for
     `model = "sm"` (and `sm` for none), with `INTERNED_SM` named.
     `extract_interactions.py` keys `("sm", restrict)` as
     `sm-ckm_no_b_mass` unchanged. `gen_amplitude.py` passes `model` through
     untouched; only its comment changed.
3. **The root override.** `gen_couplings.py`: `MG5_ROOT =
   os.environ.get("VG_MG5_ROOT", <submodule>)`, so its `import_ufo` /
   `model_reader` resolve `sm-ckm_no_b_mass` in the same copy
   `mg5_pinned.sh` generated from. **Correction to the brief and F1's
   report:** `gen_amplitude.py` and `gen_amplitude_tables.py` do not hard-code
   `MG5_ROOT` and import nothing from MadGraph (they load the compiled f2py
   modules from `output/f2py`), so they need no override. The other
   MG5_ROOT hard-coders (`decay_semianalytic.py`, `dump_sm_decay_widths.py`,
   `gen_decay_amplitudes.py`) read only `restrict_default` and are not part
   of this banking; I left them alone.
4. `vibegraph-lib/tests/interned_sm_rows.rs` (new): the spelling resolves
   (`no_b_mass`, default, `ckm_no_b_mass` → the identical cached `Arc`),
   a misspelt suffix is refused naming it and listing the variants, every
   manifest row naming the interned SM resolves (0 today), and
   `interned_sm_defaults_are_madgraphs_generated_param_card`: every
   interned-SM amplitude table's banked MadGraph `param_card.dat` agrees with
   the row's restricted defaults external by external, at the card's seven
   printed digits (5e-7 relative). Runs today on 26 default-restriction
   tables, 468 externals; `aS` is skipped because a hadronic run's card carries
   its PDF set's αs(M_Z) (`pp_to_ll_qcd0` reads 0.13 against the
   restriction's 0.118, found by this test's first run). After M3 it is what
   pins `WTau = 2.27e-12` surviving MadGraph's restriction.
5. `validation/madgraph/README.md` documents the spelling and the copy;
   `validation-report/src/manifest.rs` doc comments.

No manifest rows, scripts or references were added.

## Gate

All in `/home/user/wt/G` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
(log `/tmp/claude-0/-home-user-vibegraph/5fc875eb-a86f-52ce-bbc1-f353ce6c8fb0/scratchpad/ckmG-V1-gate.log`):

```
$ cargo fmt --all --check
fmt exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 09s
clippy exit 0
$ cargo clippy -p vibegraph-lib -p vibegraph --features vibegraph-lib/extended-validation --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 10s
exit 0          # compile check of the feature-gated gates that share tests/common
$ cargo test --workspace
test exit 0        # 42 test binaries, every `test result` line "0 failed"
  unittests src/lib.rs:                 ok. 1009 passed; 0 failed; 10 ignored
  unittests src/main.rs (cli):          ok. 106 passed; 0 failed
  tests/coupling_oracle.rs:             ok. 48 passed; 0 failed
  tests/amplitude_oracle.rs:            ok. 49 passed; 0 failed
  tests/validate_madgraph_diagrams.rs:  ok. 57 passed; 0 failed
  tests/sm_ckm_no_b_mass.rs:            ok. 9 passed; 0 failed
  tests/interned_sm_rows.rs:            ok. 4 passed; 0 failed
$ cargo test -p vibegraph-lib --test sm_ckm_no_b_mass --test interned_sm_rows -- --nocapture
b sector: probe 8 couplings, poisoning 8; missed, all locked at zero: {}
CKM sector: probe 24 couplings, poisoning 30; missed, all locked at zero: {"GC_16", "GC_18", "GC_20", "GC_22", "GC_23", "GC_24"}
108 couplings at the card: 24 exactly zero, smallest non-zero 1.561e-3
35 coupling pairs MadGraph could merge, every one an identity
25 tables, 49 couplings compared (worst 8.85e-15), 4 skipped as b- or CKM-dependent: {"GC_100", "GC_15", "GC_82", "GC_83"}
test result: ok. 9 passed; 0 failed
0 manifest rows name the interned SM
26 interned-SM tables, 468 externals compared: {"default": 26}
test result: ok. 4 passed; 0 failed
$ python3 -m py_compile validation/madgraph/gen_couplings.py validation/madgraph/gen_amplitude.py validation/madgraph/extract_interactions.py
compiled
$ python3 -I .../ckmG-V1-pycheck.py validation/madgraph/gen_couplings.py   # model_import_path extracted by AST, no MadGraph import
sm                                      # model None
sm                                      # model "sm", restrict None
sm-ckm_no_b_mass                        # model "sm", restrict "ckm_no_b_mass"
/OUT/models/vibegraph_toy_UFO-yukawa    # vendored UFO, unchanged
AssertionError: e: a restrict card without a model
$ python3 scripts/kb.py lint
826 concepts, 290 backlog items; 0 errors, 4 warnings   # the 4 warnings predate this session
```

The banked layer (`pixi run --skip-deps validate`) was not run: pixi is not
installed here and `validation/madgraph/output` is absent. The only behaviour
change in code a banked gate runs is `load_row_model`'s new interned-SM branch,
which no current row reaches (`0 manifest rows name the interned SM`); the
UFO-directory branch is the previous body moved unchanged.

## Mutation evidence

Driver `/tmp/claude-0/-home-user-vibegraph/5fc875eb-a86f-52ce-bbc1-f353ce6c8fb0/scratchpad/ckmG-V1-mutate.sh`,
log `ckmG-V1-mutate.log` beside it. Each mutation by `sed`, restored with `cp`
from a backup **and `touch`**, then the next built; baseline rerun at the end.

| mutation | failing tests (others pass) |
|---|---|
| M1 b-sector probe moves `MB` only, not `ymb` | value probes (`GC_…` missed by the probe and a vertex carries it), couplings compose, vertices compose, MadGraph anchor |
| M2 CMS card `ymb` 0 → 1e-12 | **zero thresholds** (b-sector couplings ~1e-17 survive this crate's 1e-20 prune, which MadGraph's 1e-13 would drop), card values, tau/b locks, couplings compose, vertices compose, vendored bytes |
| M3 merge-pair bound 1e-9 → 3e-2 (pairs non-identities) | merge identities (`… at the card but not off it`) |
| M4 interned-SM row always loads `Default` | `interned_sm_row_loads_the_named_restriction` |
| M5 interned-SM branch disabled (falls through to a `sm/` UFO directory) | resolution, unknown-suffix refusal |
| M6 `restrict_default.dat` `WH` ×10 | `interned_sm_defaults_are_madgraphs_generated_param_card` (`[uux_to_ccx_emmm_qcd0] WH defaults to 0.0638233934 … MadGraph's generated card says 0.006382339`) |
| baseline after restore | 9 passed, 4 passed |

## Banking request (M3, MadGraph 3.7.1 at b7687064)

Closes the item's `closes_when` (couplings and a census pinned against
MadGraph at this restriction). Needs the sprint branch at or after this
session's commit, the populated `research/refs/mg5amcnlo` submodule, and a
work area holding the pinned reference bundle (the `extract_*` scripts rewrite
whole committed files from `validation/madgraph/output/`, so a partial work
area would drop other rows).

### 1. The MadGraph copy

```sh
cd <repo>
WORK="$(mktemp -d)"
cp -R research/refs/mg5amcnlo "$WORK/mg5-ckmnob"
cp vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat "$WORK/mg5-ckmnob/models/sm/"
sha256sum "$WORK/mg5-ckmnob/models/sm/restrict_ckm_no_b_mass.dat"
#   must print ad6e3eaf03443bc14148d392fe98deb712de29166061a7408eff521a47591b05
export VG_MG5_ROOT="$WORK/mg5-ckmnob"
```

Never copy the card into the submodule itself: `sm_interned_blob` fails if
`research/refs/mg5amcnlo/models/sm/restrict_ckm_no_b_mass.dat` exists.

### 2. Two scripts

`validation/madgraph/scripts/ubx_to_epve_ckm_no_b_mass.mg5`:

```
# MadGraph5 batch script: u b~ > e+ ve in the Standard Model under
# restrict_ckm_no_b_mass.dat, the card CMS genproductions bundles in its copy of
# models/sm.
#
# Why this exact process:
#   - One s-channel W+ whose quark vertex carries V_ub, which carries the
#     Wolfenstein phase etaWS at leading order, on a massless b. The amplitude gate
#     fixes the process-wide constant to +-i, so a conjugated or rotated V_ub
#     fails this single diagram instead of hiding in a global phase.
#   - Its coupling table banks MadGraph's whole coupling_dict at this card, the
#     CKM couplings included, which no default-restriction table reaches.
#
# MadGraph does not ship this card: the run uses a copy of the pinned tree that
# carries it, named by VG_MG5_ROOT (see mg5_pinned.sh).
#
# `display interactions` prints the post-restriction interaction count into the
# build log, which is where extract_interactions.py reads it from.
import model sm-ckm_no_b_mass
display interactions
generate u b~ > e+ ve
output ubx_to_epve_ckm_no_b_mass -nojpeg
launch
set lpp1 0
set lpp2 0
set ebeam1 250
set ebeam2 250
set use_syst False
done
```

`validation/madgraph/scripts/bbx_to_tapta_ckm_no_b_mass.mg5`:

```
# MadGraph5 batch script: b b~ > ta+ ta- in the Standard Model under
# restrict_ckm_no_b_mass.dat, the card CMS genproductions bundles in its copy of
# models/sm.
#
# Why this exact process:
#   - ymb = 0 removes the h b b~ vertex, so the process is the photon and Z
#     exchanges alone: two diagrams, where the default restriction has three.
#     The diagram count is MadGraph's pruning of the b Yukawa at this card.
#   - Massless b spinors on the incoming legs, massive taus on the outgoing.
#
# MadGraph does not ship this card: the run uses a copy of the pinned tree that
# carries it, named by VG_MG5_ROOT (see mg5_pinned.sh).
#
# `display interactions` prints the post-restriction interaction count into the
# build log, which is where extract_interactions.py reads it from.
import model sm-ckm_no_b_mass
display interactions
generate b b~ > ta+ ta-
output bbx_to_tapta_ckm_no_b_mass -nojpeg
launch
set lpp1 0
set lpp2 0
set ebeam1 250
set ebeam2 250
set use_syst False
done
```

### 3. Two manifest rows

Append to `validation/manifest.toml` (single-channel section; seeds 277 and
281 are unused). Register the cells `info`, and flip them to `gate` with the
measured numbers in their notes once step 5 passes, in the same commit:

```toml
[[process]]
key = "ubx_to_epve_ckm_no_b_mass"
process = "u b~ > e+ ve"
model = "sm"
restrict = "ckm_no_b_mass"
script = "validation/madgraph/scripts/ubx_to_epve_ckm_no_b_mass.mg5"
class = "single-channel"
n_final = 2
bundled = false
rationale = "The Standard Model under the CMS genproductions card restrict_ckm_no_b_mass.dat, which MadGraph does not ship: a five-flavour CKM model with MB = ymb = 0 and the tau width kept. One s-channel W+ whose quark vertex carries V_ub, which carries the Wolfenstein phase at leading order, on a massless b; its coupling table is MadGraph's whole coupling_dict at this card."
artifacts = ["output/ubx_to_epve_ckm_no_b_mass", "output/ubx_to_epve_ckm_no_b_mass.json", "output/ubx_to_epve_ckm_no_b_mass_amplitude.csv", "amplitudes/ubx_to_epve_ckm_no_b_mass.json", "couplings/ubx_to_epve_ckm_no_b_mass.json"]
mg_amplitude = { process = "u b~ > e+ ve", pdgs_in = [2, -5], pdgs_out = [-11, 12], sqrt_s_list = [10.0, 80.0, 200.0, 500.0], npoints = 10, seed = 277 }
[process.categories]
diagrams = { tier = "hermetic", mode = "info", note = "1 diagram, the s-channel W+; none exists at the default restriction, whose CKM matrix is the identity" }
amplitudes = { tier = "hermetic", mode = "info", note = "<measured: |M|^2 max_rel, per-diagram, JAMP, fitted G>" }
integrals = { tier = "uncovered", note = "the launch banks a cross section that is not extracted: the row exists for the couplings and amplitudes at this restriction, and its one diagram has no tau propagator for the card's tau width to reach" }
samples = { tier = "uncovered", note = "the launch's events feed the amplitude table's event points only; no sample is banked for this restriction" }

[[process]]
key = "bbx_to_tapta_ckm_no_b_mass"
process = "b b~ > ta+ ta-"
model = "sm"
restrict = "ckm_no_b_mass"
script = "validation/madgraph/scripts/bbx_to_tapta_ckm_no_b_mass.mg5"
class = "single-channel"
n_final = 2
bundled = false
rationale = "The b sector of the CMS ckm_no_b_mass card: ymb = 0 removes the h b b~ vertex, so b b~ > ta+ ta- is the photon and Z exchanges alone, two diagrams where the default restriction has three, on massless b spinors."
artifacts = ["output/bbx_to_tapta_ckm_no_b_mass", "output/bbx_to_tapta_ckm_no_b_mass.json", "output/bbx_to_tapta_ckm_no_b_mass_amplitude.csv", "amplitudes/bbx_to_tapta_ckm_no_b_mass.json", "couplings/bbx_to_tapta_ckm_no_b_mass.json"]
mg_amplitude = { process = "b b~ > ta+ ta-", pdgs_in = [5, -5], pdgs_out = [-15, 15], sqrt_s_list = [10.0, 91.188, 200.0, 500.0], npoints = 10, seed = 281 }
[process.categories]
diagrams = { tier = "hermetic", mode = "info", note = "2 diagrams, photon and Z; the default restriction's third, the h exchange, is pruned with ymb" }
amplitudes = { tier = "hermetic", mode = "info", note = "<measured>" }
integrals = { tier = "uncovered", note = "the launch banks a cross section that is not extracted: the row exists for the vertex pruning and amplitudes at this restriction" }
samples = { tier = "uncovered", note = "the launch's events feed the amplitude table's event points only; no sample is banked for this restriction" }
```

### 4. Generators (madgraph environment, `VG_MG5_ROOT` exported)

Run the scripts directly, not the pixi tasks: the tasks' `depends-on` chains
rerun every row.

```sh
pixi run -e madgraph bash validation/madgraph/build.sh          # existing dirs are skipped; builds the two new ones
grep -h "Current model contains" validation/madgraph/output/*_ckm_no_b_mass/build.log   # expect 133 (this crate's count)
pixi run -e madgraph python validation/madgraph/extract_diagrams.py      # diagrams.json + output/<row>.json
pixi run -e madgraph python validation/madgraph/extract_interactions.py  # interactions.json gains "sm-ckm_no_b_mass"
pixi run -e madgraph python validation/madgraph/extract_configs.py       # configs.json
pixi run -e madgraph bash validation/madgraph/build_amplitude.sh ubx_to_epve_ckm_no_b_mass bbx_to_tapta_ckm_no_b_mass
pixi run -e madgraph python validation/madgraph/gen_amplitude.py ubx_to_epve_ckm_no_b_mass bbx_to_tapta_ckm_no_b_mass
pixi run -e madgraph python validation/madgraph/gen_amplitude_tables.py ubx_to_epve_ckm_no_b_mass bbx_to_tapta_ckm_no_b_mass
pixi run -e madgraph python validation/madgraph/gen_couplings.py ubx_to_epve_ckm_no_b_mass bbx_to_tapta_ckm_no_b_mass
git diff --stat   # expect only: manifest.toml, the two scripts, diagrams.json, interactions.json, configs.json,
                  # amplitudes/<2 rows>.json, couplings/<2 rows>.json
```

Do not run `extract_sigma.py` (it would bank the two cross sections into
`sigma_reference.json` for cells registered `uncovered`).

### 5. Tests that read them, and what each pass means

| command | reads | a pass means |
|---|---|---|
| `cargo test -p vibegraph-lib --test coupling_oracle -- --nocapture` | `couplings/<row>.json` | every coupling in MadGraph's restricted `coupling_dict` at the CMS card (CKM couplings with the `etaWS` phase included) equals this crate's at 1e-13; MadGraph has no coupling this crate lacks; its Fortran agrees with its Python within `FORTRAN_REL_TOL` or a recorded writer deviation. A new Fortran/Python gap on a CKM coupling is a writer-precision entry for `KNOWN_FORTRAN_DEVIATIONS` (Python is the arbiter), not a crate defect |
| `cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture` | `amplitudes/<row>.json` | per-diagram, per-helicity, per-flow amplitudes equal MadGraph's up to one constant G = ±i: the V_ub vertex's phase convention (a conjugation would rotate G off the imaginary axis) and massless-b spinors |
| `cargo test -p vibegraph-lib --test validate_madgraph_diagrams` | `diagrams.json`, the scripts | 1 and 2 diagrams, as this crate enumerates (`u b~ > e+ ve`: 1; `b b~ > ta+ ta-`: 2, against 3 at default); the script's `import model sm-ckm_no_b_mass` equals the manifest's `sm` + `ckm_no_b_mass` |
| `cargo test -p vibegraph-lib --test smeftsim restricted_interaction_counts_match_madgraph -- --nocapture` | `interactions.json` | MadGraph's interaction count at this card equals this crate's 133 vertices: the MadGraph-side check on its split and pruning, the one thing the hermetic threshold and merge tests cannot reach |
| `cargo test -p vibegraph-lib --test interned_sm_rows -- --nocapture` | `amplitudes/<row>.json`'s `param_card`, the manifest | `2 manifest rows name the interned SM`; the defaults test prints `{"ckm_no_b_mass": 2, "default": 26}` and has read `DECAY 15 = 2.27e-12` and `MASS 5 = 0` from MadGraph's own generated card: WTau survives and MB locks under MadGraph's restriction, as a run rather than a reading of its source |
| `cargo test -p vibegraph-lib --test sm_ckm_no_b_mass` | — | unchanged; the MadGraph anchor skips the new tables (they name a model) |
| `cargo test --workspace`, `pixi run --skip-deps validate` | all | the rest of the suite, including `validate_sigma::the_channel_count_is_madgraphs_configuration_count` on `configs.json` for the two rows |

Do not add either process to `MG_VALIDATED_PROCESSES` (`helas/eval/compile.rs`):
that list compiles against the default SM, where `u b~ > e+ ve` has no diagram.

**Risk worth knowing before step 5.** No SM restriction has ever had its
`display interactions` count banked (`interactions.json` holds only SMEFTsim
and toy-model pairs). If the count differs from 133, the first question is
whether this crate's SM split disagrees with MadGraph's at *every*
restriction; banking `display interactions` for one default-restriction row
would localise it before anyone blames the card.

## Found

- **No test compares MadGraph's generated param card with the restricted
  defaults for the interned SM before this session**, and the SMEFTsim
  version (`smeftsim::restricted_defaults_are_madgraphs_generated_param_card`)
  compares at 1e-12, which is exact only because those cards print the
  restrict card's own digits. The interned-SM version needed the card's 5e-7
  print precision (`WH = 6.38233934e-03` prints as `6.382339e-03`). Nothing
  wrong today; worth one line in the validation notes on generated cards.
- **A hadronic run's banked `param_card` carries its PDF set's αs(M_Z) in
  `aS`** (`pp_to_ll_qcd0`: 0.13 against the restriction's 0.118). Any
  consumer that treats an amplitude table's banked card as "the model's
  defaults" is wrong for `aS` on hadronic rows. Closes with a line in the
  `madgraph-defects` or amplitude-table note, after confirming which MadGraph
  step writes it.
- **Probe-by-value is incomplete for couplings with a locked-zero cofactor.**
  F1's CKM probe misses six `yc·CKM` couplings. Harmless here, but the
  pattern F1 reported (restriction diffs miss joint dependencies) has a second
  form: one-parameter value probes miss dependencies whose cofactor is zero at
  the probe point. NaN poisoning (`poisoned` in `sm_ckm_no_b_mass.rs`) is the
  structural answer and could replace value probes elsewhere. Closes with a
  grep for tests that infer coupling dependence by moving a parameter.
- **`extract_interactions.py` names a `(model, None)` pair `<model>-None`.**
  No row has a model without a restriction today; a `model = "sm"` row
  without `restrict` would bank under `sm-None`. Closes with a one-line fix
  when such a row is added.
- **Three generators still hard-code `MG5_ROOT`**
  (`decay_semianalytic.py`, `dump_sm_decay_widths.py`,
  `gen_decay_amplitudes.py`). They read only `restrict_default`, so nothing
  needs the override yet; `gen_reweight_oracle.py` and `mg5_pinned.sh` honour
  `VG_MG5_ROOT`, `gen_couplings.py` now does. Closes with the same one-line
  change if any of them is ever run against a copy.
