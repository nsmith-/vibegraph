#!/usr/bin/env bash
# Apple silicon only: record the evaluator under Instruments' CPU Counters template and
# print its bottleneck breakdown — the share of cycles that were useful, stalled on
# instruction delivery, stalled on processing, or discarded (flushed after a
# misprediction). No sudo is needed; xctrace ships with Xcode.
#
# Usage: scripts/xctrace_bottlenecks.sh <bench-binary> <criterion-filter> [schedule...]
#   <bench-binary>      an `eval_strategies` executable, e.g. from
#                       `cargo bench -p vibegraph-lib --bench eval_strategies --no-run
#                        --features eval-schedule-study`
#   <criterion-filter>  one benchmark, e.g. 'eval_m2/forward/gg_to_gg$'
#   [schedule...]       VIBEGRAPH_EVAL_SCHEDULE orders to record (default: opblocked)
#   PROFILE_TIME        seconds of criterion's --profile-time loop (default: 10)
#
# Each run's shares are cycle-weighted over the 10 ms buckets of its last
# (PROFILE_TIME - 1) seconds, so evaluator construction stays out of them. The
# template's default counting mode is Apple's guided "bottlenecks" mode; picking
# individual counters (e.g. indirect-branch mispredicts) needs a template saved from
# the Instruments GUI and passed as --template.
set -euo pipefail

bin="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
filter="$2"
shift 2
schedules=("$@")
[ ${#schedules[@]} -gt 0 ] || schedules=(opblocked)
profile_time="${PROFILE_TIME:-10}"

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/target/xctrace"
mkdir -p "$out"

traces=()
for sched in "${schedules[@]}"; do
    trace="$out/$(basename "$bin")@$sched.trace"
    rm -rf "$trace"
    echo "recording $sched" >&2
    (cd "$root/vibegraph-lib" &&
        xcrun xctrace record --quiet --no-prompt --template 'CPU Counters' \
            --time-limit "$((profile_time + 110))s" --output "$trace" \
            --env VIBEGRAPH_EVAL_SCHEDULE="$sched" \
            --launch -- "$bin" --bench --profile-time "$profile_time" "$filter" >/dev/null)
    traces+=("$trace")
done

python3 - "$profile_time" "${traces[@]}" <<'EOF'
import collections, subprocess, sys, xml.etree.ElementTree as ET

window_ns = (int(sys.argv[1]) - 1) * 1_000_000_000
keys = ["useful", "delivery", "processing", "discarded"]

def rows(trace, schema):
    xml = subprocess.run(
        ["xcrun", "xctrace", "export", "--input", trace, "--xpath",
         f'/trace-toc/run[@number="1"]/data/table[@schema="{schema}"]'],
        capture_output=True, text=True, check=True).stdout
    seen = {}
    for row in ET.fromstring(xml).iter("row"):
        vals = []
        for el in row:
            if "ref" in el.attrib:
                el = seen[el.attrib["ref"]]
            elif "id" in el.attrib:
                seen[el.attrib["id"]] = el
            vals.append(el)
        yield vals

print(f"{'run':<52}" + "".join(f"{k:>12}" for k in keys) + f"{'Gcycles':>10}")
for trace in sys.argv[2:]:
    buckets = collections.defaultdict(dict)
    # columns: start, duration, process, metric-int, metric-double, metric-name, is-precise
    for r in rows(trace, "MetricAggregationForProcess"):
        if int(r[1].text) != 10_000_000 or r[6].text != "0":
            continue
        name = r[5].text
        buckets[int(r[0].text)][name] = int(r[3].text) if name == "cycle" else float(r[4].text)
    end = max(buckets)
    share, cycles = collections.Counter(), 0
    for t, m in buckets.items():
        total = sum(m.get(k, 0.0) for k in keys)
        if t < end - window_ns or "cycle" not in m or total <= 0:
            continue
        for k in keys:
            share[k] += m["cycle"] * m.get(k, 0.0) / total
        cycles += m["cycle"]
    name = trace.rsplit("/", 1)[-1].removesuffix(".trace")
    print(f"{name:<52}" + "".join(f"{100 * share[k] / cycles:>11.1f}%" for k in keys)
          + f"{cycles / 1e9:>10.2f}")
EOF
