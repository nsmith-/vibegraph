#!/usr/bin/env bash
# Sweep the evaluator's execution orders on the `eval_strategies` bench: one binary built
# with the `eval-schedule-study` feature runs under every `VIBEGRAPH_EVAL_SCHEDULE` order
# given (the order is chosen when a program is built, so no rebuild is needed). Further
# arms, as `name=path` to a prebuilt `eval_strategies` binary, compare builds — another
# commit, or other codegen flags — under the same orders.
#
# Cells run round-robin, so host drift lands on all of them; the summary is the
# geometric mean over rows of each cell's min-over-rounds time, relative to the first
# arm under the first order. Min, not median: on Apple silicon a thread cannot be pinned
# to a performance core, and a round that lands on an efficiency core runs about 2x slow.
#
# Each round also pads the environment by a different length (the same for every arm
# in the round). A process's memory layout follows its environment and moves a single
# row by up to ~25% on the M3 Max, in a different direction per binary, so a sweep that
# ran every round in one layout would compare layouts as much as builds; the min over
# rounds is each cell's best layout among those drawn. Use at least 4 rounds.
#
# Usage: scripts/bench_schedule.sh [rounds] [schedules...]
#   default schedules: opblocked arena
#   BENCH_ARMS    extra arms, `name=path` each (the built arm is `head`)
#   RUSTFLAGS     codegen flags for the built arm (default: -C target-cpu=native)
#   BENCH_FILTER  criterion filter (default: forward, lanes4, lanes8)
set -euo pipefail

rounds="${1:-4}"
shift || true
schedules=("$@")
[ ${#schedules[@]} -gt 0 ] || schedules=(opblocked arena)
filter="${BENCH_FILTER:-eval_m2/(forward|lanes[48])/}"
export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
out="$root/target/schedule-sweep"
rm -rf "$out"
mkdir -p "$out"

# One variable per arm, not an associative array: macOS ships bash 3.2.
bin_head="$(CARGO_TARGET_DIR="$root/target/schedule-study" cargo bench \
    -p vibegraph-lib --bench eval_strategies --no-run --features eval-schedule-study \
    --message-format=json |
    python3 -c 'import json,sys
for l in sys.stdin:
    m = json.loads(l)
    if m.get("reason") == "compiler-artifact" and m["target"]["name"] == "eval_strategies" and m.get("executable"):
        print(m["executable"])')"
names=(head)
read -r -a extra <<<"${BENCH_ARMS:-}"
for arm in ${extra[@]+"${extra[@]}"}; do
    eval "bin_${arm%%=*}=\"${arm#*=}\""
    names+=("${arm%%=*}")
done

pads=(0 200 640 1500 3000 5000 96 900)
for r in $(seq 1 "$rounds"); do
    pad="$(printf '%*s' "${pads[$(((r - 1) % ${#pads[@]}))]}" '' | tr ' ' x)"
    for sched in "${schedules[@]}"; do
        for arm in "${names[@]}"; do
            eval "bin=\$bin_$arm"
            dir="$out/$arm@$sched-$r"
            mkdir -p "$dir"
            echo "round $r: $arm @ $sched" >&2
            (cd "$root/vibegraph-lib" &&
                VIBEGRAPH_BENCH_PAD="$pad" VIBEGRAPH_EVAL_SCHEDULE="$sched" \
                    CRITERION_HOME="$dir/criterion" \
                    "$bin" --bench --noplot "$filter" >"$dir/log.txt")
        done
    done
done

python3 - "$out" "${names[0]}@${schedules[0]}" <<'EOF'
import collections, json, math, pathlib, sys
out, ref = pathlib.Path(sys.argv[1]), sys.argv[2]
est = collections.defaultdict(list)
for d in out.iterdir():
    cell = d.name.rsplit("-", 1)[0]
    for e in (d / "criterion" / "eval_m2").glob("*/*/new/estimates.json"):
        est[(cell, e.parts[-4], e.parts[-3])].append(
            json.loads(e.read_text())["median"]["point_estimate"])
cells = sorted({c for c, _, _ in est}, key=lambda c: (c.split("@")[1], c))
benches = sorted({b for _, b, _ in est}, key=lambda b: (b != "forward", b))
rows = sorted({r for _, _, r in est})
print(f"geomean over {len(rows)} rows of time / {ref} [per-row min..max]")
print(f"{'cell':<22}" + "".join(f"{b:>22}" for b in benches))
for c in cells:
    line = f"{c:<22}"
    for b in benches:
        q = [min(est[(c, b, r)]) / min(est[(ref, b, r)])
             for r in rows if est.get((c, b, r)) and est.get((ref, b, r))]
        g = math.exp(sum(map(math.log, q)) / len(q))
        line += f"{g:>8.3f} [{min(q):.2f}..{max(q):.2f}]"
    print(line)
EOF
