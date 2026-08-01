#!/usr/bin/env bash
# Generate the MadGraph5 reference cross sections for H7 (hadronic-sigma):
# p p > e+ e- at 13 TeV, LO, PDF = NNPDF23_lo_as_0130_qed (lhaid 247000),
# fixed mu_F = m_Z, for two run cards (default cuts; m_ll in [60,120]).
#
# The committed run-card files (dy13_*_run_card.dat) are copied verbatim into
# each MadGraph process directory's Cards/run_card.dat, so MadGraph and vibegraph
# consume the identical cut/beam/PDF settings. The banked result lands in
# hadronic_sigma_reference.json (committed).
#
# Each run also banks its unweighted events under
# output/dy13_<name>/Events/run_<name>/ — MadGraph names the directory after the
# run tag — which is what the `samples` gate for this row compares against. The
# cross section and the sample come out of one invocation, so there is no
# configuration between them to drift.
#
# The generator is the pinned submodule (validation/madgraph/mg5_pinned.sh), not
# the packaged mg5_aMC on PATH; see that script for why.
#
# Usage: pixi run -e madgraph generate-hadronic-sigma
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/output"
mkdir -p "$OUT"
REF_JSON="$HERE/hadronic_sigma_reference.json"

# LHAPDF resolves `lhaid 247000` against the set validation/pdf/ holds rather
# than downloading one of its own; the installed data directory stays on the path
# for lhapdf.conf and the set index.
LHAPDF_DATA_PATH="$HERE/../pdf${LHAPDF_DATA_PATH:+:$LHAPDF_DATA_PATH}"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

# A generated process directory carries its own copy of MadGraph's settings, and
# `bin/generate_events` reads that copy rather than anything the caller exports.
# Both of these default to on and both reach the desktop — one opens a browser at
# the run's HTML summary, the other posts a notification — so a batch of runs
# takes over the machine it runs on. Rewrite rather than uncomment, so the value
# is what this says whatever the file arrived holding.
silence_madgraph_ui() {
  local cfg="$1"
  [ -f "$cfg" ] || return 0
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center)\s*=' "$cfg" > "$cfg.tmp"
  {
    printf 'automatic_html_opening = False\n'
    printf 'notification_center = False\n'
  } >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Run one card through madevent and print "<sigma> <err>" (pb) on stdout.
run_one() {
  local name="$1" card="$2" procdir="$OUT/dy13_$1"

  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> [$name] generating process p p > e+ e- ..." >&2
    local tmp_mg5
    tmp_mg5="$(mktemp -t gen_dy_XXXX).mg5"
    cat > "$tmp_mg5" <<EOF
generate p p > e+ e-
output $procdir -nojpeg
EOF
    bash "$HERE/mg5_pinned.sh" "$tmp_mg5" >&2
    rm -f "$tmp_mg5"
  fi

  local log="$OUT/generate_$name.log"
  # The work area is the cache, as it is for the process directories: a run that
  # already banked its events and its combined result is not re-run, so the
  # reference this script writes is a pure function of what is on disk and the
  # events the `samples` gate compares against cannot drift away from the cross
  # section the `integrals` gate uses. VG_FORCE=1 runs madevent regardless.
  if [ "${VG_FORCE:-0}" != 1 ] &&
    [ -s "$procdir/Events/run_$name/unweighted_events.lhe.gz" ] &&
    [ -s "$procdir/SubProcesses/results.dat" ]; then
    echo ">>> [$name] already run — reading its banked result" >&2
  else
    silence_madgraph_ui "$procdir/Cards/me5_configuration.txt"

    echo ">>> [$name] installing shared run card $card ..." >&2
    cp "$HERE/$card" "$procdir/Cards/run_card.dat"

    echo ">>> [$name] running madevent ..." >&2
    # The conda activation exports its own LDFLAGS, which suppresses MadGraph's
    # make_opts `STDLIB=-lc++` (its `ifeq($(origin LDFLAGS),undefined)` guard sees
    # LDFLAGS as already-set), so the LHAPDF C++ runtime symbols (__cxa_throw,
    # __gxx_personality_v0) go unresolved when madevent links libpdf.a. Append
    # -lc++ so the gensym/madevent link finds them.
    LDFLAGS="${LDFLAGS:-} -lc++" "$procdir/bin/generate_events" -f "run_$name" >"$log" 2>&1
  fi

  # MadGraph writes the combined result to SubProcesses/results.dat: field 1 is
  # the cross section (pb), field 2 its Monte-Carlo error (pb).
  local results="$procdir/SubProcesses/results.dat"
  if [ ! -s "$results" ]; then
    echo "!!! [$name] no results.dat at $results" >&2
    tail -25 "$log" >&2
    exit 1
  fi
  local s e
  s="$(awk 'NR==1{printf "%.10g", $1}' "$results")"
  e="$(awk 'NR==1{printf "%.10g", $2}' "$results")"
  if [ -z "$s" ] || [ -z "$e" ]; then
    echo "!!! [$name] could not parse sigma/err from $results" >&2
    exit 1
  fi
  echo ">>> [$name] sigma = $s +- $e pb" >&2
  echo "$s $e"
}

read -r SD ED < <(run_one default dy13_default_run_card.dat)
read -r SM EM < <(run_one mmll_60_120 dy13_mmll_run_card.dat)

python3 - "$REF_JSON" "$SD" "$ED" "$SM" "$EM" <<'PY'
import json, sys
out, sd, ed, sm, em = sys.argv[1:6]
doc = {
    "_comment": "MadGraph5 LO reference sigma(pp->e+e-) at 13 TeV, "
                "PDF NNPDF23_lo_as_0130_qed (lhaid 247000), fixed muF=muR=91.188 GeV. "
                "Generated by validation/madgraph/gen_hadronic_sigma.sh from the "
                "committed dy13_*_run_card.dat files (shared verbatim with vibegraph).",
    "default": {
        "run_card": "dy13_default_run_card.dat",
        "sigma_pb": float(sd),
        "sigma_err_pb": float(ed),
    },
    "mmll_60_120": {
        "run_card": "dy13_mmll_run_card.dat",
        "sigma_pb": float(sm),
        "sigma_err_pb": float(em),
    },
}
with open(out, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print("banked ->", out)
print(json.dumps(doc, indent=2))
PY
