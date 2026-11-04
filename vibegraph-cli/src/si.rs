//! Fixed-width SI-prefixed number formatting.
//!
//! `fmt_si` takes a value already expressed in its *base* unit — barns for a
//! cross section, seconds for a duration — and renders it with whichever
//! decimal SI prefix keeps the mantissa in `[1, 1000)`, at a constant number
//! of significant figures. A vibegraph cross section (fb…nb) or an eval
//! timing (ns…ms) lands on the expected prefix purely from its magnitude;
//! nothing here special-cases either quantity.
//!
//! Holding the significant-figure count fixed, rather than the decimal-place
//! count, is what keeps the printed width from jittering as a live value
//! drifts across a decade: `1.00`, `12.3` and `123` are all three digits of
//! precision, just with the point in a different place.
//!
//! `fmt_si_measured` is the other rule, for a final measurement: the prefix is
//! chosen the same way, but the digits are set by the error, to its second
//! significant figure, and the value is printed to the same decimal place.

/// Decimal SI prefixes `fmt_si` selects between, smallest exponent first.
const PREFIXES: &[(i32, &str)] = &[
    (-21, "z"),
    (-18, "a"),
    (-15, "f"),
    (-12, "p"),
    (-9, "n"),
    (-6, "\u{b5}"),
    (-3, "m"),
    (0, ""),
    (3, "k"),
    (6, "M"),
    (9, "G"),
    (12, "T"),
];

/// One picobarn in barns: a cross section in pb times this is `fmt_si`'s base unit.
pub(crate) const PICOBARN: f64 = 1.0e-12;

/// One GeV in electronvolts: a width in GeV times this is `fmt_si`'s base unit.
pub(crate) const GEV: f64 = 1.0e9;

/// The prefix exponent a value of decimal exponent `exp10` is printed under: the
/// multiple of three at or below it, clamped to the table.
fn prefix_exponent(exp10: i32) -> i32 {
    let min_exp = PREFIXES[0].0;
    let max_exp = PREFIXES[PREFIXES.len() - 1].0;
    (3 * exp10.div_euclid(3)).clamp(min_exp, max_exp)
}

fn prefix_symbol(exp: i32) -> &'static str {
    PREFIXES
        .iter()
        .find(|(e, _)| *e == exp)
        .map(|(_, s)| *s)
        .unwrap_or("")
}

/// Round `abs_value` (must be finite and `>= 0`) to `sig_figs` significant
/// figures, returned as a mantissa in `[1, 10)` (or exactly `0` when the input
/// is `0`) with its base-10 exponent.
fn normalize(abs_value: f64, sig_figs: i32) -> (f64, i32) {
    if abs_value == 0.0 {
        return (0.0, 0);
    }
    let mut exp10 = abs_value.log10().floor() as i32;
    let mut mantissa = abs_value / 10f64.powi(exp10);
    let decimals = (sig_figs - 1).max(0);
    let scale = 10f64.powi(decimals);
    mantissa = (mantissa * scale).round() / scale;
    // Rounding a mantissa like 9.9995 up to 10.00 crosses into the next decade;
    // renormalize so the mantissa is always in [1, 10) going into the caller.
    if mantissa >= 10.0 {
        mantissa /= 10.0;
        exp10 += 1;
    } else if mantissa < 1.0 {
        // Guards float noise at a decade edge (e.g. log10(1000.0) landing a hair
        // under 3.0); the true mantissa is always >= 1 for a nonzero input.
        mantissa *= 10.0;
        exp10 -= 1;
    }
    (mantissa, exp10)
}

/// The SI exponent, mantissa and decimal-place count `value` renders at.
fn si_components(value: f64, sig_figs: usize) -> (f64, i32, usize) {
    let sig_figs = (sig_figs as i32).max(1);
    let sign = if value.is_sign_negative() && value != 0.0 {
        -1.0
    } else {
        1.0
    };
    let (mantissa, exp10) = normalize(value.abs(), sig_figs);
    let si_exp = prefix_exponent(exp10);
    let shift = exp10 - si_exp;
    let digits_before_decimal = shift + 1;
    let decimals = (sig_figs - digits_before_decimal).max(0) as usize;
    (sign * mantissa * 10f64.powi(shift), si_exp, decimals)
}

/// Render `value` — and, if given, its uncertainty `unc` — in `unit`, prefixed
/// by whichever SI prefix keeps `value`'s mantissa in `[1, 1000)` at
/// `sig_figs` significant figures. `unc` shares `value`'s prefix and decimal
/// width rather than choosing its own, so a fluctuating uncertainty cannot
/// change the line's width on its own; this also means `unc` can render with
/// a mantissa outside `[1, 1000)` when it is much larger than `value`, which
/// is the expected reading of "the result is smaller than its own error bar."
///
/// `value` and `unc` are the quantity's own base unit — barns for a cross
/// section, seconds for a duration — not a unit that already carries a
/// prefix; `unit` is the bare symbol (`"b"`, `"s"`) that prefix is written
/// against.
pub(crate) fn fmt_si(value: f64, unc: Option<f64>, unit: &str, sig_figs: usize) -> String {
    let (mantissa, si_exp, decimals) = si_components(value, sig_figs);
    let prefix = prefix_symbol(si_exp);
    match unc {
        None => format!("{mantissa:.decimals$} {prefix}{unit}"),
        Some(u) => {
            let u_mantissa = u.abs() / 10f64.powi(si_exp);
            format!("{mantissa:.decimals$} \u{b1} {u_mantissa:.decimals$} {prefix}{unit}")
        }
    }
}

/// The significant decimal digits that survive a round trip through an `f64`
/// (C's `DBL_DIG`). A measurement's value is printed to no more than these, and
/// to exactly these when its error bounds no digit: a digit past them can be
/// the noise of the unit conversion rather than the value's.
const F64_DIGITS: i32 = 15;

/// The decimal exponent of `text`, a number formatted with `{:e}`.
fn exponent(text: &str) -> i32 {
    text.rsplit_once('e')
        .and_then(|(_, e)| e.parse().ok())
        .expect("a number formatted with {:e} carries its exponent")
}

/// Render a measurement `value ± error` in `unit`, both under the one SI prefix
/// that puts `value`'s mantissa in `[1, 1000)`: `8.6812 ± 0.0023 ab`.
///
/// The error is rounded to two significant figures, and the value is printed to
/// the same decimal place, so the two carry the same number of decimals and the
/// line holds the digits the error supports and no more. A value whose rounding
/// at that place reaches 1000 moves to the next prefix (`999.996 ± 0.12` pb is
/// `1.00000 ± 0.00012 nb`). Where the error's second figure lies above the
/// units (an error larger than the value can put it there), both print with no
/// decimals: the value to its units digit, so it never rounds to zero under the
/// prefix it chose, and the error to its two figures (`34 ± 440 ab`). The value
/// is never given more than the 15 significant digits an `f64` holds exactly.
///
/// An error that bounds no digit — zero, as a constant integrand's can be, or
/// not finite — leaves the value at those 15 digits, its full precision, with
/// the error printed at the same decimals (`0.125` pb ± 0 is
/// `125.000000000000 ± 0.000000000000 fb`; a NaN error prints `NaN`). A zero
/// value takes its prefix from the error (`0.0 ± 2.3 zb`); with no error to
/// take it from, or a value that is not finite, nothing chooses a prefix and
/// both print as they are, in the base unit (`0 ± 0 b`).
///
/// `value` and `error` are in the quantity's base unit, as for [`fmt_si`].
pub(crate) fn fmt_si_measured(value: f64, error: f64, unit: &str) -> String {
    let bounded = error > 0.0 && error.is_finite();
    if !value.is_finite() || (value == 0.0 && !bounded) {
        return format!("{value} \u{b1} {error} {unit}");
    }
    // A negative zero prints its sign; nothing here measured one.
    let value = if value == 0.0 { 0.0 } else { value };
    // The error as printed, and the decimal exponent of the value's last digit.
    let (shown_error, last_place) = if bounded {
        let text = format!("{error:.1e}");
        (
            text.parse::<f64>().expect("a formatted f64 parses"),
            exponent(&text) - 1,
        )
    } else {
        // Rounded first, so a value that carries into the next decade keeps
        // its digit count there.
        let text = format!("{value:.prec$e}", prec = (F64_DIGITS - 1) as usize);
        (error, exponent(&text) - (F64_DIGITS - 1))
    };
    let magnitude = if value == 0.0 {
        shown_error
    } else {
        value.abs()
    };
    let decade = exponent(&format!("{magnitude:e}"));
    let last_place = last_place.max(decade - (F64_DIGITS - 1));
    let max_exp = PREFIXES[PREFIXES.len() - 1].0;
    let mut si_exp = prefix_exponent(decade);
    loop {
        let decimals = (si_exp - last_place).max(0) as usize;
        let scale = 10f64.powi(si_exp);
        let shown = format!("{:.decimals$}", value / scale);
        let mantissa: f64 = shown.parse().expect("a formatted f64 parses");
        if mantissa.abs() >= 1000.0 && si_exp < max_exp {
            si_exp += 3;
            continue;
        }
        let prefix = prefix_symbol(si_exp);
        return format!(
            "{shown} \u{b1} {:.decimals$} {prefix}{unit}",
            shown_error / scale
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{fmt_si, fmt_si_measured, PICOBARN};

    /// `value ± error` in picobarns, rendered in barns.
    fn pb(value: f64, error: f64) -> String {
        fmt_si_measured(value * PICOBARN, error * PICOBARN, "b")
    }

    /// The error keeps two significant figures and the value its decimal place,
    /// under the one prefix the value's magnitude picks.
    #[test]
    fn a_measurement_is_printed_to_its_errors_second_figure() {
        for (value, error, printed) in [
            (8.681_234_567e-6, 2.345e-9, "8.6812 \u{b1} 0.0023 ab"),
            (1.234_567, 0.000_123, "1.23457 \u{b1} 0.00012 pb"),
            (934.4237, 0.8712, "934.42 \u{b1} 0.87 pb"),
            (1.952_37e-3, 1.96e-4, "1.95 \u{b1} 0.20 fb"),
            (5.7e4, 3.1e2, "57.00 \u{b1} 0.31 nb"),
            (-1.234_56e-2, 1.1e-4, "-12.35 \u{b1} 0.11 fb"),
            // The error's rounding carries it into the next decade.
            (4.0e-7, 9.96e-9, "400 \u{b1} 10 zb"),
            // The value's rounding carries it into the next decade, not prefix.
            (9.999_96, 0.0012, "10.0000 \u{b1} 0.0012 pb"),
        ] {
            assert_eq!(pb(value, error), printed, "{value} ± {error} pb");
        }
    }

    /// The prefix is the value's, at both edges of `[1, 1000)`, and a value
    /// that rounds up to 1000 at its error's place takes the next one.
    #[test]
    fn the_values_magnitude_picks_the_prefix_at_both_edges() {
        for (value, error, printed) in [
            (1.0004, 0.0012, "1.0004 \u{b1} 0.0012 pb"),
            (0.9994, 0.0012, "999.4 \u{b1} 1.2 fb"),
            (999.4, 1.2, "999.4 \u{b1} 1.2 pb"),
            (1000.4, 1.2, "1.0004 \u{b1} 0.0012 nb"),
            (999.996, 0.12, "1.00000 \u{b1} 0.00012 nb"),
            (0.999_96, 0.0012, "1.0000 \u{b1} 0.0012 pb"),
        ] {
            assert_eq!(pb(value, error), printed, "{value} ± {error} pb");
        }
    }

    /// An error above the value keeps its two figures; where they sit above the
    /// units, both numbers print as integers and the value keeps its units digit.
    #[test]
    fn an_error_larger_than_the_value_prints_as_integers() {
        assert_eq!(pb(3.0e-5, 4.4e-4), "30 \u{b1} 440 ab");
        assert_eq!(pb(3.4e-5, 4.37e-4), "34 \u{b1} 440 ab");
        assert_eq!(pb(3.4e-6, 4.37e-4), "3 \u{b1} 440 ab");
        assert_eq!(pb(3.0e-5, 4.4e-5), "30 \u{b1} 44 ab");
        assert_eq!(pb(3.0e-5, 4.4e-6), "30.0 \u{b1} 4.4 ab");
    }

    /// An error that bounds no digit leaves the value at its full 15 digits.
    #[test]
    fn an_error_that_bounds_nothing_leaves_the_full_precision() {
        assert_eq!(pb(0.125, 0.0), "125.000000000000 \u{b1} 0.000000000000 fb");
        assert_eq!(pb(0.125, f64::NAN), "125.000000000000 \u{b1} NaN fb");
        assert_eq!(pb(0.125, f64::INFINITY), "125.000000000000 \u{b1} inf fb");
        // Fifteen digits after the rounding has carried the value up a prefix.
        assert_eq!(
            fmt_si_measured(999.999_999_999_999_9, 0.0, "u"),
            "1.00000000000000 \u{b1} 0.00000000000000 ku"
        );
    }

    /// A zero value takes its prefix from the error; with none, and for a value
    /// that is not finite, nothing chooses a prefix.
    #[test]
    fn a_zero_value_takes_the_errors_prefix() {
        assert_eq!(pb(0.0, 2.3e-9), "0.0 \u{b1} 2.3 zb");
        assert_eq!(pb(-0.0, 2.3e-9), "0.0 \u{b1} 2.3 zb");
        assert_eq!(pb(0.0, 0.0), "0 \u{b1} 0 b");
        assert_eq!(fmt_si_measured(f64::NAN, 1.0, "b"), "NaN \u{b1} 1 b");
    }

    /// A value is never printed past the digits an `f64` holds, however small
    /// its error.
    #[test]
    fn a_value_keeps_no_more_digits_than_an_f64_holds() {
        assert_eq!(
            fmt_si_measured(1.25, 1.0e-30, "u"),
            "1.25000000000000 \u{b1} 0.00000000000000 u"
        );
    }

    #[test]
    fn a_typical_cross_section_lands_on_pb() {
        // 802.94 pb and its 3.11 pb error, both expressed in barns.
        assert_eq!(
            fmt_si(802.94e-12, Some(3.11e-12), "b", 3),
            "803 \u{b1} 3 pb"
        );
    }

    #[test]
    fn a_typical_eval_time_lands_on_ns() {
        assert_eq!(fmt_si(212.0e-9, None, "s", 3), "212 ns");
    }

    /// The stated boundary case: rounding 999.95 to 3 significant figures
    /// overflows into the next decade, so the prefix must bump rather than
    /// print a four-digit mantissa.
    #[test]
    fn rounding_at_the_top_of_a_bracket_bumps_the_prefix() {
        assert_eq!(fmt_si(999.95, None, "u", 3), "1.00 ku");
        // One tick below the boundary stays in the same bracket.
        assert_eq!(fmt_si(999.4, None, "u", 3), "999 u");
    }

    /// An uncertainty larger than the value it qualifies is not an error case:
    /// it renders in the value's own prefix and decimal width, even past 3
    /// digits of mantissa.
    #[test]
    fn an_uncertainty_larger_than_the_value_still_renders() {
        assert_eq!(fmt_si(1.2, Some(45.6), "u", 3), "1.20 \u{b1} 45.60 u");
    }

    /// The table reaches atto and zepto, where a small cross section lands.
    #[test]
    fn a_cross_section_below_a_femtobarn_lands_on_ab() {
        assert_eq!(fmt_si(8.68e-18, None, "b", 3), "8.68 ab");
        assert_eq!(fmt_si(8.68e-21, None, "b", 3), "8.68 zb");
    }

    #[test]
    fn zero_renders_without_a_prefix() {
        assert_eq!(fmt_si(0.0, None, "u", 3), "0.00 u");
        assert_eq!(fmt_si(0.0, Some(0.0), "u", 3), "0.00 \u{b1} 0.00 u");
    }

    #[test]
    fn negative_values_keep_their_sign_and_the_uncertainty_stays_positive() {
        assert_eq!(
            fmt_si(-802.94e-12, Some(3.11e-12), "b", 3),
            "-803 \u{b1} 3 pb"
        );
    }

    /// Every decade in a full three-step (femto → pico → nano) sweep renders
    /// at exactly `sig_figs` significant digits: the point moves, the digit
    /// budget does not, which is what keeps a live-updating line from
    /// changing width as the value it shows drifts.
    #[test]
    fn digit_count_is_stable_across_a_decade_sweep() {
        let sig_figs = 3;
        for exponent in -14..=14 {
            let value = 1.2345 * 10f64.powi(exponent);
            let rendered = fmt_si(value, None, "u", sig_figs);
            let mantissa = rendered.split(' ').next().unwrap();
            let digit_count = mantissa.chars().filter(|c| c.is_ascii_digit()).count();
            assert_eq!(
                digit_count, sig_figs,
                "value {value:e} rendered {rendered:?} with {digit_count} digits, want {sig_figs}"
            );
        }
    }

    /// Sweeping across one full 1000x decade (the period of the prefix table)
    /// visits every decimal-place bracket exactly once and returns to the
    /// first: the same shape a value passing through a prefix boundary during
    /// a live run repeats indefinitely.
    #[test]
    fn a_full_decade_sweep_cycles_through_every_bracket() {
        let renders: Vec<String> = [1.0, 10.0, 100.0, 1000.0]
            .into_iter()
            .map(|v| fmt_si(v, None, "u", 3))
            .collect();
        assert_eq!(renders, vec!["1.00 u", "10.0 u", "100 u", "1.00 ku"]);
    }
}
