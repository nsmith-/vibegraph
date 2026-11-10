---
type: Backlog Item
title: "Compiling SMEFTsim u u~ > t t~ b b~ NP<=1 panics on an empty sum"
description: "AmplitudeEvaluator::compile panics (index out of bounds in analysis::momentum_into) on the 2,517-diagram SMEFTsim four-quark process; the 92-diagram SM limit of the same card integrates."
area: validation
state: open
priority: medium
closes_when: "integrate on SMEFTsim_topU3l_MwScheme_UFO-massless `u u~ > t t~ b b~ NP<=1` compiles, the cause is named (what produced an Op::Add with no operands), and the process's amplitudes are compared per diagram against MadGraph standalone like the other SMEFTsim rows."
blocked_by: []
opened: 2026-11-09
tags: [evaluator, smeft, ufo, panic, amplitudes]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: design, resource: "../../pipeline/enumerate-and-diagram-ranking.md", title: "Enumerate design, Found while designing"}
---
Found while measuring program sizes for the
[enumerate design](../../pipeline/enumerate-and-diagram-ranking.md).

Reproduce (fixed-energy beams, so no PDF set is needed):

```
import model SMEFTsim_topU3l_MwScheme_UFO-massless
generate u u~ > t t~ b b~ NP<=1
```

```
vibegraph integrate card.dat --run-card run_card.dat --ufo-dir validation/ufo \
    --neval 1000 --fixed-budget --niter 1
```

with a run card of `0 = lpp1`, `0 = lpp2`, `500.0 = ebeam1`, `500.0 = ebeam2`.
Enumeration finds 2,517 diagrams; the run exits 101:

```
panicked at vibegraph-lib/src/helas/eval/analysis.rs:603:29:
index out of bounds: the len is 0 but the index is 0
```

`analysis::momentum_into` reads `kids[0]` of an `Op::Add` that has no operands.
It is reached from `Folded::build` (`fold.rs:220`) inside
`AmplitudeEvaluator::compile`, so the folded arena already holds the empty sum;
lowering (`lower::lower_flows` / `lower::optimize`) or rooting produced it.

The same card under `restrict_SMlimit_massless` (92 diagrams) and under
`restrict_massless` with `NP=0` both enumerate the SM's 92 diagrams, and the
SM-limit run integrates (σ = 142.3 ± 1.8 fb at 1,000 points × 1 iteration). So
the trigger is in the 2,425 diagrams the Wilson coefficients add, most likely
the four-quark contact operators. `g g > t t~ g NP<=1` and
`g g > t t~ g g NP<=1` on the same model compile.

An empty sum is a value (zero), so the fix may be either not emitting it or
giving it a defined momentum. Which one is right depends on what the empty sum
stands for: an amplitude whose every term vanished is benign, while a colour
flow or configuration that lost all its contributions would mean a dropped
diagram.
