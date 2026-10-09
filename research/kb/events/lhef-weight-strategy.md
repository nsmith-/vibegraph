---
type: Design Decision
title: "LHEF weight strategy: Buffer (IDWTUP = −4) or StochasticRounding (+3)"
description: "σ is the mean XWGTUP at −4, the sum at −3, XSECUP at +3. Buffer writes pb weights normalised to the integration; StochasticRounding streams floor(w)+Bernoulli unit copies."
status: draft
tags: [events, lhef, idwtup, unweighting, decision]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-e2corr, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L344-L369", title: "Note 23 E2 correction: unit weights can carry the overweight tail"}
  - {id: n23-e4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L571-L712", title: "Note 23 E4, the two strategies"}
  - {id: n27-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L482-L715", title: "Note 27 B4, IDWTUP = −3 and event_norm"}
  - {id: n41-p12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2299-L2357", title: "Note 41 P12, each part normalised to its integration"}
  - {id: mg-event-norm, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L4298", title: "MadGraph banner.py, event_norm declaration"}
---

# LHEF weight strategy

The unweighting pass hands over events of dimensionless weight `1`, or
`w/w_max > 1` for a point above its channel's estimated maximum
([events/unweighting](unweighting.md)). A file has to represent that tail
somehow. `--strategy` picks between the two `UnweightStrategy`s in
`vibegraph-lib/src/lhef/emit.rs`; `buffer` is the default.

## What `IDWTUP` says about σ

| `IDWTUP` | `XWGTUP` | σ of the file is | who writes it |
|---|---|---|---|
| `-4` | pb | the **mean** of `XWGTUP` | vibegraph `Buffer`; MadGraph under `event_norm = average` |
| `-3` | pb | the **sum** of `XWGTUP` | MadGraph under `event_norm = sum` |
| `+3` | `1` | `XSECUP` | vibegraph `StochasticRounding` |

MadGraph picks `-4` or `-3` from the run card's `event_norm`, whose card
default is `average` but whose *system* default (used when the card never names
it) is `sum`[^mg-event-norm]:

```python
self.add_param("event_norm", "average", allowed=['sum','average','unity'],
               include=False, sys_default='sum', hidden=True)
```

So MadGraph's own full cards produce `-4` files and a hand-written minimal
card produces `-3`, whose weights differ by a factor of the event count with
nothing else in the file to distinguish them. Reading σ without looking at
`IDWTUP` is therefore a factor-`N` error; it was found once as a uniform
`2.0e5` ratio on a 200k-event Drell–Yan bank[^n27-b4]. `lhef::record::WeightStrategy`
names `+3`, `-3` and `-4` and keeps any other value verbatim as `Other(v)`, so a
file round-trips. The `samples` gate's reader (`EventSample::from_lhe`,
`vibegraph-lib/src/validation/samples.rs`) takes the mean under `-4`, the sum
under `-3`, `XSECUP` under `+3`, and **panics** on any other `IDWTUP` rather
than guessing. vibegraph never writes `-3`; a card that moves `event_norm`
off its default is refused at parse.

## The two strategies

| | `Buffer` (default) | `StochasticRounding` |
|---|---|---|
| `IDWTUP` | `-4` | `+3` |
| `XWGTUP` | pb, each part's weights scaled so their mean over the file's `N` events is that part's integrated σ | `1` |
| `XSECUP` | per process, the integrated σ (split by event-weight share within a part) | per process, the integration's σ split by the realised event share |
| `XMAXUP` | the largest weight written | `1` |
| overweight tail | kept as a weight `> 1`, visible per event | kept as `floor(w) + Bernoulli(frac(w))` unit-weight copies |
| passes | one; the whole sample is held (~424 B/event at 2 → 2, ~42 MB at 100k) | one, streaming, no seekable sink; two (count, restart, write) when several `@N` need their shares in `<init>` first |
| several multiplicities | supported, one normalisation per part | **refused** (`refuse_rounding_on_mixed_multiplicity`, `vibegraph-cli/src/generate.rs:436`, and in `emit`) |
| header | `sample estimate before normalisation σ̂ +- err`, per part on a sum | — |

The normalisation rule and the `<init>` split are
[events/multi-process-normalisation](multi-process-normalisation.md)[^n41-p12].

## Why `-4` is a choice, not a necessity

A unit-weight file represents the overweight tail perfectly well, as
**multiplicity**: writing an event `k = floor(w) + Bernoulli(frac(w))` times
gives `E[k] = w` exactly[^n23-e2corr]. What `-4` buys is that the tail stays
visible event by event (a consumer reads `w = 8.4` instead of eight or nine
copies of one point) and that the file holds exactly the events asked for. What
it costs is that the normalising mean and `XMAXUP` are properties of the
realised sample, so a `-4` writer must hold the sample or replay it.

Stochastic rounding's variance is `f(1−f) ≤ ¼` with `f = frac(w)`, strictly
below the `Var = w` of a Poisson resample with the same mean, and exactly zero on
integer weights; for `w ≤ 1` it degenerates to plain accept/reject, so it only
touches the tail. It may overshoot `--nevents` by less than one event's
multiplicity rather than truncate the last copies[^n23-e4].

## Why an interface that owns the loop

`<init>` precedes the first event and carries `XSECUP`, `XERRUP`, `XMAXUP`. A
strategy whose `<init>` depends on the realised sample cannot stream, so
`UnweightStrategy::emit(source, plan, sink)` owns the generation loop rather
than filtering events. `EventSource::restart` must reseed every stream and reset
every accumulator; `restarting_the_source_replays_the_same_events`
(`vibegraph-cli/src/generate.rs`) pins it, and the mutation of dropping the
reseed fires. That hook is all a third mode — streaming `-4` by a two-pass
replay, memory independent of event count — would need; it is
[feature/lhef-idwtup-minus4-buffers-whole-sample](../backlog/feature/lhef-idwtup-minus4-buffers-whole-sample.md)[^n23-e4].

## What the tests can and cannot see

- `the_rounding_rule_is_not_free` (`lhef/emit.rs`) pins the rounding on
  controlled weights: mean `1.300` against `floor`/`ceil`/`round`, variance
  against `f(1−f)`, and below `½w` against Poisson. The physics-level gate
  cannot see the rule: almost every real event has weight exactly `1`, where
  every plausible rule agrees.
- σ recovered from a `+3` file is exact by construction (its `XSECUP` is the
  integration's), so the strategies are compared on shape, on independent seeds
  (with a shared seed the source hands both the same accepted events)[^n23-e4].
- The Pythia read-back gate only ever feeds `-4` files to Pythia, and does not
  check Pythia's reading of the `<init>` cross section:
  [validation/pythia-gate-header-semantics-unchecked](../backlog/validation/pythia-gate-header-semantics-unchecked.md).

## Open

Per-part normalisation for `StochasticRounding` needs per-part event counts
drawn from the parts' integrated σ (MadEvent's `unwgt.f` scales by
`xsecabs/xsum`):
[feature/stochastic-rounding-refuses-mixed-multiplicity](../backlog/feature/stochastic-rounding-refuses-mixed-multiplicity.md).

[^n23-e2corr]: Note 23 E2 correction: multiplicity carries the tail as well as weight does.
[^n23-e4]: Note 23 E4: the strategy table, the rounding variance, the restart hook and the gate's blind spots.
[^n27-b4]: Note 27 B4: the `IDWTUP = −3` defect in the samples reader and `event_norm`'s system default.
[^n41-p12]: Note 41 P12, policy 1.
[^mg-event-norm]: MadGraph `banner.py:4298` (documented at `:2846`; handed over in `madevent_interface.py:3885`; `lhe_parser.py:517-526` turns `sum` into `-3`).
