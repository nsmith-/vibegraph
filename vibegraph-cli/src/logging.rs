//! Where the binary's diagnostics go.
//!
//! Every command splits its output in two: `stdout` carries the result — the
//! cross section, the path written, the `check-events` report — and nothing
//! else, so a pipeline reading it sees the same bytes at every verbosity;
//! everything the run has to say about itself is a `tracing` event and lands on
//! `stderr`.
//!
//! Levels follow the usual reading: `error`/`warn` are things gone wrong,
//! `info` is the MadGraph-style running commentary a person watching the run
//! wants, `debug` is per-stage internals and `trace` is per-item detail.
//! `RUST_LOG` is the expert override — when it is set it replaces the level the
//! flags ask for and can scope directives per module.
//!
//! Where the lines go depends on whether anything is watching. Piped, they are
//! written to `stderr` as they arrive. On a terminal a status display takes
//! them instead, formatted the same way but handed over as strings for it to
//! push into the scrollback, and a second layer — filtered on the progress
//! target alone, so it is unaffected by the visible level — keeps the display's
//! state current.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use clap::{ArgAction, Args, ValueEnum};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::{filter_fn, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::{Filter, Layer, SubscriberExt};
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;
use vibegraph::progress;

use crate::tui::state::ProgressLayer;
use crate::tui::Feed;

/// The level a run reports at when nothing asks for another one.
const DEFAULT_LEVEL: LevelFilter = LevelFilter::INFO;

/// Verbosity levels, spelled out for `--log-level`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

/// How loud the run is and where its diagnostics are recorded.
#[derive(Args, Debug, Clone)]
pub struct LogArgs {
    /// Raise the level: `-v` run notices, `-vv` pipeline internals, `-vvv` all.
    #[arg(
        long,
        short = 'v',
        action = ArgAction::Count,
        global = true,
        conflicts_with_all = ["quiet", "log_level"],
    )]
    pub verbose: u8,

    /// Report only warnings and errors.
    #[arg(long, short = 'q', global = true, conflicts_with = "log_level")]
    pub quiet: bool,

    /// Report at this level, instead of counting `-v`s.
    #[arg(long, value_enum, value_name = "LEVEL", global = true)]
    pub log_level: Option<LogLevel>,

    /// Also record every event, at `trace`, to this file.
    ///
    /// The file is written whatever the terminal is showing, which is what makes
    /// a run reviewable afterwards at a level nobody chose to watch it at.
    #[arg(long, value_name = "PATH", global = true)]
    pub log_file: Option<PathBuf>,

    /// Draw the live status pane even where one would not be drawn by default.
    #[arg(long, global = true, conflicts_with = "no_tui")]
    pub tui: bool,

    /// Never draw the live status pane; report in plain lines.
    #[arg(long, global = true)]
    pub no_tui: bool,
}

impl LogArgs {
    /// The level the flags ask for. `RUST_LOG`, when set, replaces it.
    fn level(&self) -> LevelFilter {
        if let Some(level) = self.log_level {
            return level.into();
        }
        if self.quiet {
            return LevelFilter::WARN;
        }
        match self.verbose {
            0 | 1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }

    /// Whether this run should draw the live status pane.
    ///
    /// Both streams have to be terminals: the pane is drawn on `stderr`, and a
    /// redirected `stdout` means something downstream is reading the result,
    /// which is a run nobody is watching.
    pub(crate) fn wants_tui(&self) -> bool {
        if self.no_tui {
            return false;
        }
        self.tui || (std::io::stdout().is_terminal() && std::io::stderr().is_terminal())
    }
}

/// Hands each formatted event to the display as a string instead of writing it
/// to a stream: the display owns the terminal, and two writers to one screen
/// would interleave mid-line.
#[derive(Clone)]
struct LineChannel(Sender<String>);

/// One event's formatted text, sent on when the formatter is done with it.
struct FormattedEvent {
    text: Vec<u8>,
    lines: Sender<String>,
}

impl std::io::Write for FormattedEvent {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.text.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for FormattedEvent {
    fn drop(&mut self) {
        let text = String::from_utf8_lossy(&self.text);
        for line in text.trim_end_matches('\n').split('\n') {
            // A display that has already come down is not an error: the run is
            // over and the line has nowhere left to go.
            let _ = self.lines.send(line.to_string());
        }
    }
}

impl<'a> MakeWriter<'a> for LineChannel {
    type Writer = FormattedEvent;

    fn make_writer(&'a self) -> Self::Writer {
        FormattedEvent {
            text: Vec::new(),
            lines: self.0.clone(),
        }
    }
}

/// The line layer's filter, swappable while the process runs.
///
/// Holding it is what lets a caller change what is on screen without tearing
/// the subscriber down; the events already written are unaffected, so a change
/// only applies from the next event on.
pub(crate) struct LogHandle {
    #[allow(dead_code)]
    visible: reload::Handle<EnvFilter, Registry>,
}

#[allow(dead_code)]
impl LogHandle {
    /// Show events at `level` and below from here on.
    pub(crate) fn set_level(&self, level: LevelFilter) -> Result<(), String> {
        self.visible
            .reload(base_filter(level))
            .map_err(|e| format!("cannot change the log level: {e}"))
    }
}

/// A filter admitting everything at `level` and below, with `RUST_LOG` — when
/// the environment sets it — replacing that default outright.
fn base_filter(level: LevelFilter) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy()
}

/// The layer that turns events into lines, in the form the level asks for.
///
/// At `info` the lines are a commentary and read best bare. Past it they are
/// evidence, and which module produced an event and how far into the run it
/// arrived are the two things that make one line comparable with another.
fn line_layer<W>(
    writer: W,
    ansi: bool,
    detailed: bool,
    filter: reload::Layer<EnvFilter, Registry>,
) -> Box<dyn Layer<Registry> + Send + Sync>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let lines = fmt::layer().compact().with_ansi(ansi).with_writer(writer);
    if detailed {
        lines
            .with_timer(fmt::time::uptime())
            .with_filter(filter)
            .boxed()
    } else {
        lines
            .without_time()
            .with_target(false)
            .with_filter(filter)
            .boxed()
    }
}

/// Install the process-wide subscriber. Called once, before any command runs.
///
/// `display`, when present, is a running status pane: the formatted lines go to
/// it instead of to `stderr`, and it is given a layer of its own that folds
/// progress measurements into what it draws.
pub(crate) fn init(args: &LogArgs, display: Option<Feed>) -> Result<LogHandle, String> {
    let filter = base_filter(args.level());
    // The filter, not the flags, decides the format: `RUST_LOG=debug` is as much
    // a request for the detailed form as `-vv` is.
    let detailed = Filter::<Registry>::max_level_hint(&filter)
        .unwrap_or(DEFAULT_LEVEL)
        .ge(&LevelFilter::DEBUG);
    let (filter, visible) = reload::Layer::new(filter);

    let (line, progress) = match display {
        // The display renders the text itself, so escape codes in it would be
        // drawn as the characters they are made of.
        Some(feed) => (
            line_layer(LineChannel(feed.lines), false, detailed, filter),
            Some(
                ProgressLayer::new(feed.state)
                    .with_filter(filter_fn(|meta| meta.target() == progress::TARGET))
                    .boxed(),
            ),
        ),
        // Escape codes belong on a terminal and nowhere else: the notice stream
        // is routinely captured and compared.
        None => (
            line_layer(
                std::io::stderr as fn() -> std::io::Stderr,
                std::io::stderr().is_terminal(),
                detailed,
                filter,
            ),
            None,
        ),
    };

    let mut layers = vec![line];
    layers.extend(progress);
    if let Some(path) = &args.log_file {
        let file = std::fs::File::create(path)
            .map_err(|e| format!("cannot open log file {}: {e}", path.display()))?;
        layers.push(
            fmt::layer()
                .with_ansi(false)
                .with_writer(Arc::new(file))
                .with_filter(LevelFilter::TRACE)
                .boxed(),
        );
    }

    Registry::default()
        .with(layers)
        .try_init()
        .map_err(|e| format!("cannot install the log subscriber: {e}"))?;
    Ok(LogHandle { visible })
}

#[cfg(test)]
mod tests {
    use super::{LogArgs, LogLevel};
    use tracing::level_filters::LevelFilter;

    fn args(verbose: u8, quiet: bool, log_level: Option<LogLevel>) -> LogArgs {
        LogArgs {
            verbose,
            quiet,
            log_level,
            log_file: None,
            tui: false,
            no_tui: false,
        }
    }

    #[test]
    fn a_bare_run_reports_at_info() {
        assert_eq!(args(0, false, None).level(), LevelFilter::INFO);
    }

    #[test]
    fn the_flags_move_the_level_both_ways() {
        assert_eq!(args(1, false, None).level(), LevelFilter::INFO);
        assert_eq!(args(2, false, None).level(), LevelFilter::DEBUG);
        assert_eq!(args(3, false, None).level(), LevelFilter::TRACE);
        assert_eq!(args(9, false, None).level(), LevelFilter::TRACE);
        assert_eq!(args(0, true, None).level(), LevelFilter::WARN);
    }

    /// The overrides decide on their own, without asking what the streams are:
    /// `--no-tui` is how a run under a terminal is made to report in plain
    /// lines, and `--tui` how one that is not is made to draw anyway.
    #[test]
    fn the_display_overrides_outrank_the_streams() {
        let mut forced = args(0, false, None);
        forced.tui = true;
        assert!(forced.wants_tui());

        let mut refused = args(0, false, None);
        refused.no_tui = true;
        assert!(!refused.wants_tui());

        // Refusing outranks asking, so a script passing both gets plain lines.
        let mut both = args(0, false, None);
        both.tui = true;
        both.no_tui = true;
        assert!(!both.wants_tui());
    }

    /// An explicit level is the level, whatever else was passed.
    #[test]
    fn an_explicit_level_wins() {
        assert_eq!(
            args(3, false, Some(LogLevel::Error)).level(),
            LevelFilter::ERROR
        );
        assert_eq!(args(0, true, Some(LogLevel::Off)).level(), LevelFilter::OFF);
    }
}
