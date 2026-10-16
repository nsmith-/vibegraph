# MadGraph-only reproducers

The reproducers for MadGraph defects live under `research/upstream/madgraph-*/`.
Each folder holds a reproducer, a README and, if the defect is still present
upstream, a report draft. The index is
`research/kb/validation/madgraph-upstream-report-drafts.md`.

`vibegraph-lib/tests/scales_run_cards.rs` scans every subdirectory here for
run cards, so a reproducer whose run cards should pass this crate's parser
check goes in a subdirectory here.
