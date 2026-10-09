---
type: Backlog Item
title: A generate run needs the binary, artifact, cards, PDF set and UFO directory
description: The integration artifact is not self-contained; a clean worker also needs both cards, the PDF set and (for non-SM) the UFO directory, and recompiles the program.
area: feature
state: open
priority: medium
closes_when: A clean worker samples events from the binary plus one artifact file, which carries the compiled program and the PDF data the run reads, and refuses on any mismatch as it does today.
blocked_by: []
opened: 2026-08-02
tags: [generate, artifact, pdf, vegas, post-v0.1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L835-L849", title: "TODO.md entry T070"}
---
Today a proton-beam worker needs:

- the binary;
- the integration artifact;
- both cards;
- the PDF set, because unweighting reads densities and grid αs per trial point;
- for a non-SM run, the model's UFO directory.

The user requested this for after v0.1 (2026-08-02). It has three pieces, taken as
one feature:

1. **Bundle the compiled program.** The design is in
   [note 23](../../history/notes/23-event-output-lhef-plan.md), keyed by
   `(model digest, process, compiler schema version)` from fields already banked.
   It has three recorded obstacles:
   - `helas::eval` has no serde;
   - `folded_hel` is a lazy `OnceLock` over a large expanded arena
     (`vibegraph-lib/src/helas/eval/compile.rs:69`);
   - `prune_zero_helicities`' kinematic contract (`compile.rs:678`) needs a recheck
     on load.
2. **Bundle the PDF data the run reads.** Either the member's grid file verbatim, or
   a subgrid slice pinned to the run's (x, Q²) support. Choosing between them is part
   of the design. Either way the artifact must keep refusing on a mismatch.
3. **Investigate compactifying the VEGAS grids.** They dominate artifact size on
   multichannel processes. Quantization, sparser binning and shared axes are all
   unexplored.

Pieces 1 and 2 are needed for self-containment. Piece 3 is a size study that can
close separately.
