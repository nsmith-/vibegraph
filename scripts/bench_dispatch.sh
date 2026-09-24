#!/usr/bin/env bash
# Sweep the evaluator's instruction dispatchers against its execution orders on the
# `eval_strategies` bench: the `match` loop and the tail-call-threaded handlers
# (`threaded-dispatch`, which needs nightly for `become`), each under every
# `VIBEGRAPH_EVAL_SCHEDULE` order given. Both arms build with the same pinned nightly,
# the same RUSTFLAGS and the `eval-schedule-study` feature, so neither the compiler nor
# the schedule hook is a variable between them; each arm keeps its own target
# directory. The order is chosen when a program is built, so one binary per arm
# serves every schedule.
#
# Cells run round-robin, so host drift lands on all of them; the summary is the
# geometric mean over rows of each cell's median-over-rounds time, relative to
# `match` under the production order.
#
# Usage: scripts/bench_dispatch.sh [rounds] [schedules...]
#   default schedules: opblocked arena opwin32
#   VIBEGRAPH_NIGHTLY    toolchain (default: the pin below, shared with ci.yml)
#   RUSTFLAGS            codegen flags for both arms (default: -C target-cpu=native)
#   BENCH_FILTER         criterion filter (default: forward, lanes4, lanes8)
set -euo pipefail

rounds="${1:-2}"
shift || true
schedules=("$@")
[ ${#schedules[@]} -gt 0 ] || schedules=(opblocked arena opwin32)
filter="${BENCH_FILTER:-eval_m2/(forward|lanes[48])/}"
toolchain="${VIBEGRAPH_NIGHTLY:-nightly-2026-09-24}"
export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
out="$root/target/dispatch-sweep"
rm -rf "$out"
mkdir -p "$out"

build() { # <arm> <features>
    CARGO_TARGET_DIR="$root/target/dispatch-$1" cargo "+$toolchain" bench \
        -p vibegraph-lib --bench eval_strategies --no-run --features "$2" \
        --message-format=json |
        python3 -c 'import json,sys
for l in sys.stdin:
    m = json.loads(l)
    if m.get("reason") == "compiler-artifact" and m["target"]["name"] == "eval_strategies" and m.get("executable"):
        print(m["executable"])'
}

declare -A bin
bin[match]="$(build match eval-schedule-study)"
bin[threaded]="$(build threaded eval-schedule-study,threaded-dispatch)"

for r in $(seq 1 "$rounds"); do
    for sched in "${schedules[@]}"; do
        for arm in match threaded; do
            dir="$out/$arm@$sched-$r"
            mkdir -p "$dir"
            echo "round $r: $arm @ $sched" >&2
            (cd "$root/vibegraph-lib" &&
                VIBEGRAPH_EVAL_SCHEDULE="$sched" CRITERION_HOME="$dir/criterion" \
                    "${bin[$arm]}" --bench --noplot "$filter" >"$dir/log.txt")
        done
    done
done

python3 - "$out" <<'EOF'
import collections, json, math, pathlib, statistics, sys
out = pathlib.Path(sys.argv[1])
est = collections.defaultdict(list)
for d in out.iterdir():
    cell = d.name.rsplit("-", 1)[0]
    for e in (d / "criterion" / "eval_m2").glob("*/*/new/estimates.json"):
        est[(cell, e.parts[-4], e.parts[-3])].append(
            json.loads(e.read_text())["median"]["point_estimate"])
cells = sorted({c for c, _, _ in est}, key=lambda c: (c.split("@")[1], c))
benches = sorted({b for _, b, _ in est}, key=lambda b: (b != "forward", b))
rows = sorted({r for _, _, r in est})
ref = "match@opblocked"
print(f"geomean over {len(rows)} rows of time / {ref} [per-row min..max]")
print(f"{'cell':<22}" + "".join(f"{b:>22}" for b in benches))
for c in cells:
    line = f"{c:<22}"
    for b in benches:
        q = [statistics.median(est[(c, b, r)]) / statistics.median(est[(ref, b, r)])
             for r in rows if est.get((c, b, r)) and est.get((ref, b, r))]
        g = math.exp(sum(map(math.log, q)) / len(q))
        line += f"{g:>8.3f} [{min(q):.2f}..{max(q):.2f}]"
    print(line)
EOF
