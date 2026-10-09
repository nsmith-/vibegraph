---
type: Caveat
title: Partonic sigma is not comparable across some refdata boundaries
description: "A partonic sigma from refdata-2 is not comparable to refdata-3 or later (alpha_s 0.130 vs 0.118), nor are the four re-carded runs across refdata-4 to refdata-5 (up to -9.8%)."
status: draft
tags: [validation, refdata, madgraph, alpha-s, pdf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: fact, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/facts/refdata-sigma-comparability.md#L11-L20", title: "Phase B fact: refdata sigma comparability"}
  - {id: n27-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L716-L911", title: "Note 27 B5 (the 3.7.1 re-bank and the alpha_s finding)"}
  - {id: n29-g2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5783-L5799", title: "Note 29 G.2 (the cross sections move, and by how much)"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml#L130-L197", title: "validation/manifest.toml [refdata] cut history"}
---

The banked MadGraph references come in numbered cuts (`refdata-N`, pinned in
the `[refdata]` table of `validation/manifest.toml`; see
[the refdata bundle](refdata-bundle.md)). Most cuts add runs and leave the old
ones byte-identical. Two boundaries changed what an existing run's cross
section *means*, so a σ quoted from one side is a different quantity from a σ
quoted from the other.

**Rule.** Compare a banked σ only against a reference from the same side of
both boundaries below.[^fact] A shift across a boundary is a change of inputs, not a
regression and not a fix.

## refdata-2 → refdata-3: partonic α_s

`refdata-2` was generated with MadGraph 3.5.7, `refdata-3` and later with the
pinned 3.7.1 (see [MadGraph oracle pinning](madgraph-oracle-pinning.md)).
3.5.7 applied the PDF set's `αs(M_Z) = 0.130` override to every `lpp = 0`
run, even though those beams carry no PDF; 3.7.1 keeps the model's own
`0.118`.[^n27-b5]

| | 3.5.7 (`refdata-2`) | 3.7.1 (`refdata-3`+) |
|---|---|---|
| `run_card.dat` `lpp1`/`lpp2` | `0`/`0` | `0`/`0` |
| `run_card.dat` `pdlabel` | `nn23lo1` | `nn23lo1` |
| `param_card.dat` `SMINPUTS 3` | `1.300000e-01` | `1.180000e-01` |
| banked `SCALUP` | `250.0` | `250.0` |
| banked `AQCDUP` | `0.1113305` | `0.1024649` |

The partonic σ therefore scales as `0.920ⁿ` in the power of α_s: the pure-QCD
2→2 rows (`gg_to_gg`, `gg_to_ttx`, `uux_to_uux`) moved −15.4%, the
`QCD=2 QED=2` 2→3 rows −8%, and every pure-QED row did not move. Our side
did not move with it in any meaningful sense: every gate resolves α_s from the
run's own parameter card, so our σ tracked the step and the gates stayed
green.[^n27-b5]

The six runs that stay at 3.5.7 on purpose (the `ee_to_mumu_tata_qcd0`
window, anti-window and control runs, and `var_sde1`, the evidence for a 3.5.7
defect; see [MadGraph defects](madgraph-defects.md)) are `QCD=0`
lepton-collider runs, so this boundary does not change their σ.[^manifest]

## refdata-4 → refdata-5: four re-carded runs

Cut 5 replaced four runs, name for name, re-carding them from MadGraph's
internal `nn23lo1` onto `pdlabel = lhapdf` at `lhaid = 247000`
(`NNPDF23_lo_as_0130_qed`). These are different parton densities, not two
spellings of one set, and the `b b~` rows say so at 10%.[^n29-g2][^manifest]
MadGraph's own `Integrated weight (pb)` on each side:

| run | `nn23lo1` (cut 4) | `lhaid 247000` (cut 5+) | shift |
|---|---|---|---|
| `pp_to_bb` | 417 202 400.0 | 376 243 100.0 | −9.8% |
| `pp_to_bb_qcd2` | 417 208 048.74 | 376 246 848.68 | −9.8% |
| `pp_to_llj` | 503.5553 | 504.6288 | +0.21% |
| `pp_to_ll_scalefact2` | 1958.82 | 1958.95 | +0.0066% |

The superseded cut-4 runs are not in any later bundle; they survive only in
the local retired area of the machine that banked them.[^manifest]

## What this does not cover

Cuts after 5 (6 through the current pin) are stated in the manifest as
byte-identical supersets of their predecessor plus new runs, apart from one
amplitude CSV header line in cut 7, so they introduce no further σ
boundary.[^manifest] A reader comparing against numbers in archived notes
should still check which cut the note measured against before reading a shift
as physics.

[^fact]: The Phase B fact this concept replaces.
[^n27-b5]: Note 27 B5, the 3.7.1 re-bank and the α_s table.
[^n29-g2]: Note 29 G.2, the four re-carded runs' shifts.
[^manifest]: The `[refdata]` comment block in `validation/manifest.toml`, which records each cut's relation to the previous one.
