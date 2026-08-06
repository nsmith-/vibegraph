//! The status pane drawn below the log, as a pure function of [`UiState`].
//!
//! Two columns: what is being run on the left — model, process, the stage in
//! progress and its bar — and what has been measured on the right. Nothing here
//! reads a clock or a channel, so a given state always draws the same cells and
//! a test can assert on them.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Widget};
use vibegraph::progress::stage;

use crate::si::fmt_si;
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

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
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
                dim(),
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
                dim(),
            ));
        }
        Line::from(spans)
    }

    fn stage_line(&self) -> Line<'static> {
        let Some(stage_name) = &self.state.stage else {
            return Line::from(Span::styled("starting", dim()));
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
                dim(),
            ));
        }
        Line::from(spans)
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
    fn keys_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled("level: ", dim()),
            Span::raw(self.state.level.label()),
            Span::styled(" \u{25b2}\u{25bc}  ", dim()),
            Span::styled("scope: ", dim()),
            Span::raw(self.state.scope.label()),
            Span::styled(" \u{25c2}\u{25b8}", dim()),
        ])
    }

    /// What a stop key will do next: ask for one, or take one that was already
    /// asked for and is being waited on.
    fn abort_line(&self) -> Line<'static> {
        if self.state.stopping {
            Line::from(Span::styled(
                "^C again  quit now",
                Style::default().fg(Color::Yellow),
            ))
        } else {
            Line::from(Span::styled("q / ^C  abort", dim()))
        }
    }

    fn sigma_line(&self) -> Line<'static> {
        match self.state.sigma_pb {
            None => Line::from(Span::styled("\u{3c3} = \u{2014}", dim())),
            Some(sigma_pb) => Line::from(Span::styled(
                format!("\u{3c3} = {}", cross_section(sigma_pb, self.state.err_pb)),
                Style::default().add_modifier(Modifier::BOLD),
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
            Style::default().add_modifier(Modifier::BOLD),
        ))
    }

    fn efficiency_line(&self) -> Line<'static> {
        let Some(efficiency) = self.state.efficiency.filter(|e| *e > 0.0) else {
            return Line::default();
        };
        Line::from(Span::styled(
            format!("efficiency  {:.1}%", 100.0 * efficiency),
            dim(),
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
            dim(),
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
                None => self.gauge().render(
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
            dim(),
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
            level: LogLevel::Info,
            scope: Scope::All,
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
        assert!(rows[5].contains('\u{25b2}') && rows[5].contains('\u{25bc}'), "{rows:?}");
        assert!(rows[5].contains('\u{25c2}') && rows[5].contains('\u{25b8}'), "{rows:?}");
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
