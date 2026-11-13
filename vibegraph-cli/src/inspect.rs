//! `vibegraph inspect`: the results a saved integration artifact records,
//! read back without re-running anything, and its configuration ranking.

use std::path::{Path, PathBuf};

use std::io::Write;

use clap::Args;
use serde::Serialize;
use vibegraph::artifact::{ConfigShare, IntegrateArtifact};
use vibegraph::hadronic::Observable;
use vibegraph::vegas::Chi2PerDof;

use crate::error::{err, CliError, ErrorKind};
use crate::integrate::{
    measurement, multiplicity_line, recorded_observable, result_object, FactorizationScale,
};
use crate::output;
use crate::ranking::{self, DiagramsSummary, Ranking};

#[derive(Args, Debug)]
pub(crate) struct InspectArgs {
    /// The artifact `vibegraph integrate` wrote (`grid.bin.zst`).
    pub(crate) artifact: PathBuf,

    /// Write the ranking JSON to this file (`-` for stdout): the run's
    /// configurations by decreasing share of the observable, with the
    /// sampler's channels and the diagrams' drawings.
    #[arg(long, value_name = "FILE")]
    pub(crate) ranking: Option<PathBuf>,

    /// The `vibegraph enumerate` output directory (or its `diagrams.json`) to
    /// pair the ranking with. Refused unless it describes the diagram sets the
    /// integration did; the ranking then takes its drawing paths from it.
    #[arg(long, value_name = "DIR|FILE", requires = "ranking")]
    pub(crate) diagrams: Option<PathBuf>,

    /// Overwrite an existing `--ranking` file.
    #[arg(long, requires = "ranking")]
    pub(crate) force: bool,
}

/// The `--output-ldjson` result of `inspect --ranking`: where the ranking was
/// written, and the ranking itself, nested so its `schema_version` stays apart
/// from the stream's.
#[derive(Debug, Serialize)]
struct RankingResult<'a> {
    ranking_path: String,
    ranking: &'a Ranking,
}

/// Whether `path` names stdout.
fn is_stdout(path: &Path) -> bool {
    path == Path::new("-")
}

/// Print `args.artifact`'s recorded results: as text, or under
/// `--output-ldjson` as the `result` object the integration that wrote it
/// emitted.
pub(crate) fn run(args: &InspectArgs) -> Result<(), CliError> {
    let path = &args.artifact;
    if let Some(out) = &args.ranking {
        // Refused before the artifact is read, so nothing is spent on a run
        // whose output could not be written.
        if is_stdout(out) && output::mode() == output::Mode::Ldjson {
            return Err(err(
                ErrorKind::Usage,
                "--ranking - writes to stdout, which --output-ldjson already carries; \
                 name a file instead",
            ));
        }
        if !is_stdout(out) && !args.force && out.exists() {
            return Err(err(
                ErrorKind::Io,
                format!(
                    "{} already exists (pass --force to overwrite)",
                    out.display()
                ),
            ));
        }
    }
    let artifact = IntegrateArtifact::read_from_path(path).map_err(|e| {
        err(
            ErrorKind::Artifact,
            format!("cannot read {}: {e}", path.display()),
        )
    })?;
    let observable = recorded_observable(&artifact)?;
    if let Some(out) = &args.ranking {
        return write_ranking(&artifact, observable, out, args.diagrams.as_deref());
    }
    let object = result_object(&artifact, path, observable)?;
    if output::mode() == output::Mode::Ldjson {
        output::result(&object);
        return Ok(());
    }

    let line = |text: String| output::text_line(format_args!("{text}"));
    line(format!("process:  {}", artifact.process));
    line(format!(
        "model:    {} ({})",
        object.model.label, object.model.digest
    ));
    line(format!(
        "diagrams: {} sets ({})",
        artifact.sets.len(),
        artifact.diagram_set_digest
    ));
    line(format!(
        "params:   {}",
        if artifact.param_card.is_some() {
            "param card banked"
        } else {
            "model defaults (no param card)"
        }
    ));
    line(format!(
        "{} = {}",
        observable.symbol(),
        measurement(observable, artifact.sigma_pb, artifact.sigma_err_pb)
    ));
    line(format!(
        "χ²/dof:   {:.3}",
        Chi2PerDof(artifact.chi2_per_dof)
    ));
    line(format!(
        "scale:    {}",
        FactorizationScale::recorded_in(&artifact)?
    ));
    line(format!("seed:     {}", artifact.seed));
    if artifact.multiplicities.len() > 1 {
        for part in &artifact.multiplicities {
            line(multiplicity_line(part));
        }
    }
    for text in configuration_lines(&artifact, observable) {
        line(text);
    }
    for (j, c) in artifact.channels.iter().enumerate() {
        line(format!(
            "channel {j} ({:?}): α = {}, {} = {} (χ²/dof = {:.3})",
            c.key,
            c.alpha,
            observable.symbol(),
            measurement(observable, c.sigma_pb, c.sigma_err_pb),
            Chi2PerDof(c.chi2_per_dof)
        ));
    }
    line(format!(
        "artifact: {} (format version {})",
        path.display(),
        artifact.format_version
    ));
    Ok(())
}

/// Build `artifact`'s ranking, paired with the `enumerate` output at `diagrams`
/// where given, and write it to `out`.
fn write_ranking(
    artifact: &IntegrateArtifact,
    observable: Observable,
    out: &Path,
    diagrams: Option<&Path>,
) -> Result<(), CliError> {
    let summary = diagrams
        .map(|d| DiagramsSummary::read(d, &artifact.diagram_set_digest))
        .transpose()?;
    let document = ranking::build(artifact, observable, summary.as_ref())?;
    let json = serde_json::to_string_pretty(&document).expect("the ranking serializes") + "\n";
    if is_stdout(out) {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(json.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(|e| err(ErrorKind::Io, format!("cannot write the ranking: {e}")))?;
        return Ok(());
    }
    std::fs::write(out, json).map_err(|e| {
        err(
            ErrorKind::Io,
            format!("cannot write {}: {e}", out.display()),
        )
    })?;
    if output::mode() == output::Mode::Ldjson {
        output::result(&RankingResult {
            ranking_path: out.display().to_string(),
            ranking: &document,
        });
        return Ok(());
    }
    let (n, top) = document.headline();
    let top = match top {
        Some((id, share)) => format!(", {id} first at {:.2}%", 100.0 * share),
        None => String::new(),
    };
    output::text_line(format_args!(
        "wrote {} ({n} configuration{} ranked{top})",
        out.display(),
        if n == 1 { "" } else { "s" }
    ));
    Ok(())
}

/// The configuration shares, per integrated set in the artifact's order, each
/// set's configurations by decreasing share with their diagrams and share of
/// the run's total. A set's unattributed weight is listed only where it is not
/// zero.
fn configuration_lines(artifact: &IntegrateArtifact, observable: Observable) -> Vec<String> {
    let mut sets: Vec<usize> = Vec::new();
    for c in &artifact.configurations {
        if !sets.contains(&c.set) {
            sets.push(c.set);
        }
    }
    let mut out = Vec::new();
    for set in sets {
        let mut shares: Vec<&ConfigShare> = artifact
            .configurations
            .iter()
            .filter(|c| c.set == set && (c.configuration.is_some() || c.sigma != 0.0))
            .collect();
        shares.sort_by(|a, b| b.sigma.total_cmp(&a.sigma));
        let label = artifact
            .sets
            .get(set)
            .map(|s| s.label())
            .unwrap_or_default();
        let members = shares.first().map_or(1, |c| c.members.len());
        out.push(if members > 1 {
            format!("set {set} ({label}, a flavour group of {members} sets):")
        } else {
            format!("set {set} ({label}):")
        });
        for c in shares {
            let name = match c.configuration {
                Some(k) => format!(
                    "config {k} (diagrams {})",
                    c.diagrams
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => "no configuration".to_string(),
            };
            let share = if artifact.sigma_pb != 0.0 {
                100.0 * c.sigma / artifact.sigma_pb
            } else {
                0.0
            };
            out.push(format!(
                "  {name}: {} = {} ({share:.2}% of the total)",
                observable.symbol(),
                measurement(observable, c.sigma, c.sigma_err)
            ));
        }
    }
    out
}
