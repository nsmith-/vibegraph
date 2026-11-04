---
type: Session Report
title: "G-F5 report: param-card decay tables carried; a negative mass negates its width"
description: "Decay tables recorded in the artifact (v18) and written into the <slha> header in MadWidth's layout; EvaluatedModel ports write_inc_file's negative-mass width rule, pinned per helicity against MadGraph on an SM negative-MZ row because Majorana fermions (the MSSM case) are refused."
status: draft
tags: [param-card, decay-table, negative-mass, slha, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Branch `sprintg-refusals-f5`, worktree `/home/user/wt/G1-F5`, base b5fe264.
Implementation commit `4adc70b`; this report and a kb index refresh on top.

## Brief correction (load-bearing)

The MSSM_SLHA2 per-helicity pin is impossible: diagram generation refuses
every Majorana fermion, external or internal (`ConvertError::MajoranaFermion`,
`diagrams/diagram.rs` `check_fermion_flow`; backlog
`majorana-fermions-unsupported`). Measured: the standalone row
`e- el+ > e- el+` (s-channel n1..n4) panicked with
`Convert(MajoranaFermion { particle: "n1", role: "an internal propagator" })`.
`Mneu3` is the only negative mass in any shipped MadGraph model (census of
all `models/*/restrict_*.dat`, `param_card*.dat`), so no evaluable MSSM
amplitude reads the rule today; G-F3 Found 2's "every MSSM n3 propagator has
the wrong sign today" is moot for the same reason.

Substitute pin: `standalone_jamps::ee_to_mumu_negmz`, `e+ e- > mu+ mu-` at
90 GeV with `MZ = -91.188`. MadGraph has three rules: `write_inc_file`
(MadEvent: negate every external DECAY width whose card mass is negative),
standalone Fortran `width = sign(width, mass)` (Majorana only,
`export_v4.py` L9793-L9800) and Python `-abs(width)` (Majorana only,
`model_reader.py` L241-L248). The generator row runs MadGraph's own
`write_inc_file` into `Source/param_card.inc` and points `param_read.inc` at it
(exactly what `output madevent` writes), so the reference is MadEvent's rule
executed by MadGraph's code, then MadGraph's Fortran amplitudes.

## What landed

1. `ufo/slha.rs`: decay-table rows parsed (`DecayTable`, `DecayChannel` in
   `artifact.rs`), daughters kept in card order, channel identity = sorted
   daughters as MadGraph's `load_decay`. Refused at their line: NDA != daughter
   count, no daughter, non-finite BR, non-integer code, same channel with a
   different BR. Same-BR repeat kept once.
2. `ufo/bind.rs`: `DecayTable` and `NegativeMass` refusals removed; new
   `DecayTableParticle` (table under a code the header writes no DECAY line
   for, e.g. `DECAY -24`). `bind_recorded(model, params, decays)`.
3. `ufo/mod.rs` `EvaluatedModel`: carries `decay_tables`; `apply_width_signs`
   negates each external DECAY [pdg] width (not in zeros/ones) while the mass
   at [pdg] is negative (external MASS [pdg], else the particle's mass), and
   re-evaluates dependents; `recompute` re-applies it; `card_value()` gives the
   un-negated value. Applies to model defaults, restriction and param card alike.
4. `artifact.rs`: `RunInputs::decays`; FORMAT_VERSION and OLDEST_READABLE 18;
   v13-17 and v19 refused (new real-shape v17 test).
5. `lhef/write.rs` `slha_block`: card values for widths; table rows below the
   free or dependent DECAY line in MadWidth's layout (heading, `BR NDA IDs # partial width`).
6. `card_classes.rs` rows: decay table and negative mass now Consumed; five
   new rows; KB docs (param/restrict classification, run-inputs) updated.
7. `gen_standalone_jamps.py`: `card_edits`, `madevent_params`; table
   `validation/madgraph/standalone/ee_to_mumu_negmz.json` banked.

### Lines touched in `lhef/` and `generate`
- `vibegraph-cli/src/generate.rs` L413: `bind_recorded(..., &artifact.inputs.params, &artifact.inputs.decays)`.
- `vibegraph-cli/src/integrate.rs` (artifact build): `decays: evaluated.decay_tables().to_vec()`.
- `vibegraph-lib/src/lhef/write.rs`: `slha_block` doc (L357-L363), the external-line loop (L386-L390), `slha_value` -> `card_value` and new `slha_decay_rows` (L407-L438), `slha_dependent_lines` (L558-L569), two tests (L938-L986).
- `vibegraph-lib/src/generate.rs`: untouched.

## MadGraph comparison

| row | before port | after port |
|---|---|---|
| `ee_to_mumu_negmz` (48 flow, 96 graph entries) | worst graph 1.450e-1, worst flow 1.204e-1, |M|² 8.240e-3, |G| = 0.99587 | G = -i (|G|-1 5e-14), worst flow 1.059e-13, worst graph 1.209e-13, |M|² 1.350e-13 |

Commands: `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture ee_to_mumu_negmz`
(pre: lib src stashed; log `G-F5-pre-negmz.log`; post: `G-F5-post-targeted.log`).
The table was generated with
`PYTHONPATH=<stub pylhe> VG_MG5_ROOT=<scratch copy> python3 validation/madgraph/gen_standalone_jamps.py ee_to_mumu_negmz`;
MadGraph wrote `MDL_WZ = -2.441404D+00` beside `MDL_MZ = -9.118800D+01`.
A first attempt left `output standalone`'s precompiled model in place (the
makefile does not track `param_read.inc`) and the row then passed on the
pre-change code; the generator now deletes `rw_para.o`/`libmodel.a`.

Header against MadGraph: `e+ e- > mu+ mu-` launched by the pinned MadGraph
with a card carrying tables (t, W, H, and τ under a dependent line). MadGraph's
`<slha>` is the card verbatim (diff: only a trailing space). vibegraph's
differs: values re-printed (`%e` when it reads back exactly), each row's
comment is the partial width, the heading is written for every table, the
daughters keep the card's order, other card lines (comments) are not carried.

## Mutation evidence (pre-change code, `git stash push -- vibegraph-lib/src`)

`cargo test -p vibegraph-lib --test input_card_refusals`: `FAILED. 18 passed; 5 failed`:
- `a_decay_table_is_carried_as_written`: `Err(DecayTable { pdg: 6 })`
- `a_decay_row_whose_nda_disagrees_is_refused`: needle `NDA = 3` not in the old DecayTable message
- `a_decay_channel_given_two_branching_ratios_is_refused`: needle `line 3` absent
- `a_decay_table_under_a_negative_code_is_refused`: needle `positive PDG code` absent
- `a_negative_mass_negates_its_width`: `Err(NegativeMass { name: "MT", .. })`
`ee_to_mumu_negmz` fails pre-change as in the table above.

## Gates

Done in this worktree (`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, `--jobs 1`):
- `cargo fmt --all --check`: exit 0.
- `cargo build --workspace --all-targets`: exit 0, no warnings.
- `cargo test -p vibegraph-lib --lib -- ufo:: lhef::write artifact::`: `ok. 198 passed; 0 failed`.
- `cargo test -p vibegraph-lib --test input_card_refusals --test standalone_jamps`: `ok. 23 passed`, `ok. 50 passed`.
- `cargo test -p vibegraph --test cli_param_card`: `ok. 8 passed`.
- `python3 scripts/kb.py lint`: first run reported a stale `model/index.md`; regenerated with `python3 scripts/kb.py index` (committed).

NOT completed at hand-back (the detached gate script
`scratchpad/G-F5-gates.sh`, log `scratchpad/G-F5-gates.log`, was still
running): both clippy runs, `cargo test --workspace`, `amplitude_oracle`,
`validate_lhef`, `validate_param_card_sigma`, the final kb lint. The manager
must read that log (each step's full log is `G-F5-gate-<name>.log`) before
merging. Banked amplitudes: census of all 259 banked cards/tables found no
negative mass except the new row, so they are expected unchanged; the
`standalone_jamps` run above (50/50) confirms that file.

## What the tests cannot detect

- The negmz row moves only an external mass; a dependent negative mass (MG
  reads the card's dependent entry) and a width fixed to zero/one are not exercised.
- A negative card width with a negative mass (MadGraph's `'-%s'` would crash
  on `--x`; here it becomes positive).
- Header tests check vibegraph's own layout and a MadWidth-like comparison by eye, not a shower's parse.
- Refusal tests match needles; classification rows are hand-written from MadGraph's source.

## Found

1. **Downstream consumers of a negative width are unaudited.** `lhef/resonance.rs`
   (`width > 0.0` else zero-width line), `onshell.rs`, `cuts.rs`,
   `hadronic/tail_probe.rs`, phase-space BW (`bw_scale`, now active for m<0,w<0)
   see the negated width as MadEvent's Fortran does, but their MadGraph
   counterparts (e.g. `addmothers.f`, `genps`) were not compared. Close: audit each against MadEvent with a negative-mass card.
2. **MadGraph's three negative-mass rules disagree** for non-Majorana particles
   (MadEvent negates; standalone and Python do not). vibegraph follows MadEvent.
   The restriction-time coupling evaluation here also uses MadEvent's rule where
   MadGraph's Python restriction uses `-abs` for Majorana only (differs only for
   width-dependent couplings). Worth a decision record.
3. **G7's MSSM motivation is blocked by Majorana refusal** (`majorana-fermions-unsupported`); the MSSM n3 pin belongs to that item's closure.
4. **`gen_standalone_jamps.py` imports `pylhe` transitively** (via
   `gen_amplitude_tables`) though it uses none of it; it cannot run without pylhe. Close: import the two helpers lazily or move them.
5. **`gen_standalone_jamps.py` `zero_widths` rows and standalone precompilation**:
   any future edit of Source includes needs the stale-object removal added here; a `make clean` helper would be safer.
