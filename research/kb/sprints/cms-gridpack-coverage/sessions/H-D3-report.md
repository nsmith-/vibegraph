---
type: Session Report
title: "H-D3 report: the 11 duplicate-subprocess gridpacks against MadGraph; two decay assignments on two lines are two subprocesses too"
description: "MadGraph's generation of all 11 survey cards, measured on the pinned checkout: every flavour content two decay assignments reach is two (process, decays) entries, on one line (Zto2Q, Wto2Q, VLLD) or on two (Wto2Q's W+ and W- lines, TTToHcToWA, TpTp, VLLVLL). H-D2 lifted only the one-line case, so 7 of the 11 cards still stopped at the cross-line DuplicateSubprocess refusal. Two lines whose sets force different decay assignments are now kept as MadGraph keeps them; one assignment on two lines, or a decay chain and a plain line, stay refused. vibegraph's enumeration matches MadGraph's per-entry diagram counts and doubly reached contents on every BSM card (890 subprocesses, 417 doubly reached) and on the SM families' lines (Wto2Q only up to a leg-order difference on b̄-initial crossings). Census rows: the five-flavour Zto2Q line and two two-line SM cards. After enumeration the cards stop at GroupCutsDiffer (Zto2Q, Wto2Q, TTToHcToWA), massive incoming u (VLLD, VLLVLL, item 6) and a colour-basis split (TpTp)."
status: draft
tags: [decay-chains, madgraph-parity, cms, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: hd1, resource: "H-D1-report.md", title: "H-D1 report: MadGraph sums the two decay assignments incoherently"}
  - {id: hd2, resource: "H-D2-report.md", title: "H-D2 report: a subprocess reached by two decay assignments is two subprocesses"}
  - {id: mg-optimize-order, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L5643", title: "MadGraph madgraph_interface.py optimize_order, applied by do_define"}
  - {id: mg-import-file, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/extended_cmd.py#L1691", title: "MadGraph extended_cmd.py import_command_file, which strips every line"}
---
Session H-D3 of sprint H, item `decay-chain-duplicate-subprocess-vs-madgraph`.
Worktree `/home/user/wt-decay-duplicate`, branch `sprinth-decay-duplicate` at
`e7442ee`, verified before any action:

```
$ cd /home/user/wt-decay-duplicate && git rev-parse --show-toplevel && git branch --show-current
/home/user/wt-decay-duplicate
sprinth-decay-duplicate
```

Mid-session the manager asked for `origin/main` (#88, `d571a5d6`) to be merged in.
That is merge commit `b78d280d`. Step 1 was rerun on `d571a5d6` (§1). Scratch, none
committed, is under
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/H-D3/`
(below: `$S`). It holds:

- `cards/`: a copy of the host's 11 directories;
- `ufo/`: the three bundled models, each `ufomodel/` copied under its import name,
  plus `restrict_type1_5FS.dat` for 2HDM;
- `mg5/`: a copy of the pinned MadGraph with the three models and
  `restrict_ckm_no_b_mass.dat` added under `models/`;
- `mgjson/`: MadGraph's census per card or line;
- `scripts/`: `H-D3_mg_census.py`, `H-D3_mg_dups.py`, `H-D3_bundle.py`,
  `H-D3_cli_all.sh`;
- `scratch_tests/hd3_census.rs`: the census test reading a scratch JSON and a UFO
  directory; it sat in `vibegraph-lib/tests/` only while it ran;
- `runs/<binary>/<card>/log.txt`: CLI logs;
- `H-D3_*.log`.

Every MadGraph number below comes from `census()` of
`validation/madgraph/dump_decay_chain_census.py`: `MasterCmd`, line by line,
then `HelasMultiProcess`, the generation `output` runs. No process directory was
written, so there were no build products to delete.

## 1. Main's refusal reproduces (step 1)

Binaries: `vibegraph-main` built at `fa10e2d6` and `vibegraph-main2` built at
`d571a5d6` (post-#88). Each was built in its own detached checkout of `origin/main`
at `/home/user/H-D3-main`. Command per card (`$S/scripts/H-D3_cli_all.sh`):

```
vibegraph integrate <card>/proc_card_mg5.dat --run-card <card>/run_card.dat \
  --param-card <card>/param_card.dat [--ufo-dir $S/ufo] --neval 1000 --niter 1 \
  --fixed-budget --seed 1 -j 2 --no-tui --yes --out …
```

`VIBEGRAPH_HOME=$S/vghome` holds the downloaded LHAPDF 325300/325500. With
`--no-network` the CLI stops at the PDF index before it enumerates.

The output of `vibegraph-main2`, `d571a5d6` (`H-D3_cli_main2.log`,
`H-D3_cli_main2_tt.log`), is below. Each first error is verbatim up to the first
`;`, and identical to `refusal.txt`:

| card | proc card | first error |
|---|---|---|
| TTToHcToWA ×2 | `proc_card_mg5.dat` | `cannot parse the proc card …: Process parse error: 't~>' in 't~> b~ w-' is not a particle` |
| TTToHcToWA ×2 | `InputCards/…_proc_card.dat` | `subprocess 'g g > b u d~ b~ mu+ mu- e- ve~' is produced by two process lines ('p p > t t~, (t > b h+, (h+ > h3 j j, h3 > mu+ mu-)), (t~ > b~ w-, w- > l- vl~)' and 'p p > t t~, (t > b w+, w+ > j j), (t~ > b~ h-, (h- > h3 l- vl~, h3 > mu+ mu-))')` |
| TpTp | as-run | `subprocess 'g g > b b~ t mu+ mu- t~' is produced by two process lines ('p p > tp tp~, (tp > s0 t, s0 > mu+ mu-), (tp~ > s0 t~, s0 > b b~)' and 'p p > tp tp~, (tp > s0 t, s0 > b b~), (tp~ > s0 t~, s0 > mu+ mu-)')` |
| VLLD_ele / VLLD_mu | as-run | `subprocess 'u u~ > H e- Z e+'` / `'u u~ > H mu- Z mu+' is produced by two process lines ('p p > L L, L > boson lepton' and 'p p > L L, L > boson lepton')` |
| VLLVLL ×2 | as-run | `subprocess 'u u~ > mu+ mu- e- Z e+' is produced by two process lines ('p p > lp lp~, lp > z e-, (lp~ > h e+, h > mu+ mu-)' and 'p p > lp lp~, (lp > h e-, h > mu+ mu-), lp~ > z e+')` |
| Wto2Q ×2 | as-run | `subprocess 'u c > u d~ c d' is produced by two process lines ('p p > w+ j j $ t t~ h, w+ > j j' and 'p p > w+ j j $ t t~ h, w+ > j j')` |
| Zto2Q ×2 | as-run | `subprocess 'g g > u u~ c c~' is produced by two process lines ('p p > z j j, z > j j' and 'p p > z j j, z > j j')` |

`vibegraph-main` (`fa10e2d6`, `H-D3_cli_main.log`) gave the same 11 lines.

**#88 changes nothing here.** Every bundled model loads under both binaries with
no warning (`grep -c WARN` is 0 in all 13 logs). The first error is the same text.
Only the model digest in the `ufo_load` line differs: VLL `10196b5b3416…` →
`2912b0310291…`, eVLQS012 `fb03826ed475…` → `7c1c39892277…`. sm-ckm_no_b_mass is
unchanged (`73b3d9782dad…`). The branch rows of §4 run on the merged tree.

**The as-run TTToHcToWA card is not readable by MadGraph either.** Its
`proc_card_mg5.dat` breaks `(t~\` and ` > b~ w-` across a line with a backslash.
`import_command_file` strips each line before `precmd` joins the continuation
(`extended_cmd.py:1715`, `:1018`), so MadGraph reads `(t~> b~ w-`. Fed line by
line to MadGraph's own `MasterCmd`:

```
$ MG5AMCNLO_PATH=$S/mg5 python3 - <<EOF   # census() on the raw file, output line dropped
…
InvalidCmd: No particle t~> in model 0 0
```

So vibegraph's parse error is MadGraph's own refusal of that file. The gridpack
ran its `InputCards/` card, which reaches the survey's refusal (table above).
Everything below uses the `InputCards/` card for TTToHcToWA. For the other nine,
the two cards' `import`, `define`, `generate` and `add` lines agree up to the
defines MadGraph adds to its history (the 4F defaults and its 5F rewrite
`define p = 21 2 4 1 3 -2 -4 -1 -3 5 -5`, `define j = p`).

**Card 024.** The host's note holds. The Wto2Q card's line is
`p p > w+ j j $ t t~ h, w+ > j j` under `sm-ckm_no_b_mass` in five flavours. With
the CKM, `u c > u d~ c d` is W⁺ → u d̄ on the `u c > w+ c d` core and W⁺ → c d̄ on
the `u c > w+ u d` core (V_cd). H-D1 used the built-in SM, whose CKM is the
identity, so it could not see this.

## 2. MadGraph's behaviour on the 11 cards (step 2)

`python3 $S/scripts/H-D3_mg_census.py <card> <out.json> [lines]` runs `census()`
on the card. `[lines]` optionally keeps only some generate/add lines, along with
every import, define and set line. Then `python3 $S/scripts/H-D3_mg_dups.py`
keys every entry by (sorted initial, sorted final) (`H-D3_mg_all.log`; output
verbatim, abridged to the counts):

| card (lines) | MEs | entries | contents | covered twice (×3) | two entries in one ME | across process ids | s |
|---|--:|--:|--:|--:|--:|--:|--:|
| TTToHcToWA_MA125 (all 6) | 88 | 396 | 198 | 198 | 0 | 198 | 26 |
| TTToHcToWA_MA15 (all 6) | 88 | 396 | 198 | 198 | 0 | 198 | 26 |
| TpTp (both) | 4 | 10 | 5 | 5 | 0 | 5 | 1 |
| VLLD_ele (all 3) | 36 | 36 | 32 | 4 | 0 | 0 | 1 |
| VLLD_mu (all 3) | 36 | 36 | 32 | 4 | 0 | 0 | 1 |
| VLLVLL_250 (both) | 8 | 8 | 4 | 4 | 0 | 4 | 1 |
| VLLVLL_500 (both) | 8 | 8 | 4 | 4 | 0 | 4 | 0 |
| Zto2Q `@0` | 16 | 75 | 75 | 0 | 0 | 0 | 1 |
| Zto2Q `@1` | 89 | 475 | 415 | 60 | 4 | 0 | 4 |
| Zto2Q `@2` | 131 | 725 | 565 | 160 | 8 | 0 | 18 |
| Zto2Q `@3` | 406 | 2425 | 1525 | 840 (60 ×3) | 22 | 0 | 319 |
| Wto2Q `@0`+`@3` | 150 | 216 | 216 | 0 | 0 | 0 | 3 |
| Wto2Q `@1`+`@4` | 1028 | 2160 | 1776 | 384 | 0 | 258 | 31 |
| Wto2Q `@2`+`@5` | 1590 | 3456 | 2586 | 786 (84 ×3) | 0 | 618 | 166 |
| Wto2Q `@1` / `@4` alone | 514 | 1080 | 1017 | 63 | 0 | 0 | 15 |
| Wto2Q `@2` / `@5` alone | 795 | 1728 | 1602 | 126 | 0 | 0 | 70 |

How the table was built:

- The Zto2Q lines were run one at a time. Lines of different multiplicity cannot
  share a content.
- The Wto2Q W⁺ and W⁻ lines of one multiplicity were run together. A neutral
  final state can come from both, for example `g g > u d~ d u~`: W⁺ → u d̄ on
  `g g > w+ d u~`, and W⁻ → d ū on `g g > w- u d~`. This was missed at first and
  then found by the branch CLI (§4).

Reading:

- **Every doubly covered content is two different decay assignments.** No entry
  is repeated. Within a line, MadGraph keeps them as two matrix elements, or as
  two PDF terms of one where the tags agree. In five flavours that is
  Z → ss̄ on the bb̄ core and Z → bb̄ on the ss̄ core (`ME6: g g > s s~ b b~, g g > b b~ s s~`).
- **Across lines, MadGraph sums them too**, always as different matrix elements
  (different process ids). This covers all of TTToHcToWA (198 of 198 contents:
  `@0` with `@4`, `@1` with `@3`, `@2` with `@5`), TpTp, VLLVLL and 258 Wto2Q contents.
- **Two families differ only in masses or HT bins.**
  - Within Wto2Q, Zto2Q, TTToHcToWA and VLLVLL, the proc cards' import, define and
    process lines are identical, as shown by `diff` with the `output` line
    removed.
  - The run cards, with values sorted and hidden-parameter order ignored, differ
    only in `ihtmin`/`ihtmax` (Wto2Q 400/800 against 800/1500, Zto2Q 100/400
    against 800/1500).
  - The param cards differ only in `MH3` 125/15 (TTToHcToWA) and in
    `mlp`, `mh` 250/95 against 500/80 (VLLVLL).
  - VLLD's two cards differ in `define lepton`/`neutrino` (e against μ) and in
    `mlp`/`mvlp` 600 against 1150.
  - The census above has MadGraph identical for each pair up to that relabelling.

## 3. The departure H-D2 left, and the fix

H-D2 removed the one-line refusal and kept the check across lines
(`diagrams/mod.rs`, `enumerate`). Seven of the 11 cards go through that check:
TTToHcToWA ×2, TpTp, VLLVLL ×2 and Wto2Q ×2. On the branch before this session,
`vibegraph-branch` stopped them as follows (`H-D3_cli_branch.log`):

- TpTp and VLLVLL stopped with main's message.
- Wto2Q passed its one-line duplicates and then stopped at
  `subprocess 'g g > d u~ u d~' is produced by two process lines ('p p > w+ j j $ t t~ h, w+ > j j' and 'p p > w- j j $ t t~ h, w- > j j')`.

Decision D3 says that both decay assignments of one flavour content are kept as
separate subprocesses and summed incoherently. In these seven cards the
assignments are written on two lines. Measured against MadGraph, that is the same
physics as on one line. **The brief's premise, "H-D2 made a flavour content
reached by two decay assignments become two subprocesses", held only for 4 of the
11 cards.** The change below extends D3 to two lines. **It is a deviation from the
brief's scope** (see the summary for the manager).

**Rule (`diagrams/mod.rs`, `assignments`, `distinct_assignments`).**

- Two sets of one content from two lines are kept when both are decay chains and no
  decay assignment is shared. An assignment is, per diagram, the sorted list of
  (forced-line particle, sorted particles of the final-state legs it decays to).
  That form does not depend on leg order.
- Everything else falls through to the check as it was: helicity states that
  overlap are refused. That covers:
  - one assignment on two lines (`@1`/`@2` of the same chain);
  - a chain line against a plain line, which contains the resonant diagrams;
  - two plain lines.

**Tests** (`tests/diagrams.rs`):

- `two_assignments_on_two_lines_are_two_subprocesses`. It replaces
  `two_assignments_on_two_lines_are_refused` and uses H-D1's equal-tag card
  `e+ e- > z u u~, z > c c~ @1` + `e+ e- > z c c~, z > u u~ @1`, which MadGraph
  makes one matrix element with two PDF terms. It also uses
  `g g > w+ d u~, w+ > u d~` + `g g > w- u d~, w- > d u~`. Each set must keep its
  own forced line.
- `one_assignment_on_two_lines_is_refused`: the same chain twice, chain plus plain,
  and plain plus chain.
- `a_subprocess_reached_from_two_lines_is_refused` is unchanged and passes.

Before the fix (M0, `mod.rs` from `HEAD`, everything else as committed; `H-D3_mut0.log`):

```
test two_assignments_on_two_lines_are_two_subprocesses ... FAILED
: subprocess 'e+ e- > u u~ c c~' is produced by two process lines ('e+ e- > z u u~, z > c c~' and 'e+ e- > z c c~, z > u u~'); …
test result: FAILED. 3 passed; 1 failed; …
```

and the census gate with M0 (`H-D3_census_mut0.log`):

```
unexpected enumeration error: subprocess 'e+ e- > u u~ c c~' is produced by two process lines …
test result: FAILED. 0 passed; 1 failed
```

Mutation M1 keeps every pair of chains (`(Some(_), Some(_)) => true`) and is
caught (`H-D3_mut1.log`):

```
test one_assignment_on_two_lines_is_refused ... FAILED
test result: FAILED. 3 passed; 1 failed; …
```

`cmp` against the saved fixed `mod.rs` printed `restored` after each mutation.

What the rule cannot see is a diagram that contains both assignments'
resonances. The same graph would then sit in both sets, once with each
assignment's lines forced. MadGraph counts it in both matrix elements, and so do
the two sets of one line since H-D2. So this follows MadGraph and is not a guard
against it.

## 4. vibegraph's enumeration against MadGraph, per card

**Harness.** `$S/scratch_tests/hd3_census.rs` is `tests/decay_chain_census.rs`
with three changes:

- the census JSON comes from `HD3_JSON`;
- a non-SM model is `UFOModel::load($HD3_UFO/<name>, restrict_<variant>.dat)`;
- a refusal is reported instead of panicking.

It compares the (process, decays) keys, the per-entry diagram count on
MadGraph's decay blocks, and the number of doubly reached contents. It ran on the
merged, fixed tree (`H-D3_vg_bsm_merged.log`, `H-D3_vg_sm_ord_merged.log`;
verbatim):

```
TT125: ok=true here 396 sets / 1984 diagrams, MadGraph 396 entries / 1312 diagrams (per entry); shared contents here 198 MadGraph 198
TT15: ok=true here 396 sets / 1984 diagrams, MadGraph 396 entries / 1312 diagrams (per entry); shared contents here 198 MadGraph 198
TpTp: ok=true here 10 sets / 14 diagrams, MadGraph 10 entries / 14 diagrams (per entry); shared contents here 5 MadGraph 5
VLLD_ele: ok=true here 36 sets / 68 diagrams, MadGraph 36 entries / 52 diagrams (per entry); shared contents here 4 MadGraph 4
VLLD_mu: ok=true here 36 sets / 68 diagrams, MadGraph 36 entries / 52 diagrams (per entry); shared contents here 4 MadGraph 4
VLLVLL250: ok=true here 8 sets / 16 diagrams, MadGraph 8 entries / 16 diagrams (per entry); shared contents here 4 MadGraph 4
VLLVLL500: ok=true here 8 sets / 16 diagrams, MadGraph 8 entries / 16 diagrams (per entry); shared contents here 4 MadGraph 4
7 cards: 7 stitched and matched (890 subprocesses, 417 of them reached by two decay assignments)
Z_l0: ok=true here 75 sets / 170 diagrams, MadGraph 75 entries / 150 diagrams (per entry); shared contents here 0 MadGraph 0
Z_l1: ok=true here 475 sets / 3640 diagrams, MadGraph 475 entries / 2600 diagrams (per entry); shared contents here 60 MadGraph 60
Z_l2: ok=false here 725 sets / 35530 diagrams, MadGraph 725 entries / 23000 diagrams (per entry); shared contents here 160 MadGraph 160
W_l03: ok=true here 216 sets / 432 diagrams, MadGraph 216 entries / 432 diagrams (per entry); shared contents here 0 MadGraph 0
W_l14: ok=false here 2160 sets / 8184 diagrams, MadGraph 2160 entries / 7200 diagrams (per entry); shared contents here 384 MadGraph 384
W_l25: ok=false here 3456 sets / 76320 diagrams, MadGraph 3456 entries / 64512 diagrams (per entry); shared contents here 786 MadGraph 786
  Z_l2: subprocesses differ (40 only here, 40 only MadGraph):
  W_l14: subprocesses differ (96 only here, 96 only MadGraph):
  W_l25: subprocesses differ (408 only here, 408 only MadGraph):
```

"here … diagrams" counts every pairing of identical particles. "MadGraph …
(per entry)" sums each entry's matrix-element count. `ok` is the per-entry
diagram count on MadGraph's blocks, which is equal wherever the keys meet.

- **The BSM families match MadGraph entry for entry**, with the cross-line pairs
  of TTToHcToWA, TpTp and VLLVLL included. Before the fix, the same harness
  refused 5 of 7 with the cross-line message (`H-D3_vg_bsm.log`).
- **The SM families match where the leg order does.** Every set count and every
  doubly reached count agree (60, 160, 384, 786). Two leg-order differences make
  keys disagree, and both lie outside duplicate handling:
  1. **`define` order.** MadGraph's `do_define` sorts the members
     (`optimize_order`: massless first, then colour octet, triplet, singlet,
     antitriplet). `define p = 21 2 4 1 3 -2 -4 -1 -3 5 -5` is stored as
     `[21, 2, 4, 1, 3, 5, -2, -4, -1, -3, -5]` (traced: `define p = … -> p=[21, 2, 4, 1, 3, 5, -2, -4, -1, -3, -5]`).
     vibegraph takes the order as written. Run that way, the as-run card misses
     on 20 of Z_l1's keys and 36 of W_l03's (`H-D3_vg_sm_fixed.log`, e.g. here
     `([-4, 1], [-4, 5, 21])` against MadGraph `([-4, 1], [5, -4, 21])`). With
     the define written in MadGraph's stored order, which MadGraph stores
     identically, Z_l1 and W_l03 match fully.
  2. **Core leg order on crossed initial states with b̄.** What remains is Z_l2,
     W_l14 and W_l25. For example, MadGraph's ME447 is `s~ b~ > w+ s~ c~`
     (final `[2, -5, -3, -4]`) where vibegraph has `[2, -5, -4, -3]`. The core
     `j j` order (s̄ c̄) is not the first one `itertools.product` meets over `j`
     (c̄ s̄). It appears only where both beams are antiquarks and one is b̄. That
     points at MadGraph reusing a crossed process's leg order, not at
     product-order generation.
- **Full cards through the CLI** (§5). The per-line counts of doubly reached
  subprocesses in the enumeration log equal MadGraph's:
  - Zto2Q: 60 / 160 / 840; 3700 subprocesses against 75 + 475 + 725 + 2425
    entries.
  - Wto2Q: 63 / 126 per W line.
  - VLLD: 4.

## 5. Where each card stops now (step 4)

`vibegraph-branch2` is `b78d280d` plus this session's working tree, with the fix
(`H-D3_cli_branch2.log`). Same command as §1, `timeout 900`:

| card | enumeration | first error after it |
|---|---|---|
| TTToHcToWA ×2 (InputCards) | `enumerated 1984 diagrams in 396 subprocesses` | `failed to build the hadronic integrand: groups 0 and 2 compile to different cut filters, so the sum has no single phase-space indicator` (D4) |
| Wto2Q 400–800 | 63 / 126 / 63 / 126 subprocesses from two assignments; `enumerated 84936 diagrams in 5832 subprocesses` | `groups 0 and 34 compile to different cut filters` (D4) |
| Zto2Q 100–400, full card | 60 / 160 / 840; `enumerated 674680 diagrams in 3700 subprocesses (222.210 s)` | none within 900 s: killed (`rc=124`) after compiling 1457 evaluators |
| Zto2Q, lines `@0`+`@1` only (`$S/H-D3_zto2q_l01_proc_card.dat`) | `enumerated 3810 diagrams in 550 subprocesses` | `groups 0 and 1 compile to different cut filters` (D4) |
| TpTp | `enumerated 14 diagrams in 10 subprocesses` | `failed to decompose into flavour groups: subprocesses g g > mu+ mu- t b b~ t~ and u u~ > mu+ mu- t b b~ t~ share a matrix element but not a colour basis` |
| VLLD_ele / VLLD_mu | 4 from two assignments; `68 diagrams in 36 subprocesses` | `subprocess u u~ > Z e- Z e+ has an incoming leg of mass 0.00255 GeV; a parton distribution supplies massless partons` (item 6, `massive-incoming-partons-refused`; the VLL model's `MU = 2.55e-3`) |
| VLLVLL ×2 | `16 diagrams in 8 subprocesses` | the same massive-u refusal (`u u~ > Z e- mu+ mu- e+`) |

The second card of Wto2Q and of Zto2Q (800–1500) differs only in `ihtmin` and
`ihtmax` (§2). Their main-side run is in §1, and they were not rerun on the
branch.

## 6. Census rows (step 3)

`dump_decay_chain_census.py` gains three cards:

| row | card | MadGraph | why |
|---|---|---|---|
| `cms_zto2q_zjj_jj` | `import model sm-ckm_no_b_mass` / `define p = g u c d s b u~ c~ d~ s~ b~` / `define j = p` / `generate p p > z j j, z > j j @1` | 89 MEs, 516 diagrams; 60 contents twice, 4 as two PDF terms of one ME | Zto2Q's first duplicate line, five flavours; the define written in MadGraph's stored order (§4, item 1) |
| `ee_zuu_cc_two_lines` | `generate e+ e- > z u u~, z > c c~ @1` / `add process e+ e- > z c c~, z > u u~ @1` | 1 ME, two PDF terms | the cross-line case with equal tags |
| `gg_wpjj_wmjj_two_lines` | `generate g g > w+ j j, w+ > j j @1` / `add process g g > w- j j, w- > j j @2` | 2 MEs, 16 diagrams; 4 contents, each from both lines | the Wto2Q cross-line mechanism, built-in SM |

How the census carries a model:

- **Python side.** An `import model sm-<r>` whose restriction the pinned tree
  lacks, but which `vibegraph-lib/src/ufo/sm_assets/` bundles (`ckm_no_b_mass`),
  is pointed at a temporary copy of `models/sm` that carries it (`_model_line`).
  The card in the JSON keeps its own import line.
- **Rust side.** `tests/decay_chain_census.rs` now stitches each card under the
  SM restriction it imports (`model_for`; before, always the default).
- **Other models.** The census cannot carry a non-SM model. A BSM card would need
  the UFO in the repository, and TTToHcToWA (2HDM_UFO), TpTp (eVLQS012_v8_4FNS_UFO),
  VLLD and VLLVLL (VLL) all need one. They are not vendored (licences unchecked).
  Their comparison is §4's scratch harness. The Rust side would take an
  `UFOModel::load` path in a few lines, as the scratch harness shows.
- **Wto2Q gets no row.** Its smallest duplicate-reaching piece (`@1`, or `@1`+`@4`)
  fails on the b̄-crossing leg order of §4 (48 keys on `@1` alone,
  `H-D3_vg_w1.log`), which is unrelated to duplicates. Its cross-line mechanism is
  pinned by `gg_wpjj_wmjj_two_lines`.

Determinism: the regenerated JSON is pure insertions against the committed one
(`git diff --stat`: `18033 +`, no `-` lines).

```
$ python3 validation/madgraph/dump_decay_chain_census.py
cms_zto2q_zjj_jj: 89 matrix elements, 516 diagrams
ee_zuu_cc_two_lines: 1 matrix elements, 8 diagrams
gg_wpjj_wmjj_two_lines: 2 matrix elements, 16 diagrams
wrote …/decay_chain_census.json: 30 cards, 2 refused by MadGraph
```

Gate (`H-D3_census_gate.log`):

```
cms_zto2q_zjj_jj: 60 subprocesses from two decay assignments; [1, -1, -5, -5]: 8 (+0 permuted), …
ee_zuu_cc_two_lines: 1 subprocesses from two decay assignments; [2, -2, 4, -4]: 8 (+0 permuted), …; [4, -4, 2, -2]: 8 (+0 permuted), …
gg_wpjj_wmjj_two_lines: 4 subprocesses from two decay assignments; [1, -2, 2, -1]: 8 (+0 permuted), …
30 cards: 27 stitched and matched (842 subprocesses, 107 of them reached by two decay assignments), 3 refused as MadGraph refuses or drops
test result: ok. 1 passed; 0 failed
```

`manifest.toml`'s `decay-chain-census` rationale said 24 cards, which was stale
since H-D2's 27. It now says 30 and names the restriction handling.

## 7. Gates

All commands ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` and `-j 2`, on the merged tree:

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | `GATE fmt 0` |
| clippy | `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | `Finished`, `GATE clippy 0` |
| hermetic suite | `cargo test -j 2 --workspace` | `GATE test 0`; 54 `test result` lines, 1893 passed, 0 failed, 25 ignored (`H-D3_test.log`) |
| banked layer | `CARGO_BUILD_JOBS=2 nohup setsid bash validation/validate.sh` with an untracked `.cargo/config.toml` (`[profile.release-debug] debug = 0`), deleted afterwards | see the addendum below |

## 8. closes_when

- MadGraph's handling of every card the item counts has been measured on the
  pinned checkout. That is now all 11, not only 024, 029 and 068.
- vibegraph matches it:
  - at enumeration, on all 11 (§4, with the two leg-order caveats for the SM
    lines);
  - in the committed census, through `cms_zto2q_zjj_jj` and the two two-line rows.
- The item file is deleted. The survey concept's link to it now names the sprint
  record instead.

## Found

- **`proc-card-grammar.md` describes the old cross-line rule.** It says *"Here
  any concrete subprocess reached from two lines is refused"*
  (`research/kb/process/proc-card-grammar.md:159`). It now holds only for one
  assignment on two lines, or a chain against a plain line. Closing it takes a
  kb edit naming the distinct-assignment exception.
- **`define` does not apply MadGraph's `optimize_order`.**
  - MadGraph sorts every `define`d label's members (massless first, then colour
    8 > 3 > 1 > 3̄), so `define p = 21 2 4 1 3 -2 -4 -1 -3 5 -5` lists b among the
    quarks.
  - vibegraph keeps the written order. The five-flavour p/j rewrite also differs:
    vibegraph appends `b b~`, while MadGraph's `j` comes out as
    `[21, 2, 4, 1, 3, 5, -2, -4, -1, -3, -5]` straight after the import.
  - Effect: different leg orders for the same subprocess, so different
    subprocess labels and LHE particle order. σ is the same. Evidence: §4,
    item 1 (20 of Z_l1's keys).
  - Closing it takes porting `optimize_order` into label resolution, and
    checking which banked references key on leg order.
- **Core leg order on crossed initial states with b̄ differs from MadGraph.**
  `s~ b~ > w+ s~ c~` in MadGraph against `s~ b~ > w+ c~ s~` here (W_l14 96 keys,
  W_l25 408, Z_l2 40; §4, item 2). Closing it takes reading MadGraph's crossing
  reuse in `generate_multi_amplitudes` (`Check for crossed processes`) and
  matching its leg order. It also blocks a Wto2Q census row.
- **TpTp stops at a colour-basis split.** `g g > mu+ mu- t b b~ t~` and
  `u u~ > mu+ mu- t b b~ t~` share a matrix element but not a colour basis (§5).
  This looks like the case of item 1 (colour-basis splits, H-G0); worth checking
  there.
- **VLLD and VLLVLL stop at massive incoming u** (`MU = 2.55 MeV` in the bundled
  VLL model). That is item 6's refusal; these 4 cards belong in its count.
- **The full Zto2Q card did not finish in 900 s.** It took 222 s to enumerate
  674 680 diagrams and was still compiling 3700 subprocesses' evaluators when
  killed. Its first post-enumeration refusal was measured on `@0`+`@1` only.
  Closing it takes a longer run, or a lighter compile, once D4 lands.
- **The as-run `proc_card_mg5.dat` of TTToHcToWA is unreadable by MadGraph.** Its
  history writer splits a line before `>`, and its reader strips the
  continuation (§1). vibegraph refuses it the same way. A survey or `gpdb` attempt
  that reads `proc_card_mg5.dat` rather than `InputCards/` will see a parse error
  on such cards. That is a `madgraph-defects` candidate.
- **The BSM census needs vendored models.** The census Rust side reads only the
  built-in SM. Pinning TTToHcToWA, TpTp, VLLD and VLLVLL in the committed census
  takes vendoring their UFOs, after a licence check, and a model path per case.

## Addendum: banked layer

Filled in at commit time below.
