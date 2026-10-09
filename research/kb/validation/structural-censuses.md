---
type: Validation Gate
title: Structural censuses against MadGraph
description: "Diagram counts against matrix<N>_orig.f NGRAPHS (not MAPCONFIG), and the proc-grammar, s-channel, decay-chain and polarization censuses dumped from MadGraph and matched hermetically."
status: draft
tags: [validation, diagrams, census, proc-card, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n19-v7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L751-L787", title: "Note 19 V7 (per-flavour diagram matching; NGRAPHS vs MAPCONFIG)"}
  - {id: n38-g1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L310-L361", title: "Note 38 G1 (grammar oracle)"}
  - {id: n38-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L433-L496", title: "Note 38 S2 (s-channel census)"}
  - {id: n38-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L599-L688", title: "Note 38 D2 (decay-chain stitching and census)"}
  - {id: n38-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L882-L1031", title: "Note 38 P1 (polarization census and frame)"}
  - {id: vdiag, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_madgraph_diagrams.rs#L1-L200", title: "validate_madgraph_diagrams.rs"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml diagrams cells and standalone census rows"}
  - {id: mg-helas, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L4581", title: "MadGraph helas_objects.py identical_decay_chain_factor"}
---

These gates compare *structure* (which diagrams, subprocesses, s-channels,
helicities) against MadGraph, before any number is integrated. All of them read
committed JSON dumped once from MadGraph by the oracle toolchain, so all are
hermetic and run on a bare clone ([validation layers](validation-layers.md)).

| gate | test | reference (dumped by) | compares |
|---|---|---|---|
| diagram counts (the `diagrams` category) | `validate_madgraph_diagrams` | `validation/madgraph/diagrams.json` (`extract_diagrams.py`) | per process, MadGraph's `NGRAPHS` per P-class representative, summed |
| proc-card grammar | `proc_grammar_oracle` | `proc_grammar.json` (`dump_proc_grammar.py`) | MadGraph's parsed `ProcessDefinition`, field by field |
| s-channel restrictions | `schannel_census` | `schannel_census.json` (`dump_schannel_census.py`) | per subprocess: count and oriented s-channel multiset per diagram |
| decay chains | `decay_chain_census` | `decay_chain_census.json` (`dump_decay_chain_census.py`) | per (process, decays): matrix elements and diagram counts |
| polarization | `polarization_census`, `polarization_frame` | `polarization_census.json` (`dump_polarization_census.py`) | legs, `NHEL` set and `IDEN` per subprocess |

Each census includes cards MadGraph refuses, and asserts both sides refuse
them; a card refused here only is a recorded, deliberate
difference.[^n38-g1][^n38-s2][^n38-d2][^n38-p1]

## Diagram counts

The reference is the representative subprocess's true `NGRAPHS` from
`matrix<N>_orig.f`, **not** `MAPCONFIG(0)`, which counts the integration-channel
union across a P-class (2672 against the actual 2316 for
`u u~ > u u~ l+ l- l+ l-`).[^n19-v7] The test enumerates the row's `.mg5`
`generate` line and counts MadGraph's way, one representative per (initial type
class, final type class) group (`count_mg_style_topologies`), since MadGraph
collapses flavour-equivalent subprocesses.[^vdiag] That assumes our first
enumerated subprocess in a class has the same diagram count as MadGraph's
representative. Matching every concrete flavour (parsing each
`matrix<N>_orig.f`'s `C Process:` header lines and keying by sorted PDGs) is
open as
[per-flavour diagram union unmatched](../backlog/validation/per-flavour-diagram-union-unmatched.md).

`diagrams.json` holds exactly the rows the manifest declares `diagrams`
hermetic, and a hermetic test asserts the set equality; see
[validation layers](validation-layers.md#cell-tiers-in-the-manifest). Small
counts are worth checking against the file rather than memory: `e+ e- > mu+ mu-`
is 2 diagrams (γ and Z), not 1.

**The four-gluon contact convention.** MadGraph writes a four-gluon contact as
one graph per colour structure (`VVVV1_0`/`VVVV3_0`/`VVVV4_0` into separate
`AMP()`s); we write one diagram whose vertex carries all three. So `g g > g g`
is 6 there against 4 here, and `gg_to_gg_cg` (SMEFT `O_G`) 27 against 21,
where MadGraph also *draws* 21 graphs and 27 is its `NGRAPHS`. Both cells are
`info`, with the amplitude gate pinning the process at a finer level than a
count; `pp_to_jj`'s cell is `uncovered` for the same reason (15 against
17).[^manifest] See [diagram enumeration](../process/diagram-enumeration.md).

## Proc-card grammar

132 cards run through MadGraph's own `MasterCmd` with generation stubbed: all 92
MadGraph reads agree field for field, all 40 it refuses are refused
here.[^n38-g1] Facts the oracle established (see
[proc-card grammar](../process/proc-card-grammar.md)):

- MadGraph's `Duplicate process` fires only for identical amplitudes, process
  number included; identical lines with different or implicit `@N` generate
  twice. We refuse any subprocess reached from two lines.
- Decay assignment: when the number of decays equals the number of decaying
  core legs, decays go to legs in order; otherwise every combination with
  repetition.
- `p p>e+ e-` is a MadGraph error (its spacing fix-up only matches `]`); only
  `=`, `<=`, `==`, `>` are order operators; `/` forbids by |PDG|, so `/ X` for a
  charged `X` forbids both orientations.

## s-channel restrictions (`>`, `$$`)

57 cards through MadGraph's generation: 43 generated and matched subprocess for
subprocess (count and per-diagram oriented s-channel multiset), 6 refused by
both, 8 one-initial-particle cards banked for decays.[^n38-s2] The census is
not vacuous by construction: flipping the orientation sign fails 10 cards, and
disabling the five-flavour `p`/`j` rewrite (b added when the model's b is
massless) fails 21. MadGraph emits no gauge-invariance warning for `>` or `$$`;
the only statement is a release note. See
[s-channel restrictions](../process/s-channel-restrictions.md).

## Decay chains

Three oracles, because each is blind to something:[^n38-d2]

1. **Container equality**: the stitched chain against the undecayed final state
   enumerated at the same `WEIGHTED` order and filtered to diagrams whose chain
   resonances are s-channels with exactly the stated daughters (10 cards, 147
   diagrams).
2. **Per-diagram amplitudes**: all 147 pairs agree per helicity and per flow.
3. **MadGraph census**: 19 cards, 17 matched on 54 (process, decays), final-state
   order included.

Mutations show why all three are kept. Dropping the between-block permutations
of identical particles fails oracle 1. A naive sign (core × decays, not rebuilt
from the graph) **passes every brief case** and fails only on
`g b > w- t, t > w+ b`, where the initial `b` line gains the `t` propagator: a
global sign, visible only to container equality and per-diagram amplitudes,
never to `|M|²`. An ambiguous forced line (`e+ e- > z e+ e-, z > e+ e-`) is
refused.

MadGraph does **not** permute identical particles between decays; it divides by
`identical_decay_chain_factor`[^mg-helas] (2 for `z z, z > e+ e-`), where we
keep both pairings and their interference. So σ for such a card differs from
MadGraph's by that interference, and the census compares MadGraph's per-ME count
with the stitched diagrams whose forced lines lead to its blocks. See
[decay chains](../process/decay-chains.md) and
[identical particles across decays](../process/identical-particles-across-decays.md).
Decay-chain cards are enumerated by `generate_from_proc_card` directly; no
separate enumerable-card path exists.

## Polarization

35 cards through MadGraph's generation and `HelasMatrixElement`: 24 matched on
legs, `NHEL` set and `IDEN`, 7 refused by both, 4 refused here only (a code that
is no helicity state of the particle, a repeated code, two lines that would both
count a helicity state).[^n38-p1] A polarized `|M|²` is not Lorentz invariant, so
the frame is part of the answer: `polarization_frame` boosts the default-frame
rows by β = (0.3, −0.2, 0.5) and requires the polarized massive rows to move
(0.32–0.95) while helicity sums stay within 1.6e-14. A boosted off-shell W's
helicity sum moves by about 2e-8 (∝ β²), so that control projects on shell
first. See [polarization](../process/polarization.md).

## What these cannot see

A count or a set says nothing about the *content* of a listed diagram (its
couplings, phase or colour); the [amplitude oracle](amplitude-oracle.md) does.
A census row matched on structure can still be refused at the integrate stage,
and the σ of a matched card is the [σ gate's](sigma-gate.md) business.

[^n19-v7]: Note 19 V7.
[^n38-g1]: Note 38 G1 outcome.
[^n38-s2]: Note 38 S2 outcome.
[^n38-d2]: Note 38 D2 outcome.
[^n38-p1]: Note 38 P1 outcome.
[^vdiag]: `vibegraph-lib/tests/validate_madgraph_diagrams.rs`, module docs and `count_mg_style_topologies`.
[^manifest]: `validation/manifest.toml`, the `gg_to_gg`, `gg_to_gg_cg` and `pp_to_jj` `diagrams` cells.
[^mg-helas]: MadGraph `helas_objects.py:4581` at `b7687064`, a denominator rather than a sum over pairings:
    ```python
    def identical_decay_chain_factor(self, decay_chains):
        """Calculate the denominator factor from identical decay chains"""
    ```
