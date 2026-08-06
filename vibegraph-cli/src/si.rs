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

/// Decimal SI prefixes `fmt_si` selects between, smallest exponent first.
const PREFIXES: &[(i32, &str)] = &[
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
    let min_exp = PREFIXES[0].0;
    let max_exp = PREFIXES[PREFIXES.len() - 1].0;
    let si_exp = (3 * exp10.div_euclid(3)).clamp(min_exp, max_exp);
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

#[cfg(test)]
mod tests {
    use super::fmt_si;

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
