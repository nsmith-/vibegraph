---
type: Backlog Item
title: A generate run needs the PDF set beside the binary and the artifact
description: "The artifact embeds the model and diagrams (v0.2.2 reads no UFO directory), but a clean worker still needs the PDF set via LHAPDF_DATA_PATH; VEGAS grids dominate its size."
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
In v0.2.2 a proton-beam worker needs:

- the binary;
- the integration artifact, which banks the proc card, the run card's
  generate-time settings, the param card's moved parameters and decay tables,
  the restricted model and the diagram sets (`generate` refuses `--ufo-dir`);
- the PDF set, through `LHAPDF_DATA_PATH`, because unweighting reads densities
  and grid αs per trial point. A systematics request needs every set it names.

The worker rebuilds the amplitude program from the banked diagrams on start.
[The enumerate design](../../pipeline/enumerate-and-diagram-ranking.md)
measures why the compiled program is not banked.

The user requested this for after v0.1 (2026-08-02). It has three pieces, taken as
one feature:

1. **Bundle the model and the diagrams.** Done; shipped by v0.2.2. Designed in
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

Piece 2 is what remains for self-containment. Piece 3 is a size study that can
close separately.
