//! `vibegraph parameters`: a model's external parameters, and with a process the
//! ones reaching its diagrams, as `parameters.json`, plus the default
//! `param_card.dat` of the restricted model.
//!
//! The records come from the library ([`vibegraph::ufo::report`]), which reads
//! them from the model's own restriction, binder and reweight state, so a reader
//! of this output holds no copy of those rules. `enumerate` writes the same two
//! files beside `diagrams.json`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Args;
use serde::Serialize;
use vibegraph::config::GlobalConfig;
use vibegraph::diagrams::{enumerate_lines_in, DiagramSet, ParsingOptions};
use vibegraph::ufo::report::{default_param_card, describe_parameters_at, ParameterReport};
use vibegraph::ufo::UFOModel;
use vibegraph::widths::ComputedWidth;

use crate::assets;
use crate::error::{err, io_error, CliError, ErrorKind};
use crate::output::{self, ModelRef};
use crate::parallel::ParallelArgs;

/// The version of the `parameters.json` schema.
///
/// Adding a field keeps it; removing, renaming or re-meaning one bumps it.
pub(crate) const SCHEMA_VERSION: u32 = 1;

pub(crate) const REPORT_FILENAME: &str = "parameters.json";
pub(crate) const PARAM_CARD_FILENAME: &str = "param_card.dat";

#[derive(Args, Debug)]
pub(crate) struct ParametersArgs {
    /// Process card selecting the model (`import model`) and, optionally, a
    /// process (`generate`); `-` reads the card from stdin. Without a process the
    /// report is of the model alone and lists no entering parameters.
    pub(crate) proc_card: PathBuf,

    /// Output directory for `parameters.json` and `param_card.dat`.
    #[arg(long, default_value = ".")]
    pub(crate) out: PathBuf,

    /// Overwrite the outputs of an earlier run in `--out`.
    #[arg(long)]
    pub(crate) force: bool,

    /// SLHA `param_card.dat` to bind, as `integrate` binds it. Each width it sets
    /// to `Auto` is computed, and its record carries the computation
    /// (`computed_width`: the total, the bound value, the channels and the modes
    /// left out); a card `integrate` refuses is refused here too.
    #[arg(long)]
    pub(crate) param_card: Option<PathBuf>,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long)]
    pub(crate) ufo_dir: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) parallel: ParallelArgs,
}

/// `parameters.json`.
#[derive(Debug, Serialize)]
pub(crate) struct ParametersJson {
    schema_version: u32,
    /// The release that wrote the report.
    version: &'static str,
    model: ModelRef,
    /// The process the entering parameters are for; `null` for a model-only report.
    process: Option<String>,
    #[serde(flatten)]
    report: ParameterReport,
}

/// The report of `model`, and the default card text.
pub(crate) fn build(
    model: &Arc<UFOModel>,
    model_ref: ModelRef,
    process: Option<String>,
    sets: Option<&[DiagramSet]>,
    computed: &[ComputedWidth],
) -> (ParametersJson, String) {
    let report = describe_parameters_at(model, sets, computed);
    (
        ParametersJson {
            schema_version: SCHEMA_VERSION,
            version: env!("VIBEGRAPH_VERSION"),
            model: model_ref,
            process,
            report,
        },
        default_param_card(model),
    )
}

/// Write `parameters.json` and `param_card.dat` into `out` (created if need be),
/// returning their paths.
pub(crate) fn write(
    out: &Path,
    json: &ParametersJson,
    card: &str,
) -> Result<(PathBuf, PathBuf), CliError> {
    std::fs::create_dir_all(out).map_err(|e| io_error("create", out, e))?;
    let json_path = out.join(REPORT_FILENAME);
    let card_path = out.join(PARAM_CARD_FILENAME);
    let text = serde_json::to_string_pretty(json).expect("the report serializes");
    std::fs::write(&json_path, text + "\n").map_err(|e| io_error("write", &json_path, e))?;
    std::fs::write(&card_path, card).map_err(|e| io_error("write", &card_path, e))?;
    Ok((json_path, card_path))
}

/// The `--output-ldjson` result object.
#[derive(Debug, Serialize)]
struct ParametersResult<'a> {
    report_path: String,
    param_card_path: String,
    report: &'a ParametersJson,
}

pub(crate) fn run(args: &ParametersArgs) -> Result<(), CliError> {
    args.parallel
        .install()
        .map_err(|e| err(ErrorKind::Usage, e))?;
    let json_path = args.out.join(REPORT_FILENAME);
    let card_path = args.out.join(PARAM_CARD_FILENAME);
    if !args.force {
        for path in [&json_path, &card_path] {
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
    let (parsed, _) = crate::read_proc_card_text(&args.proc_card, &ParsingOptions::default())?;
    let config = GlobalConfig {
        ufo_search_path: assets::resolve_ufo_search_path(
            parsed.model.as_ref(),
            args.ufo_dir.as_deref(),
        )
        .map_err(|e| err(ErrorKind::Model, e))?,
        restrict_path_override: None,
        run_card_path: None,
    };
    let loaded = config
        .load_model(&parsed.model)
        .map_err(|e| err(ErrorKind::Model, format!("failed to load model: {e}")))?;
    let sets = if parsed.processes.is_empty() {
        None
    } else {
        let lines = enumerate_lines_in(&parsed, &loaded.model, args.parallel.enumeration())
            .map_err(|e| err(ErrorKind::Process, e.to_string()))?;
        Some(lines.sets)
    };
    let model_ref = ModelRef {
        label: loaded.identity.label(),
        digest: loaded.identity.digest.clone(),
    };
    let computed = match &args.param_card {
        Some(path) => crate::integrate::bind_param_card(Some(path), &loaded.model)?.auto_widths,
        None => Vec::new(),
    };
    let (json, card) = build(
        &loaded.model,
        model_ref,
        parsed.process_string(),
        sets.as_deref(),
        &computed,
    );
    let (report_path, card_path) = write(&args.out, &json, &card)?;
    if output::mode() == output::Mode::Ldjson {
        output::result(&ParametersResult {
            report_path: report_path.display().to_string(),
            param_card_path: card_path.display().to_string(),
            report: &json,
        });
        return Ok(());
    }
    let line = |text: String| output::text_line(format_args!("{text}"));
    line(format!(
        "model:    {} ({})",
        json.model.label, json.model.digest
    ));
    if let Some(process) = &json.process {
        line(format!("process:  {process}"));
    }
    line(format!("{} parameters", json.report.parameters.len()));
    if let Some(entering) = &json.report.entering {
        line(format!("{} enter the couplings", entering.len()));
    }
    line(format!("wrote {}", report_path.display()));
    line(format!("wrote {}", card_path.display()));
    Ok(())
}
