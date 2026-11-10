//! `vibegraph inspect`: the results a saved integration artifact records,
//! read back without re-running anything.

use std::path::PathBuf;

use clap::Args;
use vibegraph::artifact::{ConfigShare, IntegrateArtifact};
use vibegraph::hadronic::Observable;
use vibegraph::vegas::Chi2PerDof;

use crate::error::{err, CliError, ErrorKind};
use crate::integrate::{
    measurement, multiplicity_line, recorded_observable, result_object, FactorizationScale,
};
use crate::output;

#[derive(Args, Debug)]
pub(crate) struct InspectArgs {
    /// The artifact `vibegraph integrate` wrote (`grid.bin.zst`).
    pub(crate) artifact: PathBuf,
}

/// Print `args.artifact`'s recorded results: as text, or under
/// `--output-ldjson` as the `result` object the integration that wrote it
/// emitted.
pub(crate) fn run(args: &InspectArgs) -> Result<(), CliError> {
    let path = &args.artifact;
    let artifact = IntegrateArtifact::read_from_path(path).map_err(|e| {
        err(
            ErrorKind::Artifact,
            format!("cannot read {}: {e}", path.display()),
        )
    })?;
    let observable = recorded_observable(&artifact)?;
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
