#!/usr/bin/env bash
# Usage: [RUSTFLAGS=...] mg_perf_compare.sh [--skip-bench]
#
# Measures the vibegraph-vs-MadGraph evaluator gap on the current host and
# emits a ratio table comparable across platforms. Two sides:
#
#   MG:        MATRIX1 ns/eval from mg_timings.json's `processes` table.
#              Preferred source: validation/madgraph/output/mg_timings.json,
#              written by `pixi run -e madgraph generate-amplitude` on THIS
#              machine (regenerate it before trusting the ratios). Falls back
#              to the committed validation/madgraph/mg_timings.json — a
#              host-labelled snapshot for a work area that has none — in
#              which case the ratios compare two different machines and the
#              script says so. Either way the file's own `host` block, not
#              its name or its build products, is the source of that host
#              identity.
#   vibegraph: the honest release bench — criterion median ns/iter of the
#              `eval_m2/forward/*` rows of benches/eval_strategies.rs, divided
#              by the bench's points-per-iteration batch size. Never the
#              `amplitude_oracle` timing report (its extended-validation build
#              compiles per-node cross-checks into the eval loop). The bench's
#              own row set comes from validation/manifest.toml's
#              `mg_amplitude` tables, so it and mg_timings.json are always
#              generated from the same registry.
#
# Runs the forward (scalar) bench rows only; SIMD lane-width questions are the
# separate AVX-512 kit (scripts/dump_lane_asm.sh, note 18 §5). RUSTFLAGS is
# honored from the environment, but recorded reference tables use default
# codegen on both sides — if you raise one side (e.g. -C target-cpu=native),
# raise the Fortran side too (add -march=native to build_amplitude.sh's
# --f77flags) or the ratio is not comparable.
#
# --skip-bench joins against existing target/criterion results without
# re-running the bench (each result row prints its measurement mtime, so stale
# joins are visible).
#
# A process on only one side of the mg_timings.json / criterion join is
# reported, not silently dropped: either the manifest gained a row the bench
# has not been re-run against, or a bench row's compiled MATRIX1 module and
# timing entry do not exist yet.
#
# Output: fingerprint + table on stdout; the same as markdown + TSV in
# target/mg-perf/mg_compare_<os>_<arch>.{md,tsv} for banking in the perf
# notes (15, 20).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MG_JSON_WORKAREA="$REPO_ROOT/validation/madgraph/output/mg_timings.json"
MG_JSON_COMMITTED="$REPO_ROOT/validation/madgraph/mg_timings.json"
BENCH_SRC="$REPO_ROOT/vibegraph-lib/benches/eval_strategies.rs"
CRIT_DIR="$REPO_ROOT/target/criterion/eval_m2/forward"
OUT_DIR="$REPO_ROOT/target/mg-perf"

SKIP_BENCH=0
if [[ "${1:-}" == "--skip-bench" ]]; then SKIP_BENCH=1; fi

if [[ -f "$MG_JSON_WORKAREA" ]]; then
    MG_JSON="$MG_JSON_WORKAREA"
    MG_JSON_SOURCE="work area"
elif [[ -f "$MG_JSON_COMMITTED" ]]; then
    MG_JSON="$MG_JSON_COMMITTED"
    MG_JSON_SOURCE="committed fallback"
    echo "note: no work-area mg_timings.json — falling back to the committed" >&2
    echo "      snapshot ($MG_JSON_COMMITTED). Regenerate on this machine with" >&2
    echo "      'pixi run -e madgraph generate-amplitude' to compare like-for-like." >&2
else
    echo "error: no mg_timings.json in the work area or committed" >&2
    echo "generate it on this machine first: pixi run -e madgraph generate-amplitude" >&2
    exit 1
fi

# Points per bench iteration: the bench sums eval_m2 over one fixed batch of
# phase-space points per iteration, so ns/eval = criterion ns/iter ÷ batch.
PPI=$(grep -o '(0\.\.[0-9]*)' "$BENCH_SRC" | head -1 | grep -o '[0-9]*' | tail -1)
if [[ -z "$PPI" ]]; then
    echo "error: could not parse the points-per-iteration batch size from $BENCH_SRC" >&2
    exit 1
fi

if [[ "$SKIP_BENCH" -eq 0 ]]; then
    cargo bench -p vibegraph-lib --bench eval_strategies -- '^eval_m2/forward/'
fi
if [[ ! -d "$CRIT_DIR" ]]; then
    echo "error: no criterion results at $CRIT_DIR (run without --skip-bench)" >&2
    exit 1
fi

case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
    darwin) CPU=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown) ;;
    linux)  CPU=$(sed -n 's/^model name[^:]*: //p' /proc/cpuinfo | head -1) ;;
    *)      CPU=unknown ;;
esac
GIT_HEAD=$(git -C "$REPO_ROOT" rev-parse --short HEAD)
git -C "$REPO_ROOT" diff --quiet HEAD 2>/dev/null || GIT_HEAD="$GIT_HEAD-dirty"

mkdir -p "$OUT_DIR"
OS_TAG=$(uname -s | tr '[:upper:]' '[:lower:]')
OUT_BASE="$OUT_DIR/mg_compare_${OS_TAG}_$(uname -m)"

env CPU="$CPU" GIT_HEAD="$GIT_HEAD" RUSTFLAGS="${RUSTFLAGS:-}" \
    MG_JSON="$MG_JSON" MG_JSON_SOURCE="$MG_JSON_SOURCE" CRIT_DIR="$CRIT_DIR" \
    PPI="$PPI" OUT_BASE="$OUT_BASE" THIS_OS="$OS_TAG" \
python3 - <<'EOF'
import glob, json, math, os, platform, subprocess, time

mg_full = json.load(open(os.environ["MG_JSON"]))
# An unwrapped {name: timing} map is the pre-`host` layout: it carries no host
# identity, so its ns/eval cannot be attributed to a machine. Say that, rather
# than reading an absent `processes` key as an empty table and reporting the
# empty join it produces.
if "processes" not in mg_full:
    raise SystemExit(
        f"{os.environ['MG_JSON']} predates the host-labelled schema (no "
        "`processes`/`host` keys). Regenerate it with "
        "'pixi run -e madgraph generate-amplitude'."
    )
mg = mg_full["processes"]
mg_host = mg_full.get("host", {}) or {}
mg_mtime = os.path.getmtime(os.environ["MG_JSON"])
mg_source = os.environ["MG_JSON_SOURCE"]
ppi = int(os.environ["PPI"])

# The other-host warning: driven by mg_timings.json's own `host` block, not by
# inferring the build platform from a build product's filename.
mg_os = (mg_host.get("os") or {}).get("family")
this_os = os.environ["THIS_OS"]
host_mismatch = mg_os is not None and mg_os != this_os
if host_mismatch or mg_source == "committed fallback":
    print(f"WARNING: mg_timings.json (source: {mg_source}) was captured on "
          f"{mg_host.get('cpu', {}).get('model', 'an unknown CPU')} "
          f"({mg_os or 'unknown OS'}) at {mg_host.get('captured', 'an unknown time')}, "
          f"not on this host ({this_os}). Ratios compare two machines; "
          f"regenerate with 'pixi run -e madgraph generate-amplitude' on this "
          f"machine before trusting them.", file=__import__("sys").stderr)

rows = []
for est_path in sorted(glob.glob(os.path.join(os.environ["CRIT_DIR"], "*", "new", "estimates.json"))):
    name = os.path.basename(os.path.dirname(os.path.dirname(est_path)))
    if name not in mg:
        continue
    median = json.load(open(est_path))["median"]["point_estimate"]
    rows.append({
        "process": name,
        "mg_ns": mg[name]["ns_per_eval"],
        "vg_ns": median / ppi,
        "bench_date": time.strftime("%Y-%m-%d %H:%M", time.localtime(os.path.getmtime(est_path))),
    })
if not rows:
    raise SystemExit("no criterion process overlaps mg_timings.json")

# Report, rather than silently drop, whichever side of the join a process is
# missing from: a manifest row the bench has not been re-run against, or a
# bench row with no compiled MATRIX1 module / timing entry yet.
bench_names = {
    os.path.basename(os.path.dirname(os.path.dirname(p)))
    for p in glob.glob(os.path.join(os.environ["CRIT_DIR"], "*", "new", "estimates.json"))
}
mg_only = sorted(set(mg) - bench_names)
bench_only = sorted(bench_names - set(mg))
unjoined_lines = []
if mg_only:
    unjoined_lines.append(f"in mg_timings.json but not in the criterion results: {', '.join(mg_only)}")
if bench_only:
    unjoined_lines.append(f"in the criterion results but not in mg_timings.json: {', '.join(bench_only)}")

rows.sort(key=lambda r: r["mg_ns"])
for r in rows:
    r["ratio"] = r["vg_ns"] / r["mg_ns"]
geomean = math.exp(sum(math.log(r["ratio"]) for r in rows) / len(rows))

rustc = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
mg_cpu = (mg_host.get("cpu") or {}).get("model", "unknown")
fingerprint = [
    ("host (vibegraph)", f"{platform.system()} {platform.machine()}  ({os.environ['CPU']})"),
    ("git", os.environ["GIT_HEAD"]),
    ("rustc", rustc),
    ("RUSTFLAGS", os.environ["RUSTFLAGS"] or "(unset — default codegen, matches recorded tables)"),
    ("host (MG)", f"{mg_cpu} ({mg_os or 'unknown OS'}), captured {mg_host.get('captured', 'unknown')}"),
    ("mg_timings.json", f"{mg_source}, file mtime {time.strftime('%Y-%m-%d %H:%M', time.localtime(mg_mtime))}"),
    ("run date", time.strftime("%Y-%m-%d %H:%M")),
]

def fmt(ns):
    return f"{ns:,.0f}"

md = ["# vibegraph vs MadGraph evaluator timing", ""]
md += [f"- **{k}**: {v}" for k, v in fingerprint]
md += ["", "| process | MG ns/eval | vibegraph ns/eval | vg/MG | bench measured |",
       "|---|--:|--:|--:|:--|"]
for r in rows:
    md.append(f"| {r['process']} | {fmt(r['mg_ns'])} | {fmt(r['vg_ns'])} "
              f"| {r['ratio']:.2f}× | {r['bench_date']} |")
md += ["", f"Geometric-mean ratio: **{geomean:.2f}×** over {len(rows)} processes "
           f"(range {min(r['ratio'] for r in rows):.2f}×–{max(r['ratio'] for r in rows):.2f}×).", ""]
if unjoined_lines:
    md += ["Not joined (present on one side only):", ""]
    md += [f"- {line}" for line in unjoined_lines]
    md += [""]
md_text = "\n".join(md)

out_base = os.environ["OUT_BASE"]
with open(out_base + ".md", "w") as f:
    f.write(md_text)
with open(out_base + ".tsv", "w") as f:
    for k, v in fingerprint:
        f.write(f"# {k}: {v}\n")
    for line in unjoined_lines:
        f.write(f"# not joined: {line}\n")
    f.write("process\tmg_ns_per_eval\tvg_ns_per_eval\tratio\n")
    for r in rows:
        f.write(f"{r['process']}\t{r['mg_ns']:.1f}\t{r['vg_ns']:.1f}\t{r['ratio']:.3f}\n")

print()
for k, v in fingerprint:
    print(f"{k:>18}: {v}")
print()
print(f"{'process':<24}{'MG ns/eval':>12}{'vg ns/eval':>14}{'vg/MG':>8}   bench measured")
for r in rows:
    print(f"{r['process']:<24}{fmt(r['mg_ns']):>12}{fmt(r['vg_ns']):>14}"
          f"{r['ratio']:>7.2f}×   {r['bench_date']}")
print()
print(f"geomean ratio {geomean:.2f}× over {len(rows)} processes")
for line in unjoined_lines:
    print(f"not joined: {line}")
print(f"banked: {out_base}.md  {out_base}.tsv")
EOF
