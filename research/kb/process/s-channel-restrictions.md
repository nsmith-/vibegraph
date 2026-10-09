---
type: Physics Convention
title: "Required, forbidden and on-shell-vetoed s-channels (>, $$, $)"
description: "> and $$ are diagram filters inside the WEIGHTED search with MadGraph's line orientation; $ zeroes marked core lines inside the bwcutoff window pointwise in the amplitude, keeping interference."
status: draft
tags: [process-grammar, s-channel, onshell-veto, madgraph-parity, diagrams]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-sem, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L77-L128", title: "Note 38 §1.2, the s-channel restrictions in MadGraph"}
  - {id: n38-pred, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L292-L307", title: "Note 38 §3.4, the s-channel predicate"}
  - {id: n38-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L433-L496", title: "Note 38 §4 S2, > and $$ as diagram filters"}
  - {id: n38-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L802-L881", title: "Note 38 §4 S3, $ as the pointwise integrand"}
  - {id: n38-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1528-L1686", title: "Note 38 §8.5, the seeded σ gate"}
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 §4 E1, $ on a chain's core"}
  - {id: mg-schannel-id, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/base_objects.py#L2435", title: "MadGraph base_objects.py Vertex.get_s_channel_id"}
  - {id: mg-filters, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L715-L795", title: "MadGraph diagram_generation.py, required, forbidden and on-shell-forbidden s-channels"}
  - {id: mg-p1d, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/helas_call_writers.py#L1184", title: "MadGraph helas_call_writers.py, the P1D flag"}
  - {id: mg-fkw, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L4820", title: "MadGraph export_v4.py, fk_W"}
  - {id: mg-banner-sde, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L5055", title: "MadGraph banner.py, $ forces sde_strategy = 1"}
  - {id: code-onshell, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/onshell.rs#L1-L54", title: "vibegraph-lib/src/onshell.rs module documentation"}
measured:
  - {commit: e9177b5, pr: 12, landed_in: 1539abc, command: "validate_sigma the_grammar_rows_match_madevents_seeds: ten seeds at 160 000 × 6 against grammar_sigma_reference.json"}
  - {commit: 7a1eb52, pr: 12, landed_in: 1539abc, command: "cli_onshell_veto, five seeds, against validation/madgraph/onshell_veto_reference.json"}
---

MadGraph's process line has three s-channel restrictions: `> A >` (required), `$$ A`
(forbidden) and `$ A` (forbidden on shell). The first two are pure diagram filters. The
third keeps every diagram and acts on the integrand. All three are supported. How they
are written and parsed is in [proc-card grammar](proc-card-grammar.md).

## Which lines are s-channels, and which way they point

MadGraph compares the lists with `Vertex.get_s_channel_id` [^mg-schannel-id]: the
propagator's PDG id **oriented as it flows toward the final state**, and 0 for a t-channel
line. Orientation is load-bearing: `p p > w- > e+ ve` has **no** diagrams (the W there is
a W+), and `$$ t` leaves an s-channel `t~` alone. [^n38-sem]

vibegraph reads the same thing off a converted `Diagram` (`diagrams/schannel.rs`), whose
propagators carry the signed combination of external momenta they are the sum of:
[^n38-pred] [^n38-s2]

- With **two initial particles**, a propagator is s-channel **iff it is not spacelike**.
- Its **oriented id** is the particle if the line's energy is positive along
  `endpoints[0] → endpoints[1]`, the antiparticle otherwise. The energy is evaluated at
  `E_in = n_out`, `E_out = n_in`: every external energy positive and
  `Σ p_in − Σ p_out` vanishing, so the sign does not depend on which representative of
  the momentum the diagram stores.
- With **one initial particle** (a decay), every propagator is s-channel, oriented away
  from the decaying particle.

Flipping the orientation sign fails 10 census cards. Note 06's older idea of an
s-channel as "a sum of initial-state momenta only" is not what MadGraph does and not
what is implemented.

## `>` and `$$`: diagram filters

- **`> A B | C >` required** (`diagram_generation.py:715`): a diagram is kept if every id
  of some one alternative occurs among its s-channel ids. It is a membership test, not a
  count. [^mg-filters]
- **`$$ A` forbidden** (`:742`): a diagram is dropped if any of its s-channel ids is
  forbidden. MadGraph's separate one-initial path (`:754`) allows the decaying particle's
  own line; in vibegraph's representation a decay's incoming line is external, so it is
  never a propagator. On decays: `t > w+ > b e+ ve` keeps its one diagram and
  `t > w- > …` has none; `t > b e+ ve $$ t` keeps its diagram, `$$ w+` empties it;
  `h > e+ e- mu+ mu- $$ h` keeps it, `$$ z` empties it.

`SChannelFilter::keeps` runs on converted diagrams **inside the automatic lowest-`WEIGHTED`
search**, so an order the filter leaves empty moves the search on, as
`find_optimal_process_orders` does: `u u~ > a > d d~` lands at `WEIGHTED = 4`, as in
MadGraph. A process line none of whose subprocesses keeps a diagram is
`DiagramError::NoDiagrams`, MadGraph's `NoDiagramException`.

Neither filter is gauge invariant in general. MadGraph prints no warning for either; the
only statement is the 1.4.3 release note on `$$` (`UpdateNotes.txt:2149`), which the log
warning here quotes.

**Evidence** [^n38-s2]: the census `validation/madgraph/dump_schannel_census.py` →
`schannel_census.json` (hermetic `schannel_census` test) runs 57 cards through MadGraph's
own generation. 43 are generated and matched subprocess for subprocess, on the diagram
count and each diagram's multiset of oriented s-channel ids; 6 are refused by both
(`u d~ > w- > e+ ve`, `p p > w- > e+ ve`, `e+ e- > z a > mu+ mu- a`,
`u d~ > e+ ve $$ w+`, `e+ e- > mu+ mu- WEIGHTED<=3`, one `/ j` card); 8 one-initial cards
are banked for decays.

σ at 500 GeV is gated against seeded MadEvent references (`grammar_sigma_reference.json`,
`validate_sigma`): ten seeds here at 160 000 × 6 against MadEvent's five, each side's
inverse-variance mean with an error no smaller than spread/√n. [^n38-z2]

| Card | MadEvent (pb) | here (pb) | rel | pull |
|---|---|---|---|---|
| `e+ e- > z > mu+ mu-` | 0.05220506 ± 8.7e-6 | 0.05219763 ± 5.7e-6 | −1.42e-4 | −0.72 |
| `e+ e- > e+ e- $$ z` | 157.226 ± 0.195 | 157.618 ± 0.083 | +2.50e-3 | +1.85 |

The unrestricted controls (0.4198 pb and 154.9 pb on MadEvent's side) are the known-wrong
comparisons, and the gate fails if a restriction removed nothing. The `$$ z` pull is the
reference's spread: MadEvent's five seeds sit at χ²/dof 20 (four at 157.26–157.53, one at
156.48), and this side agrees with the majority. This side's own `$$ z` mean climbs about
0.5% with budget before plateauing at the gated rung
([backlog](../backlog/validation/ee-ee-nsz-sigma-low-at-low-budget.md)).

## `$`: the on-shell veto, pointwise in the amplitude

`$ A` (`diagram_generation.py:781`) keeps every diagram and marks each s-channel line of
`A` `onshell = False`. MadGraph acts on the mark twice [^n38-sem] [^code-onshell]:

1. **In the amplitude.** A marked propagator gets ALOHA's `P1D` form
   (`helas_call_writers.py:1184`: "D is for $ syntax -> offshell propagator only")
   [^mg-p1d], which multiplies it by
   `THETA_FUNCTIONR((p² − (M − c·Γ)²)(p² − (M + c·Γ)²) ≥ 0)`, with `c = bwcutoff` and
   `Γ = fk_W = max(|W|, |M·small_width_treatment|)` (zero where `W` is zero)
   [^mg-fkw]. The line is **zero wherever `√p²` is strictly inside `(M − cΓ, M + cΓ)`**
   (`(cΓ − M, M + cΓ)` once `cΓ > M`), **whatever Γ/M**.
2. **In the phase space.** The mark becomes `gForceBW = 2`, and MadEvent's `cut_bw`
   rejects a point in a configuration whose own marked line is on the same window, at
   `sde_strat = 1` and `Γ/M < 0.1`. A `$` forces `sde_strategy = 1` in the generated
   card [^mg-banner-sde]. The rejection moves no cross section: that configuration's
   diagrams carry the zeroed line, so its `AMP2` share was already zero there.

So what MadEvent integrates is

```text
F(x) = |M'(x)|²,   M' = M with every marked line zeroed on its window,
```

which **keeps the interference among the surviving diagrams**. The tempting alternative,
`|M|²·(1 − Σ_{c ∋ forbidden} w_c 1_W)` (reweighting the full |M|² by the surviving
configurations' `AMP2` share), integrates `e+ e- > mu+ mu- $ z` at 100 GeV 0.93% below
MadEvent (35σ); |M'|² agrees. [^n38-s3]

**Implementation** (`vibegraph-lib/src/onshell.rs`). Per subprocess, the s-channel lines
whose oriented id is listed are marked, keyed by (final-side legs, M, Γ). For every
non-empty subset of them an amplitude is compiled from the diagrams carrying none of its
lines; a point evaluates the subset currently on its window. At most
`MAX_MARKED_LINES = 6` distinct lines per subprocess (`OnShellVetoError::TooManyLines`).
`hadronic.rs` and `proton.rs` use |M'|² in the survey, the integral, `event_in_channel`
and `select_event`. The scale's and colour flow's `AMP2` draws drop the configurations
whose representative carries a zeroed line at `SDE_strategy = 1`; at 2, MadEvent's own
channel-cut weights are left alone, and the veto applies at any SDE strategy. A colour
flow of a reduced amplitude is mapped into the whole amplitude's basis by its tag row.
Flavour-group members must share the representative's marking (checked).

Scope of `$` here:

- **On a decay chain**, the veto marks **core** lines only (not forced lines, not lines
  inside a decay), as MadGraph marks the core amplitude before attaching decays. Marking the
  forced lines too would zero the chain's own resonance: `e+ e- > mu+ mu- z $ z, z > e+ e-`
  would read σ = 0.
  [^n38-e1]
- **`$` inside a decay** is refused (`Unsupported::DecayOnShellVeto`,
  [backlog](../backlog/feature/onshell-veto-on-decay-refused.md)). `z > e+ e- $ a` would be
  vacuous anyway: a two-body decay has no propagator, and a zero-width photon no window.
- **Different `$` lists on different lines** of one card are refused
  (`DiagramError::MixedOnShellVeto`,
  [backlog](../backlog/feature/onshell-veto-lists-differing-per-line-refused.md)).
- Reweighting refuses `$` processes
  ([backlog](../backlog/feature/reweight-forbidden-schannel-and-as-refused.md)).

**Pointwise pins.** On the Z pole and at 100 GeV the zeroed amplitude of
`e+ e- > mu+ mu- $ z` equals a separately compiled `e+ e- > mu+ mu- / z` to 1e-12. For
`u u~ > w+ b w- b~ $ t t~`, MadGraph's standalone `SMATRIX` (with its `FFV2P1D_1` defect
patched) equals the zeroed amplitudes here to 1e-12 at six points: both tops, one, the
other and neither in the window. [^n38-s3]

**σ against MadEvent 3.7.1** (five seeds each side, mean ± max(quoted, spread/√n)):

| Card | MadEvent | here | pull |
|---|---|---|---|
| `e+ e- > mu+ mu- $ z`, √s = M_Z | 10.7673 ± 0.0021 pb | 10.7678 ± 0.0020 | +0.16 |
| same, √s = 100 GeV | 9.00988 ± 0.0017 | 9.01035 ± 0.0016 | +0.19 |
| same, √s = 100, `bwcutoff = 3` (outside the window) | 51.282 ± 0.012 | 51.289 ± 0.010 | +0.43 |
| same, √s = 200 (outside) | 2.78715 ± 0.00064 | 2.78740 ± 0.00050 | +0.31 |
| `t > b e+ ve $ w+` | 1.43508e-3 ± 7.4e-7 GeV | 1.43423e-3 ± 4.8e-7 | −0.97 |
| `p p > e+ e- $ z`, dy13 card | 303.83 ± 0.27 | 303.84 ± 0.14 | +0.05 |
| `e+ e- > mu+ mu- z $ z, z > e+ e-` | 2.97134e-4 ± 3.6e-7 | 2.97573e-4 ± 2.2e-7 (+0.15%) | +1.05 |

Known-wrong comparisons (the unrestricted runs): 2023.95 pb at the pole, 51.28 pb at
100 GeV, 933.15 pb for Drell–Yan.

**`u u~ > w+ b w- b~ $ t t~` is reported, not gated.** MadGraph 3.7.1 mis-generates one
propagator form: a fermion `P1D` built from its second spinor slot (`FFV2P1D_1`, the `t`)
has the momentum sign flip applied inside the square, `(p² + (M − cΓ)²)(p² + (M + cΓ)²)`,
which is never negative, so MadEvent never zeroes the `t` window (it zeroes the `t~`'s).
Unpatched MadEvent reads 0.06811 ± 0.00008 pb against 0.04982 ± 0.00004 here. With that
one line patched in the generated `Source/DHELAS`, at fixed μ = 173 GeV, MadEvent reads
0.04934 ± 0.00007 (50k events) against 0.050695 ± 0.000024 here: about 2% apart, with
MadEvent moving down with budget while this side does not (nor under another split-angle
map), and the unrestricted fixed-μ rows agreeing (15.060 against 15.052). The row stays
informational. See [MadGraph defects](../validation/madgraph-defects.md) and the
[backlog item](../backlog/validation/uux-wbwb-onshell-veto-tt-2pct-off.md).

## `$` and a decay chain are complements

With W the on-shell window:

- `p p > z, z > l+ l-` integrates ∫_W |M_Z|²;
- `p p > l+ l- $ z` integrates ∫_outside W |M|² + ∫_W |M_γ|².

Together they reproduce `p p > l+ l-` except for the γ–Z interference inside the window.
`p p > z > l+ l-` is **not** the complement of `$ z`: it has no window and double-counts
the off-shell Z tail. Measured on the Drell–Yan card: MadEvent's chain plus `$` gives
934.91 pb against its full 933.15 (+1.75 ± 0.77); with this side's `$` and full
(933.57 ± 0.64), +1.35 ± 0.75. The difference is minus the in-window interference,
−0.15% of the total. [^n38-s3]

Status-2 records for resonances inside their window are an event-record question; see
[resonance records](../events/resonance-records.md). How channels and `AMP2` enter the
integrand is in [multichannel](../phase-space/multichannel.md); forced decay-chain lines
are in [decay chains](decay-chains.md); the enumeration the filters act on is in
[diagram enumeration](diagram-enumeration.md).

[^n38-sem]: Note 38 §1.2, with its in-note correction: MadGraph's three restrictions, the `P1D` theta, `cut_bw`, the complement identity.
[^n38-pred]: Note 38 §3.4: the s-channel predicate on `Prop.momentum`, filters inside the WEIGHTED search.
[^n38-s2]: Note 38 §4, `>` and `$$` as diagram filters: orientation rule, census, σ rows, gauge-invariance warning, one-initial behaviour.
[^n38-s3]: Note 38 §4, `$` as the pointwise integrand: |M'|², pointwise pins, σ rows, the `FFV2P1D_1` defect, the complement measurement.
[^n38-z2]: Note 38 §8.5: the seeded σ gate for the `> z` and `$$ z` rows, its budget ladder and MadEvent's Bhabha seed spread.
[^n38-e1]: Note 38 §4, event records and `add process` completion: `$` on a chain's core, `$` on a decay refused.
[^mg-schannel-id]: `madgraph/core/base_objects.py` `get_s_channel_id`, L2435.
[^mg-filters]: `madgraph/core/diagram_generation.py` L715 (required), L742 and L754 (forbidden), L781 (on-shell forbidden).
[^mg-p1d]: `madgraph/iolibs/helas_call_writers.py` L1184.
[^mg-fkw]: `madgraph/iolibs/export_v4.py` L4820, the `fk_W` definition.
[^mg-banner-sde]: `madgraph/various/banner.py` L5055.
[^code-onshell]: `vibegraph-lib/src/onshell.rs` module documentation.
