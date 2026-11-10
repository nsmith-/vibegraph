---
type: Backlog Item
title: A generate run needs the binary, artifact, PDF set and UFO directory
description: The integration artifact is not self-contained; a clean worker also needs the PDF set and (for non-SM) the UFO directory, and enumerates and compiles the program again.
area: feature
state: open
priority: medium
closes_when: A clean worker samples events from the binary plus one artifact file, which carries the restricted model, the diagrams and the PDF data the run reads, and refuses on any mismatch as it does today.
blocked_by: []
opened: 2026-08-02
tags: [generate, artifact, pdf, vegas, post-v0.1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L835-L849", title: "TODO.md entry T070"}
---
Today a proton-beam worker needs:

- the binary;
- the integration artifact, which banks the proc card, the run card and the
  param card's moved parameters and decay tables;
- the PDF set, because unweighting reads densities and grid αs per trial point;
- for a non-SM run, the model's UFO directory.

It also enumerates the diagrams again and recompiles the amplitude program.
[The enumerate design](../../pipeline/enumerate-and-diagram-ranking.md) banks
the restricted model and the diagrams in the artifact, and measures why the
compiled program is not banked.

The user requested this for after v0.1 (2026-08-02). It has three pieces, taken as
one feature:

1. **Bundle the model and the diagrams.** Designed in
   [the enumerate design](../../pipeline/enumerate-and-diagram-ranking.md) §4–5:
   the restricted `ParsedModel` (5–46 KB compressed) and the enumerated diagram
   sets, with an `amplitude_probe` that refuses an artifact whose |M|² the
   running build no longer reproduces. The compiled program is not banked: it
   is 40–80× the folded AST and rebuilds in under 3 s on every card measured.
2. **Bundle the PDF data the run reads.** Either the member's grid file verbatim, or
   a subgrid slice pinned to the run's (x, Q²) support. Choosing between them is part
   of the design. Either way the artifact must keep refusing on a mismatch.
3. **Investigate compactifying the VEGAS grids.** They dominate artifact size on
   multichannel processes. Quantization, sparser binning and shared axes are all
   unexplored.

Pieces 1 and 2 are needed for self-containment. Piece 3 is a size study that can
close separately.
