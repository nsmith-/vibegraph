"""Parsers for the files kept from a MadGraph gridpack and for vibegraph's output.

Everything here is a pure function of file text, so the database can be rebuilt
from the extract cache whenever a parser improves (bump PARSER_VERSION). Nothing
from a gridpack is ever imported or executed: UFO model files are read with
`ast.parse`, never `import`.
"""

import ast
import hashlib
import json
import re
import warnings

PARSER_VERSION = 1

# --- proc card -------------------------------------------------------------

PROC_KINDS = {"generate", "add"}
ORDER_RE = re.compile(r"\b(\w+)\s*(\^2)?\s*(==|<=|>=|=|<|>)\s*(-?\d+)")


def parse_import(line):
    """`import model NAME[-RESTRICTION] [flags]` -> (base, restriction, flags), else None."""
    m = re.match(r"\s*import\s+model\s+(\S+)(.*)$", line)
    if not m:
        return None
    name, rest = m.group(1), m.group(2).strip()
    base, restr = name, None
    tail = name.rsplit("/", 1)[-1]
    if "-" in tail:
        base, restr = name.rsplit("-", 1)
    return base, restr, rest


def parse_process(text):
    """One `generate`/`add process` body (after the keyword) -> dict of its parts.

    The core process is the part before the first decay-chain comma; its final
    state is the segment after the last `>` (required s-channel intermediates
    sit between `>`s), with coupling orders, `/` and `$` exclusions, `@N`,
    `[...]` and `{...}` removed.
    """
    body = text.strip()
    tag = None
    m = re.search(r"@\s*(\d+)", body)
    if m:
        tag = int(m.group(1))
    body_wo_tag = re.sub(r"@\s*\d+", "", body)
    nlo = re.search(r"\[([^\]]*)\]", body_wo_tag)
    core = body_wo_tag.split(",")[0]
    orders = {}
    for name, sq, op, val in ORDER_RE.findall(core):
        orders[name + (sq or "")] = op + val
    stripped = ORDER_RE.sub(" ", core)
    stripped = re.sub(r"\[[^\]]*\]|\{[^}]*\}", " ", stripped)
    excluded = []
    m = re.search(r"([/$]+)\s*(.*)$", stripped)
    if m:
        # kept with their `/` (no such propagator) and `$` (not on shell) markers
        excluded = (m.group(1) + " " + m.group(2)).split()
        stripped = stripped[: m.start()]
    parts = [p.split() for p in stripped.split(">")]
    initial = parts[0] if parts else []
    final = parts[-1] if len(parts) > 1 else []
    return {
        "tag": tag,
        "initial_state": " ".join(initial),
        "final_state": " ".join(final),
        "n_final": len(final),
        "coupling_orders": orders,
        "excluded": excluded,
        "has_decay_chain": "," in body_wo_tag.split("[")[0],
        "nlo_bracket": nlo.group(1).strip() if nlo else None,
    }


def parse_proc_card(text):
    """-> {'import': (base, restr, flags)|None, 'defines': {label: members},
    'processes': [(kind, body, parsed)], 'commands': [(command, line)]}."""
    out = {"import": None, "defines": {}, "processes": [], "commands": []}
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        tok = line.split()
        cmd = tok[0].lower()
        if cmd == "import" and out["import"] is None and parse_import(line):
            out["import"] = parse_import(line)
        elif cmd == "define" and "=" in line:
            label, members = line[len(tok[0]):].split("=", 1)
            out["defines"][label.strip()] = " ".join(members.split())
        elif cmd == "generate":
            body = line[len(tok[0]):]
            out["processes"].append(("generate", body.strip(), parse_process(body)))
        elif cmd == "add" and len(tok) > 1 and tok[1].lower() == "process":
            body = line.split(None, 2)[2] if len(tok) > 2 else ""
            out["processes"].append(("add", body.strip(), parse_process(body)))
        elif cmd in ("output", "import"):
            continue
        else:
            out["commands"].append((cmd, line))
    return out


# --- run card --------------------------------------------------------------

def parse_run_card(text):
    """MadGraph's reader: `value = name ! comment`, with a `#` after the name
    marking a hidden parameter (MadGraph still applies it). -> [(key, value, hidden)]."""
    out, seen = [], set()
    for raw in text.splitlines():
        if raw.lstrip().startswith("#"):
            continue
        body = raw.split("!", 1)[0]
        if "=" not in body:
            continue
        value, name = body.rsplit("=", 1)
        hidden = "#" in name
        key = name.split("#", 1)[0].strip().lower()
        if not key or key in seen:
            continue
        seen.add(key)
        out.append((key, value.strip(), hidden))
    return out


def _norm_value(x):
    if isinstance(x, bool):
        return 1.0 if x else 0.0
    if isinstance(x, (int, float)):
        return float(x)
    if isinstance(x, (list, dict)):
        return json.dumps(x, sort_keys=True)
    s = str(x).strip().strip("'\"")
    low = s.lower()
    if low in ("true", ".true.", "t"):
        return 1.0
    if low in ("false", ".false.", "f"):
        return 0.0
    try:
        return float(low.replace("d", "e"))
    except ValueError:
        return low


def load_mg_defaults(path):
    with open(path) as fh:
        raw = json.load(fh)
    return {k.lower(): v for k, v in raw.items()}


def default_for(defaults, key, value):
    """(default as text, is_default as 0/1) or (None, None) when MG has no such key."""
    if key not in defaults:
        return None, None
    d = defaults[key]
    dtext = json.dumps(d) if isinstance(d, (list, dict)) else str(d)
    if isinstance(d, (list, dict)):
        return dtext, None
    return dtext, int(_norm_value(value) == _norm_value(d))


# --- param card ------------------------------------------------------------

def param_entries(text):
    """{(block, ids): value} for an SLHA card; DECAY lines as ('DECAY', pdg)."""
    out, block = {}, None
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        tok = line.split()
        head = tok[0].upper()
        if head == "BLOCK":
            block = tok[1].upper() if len(tok) > 1 else None
        elif head == "DECAY":
            block = None
            if len(tok) >= 3:
                out[("DECAY", tok[1])] = tok[2]
        elif block and len(tok) >= 2:
            out[(block, " ".join(tok[:-1]))] = tok[-1]
    return out


def param_diff(card_text, default_text):
    a, b = param_entries(card_text), param_entries(default_text)
    diffs = []
    for k in sorted(set(a) | set(b)):
        va, vb = a.get(k), b.get(k)
        try:
            same = va is not None and vb is not None and float(va.replace("d", "e").replace("D", "e")) == float(
                vb.replace("d", "e").replace("D", "e"))
        except ValueError:
            same = va == vb
        if not same:
            diffs.append((k[0], k[1], va, vb))
    return diffs


# --- MadGraph generation log -----------------------------------------------

DURATION_TOKEN = re.compile(r"(\d+(?:\.\d+)?)\s*(days?|h|m|s)(?![a-z])")


def parse_duration(text):
    """'1m 12s', '4.4s', '1 day,16h32m55s', '96 days,18h19m24s' -> seconds."""
    total, hit = 0.0, False
    for num, unit in DURATION_TOKEN.findall(text.replace(",", " ")):
        hit = True
        v = float(num)
        total += v * {"d": 86400, "h": 3600, "m": 60, "s": 1}[unit[0]]
    return total if hit else None


STEP_MARKERS = [
    ("survey", re.compile(r"INFO: Running Survey")),
    ("refine", re.compile(r"INFO: (Refining results|Running Refine)")),
    ("gridpack_test", re.compile(r"^Now generating \d+ events")),
    ("nlo_run", re.compile(r"INFO: Starting run")),
]
PROGRESS = re.compile(r"Idle:\s*(\d+),\s*Running:\s*(\d+),\s*Completed:\s*(\d+)\s*\[\s*([^\]]*?)\s*\]")
LO_XSEC = re.compile(r"Cross-section\s*:\s*([0-9.eE+-]+)\s*\+-\s*([0-9.eE+-]+)\s*pb")
NLO_XSEC = re.compile(r"Total cross section:\s*([0-9.eE+-]+)\s*\+-\s*([0-9.eE+-]+)\s*pb")
NB_EVENTS = re.compile(r"Nb of events\s*:\s*(\d+)")
DIAG_GEN = re.compile(r"^(\d+) processes with (\d+) diagrams generated in ([0-9.]+) s")
DIAG_TOTAL = re.compile(r"^Total: (\d+) processes with (\d+) diagrams")
HELAS = re.compile(r"Generated helas calls for (\d+) subprocesses \((\d+) diagrams\) in ([0-9.]+) s")
WROTE = re.compile(r"Wrote files for (\d+) helas calls in ([0-9.]+) s")
NLO_GEN = re.compile(r"Generated (\d+) subprocesses with (\d+) real emission diagrams, (\d+) born diagrams "
                     r"and (\d+) virtual diagrams")
CPU = re.compile(r"sum of cpu time of last step:\s*(.*)$")
SYSTEM = re.compile(r"^System release\s+(.*)$")


def _f(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def parse_gen_log(text):
    """-> {'generation': {...}, 'diagram_gen': [...], 'steps': [...], 'xsecs': [...], 'os': str|None}.

    Each step runs from its marker line to the next marker. Wall time is the
    longest elapsed time its progress lines print; MadGraph prints the clock
    time instead (`current time: 15h42`) on a step's first line, which is skipped.
    """
    gen = {}
    diagram_gen, steps, xsecs = [], [], []
    host_os = None
    cur = None
    nlo_collecting = False
    pending = None
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if host_os is None:
            m = SYSTEM.match(s)
            if m:
                host_os = m.group(1).strip()
        m = DIAG_GEN.match(s)
        if m:
            diagram_gen.append({"n_processes": int(m.group(1)), "n_diagrams": int(m.group(2)),
                                "seconds": float(m.group(3))})
        m = DIAG_TOTAL.match(s)
        if m:
            gen["n_processes"], gen["n_diagrams"] = int(m.group(1)), int(m.group(2))
        m = HELAS.search(s)
        if m:
            gen["n_helas_subprocesses"], gen["n_helas_diagrams"] = int(m.group(1)), int(m.group(2))
            gen["helas_gen_s"] = float(m.group(3))
        m = WROTE.search(s)
        if m:
            gen["n_helas_calls"], gen["write_files_s"] = int(m.group(1)), float(m.group(2))
        m = NLO_GEN.search(s)
        if m:
            (gen["nlo_subprocesses"], gen["nlo_real_diagrams"], gen["nlo_born_diagrams"],
             gen["nlo_virtual_diagrams"]) = (int(g) for g in m.groups())
        for name, rx in STEP_MARKERS:
            if rx.search(line):
                cur = {"step": name, "first_line": i, "wall_s": None, "cpu_s": None, "n_jobs": None}
                steps.append(cur)
                nlo_collecting = False
                break
        if cur is not None:
            m = PROGRESS.search(s)
            if m:
                jobs = sum(int(g) for g in m.groups()[:3])
                cur["n_jobs"] = max(cur["n_jobs"] or 0, jobs)
                if "current time" not in m.group(4):
                    d = parse_duration(m.group(4))
                    if d is not None:
                        cur["wall_s"] = max(cur["wall_s"] or 0.0, d)
            m = CPU.search(s)
            if m:
                cur["cpu_s"] = parse_duration(m.group(1))
        if "INFO: Collecting events" in s:
            nlo_collecting = True
        m = LO_XSEC.search(s)
        if m:
            pending = {"source": "gen_log", "line": i, "step": cur["step"] if cur else "unknown",
                       "sigma_pb": _f(m.group(1)), "err_pb": _f(m.group(2)), "nevents": None}
            xsecs.append(pending)
            continue
        m = NLO_XSEC.search(s)
        if m:
            step = "nlo_final" if nlo_collecting else "nlo_intermediate"
            xsecs.append({"source": "gen_log", "line": i, "step": step,
                          "sigma_pb": _f(m.group(1)), "err_pb": _f(m.group(2)), "nevents": None})
            pending = None
            continue
        m = NB_EVENTS.search(s)
        if m and pending is not None:
            pending["nevents"] = int(m.group(1))
            pending = None
    return {"generation": gen, "diagram_gen": diagram_gen, "steps": steps, "xsecs": xsecs, "os": host_os}


def parse_results_dat(text):
    """SubProcesses/results.dat: first line's columns 1-2 are σ and its error (pb)."""
    out = {"xsec": None, "cumulated_time_s": None}
    lines = text.splitlines()
    if lines:
        tok = lines[0].split()
        if len(tok) >= 2 and _f(tok[0]) is not None:
            out["xsec"] = {"source": "results_dat", "line": 1, "step": "survey",
                           "sigma_pb": _f(tok[0]), "err_pb": _f(tok[1]), "nevents": None}
    m = re.search(r"<cumulated_time>\s*([0-9.eE+-]+)\s*</cumulated_time>", text)
    if m:
        out["cumulated_time_s"] = _f(m.group(1))
    return out


def usable(x):
    return (x["sigma_pb"] is not None and x["err_pb"] is not None and x["sigma_pb"] > 0
            and x["err_pb"] < x["sigma_pb"])


def choose_reference(xsecs):
    """Index of the σ comparisons should use, or None.

    NLO: the last result printed after event collection, else the last
    intermediate one. LO: the last survey/refine result from the log, else
    results.dat. The gridpack's own test-generation run is never the
    reference, and neither is a result whose error exceeds its value (an
    unconverged pilot survey prints those).
    """
    order = [("nlo_final",), ("nlo_intermediate",), ("refine", "survey")]
    for steps in order:
        for src in ("gen_log", "results_dat"):
            cands = [i for i, x in enumerate(xsecs) if x["step"] in steps and x["source"] == src and usable(x)]
            if cands:
                return cands[-1]
    return None


# --- UFO models --------------------------------------------------------------

def _kw(call, name):
    for k in call.keywords:
        if k.arg == name:
            try:
                return ast.literal_eval(k.value)
            except (ValueError, SyntaxError):
                return None
    return None


def _calls(tree, fname):
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            f = node.func
            name = f.id if isinstance(f, ast.Name) else f.attr if isinstance(f, ast.Attribute) else None
            if name == fname:
                yield node


def _attr_names(node):
    out = []
    if isinstance(node, (ast.List, ast.Tuple)):
        for e in node.elts:
            if isinstance(e, ast.Attribute):
                out.append(e.attr)
            elif isinstance(e, ast.Name):
                out.append(e.id)
    return out


def ufo_stats(files):
    """Counts and features of a UFO model from {relpath: text} of its .py files.

    has_fnv_vertex is a heuristic: a vertex joining exactly two Dirac
    fermions that are both particles, or both antiparticles.
    """
    trees, dialect = {}, "py3"
    for rel, text in files.items():
        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                trees[rel] = ast.parse(text)
        except SyntaxError:
            dialect = "py2"
    st = {"python_dialect": dialect}
    particles = {}
    if "particles.py" in trees:
        for c in _calls(trees["particles.py"], "Particle"):
            name, anti = _kw(c, "name"), _kw(c, "antiname")
            particles[name] = {"spin": _kw(c, "spin"), "color": _kw(c, "color"),
                               "self_conjugate": name == anti}
        pyvars = {}
        for node in ast.walk(trees["particles.py"]):
            if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call):
                for c in _calls(node.value, "Particle"):
                    for t in node.targets:
                        if isinstance(t, ast.Name):
                            pyvars[t.id] = _kw(c, "name")
        st["n_particles"] = len(particles)
        st["spins"] = ",".join(str(s) for s in sorted({p["spin"] for p in particles.values() if p["spin"] is not None}))
        st["colour_reps"] = ",".join(str(s) for s in sorted({p["color"] for p in particles.values() if p["color"] is not None}))
        st["has_majorana"] = int(any(p["self_conjugate"] and p["spin"] in (2, 4) for p in particles.values()))
    else:
        pyvars = {}
    if "vertices.py" in trees:
        n, fnv = 0, False
        for c in _calls(trees["vertices.py"], "Vertex"):
            n += 1
            plist = next((k.value for k in c.keywords if k.arg == "particles"), None)
            ferm = []
            for v in _attr_names(plist):
                anti = v.endswith("__tilde__")
                base = pyvars.get(v[: -len("__tilde__")] if anti else v)
                info = particles.get(base)
                if info and info["spin"] in (2, 4) and not info["self_conjugate"]:
                    ferm.append(anti)
            if len(ferm) == 2 and ferm[0] == ferm[1]:
                fnv = True
        st["n_vertices"] = n
        st["has_fnv_vertex"] = int(fnv)
    for fname, rel, key in (("Coupling", "couplings.py", "n_couplings"), ("Lorentz", "lorentz.py", "n_lorentz")):
        if rel in trees:
            st[key] = sum(1 for _ in _calls(trees[rel], fname))
    if "parameters.py" in trees:
        params = list(_calls(trees["parameters.py"], "Parameter"))
        st["n_parameters"] = len(params)
        st["n_external_parameters"] = sum(1 for c in params if _kw(c, "nature") == "external")
    cond = False
    for rel in ("couplings.py", "parameters.py"):
        if rel in trees:
            for c in _calls(trees[rel], "Coupling" if rel == "couplings.py" else "Parameter"):
                v = _kw(c, "value")
                if isinstance(v, str) and re.search(r"\bif\b.*\belse\b", v):
                    cond = True
    st["has_conditional_expr"] = int(cond)
    if "coupling_orders.py" in trees:
        st["coupling_orders"] = ",".join(sorted(
            n for n in (_kw(c, "name") for c in _calls(trees["coupling_orders.py"], "CouplingOrder")) if n))
    return st


def model_identity(file_shas):
    """sha256 over the sorted 'relpath\\tsha256' lines of a model's .py files."""
    h = hashlib.sha256()
    for rel in sorted(file_shas):
        h.update(("%s\t%s\n" % (rel, file_shas[rel])).encode())
    return h.hexdigest()


# --- vibegraph output ------------------------------------------------------

VG_SIGMA = re.compile(r"^σ = ([0-9.eE+-]+) ± ([0-9.eE+-]+) (\w+)", re.M)
VG_MULT = re.compile(r"(\d+) outgoing legs \(@(\d+)\): σ = ([0-9.eE+-]+) ± ([0-9.eE+-]+) (\w+) over (\d+) channels")
VG_VEGAS = re.compile(r"VEGAS:\s+(\d+) evals over (\d+) iters, seed (\d+) \(χ²/dof = ([0-9.eE+-]+|NaN|nan)\)")
VG_CHANNELS = re.compile(r"channels:\s+(\d+) grids")
VG_MODEL = re.compile(r"model:?\s+(\S+) \(([0-9a-f]{64})\)")
VG_ENUMERATED = re.compile(r"enumerated (\d+) diagrams in (\d+) subprocess")
VG_TS = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?)Z\s+[A-Z]+\s+(\S+)")
VG_STAGES = ("proc_card", "ufo_load", "enumerate", "compile", "alpha_survey", "vegas")
UNIT_TO_PB = {"fb": 1e-3, "pb": 1.0, "nb": 1e3, "ub": 1e6, "mb": 1e9}

# Ordered: the first pattern that matches the error text names the class.
REFUSAL_CLASSES = [
    ("nlo_bracket", r"\[QCD\]|NLO|loop-induced|\[\s*virt|\[\s*real"),
    ("restriction_unknown", r"restrict(ion)? (variant|card)|no restriction|unknown restriction"),
    ("event_norm", r"event_norm"),
    ("time_of_flight", r"time_of_flight"),
    ("ht_cuts", r"\b(ihtmin|ihtmax|htjmin|htjmax|ht[234]m(in|ax))\b"),
    ("unimplemented_cut", r"activates cut"),
    ("flavour_mixed_mass", r"different outgoing masses"),
    ("massive_initial_leg", r"incoming leg of mass"),
    ("me_frame", r"me_frame|polarizes the massive leg"),
    ("colour_crossing", r"colou?r[- ]basis|crossing"),
    ("no_diagrams", r"no diagrams|0 diagrams|has no diagram"),
    ("particle_name", r"unknown particle|no particle named|not a particle"),
    ("model_load", r"failed to load model|UFO|parse"),
    ("proc_card_command", r"proc[_ ]card"),
    ("pdf", r"PDF|lhaid|pdlabel"),
]


def classify_error(text):
    for name, rx in REFUSAL_CLASSES:
        if re.search(rx, text, re.I):
            return name
    return "other"


def parse_vg_output(stdout_text, trace_text, exit_code, timed_out):
    """Outcome, σ and per-multiplicity results from the CLI's output, and
    per-stage wall times from the `--log-file` trace (span of each stage's
    first to last timestamped line)."""
    both = stdout_text or ""
    res = {"sigma_pb": None, "err_pb": None, "chi2_dof": None, "n_evals": None, "n_channels": None,
           "n_subprocesses": None, "n_diagrams": None, "vg_model_hash": None, "multiplicities": [],
           "stages": {}, "error_text": None, "refusal_class": None}
    src = both + "\n" + (trace_text or "")
    sig = list(VG_SIGMA.finditer(both))
    if sig:
        m = sig[-1]
        f = UNIT_TO_PB.get(m.group(3), None)
        if f is not None:
            res["sigma_pb"], res["err_pb"] = float(m.group(1)) * f, float(m.group(2)) * f
    for m in VG_MULT.finditer(src):
        f = UNIT_TO_PB.get(m.group(5), 1.0)
        res["multiplicities"].append({"n_out": int(m.group(1)), "tag": int(m.group(2)),
                                      "sigma_pb": float(m.group(3)) * f, "err_pb": float(m.group(4)) * f,
                                      "n_channels": int(m.group(6))})
    seen = {}
    for mm in res["multiplicities"]:
        seen[mm["tag"]] = mm
    res["multiplicities"] = list(seen.values())
    m = None
    for m in VG_VEGAS.finditer(src):
        pass
    if m:
        res["n_evals"], res["chi2_dof"] = int(m.group(1)), _f(m.group(4))
    m = VG_CHANNELS.search(src)
    if m:
        res["n_channels"] = int(m.group(1))
    m = VG_MODEL.search(src)
    if m:
        res["vg_model_hash"] = m.group(2)
    enum = [(int(d), int(n)) for d, n in VG_ENUMERATED.findall(both)]
    if enum:
        res["n_diagrams"], res["n_subprocesses"] = sum(d for d, _ in enum), sum(n for _, n in enum)
    if trace_text:
        first, last = {}, {}
        from datetime import datetime
        for line in trace_text.splitlines():
            m = VG_TS.match(line)
            if not m:
                continue
            stage = re.sub(r"\{.*", "", m.group(2)).rstrip(":")
            if stage not in VG_STAGES:
                continue
            t = datetime.fromisoformat(m.group(1)).timestamp()
            first.setdefault(stage, t)
            last[stage] = t
        res["stages"] = {s: last[s] - first[s] for s in first}
    if timed_out:
        res["outcome"] = "timeout"
    elif exit_code == 0 and res["sigma_pb"] is not None:
        res["outcome"] = "ok"
    else:
        lines = both.splitlines()
        if "panicked at" in both:
            res["outcome"] = "panic"
            idx = next(i for i, l in enumerate(lines) if "panicked at" in l)
        else:
            idx = next((i for i, l in enumerate(lines) if re.search(r"\bERROR\b|^error:", l)), None)
            res["outcome"] = "refused" if idx is not None and exit_code not in (None, 0) else "error"
        res["error_text"] = "\n".join(lines[idx:] if idx is not None else lines[-15:])[:6000]
        if res["outcome"] == "refused":
            res["refusal_class"] = classify_error(res["error_text"])
    return res
