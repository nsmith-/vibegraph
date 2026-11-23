---
type: Session Report
title: "Sprint L pipeline close-out notes: Found items for filing"
description: "The vg-sprint-L-pipeline session's unfiled Found items from PRs 3, 3b, 13 and 14, with their evidence, for the planner to file."
status: draft
tags: [sprint-l, close-out, found, performance, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
These are the pipeline half's Found items that are not yet filed. Each entry
names its source and evidence. The planner files them; nothing here edits the
backlog.

## Performance

1. **The artifact encode is a serial zstd-19 pass** (L-T3 Found 1).
   - On DY+0–3j `enumerate`, about 3.4 s of the 4.3 s left at `-j 4` is the
     encode (release build).
   - A multithreaded encoder changes the compressed bytes, but not the
     digests. Deciding whether the file bytes are a contract comes first.
2. **feyngraph clones the whole model for every subprocess** (L-T3 Found 2).
   - `DiagramGenerator::new` and `assign_topologies` each clone the model,
     once per surviving subprocess. That clone is the ~11 ms floor L-T2
     measured on empty subprocesses.
   - Fix: an upstream `Arc` model, or a fork rev.
3. **Pairing splits and on-shell vetoes compile their sub-sets one at a
   time** (L-T3 Found 3). `PairingSplit::new` (`pairing.rs`) and
   `group_vetoes` (`onshell.rs`) call `compile_class` per group, serially,
   inside the compile phase.
4. **The 4j enumeration walks use half the pool** (L-T3 Found 5).
   - W+4j and Z(→νν)+4j ran at user/real 2.2 on 4 threads.
   - The 15.9M-candidate walk itself is still serial.
5. **The LHE writer is serial and bounds `generate` on cheap processes.**
   - Unmatched p p > t t~ (249k events) at `-j 4` spends about 4 s of 10.9 s
     writing the 225 MB file after the last unweight record. It scales
     2.24× against 3.38× on an MLM card.
   - The `-j 8` number on cheap processes is set by the writer, not by
     sampling.
   - Evidence: `sessions/L-S1-report.md`, four-core table.

## Validation and tests

6. **bbx 2→6 samples row: thin margins** (vg-validator, 5b546464).
   - Trials per event are 355–363 against the 400 budget, so 89–91% of it.
   - The worst beam-2 `pz` deviation is 4.43e-9 against a 5.00e-9
     tolerance, 89% of it.
   - The row passes, but a slightly different maximum or seed could tip it.
7. **`groups` differs between `compile` and `alpha_survey` progress records**
   (L-Y1 Found a).
   - When a single-particle part is dropped because its cuts zero it,
     `compile` counts it and `alpha_survey` does not.
   - Fix: decide the drop before the compile loop.
8. **The proton phase-timing test still needs the fetched NNPDF set**
   (L-Y1 Found b).
   - `a_proton_integration_streams_its_phases_in_order` sits behind
     `extended-validation`.
   - It could use the smooth stand-in PDF set that
     `cli_progress_position.rs` writes, and so run in the default suite.

## Documentation

9. **kb pages still describe `--parallel-diagrams` as the only way to get
   parallel enumeration** (L-T3 Found 4).
   - `pipeline/enumerate-and-diagram-ranking.md` (~L112).
   - `phase-space/rng-substreams-and-parallel-determinism.md` (~L118).
   - The last paragraph of `feyngraph-assign-counts-allocation`.
   - Enumeration now uses the pool by default above a measured threshold.

## Campaign records

10. **gpdb records an external-UFO card's model as `sm` and holds no UFO for
    it** (vg-validator, PR 3b enumerate diff).
    - Sixteen released cards cannot be re-staged: zprime_UFO ×7, HAHM ×7 and
      LPstable_cascade ×2.
    - So the flavour prune is unverified on those three models.

## Carried from earlier in the sprint

11. **dim6top has no MadGraph cross-check** (PR 11).
12. **Conditional parameter definitions outside `parameters.py` are still
    refused** (PR 7).

The PR 13 `-j 8` table belongs to the planner, measured on the released
binary after the tag. The setup files are in [pr13-speedup](pr13-speedup/).
