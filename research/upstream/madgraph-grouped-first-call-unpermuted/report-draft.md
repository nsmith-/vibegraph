---
severity: high
rank: 1  # of 7; 1 = most severe
severity_reason: "Default settings give a cross section about 4 % too low for common multi-jet processes, with no warning, in every release since 3.6.0."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** Grouped output sets the scales (and applies the `xqcut` rejection) on the unpermuted phase-space point: σ about 4 % low on channels with symmetry permutations

**Affected versions:**
- **3.8.1:** affected. Tested.
- **3.5.17 (LTS):** not affected. Tested. There the first `setclscales` call
  runs inside `PASSCUTS(P1)`, on the permuted point.
- **Introduced:** the `update_scale_coupling(pp, wgt)` call first appears in
  3.6.0. That is from reading the tags; I did not bisect.

### Summary

In grouped output (`group_subprocesses True`, the default), `DSIGPROC`
evaluates the matrix element on `P1`, the channel's symmetry permutation of the
sampled point `PP`. The first `setclscales` call, made through
`update_scale_coupling`, is handed the unpermuted `PP` instead. That call does
three things:
- it sets μR;
- it sets the PDF scales;
- with matching, it can reject the point through the `xqcut` test.

On channels whose permutation is not the identity, it clusters an event whose
momenta don't match the flavours and diagram being evaluated. σ comes out low
at default cards, and also with MLM matching.

### How to reproduce

```
import model sm
define q = u d
generate u q > z u q
output grouped
```

Launch with the default run card (`nevents = 5000` is enough), with several
`iseed` values, each in a fresh copy of the output. Then apply the one-line fix
below to the generated `SubProcesses/P1_qq_zqq/auto_dsig.f`, and run the same
seeds again. `P1_qq_zqq/symperms.inc` shows the `(1,2,3,5,4)` final-quark swap
on configs 2, 5, 6 and 8. The script below automates this, including a
`group_subprocesses False` comparison.

<details>
<summary><code>repro.sh</code></summary>

```bash
#!/usr/bin/env bash
# MadEvent self-consistency test: the same process integrated with
# `group_subprocesses True` and `False` must give the same cross section.
#
# Usage:
#   MG5=/path/to/bin/mg5_aMC ./repro.sh [workdir] [seed ...]
#
#   MG5       command that runs an MG5 script file (default: mg5_aMC on PATH)
#   workdir   scratch directory for the process directories (default: ./work)
#   seeds     iseed values, one fresh process directory per seed and variant
#             (default: 11 12 13 14 15)
#   VARIANTS  which runs to make (default: "grouped nongrouped patched")
#   NB_CORE   cores per madevent run (default: 2)
#   RUN_CARD  run card to install (default: run_card.dat next to this script)
#   KEEP=1    keep each run's process directory
#
# Variants:
#   grouped     output with group_subprocesses True (MadGraph's default)
#   nongrouped  output with group_subprocesses False
#   patched     grouped, with the first update_scale_coupling call in each
#               generated SubProcesses/P*/auto_dsig.f handed P1 instead of PP
#               (the one-line fix in report-draft.md, applied to the generated file)
#
# Prints one line per run to stdout and appends it to <workdir>/results.txt,
# and each run's per-channel results (G directory, sigma, error) to
# <workdir>/channels.txt. Ends with a summary per variant over every run in
# results.txt: mean, sample standard deviation over seeds, and the pull of
# each variant against `nongrouped` using those measured spreads.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MG5="${MG5:-mg5_aMC}"
WORK="$(mkdir -p "${1:-./work}" && cd "${1:-./work}" && pwd)"
shift || true
SEEDS=("$@")
[ ${#SEEDS[@]} -gt 0 ] || SEEDS=(11 12 13 14 15)
VARIANTS="${VARIANTS:-grouped nongrouped patched}"
NB_CORE="${NB_CORE:-2}"
RUN_CARD="$(cd "$(dirname "${RUN_CARD:-$HERE/run_card.dat}")" && pwd)/$(basename "${RUN_CARD:-run_card.dat}")"
RESULTS="$WORK/results.txt"

generate() {  # <mg5 script> <output dir>
  local src="$1" out="$2"
  [ -f "$out/bin/generate_events" ] && return 0
  sed "s#@OUTPUT@#$out#" "$src" > "$WORK/$(basename "$src")"
  (cd "$WORK" && $MG5 "$WORK/$(basename "$src")") > "$out.gen.log" 2>&1 || {
    tail -20 "$out.gen.log"; exit 1; }
}

configure() {  # <process dir> <seed>
  local d="$1" seed="$2" cfg="$1/Cards/me5_configuration.txt"
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center|run_mode|nb_core)\s*=' "$cfg" > "$cfg.tmp" || true
  printf 'automatic_html_opening = False\nnotification_center = False\nrun_mode = 2\nnb_core = %s\n' "$NB_CORE" >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
  sed "s/^ *[0-9]* *= *iseed /  $seed = iseed /" "$RUN_CARD" > "$d/Cards/run_card.dat"
  grep -q "^  $seed = iseed " "$d/Cards/run_card.dat"
}

patch_first_call() {  # <process dir>
  local f n=0
  for f in "$1"/SubProcesses/P*/auto_dsig.f; do
    grep -q 'CALL UPDATE_SCALE_COUPLING(PP, WGT)' "$f" || { echo "no first call in $f" >&2; exit 1; }
    sed -i.orig 's/CALL UPDATE_SCALE_COUPLING(PP, WGT)/CALL UPDATE_SCALE_COUPLING(P1, WGT)/' "$f"
    n=$((n + 1))
  done
  [ "$n" -gt 0 ]
}

generate "$HERE/proc_grouped.mg5" "$WORK/pristine_grouped"
generate "$HERE/proc_nongrouped.mg5" "$WORK/pristine_nongrouped"

for seed in "${SEEDS[@]}"; do
  for v in $VARIANTS; do
    case "$v" in
      grouped|patched) src="$WORK/pristine_grouped" ;;
      nongrouped) src="$WORK/pristine_nongrouped" ;;
      *) echo "unknown variant $v" >&2; exit 1 ;;
    esac
    d="$WORK/${v}_$seed"
    # A failed build (MadEvent's parallel Source compile occasionally leaves an
    # unindexed archive) is retried once in a fresh copy.
    for attempt in 1 2; do
      rm -rf "$d"; cp -r "$src" "$d"
      configure "$d" "$seed"
      [ "$v" = patched ] && patch_first_call "$d"
      t0=$(date +%s)
      if "$d/bin/generate_events" -f "run_$seed" > "$d.log" 2>&1 && [ -s "$d/SubProcesses/results.dat" ]; then
        break
      fi
      echo "$v seed $seed: attempt $attempt failed, see $d.log" >&2
      tail -5 "$d.log" >&2
      [ "$attempt" = 2 ] && exit 1
    done
    read -r xs err _ < "$d/SubProcesses/results.dat"
    line="$(printf '%-10s seed %-6s sigma %.6g +- %.3g pb  (%ds)' "$v" "$seed" "$xs" "$err" $(( $(date +%s) - t0 )))"
    echo "$line" | tee -a "$RESULTS"
    for g in "$d"/SubProcesses/P*/G*/results.dat; do
      gd="${g%/results.dat}"
      printf '%s %s %s/%s %s\n' "$v" "$seed" "$(basename "$(dirname "$gd")")" "$(basename "$gd")" \
        "$(awk 'NR==1{print $1, $2}' "$g")" >> "$WORK/channels.txt"
    done
    [ "${KEEP:-0}" = 1 ] || rm -rf "$d"
  done
done

python3 - "$RESULTS" <<'PY'
import collections, math, re, sys
runs = collections.defaultdict(dict)
for line in open(sys.argv[1]):
    m = re.match(r"(\S+)\s+seed (\S+)\s+sigma (\S+) \+- (\S+)", line)
    if m:
        runs[m.group(1)][m.group(2)] = (float(m.group(3)), float(m.group(4)))
stats = {}
print("\nvariant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err")
for v, r in runs.items():
    xs = [x for x, _ in r.values()]
    n = len(xs)
    mean = sum(xs) / n
    sd = math.sqrt(sum((x - mean) ** 2 for x in xs) / (n - 1)) if n > 1 else float("nan")
    stats[v] = (n, mean, sd / math.sqrt(n) if n > 1 else float("nan"))
    q = sum(e for _, e in r.values()) / n
    print(f"{v:10s} {n:2d}  {mean:.6g}  {sd:.3g}      {stats[v][2]:.3g}    {q:.3g}")
if "nongrouped" in stats:
    _, m0, e0 = stats["nongrouped"]
    for v, (_, m, e) in stats.items():
        if v != "nongrouped":
            print(f"{v} - nongrouped: {m - m0:+.4g} pb ({(m / m0 - 1) * 100:+.2f}%), "
                  f"pull {(m - m0) / math.hypot(e, e0):+.1f} sigma (measured spreads)")
PY
```

</details>

<details>
<summary><code>proc_grouped.mg5</code></summary>

```
import model sm
define q = u d
generate u q > z u q
output @OUTPUT@ -nojpeg
```

</details>

<details>
<summary><code>proc_nongrouped.mg5</code></summary>

```
import model sm
set group_subprocesses False
define q = u d
generate u q > z u q
output @OUTPUT@ -nojpeg
```

</details>

The script installs a run card per run. Make the two cards from the default
`Cards/run_card.dat` that `output` writes:
- `run_card_ickkw0.dat`: set `nevents = 5000` and `use_syst = False`.
- `run_card.dat` (the matched test): the same changes, plus `ickkw = 1`,
  `xqcut = 40` and `drjj = 0`.

Select one with `RUN_CARD=…`.

### Expected vs observed

The one-line fix should not change σ, since grouping is an implementation
detail. Five seeds per row on 3.8.1; errors are the spread over seeds:

| card | grouped (pb) | grouped + fix (pb) | change |
|---|---|---|---|
| default | 106.64 ± 0.20 | 110.55 ± 0.19 | +3.7 % |
| `ickkw = 1`, `xqcut = 40` | 55.61 ± 0.09 | 57.53 ± 0.09 | +3.4 % |

The deficit sits in the channels holding the permuted configs (`G4` and `G7`
at default cards).

**Comparing with `group_subprocesses False` in 3.8.1.** In 3.7.1, grouped + fix
agreed with the non-grouped output to 0.05 %, while stock grouped was 4 % low.
In 3.8.1 that comparison has a second, unrelated offset. Setting
`keep_grid_for_refine = False` in `gen_ximprove.py` (commit `ebbdc974`) raises
grouped σ by about 2 % in every channel, including unpermuted ones.
- With `keep_grid_for_refine = True` restored on 3.8.1, grouped + fix is
  108.65 ± 0.32 pb against non-grouped 108.60 ± 0.45 pb, and stock grouped is
  3.9 % low.
- I don't know which refine setting gives the right answer, so grouped + fix
  against stock grouped is the cleaner test.

### Cause

`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`, `DSIGPROC`
(3.8.1):

```fortran
        CALL SWITCHMOM(PP,P1,PERMS(1,MAPCONFIG(ICONFIG)),JC,NEXTERNAL)   ! line 819
        ...                                                              ! mirror of P1, line 829
            call update_scale_coupling(pp, wgt)                          ! line 856
```

The matrix element and `REWGT`'s second `setclscales` call read `P1`; the
first call reads `PP`.

### Suggested fix

```diff
       IF (VECSIZE_MEMMAX.LE.1.and.imode.ne.5) THEN ! no-vector (NB not VECSIZE_USED!)
-            call update_scale_coupling(pp, wgt)
+            call update_scale_coupling(p1, wgt)
       endif
```

This covers the scalar path (`vector_size = 1`, the default). By reading, the
vectorised path has the same problem and needs a larger change.
`DSIG_VEC` calls `update_scale_coupling_vec(all_p, …)` (line 317) before
`DSIGPROC_VEC` builds and mirrors `ALL_P1`, so the call would have to move after
the mirror loop. I have not tested that path.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
