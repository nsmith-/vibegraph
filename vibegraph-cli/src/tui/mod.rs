//! The live status display: a status pane pinned below the log, on a terminal.
//!
//! The terminal keeps its own scrollback — the pane is an *inline* viewport of
//! [`footer::HEIGHT`] rows at the bottom of the screen, and log lines are pushed
//! into the history above it, so scrolling, searching and copying work as they
//! do for any other command and the history survives the run. Nothing is drawn
//! on an alternate screen and nothing is erased on exit but the pane itself.
//!
//! One thread owns the terminal. It drains formatted log lines from a channel
//! the subscriber's line layer writes into, redraws the pane from the shared
//! [`UiState`] on a fixed tick, and watches for the keys that abort the run. The
//! command itself runs on the main thread and never touches the terminal.
//!
//! `stdout` is not the display's to write. The command's result lines are held
//! back while the pane is up and printed once it has been taken down, so the
//! bytes a caller reads are the same ones, in the same order, that a run without
//! a display prints — and they land in the scrollback after the pane is gone
//! rather than through the middle of it.

pub(crate) mod footer;
pub(crate) mod state;

use std::fmt;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::Position;
use ratatui::style::Style;
use ratatui::{Terminal, TerminalOptions, Viewport};
use unicode_width::UnicodeWidthChar;

use crate::si::fmt_si;
use crate::tui::footer::Footer;
use crate::tui::state::UiState;

/// How often the pane is redrawn, and how long a key press waits to be seen.
const TICK: Duration = Duration::from_millis(50);

/// Exit status for a run the operator stopped, by the usual convention for a
/// process killed by SIGINT.
const ABORT_EXIT: i32 = 130;

/// The terminal the display writes to: everything it draws is a diagnostic, and
/// diagnostics belong on `stderr`.
type Screen = Terminal<CrosstermBackend<std::io::Stderr>>;

/// Whether a display is currently drawing, and therefore whether a result line
/// has to wait for it.
static LIVE: AtomicBool = AtomicBool::new(false);

/// Result lines printed while the pane was up, in the order they were produced.
static HELD: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// What the subscriber needs in order to feed a running display.
pub(crate) struct Feed {
    /// Formatted log lines, one per event, to be pushed into the scrollback.
    pub(crate) lines: Sender<String>,
    /// The state the progress layer folds measurements into.
    pub(crate) state: Arc<Mutex<UiState>>,
}

/// A running display, and the handle that takes it down again.
pub(crate) struct Tui {
    lines: Sender<String>,
    state: Arc<Mutex<UiState>>,
    stop: Arc<AtomicBool>,
    worker: JoinHandle<()>,
    started: Instant,
}

impl Tui {
    /// Take the terminal and start drawing.
    ///
    /// Fails without having touched anything if the terminal cannot be put into
    /// raw mode or the viewport cannot be reserved, so a caller can fall back to
    /// plain lines.
    pub(crate) fn start() -> Result<Self, String> {
        enable_raw_mode().map_err(|e| format!("cannot put the terminal in raw mode: {e}"))?;
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(footer::HEIGHT),
            },
        )
        .map_err(|e| {
            let _ = disable_raw_mode();
            format!("cannot reserve a status pane: {e}")
        })?;

        let (lines, incoming) = mpsc::channel();
        let state = Arc::new(Mutex::new(UiState::default()));
        state::install(&state);
        install_panic_hook();

        let stop = Arc::new(AtomicBool::new(false));
        let worker = {
            let state = Arc::clone(&state);
            let stop = Arc::clone(&stop);
            std::thread::Builder::new()
                .name("vibegraph-tui".to_string())
                .spawn(move || draw_loop(terminal, &incoming, &state, &stop))
                .map_err(|e| {
                    let _ = disable_raw_mode();
                    format!("cannot start the status display: {e}")
                })?
        };
        LIVE.store(true, Ordering::SeqCst);
        Ok(Self {
            lines,
            state,
            stop,
            worker,
            started: Instant::now(),
        })
    }

    /// What the subscriber writes into.
    pub(crate) fn feed(&self) -> Feed {
        Feed {
            lines: self.lines.clone(),
            state: Arc::clone(&self.state),
        }
    }

    /// Take the pane down, close the run with a plain summary line, and release
    /// the result lines that were held back while it was up.
    pub(crate) fn finish(self) {
        LIVE.store(false, Ordering::SeqCst);
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.worker.join();
        let state = match self.state.lock() {
            Ok(state) => state.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        let mut err = std::io::stderr();
        let _ = writeln!(err, "{}", summary(&state, self.started.elapsed()));
        let _ = err.flush();
        release_results();
    }
}

/// Print a line of the command's result.
///
/// With no display running this is `println!` and nothing more. With one running
/// the line is held until the pane comes down, so it is never written through
/// the middle of it — the bytes and their order are the same either way.
pub(crate) fn result_line(line: fmt::Arguments<'_>) {
    if LIVE.load(Ordering::SeqCst) {
        if let Ok(mut held) = HELD.lock() {
            held.push(line.to_string());
            return;
        }
    }
    println!("{line}");
}

fn release_results() {
    let held: Vec<String> = match HELD.lock() {
        Ok(mut held) => held.drain(..).collect(),
        Err(poisoned) => poisoned.into_inner().drain(..).collect(),
    };
    for line in held {
        println!("{line}");
    }
}

/// The line the scrollback ends on: what the run was doing, what it measured,
/// and how long it took.
fn summary(state: &UiState, elapsed: Duration) -> String {
    let stage = state.stage.as_deref().unwrap_or("nothing to do");
    let mut line = format!("vibegraph: {stage}");
    if let Some(sigma_pb) = state.sigma_pb {
        line.push_str(&format!(
            ", \u{3c3} = {}",
            footer::cross_section(sigma_pb, state.err_pb)
        ));
    }
    line.push_str(&format!(", {}", elapsed_text(elapsed)));
    line
}

fn elapsed_text(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs_f64();
    if seconds < 60.0 {
        fmt_si(seconds, None, "s", 3)
    } else {
        let whole = elapsed.as_secs();
        format!("{}m {:02}s", whole / 60, whole % 60)
    }
}

/// Split a formatted line into terminal rows, breaking at the display width.
///
/// Hard-breaking rather than wrapping at word boundaries is what makes the row
/// count predictable: the number of rows requested from `insert_before` has to
/// be the number of rows that will be drawn, or the tail is lost.
fn rows_of(line: &str, width: u16) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let width = width as usize;
    let mut rows = Vec::new();
    let mut row = String::new();
    let mut used = 0;
    for c in line.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > width {
            rows.push(std::mem::take(&mut row));
            used = 0;
        }
        row.push(c);
        used += w;
    }
    rows.push(row);
    rows
}

/// Push one formatted log line into the terminal's history above the pane.
fn insert_line(terminal: &mut Screen, line: &str) {
    let width = terminal.get_frame().area().width;
    let rows = rows_of(line, width);
    let height = u16::try_from(rows.len()).unwrap_or(u16::MAX).max(1);
    let _ = terminal.insert_before(height, |buf| {
        for (y, row) in rows.iter().enumerate() {
            let Ok(y) = u16::try_from(y) else {
                break;
            };
            buf.set_stringn(0, y, row, width as usize, Style::default());
        }
    });
}

fn redraw(terminal: &mut Screen, state: &Mutex<UiState>) {
    let snapshot = match state.lock() {
        Ok(state) => state.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let _ = terminal.draw(|frame| frame.render_widget(Footer::new(&snapshot), frame.area()));
}

/// Clear the pane, put the cursor back where it began, and give the terminal
/// back. Anything printed afterwards continues from where the pane was.
fn take_down(terminal: &mut Screen) {
    let origin = terminal.get_frame().area();
    let _ = terminal.clear();
    let _ = terminal.set_cursor_position(Position::new(origin.x, origin.y));
    let _ = terminal.show_cursor();
    let _ = Backend::flush(terminal.backend_mut());
    let _ = disable_raw_mode();
}

/// Whether a key event asks for the run to stop. Raw mode suppresses the
/// terminal's own interrupt, so `^C` has to be recognised here or it would do
/// nothing at all.
fn is_abort(event: &Event) -> bool {
    let Event::Key(key) = event else {
        return false;
    };
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Char('q') => true,
        _ => false,
    }
}

fn draw_loop(
    mut terminal: Screen,
    incoming: &Receiver<String>,
    state: &Mutex<UiState>,
    stop: &AtomicBool,
) {
    // A terminal whose events cannot be read still shows the pane; it just has
    // no keys, and must not spin on the failing poll.
    let mut keys = true;
    loop {
        drain(&mut terminal, incoming);
        redraw(&mut terminal, state);
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if !keys {
            std::thread::sleep(TICK);
            continue;
        }
        match event::poll(TICK) {
            Ok(true) => match event::read() {
                Ok(event) if is_abort(&event) => {
                    take_down(&mut terminal);
                    let _ = writeln!(std::io::stderr(), "vibegraph: aborted");
                    std::process::exit(ABORT_EXIT);
                }
                Ok(_) => {}
                Err(_) => keys = false,
            },
            Ok(false) => {}
            Err(_) => keys = false,
        }
    }
    // Whatever was emitted between the last drain and the stop belongs in the
    // history too: the sender saw the stop set only after its send returned.
    drain(&mut terminal, incoming);
    take_down(&mut terminal);
}

fn drain(terminal: &mut Screen, incoming: &Receiver<String>) {
    loop {
        match incoming.try_recv() {
            Ok(line) => insert_line(terminal, &line),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => return,
        }
    }
}

/// Give the terminal back before a panic message is printed, so the message is
/// readable and the shell that follows it is usable.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), Show);
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::{elapsed_text, rows_of, summary};
    use crate::tui::state::UiState;

    use std::time::Duration;

    #[test]
    fn a_short_line_is_one_row() {
        assert_eq!(
            rows_of("compiled 4 channels", 40),
            vec!["compiled 4 channels"]
        );
    }

    /// The row count is what `insert_before` is asked to reserve, so a line
    /// longer than the terminal has to break into exactly as many rows as it
    /// will occupy — one too few and the tail is never drawn.
    #[test]
    fn a_long_line_breaks_at_the_display_width() {
        let line = "x".repeat(25);
        let rows = rows_of(&line, 10);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].chars().count(), 10);
        assert_eq!(rows[2].chars().count(), 5);
        assert_eq!(rows.concat(), line);
    }

    /// Breaking counts display columns, not bytes: a multi-byte character is
    /// one column wide and must not shorten the row it sits in.
    #[test]
    fn multibyte_characters_count_as_their_display_width() {
        let line = "\u{3c3} = 802.94 \u{b1} 3.11 pb";
        let rows = rows_of(line, 10);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.concat(), line);
        // Ten columns, not ten bytes: `σ` costs one of each ten.
        assert_eq!(rows[0], "\u{3c3} = 802.94");
    }

    #[test]
    fn an_empty_line_is_still_one_row() {
        assert_eq!(rows_of("", 40), vec![String::new()]);
        assert_eq!(rows_of("anything", 0), vec![String::new()]);
    }

    #[test]
    fn the_summary_states_the_stage_the_result_and_the_time() {
        let state = UiState {
            stage: Some("vegas".to_string()),
            sigma_pb: Some(802.94),
            err_pb: 3.11,
            ..UiState::default()
        };
        assert_eq!(
            summary(&state, Duration::from_secs(64)),
            "vibegraph: vegas, \u{3c3} = 802.9 \u{b1} 3.1 pb, 1m 04s"
        );
    }

    #[test]
    fn a_run_with_no_measurement_still_closes_with_a_line() {
        let state = UiState::default();
        assert_eq!(
            summary(&state, Duration::from_millis(1500)),
            "vibegraph: nothing to do, 1.50 s"
        );
    }

    #[test]
    fn elapsed_time_reads_in_seconds_below_a_minute_and_in_minutes_above() {
        assert_eq!(elapsed_text(Duration::from_millis(212)), "212 ms");
        assert_eq!(elapsed_text(Duration::from_secs(59)), "59.0 s");
        assert_eq!(elapsed_text(Duration::from_secs(60)), "1m 00s");
        assert_eq!(elapsed_text(Duration::from_secs(3_601)), "60m 01s");
    }
}
