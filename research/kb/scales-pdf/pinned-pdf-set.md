---
type: Caveat
title: The pinned PDF set is NNPDF23_lo_as_0130_qed, lhaid 247000
description: "The reference set is lhaid 247000; 244600 and 230000 are different NLO sets; cards with pdlabel = nn23lo1 use MadGraph's internal PDF, which the pdf layer cannot load, so rows bank at 247000."
status: draft
tags: [pdf, lhapdf, nnpdf, reference-data, caveat]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-22, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L230-L245", title: "Note 18 §2.2 (PDF evaluation, the pinned set)"}
  - {id: n18-h, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records (H1 grid structure, H7 lhaid surprise)"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 outcome (lhaid correction)"}
  - {id: n28-z4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L4227-L4258", title: "Note 28 Z.4 (nn23lo1 is MadGraph-internal)"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/manifest.toml#L130-L147", title: "validation/manifest.toml [refdata] pin and cut 5 re-carding"}
  - {id: alphas-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/coupling/alphas.rs#L237-L294", title: "coupling/alphas.rs pdf_label_alpha_s"}
---

# The pinned PDF set is `NNPDF23_lo_as_0130_qed`, lhaid 247000

## The set

Every reference cross section on proton beams is computed with **`NNPDF23_lo_as_0130_qed`,
LHAPDF id 247000**, member 0. It is MadGraph 5's LO default (`nn23lo1` names it), and it is
vibegraph's default PDF set (`cache::pinned::DEFAULT_PDF_SET`; the CLI's `--pdf-set`).
Fetching and caching it is [tooling/pdf-set-distribution](../tooling/pdf-set-distribution.md),
and the paper is [references/papers/nnpdf23-qed](../references/papers/nnpdf23-qed.md).[^n18-22]

Two ids that look plausible are **different sets**:[^n18-h][^n18-outcome]

| lhaid | set | |
|---|---|---|
| **247000** | NNPDF23_lo_as_0130_qed | the pinned set |
| 244600 | NNPDF23_nlo_as_0118_qed | NLO |
| 230000 | NNPDF23_nlo_as_0119 | NLO, not QED; the value MadGraph's own run-card template carries |

Because both MadGraph (`pdlabel = lhapdf`, `lhaid = 247000`) and vibegraph read this exact
grid with the same interpolation scheme, a σ comparison between them carries no
interpolation-scheme systematic ([scales-pdf/lhapdf-interpolation](lhapdf-interpolation.md)).

Each member file of this set is a **single** `lhagrid1` subgrid (nx = 100, nq = 50,
14 flavours, `x ∈ [1e-9, 1]`, `Q ∈ [1, 10000]` GeV). So it exercises no internal
flavour-threshold seam. The multi-subgrid band walk is pinned by unit tests on a synthetic
two-band fixture and by `NNPDF31_lo_as_0130` as a shape fixture. The set's `.info` has no
`ForcePositive` key, so it is read unclamped
([scales-pdf/pdf-extrapolation-and-force-positive](pdf-extrapolation-and-force-positive.md)).[^n18-h]

## `pdlabel = nn23lo1` is not this grid

MadGraph's shipped run card says `pdlabel = nn23lo1` with `lhaid = 230000`. `nn23lo1` is
MadGraph's **internal** parton-density parameterisation, not an LHAPDF6 grid, and the
`lhaid` it carries is not read. The `pdf/` layer loads only LHAPDF6 grids, so it cannot
read what MadGraph read for such a run. A cross section computed here against a `nn23lo1`
MadGraph run convolves different densities and measures that difference: the `b b̄` pair
moved **9.8%** between the two.[^n28-z4][^manifest]

So every banked proton row is carded with `pdlabel = lhapdf` and `lhaid = 247000`. The four
runs once banked at `nn23lo1` (`pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_llj`,
`pp_to_ll_scalefact2`) are re-carded onto 247000 and gate there. The bundle in force is
`refdata` version 9 (`validation/manifest.toml`, `[refdata]`). Their cross sections are not
comparable with the earlier `nn23lo1` banks
([validation/refdata-sigma-comparability](../validation/refdata-sigma-comparability.md)).

What vibegraph does with a `nn23lo1` card:

- **Densities.** The CLI loads the set `--pdf-set` names (default 247000), whatever
  `pdlabel` says; it does not map a label to a grid.
- **`α_s`.** It follows MadGraph's label table (`pdf_label_alpha_s`, `coupling/alphas.rs`):
  `nn23lo1` gives `αs(M_Z) = 0.130` at two loops, overriding the parameter card's value
  on proton beams. On fixed-energy beams `setrun.f` overwrites `pdlabel` with `none`, so
  the parameter card's value stands.[^alphas-rs]
- **`PDFSUP`.** MadEvent 3.7.1 writes `247000` for `nn23lo1`, even on fixed-energy beams
  that read no density; `RunCard::pdfsup` does the same.

How the strong coupling is sourced in each case, including the grid's own `α_s` for
`pdlabel = lhapdf`, is [scales-pdf/alpha-s-sources](alpha-s-sources.md).

[^n18-22]: Note 18 §2.2, the pinned set and the LHAPDF-mirroring design.
[^n18-h]: Note 18 §5: H1 (single-subgrid structure) and H7 (the lhaid correction).
[^n18-outcome]: Note 18 outcome, the lhaid correction.
[^n28-z4]: Note 28 Z.4, `nn23lo1` is MadGraph-internal; the re-carded twins gate.
[^manifest]: `validation/manifest.toml`, the `[refdata]` pin and the cut-5 re-carding note.
[^alphas-rs]: `pdf_label_alpha_s` and `RunningAlphaS::from_run_card`, `vibegraph-lib/src/coupling/alphas.rs`.
