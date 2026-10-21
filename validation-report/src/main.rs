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
//! verdict. The gates with a driver of their own in the Pythia environment (event
//! consumption, and the matched MLM comparison through the shower) write
//! `standalone/pythia_consumption.json` and `standalone/mlm_pythia.json`, which
//! are rendered when present. They run under separate tasks, so an absent file
//! reads as "not run in this invocation" and is not a failure; nothing in the
//! manifest marks a standalone gate as required here. A standalone gate that
//! declares a `mode` must have written the same mode into its file.

mod cells;
mod manifest;
mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::cells::{RowFile, Status};
use crate::manifest::{Category, Layer, Manifest, Mode, Process, ProcessStatus, Tier, CATEGORIES};

/// One cell of the rendered table.
pub(crate) struct ResolvedCell {
    pub(crate) category: Category,
    pub(crate) tier: Tier,
    pub(crate) mode: Option<Mode>,
    /// The glyph the table shows, and the metric or reason beside it.
    pub(crate) mark: &'static str,
    pub(crate) body: String,
    /// Why this cell is not a plain green measurement, for the note list under
    /// the table.
    pub(crate) note: Option<String>,
    /// What the manifest says about a cell that needs no note — the run-card cut
    /// that regulates a divergence, the point of a row. Reported with the
    /// measurements rather than in the table.
    pub(crate) context: Option<String>,
    /// Assigned in table order to the cells that carry a note.
    pub(crate) note_ref: Option<usize>,
    /// The measurements behind the cell, worst first.
    pub(crate) detail: Vec<String>,
    pub(crate) sources: Vec<String>,
    /// What each of those measurements cost in wall-clock seconds, labelled the
    /// way the cell labels them. Only the ones whose gate timed itself appear.
    pub(crate) durations: Vec<(String, f64)>,
}

pub(crate) struct ResolvedRow<'a> {
    pub(crate) process: &'a Process,
    pub(crate) cells: Vec<ResolvedCell>,
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
    check_standalone_tasks(&manifest, &mut problems);
    check_standalone_modes(&manifest.standalone, &report_dir, &mut problems);
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

/// A standalone row declaring `oracle` has no row file of its own, so it must
/// carry a `task` or `standalone_verdict` has nothing to name.
fn check_standalone_tasks(manifest: &Manifest, problems: &mut Vec<String>) {
    for standalone in &manifest.standalone {
        if standalone.layer == Layer::Oracle && standalone.task.is_none() {
            problems.push(format!(
                "standalone '{}' declares layer 'oracle' but names no task",
                standalone.key
            ));
        }
    }
}

/// A standalone gate that declares a `mode` and whose row file is present must
/// have written that mode: a driver reporting `info` cannot stand in for a gate
/// the manifest enforces, nor the other way round. An absent row file is "not
/// run in this invocation", as for any standalone gate with its own task.
fn check_standalone_modes(
    standalones: &[manifest::Standalone],
    report_dir: &std::path::Path,
    problems: &mut Vec<String>,
) {
    for standalone in standalones {
        let (Some(row), Some(mode)) = (standalone.row.as_deref(), standalone.mode) else {
            continue;
        };
        let path = report_dir.join("standalone").join(row);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let written = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("mode").and_then(|m| m.as_str()).map(str::to_string));
        if written.as_deref() != Some(mode.as_str()) {
            problems.push(format!(
                "standalone '{}' is declared mode '{}' but {} records mode {}",
                standalone.key,
                mode.as_str(),
                path.display(),
                written.map_or("(none)".to_string(), |m| format!("'{m}'")),
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
    // gate cell; without one it waits as ⏳.
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
        // A planned row's reference run does not exist yet, so its gates have
        // nothing to measure and write no cell. The cell waits for the run; once
        // the run is banked and the gate writes a cell it is rendered from it
        // whatever the status says.
        if process.status == Some(ProcessStatus::Planned) {
            cell.mark = "⏳";
            cell.body = "planned: no reference run yet".to_string();
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
        if m.mode != mode {
            problems.push(format!(
                "{where_}: the manifest declares mode '{}' and {} was written as '{}'",
                mode.as_str(),
                m.path.display(),
                m.mode.as_str()
            ));
        }
        let consistent = match m.status {
            Status::Pass => mode == Mode::Gate,
            Status::Info => mode == Mode::Info,
            Status::Fail => true,
        };
        if !consistent {
            problems.push(format!(
                "{where_}: {} reports status '{}' under mode '{}'",
                m.path.display(),
                m.status.as_str(),
                mode.as_str()
            ));
        }
        if m.status == Status::Fail {
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
    cell.mark = match worst.status {
        Status::Fail => "❌",
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
    if mode == Mode::Info || worst.status == Status::Fail {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The unequal-beam row is declared `planned` with banked cells and no gate
    /// has written one: the collator waits on it without a problem. The same
    /// row declared as an ordinary row is a missing cell.
    #[test]
    fn a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/manifest.toml");
        let text = std::fs::read_to_string(path).expect("the committed manifest");
        let resolve = |text: &str| {
            let manifest = toml::from_str::<Manifest>(text).unwrap();
            let process = manifest
                .processes
                .iter()
                .find(|p| p.key == "pp_to_ll_beams_6500_4000")
                .expect("the unequal-beam row");
            let declared = process.categories.get(Category::Integrals);
            let mut problems = Vec::new();
            let cell = resolve_cell(process, Category::Integrals, declared, &[], &mut problems);
            (cell.mark, problems)
        };
        let (mark, problems) = resolve(&text);
        assert_eq!((mark, problems.len()), ("⏳", 0), "{problems:?}");
        let (mark, problems) = resolve(&text.replace("status = \"planned\"\n", ""));
        assert_eq!((mark, problems.len()), ("❌", 1), "{problems:?}");
    }

    fn standalone(row: Option<&str>, mode: Option<&str>) -> manifest::Standalone {
        let mut text =
            String::from("key = \"gate-under-test\"\nlayer = \"oracle\"\nrationale = \"r\"\n");
        if let Some(row) = row {
            text += &format!("row = \"{row}\"\n");
        }
        if let Some(mode) = mode {
            text += &format!("mode = \"{mode}\"\n");
        }
        toml::from_str(&text).expect("a standalone entry")
    }

    /// The problems `check_standalone_modes` reports for one entry whose row
    /// file (when `written` is given) holds that text.
    fn problems_for(entry: manifest::Standalone, written: Option<&str>) -> Vec<String> {
        let dir = tempfile::tempdir().expect("a scratch directory");
        std::fs::create_dir(dir.path().join("standalone")).unwrap();
        if let Some(text) = written {
            std::fs::write(dir.path().join("standalone/row.json"), text).unwrap();
        }
        let mut problems = Vec::new();
        check_standalone_modes(&[entry], dir.path(), &mut problems);
        problems
    }

    #[test]
    fn a_row_recording_the_declared_mode_is_clean() {
        for mode in ["gate", "info"] {
            let written = format!("{{\"mode\": \"{mode}\"}}");
            assert!(
                problems_for(standalone(Some("row.json"), Some(mode)), Some(&written)).is_empty()
            );
        }
    }

    /// A driver recording `info` for a declared gate (and the converse) is named,
    /// with both modes and the row file.
    #[test]
    fn a_row_recording_the_other_mode_is_reported() {
        for (declared, written) in [("gate", "info"), ("info", "gate")] {
            let text = format!("{{\"mode\": \"{written}\"}}");
            let problems = problems_for(standalone(Some("row.json"), Some(declared)), Some(&text));
            assert_eq!(problems.len(), 1, "{problems:?}");
            assert!(
                problems[0].starts_with("standalone 'gate-under-test' "),
                "{problems:?}"
            );
            assert!(
                problems[0].contains(&format!("is declared mode '{declared}' but ")),
                "{problems:?}"
            );
            assert!(
                problems[0].ends_with(&format!("row.json records mode '{written}'")),
                "{problems:?}"
            );
        }
    }

    /// A row with no mode, or one that is not JSON, cannot stand in for a declared mode.
    #[test]
    fn a_row_without_a_readable_mode_is_reported() {
        for text in ["{\"status\": \"pass\"}", "not json", "{\"mode\": 3}"] {
            let problems = problems_for(standalone(Some("row.json"), Some("gate")), Some(text));
            assert_eq!(problems.len(), 1, "{text}: {problems:?}");
            assert!(
                problems[0].contains("records mode (none)"),
                "{text}: {problems:?}"
            );
        }
    }

    /// An absent row file is "not run in this invocation", and an entry that
    /// declares no mode (or no row) is not checked, whatever its file says.
    #[test]
    fn absent_rows_and_undeclared_modes_are_left_alone() {
        assert!(problems_for(standalone(Some("row.json"), Some("gate")), None).is_empty());
        for (row, mode) in [(Some("row.json"), None), (None, Some("gate"))] {
            for written in ["{\"mode\": \"info\"}", "{}", "not json"] {
                assert!(
                    problems_for(standalone(row, mode), Some(written)).is_empty(),
                    "{written}"
                );
            }
        }
    }
}
