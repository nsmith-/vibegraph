---
type: Session Report
title: "F1 report: the CMS ckm_no_b_mass card interned in the built-in SM"
description: "SMRestrict::CkmNoBMass loads the vendored CMS card (sha256-pinned); WTau survives, MB/ymb lock; couplings pinned by composition with ckm, no_b_mass and MadGraph's default-restriction tables; MadGraph banking at this card still owed."
status: draft
tags: [model, restriction, ckm, sm, cms, sprint-d, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/9dc76b47d66af0e49e72c88e2b3d7ace8d8af66f", title: "feat(ufo): intern the CMS ckm_no_b_mass restriction of the SM (9dc76b4)"}
  - {id: mg-special-params, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L2615", title: "MadGraph import_ufo.py detect_special_parameters (exact-zero test)"}
---
## What landed (commit 9dc76b4)

- `vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat`: the CMS card,
  moved unchanged from the manager's decoded copy (1350 bytes, sha256
  `ad6e3eaf…1b05`, checked before and after the commit with `git show | sha256sum`).
- `SMRestrict::CkmNoBMass` (suffix `ckm_no_b_mass`, strum-derived), in `ALL`
  (now 10) and `restrict_card_text`. Every exhaustive list goes through `ALL`
  (both per-variant caches, `sm_digest`, the identity digest tests, the CLI's
  unknown-variant message, the config test), so they pick it up.
- `SMRestrict::ships_with_madgraph()`: false only for the new card.
  `regenerate` (`gen_sm_blob`) copies only MadGraph's cards from the submodule
  and leaves the vendored one alone. `sm_interned_blob` (the
  `check-sm-blob-fresh` test) compares only MadGraph's cards byte for byte, and
  asserts the vendored card's name is **absent** from the submodule's
  `models/sm`. If MadGraph ever ships one, someone has to decide which card the
  suffix means.
- **The blob does not need regenerating.** `sm_parsed.bin.zst` is the
  pre-restriction model, and a restrict card is applied at load. Regenerating
  it (`cargo run -p vibegraph-lib --bin gen_sm_blob`) needs the mg5amcnlo
  submodule's `models/sm`, which is empty in this worktree. It would leave the
  vendored card untouched.
- Provenance is recorded in `THIRD-PARTY-NOTICES` (new paragraph in the
  MadGraph SM section) and `research/kb/tooling/licensing.md` (new bullet). The
  user guide (`docs/src/guide/02-ufo.md`) and the `gen_sm_blob` doc comment
  name the card.
- New hermetic test file `vibegraph-lib/tests/sm_ckm_no_b_mass.rs`, with six
  tests. Its module doc says what composition cannot see.

## Corrections to the brief and the sprint note

1. **The tau width parameter is `WTau`, not `WTA`.** In `parameters.py`,
   `WTau` defaults to 2.27e-12, so the CMS card restores the UFO's own
   default, which `restrict_ckm.dat` overrides to 0.
2. **No coupling references exist at `ckm` or `no_b_mass`.** Every
   interned-SM coupling table (25) is banked at `restrict_default`. The
   composition was therefore done against this crate's own `ckm` and
   `no_b_mass` restrictions. It reaches MadGraph only through the default
   tables, and only for couplings outside the b and CKM sectors.
3. **The b and CKM sectors overlap.** `I1x31 = yb·conj(CKM3x1)`,
   `I1x32`, `I4x13 = CKM1x3·yb` and `I4x23` feed GC_13, GC_14, GC_28 and
   GC_29. These depend on both the b Yukawa and an off-diagonal CKM element.
   My first composition found the b sector as "couplings that differ between
   `default` and `no_b_mass`". That is blind to these four, because both
   factors vanish at those restrictions' identity CKM. The test caught it:
   GC_13 at this restriction is 0, and under `ckm` it is
   3.29e-5 + 8.51e-5i. The landed test probes instead: it moves MB/ymb
   under `ckm` (generic CKM, nothing locked). The b sector it finds is
   {GC_13, GC_14, GC_15, GC_28, GC_29, GC_30, GC_82, GC_83}, of which
   {GC_13, GC_14, GC_28, GC_29} are CKM-dependent. Mutation M8 below shows
   the restriction-comparison version fails.

## Tests and what each would miss

| test | pins | blind to |
|---|---|---|
| `vendored_card_is_the_cms_genproductions_bytes` | sha256 (via `identity::digest_bytes`, `sha2`), length, suffix round trip, `ships_with_madgraph() == false` | whether the CMS bytes are what CMS ran (the survey's 88 identical copies are the evidence) |
| `card_is_restrict_ckm_with_b_massless_and_a_tau_width` | every SM external reads as `restrict_ckm`'s except MB = ymb = 0 and WTau = 2.27e-12 | card entries no external names |
| `tau_width_survives_and_b_mass_locks` | locked set = ckm's ∪ {MB, ymb} ∖ {WTau}; WTau evaluates to 2.27e-12 and a card can move it, while MB stays locked against a card's 4.7 | MadGraph's own treatment (read from `detect_special_parameters`: exact zero only, not run) |
| `couplings_compose_ckm_and_no_b_mass` | every coupling outside the probed b sector equals `ckm`'s bit for bit; every b-sector coupling is 0 and equals `no_b_mass`'s; no coupling moves with WTau | anything this crate and its `ckm` share wrongly |
| `vertices_compose_ckm_and_no_b_mass` | per-vertex coupling sets equal `ckm`'s minus the b sector, with empty vertices dropped; `no_b_mass`'s dropped vertices are gone; some ckm-only vertex survives | MadGraph's 1e-13 zero threshold and coupling merging at this card |
| `madgraph_default_couplings_hold_outside_the_b_and_ckm_sectors` | 49 MadGraph-Python couplings over 25 default tables, evaluated by this restriction's model, worst 8.85e-15 (bound 1e-13, `coupling_oracle`'s); 4 skipped (GC_100, GC_15, GC_82, GC_83) | the b and CKM sectors, which have no MadGraph reference at all |

## Gate

All run in `/home/user/wt/G` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, at the tree committed as 9dc76b4:

```
$ cargo fmt --all --check
fmt exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 3m 19s
clippy exit 0
$ cargo clippy -p vibegraph-lib --features extended-validation --test sm_interned_blob -- -D warnings
    Finished `dev` profile [optimized] target(s) in 33.77s
exit 0          # compile check only: the submodule is empty here, so the test cannot run
$ cargo test --workspace
test exit 0
  unittests src/lib.rs:            ok. 1009 passed; 0 failed; 10 ignored
  unittests src/main.rs (cli):     ok. 106 passed; 0 failed
  tests/coupling_oracle.rs:        ok. 48 passed; 0 failed
  tests/amplitude_oracle.rs:       ok. 49 passed; 0 failed
  tests/validate_madgraph_diagrams.rs: ok. 57 passed; 0 failed
  tests/diagrams.rs:               ok. 20 passed; 0 failed
  tests/schannel_census.rs:        ok. 1 passed; 0 failed
  tests/proc_grammar_oracle.rs:    ok. 1 passed; 0 failed
  tests/sm_ckm_no_b_mass.rs:       ok. 6 passed; 0 failed
  (41 test binaries, none failed)
$ cargo test -p vibegraph-lib --test sm_ckm_no_b_mass -- --nocapture
b sector {"GC_13", "GC_14", "GC_15", "GC_28", "GC_29", "GC_30", "GC_82", "GC_83"}; of which CKM-dependent ["GC_13", "GC_14", "GC_28", "GC_29"]
25 tables, 49 couplings compared (worst 8.85e-15), 4 skipped as b- or CKM-dependent: {"GC_100", "GC_15", "GC_82", "GC_83"}
test result: ok. 6 passed; 0 failed
$ python3 scripts/kb.py lint
824 concepts, 290 backlog items; 0 errors, 4 warnings   # the 4 warnings predate this session (blocked items without blocked_by)
```

CLI smoke test, from the debug binary:

```
$ vibegraph --no-network integrate proc.dat --run-card run.dat --out out --neval 2000 --niter 2 --no-tui
  # proc.dat: import model sm-ckm_no_b_mass / generate e+ e- > b b~ ; run.dat: lpp 0, 250+250 GeV
 INFO model:    sm-ckm_no_b_mass (73b3d9782dad80cafe5828c9f12cc4e78ad8a501a8e050ee2b6659d4cf8e42a1)
σ = 0.382090 ± 0.000371 pb
$ (same with sm-ckm_no_b_mas)
ERROR failed to load model: unknown SM restrict variant 'ckm_no_b_mas' (the built-in SM has: default, c_mass, ckm, lepton_masses, no_b_mass, no_masses, no_tau_mass, no_widths, zeromass_ckm, ckm_no_b_mass)
```

## Mutation evidence

Each mutation was applied with `sed`. The test file was then run with
`cargo test -p vibegraph-lib --test sm_ckm_no_b_mass`, and the change
reverted. The driver is
`/tmp/claude-0/-home-user-vibegraph/5fc875eb-a86f-52ce-bbc1-f353ce6c8fb0/scratchpad/ckmG-F1-mutate.sh`
and the logs are `ckmG-F1-mutate.log` and `ckmG-F1-mutate78.log` beside it.

The first sweep restored files with `mv` from a backup. That left the
restored files' mtimes older than the mutated build, so cargo reused the
M6-mutated library for M7, M8 and the baseline, and those three results were
invalid. M1 to M6 were unaffected, because each of them rewrote its own file.
After a `touch`, M7, M8 and the baseline were rerun:

| mutation | failing tests (others pass) |
|---|---|
| M1 card `DECAY 15` 2.27e-12 → 2.28e-12 | vendored bytes, card values, tau width |
| M2 card `lamWS` 0.2253 → 0.2254 | vendored bytes, card values, couplings compose |
| M3 card `ymb` back to 4.7 | 5 of 6 (all but the MadGraph anchor) |
| M4 `ships_with_madgraph` always true | vendored bytes |
| M5 `apply_restrict` locks `|v| < 1e-10` as zero | tau width survives |
| M6 restriction skips zero-coupling pruning | vertices compose |
| M7 MadGraph anchor excludes only the CKM sector | MadGraph anchor (GC_15 etc. evaluated at ymb = 0 against MadGraph's 4.7) |
| M8 b sector found as "moves between default and no_b_mass" (the blind probe) | couplings compose, vertices compose |
| baseline after rerun | 6 passed |

## Banking request (M3, MadGraph 3.7.1 at b7687064)

This closes the item's `closes_when`: a coupling or census test pinned
against MadGraph **at this restriction**. Two plumbing changes come first,
for V1 or a follow-up:

1. **The manifest cannot name an SM restriction today.**
   `gen_couplings.model_import_path` asserts that `restrict is None` when
   `model is None`. `tests/common::load_model_for_row` maps a model-less row
   to `sm_model()` at its default restriction. Add a spelling for "the
   interned SM at restriction X", for example `model = "sm"`,
   `restrict = "ckm_no_b_mass"`. On the Python side it resolves to
   `import model sm-ckm_no_b_mass`. On the Rust side it resolves to
   `sm_model(SMRestrict::from_suffix(..))`, which keeps the gate hermetic.
2. **Do not drop the card into the submodule's `models/sm`.**
   `interned_blob_matches_submodule_exactly` now fails if
   `research/refs/mg5amcnlo/models/sm/restrict_ckm_no_b_mass.dat` exists.
   Copy the pinned tree instead and add the card to the copy:
   `cp -R research/refs/mg5amcnlo $WORK/mg5-ckmnob && cp vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat $WORK/mg5-ckmnob/models/sm/`.
   `mg5_pinned.sh` already honours `VG_MG5_ROOT=$WORK/mg5-ckmnob`.
   `gen_couplings.py` and `gen_amplitude*.py` hard-code
   `MG5_ROOT = research/refs/mg5amcnlo` for `model_reader`, so they need
   the same override.

Rows to add. One `mg_amplitude` row already gives the whole restricted
`coupling_dict` on the Python side.

- `ubx_to_epve_ckm_no_b_mass`: script `import model sm-ckm_no_b_mass`,
  `generate u b~ > e+ ve`, `output ubx_to_epve_ckm_no_b_mass -nojpeg`.
  It reaches the complex V_ub (`etaWS` phase) on a massless b. Its
  `mg_amplitude` uses `pdgs_in = [2, -5]` and `pdgs_out = [-11, 12]`, with
  the usual √s list.
- Optionally `bbx_to_epem_ckm_no_b_mass` (`b b~ > e+ e-`): a massless-b
  amplitude with no CKM, the control row.

Commands, in the madgraph environment with `VG_MG5_ROOT` set:

```
pixi run -e madgraph build-diagrams          # or build.sh for the new rows only
pixi run -e madgraph build-amplitude         # mg_<row> / mg_amp_probe_<row>
python validation/madgraph/gen_amplitude_tables.py ubx_to_epve_ckm_no_b_mass
python validation/madgraph/gen_couplings.py ubx_to_epve_ckm_no_b_mass
python validation/madgraph/extract_diagrams.py   # diagrams.json entry, if the row has a diagrams cell
```

Files produced:
- `validation/madgraph/couplings/ubx_to_epve_ckm_no_b_mass.json`
- `validation/madgraph/amplitudes/ubx_to_epve_ckm_no_b_mass.json`
- the `output/<row>` process directory for the refdata bundle
- optionally a `diagrams.json` entry

The tests that read them are `coupling_oracle.rs`, which gates every
coupling in MadGraph's restricted `coupling_dict` at 1e-13 (b-sector and CKM
couplings included), and `amplitude_oracle.rs`. `interactions.json` (via
`extract_interactions.py`, from the row's `display interactions`) would add
MadGraph's interaction count at this card. That count is the check on
coupling merging and the 1e-13 zero threshold, which composition cannot see.

Until then the item stays open, as the brief says: the model loads and is
pinned by composition, and no test compares it against MadGraph at this
restriction.

## Found

- **Stale "nine restrict cards" in the knowledge bundle.** These still say
  nine and do not mention the vendored card:
  `research/kb/model/restriction-semantics.md` ("interned with its nine
  restrict cards"), `research/kb/model/ufo-parsing.md` ("one of nine baked-in
  restrict cards"), `research/kb/references/codebases/madgraph5-amcnlo.md`
  (correct about MadGraph's own nine). I left them alone: the bundle is not this session's to edit beyond
  the provenance record. Closes with a sweep that names the tenth card.
- **The CMS card's licence is not established.** The card is MadGraph's
  param-card writer's output with three values edited by CMS, taken from a
  gridpack. Whether CMS genproductions attaches terms of its own has not
  been checked. `licensing.md` now says so. Closes with a user decision or a
  read of the genproductions licence.
- **A comparison of restrictions misses joint dependencies.** Any test that
  infers "which couplings depend on X" by diffing two restrictions misses
  couplings that are products of two parameters zero at both. That is how
  GC_13/14/28/29 escaped my first version. Nothing else in the tree was
  checked for this pattern. Closes with a grep for tests that diff
  `sm_model(A)` against `sm_model(B)` to infer dependence.
- **The mutation drivers' `mv` restore defeats cargo's freshness check.** A
  backup's mtime predates the mutated build, so the next run silently reuses
  the mutated artifact. Any sprint driver that restores by `mv` or `cp -p`
  needs a `touch`. This is worth a line in the dev-agent prompt or the
  validation methodology.
