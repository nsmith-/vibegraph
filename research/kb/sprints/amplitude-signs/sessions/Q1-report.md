---
type: Session Report
title: "Q1 report: same-flavour four-lepton contacts agree with MadGraph per graph and per pairing"
description: "`e+ e- > e+ e- NP<=1` under SMEFTsim's four-lepton card is banked as a standalone row and gated per helicity, per flow and per MadGraph graph. 22 diagrams here make up MadGraph's 19 graphs, and each contact's two pairings carry MadGraph's relative sign exactly. Flipping that sign fails the gate at O(1)."
status: draft
tags: [validation, four-fermion, smeft, diagram-count, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: gate, resource: "https://github.com/nsmith-/vibegraph/commit/98257fb", title: "test(amplitudes): gate same-flavour four-lepton contacts per MadGraph graph"}
---
Closes `same-flavour-four-fermion-diagram-count-ungated`, if the convention
wording under "For user approval" is accepted.

## What was done

**Staging.** I staged the SMEFTsim models by running only `stage_models` from
`validation/madgraph/build.sh`. A scratch script set `SCRIPT_DIR`, `REPO_ROOT`,
`OUTPUT_BASE` and `MODELS_DIR` the way `build.sh` does. It then `eval`ed the
`stage_models()` and `model_card_dir()` bodies, extracted with
`sed -n '/^stage_models() {/,/^}/p'`, and called `stage_models`, all under
`pixi run -e madgraph bash <script>`. Output: `staged model:
SMEFTsim_topU3l_MwScheme_UFO (converted from Python 2)`, `vibegraph_toy_UFO`,
`vibegraph_toy_color_UFO`. No process directory was built.

**The row.** I added `ee_to_ee_4f` to `gen_standalone_jamps.py`:
`e+ e- > e+ e- NP<=1`, SMEFTsim `vg_c4l` (`cll1`, `cee`, `cle`), loaded on the
Rust side through `ee_to_mumu_4f`'s manifest row. It has zero widths because of
the spacelike Z, √s = 500 GeV, 4 RAMBO points, seed 4064, and the 16 helicities
of MadGraph's NHEL table. The MadGraph log reads `VERSION 3.7.1`, through
`mg5_pinned.sh`. A new `bank_amps` row flag banks `AMP(1..19)` and JAMP(1)'s
coefficient on each one, parsed from `matrix.f`. The coefficients are −1 on
`AMP(1..11)` and +1 on `AMP(12..19)`. The committed table is
`validation/madgraph/standalone/ee_to_ee_4f.json`. Its points are identical to
the diagnosis run's.

**Census, by machine.** MadGraph has `NGRAPHS=19`:

- `AMP(1..3)` are the contacts: `FFFF113_0`·`GC_54` (`cle`, LR),
  `FFFF114_0`·`GC_29` (`cee`, RR) and `FFFF28_0`·`GC_60` (`cll1`, LL).
- `AMP(4..11)` are the s-channel γ/Z diagrams.
- `AMP(12..19)` are the t-channel γ/Z diagrams.

This side enumerates 22 diagrams:

- 6 contacts: `V_727#1` {`FFFF13,14,16,17`}, `V_727#2` {`FFFF15,18`} and
  `V_728` {`FFFF3,4`}, each at flow group `@0` and `@1`.
- 16 exchange diagrams.

All 22 have `fermi_sign = −1`. **The brief's count was imprecise.** MadGraph
draws one graph per contact *coupling*, three in all. Each graph is one merged
ALOHA routine that holds both pairings. Its Fortran header writes the second
pairing as `-1.*(…)`, which is the `get_sign_flow` sign. This side draws two
diagrams per coupling, one per pairing.

**Per-pairing comparison (diagnosis, before any gate).** I used a throwaway lib
probe, not committed and since reverted. It was `compile_diagram_ast` plus the
forward pass, as `probe_process_diagrams` does, but on SMEFTsim and the
standalone table. To get MadGraph per pairing, I copied the standalone
directory twice. In each copy I zeroed one term group of each merged FFFF
routine: the `−CI` group in one, the `+CI` group in the other. I then rebuilt
DHELAS and the dump driver. In every entry the two MadGraph halves summed to the
full `AMP()` exactly, and the 16 exchange `AMP()`s were unchanged. Least
squares of each of our contact diagrams on the six MadGraph pieces
(`G·c_k·{A_k, B_k}`) gave:

| ours | = | residual / scale |
|---|---|---|
| `V_727#1@0` (cle) | 1.000000·A1 | 1.4e-16 |
| `V_727#1@1` (cle) | 1.000000·B1 | 1.1e-16 |
| `V_727#2@0`, `@1` (cee) | 0.5·(A2+B2) each | 1.1e-16 |
| `V_728@0`, `@1` (cll1) | 0.5·(A3+B3) each | 1.8e-16, 2.1e-16 |

**The per-pairing relative sign is +1 against MadGraph's merged routine, on
every contact.** Each of our pairing diagrams equals MadGraph's term group for
that pairing, inside the same global `G`. For the same-chirality contacts (`cee`,
`cll1`), MadGraph's two term groups are equal: `TMP9 = −TMP8` in the routine
with opposite `±CI`. So the two pairings are Fierz-equal, and each of our
diagrams is half the graph. A wrong relative sign would cancel those contacts
entirely. For `cle` the two pairings differ, and they interfere. Other
measurements from the probe:

- Probe sum against production `eval_amplitude`: 2.0e-16 of the largest diagram.
- Exchange diagrams: each equals `G·c_k·AMP_k` to at most 3.6e-14.
- `G = −1.00000000000001i`.

**The gate.** `standalone_jamps` gains a per-graph level for rows that bank
`AMP()`. `MG_GRAPH_DIAGRAMS` declares, for each MadGraph graph, the group of our
diagrams that makes it up. The groups must partition all of our diagrams and
must number `NGRAPHS`, so the 22→19 convention is a check, not prose. Each
diagram is compiled alone (a one-diagram `DiagramSet` through
`AmplitudeEvaluator::compile`, the device `amplitude_oracle` uses), so the
per-graph level reads production, not the probe. Each group's sum must equal
`G·c_k·AMP_k`, with `G` fitted from the flows, within `REL_TOL = 1e-12`. The
row went straight to gating. Its informational measurement is the probe
comparison above, which already agreed. Listing it in `KNOWN_DISAGREEMENT`
would have failed that list's two-way check.

## Gates

Every cargo invocation ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`:
  ```
  ee_to_ee_4f: 64 flow entries, G = Complex { re: -5.846016896213112e-17, im: -1.0000000000000109 }, worst flow 4.866e-14, worst |M|² 8.657e-14
  ee_to_ee_4f: 1216 graph entries, worst graph 2.981e-14
  test ee_to_ee_4f ... ok
  test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
  ```
  `|G|−1 = 1.1e-14`. The other ten rows are unchanged to the printed digit.
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`:
  `Finished dev profile … in 1m 53s`, exit 0.
- `cargo test --workspace`: exit 0, 38 suites, 1396 passed, 0 failed,
  16 ignored (summed `test result` lines).
- `pixi run --skip-deps validate`: exit 0, 63 suites, 1781 passed, 0 failed,
  84 ignored. Report: `64 rows × 4 categories = 256 cells: 203 measured in the
  layers this run drove (196 ✅, 7 ⚠️, 13 ⏳, 40 — / uncovered)` and `the
  measured cells are the declared cells`. No enforced cell changed. The change
  touches no production code.

## Falsifier

The temporary mutation was in `compile_single_diagram`: `fermi_sign` is
multiplied by `−1` for any diagram with a vertex at `flow_group == 1`, which
flips the relative sign between the two pairings of every same-flavour contact.
Then `cargo test -p vibegraph-lib --test standalone_jamps`:

```
ee_to_ee_4f: 64 flow entries, G = Complex { re: 8.106189339694753e-17, im: -0.12694975993715618 }, worst flow 2.010e0, worst |M|² 4.333e0
ee_to_ee_4f: 1216 graph entries, worst graph 7.261e-1
ee_to_ee_4f: point 2, helicities [1, 1, 1, 1]: |M|² 1.6524780033848456e0 against |G|²·MadGraph 2.0366309183334087e-3 (4.333e0)
ee_to_ee_4f: the flows agree only up to |G| = 0.12694975993715618
test ee_to_ee_4f ... FAILED
test result: FAILED. 10 passed; 1 failed
```

The other ten rows pass under the mutation, unchanged to the printed digit,
and serve as the control. That is consistent with none of them having a vertex
at flow group 1, which I did not check separately. I reverted the mutation
(`git checkout` of `root_diagram.rs`). The committed tree's `git diff --stat`
against the plan commit shows only the generator, the test and the table.

## Blind spots

- **Inside a group.** The per-graph level checks each contact's *sum* of
  pairings. A sign error that swaps the two pairing diagrams, or splits a
  Fierz-equal pair as (½+ε, ½−ε), passes it. The per-pairing equalities above
  come from the probe against a hand-patched MadGraph. They are a recorded
  measurement, not a gate: the patched routines are not MadGraph's own output,
  and I did not bank them.
- **`AMP(5)`/`AMP(6)` and `AMP(13)`/`AMP(14)` are equal** (the SM and SMEFT
  photon couplings on the two vertices, in either order). Their pairing is
  banked in enumeration order, and a swap is invisible.
- The global phase `G` (a sign common to every diagram), as for every
  standalone row.
- Four points at one energy. The contacts have no pole, so a kinematic region
  is not the concern. A Z-pole energy would weight the interference differently
  and is not sampled.
- The mutation was not run against `ee_to_mumu_4f` in `amplitude_oracle`. The
  claim that flow group 1 occurs only on same-flavour interactions (note 35's
  census) was not re-measured here.

## For user approval

The diagram-count convention cell. It is recorded in code as the
`MG_GRAPH_DIAGRAMS` entry for `ee_to_ee_4f` and its doc comment in
`vibegraph-lib/tests/standalone_jamps.rs`. Standalone rows have no manifest row,
so there is no manifest `diagrams` cell to hold it, unlike `gg_to_gg_cg`'s.
Proposed decision wording:

> **Four-fermion contacts with two spinor pairings count one diagram per
> pairing.** For a same-flavour four-fermion contact
> (`e+ e- > e+ e- NP<=1` under SMEFTsim's four-lepton operators), vibegraph
> emits one diagram per spinor pairing of each contact interaction. Each
> diagram is built on that pairing's own fermion lines and carries its own Fermi
> sign. MadGraph emits one graph per contact coupling, built on the
> interaction's canonical pairing, and writes the other pairing into the same
> `AMP()` with the `get_sign_flow` sign. vibegraph's count therefore exceeds
> `NGRAPHS` by one per such contact: 22 against 19 for `ee_to_ee_4f`. This is a
> counting convention, not a missing or extra diagram, in the class of
> `gg_to_gg_cg`'s 21 against 27. It is checked rather than asserted: the
> standalone gate requires the declared diagram groups to partition vibegraph's
> diagrams into exactly MadGraph's graphs. Each group's sum, from the
> production evaluator, must equal MadGraph's `AMP()` times its JAMP
> coefficient under the process's one fitted phase, to 1e-12 (measured 3.0e-14).
> Per pairing, each vibegraph diagram matches MadGraph's own term group at
> relative sign +1, measured to 2e-16 against a term-split MadGraph build. This
> is a recorded diagnosis, not a gate.

## Found

- **Standalone rows are invisible to the validation report.** The 11
  `standalone_jamps` rows, now including this one, have no manifest row and no
  report cell. A convention cell for them can live only in test code, and a
  green `ee_to_ee_4f` shows in the report only as a passing suite. To close it:
  give standalone tables a manifest home (a `[[standalone]]` entry listing its
  rows, or per-row `amplitudes`/`diagrams` cells) and have `standalone_jamps`
  write report rows.
- **The other standalone rows do not check the diagram count.** Only a row that
  banks `AMP()` is checked against `NGRAPHS`. `gg_to_ggg`, `uux_to_ggg` and the
  others compare flows only, and their count difference from MadGraph (the
  four-gluon contact's per-structure graphs) is unrecorded. To close it: bank
  `NGRAPHS` against our count for every standalone row, with declared exceptions.
- **No SMEFTsim per-diagram probe.** `probe_process_diagrams` reads only SM
  MadEvent work areas (`output/<name>_amplitude.csv`, interned SM model), so
  this session wrote a throwaway one for a SMEFTsim standalone table. To close
  it: a probe variant that takes a standalone table and its `model_row`.
- **The per-pairing check is not banked.** Swapping the two pairing diagrams
  inside a contact group is invisible to the committed gate (see Blind spots).
  To close it: a gate on MadGraph's per-pairing term groups, which would need
  MadGraph to expose the pairing split without hand-patched DHELAS, or a
  hermetic Fierz identity check on the two `cee`/`cll1` pairing diagrams
  (each must be exactly half of the pair's sum).
