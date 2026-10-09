---
type: Design
title: Network consent policy
description: "NetworkPolicy Deny/Ask/Allow from --no-network, $VIBEGRAPH_NO_NETWORK and --yes; refusal outranks consent; no terminal means refusal; HttpFetch is built in one place."
status: draft
tags: [cli, network, consent, pdf]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-u4-policy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L2715-L2793", title: "Note 24 §U4 outcome: the interaction policy, as implemented"}
  - {id: code-network, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-cli/src/network.rs", title: "vibegraph-cli/src/network.rs"}
  - {id: code-assets, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-cli/src/assets.rs", title: "vibegraph-cli/src/assets.rs"}
  - {id: code-fetch-common, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/fetch_common.sh", title: "validation/fetch_common.sh (the dev-side counterpart)"}
---
# Network consent policy

Every download the binary performs is an asset the user did not ask for by
path: a PDF set named only by its set name and resolved on their behalf
([asset resolution](asset-resolution.md)). So the question to decide is
consent, not reachability, and it is decided by a pure function in
`vibegraph-cli/src/network.rs`.

## The policy

`NetworkPolicy::resolve(no_network_flag, consent_flag, env_denies)`, computed
once in `main` and threaded down:

| input | policy |
|---|---|
| `--no-network` | `Deny(Denial::Flag)` |
| `$VIBEGRAPH_NO_NETWORK` set to **any** value | `Deny(Denial::Env)` |
| `--yes` / `-y` | `Allow` |
| none of these | `Ask` |

The order of the `if` chain is the rule: **refusal outranks consent in every
combination**, so a `--yes` inherited from a wrapper script cannot undo an
offline environment (`the_kill_switch_outranks_explicit_consent`,
`refusal_outranks_consent_however_it_is_expressed`). When both the flag and
the variable refuse, the flag is reported, because the refusal must name the
one change that actually unblocks the run; "drop `--no-network`" while the
variable is still set would be advice that does not work.

`--no-network` and `$VIBEGRAPH_NO_NETWORK` are one switch with two spellings,
not two mechanisms. `--yes` sits on the same axis in the other direction. It
exists because with "no terminal ⇒ refuse" as the default, an unattended job
that is supposed to download (release acceptance, run from a runner with no
terminal) would otherwise have no way to say yes.

## Asking

`Ask` means ask if there is a terminal, and **asking without a terminal is a
refusal**. On the stream path both `stdin` and `stderr` must be terminals: a
redirected `stdin` cannot answer and a redirected `stderr` hides the question.
The prompt goes to `stderr`, so piping `stdout` never swallows it. Only `y` or
`yes` (case-insensitive) is a yes; an empty line, any other text, and a closed
or unreadable stream are refusals. A test harness, a CI job, a cron entry and a
container build therefore cannot start a download by accident.

When the live status pane holds the terminal (raw mode, the display's thread
reading keys), `network::confirm` puts the question through the pane instead
(`tui::ask_to_download`), with the same terms written to the scrollback and
the same decline text. See [logging and the TUI](logging-and-tui.md).

`decide()` takes its terminal-ness and its input and output streams as
parameters, so the whole matrix is exercised offline by unit tests;
`confirm()` is the thin wrapper that supplies the real ones.

## What a prompt and a refusal say

Every prompt and every refusal states the URL, the size, the SHA-256 **and the
destination directory**, so a user who cannot or will not let the binary fetch
has everything needed to do it by hand. The destination is `Located::dir` from
resolution. A refusal names the switch that would allow the download
(`-y`, "drop `--no-network`", or "unset `$VIBEGRAPH_NO_NETWORK`"). Observed on a
real pty:

```
PDF set NNPDF23_lo_as_0130_qed is not available locally. It can be downloaded now:
  source:  https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz
  size:    26.3 MB
  sha256:  60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63
  unpacks to: /…/home/pdf/NNPDF23_lo_as_0130_qed
Download it? [y/N] n
```

(26.3 MB is the 27,625,668-byte archive in MiB.)

## The structural invariant

`HttpFetch` is constructed in exactly one place in the codebase: the branch in
`assets::resolve_pdf_set_dir` immediately after `network::confirm` returned
`Consent::Granted`. Every other call into the cache layer passes
`RefusingFetch`, including the "already cached" branch. A resolution step that
was not supposed to fetch therefore cannot fetch even if `pdf_set_is_cached`
is wrong about it. The invariant is the safety property; the predicate is only
the fast path.[^n24-u4-policy]

## How the tests stay offline

The unit tests are pure. The CLI tests in `vibegraph-cli/tests/cli_first_run.rs`
deliberately run under the *default* policy with `$VIBEGRAPH_NO_NETWORK`
unset, because "an unattended run refuses" is the property under test and
setting the kill switch would test the kill switch instead. That configuration
once pulled 27 MB from CERN during a test run, so those tests carry a second
guard: `ALL_PROXY` pointed at a closed local port, which `ureq` reads from the
environment by default. If the refusal regresses, the fetch fails in
milliseconds instead of downloading, and the assertion fails either way.

## The dev-side counterpart

`validation/fetch_common.sh` (`vg_download`) applies the same shape to the
validation layer's own downloads (submodule, LHAPDF sets, refdata bundle):
`VIBEGRAPH_NO_NETWORK=1` refuses; `VIBEGRAPH_FETCH_CONSENT=1` grants (CI's
`banked` job sets it); an interactive terminal is asked, default no; otherwise
refuse, naming the variable. Two differences matter. The script tests for the
literal value `1`, while the binary refuses on any value, so
`VIBEGRAPH_NO_NETWORK=true` stops the binary but not the fetch scripts. And the
script's consent variable is `VIBEGRAPH_FETCH_CONSENT`, not a flag; the binary
does not read it.

[^n24-u4-policy]: Note 24 §U4, "The interaction policy, as implemented".
