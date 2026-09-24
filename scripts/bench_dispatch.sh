#!/usr/bin/env bash
# A/B the evaluator's two instruction dispatchers on the `eval_strategies` bench:
# the `match` loop against the tail-call-threaded handlers (`threaded-dispatch`,
# which needs nightly for `become`). Both arms build with the same pinned nightly
# and the same RUSTFLAGS, so the compiler is not a variable; each arm keeps its own
# target directory, so criterion's per-arm results never overwrite each other.
#
# The arms run interleaved, A B A B ..., so host drift lands on both; the summary
# takes the median of each arm's per-round criterion point estimates and prints
# threaded/match per row and benchmark.
#
# Usage: scripts/bench_dispatch.sh [rounds] [criterion filter]
#   VIBEGRAPH_NIGHTLY    toolchain (default: the pin below, shared with ci.yml)
#   RUSTFLAGS            codegen flags for both arms (default: -C target-cpu=native)
set -euo pipefail

rounds="${1:-2}"
filter="${2:-eval_m2/(forward|lanes[248])/}"
toolchain="${VIBEGRAPH_NIGHTLY:-nightly-2026-09-24}"
export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
out="$root/target/dispatch-ab"
mkdir -p "$out"

build() { # <arm> [cargo args...]
    local arm="$1"
    shift
    CARGO_TARGET_DIR="$root/target/dispatch-$arm" cargo "+$toolchain" bench \
        -p vibegraph-lib --bench eval_strategies --no-run --message-format=json "$@" |
        python3 -c 'import json,sys
for l in sys.stdin:
    m = json.loads(l)
    if m.get("reason") == "compiler-artifact" and m["target"]["name"] == "eval_strategies" and m.get("executable"):
        print(m["executable"])'
}

match_bin="$(build match)"
threaded_bin="$(build threaded --features threaded-dispatch)"

for r in $(seq 1 "$rounds"); do
    for arm in match threaded; do
        bin="${arm}_bin"
        echo "round $r: $arm" >&2
        # Criterion writes under the bench's working directory's target/criterion;
        # a per-arm, per-round directory keeps every estimate on disk.
        dir="$out/$arm-$r"
        mkdir -p "$dir"
        (cd "$dir" && CRITERION_HOME="$dir/criterion" "${!bin}" --bench --noplot "$filter" >"$dir/log.txt")
    done
done

python3 - "$out" "$rounds" <<'EOF'
import json, pathlib, statistics, sys
out, rounds = pathlib.Path(sys.argv[1]), int(sys.argv[2])

def estimates(arm):
    per = {}
    for r in range(1, rounds + 1):
        base = out / f"{arm}-{r}" / "criterion" / "eval_m2"
        for est in base.glob("*/*/new/estimates.json"):
            bench, row = est.parts[-4], est.parts[-3]
            per.setdefault((row, bench), []).append(
                json.loads(est.read_text())["median"]["point_estimate"])
    return {k: statistics.median(v) for k, v in per.items()}

m, t = estimates("match"), estimates("threaded")
benches = sorted({b for _, b in m}, key=lambda b: (b != "forward", b))
rows = sorted({r for r, _ in m})
print("threaded / match, median over rounds of criterion's median (<1 is faster)")
print(f"{'row':<24}" + "".join(f"{b:>10}" for b in benches))
ratios = {b: [] for b in benches}
for row in rows:
    cells = []
    for b in benches:
        if (row, b) in m and (row, b) in t:
            q = t[(row, b)] / m[(row, b)]
            ratios[b].append(q)
            cells.append(f"{q:>10.3f}")
        else:
            cells.append(f"{'-':>10}")
    print(f"{row:<24}" + "".join(cells))
print(f"{'median':<24}" + "".join(f"{statistics.median(ratios[b]):>10.3f}" for b in benches))
print()
print("match-arm time per event, µs")
print(f"{'row':<24}" + "".join(f"{b:>10}" for b in benches))
for row in rows:
    print(f"{row:<24}" + "".join(
        f"{m[(row, b)] / 16 / 1e3:>10.3f}" if (row, b) in m else f"{'-':>10}" for b in benches))
EOF
