//! `vibegraph run-card`: a run card resolved and classified, or the per-release
//! table of every run-card parameter, without sampling anything.
//!
//! Both views come from the library's own lists ([`vibegraph::runcard::report`]),
//! so a consumer that reads this output holds no copy of the parser, the
//! defaults or the refusal rules. A card an integration would refuse is still
//! reported: the refusal is a field of the result, not a failure of the command.

use std::path::PathBuf;

use clap::{ArgGroup, Args};
use serde::Serialize;
use vibegraph::config::{process_facts, GlobalConfig};
use vibegraph::diagrams::{enumerate_lines_in, ParsingOptions};
use vibegraph::runcard::report::{classification_table, describe_card, CardReport, TableEntry};
use vibegraph::runcard::RunCard;

use crate::assets;
use crate::error::{err, CliError, ErrorKind};
use crate::integrate::{load_run_card, process_string};
use crate::output;
use crate::parallel::ParallelArgs;

/// The version of the JSON this command writes.
///
/// Adding a field keeps it; removing, renaming or re-meaning one bumps it.
pub(crate) const SCHEMA_VERSION: u32 = 1;

#[derive(Args, Debug)]
#[command(group(ArgGroup::new("what").required(true).multiple(true).args(["run_card", "proc_card", "table"])))]
pub(crate) struct RunCardArgs {
    /// MadGraph `run_card.dat` to resolve and classify.
    #[arg(conflicts_with = "table")]
    pub(crate) run_card: Option<PathBuf>,

    /// Process card, for the defaults the process implies (the incoming-parton
    /// `maxjetflavor`, a decay's removed cuts, matching under jet multiplicities).
    /// With no run card, the report is of the card `integrate` would build.
    #[arg(long, value_name = "PROC_CARD", conflicts_with = "table")]
    pub(crate) proc_card: Option<PathBuf>,

    /// Print the table of every known run-card parameter, as honoured, ignored
    /// or refused with the refused values and a one-line note, instead of
    /// resolving a card. The same table for every card; it changes with the
    /// release.
    #[arg(long)]
    pub(crate) table: bool,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long, requires = "proc_card")]
    pub(crate) ufo_dir: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) parallel: ParallelArgs,
}

/// The `--table` result.
#[derive(Debug, Serialize)]
struct TableResult {
    schema_version: u32,
    /// The release the table describes.
    version: &'static str,
    parameters: Vec<TableEntry>,
}

/// The resolved-card result.
#[derive(Debug, Serialize)]
struct CardResult {
    schema_version: u32,
    version: &'static str,
    run_card: Option<String>,
    process: Option<String>,
    #[serde(flatten)]
    report: CardReport,
}

fn print_json(value: &impl Serialize) {
    let json = serde_json::to_string_pretty(value).expect("a report serializes");
    output::text_line(format_args!("{json}"));
}

pub(crate) fn run(args: &RunCardArgs) -> Result<(), CliError> {
    if args.table {
        let table = TableResult {
            schema_version: SCHEMA_VERSION,
            version: env!("VIBEGRAPH_VERSION"),
            parameters: classification_table(),
        };
        print_json(&table);
        output::result(&table);
        return Ok(());
    }

    let text = match &args.run_card {
        Some(path) => Some(std::fs::read_to_string(path).map_err(|e| {
            err(
                ErrorKind::Io,
                format!("cannot read the run card {}: {e}", path.display()),
            )
        })?),
        None => None,
    };

    let (process, decay, process_default) = match &args.proc_card {
        Some(path) => {
            args.parallel
                .install()
                .map_err(|e| err(ErrorKind::Usage, e))?;
            let (parsed, _) = crate::read_proc_card_text(path, &ParsingOptions::default())?;
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
            let model = &loaded.model;
            let pool = args.parallel.enumeration();
            let default: RunCard = load_run_card(&config, &parsed, model, || {
                let lines = enumerate_lines_in(&parsed, model, pool).map_err(|e| {
                    err(
                        ErrorKind::Process,
                        format!("failed to enumerate process: {e}"),
                    )
                })?;
                process_facts(&parsed, model, &lines, pool).map_err(|e| {
                    err(
                        ErrorKind::Card,
                        format!("cannot build the default run card: {e}"),
                    )
                })
            })?;
            (
                Some(process_string(&parsed)?),
                parsed.is_decay(),
                Some(default),
            )
        }
        None => (None, false, None),
    };

    let result = CardResult {
        schema_version: SCHEMA_VERSION,
        version: env!("VIBEGRAPH_VERSION"),
        run_card: args.run_card.as_ref().map(|p| p.display().to_string()),
        process,
        report: describe_card(text.as_deref(), decay, process_default.as_ref()),
    };
    print_json(&result);
    output::result(&result);
    Ok(())
}
