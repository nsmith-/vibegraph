# Shared by the generators that bank a MadEvent reference as one run per seed
# (gen_decay_widths.sh, gen_decay_chain_sigma.sh, gen_decay_chain_events.sh,
# gen_onshell_veto.sh, gen_grammar_sigma.sh). Sourced, not run.
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
  local started
  started="$(date +%s)"
  "$procdir/bin/generate_events" -f "run_$tag" > "$log" 2>&1 || {
    echo "!!! [$tag] generate_events failed; see $log" >&2
    tail -40 "$log" >&2
    return 1
  }
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
