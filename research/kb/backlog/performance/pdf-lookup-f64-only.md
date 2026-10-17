---
type: Backlog Item
title: PDF lookup and interpolation are f64-only
description: "xfx_q2 takes one f64 x and Q² and binary-searches its knots, so a lane batch must drop to scalar for every PDF call; the lookup, not the grid's storage, is what needs to be generic."
area: performance
state: blocked
priority: low
closes_when: "PDF xfx_q2 (and the αs knot lookup) is generic over F: Real using the lane bin-search and select extension, with a lane-vs-scalar test matching per lane across the grid, its subgrid boundaries and both extrapolation branches, and byte-identical output at F = f64."
blocked_by: [real-has-no-lane-masks-or-bin-search]
opened: 2026-10-17
tags: [simd, lanes, pdf, real-trait, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
Every hadronic point calls the PDF twice per flavour, and the call is scalar:
`PdfMember::xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> f64` (`pdf/mod.rs:203`) over the
`Bicubic2D` trait (`pdf/interp.rs:96-99`). The grid's knots and coefficients
stay `f64` storage; what has to become lane-wise is the lookup and the
arithmetic on top of it:

- **Bin search.** Subgrid choice (`interp.rs:248`) and the knot index
  (`interp.rs:343`, `locate` at `:456`) are `partition_point` calls; the αs
  table does the same (`pdf/alphas.rs:172`). Lanes need a per-lane index and a
  gather of the cell's coefficients.
- **Branches.** In-grid, below-x and above-Q² extrapolation (`pdf/extrap.rs`)
  and the `Result<_, OutOfRange>` error path diverge per lane; they need masks
  and a select, or a per-lane error mask.
- **Arithmetic.** The bicubic evaluation itself is straight-line and generic
  once its inputs are gathered.

Unblocks when [real-has-no-lane-masks-or-bin-search](real-has-no-lane-masks-or-bin-search.md)
lands. Feeds [lane-batching-not-in-production](lane-batching-not-in-production.md).
