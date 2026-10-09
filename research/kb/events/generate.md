---
type: Design
title: "generate: from an integrate artifact to an event file"
description: "generate refuses card, model or PDF mismatch, installs the banked grids and channel weights, draws labels per accepted event and writes LHE; check-events checks a file's structure, not its physics."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [events, generate, cli, artifact, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-e4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L571-L712", title: "Note 23 E4, the generate CLI"}
  - {id: n24-built, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1834-L1849", title: "Note 24 P4, generate at proton beams"}
  - {id: n24-flavour, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1850-L1884", title: "Note 24 P4, the (member, ordering) draw"}
  - {id: n24-mirror, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1885-L1906", title: "Note 24 P4, the exchanged beam ordering"}
  - {id: n24-mirror-p2c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1349-L1386", title: "Note 24 P2c, the mirror reflects outgoing legs only"}
  - {id: n24-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1954-L2010", title: "Note 24 P4, the four gates"}
  - {id: n24-corr, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2011-L2041", title: "Note 24 P4, plan corrections"}
  - {id: n40, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L66-L80", title: "Note 40 §3, our own events replay in their own group"}
  - {id: n24-check, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2794-L2815", title: "Note 24 U4, check-events"}
---

# generate: from an integrate artifact to an event file

`vibegraph generate <artifact> <proc-card> [--run-card …] [--nevents N]
[-o events.lhe] [--strategy buffer|stochastic-rounding] [--seed …]`
(`vibegraph-cli/src/generate.rs`) turns a finished `integrate` run into an
unweighted Les Houches file. It covers fixed-energy partonic beams
(`lpp = 0`), proton beams (`lpp = 1`, including `p p > e+ e-` through the
general `ProtonIntegrand`), decays, decay chains, sums over several `@N`
processes and final-state multiplicities, and MLM-matched cards. The artifact
format and its versioning are [pipeline/integrate-artifact](../pipeline/integrate-artifact.md).

## The cards are a fingerprint, checked rather than trusted

The artifact banks the process string, the model identity and the resolved run
card, but not the compiled amplitude, so the process is re-enumerated from a
proc card the caller supplies. `card_mismatches` then compares, exactly:

- the model's `import model` label, and, when the labels agree, its content
  digest (a model whose assets changed under an unchanged name);
- the process string;
- **every** run-card parameter, including ones no physics reads, as the
  parser's own output (floats by equality, not tolerance).

`pdf_mismatches` compares the artifact's PDF set and member with the ones this
run would load, before loading anything; the run card pins only the LHAPDF id,
while the set actually loaded is named by `--pdf-set`. A fixed-energy run is
held to the artifact's `none`/`0`. Any mismatch is refused with the offending
names listed[^n23-e4][^n24-gates]. The policy is deliberately blunt: deciding
case by case which differences are harmless is how a mismatched grid gets
sampled.

Further refusals keyed on the artifact's format version: a clustering-scale
card with an artifact older than `SCALE_DRAW_VERSION`, a multi-multiplicity
card older than `MULTIPLICITY_VERSION`, and a process whose channels merge with
an artifact older than the merged-channel keys (`refuse_unmerged_grids`).
`check_channel_keys` matches the banked channel keys to the rebuilt ones
position by position.

## Replay is exact, not re-derived

The channel weights `α_j` enter every channel's term, so an integrand whose
weights were re-surveyed would only match the trained grids by accident. The
banked `α_j` are installed directly (`use_multichannel_with_alphas` at fixed
beams, `set_channel_alphas` at proton beams), and the phase-space maps are
rebuilt under the artifact's recorded map options rather than today's
defaults[^n23-e4].

`banked_channel_weights_rebuild_the_integrand_bit_for_bit`
(`vibegraph-lib/src/hadronic.rs`) pins `value_in_channel` against the adapted
integrand. Two traps it guards: on a two-channel process α converges to
uniform, where it cancels between a channel's weight and the mixture density,
so the test runs on the five-channel `e+ e- > ta+ ta- h` and asserts the
weights are not uniform; and it installs a skewed weight set and requires the
probes to move.

## Per accepted event

1. Unweighting draws a channel and an accepted point
   ([events/unweighting](unweighting.md)).
2. The momenta are rebuilt from the same channel map at the same `u`.
3. Labels are drawn, each a selection that moves no weight
   ([events/colour-and-helicity-selection](colour-and-helicity-selection.md)).
   At proton beams, in order: the flavour group
   `∝ avg_g (L_direct |M_g(q)|² + L_mirror |M_g(Rq)|²)` with each group's term at
   its own scales; then `(member, ordering)` `∝ S_i · L_i^o(x₁,x₂) · |M(q or Rq)|²`;
   then helicity, configuration and colour flow (`ProtonIntegrand::select_event`).
4. The record's scales are the ones the weight was taken at
   (`record_scales`, `event_scales_at` on the point's own `u`), see
   [events/lhef-record-conventions](lhef-record-conventions.md).
5. The record is built (`SubprocessRecord::event` or `event_with_intermediates`
   for resonances, `matched_event_record` under matching).

The strategy then turns the weights into the file
([events/lhef-weight-strategy](lhef-weight-strategy.md)).

### The `(member, ordering)` draw

Members of a flavour group share their matrix element exactly, so at a fixed
beam ordering the draw is the member's share of the group's parton luminosity
at the event's `(x₁, x₂)`, times its identical-particle factor. The ordering
split is where the matrix element re-enters, since `|M(q)|²` and `|M(Rq)|²`
differ[^n24-flavour]. Each group's term, and so each member's label, uses that
group's own clustering scale, drawn from that group's `AMP2` per beam ordering
([scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md)),
so the drawn term's scales are the ones written. Replaying 2000 generated
events per card in their own group's configurations returns every `SCALUP`
(to 1e-6) and `AQCDUP` on the dynamic-scale `p p > t t~` (decayed) and
`p p > l+ l- j` cards (`cli_decay_chain_events::our_own_events_replay_in_their_own_flavour_group`);
that check accepts any configuration inside the group and cannot see the
mirror orientation on those cards[^n40].

`a_members_share_of_the_draw_is_its_share_of_the_parton_luminosity`
(`proton.rs`) sweeps the uniform rather than sampling it, so the measured shares
carry no Monte Carlo error: deviation 0.18 sweep resolutions, against ≥ 6.3 for
a uniform draw and up to 72.3 for an exchanged-beam assignment. Its oracle forms
the `x·f` products independently of `FlavorGroup::member_luminosity`, at an
off-central point (`x₁ > 5x₂`) and with a probe PDF whose `x` shape differs by
flavour; without both, the beam orientation would be unpinned.

### The mirror identity

The exchanged ordering evaluates the representative at `Rq`, with `R` reflecting
the **outgoing legs only**; applied to the whole point it would put beam 0 on
`−z` and break the partonic-centre-of-mass contract the pruned evaluator
asserts[^n24-mirror-p2c]. In the record, the two incoming legs of every per-leg
field (helicity, colour tags, flavour) trade places and the outgoing legs are
untouched (`ColorFlowTags::permuted`, `SubprocessRecord::relabelled`, which is
how one compiled amplitude serves every member under both orderings)[^n24-built].
`an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field` checks this
against the mirrored subprocess compiled from its own card: per-helicity `|M_c|²`
and per-flow `JAMP2` agree to 1e-11 of the summed `|M|²` on all six `ℓℓj`
groups, matched by helicity tuple. MadGraph's banked events agree
independently: `g u > e⁺e⁻ u` with the quark on beam 1 carries the two incoming
`ICOLUP` rows exchanged and the same integers[^n24-mirror]. The test cannot see
a flow-index permutation on `ℓℓj` (one flow per subprocess);
[events/per-member-colour-flow-tables](per-member-colour-flow-tables.md) covers
the multi-flow case.

## What the gates can and cannot see

The generate gates are listed in [validation/event-output-gates](../validation/event-output-gates.md).
Their shared limits, which decide what a green run means:

- **A self-consistently wrong format.** The self-read checks use our own
  `lhef::parse`, which shares its assumptions with the writer. Format evidence
  is the byte round trip of MadGraph's own files ([events/lhef-io-design](lhef-io-design.md))
  and the Pythia read-back ([events/pythia-interop](pythia-interop.md)).
- **Anything wrong in the integrand.** A wrong matrix element, cut or sampler is
  replayed faithfully; the amplitude, σ and unweighting gates cover those.
- **Label frequencies.** The proton gate's two MadGraph oracles check that every
  emitted `IDUP` row is one of `leshouche.inc`'s subprocesses (or its beam
  exchange) and every colour connectivity one MadGraph's events exhibit; they
  say which labels are admissible, not how often each is drawn. Frequencies are
  the `samples` gate ([validation/samples-gate](../validation/samples-gate.md))[^n24-gates].
- **`σ` from a unit-weight file** is exact by construction, since its `XSECUP`
  is the integration's.

`AQCDUP` and `AQEDUP` sit a few 1e-7 from MadGraph's printed fields for stated
reasons (two interpolations of the same α_s grid; the model's `aEW` against
MadGraph's `1/aEWM1`); neither is gated at that level[^n24-corr].

## `check-events`

`vibegraph check-events <file> [--tolerance 1e-6] [--min-events N]`
(`vibegraph-cli/src/check.rs`) lets a released binary validate a file with no
toolchain and no shower. It checks momentum balance over incoming minus
outgoing legs (intermediates excluded), mass shells referenced to `E²`,
`XWGTUP ≤ XMAXUP`, unit weights actually unit at `IDWTUP = +3`, `IDPRUP` naming
a declared `<init>` process, mother indices in range and a minimum event count.
The default tolerance is three orders above the writer's ~11 significant
digits[^n24-check].

It **catches damage, not error**: it shares the writer's assumptions, so a
self-consistently wrong format passes, and a wrong matrix element, cut or
sampler produces events satisfying every identity it checks.
`a_generated_sample_passes_check_events_and_a_damaged_one_does_not`
(`vibegraph-cli/tests/cli_first_run.rs`) keeps it non-vacuous by displacing one
leg's `pz` by a GeV and requiring the rejection.

## Open edges

- A worker needs the binary, the artifact, both cards, the PDF set and (for a
  non-SM model) the UFO directory, and recompiles the program:
  [feature/generate-artifact-not-self-contained](../backlog/feature/generate-artifact-not-self-contained.md).
- The graceful-stop key does nothing under `generate`:
  [feature/generate-has-no-graceful-stop](../backlog/feature/generate-has-no-graceful-stop.md).

[^n23-e4]: Note 23 E4 outcome.
[^n24-flavour]: Note 24 P4, the flavour draw and its sweep test.
[^n24-mirror]: Note 24 P4, the exchanged ordering against MadGraph's banked events.
[^n24-mirror-p2c]: Note 24 P2c, plan correction 6: the mirror reflects the outgoing legs only.
[^n24-gates]: Note 24 P4, the four gates (round trip, self-read, PDF refusal, dynamical-card refusal).
[^n24-corr]: Note 24 P4, plan correction 5 (AQCDUP/AQEDUP digits).
[^n24-check]: Note 24 U4, check-events and its blind spots.
[^n24-built]: Note 24 P4, what was built for proton-beam generation.
[^n40]: Note 40 §3, our own events replay in their own flavour group.
