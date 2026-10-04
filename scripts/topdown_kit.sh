#!/usr/bin/env bash
# Hardware-counter kit for the helicity evaluator on an x86 host with a PMU.
#
# Builds the `eval_loop` example (`-C target-cpu=native`, release), then runs it under
# `perf stat` over the `eval_strategies` bench rows at lane widths 1, 4 and 8, plus
# execution-order controls, one counter pass at a time. Each run counts only the timed
# evaluation loop: perf starts disabled (`-D -1`) and the driver enables it through
# perf's control FIFO after setup and warm-up. A `perf record` profile of the 2→6 and
# a summary table come last, and everything is packed into one tarball.
#
# Runs as an ordinary user when kernel.perf_event_paranoid <= 2 (counts the user-space
# part of our own process only); it stops with the one sudo command needed otherwise.
#
# Usage: scripts/topdown_kit.sh
#   TOPDOWN_SECONDS  counted seconds per run (default 5)
#   TOPDOWN_QUICK=1  three rows instead of eight (about half the time)
#   TOPDOWN_CPU      CPU to pin to (default: the last online core, a P-core on hybrids)
set -uo pipefail

seconds="${TOPDOWN_SECONDS:-5}"
quick="${TOPDOWN_QUICK:-0}"

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
name="topdown-$(hostname -s)-$stamp"
base="$root/target/topdown"
out="$base/$name"
mkdir -p "$out/host" "$out/runs" "$out/record"
log() { echo "[kit $(date -u +%H:%M:%S)] $*" | tee -a "$out/kit.log" >&2; }
die() { log "ERROR: $*"; exit 1; }

# --- prerequisites -------------------------------------------------------------------
command -v perf >/dev/null || die "perf not found. Install the perf matching your kernel \
(Debian/Ubuntu: linux-tools-\$(uname -r) or linux-perf; Fedora/RHEL: perf), then re-run."
command -v cargo >/dev/null || die "cargo not found; install Rust from https://rustup.rs"
command -v python3 >/dev/null || log "python3 not found: the summary table will be skipped (raw data is still packed)"
perf stat -e instructions -x, -o /dev/null -- true >/dev/null 2>&1 ||
    log "WARNING: a trivial 'perf stat' failed; see the paranoid check below"

paranoid="$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null || echo unknown)"
log "kernel.perf_event_paranoid = $paranoid"
if [ "$paranoid" != unknown ] && [ "$paranoid" -gt 2 ] && [ "$(id -u)" != 0 ]; then
    die "perf_event_paranoid=$paranoid blocks unprivileged counting. Run once:
    sudo sysctl kernel.perf_event_paranoid=2
(resets at reboot), then re-run this script as yourself."
fi
nmi="$(cat /proc/sys/kernel/nmi_watchdog 2>/dev/null || echo unknown)"
[ "$nmi" = 1 ] && log "note: the NMI watchdog holds one counter, so large passes multiplex more. \
Optional: sudo sysctl kernel.nmi_watchdog=0 (restore with =1 afterwards)."
grep -q hypervisor /proc/cpuinfo && log "WARNING: this looks like a VM ('hypervisor' flag); counters may be missing or virtualised"

# --- host record ---------------------------------------------------------------------
{
    uname -a
    echo "perf: $(perf version 2>&1)"
    echo "paranoid: $paranoid  nmi_watchdog: $nmi"
    echo "rustc: $(rustc --version)"
    echo "git: $(git rev-parse HEAD 2>/dev/null) $(git status --porcelain 2>/dev/null | wc -l) dirty paths"
} >"$out/host/info.txt"
lscpu >"$out/host/lscpu.txt" 2>&1
grep -m1 -A26 '^processor' /proc/cpuinfo >"$out/host/cpuinfo.txt" 2>&1
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null | sort | uniq -c >"$out/host/governor.txt"
cat /sys/devices/system/cpu/intel_pstate/no_turbo >"$out/host/no_turbo.txt" 2>/dev/null
perf list >"$out/host/perf-list.txt" 2>&1

# --- build ---------------------------------------------------------------------------
log "building eval_loop (release, target-cpu=native); takes a few minutes the first time"
RUSTFLAGS="-C target-cpu=native" cargo build --release --example eval_loop \
    --features eval-schedule-study >"$out/build.log" 2>&1 || die "build failed; see $out/build.log"
bin="$root/target/release/examples/eval_loop"
cp "$bin" "$out/record/eval_loop"

# --- CPU to pin to -------------------------------------------------------------------
last_cpu() { tr ',' '\n' <"$1" | tail -1 | sed 's/.*-//'; }
cpu="${TOPDOWN_CPU:-}"
if [ -z "$cpu" ]; then
    if [ -r /sys/devices/cpu_core/cpus ]; then
        cpu="$(last_cpu /sys/devices/cpu_core/cpus)"
        log "hybrid CPU: pinning to P-core $cpu"
    else
        cpu="$(last_cpu /sys/devices/system/cpu/online)"
    fi
fi
pin=()
command -v taskset >/dev/null && pin=(taskset -c "$cpu") || log "taskset not found: runs are not pinned"
log "pinning runs to CPU $cpu"

# --- perf capabilities ---------------------------------------------------------------
probe_out="$(mktemp)"
user=()
perf stat --all-user -e instructions -x, -o /dev/null -- true >/dev/null 2>&1 && user=(--all-user)
log "user-space-only counting flag: ${user[*]:-(none; perf falls back by itself)}"

control=0
fifo_dir="$(mktemp -d)"
mkfifo "$fifo_dir/ctl" "$fifo_dir/ack"
if perf stat ${user[@]+"${user[@]}"} -D -1 --control "fifo:$fifo_dir/ctl,$fifo_dir/ack" -e instructions \
    -x, -o /dev/null -- true >/dev/null 2>&1; then
    control=1
fi
log "perf --control fifo support: $control (without it, runs skip setup with a fixed -D delay)"

# An event or group is usable if perf opens it and counts something on a busy loop.
usable() {
    perf stat ${user[@]+"${user[@]}"} -x, -o "$probe_out" "$@" -- python3 -c 'sum(range(10**6))' >/dev/null 2>&1 ||
        perf stat ${user[@]+"${user[@]}"} -x, -o "$probe_out" "$@" -- sh -c 'i=0; while [ $i -lt 20000 ]; do i=$((i+1)); done' >/dev/null 2>&1 ||
        return 1
    ! grep -q -e '<not supported>' -e '<not counted>' "$probe_out"
}

pass_names=()
declare -A pass_args
add_events_pass() {
    local pass="$1"; shift
    local kept=()
    for ev in "$@"; do
        if usable -e "$ev"; then kept+=("$ev"); else echo "$pass: $ev" >>"$out/skipped.txt"; fi
    done
    if [ ${#kept[@]} -gt 0 ]; then
        pass_names+=("$pass")
        pass_args[$pass]="-e $(IFS=,; echo "${kept[*]}")"
        echo "$pass: ${kept[*]}" >>"$out/passes.txt"
    fi
}
add_first_usable() {
    local pass="$1" flag="$2"; shift 2
    for spec in "$@"; do
        if usable "$flag" "$spec"; then
            pass_names+=("$pass")
            pass_args[$pass]="$flag $spec"
            echo "$pass: $flag $spec" >>"$out/passes.txt"
            return
        fi
        echo "$pass: $flag $spec" >>"$out/skipped.txt"
    done
}

: >"$out/passes.txt"; : >"$out/skipped.txt"
log "probing counters"
add_events_pass basic cycles instructions branches branch-misses ref-cycles
add_first_usable td_raw -e \
    '{slots,topdown-retiring,topdown-bad-spec,topdown-fe-bound,topdown-be-bound,topdown-heavy-ops,topdown-br-mispredict,topdown-fetch-lat,topdown-mem-bound}' \
    '{slots,topdown-retiring,topdown-bad-spec,topdown-fe-bound,topdown-be-bound}'
add_first_usable td_metric -M TopdownL1,TopdownL2 TopdownL1 PipelineL1,PipelineL2 PipelineL1
add_events_pass fp cycles instructions \
    fp_arith_inst_retired.scalar_double fp_arith_inst_retired.128b_packed_double \
    fp_arith_inst_retired.256b_packed_double fp_arith_inst_retired.512b_packed_double \
    fp_ret_sse_avx_ops.all
add_events_pass mem cycles instructions \
    mem_inst_retired.all_loads mem_inst_retired.all_stores \
    mem_load_retired.l1_hit mem_load_retired.l1_miss mem_load_retired.l2_hit \
    mem_load_retired.l2_miss mem_load_retired.l3_hit mem_load_retired.fb_hit
add_events_pass stalls cycles instructions \
    cycle_activity.stalls_total cycle_activity.stalls_l1d_miss cycle_activity.stalls_l2_miss \
    exe_activity.bound_on_loads exe_activity.bound_on_stores exe_activity.1_ports_util \
    exe_activity.2_ports_util exe_activity.2_3_ports_util resource_stalls.sb \
    resource_stalls.scoreboard
add_events_pass ports cycles instructions \
    uops_dispatched.port_0 uops_dispatched.port_1 uops_dispatched.port_5_11 \
    uops_dispatched.port_5 uops_dispatched.port_6 uops_dispatched.port_2_3_10 \
    uops_dispatched.port_2_3 uops_dispatched.port_4_9 uops_dispatched.port_7_8 \
    uops_issued.any uops_retired.slots
add_events_pass frontend cycles instructions \
    idq.dsb_uops idq.mite_uops idq.ms_uops icache_data.stalls icache_64b.iftag_stall \
    dsb2mite_switches.penalty_cycles int_misc.clear_resteer_cycles baclears.any
add_events_pass branch cycles instructions \
    br_inst_retired.all_branches br_misp_retired.all_branches br_misp_retired.indirect \
    br_inst_retired.indirect int_misc.recovery_cycles machine_clears.count
log "passes: ${pass_names[*]} (events per pass in passes.txt, dropped ones in skipped.txt)"
[ ${#pass_names[@]} -gt 0 ] || die "no usable counters; is this a VM without a PMU?"

# --- cells ---------------------------------------------------------------------------
rows=(ee_to_mumu ee_to_wpwm uux_to_uux gg_to_gg gg_to_ttx ee_to_mumua ee_to_mumu_tata_qcd0 uux_to_ccx_emmm_qcd0)
[ "$quick" = 1 ] && rows=(gg_to_gg ee_to_mumu_tata_qcd0 uux_to_ccx_emmm_qcd0)
cells=()
for r in "${rows[@]}"; do for w in 1 4 8; do cells+=("$r:$w:production"); done; done
# Execution-order controls: interning order serialises dependent instructions; a shuffle
# within levels makes the 2->6 dispatch stream unpredictable (a mispredict calibration).
for r in gg_to_gg ee_to_mumu_tata_qcd0 uux_to_ccx_emmm_qcd0; do
    for w in 1 4; do cells+=("$r:$w:arena"); done
done
cells+=("uux_to_ccx_emmm_qcd0:1:levelshuffle")

runs=$((${#cells[@]} * ${#pass_names[@]}))
log "${#cells[@]} cells x ${#pass_names[@]} passes = $runs runs of ${seconds}s counted \
(about $(awk -v r="$runs" -v s="$seconds" 'BEGIN { printf "%d", r * (s + 3) / 60 }') min)"

# --- runs ----------------------------------------------------------------------------
# One perf invocation: counting starts disabled and the driver brackets its timed loop.
run_perf() {
    local tool="$1" outfile="$2" jsonfile="$3" sched="$4" row="$5" width="$6"; shift 6
    local sched_env=()
    [ "$sched" != production ] && sched_env=(VIBEGRAPH_EVAL_SCHEDULE="$sched")
    if [ "$control" = 1 ]; then
        perf "$tool" ${user[@]+"${user[@]}"} -D -1 --control "fifo:$fifo_dir/ctl,$fifo_dir/ack" -o "$outfile" "$@" -- \
            env TOPDOWN_CTL_FIFO="$fifo_dir/ctl" TOPDOWN_ACK_FIFO="$fifo_dir/ack" ${sched_env[@]+"${sched_env[@]}"} \
            ${pin[@]+"${pin[@]}"} "$bin" --row "$row" --width "$width" --seconds "$seconds" >"$jsonfile"
    else
        perf "$tool" ${user[@]+"${user[@]}"} -D 4000 -o "$outfile" "$@" -- \
            env ${sched_env[@]+"${sched_env[@]}"} ${pin[@]+"${pin[@]}"} "$bin" --row "$row" --width "$width" \
            --seconds "$seconds" --start-at-ms 4200 >"$jsonfile"
    fi
}

n=0
for cell in "${cells[@]}"; do
    IFS=: read -r row width sched <<<"$cell"
    dir="$out/runs/$row-w$width-$sched"
    mkdir -p "$dir"
    for pass in "${pass_names[@]}"; do
        n=$((n + 1))
        log "[$n/$runs] $row width=$width order=$sched pass=$pass"
        # shellcheck disable=SC2086 # pass_args holds perf's own option words
        run_perf stat "$dir/$pass.csv" "$dir/$pass.json" "$sched" "$row" "$width" -x, ${pass_args[$pass]} \
            2>>"$dir/stderr.txt" || log "  run failed (see $dir/stderr.txt); continuing"
    done
done

# --- sampling profile of the 2->6 ----------------------------------------------------
log "perf record: uux_to_ccx_emmm_qcd0, scalar and lanes4"
for width in 1 4; do
    run_perf record "$out/record/perf-2to6-w$width.data" "$out/record/perf-2to6-w$width.json" \
        production uux_to_ccx_emmm_qcd0 "$width" -F 1999 -e cycles 2>>"$out/record/stderr.txt" ||
        log "  perf record failed (see record/stderr.txt); continuing"
    perf report -i "$out/record/perf-2to6-w$width.data" --stdio --no-children --sort sym \
        2>/dev/null | head -80 >"$out/record/report-2to6-w$width.txt"
done

# --- summary and tarball -------------------------------------------------------------
if command -v python3 >/dev/null; then
    python3 "$root/scripts/topdown_summary.py" "$out" >"$out/summary.md" 2>>"$out/kit.log" ||
        log "summary failed; raw data is still packed"
fi
rm -rf "$fifo_dir" "$probe_out"
tar -czf "$base/$name.tar.gz" -C "$base" "$name"
log "done: $base/$name.tar.gz ($(du -h "$base/$name.tar.gz" | cut -f1))"
echo "$base/$name.tar.gz"
