---
type: Backlog Item
title: "No machine-readable way to ask whether a UFO model loads"
description: "Consumers check a model by running enumerate on a dummy process and parsing log lines; every outcome exits 1. Add a model-only check with JSON output and distinct exit codes."
area: feature
state: open
priority: medium
closes_when: "A command such as vibegraph model check --ufo-dir D 'X-restrict' --json loads the model only and prints status, label, model digest, refusal kind and message, and whether loop or counterterm content was present and ignored; exit codes separate loaded, refused and usage error; a CLI test covers each."
blocked_by: []
opened: 2026-11-16
tags: [cli, ufo, json, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop, a CMS private-MC web app, after loading all 452 UFO roots
of the CMS model store with vibegraph 0.2.1. It runs `enumerate` on a card with
`import model X` and `generate zz9 > zz9`, then parses the `ufo_load: model L
(digest)` and `ERROR failed to load model:` log lines. The exit code is 1 in
every case, so its parser is pinned to 0.2.1's log text.

The output should also say whether `CT_vertices.py` or other loop content was
present and ignored. 85 of the 93 store models that ship counterterms load
without comment, so "loaded" is necessary but not sufficient for a consumer.
The alternative is to refuse NLO-only models by name; see
[`loop-level-ufo-models-unsupported`](loop-level-ufo-models-unsupported.md).
