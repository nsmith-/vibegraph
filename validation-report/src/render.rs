//! The two renderings of a resolved report: the markdown a reader reads and the
//! JSON another program reads. Neither decides anything — every mark, metric and
//! problem is settled before either is called.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

use serde_json::{json, Value};

use crate::manifest::{Manifest, Standalone, CATEGORIES};
use crate::ResolvedRow;

pub fn markdown(
    manifest: &Manifest,
    resolved: &[ResolvedRow],
    report_dir: &Path,
    problems: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("# vibegraph validation report\n\n");
    out.push_str(
        "One row per validated process, one column per validation category, rendered from the\n\
         row files the gates wrote under `target/validation-report/` and checked against the\n\
         cell set `validation/manifest.toml` declares. A cell is ✅ when its gate ran and\n\
         passed, ⚠️ when it is measured and deliberately not enforced, ❌ when it failed,\n\
         ⏳ when the oracle layer owns it and its driver did not run in this cycle, or the\n\
         pinned bundle does not carry the row yet,\n\
         ⛔ when a named feature blocks it, `—` when another\n\
         row covers the same physics, and `uncovered` when nothing measures it and nothing\n\
         claims to.\n\n",
    );
    out.push_str(&format!(
        "Banked reference bundle: `{}` (version {}, {} bytes, sha256 `{}`, published: {}).\n\n",
        manifest.refdata.archive,
        manifest.refdata.version,
        manifest.refdata.size_bytes,
        manifest.refdata.sha256,
        if manifest.refdata.published {
            "yes"
        } else {
            "no — fetched through $VIBEGRAPH_REFDATA_SOURCE meanwhile"
        },
    ));

    out.push_str("| process | 2→N | diagrams | amplitudes | integrals | samples |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    let mut multi_channel_started = false;
    for row in resolved {
        if row.process.class != "single-channel" && !multi_channel_started {
            multi_channel_started = true;
            out.push_str("| **multi-channel** | | | | | |\n");
        }
        out.push_str(&format!(
            "| `{}` `{}` | {} |",
            row.process.key, row.process.process, row.process.n_final
        ));
        for cell in &row.cells {
            let marker = cell.note_ref.map(|n| format!(" [{n}]")).unwrap_or_default();
            let mark = if cell.mark.is_empty() {
                String::new()
            } else {
                format!("{} ", cell.mark)
            };
            out.push_str(&format!(" {mark}{}{marker} |", cell.body));
        }
        out.push('\n');
    }
    out.push('\n');

    // Cells sharing a note share its number, so each note is listed once with
    // every cell it belongs to.
    let mut notes: BTreeMap<usize, (Vec<String>, String)> = BTreeMap::new();
    for row in resolved {
        for cell in &row.cells {
            let (Some(n), Some(text)) = (cell.note_ref, cell.note.clone()) else {
                continue;
            };
            let entry = notes.entry(n).or_insert_with(|| (Vec::new(), text));
            entry.0.push(format!(
                "`{}` · {}",
                row.process.key,
                cell.category.as_str()
            ));
        }
    }
    if !notes.is_empty() {
        out.push_str("## Cell notes\n\n");
        for (n, (cells, text)) in &notes {
            out.push_str(&format!("{n}. {} — {text}\n", cells.join(", ")));
        }
        out.push('\n');
    }

    out.push_str("## Standalone gates\n\n");
    out.push_str(
        "Process-independent checks, run once per invocation of their layer. A gate whose\n\
         driver is a Rust test carries no row file of its own: it ran inside the same\n\
         `cargo test` invocation as the cells above, so a failure in it fails the suite\n\
         before this report is rendered.\n\n",
    );
    out.push_str("| gate | layer | verdict |\n|---|---|---|\n");
    for standalone in &manifest.standalone {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            standalone.key,
            standalone.layer,
            standalone_verdict(standalone, report_dir),
        ));
    }
    out.push('\n');

    out.push_str("## Measurement detail\n\n");
    for row in resolved {
        for cell in &row.cells {
            for detail in &cell.detail {
                out.push_str(&format!(
                    "- `{}` · {} — {detail}\n",
                    row.process.key,
                    cell.category.as_str()
                ));
            }
            if let Some(context) = &cell.context {
                out.push_str(&format!(
                    "- `{}` · {} — manifest: {context}\n",
                    row.process.key,
                    cell.category.as_str()
                ));
            }
        }
    }
    out.push('\n');

    let unbundled: Vec<&str> = manifest
        .processes
        .iter()
        .filter(|p| !p.bundled)
        .map(|p| p.key.as_str())
        .collect();
    let planned: Vec<&str> = manifest
        .processes
        .iter()
        .filter(|p| p.status.as_deref() == Some("planned"))
        .map(|p| p.key.as_str())
        .collect();
    out.push_str("## Coverage bookkeeping\n\n");
    out.push_str(&format!(
        "- rows whose banked artifacts are not in the pinned bundle: {}\n",
        list_or_none(&unbundled)
    ));
    out.push_str(&format!(
        "- rows whose reference run does not exist yet: {}\n\n",
        list_or_none(&planned)
    ));

    out.push_str(&timing(resolved, report_dir));

    out.push_str("## Verification\n\n");
    let (measured, marks) = tally(resolved);
    out.push_str(&format!(
        "{} rows × {} categories = {} cells: {measured} measured in the layers this run \
         drove ({}).\n\n",
        resolved.len(),
        CATEGORIES.len(),
        resolved.len() * CATEGORIES.len(),
        marks,
    ));
    if problems.is_empty() {
        out.push_str(
            "The measured cells are exactly the cells the manifest declares, every gate cell\n\
             passed, and every measurement agrees with the manifest about what it is.\n",
        );
    } else {
        out.push_str("The report does not match the manifest:\n\n");
        for problem in problems {
            out.push_str(&format!("- {problem}\n"));
        }
    }
    out
}

/// What this invocation cost, and on what machine.
///
/// Wall time per row as the gates timed themselves, against the host block they
/// wrote beside their rows. The rows overlap — `cargo test` runs a binary's
/// tests in parallel and the integrators fan out under them — so the per-category
/// figure is the sum of concurrent spans and not the invocation's elapsed time.
/// It is the shape of where a run's time goes, not a benchmark.
fn timing(resolved: &[ResolvedRow], report_dir: &Path) -> String {
    let mut per_category: BTreeMap<&str, (usize, f64)> = BTreeMap::new();
    let mut rows: Vec<(f64, String)> = Vec::new();
    for row in resolved {
        for cell in &row.cells {
            for (label, seconds) in &cell.durations {
                let entry = per_category
                    .entry(cell.category.as_str())
                    .or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += seconds;
                rows.push((
                    *seconds,
                    format!(
                        "`{}` · {} · {label}",
                        row.process.key,
                        cell.category.as_str()
                    ),
                ));
            }
        }
    }
    if rows.is_empty() {
        return String::new();
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut out = String::from("## Timing\n\n");
    out.push_str(&format!("- host: {}\n", host_line(report_dir)));
    out.push_str(
        "- per-row wall times overlap: the gates run in parallel threads, so a category's total \
         is the sum of concurrent spans rather than this invocation's elapsed time\n\n",
    );
    out.push_str("| category | timed measurements | summed wall time |\n|---|--:|--:|\n");
    for (category, (n, seconds)) in &per_category {
        out.push_str(&format!("| {category} | {n} | {seconds:.1} s |\n"));
    }
    out.push_str("\nSlowest measurements:\n\n");
    for (seconds, what) in rows.iter().take(10) {
        out.push_str(&format!("- {seconds:8.1} s — {what}\n"));
    }
    out.push('\n');
    out
}

/// The one-line summary of the machine block the gates wrote, or why there is
/// none to summarise.
fn host_line(report_dir: &Path) -> String {
    let path = report_dir.join("host.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return "no `host.json` — the durations below name no machine".to_string();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return "`host.json` is unreadable".to_string();
    };
    let at = |path: [&str; 2]| -> String {
        v.get(path[0])
            .and_then(|o| o.get(path[1]))
            .map(|x| match x.as_str() {
                Some(s) => s.to_string(),
                None => x.to_string(),
            })
            .unwrap_or_else(|| "?".to_string())
    };
    format!(
        "{} ({} logical cores), {}, {}, profile {} — full block in `host.json`",
        at(["cpu", "model"]),
        at(["cpu", "logical_cpus"]),
        at(["os", "kernel"]),
        at(["toolchain", "rustc"]),
        at(["build", "profile"]),
    )
}

fn list_or_none(keys: &[&str]) -> String {
    if keys.is_empty() {
        "none".to_string()
    } else {
        keys.iter()
            .map(|k| format!("`{k}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// How many cells carry each mark, for the line under the table.
fn tally(resolved: &[ResolvedRow]) -> (usize, String) {
    let mut measured = 0;
    let mut counts = [0usize; 6];
    for row in resolved {
        for cell in &row.cells {
            let index = match cell.mark {
                "✅" => 0,
                "⚠️" => 1,
                "❌" => 2,
                "⏳" => 3,
                "⛔" => 4,
                _ => 5,
            };
            counts[index] += 1;
            if index <= 2 {
                measured += 1;
            }
        }
    }
    let names = ["✅", "⚠️", "❌", "⏳", "⛔", "— / uncovered"];
    let text = names
        .iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(name, n)| format!("{n} {name}"))
        .collect::<Vec<_>>()
        .join(", ");
    (measured, text)
}

/// What the list under the table says about one standalone gate.
fn standalone_verdict(standalone: &Standalone, report_dir: &Path) -> String {
    let Some(row) = standalone.row.as_deref() else {
        if standalone.layer == "oracle" {
            return format!(
                "the oracle layer runs it — `pixi run{} {}` ({})",
                standalone
                    .environment
                    .as_deref()
                    .map(|e| format!(" -e {e}"))
                    .unwrap_or_default(),
                standalone.task.as_deref().unwrap_or("<task>"),
                standalone.targets.join(", "),
            );
        }
        return format!(
            "ran with the {} layer's suite ({})",
            standalone.layer,
            standalone.targets.join(", ")
        );
    };
    let path = report_dir.join("standalone").join(row);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return format!(
            "not run in this invocation — `pixi run{} {}` writes `standalone/{row}`",
            standalone
                .environment
                .as_deref()
                .map(|e| format!(" -e {e}"))
                .unwrap_or_default(),
            standalone.task.as_deref().unwrap_or("<task>"),
        );
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return format!("`standalone/{row}` is unreadable");
    };
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let samples = value
        .get("samples")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let consumed: u64 = samples
        .iter()
        .filter_map(|s| s.get("n_consumed").and_then(Value::as_u64))
        .sum();
    let total: u64 = samples
        .iter()
        .filter_map(|s| s.get("n_total").and_then(Value::as_u64))
        .sum();
    let control = value
        .get("negative_control")
        .and_then(|c| c.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    format!(
        "{} — {consumed}/{total} events consumed over {} sample(s){}, negative control {control}{}",
        if status == "pass" { "✅" } else { "❌" },
        samples.len(),
        value
            .get("pythia_version")
            .and_then(Value::as_str)
            .map(|v| format!(" (Pythia {v})"))
            .unwrap_or_default(),
        if recorded_with_this_run(&path, report_dir) {
            String::new()
        } else {
            format!(
                " (recorded before this invocation's gates; `pixi run{} {}` refreshes it)",
                standalone
                    .environment
                    .as_deref()
                    .map(|e| format!(" -e {e}"))
                    .unwrap_or_default(),
                standalone.task.as_deref().unwrap_or("<task>"),
            )
        }
    )
}

/// Whether a standalone gate's verdict is at least as new as the newest per-row
/// measurement. A gate with its own task can have been run long ago, and a
/// number carried forward from then should not read as this run's.
fn recorded_with_this_run(path: &Path, report_dir: &Path) -> bool {
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let Some(own) = modified(path) else {
        return false;
    };
    let newest_cell = CATEGORIES
        .iter()
        .filter_map(|c| std::fs::read_dir(report_dir.join(c.as_str())).ok())
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| modified(&e.path()))
        .max();
    match newest_cell {
        Some(newest) => own >= newest,
        None => own >= SystemTime::UNIX_EPOCH,
    }
}

pub fn json(
    manifest: &Manifest,
    resolved: &[ResolvedRow],
    report_dir: &Path,
    problems: &[String],
) -> String {
    let rows: Vec<Value> = resolved
        .iter()
        .map(|row| {
            let cells: serde_json::Map<String, Value> = row
                .cells
                .iter()
                .map(|cell| {
                    (
                        cell.category.as_str().to_string(),
                        json!({
                            "tier": cell.tier.as_str(),
                            "mode": cell.mode.map(|m| m.as_str()),
                            "mark": cell.mark,
                            "metric": cell.body,
                            "note": cell.note,
                            "detail": cell.detail,
                            "sources": cell.sources,
                            "durations_s": cell
                                .durations
                                .iter()
                                .map(|(label, seconds)| json!({"measurement": label, "seconds": seconds}))
                                .collect::<Vec<_>>(),
                        }),
                    )
                })
                .collect();
            json!({
                "key": row.process.key,
                "process": row.process.process,
                "class": row.process.class,
                "n_final": row.process.n_final,
                "rationale": row.process.rationale,
                "bundled": row.process.bundled,
                "cells": cells,
            })
        })
        .collect();
    let standalone: Vec<Value> = manifest
        .standalone
        .iter()
        .map(|s| {
            json!({
                "key": s.key,
                "layer": s.layer,
                "targets": s.targets,
                "task": s.task,
                "environment": s.environment,
                "row": s.row,
                "verdict": standalone_verdict(s, report_dir),
                "rationale": s.rationale,
                "note": s.note,
            })
        })
        .collect();
    let value = json!({
        "schema": 1,
        "manifest_schema": manifest.schema,
        "refdata": {
            "version": manifest.refdata.version,
            "archive": manifest.refdata.archive,
            "url": manifest.refdata.url,
            "sha256": manifest.refdata.sha256,
            "size_bytes": manifest.refdata.size_bytes,
            "published": manifest.refdata.published,
        },
        "rows": rows,
        "standalone": standalone,
        "problems": problems,
        "ok": problems.is_empty(),
    });
    serde_json::to_string_pretty(&value).expect("the report serialises") + "\n"
}
