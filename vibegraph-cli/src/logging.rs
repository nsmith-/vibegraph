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

/// Verbosity levels, spelled out for `--log-level` and cycled by the display's
/// own level keys.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
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

impl LogLevel {
    /// The ladder the level keys walk, quietest first.
    const LADDER: [LogLevel; 6] = [
        LogLevel::Off,
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ];

    /// The next level up, or this one at the top of the ladder.
    pub(crate) fn louder(self) -> Self {
        let i = Self::LADDER.iter().position(|l| *l == self).unwrap_or(3);
        Self::LADDER[(i + 1).min(Self::LADDER.len() - 1)]
    }

    /// The next level down, or this one at the bottom.
    pub(crate) fn quieter(self) -> Self {
        let i = Self::LADDER.iter().position(|l| *l == self).unwrap_or(3);
        Self::LADDER[i.saturating_sub(1)]
    }

    /// How the level reads on screen.
    pub(crate) fn label(self) -> &'static str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

/// Which part of the pipeline the `debug`/`trace` tiers are shown for.
///
/// Narrowing the scope leaves the run's commentary — everything at `info` and
/// above — where it was, and restricts only the per-stage detail, which is the
/// tier that arrives faster than it can be read. A scope is a set of module
/// targets because that is what an event's default target already is.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Scope {
    #[default]
    All,
    Diagrams,
    Helas,
    Sampling,
    Pdf,
}

impl Scope {
    const CYCLE: [Scope; 5] = [
        Scope::All,
        Scope::Diagrams,
        Scope::Helas,
        Scope::Sampling,
        Scope::Pdf,
    ];

    /// The modules a scope admits at the detailed tiers. Empty for [`Scope::All`],
    /// which admits every module and needs no per-target directive at all.
    fn targets(self) -> &'static [&'static str] {
        match self {
            Scope::All => &[],
            Scope::Diagrams => &["vibegraph::diagrams", "vibegraph::ufo"],
            Scope::Helas => &["vibegraph::helas"],
            Scope::Sampling => &[
                "vibegraph::vegas",
                "vibegraph::budget",
                "vibegraph::phasespace",
                "vibegraph::unweight",
            ],
            Scope::Pdf => &["vibegraph::pdf", "vibegraph::proton", "vibegraph::hadronic"],
        }
    }

    /// How the scope reads on screen.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Scope::All => "all",
            Scope::Diagrams => "diagrams",
            Scope::Helas => "helas",
            Scope::Sampling => "sampling",
            Scope::Pdf => "pdf",
        }
    }

    pub(crate) fn next(self) -> Self {
        let i = Self::CYCLE.iter().position(|s| *s == self).unwrap_or(0);
        Self::CYCLE[(i + 1) % Self::CYCLE.len()]
    }

    pub(crate) fn previous(self) -> Self {
        let i = Self::CYCLE.iter().position(|s| *s == self).unwrap_or(0);
        Self::CYCLE[(i + Self::CYCLE.len() - 1) % Self::CYCLE.len()]
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
    pub(crate) fn level(&self) -> LogLevel {
        if let Some(level) = self.log_level {
            return level;
        }
        if self.quiet {
            return LogLevel::Warn;
        }
        match self.verbose {
            0 | 1 => LogLevel::Info,
            2 => LogLevel::Debug,
            _ => LogLevel::Trace,
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
    visible: reload::Handle<EnvFilter, Registry>,
}

impl LogHandle {
    /// Show `level` and above, narrowed to `scope` where the level is detailed
    /// enough for a scope to mean anything.
    pub(crate) fn show(&self, level: LogLevel, scope: Scope) -> Result<(), String> {
        self.visible
            .reload(base_filter(level, scope))
            .map_err(|e| format!("cannot change what is shown: {e}"))
    }
}

/// A filter admitting everything at `level`, with `RUST_LOG` — when the
/// environment sets it — replacing that default outright.
///
/// A narrowed `scope` splits the filter in two: the commentary tiers stay global
/// (a scope that could hide a warning would be a filter nobody could trust) and
/// the level asked for is granted to the scope's own modules alone.
///
/// The progress stream is excluded whatever the level, because it is a stream of
/// measurements for a display to fold in rather than lines for a person to read;
/// a `--log-file` sink records it from a layer of its own.
fn base_filter(level: LogLevel, scope: Scope) -> EnvFilter {
    let detailed = level > LogLevel::Info;
    let targets = scope.targets();
    let global = if detailed && !targets.is_empty() {
        LogLevel::Info
    } else {
        level
    };
    let mut filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::from(global).into())
        .from_env_lossy()
        .add_directive(
            format!("{}=off", progress::TARGET)
                .parse()
                .expect("the progress target is a well-formed directive"),
        );
    if detailed {
        for target in targets {
            filter = filter.add_directive(
                format!("{target}={}", level.label().to_lowercase())
                    .parse()
                    .expect("a module target and a level are a well-formed directive"),
            );
        }
    }
    filter
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
    let filter = base_filter(args.level(), Scope::default());
    // The filter, not the flags, decides the format: `RUST_LOG=debug` is as much
    // a request for the detailed form as `-vv` is.
    let detailed = Filter::<Registry>::max_level_hint(&filter)
        .unwrap_or(DEFAULT_LEVEL)
        .ge(&LevelFilter::DEBUG);
    let (filter, visible) = reload::Layer::new(filter);

    let (line, progress) = match display {
        // The display renders the text itself, and parses these codes back into
        // styles as it pushes each line into its history, so an event reads the
        // same whether or not a pane is drawing.
        Some(feed) => (
            line_layer(LineChannel(feed.lines), true, detailed, filter),
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
    use super::{base_filter, LogArgs, LogLevel, Scope};

    use tracing_subscriber::layer::{Layer, SubscriberExt};
    use tracing_subscriber::Registry;
    use vibegraph::progress;

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
        assert_eq!(args(0, false, None).level(), LogLevel::Info);
    }

    #[test]
    fn the_flags_move_the_level_both_ways() {
        assert_eq!(args(1, false, None).level(), LogLevel::Info);
        assert_eq!(args(2, false, None).level(), LogLevel::Debug);
        assert_eq!(args(3, false, None).level(), LogLevel::Trace);
        assert_eq!(args(9, false, None).level(), LogLevel::Trace);
        assert_eq!(args(0, true, None).level(), LogLevel::Warn);
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
            LogLevel::Error
        );
        assert_eq!(args(0, true, Some(LogLevel::Off)).level(), LogLevel::Off);
    }

    #[test]
    fn the_level_ladder_moves_one_step_and_stops_at_its_ends() {
        assert_eq!(LogLevel::Info.louder(), LogLevel::Debug);
        assert_eq!(LogLevel::Debug.louder(), LogLevel::Trace);
        assert_eq!(LogLevel::Trace.louder(), LogLevel::Trace);
        assert_eq!(LogLevel::Info.quieter(), LogLevel::Warn);
        assert_eq!(LogLevel::Off.quieter(), LogLevel::Off);
    }

    /// The scope cycle closes in both directions, so holding one key walks the
    /// whole list rather than stalling at an end.
    #[test]
    fn the_scope_cycle_wraps_both_ways() {
        assert_eq!(Scope::All.next(), Scope::Diagrams);
        assert_eq!(Scope::Pdf.next(), Scope::All);
        assert_eq!(Scope::All.previous(), Scope::Pdf);
        let mut scope = Scope::All;
        for _ in 0..Scope::CYCLE.len() {
            scope = scope.next();
        }
        assert_eq!(scope, Scope::All);
    }

    /// Every target a scope names, at every level, checked through the filter
    /// itself rather than through the list it was built from.
    fn shown(level: LogLevel, scope: Scope) -> Vec<String> {
        use std::sync::{Arc, Mutex};
        use tracing::{Event, Subscriber};
        use tracing_subscriber::layer::Context;

        struct Capture(Arc<Mutex<Vec<String>>>);
        impl<S: Subscriber> Layer<S> for Capture {
            fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
                let meta = event.metadata();
                if let Ok(mut seen) = self.0.lock() {
                    seen.push(format!("{}@{}", meta.target(), meta.level()));
                }
            }
        }

        let seen = Arc::new(Mutex::new(Vec::new()));
        let subscriber = Registry::default()
            .with(Capture(Arc::clone(&seen)).with_filter(base_filter(level, scope)));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "vibegraph::helas", "an info line");
            tracing::warn!(target: "vibegraph::diagrams", "a warning");
            tracing::debug!(target: "vibegraph::helas", "helas detail");
            tracing::debug!(target: "vibegraph::diagrams", "enumeration detail");
            tracing::trace!(target: "vibegraph::vegas", "sampling detail");
            progress::step(progress::stage::COMPILE, 1, Some(2));
        });
        let seen = seen.lock().expect("what the filter admitted").clone();
        seen
    }

    /// The measurement stream is for a display to fold in, not for a person to
    /// read: no level, however loud, turns it into lines.
    #[test]
    fn the_progress_stream_is_never_shown_as_lines() {
        for level in [LogLevel::Info, LogLevel::Debug, LogLevel::Trace] {
            let lines = shown(level, Scope::All);
            assert!(
                !lines.iter().any(|l| l.starts_with(progress::TARGET)),
                "{level:?}: {lines:?}"
            );
        }
    }

    /// An unscoped run at `debug` shows every module's detail, which is the
    /// baseline the narrowed scopes below are a restriction of.
    #[test]
    fn an_unscoped_run_shows_every_module() {
        let lines = shown(LogLevel::Debug, Scope::All);
        assert!(
            lines.contains(&"vibegraph::helas@DEBUG".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"vibegraph::diagrams@DEBUG".to_string()),
            "{lines:?}"
        );
    }

    /// A narrowed scope hides other modules' *detail* and keeps their
    /// commentary: a filter that could swallow a warning would be one nobody
    /// could leave switched on.
    #[test]
    fn a_narrowed_scope_hides_detail_and_keeps_the_commentary() {
        let lines = shown(LogLevel::Debug, Scope::Diagrams);
        assert!(
            lines.contains(&"vibegraph::diagrams@DEBUG".to_string()),
            "{lines:?}"
        );
        assert!(
            !lines.contains(&"vibegraph::helas@DEBUG".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"vibegraph::helas@INFO".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"vibegraph::diagrams@WARN".to_string()),
            "{lines:?}"
        );
    }

    /// The scope is a control on the detailed tiers alone: at `info` there is no
    /// detail to narrow, and narrowing must not start hiding the commentary.
    #[test]
    fn a_scope_does_nothing_at_the_commentary_tiers() {
        let all = shown(LogLevel::Info, Scope::All);
        let narrowed = shown(LogLevel::Info, Scope::Pdf);
        assert_eq!(all, narrowed, "{all:?} vs {narrowed:?}");
    }

    /// The sampling scope reaches `trace`, so the loudest tier is available
    /// module by module rather than only for the whole run at once.
    #[test]
    fn a_scope_grants_the_level_it_was_asked_for() {
        let lines = shown(LogLevel::Trace, Scope::Sampling);
        assert!(
            lines.contains(&"vibegraph::vegas@TRACE".to_string()),
            "{lines:?}"
        );
        assert!(
            !lines.contains(&"vibegraph::helas@DEBUG".to_string()),
            "{lines:?}"
        );
    }
}
