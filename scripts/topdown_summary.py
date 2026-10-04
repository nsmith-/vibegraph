#!/usr/bin/env python3
"""Summarise a `topdown_kit.sh` result directory as markdown tables.

Every run directory `runs/<row>-w<width>-<order>/` holds one `perf stat -x,` CSV per
counter pass and the driver's JSON line for that same run (events evaluated, wall
time). Counts are normalised per event with that run's own event count, so passes of
different length combine. Per-VM-instruction figures use the instruction counts of
`roofline_census` (`helas/eval/roofline.rs`), which do not depend on the host; at
lane width N one VM instruction serves N events.

Usage: topdown_summary.py <result-dir>   (markdown on stdout, summary.json beside it)
"""

import csv
import json
import pathlib
import re
import sys

# roofline_census, target-cpu=native: (VM instructions per pass, flops per event with an
# FMA as two, FP operations per event with an FMA as one).
CENSUS = {
    "ee_to_mumu": (61, 1629, 1401),
    "ee_to_wpwm": (425, 9372, 8174),
    "uux_to_uux": (123, 2662, 2262),
    "gg_to_gg": (682, 10186, 8930),
    "gg_to_ttx": (328, 6426, 5242),
    "ee_to_mumua": (346, 10915, 8907),
    "ee_to_mumu_tata_qcd0": (1739, 51587, 41399),
    "uux_to_ccx_emmm_qcd0": (36506, 1060891, 844087),
}

ROW_ORDER = list(CENSUS)
CELL_RE = re.compile(r"^(?P<row>.+)-w(?P<width>\d+)-(?P<order>[a-z0-9]+)$")


def norm(event):
    """`cpu_core/fp_arith_inst_retired.scalar_double/u` -> `fp_arith_inst_retired.scalar_double`."""
    e = event.strip()
    e = re.sub(r"^(cpu_core|cpu_atom|cpu)/", "", e)
    e = re.sub(r"/[a-zA-Z]*$", "", e)
    e = re.sub(r":[a-zA-Z]+$", "", e)
    return e.lower()


def parse_csv(path):
    """Events (summed over PMUs), perf's own metrics, and multiplexing notes."""
    events, metrics, notes = {}, {}, []
    for line in path.read_text(errors="replace").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        f = next(csv.reader([line]))
        if len(f) >= 3 and f[2]:
            name = norm(f[2])
            try:
                events[name] = events.get(name, 0.0) + float(f[0])
            except ValueError:
                notes.append(f"{name}: {f[0]}")
            try:
                if float(f[4]) < 99.5:
                    notes.append(f"{name} counted {float(f[4]):.0f}% of the time (scaled)")
            except (IndexError, ValueError):
                pass
        if len(f) >= 7 and f[5] and f[6]:
            try:
                metrics[f[6].split()[-1]] = float(f[5])
            except ValueError:
                pass
    return events, metrics, notes


def load(result):
    cells = {}
    for d in sorted((result / "runs").iterdir()):
        m = CELL_RE.match(d.name)
        if not m:
            continue
        cell = cells.setdefault(
            (m["row"], int(m["width"]), m["order"]),
            {"per_event": {}, "metrics": {}, "notes": [], "ns_per_event": {}},
        )
        for csv_path in sorted(d.glob("*.csv")):
            pass_name = csv_path.stem
            json_path = d / f"{pass_name}.json"
            try:
                run = json.loads(json_path.read_text().strip().splitlines()[-1])
            except (OSError, IndexError, ValueError):
                cell["notes"].append(f"{pass_name}: no driver output")
                continue
            events, metrics, notes = parse_csv(csv_path)
            n = run["events"]
            for k, v in events.items():
                cell["per_event"].setdefault(k, v / n)
            cell["metrics"].update({f"{pass_name}:{k}": v for k, v in metrics.items()})
            cell["notes"] += [f"{pass_name}: {x}" for x in notes]
            cell["ns_per_event"][pass_name] = run["ns_per_event"]
    return cells


def fmt(v, digits=2):
    if v is None:
        return "—"
    if abs(v) >= 1000:
        return f"{v:,.0f}"
    return f"{v:.{digits}f}"


def get(cell, *names):
    for n in names:
        if n in cell["per_event"]:
            return cell["per_event"][n]
    return None


def ratio(a, b):
    return None if a is None or b in (None, 0) else a / b


def table(title, header, rows):
    out = [f"\n## {title}\n", "| " + " | ".join(header) + " |",
           "|" + "|".join("---" if i == 0 else "--:" for i in range(len(header))) + "|"]
    out += ["| " + " | ".join(r) + " |" for r in rows]
    return "\n".join(out)


def main():
    result = pathlib.Path(sys.argv[1])
    cells = load(result)
    keys = sorted(
        cells,
        key=lambda k: (k[2] != "production", ROW_ORDER.index(k[0]) if k[0] in ROW_ORDER else 99, k[1]),
    )
    label = lambda k: f"`{k[0]}` w{k[1]}" + ("" if k[2] == "production" else f" {k[2]}")
    vm_per_event = lambda k: CENSUS[k[0]][0] / k[1] if k[0] in CENSUS else None
    out = [f"# Top-down summary: {result.name}\n",
           "Per-event figures use each run's own event count. A VM instruction serves "
           "`width` events. Fractions are of cycles or of pipeline slots as labelled."]

    rows = []
    for k in keys:
        c = cells[k]
        cyc, ins = get(c, "cycles"), get(c, "instructions")
        ns = c["ns_per_event"].get("basic") or next(iter(c["ns_per_event"].values()), None)
        vm = vm_per_event(k)
        rows.append([label(k), fmt(ns, 1), fmt(ratio(cyc, ns), 2), fmt(ratio(ins, cyc)),
                     fmt(ratio(cyc, vm), 1), fmt(ratio(ins, vm), 1),
                     fmt(ratio(get(c, "branch-misses"), vm), 3)])
    out.append(table("Rates", ["cell", "ns/event", "GHz", "IPC", "cycles/VM instr",
                               "instrs/VM instr", "branch misses/VM instr"], rows))

    rows = []
    for k in keys:
        c = cells[k]
        slots = get(c, "slots")
        f = lambda n: ratio(get(c, n), slots)
        ret, bad, fe, be = f("topdown-retiring"), f("topdown-bad-spec"), f("topdown-fe-bound"), f("topdown-be-bound")
        heavy, misp, flat, mem = f("topdown-heavy-ops"), f("topdown-br-mispredict"), f("topdown-fetch-lat"), f("topdown-mem-bound")
        pct = lambda v: fmt(None if v is None else 100 * v, 1)
        sub = lambda a, b: None if a is None or b is None else a - b
        rows.append([label(k), pct(ret), pct(bad), pct(fe), pct(be), pct(heavy), pct(misp),
                     pct(flat), pct(sub(fe, flat)), pct(mem), pct(sub(be, mem))])
    if any(any(x != "—" for x in r[1:]) for r in rows):
        out.append(table("Top-down from slots (% of slots)",
                         ["cell", "retiring", "bad spec", "frontend", "backend", "heavy ops",
                          "br mispredict", "fetch latency", "fetch bandwidth", "memory bound",
                          "core bound"], rows))
    metric_names = sorted({m for c in cells.values() for m in c["metrics"]})
    if metric_names:
        rows = [[label(k)] + [fmt(cells[k]["metrics"].get(m), 1) for m in metric_names] for k in keys]
        out.append(table("perf's top-down metrics (as perf reports them, usually %)",
                         ["cell"] + [m.split(":", 1)[1] for m in metric_names], rows))

    rows = []
    for k in keys:
        c = cells[k]
        s, x, y, z = (get(c, f"fp_arith_inst_retired.{w}") for w in
                      ("scalar_double", "128b_packed_double", "256b_packed_double", "512b_packed_double"))
        parts = [v for v in (s, x, y, z) if v is not None]
        flops = None if not parts else (s or 0) + 2 * (x or 0) + 4 * (y or 0) + 8 * (z or 0)
        census = CENSUS.get(k[0], (None, None, None))[1]
        share = lambda v, w: fmt(None if v is None or not flops else 100 * w * v / flops, 0)
        rows.append([label(k), fmt(flops, 0), fmt(census, 0), fmt(ratio(flops, census), 3),
                     share(s, 1), share(x, 2), share(y, 4), share(z, 8),
                     fmt(ratio(get(c, "fp_ret_sse_avx_ops.all"), 1), 0)])
    out.append(table("FP work (fp_arith_inst_retired counts an FMA twice, as the census does)",
                     ["cell", "measured flops/event", "census flops/event", "measured/census",
                      "% scalar", "% 128b", "% 256b", "% 512b", "AMD fp ops/event"], rows))

    rows = []
    for k in keys:
        c = cells[k]
        cyc = get(c, "cycles")
        loads, l1m = get(c, "mem_inst_retired.all_loads"), get(c, "mem_load_retired.l1_miss")
        rows.append([label(k), fmt(loads, 0), fmt(get(c, "mem_inst_retired.all_stores"), 0),
                     fmt(ratio(loads, vm_per_event(k)), 1),
                     fmt(None if ratio(l1m, loads) is None else 100 * l1m / loads, 2),
                     fmt(get(c, "mem_load_retired.l2_miss"), 1),
                     *(fmt(None if ratio(get(c, e), cyc) is None else 100 * get(c, e) / cyc, 1) for e in (
                         "cycle_activity.stalls_total", "cycle_activity.stalls_l1d_miss",
                         "cycle_activity.stalls_l2_miss", "exe_activity.bound_on_loads",
                         "exe_activity.bound_on_stores", "resource_stalls.sb"))])
    out.append(table("Memory and stalls (stall columns are % of cycles)",
                     ["cell", "loads/event", "stores/event", "loads/VM instr", "L1 miss %",
                      "L2 misses/event", "stalls total", "stalls L1d miss", "stalls L2 miss",
                      "bound on loads", "bound on stores", "store buffer full"], rows))

    ports = sorted({e for c in cells.values() for e in c["per_event"] if e.startswith("uops_dispatched.port")})
    if ports:
        rows = [[label(k)] + [fmt(ratio(get(cells[k], p), get(cells[k], "cycles"))) for p in ports]
                + [fmt(ratio(get(cells[k], "uops_issued.any"), get(cells[k], "cycles")))] for k in keys]
        out.append(table("Port use (uops dispatched per cycle)",
                         ["cell"] + [p.split(".")[-1] for p in ports] + ["issued"], rows))

    rows = []
    for k in keys:
        c = cells[k]
        cyc = get(c, "cycles")
        dsb, mite, ms = get(c, "idq.dsb_uops"), get(c, "idq.mite_uops"), get(c, "idq.ms_uops")
        tot = None if None in (dsb, mite) else dsb + mite + (ms or 0)
        pc = lambda e: fmt(None if ratio(get(c, e), cyc) is None else 100 * get(c, e) / cyc, 1)
        rows.append([label(k), fmt(None if not tot else 100 * dsb / tot, 1),
                     pc("icache_data.stalls"), pc("icache_64b.iftag_stall"),
                     pc("dsb2mite_switches.penalty_cycles"), pc("int_misc.clear_resteer_cycles"),
                     pc("int_misc.recovery_cycles"),
                     fmt(ratio(get(c, "br_misp_retired.indirect"), vm_per_event(k)), 3),
                     fmt(ratio(get(c, "br_misp_retired.all_branches"), vm_per_event(k)), 3)])
    out.append(table("Front end and speculation (% of cycles unless per VM instr)",
                     ["cell", "uops from DSB %", "icache data stalls", "icache tag stalls",
                      "DSB→MITE switch", "resteer", "recovery", "indirect misp/VM instr",
                      "all misp/VM instr"], rows))

    notes = [f"- {label(k)}: {n}" for k in keys for n in cells[k]["notes"]]
    if notes:
        out.append("\n## Notes\n\n" + "\n".join(notes[:200]))
    print("\n".join(out))
    (result / "summary.json").write_text(json.dumps(
        {f"{k[0]}|{k[1]}|{k[2]}": cells[k] for k in keys}, indent=1, sort_keys=True))


if __name__ == "__main__":
    main()
