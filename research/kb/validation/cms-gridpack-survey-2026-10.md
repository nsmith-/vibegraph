---
type: Measurement
title: CMS gridpack survey, 88 production cards through the CLI (e3f543f)
description: "88 CMS MadGraph gridpacks run through vibegraph integrate as shipped: 1 gives a σ, 5 after run-card resets; three silent σ changes, four loader/parser bugs, and the refusals ranked by frequency."
status: draft
tags: [validation, cms, gridpacks, run-card, proc-card, param-card, pdf, survey, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
measured:
  commit: e3f543f
  host: "CERN VM, Intel Xeon (Ice Lake), 4 vCPUs, Linux"
  command: "vibegraph integrate <proc_card> --run-card <process/**/Cards/run_card.dat> --pdf-dir validation/pdf --fixed-budget --neval 10000 --niter 3 --seed 1 -j 4 --parallel-diagrams --no-tui --yes [--ufo-dir stage/<id>], 15 min timeout per attempt"
  data: "survey/ on the CERN host session's scratchpad: REPORT.md, results.jsonl, results_pass2.jsonl, results.csv (gridpack paths and SHA-256 per card), feature_freq_LO.tsv, hidden_offdefault.tsv, syst.tsv, tools/, logs/"
---
A Claude session on a CERN host ran 88 CMS MadGraph gridpacks from CVMFS
through the `vibegraph integrate` CLI at e3f543f, to find which card features
production uses that vibegraph does not support, and which it mishandles. This
page is that session's report, condensed. **Measured** means a recorded run or
a file read backs it; **inferred** means a reading of source or cards with no
run behind it. Every σ here is one seed at `--neval 10000 --niter 3`: a
consistency check, not a gate.

## Sample

The inventory:
- `/cvmfs/cms-griddata.cern.ch/phys_generator/gridpacks_tarball`: 33,509 tarballs.
- `/cvmfs/cms.cern.ch/phys_generator/gridpacks`: 118,508 tarballs.

The selection is deterministic (`tools/sample.py`) and takes 88 gridpacks:
- **31 Run 3 SM backgrounds**: the median member of each family's largest
  binned stems, from `{PdmV,CommonBkg}/{RunIII2024Summer24,Run3Summer22}`.
- **13 curated Run 3 SM**: ttZ, ttWW, ttHH, EFT ttZ/tZq, ttH/tHq CP, VBF HH,
  polarised same-sign WW, WZZ, and some NLO.
- **30 Run 3 BSM**: one from each of the 30 largest `pp/13p6TeV/madgraph*` folders.
- **14 Run 2**: MadGraph 2.4.2 and 2.6.5 families.

By `[..]` in the proc card, 65 are LO and 23 NLO. Only the cards were
extracted: InputCards, the final `Cards/*.dat`, the bundled `ufomodel`,
`runcmsgrid.sh` and the generation log.

The run card used is the final one MadGraph saw; the InputCards copy still
holds genproductions placeholders. A non-SM model is the gridpack's own
`bin/internal/ufomodel`, staged under the imported name. Pass 1 runs each card
as is, then with a minimal named fix. Pass 2 takes the 25 LO cards blocked by
the run card and resets each refused field to the MadGraph default its message
quotes, up to six times.

## Outcome

| first blocker (pass 1) | LO | NLO |
|---|--:|--:|
| NLO `[QCD]` | – | 23 |
| `event_norm=sum` refused | 14 | – |
| unimplemented cut (`htjmin` 9, `ihtmin` 1, `missetmax` 1) | 10 | – |
| 15-minute timeout (4-jet MLM, CKM model) | 8 | – |
| panic (exit 101) | 7 | – |
| `import model … --modelname` refused | 5 | – |
| particle name unknown (no MadGraph default renaming) | 3 | – |
| duplicate subprocess across decay assignments | 3 | – |
| no diagrams where MadGraph finds them | 2 | – |
| flavour groups separated at 1e-6 to 1e-8 refused | 2 | – |
| squared-order `X^2==n` refused | 2 | – |
| massive incoming parton refused | 2 | – |
| `xqcut` with `ptj<xqcut` refused | 2 | – |
| polarisation (decayed particle, `me_frame`) | 2 | – |
| `time_of_flight` refused | 1 | – |
| mixed-mass flavour group refused | 1 | – |
| σ produced | 1 | – |

26 of 65 LO cards import `sm-ckm_no_b_mass`, which the built-in SM lacks; they
reached the table above only through the gridpack's own model.

Pass 2 found the next blocker behind the run-card refusals:
- **Colour basis:** a crossing pair that shares a matrix element but not a
  colour basis is refused on 8 HT-binned γ/QCD MLM cards. Examples:
  `u u > g g u u` with `u~ u~ > g g u~ u~`, and `u b > b g u` with
  `u~ b~ > b~ g u~`.
- **More of the pass-1 blockers:** further panics, the order-search cap,
  unknown particle names, near-degenerate flavour groups, massive incoming
  partons and timeouts.

It produced σ on 4 more cards, 5 in all.

| card | as shipped | param card baked into the UFO + gridpack PDF | MadGraph |
|---|--:|--:|--:|
| 033 TTHH | 0.000892 pb | 0.000775 ± 0.000007 (PDF only) | 0.0007542 ± 0.0000026 |
| 072 TTALP | 23.64 pb | 0.004583 ± 0.000056 | 0.004517 ± 0.000035 |
| 075 TTDMsimpSpin0-ps (MLM) | 0.02359 ± 0.00232 | 0.001952 ± 0.000196 | 0.001936 ± 0.0000075 |
| 032 TTZ2b | 89.28 pb | – | MadGraph's survey line is broken |
| 048 vector triplet | 0.09296 pb | one nonzero point, μF = 0 logged | MadGraph's survey line is implausible |

On 075 the param card alone is ×12.2 → ×1.53, and the PDF the rest. On 033 the
built-in SM ignored the staged model (same model hash), so only the PDF moved.

## Silent physics changes (exit 0, no warning)

1. **The run card's `lhaid` never selects the PDF.** The densities always come
   from `--pdf-set`, default `NNPDF23_lo_as_0130_qed`. Measured on
   `p p > e+ e-` with `50 = mmll`: no `lhaid`, `325300 = lhaid` and
   `230000 = lhaid` all give 690.095053 ± 11.511817 pb, and every run logs
   NNPDF23. Every LO CMS card names another set: 325300 (41), 325500 (17),
   306000 (6), 325100 (1).
2. **`# hidden_parameter` run-card lines are dropped.** `split_line`
   (`runcard.rs:511-525`) strips only `!`, so the name keeps the comment and
   the line is skipped as a template placeholder. MadGraph strips `#` first
   (`banner.py:2900`).
   - Measured: `50 = mmll # hidden_parameter` gives 904.055818 ± 15.886729 pb,
     identical to no `mmll` line.
   - 66 of 88 cards carry at least one such line off vibegraph's default, 277
     lines in all: `pdfwgt` 37, `deltaeta` 24, `dral`/`draj` 22, `pta` 21,
     `etaa` 16, `drjj` 16, `ptj` 15, `draa` 14, `etaj` 13, `ptl` 12, `etal` 11,
     `drll` 11, `drjl` 11, `etaonium` 8, `maxjetflavor` 7, `ptonium` 6,
     `cut_decays` 6, `mmll` 2, `ptb` 1.
   - `False = pdfwgt # hidden_parameter` is on 20 of the 25 MLM cards.
     MadGraph then drops the PDF ratio in MLM reweighting (`reweight.f:1496`);
     vibegraph keeps it.
3. **The param card and customizecards are not read**; there is no
   `--param-card`.
   - 54 of 65 LO cards differ from `param_card_default`, and 53 have
     customizecards `set` lines.
   - The most common moves: top mass 172.5 (42), top Yukawa (41), MW 79.907
     (39), Γt 1.311 (36), ΓZ (34), ΓW (33), MZ 91.1535 (33), 1/α 128.825 (32),
     G_F (32).
   - The size is in the table above: ×12 on 075 and ×5,200 on 072.

`output <name> -nojpeg` is accepted silently, which is benign.

## Loader and parser bugs on input MadGraph accepts

- **The automatic order search stops too early.**
  - vibegraph caps it at (n_ext − 2) × the largest hierarchy
    (`diagrams/mod.rs:684-686`). MadGraph caps it at (n_ext − 2) × the largest
    WEIGHTED of any interaction (`diagram_generation.py:1998`,
    `base_objects.py:1481`).
  - So a QED+NP or QED+LAM112 vertex is never reached, and four BSM cards find
    no diagrams: 060, 067, 050 (MadGraph steps to WEIGHTED ≤ 6, vibegraph stops
    at 4) and 071 (18 against 16).
  - MadGraph's log for 060 steps WEIGHTED ≤ 4, then ≤ 5, and finds one diagram
    per subprocess.
- **A UFO `parameters.py` with an `if configuration.…:` branch panics.**
  `MW` is defined only inside such a branch, the loader takes neither branch,
  and `ufo/expr.rs:117` panics (034, 035, dim6top_LO_UFO).
- **Fermion-number-violating and Majorana models panic instead of refusing.**
  - `helas/eval/run.rs:1354`: 047, 053, 054, 061.
  - `run.rs:1281`: 052.
  - `helas/eval/kernel.rs:569`: 049.
- **MadGraph's default particle names are not provided.** MadGraph renames by
  PDG code from `input/particles_name_default.txt` unless `--modelname` is
  given (`base_objects.py:1577`). vibegraph does not, so these fail:
  - `mu+` on `loop_sm`, whose muon is `m+` (003, 021);
  - `u~` on InertDoublet (064);
  - `mu-` on HC_NLO_X0 (036).
- **`--modelname` is refused** on 5 cards, although it only switches that
  renaming off.

## Refusals and gaps, by frequency over the 65 LO cards

Card features (measured on the card text):

| feature | cards |
|---|--:|
| `use_syst = True` | 65 |
| `output -nojpeg` | 55 |
| param card ≠ default | 54 |
| customizecards `set` lines | 53 |
| `define` | 44 |
| `add process` | 35 |
| `@N` | 33 |
| `ckm_no_b_mass` restriction (26 sm, 5 loop_sm, 1 topBSM) | 32 |
| BSM UFO | 31 |
| `extramodels` | 29 |
| MLM (`ickkw=1`) | 25 |
| `event_norm=sum` | 20 |
| decay chains | 18 |
| amplitude orders | 15 |
| `htjmin > 0` | 12 |
| `/` exclusion | 12 |
| `mmll > 0` | 11 |
| MadSpin | 11 |
| `mmjj > 0` | 10 |
| `$$` veto | 9 |
| `ihtmin > 0` | 8 |
| required s-channel | 8 |
| reweight card | 7 |
| `$` veto | 6 |
| `--modelname` | 5 |
| no-op `missetmax`/`ptjNmax = 1e5` | 3 |
| polarisation `{0}` | 2 |
| `X^2==n` | 2 |
| `dynamical_scale_choice = 4` | 2 |

Systematics:
- `use_syst = True` on all 65 LO cards. The weights CMS stores come from
  genproductions' `runcmsgrid.sh`, which runs MadGraph's systematics module
  after generation on 59 of 65 LO and 20 of 23 NLO gridpacks.
- Its PDF lists give about 880, 491 or more, or 1,275 PDF weights. The scale
  arguments are always `--mur=1,2,0.5 --muf=1,2,0.5 --together=muf,mur,dyn
  --dyn=-1,1,2,3,4 --alps=0.5,1,2`.
- The other 6 LO cards are Run 2 packs that use SysCalc from the run card.
- vibegraph writes no scale, PDF or αs variation weights, and drops the
  `systematics_*` and `sys_*` fields as benign.

Other refusals:
- `htjmin`/`ihtmin`/`htjmax` are refused only after full enumeration (up to
  92 s), under a misleading "failed to decompose into flavour groups: cut
  compilation failed" prefix.
- No-op cut values (`missetmax`, `ptj1max…ptj4max` = 1e5) are refused as
  active cuts.
- Massive incoming partons: 4 cards. Mixed outgoing masses in one flavour
  group: 2.
- The duplicate-subprocess refusal across decay assignments (024
  `p p > w+ j j, w+ > j j`, 029, 068) may be stricter than MadGraph's
  decay-chain semantics. **Inferred.**
- Flavour groups separated by 9.08e-7 or 3.88e-8 with a CKM model (006, 010,
  020, 089) may be a real CKM-phase difference rather than rounding.
  **Inferred.**
- 4-jet MLM with a CKM model hits the 15-minute timeout on 10 cards. Some are
  still enumerating after 900 s (`b~ b~ > ta+ ta- g b~ b~`), others are
  integrating 1,011 of 1,068 channels at the point cap.
- No LO run-card field is unrecognised once comments are stripped.

## What this survey cannot see

- Only 5 of 88 cards produce a σ, so coverage behind the proc-card and model
  blockers is unmeasured.
- A σ comparison with MadGraph means something only with the param card baked
  and the matching PDF, which holds for 072 and 075 alone.
- Every frequency is over one 88-card stratified sample, not weighted by how
  many events CMS produces from each gridpack.

## Backlog filed from this survey

- Silent σ changes: run-card `# hidden_parameter` lines dropped (fixed: `split_line` strips `#` comments),
  [no param card in the CLI](../backlog/feature/cli-has-no-param-card-option.md) (updated),
  [PDF set not taken from the run card](../backlog/feature/pdf-set-not-taken-from-run-card.md).
- Bugs and panics: WEIGHTED search cap (closed by [the weighted-cap sprint](../sprints/weighted-cap/sprint.md)),
  UFO `if configuration` branches (closed: refused by name at load),
  [fermion-number violation panics](../sprints/fnv-refusal/sprint.md) (closed: now refused),
  default particle names and `--modelname` (closed by [the default-particle-names sprint](../sprints/default-particle-names/sprint.md)).
- Triage: [card 048's one-point σ](../backlog/validation/vector-triplet-chain-one-nonzero-point.md),
  [near-degenerate CKM flavour groups](../backlog/validation/ckm-flavour-groups-near-degenerate-refused.md),
  [duplicate subprocess across decay assignments](../backlog/validation/decay-chain-duplicate-subprocess-vs-madgraph.md).
- Feature gaps: [variation weights](../backlog/feature/systematics-variation-weights-not-written.md)
  and the silent drop of a systematics request (now a warning),
  `ckm_no_b_mass` (closed by [the ckm-no-b-mass sprint](../sprints/ckm-no-b-mass/sprint.md)),
  `event_norm = sum` (closed by [the event-norm-sum sprint](../sprints/event-norm-sum/sprint.md)),
  [HT cuts](../backlog/feature/ht-cuts-unimplemented.md) (their late refusal is
  since fixed),
  [MadSpin](../backlog/feature/madspin-decays-unsupported.md),
  [crossing pairs without a shared colour basis](../backlog/feature/crossing-pair-colour-basis-differs-refused.md),
  [massive incoming partons](../backlog/feature/massive-incoming-partons-refused.md),
  [mixed outgoing masses in a group](../backlog/feature/flavour-group-mixed-outgoing-masses-refused.md),
  [enumeration cost](../backlog/feature/diagram-enumeration-cost-unmeasured.md) (updated).
