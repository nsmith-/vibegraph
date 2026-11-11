//! `vibegraph enumerate`: a proc card's diagrams, as the diagram artifact, a
//! summary JSON and one SVG drawing per diagram.
//!
//! The summary's schema is documented with the knowledge bundle
//! (`research/kb/pipeline/diagrams-json.md`); [`SCHEMA_VERSION`] is its version.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Args;
use serde::Serialize;
use tracing::info;
use vibegraph::config::GlobalConfig;
use vibegraph::diagrams::artifact::DiagramArtifact;
use vibegraph::diagrams::diagram::OnShell;
use vibegraph::diagrams::draw::DiagramDrawer;
use vibegraph::diagrams::{DiagramSet, ParsingOptions};
use vibegraph::helas::eval::config_groups;
use vibegraph::multiplicity::multiplicity_groups;
use vibegraph::runcard::process_defaults::ProcessFacts;
use vibegraph::runcard::BeamMode;
use vibegraph::ufo::UFOModel;

use crate::assets;
use crate::error::{err, CliError, ErrorKind};
use crate::integrate::{bind_param_card, load_run_card};
use crate::output::{self, ModelRef};
use crate::parallel::ParallelArgs;

/// The version of the `diagrams.json` schema.
///
/// Adding a field keeps it; removing, renaming or re-meaning one bumps it.
pub(crate) const SCHEMA_VERSION: u32 = 1;

const ARTIFACT_FILENAME: &str = "diagrams.bin.zst";
pub(crate) const SUMMARY_FILENAME: &str = "diagrams.json";
const SVG_DIRNAME: &str = "svg";

/// Per-set cap on drawings when `--svg` is not given. Every set of the cards
/// whose enumeration cost has been measured (up to `g g > g g g g`, 220
/// diagrams, and `p p > e+ e- j j j`, 6,400 diagrams over 96 sets) is drawn in
/// full under it; a set past it (SMEFTsim `g g > t t~ g g NP<=1`, 2,387) is
/// drawn up to it, and `--svg all` draws the rest.
const DEFAULT_SVG_PER_SET: usize = 500;

/// Which diagrams `enumerate` draws.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SvgSelection {
    All,
    None,
    /// The first `N` diagrams of every set.
    First(usize),
}

impl SvgSelection {
    fn drawn(self, n: usize) -> usize {
        match self {
            SvgSelection::All => n,
            SvgSelection::None => 0,
            SvgSelection::First(cap) => n.min(cap),
        }
    }
}

impl FromStr for SvgSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "all" => Ok(SvgSelection::All),
            "none" => Ok(SvgSelection::None),
            _ => s
                .strip_prefix("first=")
                .and_then(|n| n.parse().ok())
                .map(SvgSelection::First)
                .ok_or_else(|| format!("expected all, none or first=N, not '{s}'")),
        }
    }
}

impl std::fmt::Display for SvgSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SvgSelection::All => f.write_str("all"),
            SvgSelection::None => f.write_str("none"),
            SvgSelection::First(n) => write!(f, "first={n}"),
        }
    }
}

#[derive(Args, Debug)]
pub(crate) struct EnumerateArgs {
    /// Process card selecting the model and process (`import model` +
    /// `generate`); `-` reads the card from stdin.
    pub(crate) proc_card: PathBuf,

    /// Output directory for `diagrams.bin.zst`, `diagrams.json` and `svg/`.
    #[arg(long, default_value = ".")]
    pub(crate) out: PathBuf,

    /// Overwrite the outputs of an earlier run in `--out`. The drawings it wrote
    /// under `svg/` are removed first, so none is left from the earlier run.
    #[arg(long)]
    pub(crate) force: bool,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long)]
    pub(crate) ufo_dir: Option<PathBuf>,

    /// Which diagrams to draw, as `svg/<set>/<diagram>.svg`: `all`, `none`, or
    /// `first=N` per set. `diagrams.json` records which were drawn.
    #[arg(long, value_name = "all|none|first=N", default_value_t = SvgSelection::First(DEFAULT_SVG_PER_SET))]
    pub(crate) svg: SvgSelection,

    /// MadGraph `run_card.dat` for a preview of the flavour groups `integrate`
    /// would form; the param card not given takes `integrate`'s default. With
    /// neither card, no flavour groups are reported.
    #[arg(long)]
    pub(crate) run_card: Option<PathBuf>,

    /// SLHA `param_card.dat` for the flavour-group preview; the run card not
    /// given takes `integrate`'s default.
    #[arg(long)]
    pub(crate) param_card: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) parallel: ParallelArgs,
}

/// `diagrams.json`.
#[derive(Debug, Serialize)]
pub(crate) struct Summary {
    schema_version: u32,
    process: String,
    model: ModelRef,
    diagram_set_digest: String,
    /// Present only when a run or param card was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    flavour_groups: Option<FlavourPreview>,
    sets: Vec<SetSummary>,
}

#[derive(Debug, Serialize)]
struct SetSummary {
    id: String,
    label: String,
    particles_in: Vec<String>,
    particles_out: Vec<String>,
    polarizations: Vec<Option<Vec<i32>>>,
    mirrored: bool,
    diagrams: usize,
    coupling_orders: Vec<OrderCount>,
    configurations: Vec<Configuration>,
    contact: Vec<String>,
    /// Present only when [`Summary::flavour_groups`] is.
    #[serde(skip_serializing_if = "Option::is_none")]
    flavour_group: Option<Option<String>>,
    diagram_list: Vec<DiagramSummary>,
}

#[derive(Debug, Serialize)]
struct OrderCount {
    orders: BTreeMap<String, usize>,
    diagrams: usize,
}

#[derive(Debug, Serialize)]
struct Configuration {
    id: String,
    diagrams: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DiagramSummary {
    id: String,
    propagators: Vec<PropagatorSummary>,
    vertices: Vec<String>,
    coupling_orders: BTreeMap<String, usize>,
    configuration: Option<String>,
    svg: Option<String>,
}

#[derive(Debug, Serialize)]
struct PropagatorSummary {
    particle: String,
    forced_onshell: bool,
}

/// The `--output-ldjson` result object: where the outputs were written, and the
/// summary itself. The summary is nested rather than flattened so that its own
/// `schema_version` does not collide with the stream's.
#[derive(Debug, Serialize)]
struct EnumerateResult<'a> {
    artifact: String,
    summary_path: String,
    /// `null` when nothing was drawn.
    svg_dir: Option<String>,
    drawings: usize,
    summary: &'a Summary,
}

/// The flavour groups `integrate` would form under the given cards.
#[derive(Debug, Serialize)]
struct FlavourPreview {
    /// The run card given, or `null` for the default `integrate` builds.
    run_card: Option<String>,
    /// The param card given, or `null` for the model's defaults.
    param_card: Option<String>,
    /// `proton`, `fixed_energy` or `decay`. Only proton beams group flavours;
    /// the other two integrate every set together and report no group.
    beams: &'static str,
    groups: Vec<FlavourGroupSummary>,
}

#[derive(Debug, Serialize)]
struct FlavourGroupSummary {
    id: String,
    /// The phase space (final-state multiplicity and outgoing masses) the
    /// group is integrated in, in `integrate`'s order.
    phase_space: usize,
    representative: String,
    members: Vec<String>,
}

/// A set's id: `s<k>`, `k` its index in enumeration order.
pub(crate) fn set_id(k: usize) -> String {
    format!("s{k}")
}

/// A diagram's id: `s<k>.d<j>`, `j` its index within set `k`.
pub(crate) fn diagram_id(k: usize, j: usize) -> String {
    format!("s{k}.d{j}")
}

/// A configuration's id within its set: `c<j>`, `j` its index among the set's
/// [`config_groups`].
pub(crate) fn configuration_id(c: usize) -> String {
    format!("c{c}")
}

/// `svg/<set>/<diagram>.svg`, relative to the output directory.
pub(crate) fn svg_path(k: usize, j: usize) -> String {
    format!("{SVG_DIRNAME}/{}/{}.svg", set_id(k), diagram_id(k, j))
}

/// Whether `name` is a file name this command writes under `svg/<set>/`.
fn is_drawing_name(set: &str, name: &str) -> bool {
    name.strip_prefix(set)
        .and_then(|rest| rest.strip_prefix(".d"))
        .and_then(|rest| rest.strip_suffix(".svg"))
        .is_some_and(|j| !j.is_empty() && j.bytes().all(|b| b.is_ascii_digit()))
}

fn is_set_name(name: &str) -> bool {
    name.strip_prefix('s')
        .is_some_and(|k| !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit()))
}

fn io_error(what: &str, path: &Path, e: std::io::Error) -> CliError {
    err(
        ErrorKind::Io,
        format!("cannot {what} {}: {e}", path.display()),
    )
}

/// Remove the drawings an earlier run wrote under `svg/`: every
/// `svg/s<k>/s<k>.d<j>.svg`, then each `svg/s<k>/` and `svg/` itself if that
/// leaves them empty. Nothing else is touched.
fn remove_earlier_drawings(svg: &Path) -> Result<(), CliError> {
    if !svg.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(svg).map_err(|e| io_error("read", svg, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| io_error("read", svg, e))?;
        let set = entry.file_name().to_string_lossy().into_owned();
        let dir = entry.path();
        if !is_set_name(&set) || !dir.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&dir).map_err(|e| io_error("read", &dir, e))? {
            let file = file.map_err(|e| io_error("read", &dir, e))?;
            if is_drawing_name(&set, &file.file_name().to_string_lossy()) {
                let path = file.path();
                std::fs::remove_file(&path).map_err(|e| io_error("remove", &path, e))?;
            }
        }
        // Left in place when something not written here remains in it.
        let _ = std::fs::remove_dir(&dir);
    }
    let _ = std::fs::remove_dir(svg);
    Ok(())
}

/// The coupling-order histogram of `set`: each distinct order map, with how many
/// diagrams carry it, in the maps' order.
fn order_histogram(orders: &[BTreeMap<String, usize>]) -> Vec<OrderCount> {
    let mut counts: BTreeMap<&BTreeMap<String, usize>, usize> = BTreeMap::new();
    for o in orders {
        *counts.entry(o).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(orders, diagrams)| OrderCount {
            orders: orders.clone(),
            diagrams,
        })
        .collect()
}

fn summarize_set(
    k: usize,
    set: &DiagramSet,
    model: &UFOModel,
    svg: SvgSelection,
    flavour_group: Option<Option<String>>,
) -> SetSummary {
    let groups = config_groups(&set.diagrams, model);
    let mut configuration_of: Vec<Option<usize>> = vec![None; set.diagrams.len()];
    for (c, group) in groups.iter().enumerate() {
        for &d in group {
            configuration_of[d] = Some(c);
        }
    }
    let drawn = svg.drawn(set.diagrams.len());
    let orders: Vec<_> = set
        .diagrams
        .iter()
        .map(|d| d.coupling_orders(model))
        .collect();
    let diagram_list = set
        .diagrams
        .iter()
        .zip(&orders)
        .enumerate()
        .map(|(j, (d, orders))| DiagramSummary {
            id: diagram_id(k, j),
            propagators: d
                .props
                .iter()
                .map(|p| PropagatorSummary {
                    particle: model.particle(p.particle).name.clone(),
                    forced_onshell: p.onshell == OnShell::Forced,
                })
                .collect(),
            vertices: d
                .vertices
                .iter()
                .map(|v| v.interaction_name(model).to_string())
                .collect(),
            coupling_orders: orders.clone(),
            configuration: configuration_of[j].map(configuration_id),
            svg: (j < drawn).then(|| svg_path(k, j)),
        })
        .collect();
    SetSummary {
        id: set_id(k),
        label: set.label(),
        particles_in: set.particles_in.clone(),
        particles_out: set.particles_out.clone(),
        polarizations: set.polarizations.clone(),
        mirrored: set.mirrored,
        diagrams: set.diagrams.len(),
        coupling_orders: order_histogram(&orders),
        configurations: groups
            .iter()
            .enumerate()
            .map(|(c, group)| Configuration {
                id: configuration_id(c),
                diagrams: group.iter().map(|&j| diagram_id(k, j)).collect(),
            })
            .collect(),
        contact: (0..set.diagrams.len())
            .filter(|&j| configuration_of[j].is_none())
            .map(|j| diagram_id(k, j))
            .collect(),
        flavour_group,
        diagram_list,
    }
}

/// Whether `member` is `set`: the same card line, beams in order, the same
/// outgoing content with its polarizations whatever their order, the same
/// mirroring and the same number of diagrams.
fn same_subprocess(member: &DiagramSet, set: &DiagramSet) -> bool {
    let content = |s: &DiagramSet| {
        let n_in = s.particles_in.len();
        let mut out: Vec<(String, Option<Vec<i32>>)> = s
            .particles_out
            .iter()
            .cloned()
            .zip(s.polarizations[n_in..].iter().cloned())
            .collect();
        out.sort();
        (
            s.particles_in.clone(),
            s.polarizations[..n_in].to_vec(),
            out,
            s.mirrored,
            s.diagrams.len(),
            s.diagrams.first().map(|d| d.provenance.process),
        )
    };
    content(member) == content(set)
}

/// The flavour-group preview: `integrate`'s run card and param card, defaulted
/// as `integrate` defaults them, and its grouping. Each set is matched to the
/// group member that is the same subprocess; of several sets that are (decay
/// chains reaching one subprocess through different assignments), members take
/// them in enumeration order.
fn flavour_preview(
    args: &EnumerateArgs,
    config: &GlobalConfig,
    parsed: &vibegraph::diagrams::SupportedCard,
    model: &std::sync::Arc<UFOModel>,
    sets: &[DiagramSet],
    facts: Option<&ProcessFacts>,
) -> Result<(FlavourPreview, Vec<Option<String>>), CliError> {
    let card = load_run_card(config, parsed, model, || {
        facts.cloned().ok_or_else(|| {
            err(
                ErrorKind::Process,
                "the enumeration banked no process facts to build the default run card from",
            )
        })
    })?;
    let split = if parsed.is_decay() {
        card.split_decay()
    } else {
        card.split()
    };
    let (rc, _) = split.map_err(|e| err(ErrorKind::Card, e.to_string()))?;
    let (evaluated, _, _) = bind_param_card(args.param_card.as_deref(), model)?;
    let mut preview = FlavourPreview {
        run_card: args.run_card.as_ref().map(|p| p.display().to_string()),
        param_card: args.param_card.as_ref().map(|p| p.display().to_string()),
        beams: "proton",
        groups: Vec::new(),
    };
    let mut group_of = vec![None; sets.len()];
    if parsed.is_decay() {
        preview.beams = "decay";
        return Ok((preview, group_of));
    }
    if rc.beam_mode() == BeamMode::FixedEnergy {
        preview.beams = "fixed_energy";
        return Ok((preview, group_of));
    }
    let groups = multiplicity_groups(parsed, model, &evaluated, &rc, args.parallel.enumeration())
        .map_err(|e| err(ErrorKind::Process, e))?;
    for (part, part_groups) in groups.groups.iter().enumerate() {
        for group in part_groups.groups() {
            let id = format!("g{}", preview.groups.len());
            let mut members = Vec::new();
            for i in 0..group.members().len() {
                let member = group.member_diagram_set(i);
                let k = (0..sets.len())
                    .find(|&k| group_of[k].is_none() && same_subprocess(member, &sets[k]))
                    .ok_or_else(|| {
                        err(
                            ErrorKind::Run,
                            format!(
                                "the flavour grouping has a member {} that is none of the \
                                 enumerated subprocesses",
                                member.label()
                            ),
                        )
                    })?;
                group_of[k] = Some(id.clone());
                members.push(set_id(k));
            }
            preview.groups.push(FlavourGroupSummary {
                id,
                phase_space: part,
                representative: members[0].clone(),
                members,
            });
        }
    }
    Ok((preview, group_of))
}

/// Run `vibegraph enumerate`.
pub(crate) fn run(args: &EnumerateArgs) -> Result<(), CliError> {
    args.parallel
        .install()
        .map_err(|e| err(ErrorKind::Usage, e))?;
    let artifact_path = args.out.join(ARTIFACT_FILENAME);
    let summary_path = args.out.join(SUMMARY_FILENAME);
    let svg_dir = args.out.join(SVG_DIRNAME);
    // Refused before the card is read, so a run that would clobber an earlier
    // one spends nothing.
    if !args.force {
        for path in [&artifact_path, &summary_path, &svg_dir] {
            if path.exists() {
                return Err(err(
                    ErrorKind::Io,
                    format!(
                        "{} already exists (pass --force to overwrite)",
                        path.display()
                    ),
                ));
            }
        }
    }

    let opts = ParsingOptions::default();
    let (parsed, proc_card) = crate::read_proc_card_text(&args.proc_card, &opts)?;
    let config = GlobalConfig {
        ufo_search_path: assets::resolve_ufo_search_path(
            parsed.model.as_ref(),
            args.ufo_dir.as_deref(),
        )
        .map_err(|e| err(ErrorKind::Model, e))?,
        restrict_path_override: None,
        run_card_path: args.run_card.clone(),
    };
    let loaded = config
        .load_model(&parsed.model)
        .map_err(|e| err(ErrorKind::Model, format!("failed to load model: {e}")))?;
    let artifact =
        DiagramArtifact::enumerate(&proc_card, &parsed, &loaded, args.parallel.enumeration())
            .map_err(|e| err(ErrorKind::Process, e.to_string()))?;
    let model = &loaded.model;

    let preview = if args.run_card.is_some() || args.param_card.is_some() {
        Some(flavour_preview(
            args,
            &config,
            &parsed,
            model,
            &artifact.sets,
            artifact.process_facts.as_ref(),
        )?)
    } else {
        None
    };
    let (flavour_groups, group_of) = match preview {
        Some((preview, group_of)) => (Some(preview), group_of.into_iter().map(Some).collect()),
        None => (None, vec![None; artifact.sets.len()]),
    };

    let summary = Summary {
        schema_version: SCHEMA_VERSION,
        process: artifact.process.clone(),
        model: ModelRef {
            label: artifact.model.label(),
            digest: artifact.model.digest.clone(),
        },
        diagram_set_digest: artifact.diagram_set_digest.clone(),
        flavour_groups,
        sets: artifact
            .sets
            .iter()
            .zip(group_of)
            .enumerate()
            .map(|(k, (set, group))| summarize_set(k, set, model, args.svg, group))
            .collect(),
    };

    // Every drawing is made before anything is written, so a diagram the
    // renderer refuses leaves the output directory as it was.
    let drawer = DiagramDrawer::new(model);
    let mut drawings = Vec::new();
    for (k, set) in artifact.sets.iter().enumerate() {
        for (j, d) in set
            .diagrams
            .iter()
            .take(args.svg.drawn(set.diagrams.len()))
            .enumerate()
        {
            let svg = drawer.svg(d).map_err(|e| {
                err(
                    ErrorKind::Run,
                    format!("cannot draw {} ({}): {e}", diagram_id(k, j), set.label()),
                )
            })?;
            drawings.push((k, svg_path(k, j), svg));
        }
    }

    std::fs::create_dir_all(&args.out).map_err(|e| io_error("create", &args.out, e))?;
    artifact
        .write_to_path(&artifact_path, args.force)
        .map_err(|e| err(ErrorKind::Io, e.to_string()))?;
    let json = serde_json::to_string_pretty(&summary).expect("the summary serializes");
    std::fs::write(&summary_path, json + "\n").map_err(|e| io_error("write", &summary_path, e))?;
    if args.force {
        remove_earlier_drawings(&svg_dir)?;
    }
    for k in 0..artifact.sets.len() {
        if drawings.iter().any(|(set, _, _)| *set == k) {
            let dir = svg_dir.join(set_id(k));
            std::fs::create_dir_all(&dir).map_err(|e| io_error("create", &dir, e))?;
        }
    }
    for (_, relative, svg) in &drawings {
        let path = args.out.join(relative);
        std::fs::write(&path, svg).map_err(|e| io_error("write", &path, e))?;
    }

    let total: usize = artifact.sets.iter().map(|s| s.diagrams.len()).sum();
    info!(
        "{total} diagrams in {} subprocesses, {} drawn",
        artifact.sets.len(),
        drawings.len()
    );
    if output::mode() == output::Mode::Ldjson {
        output::result(&EnumerateResult {
            artifact: artifact_path.display().to_string(),
            summary_path: summary_path.display().to_string(),
            svg_dir: (!drawings.is_empty()).then(|| svg_dir.display().to_string()),
            drawings: drawings.len(),
            summary: &summary,
        });
        return Ok(());
    }
    let line = |text: String| output::text_line(format_args!("{text}"));
    line(format!("process:  {}", summary.process));
    line(format!(
        "model:    {} ({})",
        summary.model.label, summary.model.digest
    ));
    for set in &summary.sets {
        let group = match &set.flavour_group {
            Some(Some(g)) => format!(", flavour group {g}"),
            _ => String::new(),
        };
        line(format!(
            "{:<4} {}{}: {} diagram{}, {} configuration{}{group}",
            set.id,
            set.label,
            if set.mirrored { " (mirrored)" } else { "" },
            set.diagrams,
            if set.diagrams == 1 { "" } else { "s" },
            set.configurations.len(),
            if set.configurations.len() == 1 {
                ""
            } else {
                "s"
            },
        ));
    }
    line(format!(
        "total:    {total} diagrams in {} subprocess{}",
        summary.sets.len(),
        if summary.sets.len() == 1 { "" } else { "es" }
    ));
    line(format!("wrote {}", artifact_path.display()));
    line(format!("wrote {}", summary_path.display()));
    if !drawings.is_empty() {
        line(format!(
            "wrote {} drawing{} under {}",
            drawings.len(),
            if drawings.len() == 1 { "" } else { "s" },
            svg_dir.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_selection_parses_its_three_forms() {
        assert_eq!("all".parse(), Ok(SvgSelection::All));
        assert_eq!("none".parse(), Ok(SvgSelection::None));
        assert_eq!("first=3".parse(), Ok(SvgSelection::First(3)));
        for bad in ["", "first=", "first=x", "some", "3"] {
            assert!(bad.parse::<SvgSelection>().is_err(), "{bad}");
        }
        assert_eq!(SvgSelection::First(3).drawn(2), 2);
        assert_eq!(SvgSelection::First(3).drawn(5), 3);
        assert_eq!(SvgSelection::None.drawn(5), 0);
        assert_eq!(SvgSelection::All.drawn(5), 5);
    }

    /// `--force` removes only the file names this command writes.
    #[test]
    fn only_drawing_names_are_recognised() {
        assert!(is_set_name("s0") && is_set_name("s12"));
        assert!(!is_set_name("s") && !is_set_name("x0") && !is_set_name("s1a"));
        assert!(is_drawing_name("s3", "s3.d0.svg"));
        assert!(is_drawing_name("s3", "s3.d17.svg"));
        assert!(!is_drawing_name("s3", "s4.d0.svg"));
        assert!(!is_drawing_name("s3", "s3.d.svg"));
        assert!(!is_drawing_name("s3", "s3.d1.png"));
        assert!(!is_drawing_name("s3", "notes.svg"));
    }
}
