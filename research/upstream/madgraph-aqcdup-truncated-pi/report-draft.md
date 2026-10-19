---
severity: low
rank: 7  # of 9; 1 = most severe
severity_reason: "The strong coupling written into each event is off by about two parts in a hundred million, which changes the last printed digit on roughly one event in eight; nothing in MadGraph's own result depends on it."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** LHE `AQCDUP`/`AQEDUP` are computed with π truncated to `3.1415926`

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

The `AQCDUP` and `AQEDUP` fields of each LHE event line are computed by dividing
`g²` by `4·3.1415926`. But `g` was built as `sqrt(4πα)` with full-precision π,
so the written value is α·π/3.1415926 = α·(1 + 1.7e-8).

That is about a sixth of the last printed digit. When αs runs event by event,
the printed `AQCDUP` is one unit too high on about 12 % of events. Anything that
compares the field with αs at the printed precision, as a downstream check or
replay would, sees the mismatch.

### How to reproduce

Run `e+ e- > u u~ g` at √s = 1 TeV with a per-event scale
(`dynamical_scale_choice = 3`, μR = Σ m_T / 2), then compare each event's
`AQCDUP` with αs(μR) from MadGraph's own `alphas()`.

The script below does this. It rebuilds μR from the event momenta and compiles a
small driver against the process's `Source/alfas_functions.f`.

<details>
<summary><code>repro.sh</code></summary>

```bash
#!/usr/bin/env bash
# The LHE event line's AQCDUP and AQEDUP fields (unwgt.f) are computed as
#     aaqcd = g*g/4d0/3.1415926d0
#     aaqed = gal(1)*gal(1)/4d0/3.1415926d0
# while g = sqrt(4*pi*alpha_s) was built with full double-precision pi. The
# written value is alpha * pi/3.1415926 = alpha * (1 + 1.7e-8): a one-sided bias
# of about a sixth of the last printed digit.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC FC=gfortran ./repro.sh [workdir]
#
#   MG5      command that runs an MG5 script file (default: mg5_aMC on PATH)
#   FC       Fortran compiler (default: gfortran)
#   workdir  scratch directory (default: ./work)
#
# Process `e+ e- > u u~ g` at sqrt(s) = 1 TeV, with dynamical_scale_choice = 3
# (mu_R = sum of transverse masses / 2), so alpha_s runs event by event.
#
# For every event the script rebuilds mu_R from the event's momenta and
# evaluates alpha_s(mu_R) with MadGraph's own alphas() (the process's
# Source/alfas_functions.f, asmz = aS from the param card, 2-loop running, as
# MadEvent uses without PDFs). It prints both alpha_s and
# 4*pi*alpha_s/(4*3.1415926) in the LHE format (e15.7), and counts how often
# each matches the AQCDUP MadGraph wrote.
set -euo pipefail

MG5="${MG5:-mg5_aMC}"
FC="${FC:-gfortran}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
PROC="$WORK/eeuug"

mg5() {  # <script>
  (cd "$WORK" && $MG5 "$1") >> "$WORK/mg5.log" 2>&1 || {
    tail -30 "$WORK/mg5.log"; exit 1; }
}

rm -rf "$PROC" "$WORK/mg5.log"
{
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
  printf 'import model sm\ngenerate e+ e- > u u~ g\noutput %s -nojpeg\n' "$PROC"
  printf 'launch %s -n run\n' "$PROC"
  printf 'set lpp1 0\nset lpp2 0\nset ebeam1 500\nset ebeam2 500\n'
  printf 'set nevents 2000\nset iseed 1\nset use_syst False\n'
  printf 'set dynamical_scale_choice 3\nset scalefact 1\n'
  printf 'done\n'
} > "$WORK/run.mg5"
mg5 "$WORK/run.mg5"

# aS as MadEvent compiled it (Source/param_card.inc, e.g. "AS = 1.180000D-01").
AS=$(sed -n 's/^ *AS *= *\([0-9.]*\)D\([-+0-9]*\).*/\1e\2/p' "$PROC/Source/param_card.inc")
[ -n "$AS" ] || { echo "no AS in Source/param_card.inc" >&2; exit 1; }

cat > "$WORK/aqcd.f" <<'FORTRAN'
      program aqcd
      implicit none
      integer nloop
      double precision asmz
      common/a_block/asmz,nloop
      double precision alphas, mu, as, g, pi
      external alphas
      character*32 arg
      call get_command_argument(1, arg)
      read(arg,*) asmz
      nloop = 2
      pi = 4d0*atan(1d0)
 10   read(*,*,end=20) mu
      as = alphas(mu)
      g = sqrt(4d0*pi*as)
      write(*,'(2e15.7)') as, g*g/4d0/3.1415926d0
      goto 10
 20   end
FORTRAN
(cd "$WORK" && $FC -O0 -o aqcd aqcd.f "$PROC/Source/alfas_functions.f")

LHE="$PROC/Events/run/unweighted_events.lhe.gz"
# Per event: the written AQCDUP, and mu_R = sum over final-state particles of
# sqrt((E+pz)(E-pz)) / 2.
gzip -dc "$LHE" | awk '
  /<event>/ { getline; aq = $6; mu = 0; inev = 1; next }
  inev && $2 == 1 { mu += sqrt(($10 + $9) * ($10 - $9)) }
  /<\/event>/ { printf "%s %.15e\n", aq, mu / 2; inev = 0 }' > "$WORK/events.txt"
awk '{ print $2 }' "$WORK/events.txt" | "$WORK/aqcd" "$AS" > "$WORK/alphas.txt"

paste -d' ' "$WORK/events.txt" "$WORK/alphas.txt" | awk -v as="$AS" '
  { n++; if ($1 == $3) true_match++; if ($1 == $4) biased_match++
    if ($3 != $4 && shown++ < 3)
      printf "  e.g. mu_R %.4f GeV: written %s, alpha_s %s, alpha_s*pi/3.1415926 %s\n", $2, $1, $3, $4 }
  END {
    printf "aS(MZ) = %s, events = %d\n", as, n
    printf "written AQCDUP equals alpha_s(mu_R)                : %d of %d\n", true_match, n
    printf "written AQCDUP equals alpha_s(mu_R)*pi/3.1415926   : %d of %d\n", biased_match, n
  }'
echo "expected: AQCDUP = alpha_s(mu_R) on every event"
```

</details>

### Expected vs observed

The written `AQCDUP` should equal αs(μR) on every event.

| version | equals αs(μR) | equals αs(μR)·π/3.1415926 |
|---|---|---|
| 3.5.17 | 1752 of 2000 | 1999 of 2000 |
| 3.8.1 | 1762 of 2000 | **2000 of 2000** |

The single 3.5.17 event that matches neither sits exactly on a rounding
boundary: its recomputed value is 3e-15 below it.

Example event (3.8.1): μR = 439.27 GeV, αs = 0.09546987, written
`0.9546988E-01`.

With a fixed scale the effect cannot show. `aS` is compiled into
`Source/param_card.inc` at 7 significant digits, so the +1.7e-8 shift never
crosses a rounding boundary of the printed field.

### Cause

`Template/LO/SubProcesses/unwgt.f` (3.8.1 lines 768–769; 3.5.17 lines
719–720):

```fortran
      aaqcd = g*g/4d0/3.1415926d0
      aaqed = gal(1)*gal(1)/4d0/3.1415926d0
```

### Suggested fix

```diff
-      aaqcd = g*g/4d0/3.1415926d0
-      aaqed = gal(1)*gal(1)/4d0/3.1415926d0
+      aaqcd = g*g/(16d0*atan(1d0))
+      aaqed = gal(1)*gal(1)/(16d0*atan(1d0))
```

Using a shared `pi` constant would do equally well.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
