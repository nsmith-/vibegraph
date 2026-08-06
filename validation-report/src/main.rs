//! The validation report collator.
//!
//! Reads every row file the validation gates wrote under
//! `target/validation-report/`, renders the per-process × per-category table
//! (`report.md`) and its machine form (`report.json`), and asserts that what was
//! measured is exactly what `validation/manifest.toml` declares. It exits
//! nonzero on a missing cell, an unexpected cell, a cell whose gate failed, or a
//! disagreement between the manifest's declaration and the measurement's own
//! account of itself.
//!
//! # Where a cell's mark comes from
//!
//! Every cell in the table is decided by the manifest's tier for it, and — where
//! that tier says the cell is measured in a layer this run drives — by the row
//! file the gate wrote:
//!
//! | tier | expects a row file | mark |
//! |---|---|---|
//! | `hermetic`, `banked` | yes | ✅ when the gate passed, ⚠️ when the cell is informational, ❌ when it failed |
//! | `long` | only when its own driver ran | ⏳ without one; otherwise rendered from it like a banked cell |
//! | `blocked` | no | ⛔, naming the blocker |
//! | `covered-by` | no | `—`, naming the rows that cover it |
//! | `uncovered` | no | `uncovered`, with the manifest's account of the gap |
//!
//! A row file for a cell in one of the last three tiers is an *unexpected* cell
//! and fails: it means something was measured that the manifest says is not.
//!
//! **A `long` cell is measured by a task of its own**, not by `pixi run validate`
//! — that is the whole content of the tier, which is about cost and not about
//! dependencies. So its cell is ⏳ in a bare banked run and a rendered
//! measurement in a run whose driver went first. Nothing is inferred either way:
//! `validate.sh` clears the per-category row files before the gates run, so a
//! long cell reads as measured only when its driver ran in the same cycle as the
//! collation, and reverts to ⏳ the moment it does not.
//!
//! **Hermetic cells are rendered from measurements, not inferred.** The
//! alternative — taking a hermetic cell's mark from the manifest tier plus the
//! fact that the hermetic suite passed — would print a green cell that no
//! recorded measurement stands behind, and would keep printing it after the gate
//! stopped covering the row. So the two hermetic-tier categories write row files
//! of their own (`amplitude_oracle` and `validate_madgraph_diagrams`, through the
//! same writer the other two categories use), and a hermetic cell with no row
//! file is a missing cell like any other. Nothing here infers a cell.
//!
//! # A row measured more than once
//!
//! One process can carry several measurements of the same category — `pp_to_ll`
//! is integrated on two run cards, and arrives as `pp_to_ll__default` and
//! `pp_to_ll__mmll_60_120`. The cell reports the **worst** of them (the largest
//! `|pull|`, the smallest p-value, the largest deviation) and lists every
//! measurement's own value beside it, so the cell cannot be made green by adding
//! an easier variant.
//!
//! # The standalone gates
//!
//! The gates that belong to no row are listed under the table. Those whose driver
//! is a Rust test carry no row file: they ran inside the same `cargo test`
//! invocation as the cells, so a failure in one of them fails the suite before
//! this collator is reached — the list records what they are, not a fresh
//! verdict. The one gate with a driver of its own (Pythia consumption, its own
//! pixi environment) writes `standalone/pythia_consumption.json`, which is
//! rendered when present. It runs under a separate task, so its absence reads as
//! "not run in this invocation" and is not a failure; nothing in the manifest
//! marks a standalone gate as required here.

mod cells;
mod manifest;
mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::cells::RowFile;
use crate::manifest::{Category, Manifest, Mode, Process, Tier, CATEGORIES};

/// One cell of the rendered table.
pub struct ResolvedCell {
    pub category: Category,
    pub tier: Tier,
    pub mode: Option<Mode>,
    /// The glyph the table shows, and the metric or reason beside it.
    pub mark: &'static str,
    pub body: String,
    /// Why this cell is not a plain green measurement, for the note list under
    /// the table.
    pub note: Option<String>,
    /// What the manifest says about a cell that needs no note — the run-card cut
    /// that regulates a divergence, the point of a row. Reported with the
    /// measurements rather than in the table.
    pub context: Option<String>,
    /// Assigned in table order to the cells that carry a note.
    pub note_ref: Option<usize>,
    /// The measurements behind the cell, worst first.
    pub detail: Vec<String>,
    pub sources: Vec<String>,
    /// What each of those measurements cost in wall-clock seconds, labelled the
    /// way the cell labels them. Only the ones whose gate timed itself appear.
    pub durations: Vec<(String, f64)>,
}

pub struct ResolvedRow<'a> {
    pub process: &'a Process,
    pub cells: Vec<ResolvedCell>,
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("the crate directory has a parent");
    let manifest_path = repo_root.join("validation/manifest.toml");
    let manifest = match Manifest::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("!!! {e}");
            std::process::exit(1);
        }
    };
    let report_dir = report_dir(&repo_root);

    let (rows, mut problems) = cells::load_all(&report_dir);
    check_standalone_layers(&manifest, &mut problems);
    let mut resolved = resolve(&manifest, &rows, &mut problems);
    number_notes(&mut resolved);

    let markdown = render::markdown(&manifest, &resolved, &report_dir, &problems);
    let json = render::json(&manifest, &resolved, &report_dir, &problems);

    std::fs::create_dir_all(&report_dir).expect("the report directory");
    let md_path = report_dir.join("report.md");
    let json_path = report_dir.join("report.json");
    std::fs::write(&md_path, &markdown).expect("write report.md");
    std::fs::write(&json_path, json).expect("write report.json");

    print!("{markdown}");
    eprintln!("[report] wrote {}", md_path.display());
    eprintln!("[report] wrote {}", json_path.display());

    if problems.is_empty() {
        eprintln!(
            "[report] {} rows x {} categories: the measured cells are the declared cells",
            manifest.processes.len(),
            CATEGORIES.len()
        );
        return;
    }
    eprintln!("!!! the report does not match {}:", manifest_path.display());
    for problem in &problems {
        eprintln!("    - {problem}");
    }
    std::process::exit(1);
}

/// `<target>/validation-report`, honouring `CARGO_TARGET_DIR` so a run with a
/// relocated target directory reads where the gates wrote.
fn report_dir(repo_root: &std::path::Path) -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => repo_root.join("target"),
    }
    .join("validation-report")
}

/// `Standalone::layer` is a free `String`, so a typo would otherwise render
/// silently as "ran with the {typo} layer's suite" instead of failing. The
/// layer set is a declaration like any other in this manifest and is enforced
/// the same way: every standalone row's layer must be one the report
/// understands, and a row declaring `oracle` (which has no row file of its
/// own) must carry a `task` or `standalone_verdict` has nothing to name.
fn check_standalone_layers(manifest: &Manifest, problems: &mut Vec<String>) {
    for standalone in &manifest.standalone {
        if !matches!(standalone.layer.as_str(), "hermetic" | "banked" | "oracle") {
            problems.push(format!(
                "standalone '{}' declares layer '{}', which is none of hermetic/banked/oracle",
                standalone.key, standalone.layer
            ));
        }
        if standalone.layer == "oracle" && standalone.task.is_none() {
            problems.push(format!(
                "standalone '{}' declares layer 'oracle' but names no task",
                standalone.key
            ));
        }
    }
}

/// Match every declared cell against what was measured, collecting the
/// disagreements as they are found.
fn resolve<'a>(
    manifest: &'a Manifest,
    rows: &'a [RowFile],
    problems: &mut Vec<String>,
) -> Vec<ResolvedRow<'a>> {
    let known: BTreeSet<&str> = manifest.by_key().keys().copied().collect();
    let mut by_cell: BTreeMap<(&str, Category), Vec<&RowFile>> = BTreeMap::new();
    for row in rows {
        if !known.contains(row.row.as_str()) {
            problems.push(format!(
                "{} measures the row '{}', which no [[process]] of the manifest declares",
                row.path.display(),
                row.row
            ));
            continue;
        }
        by_cell
            .entry((row.row.as_str(), row.category))
            .or_default()
            .push(row);
    }

    let mut resolved = Vec::new();
    for process in manifest.ordered_rows() {
        let mut cells = Vec::new();
        for category in CATEGORIES {
            let declared = process.categories.get(category);
            let mut measured = by_cell
                .remove(&(process.key.as_str(), category))
                .unwrap_or_default();
            measured.sort_by(|a, b| b.severity().total_cmp(&a.severity()));
            cells.push(resolve_cell(
                process, category, declared, &measured, problems,
            ));
        }
        resolved.push(ResolvedRow { process, cells });
    }
    resolved
}

fn resolve_cell(
    process: &Process,
    category: Category,
    declared: &manifest::Cell,
    measured: &[&RowFile],
    problems: &mut Vec<String>,
) -> ResolvedCell {
    let where_ = format!("{} / {}", process.key, category.as_str());
    let mut cell = ResolvedCell {
        category,
        tier: declared.tier,
        mode: declared.mode,
        mark: "",
        body: String::new(),
        note: declared.note.clone(),
        context: None,
        note_ref: None,
        detail: Vec::new(),
        sources: measured
            .iter()
            .map(|m| m.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect(),
        durations: measured
            .iter()
            .filter_map(|m| m.duration_s.map(|s| (m.label().to_string(), s)))
            .collect(),
    };

    // A `long` cell's driver is a task of its own, so whether it has a row file
    // says whether that task ran in this cycle — not whether the cell was declared
    // wrongly. With one, it is rendered from the measurement below like any other
    // gate cell; without one it waits, the way it did before its driver existed.
    let driven = declared.tier == Tier::Long && !measured.is_empty();
    if !declared.tier.is_measured_here() && !driven {
        if !measured.is_empty() {
            problems.push(format!(
                "{where_} is declared '{}' and yet {} row file(s) measured it",
                declared.tier.as_str(),
                measured.len()
            ));
        }
        match declared.tier {
            Tier::Long => {
                cell.mark = "⏳";
                cell.body = "oracle layer".to_string();
            }
            Tier::Blocked => {
                let blocker = declared.blocker.clone().unwrap_or_else(|| {
                    problems.push(format!("{where_} is blocked and names no blocker"));
                    "unnamed".to_string()
                });
                cell.mark = "⛔";
                cell.body = format!("`{blocker}`");
            }
            Tier::CoveredBy => {
                if declared.rows.is_empty() {
                    problems.push(format!("{where_} is covered-by and names no rows"));
                }
                cell.mark = "—";
                cell.body = format!(
                    "covered by {}",
                    declared
                        .rows
                        .iter()
                        .map(|r| format!("`{r}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                // The pointer is in the cell; a note would repeat it.
                cell.note = None;
            }
            Tier::Uncovered => {
                cell.mark = "";
                cell.body = "uncovered".to_string();
                if declared.note.is_none() {
                    problems.push(format!(
                        "{where_} is uncovered and says nothing about what would fill it"
                    ));
                }
            }
            Tier::Hermetic | Tier::Banked => unreachable!("measured tiers are handled below"),
        }
        return cell;
    }

    let Some(mode) = declared.mode else {
        problems.push(format!(
            "{where_} is tier '{}' and declares no mode",
            declared.tier.as_str()
        ));
        cell.mark = "❌";
        cell.body = "no mode declared".to_string();
        return cell;
    };

    if measured.is_empty() {
        // A row the pinned bundle does not carry is *declared* absent from a
        // fetching checkout, so its gate legitimately found no run to measure and
        // wrote no cell. That is a complete environment with respect to what the
        // bundle promises, not a missing measurement — the cell waits on the next
        // re-cut the way an oracle-layer cell waits on its driver. A row the bundle
        // does carry has no such excuse.
        if !process.bundled {
            cell.mark = "⏳";
            cell.body = "awaiting the bundle".to_string();
            return cell;
        }
        problems.push(format!(
            "{where_} is declared '{}' and no gate wrote it — the cell is missing, not empty",
            declared.tier.as_str()
        ));
        cell.mark = "❌";
        cell.body = "not measured".to_string();
        return cell;
    }

    for m in measured {
        if m.mode != mode.as_str() {
            problems.push(format!(
                "{where_}: the manifest declares mode '{}' and {} was written as '{}'",
                mode.as_str(),
                m.path.display(),
                m.mode
            ));
        }
        let consistent = match m.status.as_str() {
            "pass" => mode == Mode::Gate,
            "info" => mode == Mode::Info,
            "fail" => true,
            _ => false,
        };
        if !consistent {
            problems.push(format!(
                "{where_}: {} reports status '{}' under mode '{}'",
                m.path.display(),
                m.status,
                mode.as_str()
            ));
        }
        if m.status == "fail" {
            problems.push(format!(
                "{where_}: the gate failed{}",
                m.note
                    .as_deref()
                    .map(|n| format!(" — {n}"))
                    .unwrap_or_default()
            ));
        }
        if let (Some(claimed), Some(observed)) = (declared.factorized, m.bool_at("factorized")) {
            if claimed != observed {
                problems.push(format!(
                    "{where_}: the manifest states factorized = {claimed} and the gate measured {observed}"
                ));
            }
        }
        // A measurement may specialise the row's process — the two Drell-Yan run
        // cards integrate `p p > e+ e-` where the row is `p p > l+ l-` — so the
        // detail names what was measured wherever the two differ.
        let label = if m.process.is_empty() || m.process == process.process {
            m.label().to_string()
        } else {
            format!("{} (`{}`)", m.label(), m.process)
        };
        match m.detail() {
            Ok(detail) => cell.detail.push(format!("{label}: {detail}")),
            Err(e) => problems.push(e),
        }
    }

    let worst = measured[0];
    cell.mark = match worst.status.as_str() {
        "fail" => "❌",
        _ if mode == Mode::Info => "⚠️",
        _ => "✅",
    };
    cell.body = match worst.metric() {
        Ok(metric) => metric,
        Err(e) => {
            problems.push(e);
            "unreadable".to_string()
        }
    };
    if measured.len() > 1 {
        let all: Vec<String> = measured
            .iter()
            .map(|m| match m.short_metric() {
                Ok(v) => format!("{} {v}", m.label()),
                Err(_) => format!("{} unreadable", m.label()),
            })
            .collect();
        cell.body = format!(
            "{} (worst of {}: {})",
            cell.body,
            measured.len(),
            all.join("; ")
        );
    }
    // The manifest's account of a discrepancy is the curated one and wins; a
    // measurement's own note stands in only where the manifest says nothing.
    if mode == Mode::Info || worst.status == "fail" {
        if cell.note.is_none() {
            cell.note = worst.note.clone();
        }
        if cell.note.is_none() {
            problems.push(format!(
                "{where_} is informational and names no discrepancy"
            ));
        }
    } else {
        // A green cell's manifest note is background, not a finding.
        cell.context = cell.note.take();
    }
    cell
}

/// Number the cells that carry a note, in table order, so the table can point at
/// the list beneath it. Cells sharing a note text share its number — four llj
/// parton rows blocked for one reason are one note, not four copies of it.
fn number_notes(resolved: &mut [ResolvedRow]) {
    let mut numbers: BTreeMap<String, usize> = BTreeMap::new();
    for row in resolved.iter_mut() {
        for cell in row.cells.iter_mut() {
            if let Some(note) = cell.note.clone() {
                let next = numbers.len() + 1;
                cell.note_ref = Some(*numbers.entry(note).or_insert(next));
            }
        }
    }
}
