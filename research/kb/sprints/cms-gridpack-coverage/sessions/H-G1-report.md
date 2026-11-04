---
type: Session Report
title: "H-G1 report: conjugate pairs share one flavour group under the flow permutation"
description: "The review of colour-split-groups, then the manager's scope update: colour_pairing takes π from the fingerprints and checks the CF matrix under π, so conjugate pairs form one group and only genuinely unpaired subprocesses (d d > d d / b b > b b at a zero b Yukawa) split. Mutations, the banked layer, and local MadGraph σ and per-subprocess colour-label oracles on a 2→3 pair and an HT-binned 12-flow pair."
status: draft
tags: [cms, flavour-groups, colour, crossing, sprint-h, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Session H-G1, worktree `/home/user/wt-colour`, branch `colour-split-groups`,
on top of `729e88a` (the CERN host's commit) over `main` `58ab186`. Every cargo
command ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` and two jobs. Scratch work (MadGraph directories,
event files, analysis scripts) is under `/home/user/H-G1-scratch/`; logs named
`H-G1-*` are in the session scratchpad.

## What changed, and why the brief changed under me

The brief asked for a review of `729e88a`, which splits a subprocess that
matches a group's `|M|²` but not its colour bookkeeping into a group of its own.
Mid-session the manager widened the scope on H-G0's diagnosis: the QCD
conjugate pairs have one basis up to the permutation `flow_permutation` already
finds, and the split came only from comparing `cf_matrix()` in each basis's own
order. So the work on top of `729e88a` is:

- `colour_pairing` (`vibegraph-lib/src/proton.rs`) now takes `π` from
  `flow_permutation` first and then requires
  `CF_rep[i][j] == CF_mem[π(i)][π(j)]` exactly. `π` is never derived from the CF
  matrix. The partition loop, the separation-check exemption and the
  per-member reindexing of `729e88a` are unchanged.
- The `729e88a` test is replaced by two:
  `a_conjugate_pair_shares_one_group_under_the_flow_permutation` (`u d > d g u`
  and the 12-flow `u u > g g u u`, each with its conjugate: one group, the
  member's table equal to `conjugated()` of the head's as line connectivity,
  `JAMP2` under `π` to 1e-12 at unfitted points, `π` pinned on the 12-flow pair)
  and `a_pair_whose_flows_cannot_be_paired_is_grouped_apart` (`d d > d d QED=2`
  with `b b > b b QED=2` under the default restriction, a card setting MB and
  ymb to zero, `maxjetflavor = 5`: equal `|M|²`, extra zero-valued Higgs
  diagrams in the b member, no fingerprint pairing, two groups of one).
- kb: `hadronic/flavour-groups` (steps 4–7 and the tests), `events/per-member-colour-flow-tables`
  (design step 2 and the residual-risk paragraph), and
  `amplitudes/colour-flow-lines-and-conjugation` (the degenerate-flow
  paragraph) describe the behaviour as it now is. `grep -rn "ColorStructureDiffers\|ColorFlowPairing" research/kb vibegraph-lib`
  leaves only the backlog item, which I did not edit.

The brief's item 1 ("reproduce both claimed mutations" of `729e88a`) is
superseded: both mutations were of code that no longer exists in that form. The
analogous mutations of the new code are below, plus the fails-before runs on
both `729e88a` and `main`. The brief's claim that `u d > g d u` groups fine on
`729e88a` was not rerun, and that leg order is not measured under the new rule
either (only the `u d > d g u` order is in the test).

## Gates

All on the tree committed as `eda34d1`: the runs started from that source, and
only this report was added after. The mutation runs below predate one
`cargo fmt` rewrap of the new tests (whitespace only).

| gate | command | result |
|---|---|---|
| rustfmt | `cargo fmt --all --check` | exit 0 (after one `cargo fmt --all` that rewrapped four assertions in the new tests) |
| clippy | `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | `clippy exit 0` (`H-G1-clippy2.log`); also clean on `729e88a` before my change (`H-G1-clippy.log`) |
| clippy, banked targets | inside `validate.sh` (`--features vibegraph/extended-validation,vibegraph-lib/extended-validation`) | clean (`validate.sh` exits on lints after the gates; it exited 0) |
| hermetic | `cargo test -j 2 --workspace` | `test exit 0`; 51 targets, 1812 passed, 0 failed, 25 ignored (`H-G1-test2.log`) |
| kb lint | `python3 scripts/kb.py lint` | the concept indexes I touched regenerated (`events/index.md`, `hadronic/index.md`); 3 errors left, all stale sprint indexes (`sprints/index.md`, `sprints/cms-gridpack-coverage/{,sessions/}index.md`), left alone as the brief asks so as not to conflict with `sprinth-plan` |
| banked layer | `bash validation/validate.sh` (pixi is not installed; inputs already on disk, so this is `pixi run --skip-deps validate` without pixi) | `validate exit 0`; 81 targets, 2263 passed, 0 failed, 110 ignored; the collator: "77 rows × 4 categories = 308 cells: 218 measured … (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered). The measured cells are exactly the cells the manifest declares, every gate cell passed" (`H-G1-validate.log`; report copied to `/home/user/H-G1-scratch/H-G1-validation-report/`) |

The gates the `extended-validation` skill maps to a flavour-grouping or colour
change, from that run:

| target | result |
|---|---|
| `validate_hadronic` | 20 passed, 0 failed, 26 ignored |
| `color_cf_oracle` | 112 passed |
| `color_flow_tags_oracle` | 186 passed |
| `amplitude_oracle` | 51 passed |
| `validate_lhef` | 3 passed |
| `validate_samples` / `validate_samples_proton` | 6 passed / 12 passed |
| `validate_unweighting` | 1 passed |
| `cli_generate_proton` | 11 passed |
| `validate_mlm_records` | 1 passed |
| `validate_scales` / `validate_sigma` | 10 passed / 8 passed |
| lib unit tests (release-debug, features on) | 1215 passed, both new tests among them |

The `pp_to_jj` samples cell, the one row whose groups hold conjugate and
crossing members, reads "worst column flavour at chi2 p 0.006" against its
1e-4 floor. The four probe cards above show its partition unchanged, and on an
unchanged partition every member's `π` is the one it had, so no
already-enforced cell can move from this change.

The oracle-layer MLM gates the skill also maps here, run from the banked
build's test binaries (pixi absent):

| gate (task) | command (from `vibegraph-lib` or `vibegraph-cli`) | result |
|---|---|---|
| `validate-mlm-sigma` | `target/release-debug/deps/validate_hadronic-… --ignored --nocapture sigma_llj_xqcut_only_vs_madevent sigma_llj_mlm sigma_ll_0j2j_mlm_vs_madevent sigma_ttx_0j1j_mlm_vs_madevent` | `test result: ok. 5 passed; 0 failed` (`H-G1-mlm-sigma.log`): `pp_to_llj_xqcut_only` rel +0.0019 pull +0.85; `pp_to_llj_mlm` +0.0023 / +1.00; `pp_to_llj_mlm_alps2` +0.0020 / +0.88; `pp_to_ll_0j2j_mlm` total +0.0015 / +1.42 (`@2` +0.0073 / +2.01); `pp_to_ttx_0j1j_mlm` total +0.0010 / +0.95; every row `gate: pass` |
| `validate-mlm-samples` | `target/release-debug/deps/cli_generate_proton-… --ignored --nocapture matched_sample_record_fractions_against_madevent` | `ok`; pulls +0.44, +0.01, −0.01, −0.00 (`H-G1-mlm-samples.log`) |

`validate_mlm_dumps` (`--ignored`) could not run: its per-event replay dumps are
outside the bundle and absent here (`pp_to_llj_mlm: no dump at
…/validation/madgraph/output/ktdump/dumps/pp_to_llj_mlm.jsonl.gz`); 1 passed, 2
failed on that missing input. `validate-mlm-pythia` and `validate-pythia` need
Pythia and were not run.

## Tests that fail before, and the mutations

Each variant is `proton.rs` swapped in whole, rebuilt and run with
`cargo test -j 2 -p vibegraph-lib --lib -- --nocapture --test-threads 1
a_conjugate_pair_shares a_pair_whose_flows h_g1_probe a_conjugate_pair_with_unpaired`
(driver `/home/user/H-G1-scratch/mut/run.sh`, logs
`/home/user/H-G1-scratch/mut/H-G1-<variant>.log`), and the tree was restored
byte for byte after (`cmp` clean).

| variant | change | result |
|---|---|---|
| new code | as committed | both new tests `ok` (48/48 `proton::` lib tests pass) |
| `729e88a` + new tests | the CERN commit's raw-order CF check | conjugate test FAILED at the group count: `left: 2, right: 1` (`u d > d g u`); the split test `ok`; the old `a_conjugate_pair_with_unpaired_colour_bases_is_grouped_apart` `ok` |
| `main` + new tests | before the split existed | conjugate test FAILED: `a conjugate pair is one group: ColorStructureDiffers { a: "u d > d g u", b: "u~ d~ > d~ g u~" }`; split test FAILED: `ColorFlowPairing { a: "d d > d d", b: "b b > b b", reason: "flow 1 of the representative has no counterpart in the member's basis" }` |
| M1 | CF compared in raw order (`theirs[i*n+j]`) | conjugate test FAILED, `left: 2, right: 1` |
| M2 | `π` replaced, on the 12-flow pair, by another CF-consistent pairing | log `H-G1 mutation: pi [7, 3, 5, 1, 8, 10, 9, 11, 6, 2, 4, 0] replaced by [0, 2, 4, 6, 11, 10, 9, 8, 1, 3, 5, 7]`; the group **formed** (the CF check passed it) and the test FAILED at the conjugation assertion (`u u > g g u u`, flow 0 connectivity `{[(0,1),(4,1)], …}` against `{[(0,1),(3,1)], …}`), before the pinned-`π` line |
| M3 | separation-check exemption disabled (`if false && …`) | split test FAILED: `DegenerateGroups { a: "d d > d d", b: "b b > b b", rel: 0.0 }` |

M2 is the blind-spot demonstration H-G0 asked for: the CF check under `π`
cannot see a pairing error inside the CF matrix's automorphism group, and the
`conjugated()` connectivity check, derived without fingerprints, does.

## Does the rule change any partition that grouped before?

The raw-order check and the check under `π` could in principle differ on a
pair whose `π` is not the identity. A scratch probe (uncommitted, run as part of
variant M0) walked every joinable pair (equal polarizations, `|M|²` within
`GROUP_REL_TOL`, equal `n_flows`) of four cards, and replayed the old rule's
partition beside `derive_flavor_groups`:

```
p p > j j: 65 subprocesses, joinable pairs 432, no pi 0, raw-equal 432, equal under pi 432, raw-but-not-pi 0, pi-but-not-raw 0
p p > j j: raw-order rule 7 groups (refused: false); new rule Ok((7, 65))
p p > l+ l- j QCD=2 QED=2: 24 subprocesses, joinable pairs 36, … raw-equal 36, equal under pi 36, raw-but-not-pi 0, pi-but-not-raw 0
p p > l+ l- j QCD=2 QED=2: raw-order rule 6 groups (refused: false); new rule Ok((6, 24))
p p > j j j: 97 subprocesses, joinable pairs 344, … raw-equal 344, equal under pi 344, raw-but-not-pi 0, pi-but-not-raw 0
p p > j j j: raw-order rule 15 groups (refused: false); new rule Ok((15, 97))
p p > t t~ j: 13 subprocesses, joinable pairs 18, … raw-equal 18, equal under pi 18, raw-but-not-pi 0, pi-but-not-raw 0
p p > t t~ j: raw-order rule 4 groups (refused: false); new rule Ok((4, 13))
```

On these cards the two checks agree pair by pair, so every already-gated
partition is unchanged. The cards that differ are exactly the ones the old rule
refused.

## Review of the downstream consumers

Every subprocess still belongs to exactly one group; what moves is whether a
conjugate member sits in its head's group (now) or in a group of its own
(`729e88a`, and still for unpaired pairs). I read each consumer for an
assumption that groups are distinct, that their traces differ (H-G0's Found 3),
or that a subprocess appears once.

| consumer | where | finding |
|---|---|---|
| σ and its per-group terms | `ProtonIntegrand::select_event`, `per_group_sum` (`proton.rs`) | per group, `spin_color_average × Σ_ordering lumi × m2`; two groups with one trace are two terms of the sum, each over its own members. Nothing compares traces. |
| multichannel / VEGAS setup | `ProtonIntegrand::build` (`proton.rs:~1680`) | channels are `(group, diagram)`; with `merge_maps` two channels with the same `map_identity()` share one grid, which is how a split group's channels merge with its origin's. That is the design ("the point's value never reads which pair drew it"), not an assumption of distinct traces. `AmplitudeMismatch` is `ptr::eq` per group, so distinct evaluators per group are required and given. |
| `GroupCutsDiffer` | `ProtonIntegrand::build` | still a refusal; a split group whose cuts differ from another group's is refused there instead of by `CutIndicatorDiffers`. A conjugate pair has the same cut classes, so it is never reached by these pairs. |
| event writing | `flavor_records` (`generate.rs:775`), `member_offsets` (`generate/source.rs:331`), `part_process_ids` (`vibegraph-cli/src/integrate.rs:960`) | per group, per member; process ids are deduplicated; the init block splits XSECUP by event counts per process id. `subprocess_count()` sums members, unchanged by where a member sits. |
| MLM bookkeeping | `group_scales`, `group_rewgt`, `member_rewgt`, `group_records` (`proton.rs`) | sized and indexed per group and per member; no cross-group lookup. |
| `split_by_outgoing_masses` | `multiplicity.rs:108` | runs before `derive_flavor_groups` on diagram sets; independent of the partition. |
| `$` on-shell vetoes | `check_group_members`, `group_vetoes` (`onshell.rs:445`) | keyed per member by flavour; per group. |
| resonances | `group_resonances` (`generate.rs:1308`) | per group and member. |

The separation-check exemption (pairs of groups with `rel < GROUP_REL_TOL` whose
heads fail `colour_pairing`):

- **Can it hide a real knife-edge partition?** No case it exempts was ever
  checked. Before the split existed, two heads that close and polarized alike
  could not both be heads: the second would have tried to join the first and
  been refused. A colour-compatible pair that close is still one group (the join
  takes the first compatible head), so it never reaches the check, and a pair in
  `[GROUP_REL_TOL, GROUP_SEPARATION_MIN]` is still refused whatever its colour.
- **Can a colour-incompatible pair that close be one where merging was wrong
  anyway?** Splitting is always safe for σ: each group is evaluated with its own
  compiled amplitude, so the sum does not depend on the two traces being equal.
  The `d d`/`b b` test pins it (`rel = 0.0`, two groups, unfitted traces agree).
- **Order dependence.** A subprocess that fails pairing with head A is offered to
  every later head and joins the first compatible one, which may be another split
  group B. The colour relation is an equivalence (equal `n_flows`, equal
  fingerprint multisets with no duplicates — which makes `π` the unique
  fingerprint bijection — and CF equal under it, all transitive by composing the
  unique bijections), and the `|M|²` agreement is exact in practice, so the
  partition is the set of classes whatever the order; only the choice of
  representative depends on the enumeration order, and members' `JAMP2` agree
  with it under `π` (T12).

No bug found in the consumers; no fix commit.

## The MadGraph oracle, run locally

MadGraph is the submodule's `3.7.1` (`python3 /home/user/vibegraph/research/refs/mg5amcnlo/bin/mg5_aMC`).
Cards: 13 TeV, `ptj = 20`, `etaj = 5`, `drjj = 0.4`, fixed
`μR = μF = 91.188 GeV`, `use_syst = False`. MadGraph ran `pdlabel = nn23lo1`
(no LHAPDF here); vibegraph refuses every built-in label and ran
`pdlabel = lhapdf`, `lhaid = 247000` on `validation/pdf/NNPDF23_lo_as_0130_qed`.

**The brief's "nn23lo1 matches the set" is not quite right.** The kb already
records that `nn23lo1` is the same fit as 247000 but not the same numbers
(`scales-pdf/pinned-pdf-set`: a 9.8% shift in a `b b̄` σ). So every σ comparison
below carries an unmeasured PDF-implementation offset, and a single-subprocess
control is what separates that offset from anything this change could do. αs
does not contribute: `AQCDUP` is `0.1300000` in MadGraph's events and
`0.13000271` in ours. I tried to build LHAPDF locally to remove the offset; the
permission system refused building downloaded code, so I did not.

**Explicit-flavour cards differ by the beam mirror.** MadGraph's
`mirrorprocs.inc` is `.FALSE.` for both `P1_qq_qgq` (`u d > d g u`) and
`P2_qq_qgq`: a card naming `u d` integrates `u` from beam 1 only. vibegraph adds
the `d u` ordering (`Subprocess::has_mirror`), so on this card it reports twice
MadGraph's σ. With symmetric beams and cuts the two orderings are equal, so the
comparison is `σ_vg / 2` against `σ_MG`. Identical-parton initial states
(`u u`, `u~ u~`) have no mirror on either side. See Found.

### (a) σ, ≥5 seeds per side

Seeds are independent runs (MadGraph `iseed` 11–15 and 21–25 in fresh
`generate_events` runs of one process directory; vibegraph `--seed` 101–105 and
201–205, each `integrate` to `--target-rel` 0.2% or 0.3%). Unweighted seed means
(`tools/H-G1-seedstats.py`):

| card | side | mean | sd | sem | χ²/dof about the mean |
|---|---|---|---|---|---|
| `u d > d g u` + `u~ d~ > d~ g u~` | MadGraph | 185 355 pb | 900.8 (0.49%) | 402.8 (0.22%) | 4.31 |
| same | vibegraph | 375 235 pb (÷2 = 187 617) | 1003 (0.27%) | 448.7 | 2.02 |
| HT-binned (`ihtmin = 300`) `u u > g g u u` + `u~ u~ > g g u~ u~` | MadGraph | 22 352.2 pb | 109.6 (0.49%) | 49.0 | 2.24 |
| same, head `u~ u~` | vibegraph | 22 871.0 pb | 43.0 (0.19%) | 19.2 | 0.43 |

MadGraph's own seeds scatter at 2–4× their quoted errors (χ²/dof 4.31 and
2.24), so its quoted error understates its spread; the sems above are from the
scatter.

| comparison | ratio | pull (sems) |
|---|---|---|
| 2→3 pair, `σ_vg/2` against `σ_MG` | 1.0122 | 4.9 |
| 2→3, process 1 (`u d`, the head) from the init block | 1.0114 | 3.5 |
| 2→3, process 2 (`u~ d~`, the member) | 1.0192 | 1.8 |
| 2→3, `u~ d~` share of the total | 0.1057 ± 0.0011 (vg) against 0.1050 ± 0.0004 (MG) | 0.6 |
| HT-binned total | 1.0232 | 9.9 |
| **control**: `u d > d g u` alone, vibegraph, 3 seeds (336 724, 336 503, 336 387 pb; ÷2 = 168 269) against MadGraph's process 1 (165 901) | 1.0143 | — |
| **control**: `u u > g g u u` alone, vibegraph, 3 seeds (22 525, 22 480, 22 465 pb) against MadGraph's process 1 (22 028 ± 48) | 1.0210 | 9.0 |
| grouped member `u u` (through `π`, init block, 5 seeds, mean 22 530) against the same subprocess integrated alone (22 490 ± 18) | 1.0018 | about 1 (the init-block split carries the per-event sampling error, ~0.3% a seed) |

Reading: vibegraph sits 1.1–2.3% above MadGraph on every row, **including the
single-subprocess controls that no grouping touches** (1.4% and 2.1%), and the
member's share of the 2→3 pair agrees with MadGraph's (0.6σ). Inside vibegraph,
the 12-flow member's σ in its head's group equals the same subprocess integrated
alone to 0.2%. An error in how a member is summed into its head's group
would move the share, not the single-subprocess control. So the offset is the
`nn23lo1`-against-LHAPDF difference (or something else common to every proton
run), not this change; it is not a pass of the σ oracle either. What it cannot
see: anything below that 1–2% offset, i.e. a luminosity error of the member
smaller than about 1%. The rerun that closes it is in Found.

### (b) Per-subprocess colour labels against MadGraph's own events

Each event's colour connectivity is canonicalised independent of the line
integers and of the record order (each particle named by status, PDG and
occurrence; an incoming leg's colour crossed to an outgoing anticolour), and
compared per subprocess against MadGraph's `leshouche.inc` and against the
flows MadEvent itself wrote for that subprocess's own matrix element
(`tools/H-G1-colour-patterns.py`; frequencies weighted by `XWGTUP`, χ² with the
full multinomial covariance of both samples, one bin dropped). MadGraph keeps
each pair as two matrix elements with their own tables, so its events are the
reference for "the member's own matrix element". Outputs are in
`/home/user/H-G1-scratch/H-G1-colour-*.txt`.

Three readings of the same events, because the first is blind:

- **record order** — the flow as `leshouche.inc` indexes it;
- **split** (2→3) — the flow crossed with which outgoing quark is nearest the
  gluon in ΔR;
- **pT-ranked** (2→4) — identical particles named by pT rank (incoming by the
  sign of pz) instead of record slot, so a flow says "the leading gluon joins the
  subleading quark".

| card | subprocess | events (MG / vg) | outside the subprocess's own flows | record order | kinematic reading |
|---|---|---|---|---|---|
| 2→3 | `u d > d g u` (head) | 44 711 / 44 669 | 0 | χ² 3.04/1, p 0.081 | split 6.44/3, p 0.092 |
| 2→3 | `u~ d~ > d~ g u~` (**member, through π**) | 5 289 / 5 331 | 0 | 1.47/1, p 0.225 | split 5.07/3, p 0.167 |
| HT 2→4, head `u~ u~` | `u~ u~ > g g u~ u~` (head) | 242 / 368 | 0 | 4.20/11, p 0.964 | pT-ranked 9.15/11, p 0.608 |
| HT 2→4 | `u u > g g u u` (**member, through π**) | 17 138 / 24 632 | 0 | 7.72/11, p 0.738 | pT-ranked 9.61/11, p 0.565 |

(The HT runs used `u~ u~ > g g u~ u~` as the first card line, so the dominant
`u u` subprocess is the member whose events are labelled through `π`. Only two
of the four flows of the 2→3 bases ever occur, on both sides.)

**Sensitivity, measured.** The same comparison with the vibegraph member's
flow `g` replaced by `σ(g)` before analysis:

| member | `σ` | record order | kinematic reading |
|---|---|---|---|
| `u~ d~ > d~ g u~` | swap of its two occurring flows, `[3,1,2,0]` | χ² 0.47/1, p 0.494 — **blind** | split 310.9/3, p ≈ 0 |
| `u u > g g u u` | the CF automorphism `[11,9,10,8,7,5,6,4,3,1,2,0]` that M2's wrong pairing induces (`π⁻¹∘P`) | 11.94/11, p 0.368 — **blind** | pT-ranked 377.7/11, p ≈ 0 |

So flow frequencies alone are blind to exactly the error class the CF check is
blind to (a pairing inside the automorphism group): the aggregate shares of
the occurring flows are equal by symmetry. The kinematically resolved readings
see it at 300–380 units of χ² on these samples. Independently of the events,
MadGraph's own CF matrices satisfy
`CF_MG(u u)[i][j] = CF_MG(u~ u~)[π(i)][π(j)]` with the `π` the code finds, and
not in raw order, and `σ` is an automorphism of `CF_MG(u u)`: both checked
from `matrix1_orig.f` of the two P-dirs.

What these cannot see: a pairing error that maps a flow to one with the same
kinematic correlation in the variable chosen (the near-degenerate couples in
the pT-ranked table, e.g. the ±z beam symmetry); helicity labels (not compared);
anything at the level of the 1–2% σ offset.

I also banked MadGraph standalone per-helicity, per-flow JAMPs for `u d > d g u`
and `u~ d~ > d~ g u~` (`/home/user/H-G1-scratch/{ud_to_dgu,uxdx_to_dxgux}.json`,
4 points × 32 helicities × 4 flows, via `gen_standalone_jamps.py`'s `bank()`),
but did not wire them into the Rust harness: that needs uncommitted test
registrations in the tree while the banked layer was building from it. The
MadEvent-event comparison above measures the member's own matrix element
end-to-end instead.

### The HT-binned card

The card is a local stand-in for the survey's HT-binned γ/QCD cards: the pure-QCD
`u u > g g u u` + `u~ u~ > g g u~ u~` pair the backlog item names, with
`ihtmin = 300` and the jet cuts above, but **not** MLM-matched (`ickkw = 0`,
`xqcut = 0`) and without the photon. It reaches the conjugate pair in one group
(`π` pinned in the unit test) and passes grouping with event colour labels
checked per subprocess as above. A MadEvent seed took 7–27 min here under load;
a matched multi-jet card was out of reach. The CERN-host rerun request is in
Found.

## The backlog item

`crossing-pair-colour-basis-differs-refused`, `closes_when`: "A flavour group may
hold subprocesses whose colour bases differ by a conjugation or crossing, with
each event's colour flow drawn in its own subprocess's basis, and an HT-binned
γ/QCD MLM card from the survey passes grouping with the event colour labels
checked per subprocess."

- *One group holding a conjugate pair*: yes, on both QCD pairs (unit test;
  M1 and both fails-before runs show it was refused or split before).
- *Each event's flow in its own subprocess's basis*: the flow is drawn on the
  head's `JAMP2` and written from the member's own table under `π`; `JAMP2`
  agrees under `π` (unit test, 1e-12) and the written labels match MadGraph's
  own events for the member's matrix element, including the kinematic readings
  that can see a wrong `π`. I read that as met in effect.
- *An HT-binned γ/QCD MLM card from the survey*: **not met as written.** My card
  is a QCD stand-in, unmatched, not from the survey. The 19 CERN gridpacks are
  the evidence that would close it.

So the refusal-lifting part is evidenced at the level I could measure here,
with the σ oracle limited to about 1–2% by the PDF offset. Whether the PR
deletes the item turns on the CERN-host rerun of the survey cards.

## Facts that bear on H-G0

H-G0's report reached me as a scope update, so this only confirms with
independent measurements:

- MadGraph's packed CF rows for `u d > d g u` (`12,8,0,8`) and
  `u~ d~ > d~ g u~` (`12,8,8,0`) in one P-dir (`define uu = u u~; define dd = d d~;
  generate uu dd > dd g uu`) are related by relabellings of the flows (eight
  such permutations), and MadGraph keeps the four flavour assignments as four
  matrix elements in `P1_qq_qgq`.
- With explicit `add process` lines MadGraph never groups across process ids
  (separate `P1`/`P2`), as H-G0 found.
- `π` from our fingerprints maps MadGraph's own CF(u u) onto CF(u~ u~) on the
  12-flow pair, which also confirms that our flow order equals MadGraph's matrix
  order for both members.

## Found

1. **A card naming an explicit unordered initial state is integrated with both
   beam orderings; MadGraph integrates the named one.** Evidence: MadGraph's
   `mirrorprocs.inc` is `.FALSE.` for `generate u d > d g u` and for
   `add process u~ d~ > d~ g u~` (`/home/user/H-G1-scratch/H-G1-mg-conj/SubProcesses/P{1,2}_qq_qgq/mirrorprocs.inc`),
   and vibegraph's σ on that card is 2.024 × MadGraph's (375 235 against 185 355
   pb, five seeds each); `Subprocess::has_mirror` is `a != b`, so `u d` always
   gets the `d u` term. Cards built from `p`/`j` multiparticles are unaffected
   (MadGraph generates both orderings there). To close: decide the semantics —
   follow MadGraph (no mirror for a literal flavour assignment, which needs the
   enumeration to remember whether an ordering came from a multiparticle) or
   document the difference — and pin it with a σ row on an explicit-flavour card.
2. **The local σ oracle on these cards is limited to 1–2% by `nn23lo1`
   against LHAPDF 247000.** Evidence: single-subprocess controls untouched by
   grouping sit 1.43% (`u d > d g u`) and 2.10% (`u u > g g u u`, HT-binned)
   above MadGraph, at 9σ on the latter. To close (CERN host, LHAPDF installed):
   rerun the two MadGraph cards of this report with `pdlabel = lhapdf`,
   `lhaid = 247000`, 5 seeds each — the run cards are
   `/home/user/H-G1-scratch/H-G1-conj_run_card.dat` and
   `H-G1-uugguu_run_card_5k.dat` with the two PDF lines changed, the proc cards
   `generate u d > d g u` / `add process u~ d~ > d~ g u~` and
   `generate u u > g g u u` / `add process u~ u~ > g g u~ u~` — and compare
   against the vibegraph seed means above (375 235 / 2 and 22 871 pb). If the
   offset survives the same PDF, it is a finding of its own on every proton run,
   not on grouping.
3. **The CERN-host rerun that closes the backlog item.** Rerun the 19 affected
   gridpacks (15 `ColorStructureDiffers` + 4 `ColorFlowPairing`) at this branch's
   tip. Expected: the 15 QCD-conjugate ones group with fewer groups than on
   `729e88a` (one per conjugate pair instead of two); the 4 Singlet ones split
   (H-G0: 184 zero-coupling b-Yukawa diagrams). For at least one HT-binned γ/QCD
   MLM card, write events and run this report's colour check
   (`/home/user/H-G1-scratch/tools/H-G1-colour-patterns.py --ptrank` or
   `--split`) per subprocess against a MadEvent sample of the same card; record
   σ against the gridpack's own banked σ.
4. **Flow-frequency comparisons are blind to a wrong pairing inside the CF
   automorphism group.** Evidence: relabelling the vibegraph member's events by
   the automorphism M2 induces leaves the record-order χ² at p 0.368 (12 flows)
   and 0.494 (2→3); the pT-ranked and ΔR-split readings reject it at χ²
   377.7/11 and 310.9/3. The `pp_to_jj` samples cell's `ICOLUP` χ² is a
   record-order frequency comparison of this kind. To close: give the colour
   cell of the samples gate a kinematically resolved reading (flow × pT rank of
   identical particles, or × nearest-parton category), and re-measure its
   floor.
5. **The per-flow JAMP comparison of a multi-flow quark pair against MadGraph
   standalone has no committed row.** `q q > q g q` and `q q > g g q q` have no
   `gen_standalone_jamps.py` row; I banked tables for `u d > d g u` and
   `u~ d~ > d~ g u~` in scratch only. To close: add rows for those two and for
   `u u > g g u u` / `u~ u~ > g g u~ u~` and their `standalone_jamps.rs`
   registrations, which pins per-flow JAMP order and value against MadGraph for
   both members of a conjugate pair.
6. **The backlog item's MadGraph sentence is wrong** (also H-G0's Found 4):
   MadGraph puts the pair in one P-dir as separate matrix elements and maps no
   colour between them; with separate `add process` lines it puts them in
   separate P-dirs. To close: correct the item text at close-out.
7. **The brief's `nn23lo1` equivalence** ("the built-in set that matches
   NNPDF23_lo_as_0130_qed") contradicts `scales-pdf/pinned-pdf-set`. Not a code
   defect; noted so the next MadGraph-oracle brief on this host plans for it.
