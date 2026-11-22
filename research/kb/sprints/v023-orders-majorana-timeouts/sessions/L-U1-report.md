---
type: Session Report
title: "L-U1 report: $ placeholders refused, configuration-conditional UFO parameters loaded, time_of_flight scoped"
description: "An unfilled $ run-card placeholder is refused by name; if configuration.<flag> blocks and conditional parameter values are evaluated against configuration.py, so ctGpatched and dim6top_LO_UFO load; time_of_flight is documented as off-only."
status: draft
tags: [run-card, refusal, ufo, model-loading, cms-gridpacks]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-22}
---
Branch `sprintl-refusals` off main b2434a0. The five closed items' files are
deleted in the final commit; one link to `ufo-coupling-order-expressions-refused`
in the CMS refusal census was repaired.

## What changed

**1. `$` placeholders** (`vibegraph-lib/src/runcard.rs`, `runcard/scan.rs`,
`runcard/report.rs`; commit c7144b9). A `$`-led template line is a
`RunCardError::UnfilledPlaceholder` naming the key, the placeholder, the line,
and that CMS's `gridpack_generation.sh` fills it. `RunCard::parse`, so
`integrate`, `enumerate` and `--check`, refuse it; `vibegraph run-card` sets the
`refusal` of the key (where the key is a run-card parameter, `lhaid`) and still
lists the line under `unknown_keys` with `problem: "placeholder"`.

The placeholder set CMS substitutes, read from
`genproductions/bin/MadGraph5_aMCatNLO/gridpack_generation.sh` (L659) and the
helper it sources, `Utilities/gridpack_helpers.sh` (`set_run_card_pdf`):
`$DEFAULT_PDF_SETS` (replaced by the central set), `$DEFAULT_PDF_MEMBERS` (the
whole line deleted), `$DEFAULT_nPDF_SETS`, `$DEFAULT_nPDF_MEMBERS` (same, for
nuclear PDFs). The test covers all four; the refusal is by line shape, so any
other `$` line is refused too.

`generate` is not covered, and cannot be: it takes a grid artifact and reads no
run card (`vibegraph-cli/src/generate.rs` names none). The artifact `integrate`
writes only exists for a card that passed, so a placeholder never reaches it.
The brief's "integrate, generate and --check" over-counts by one command.

**2. ctGpatched.** The store's `SMEFTsim_topU3l_MwScheme_UFO_ctGpatched.tar.gz`
gives
`Gstrong = Parameter(..., value = 'G' if configuration.norm_chromo_gs else '1.', ...)`
(`parameters.py` L2020-L2024): a conditional expression whose two arms are
strings, so the `value` is a Python `str` once the module is imported. It is the
same form as dim6top's L3809. MadGraph imports the model as a Python module
(`models/__init__.py` `load_model`, `__import__`, L67-L106), `Parameter.__init__`
(`models/sm/object_library.py` L150) records the value that Python computed, and
`model_reader.py` L232 `exec`s that string. So MadGraph loads it with
`Gstrong = G` when `configuration.norm_chromo_gs` is true, which it is in this
model's `configuration.py` (`norm_chromo_gs = True`).

**3. Conditional parameters** (new `vibegraph-lib/src/ufo/config_eval.rs`;
`parameters.rs`, `couplings.rs`, `mod.rs`). A stated subset of Python, evaluated
against the values `configuration.py` binds:
integer and boolean literals, unary `-`, `+` and `not`, binary `+`, `-`, `*`,
`and`/`or` with Python's operand-returning semantics, `A if T else B`,
`configuration.<name>` through any alias the file imports it as, and (inside
`configuration.py`) a bare name bound earlier. `parameters.py` is flattened to
the branches that run (`if`/`elif`/`else`, any depth) and each `value` is
reduced to its selected arm before the existing string/number handling. A
condition or `value` test outside the subset (an unbound flag, a comparison, a
call, a non-integer binding) is refused by name:
`parameter 'X': value is a conditional expression whose condition is outside the subset...`
or `parameters.py has an if whose condition is outside the subset...`.
`configuration.py` is now read in order, so `QCD_inc = 1 if norm_chromo_gs else 0`
binds, and coupling orders use the same evaluator (`'QCD': 0+conf.QCD_inc`, in
ctGpatched's couplings, now loads; it was not a form the order evaluator read).
Other files' conditional definitions (`particles.py`, `couplings.py`, ...) keep
their refusal.

Deviation from the brief: ctGpatched and dim6top are the same loader gap (one
conditional-value form, plus dim6top's `if` blocks), so one evaluator closes
`ufo-non-string-parameter-value-refused`, `ufo-conditional-parameter-definitions-refused`
and `ufo-coupling-order-expressions-refused`; the first item's description
("value must be a string") named a symptom, the cause is that the value is a
conditional expression.

**4. time_of_flight.** Not small, so documented, as the item allows. MadGraph's
`add_time_of_flight` (`madgraph/interface/common_run_interface.py` L2476-L2528)
draws `vtim = c * random.expovariate(width/hbar)` (c = 299792458000 mm/s,
hbar = 6.58211915e-25 GeV s) for every event particle with a nonzero
`param_card` width and writes it when it exceeds the threshold; `time_of_flight`
is `-1` (off) in the template (`banner.py` L4213). Matching it needs a per-particle
width, a separate event-record RNG stream, and the run card at `generate`, which
reads none. `research/kb/pipeline/release-scope.md` states that only `-1` is
supported, and the refusal's reason (`runcard/classes.rs`, `P_TIME_OF_FLIGHT`)
now says why. No MadGraph sample was needed or run.

## Oracles

All commands from `/home/user/wt/sprintl-refusals` with
`CARGO_TARGET_DIR=$PWD/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo test -p vibegraph --test cli_run_card
test result: ok. 8 passed; 0 failed
$ cargo test -p vibegraph-lib --lib ufo::
test result: ok. 192 passed; 0 failed
$ cargo test -p vibegraph-lib --test ufo_value_forms
test result: ok. 16 passed; 0 failed
$ cargo fmt --all --check                                      -> clean
$ cargo clippy --workspace --all-targets -- -D warnings        -> Finished, no warnings
$ cargo test --workspace                                       -> cargo test --workspace --no-fail-fast: 69 "test result" lines, 2249 passed, 0 failed (the first attempt, before ConditionOutside named the parameters, failed `toy_models::parameter_defined_only_under_a_conditional_is_refused_by_name`; fixed and rerun)
```

**Real models** (store tarballs unpacked under the scratchpad; binary built from
this branch; `vibegraph --no-network enumerate <card> --ufo-dir <model dir> --svg none`):

```
ctGpatched, card "import model SMEFTsim_topU3l_MwScheme_UFO_ctGpatched[-<restriction>]\ngenerate e+ e- > t t~"
== bare: total:    3 diagrams in 1 subprocess
== -SMlimit_massless: total:    2 diagrams in 1 subprocess
== -massless: total:    2 diagrams in 1 subprocess
== -massless_2t: total:    2 diagrams in 1 subprocess
== -massless_2tn: total:    2 diagrams in 1 subprocess
== -massless_4t: total:    2 diagrams in 1 subprocess
== -massless_4t_2t2b: total:    2 diagrams in 1 subprocess
== -massless_4t_2t2b_cQQ1: total:    2 diagrams in 1 subprocess
== -massless_VBS: total:    2 diagrams in 1 subprocess
== -massless_VH: total:    2 diagrams in 1 subprocess
== -massless_Wtb: total:    2 diagrams in 1 subprocess
dim6top_LO_UFO
generate g g > t t~              -> total: 15 diagrams in 1 subprocess
generate u b > t z d             -> total: 28 diagrams in 1 subprocess
define p = g u c d s b u~ c~ d~ s~ b~; define j = g u c d s u~ c~ d~ s~
generate p p > t z j $$ w+ w-    -> total: 1236 diagrams in 60 subprocesses
```

The tZq_EFT_CPV_point_1_13p6 proc card could not be fetched (github.com HTML and
the tree API are blocked here; the raw path guesses under
`cards/production/13p6TeV/` returned 404), so the third dim6top line is a
tZq-shaped process of my own, not the CMS card's.

**SM unchanged, bit for bit.** Binary from the merge base b2434a0 (`git archive`,
built into the same target dir) against this branch, `--fixed-budget --neval 20000
--niter 3 --seed 11 -j 1`, `LHAPDF_DATA_PATH=validation/pdf`:

```
== main ee  (e+ e- > mu+ mu-, 45.6 x 45.6 GeV, lpp 0)  σ = 2.0180 ± 0.0033 nb
== mine ee                                              σ = 2.0180 ± 0.0033 nb
== main dy  (dy13: p p > e+ e-, default run card)       σ = 927 ± 10 pb
== mine dy                                              σ = 927 ± 10 pb
$ sha256sum o_main_ee/grid.bin.zst o_mine_ee/grid.bin.zst
1c28fdeb0b61ad713f04a87ba36462a3ceaad8515a82a39ebfb06a52b16e263d  (both)
$ sha256sum o_main_dy/grid.bin.zst o_mine_dy/grid.bin.zst
7585bff4e865f2686124aa0424e40a0789269e478cc2fcc93c097ca552484c35  (both)
```

## What each test cannot detect

- The placeholder test pins refusal by line shape for the four CMS placeholders.
  It cannot say a `$` appearing inside a value that is not at line start (for
  example `5 = $FOO`) says "placeholder": that line is refused as an unknown
  parameter named `$FOO`, with that message.
- The ctGpatched and dim6top loads show the models get past load and enumerate;
  no amplitude, coupling or parameter value was compared with a MadGraph
  evaluation. The minimal-UFO tests pin which branch is taken and the value
  selected (`MS` against `1.`; `1/EWM1` against `79.5`), which a loader that
  took the wrong arm would fail, but they share my reading of Python precedence.
- The enumeration counts are diagram counts, not MadGraph's: no MadGraph run was
  made, so a model-specific order or restriction difference is not seen.
- The bit-for-bit check covers two SM cards, whose `parameters.py` has no `if`
  and no `configuration`: it shows the flatten step and the reworked reading are
  the identity on them, not anything about a conditional model.
- `time_of_flight` is a documentation change; nothing tests the refusal's new
  wording beyond the existing `IgnoredPhysics` class test.

## Found

1. **The tZq_EFT_CPV_point_1_13p6 process card was not read.** Evidence: no
   raw-GitHub path found; `enumerate` was run on tZq-shaped processes of dim6top
   instead. Closing it: run the campaign card (CERN host) on this branch and
   compare the diagram counts and σ of tZq and ttZ to MadGraph's; that is also
   the first amplitude-level check of dim6top.
2. **Conditional definitions in other UFO files are still refused**
   (`particles.py`, `couplings.py`, `vertices.py`, `lorentz.py`, `coupling_orders.py`,
   `propagators.py`; `mod.rs` `refuse_conditional_definitions`). The new evaluator
   could flatten those too. No store model known to need it.
3. **Comparison and call forms in conditions are outside the subset** (`configuration.x > 1`,
   `abs(...)`). Refused by name; no store model known to need them.
