"""Read every emitted Les Houches event back through Pythia 8.

The rest of the suite compares vibegraph against MadGraph. This gate asks a
different question, and it is the only one that asks it: can the *consumer* at
the far end of the pipeline read what we wrote? Pythia parses the file, rebuilds
the hard process from the `<init>` and `<event>` records, and — before it
showers anything — insists that the colour indices connect. Those colour lines
have been checked against `leshouche.inc` as data; here they are used as input.

Metric: **n/n events consumed**, per emitted sample.

What is measured, and how the three failure modes are kept apart:

* **LHEF/process level** — the gate. `PartonLevel` and `HadronLevel` are off, so
  a refused event is a refused *file*: an unreadable record, an unmatched colour
  index, a particle Pythia cannot put on its mass shell. Each sample must yield
  exactly as many successful `Pythia::next()` calls as the file has `<event>`
  blocks, with the reconstructed final state matching the one we wrote, and the
  next call after them must fail at end of file — so a truncated read scores
  short rather than passing.
* **Shower level** — informational. The same file run through the full chain,
  reported but never gating: a shower that gets stuck in a loop is Pythia's
  physics, not our format.
* **Vacuity** — the negative control. One event of the coloured sample is
  rewritten with a dangling colour index and fed back in; Pythia must reject
  exactly that event and accept its neighbours. Without it, "n/n consumed" would
  be consistent with Pythia ignoring colour entirely, which is precisely the
  property this gate exists to exercise.

Each (sample, pass) runs in a child process, so Pythia's own messages are
captured whole from its exit-flushed stdout and a hard abort inside Pythia is
recorded rather than taking the driver down with it.

Usage:
    pixi run -e pythia validate-pythia
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path

import pylhe

ROOT = Path(__file__).resolve().parents[2]
SAMPLE_DIR = ROOT / "target" / "pythia-samples"
ROW_PATH = ROOT / "target" / "validation-report" / "standalone" / "pythia_consumption.json"
ROW_ID = "pythia-consumption"

#: Colour index the negative control writes over one final-state parton's tag,
#: chosen above every index our writer emits so it can match nothing.
DANGLING_COLOUR = 599

#: Log lines kept per pass. Pythia's process-level messages are a handful of
#: lines each; the cap only bites when a sample fails wholesale.
MAX_LOG_LINES = 200

#: A Pythia message, wherever it sits on the line — on its own where it was
#: raised, framed by table rules in the end-of-run summary.
MESSAGE_RE = re.compile(r"(?:Abort|Error|Warning)\s+(?:from|in)\s+\S.*")
END_OF_FILE = "reached end of Les Houches Events File"


@dataclass
class Pass:
    """One Pythia run over one file."""

    version: str = ""
    init: bool = False
    n_consumed: int = 0
    failed: list[int] = field(default_factory=list)
    mismatched: list[int] = field(default_factory=list)
    eof_after: bool = False
    crashed: bool = False
    log: list[str] = field(default_factory=list)


# ─────────────────────────────── the child ────────────────────────────────


def outgoing_of(lhe: Path) -> list[list[int]]:
    """The outgoing PDG codes of every event in the file, in file order.

    pylhe's own record classes do the field conversion, so what this compares
    against is what a pylhe user reads out of our file.
    """
    out: list[list[int]] = []
    with lhe.open("rb") as fileobj:
        for _, element in ET.iterparse(fileobj, events=["end"]):
            if element.tag != "event":
                continue
            lines = (element.text or "").strip().split("\n")
            particles = [
                pylhe.LHEParticle.fromstring(line)
                for line in lines[1:]
                if not line.strip().startswith("#")
            ]
            out.append(sorted(int(p.id) for p in particles if int(p.status) == 1))
            element.clear()
    return out


def run_pythia(lhe: Path, mode: str, seed: int, expect: list[list[int]]) -> dict:
    """Read `lhe` end to end and report what came back.

    `mode` is `"process"` for the gating pass — hard process only, so a refusal
    is about the file — or `"shower"` for the informational full chain.
    """
    import pythia8

    pythia = pythia8.Pythia("", False)
    pythia.readString("Beams:frameType = 4")
    pythia.readString(f"Beams:LHEF = {lhe}")
    # Progress counters and event listings off, messages left on: the whole point
    # of capturing this child's output is to keep Pythia's verdict on our file.
    pythia.readString("Next:numberCount = 0")
    pythia.readString("Next:numberShowLHA = 0")
    pythia.readString("Next:numberShowInfo = 0")
    pythia.readString("Next:numberShowProcess = 0")
    pythia.readString("Next:numberShowEvent = 0")
    pythia.readString("Init:showProcesses = off")
    pythia.readString("Init:showMultipartonInteractions = off")
    pythia.readString("Init:showChangedSettings = off")
    pythia.readString("Init:showChangedParticleData = off")
    pythia.readString("Random:setSeed = on")
    pythia.readString(f"Random:seed = {seed}")
    if mode == "process":
        pythia.readString("PartonLevel:all = off")
        pythia.readString("HadronLevel:all = off")

    result = {
        "version": f"{pythia.parm('Pythia:versionNumber'):.3f}",
        "init": bool(pythia.init()),
        "n_consumed": 0,
        "failed": [],
        "mismatched": [],
    }
    if not result["init"]:
        return result

    for index in range(len(expect)):
        if not pythia.next():
            result["failed"].append(index)
            continue
        result["n_consumed"] += 1
        # `process` holds the hard process Pythia rebuilt: entries 0-2 are the
        # system and the two beams, the incoming partons carry status -21, and
        # status 23 is an outgoing leg of the hard scattering.
        got = sorted(
            pythia.process[i].id()
            for i in range(pythia.process.size())
            if pythia.process[i].status() == 23
        )
        if got != expect[index]:
            result["mismatched"].append(index)

    # One more read: the file must be exhausted, so a short read cannot pass by
    # having been counted against a short expectation.
    result["eof_after"] = not pythia.next()
    pythia.stat()
    return result


def child(spec_path: Path, out_path: Path) -> int:
    spec = json.loads(spec_path.read_text())
    lhe = Path(spec["lhe"])
    result = run_pythia(lhe, spec["mode"], spec["seed"], outgoing_of(lhe))
    out_path.write_text(json.dumps(result))
    return 0


# ─────────────────────────────── the driver ───────────────────────────────


def run_pass(lhe: Path, mode: str, seed: int, work: Path) -> Pass:
    spec = work / f"{lhe.stem}.{mode}.spec.json"
    out = work / f"{lhe.stem}.{mode}.result.json"
    spec.write_text(json.dumps({"lhe": str(lhe), "mode": mode, "seed": seed}))
    out.unlink(missing_ok=True)

    proc = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--child", str(spec), str(out)],
        capture_output=True,
        text=True,
        check=False,
    )
    log = [line for line in (proc.stdout + proc.stderr).splitlines() if line.strip()]
    if not out.is_file():
        return Pass(crashed=True, log=log[-MAX_LOG_LINES:])
    raw = json.loads(out.read_text())
    return Pass(
        version=raw.get("version", ""),
        init=raw["init"],
        n_consumed=raw["n_consumed"],
        failed=raw["failed"],
        mismatched=raw["mismatched"],
        eof_after=raw.get("eof_after", False),
        log=log[-MAX_LOG_LINES:],
    )


def complaints(log: list[str]) -> list[str]:
    """The message lines out of a Pythia log, deduplicated in first-seen order.

    Pythia prefixes every message with `Abort from` / `Error in` / `Warning in`,
    both where it happens and again in its end-of-run summary table, so matching
    the prefix anywhere in the line keeps the verdict and drops the banners and
    the table's own framing.

    The end-of-file abort is dropped: it is how a complete read *ends*, and the
    row records that separately as `read_to_end_of_file`.
    """
    seen: dict[str, None] = {}
    for line in log:
        match = MESSAGE_RE.search(line)
        if not match:
            continue
        message = match.group(0).rstrip(" |")
        if message.endswith(END_OF_FILE):
            continue
        seen.setdefault(message, None)
    return list(seen)


def mutate_colour(src: Path, dst: Path) -> dict:
    """Copy `src` with one final-state colour tag rewritten to a dangling index.

    The first event carrying a coloured final-state parton is used, and only its
    `ICOLUP(1)` moves, so the mutation is a single unmatched index rather than a
    malformed record — the failure mode a colour-blind reader would miss.
    """
    text = src.read_text()
    position, index = 0, 0
    while True:
        start = text.find("<event>", position)
        if start < 0:
            break
        end = text.index("</event>", start)
        # `<event>` on the first line, the event info on the second, one particle
        # per line after that.
        lines = text[start:end].split("\n")
        for row, line in enumerate(lines[2:], start=2):
            fields = line.split()
            if len(fields) < 13 or fields[1] != "1" or int(fields[4]) == 0:
                continue
            tokens = re.split(r"(\s+)", line)
            slots = [i for i, token in enumerate(tokens) if token.strip()]
            tokens[slots[4]] = str(DANGLING_COLOUR)
            lines[row] = "".join(tokens)
            dst.write_text(text[:start] + "\n".join(lines) + text[end:])
            return {"event_index": index, "particle": row - 2, "colour": int(fields[4])}
        position, index = end, index + 1
    msg = f"{src}: no coloured final-state parton to mutate"
    raise ValueError(msg)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--child", nargs=2, metavar=("SPEC", "OUT"), help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.child:
        return child(Path(args.child[0]), Path(args.child[1]))

    manifest = json.loads((SAMPLE_DIR / "samples.json").read_text())
    seed = manifest["seed"]
    work = SAMPLE_DIR / "pythia"
    work.mkdir(parents=True, exist_ok=True)

    rows = []
    ok = True
    version = ""
    for entry in manifest["samples"]:
        lhe = SAMPLE_DIR / entry["lhe"]
        n_total = len(outgoing_of(lhe))
        gate = run_pass(lhe, "process", seed, work)
        shower = run_pass(lhe, "shower", seed, work)
        version = version or gate.version

        passed = (
            gate.init
            and not gate.crashed
            and gate.n_consumed == n_total
            and not gate.failed
            and not gate.mismatched
            and gate.eof_after
        )
        ok = ok and passed
        rows.append(
            {
                "key": entry["key"],
                "process": entry["process"],
                "lhe": entry["lhe"],
                "run_card": entry["run_card"],
                "seed": seed,
                "n_total": n_total,
                "n_consumed": gate.n_consumed,
                "failed_events": gate.failed,
                "mismatched_events": gate.mismatched,
                "read_to_end_of_file": gate.eof_after,
                "complaints": complaints(gate.log),
                "shower": {
                    "n_consumed": shower.n_consumed,
                    "failed_events": shower.failed,
                    "complaints": complaints(shower.log),
                },
                "status": "pass" if passed else "fail",
            }
        )
        mark = "n/n" if passed else "MISS"
        print(
            f"[{entry['key']}] {gate.n_consumed}/{n_total} consumed ({mark}), "
            f"shower {shower.n_consumed}/{n_total}"
        )
        for message in complaints(gate.log):
            print(f"    process-level: {message}")
        for message in complaints(shower.log):
            print(f"    shower-level:  {message}")

    coloured = SAMPLE_DIR / manifest["samples"][0]["lhe"]
    broken = work / "negative_control.lhe"
    mutation = mutate_colour(coloured, broken)
    control = run_pass(broken, "process", seed, work)
    n_broken = len(outgoing_of(broken))
    control_ok = control.init and control.failed == [mutation["event_index"]]
    ok = ok and control_ok
    print(
        f"[negative control] dangling colour {DANGLING_COLOUR} on event "
        f"{mutation['event_index']} of {coloured.name}: "
        f"rejected={control.failed}, {control.n_consumed}/{n_broken} others consumed"
    )
    for message in complaints(control.log):
        print(f"    {message}")

    row = {
        "row": ROW_ID,
        "layer": "banked",
        "metric": "events consumed",
        "task": "validate-pythia",
        "pythia_version": version,
        "samples": rows,
        "negative_control": {
            "sample": manifest["samples"][0]["key"],
            "mutation": mutation,
            "rejected_events": control.failed,
            "n_consumed": control.n_consumed,
            "n_total": n_broken,
            "complaints": complaints(control.log),
            "status": "pass" if control_ok else "fail",
        },
        "status": "pass" if ok else "fail",
    }
    ROW_PATH.parent.mkdir(parents=True, exist_ok=True)
    ROW_PATH.write_text(json.dumps(row, indent=2) + "\n")
    print(f"wrote {ROW_PATH}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
