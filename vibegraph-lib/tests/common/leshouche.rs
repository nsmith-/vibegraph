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

use std::path::{Path, PathBuf};

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
