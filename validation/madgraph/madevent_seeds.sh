# Shared by the generators that bank a MadEvent reference as one run per seed
# (gen_decay_widths.sh, gen_decay_chain_sigma.sh, gen_decay_chain_events.sh,
# gen_onshell_veto.sh, gen_grammar_sigma.sh, gen_mlm_references.sh,
# bank_seeded_sigma.sh). Sourced, not run.
#
# The caching rule is the madgraph stage's: a process directory that exists is
# never regenerated, and a seed whose run already finished under the same run
# card is read back rather than re-run. Each finished run leaves
# `Events/run_<tag>/vg_seed_result.txt` holding its results.dat value, error,
# wall time and the SHA-256 of the run card it ran with; a later call with the
# same tag reuses it only when the card it would install hashes the same, so
# editing a committed run card re-runs every seed that read it. VG_FORCE=1
# re-runs every seed regardless.
#
# The caller sets HERE (validation/madgraph) before sourcing.

mes_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

# The LDFLAGS a madevent build needs. A proton run links MadEvent against
# LHAPDF's C++ glue, and the conda activation's own LDFLAGS suppresses the
# `STDLIB` line of MadGraph's make_opts (its `ifeq ($(origin LDFLAGS),undefined)`
# guard sees LDFLAGS as already set), so the C++ runtime is named here: libc++ on
# macOS, libstdc++ on Linux. Naming the other one fails the link (build.sh
# carries the full explanation).
mes_ldflags() {
  case "$(uname -s)" in
    Darwin) printf '%s -lc++\n' "${LDFLAGS:-}" ;;
    *) printf '%s -lstdc++\n' "${LDFLAGS:-}" ;;
  esac
}

# A generated process directory carries its own copy of MadGraph's settings, and
# `bin/generate_events` reads that copy: no browser, no desktop notification,
# multicore on NB_CORE cores.
mes_configure_dir() {
  local procdir="$1" nb_core="${2:-2}"
  local cfg="$procdir/Cards/me5_configuration.txt"
  grep -vE '^\s*#?\s*(automatic_html_opening|notification_center|run_mode|nb_core)\s*=' "$cfg" > "$cfg.tmp" || true
  printf 'automatic_html_opening = False\nnotification_center = False\nrun_mode = 2\nnb_core = %s\n' "$nb_core" >> "$cfg.tmp"
  mv "$cfg.tmp" "$cfg"
}

# Generate one process directory unless it exists. `lines` are the proc card's
# lines after `import model sm`, `;`-separated. The generated default run card is
# kept as Cards/run_card_default.dat. VG_MG5_ROOT, when set, names the MadGraph
# tree mg5_pinned.sh runs.
mes_generate_dir() {
  local procdir="$1" lines="$2" nb_core="${3:-2}"
  if [ ! -f "$procdir/bin/generate_events" ]; then
    echo ">>> generating '$lines' into $procdir ..." >&2
    local script
    script="$(mktemp "${TMPDIR:-/tmp}/vg_seeds_XXXXXX")"
    {
      printf 'import model sm\n'
      printf '%s\n' "$lines" | tr ';' '\n'
      printf 'output %s -nojpeg\n' "$procdir"
    } > "$script"
    bash "$HERE/mg5_pinned.sh" "$script" >&2
    rm -f "$script"
    cp "$procdir/Cards/run_card.dat" "$procdir/Cards/run_card_default.dat"
  else
    echo ">>> $procdir exists, not regenerated" >&2
  fi
  mes_configure_dir "$procdir" "$nb_core"
}

# Write `src` into `dst` with `overrides` (`key=value;...`) and iseed applied. A
# key the card does not carry is appended.
mes_install_card() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import re, sys
src, dst, overrides, seed = sys.argv[1:5]
text = open(src).read()
pairs = [kv.split("=", 1) for kv in overrides.split(";") if kv] + [["iseed", seed]]
for key, value in pairs:
    pat = r"^\s*\S+\s*=\s*%s\b" % re.escape(key)
    text, n = re.subn(pat, "  %s = %s" % (value, key), text, flags=re.M | re.I)
    assert n <= 1, "%s found %d times" % (key, n)
    if n == 0:
        text += "  %s = %s\n" % (value, key)
open(dst, "w").write(text)
PY
}

# Run madevent once on the card already installed in the process directory and
# echo "<value> <err> <wall_s>" from results.dat (pb, or GeV for a decay). A run
# of the same tag that finished under a byte-identical run card is read back
# instead.
mes_run_seed() {
  local procdir="$1" tag="$2" log="$3"
  local rundir="$procdir/Events/run_$tag"
  local record="$rundir/vg_seed_result.txt"
  local card_sha
  card_sha="$(mes_sha256 "$procdir/Cards/run_card.dat")"
  if [ "${VG_FORCE:-0}" != 1 ] && [ -s "$record" ]; then
    local value err wall sha
    read -r value err wall sha < "$record"
    if [ "$sha" = "$card_sha" ]; then
      echo ">>> [$tag] cached: $value +- $err" >&2
      printf '%s %s %s\n' "$value" "$err" "$wall"
      return 0
    fi
    echo ">>> [$tag] cached under another run card; re-running" >&2
  fi
  echo ">>> [$tag] madevent ..." >&2
  rm -rf "$rundir"
  local started scratch status=0
  started="$(date +%s)"
  # From a scratch working directory: a failed generate_events leaves an
  # ME5_debug link in whatever directory it was started from.
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/vg_seed_cwd_XXXXXX")"
  (cd "$scratch" && LDFLAGS="$(mes_ldflags)" "$procdir/bin/generate_events" -f "run_$tag") \
    > "$log" 2>&1 || status=$?
  rm -rf "$scratch"
  if [ "$status" != 0 ]; then
    echo "!!! [$tag] generate_events failed; see $log" >&2
    tail -40 "$log" >&2
    return 1
  fi
  local wall=$(($(date +%s) - started))
  local result
  result="$(awk -v wall="$wall" 'NR==1{printf "%.10g %.10g %d\n", $1, $2, wall}' \
    "$procdir/SubProcesses/results.dat")"
  [ -d "$rundir" ] || {
    echo "!!! [$tag] madevent wrote no $rundir" >&2
    return 1
  }
  printf '%s %s\n' "$result" "$card_sha" > "$record"
  echo "$result"
}

# mes_check_run PROCDIR TAG NEVENTS — the run `run_<tag>` holds what it ran
# for: a positive finite cross section on results.dat's first line, and at least
# NEVENTS events in its event file. generate_events exits 0 on runs that hold
# neither (a parallel compile that lost the `libgammaUPC.a` race leaves an empty
# results.dat), so its exit status is not this check. The count is written to
# `Events/run_<tag>/vg_events_written.txt`.
mes_check_run() {
  local procdir="$1" tag="$2" nev="$3"
  local rundir="$procdir/Events/run_$tag"
  python3 - "$procdir/SubProcesses/results.dat" "$rundir/unweighted_events.lhe.gz" \
    "$nev" "$rundir/vg_events_written.txt" <<'PY'
import gzip, math, re, sys
results, lhe, nev, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
try:
    fields = open(results).readline().split()
    value, err = (float(f.replace("D", "E")) for f in fields[:2])
except (OSError, ValueError) as e:
    sys.exit("no cross section on the first line of %s: %s" % (results, e))
if not (math.isfinite(value) and value > 0 and math.isfinite(err)):
    sys.exit("%s reports %r +- %r, not a positive finite cross section" % (results, value, err))
event = re.compile(r"^\s*<event[\s>]")
try:
    with gzip.open(lhe, "rt") as f:
        n = sum(1 for line in f if event.match(line))
except (OSError, EOFError) as e:
    sys.exit("cannot read the event file %s: %s" % (lhe, e))
if n < nev:
    sys.exit("%s holds %d events, the run card asked for %d" % (lhe, n, nev))
open(out, "w").write("%d\n" % n)
PY
}

# mes_launch_sets SCRIPT — the `set key value` lines of a .mg5 script's launch
# block, one `key value` per line.
mes_launch_sets() {
  sed -n '/^[[:space:]]*launch[[:space:]]*$/,/^[[:space:]]*done[[:space:]]*$/p' "$1" |
    awk '$1 == "set" { print $2, $3 }'
}

# mes_check_card SCRIPT CARD [KEYS] — every launch-block setting of SCRIPT is
# what the run card CARD says, so a run made on CARD is the run SCRIPT describes.
# KEYS (space-separated) are left out: the ones the caller sets per run itself.
# A script with no setting left to compare is refused: the check would hold
# whatever CARD says.
mes_check_card() {
  mes_launch_sets "$1" | awk -v skip=" ${3:-} " 'index(skip, " " $1 " ") == 0' | python3 -c '
import re, sys
card = open(sys.argv[1]).read()
bad = []
compared = 0
for line in sys.stdin:
    key, want = line.split()
    compared += 1
    m = re.search(r"^\s*(\S+)\s*=\s*%s\b" % re.escape(key), card, re.M | re.I)
    got = m.group(1) if m else None
    try:
        same = got is not None and float(got) == float(want)
    except ValueError:
        same = got is not None and got.lower() == want.lower()
    if not same:
        bad.append("%s: script %s, card %s" % (key, want, got))
if not compared:
    sys.exit("no launch-block setting to compare (a bare `launch` line, then `set key value`)")
if bad:
    sys.exit("run card disagrees with the script launch block: " + "; ".join(bad))
' "$2"
}

# mes_p1d_tree WORK PATCH — a copy of the pinned MadGraph tree with PATCH
# applied, made once under WORK and printed. A copy that exists is reused only
# if it was made from this patch; the pinned checkout itself is never modified.
mes_p1d_tree() {
  local work="$1" patch_file="$2" tree stamp root
  root="$(cd "$HERE/../.." && pwd)"
  tree="$work/mg5amcnlo-p1d"
  stamp="$(mes_sha256 "$patch_file")"
  if [ ! -f "$tree/.vg_patch" ] || [ "$(cat "$tree/.vg_patch")" != "$stamp" ]; then
    echo ">>> copying the pinned MadGraph tree and applying $(basename "$patch_file")" >&2
    rm -rf "$tree"
    mkdir -p "$work"
    cp -a "$root/research/refs/mg5amcnlo" "$tree"
    rm -rf "$tree/aloha/__pycache__"
    find "$tree/models" -name aloha.pkl -delete
    patch -p1 -s -d "$tree" < "$patch_file"
    echo "$stamp" > "$tree/.vg_patch"
  fi
  echo "$tree"
}
