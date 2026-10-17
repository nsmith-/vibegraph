---
severity: medium
rank: 4  # of 7; 1 = most severe
severity_reason: "In a specific MLM setup (five-flavour proton, four-flavour jets), b-quark events get a PDF weight computed for the wrong quark, which depends on the events before them."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** MLM `rewgt`: a non-jet final-state parton keeps the flavour code left by an earlier event, so its PDF ratio uses the wrong flavour

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

Under MLM matching (`ickkw = 1`), `rewgt` relabels the clustering tree with the
selected flavour combination, but only for legs that are jets. A final-state
parton that is not a jet keeps whatever PDG code the table held before. That is
the subprocess's first combination, or the last event in which that leg was a
jet.

When one subprocess mixes jet and non-jet flavours on the same leg, the PDF
ratio for that line is taken for the wrong quark flavour. Which wrong flavour
depends on the events generated before it.

### How to reproduce

The case needs a five-flavour proton with a four-flavour jet definition:

```
import model sm-no_b_mass
define p = g u c d s b u~ c~ d~ s~ b~
define j = p
generate p p > z j
output zj
launch zj
set ickkw 1
set xqcut 20
set maxjetflavor 4
set use_syst True
set systematics_program none
set nevents 2000
done
```

`P1_gq_zq` then holds `g d > z d`, `g s > z s` and `g b > z b` in one
subprocess, with `d` first. At `maxjetflavor = 4` the `b` is a parton but not a
jet.

For each `g b > z b` event, read the gluon beam's `<pdfrwt>` entry in the
`<mgrwt>` block. As a control, generate `g b > z b` and `g b~ > z b~` alone
with the same cards. The script below runs both and tallies the codes.

<details>
<summary><code>repro.sh</code></summary>

```bash
#!/usr/bin/env bash
# MadEvent's MLM `rewgt` (reweight.f) labels the clustering tree with the drawn
# flavour combination's PDG codes only for legs that are jets:
#     if(isjet(idup(i,ipsel,iproc)))
#    $     ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)
# A final-state parton that is not a jet keeps whatever code `ipdgcl` last
# held: the subprocess's first combination, or an earlier event's. When one
# IPROC holds combinations that differ in that leg's jet-ness, the PDF-ratio
# chain is evaluated for the wrong flavour, and which wrong flavour depends on
# the events before it.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir  scratch directory (default: ./work)
#
# Process `p p > z j` in a five-flavour proton (`sm-no_b_mass`, b in p and j),
# with `ickkw = 1`, `xqcut = 20` and `maxjetflavor = 4`, so the b is a parton
# but not a jet. P1_gq_zq's IPROC 2 holds `g d > z d`, `g s > z s` and
# `g b > z b` (and IPROC 4 their antiquarks). `use_syst = T` writes the
# PDF-ratio chain each event used (<mgrwt><pdfrwt>), with
# `systematics_program = none`.
#
# For every event with one incoming gluon and an outgoing quark, the script
# tallies the codes on the gluon beam's chain and the number of alpha_s
# ratios (<asrwt>). A jet quark reads 21,-q (the spacelike line after
# g -> q q~) with one alpha_s ratio.
#
# Two process directories, both stock:
#   mixed   p p > z j as above: b shares an IPROC with d and s
#   alone   g b > z b and g b~ > z b~ only, with the same cards; there the b
#           is its IPROC's first combination, so the label is right from the
#           start; MadEvent then takes one alpha_s ratio and no PDF ratio on the b line
#
# There is no one-line fix to run as a third variant. Giving every leg the
# drawn combination's code (dropping the isjet guard) makes the b a non-jet in
# setclscales too; the number of jets per config then varies with the flavour
# combination, and the jet memo (njetstore/iqjetstore per iconfig) stops the
# run with "Error: Failed despite same graph".
set -euo pipefail

MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
PROC="$WORK/zj"

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}
opts() {
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
}

rm -rf "$PROC" "$WORK/zb" "$WORK/mg5.log"
{
  opts
  printf 'import model sm-no_b_mass\n'
  printf 'define p = g u c d s b u~ c~ d~ s~ b~\ndefine j = p\n'
  printf 'generate p p > z j\noutput %s -nojpeg\n' "$PROC"
  printf 'define bq = b b~\n'
  printf 'generate g bq > z bq\noutput %s -nojpeg\n' "$WORK/zb"
} > "$WORK/gen.mg5"
mg5 "$WORK/gen.mg5"

{
  opts
  for d in "$PROC mixed" "$WORK/zb alone"; do
    set -- $d
    printf 'launch %s -n %s\n' "$1" "$2"
    printf 'set nevents 2000\nset iseed 1\n'
    printf 'set ickkw 1\nset xqcut 20\nset maxjetflavor 4\n'
    printf 'set use_syst True\nset systematics_program none\n'
    printf 'done\n'
  done
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

tally() {  # <lhe.gz>
  gzip -dc "$1" | awk '
    /<event>/ { getline; getline; a = $1; getline; b = $1; getline; getline; c = $1; chain = "-"; nas = 0 }
    /<asrwt>/ { x = $0; sub(/.*<asrwt> */, "", x); nas = x + 0 }
    /<pdfrwt beam=/ {
      beam = ($0 ~ /beam="1"/) ? 1 : 2
      sub(/.*">/, ""); n = $1; codes = ""
      for (i = 2; i <= n + 1; i++) codes = codes (i > 2 ? "," : "") $i
      if ((beam == 1 && a == 21) || (beam == 2 && b == 21)) chain = (n > 0 ? codes : "-")
    }
    /<\/event>/ {
      if ((a == 21) != (b == 21) && c != 21) {
        q = (a == 21) ? b : a
        cnt[q " " chain " " nas]++
      }
    }
    END { for (k in cnt) { split(k, f, " "); printf "  g %2s > z %2s   gluon-beam chain %-6s alpha_s ratios %s  %5d events\n", f[1], f[1], f[2], f[3], cnt[k] } }' |
    sort -k2,2n -k7,7
}

for r in "$PROC mixed" "$WORK/zb alone"; do
  set -- $r
  echo "== $2"
  tally "$1/Events/$2/unweighted_events.lhe.gz"
done
echo "expected: the b events of 'mixed' read as in 'alone' (where the label is right): no PDF ratio (chain 21) and one alpha_s ratio"
echo "observed in 'mixed': most b events take the PDF ratio of a d or s line (chain 21,+-1 or 21,+-3), whichever an earlier event left"
```

</details>

### Expected vs observed

- **Expected:** the `g b` events behave as in the control, with no PDF ratio on
  the gluon line (`<pdfrwt>` codes `21` only) and one α_s ratio.
- **Observed:** most `g b` events take the PDF ratio of a `d` or `s` line, with
  codes `21,±1` or `21,±3`.

| version | `g b` events with a d/s PDF ratio | control (`g b` alone) |
|---|---|---|
| 3.5.17 | 97 of 103 | 0 of 2000 |
| 3.8.1 | 87 of 97 | 0 of 2000 |

The `g u`, `g c`, `g d` and `g s` events all read their own flavour.

### Cause

`Template/LO/SubProcesses/reweight.f`, in `rewgt` (3.8.1 lines 1537–1538;
3.5.17 lines 1504–1505):

```fortran
c        Set jet identities according to chosen subprocess
         if(isjet(idup(i,ipsel,iproc)))
     $        ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)
```

`ipdgcl` is indexed by (leg, graph, subprocess), not by flavour combination.
`initcluster` fills it from the first combination (`cluster.f:373`). After
that, a non-jet leg's code changes only when some event labels that leg a jet.

### Possible fix

I don't have a one-line fix.

- Dropping the `isjet` guard, so every leg takes the drawn combination's code,
  makes the survey stop with `Error: Failed despite same graph`. The per-config
  jet memo (`njetstore`/`iqjetstore`) assumes a config always has the same
  number of jets, which no longer holds once the `b` is correctly a non-jet.
- Two options that would avoid the problem:
  - do not group flavour combinations whose final-state jet-ness differs at
    the run's `maxjetflavor`;
  - key the jet memo by combination.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
