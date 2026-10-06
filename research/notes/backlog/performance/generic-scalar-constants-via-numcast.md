---
type: Backlog Item
title: Generic code builds scalar constants at run time
description: Generic kernels spell constants as F::one() + F::one() or F::from(4).expect(..), which carries an Option branch and folds to an immediate only by luck.
area: performance
state: open
priority: low
closes_when: Generic code reads small constants from associated constants (or an equivalent guaranteed-const form) on Real, and one hot kernel's emitted assembly is shown unchanged or better before and after.
blocked_by: []
opened: 2026-09-20
tags: [generics, real-trait, codegen]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1363-L1371", title: "TODO.md entry T119"}
---
`let two = F::one() + F::one()` and `F::from(<literal>).expect(..)` appear
about 31 times in `vibegraph-lib/src` (`helas/repr/lorentz.rs`, `helas/vertex.rs`,
`phasespace/diagram_channel.rs`, `phasespace/rambo.rs`, …). After
monomorphisation and inlining they fold to immediates in practice, but nothing
guarantees it, and the `NumCast` route carries an `Option` branch in the source.

Proposed: associated constants on `Real` (`ZERO`, `ONE`, `TWO`, `HALF`,
`FOUR`, …) replacing every site. `Real` is blanket-implemented and its doc
deliberately avoids associated-const bounds because `LaneField` builds its
zero by splatting at run time (`vibegraph-lib/src/helas/repr/mod.rs:71`), so
the design must cover `LaneField<N>` as well as `f64`/`f32`.

Verify by inspecting one hot kernel's emitted assembly before and after,
following [fill-arenas-asm-study-results](../../fill-arenas-asm-study-results.md).
