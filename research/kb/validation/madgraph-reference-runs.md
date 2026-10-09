---
type: Design
title: How MadGraph reference runs are made and read
description: "Where each reference number comes from (results.dat, standalone dumps, banked events), the run card as shared input, and the MadGraph output behaviours a reader must know."
status: draft
tags: [madgraph, reference, run-card, lhe, standalone]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n12-roots, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L37-L87", title: "Note 12, root causes and the two oracle defects"}
  - {id: n19-survey, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L32-L76", title: "Note 19 §2, survey of the banked runs"}
  - {id: n27-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L482-L715", title: "Note 27 B4, the 3.7.1 mechanism, IDWTUP and packed CF"}
  - {id: n27-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L716-L911", title: "Note 27 B5, the re-bank and the LHE dialects"}
  - {id: n27-reg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L1223-L1243", title: "Note 27 findings register"}
  - {id: n29-d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3324-L3770", title: "Note 29 chain D measurements"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3076-L3402", title: "Note 41 Z1 and B1 records"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml"}
---
# How MadGraph reference runs are made and read

Every reference row in [the manifest](process-manifest.md) names a
`validation/madgraph/scripts/<row>.mg5` script that generates the process and
launches it with a run card. The generated process directories live in the
gitignored work area `validation/madgraph/output/<row>/`; the parts a gate needs
are carried by [the reference bundle](refdata-bundle.md). The scripts are run by
the pinned MadGraph ([madgraph-oracle-pinning](madgraph-oracle-pinning.md));
the commands are in
[tooling/madgraph-reference-generation](../tooling/madgraph-reference-generation.md),
and the code being read is described in
[the MadGraph codebase concept](../references/codebases/madgraph5-amcnlo.md).

## Where a reference number comes from

| reference | source | read by |
|---|---|---|
| fixed-energy σ̂ (`lpp = 0`) | the run's `results.dat` / the banner's `Integrated weight`, extracted to `sigma_reference.json` | `validate_sigma` |
| seeded MadEvent σ | per seed, `Events/run_<tag>/vg_seed_result.txt`, written by the generators that source `madevent_seeds.sh`, combined into committed JSON (`mlm_sigma_reference.json`, `grammar_sigma_reference.json`, the decay and on-shell-veto tables) | the σ gates, under [the seed policy](madevent-reference-seed-policy.md) |
| amplitudes, couplings | MadGraph's standalone `MATRIX1`, compiled through f2py by `build_amplitude.sh`; tables by `gen_amplitude_tables.py`, couplings by `gen_couplings.py` | [amplitude-oracle](amplitude-oracle.md), [coupling-oracle](coupling-oracle.md) |
| event samples, per-event fields | `Events/run_01/unweighted_events.lhe.gz` and its banner | samples, scale and `AQCDUP` replays, the LHE round trip |
| per-event intermediates | instrumented MadEvent replays (kT and MLM dumps) | [kt-cluster-dump-oracle](kt-cluster-dump-oracle.md), [mlm-dump-oracle](mlm-dump-oracle.md) |
| structure | `configs.inc`, `leshouche.inc`, `matrix*_orig.f`, `display interactions` in `build.log` | the structural censuses and colour oracles |

`lpp = 0` scripts launch fixed-energy beams, so their σ is the partonic σ̂ and
compares directly with a vibegraph integral with no PDF. Only `pp_*` rows use
proton beams. `sigma_reference.json` and `hadronic_sigma_reference.json` predate
the seed policy and hold single runs; a row moved onto a seeded reference says so
in its cell[^manifest].

## The run card is shared input

The committed run cards are read verbatim by both MadGraph and this crate, and
the run card MadGraph actually ran is part of the reference:
- **MadGraph chooses settings per process.** `gg_to_gg_cg` ran at
  `dynamical_scale_choice = 3` because SMEFTsim's `cG` vertex makes `banner.py`
  force H_T/2; `wpwm_to_wpwmz_cw` ran at `nhel = 1` (Monte Carlo over helicities);
  the default card MadGraph writes for a mixed-multiplicity jet process turns on
  `ickkw = 1`, `xqcut = 30` (`banner.py:4924-4966`). None of these was in the
  script. Read the card from `output/<row>/Cards/run_card.dat`, not from the
  script.
- **A hand-written card gets system defaults.** A parameter absent from the card
  takes `sys_default`, not the full card's value. `event_norm`'s `sys_default` is
  `sum` (`banner.py:4298`), so a minimal card writes `IDWTUP = -3`, where σ is
  the **sum** of `XWGTUP`; MadGraph's own full cards name `average` and write
  `-4`, where σ is the mean. The sample reader dispatches on `IDWTUP` and panics
  on a value it does not know[^n27-b4].
- **`use_syst = T` with an error-set PDF** appends a few hundred `<wgt>` lines a
  record: 4.07 GB of text for 200000 events. Cards that bank events set
  `use_syst = F`, which changes neither σ nor any event field. Under `use_syst`,
  MadGraph forces `alpsfact = 1`.
- **The seed record is the banner**, not the card's `iseed` line, which MadGraph
  resets. The generators check each run card against its script's launch block
  and each banner for the settings that must hold (for example `vector_size = 1`).

## Output behaviours a reader must know

- **The standalone parameter card is baked in.** The generated Fortran's
  `SETPARA` ignores a runtime card: `param_read.inc` statically includes the card
  baked at generation time. A comparison at another card means regenerating. The
  restrict card's parameters must be the ones both sides use: an amplitude probe
  that silently ran a massive τ against a massless-baked reference carried an
  O((m_τ/E)²) ≈ 3e-4 floor under every per-diagram comparison until it was
  found[^n12-roots]. Match parameter provenance first
  ([bit-exact-amplitude-debugging](bit-exact-amplitude-debugging.md)).
- **3.7.1 writes the colour matrix packed**: integers over one `DENOM`, upper
  triangle only, contracted with `DO J = I, NCOLOR`. Off-diagonal entries carry
  twice the symmetric value: `CF(I,I) = packed/DENOM`,
  `CF(I,J) = CF(J,I) = packed/(2·DENOM)`. 3.5.x wrote a square real array.
  Hand-written parsers must take both; the f2py path executes MadGraph's own
  contraction and is unaffected.
- **The banked `.lhe` is written twice.** `rw_events.f` writes it, then MadEvent's
  Python reads it back through `lhe_parser.py` and writes it again. If that
  read-back ran in `wgt_only` mode, every field except the rescaled weight passes
  through as text (dialect F: `0.2500000E+03`, `0. 1.`); a full parse
  re-serialises it (dialect P: `2.50000000e+02`, `+0.0000000000e+00`). Which one
  survives tracks whether the systematics step forced a full parse, which tracks
  `use_syst`, which tracks the beams. It is a fast path, not a defect, and not
  reachable from the run card[^n27-b5]. The reader takes both; the round-trip
  gate keeps each file's own spelling ([event-output-gates](event-output-gates.md)).
  On the event line the scales and couplings carry `rw_events.f`'s seven
  significant digits; MadGraph's Python prints the same fields two digits wider,
  and those two digits are padding.
- **`SCALUP` is `sqrt(max(q2fact(1), q2fact(2)))`**, the factorisation scale, not
  μR; μR is in `AQCDUP` as `αs(μR)`, with π truncated
  ([madgraph-defects](madgraph-defects.md),
  [scales-pdf/record-scales](../scales-pdf/record-scales.md)).
- **3.7.1 no longer prints the per-scale `αs` diagnostic**: `setclscales` moved
  into a vectorised `reweight.f` and its 17-digit `alpha_s for scale` line is
  commented out. The `New value of alpha_s from PDF lhapdf` line still prints.
- **`<MGRunCard>`** in the banner is the card after `banner.py`'s own edits,
  wrapped in `<![CDATA[ … ]]>`; it is not the resolved card `setcuts.f` and
  `setrun.f` later act on ([events/pythia-interop](../events/pythia-interop.md)).
- **2 → 1 rows** leave `matrix1_optim.f` as the un-recycled per-helicity
  `MATRIX1`, write one `AMP2` accumulator under a fake channel id, and bank no σ
  or events (MadEvent has no volume to integrate).
- **Generated text that varies run to run**: the order of `FK_*` declarations in
  `matrix1_orig.f`; `configs.inc`'s `fake_id` lines differ between 3.5.7 and
  3.7.1 while building the same channels.
- **A checkout without `.git`** makes MadGraph print a "development version"
  banner and ANSI escapes into the LHE banner; `build.sh` strips the escapes.
- **Multi-group runs do not regenerate bit-identically.** `pp_to_jj`'s five
  subprocess directories make its unweighting draw scheduling-sensitive; compare
  such runs by distribution
  ([refdata-banking-procedure](refdata-banking-procedure.md)).
- **MadEvent's quoted error is not its spread**; references are read under the
  seed policy, never from one run's quote.

The LHE format strings, field by field, are in
[madgraph-lhe-output](../references/codebases/madgraph-lhe-output.md).

## Measured inconsistencies inside MadEvent

Not located in MadGraph's code, but each bounds how a MadEvent number may be
read (code-located defects are [madgraph-defects](madgraph-defects.md)):
- **Quoted errors are not spreads.** Seeds have scattered at χ²/dof up to 93
  (`e+ e- > e+ e-`) against their own quotes. Seeds run in one shared
  directory each inherit their predecessors' grids and are not independent
  draws: on the MLM rows they scattered *less* than they quoted (χ²/dof
  0.3–0.8 across the rows, only the top-pair `@0` at 1.03). The
  shared-directory `pp_to_ll_0j2j_mlm` reference, whose seeds read χ²/dof
  0.4–0.7, read `@1` 0.43% high against 21 independent directories.
  The seed policy and one freshly generated directory per seed answer
  both[^n41-z].
- **A `dummy_cuts`-windowed run understates its own seed spread**, by about 2× in
  two `pt(γ)` windows of `ee_to_mumua`.
- **A run's unweighted sample can contradict its own windowed σ.** On
  `ee_to_mumua`, MadEvent's sample puts 8.73% of σ in `pt(γ) ∈ [10, 20)` against
  9.40% from its own windowed runs (about 20σ), in both 3.5.7 and 3.7.1, and two
  complete `dummy_cuts` partitions of one run's phase space (in `pt(γ)` and in
  `m(μμ)`) disagree with each other by 16.7σ[^n29-d]. The open part is
  [ee-mumua-radiative-return-sigma-high](../backlog/validation/ee-mumua-radiative-return-sigma-high.md);
  the method is [windowed-partition-closure](windowed-partition-closure.md).
- **Not bit-reproducible**: multi-group unweighting is scheduling-sensitive, and
  the mixed MLM rows' runs differ across hosts.

## How many rows

Do not quote a row count from a note: read `validation/manifest.toml` (the
`[[process]]` rows) or the collator's census line. Banked σ values live in the
committed JSON tables and the bundle, not in prose.

[^manifest]: `validation/manifest.toml`, header "MadEvent references: the seed policy".
[^n27-b4]: Note 27 B4: `event_norm`, the packed `CF`, and the `use_syst` file sizes.
[^n12-roots]: Note 12, oracle defects 6′ and 7′.
[^n27-b5]: Note 27 B5, "The LHE dialect blocker", and the findings register.
[^n41-z]: Note 41 Z1 record (the independent-directory reference); note 38 §8.4 for Bhabha.
[^n29-d]: Note 29 chain D, runs D.M4 and D.M7 and the `m(μμ)` axis.
