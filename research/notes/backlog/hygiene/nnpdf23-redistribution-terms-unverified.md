---
type: Backlog Item
title: NNPDF23_lo_as_0130_qed's redistribution terms are unchecked
description: "The PDF-distribution decision rests on 'no redistribution grant exists' for NNPDF23_lo_as_0130_qed; nobody has checked the set's actual licence or LHAPDF's terms."
area: hygiene
state: open
priority: low
closes_when: "The licence covering NNPDF23_lo_as_0130_qed (and the other fetched sets) is read and recorded with its source; the fetch-on-first-use design and the cache/pinned.rs module doc are kept or changed to match."
blocked_by: []
opened: 2026-10-09
tags: [licensing, pdf, distribution]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User comment on the taxonomy review, 2026-10-09"}
---
vibegraph does not embed the PDF set, although member 0 (~211–277 kB) would
fit. It fetches the set on first use, pinned by SHA-256, because no
redistribution grant was found for `NNPDF23_lo_as_0130_qed`
([note 24](../../24-user-distribution-and-proton-events-plan.md)). Whether
that is true was never checked: NNPDF sets and the LHAPDF distribution
publish their own terms.

To resolve: find the licence that applies to the set (the NNPDF
collaboration's release terms, the LHAPDF set index, the set's own `.info`
file), record it with a link, and decide whether embedding member 0 or
bundling the set into release binaries is allowed. Repeat for any other set
the CLI fetches.

The code states the unchecked premise as fact: the module doc of
`vibegraph-lib/src/cache/pinned.rs` (~:9-17) says "no redistribution grant is
published for these sets". Whatever the licence turns out to be, that doc
should say what was found and cite it (added 2026-10-09; Phase 3 verifier V15).
