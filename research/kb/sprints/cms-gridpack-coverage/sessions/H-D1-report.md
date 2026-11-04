---
type: Session Report
title: "H-D1 report: MadGraph sums the two decay assignments of one subprocess incoherently; their interference is below 1e-3"
description: "On the pinned checkout, one subprocess reached by two disjoint decay assignments becomes two matrix elements (or two PDF terms of one), so MadEvent sums |M_a|² + |M_b|², each inside its own Breit–Wigner window; measured on p p > z j j, z > j j (30 such subprocesses, the same 30 vibegraph refuses, diagram totals equal). With vibegraph's own evaluators the interference is ≤ 1.4e-3 ± 1.1e-3 (colour-aligned e+ e-), 3.5e-4 (g g) and exactly 0 (q q̄ initial). A coherent sum differs from MadGraph mainly through the window union (+0.24% to +1.1%) and through cut_decays = F, where it moves σ by −23% to −26% or is IR-unsafe. Recommendation: match MadGraph. Neither option unblocks the hadronic cards alone: both then stop at the per-group cut-filter refusal."
status: draft
tags: [decay-chains, madgraph-parity, cms, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: mg-combine, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L5427", title: "MadGraph helas_objects.py combine_decay_chain_processes"}
  - {id: mg-multi, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L5866", title: "MadGraph helas_objects.py HelasMultiProcess.generate_matrix_elements, decay-chain branch"}
  - {id: mg-pdf, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L2024", title: "MadGraph export_v4.py get_pdf_lines, one PD term per listed process"}
  - {id: mg-group, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L84", title: "MadGraph super_auto_dsig_group_v4.inc, the sum over a P-directory's matrix elements"}
---
Session H-D1 of sprint H, item `decay-chain-duplicate-subprocess-vs-madgraph`.
Measure, do not implement (decision D2). Worktree `/home/user/wt-decay-duplicate`,
branch `sprinth-decay-duplicate` off `sprinth-plan` (`9ba3d96`), verified before
any action. MadGraph is the pinned submodule (`b7687064`, VERSION 3.7.1), copied
to the scratchpad and run there (`output madevent`, `launch`), gfortran 
present. The host is a shared 4-core container, under load from three sibling
sessions.

Scratch artefacts, none committed (scratchpad
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/hd1/`):
the MadGraph scripts and outputs (`hd1_zto2q`, `H-D1_*`), the census script
`scripts/me_census.py` and its `*_census.txt`, the vibegraph probe patch
`H-D1_scratch_patch.diff` (an environment switch in `chain.rs` plus the test module
`hd1_scratch.rs`), and the logs `H-D1_interference2.log`, `H-D1_single*_all.log`,
`vg/H-D1_*.log`. The patch was reverted before the commit; the tree holds only
this report.

## 1. MadGraph's behaviour, from source

### Where the duplicate comes from

`HelasDecayChainProcess.combine_decay_chain_processes`
(`madgraph/core/helas_objects.py:5427`) loops over the **core matrix elements**
(`while self['core_processes']`, `:5467`). The core matrix elements are already
flavour-merged: `g g > z u u~` and `g g > z c c~` are one core matrix element
with two processes (log of the `p p > z j j, z > j j` output: *"Combined process
g g > z c c~ … with process g g > z u u~"*). Each core matrix element is
combined with every decay assignment (`itertools.product`, `:5572`;
unordered combinations of one particle's decays are kept once through
`red_decay_chains`, `:5549`–`:5566`). `insert_decay_chains` (`:5609`) gives the
combined matrix element, whose process list is the core's process list with that
decay applied to each.

So `z > c c~` on the core `{g g > z u u~, g g > z c c~}` and `z > u u~` on the
same core are two combined matrix elements. Between them they list
`g g > c c~ u u~` (Z → cc̄ on the uū core) and `g g > u u~ c c~` (Z → uū on the
cc̄ core): one subprocess, two disjoint assignments.

### What the tag merge does with them

- Each combination gets `IdentifyMETag.create_tag(base_amplitude,
  identical_particle_factor)` (`:5611`). An equal tag appends the combination's
  processes to the earlier matrix element's list (`:5620`, `:5638`). Otherwise
  the combination is a new matrix element (`:5626`). `HelasMultiProcess.
  generate_matrix_elements` repeats this across process lines with a tag without
  the factor (`:5886`–`:5906`).
- The tag (`helas_objects.py:59`–`:130`) keys on couplings, Lorentz and colour
  structures, masses and widths, the PDG code of on-shell s-channel lines
  (`:172`, and `vertex_id_from_vertex`), process id, mirror flag,
  identical-particle factor, and the permutation structure of identical external
  particles (`prepare_comp_dict`). It ignores the PDG codes of external legs.
- **Nothing in either path deduplicates a process list.** A subprocess already
  listed is appended again.

### How a P-directory sums them

- `get_pdf_lines` (`madgraph/iolibs/export_v4.py:2024`–`:2051`) writes one
  `IPROC=IPROC+1; PD(IPROC)=…; PD(0)=PD(0)+DABS(PD(IPROC))` per listed
  process, with no set over processes: two listings of one subprocess are two PD
  terms of one |M|².
- `super_auto_dsig_group_v4.inc:84`–`:96`: the grouped `DSIG` loops `IPROC = 1,
  MAXSPROC` over every matrix element of the P-directory and adds each
  `DSIGPROC` into `SUMPROB`. One process is then selected for the event, and the
  weight is renormalised by `SUMPROB/SELPROC` (`:211`). The integrand is the
  **sum over matrix elements**.
- Each matrix element applies its own Breit–Wigner window to its own forced
  line, and with `cut_decays = F` it leaves its own decay products uncut.
- `identical_decay_chain_factor` (`helas_objects.py:4581`) reads only one
  combination's own core legs and decay chains, and the measured `IDEN` is the
  plain spin × colour average (256 for gg, 72 for qq̄, 4 for e+e−). No factor
  divides the two assignments by 2.

**Reading:** MadGraph sums the two assignments incoherently,
σ = ∫|M_a|²Θ_A + ∫|M_b|²Θ_B. That holds whether they land in two matrix elements
(different tags) or in one with two PD terms (equal tags). Section 2 confirms both
paths by measurement.

## 2. MadGraph's behaviour, measured (`output madevent`, pinned checkout)

`me_census.py` reads each P-directory's `matrix*_orig.f` (`NGRAPHS`, `IDEN`) and
`auto_dsigN.f` (the `IPROC` process comments). It keys every PD term by
(initial, sorted final) and lists the keys that more than one term covers.

| card | P-dirs (matrix elements) | distinct subprocesses | covered twice | how |
|---|---|--:|--:|---|
| `p p > z j j, z > j j` | gg_zqq 6, gq_zgq 12, qq_zgg 4, qq_zqq 64 | 226 | **30** | always two different matrix elements |
| `p p > w+ j j, w+ > j j` | gg 1, gq 2, qq_wpgg 1, qq_wpqq 25 | 96 | **0** | — |
| `e+ e- > z j j, z > j j` | ll_zqq 6 | 10 | 6 | two matrix elements |
| `e+ e- > z j j / z, z > j j` | ll_zqq 6 | 10 | 6 | two matrix elements |
| `e+ e- > z u u~, z > c c~ @1` + `add process e+ e- > z c c~, z > u u~ @1` | 1 (1) | 1 | 1 | **one** matrix element, two PD terms (equal tags) |
| same two lines without `@1` | P1 (1), P2 (1) | 1 | 1 | two P-directories (process ids differ) |

`P1_gg_zqq_z_qq` of the first card, verbatim from the census:

```
ME1: 8 diagrams, IDEN 256, 2 processes: g g > u u~ u u~, g g > u u~ c c~
ME2: 8 diagrams, IDEN 256, 2 processes: g g > c c~ u u~, g g > c c~ c c~
ME3: 8 diagrams, IDEN 256, 4 processes: g g > d d~ u u~, g g > d d~ c c~, g g > s s~ u u~, g g > s s~ c c~
ME4: 8 diagrams, IDEN 256, 4 processes: g g > u u~ d d~, g g > u u~ s s~, g g > c c~ d d~, g g > c c~ s s~
...
g g > c c~ u u~: 2x  P1_gg_zqq_z_qq/ME1:g g > u u~ c c~ | P1_gg_zqq_z_qq/ME2:g g > c c~ u u~
```

The legs are in each matrix element's own order: the Z's products first, at the
Z's position. So ME1's `u u~ c c~` is Z → uū and ME2's `c c~ u u~` is Z → cc̄.
The 30 doubly covered subprocesses are the six gg ones and 24 qq̄ → qq̄ q'q̄'
ones (e.g. `u u~ > u u~ c c~`: ME12 and ME14 of `P1_qq_zqq_z_qq`). The qg
directories have none.

**vibegraph refuses exactly these 30.** With the duplicate refusal switched to a
merge (scratch patch), `p p > z j j, z > j j` merged 30 subprocesses. As
(sorted initial, sorted final) sets they are MadGraph's 30 (`30 30 True`). Each
merged diagram count equals the sum of `NGRAPHS` of MadGraph's two matrix
elements (`equal diagram totals: 30 differ: []`; e.g. gg: 8 + 8 = 16).

**σ, incoherent normalisation.** MadEvent, five seeds each, 5000 events, on the
single assignment (by u ↔ c symmetry, massless with equal couplings, the
incoherent σ of the subprocess is twice it):

| card (MadEvent) | run card | seeds | σ (pb), per seed | mean ± sd/√5 |
|---|---|---|---|---|
| `g g > z u u~, z > c c~` | lpp 0, √s 500, pT>20 and ΔR>0.4 on all four quarks (`cut_decays T`), no η cut, μ fixed at MZ | 21–25 | 0.03581, 0.03553, 0.03547, 0.03538, 0.03562 | 0.035562 ± 0.000073 |
| `e+ e- > z u u~ / z, z > c c~` | √s 1000, same cuts | 31–35 | 5.075e-4, 5.009e-4, 5.036e-4, 5.041e-4, 4.990e-4 | 5.0302e-4 ± 0.0145e-4 |

Without cuts the e+e− card is IR-unsafe: γ* → uū at small mass, recoiling
against an ISR Z. MadEvent then gives 62 and 83 events and 5.1e-3 against
4.3e-3 pb on two seeds. Those runs are discarded.

## 3. The vibegraph side: what each option computes, and the interference

### What the refused card would give

- **Incoherent (match MadGraph):** two subprocesses with the same particles,
  each with its own assignment's diagrams, forced line and window. This is
  exactly MadGraph's two matrix elements.
- **Coherent (deviation):** one subprocess holding both assignments' diagrams.
  For gg → uūcc̄ that is 16 diagrams with two forced-line patterns. vibegraph
  already handles several patterns in one subprocess the way it does identical
  pairings: the windows are a union (`cuts.rs:1013`, `decay_windows`), and
  `cut_decays = F` exempts only the legs that are decay products in *every*
  pattern (`ForcedResonances::decay_products`, `cuts.rs:177`). With two disjoint
  patterns that is no leg, so all four quarks are cut.

### Method

Scratch test `hd1_scratch.rs`, run with `VG_HD1_DUP=coherent` so the chain
enumeration merges the duplicate. Per card, the merged subprocess's diagrams are
split by forced-line pattern into A and B, and A, B and A∪B are each compiled
with `AmplitudeEvaluator` and evaluated by `BoundAmplitude::eval_m2` at the same
points. `eval_m2` is the colour- and helicity-summed |M|², and σ divides by IDEN.

Phase space is an importance-sampled mixture of two channels, one per pattern.
Each channel draws the pattern's pair mass from a Breit–Wigner on its 15Γ window,
the other pair's mass flat, and all decay angles flat. The point's density under
both channels gives the weight 1/(½p_A + ½p_B). The sampler covers Θ_A ∪ Θ_B.
All integrals share the points, and each difference is accumulated per point, so
its error is that of the difference. Two million points per card at a fixed seed.

Estimators (Θ_k: pattern k's pair inside its window; "cuts" as in the table
above unless stated):

- I = ∫|M_A|²Θ_A + |M_B|²Θ_B (MadGraph)
- C = ∫|M_{A∪B}|² Θ_{A∪B} (coherent, window union)
- C − I = interference over the union, 2Re(M_A M_B*), plus tails:
  |M_A|²Θ_{B\A} + |M_B|²Θ_{A\B}

Command (scratch patch applied):

```
VG_HD1_DUP=coherent cargo test --release -p vibegraph-lib --lib hd1_ -- --nocapture --test-threads 2
```

### Results (`H-D1_interference2.log`, verbatim numbers)

| subprocess | √s | diagrams (A + B) | σ_I (pb) | (C−I)/I | interference/I | tails/I |
|---|---|---|---|---|---|---|
| e+e− → uū cc̄ (`/ z`) | 1000 | 4 + 4 | 1.00408e-3 ± 1.6e-5 | +1.21e-2 ± 1.2e-3 | +1.19e-3 ± 1.4e-3 | +1.09e-2 ± 0.9e-3 |
| e+e− → uū dd̄ (`/ z`) | 1000 | 4 + 4 | 7.4297e-4 ± 1.5e-5 | +1.25e-2 ± 1.0e-3 | +1.41e-3 ± 1.1e-3 | +1.11e-2 ± 0.5e-3 |
| gg → uū cc̄ | 500 | 8 + 8 | 7.1420e-2 ± 2.4e-4 | +2.77e-3 ± 0.04e-3 | +3.48e-4 ± 0.22e-4 | +2.42e-3 ± 0.03e-3 |
| uū → uū cc̄ | 500 | 8 + 4 | 0.8600 ± 0.019 | +3.11e-3 ± 0.14e-3 | +3.9e-18 (zero) | +3.11e-3 ± 0.14e-3 |

**The incoherent estimator reproduces MadEvent:**

- gg: 7.1420e-2 ± 2.4e-4 against 2 × 0.035562 = 7.1124e-2 ± 1.5e-4 (+0.42%,
  1.0σ combined);
- e+e−: 1.00408e-3 ± 1.6e-5 against 2 × 5.0302e-4 = 1.00604e-3 ± 0.29e-5
  (−0.19%, 0.12σ).

That pins the sampler, the windows and the normalisation, so the C and
interference columns are read against MadGraph's own number.

**Reading:**

- **The interference is at most about 1e-3.** It is largest where the two
  assignments have the same colour structure: e+e−, both pairs colour singlets,
  +1.2e-3 ± 1.4e-3, consistent with zero. It is colour-suppressed in gg
  (+3.5e-4 ± 0.2e-4) and **exactly zero** for a qq̄ initial state, where one
  assignment's non-decay pair comes from an s-channel gluon (octet) and the
  other's is a Z singlet. That is Tr Tᵃ = 0, not a numerical accident (1e-18).
  The part from points with both pairs inside their windows is smaller still
  (gg 7.6e-5 ± 1.9e-5). This is the O(Γ/M) suppression of interference between
  distinct narrow resonances, as in the identical-pairing study (0.23%).
- **What a coherent sum moves is the window union, not the interference.** Each
  assignment's off-window tail counts wherever the other assignment's window
  holds: +0.24% (gg) to +1.1% (e+e−). These are genuine non-resonant pieces of
  the full process, but they are a choice of observable, not a higher-order
  correction.
- **`cut_decays = F`, MadGraph's default, decides it.** The same points, with
  each assignment cutting only its own core pair (MadGraph):

  | subprocess | σ_I, cut_decays F (pb) | C, all four cut (vibegraph's current rule) / I_F | C, each pattern cutting its own core / I_F |
  |---|---|---|---|
  | e+e− → uū cc̄ | 1.3206e-3 ± 1.8e-5 | 0.770 ± 0.012 | 1.071 ± 0.015 |
  | e+e− → uū dd̄ | 9.862e-4 ± 1.7e-5 | 0.763 ± 0.015 | 1.066 ± 0.018 |
  | gg → uū cc̄ | 9.6247e-2 ± 2.8e-4 | 0.744 ± 0.003 | 1.39 ± 0.05 |
  | uū → uū cc̄ | 1.134 ± 0.020 | 0.761 ± 0.017 | 1.84 ± 0.22 |

  - Under the current rule a coherent subprocess cuts the Z's products too: σ
    is 23–26% below MadGraph.
  - The obvious alternative counts |M_A|² where only B's window and B's core
    cuts hold, so A's core quarks are uncut. For a QCD core those quarks carry
    the t-channel collinear singularity, and that integral is IR-unsafe: the
    estimate is large and noisy (1.39 ± 0.05, 1.84 ± 0.22) and would grow
    without a cutoff.
  - So coherent cannot reproduce MadGraph's default run card at all. It needs a
    new rule for which legs a cut sees, and any such rule is a different
    observable at the 25% level.

## 4. What else the measurement found

- **Neither option unblocks the hadronic cards.** With the refusal switched to
  either merge or two subprocesses, `vibegraph integrate` on
  `p p > z j j, z > j j` stops at *"groups 0 and 1 compile to different cut
  filters, so the sum has no single phase-space indicator; a process whose
  groups are cut differently needs one indicator per group"*
  (`vg/H-D1_zjj_{incoherent,coherent}.log`), with `cut_decays T` as well
  (`vg/H-D1_zjj_cd_incoherent.log`).
  - `p p > w+ j j, w+ > j j` stops at the same refusal with no duplicate at all.
  - Per-group cut indicators are out of the sprint's scope (`sprint.md`). The
    11 gridpacks this item counts stay refused until that lands, whichever option
    is chosen.
- **No fixed-beam card can show the case through the CLI.** The duplicate needs
  `j` in the decay, which also yields other final states, and fixed beams
  refuse a card whose subprocesses differ in content (*"fixed-energy beams
  require every subprocess to share the same external particle content"*). The
  σ measurement above therefore runs on the library evaluators.
- **Card 024's stated form does not reproduce the refusal.** In the built-in SM,
  `p p > w+ j j, w+ > j j` has no duplicate subprocess in MadGraph (0 of 96) or
  in vibegraph (enumeration passes; the per-group cut-filter refusal is what
  stops it). A W⁺ → (up, anti-down) pair and the core's (down, anti-up) pair
  cannot be swapped, even with CKM mixing. The survey card that hit
  `DuplicateSubprocess` must differ: another model, `j` definition or extra
  lines. Cards 029, 068, Zto2Q and TTToHcToWA are not in this repository either.
- **e+e− `z j j, z > j j` without `/ z`** is refused here for `e+ e- > u u~ u u~`
  as an ambiguous forced line (a known, documented refusal). MadGraph generates
  it. `/ z` in the core removes the ambiguity, and the measurement uses it.
- **The refusal's message** at `chain.rs:210` names the card's one process line
  as both "process lines" (*"is produced by two process lines ('p p > z j j,
  z > j j' and 'p p > z j j, z > j j')"*), as the brief says.

## 5. Recommendation: match MadGraph (incoherent)

1. **MadGraph's behaviour is unambiguous and measured.** Both of its paths, two
   matrix elements and two PD terms of one, give |M_a|²Θ_A + |M_b|²Θ_B.
   vibegraph's duplicate set is MadGraph's, 30 of 30, with equal diagram
   totals.
2. **The physics a coherent sum adds is ≤ 1e-3.** The interference is bounded
   by the colour-aligned e+e− case (1.2e-3 ± 1.4e-3), is 3.5e-4 in gg and is
   zero for a qq̄ initial state. That is below every gated σ tolerance (0.2–0.8%)
   and below the narrow-width approximation's own O(Γ/M) accuracy, which the
   decay-chain card already asks for.
3. **The coherent option's real effect is a change of observable, mainly under
   `cut_decays = F`.** With the current cut rule σ falls 23–26% from
   MadGraph's. With per-pattern cuts it is IR-unsafe. A coherent option would
   need a new cut semantics and a registered deviation tens of percent wide on
   default run cards, for 0.1% of physics.
4. **The case against, and why it does not carry over.** vibegraph already keeps
   coherent sums over pairings of *identical* particles
   ([identical particles across decays](../../../process/identical-particles-across-decays.md)),
   and matching MadGraph here would treat `g g > u u~ u u~` coherently but
   `g g > u u~ c c~` incoherently. In the identical case, though, every pattern's
   products are the same species, so `cut_decays` and the window union are
   symmetric and well defined, and MadGraph's single pairing needs an
   identical-chain factor that is an approximation. With distinct flavours,
   MadGraph's incoherent sum is the standard narrow-width result, and the only
   differences are the interference (≤ 1e-3) and the window tails.

## 6. What the implementation would touch

**Incoherent (recommended):**

- `vibegraph-lib/src/diagrams/chain.rs:203`–`220`: drop the
  `DuplicateSubprocess` return for a subprocess reached by two different
  assignments. Within one chain line no two assignments are the same
  combination (`assignments` keeps an unordered combination once, as
  `red_decay_chains` does), so every duplicate here is a distinct assignment.
  The cross-line check (`diagrams/mod.rs:433`) stays as it is: MadGraph sums
  two lines too (section 2's equal-tag row), but that case is item-external.
- Audit what assumes one `DiagramSet` per flavour content within a line:
  flavour grouping in the proton path, event-record subprocess selection,
  `ForcedResonances` per set. Each set keeps one pattern, so none of the cut
  machinery changes.
- Oracle: add `p p > z j j, z > j j` and `e+ e- > z j j / z, z > j j` to
  `validation/madgraph/dump_decay_chain_census.py`. That is pure Python, pinned
  checkout, runnable here. `decay_chain_census.rs` then matches every (process,
  decays) pair against MadGraph's matrix elements, the 30 doubled subprocesses
  included.
- A σ row (section 7) once the per-group cut indicator exists. Until then,
  e+e− `z j j / z, z > j j` is not runnable on fixed beams (mixed content)
  either.

**Coherent (deviation):**

- `chain.rs`: merge each duplicate into one set, relabelling the second
  assignment's legs to the first's order (`relabelled`, `chain.rs:633`) and
  deduplicating by canonical form. That is about 40 lines; the scratch patch did
  exactly this.
- `cuts.rs` (`ForcedResonances::decay_products`, `decay_windows`,
  `Cuts::compile_with`): a decided rule for `cut_decays = F` with several
  patterns. Section 3 shows the current rule moves σ by −25%, and the
  per-pattern alternative is IR-unsafe.
- Event records: the configuration draw restricted to patterns inside their
  windows already exists for identical pairings (resonance records).
- A registered deviation in `validation/madgraph-defects.md`, and the census
  comparing per-pattern diagram subsets against MadGraph's matrix elements (as
  oracle 3 of decay chains does for identical decays).

## 7. What still needs the M3 host

Everything that did not need banking was run here. What remains:

1. **The survey cards.** The literal `proc_card_mg5.dat` and model of 024, 029,
   068, `Zto2Q-4Jets_Bin-HT-*` and `TTToHcToWA_AToMuMu_*` from the gridpack
   database. 024's stated form does not hit the refusal (section 4). For each,
   run in the pinned checkout:

   ```
   set automatic_html_opening False
   import model <the gridpack's model>
   <the card's define and generate/add process lines, verbatim>
   output madevent H-D1_<card> -nojpeg
   ```

   Then bank: every `SubProcesses/P*/processes.dat`, every
   `auto_dsig[0-9]*.f` `IPROC` comment line, `NGRAPHS` and `IDEN` of every
   `matrix*_orig.f`, and the `Combined`/`Combine` lines of the log. The census
   script `me_census.py` (scratchpad) prints exactly that. It is enough to
   show whether each card's duplicates are the distinct-assignment case
   measured here.
2. **A banked σ row for the `Zto2Q`-like card,** to be compared once vibegraph
   can run hadronic decay chains with per-group cuts:

   ```
   import model sm
   generate p p > z j j, z > j j
   output madevent H-D1_zto2q -nojpeg
   launch H-D1_zto2q -n s<seed>
   set iseed <seed>        # 5 seeds: 11 12 13 14 15
   set nevents 20000
   set pdlabel nn23lo1
   set ebeam1 6500
   set ebeam2 6500
   set ptj 20
   set drjj 0.4
   set cut_decays False
   set use_syst False
   done
   ```

   Bank σ ± error per seed, `results.dat`, and the run and param cards, under
   `validation/madgraph/` in the reference bundle. With `cut_decays True` as a
   second row, the cut semantics can be checked separately.

## Found

- **Per-group cut indicators block every hadronic decay chain with jets in the
  decay.** `p p > z j j, z > j j` (either option) and `p p > w+ j j, w+ > j j`
  (no duplicate at all) both stop at *"groups … compile to different cut
  filters"*, with `cut_decays` T or F. Item 5's 11 gridpacks stay refused
  until it lands. Evidence: `vg/H-D1_zjj_*.log`, `vg/H-D1_wpjj.log` (section 4).
  Closing it takes one cut indicator per flavour group in the proton integrand.
- **Survey card 024's form does not reproduce its refusal.** The stated
  `p p > w+ j j, w+ > j j` gives no duplicate in MadGraph or vibegraph in the
  built-in SM. The survey's attribution of 024 (and the backlog item's text,
  which still cites `chain.rs:204`) needs the literal card. Closing it takes
  re-running 024 from the gridpack database with `-v` and recording the
  duplicate subprocess named.
- **The refusal's message names one line twice** (`chain.rs:210`–`218`, CLI
  output in section 4). Closing it takes a message of its own for "one
  subprocess from two decay assignments of one line", naming the subprocess and
  both assignments. It is moot if the incoherent option lands.
- **The decay-chain census has no card with a duplicate assignment.** Adding
  `p p > z j j, z > j j` and `e+ e- > z j j / z, z > j j` to
  `dump_decay_chain_census.py` would bank MadGraph's 30 doubled subprocesses
  (pure Python, runs without M3) and pin whichever option lands.
- **Fixed-beam runs refuse any decay-chain card whose decay is a label**
  (`z > j j` yields several final-state contents). That rules out the cheapest
  σ row for this case. Closing it takes one fixed-beam integrand per external
  content, summed.
