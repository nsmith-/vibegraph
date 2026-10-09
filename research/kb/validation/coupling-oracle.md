---
type: Validation Gate
title: Coupling-level oracle
description: "Crate couplings against MadGraph's Python model_reader (the arbiter) and the Fortran COMMON after SETPARA, per banked amplitude row, so a drifting coupling is named before amplitudes move."
status: draft
tags: [couplings, madgraph, oracle, ufo, model]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n36-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L413-L457", title: "Note 36 B5 — a coupling-level oracle ahead of the amplitude gate"}
  - {id: code-coupling, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/coupling_oracle.rs", title: "vibegraph-lib/tests/coupling_oracle.rs"}
  - {id: code-gen, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/gen_couplings.py", title: "validation/madgraph/gen_couplings.py"}
---

# Coupling-level oracle

`vibegraph-lib/tests/coupling_oracle.rs` compares this crate's evaluated model
couplings against MadGraph's two evaluations of the same model on the same
`param_card.dat`, for every row that carries an `mg_amplitude` table (47 tables
under `validation/madgraph/couplings/` at `787070e`). It is hermetic: the tables
are committed. It is the `couplings-mg` standalone in the manifest; regenerate
with `pixi run -e madgraph generate-couplings`, run with `validate-couplings`.

## Why it exists

In [the amplitude oracle](amplitude-oracle.md) a coupling enters multiplied by
kinematics and summed with its neighbours, so a coupling off in its tenth digit
reads there as a spread of unit-modulus per-diagram constants that agree to ten
digits and not eleven — a symptom with no name. Here the same defect reads as one
coupling, by name, with its size. That is how `ee_to_zh_smeft`'s 1.2e-8 per-diagram
spread was attributed[^n36-b5].

## The three evaluations

Each table holds, per row:

1. **`python`**: every coupling of the restricted model, evaluated by MadGraph's
   `models.model_reader.ModelReader` from the UFO expressions directly.
2. **`fortran`**: the `COMMON/COUPLINGS/` block of the row's compiled matrix
   element, read out of the f2py module after `SETPARA` parsed the card. Only the
   couplings the subprocess uses are declared there, and every one has passed
   through MadGraph's Python-to-Fortran writer.
3. This crate's own evaluation of the row's model, restriction and card.

**MadGraph's Python is the arbiter**; its Fortran is not. The Fortran writer
emits derived parameters into `Source/couplings.f` at a fixed number of
significant digits, so a coupling built from a long UFO literal arrives short of
digits. The case on record is `GC_303 = 2i·gHza/vevhat`, SMEFTsim's loop-induced
h-Z-γ coupling: the UFO writes `gHza` from literals like `0.4583333333333333`
(11/24), the writer prints `4.583333D-01`, and the Fortran value sits 1.2e-8 off
MadGraph's own Python. That defect belongs to the list in
[MadGraph defects](madgraph-defects.md). Matching it would mean rounding a literal
on purpose, so `ee_to_zh_smeft`'s amplitude cell stays informational instead (see
also [the defect policy](madgraph-defect-policy.md)). `wpwm_to_wpwmz_cw` carries
the same deviation, but its amplitude cell is informational for a far larger,
unrelated residual (`|M|²` 2.79e1; see [the amplitude oracle](amplitude-oracle.md)).

## Tolerances and the two lists

| comparison | bound | measured | list |
|---|---|---|---|
| crate vs Python | `PYTHON_REL_TOL = 1e-13` | over the 41 rows at landing (`a8a19e0`, not re-quoted since): worst 8.85e-15 (40 ulp, SM `GC_64`); SMEFTsim's 355-coupling `ee_to_ttx_smeft` 6.48e-15; the toy models exact | `KNOWN_CRATE_DEFECTS`, **empty** |
| Fortran vs Python | `FORTRAN_REL_TOL = 1e-14` | 3.00e-16 (1.4 ulp) over every agreeing coupling | `KNOWN_FORTRAN_DEVIATIONS`: `GC_303` on `ee_to_zh_smeft` and `wpwm_to_wpwmz_cw`, 1.2e-8 |

The crate/Python bound sits ~11× above the measured maximum: both sides evaluate
the same expressions in binary64, so what separates them is association order
and the last bits of `sqrt`, `atan` and the complex-power forms — tens of ulp,
not one. A libm differing in its last bit must not fail it; a wrong expression
cannot hide under it.

Both lists are two-way. A Fortran deviation entry asserts the gap is there *and
no larger*; it fails when a new coupling starts deviating and when a listed one
stops. A crate-defect entry must still be deviating. `KNOWN_CRATE_DEFECTS` is
kept in the gate while empty because the alternative to naming a known difference
is loosening the tolerance that would otherwise catch it. It last held the
unary-minus precedence defect of `ufo/expr.rs` (`-ee**2/(2.*cw)` read as
`(-ee)**2/…`, reaching SM `GC_7`/`GC_54` and SMEFTsim's `dWT`), which is fixed.

## Blind spots

- **A rounding all three share is invisible.** Where the card or a UFO literal is
  itself the rounded number, every side agrees on it. The gate compares
  implementations, not physical inputs — the same blind spot the amplitude gate
  has.
- **Couplings the restriction dropped are not compared.** MadGraph's
  `RestrictModel` deletes vanishing couplings and merges coincident ones, so its
  set is a subset of this crate's; a coupling only this crate retains is
  unreachable from any vertex MadGraph kept, and the diagram gate compares the
  vertex sets. See [restriction semantics](../model/restriction-semantics.md).
- **The running strong coupling is not reached.** Both sides read the card's own
  `aS`; `validate_scale_couplings.rs` compares the constant pools as `aS` moves.

The manifest's `couplings-mg` note still describes the precedence defect as a
live `KNOWN_CRATE_DEFECTS` entry; the code is the current state.

[^n36-b5]: Note 36 B5. Its "≤ 1 ulp against Python" plan was replaced by the measured `1e-13`, and the precedence defect it found was fixed in the same sprint.
