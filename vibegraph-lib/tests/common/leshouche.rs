//! MadGraph's `leshouche.inc` read as an oracle, and the event patterns it allows.
//!
//! Each `SubProcesses/P*/leshouche.inc` holds one table per subprocess index
//! `isproc`: the external PDG codes `IDUP(leg, iproc, isproc)` — one row per
//! concrete flavour assignment sharing that table — and the Les Houches colour
//! tags `ICOLUP(slot, leg, iflow, isproc)`. `isproc N` is the process the sibling
//! `matrix<N>_orig.f`'s `C     Process:` header names.
//!
//! Reading the tables out of the generated Fortran is what makes them an oracle
//! rather than a transcription: they are MadGraph's own statement about every
//! subprocess it wrote code for, including the ones a finite event sample happens
//! never to reach.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use vibegraph::helas::color::flow_tags::slots_for;
use vibegraph::helas::repr::color::ColorRep;
use vibegraph::lhef::record::LheEvent;

/// One subprocess of one `P*` directory: MadGraph's `isproc`, its flavour rows and
/// its colour-flow table.
#[derive(Clone, Debug)]
pub struct Subprocess {
    /// MadGraph's `isproc`, one-based.
    pub isproc: usize,
    /// `IDUP(·, iproc, isproc)` for every `iproc`, in `iproc` order. All of them
    /// share the colour table below.
    pub idup: Vec<Vec<i32>>,
    /// `[flow][leg] = [colour, anticolour]`.
    pub flows: Vec<Vec<[u32; 2]>>,
}

/// Parse every subprocess of one `leshouche.inc`, in `isproc` order.
pub fn parse(path: &Path) -> Result<Vec<Subprocess>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    let mut subprocs: Vec<Subprocess> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let (name, rest) = if let Some(rest) = trimmed.strip_prefix("DATA (ICOLUP(") {
            ("ICOLUP", rest)
        } else if let Some(rest) = trimmed.strip_prefix("DATA (IDUP(") {
            ("IDUP", rest)
        } else {
            continue;
        };
        let (args, tail) = rest
            .split_once(')')
            .ok_or_else(|| format!("unterminated {name} subscript: {line}"))?;
        let subscripts: Vec<&str> = args.split(',').map(str::trim).collect();
        if subscripts.len() != 3 + usize::from(name == "ICOLUP") {
            return Err(format!("unexpected {name} rank: {line}"));
        }
        let index = |i: usize| -> Result<usize, String> {
            subscripts[i]
                .parse::<usize>()
                .map_err(|_| format!("bad {name} subscript '{}': {line}", subscripts[i]))
        };
        let isproc = index(subscripts.len() - 1)?;
        let body = tail
            .split_once('/')
            .and_then(|(_, r)| r.rsplit_once('/').map(|(v, _)| v))
            .ok_or_else(|| format!("no /…/ in {name} line: {line}"))?;
        let values: Vec<i64> = body
            .split(',')
            .map(|t| {
                t.trim()
                    .parse::<i64>()
                    .map_err(|_| format!("bad {name} value '{t}'"))
            })
            .collect::<Result<_, _>>()?;
        if subprocs.len() < isproc {
            subprocs.resize_with(isproc, || Subprocess {
                isproc: 0,
                idup: Vec::new(),
                flows: Vec::new(),
            });
        }
        let sub = &mut subprocs[isproc - 1];
        sub.isproc = isproc;
        match name {
            "IDUP" => {
                let iproc = index(1)?;
                if sub.idup.len() < iproc {
                    sub.idup.resize(iproc, Vec::new());
                }
                sub.idup[iproc - 1] = values
                    .into_iter()
                    .map(|v| i32::try_from(v).map_err(|_| format!("IDUP out of range: {line}")))
                    .collect::<Result<_, _>>()?;
            }
            _ => {
                let slot = index(0)?;
                let flow = index(2)?;
                if !(1..=2).contains(&slot) {
                    return Err(format!("ICOLUP slot outside 1..2: {line}"));
                }
                if sub.flows.len() < flow {
                    sub.flows.resize(flow, Vec::new());
                }
                let row = &mut sub.flows[flow - 1];
                if row.is_empty() {
                    row.resize(values.len(), [0; 2]);
                }
                if row.len() != values.len() {
                    return Err(format!("ICOLUP leg count changes within a flow: {line}"));
                }
                for (leg, v) in values.into_iter().enumerate() {
                    row[leg][slot - 1] = u32::try_from(v)
                        .map_err(|_| format!("negative ICOLUP label: {line}"))?;
                }
            }
        }
    }
    for (i, sub) in subprocs.iter().enumerate() {
        if sub.isproc == 0 {
            return Err(format!("gap at isproc {} in {path:?}", i + 1));
        }
        if sub.idup.iter().any(Vec::is_empty) || sub.flows.iter().any(Vec::is_empty) {
            return Err(format!("gap in isproc {}'s tables in {path:?}", sub.isproc));
        }
    }
    Ok(subprocs)
}

/// Every `SubProcesses/P*/leshouche.inc` under one MadGraph run directory, sorted.
pub fn files_under(run_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect(run_dir, &mut out);
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("leshouche.inc")
            && path.parent().is_some_and(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with('P'))
            })
        {
            out.push(path);
        }
    }
}

/// A PDG code reduced to the role it plays in the colour flow. Two events with the
/// same roles on the same legs must carry the same colour connectivity, whatever
/// generation or lepton flavour they happen to be.
pub fn role(pdg: i32) -> i32 {
    match pdg {
        21 => 21,
        q if (1..=6).contains(&q) => 1,
        q if (-6..=-1).contains(&q) => -1,
        _ => 0,
    }
}

/// The roles of an event's external legs, in record order.
pub fn roles(event: &LheEvent) -> Vec<i32> {
    event.particles.iter().map(|p| role(p.pdg)).collect()
}

/// The colour lines an event carries: the `(leg, slot)` endpoints sharing a label,
/// with the label itself discarded because any consistent relabelling is the same
/// event.
pub fn connectivity(event: &LheEvent) -> Vec<Vec<(usize, usize)>> {
    event.color_connectivity()
}

/// The same, off a raw `ICOLUP` table row rather than off a parsed event.
pub fn connectivity_of(tags: &[[u32; 2]]) -> Vec<Vec<(usize, usize)>> {
    let mut labels: Vec<u32> = tags.iter().flatten().copied().filter(|&c| c != 0).collect();
    labels.sort_unstable();
    labels.dedup();
    let mut lines: Vec<Vec<(usize, usize)>> = labels
        .into_iter()
        .map(|label| {
            let mut ends = Vec::new();
            for (leg, pair) in tags.iter().enumerate() {
                for (slot, &c) in pair.iter().enumerate() {
                    if c == label {
                        ends.push((leg, slot));
                    }
                }
            }
            ends
        })
        .collect();
    lines.sort_unstable();
    lines
}

/// One event's flavour roles paired with its colour connectivity.
pub type ColourPattern = (Vec<i32>, Vec<Vec<(usize, usize)>>);

/// The `(roles, connectivity)` patterns a set of events exhibits.
pub fn patterns_of_events<'a>(
    events: impl IntoIterator<Item = &'a LheEvent>,
) -> BTreeSet<ColourPattern> {
    events
        .into_iter()
        .map(|e| (roles(e), connectivity(e)))
        .collect()
}

/// Every `(roles, connectivity)` pattern MadGraph's tables admit for one run: each
/// `P*` directory, each `isproc`, each flavour row, each colour flow, and each of
/// the two beam orderings.
///
/// The exchanged ordering is included because the enumeration produces one ordering
/// of each unordered initial state and the mirror term supplies the other, so an
/// emitted event may carry either.
pub fn allowed_patterns(run_dir: &Path) -> Result<BTreeSet<ColourPattern>, String> {
    let files = files_under(run_dir);
    if files.is_empty() {
        return Err(format!("no SubProcesses/P*/leshouche.inc under {run_dir:?}"));
    }
    let mut out = BTreeSet::new();
    for file in &files {
        for sub in parse(file)? {
            for idup in &sub.idup {
                for flow in &sub.flows {
                    if flow.len() != idup.len() {
                        return Err(format!(
                            "{file:?} isproc {}: {} colour legs against {} flavour legs",
                            sub.isproc,
                            flow.len(),
                            idup.len()
                        ));
                    }
                    let direct: Vec<i32> = idup.iter().map(|&p| role(p)).collect();
                    out.insert((direct.clone(), connectivity_of(flow)));
                    let mut exchanged_roles = direct;
                    exchanged_roles.swap(0, 1);
                    let mut exchanged_tags = flow.clone();
                    exchanged_tags.swap(0, 1);
                    out.insert((exchanged_roles, connectivity_of(&exchanged_tags)));
                }
            }
        }
    }
    Ok(out)
}

/// The SU(3) rep a PDG code carries, for a scan that has only a written event file
/// to read.
///
/// This is a second PDG → rep table, and deliberately so: it is what lets the scan
/// below be *reference-free*, judging an emitted record against the Les Houches
/// convention itself rather than against anything the generator computed. The
/// convention leaves no room — `ICOLUP(1)` is the colour line and `ICOLUP(2)` the
/// anticolour, so a quark occupies the first and an antiquark the second — so there
/// is no convention here for the two tables to disagree about.
pub fn color_rep(pdg: i32) -> ColorRep {
    match pdg {
        21 => ColorRep::Octet,
        q if (1..=6).contains(&q) => ColorRep::Triplet,
        q if (-6..=-1).contains(&q) => ColorRep::AntiTriplet,
        _ => ColorRep::Singlet,
    }
}

/// Every leg of `event` whose occupied `ICOLUP` slots are not the ones its own PDG
/// code's colour rep allows, as `(leg, [colour, anticolour] occupancy)`.
///
/// A triplet fills only the colour slot, an antitriplet only the anticolour slot, an
/// octet both and a singlet neither. No reference is consulted.
pub fn illegal_slots(event: &LheEvent) -> Vec<(usize, [bool; 2])> {
    event
        .particles
        .iter()
        .enumerate()
        .filter_map(|(leg, p)| {
            let got = [p.color[0] != 0, p.color[1] != 0];
            (got != slots_for(color_rep(p.pdg))).then_some((leg, got))
        })
        .collect()
}
