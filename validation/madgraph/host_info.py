"""The machine-identity block every timing artifact this directory writes embeds.

Same shape as the validation report's own `host.json`
(`vibegraph-lib/tests/common/report.rs::host_record`): CPU and its core
classes, memory, OS, and the toolchain that produced the measurement. A
duration without this is not a measurement of anything — it cannot be told
apart from a run on different hardware.
"""

import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def sysctl(key):
    try:
        v = subprocess.run(["sysctl", "-n", key], capture_output=True, text=True)
    except FileNotFoundError:
        return None
    v = v.stdout.strip()
    return v or None


def first_line(argv):
    try:
        r = subprocess.run(argv, capture_output=True, text=True)
    except FileNotFoundError:
        return None
    return r.stdout.strip().splitlines()[0] if r.stdout.strip() else None


def as_int(v):
    try:
        return int(v)
    except (TypeError, ValueError):
        return None


def host_block():
    return {
        "captured": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "cpu": {
            "arch": os.uname().machine,
            "model": sysctl("machdep.cpu.brand_string"),
            "logical_cpus": as_int(sysctl("hw.logicalcpu")),
            "physical_cpus": as_int(sysctl("hw.physicalcpu")),
            "performance_logical_cpus": as_int(sysctl("hw.perflevel0.logicalcpu")),
            "efficiency_logical_cpus": as_int(sysctl("hw.perflevel1.logicalcpu")),
            "frequency_hz": as_int(sysctl("hw.cpufrequency_max")),
            "frequency_note": "null where the OS exposes no clock (Apple Silicon: "
            "`hw.cpufrequency`/`hw.cpufrequency_max` are empty)",
            "memory_bytes": as_int(sysctl("hw.memsize")),
        },
        "scheduling": {
            "affinity": "none — MadGraph is run as it is in production, with no core "
            "pinning; madevent forks its own subprocess jobs and the OS places them",
            "cpu_count_seen_by_python": os.cpu_count(),
        },
        "os": {
            "family": sys.platform,
            "kernel": f"{os.uname().sysname} {os.uname().release}",
            "release": first_line(["sw_vers", "-productVersion"]),
        },
        "toolchain": {
            "python": sys.version.split()[0],
            "gfortran": first_line(["gfortran", "--version"]),
            "gcc": first_line(["gcc", "--version"]),
            "madgraph": (ROOT / "research/refs/mg5amcnlo/VERSION").read_text().strip()
            if (ROOT / "research/refs/mg5amcnlo/VERSION").is_file()
            else None,
        },
    }
