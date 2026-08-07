//! The status pane drawn below the log, as a pure function of [`UiState`].
//!
//! Two columns: what is being run on the left — model, process, the stage in
//! progress and its bar — and what has been measured on the right. Nothing here
//! reads a clock or a channel, so a given state always draws the same cells and
//! a test can assert on them.

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Widget};
use vibegraph::progress::stage;

use crate::si::fmt_si;
use crate::tui::elapsed_text;
use crate::tui::state::UiState;

/// Rows the footer occupies: one rule and five of content.
pub(crate) const HEIGHT: u16 = 6;

/// One picobarn in barns. [`UiState`] carries the cross section in the unit the
/// result line and the artifact use; `fmt_si` picks a prefix for a base unit.
const PB_IN_BARN: f64 = 1.0e-12;

/// Width the measurement column is given when the terminal can spare it.
const MEASUREMENT_WIDTH: u16 = 26;

const LOGO: &str = "~vibegraph~";

/// The logo's colour ramp, cycled per character. Rotating the starting index
/// walks the ramp along the word.
const LOGO_COLORS: [Color; 6] = [
    Color::Rgb(255, 105, 97),
    Color::Rgb(255, 170, 90),
    Color::Rgb(240, 225, 120),
    Color::Rgb(140, 215, 140),
    Color::Rgb(120, 190, 255),
    Color::Rgb(190, 150, 255),
];

/// The footer's own reading of a stage name. An unrecognised name is shown as
/// emitted rather than hidden.
fn stage_label(stage_name: &str) -> &str {
    match stage_name {
        stage::UFO_LOAD => "loading model",
        stage::ENUMERATE => "enumerating",
        stage::COMPILE => "compiling",
        stage::ALPHA_SURVEY => "channel survey",
        stage::VEGAS => "integrating",
        stage::WEIGHT_SCAN => "weight scan",
        stage::UNWEIGHT => "unweighting",
        other => other,
    }
}

/// A count, abbreviated once it stops being readable in full.
fn count(n: u64) -> String {
    if n < 100_000 {
        n.to_string()
    } else {
        fmt_si(n as f64, None, "", 3).trim_end().to_string()
    }
}

/// The pane's palette, given literally rather than by ANSI name.
///
/// A named colour belongs to the terminal's theme, and two of the ways a theme
/// may spend it leave text unreadable here. Solarized fills its eight bright
/// slots with a greyscale ramp — bright black is the background itself, and
/// bright blue, green, yellow and cyan are its base tones — so a named colour
/// drawn bold turns grey on any terminal that renders bold in the bright
/// variant, as iTerm2 does by default. Bright black is worse than grey: as a
/// foreground it is the background, and the cells come out blank. Literal RGB
/// is passed through instead of looked up, which is why the logo ramp above
/// keeps its colours while bold.
///
/// Labels sit below the default foreground rather than above it, which rules
/// out ANSI white as well — Solarized maps it to a near-white brighter than the
/// text it would be annotating.
const MUTED: Color = Color::Rgb(128, 128, 128);
const ACCENT: Color = Color::Rgb(120, 190, 255);
const ATTENTION: Color = Color::Rgb(255, 170, 90);

/// Labels and hints, set back from the values they annotate.
fn muted() -> Style {
    Style::default().fg(MUTED)
}

/// The numbers this run is producing.
fn accent() -> Style {
    Style::default().fg(ACCENT)
}

/// The pane wanting something from the reader: an answer, or a second stop key.
fn attention() -> Style {
    Style::default().fg(ATTENTION)
}

/// A cross section and its uncertainty, in the prefix the value's own magnitude
/// picks.
///
/// The significant figures are chosen from how far the uncertainty sits below
/// the value, because both are rendered at the value's width: a result quoted to
/// a part in 10⁴ at three figures would print its error bar as `0.00`, which
/// states a precision nobody measured. One digit past the gap between them is
/// the fewest that leaves the uncertainty something to say.
pub(crate) fn cross_section(sigma_pb: f64, err_pb: f64) -> String {
    let digits = if err_pb > 0.0 && sigma_pb.abs() > 0.0 {
        ((sigma_pb.abs() / err_pb).log10().ceil() as i64 + 1).clamp(3, 8) as usize
    } else {
        4
    };
    fmt_si(
        sigma_pb * PB_IN_BARN,
        Some(err_pb * PB_IN_BARN),
        "b",
        digits,
    )
}

/// The status pane. Construct it around the state to draw and render it into a
/// [`HEIGHT`]-row area.
pub(crate) struct Footer<'a> {
    state: &'a UiState,
}

impl<'a> Footer<'a> {
    pub(crate) fn new(state: &'a UiState) -> Self {
        Self { state }
    }

    fn logo_line(&self) -> Line<'static> {
        let mut spans: Vec<Span<'static>> = LOGO
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let color = LOGO_COLORS[(i + self.state.logo_phase) % LOGO_COLORS.len()];
                Span::styled(
                    c.to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        if let Some(model) = &self.state.model {
            let digest: String = model.digest.chars().take(6).collect();
            spans.push(Span::styled(
                format!(
                    "  {} ({digest}\u{2026})  {}p {}v {}c",
                    model.label, model.particles, model.vertices, model.couplings
                ),
                muted(),
            ));
        }
        Line::from(spans)
    }

    fn process_line(&self) -> Line<'static> {
        let mut spans = Vec::new();
        if let Some(process) = &self.state.process {
            spans.push(Span::styled(
                process.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ));
        }
        if let Some(channels) = self.state.channels {
            let plural = if channels == 1 { "" } else { "s" };
            spans.push(Span::styled(
                format!("   {channels} channel{plural}"),
                muted(),
            ));
        }
        Line::from(spans)
    }

    fn stage_line(&self) -> Line<'static> {
        let Some(stage_name) = &self.state.stage else {
            return Line::from(Span::styled("starting", muted()));
        };
        let mut text = stage_label(stage_name).to_string();
        match self.state.total {
            Some(total) => {
                text.push_str(&format!("   {}/{}", count(self.state.done), count(total)))
            }
            None => text.push_str(&format!("   {}", count(self.state.done))),
        }
        let mut spans = vec![Span::raw(text)];
        if self.state.chi2 > 0.0 {
            spans.push(Span::styled(
                format!("   \u{3c7}\u{b2}/dof {:.2}", self.state.chi2),
                muted(),
            ));
        }
        Line::from(spans)
    }

    /// How much longer the reporting stage looks like taking: its own elapsed
    /// time scaled by the work still to do over the work done. For the VEGAS
    /// stage the total is itself the projected convergence iteration, so this
    /// is time-to-target, re-priced as the projection moves. Absent whenever
    /// the arithmetic would be a guess: no total, nothing done yet, or no tick
    /// has stamped the clock since the stage began.
    fn remaining(&self) -> Option<Duration> {
        let done = self.state.done;
        let total = self.state.total.filter(|&t| t >= done && done > 0)?;
        let in_stage = self
            .state
            .elapsed
            .checked_sub(self.state.stage_started)
            .filter(|t| !t.is_zero())?;
        Some(in_stage.mul_f64((total - done) as f64 / done as f64))
    }

    /// The gauge's row: the bar, flanked by the run's elapsed time on the left
    /// and the stage's estimated remaining time on the right — spent time in
    /// the measurement colour, owed time in the attention colour, so the two
    /// read apart at a glance. A row too narrow to afford the times, or a
    /// state no tick has stamped a clock into, carries the bar alone.
    fn gauge_row(&self, area: Rect, buf: &mut Buffer) {
        const TIME_WIDTH: u16 = 9;
        if self.state.elapsed.is_zero() || area.width < 4 * TIME_WIDTH {
            self.gauge().render(area, buf);
            return;
        }
        let [left, bar, right] = Layout::horizontal([
            Constraint::Length(TIME_WIDTH),
            Constraint::Min(10),
            Constraint::Length(TIME_WIDTH),
        ])
        .areas(area);
        buf.set_line(
            left.x,
            left.y,
            &Line::from(Span::styled(elapsed_text(self.state.elapsed), accent())),
            left.width.saturating_sub(1),
        );
        self.gauge().render(bar, buf);
        if let Some(remaining) = self.remaining() {
            let text = format!("-{}", elapsed_text(remaining));
            let width = (text.chars().count() as u16).min(right.width.saturating_sub(1));
            buf.set_line(
                right.x + right.width - width,
                right.y,
                &Line::from(Span::styled(text, attention())),
                width,
            );
        }
    }

    /// The progress bar. A stage that does not know its extent draws an empty
    /// bar labelled with the count alone — the honest reading of "this many so
    /// far, out of an unknown number".
    fn gauge(&self) -> Gauge<'static> {
        let (ratio, label) = match self.state.total {
            Some(total) if total > 0 => (
                (self.state.done as f64 / total as f64).clamp(0.0, 1.0),
                format!("{}/{}", count(self.state.done), count(total)),
            ),
            _ => (0.0, count(self.state.done)),
        };
        Gauge::default()
            .ratio(ratio)
            .label(Span::styled(
                label,
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .gauge_style(Style::default().fg(Color::Cyan))
    }

    /// What the keys are set to and which key moves what. The state is shown
    /// rather than only the key, because a filter narrowed three presses ago and
    /// then forgotten is indistinguishable from a stage that has gone quiet.
    ///
    /// While a question is pending the row carries the question instead: the
    /// keys mean something else until it is answered, and a row still offering
    /// the level ladder would be describing keys that do not work.
    fn keys_line(&self) -> Line<'static> {
        if let Some(question) = &self.state.prompt {
            return Line::from(Span::styled(
                question.clone(),
                attention().add_modifier(Modifier::BOLD),
            ));
        }
        Line::from(vec![
            Span::styled("level: ", muted()),
            Span::raw(self.state.level.label()),
            Span::styled(" \u{25b2}\u{25bc}  ", muted()),
            Span::styled("scope: ", muted()),
            Span::raw(self.state.scope.label()),
            Span::styled(" \u{25c2}\u{25b8}", muted()),
        ])
    }

    /// What a stop key will do next: ask for one, or take one that was already
    /// asked for and is being waited on.
    fn abort_line(&self) -> Line<'static> {
        if self.state.prompt.is_some() {
            return Line::from(Span::styled("y allow  n decline", attention()));
        }
        if self.state.stopping {
            Line::from(Span::styled("^C again  quit now", attention()))
        } else {
            Line::from(Span::styled("q / ^C  abort", muted()))
        }
    }

    fn sigma_line(&self) -> Line<'static> {
        match self.state.sigma_pb {
            None => Line::from(Span::styled("\u{3c3} = \u{2014}", muted())),
            Some(sigma_pb) => Line::from(Span::styled(
                format!("\u{3c3} = {}", cross_section(sigma_pb, self.state.err_pb)),
                accent().add_modifier(Modifier::BOLD),
            )),
        }
    }

    /// The accept/reject pass's own headline: how much of the sample exists, and
    /// what fraction of the points drawn for it are expected to survive.
    ///
    /// It replaces the cross section rather than joining it because during
    /// unweighting σ is an input read from the artifact, not a measurement this
    /// run is making — the cell would be reporting a number that never moves.
    fn events_line(&self) -> Line<'static> {
        Line::from(Span::styled(
            match self.state.total {
                Some(total) => format!("events  {}/{}", count(self.state.done), count(total)),
                None => format!("events  {}", count(self.state.done)),
            },
            accent().add_modifier(Modifier::BOLD),
        ))
    }

    fn efficiency_line(&self) -> Line<'static> {
        let Some(efficiency) = self.state.efficiency.filter(|e| *e > 0.0) else {
            return Line::default();
        };
        Line::from(Span::styled(
            format!("efficiency  {:.1}%", 100.0 * efficiency),
            muted(),
        ))
    }

    fn eval_line(&self) -> Line<'static> {
        let Some(ns) = self.state.ns_per_eval.filter(|ns| *ns > 0.0) else {
            return Line::default();
        };
        Line::from(Span::styled(
            format!(
                "eval  {}  {}",
                fmt_si(ns * 1.0e-9, None, "s", 3),
                fmt_si(1.0e9 / ns, None, "/s", 3)
            ),
            muted(),
        ))
    }

    fn render_run(&self, area: Rect, buf: &mut Buffer) {
        let lines = [
            Some(self.logo_line()),
            Some(self.process_line()),
            Some(self.stage_line()),
            None,
            Some(self.keys_line()),
        ];
        for (row, line) in lines.iter().enumerate() {
            let Some(y) = area
                .y
                .checked_add(row as u16)
                .filter(|y| *y < area.bottom())
            else {
                break;
            };
            match line {
                Some(line) => {
                    buf.set_line(area.x, y, line, area.width);
                }
                None => self.gauge_row(
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                ),
            }
        }
    }

    /// Whether the run is producing events rather than measuring a cross
    /// section, which is what decides the measurement column's headline.
    fn unweighting(&self) -> bool {
        self.state.stage.as_deref() == Some(stage::UNWEIGHT)
    }

    fn render_measurements(&self, area: Rect, buf: &mut Buffer) {
        let (headline, under) = if self.unweighting() {
            (self.events_line(), self.efficiency_line())
        } else {
            (self.sigma_line(), Line::default())
        };
        let lines = [
            Line::default(),
            headline,
            under,
            self.eval_line(),
            self.abort_line(),
        ];
        for (row, line) in lines.iter().enumerate() {
            let Some(y) = area
                .y
                .checked_add(row as u16)
                .filter(|y| *y < area.bottom())
            else {
                break;
            };
            buf.set_line(area.x, y, line, area.width);
        }
    }
}

impl Widget for Footer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        buf.set_string(
            area.x,
            area.y,
            "\u{2500}".repeat(area.width as usize),
            muted(),
        );
        let body = Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        };
        if body.is_empty() {
            return;
        }
        let [run, measurements] =
            Layout::horizontal([Constraint::Min(20), Constraint::Length(MEASUREMENT_WIDTH)])
                .areas(body);
        self.render_run(run, buf);
        self.render_measurements(measurements, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::{Footer, HEIGHT};
    use crate::logging::{LogLevel, Scope};
    use crate::tui::state::{ModelBrief, UiState};

    use std::time::Duration;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use vibegraph::phasespace::GEV2_TO_PB;
    use vibegraph::progress::stage;

    /// Draw a state into a terminal of the given width and hand back one string
    /// per row, trailing blanks trimmed.
    fn rows(state: &UiState, width: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, HEIGHT)).expect("a test terminal");
        terminal
            .draw(|frame| frame.render_widget(Footer::new(state), frame.area()))
            .expect("a drawn frame");
        let buffer = terminal.backend().buffer().clone();
        (0..HEIGHT)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    fn integrating() -> UiState {
        UiState {
            stage: Some(stage::VEGAS.to_string()),
            done: 4,
            total: Some(12),
            sigma_pb: Some(802.94),
            err_pb: 3.11,
            chi2: 1.02,
            ns_per_eval: Some(212.0),
            model: Some(ModelBrief {
                label: "sm-default".to_string(),
                digest: "d41f8cd98f00b204".to_string(),
                particles: 17,
                vertices: 64,
                couplings: 12,
            }),
            process: Some("p p > e+ e- QED=2 QCD=0".to_string()),
            channels: Some(4),
            efficiency: None,
            logo_phase: 0,
            elapsed: Duration::ZERO,
            stage_started: Duration::ZERO,
            level: LogLevel::Info,
            scope: Scope::All,
            prompt: None,
            stopping: false,
        }
    }

    #[test]
    fn the_footer_carries_the_run_brief_and_the_measurement() {
        let rows = rows(&integrating(), 80);
        assert_eq!(rows.len(), HEIGHT as usize);
        assert!(rows[0].starts_with('\u{2500}'), "{rows:?}");
        assert!(rows[1].contains("~vibegraph~"), "{rows:?}");
        assert!(rows[1].contains("sm-default (d41f8c\u{2026})"), "{rows:?}");
        assert!(rows[1].contains("17p 64v 12c"), "{rows:?}");
        assert!(rows[2].contains("p p > e+ e- QED=2 QCD=0"), "{rows:?}");
        assert!(rows[2].contains("4 channels"), "{rows:?}");
        assert!(rows[3].contains("integrating"), "{rows:?}");
        assert!(rows[3].contains("4/12"), "{rows:?}");
        assert!(rows[3].contains("\u{3c7}\u{b2}/dof 1.02"), "{rows:?}");
        assert!(rows[5].contains("q / ^C"), "{rows:?}");
    }

    /// The cross section is the reason the pane exists: it renders in the value's
    /// own SI prefix, with the uncertainty in the same one.
    #[test]
    fn the_cross_section_renders_with_its_uncertainty_in_picobarns() {
        let rows = rows(&integrating(), 80);
        assert!(
            rows[2].contains("\u{3c3} = 802.9 \u{b1} 3.1 pb"),
            "{rows:?}"
        );
    }

    /// A run converged four decades below its own value must still show an
    /// error bar: rendering both at a fixed three figures would print
    /// "2.025 ± 0.000 nb" and claim an exact result.
    #[test]
    fn an_uncertainty_far_below_the_value_keeps_a_digit() {
        let mut state = integrating();
        state.sigma_pb = Some(2024.729713);
        state.err_pb = 0.362891;
        let rows = rows(&state, 80);
        assert!(
            rows[2].contains("\u{3c3} = 2.0247 \u{b1} 0.0004 nb"),
            "{rows:?}"
        );
    }

    /// A cross section three decades away moves to the neighbouring prefix
    /// rather than growing the cell — the property the footer needs to hold a
    /// fixed layout while a live value drifts.
    #[test]
    fn a_cross_section_in_another_decade_changes_prefix_not_width() {
        let mut state = integrating();
        state.sigma_pb = Some(802_940.0);
        state.err_pb = 3_110.0;
        let wide = rows(&state, 80);
        assert!(
            wide[2].contains("\u{3c3} = 802.9 \u{b1} 3.1 nb"),
            "{wide:?}"
        );
    }

    /// Warm-up iterations report no estimate, and the cell says so rather than
    /// showing a zero that looks like a measurement.
    #[test]
    fn an_unmeasured_cross_section_is_shown_as_absent() {
        let mut state = integrating();
        state.sigma_pb = None;
        state.chi2 = 0.0;
        let rows = rows(&state, 80);
        assert!(rows[2].contains("\u{3c3} = \u{2014}"), "{rows:?}");
        assert!(!rows[2].contains("pb"), "{rows:?}");
        assert!(!rows[3].contains("dof"), "{rows:?}");
    }

    #[test]
    fn the_evaluation_cost_renders_as_a_time_and_a_rate() {
        let rows = rows(&integrating(), 80);
        assert!(rows[4].contains("eval  212 ns"), "{rows:?}");
        assert!(rows[4].contains("4.72 M/s"), "{rows:?}");
    }

    /// The bar fills in proportion to the fraction reported.
    #[test]
    fn the_bar_fills_in_proportion_to_the_reported_fraction() {
        let mut state = integrating();
        state.done = 6;
        state.total = Some(12);
        let rows = rows(&state, 80);
        let filled = rows[4].chars().filter(|c| *c == '\u{2588}').count();
        // The bar spans the run column, and its label occupies cells at the
        // centre, so half a bar is somewhat under half the column's width.
        assert!((20..=27).contains(&filled), "{filled} filled in {rows:?}");
        assert!(rows[4].contains("6/12"), "{rows:?}");
    }

    /// A stage with no known extent must not draw a full bar: an absent total
    /// is "how many more is unknown", not "all of them are done".
    #[test]
    fn an_unknown_total_draws_an_empty_bar_labelled_with_the_count() {
        let mut state = integrating();
        state.stage = Some(stage::ENUMERATE.to_string());
        state.done = 12;
        state.total = None;
        // The measurement column shares these rows; with nothing measured the
        // bar's row carries the bar alone.
        state.ns_per_eval = None;
        let rows = rows(&state, 80);
        assert!(!rows[4].contains('\u{2588}'), "{rows:?}");
        assert!(rows[4].contains("12"), "{rows:?}");
        assert!(rows[3].contains("enumerating   12"), "{rows:?}");
    }

    /// Once the drawing thread has stamped a clock into the state, the gauge
    /// row carries the run's elapsed time at its left end and the stage's
    /// estimated remaining time at its right — the stage is 4 of 12 done after
    /// a minute of its own time, so two more of them are owed.
    #[test]
    fn the_gauge_row_carries_elapsed_and_remaining() {
        let mut state = integrating();
        state.elapsed = Duration::from_secs(90);
        state.stage_started = Duration::from_secs(30);
        let rows = rows(&state, 80);
        assert!(rows[4].starts_with("1m 30s"), "{rows:?}");
        assert!(rows[4].contains("-2m 00s"), "{rows:?}");
        assert!(rows[4].contains("4/12"), "{rows:?}");
        assert!(rows[4].contains('\u{2588}'), "{rows:?}");
        // The remaining estimate sits at the right end of the bar, before the
        // measurement column's cells.
        let bar_end = 80 - super::MEASUREMENT_WIDTH as usize;
        let run_row: String = rows[4].chars().take(bar_end).collect();
        assert!(run_row.trim_end().ends_with("-2m 00s"), "{run_row:?}");
    }

    /// A stage that does not know its extent owes an unknowable amount of
    /// work: the elapsed time still shows, and no remaining estimate does.
    #[test]
    fn an_unknown_total_estimates_no_remaining_time() {
        let mut state = integrating();
        state.stage = Some(stage::ENUMERATE.to_string());
        state.total = None;
        state.elapsed = Duration::from_secs(90);
        state.stage_started = Duration::from_secs(30);
        state.ns_per_eval = None;
        let rows = rows(&state, 80);
        assert!(rows[4].starts_with("1m 30s"), "{rows:?}");
        assert!(!rows[4].contains('-'), "{rows:?}");
    }

    /// Before any tick has stamped a clock — every state a unit test builds,
    /// and the instant before the first draw — the row is the bar alone, so a
    /// zero elapsed is never shown as a measurement.
    #[test]
    fn an_unstamped_clock_leaves_the_times_off() {
        let rows = rows(&integrating(), 80);
        // The bar begins in the row's first cell: nothing flanks it.
        assert!(rows[4].starts_with('\u{2588}'), "{rows:?}");
        assert!(!rows[4].contains("0 s"), "{rows:?}");
    }

    /// A count past five digits is abbreviated so the bar's label cannot grow
    /// past the bar.
    #[test]
    fn large_counts_are_abbreviated() {
        let mut state = integrating();
        state.done = 1_200_000;
        state.total = Some(2_000_000);
        let rows = rows(&state, 80);
        assert!(rows[4].contains("1.20 M/2.00 M"), "{rows:?}");
    }

    /// A state with nothing reported yet still draws, so the pane appears the
    /// moment the run starts rather than at the first measurement.
    #[test]
    fn an_empty_state_draws_a_pane() {
        let rows = rows(&UiState::default(), 80);
        assert_eq!(rows.len(), HEIGHT as usize);
        assert!(rows[1].contains("~vibegraph~"), "{rows:?}");
        assert!(rows[3].contains("starting"), "{rows:?}");
        assert!(rows[2].contains("\u{3c3} = \u{2014}"), "{rows:?}");
    }

    /// A terminal too narrow for both columns must still draw: the measurement
    /// column gives way and nothing panics on the arithmetic.
    #[test]
    fn a_narrow_terminal_still_draws() {
        for width in [12, 20, 24, 30, 46] {
            let rows = rows(&integrating(), width);
            assert_eq!(rows.len(), HEIGHT as usize, "width {width}");
            for row in &rows {
                assert!(
                    row.chars().count() <= width as usize,
                    "width {width}: {row}"
                );
            }
        }
    }

    /// The keys are only usable if the pane says what they are set to, and only
    /// trustworthy if it says what they are set to *now*.
    #[test]
    fn the_keys_row_states_the_level_and_the_scope() {
        let mut state = integrating();
        state.level = LogLevel::Debug;
        state.scope = Scope::Sampling;
        let rows = rows(&state, 80);
        assert!(rows[5].contains("level: DEBUG"), "{rows:?}");
        assert!(rows[5].contains("scope: sampling"), "{rows:?}");
        assert!(
            rows[5].contains('\u{25b2}') && rows[5].contains('\u{25bc}'),
            "{rows:?}"
        );
        assert!(
            rows[5].contains('\u{25c2}') && rows[5].contains('\u{25b8}'),
            "{rows:?}"
        );
    }

    /// A pending question takes over the bottom row entirely: the question on
    /// the left, the keys that answer it on the right, and none of the hints
    /// for keys that will not be read as themselves until it is answered.
    #[test]
    fn a_pending_question_replaces_the_key_hints() {
        let mut state = integrating();
        state.prompt = Some("download PDF set TestSet (26.3 MB)?".to_string());
        let rows = rows(&state, 80);
        assert!(rows[5].contains("download PDF set TestSet"), "{rows:?}");
        assert!(rows[5].contains("y allow  n decline"), "{rows:?}");
        assert!(!rows[5].contains("level:"), "{rows:?}");
        assert!(!rows[5].contains("abort"), "{rows:?}");
    }

    /// Once a stop has been asked for, the hint has to change: the same key
    /// press now means something else, and a pane still offering "abort" would
    /// be inviting an immediate quit under the name of a graceful one.
    #[test]
    fn the_stop_hint_changes_once_a_stop_has_been_asked_for() {
        let running = rows(&integrating(), 80);
        assert!(running[5].contains("q / ^C  abort"), "{running:?}");

        let mut state = integrating();
        state.stopping = true;
        let asked = rows(&state, 80);
        assert!(asked[5].contains("^C again  quit now"), "{asked:?}");
        assert!(!asked[5].contains("abort"), "{asked:?}");
    }

    /// Generating events measures no cross section, so the measurement column
    /// carries the sample instead: how much of it exists, and how much of what
    /// is drawn for it survives the accept/reject.
    #[test]
    fn the_measurement_column_carries_the_sample_while_unweighting() {
        let mut state = integrating();
        state.stage = Some(stage::UNWEIGHT.to_string());
        state.done = 250;
        state.total = Some(1_000);
        state.efficiency = Some(0.541);
        let rows = rows(&state, 80);
        assert!(rows[2].contains("events  250/1000"), "{rows:?}");
        assert!(!rows[2].contains('\u{3c3}'), "{rows:?}");
        assert!(rows[3].contains("efficiency  54.1%"), "{rows:?}");
        // The bar the accept/reject pass drives is the one it already reported.
        assert!(rows[4].contains("250/1000"), "{rows:?}");
    }

    /// The swap is the unweighting stage's alone: every other stage of a
    /// `generate` run is still measured against the cross section.
    #[test]
    fn the_weight_scan_still_shows_the_cross_section() {
        let mut state = integrating();
        state.stage = Some(stage::WEIGHT_SCAN.to_string());
        state.efficiency = Some(0.541);
        let rows = rows(&state, 80);
        assert!(rows[2].contains("\u{3c3} = 802.9"), "{rows:?}");
        assert!(!rows[2].contains("events"), "{rows:?}");
    }

    /// A scan that has not run yet leaves the row empty rather than claiming an
    /// efficiency of zero.
    #[test]
    fn an_unmeasured_efficiency_is_left_blank() {
        let mut state = integrating();
        state.stage = Some(stage::UNWEIGHT.to_string());
        state.efficiency = None;
        let rows = rows(&state, 80);
        // The stage label on the same row is the left column's; the measurement
        // column has nothing to say, and says nothing.
        assert!(!rows[3].contains('%'), "{rows:?}");
    }

    /// The ramp walks along the word: the same state at a later phase paints the
    /// same characters in different colours, which is the whole of the
    /// animation.
    #[test]
    fn the_logo_phase_moves_the_colour_ramp_and_nothing_else() {
        let mut later = integrating();
        later.logo_phase = 3;
        assert_eq!(rows(&integrating(), 80), rows(&later, 80));

        let colors = |state: &UiState| {
            let mut terminal =
                Terminal::new(TestBackend::new(80, HEIGHT)).expect("a test terminal");
            terminal
                .draw(|frame| frame.render_widget(Footer::new(state), frame.area()))
                .expect("a drawn frame");
            let buffer = terminal.backend().buffer().clone();
            (0..11)
                .map(|x| buffer[(x, 1)].style().fg)
                .collect::<Vec<_>>()
        };
        assert_ne!(colors(&integrating()), colors(&later));
    }

    /// The GeV⁻² → pb conversion the ingestion side performs, checked at the
    /// value the footer actually prints: a mistaken conversion here is invisible
    /// to every test that only reads `UiState`.
    #[test]
    fn the_printed_cross_section_matches_the_result_line_arithmetic() {
        let mut state = integrating();
        let sigma_gev2 = 1.0e-6;
        state.sigma_pb = Some(sigma_gev2 * GEV2_TO_PB);
        state.err_pb = 0.0;
        let rows = rows(&state, 80);
        // 1e-6 GeV⁻² × 3.893793721e8 pb·GeV² = 389.4 pb.
        assert!(rows[2].contains("389.4 \u{b1} 0.0 pb"), "{rows:?}");
    }
}
