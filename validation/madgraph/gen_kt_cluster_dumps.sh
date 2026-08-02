#!/usr/bin/env bash
# Build the kT-clustering oracle: an instrumented replay of the banked MadGraph
# runs that records, per written event, the whole clustering history behind
# `dynamical_scale_choice = -1` — every candidate pair with its measure, every
# merge, the 2 -> 2 core, the jfirst/jlast/jcentral walk, and the mu_R / mu_F
# branches that turn them into a scale.
#
# The instrumentation never touches the pinned submodule. Each replay gets its
# own process directory under output/ktdump/, generated fresh from the same
# `generate` line as the banked row, and the clustering sources MadGraph copies
# into it are patched there — the same shape as the amplitude probes, which
# patch the generated matrix1_orig.f rather than the template it came from.
#
# The replay is only worth reading if it is the *same* run: the cards are the
# banked ones verbatim and the seed is the one the banked banner records, so the
# unweighted event file must come out byte-identical to the bank apart from the
# banner's own run metadata. That comparison is a hard precondition here — a
# replay that fails it is not compared against anything, it stops the run.
#
# Usage:
#   bash validation/madgraph/gen_kt_cluster_dumps.sh [process ...]
#   pixi run -e madgraph generate-kt-cluster-dumps
#
# Environment:
#   VG_FORCE=1        re-run a replay whose events and dump are already there
#   VG_KT_STAGE=prep|run|extract   stop after that stage (default: all)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$HERE/output"
WORK="$OUT/ktdump"
WRAPPERS="$HERE/wrappers"
STAGE="${VG_KT_STAGE:-all}"

# The banked runs whose clustering has no closed form, plus the degenerate rows
# that do: each control covers one collapse of the general path (uux_to_uux the
# beam-crossing tie-break, pp_to_bb_qcd2 both the beam-leg and the mt2last
# route, ee_to_ttx the colourless-beam branch with a coloured final state).
DEFAULT_PROCESSES=(
    pp_to_llj
    ee_to_mumua
    ee_to_mumu_tata_qcd0
    bbx_to_ccx_emmm_qcd0
    uux_to_ccx_emmm_qcd0
    uux_to_uux
    pp_to_bb_qcd2
    ee_to_ttx
)

PROCESSES=("$@")
[ ${#PROCESSES[@]} -gt 0 ] || PROCESSES=("${DEFAULT_PROCESSES[@]}")

LHAPDF_DATA_PATH="$ROOT/validation/pdf${LHAPDF_DATA_PATH:+:$LHAPDF_DATA_PATH}"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

say() { printf '%s\n' ">>> $*" >&2; }
die() { printf '%s\n' "!!! $*" >&2; exit 1; }

# A generated process directory carries its own MadGraph settings and
# `bin/generate_events` reads that copy, not the caller's environment. Both of
# these default to on and both reach the desktop, so a batch of replays would
# take over the machine.
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

# banked_seed DIR — the seed MadGraph assigned the banked run. The run card on
# disk still says 0 ("assign automatically"); the banner records what that
# resolved to, and handing it back explicitly is what makes the replay the same
# run.
banked_seed() {
  awk '/= iseed /{print $1; exit}' "$1/Events/run_01/run_01_tag_1_banner.txt"
}

# prepare NAME — a pristine process directory for NAME with the clustering
# instrumentation applied and the banked cards installed.
#
# Always from scratch. A second `generate_events` in a directory that has
# already run inherits its survey grids and integrates a different number of
# points, so it is a different run with a different cross section — which shows
# up as a per-event weight that does not match the bank.
prepare() {
  local name="$1"
  local banked="$OUT/$name" pdir="$WORK/$name"

  [ -d "$banked/SubProcesses" ] || die "$name: no banked run at $banked"
  [ -f "$HERE/scripts/$name.mg5" ] || die "$name: no .mg5 script"

  rm -rf "$pdir"
  mkdir -p "$WORK"

  say "[$name] generating a fresh process directory"
  local driver
  driver="$(mktemp -t vg_kt_XXXXXX).mg5"
  # Everything up to `launch`: the model/define/generate lines that fix the
  # diagram content, with `output` redirected. The launch block's run-card
  # settings arrive with the banked card instead, verbatim.
  sed -E "s|^([[:space:]]*output[[:space:]]+)[^[:space:]]+|\1$pdir|" "$HERE/scripts/$name.mg5" |
    sed -E '/^[[:space:]]*launch[[:space:]]*$/,$d' > "$driver"
  grep -qE "^[[:space:]]*output[[:space:]]+$pdir( |$)" "$driver" ||
    die "$name: .mg5 script has no 'output' line to redirect"

  local log="$WORK/$name.generate.log"
  if ! LDFLAGS="${LDFLAGS:-} -lc++" bash "$HERE/mg5_pinned.sh" "$driver" > "$log" 2>&1; then
    tail -40 "$log" >&2
    die "$name: process generation failed (log: $log)"
  fi
  rm -f "$driver"

  say "[$name] installing the banked cards"
  cp "$banked/Cards/run_card.dat" "$pdir/Cards/run_card.dat"
  cp "$banked/Cards/param_card.dat" "$pdir/Cards/param_card.dat"
  local seed
  seed="$(banked_seed "$banked")"
  [ -n "$seed" ] || die "$name: could not read the banked seed"
  say "[$name] banked seed = $seed"
  # `0 = iseed` means "assign automatically", which the banked run resolved to
  # the banner's value. Pinning it is what makes the replay reproduce the bank.
  perl -pi -e "s/^(\s*)\S+(\s*= iseed)/\${1}$seed\${2}/" "$pdir/Cards/run_card.dat"
  grep -qE "^\s*$seed\s*= iseed" "$pdir/Cards/run_card.dat" ||
    die "$name: failed to pin iseed = $seed in the replay run card"
  silence_madgraph_ui "$pdir/Cards/me5_configuration.txt"

  say "[$name] applying the clustering instrumentation"
  bash "$WRAPPERS/ktdump_apply.sh" "$pdir"
}

# replay NAME — run the instrumented build and check it reproduced the bank.
replay() {
  local name="$1"
  local banked="$OUT/$name" pdir="$WORK/$name"
  local lhe="$pdir/Events/run_01/unweighted_events.lhe.gz"

  mkdir -p "$pdir/ktdump"
  say "[$name] running madevent (this is the slow part)"
  local log="$WORK/$name.madevent.log"
  VG_KTDUMP="$pdir/ktdump/raw" \
  LDFLAGS="${LDFLAGS:-} -lc++" \
    "$pdir/bin/generate_events" -f run_01 > "$log" 2>&1 ||
    { tail -40 "$log" >&2; die "$name: madevent failed (log: $log)"; }

  [ -s "$lhe" ] || die "$name: replay produced no events"

  say "[$name] checking the replay reproduces the banked event file"
  python3 "$HERE/gen_kt_cluster_dumps.py" reproduce \
    --banked "$banked/Events/run_01/unweighted_events.lhe.gz" \
    --replay "$lhe" ||
    die "$name: replay is NOT byte-identical to the bank — the dump measures a different run"
}

# extract NAME — normalise the raw Fortran dump to JSONL and check that every
# event's dumped scale reproduces the event file's own.
extract() {
  local name="$1"
  local pdir="$WORK/$name"
  say "[$name] normalising the dump and checking the scale gate"
  python3 "$HERE/gen_kt_cluster_dumps.py" extract \
    --name "$name" \
    --raw-dir "$pdir/ktdump" \
    --lhe "$pdir/Events/run_01/unweighted_events.lhe.gz" \
    --out "$WORK/dumps/$name.jsonl.gz"
}

# One run's failure costs its own hours, not the batch's: each is carried
# through on its own and named at the end, and the batch still exits nonzero.
FAILED=()
mkdir -p "$WORK/dumps"
for name in "${PROCESSES[@]}"; do
  # The work area is the cache, as it is for the banked runs: a replay that
  # already produced its events and its dump is read rather than repeated.
  if [ -s "$WORK/$name/Events/run_01/unweighted_events.lhe.gz" ] &&
     [ -s "$WORK/dumps/$name.jsonl.gz" ] && [ "${VG_FORCE:-0}" != 1 ]; then
    say "[$name] already replayed — reading its dump"
    continue
  fi
  ( prepare "$name"
    [ "$STAGE" = prep ] && exit 0
    replay "$name"
    [ "$STAGE" = run ] && exit 0
    extract "$name" ) || FAILED+=("$name")
done

if [ "$STAGE" = all ]; then
  say "writing the dump manifest"
  python3 "$HERE/gen_kt_cluster_dumps.py" manifest \
    --dumps "$WORK/dumps" --out "$HERE/kt_cluster_dump_manifest.json" \
    "${PROCESSES[@]}"
fi

if [ ${#FAILED[@]} -gt 0 ]; then
  die "these replays did not pass: ${FAILED[*]}"
fi
say "all replays reproduced their banked event file and their own scales"
