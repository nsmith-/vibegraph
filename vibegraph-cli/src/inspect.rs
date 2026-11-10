//! `vibegraph inspect`: the results a saved integration artifact records,
//! read back without re-running anything.

use std::path::PathBuf;

use clap::Args;
use vibegraph::artifact::IntegrateArtifact;
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
