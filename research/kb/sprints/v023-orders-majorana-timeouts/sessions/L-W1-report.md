---
type: Session Report
title: "L-W1 report: DECAY Auto bound to the tree-level two-body width"
description: "DECAY <pdg> Auto computed from the model's three-point vertices through the amplitude evaluation, bound at MadGraph's printed digits, banked as the resolved card, reported per channel, and matched channel by channel to MadGraph 3.7.1 compute_widths for the SM t, W, Z, H and the g2HDM heavy Higgs bosons."
status: draft
tags: [param-card, widths, madgraph-parity, simshop, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-auto-widths` off main d5e7c71 (brief commit ce5109b). Closes
`auto-widths-not-computed`; its file is deleted in the final commit.

## MadGraph semantics (pinned `research/refs/mg5amcnlo`, 3.7.1)

- **Where Auto becomes a computation.** `common_run_interface.py:3742-3769`
  (`static_check_param_card`): a `decay <pdg> auto(@NLO)` line triggers
  `compute_widths <pdgs>` (with `--nlo` only for `Auto@NLO`) at launch,
  through `static_compute_widths` (`:3805-3845`). Defaults
  (`madgraph_interface.py:1803-1804`, `common_run_interface.py:279-281`):
  `body_decay=4.0025`, `precision_channel=0.01`, `min_br` = decimal part of
  `body_decay` / 5 (`madgraph_interface.py:1872-1873`).
- **Auto widths during the computation.** Every `Auto` value in the card is set
  to 1 before parameters are evaluated (`madgraph_interface.py:9848-9861`); so
  no computed width feeds another, and no ordering exists.
- **Two-body widths.** If every requested particle has `partial_widths` (any
  model shipping `decays.py`: `import_ufo.py:426-436` gives *every* particle the
  attribute, empty when it has no `Decay`), the widths are the FeynRules
  formulas (`madgraph_interface.py:9868-9922`), with α_s re-evaluated at the
  parent mass (`set_parameters_and_couplings(..., scale=mass)`, 2-loop
  `Alphas_Runner`, `model_reader.py:186-203, 287-360`). A channel is skipped
  when `M − Σ|m_i| ≤ 0` (`:9886-9896`); a coloured parent's partial width in
  (0, 0.1) GeV is set to zero (`:9909-9912`). Without `decays.py`, MadWidth
  generates the 1→2 amplitudes and integrates them with MadEvent
  (`:9924-9998`, `precision_channel` 0.01, 1000 points × 9 iterations). The
  symmetry factor, colour and helicity sums are the matrix element's (the
  formulas and MadEvent's `IDEN`).
- **Beyond two bodies.** With `body_decay=4.0025` MadWidth then looks for 3-
  and 4-body modes (`do_decay_diagram`, `:10179-10237`): channels through an
  *off-shell* daughter of a lower-level channel (`decay_objects.py:756-760`:
  "any further decay of on-shell sub-level channel is not a new channel"),
  added while the estimated error exceeds 0.0025 (`estimate_width_error`,
  `decay_objects.py:437-456`). Measured below: it adds nothing to t, W, Z, and
  12.8% to the SM Higgs.
- **Written card.** `update_width_in_param_card` (`madevent_interface.py:2998-3077`)
  writes the total with `%e` (seven digits: the number the run then reads), and
  a BR table sorted descending, each row commented with `BR·width` in Python
  `repr` (the full-precision partial width the gate reads). An existing table
  under a computed particle is dropped and replaced (`:3014-3051`).
- **Restrict card Auto.** `import_ufo.py:2386-2388, 2464-2473`: MadGraph keeps
  the width free and writes it back as `Auto` for launch. **Kept refused**
  (`UfoError::RestrictAutoWidth`): here a restriction's default is a number
  every evaluation without a card uses (`EvaluatedModel::from_model`, the
  parameters report, the default card), so honouring it means computing widths
  at model load. The message now says to give `DECAY <pdg> Auto` in the param
  card, which works. Found item below.

## What changed

**`vibegraph-lib/src/widths.rs` (new).** `compute_width(model, evaluated, pdg,
auto_params)`: channels are each unordered daughter pair a physical three-point
vertex couples the parent to (daughters = antiparticles of the vertex's other
legs, UFO legs being incoming); open iff `M − m1 − m2 > 0`; width = `Σ|M|²` at
one rest-frame point × `initial_spin_color_average` × `outgoing_symmetry_factor`
× `p*/(8πM²)`, via `generate P > X Y` (a one-particle process: every coupling
order, `diagram_generation.py:1978-1980`), `compile_class`, `BoundAmplitude`.
MadGraph's coloured <0.1 GeV rule is matched (`below_qcd_scale`). The bound
value is `total` to `%e` (`madgraph_rounded`). `not_included` lists every
three-body mode through an off-shell daughter of a closed channel whose masses
allow it (massless self-emission `t > t* g` excluded as radiative).
`resolved_card_text` writes the card as MadGraph writes it (computed `DECAY`
line with a `# <param> Auto: tree_level_two_body width computed by vibegraph
<ver>, total <full>` comment, BR table with partial-width comments, comment
lines for modes not included).

Named refusals (`AutoWidthError`, wrapped in `CardError::AutoWidth { pdg,
reason }`): `UnknownParticle` (no physical particle, or a non-positive code),
`NotFree` (width `ZERO`, internal, or the restriction's zero/one/merged),
`RunsWithAlphaS` (a channel coupling depends on `aS`: MadGraph runs it to the
parent mass), `DependsOnAutoWidth` (a channel coupling or mass reads another
Auto width: MadGraph evaluates it at 1), `OnlyBeyondTwoBody` (no open two-body
channel but a three-body mode exists: our 0 would not be MadGraph's), and
`ModeNotComputed` (a table given below the Auto line lists a two-body mode no
tree-level vertex gives — the loop-induced case — or a mode beyond two bodies),
`Channel` (generation/compilation refuses the channel, e.g. a Majorana daughter).
A particle with no channel at all and no three-body mode binds to 0, as
MadGraph's stable particle does.

**`ufo/bind.rs`.** `bind_card` validates Auto codes, evaluates the card with
each Auto width at its restricted default as a placeholder (no computed channel
can read it: refusal above), computes, re-checks merged groups on the resolved
card, and binds the resolved card (`ParamCard::with_widths_set`): the width is
in `moved` like a typed number, the computed BR table replaces any given one.
`BoundCard::auto_widths` carries the computations; a `warn!` names the
three-body modes left out. `EvaluatedModel::from_model_card` still refuses Auto,
now as `CardError::AutoWidthUnbound`.

**Provenance in the artifact (decision).** The bound card holds the computed
number. No artifact format change: `inputs.params` carries the width as a
`MovedParam` (so `generate` replays the exact bits), `inputs.decays` the
computed table, and `param_card` banks the *resolved* card text (the computed
width, not the literal `Auto`, with the method and full total in the comment).
Integrating that banked card reproduces the run bit for bit (CLI test below).
Recomputing on replay was rejected: the banked number is what the grids were
trained at, and the amplitude probe already guards a build that computes
differently.

**Parameter report.** `Constraint::NotAuto` removed; `ParameterRecord` gains
`computed_width: Option<ComputedWidth>` (`null` unless the report was made at a
card setting that width to `Auto`); `describe_parameters_at`; `vibegraph
parameters --param-card` and `enumerate --param-card` fill it.
**`schema_version` stays 1**: the field set only grows; `not_auto` was the
binder's rule, which the report describes and which changed (the stated purpose
of the report is that a reader holds no copy of the rules). A consumer that
keyed on `not_auto` to forbid `Auto` now simply never sees it, which is the
intended behaviour for simshop. If the manager reads dropping an enum value as
"removing", bump to 2 — a one-line change.

**Docs.** `docs/src/cli/overview.md` (param-card paragraph, `computed_width`,
`--param-card` on `parameters`), `docs/src/cli/reference.md` regenerated.
`card_classes.rs`: the param-card `Auto` row is now `Consumed`; the restrict
row stays `Refused` with the new reason.

**Reference.** `validation/madgraph/gen_auto_widths.sh` +
`parse_auto_widths.py` → `auto_widths_reference.json`;
`auto_widths_g2hdm_m1000_param_card.dat` (MadGraph's own card for the model,
mh2 = mh3 = mhc = 1000, every Yukawa zero but yukawagur(3,2)=0.4,
(3,3)=0.6, DECAY 25 4.07e-3). Run end to end on MadGraph 3.7.1 (`rc=0 384s`).
This card reproduces the vg-validator gridpack/3.7.1 default totals exactly
(H± 6.360000e+01, H2 1.850258e+02, H3 7.024455e+01), confirming the card.

## Oracles

Environment: `CARGO_TARGET_DIR=/home/user/wt/target-auto-widths`, lean cargo
variables, every heavy command under `flock /home/user/wt/heavy.lock`. Logs in
`/home/user/wt/logs/lw1-*`.

### 1. Per channel against MadGraph 3.7.1 (`auto_widths_reference.json`)

Generator, end to end:

```
$ G2HDM_UFO=…/g2HDM_UFO VG_MG5_ROOT=…/mg371src AUTO_WIDTHS_WORK=…/gen HEAVY_LOCK=/home/user/wt/heavy.lock \
    bash validation/madgraph/gen_auto_widths.sh
wrote …/validation/madgraph/auto_widths_reference.json
rc=0 384s
sm two_body {'6': ('1.491472e+00', 1), '23': ('2.441755e+00', 11), '24': ('2.047910e+00', 5), '25': ('5.649527e-03', 2)}
sm default  {'6': ('1.491472e+00', 1), '23': ('2.441755e+00', 11), '24': ('2.047910e+00', 5), '25': ('6.478439e-03', 17)}
g2hdm_m1000 two_body {'35': ('1.696361e+02', 11), '36': ('6.855587e+01', 10), '37': ('5.936446e+01', 3)}
g2hdm_m1000 default  {'35': ('1.850258e+02', 15), '36': ('7.024455e+01', 18), '37': ('6.360000e+01', 10)}
```

Gate (`vibegraph-lib/tests/auto_widths.rs`; SM hermetic, g2HDM `#[ignore]`d,
needs `VG_G2HDM_UFO`):

```
$ VG_G2HDM_UFO=…/g2HDM_UFO cargo test -p vibegraph-lib --test auto_widths -- --include-ignored --nocapture
sm             6   total ours 1.491472143911e0 (binds 1.491472e0)  MadGraph 1.491472e+00  rel 9.6e-8
sm            23   total ours 2.441755476980e0 (binds 2.441755e0)  MadGraph 2.441755e+00  rel 2.0e-7
sm            24   total ours 2.047910051313e0 (binds 2.04791e0)  MadGraph 2.047910e+00  rel 2.5e-8
sm            25   total ours 5.649526939116e-3 (binds 5.649527e-3)  MadGraph 5.649527e-03  rel 1.1e-8
sm            25   default run: total 6.478439e-03 adds 15 three-body and 0 more-body modes (12.795% of it); 21 three-body modes listed as not included
sm            23   default run: total 2.441755e+00 adds 0 three-body and 0 more-body modes (0.000% of it); 10 three-body modes listed as not included
sm: worst per-channel relative difference 5.3e-15
g2hdm_m1000   35   total ours 1.696360887079e2 (binds 1.696361e2)  MadGraph 1.696361e+02  rel 6.7e-8
g2hdm_m1000   35   default run: total 1.850258e+02 adds 4 three-body and 0 more-body modes (8.318% of it); 130 three-body modes listed as not included
g2hdm_m1000   36   total ours 6.855587085162e1 (binds 6.855587e1)  MadGraph 6.855587e+01  rel 1.2e-8
g2hdm_m1000   36   default run: total 7.024455e+01 adds 2 three-body and 6 more-body modes (2.404% of it); 130 three-body modes listed as not included
g2hdm_m1000   37   total ours 5.936446253431e1 (binds 5.936446e1)  MadGraph 5.936446e+01  rel 4.3e-8
g2hdm_m1000   37   default run: total 6.360000e+01 adds 5 three-body and 2 more-body modes (6.660% of it); 70 three-body modes listed as not included
g2hdm_m1000: worst per-channel relative difference 4.0e-15
test result: ok. 9 passed; 0 failed; 0 ignored
```

(Per-channel lines in the log, e.g. `35 > [25, 25] ours 8.086458675890e1
MadGraph 8.086458675890e1 rel 1.8e-15`.) Tolerances: per channel `1e-9`
relative (measured worst 5.3e-15 SM, 4.0e-15 g2HDM: ~10⁵ headroom; both sides
are tree level in double, the limit is rounding); total against MadGraph's
printed `%e` at `5e-7` (the printing is the limit; measured ≤ 2.0e-7), and the
bound value equal to MadGraph's printed digits exactly. Channels below `1e-12`
of the total on both sides (`h2 > a a`, `a h1`: ~1e-18 GeV, an analytically
cancelling coupling) compare as zero. Every three-body mode the default run
adds is among `not_included` (checked); four-body modes are counted only.

Blind spots: shared parameter evaluation (both read the same card; formulas are
MadGraph's Python, couplings ours, so not shared in practice); orientation
(one point per channel; `decay_widths.rs` pins isotropy for SM channels); the
SM set has no identical-daughter two-body channel (the g2HDM `h2 > h1 h1`,
`Z Z` rows and the analytic test cover it, see mutation); no model without
`decays.py` (MadWidth's MadEvent path) is compared; colour-octet or triplet
parents with QCD channels (refused) are not covered.

### 2. Kinematic edges (analytic, same file)

`identical_daughters_take_the_symmetry_factor` (H → ZZ with the 1/2, H → WW
without, M_H = 300, rel < 1e-12), `massless_daughters_take_the_analytic_width`
(Z → νν̄ ×3 = e²M_Z/(96π s²c²), W → eν, rel < 1e-12),
`a_channel_at_threshold_is_closed_and_opens_above_it` (M_H = 2M_Z closed and
listed; 1e-6 above: on the analytic √(1−4x) edge at 1e-8),
`a_closed_channel_is_listed_and_adds_nothing` (Z, H → t t̄),
`a_scalar_into_identical_scalars_takes_the_analytic_width` (toy cubic scalar),
plus the refusals `a_channel_running_with_alpha_s_is_refused`,
`a_channel_reading_another_auto_width_is_refused`, and in `ufo::bind` tests
`auto_widths_that_cannot_be_computed_are_refused_naming_why` (photon, −24, 9999,
top at 80 GeV → OnlyBeyondTwoBody naming `t > ve b e+`, table rows `22 22` and
`5 -5 21`, two refusals listed together),
`a_given_table_is_replaced_and_a_stable_particle_is_zero`,
`an_auto_width_binds_like_the_number_it_computes` (Auto ≡ typed number + table
≡ resolved card text, every parameter and coupling bit for bit). Blind spot:
the analytic formulas read the model's own `vev`, `ee`, `sw`, so a wrong
parameter shared by both passes (MadGraph rows cover that).

Mutation (convention pin): dropping `outgoing_symmetry_factor` from
`channel_width`:

```
$ cargo test -p vibegraph-lib --test auto_widths   (mutated)
test a_channel_at_threshold_is_closed_and_opens_above_it ... FAILED
test identical_daughters_take_the_symmetry_factor ... FAILED
test a_channel_reading_another_auto_width_is_refused ... FAILED
test a_scalar_into_identical_scalars_takes_the_analytic_width ... FAILED
test sm_auto_widths_match_madgraph_per_channel ... ok
test result: FAILED. 4 passed; 4 failed; 1 ignored
```

Source restored (`git diff --stat vibegraph-lib/src/widths.rs` empty). The SM
MadGraph row passing under the mutation is the blind spot named above.

### 3. Bit for bit, main d5e7c71 against the branch

Release binaries (main built from `git archive d5e7c71`), `integrate --seed
20261122 --fixed-budget --no-network`, each with a param card:

```
ee_to_mumu base   | σ = 2.02489 ± 0.00072 nb | grid.bin.zst sha256 5ae24bd9a109e164cc19f7db5f8e6bac7fd9e590e4a4bedd72ec8219467129be
ee_to_mumu branch | σ = 2.02489 ± 0.00072 nb | grid.bin.zst sha256 5ae24bd9a109e164cc19f7db5f8e6bac7fd9e590e4a4bedd72ec8219467129be
dy13_default base   | σ = 934.5 ± 1.0 pb | grid.bin.zst sha256 3c17e0c383859f6c19d4a1767559191398457669a713298ade454697c040ea41
dy13_default branch | σ = 934.5 ± 1.0 pb | grid.bin.zst sha256 3c17e0c383859f6c19d4a1767559191398457669a713298ade454697c040ea41
t_bev_decay base   | Γ = 163.282 ± 0.081 MeV | grid.bin.zst sha256 21326a0af17c698986ce19df2ef43d0acdb6fda3e70d1584c3f52cb0a1f78d85
t_bev_decay branch | Γ = 163.282 ± 0.081 MeV | grid.bin.zst sha256 21326a0af17c698986ce19df2ef43d0acdb6fda3e70d1584c3f52cb0a1f78d85
```

(ee_to_mumu: banked run and param card; dy13_default: `dy13_*` cards with
PDFs; t_bev: `onshell_t_bev_run_card.dat`, `dy13_param_card.dat`. The banked
`z_to_uuxddx_decay` run card is refused by both binaries for an uncut
collinear q q̄ pair, so it was replaced by t > b e+ ve.) The whole artifact
hashes equal, so no format change leaked in.

### 4. End-to-end σ: `e+ e- > mu+ mu-` at √s = 91.2 GeV (banked ee_to_mumu run card)

Release branch binary, `--fixed-budget`, seeds 1–5; `auto` = `DECAY 23 Auto`,
`typed` = `DECAY 23 2.441755` (the number the run logged), `default` = no card
(WZ = 2.441404):

```
auto    seeds 1-5: 2.02281 2.02357 2.02376 2.02282 2.02338 nb (each ± 0.00072)
typed   seeds 1-5: 2.02281 2.02357 2.02376 2.02282 2.02338 nb
default seeds 1-5: 2.02339 2.02415 2.02434 2.02340 2.02396 nb
auto    mean 2.023268  sd 0.000435  sd/√5 0.000194
typed   mean 2.023268  sd 0.000435
default mean 2.023848  sd 0.000435
auto/default per seed 0.999713 (all five); (Γ_default/Γ_auto)² = 0.999713
```

Auto and typed are identical per seed (the bound width is the same bits; the
CLI test `an_auto_width_integrates_as_the_card_it_banks` asserts the whole
artifact equal but for the banked card text). The shift against the default
width is the peak's `1/Γ²` to six digits — the known-wrong comparison that shows
the computed width is live. Cannot see: anything common to both cards (this is
vibegraph against itself; σ against MadGraph is the banked layer's).

### Gates

```
$ cargo fmt --all --check            → (clean)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 5.88s ; rc=0
$ cargo test --workspace              → rc=0; 69 suites, 2120 passed, 0 failed, 27 ignored
    (incl. cli_param_card: an_auto_width_integrates_as_the_card_it_banks ok,
     cards_the_model_cannot_honour_are_refused ok; ufo::card_classes
     every_param_card_construct_behaves_as_classified ok)
$ cargo test -p vibegraph --features extended-validation --test validate_param_card_sigma
    test result: ok. 3 passed; 0 failed
$ cargo test -p vibegraph-lib --features extended-validation --test validate_lhef
    test result: ok. 3 passed; 0 failed
$ cargo test -p vibegraph-lib --features extended-validation --test ufo
    test result: ok. 5 passed; 0 failed; 1 ignored
```

`pixi run validate` not run (CI's banked job runs it, per the brief).

## Long-run requests

None. Every run here fit in the container (MG `compute_widths` 2-body: seconds;
default: ≤ 6 min for all four runs).

## Found

- **Beyond two bodies is a real, warned-only difference.** MadGraph's default
  Auto adds 3/4-body modes: SM H +12.8% (binds 5.649527e-3, MadGraph launches
  at 6.478439e-3), g2HDM H2 +8.3%, H± +6.7%, H3 +2.4%. vibegraph warns and
  lists `not_included` but binds the two-body value. Closing: compute
  three-body widths (MadWidth's estimate, then integrate the off-shell modes
  with the existing 1→n integrand), or make a non-empty `not_included` a
  refusal on request.
- **`not_included` is a superset**, not MadWidth's list: it names every
  kinematically open three-body mode through an off-shell daughter (130 for
  g2HDM H2, where MadGraph added 4), with no size estimate, and no four-body
  modes. Closing: port MadWidth's `get_apx_decaywidth_nextlevel` estimate.
- **α_s running refused.** A channel whose coupling depends on `aS` is refused
  (`RunsWithAlphaS`); MadGraph runs α_s (2-loop `Alphas_Runner`, Newton tol
  5e-4) to the parent mass. Closing: port that runner and evaluate at it.
- **`decays.py` is not read.** For a model shipping `decays.py`, MadGraph's
  two-body set is that file's (`import_ufo.py:426-436` gives a particle absent
  from it an empty set, so its two-body width is 0, and a mode the file lacks is
  absent), while vibegraph enumerates vertices. They agree on SM and g2HDM;
  elsewhere a difference is silent. Closing: parse the mode keys of
  `decays.py` and refuse or report a mismatch.
- **MadWidth's numerical path untested.** No reference row uses a model without
  `decays.py` (MadEvent-integrated 2-body, `precision_channel` 0.01).
  Closing: a row on such a model (e.g. SMEFTsim or an authored toy).
- **Restrict-card `Auto` stays refused** (`RestrictAutoWidth`). Closing: let a
  restriction's default width be `Auto`, computed wherever a default evaluation
  happens (`EvaluatedModel::from_model`, the report, the default card).
- **Reweighting and `Auto`.** A hypothesis that moves a parameter an Auto width
  depends on does not recompute the width (the bound number is a recorded
  parameter). Not checked whether `--reweight` refuses or silently keeps it.
  Closing: recompute per hypothesis, or refuse a move reaching a computed
  width's channels.
- **Particle names that collide with default multiparticle labels** (a model
  particle named `P`, `j`, `l+` …) make that channel's generated process card
  fail; it is a named `Channel` refusal, found by the toy test. Closing: build
  the 1→2 diagram set without going through process-card text.
- **Majorana daughters** (neutralinos) are refused through `Channel` until
  diagram generation accepts Majorana fermions (sprint L's Majorana session).
- **`Auto@NLO`** (MadGraph's `--nlo`, SMWidth) is a parse error ("expected a
  number or Auto"), named but not specific. No other MadWidth option
  (`body_decay`, `precision_channel`, `min_br`, `path`, `output`) can be set
  from a param or run card, so nothing else needs matching or refusing.
- **The g2HDM gate is not hermetic.** The UFO carries no licence to vendor it.
  Closing: a pinned (sha256) fetch of `g2HDM_UFO.tar.gz` from the CMS model
  store as a banked input, making the row a banked gate.
- **Knowledge bundle pages are stale** (not edited, per the rules):
  `research/kb/model/param-card-classification.md:53` ("refused
  (`AutoWidth`)"), `restrict-card-classification.md:46` (reason),
  `restriction-semantics.md:117-124`, and the parameters-report concept if any
  lists `not_auto`. Close-out edit.
- **Brief corrections.** (1) The brief's `CARGO_TARGET_DIR` (in-worktree
  `target`) disagrees with the assignment (`/home/user/wt/target-auto-widths`);
  I followed the assignment. (2) "`H → γγ` is loop-level so absent at tree
  level" holds for `sm`, but g2HDM has tree `h2 a a` vertices with
  analytically cancelling couplings (~1e-19 GeV both sides); harmless.
  (3) "copy mg371 with `cp -al`": that directory holds process outputs; the MG
  source is `sprintl-vbs-mg/dl/mg371src`, which I hard-link-copied with
  `models/` and `input/` de-linked (MadGraph rewrites model caches in place,
  which would write through a hard link into the original). (4) The
  bit-for-bit candidate `z_to_uuxddx_decay`'s banked run card is refused by
  main and branch alike (uncut collinear pair).
