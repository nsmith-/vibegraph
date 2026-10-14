---
type: Design Decision
title: "Hygiene sprint scope: the fresh review plus localised items from any area"
description: "The sprint claims filed items, from any area, that are local, need no MadGraph or Pythia run or seed re-measurement, and settle no open physics question; the rest stay out."
decided: 2026-10-09
decided_by: human:nsmith-
status: stable
tags: [hygiene, scope, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: "human:nsmith-", at: 2026-10-09}]
sources:
  - {id: user, resource: "../log.md", title: "Hygiene sprint log, 2026-10-09: the user's answers in the planning session"}
---
Chosen over "fresh review only" and "everything runnable".

## The rule

An item filed under **any** area is in scope when all three hold:

1. **Local.** The fix is confined to the sites the item names. It is a
   comment, an assertion, a test's placement, or a small code change pinned
   by a test.
2. **No new runs.** It needs no MadGraph or Pythia run and no seed
   re-measurement. A reference it adds is extracted from data already banked.
3. **No open question.** It settles no physics or convention question. An item
   whose resolution is "decide which side is right" is validation work.

An item that is `needs-user`, or blocked on one, is out whatever it is.

The folder an item is filed under is not the test. Hygiene items can fail the
rule (MadGraph re-extractions), and validation items can pass it (an assertion
too loose to fail). The first draft of this scope read only `backlog/hygiene/`.
The validation items below were added after a pass over `backlog/validation/`
against the rule (`log.md`, 2026-10-09).

## How the rule split the boundary items

**In:**
- the stale-comment lists, where the replacement figures are already recorded
  (`validate-hadronic-calibration-comments-superseded` says so itself);
- the latent bugs: `asin`/`acos`, `make_anti`, the SDE = 1 configuration
  weights, the dead reweight guard, the artifact version arm;
- `runcard-opaque-defaults-unverified`, a comparison arm that compares nothing;
- small tooling and CI fixes;
- from `validation/`:
  - `jj-banked-orderings-eta-uses-wrong-components`, a probe reading the wrong
    momentum components; the re-recorded counts come from banked events;
  - `config-amp-phase-and-sign-unpinned`, an assertion (|k| = 1) looser than
    the measured fact (k/G = ±1). Its sign-pattern table is extracted from the
    banked set;
  - `smeftsim-vendored-checksum-not-hermetic`, a hermetic check placed in a
    feature-gated target.

**Out:**
- `llj-gate-comments-quote-pre-floor-ladders` and `sigma-calibration-comments-stale`
  (rule 2: new ladders or seed re-runs);
- `kt-dump-tables-lack-directory-key` and `pp-to-jj-tie-break-no-cluster-dump`
  (rule 2: MadGraph re-extraction);
- `jioxxx-reference-port-comparison-has-no-teeth` (rule 3: closing it can mean
  deciding HELAS against ALOHA);
- `pythia-gate-momenta-unchecked` (rule 2: needs the Pythia environment);
- `nnpdf23-redistribution-terms-unverified` (licence research, not code);
- `lorentz-coefficients-still-f64` (a refactor, see [D3](D3-fix-small-file-large.md));
- every `needs-user` item and the items blocked on one.

The full claim list, with the session that closes each item, is in
[sprint.md](../sprint.md), "Scope".
