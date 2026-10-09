---
type: Design Decision
title: The default PDF set is fetched, not embedded
description: "NNPDF23_lo_as_0130_qed is fetched on first use and SHA-256 pinned, not embedded, on an unchecked premise that no redistribution grant exists; member 0 (211–277 kB) was small enough to embed."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [pdf, licensing, distribution, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-licence, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2358-L2410", title: "Note 24 §U2 outcome: licence finding (not settled)"}
  - {id: n24-sizes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2411-L2432", title: "Note 24 §U2 outcome: measured sizes"}
  - {id: n24-decision, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2433-L2444", title: "Note 24 §U2 outcome: decision, fetch at first use"}
  - {id: code-pinned, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/cache/pinned.rs#L1-L22", title: "vibegraph-lib/src/cache/pinned.rs module doc"}
measured:
  command: "download of https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz on 2026-07-30; sizes of the archive, member 0 and .info, raw and recompressed"
---
# The default PDF set is fetched, not embedded

**Decision.** The default PDF set, `NNPDF23_lo_as_0130_qed` (LHAPDF ID 247000,
MadGraph's LO default `nn23lo1`), is not shipped inside the binary. A
hadronic run that needs it downloads it from the LHAPDF data server on first
use, with consent, verifies it against a compiled-in SHA-256, and caches it
under `~/.vibegraph/pdf/`. The mechanism is
[asset resolution](asset-resolution.md); the consent rules are
[network consent](network-consent.md); why this set is the default is
[the pinned PDF set](../scales-pdf/pinned-pdf-set.md).

## Why: licence, not size

**The premise is unchecked.** The decision rests on the statement that no
redistribution grant exists for `NNPDF23_lo_as_0130_qed`. That was concluded
from finding no licence at the distribution points listed below. Nobody has
yet established the terms that actually apply (the NNPDF collaboration's
release terms, LHAPDF's terms for the sets it serves). Until the backlog item
[nnpdf23-redistribution-terms-unverified](../backlog/hygiene/nnpdf23-redistribution-terms-unverified.md)
closes, treat "no grant exists" as a hypothesis, not a fact.

What was looked at, each an upstream distribution point:[^n24-licence]

| source | what it says about redistribution |
|---|---|
| the set's archive `NNPDF23_lo_as_0130_qed.tar.gz` | 101 `.dat` members and one `.info`; no `LICENSE`, `COPYING`, copyright notice or terms file |
| the `.info` metadata | `SetDesc`, `SetIndex`, `Authors`, `Reference` (empty), physics parameters; the LHAPDF6 `.info` format has no licence field |
| `lhapdf.org` and its set index | no terms of use or redistribution statement; asks that the LHAPDF6 paper be cited |
| `gitlab.com/hepcedar/lhapdf` `COPYING` | plain GPLv3, covering LHAPDF's source code, with no clause about the grids it downloads |
| NNPDF's NNPDF2.3QED and unpolarised-sets pages | download links and set listings; no licence or terms |
| arXiv:1308.0598, the set's reference paper | arXiv non-exclusive distribution licence, which licenses the paper, not the grids |

NNPDF's fitting code is GPLv3 and its recent papers are often CC-BY; neither
covers a 2014 grid file. An earlier plan assumed "NNPDF sets are CC-BY-4.0";
nothing found supports that for this set. This is not a legal opinion on
whether numerical tables are copyrightable at all. The point is that the
project could not demonstrate permission, so it does not act as if it has it.
The working rule applied: when the terms are ambiguous or cannot be
established from an authoritative source, fetch at first use rather than
embed. Embedding member 0 alone would be a partial redistribution of the set,
which is no less licence-sensitive than shipping it whole.

**Size would have allowed embedding.** The threshold applied was about 1 MB of
added binary weight. Measured on the archive fetched on 2026-07-30:[^n24-sizes]

| quantity | bytes |
|---|---|
| full archive (`.tar.gz`) | 27,625,668 (27.6 MB) |
| full set extracted (101 members + `.info`) | ~106,000,000 |
| member 0 (`…_0000.dat`) raw | 1,052,028 |
| `.info` raw | 2,138 |
| member 0 + `.info` as `.tar.gz` | 276,628 |
| member 0 alone, `gzip -9` | 268,875 |
| member 0 alone, `zstd -19` | 211,046 |

Only member 0 is consumed (`PDF_MEMBER = 0`; error replicas are not used at
LO), so the embeddable unit is 211–277 kB, a small fraction of a few-MB
stripped binary. The decision would have been to embed member 0. It is worth
saying plainly because it is easy to misremember later as "the set was too
big to embed". It was not.[^n24-decision] The archive SHA-256 is
`60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63`.

## Revisiting

If the terms are established and permit it, the size numbers above say
embedding member 0 is comfortable, and the fetch path could become a fallback.
The backlog item asks the same question of every other set the CLI fetches;
today the pin table has one entry.

The citation every distribution point asks for, worth carrying whatever the
decision:

> Parton distributions: NNPDF2.3 (`NNPDF23_lo_as_0130_qed`, LHAPDF ID 247000),
> NNPDF Collaboration: R. D. Ball, V. Bertone, S. Carrazza, L. Del Debbio,
> S. Forte, A. Guffanti, N. P. Hartland, J. Rojo, "Parton distributions with
> QED corrections", Nucl. Phys. B877 (2013) 290, arXiv:1308.0598
> ([paper](https://arxiv.org/abs/1308.0598)). Delivered via LHAPDF6:
> A. Buckley et al., Eur. Phys. J. C75 (2015) 132, arXiv:1412.7420
> ([paper](https://arxiv.org/abs/1412.7420)).

The licensing of what the binary *does* embed, the MadGraph SM model, is
[licensing](licensing.md).

[^n24-licence]: Note 24 §U2, "License finding — NOT SETTLED".
[^n24-sizes]: Note 24 §U2, "Measured sizes".
[^n24-decision]: Note 24 §U2, "Decision: fetch at first use".
