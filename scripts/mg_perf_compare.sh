#!/usr/bin/env bash
# Usage: [RUSTFLAGS=...] mg_perf_compare.sh [--skip-bench]
#
# Measures the vibegraph-vs-MadGraph evaluator gap on the current host and
# emits a ratio table comparable across platforms. Two sides:
#
#   MG:        MATRIX1 ns/eval from validation/madgraph/output/mg_timings.json,
#              written by `pixi run -e madgraph generate-amplitude`. The JSON
#              (and the mg_*.so f2py modules it times) are host-specific —
#              regenerate them on THIS machine before trusting the ratios.
#   vibegraph: the honest release bench — criterion median ns/iter of the
#              `eval_m2/forward/*` rows of benches/eval_strategies.rs, divided
#              by the bench's points-per-iteration batch size. Never the
#              `validate_helas_mg` timing report (its extended-validation build
#              compiles per-node cross-checks into the eval loop).
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
# Output: fingerprint + table on stdout; the same as markdown + TSV in
# target/mg-perf/mg_compare_<os>_<arch>.{md,tsv} for banking in the perf
# notes (15, 20).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MG_JSON="$REPO_ROOT/validation/madgraph/output/mg_timings.json"
BENCH_SRC="$REPO_ROOT/vibegraph-lib/benches/eval_strategies.rs"
CRIT_DIR="$REPO_ROOT/target/criterion/eval_m2/forward"
OUT_DIR="$REPO_ROOT/target/mg-perf"

SKIP_BENCH=0
if [[ "${1:-}" == "--skip-bench" ]]; then SKIP_BENCH=1; fi

if [[ ! -f "$MG_JSON" ]]; then
    echo "error: $MG_JSON not found" >&2
    echo "generate it on this machine first: pixi run -e madgraph generate-amplitude" >&2
    exit 1
fi

# The f2py module filenames embed the platform they were built for; a mismatch
# with the running OS means mg_timings.json was produced on a different host.
OS_TAG=$(uname -s | tr '[:upper:]' '[:lower:]')
SO_SAMPLE=$(ls "$REPO_ROOT/validation/madgraph/"mg_ee_to_mumu.*.so 2>/dev/null | head -1 || true)
if [[ -n "$SO_SAMPLE" && "$SO_SAMPLE" != *"$OS_TAG"* ]]; then
    echo "WARNING: $(basename "$SO_SAMPLE") was built for another OS — mg_timings.json" >&2
    echo "         is from a different host; rerun: pixi run -e madgraph generate-amplitude" >&2
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

case "$OS_TAG" in
    darwin) CPU=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown) ;;
    linux)  CPU=$(sed -n 's/^model name[^:]*: //p' /proc/cpuinfo | head -1) ;;
    *)      CPU=unknown ;;
esac
GIT_HEAD=$(git -C "$REPO_ROOT" rev-parse --short HEAD)
git -C "$REPO_ROOT" diff --quiet HEAD 2>/dev/null || GIT_HEAD="$GIT_HEAD-dirty"

mkdir -p "$OUT_DIR"
OUT_BASE="$OUT_DIR/mg_compare_${OS_TAG}_$(uname -m)"

env CPU="$CPU" GIT_HEAD="$GIT_HEAD" RUSTFLAGS="${RUSTFLAGS:-}" \
    MG_JSON="$MG_JSON" CRIT_DIR="$CRIT_DIR" PPI="$PPI" OUT_BASE="$OUT_BASE" \
python3 - <<'EOF'
import glob, json, math, os, platform, subprocess, time

mg = json.load(open(os.environ["MG_JSON"]))
mg_mtime = os.path.getmtime(os.environ["MG_JSON"])
ppi = int(os.environ["PPI"])

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
rows.sort(key=lambda r: r["mg_ns"])
for r in rows:
    r["ratio"] = r["vg_ns"] / r["mg_ns"]
geomean = math.exp(sum(math.log(r["ratio"]) for r in rows) / len(rows))

rustc = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
fingerprint = [
    ("host", f"{platform.system()} {platform.machine()}  ({os.environ['CPU']})"),
    ("git", os.environ["GIT_HEAD"]),
    ("rustc", rustc),
    ("RUSTFLAGS", os.environ["RUSTFLAGS"] or "(unset — default codegen, matches recorded tables)"),
    ("mg_timings.json", time.strftime("%Y-%m-%d %H:%M", time.localtime(mg_mtime))),
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
md_text = "\n".join(md)

out_base = os.environ["OUT_BASE"]
with open(out_base + ".md", "w") as f:
    f.write(md_text)
with open(out_base + ".tsv", "w") as f:
    f.write("process\tmg_ns_per_eval\tvg_ns_per_eval\tratio\n")
    for r in rows:
        f.write(f"{r['process']}\t{r['mg_ns']:.1f}\t{r['vg_ns']:.1f}\t{r['ratio']:.3f}\n")

print()
for k, v in fingerprint:
    print(f"{k:>16}: {v}")
print()
print(f"{'process':<24}{'MG ns/eval':>12}{'vg ns/eval':>14}{'vg/MG':>8}   bench measured")
for r in rows:
    print(f"{r['process']:<24}{fmt(r['mg_ns']):>12}{fmt(r['vg_ns']):>14}"
          f"{r['ratio']:>7.2f}×   {r['bench_date']}")
print()
print(f"geomean ratio {geomean:.2f}× over {len(rows)} processes")
print(f"banked: {out_base}.md  {out_base}.tsv")
EOF
