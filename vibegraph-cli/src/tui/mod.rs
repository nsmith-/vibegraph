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
//! [`UiState`] on a fixed tick, and reads the keys. The command itself runs on
//! the main thread and never touches the terminal.
//!
//! The keys reach the run by three routes, none of which is the terminal. The
//! level and scope keys swap the line layer's filter through the reload handle
//! the subscriber hands over once it is built, so what is shown changes from the
//! next event on and the history already written stays as it was. The stop key
//! raises a [`StopSignal`] the integration reads at its own iteration boundary —
//! the run decides when it is safe to stop, and the second press is what takes
//! that decision away from it. And a run that has a question to put — may this
//! be downloaded? — hands it over through [`ask_to_download`] and blocks on the
//! answer, because a raw-mode terminal whose keys this thread is reading leaves
//! a plain `stdin` prompt nothing to read and nowhere clean to draw.
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
use std::sync::{Arc, Mutex, OnceLock};
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
use vibegraph::budget::StopSignal;

use crate::logging::{LogHandle, LogLevel, Scope};
use crate::si::fmt_si;
use crate::tui::footer::Footer;
use crate::tui::state::UiState;

/// How often the pane is redrawn, and how long a key press waits to be seen.
const TICK: Duration = Duration::from_millis(50);

/// How long the logo's colour ramp holds each rotation. Slow enough that the
/// wave reads as motion rather than as flicker, fast enough that a stalled
/// display is obvious.
const LOGO_STEP: Duration = Duration::from_millis(150);

/// Exit status for a run the operator stopped, by the usual convention for a
/// process killed by SIGINT.
const ABORT_EXIT: i32 = 130;

/// The terminal the display writes to: everything it draws is a diagnostic, and
/// diagnostics belong on `stderr`.
type Screen = Terminal<CrosstermBackend<std::io::Stderr>>;

/// Whether a display is currently drawing, and therefore whether a result line
/// has to wait for it.
static LIVE: AtomicBool = AtomicBool::new(false);

/// The drawing thread's name, which is how a thread asking for the terminal
/// back tells whether it is the one that would have to hand it over.
const DRAW_THREAD: &str = "vibegraph-tui";

/// Asks the drawing thread to come down, for a caller that cannot reach the
/// [`Tui`] to stop it — a panic unwinds the thread that owns it.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Set once the drawing thread has given the terminal back, so a caller that
/// asked for it can wait rather than print into a pane still being drawn.
static TAKEN_DOWN: AtomicBool = AtomicBool::new(false);

/// Result lines printed while the pane was up, in the order they were produced.
static HELD: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The run's stop request, shared between the keys that raise it and the
/// integration that reads it.
///
/// One per process, because there is one operator and one run for them to stop.
/// A command asks for it whether or not a display was started; with no display
/// nothing can raise it, and the run is decided by its budget alone.
static STOP: OnceLock<StopSignal> = OnceLock::new();

/// The signal a command hands to the integration it drives.
pub(crate) fn stop_signal() -> StopSignal {
    STOP.get_or_init(StopSignal::new).clone()
}

/// A question waiting for the drawing thread to put to the operator.
struct PromptRequest {
    /// Lines pushed into the history first, so the terms being agreed to are on
    /// record above the question.
    details: Vec<String>,
    /// The one-line question the footer shows while waiting.
    question: String,
    reply: Sender<bool>,
}

/// The question a run has posted and the display has not yet picked up.
///
/// A slot rather than a channel because there is at most one: the asker blocks
/// until its answer comes back, so a second question cannot exist before the
/// first is resolved.
static PROMPT: Mutex<Option<PromptRequest>> = Mutex::new(None);

fn take_prompt() -> Option<PromptRequest> {
    match PROMPT.lock() {
        Ok(mut slot) => slot.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}

/// Put a download question to the operator through the running display,
/// blocking until a key answers it.
///
/// `None` when no display is drawing — the caller owns its streams and asks on
/// them itself. With one drawing, the terminal is in raw mode and its keys are
/// read here, so this is the only way the question can be asked at all. A
/// display that cannot read keys, or that comes down before an answer, answers
/// no: an unanswerable question must never become consent.
pub(crate) fn ask_to_download(question: &str, details: Vec<String>) -> Option<bool> {
    if !LIVE.load(Ordering::SeqCst) {
        return None;
    }
    let (reply, answer) = mpsc::channel();
    let request = PromptRequest {
        details,
        question: question.to_string(),
        reply,
    };
    match PROMPT.lock() {
        Ok(mut slot) => *slot = Some(request),
        Err(poisoned) => *poisoned.into_inner() = Some(request),
    }
    Some(answer.recv().unwrap_or(false))
}

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
    log: Arc<OnceLock<LogHandle>>,
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
        install_signal_handler();

        let stop = Arc::new(AtomicBool::new(false));
        let log: Arc<OnceLock<LogHandle>> = Arc::new(OnceLock::new());
        let worker = {
            let state = Arc::clone(&state);
            let stop = Arc::clone(&stop);
            let log = Arc::clone(&log);
            let abort = stop_signal();
            std::thread::Builder::new()
                .name(DRAW_THREAD.to_string())
                .spawn(move || draw_loop(terminal, &incoming, &state, &stop, &log, &abort))
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
            log,
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

    /// Give the display the filter handle its level and scope keys drive.
    ///
    /// It cannot be had at [`Tui::start`]: the display takes the terminal first,
    /// because where the subscriber's lines go depends on whether it got it, and
    /// the handle only exists once that subscriber is built. Until this is
    /// called those keys have nothing to change and are ignored.
    pub(crate) fn attach(&self, handle: LogHandle) {
        let _ = self.log.set(handle);
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
    if state.stopping {
        line.push_str(" (stopped early)");
    }
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
    TAKEN_DOWN.store(true, Ordering::SeqCst);
}

/// What a key press asks the display for. Anything else is ignored, so a stray
/// keystroke cannot disturb a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    /// Stop the run: gracefully the first time, immediately the second.
    Stop,
    Louder,
    Quieter,
    ScopeForward,
    ScopeBack,
}

/// Read a terminal event as one of the display's keys.
///
/// Raw mode suppresses the terminal's own interrupt, so `^C` has to be
/// recognised here or it would do nothing at all.
fn key_of(event: &Event) -> Option<Key> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Key::Stop),
        KeyCode::Char('q') => Some(Key::Stop),
        KeyCode::Up => Some(Key::Louder),
        KeyCode::Down => Some(Key::Quieter),
        KeyCode::Right => Some(Key::ScopeForward),
        KeyCode::Left => Some(Key::ScopeBack),
        _ => None,
    }
}

/// Read a terminal event as the answer to a pending question.
///
/// `y` alone grants. Enter takes the default — no, as the plain prompt's
/// `[y/N]` spells it — and the keys that otherwise mean "get me out" (`n`, `q`,
/// Esc, `^C`) decline the download rather than acting as themselves: while a
/// question is up, no key may do anything but answer it.
fn answer_of(event: &Event) -> Option<bool> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Char('y' | 'Y') => Some(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(false),
        KeyCode::Char('n' | 'N' | 'q') | KeyCode::Esc | KeyCode::Enter => Some(false),
        _ => None,
    }
}

/// A line marking a change the operator made, pushed into the history so the
/// lines above and below it explain their own difference.
fn marker(text: &str) -> String {
    format!("\u{2500}\u{2500} {text} \u{2500}\u{2500}")
}

/// What the display is currently showing, and the abort that has been asked for.
///
/// Held by the drawing thread alone and mirrored into [`UiState`] on each change:
/// the footer draws from the state, and the keys decide from this.
struct Controls {
    level: LogLevel,
    scope: Scope,
    /// Whether a graceful stop has already been requested, which is what makes
    /// the next stop key an immediate one.
    stopping: bool,
}

fn draw_loop(
    mut terminal: Screen,
    incoming: &Receiver<String>,
    state: &Mutex<UiState>,
    stop: &AtomicBool,
    log: &OnceLock<LogHandle>,
    abort: &StopSignal,
) {
    // A terminal whose events cannot be read still shows the pane; it just has
    // no keys, and must not spin on the failing poll.
    let mut keys = true;
    let mut controls = Controls {
        level: LogLevel::default(),
        scope: Scope::default(),
        stopping: false,
    };
    let started = Instant::now();
    // The question currently on screen, waiting for a key to answer it.
    let mut pending: Option<Sender<bool>> = None;
    let mut size = terminal.size().ok();
    loop {
        // Take a resize before anything is written, for two reasons.
        //
        // The pane has to be wiped where it stands first. Reserving the viewport
        // again anchors it to the cursor, so it can land below the rows it used
        // to occupy, and those rows are cleared from the *new* origin down —
        // whatever sits above it survives, and the next line pushed into the
        // history scrolls that leftover pane up into the scrollback for good.
        //
        // And the width the history is wrapped to comes from the viewport, which
        // learns a new size only inside `draw`. Draining first would lay a line
        // out for the terminal that no longer exists and hand `insert_before` a
        // row count the terminal does not agree with.
        let current = terminal.size().ok();
        if current != size {
            let _ = terminal.clear();
            size = current;
        }
        let _ = terminal.autoresize();
        drain(&mut terminal, incoming);
        if pending.is_none() {
            if let Some(request) = take_prompt() {
                for line in &request.details {
                    insert_line(&mut terminal, line);
                }
                if keys {
                    set(state, |ui| ui.prompt = Some(request.question.clone()));
                    pending = Some(request.reply);
                } else {
                    // No keys means no way to say yes; the asker is unblocked
                    // with a no rather than left waiting forever.
                    let _ = request.reply.send(false);
                }
            }
        }
        advance_logo(state, started.elapsed());
        redraw(&mut terminal, state);
        if stop.load(Ordering::SeqCst) || INTERRUPTED.load(Ordering::SeqCst) {
            break;
        }
        if !keys {
            if let Some(reply) = pending.take() {
                let _ = reply.send(false);
                set(state, |ui| ui.prompt = None);
            }
            std::thread::sleep(TICK);
            continue;
        }
        match event::poll(TICK) {
            Ok(true) => match event::read() {
                Ok(event) if pending.is_some() => {
                    if let Some(granted) = answer_of(&event) {
                        if let Some(reply) = pending.take() {
                            let _ = reply.send(granted);
                        }
                        set(state, |ui| ui.prompt = None);
                        insert_line(
                            &mut terminal,
                            &marker(if granted {
                                "download allowed"
                            } else {
                                "download declined"
                            }),
                        );
                    }
                }
                Ok(event) => match key_of(&event) {
                    Some(Key::Stop) if controls.stopping => {
                        take_down(&mut terminal);
                        let _ = writeln!(std::io::stderr(), "vibegraph: aborted");
                        std::process::exit(ABORT_EXIT);
                    }
                    Some(Key::Stop) => {
                        controls.stopping = true;
                        abort.request();
                        set(state, |ui| ui.stopping = true);
                        insert_line(
                            &mut terminal,
                            &marker(
                                "stopping: the run is finishing what it holds; \
                                 press again to quit now",
                            ),
                        );
                    }
                    Some(key) => {
                        if let Some(change) = retune(&mut controls, key, log) {
                            set(state, |ui| {
                                ui.level = controls.level;
                                ui.scope = controls.scope;
                            });
                            insert_line(&mut terminal, &marker(&change));
                        }
                    }
                    None => {}
                },
                Err(_) => keys = false,
            },
            Ok(false) => {}
            Err(_) => keys = false,
        }
    }
    // A question still up when the display comes down is answered no, so the
    // thread that asked it is never left blocked on a pane that no longer exists.
    if let Some(reply) = pending {
        let _ = reply.send(false);
    }
    // Whatever was emitted between the last drain and the stop belongs in the
    // history too: the sender saw the stop set only after its send returned.
    drain(&mut terminal, incoming);
    take_down(&mut terminal);
}

/// Where a key moves the display's controls.
///
/// A key that changes nothing — the level already at the end of its ladder —
/// moves nowhere, so holding one down at an end neither reloads the filter nor
/// fills the scrollback with markers saying nothing happened.
fn retuned(controls: &Controls, key: Key) -> Option<(LogLevel, Scope)> {
    let next = match key {
        Key::Louder => (controls.level.louder(), controls.scope),
        Key::Quieter => (controls.level.quieter(), controls.scope),
        Key::ScopeForward => (controls.level, controls.scope.next()),
        Key::ScopeBack => (controls.level, controls.scope.previous()),
        Key::Stop => return None,
    };
    (next != (controls.level, controls.scope)).then_some(next)
}

/// Apply a level or scope key, returning what to say about it in the history.
fn retune(controls: &mut Controls, key: Key, log: &OnceLock<LogHandle>) -> Option<String> {
    let (level, scope) = retuned(controls, key)?;
    let handle = log.get()?;
    if let Err(why) = handle.show(level, scope) {
        return Some(why);
    }
    // Each key moves one of the two, so which one moved names the change.
    let moved_level = level != controls.level;
    controls.level = level;
    controls.scope = scope;
    Some(if moved_level {
        format!("log level \u{2192} {}", level.label())
    } else {
        format!("log scope \u{2192} {}", scope.label())
    })
}

/// Rotate the logo's colour ramp to where it stands at `elapsed`.
///
/// Driven by the clock rather than by a tick count so the wave keeps its pace
/// through a burst of log lines, which is when a drawing thread's ticks are
/// least even.
fn advance_logo(state: &Mutex<UiState>, elapsed: Duration) {
    let phase = (elapsed.as_millis() / LOGO_STEP.as_millis().max(1)) as usize;
    set(state, |ui| ui.logo_phase = phase);
}

fn set(state: &Mutex<UiState>, edit: impl FnOnce(&mut UiState)) {
    match state.lock() {
        Ok(mut state) => edit(&mut state),
        Err(poisoned) => edit(&mut poisoned.into_inner()),
    }
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
        take_terminal_back();
        previous(info);
    }));
}

/// Give the terminal back when the run is killed rather than stopped.
///
/// Raw mode is a property of the terminal and outlives the process that asked
/// for it, so a signal that ends the run without unwinding leaves the shell in
/// it. The signals are taken on a thread of their own rather than in a handler,
/// which is what lets this wait for the pane to come down; the signal is then
/// re-raised with its default disposition, so the exit status is the one the
/// kill would have produced and nothing here decides how the run ends.
///
/// Only installed once the display holds the terminal. A run reporting in plain
/// lines has nothing to restore and leaves the signals to their defaults.
#[cfg(unix)]
fn install_signal_handler() {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
    use signal_hook::iterator::Signals;

    // Raw mode suppresses the terminal's own interrupt, so a SIGINT here came
    // from outside rather than from the keys the display reads.
    let Ok(mut signals) = Signals::new([SIGTERM, SIGHUP, SIGQUIT, SIGINT]) else {
        return;
    };
    let _ = std::thread::Builder::new()
        .name("vibegraph-signals".to_string())
        .spawn(move || {
            for signal in signals.forever() {
                take_terminal_back();
                let _ = signal_hook::low_level::emulate_default_handler(signal);
            }
        });
}

#[cfg(not(unix))]
fn install_signal_handler() {}

/// Give the terminal back on behalf of a thread that cannot stop the display
/// the ordinary way, and wait for the pane to come down first.
///
/// Restoring the terminal without stopping the drawing thread does not hold:
/// within a tick it draws again, hides the cursor and paints the pane over
/// whatever was being printed. So the thread is asked to come down and given a
/// bounded moment to do it — bounded because a display that is wedged must not
/// take the panic message with it. The drawing thread cannot wait on itself,
/// and takes the terminal down on its way out regardless.
fn take_terminal_back() {
    let drawing = std::thread::current().name() == Some(DRAW_THREAD);
    if LIVE.load(Ordering::SeqCst) && !drawing {
        INTERRUPTED.store(true, Ordering::SeqCst);
        let deadline = Instant::now() + 10 * TICK;
        while !TAKEN_DOWN.load(Ordering::SeqCst) && Instant::now() < deadline {
            std::thread::sleep(TICK / 5);
        }
    }
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stderr(), Show);
}

#[cfg(test)]
mod tests {
    use super::{
        answer_of, ask_to_download, elapsed_text, key_of, marker, retuned, rows_of, summary,
        take_prompt, Controls, Key, LIVE,
    };
    use crate::logging::{LogLevel, Scope};
    use crate::tui::state::UiState;

    use std::time::Duration;

    use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn press(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn controls() -> Controls {
        Controls {
            level: LogLevel::Info,
            scope: Scope::All,
            stopping: false,
        }
    }

    /// Raw mode is on, so the terminal's own interrupt never fires: `^C` is a
    /// key like any other and stops the run only because it is read as one.
    #[test]
    fn ctrl_c_and_q_both_read_as_a_stop() {
        assert_eq!(
            key_of(&press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Key::Stop)
        );
        assert_eq!(
            key_of(&press(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(Key::Stop)
        );
        // A bare `c` is a letter, not an interrupt.
        assert_eq!(key_of(&press(KeyCode::Char('c'), KeyModifiers::NONE)), None);
    }

    #[test]
    fn the_arrows_read_as_level_and_scope_keys() {
        assert_eq!(
            key_of(&press(KeyCode::Up, KeyModifiers::NONE)),
            Some(Key::Louder)
        );
        assert_eq!(
            key_of(&press(KeyCode::Down, KeyModifiers::NONE)),
            Some(Key::Quieter)
        );
        assert_eq!(
            key_of(&press(KeyCode::Right, KeyModifiers::NONE)),
            Some(Key::ScopeForward)
        );
        assert_eq!(
            key_of(&press(KeyCode::Left, KeyModifiers::NONE)),
            Some(Key::ScopeBack)
        );
        assert_eq!(key_of(&press(KeyCode::Char('x'), KeyModifiers::NONE)), None);
    }

    /// A terminal reports a key twice — pressed and released — and acting on
    /// both would move the level two rungs for one press.
    #[test]
    fn only_a_press_counts() {
        let mut release = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(key_of(&Event::Key(release)), None);
    }

    #[test]
    fn the_level_keys_move_one_rung_at_a_time() {
        let controls = controls();
        assert_eq!(
            retuned(&controls, Key::Louder),
            Some((LogLevel::Debug, Scope::All))
        );
        assert_eq!(
            retuned(&controls, Key::Quieter),
            Some((LogLevel::Warn, Scope::All))
        );
    }

    #[test]
    fn the_scope_keys_move_the_scope_and_leave_the_level() {
        let controls = controls();
        assert_eq!(
            retuned(&controls, Key::ScopeForward),
            Some((LogLevel::Info, Scope::Diagrams))
        );
        assert_eq!(
            retuned(&controls, Key::ScopeBack),
            Some((LogLevel::Info, Scope::Pdf))
        );
    }

    /// At the end of the ladder the key does nothing at all — not a filter
    /// reload, and not a marker line claiming a change.
    #[test]
    fn a_level_key_at_the_end_of_its_ladder_moves_nothing() {
        let loudest = Controls {
            level: LogLevel::Trace,
            ..controls()
        };
        assert_eq!(retuned(&loudest, Key::Louder), None);
        let quietest = Controls {
            level: LogLevel::Off,
            ..controls()
        };
        assert_eq!(retuned(&quietest, Key::Quieter), None);
    }

    /// While a question is up, `y` is the only consent: Enter takes the `[y/N]`
    /// default, and every key that otherwise means "get me out" declines the
    /// download instead of acting as itself.
    #[test]
    fn y_grants_and_the_escape_keys_decline() {
        for yes in [KeyCode::Char('y'), KeyCode::Char('Y')] {
            assert_eq!(answer_of(&press(yes, KeyModifiers::NONE)), Some(true));
        }
        for no in [
            press(KeyCode::Char('n'), KeyModifiers::NONE),
            press(KeyCode::Char('N'), KeyModifiers::NONE),
            press(KeyCode::Char('q'), KeyModifiers::NONE),
            press(KeyCode::Esc, KeyModifiers::NONE),
            press(KeyCode::Enter, KeyModifiers::NONE),
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            assert_eq!(answer_of(&no), Some(false), "{no:?}");
        }
        // A key that is not an answer leaves the question standing.
        assert_eq!(answer_of(&press(KeyCode::Up, KeyModifiers::NONE)), None);
        let mut release = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(answer_of(&Event::Key(release)), None);
    }

    /// The request/reply handoff end to end: with no display the caller keeps
    /// its streams; with one live, the question lands in the slot with its
    /// terms, the answer sent back is what the asker receives, and a reply
    /// channel dropped without an answer is a no. One test rather than three
    /// because it toggles the process-wide `LIVE` flag.
    #[test]
    fn a_question_travels_through_the_slot_and_its_answer_comes_back() {
        use std::sync::atomic::Ordering;

        assert_eq!(ask_to_download("download X?", Vec::new()), None);

        LIVE.store(true, Ordering::SeqCst);
        let asker = std::thread::spawn(|| {
            ask_to_download("download X (1.0 MB)?", vec!["terms".to_string()])
        });
        let request = loop {
            if let Some(request) = take_prompt() {
                break request;
            }
            std::thread::yield_now();
        };
        assert_eq!(request.question, "download X (1.0 MB)?");
        assert_eq!(request.details, vec!["terms".to_string()]);
        let _ = request.reply.send(true);
        assert_eq!(asker.join().unwrap(), Some(true));

        let asker = std::thread::spawn(|| ask_to_download("download X?", Vec::new()));
        let request = loop {
            if let Some(request) = take_prompt() {
                break request;
            }
            std::thread::yield_now();
        };
        drop(request);
        assert_eq!(asker.join().unwrap(), Some(false));
        LIVE.store(false, Ordering::SeqCst);
    }

    /// The marker is what makes the scrollback self-explaining: the lines above
    /// and below it differ, and it says why.
    #[test]
    fn a_marker_line_states_the_change_it_marks() {
        assert_eq!(
            marker("log level \u{2192} DEBUG"),
            "\u{2500}\u{2500} log level \u{2192} DEBUG \u{2500}\u{2500}"
        );
    }

    /// A run the operator stopped closes on a line that says so, so the
    /// scrollback cannot be read as a run that finished what it was asked for.
    #[test]
    fn the_summary_records_that_a_run_was_stopped_early() {
        let state = UiState {
            stage: Some("vegas".to_string()),
            sigma_pb: Some(802.94),
            err_pb: 3.11,
            stopping: true,
            ..UiState::default()
        };
        assert_eq!(
            summary(&state, Duration::from_secs(64)),
            "vibegraph: vegas (stopped early), \u{3c3} = 802.9 \u{b1} 3.1 pb, 1m 04s"
        );
    }

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
